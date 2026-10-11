// Repository-owned implementation of Ghostty's line token boundaries.
// No value-specific validation or escape decoding: LineIterator only removes
// enclosing double quotes; each Config field interprets its own value later.
#include "tree_sitter/parser.h"

enum Token { KEY, RAW_VALUE, QUOTED_VALUE, COMMENT, BARE_KEY, ASSIGNMENT_START, BARE_START };

void *tree_sitter_ghostty_external_scanner_create(void) { return NULL; }
void tree_sitter_ghostty_external_scanner_destroy(void *payload) { (void)payload; }
unsigned tree_sitter_ghostty_external_scanner_serialize(void *payload, char *buffer) {
  (void)payload;
  (void)buffer;
  return 0;
}
void tree_sitter_ghostty_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
  (void)payload;
  (void)buffer;
  (void)length;
}

static bool horizontal(int32_t c) { return c == ' ' || c == '\t'; }

bool tree_sitter_ghostty_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
  (void)payload;
  const bool start = valid_symbols[ASSIGNMENT_START] || valid_symbols[BARE_START];
  const bool bare = valid_symbols[BARE_KEY];
  const bool key = valid_symbols[KEY] || bare || start;
  if (!key && !valid_symbols[RAW_VALUE] && !valid_symbols[QUOTED_VALUE] && !valid_symbols[COMMENT]) return false;

  while (horizontal(lexer->lookahead) || (key && lexer->lookahead == '\r')) {
    lexer->advance(lexer, true);
  }
  if (valid_symbols[COMMENT] && lexer->lookahead == '#') {
    while (!lexer->eof(lexer) && lexer->lookahead != '\n') lexer->advance(lexer, false);
    lexer->mark_end(lexer);
    lexer->result_symbol = COMMENT;
    return true;
  }
  if (lexer->eof(lexer) || lexer->lookahead == '\n' ||
      (key && (lexer->lookahead == '#' || lexer->lookahead == '='))) return false;

  if (start) {
    // Look ahead without consuming: assignment keys and bare keys have
    // different trailing-CR rules. The next scan uses the selected context.
    lexer->mark_end(lexer);
    while (!lexer->eof(lexer) && lexer->lookahead != '\n' && lexer->lookahead != '=') {
      lexer->advance(lexer, false);
    }
    const enum Token token = lexer->lookahead == '=' ? ASSIGNMENT_START : BARE_START;
    if (!valid_symbols[token]) return false;
    lexer->result_symbol = token;
    return true;
  }

  const int32_t first = lexer->lookahead;
  int32_t last = 0;
  unsigned significant_length = 0;
  unsigned length = 0;
  while (!lexer->eof(lexer) && lexer->lookahead != '\n' && (!key || lexer->lookahead != '=')) {
    const int32_t c = lexer->lookahead;
    lexer->advance(lexer, false);
    length++;
    if (!horizontal(c) && ((!bare && key) || c != '\r')) {
      last = c;
      significant_length = length;
      lexer->mark_end(lexer);
    }
  }
  if (significant_length == 0) return false;
  if (key && !bare && lexer->lookahead != '=') return false;
  // CR is trimmed at the end of a line, but remains data inside a value.
  const enum Token token = bare ? BARE_KEY : key ? KEY :
    (first == '"' && last == '"' && significant_length >= 2 ? QUOTED_VALUE : RAW_VALUE);
  if (!valid_symbols[token]) return false;
  lexer->result_symbol = token;
  return true;
}
