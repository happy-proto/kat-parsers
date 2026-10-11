use ghostty_conformance::{assert_line_structure, title_cases};
use std::{
    env, fs,
    path::Path,
    process::{Command, Output},
};

fn run(binary: &std::ffi::OsStr, args: &[&std::ffi::OsStr]) -> Output {
    Command::new(binary)
        .args(args)
        .output()
        .expect("execute GHOSTTY_BIN")
}
fn output_text(output: &Output) -> String {
    format!(
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn validate(binary: &std::ffi::OsStr, config_path: &Path) -> Output {
    // Ghostty accepts --key=value, not --key value.
    let mut argument = std::ffi::OsString::from("--config-file=");
    argument.push(config_path);
    run(binary, &["+validate-config".as_ref(), &argument])
}

#[test]
#[ignore = "requires the real Ghostty CLI; run just ghostty-check"]
fn official_parser_accepts_the_same_fixtures_and_generated_cases() {
    let binary = env::var_os("GHOSTTY_BIN").expect("GHOSTTY_BIN must name a Ghostty CLI binary");
    let version = run(&binary, &["+version".as_ref()]);
    assert!(version.status.success(), "{}", output_text(&version));
    println!("Ghostty oracle: {}", output_text(&version));
    let root = tempfile::tempdir().unwrap();
    let config_path = root.path().join("config.ghostty");
    let mut cases = title_cases();
    cases.push("# before\0after\ntitle=ok\n".into());
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/accepted");
    for entry in fs::read_dir(fixtures).unwrap() {
        cases.push(fs::read_to_string(entry.unwrap().path()).unwrap());
    }
    for source in &cases {
        assert_line_structure(source);
        fs::write(&config_path, source).unwrap();
        let output = validate(&binary, &config_path);
        assert!(
            output.status.success(),
            "Ghostty rejected {source:?}\n{}",
            output_text(&output)
        );
    }
    println!("{} accepted configs passed both parsers", cases.len());
    // A broken/no-op oracle must not pass: exercise official diagnostics too.
    for (source, diagnostic_key) in [
        ("font-size = not-a-number", "font-size"),
        (
            "unknown-ghostty-conformance-key = value",
            "unknown-ghostty-conformance-key",
        ),
        ("title\r = hello\n", "title\r"),
        ("title\0\n", "title\0"),
        ("title=one\n\u{feff}title=two\n", "\u{feff}title"),
        ("\u{feff}\u{feff}title=hello\n", "\u{feff}title"),
    ] {
        assert_line_structure(source); // preserve invalid schema keys faithfully
        fs::write(&config_path, source).unwrap();
        let output = validate(&binary, &config_path);
        assert_eq!(output.status.code(), Some(1), "{}", output_text(&output));
        assert!(!output.stdout.is_empty(), "expected an official diagnostic");
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(diagnostic_key),
            "{}",
            output_text(&output)
        );
    }
}

#[test]
#[ignore = "requires the real Ghostty CLI; run just ghostty-check"]
fn parses_all_current_upstream_default_keys_and_documentation() {
    let binary = env::var_os("GHOSTTY_BIN").expect("GHOSTTY_BIN must name a Ghostty CLI binary");
    let output = run(
        &binary,
        &[
            "+show-config".as_ref(),
            "--default".as_ref(),
            "--docs".as_ref(),
        ],
    );
    assert!(output.status.success(), "{}", output_text(&output));
    let source = String::from_utf8(output.stdout).unwrap();
    assert!(
        !source.is_empty(),
        "Ghostty returned an empty default config"
    );
    let tree = assert_line_structure(&source);
    let root = tree.root_node();
    let mut cursor = root.walk();
    let settings = root
        .named_children(&mut cursor)
        .filter(|node| node.kind() == "setting")
        .count();
    assert!(
        settings > 100,
        "expected upstream defaults, got {settings} settings"
    );
    println!("parsed {settings} upstream default settings plus documentation");
}
