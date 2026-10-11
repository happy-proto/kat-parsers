// Independently implemented from Ghostty's LineIterator / Config.loadReader.
// Values belong to their key's parser, not to a global INI/TOML type system.
module.exports = grammar({
  name: 'ghostty',
  extras: () => [/[ \t\r]+/],
  externals: $ => [$.key, $.raw_value, $.quoted_value, $.comment, $._bare_key, $._assignment_start, $._bare_start],
  rules: {
    source_file: $ => seq(
      // Tree-sitter's lexer already skips exactly one byte-zero UTF-8 BOM.
      repeat(seq(optional(choice($.setting, $.comment)), '\n')),
      optional(choice($.setting, $.comment)),
    ),
    setting: $ => choice(
      seq($._assignment_start, field('key', $.key), '=', optional(field('value', choice($.raw_value, $.quoted_value)))),
      seq($._bare_start, field('key', alias($._bare_key, $.key))),
    ),
  },
});
