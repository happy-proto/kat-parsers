use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anstyle::{AnsiColor, Style};
use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use tempfile::TempDir;

const PARSER_STAGE_VERSION: &str = "parser-stage-v3";
const PARSER_STAMP_FILE: &str = ".parser-inputs";
const TREE_SITTER_ABI_VERSION: &str = "15";

#[derive(Debug, Deserialize)]
struct GrammarMetadata {
    language: String,
    #[serde(default)]
    generator_support_files: Vec<String>,
    copied: CopiedFiles,
}

#[derive(Debug, Deserialize)]
struct CopiedFiles {
    files: Vec<String>,
}

struct GrammarSpec {
    metadata: GrammarMetadata,
    upstream_dir: PathBuf,
    vendor_dir: PathBuf,
}

#[derive(Parser)]
#[command(name = "xtask")]
/// Project maintenance tasks for kat-parsers.
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Regenerate vendored parser artifacts.
    Generate {
        /// Only regenerate the named grammar; defaults to all registered grammars when omitted.
        #[arg(value_name = "LANGUAGE")]
        language: Option<String>,
        /// Regenerate `parser.c` even when the cached parser inputs have not changed.
        #[arg(long)]
        force: bool,
    },
}

fn main() -> Result<()> {
    let repo_root = repo_root()?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { language, force } => generate(&repo_root, language.as_deref(), force),
    }
}

fn generate(repo_root: &Path, language: Option<&str>, force: bool) -> Result<()> {
    let specs = grammar_specs(repo_root)?;
    let selected: Vec<&GrammarSpec> = match language {
        Some(name) => specs
            .iter()
            .filter(|spec| spec.metadata.language == name)
            .collect(),
        None => specs.iter().collect(),
    };

    if selected.is_empty() {
        bail!("unknown grammar: {}", language.unwrap_or("<none>"));
    }

    let cli_version = tree_sitter_version()?;
    println!(
        "{} Generating {} grammar(s) with {}{}",
        paint("==>", accent_style()),
        selected.len(),
        paint(&cli_version, success_style()),
        if force {
            " (forced parser refresh)"
        } else {
            ""
        }
    );
    for spec in selected {
        generate_one(repo_root, spec, &cli_version, force)?;
    }
    Ok(())
}

fn generate_one(
    _repo_root: &Path,
    spec: &GrammarSpec,
    cli_version: &str,
    force: bool,
) -> Result<()> {
    let upstream_dir = &spec.upstream_dir;
    let vendor_dir = &spec.vendor_dir;
    let stamp_file = vendor_dir.join(PARSER_STAMP_FILE);

    print_section(spec, "Preparing temporary workspace");
    let temp_dir = TempDir::new().context("failed to create temp dir for tree-sitter generate")?;
    let temp_root = temp_dir.path();
    let workspace_dir = temp_root.join(&spec.metadata.language);
    fs::create_dir_all(workspace_dir.join("src"))?;

    for file in generator_input_files(spec) {
        copy_file(&upstream_dir.join(file), &workspace_dir.join(file))?;
    }

    for file in &spec.metadata.generator_support_files {
        copy_file(&upstream_dir.join(file), &workspace_dir.join(file))?;
    }
    write_file(
        &workspace_dir.join("tree-sitter.json"),
        &synthesized_tree_sitter_json(&spec.metadata.language),
    )?;

    // Stage 1: always regenerate the lightweight JSON outputs.
    print_section(
        spec,
        "Stage 1/3: Regenerating grammar.json and node-types.json",
    );
    run_tree_sitter_generate_no_parser(&workspace_dir)?;
    copy_file(
        &workspace_dir.join("src/grammar.json"),
        &vendor_dir.join("grammar.json"),
    )?;
    copy_file(
        &workspace_dir.join("src/node-types.json"),
        &vendor_dir.join("node-types.json"),
    )?;

    // Stage 2: always sync vendored support files. This is effectively free.
    print_section(spec, "Stage 2/3: Syncing vendored support files");
    for file in vendored_support_files(spec) {
        copy_file(
            &upstream_dir.join(file),
            &vendor_dir.join(vendored_support_destination(file)),
        )?;
    }

    // Stage 3: only regenerate parser.c when grammar.json actually changed.
    let parser_input_hash = compute_parser_input_hash(&workspace_dir, cli_version)?;
    if !force
        && stamp_matches(&stamp_file, &parser_input_hash)
        && vendor_dir.join("parser.c").is_file()
    {
        print_skip(
            spec,
            "Stage 3/3: parser.c is already up to date; skipping heavy regeneration",
        );
        return Ok(());
    }

    let parser_reason = if force {
        "forced refresh requested"
    } else {
        "parser inputs changed"
    };
    print_section(
        spec,
        &format!("Stage 3/3: Regenerating parser.c ({parser_reason})"),
    );
    run_tree_sitter_generate_parser(&workspace_dir)?;
    copy_file(
        &workspace_dir.join("src/parser.c"),
        &vendor_dir.join("parser.c"),
    )?;
    write_file(&stamp_file, &format!("{parser_input_hash}\n"))?;

    print_done(spec);
    Ok(())
}

