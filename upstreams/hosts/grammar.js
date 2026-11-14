module.exports = grammar({
  name: 'hosts',
  // Horizontal whitespace separates fields; entries cannot consume another line.
  extras: () => [],
  rules: {
    source_file: $ => seq(
      repeat(seq(optional($._line), $._newline)),
      optional($._line),
    ),
    _line: $ => choice(
      $._space,
      seq(optional($._space), choice($.entry, $.comment)),
    ),
    entry: $ => seq(
      field('address', $.address),
      $._space,
      field('hostname', $.hostname),
      repeat(seq($._space, field('alias', $.hostname))),
      optional($._space),
      optional($.comment),
    ),
    // Address-shaped tokens, not IP validation; retain IPv4-mapped IPv6 forms.
    address: _ => /[0-9A-Fa-f]*[.:][0-9A-Fa-f.:]*/,
    hostname: _ => /[^ \t\r\n#]+/,
    comment: _ => /#[^\r\n]*/,
    _space: _ => /[ \t]+/,
    _newline: _ => /\r?\n/,
  },
});
