use tree_sitter::Parser;
use tree_sitter_kat_caddyfile::LANGUAGE;

#[test]
fn parses_common_caddyfile_structures() {
    for source in [
        include_str!("fixtures/Caddyfile"),
        ":8080\nrespond \"Hello\" 200\n",
        "localhost {\r\n    file_server\r\n}\r\n",
        "example.com {\n    @local {\n        remote_ip 127.0.0.1\n    }\n    respond @local `hello`\n}\n",
        "&(shared) {\n    respond \"hello\"\n}\nexample.com {\n    invoke shared\n}\n",
    ] {
        let mut parser = Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();
        let tree = parser.parse(source, None).unwrap();
        assert!(
            !tree.root_node().has_error(),
            "{source:?}: {}",
            tree.root_node().to_sexp()
        );
    }
}
