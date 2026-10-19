use anyhow::{Context, Result, bail};
use regex::Regex;
use std::{fs, path::Path, process::Command};
use tempfile::TempDir;

pub const EXPORTER: &str = include_str!("compact_parser_export.c");
pub const VERSION: &str = "kat-tables-le-v1-zstd19-net-saving";

fn replace_array(source: &mut String, declaration: &str, replacement: &str) -> Result<()> {
    let start = source
        .find(declaration)
        .with_context(|| format!("missing {declaration}"))?;
    let end = source[start..].find("\n};").context("unterminated array")? + start + 3;
    source.replace_range(start..end, replacement);
    Ok(())
}

fn array(bytes: &[u8], width: usize, declaration: &str) -> String {
    let mut result = format!("{declaration} = {{\n");
    for chunk in bytes.chunks(width * 16) {
        result.push_str("  ");
        for value in chunk.chunks_exact(width) {
            let number = if width == 2 {
                u16::from_le_bytes([value[0], value[1]]) as u32
            } else {
                value[0] as u32
            };
            result.push_str(&format!("0x{number:x}, "));
        }
        result.pop();
        result.push('\n');
    }
    result.push_str("};");
    result
}

fn matrix(bytes: &[u8]) -> String {
    let mut result =
        String::from("static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {\n");
    for row in bytes.chunks_exact(bytes.len() / 2) {
        let row = array(row, 2, "");
        result.push_str(row.strip_prefix(" = ").unwrap().strip_suffix(';').unwrap());
        result.push_str(",\n");
    }
    result.push_str("};");
    result
}

/// 编译读取真实 C 数组的维护工具；不会运行在消费方 build.rs 中。
pub fn pack(parser: &Path, include: &Path) -> Result<bool> {
    let source = fs::read_to_string(parser)?;
    if !source.contains("static const uint16_t ts_small_parse_table[]")
        || !source.contains("#define LARGE_STATE_COUNT ")
    {
        return Ok(false);
    }
    let large = Regex::new(r"#define LARGE_STATE_COUNT (\d+)")?
        .captures(&source)
        .context("missing state count")?[1]
        .parse::<usize>()?;
    let states = Regex::new(r"#define STATE_COUNT (\d+)")?
        .captures(&source)
        .context("missing total state count")?[1]
        .parse::<usize>()?;
    if large < 2 || states <= 2 {
        return Ok(false);
    }
    let work = TempDir::new()?;
    let stub_re = Regex::new(r"void \*(tree_sitter_\w+_external_scanner)_create\(void\);")?;
    let mut helper = format!(
        "#define TREE_SITTER_HIDE_SYMBOLS\n#include \"{}\"\n",
        parser.display().to_string().replace('\\', "/")
    );
    for captures in stub_re.captures_iter(&source) {
        let prefix = &captures[1];
        helper.push_str(&format!("void *{prefix}_create(void) {{ return NULL; }}\nvoid {prefix}_destroy(void *p) {{ (void)p; }}\nbool {prefix}_scan(void *p, TSLexer *l, const bool *v) {{ (void)p; (void)l; (void)v; return false; }}\nunsigned {prefix}_serialize(void *p, char *b) {{ (void)p; (void)b; return 0; }}\nvoid {prefix}_deserialize(void *p, const char *b, unsigned n) {{ (void)p; (void)b; (void)n; }}\n"));
    }
    helper.push_str(EXPORTER);
    let helper_path = work.path().join("export.c");
    fs::write(&helper_path, helper)?;
    let executable = work.path().join(if cfg!(windows) {
        "export.exe"
    } else {
        "export"
    });
    let mut compiler = Command::new(if cfg!(windows) { "cl" } else { "cc" });
    compiler.current_dir(work.path());
    if cfg!(windows) {
        compiler
            .args(["/nologo", "/std:c11", "/Od"])
            .arg(format!("/I{}", include.display()))
            .arg(&helper_path)
            .arg(format!("/Fe:{}", executable.display()));
    } else {
        compiler
            .args(["-std=c11", "-O0", "-I"])
            .arg(include)
            .arg(&helper_path)
            .arg("-o")
            .arg(&executable);
    }
    let status = compiler
        .status()
        .context("failed to compile table exporter")?;
    if !status.success() {
        bail!("table exporter compiler exited with {status}");
    }
    let direct = work.path().join("direct.bin");
    let compact = work.path().join("compact.bin");
    let output = Command::new(executable)
        .args([&direct, &compact])
        .output()?;
    if !output.status.success() {
        bail!(
            "table exporter failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let values: Vec<usize> = String::from_utf8(output.stdout)?
        .split_whitespace()
        .map(str::parse)
        .collect::<std::result::Result<_, _>>()?;
    let [map_offset, native_bytes] = values.as_slice() else {
        bail!("invalid exporter output");
    };
    let raw = fs::read(compact)?;
    let packed = zstd::bulk::compress(&raw, 19)?;
    if packed.len() + fs::metadata(&direct)?.len() as usize >= *native_bytes {
        return Ok(false);
    }
    let mut source = source;
    source = Regex::new(r"#define LARGE_STATE_COUNT \d+")?
        .replace(&source, "#define LARGE_STATE_COUNT 2")
        .into_owned();
    replace_array(
        &mut source,
        "static const uint16_t ts_parse_table[",
        &matrix(&fs::read(direct)?),
    )?;
    replace_array(
        &mut source,
        "static const uint16_t ts_small_parse_table[] = {",
        &array(&packed, 1, "static const unsigned char ts_compact_zstd[]"),
    )?;
    replace_array(
        &mut source,
        "static const uint32_t ts_small_parse_table_map[] = {",
        "",
    )?;
    let getter = Regex::new(
        r"(?s)(?:TS_PUBLIC|extern) const TSLanguage \*(tree_sitter_\w+)\(void\) \{\s+static const TSLanguage language = \{(.*?)\n  \};\s+return &language;\s*\}",
    )?;
    let caps = getter
        .captures(&source)
        .context("unexpected language getter")?;
    let fields = caps[2]
        .replace(
            ".small_parse_table = ts_small_parse_table,",
            ".small_parse_table = NULL,",
        )
        .replace(
            ".small_parse_table_map = ts_small_parse_table_map,",
            ".small_parse_table_map = NULL,",
        );
    let replacement = format!(
        "/* KAT_COMPRESSED_TABLE_V1: LE uint16 data, aligned LE uint32 map. */\nTS_PUBLIC const TSLanguage *{}_compressed(const unsigned char *(*decode)(const unsigned char *, size_t, size_t, size_t)) {{\n  static TSLanguage language = {{{fields}\n  }};\n  const unsigned char *data = decode(ts_compact_zstd, sizeof(ts_compact_zstd), {}, {});\n  language.small_parse_table = (const uint16_t *)data;\n  language.small_parse_table_map = (const uint32_t *)(data + {});\n  return &language;\n}}",
        &caps[1],
        raw.len(),
        map_offset,
        map_offset
    );
    let rewritten = getter.replace(&source, regex::NoExpand(&replacement));
    fs::write(parser, rewritten.as_bytes())?;
    println!(
        "    tables: {native_bytes} -> {} bytes (compact {} bytes)",
        packed.len(),
        raw.len()
    );
    Ok(true)
}
