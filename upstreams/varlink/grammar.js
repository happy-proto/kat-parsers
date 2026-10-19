/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "varlink",
  conflicts: ($) => [
    // interface_declaration/typedef/error/method must be separated by newlines
    // when a newline is encountered, we don't yet know whether another definition will follow
    [$._, $._eventually_eol],
    [$.enum_member_name, $.struct_field_name],
  ],
  extras: (_) => [],
  rules: {
    interface: ($) =>
      seq(
        repeat($._),
        field("declaration", $.interface_declaration),
        repeat(seq($._eventually_eol, $.declaration)),
        repeat($._),
      ),

    _: ($) => choice($._hspace, $.comment, $._eol),
    _eventually_eol: ($) =>
      seq(repeat(choice($._hspace, $.comment)), $._eol, repeat($._)),

    _hspace: (_) => /[ \t]+/,
    comment: (_) => /#[^\n]*/,
    _eol: (_) => /\n/,

    declaration: ($) => choice($.typedef, $.error, $.method),

    keyword_interface: (_) => "interface",
    interface_declaration: ($) =>
      seq(
        field("keyword", $.keyword_interface),
        repeat1($._),
        field("name", $.interface_name),
      ),

    keyword_type: (_) => "type",
    typedef: ($) =>
      seq(
        field("keyword", $.keyword_type),
        repeat1($._),
        field("name", $.typedef_name),
        repeat($._),
        field("value", choice($.struct, $.enum)),
      ),

    keyword_error: (_) => "error",
    error: ($) =>
      seq(
        field("keyword", $.keyword_error),
        repeat1($._),
        field("name", $.error_name),
        repeat($._),
        field("value", $.struct),
      ),

    keyword_method: (_) => "method",
    arrow: (_) => "->",
    method: ($) =>
      seq(
        field("keyword", $.keyword_method),
        repeat1($._),
        field("name", $.method_name),
        repeat($._),
        field("input", $.struct),
        repeat($._),
        field("arrow", $.arrow),
        repeat($._),
        field("output", $.struct),
      ),

    struct: ($) =>
      seq(
        "(",
        repeat($._),
        optional(
          seq(
            field("member", $.struct_field),
            repeat($._),
            repeat(
              seq(
                ",",
                repeat($._),
                field("member", $.struct_field),
                repeat($._),
              ),
            ),
          ),
        ),
        ")",
      ),

    struct_field: ($) =>
      seq(
        field("name", $.struct_field_name),
        repeat($._),
        ":",
        repeat($._),
        field("value", $.type),
      ),

    // NOTE: There is no empty enum, i.e. `()` is a struct
    enum: ($) =>
      seq(
        "(",
        repeat($._),
        seq(
          field("member", $.enum_member_name),
          repeat($._),
          repeat(
            seq(",", repeat($._), field("member", $.enum_member_name), repeat($._)),
          ),
        ),
        ")",
      ),

    type: ($) => choice($.maybe, $._just),
    questionmark: (_) => "?",
    maybe: ($) =>
      seq(field("questionmark", $.questionmark), field("just", $.type)),

    _just: ($) =>
      choice(
        $.struct,
        $.enum,
        $.array,
        $.dict,
        $.bool,
        $.int,
        $.float,
        $.string,
        $.object,
        $.any,
        $.typeref,
      ),

    array: ($) => seq("[", "]", field("type", $.type)),
    dict: ($) => seq("[", $.string, "]", field("type", $.type)),

    bool: (_) => "bool",
    int: (_) => "int",
    float: (_) => "float",
    string: (_) => "string",
    object: (_) => "object",
    any: (_) => "any",

    typedef_name: ($) => $.name,
    error_name: ($) => $.name,
    method_name: ($) => $.name,
    typeref: ($) => $.name,
    struct_field_name: ($) => $._field_name,
    enum_member_name: ($) => $._field_name,

    // letters + numbers,
    //  starting with a capital letter
    name: (_) => /[A-Z][a-zA-Z0-9]*/,

    // letters + numbers + underscores,
    //  starting with a letter,
    //  no consecutive or trailing underscores
    _field_name: (_) => /[a-zA-Z](_?[a-zA-Z0-9])*/,

    // at least two dot-separated components of letters + numbers + dashes,
    //  first component starting with a letter,
    //  no initial or trailing dashes
    interface_name: (_) =>
      /[a-zA-Z](-*[a-zA-Z0-9])*(\.[a-zA-Z0-9](-*[a-zA-Z0-9])*)+/,
  },
});
