use tree_sitter_kat_nu::LANGUAGE;

#[test]
fn parses_structured_pipelines_and_environment_overlays() {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    for source in [
        "#!/usr/bin/env nu\nlet services = [{name: 'api', enabled: true}]\n$services | where enabled == true | each {|s| $\"服务：($s.name)\" }\n",
        "module warning {\n export-env {\n  const file = path self\n  error make -u {msg: $\"($file) requires overlay use\"}\n }\n}\nuse warning\nexport-env {\n let virtual_env = '/opt/example'\n let path_name = if ('Path' in $env) { 'Path' } else { 'PATH' }\n let new_path = ($env | get $path_name | prepend ($virtual_env | path join 'bin'))\n load-env { $path_name: $new_path VIRTUAL_ENV: $virtual_env }\n}\nexport alias deactivate = overlay hide activate\n",
        "def greet [name: string, --loud(-l)] {\n let text = $'hello ($name)'\n if $loud { $text | str upcase } else { $text }\n}\nlet raw = r#'a\\b'#\n",
    ] {
        let tree = parser.parse(source, None).unwrap();
        assert!(
            !tree.root_node().has_error(),
            "{source}\n{}",
            tree.root_node().to_sexp()
        );
    }
}

#[test]
fn recovers_after_incomplete_raw_strings() {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    for source in [
        "let value = r#'unfinished",
        "let value = r##'missing end'#\n",
    ] {
        let tree = parser.parse(source, None).unwrap();
        assert!(tree.root_node().has_error(), "{source}");
        let recovered = parser.parse("let value = r##'ready'##\n", None).unwrap();
        assert!(
            !recovered.root_node().has_error(),
            "{}",
            recovered.root_node().to_sexp()
        );
    }
}

#[test]
fn raw_strings_keep_hashes_and_partial_closing_delimiters_as_content() {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    for literal in ["r#'a#b'#", "r##'a#b '# and c'##", "r#''#"] {
        let source = format!("let value = {literal}\nprint $value\n");
        let tree = parser.parse(&source, None).unwrap();
        assert!(
            !tree.root_node().has_error(),
            "{source}\n{}",
            tree.root_node().to_sexp()
        );
    }
}
