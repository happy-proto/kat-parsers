use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_coffeescript() -> *const ();
    fn tree_sitter_crystal() -> *const ();
}

pub const COFFEESCRIPT_LANGUAGE: LanguageFn =
    unsafe { LanguageFn::from_raw(tree_sitter_coffeescript) };
pub const COFFEESCRIPT_GRAMMAR: &str = include_str!("../vendor/coffeescript/grammar.json");
pub const COFFEESCRIPT_NODE_TYPES: &str = include_str!("../vendor/coffeescript/node-types.json");

pub const CRYSTAL_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_crystal) };
pub const CRYSTAL_GRAMMAR: &str = include_str!("../vendor/crystal/grammar.json");
pub const CRYSTAL_NODE_TYPES: &str = include_str!("../vendor/crystal/node-types.json");

#[cfg(test)]
mod tests {
    #[test]
    fn can_load_crystal_language() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::CRYSTAL_LANGUAGE.into())
            .expect("failed to load Crystal parser");
        assert!(!super::CRYSTAL_GRAMMAR.is_empty());
        assert!(!super::CRYSTAL_NODE_TYPES.is_empty());
    }

    #[test]
    fn can_load_coffeescript_language() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::COFFEESCRIPT_LANGUAGE.into())
            .expect("failed to load CoffeeScript parser");
        assert!(!super::COFFEESCRIPT_GRAMMAR.is_empty());
        assert!(!super::COFFEESCRIPT_NODE_TYPES.is_empty());
    }
}
