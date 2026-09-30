use tree_sitter::Parser;
use tree_sitter_kat_authorized_keys::LANGUAGE;

#[test]
fn authorized_key_options_and_comments_stay_on_their_own_lines() {
    let source = "# heading\r\n\r\n  command=\"echo \\\"kat\\\"\",from=\"192.0.2.*\",restrict\tssh-ed25519\tAAAA==\t备注 with spaces\r\nssh-rsa AAAA \t\r\n";
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();
    assert!(!root.has_error(), "{}", root.to_sexp());
    let mut cursor = root.walk();
    let entries: Vec<_> = root
        .named_children(&mut cursor)
        .filter(|node| node.kind() == "entry")
        .collect();
    assert_eq!(entries.len(), 2);
    assert_eq!(
        entries[0]
            .child_by_field_name("comment")
            .unwrap()
            .utf8_text(source.as_bytes())
            .unwrap(),
        "备注 with spaces"
    );
    assert!(entries[1].child_by_field_name("comment").is_none());
}

#[test]
fn missing_key_payload_does_not_swallow_the_next_public_key() {
    let source = "ssh-ed25519\nssh-rsa AAAA comment\n";
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();
    assert!(root.has_error());
    let mut cursor = root.walk();
    assert!(
        root.named_children(&mut cursor)
            .any(|node| node.kind() == "entry"
                && !node.has_error()
                && node
                    .child_by_field_name("key_type")
                    .unwrap()
                    .utf8_text(source.as_bytes())
                    .unwrap()
                    == "ssh-rsa"),
        "{}",
        root.to_sexp()
    );
}

#[test]
fn base64_slash_suffix_belongs_to_the_entire_key_blob() {
    let source = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIExample/8\n";
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    let tree = parser.parse(source, None).unwrap();
    let entry = tree.root_node().named_child(0).unwrap();
    assert!(!tree.root_node().has_error());
    assert!(entry.child_by_field_name("options").is_none());
    assert_eq!(
        entry
            .child_by_field_name("key_type")
            .unwrap()
            .utf8_text(source.as_bytes())
            .unwrap(),
        "ssh-ed25519"
    );
    assert_eq!(
        entry
            .child_by_field_name("key_blob")
            .unwrap()
            .utf8_text(source.as_bytes())
            .unwrap(),
        "AAAAC3NzaC1lZDI1NTE5AAAAIExample/8"
    );
    assert!(entry.child_by_field_name("comment").is_none());
}

#[test]
fn public_key_fields_do_not_shift_into_authorization_options() {
    for key_type in [
        "ssh-ed25519",
        "ssh-rsa",
        "ssh-dss",
        "ecdsa-sha2-nistp256",
        "sk-ssh-ed25519@openssh.com",
        "sk-ecdsa-sha2-nistp256@openssh.com",
        "ssh-ed25519-cert-v01@openssh.com",
        "sk-ssh-ed25519-cert-v01@openssh.com",
    ] {
        for prefix in ["", "restrict ", "command=\"echo kat hello\",no-pty "] {
            for ending in ["\n", "\r\n", ""] {
                let source = format!(
                    "{prefix}{key_type} AAAAC3NzaC1lZDI1NTE5AAAAIExample dcjanusmacbook-pro tailnet{ending}"
                );
                let mut parser = Parser::new();
                parser.set_language(&LANGUAGE.into()).unwrap();
                let tree = parser.parse(&source, None).unwrap();
                let root = tree.root_node();
                assert!(!root.has_error(), "{source:?}: {}", root.to_sexp());
                let entry = root.named_child(0).unwrap();
                assert_eq!(
                    entry.child_by_field_name("options").is_some(),
                    !prefix.is_empty(),
                    "{source:?}: {}",
                    root.to_sexp()
                );
                for (field, expected) in [
                    ("key_type", key_type),
                    ("key_blob", "AAAAC3NzaC1lZDI1NTE5AAAAIExample"),
                    ("comment", "dcjanusmacbook-pro tailnet"),
                ] {
                    assert_eq!(
                        entry
                            .child_by_field_name(field)
                            .unwrap()
                            .utf8_text(source.as_bytes())
                            .unwrap(),
                        expected,
                        "{source:?}: {field}"
                    );
                }
            }
        }
    }
}
