// Independently authored syntax-highlighting grammar for Graph::Easy.
// Reference: y9c/graph-easy-py (syntax only; no implementation copied).
module.exports = grammar({
  name: 'graph_easy',

  extras: $ => [/\s/, $.comment],

  rules: {
    source_file: $ => repeat($._statement),
    _statement: $ => choice($.chain, $.defaults, ';'),
    chain: $ => prec.right(seq(
      $.node_list,
      repeat(seq($.edge, $.node_list)),
    )),
    node_list: $ => seq($._endpoint, repeat(seq(',', $._endpoint))),
    _endpoint: $ => choice($.node, $.bare_node, $.group),
    node: $ => seq('[', optional($.node_label), ']', repeat($.attributes)),
    node_label: $ => repeat1(choice($.node_text, $.escape_sequence)),
    node_text: $ => token(prec(1, /[^\]\\]+/)),
    escape_sequence: $ => token(/\\[^\r\n]/),
    bare_node: $ => seq($.identifier, repeat($.attributes)),
    identifier: $ => /[^\s\[\]{}()<>;,=:~#.-]+/,
    group: $ => seq(
      '(', optional($.group_name), repeat($._statement), ')', repeat($.attributes),
    ),
    group_name: $ => token(prec(1, /[^\s\[\]{}()#][^\[\]{}()\r\n]*/)),
    edge: $ => prec.right(seq(
      $.edge_operator,
      optional(choice(
        seq($.edge_label, $.edge_operator),
        seq($.attributes, $.edge_operator),
      )),
      repeat($.attributes),
    )),
    edge_operator: $ => token(/<?[-=.~#]+>?/),
    edge_label: $ => token(/[^\s\[\]{}()<>\r\n=~#.-][^\[\]{}()<>\r\n=~#.-]*/),
    defaults: $ => seq($.class_name, $.attributes),
    class_name: $ => token(seq(choice('graph', 'node', 'edge', 'group'), optional(/\.[a-zA-Z_][a-zA-Z0-9_-]*/))),
    attributes: $ => seq('{', repeat($.attribute), '}'),
    attribute: $ => seq($.attribute_name, ':', optional($.attribute_value), optional(';')),
    attribute_name: $ => /[a-zA-Z_][a-zA-Z0-9_-]*/,
    attribute_value: $ => repeat1(choice($.value_text, $.quoted_value, $.escape_sequence)),
    value_text: $ => token(prec(1, /[^;{}"\\\r\n]+/)),
    quoted_value: $ => seq('"', repeat(choice(token.immediate(/[^"\\]+/), $.escape_sequence)), '"'),
    comment: $ => token(/#[^#\r\n][^\r\n]*|#/),
  },
});
