use tree_sitter::{Parser, Tree};
use tree_sitter_kat_known_hosts::LANGUAGE;

fn parse(source: &str) -> Tree {
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    parser.parse(source, None).unwrap()
}

#[test]
fn parses_openssh_known_hosts_fields_and_line_endings() {
    let cases = [
        "example.com ssh-ed25519 AAAA",
        "example.com,192.0.2.1 ssh-rsa AAAA== trailing comment\n",
        "[example.com]:2222,[2001:db8::1]:2222 ssh-ed25519 AAAA\n",
        "@cert-authority *.example.org,!excluded.example.org ssh-ed25519 AAAA\n",
        "@revoked old.example.org ssh-ed25519 AAAA\n",
        "|1|c2FsdA==|aGFzaA== ssh-ed25519 AAAA\n",
        "  example.com\tssh-ed25519\tAAAA  comment with spaces\r\n",
        "# comment\r\n\r\n \t\r\nexample.com ssh-ed25519 AAAA \t\r\n",
        "example.com sk-ssh-ed25519@openssh.com AAAA\n",
        "example.com ssh-ed25519-cert-v01@openssh.com AAAA\n",
    ];
    for source in cases {
        let tree = parse(source);
        let root = tree.root_node();
        assert!(!root.has_error(), "{source:?}: {}", root.to_sexp());
        let mut cursor = root.walk();
        let entries: Vec<_> = root
            .named_children(&mut cursor)
            .filter(|node| node.kind() == "entry")
            .collect();
        assert_eq!(entries.len(), 1, "{source:?}");
        let entry = entries[0];
        assert!(entry.child_by_field_name("hosts").is_some());
        assert_eq!(
            entry
                .child_by_field_name("key_type")
                .unwrap()
                .utf8_text(source.as_bytes())
                .unwrap(),
            source
                .split_whitespace()
                .find(|word| word.starts_with("ssh-") || word.starts_with("sk-"))
                .unwrap()
        );
        assert!(entry.child_by_field_name("key_blob").is_some());
    }
}

#[test]
fn distinguishes_hashed_hosts_markers_and_host_patterns() {
    let source = "@revoked |1|c2FsdA==|aGFzaA== ssh-ed25519 AAAA\n";
    let tree = parse(source);
    let entry = tree.root_node().named_child(0).unwrap();
    assert_eq!(
        entry
            .child_by_field_name("marker")
            .unwrap()
            .named_child(0)
            .unwrap()
            .kind(),
        "revoked"
    );
    let hosts = entry.child_by_field_name("hosts").unwrap();
    assert_eq!(hosts.kind(), "hashed_host");
    assert_eq!(
        hosts
            .child_by_field_name("salt")
            .unwrap()
            .utf8_text(source.as_bytes())
            .unwrap(),
        "c2FsdA=="
    );
    assert_eq!(
        hosts
            .child_by_field_name("hash")
            .unwrap()
            .utf8_text(source.as_bytes())
            .unwrap(),
        "aGFzaA=="
    );
}

#[test]
fn rejects_missing_fields_without_consuming_the_next_entry() {
    for invalid in [
        "example.com ssh-ed25519",
        "@unknown example.com ssh-ed25519 AAAA",
        "|1|salt| ssh-ed25519 AAAA",
        "example.com, ssh-ed25519 AAAA",
    ] {
        let source = format!("{invalid}\nvalid.example ssh-ed25519 AAAA\n");
        let tree = parse(&source);
        let root = tree.root_node();
        assert!(root.has_error(), "{invalid}: {}", root.to_sexp());
        let mut cursor = root.walk();
        assert!(
            root.named_children(&mut cursor)
                .any(|node| node.kind() == "entry"
                    && !node.has_error()
                    && node.utf8_text(source.as_bytes()).unwrap()
                        == "valid.example ssh-ed25519 AAAA"),
            "{invalid}: {}",
            root.to_sexp()
        );
    }
}
