use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_less_compressed(
        decode: unsafe extern "C" fn(*const u8, usize, usize, usize) -> *const u8,
    ) -> *const ();
}
unsafe extern "C" fn language() -> *const () {
    static LANGUAGE_POINTER: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *LANGUAGE_POINTER
        .get_or_init(|| unsafe { tree_sitter_less_compressed(kat_parser_runtime::decode) as usize })
        as *const ()
}
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(language) };

pub const GRAMMAR: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/vendor/less/grammar.json"
));
pub const NODE_TYPES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/vendor/less/node-types.json"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_load_language() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&LANGUAGE.into())
            .expect("failed to load less parser");
        assert!(!GRAMMAR.is_empty());
        assert!(!NODE_TYPES.is_empty());
    }
}
