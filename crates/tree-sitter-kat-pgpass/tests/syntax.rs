use tree_sitter::{Parser, Tree};
use tree_sitter_kat_pgpass::LANGUAGE;

fn parse(source: &str) -> Tree {
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    parser.parse(source, None).unwrap()
}

#[test]
fn parses_fields_escapes_empty_values_and_line_endings() {
    for source in [
        "localhost:5432:demo:reader:example-only",
        "*:*:*:*:*\n",
        "::::\r\n",
        "# synthetic credentials only\r\n\r\nlocalhost:5432:demo:reader:fake\r\n",
        r"2001\:db8\:\:1:5432:demo\:db:DOMAIN\\reader:fake\:pass\\word",
        " host : service : database : user : fake password # literal\n",
        "localhost:5432:数据库:读者:虚构口令\n",
        r"\#host:5432:demo:reader:fake",
    ] {
        let tree = parse(source);
        assert!(
            !tree.root_node().has_error(),
            "{source:?}: {}",
            tree.root_node().to_sexp()
        );
    }
}

#[test]
fn distinguishes_wildcards_literal_stars_and_comments() {
    let source = "#localhost:5432:demo:reader:fake\n*:*:demo*:reader:*#literal\n";
    let tree = parse(source);
    let root = tree.root_node();
    assert!(!root.has_error(), "{}", root.to_sexp());
    assert_eq!(root.named_child(0).unwrap().kind(), "comment");
    let entry = root.named_child(1).unwrap();
    for field in ["hostname", "port"] {
        assert_eq!(entry.child_by_field_name(field).unwrap().kind(), "wildcard");
    }
    assert_eq!(
        entry.child_by_field_name("database").unwrap().kind(),
        "value"
    );
    let password = entry.child_by_field_name("password").unwrap();
    assert_eq!(password.kind(), "password");
    assert_eq!(password.utf8_text(source.as_bytes()).unwrap(), "*#literal");

    // Leading spaces are data, not indentation before a comment.
    let source = " #host:5432:demo:reader:fake\n";
    let tree = parse(source);
    assert!(!tree.root_node().has_error());
    assert_eq!(tree.root_node().named_child(0).unwrap().kind(), "entry");
}

#[test]
fn rejects_incomplete_entries_and_unescaped_colons() {
    for source in ["broken:5432:demo\n", "host:5432:db:user:fake:extra\n"] {
        let tree = parse(source);
        assert!(
            tree.root_node().has_error(),
            "{}",
            tree.root_node().to_sexp()
        );
    }
}
