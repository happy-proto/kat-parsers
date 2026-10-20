use std::{env, fs, path::PathBuf};

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let parser = root.join("../tree-sitter-kat-coffeescript/vendor/coffeescript/parser.c");
    println!("cargo:rerun-if-changed={}", parser.display());
    let mut source = fs::read_to_string(&parser).unwrap();
    let packed_re =
        regex::Regex::new(r"(?s)static const unsigned char ts_compact_zstd\[\] = \{(.*?)\n\};")
            .unwrap();
    let bytes = regex::Regex::new(r"0x([a-f0-9]+)").unwrap();
    let captures = packed_re
        .captures(&source)
        .expect("CoffeeScript must exercise compression");
    let packed: Vec<u8> = bytes
        .captures_iter(&captures[1])
        .map(|c| u8::from_str_radix(&c[1], 16).unwrap())
        .collect();
    let layout =
        regex::Regex::new(r"decode\(ts_compact_zstd, sizeof\(ts_compact_zstd\), (\d+), (\d+)\)")
            .unwrap();
    let caps = layout.captures(&source).unwrap();
    let length: usize = caps[1].parse().unwrap();
    let offset: usize = caps[2].parse().unwrap();
    let raw = zstd::bulk::decompress(&packed, length).unwrap();
    let mut arrays = String::from("static const uint16_t ts_small_parse_table[] = {\n");
    for (index, pair) in raw[..offset].as_chunks::<2>().0.iter().enumerate() {
        if index % 16 == 0 {
            arrays.push('\n');
        }
        arrays.push_str(&format!("{},", u16::from_le_bytes(*pair)));
    }
    arrays.push_str("\n};\nstatic const uint32_t ts_small_parse_table_map[] = {\n");
    for (index, word) in raw[offset..].as_chunks::<4>().0.iter().enumerate() {
        if index % 16 == 0 {
            arrays.push('\n');
        }
        arrays.push_str(&format!("{},", u32::from_le_bytes(*word)));
    }
    arrays.push_str("\n};");
    source = packed_re
        .replace(&source, regex::NoExpand(&arrays))
        .into_owned();
    source = source.replace("tree_sitter_coffeescript_compressed(const unsigned char *(*decode)(const unsigned char *, size_t, size_t, size_t))", "tree_sitter_coffeescript_baseline(void)");
    source = source
        .replace(
            ".small_parse_table = NULL,",
            ".small_parse_table = ts_small_parse_table,",
        )
        .replace(
            ".small_parse_table_map = NULL,",
            ".small_parse_table_map = ts_small_parse_table_map,",
        );
    let start = source
        .find("  const unsigned char *data = decode(")
        .unwrap();
    let end = source[start..].find("  return &language;").unwrap() + start;
    source.replace_range(start..end, "");
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("baseline.c");
    fs::write(&output, source).unwrap();
    cc::Build::new()
        .std("c11")
        .include(root.join("../kat-parser-common/include"))
        .file(output)
        .compile("coffeescript_baseline");
}
