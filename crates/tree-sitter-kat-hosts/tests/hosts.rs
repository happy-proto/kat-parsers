use tree_sitter::{Parser, Tree};
use tree_sitter_kat_hosts::LANGUAGE;

fn parse(source: &str) -> Tree {
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    parser.parse(source, None).unwrap()
}

#[test]
fn parses_addresses_aliases_comments_and_line_endings() {
    for source in [
        "",
        " \t\n",
        "  # comment\r\n",
        "127.0.0.1 localhost",
        "  127.0.0.1\tlocalhost loopback \t# aliases\r\n",
        "::1 localhost ip6-localhost#comment\n",
        "2001:db8::1 server.example server\n",
        "::ffff:192.0.2.1 mapped.example\n",
        "0.0.0.0 ads.example tracker.example\n",
        "# 中文注释\n127.0.0.1 localhost\n\n::1 localhost\n",
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
fn preserves_field_roles_and_stops_at_comments() {
    let source = "127.0.0.1 localhost loopback# disabled.example\n";
    let tree = parse(source);
    let entry = tree.root_node().named_child(0).unwrap();
    for (field, text) in [
        ("address", "127.0.0.1"),
        ("hostname", "localhost"),
        ("alias", "loopback"),
    ] {
        assert_eq!(
            entry
                .child_by_field_name(field)
                .unwrap()
                .utf8_text(source.as_bytes())
                .unwrap(),
            text
        );
    }
    assert_eq!(entry.named_child(3).unwrap().kind(), "comment");
}

#[test]
fn incomplete_entries_do_not_borrow_fields_from_next_line() {
    for source in ["127.0.0.1\nlocalhost\n", "127.0.0.1\n::1 localhost\n"] {
        let tree = parse(source);
        assert!(
            tree.root_node().has_error(),
            "{}",
            tree.root_node().to_sexp()
        );
    }
    let source = "broken entry\n127.0.0.1 localhost\n::1 localhost\n";
    let tree = parse(source);
    let root = tree.root_node();
    let mut cursor = root.walk();
    let entries: Vec<_> = root
        .named_children(&mut cursor)
        .filter(|n| n.kind() == "entry")
        .collect();
    assert_eq!(entries.len(), 2, "{}", root.to_sexp());
}
