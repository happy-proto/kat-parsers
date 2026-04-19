pub mod build_support {
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
