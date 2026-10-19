// Independently authored for kat. B2's default scanner uses whitespace-delimited
// words: even a keyword or delimiter is literal when joined to a quoted or
// expanded suffix. The internal regex lexer cannot look past that suffix start
// without consuming it, so this token handles only that lexical boundary.
#include "tree_sitter/parser.h"
#include <stdbool.h>
#include <stddef.h>
#include <wctype.h>

enum TokenType { LITERAL_PREFIX };

void *tree_sitter_jam_external_scanner_create(void) { return NULL; }
void tree_sitter_jam_external_scanner_destroy(void *payload) { (void)payload; }
unsigned tree_sitter_jam_external_scanner_serialize(void *payload, char *buffer) {
  (void)payload;
  (void)buffer;
  return 0;
}
void tree_sitter_jam_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
  (void)payload;
  (void)buffer;
  (void)length;
}

bool tree_sitter_jam_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
  (void)payload;
  if (!valid_symbols[LITERAL_PREFIX]) return false;
  while (iswspace(lexer->lookahead)) lexer->advance(lexer, true);
  // A comment at the start of a word always wins, including #$(name).
  if (lexer->lookahead == '#') return false;
  unsigned length = 0;
  bool starts_at = lexer->lookahead == '@';
  while (!lexer->eof(lexer) && !iswspace(lexer->lookahead)
         && lexer->lookahead != '"' && lexer->lookahead != '$') {
    // Let the grammar retain the structural @(...), rather than a raw prefix.
    if (starts_at && length == 1 && lexer->lookahead == '(') return false;
    if (lexer->lookahead == '\\') {
      lexer->advance(lexer, false);
      if (lexer->eof(lexer)) return false;
    }
    lexer->advance(lexer, false);
    ++length;
  }
  if (length == 0 || (lexer->lookahead != '"' && lexer->lookahead != '$')) return false;
  lexer->mark_end(lexer);
  lexer->result_symbol = LITERAL_PREFIX;
  return true;
}
