module.exports = grammar({
  name: 'authorized_keys',

  // Fields require horizontal whitespace; entries cannot consume another line.
  extras: () => [],

  rules: {
    source_file: $ => seq(
      repeat(seq(optional($._line), $._newline)),
      optional($._line),
    ),

    _line: $ => choice(
      $._space,
      seq(optional($._space), choice($.entry, $.comment_line)),
    ),

    comment_line: _ => /#[^\r\n]*/,

    entry: $ => seq(
      optional(seq(field('options', $.option_list), $._space)),
      field('key_type', $.key_type),
      $._space,
      field('key_blob', $.key_blob),
      optional(seq($._space, optional(field('comment', $.entry_comment)))),
    ),

    option_list: $ => seq(
      $.option,
      repeat(seq(',', $.option)),
    ),

    option: $ => choice(
      seq(
        field('name', $.option_name),
        '=',
        field('value', $.option_value),
      ),
      field('name', $.option_name),
    ),

    option_name: _ => /[A-Za-z][A-Za-z0-9-]*/,
    option_value: $ => choice($.quoted_value, $.bare_value),
    quoted_value: _ => /"([^"\\\r\n]|\\[^\r\n])*"/,
    bare_value: _ => /[^,\s#"]+/,
    // A public key may omit options. Recognize its algorithm before option_name,
    // rather than treating the algorithm as an option and shifting every field.
    key_type: _ => token(prec(1, /(?:ssh-|ecdsa-sha2-|sk-)[A-Za-z0-9@._+-]+/)),
    key_blob: _ => /[A-Za-z0-9+/]+={0,2}/,
    entry_comment: _ => /[^ \t\r\n][^\r\n]*/,
    _space: _ => /[ \t]+/,
    _newline: _ => /\r?\n/,
  },
});
