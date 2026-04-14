use std::path::Path;

fn main() {
    let crystal_dir = Path::new("vendor/crystal");

    let mut build = cc::Build::new();
    build.std("c11").include(crystal_dir);

    #[cfg(target_env = "msvc")]
    build.flag("-utf-8");

    let parser = crystal_dir.join("parser.c");
    println!("cargo:rerun-if-changed={}", parser.display());
    build.file(parser);

    let scanner = crystal_dir.join("scanner.c");
    println!("cargo:rerun-if-changed={}", scanner.display());
    build.file(scanner);

    for file in ["unicode.c"] {
        let path = crystal_dir.join(file);
        println!("cargo:rerun-if-changed={}", path.display());
    }

    for file in [
        "tree_sitter/alloc.h",
        "tree_sitter/array.h",
        "tree_sitter/parser.h",
    ] {
        let path = crystal_dir.join(file);
        println!("cargo:rerun-if-changed={}", path.display());
    }

    build.compile("tree-sitter-kat-parsers");
}
