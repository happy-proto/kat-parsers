//! 压缩与原生表的真实解析对照；仅用于仓库测试，不属于消费链路。

#[cfg(test)]
mod tests {
    use tree_sitter::{InputEdit, Language, Parser, Point};

    unsafe extern "C" {
        fn tree_sitter_coffeescript_baseline() -> *const ();
    }

    fn languages() -> [Language; 2] {
        let native =
            unsafe { tree_sitter::Language::from_raw(tree_sitter_coffeescript_baseline().cast()) };
        [tree_sitter_kat_coffeescript::LANGUAGE.into(), native]
    }

    #[test]
    fn compressed_and_native_tables_match_for_scanners_errors_and_incremental_edits() {
        first_language_load_is_shared_across_threads();
        let inputs = [
            "double = (x) -> x * 2\n",
            "text = \"hello #{name}\"\n",
            "f = ( ->\n",
        ];
        let [compressed, native] = languages();
        for source in inputs {
            let trees: Vec<_> = [&compressed, &native]
                .iter()
                .map(|language| {
                    let mut parser = Parser::new();
                    parser.set_language(language).unwrap();
                    let tree = parser.parse(source, None).unwrap();
                    let mut edited = tree.clone();
                    let (start, old_end, new_end, point, end_point, next) =
                        if let Some(index) = source.find('2') {
                            (
                                index,
                                index + 1,
                                index + 1,
                                Point::new(0, index),
                                Point::new(0, index + 1),
                                source.replacen('2', "3", 1),
                            )
                        } else {
                            let point = Point::new(source.lines().count(), 0);
                            (
                                source.len(),
                                source.len(),
                                source.len() + 1,
                                point,
                                Point::new(point.row + 1, 0),
                                format!("{source}\n"),
                            )
                        };
                    edited.edit(&InputEdit {
                        start_byte: start,
                        old_end_byte: old_end,
                        new_end_byte: new_end,
                        start_position: point,
                        old_end_position: if old_end > start {
                            Point::new(0, old_end)
                        } else {
                            point
                        },
                        new_end_position: end_point,
                    });
                    let incremental = parser.parse(&next, Some(&edited)).unwrap();
                    let fresh = parser.parse(&next, None).unwrap();
                    assert_eq!(
                        incremental.root_node().to_sexp(),
                        fresh.root_node().to_sexp()
                    );
                    assert_eq!(
                        incremental.root_node().has_error(),
                        fresh.root_node().has_error()
                    );
                    (tree.root_node().to_sexp(), tree.root_node().has_error())
                })
                .collect();
            assert_eq!(trees[0], trees[1], "{source}");
            if source != "f = ( ->\n" {
                assert!(!trees[0].1);
            }
            if source == "f = ( ->\n" {
                assert!(trees[0].1);
            }
        }
        for state in [0, 2, 17, 100] {
            let left: Vec<_> = compressed.lookahead_iterator(state).unwrap().collect();
            let right: Vec<_> = native.lookahead_iterator(state).unwrap().collect();
            assert_eq!(left, right);
        }
    }

    fn first_language_load_is_shared_across_threads() {
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(16));
        let handles: Vec<_> = (0..16)
            .map(|_| {
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    let language: Language = tree_sitter_kat_coffeescript::LANGUAGE.into();
                    let mut parser = Parser::new();
                    parser.set_language(&language).unwrap();
                    assert!(
                        !parser
                            .parse("answer = 42\n", None)
                            .unwrap()
                            .root_node()
                            .has_error()
                    );
                    language.into_raw() as usize
                })
            })
            .collect();
        let pointers: Vec<_> = handles
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect();
        assert!(pointers.iter().all(|pointer| *pointer == pointers[0]));
    }
}
