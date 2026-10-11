use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_ghostty() -> *const ();
}

pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_ghostty) };
pub const GRAMMAR: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/vendor/ghostty/grammar.json"
));
pub const NODE_TYPES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/vendor/ghostty/node-types.json"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_load_language() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&LANGUAGE.into())
            .expect("failed to load ghostty parser");
        assert!(!GRAMMAR.is_empty());
        assert!(!NODE_TYPES.is_empty());
    }
}
