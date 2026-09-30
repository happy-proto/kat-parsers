module.exports = grammar({
  name: 'known_hosts',

  // Whitespace is significant between fields; entries never cross line boundaries.
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

    entry: $ => seq(
      optional(seq(field('marker', $.marker), $._space)),
      field('hosts', choice($.host_list, $.hashed_host)),
      $._space,
      field('key_type', $.key_type),
      $._space,
      field('key_blob', $.key_blob),
      optional(seq($._space, optional(field('comment', $.entry_comment)))),
    ),

    marker: $ => choice($.cert_authority, $.revoked),
    cert_authority: _ => '@cert-authority',
    revoked: _ => '@revoked',

    host_list: $ => seq($.host_pattern, repeat(seq(',', $.host_pattern))),
    host_pattern: $ => seq(optional('!'), $.hostname),
    hostname: _ => /[^ \t\r\n,!|#@]+/,

    hashed_host: $ => seq(
      '|1|',
      field('salt', $.hash_salt),
      '|',
      field('hash', $.host_hash),
    ),
    hash_salt: _ => /[A-Za-z0-9+/]+={0,2}/,
    host_hash: _ => /[A-Za-z0-9+/]+={0,2}/,
    key_type: _ => /[A-Za-z0-9][A-Za-z0-9@._+-]*/,
    key_blob: _ => /[A-Za-z0-9+/]+={0,2}/,
    entry_comment: _ => /[^ \t\r\n][^\r\n]*/,
    comment_line: _ => /#[^\r\n]*/,
    _space: _ => /[ \t]+/,
    _newline: _ => /\r?\n/,
  },
});
