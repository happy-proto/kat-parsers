module.exports = grammar({
  name: 'pgpass',

  // libpq preserves spaces and recognizes comments only at the start of a line.
  extras: () => [],

  rules: {
    source_file: $ => seq(
      repeat(seq(optional($._line), $._newline)),
      optional($._line),
    ),
    _line: $ => choice($.comment, $.entry),
    comment: _ => token(prec(1, /#[^\r\n]*/)),
    entry: $ => seq(
      optional(field('hostname', $._match_field)), ':',
      optional(field('port', $._match_field)), ':',
      optional(field('database', $._match_field)), ':',
      optional(field('username', $._match_field)), ':',
      optional(field('password', $.password)),
    ),
    _match_field: $ => choice($.wildcard, $.value),
    wildcard: _ => '*',
    value: $ => repeat1(choice($.text, $.escape_sequence)),
    // A password '*' is literal; it never participates in connection matching.
    password: $ => repeat1(choice($.text, $.escape_sequence)),
    text: _ => /[^:\\\r\n]+/,
    escape_sequence: _ => /\\[^\r\n]/,
    _newline: _ => /\r?\n/,
  },
});
