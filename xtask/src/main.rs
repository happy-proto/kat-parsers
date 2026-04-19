use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anstyle::{AnsiColor, Style};
use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use tempfile::TempDir;

const PARSER_STAGE_VERSION: &str = "parser-stage-v4";
const PARSER_STAMP_FILE: &str = ".parser-inputs";
const TREE_SITTER_ABI_VERSION: &str = "15";
const AGGREGATE_CRATE_NAME: &str = "tree-sitter-kat-parsers";
const COMMON_CRATE_NAME: &str = "kat-parser-common";

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
    crate_dir: PathBuf,
    vendor_dir: PathBuf,
    package_name: String,
    crate_ident: String,
    constant_prefix: String,
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
        "{} Generating {} grammar crate(s) with {}{}",
        paint("==>", accent_style()),
        selected.len(),
        paint(&cli_version, success_style()),
        if force {
            " (forced parser refresh)"
        } else {
            ""
        }
    );

    write_common_crate(repo_root)?;
    cleanup_obsolete_generated_crates(repo_root, &specs)?;

    for spec in &specs {
        write_language_crate_scaffolding(spec)?;
    }

    for spec in selected {
        generate_one(spec, &cli_version, force)?;
        write_language_crate_lib(spec)?;
    }

    for spec in &specs {
        if spec.vendor_dir.join("parser.c").is_file() {
            write_language_crate_lib(spec)?;
        }
    }

    write_aggregate_crate(repo_root, &specs)?;
    format_generated_rust_files(repo_root, &specs)?;
    Ok(())
}

fn generate_one(spec: &GrammarSpec, cli_version: &str, force: bool) -> Result<()> {
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

    print_section(spec, "Stage 2/3: Syncing vendored support files");
    for file in vendored_support_files(spec) {
        copy_file(
            &upstream_dir.join(file),
            &vendor_dir.join(vendored_support_destination(file)),
        )?;
    }

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
        let package_name = language_package_name(&metadata.language);
        let crate_ident = package_name.replace('-', "_");
        let constant_prefix = language_const_prefix(&metadata.language);
        let crate_dir = repo_root.join("crates").join(&package_name);
        let vendor_dir = crate_dir.join("vendor").join(&metadata.language);
        let upstream_dir = repo_root.join("upstreams").join(&metadata.language);
        specs.push(GrammarSpec {
            metadata,
            upstream_dir,
            crate_dir,
            vendor_dir,
            package_name,
            crate_ident,
            constant_prefix,
        });
    }

    Ok(specs)
}

fn language_package_name(language: &str) -> String {
    format!("tree-sitter-kat-{}", language.replace('_', "-"))
}

fn language_const_prefix(language: &str) -> String {
    language.replace('-', "_").to_ascii_uppercase()
}

fn write_common_crate(repo_root: &Path) -> Result<()> {
    let crate_dir = repo_root.join("crates").join(COMMON_CRATE_NAME);
    write_file(&crate_dir.join("Cargo.toml"), &common_cargo_toml())?;
    write_file(&crate_dir.join("src/lib.rs"), COMMON_LIB_RS)?;
    Ok(())
}

fn common_cargo_toml() -> String {
    r#"[package]
name = "kat-parser-common"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true
description = "Shared build support for kat parser crates"
authors = ["DCjanus <DCjanus@dcjanus.com>"]

[dependencies]
cc = "1.2.60"
"#
    .to_owned()
}

fn cleanup_obsolete_generated_crates(repo_root: &Path, specs: &[GrammarSpec]) -> Result<()> {
    let crates_dir = repo_root.join("crates");
    let expected = specs
        .iter()
        .map(|spec| spec.package_name.as_str())
        .chain([AGGREGATE_CRATE_NAME, COMMON_CRATE_NAME])
        .collect::<std::collections::BTreeSet<_>>();

    let entries = fs::read_dir(&crates_dir)
        .with_context(|| format!("failed to read {}", crates_dir.display()))?;
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(OsStr::to_str) else {
            continue;
        };
        if !name.starts_with("tree-sitter-kat-") && name != COMMON_CRATE_NAME {
            continue;
        }
        if expected.contains(name) {
            continue;
        }
        fs::remove_dir_all(&path)
            .with_context(|| format!("failed to remove obsolete crate {}", path.display()))?;
    }
    Ok(())
}

