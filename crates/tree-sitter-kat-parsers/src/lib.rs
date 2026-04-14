use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_crystal() -> *const ();
}

pub const CRYSTAL_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_crystal) };

pub const CRYSTAL_NODE_TYPES: &str = include_str!("../vendor/crystal/node-types.json");

#[cfg(test)]
mod tests {
    #[test]
    fn can_load_language() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::CRYSTAL_LANGUAGE.into())
            .expect("failed to load Crystal parser");
    }
}
