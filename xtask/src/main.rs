use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use tempfile::TempDir;

const PARSER_STAGE_VERSION: &str = "parser-stage-v1";
const PARSER_STAMP_FILE: &str = ".parser-inputs";

struct GrammarSpec {
    name: &'static str,
    upstream_dir: &'static str,
    vendor_dir: &'static str,
}

fn main() -> Result<()> {
    let repo_root = repo_root()?;
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        bail!("usage: cargo run -p xtask -- generate [language] [--force]");
    };

    match command.as_str() {
        "generate" => {
            let mut force = false;
            let mut language: Option<String> = None;
            for arg in args {
                if arg == "--force" {
                    force = true;
                } else if language.is_none() {
                    language = Some(arg);
                } else {
                    bail!("unexpected argument: {arg}");
                }
            }
            generate(&repo_root, language.as_deref(), force)
        }
        other => bail!("unknown xtask command: {other}"),
    }
}

fn generate(repo_root: &Path, language: Option<&str>, force: bool) -> Result<()> {
    let specs = grammar_specs();
    let selected: Vec<&GrammarSpec> = match language {
        Some(name) => specs.iter().filter(|spec| spec.name == name).collect(),
        None => specs.iter().collect(),
    };

    if selected.is_empty() {
        bail!("unknown grammar: {}", language.unwrap_or("<none>"));
    }

    let cli_version = tree_sitter_version()?;
    for spec in selected {
        generate_one(repo_root, spec, &cli_version, force)?;
    }
    Ok(())
}

fn generate_one(
    repo_root: &Path,
    spec: &GrammarSpec,
    cli_version: &str,
    force: bool,
) -> Result<()> {
    let upstream_dir = repo_root.join(spec.upstream_dir);
    let vendor_dir = repo_root.join(spec.vendor_dir);
    let stamp_file = vendor_dir.join(PARSER_STAMP_FILE);

    let temp_dir = TempDir::new().context("failed to create temp dir for tree-sitter generate")?;
    let temp_root = temp_dir.path();
    fs::create_dir_all(temp_root.join("src"))?;

    copy_file(
        &upstream_dir.join("grammar.js"),
        &temp_root.join("grammar.js"),
    )?;
    copy_file(
        &upstream_dir.join("tree-sitter.json"),
        &temp_root.join("tree-sitter.json"),
    )?;

    // Stage 1: always regenerate the lightweight JSON outputs.
    run_tree_sitter_generate_no_parser(temp_root)?;
    copy_file(
        &temp_root.join("src/grammar.json"),
        &vendor_dir.join("grammar.json"),
    )?;
    copy_file(
        &temp_root.join("src/node-types.json"),
        &vendor_dir.join("node-types.json"),
    )?;

    // Stage 3: always sync vendored support files. This is effectively free.
    copy_file(
        &upstream_dir.join("src/scanner.c"),
        &vendor_dir.join("scanner.c"),
    )?;
    copy_file(
        &upstream_dir.join("src/unicode.c"),
        &vendor_dir.join("unicode.c"),
    )?;
    copy_file(
        &upstream_dir.join("src/tree_sitter/alloc.h"),
        &vendor_dir.join("tree_sitter/alloc.h"),
    )?;
    copy_file(
        &upstream_dir.join("src/tree_sitter/array.h"),
        &vendor_dir.join("tree_sitter/array.h"),
    )?;
    copy_file(
        &upstream_dir.join("src/tree_sitter/parser.h"),
        &vendor_dir.join("tree_sitter/parser.h"),
    )?;

    // Stage 2: only regenerate parser.c when grammar.json actually changed.
    let parser_input_hash = compute_parser_input_hash(temp_root, cli_version)?;
    if !force
        && stamp_matches(&stamp_file, &parser_input_hash)
        && vendor_dir.join("parser.c").is_file()
    {
        println!(
            "{} parser.c is up to date; skipping parser generation.",
            spec.name
        );
        return Ok(());
    }

    run_tree_sitter_generate_parser(temp_root)?;
    copy_file(
        &temp_root.join("src/parser.c"),
        &vendor_dir.join("parser.c"),
    )?;
    write_file(&stamp_file, &format!("{parser_input_hash}\n"))?;

    Ok(())
}

fn grammar_specs() -> Vec<GrammarSpec> {
    vec![GrammarSpec {
        name: "crystal",
        upstream_dir: "upstreams/crystal",
        vendor_dir: "crates/tree-sitter-kat-parsers/vendor/crystal",
    }]
}

fn repo_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .context("xtask must live under the repository root")
}

fn tree_sitter_version() -> Result<String> {
    let output = Command::new("tree-sitter")
        .arg("--version")
        .output()
        .context(
            "failed to execute tree-sitter --version; ensure the CLI is installed and in PATH",
        )?;
    if !output.status.success() {
        bail!("tree-sitter --version exited with {}", output.status);
    }
    String::from_utf8(output.stdout)
        .context("tree-sitter --version returned non-utf8 output")
        .map(|value| value.trim().to_owned())
}

fn compute_parser_input_hash(temp_root: &Path, cli_version: &str) -> Result<String> {
    let mut hasher = blake3::Hasher::new();
    hash_bytes(
        &mut hasher,
        b"parser-stage-version",
        PARSER_STAGE_VERSION.as_bytes(),
    );
    hash_bytes(&mut hasher, b"tree-sitter-cli", cli_version.as_bytes());
    let grammar_json_path = temp_root.join("src/grammar.json");
    let grammar_json = fs::read(&grammar_json_path)
        .with_context(|| format!("failed to read {}", grammar_json_path.display()))?;
    hash_bytes(&mut hasher, b"grammar.json", &grammar_json);

    Ok(hasher.finalize().to_hex().to_string())
}

fn hash_bytes(hasher: &mut blake3::Hasher, label: &[u8], bytes: &[u8]) {
    hasher.update(&(label.len() as u64).to_le_bytes());
    hasher.update(label);
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn stamp_matches(path: &Path, expected: &str) -> bool {
    fs::read_to_string(path)
        .map(|value| value.trim() == expected)
        .unwrap_or(false)
}

fn run_tree_sitter_generate_no_parser(temp_root: &Path) -> Result<()> {
    let status = Command::new("tree-sitter")
        .arg("generate")
        .arg("--no-parser")
        .current_dir(temp_root)
        .status()
        .context("failed to execute tree-sitter generate --no-parser")?;
    if !status.success() {
        return Err(anyhow!("tree-sitter generate exited with {status}"));
    }
    Ok(())
}

fn run_tree_sitter_generate_parser(temp_root: &Path) -> Result<()> {
    let status = Command::new("tree-sitter")
        .arg("generate")
        .arg("src/grammar.json")
        .current_dir(temp_root)
        .status()
        .context("failed to execute tree-sitter generate src/grammar.json")?;
    if !status.success() {
        return Err(anyhow!(
            "tree-sitter generate src/grammar.json exited with {status}"
        ));
    }
    Ok(())
}

fn copy_file(source: &Path, destination: &Path) -> Result<()> {
    let parent = destination
        .parent()
        .with_context(|| format!("{} has no parent directory", destination.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    fs::copy(source, destination).with_context(|| {
        format!(
            "failed to copy {} to {}",
            source.display(),
            destination.display()
        )
    })?;
    Ok(())
}

fn write_file(path: &Path, contents: &str) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("{} has no parent directory", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}