fn write_language_crate_scaffolding(spec: &GrammarSpec) -> Result<()> {
    write_file(
        &spec.crate_dir.join("Cargo.toml"),
        &language_crate_cargo_toml(spec),
    )?;
    write_file(&spec.crate_dir.join("build.rs"), LANGUAGE_BUILD_RS)?;
    Ok(())
}

fn language_crate_cargo_toml(spec: &GrammarSpec) -> String {
    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true
description = "Pre-generated tree-sitter parser crate for {language} used by kat"
authors = ["DCjanus <DCjanus@dcjanus.com>"]
build = "build.rs"
include = ["build.rs", "src/lib.rs", "vendor/**"]

[lib]
path = "src/lib.rs"

[dev-dependencies]
tree-sitter = "0.26.8"

[dependencies]
tree-sitter-language = "0.1.7"

[build-dependencies]
kat-parser-common = {{ path = "../kat-parser-common" }}
"#,
        package_name = spec.package_name,
        language = spec.metadata.language
    )
}

fn write_language_crate_lib(spec: &GrammarSpec) -> Result<()> {
    let parser_path = spec.vendor_dir.join("parser.c");
    let symbol_name = extract_symbol_name(&parser_path)?;
    write_file(
        &spec.crate_dir.join("src/lib.rs"),
        &language_crate_lib_rs(spec, &symbol_name),
    )?;
    Ok(())
}

fn language_crate_lib_rs(spec: &GrammarSpec, symbol_name: &str) -> String {
    format!(
        r#"use tree_sitter_language::LanguageFn;

unsafe extern "C" {{
    fn {symbol_name}() -> *const ();
}}

pub const LANGUAGE: LanguageFn = unsafe {{ LanguageFn::from_raw({symbol_name}) }};
pub const GRAMMAR: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/vendor/{language}/grammar.json"
));
pub const NODE_TYPES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/vendor/{language}/node-types.json"
));

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn can_load_language() {{
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&LANGUAGE.into())
            .expect("failed to load {language} parser");
        assert!(!GRAMMAR.is_empty());
        assert!(!NODE_TYPES.is_empty());
    }}
}}
"#,
        language = spec.metadata.language,
        symbol_name = symbol_name
    )
}

fn write_aggregate_crate(repo_root: &Path, specs: &[GrammarSpec]) -> Result<()> {
    let crate_dir = repo_root.join("crates").join(AGGREGATE_CRATE_NAME);
    write_file(&crate_dir.join("Cargo.toml"), &aggregate_cargo_toml(specs))?;
    write_file(&crate_dir.join("src/lib.rs"), &aggregate_lib_rs(specs))?;
    Ok(())
}

fn aggregate_cargo_toml(specs: &[GrammarSpec]) -> String {
    let mut output = String::from(
        r#"[package]
name = "tree-sitter-kat-parsers"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true
description = "Aggregate crate for kat-specific pre-generated tree-sitter parser crates"
authors = ["DCjanus <DCjanus@dcjanus.com>"]
include = ["src/lib.rs"]

[lib]
path = "src/lib.rs"

[dependencies]
"#,
    );
    for spec in specs {
        output.push_str(&format!(
            r#"{package_name} = {{ path = "../{package_name}" }}
"#,
            package_name = spec.package_name
        ));
    }
    output
}

fn aggregate_lib_rs(specs: &[GrammarSpec]) -> String {
    let mut output = String::new();
    for spec in specs {
        output.push_str(&format!(
            "pub use {crate_ident}::{{ GRAMMAR as {const_prefix}_GRAMMAR, LANGUAGE as {const_prefix}_LANGUAGE, NODE_TYPES as {const_prefix}_NODE_TYPES }};\n\n",
            crate_ident = spec.crate_ident,
            const_prefix = spec.constant_prefix
        ));
    }
    output.trim_end().to_owned() + "\n"
}

fn repo_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .context("xtask must live under the repository root")
}

