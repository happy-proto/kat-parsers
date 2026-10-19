use tree_sitter::{Node, Parser};
use tree_sitter_kat_jam::LANGUAGE;

fn tree(source: &str) -> tree_sitter::Tree {
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    parser.parse(source, None).unwrap()
}

fn find<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    if node.kind() == kind {
        return Some(node);
    }
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find_map(|child| find(child, kind))
}

#[test]
fn parses_original_b2_positive_test_sources() {
    for (name, source) in [
        (
            "core-language",
            include_str!("fixtures/b2/core-language.jam"),
        ),
        ("scanner", include_str!("fixtures/b2/scanner.jam")),
        ("typecheck", include_str!("fixtures/b2/typecheck.jam")),
        ("at-file", include_str!("fixtures/b2/at-file.jam")),
        ("dollars", include_str!("fixtures/b2/dollars.jam")),
        ("print", include_str!("fixtures/b2/print.jam")),
    ] {
        let parsed = tree(source);
        assert!(
            !parsed.root_node().has_error(),
            "{name}: {}",
            parsed.root_node().to_sexp()
        );
    }
}

#[test]
fn rejects_original_b2_syntax_error_test_sources() {
    for (name, source) in [
        (
            "unterminated-string",
            include_str!("fixtures/b2/unterminated-string.jam"),
        ),
        (
            "missing-semicolon",
            include_str!("fixtures/b2/missing-semicolon.jam"),
        ),
    ] {
        assert!(tree(source).root_node().has_error(), "accepted {name}");
    }
}

#[test]
fn preserves_upstream_switch_globs_and_quoted_colons() {
    for pattern in ["*[eaiou]*ue", r#""*:""#, r#""*:\\""#] {
        let source = format!("switch path {{ case {pattern} : echo matched ; }}");
        let parsed = tree(&source);
        assert!(
            !parsed.root_node().has_error(),
            "{}",
            parsed.root_node().to_sexp()
        );
        let case = find(parsed.root_node(), "case_statement").unwrap();
        let range = case.child_by_field_name("pattern").unwrap().byte_range();
        assert_eq!(&source[range], pattern);
    }
}

#[test]
fn preserves_b2_grist_word_in_comparison() {
    let source = "if <$(name)> = $(expected) { echo found ; }";
    let parsed = tree(source);
    assert!(
        !parsed.root_node().has_error(),
        "{}",
        parsed.root_node().to_sexp()
    );
    let condition = find(parsed.root_node(), "condition").unwrap();
    let lhs = condition.named_child(0).unwrap();
    assert_eq!(&source[lhs.byte_range()], "<$(name)>");
    assert_eq!(condition.child(1).unwrap().kind(), "=");
}

#[test]
fn preserves_keywords_and_delimiters_joined_to_word_suffixes() {
    // B2 src/tools/testing-aux.jam builds a bracketed string this way; a
    // whitespace-delimited '[' is a call, whereas '[$(...)' is a literal prefix.
    for value in [
        r#"[$(source:G=:J=",")"] ""#,
        r#"rule"suffix""#,
        "if$(suffix)",
    ] {
        let source = format!("local value = {value} ;");
        let parsed = tree(&source);
        assert!(
            !parsed.root_node().has_error(),
            "{}",
            parsed.root_node().to_sexp()
        );
        let local = find(parsed.root_node(), "local_statement").unwrap();
        assert_eq!(local.named_child_count(), 2, "{}", local.to_sexp());
        assert_eq!(&source[local.named_child(1).unwrap().byte_range()], value);
    }
}
