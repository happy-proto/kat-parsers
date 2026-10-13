use tree_sitter::{Node, Parser};
use tree_sitter_kat_graph_easy::LANGUAGE;

fn count(node: Node<'_>, kind: &str) -> usize {
    let mut cursor = node.walk();
    usize::from(node.kind() == kind)
        + node
            .children(&mut cursor)
            .map(|child| count(child, kind))
            .sum::<usize>()
}

#[test]
fn parses_graph_easy_syntax_with_structural_roles() {
    let source = r#"# pipeline
graph { flow: east; }
node.warning { color: red; }
( Pipeline
 [ 输入 ] { label: "read; data"; } -- 请求 --> [ Worker ]
 [ Worker ] -- { label: retry; } --> [ 输出 ]
)
[ Worker ] <--> [ Cache ]
[ Cache ] ..> [ Store ] ==> [ Backup ] ~~> [ Remote ] ##> [ Done ]
[ A ], [ B ] -> [ C ] -> [ D ]
Start -> Finish <- Other
[ 多行\n标签 ] -- [ Hash # inside ]
"#;
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();
    assert!(!root.has_error(), "{}", root.to_sexp());
    for kind in [
        "node",
        "bare_node",
        "edge",
        "edge_label",
        "group",
        "defaults",
        "attribute",
        "escape_sequence",
        "comment",
    ] {
        assert!(count(root, kind) > 0, "missing {kind}: {}", root.to_sexp());
    }
    assert_eq!(count(root, "comment"), 1);
}

#[test]
fn keeps_complete_edges_when_a_later_node_is_unfinished() {
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    let tree = parser
        .parse("[ R ] -> [ Longest ]\n[ B ] -> [ unfinished", None)
        .unwrap();
    assert!(
        count(tree.root_node(), "edge") >= 1,
        "{}",
        tree.root_node().to_sexp()
    );
}