fn grammar_specs(repo_root: &Path) -> Result<Vec<GrammarSpec>> {
    let metadata_dir = repo_root.join("metadata");
    let mut entries = fs::read_dir(&metadata_dir)
        .with_context(|| format!("failed to read {}", metadata_dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension() == Some(OsStr::new("toml")))
        .collect::<Vec<_>>();
    entries.sort();

    let mut specs = Vec::new();
    for path in entries {
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let metadata: GrammarMetadata = toml::from_str(&contents)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        let upstream_dir = repo_root.join("upstreams").join(&metadata.language);
        let vendor_dir = repo_root
            .join("crates/tree-sitter-kat-parsers/vendor")
            .join(&metadata.language);
        specs.push(GrammarSpec {
            metadata,
            upstream_dir,
            vendor_dir,
        });
    }

    Ok(specs)
}

fn repo_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .context("xtask must live under the repository root")
}

fn print_section(spec: &GrammarSpec, message: &str) {
    println!(
        "  {} {} {message}",
        paint("==>", accent_style()),
        paint(&format!("[{}]", spec.metadata.language), label_style())
    );
}

fn print_skip(spec: &GrammarSpec, message: &str) {
    println!(
        "  {} {} {message}",
        paint("[skip]", warning_style()),
        paint(&format!("[{}]", spec.metadata.language), label_style())
    );
}

fn print_done(spec: &GrammarSpec) {
    println!(
        "  {} {}",
        paint("[done]", success_style()),
        paint(&format!("[{}]", spec.metadata.language), label_style())
    );
}

fn generator_input_files(spec: &GrammarSpec) -> Vec<&str> {
    let mut files = spec
        .metadata
        .copied
        .files
        .iter()
        .filter(|file| file_extension(file).is_some_and(is_generator_extension))
        .map(String::as_str)
        .collect::<Vec<_>>();
    files.sort_unstable();
    files
}

fn vendored_support_files(spec: &GrammarSpec) -> Vec<&str> {
    let mut files = spec
        .metadata
        .copied
        .files
        .iter()
        .filter(|file| !is_license_file(file))
        .filter(|file| !file_extension(file).is_some_and(is_generator_extension))
        .map(String::as_str)
        .collect::<Vec<_>>();
    files.sort_unstable();
    files
}

fn vendored_support_destination(path: &str) -> &str {
    path.strip_prefix("src/").unwrap_or(path)
}

fn file_extension(path: &str) -> Option<&str> {
    Path::new(path).extension().and_then(OsStr::to_str)
}

fn is_generator_extension(extension: &str) -> bool {
    matches!(extension, "js" | "json")
}

fn is_license_file(path: &str) -> bool {
    Path::new(path)
        .file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.starts_with("LICENSE"))
}

fn synthesized_tree_sitter_json(language: &str) -> String {
    let template = r#"{
  "grammars": [
    {
      "name": "__LANGUAGE__",
      "path": "."
    }
  ],
  "metadata": {
    "version": "0.0.1"
  },
  "bindings": {
    "c": true,
    "rust": true
  }
}
"#;

    template.replace("__LANGUAGE__", language)
}

fn paint(text: &str, style: Style) -> String {
    format!("{}{}{}", style.render(), text, style.render_reset())
}

fn accent_style() -> Style {
    Style::new().fg_color(Some(AnsiColor::Cyan.into())).bold()
}

fn warning_style() -> Style {
    Style::new().fg_color(Some(AnsiColor::Yellow.into())).bold()
}

fn success_style() -> Style {
    Style::new().fg_color(Some(AnsiColor::Green.into())).bold()
}

fn label_style() -> Style {
    Style::new().bold()
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
    hash_bytes(
        &mut hasher,
        b"tree-sitter-abi-version",
        TREE_SITTER_ABI_VERSION.as_bytes(),
    );
    let grammar_json_path = temp_root.join("src/grammar.json");
    let grammar_json = fs::read(&grammar_json_path)
        .with_context(|| format!("failed to read {}", grammar_json_path.display()))?;
    hash_bytes(&mut hasher, b"grammar.json", &grammar_json);
    let tree_sitter_json_path = temp_root.join("tree-sitter.json");
    if tree_sitter_json_path.is_file() {
        let tree_sitter_json = fs::read(&tree_sitter_json_path)
            .with_context(|| format!("failed to read {}", tree_sitter_json_path.display()))?;
        hash_bytes(&mut hasher, b"tree-sitter.json", &tree_sitter_json);
    }

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
        .arg("--abi")
        .arg(TREE_SITTER_ABI_VERSION)
        .arg("--no-parser")
        .current_dir(temp_root)
        .status()
        .context("failed to execute tree-sitter generate --abi 15 --no-parser")?;
    if !status.success() {
        return Err(anyhow!("tree-sitter generate exited with {status}"));
    }
    Ok(())
}

fn run_tree_sitter_generate_parser(temp_root: &Path) -> Result<()> {
    let status = Command::new("tree-sitter")
        .arg("generate")
        .arg("--abi")
        .arg(TREE_SITTER_ABI_VERSION)
        .current_dir(temp_root)
        .status()
        .context("failed to execute tree-sitter generate --abi 15")?;
    if !status.success() {
        return Err(anyhow!(
            "tree-sitter generate --abi 15 exited with {status}"
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
