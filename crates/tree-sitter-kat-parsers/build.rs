use std::path::Path;

fn main() {
    let languages = [
        LanguageBuildSpec {
            vendor_dir: "vendor/coffeescript",
            has_scanner: true,
            has_unicode: false,
        },
        LanguageBuildSpec {
            vendor_dir: "vendor/crystal",
            has_scanner: true,
            has_unicode: true,
        },
    ];

    let mut build = cc::Build::new();
    build.std("c11");

    #[cfg(target_env = "msvc")]
    build.flag("-utf-8");

    for language in languages {
        let vendor_dir = Path::new(language.vendor_dir);
        build.include(vendor_dir);

        let parser = vendor_dir.join("parser.c");
        println!("cargo:rerun-if-changed={}", parser.display());
        build.file(parser);

        if language.has_scanner {
            let scanner = vendor_dir.join("scanner.c");
            println!("cargo:rerun-if-changed={}", scanner.display());
            build.file(scanner);
        }

        if language.has_unicode {
            let unicode = vendor_dir.join("unicode.c");
            println!("cargo:rerun-if-changed={}", unicode.display());
            build.file(unicode);
        }

        for file in [
            "tree_sitter/alloc.h",
            "tree_sitter/array.h",
            "tree_sitter/parser.h",
        ] {
            let path = vendor_dir.join(file);
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    build.compile("tree-sitter-kat-parsers");
}

struct LanguageBuildSpec {
    vendor_dir: &'static str,
    has_scanner: bool,
    has_unicode: bool,
}
