use ghostty_conformance::{assert_line_structure, assert_parses, title_cases};

#[test]
fn preserves_line_iterator_key_and_value_boundaries() {
    for (source, key, value, kind) in [
        (
            " \ttitle \t= \t\"say \"hi\"\" \t\r\n",
            "title",
            "\"say \"hi\"\"",
            "quoted_value",
        ),
        ("title = 'hi'", "title", "'hi'", "raw_value"),
        ("title = #literal", "title", "#literal", "raw_value"),
        (
            "title = one=two # three",
            "title",
            "one=two # three",
            "raw_value",
        ),
        ("title = \"unclosed", "title", "\"unclosed", "raw_value"),
        ("title = one\rtwo", "title", "one\rtwo", "raw_value"),
    ] {
        let tree = assert_parses(source);
        let setting = tree.root_node().named_child(0).unwrap();
        assert_eq!(setting.kind(), "setting");
        let key_node = setting.child_by_field_name("key").unwrap();
        let value_node = setting.child_by_field_name("value").unwrap();
        assert_eq!(key_node.utf8_text(source.as_bytes()).unwrap(), key);
        assert_eq!(value_node.utf8_text(source.as_bytes()).unwrap(), value);
        assert_eq!(value_node.kind(), kind);
    }
}

#[test]
fn empty_and_bare_settings_do_not_consume_the_next_line() {
    for source in [
        "font-family =\nfont-size = 14",
        "font-family = \t\r\nfont-size=14",
        "mouse-hide-while-typing\nfont-size=14",
    ] {
        let tree = assert_parses(source);
        let root = tree.root_node();
        assert_eq!(root.named_child_count(), 2);
        assert!(
            root.named_child(0)
                .unwrap()
                .child_by_field_name("value")
                .is_none()
        );
        assert_eq!(
            root.named_child(1)
                .unwrap()
                .child_by_field_name("value")
                .unwrap()
                .utf8_text(source.as_bytes())
                .unwrap(),
            "14"
        );
    }
}

#[test]
fn comments_bom_crlf_and_eof_are_supported() {
    for source in [
        "",
        " \t\r\n",
        "#EOF",
        "  #EOF",
        "\u{feff}# comment\r\ntitle = 👻",
        "# blank\n\nfont-family =\r\n#EOF",
    ] {
        assert_parses(source);
    }
}

#[test]
fn field_specific_value_syntax_stays_intact() {
    for source in title_cases() {
        assert_line_structure(&source);
    }
    for source in [
        "keybind = global:ctrl+physical:a>shift+==new_split:right",
        "keybind = table/ctrl+a=text:a=b #literal",
        "theme = light:Rose Pine Dawn,dark:Rose Pine",
        "palette = 255=#abcdef",
        "env = NAME=a=b #literal",
        "config-file = ?relative/config.ghostty",
        "quick-terminal-size = 50%,400px",
        "command = direct:echo \"hello world\"",
        "initial-command = shell:printf '%s' hello",
        "future-setting = future(value)",
    ] {
        assert_parses(source);
    }
}

#[test]
fn accepted_fixtures_preserve_all_entries() {
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/accepted");
    for entry in std::fs::read_dir(fixtures).unwrap() {
        assert_line_structure(&std::fs::read_to_string(entry.unwrap().path()).unwrap());
    }
}
