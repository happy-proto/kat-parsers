use tree_sitter_kat_mermaid::LANGUAGE;

#[test]
fn parses_common_diagrams_and_utf8_labels() {
    let sources = [
        "%% 中文注释\nflowchart TD\nA[\"客户端\"] -->|请求| B{鉴权}\n",
        "sequenceDiagram\nparticipant Alice\nAlice->>Bob: 你好\n",
        "classDiagram\nclass Animal {\n+int age\n+eat()\n}\nAnimal <|-- Duck\n",
        "stateDiagram-v2\n[*] --> Ready\nReady --> Done: 完成\n",
        "erDiagram\nCUSTOMER ||--o{ ORDER : places\n",
        "gantt\ntitle Release\ndateFormat YYYY-MM-DD\nsection Build\nCompile :a1, 2026-01-01, 1d\n",
        "pie title Traffic\n\"Success\" : 90\n\"Error\" : 10\n",
        "mindmap\n  root((系统))\n    API\n    Database\n",
    ];
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    for source in sources {
        let tree = parser.parse(source, None).unwrap();
        assert!(
            !tree.root_node().has_error(),
            "{source}\n{}",
            tree.root_node().to_sexp()
        );
    }
}

#[test]
fn recovers_incomplete_diagram_without_losing_the_header() {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    let tree = parser.parse("flowchart TD\nA[\"unfinished", None).unwrap();
    assert!(tree.root_node().to_sexp().contains("diagram_keyword"));
}
