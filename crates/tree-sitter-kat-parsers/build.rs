use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("missing manifest dir"));
    let vendor_root = crate_dir.join("vendor");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("missing OUT_DIR"));
    let specs = discover_language_specs(&vendor_root);

    let mut c_build = cc::Build::new();
    c_build.std("c11");

    let mut cpp_build = cc::Build::new();
    cpp_build.cpp(true);

    #[cfg(target_env = "msvc")]
    {
        c_build.flag("/utf-8");
        cpp_build.flag("/utf-8");
    }

    let mut has_cpp_sources = false;

    for spec in &specs {
        emit_watch_directives(&spec.vendor_dir);
        c_build.include(&spec.vendor_dir);
        cpp_build.include(&spec.vendor_dir);

        c_build.file(spec.vendor_dir.join("parser.c"));
        for path in root_c_support_files(&spec.vendor_dir) {
            c_build.file(path);
        }
        let cpp_files = root_cpp_support_files(&spec.vendor_dir);
        if !cpp_files.is_empty() {
            has_cpp_sources = true;
            for path in cpp_files {
                cpp_build.file(path);
            }
        }
    }

    c_build.compile("tree-sitter-kat-parsers");
    if has_cpp_sources {
        cpp_build.compile("tree-sitter-kat-parsers-cpp");
    }

    write_generated_bindings(&out_dir.join("generated_languages.rs"), &specs);
}

#[derive(Debug)]
struct LanguageBuildSpec {
    name: String,
    vendor_dir: PathBuf,
    symbol_name: String,
}

fn discover_language_specs(vendor_root: &Path) -> Vec<LanguageBuildSpec> {
    let mut dirs = fs::read_dir(vendor_root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", vendor_root.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && path.join("parser.c").is_file())
        .collect::<Vec<_>>();
    dirs.sort();

    dirs.into_iter()
        .map(|vendor_dir| {
            let name = vendor_dir
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_else(|| panic!("invalid vendor dir name: {}", vendor_dir.display()))
                .to_owned();
            let symbol_name = extract_symbol_name(&vendor_dir.join("parser.c"));
            LanguageBuildSpec {
                name,
                vendor_dir,
                symbol_name,
            }
        })
        .collect()
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

fn extract_symbol_name(parser_path: &Path) -> String {
    let contents = fs::read_to_string(parser_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", parser_path.display()));
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
                return candidate.to_owned();
            }
        }
    }
    panic!(
        "failed to extract tree-sitter language symbol from {}",
        parser_path.display()
    );
}

fn write_generated_bindings(path: &Path, specs: &[LanguageBuildSpec]) {
    let mut output =
        String::from("use tree_sitter_language::LanguageFn;\n\nunsafe extern \"C\" {\n");
    for spec in specs {
        output.push_str(&format!("    fn {}() -> *const ();\n", spec.symbol_name));
    }
    output.push_str("}\n\n");

    for spec in specs {
        let const_prefix = spec.name.to_ascii_uppercase();
        output.push_str(&format!(
            "pub const {const_prefix}_LANGUAGE: LanguageFn = unsafe {{ LanguageFn::from_raw({symbol}) }};\n",
            symbol = spec.symbol_name
        ));
        output.push_str(&format!(
            "pub const {const_prefix}_GRAMMAR: &str = include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/vendor/{name}/grammar.json\"));\n",
            name = spec.name
        ));
        output.push_str(&format!(
            "pub const {const_prefix}_NODE_TYPES: &str = include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/vendor/{name}/node-types.json\"));\n\n",
            name = spec.name
        ));
    }

    output.push_str("#[cfg(test)]\nmod tests {\n    use super::*;\n\n");
    for spec in specs {
        let const_prefix = spec.name.to_ascii_uppercase();
        output.push_str(&format!(
            "    #[test]\n    fn can_load_{name}_language() {{\n        let mut parser = tree_sitter::Parser::new();\n        parser\n            .set_language(&{const_prefix}_LANGUAGE.into())\n            .expect(\"failed to load {name} parser\");\n        assert!(!{const_prefix}_GRAMMAR.is_empty());\n        assert!(!{const_prefix}_NODE_TYPES.is_empty());\n    }}\n\n",
            name = spec.name
        ));
    }
    output.push_str("}\n");

    fs::write(path, output).unwrap_or_else(|error| {
        panic!(
            "failed to write generated bindings {}: {error}",
            path.display()
        )
    });
}
