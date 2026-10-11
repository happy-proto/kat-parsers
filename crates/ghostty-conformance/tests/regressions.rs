use ghostty_conformance::assert_line_structure;

#[test]
fn nul_is_data_in_comments_and_bare_keys() {
    assert_line_structure("# before\0after\ntitle=ok\n");
    assert_line_structure("title\0\n");
}

#[test]
fn cr_before_equals_remains_part_of_the_actual_key() {
    // Official CLI diagnoses `title\r`, not `title`: CR is not key whitespace.
    assert_line_structure("title\r=value\n");
    assert_line_structure("title\r = hello\n");
}

#[test]
fn only_one_file_initial_bom_is_removed() {
    // Official CLI diagnoses the later BOM as part of the unknown key.
    assert_line_structure("title=one\n\u{feff}title=two\n");
    assert_line_structure("\u{feff}\u{feff}title=hello\n");
}
