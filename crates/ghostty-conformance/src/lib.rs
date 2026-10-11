//! Grammar contracts and optional checks against the real Ghostty CLI.

pub fn parse(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_kat_ghostty::LANGUAGE.into())
        .expect("load Ghostty grammar");
    parser.parse(source, None).expect("parse Ghostty config")
}

pub fn assert_parses(source: &str) -> tree_sitter::Tree {
    let tree = parse(source);
    assert!(
        !tree.root_node().has_error(),
        "config: {source:?}\nAST: {}",
        tree.root_node().to_sexp()
    );
    tree
}

/// Check AST text ranges against the file-level contract of LineIterator.
/// Schema interpretation (booleans, colors, keybindings, etc.) belongs to Ghostty.
pub fn assert_line_structure(source: &str) -> tree_sitter::Tree {
    let tree = assert_parses(source);
    let original = source.as_bytes();
    let lines = source.strip_prefix('\u{feff}').unwrap_or(source);
    let entries: Vec<_> = lines
        .split('\n')
        .map(|line| line.trim_matches([' ', '\t', '\r']))
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (key, value) = match line.split_once('=') {
                Some((key, value)) => (
                    key.trim_matches([' ', '\t']),
                    Some(value.trim_matches([' ', '\t'])),
                ),
                None => (line, None),
            };
            (key, value.filter(|value| !value.is_empty()))
        })
        .collect();
    let root = tree.root_node();
    let settings: Vec<_> = {
        let mut cursor = root.walk();
        root.named_children(&mut cursor)
            .filter(|node| node.kind() == "setting")
            .collect()
    };
    assert_eq!(settings.len(), entries.len());
    for (setting, (key, value)) in settings.iter().zip(entries) {
        assert_eq!(
            setting
                .child_by_field_name("key")
                .unwrap()
                .utf8_text(original)
                .unwrap(),
            key
        );
        let value_node = setting.child_by_field_name("value");
        assert_eq!(
            value_node.map(|node| node.utf8_text(original).unwrap()),
            value
        );
        if let (Some(node), Some(value)) = (value_node, value) {
            let quoted = value.len() >= 2 && value.starts_with('"') && value.ends_with('"');
            assert_eq!(
                node.kind(),
                if quoted { "quoted_value" } else { "raw_value" }
            );
        }
    }
    tree
}

pub fn title_cases() -> Vec<String> {
    let values = [
        "plain",
        "two words",
        "#literal",
        "a # is a value",
        "a=b=c",
        "'single quotes stay literal'",
        "\"double quoted\"",
        "\"say \"hi\"\"",
        "\" trailing space \"",
        "\"\"",
        "\"",
        "\"not closed",
        "backslash\\n stays literal",
        "中文 👻",
        "?not-a-path",
        ";not-a-comment",
        "[not-a-section]",
        "line\rinside",
        "true",
        "123456",
        "#ff00ff",
    ];
    let mut cases = Vec::new();
    for value in values {
        for (left, right, ending) in [("", "", ""), (" \t", "\t ", "\n"), ("\t", " ", "\r\n")] {
            cases.push(format!("{left}title{right}={left}{value}{right}{ending}"));
        }
    }
    cases
}
