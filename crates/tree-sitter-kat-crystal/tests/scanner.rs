use tree_sitter_kat_crystal::LANGUAGE;

#[test]
fn shared_headers_support_literals_and_heredocs() {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    let mut source = String::new();
    for index in 0..128 {
        source.push_str(&format!("value_{index} = \"value #{{{index}}}\"\n"));
    }
    source.push_str("message = <<-TEXT\nhello #{value_0}\nTEXT\n");
    let tree = parser.parse(&source, None).unwrap();
    assert!(
        !tree.root_node().has_error(),
        "{}",
        tree.root_node().to_sexp()
    );
    assert_eq!(tree.root_node().end_byte(), source.len());
}
