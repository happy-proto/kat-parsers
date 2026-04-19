use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_markdown() -> *const ();
}

pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_markdown) };
pub const GRAMMAR: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/vendor/markdown/grammar.json"
));
pub const NODE_TYPES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/vendor/markdown/node-types.json"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_load_language() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&LANGUAGE.into())
            .expect("failed to load markdown parser");
        assert!(!GRAMMAR.is_empty());
        assert!(!NODE_TYPES.is_empty());
    }
}
