mod headers;
mod packing;

use std::ffi::OsStr;
use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use anstyle::{AnsiColor, Style};
use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use tempfile::TempDir;

mod upstreams;

const PARSER_STAMP_FILE: &str = ".parser-inputs";
const TREE_SITTER_ABI_VERSION: &str = "15";
const TREE_SITTER_JS_RUNTIME: &str = "node";
const GENERATED_FILES: [&str; 3] = ["grammar.json", "node-types.json", "parser.c"];
const SHARED_HEADERS: [&str; 3] = ["parser.h", "alloc.h", "array.h"];
const AGGREGATE_CRATE_NAME: &str = "tree-sitter-kat-parsers";
const COMMON_CRATE_NAME: &str = "kat-parser-common";

#[derive(Debug, Deserialize)]
struct GrammarMetadata {
    language: String,
    #[serde(default)]
    grammar_file: Option<String>,
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

struct SharedHeaders {
    directory: PathBuf,
    lock: Mutex<()>,
}

impl SharedHeaders {
    fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            lock: Mutex::new(()),
        }
    }

    fn output_hash(&self, vendor_dir: &Path) -> Result<Option<String>> {
        let _guard = self.lock.lock().expect("shared header lock poisoned");
        generated_output_hash(vendor_dir, &self.directory)
    }

    fn sync_from(&self, source: &Path) -> Result<()> {
        let _guard = self.lock.lock().expect("shared header lock poisoned");
        for header in SHARED_HEADERS {
            copy_file(&source.join(header), &self.directory.join(header))?;
        }
        Ok(())
    }
}

fn default_generation_jobs() -> NonZeroUsize {
    thread::available_parallelism()
        .unwrap_or(NonZeroUsize::MIN)
        .min(NonZeroUsize::new(4).unwrap())
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
    /// 批量同步上游 grammar、scanner 和许可证，并固定实际 Git revision。
    Sync {
        #[arg(value_name = "LANGUAGE")]
        language: Option<String>,
        /// 只检查更新，不修改工作区。
        #[arg(long)]
        check: bool,
        /// 重放已锁定的 revision，不追踪远端新提交。
        #[arg(long)]
        locked: bool,
    },
    /// Regenerate vendored parser artifacts.
    Generate {
        /// Only regenerate the named grammar; defaults to all registered grammars when omitted.
        #[arg(value_name = "LANGUAGE")]
        language: Option<String>,
        /// Regenerate all parser artifacts even when their inputs and outputs are unchanged.
        #[arg(long)]
        force: bool,
        /// Maximum number of grammars generated concurrently (at most four by default).
        #[arg(short, long, default_value_t = default_generation_jobs())]
        jobs: NonZeroUsize,
    },
}

fn main() -> Result<()> {
    let repo_root = repo_root()?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Sync {
            language,
            check,
            locked,
        } => upstreams::sync(&repo_root, language.as_deref(), check, locked),
        Commands::Generate {
            language,
            force,
            jobs,
        } => generate(&repo_root, language.as_deref(), force, jobs),
    }
}

fn generate(
    repo_root: &Path,
    language: Option<&str>,
    force: bool,
    jobs: NonZeroUsize,
) -> Result<()> {
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
    if !cli_version
        .split_whitespace()
        .nth(1)
        .is_some_and(|version| version.starts_with("0.27."))
    {
        bail!(
            "Tree-sitter CLI 0.27.x is required; install the version pinned in Generated Check (found {cli_version})"
        );
    }
    println!(
        "{} Generating {} grammar crate(s) with {}{}",
        paint("==>", accent_style()),
        selected.len(),
        paint(&cli_version, success_style()),
        if force {
            " (forced artifact refresh)"
        } else {
            ""
        }
    );

    let mut rust_sources = Vec::new();
    write_common_crate(repo_root, &mut rust_sources)?;
    cleanup_obsolete_generated_crates(repo_root, &specs)?;

    for spec in &specs {
        write_language_crate_scaffolding(spec, &mut rust_sources)?;
    }

    let headers =
        SharedHeaders::new(repo_root.join("crates/kat-parser-common/include/tree_sitter"));
    let next = AtomicUsize::new(0);
    thread::scope(|scope| -> Result<()> {
        let workers = (0..jobs.get().min(selected.len()))
            .map(|_| {
                scope.spawn(|| -> Result<()> {
                    while let Some(spec) = selected.get(next.fetch_add(1, Ordering::Relaxed)) {
                        if let Err(error) = generate_one(spec, &cli_version, force, &headers)
                            .with_context(|| {
                                format!("failed to generate {}", spec.metadata.language)
                            })
                        {
                            // Let in-flight grammars finish, but stop scheduling new work.
                            next.store(selected.len(), Ordering::Relaxed);
                            return Err(error);
                        }
                    }
                    Ok(())
                })
            })
            .collect::<Vec<_>>();
        for worker in workers {
            worker.join().expect("grammar generation worker panicked")?;
        }
        Ok(())
    })?;

    for spec in &specs {
        if spec.vendor_dir.join("parser.c").is_file() {
            write_language_crate_lib(spec, &mut rust_sources)?;
        }
    }

    write_aggregate_crate(repo_root, &specs, &mut rust_sources)?;
    format_generated_rust_files(&rust_sources)?;
    Ok(())
}