fn format_generated_rust_files(repo_root: &Path, specs: &[GrammarSpec]) -> Result<()> {
    let common_dir = repo_root.join("crates").join(COMMON_CRATE_NAME);
    let aggregate_dir = repo_root.join("crates").join(AGGREGATE_CRATE_NAME);
    let mut paths = vec![
        common_dir.join("src/lib.rs"),
        aggregate_dir.join("src/lib.rs"),
        repo_root.join("xtask/src/main.rs"),
    ];
    for spec in specs {
        paths.push(spec.crate_dir.join("build.rs"));
        paths.push(spec.crate_dir.join("src/lib.rs"));
    }

    let status = Command::new("rustfmt")
        .args(paths.iter().map(|path| path.as_os_str()))
        .status()
        .context("failed to execute rustfmt on generated Rust sources")?;
    if !status.success() {
        bail!("rustfmt exited with {status}");
    }
    Ok(())
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

fn extract_symbol_name(parser_path: &Path) -> Result<String> {
    let contents = fs::read_to_string(parser_path)
        .with_context(|| format!("failed to read {}", parser_path.display()))?;
    for line in contents.lines() {
        if !line.contains("TSLanguage *tree_sitter_") {
            continue;
        }
        if let Some(index) = line.find("tree_sitter_") {
            let candidate = line[index..]
                .split_once('(')
                .map(|(prefix, _)| prefix.trim())
                .unwrap_or_default();
            if !candidate.is_empty()
                && !candidate.contains("_external_")
                && candidate
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            {
                return Ok(candidate.to_owned());
            }
        }
    }
    bail!(
        "failed to extract tree-sitter language symbol from {}",
        parser_path.display()
    )
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

const LANGUAGE_BUILD_RS: &str = r#"fn main() {
    kat_parser_common::build_support::compile_single_vendor_crate();
}
"#;

const COMMON_LIB_RS: &str = r#"pub mod build_support {
    use std::{
        env, fs,
        path::{Path, PathBuf},
    };

    pub fn compile_single_vendor_crate() {
        let crate_dir =
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("missing manifest dir"));
        let vendor_dir = discover_vendor_dir(&crate_dir.join("vendor"));
        let shared_include_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("include");

        emit_watch_directives(&vendor_dir);

        let mut c_build = cc::Build::new();
        c_build.std("c11");

        let mut cpp_build = cc::Build::new();
        cpp_build.cpp(true);

        #[cfg(target_env = "msvc")]
        {
            c_build.flag("/utf-8");
            cpp_build.flag("/utf-8");
        }

        c_build.include(&vendor_dir);
        cpp_build.include(&vendor_dir);
        c_build.include(&shared_include_dir);
        cpp_build.include(&shared_include_dir);
        c_build.file(vendor_dir.join("parser.c"));

        for path in root_c_support_files(&vendor_dir) {
            c_build.file(path);
        }

        let cpp_files = root_cpp_support_files(&vendor_dir);
        let has_cpp_sources = !cpp_files.is_empty();
        for path in cpp_files {
            cpp_build.file(path);
        }

        let output_name = env::var("CARGO_PKG_NAME")
            .expect("missing package name")
            .replace('-', "_");
        c_build.compile(&output_name);
        if has_cpp_sources {
            cpp_build.compile(&format!("{output_name}_cpp"));
        }
    }

    fn discover_vendor_dir(vendor_root: &Path) -> PathBuf {
        let mut dirs = fs::read_dir(vendor_root)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", vendor_root.display()))
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_dir() && path.join("parser.c").is_file())
            .collect::<Vec<_>>();
        dirs.sort();
        match dirs.len() {
            1 => dirs.remove(0),
            0 => panic!("no vendored parser found under {}", vendor_root.display()),
            _ => panic!(
                "expected exactly one vendored parser under {}, found {}",
                vendor_root.display(),
                dirs.len()
            ),
        }
    }

    fn emit_watch_directives(vendor_dir: &Path) {
        let mut stack = vec![vendor_dir.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let mut entries = fs::read_dir(&dir)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .collect::<Vec<_>>();
            entries.sort();
            for path in entries {
                if path.is_dir() {
                    stack.push(path);
                } else {
                    println!("cargo:rerun-if-changed={}", path.display());
                }
            }
        }
    }

    fn root_c_support_files(vendor_dir: &Path) -> Vec<PathBuf> {
        let mut files = root_support_files(vendor_dir, &["c"]);
        files.retain(|path| path.file_name().and_then(|value| value.to_str()) != Some("parser.c"));
        files
    }

    fn root_cpp_support_files(vendor_dir: &Path) -> Vec<PathBuf> {
        root_support_files(vendor_dir, &["cc", "cpp"])
    }

    fn root_support_files(vendor_dir: &Path, extensions: &[&str]) -> Vec<PathBuf> {
        let mut files = fs::read_dir(vendor_dir)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", vendor_dir.display()))
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .filter(|path| {
                path.extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| extensions.iter().any(|extension| extension == &value))
            })
            .collect::<Vec<_>>();
        files.sort();
        files
    }
}
"#;
