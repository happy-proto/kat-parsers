# Jam build language

The parser targets B2's default whitespace-delimited Jam scanner, independently
implemented from the language reference. Punctuation embedded in a word stays
literal; standalone delimiters remain structural. The external scanner handles
raw word prefixes joined to quoted or expanded suffixes, where a regex lexer
cannot inspect the suffix without consuming it. This keeps keywords and `[` in
such prefixes from being mistaken for control flow or a rule call.

## Tests

Run `cargo test -p tree-sitter-kat-jam` for minimized regressions, original B2
fixtures (including the complete core language test), structural word-boundary
assertions and intentional syntax errors. Fixture provenance, license and update
rules are in [the fixture manifest](../crates/tree-sitter-kat-jam/tests/fixtures/b2/README.md).

When changing lexical rules, test the same fixed B2 revision rather than mixing
revisions. Build B2 with its `bootstrap.sh`, then run the relevant upstream tests
from its `test/` directory using Python 3:

- `core_language.py`, `core_arguments.py`, `core_scanner.py`, `core_at_file.py`
- `core_varnames.py`, `core_typecheck.py`, `core_modifiers.py`, `core_variables_in_actions.py`
- `core_source_line_tracking.py`, `core_syntax_error_exit_status.py`, `escaping_dollar_before_round_bracket.py`

These are B2 engine behavior tests. Passing them confirms the reference engine
and its expectations; kat's parser must separately consume their Jam inputs.
For broader syntax checks, inspect `test/`, `example/`, and `src/` Jam files and
literal `Tester.write` strings in `test/*.py`. Static extraction does not cover
dynamically assembled strings. Do not count an expected syntax error as a parser
regression: the unterminated-string and missing-semicolon tests are negative
fixtures; this B2 revision's unused `src/tools/python-config.jam` also produces a
syntax error in B2 itself.

## Boundaries

The grammar supports structural source highlighting, not build execution,
property validation or full equivalence to B2's runtime. Actions are balanced raw
command text without platform-specific shell parsing. The experimental B2
`--new-scanner` mode is not a separate language profile here.
