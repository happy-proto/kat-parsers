use tree_sitter_kat_dotenv::LANGUAGE;

#[test]
fn shared_headers_support_repeated_assignment_boundaries() {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    let source = (0..128)
        .map(|index| format!("KEY_{index}=\"value ${{OTHER}}\"\n"))
        .collect::<String>();
    let tree = parser.parse(&source, None).unwrap();
    assert!(
        !tree.root_node().has_error(),
        "{}",
        tree.root_node().to_sexp()
    );
    assert_eq!(tree.root_node().named_child_count(), 128);
    assert_eq!(tree.root_node().end_byte(), source.len());
}
