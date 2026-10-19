# B2 language fixtures

These fixtures come from [bfgroup/b2](https://github.com/bfgroup/b2) revision
`4ca043f11b3b6de25bb268d5d874d714344074d9`. They retain the original copyright
notices and use the [Boost Software License 1.0](LICENSE.txt).

| Fixture | Upstream source | Expected syntax |
| --- | --- | --- |
| `core-language.jam` | `test/core-language/test.jam`, unchanged | valid |
| `scanner.jam` | `test/core_scanner.py:14`, `Tester.write` string value | valid |
| `typecheck.jam` | `test/core_typecheck.py:13`, `Tester.write` string value | valid |
| `at-file.jam` | `test/core_at_file.py:14`, `Tester.write` string value | valid |
| `dollars.jam` | `test/escaping_dollar_before_round_bracket.py:16`, `Tester.write` string value | valid |
| `print.jam` | `test/print.py:25`, `Tester.write` string value | valid |
| `unterminated-string.jam` | `test/core_source_line_tracking.py:15`, `Tester.write` string value | invalid |
| `missing-semicolon.jam` | `test/core_syntax_error_exit_status.py:16`, `Tester.write` string value | invalid |

For Python tests, the string value is extracted with Python AST literal evaluation;
Python backslash escapes must be decoded before testing the Jam input. Copyright
and source comments are prepended; the Jam string values themselves are unchanged.