fn generate_one(
    spec: &GrammarSpec,
    cli_version: &str,
    force: bool,
    headers: &SharedHeaders,
) -> Result<()> {
    let upstream_dir = &spec.upstream_dir;
    let vendor_dir = &spec.vendor_dir;
    let stamp_file = vendor_dir.join(PARSER_STAMP_FILE);

    // Scanner/support changes do not affect parser generation, but must always
    // reach the vendored crate even when the grammar cache is current.
    for file in vendored_support_files(spec) {
        copy_file(
            &upstream_dir.join(file),
            &vendor_dir.join(vendored_support_destination(file)),
        )?;
    }

    let input_hash = compute_parser_input_hash(spec, cli_version)?;
    if !force
        && let Some(output_hash) = headers.output_hash(vendor_dir)?
        && stamp_matches(&stamp_file, &format!("{input_hash}\n{output_hash}"))
    {
        print_skip(spec, "Parser artifacts are up to date");
        return Ok(());
    }

    print_section(spec, "Generating parser artifacts");
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
    // Since 0.27, ABI 15 requires no synthetic tree-sitter.json. A single
    // invocation produces all three artifacts.
    let grammar_file = spec
        .metadata
        .grammar_file
        .as_deref()
        .unwrap_or("grammar.js");
    if !generator_input_files(spec).contains(&grammar_file) {
        bail!("grammar entrypoint {grammar_file} must be declared in copied.files");
    }
    run_tree_sitter_generate_parser(&workspace_dir, grammar_file)?;
    if grammar_file.ends_with(".json") {
        // CLI 读取展开 JSON 时只生成 C 和 node-types，保留输入作为公开 GRAMMAR。
        copy_file(
            &workspace_dir.join(grammar_file),
            &workspace_dir.join("src/grammar.json"),
        )?;
    }
    headers::normalize_array_header(&workspace_dir.join("src/tree_sitter/array.h"))?;
    // Use the generator's public scanner headers, not runtime-private headers
    // or historical copies bundled with individual upstream grammars.
    headers.sync_from(&workspace_dir.join("src/tree_sitter"))?;
    packing::pack(
        &workspace_dir.join("src/parser.c"),
        &workspace_dir.join("src"),
    )?;
    for file in GENERATED_FILES {
        copy_file(
            &workspace_dir.join("src").join(file),
            &vendor_dir.join(file),
        )?;
    }
    let output_hash = headers
        .output_hash(vendor_dir)?
        .context("missing generated artifacts")?;
    write_file(&stamp_file, &format!("{input_hash}\n{output_hash}\n"))?;

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

fn write_common_crate(repo_root: &Path, rust_sources: &mut Vec<(PathBuf, String)>) -> Result<()> {
    let crate_dir = repo_root.join("crates").join(COMMON_CRATE_NAME);
    write_file(&crate_dir.join("Cargo.toml"), &common_cargo_toml())?;
    rust_sources.push((crate_dir.join("src/lib.rs"), COMMON_LIB_RS.to_owned()));
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

fn write_language_crate_scaffolding(
    spec: &GrammarSpec,
    rust_sources: &mut Vec<(PathBuf, String)>,
) -> Result<()> {
    write_file(
        &spec.crate_dir.join("Cargo.toml"),
        &language_crate_cargo_toml(spec),
    )?;
    rust_sources.push((
        spec.crate_dir.join("build.rs"),
        LANGUAGE_BUILD_RS.to_owned(),
    ));
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
tree-sitter = "0.27.1"

[dependencies]
tree-sitter-language = "0.1.7"
kat-parser-runtime = {{ path = "../kat-parser-runtime" }}

[build-dependencies]
kat-parser-common = {{ path = "../kat-parser-common" }}
"#,
        package_name = spec.package_name,
        language = spec.metadata.language
    )
}

fn write_language_crate_lib(
    spec: &GrammarSpec,
    rust_sources: &mut Vec<(PathBuf, String)>,
) -> Result<()> {
    let parser_path = spec.vendor_dir.join("parser.c");
    let symbol_name = extract_symbol_name(&parser_path)?;
    rust_sources.push((
        spec.crate_dir.join("src/lib.rs"),
        language_crate_lib_rs(spec, &symbol_name),
    ));
    Ok(())
}

fn language_crate_lib_rs(spec: &GrammarSpec, symbol_name: &str) -> String {
    let binding = if symbol_name.ends_with("_compressed") {
        format!(
            r#"unsafe extern "C" {{
    fn {symbol_name}(decode: unsafe extern "C" fn(*const u8, usize, usize, usize) -> *const u8) -> *const ();
}}
unsafe extern "C" fn language() -> *const () {{
    static LANGUAGE_POINTER: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *LANGUAGE_POINTER.get_or_init(|| unsafe {{ {symbol_name}(kat_parser_runtime::decode) as usize }}) as *const ()
}}
pub const LANGUAGE: LanguageFn = unsafe {{ LanguageFn::from_raw(language) }};
"#
        )
    } else {
        format!(
            r#"unsafe extern "C" {{
    fn {symbol_name}() -> *const ();
}}
pub const LANGUAGE: LanguageFn = unsafe {{ LanguageFn::from_raw({symbol_name}) }};
"#
        )
    };
    format!(
        r#"use tree_sitter_language::LanguageFn;

{binding}
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
        binding = binding
    )
}

fn write_aggregate_crate(
    repo_root: &Path,
    specs: &[GrammarSpec],
    rust_sources: &mut Vec<(PathBuf, String)>,
) -> Result<()> {
    let crate_dir = repo_root.join("crates").join(AGGREGATE_CRATE_NAME);
    write_file(&crate_dir.join("Cargo.toml"), &aggregate_cargo_toml(specs))?;
    rust_sources.push((crate_dir.join("src/lib.rs"), aggregate_lib_rs(specs)));
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
    output.push_str("#[cfg(test)]\nmod tests {\n    #[test]\n    fn all_language_getters_link_and_initialize() {\n        for (name, getter) in [\n");
    for spec in specs {
        output.push_str(&format!(
            "            (\"{}\", super::{}_LANGUAGE.into_raw()),\n",
            spec.metadata.language, spec.constant_prefix
        ));
    }
    output.push_str("        ] {\n            assert!(!unsafe { getter() }.is_null(), \"{name}\");\n        }\n    }\n}\n");
    output.trim_end().to_owned() + "\n"
}

fn repo_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .context("xtask must live under the repository root")
}

fn format_generated_rust_files(sources: &[(PathBuf, String)]) -> Result<()> {
    // Format away from the checkout. Comparing an unformatted template with a
    // formatted file would otherwise rewrite it on every run and invalidate Cargo.
    let temp_dir = TempDir::new().context("failed to create Rust formatting workspace")?;
    let mut paths = Vec::new();
    for (index, (_, contents)) in sources.iter().enumerate() {
        let path = temp_dir.path().join(format!("{index}.rs"));
        write_file(&path, contents)?;
        paths.push(path);
    }

    let status = Command::new("rustfmt")
        .args(paths.iter().map(|path| path.as_os_str()))
        .status()
        .context("failed to execute rustfmt on generated Rust sources")?;
    if !status.success() {
        bail!("rustfmt exited with {status}");
    }
    for ((destination, _), source) in sources.iter().zip(paths) {
        copy_file(&source, destination)?;
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
    matches!(extension, "js" | "json" | "mjs" | "cjs")
}

fn is_license_file(path: &str) -> bool {
    Path::new(path)
        .file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.starts_with("LICENSE"))
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

fn compute_parser_input_hash(spec: &GrammarSpec, cli_version: &str) -> Result<String> {
    let mut hasher = blake3::Hasher::new();
    hash_bytes(
        &mut hasher,
        b"grammar-file",
        spec.metadata
            .grammar_file
            .as_deref()
            .unwrap_or("grammar.js")
            .as_bytes(),
    );
    hash_bytes(
        &mut hasher,
        b"zstd-version",
        zstd::zstd_safe::version_string().as_bytes(),
    );
    hash_bytes(&mut hasher, b"packer-version", packing::VERSION.as_bytes());
    hash_bytes(&mut hasher, b"packer-source", include_bytes!("packing.rs"));
    hash_bytes(
        &mut hasher,
        b"scanner-headers",
        include_bytes!("headers.rs"),
    );
    hash_bytes(
        &mut hasher,
        b"packer-exporter",
        packing::EXPORTER.as_bytes(),
    );
    hash_bytes(&mut hasher, b"tree-sitter-cli", cli_version.as_bytes());
    hash_bytes(
        &mut hasher,
        b"js-runtime",
        TREE_SITTER_JS_RUNTIME.as_bytes(),
    );
    hash_bytes(
        &mut hasher,
        b"tree-sitter-abi-version",
        TREE_SITTER_ABI_VERSION.as_bytes(),
    );
    let mut inputs = generator_input_files(spec);
    inputs.extend(
        spec.metadata
            .generator_support_files
            .iter()
            .map(String::as_str),
    );
    inputs.sort_unstable();
    inputs.dedup();
    for input in inputs {
        let path = spec.upstream_dir.join(input);
        let bytes =
            fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
        hash_bytes(&mut hasher, input.as_bytes(), &bytes);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

// Validate outputs as well as inputs: missing or edited artifacts must never
// become invisible to check-generated merely because the grammar is unchanged.
fn generated_output_hash(vendor_dir: &Path, shared_include_dir: &Path) -> Result<Option<String>> {
    let mut hasher = blake3::Hasher::new();
    let outputs = GENERATED_FILES
        .into_iter()
        .map(|file| vendor_dir.join(file))
        .chain(
            SHARED_HEADERS
                .into_iter()
                .map(|file| shared_include_dir.join(file)),
        );
    for path in outputs {
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(error).with_context(|| format!("failed to read {}", path.display()));
            }
        };
        hash_bytes(
            &mut hasher,
            path.file_name().unwrap().as_encoded_bytes(),
            &bytes,
        );
    }
    Ok(Some(hasher.finalize().to_hex().to_string()))
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

fn run_tree_sitter_generate_parser(temp_root: &Path, grammar_file: &str) -> Result<()> {
    // Keep the generator out of xtask's dependency graph: a prebuilt, optimized
    // CLI avoids choosing between compiling the generator quickly and running
    // it quickly. Node handles both CommonJS and the CSS ES module imported by
    // Less/SCSS; QuickJS's require loader cannot load that module.
    let status = Command::new("tree-sitter")
        .arg("generate")
        .arg(grammar_file)
        .arg("--abi")
        .arg(TREE_SITTER_ABI_VERSION)
        .arg("--js-runtime")
        .arg(TREE_SITTER_JS_RUNTIME)
        .current_dir(temp_root)
        .status()
        .context("failed to execute tree-sitter generate --abi 15")?;
    if !status.success() {
        bail!("tree-sitter generate --abi 15 exited with {status}");
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
    let bytes = fs::read(source).with_context(|| format!("failed to read {}", source.display()))?;
    write_bytes(destination, &bytes)
}

fn write_bytes(destination: &Path, contents: &[u8]) -> Result<()> {
    if fs::read(destination).is_ok_and(|existing| existing == contents) {
        return Ok(());
    }
    let parent = destination
        .parent()
        .with_context(|| format!("{} has no parent directory", destination.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    fs::write(destination, contents)
        .with_context(|| format!("failed to write {}", destination.display()))?;
    Ok(())
}

fn write_file(path: &Path, contents: &str) -> Result<()> {
    write_bytes(path, contents.as_bytes())
}

const LANGUAGE_BUILD_RS: &str = r#"fn main() {
    kat_parser_common::build_support::compile_single_vendor_crate();
}
"#;

const COMMON_LIB_RS: &str = r##"pub mod build_support {
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
        emit_watch_directives(&shared_include_dir);

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
                } else if path.extension().and_then(|value| value.to_str())
                    .is_some_and(|extension| matches!(extension, "c" | "cc" | "cpp" | "h" | "hpp")) {
                    println!("cargo:rerun-if-changed={}", path.display());
                }
            }
        }
    }

    fn root_c_support_files(vendor_dir: &Path) -> Vec<PathBuf> {
        let mut files = root_support_files(vendor_dir, &["c"]);
        files.retain(|path| path.file_name().and_then(|value| value.to_str()) != Some("parser.c"));
        let included_sources = files
            .iter()
            .flat_map(|path| {
                fs::read_to_string(path)
                    .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                    .lines()
                    .filter_map(|line| line.trim().strip_prefix("#include \"")?.strip_suffix('"'))
                    .filter(|include| include.ends_with(".c"))
                    .map(|include| vendor_dir.join(include))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        files.retain(|path| !included_sources.contains(path));
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
"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_generated_files_keep_their_modification_time() -> Result<()> {
        let dir = TempDir::new()?;
        let source = dir.path().join("source");
        let destination = dir.path().join("destination");
        write_file(&source, "unchanged")?;
        copy_file(&source, &destination)?;
        let before = fs::metadata(&destination)?.modified()?;
        std::thread::sleep(std::time::Duration::from_millis(20));
        copy_file(&source, &destination)?;
        assert_eq!(before, fs::metadata(&destination)?.modified()?);
        write_file(&destination, "unchanged")?;
        assert_eq!(before, fs::metadata(&destination)?.modified()?);
        write_file(&source, "changed")?;
        copy_file(&source, &destination)?;
        assert_eq!(fs::read_to_string(&destination)?, "changed");
        Ok(())
    }

    fn fixture(root: &Path, language: &str) -> Result<GrammarSpec> {
        let upstream_dir = root.join("upstreams").join(language);
        write_file(
            &upstream_dir.join("grammar.js"),
            &format!(
                "module.exports = grammar({{name: '{language}', rules: {{source: $ => require('../shared/token.js')}}}});"
            ),
        )?;
        write_file(
            &root.join("upstreams/shared/token.js"),
            "module.exports = 'a';",
        )?;
        write_file(&upstream_dir.join("scanner.c"), "/* initial scanner */")?;
        let crate_dir = root.join(format!("crates/tree-sitter-kat-{language}"));
        Ok(GrammarSpec {
            metadata: GrammarMetadata {
                language: language.to_owned(),
                grammar_file: None,
                generator_support_files: vec!["../shared/token.js".to_owned()],
                copied: CopiedFiles {
                    files: vec!["grammar.js".to_owned(), "scanner.c".to_owned()],
                },
            },
            upstream_dir,
            vendor_dir: crate_dir.join("vendor").join(language),
            crate_dir,
            package_name: format!("tree-sitter-kat-{language}"),
            crate_ident: format!("tree_sitter_kat_{language}"),
            constant_prefix: language.to_uppercase(),
        })
    }

    #[test]
    #[ignore = "requires Tree-sitter CLI 0.27.1 and Node; run in Generated Check"]
    fn generation_tracks_imports_and_repairs_missing_or_modified_outputs() -> Result<()> {
        let dir = TempDir::new()?;
        let spec = fixture(dir.path(), "fixture")?;
        let cli_version = tree_sitter_version()?;
        let shared_include_dir = dir
            .path()
            .join("crates/kat-parser-common/include/tree_sitter");
        let headers = SharedHeaders::new(shared_include_dir.clone());
        let run = |force| generate_one(&spec, &cli_version, force, &headers);
        run(false)?;
        let original_grammar = fs::read(spec.vendor_dir.join("grammar.json"))?;
        let parser = fs::read_to_string(spec.vendor_dir.join("parser.c"))?;
        assert!(parser.contains("#define LANGUAGE_VERSION 15"));
        assert!(parser.contains(".patch_version = 0"));

        // A shared imported file is an input even though it lives outside the
        // grammar directory. Scanner bytes are synchronized independently.
        write_file(
            &dir.path().join("upstreams/shared/token.js"),
            "module.exports = 'b';",
        )?;
        run(false)?;
        assert_ne!(
            original_grammar,
            fs::read(spec.vendor_dir.join("grammar.json"))?
        );
        let stamp_path = spec.vendor_dir.join(PARSER_STAMP_FILE);
        let stamp_time = fs::metadata(&stamp_path)?.modified()?;
        write_file(
            &spec.upstream_dir.join("scanner.c"),
            "/* changed scanner */",
        )?;
        run(false)?;
        assert_eq!(stamp_time, fs::metadata(&stamp_path)?.modified()?);
        assert_eq!(
            fs::read_to_string(spec.vendor_dir.join("scanner.c"))?,
            "/* changed scanner */"
        );

        for file in GENERATED_FILES {
            let path = spec.vendor_dir.join(file);
            let expected = fs::read(&path)?;
            fs::remove_file(&path)?;
            run(false)?;
            assert_eq!(expected, fs::read(&path)?);
            write_file(&path, "corrupt artifact")?;
            run(false)?;
            assert_eq!(expected, fs::read(&path)?);
        }
        for header in SHARED_HEADERS {
            let path = shared_include_dir.join(header);
            let expected = fs::read(&path)?;
            fs::remove_file(&path)?;
            run(false)?;
            assert_eq!(expected, fs::read(&path)?);
            write_file(&path, "corrupt header")?;
            run(false)?;
            assert_eq!(expected, fs::read(&path)?);
        }

        // Exercise the generated public array API under optimization in both
        // languages used by external scanners, including growth and deletion.
        let array_source = dir.path().join("array.c");
        write_file(&array_source, include_str!("../tests/fixtures/array.c"))?;
        for (compiler, language, standard) in [("cc", "c", "c11"), ("c++", "c++", "c++11")] {
            let binary = dir.path().join(format!("array-{language}"));
            let status = Command::new(compiler)
                .args([
                    "-x",
                    language,
                    &format!("-std={standard}"),
                    "-O2",
                    "-fstrict-aliasing",
                    "-Werror",
                    "-I",
                ])
                .arg(shared_include_dir.parent().unwrap())
                .arg(&array_source)
                .arg("-o")
                .arg(&binary)
                .status()?;
            assert!(
                status.success(),
                "{compiler} failed to compile public scanner headers"
            );
            assert!(Command::new(binary).status()?.success());
        }

        write_file(
            &dir.path().join("metadata/fixture.toml"),
            "language = 'fixture'\ngenerator_support_files = ['../shared/token.js']\n[copied]\nfiles = ['grammar.js', 'scanner.c']\n",
        )?;
        generate(dir.path(), None, false, NonZeroUsize::MIN)?;
        let paths = [
            dir.path().join("crates/kat-parser-common/src/lib.rs"),
            dir.path().join("crates/tree-sitter-kat-parsers/src/lib.rs"),
            spec.crate_dir.join("src/lib.rs"),
            spec.crate_dir.join("build.rs"),
            spec.vendor_dir.join("parser.c"),
            stamp_path,
        ];
        let before = paths
            .iter()
            .map(|path| fs::metadata(path)?.modified())
            .collect::<std::io::Result<Vec<_>>>()?;
        // Even a forced, byte-identical regeneration must preserve timestamps,
        // including Rust sources whose templates need formatting.
        generate(dir.path(), None, true, NonZeroUsize::MIN)?;
        let after = paths
            .iter()
            .map(|path| fs::metadata(path)?.modified())
            .collect::<std::io::Result<Vec<_>>>()?;
        assert_eq!(before, after);
        Ok(())
    }

    #[test]
    fn generation_requires_nonzero_concurrency() {
        assert!(Cli::try_parse_from(["xtask", "generate", "--jobs", "0"]).is_err());
    }

    #[test]
    #[ignore = "requires Tree-sitter CLI 0.27.1 and Node; run in Generated Check"]
    fn parallel_generation_preserves_artifacts_and_propagates_errors() -> Result<()> {
        fn snapshot(
            root: &Path,
        ) -> Result<std::collections::BTreeMap<PathBuf, (Vec<u8>, std::time::SystemTime)>> {
            let mut files = std::collections::BTreeMap::new();
            for entry in fs::read_dir(root)? {
                let path = entry?.path();
                if path.is_dir() {
                    files.extend(snapshot(&path)?);
                } else {
                    files.insert(
                        path.clone(),
                        (fs::read(&path)?, fs::metadata(&path)?.modified()?),
                    );
                }
            }
            Ok(files)
        }

        let dir = TempDir::new()?;
        for language in ["alpha", "beta"] {
            fixture(dir.path(), language)?;
            write_file(
                &dir.path().join(format!("metadata/{language}.toml")),
                &format!(
                    "language = '{language}'\ngenerator_support_files = ['../shared/token.js']\n[copied]\nfiles = ['grammar.js', 'scanner.c']\n"
                ),
            )?;
        }
        let parallel = NonZeroUsize::new(4).unwrap();
        generate(dir.path(), None, true, NonZeroUsize::MIN)?;
        let crates = dir.path().join("crates");
        let serial = snapshot(&crates)?;
        generate(dir.path(), None, true, parallel)?;
        assert_eq!(
            serial,
            snapshot(&crates)?,
            "parallel generation changed bytes or modification times"
        );

        let headers = crates.join("kat-parser-common/include/tree_sitter");
        fs::remove_file(headers.join("parser.h"))?;
        write_file(&headers.join("array.h"), "corrupt shared header")?;
        generate(dir.path(), None, true, parallel)?;
        let repaired = snapshot(&crates)?;
        assert_eq!(serial.len(), repaired.len());
        for (path, (expected, _)) in &serial {
            assert_eq!(expected, &repaired[path].0, "{}", path.display());
        }
        // Repaired headers and cache stamps must also work on the cached path.
        generate(dir.path(), None, false, parallel)?;
        assert_eq!(repaired, snapshot(&crates)?);

        fs::remove_file(dir.path().join("upstreams/beta/grammar.js"))?;
        let error = generate(dir.path(), None, true, parallel).unwrap_err();
        assert!(error.to_string().contains("beta"), "{error:#}");
        Ok(())
    }

    #[test]
    fn cli_version_invalidates_the_generation_cache() -> Result<()> {
        let dir = TempDir::new()?;
        let mut spec = fixture(dir.path(), "fixture")?;
        assert_ne!(
            compute_parser_input_hash(&spec, "tree-sitter 0.27.0")?,
            compute_parser_input_hash(&spec, "tree-sitter 0.27.1")?,
        );
        let before = compute_parser_input_hash(&spec, "tree-sitter 0.27.1")?;
        spec.metadata.grammar_file = Some("grammar.json".to_owned());
        assert_ne!(
            before,
            compute_parser_input_hash(&spec, "tree-sitter 0.27.1")?
        );
        Ok(())
    }

    #[test]
    #[ignore = "requires Tree-sitter CLI 0.27.1 and Node; run in Generated Check"]
    fn expanded_json_is_a_self_contained_generation_input() -> Result<()> {
        let dir = TempDir::new()?;
        let mut spec = fixture(dir.path(), "fixture")?;
        let version = tree_sitter_version()?;
        let include = dir
            .path()
            .join("crates/kat-parser-common/include/tree_sitter");
        generate_one(&spec, &version, true, &SharedHeaders::new(include.clone()))?;
        let json = fs::read(spec.vendor_dir.join("grammar.json"))?;
        write_bytes(&spec.upstream_dir.join("grammar.json"), &json)?;
        spec.metadata.grammar_file = Some("grammar.json".to_owned());
        spec.metadata.generator_support_files.clear();
        spec.metadata.copied.files = vec!["grammar.json".to_owned(), "scanner.c".to_owned()];
        generate_one(&spec, &version, true, &SharedHeaders::new(include.clone()))?;
        assert_eq!(fs::read(spec.vendor_dir.join("grammar.json"))?, json);
        assert!(
            fs::read_to_string(spec.vendor_dir.join("parser.c"))?
                .contains("#define LANGUAGE_VERSION 15")
        );
        Ok(())
    }
}
