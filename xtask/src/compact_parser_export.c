#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// 维护时在 parser.c 后包含；固定小端编码，不依赖运行工具的主机字节序。
typedef struct {
  uint16_t value;
  uint16_t count;
  uint16_t first_symbol;
  bool nonterminal;
  int head;
  int tail;
} CompactGroup;

static void write_u16(FILE *file, uint16_t value) {
  unsigned char bytes[2] = {value & 255, value >> 8};
  if (fwrite(bytes, 1, 2, file) != 2) abort();
}

static void write_u32(FILE *file, uint32_t value) {
  unsigned char bytes[4] = {value & 255, (value >> 8) & 255, (value >> 16) & 255, value >> 24};
  if (fwrite(bytes, 1, 4, file) != 4) abort();
}

static int compare_groups(const void *left, const void *right) {
  const CompactGroup *a = left;
  const CompactGroup *b = right;
  if (a->count != b->count) return (int)a->count - (int)b->count;
  if (a->nonterminal != b->nonterminal) return (int)a->nonterminal - (int)b->nonterminal;
  if (a->value != b->value) return (int)a->value - (int)b->value;
  return (int)a->first_symbol - (int)b->first_symbol;
}

// 对所有 state/symbol 逐项校验导出的原生紧凑表，而非只比较压缩大小。
static uint16_t read_le16(const unsigned char *data) { return data[0] | ((uint16_t)data[1] << 8); }
static uint32_t read_le32(const unsigned char *data) { return read_le16(data) | ((uint32_t)read_le16(data + 2) << 16); }
static void validate(const char *path, uint32_t map_offset) {
  size_t size = map_offset + (STATE_COUNT - 2) * sizeof(uint32_t);
  unsigned char *data = malloc(size);
  uint16_t *expected = calloc(SYMBOL_COUNT, sizeof(uint16_t));
  uint16_t *actual = calloc(SYMBOL_COUNT, sizeof(uint16_t));
  FILE *file = fopen(path, "rb");
  if (!data || !expected || !actual || !file || fread(data, 1, size, file) != size) abort();
  fclose(file);
  for (uint32_t state = 2; state < STATE_COUNT; state++) {
    memset(expected, 0, SYMBOL_COUNT * sizeof(uint16_t));
    memset(actual, 0, SYMBOL_COUNT * sizeof(uint16_t));
    if (state < LARGE_STATE_COUNT) memcpy(expected, ts_parse_table[state], SYMBOL_COUNT * sizeof(uint16_t));
    else {
      const uint16_t *row = ts_small_parse_table + ts_small_parse_table_map[state - LARGE_STATE_COUNT];
      uint16_t groups = *row++;
      while (groups--) {
        uint16_t value = *row++, count = *row++;
        while (count--) expected[*row++] = value;
      }
    }
    const unsigned char *row = data + 2 * read_le32(data + map_offset + 4 * (state - 2));
    uint16_t groups = read_le16(row); row += 2;
    while (groups--) {
      uint16_t value = read_le16(row), count = read_le16(row + 2); row += 4;
      while (count--) { actual[read_le16(row)] = value; row += 2; }
    }
    if (memcmp(expected, actual, SYMBOL_COUNT * sizeof(uint16_t))) abort();
  }
  free(data); free(expected); free(actual);
}

int main(int argc, char **argv) {
  if (argc != 3 || STATE_COUNT < 2 || LARGE_STATE_COUNT < 2) return 2;
  FILE *direct = fopen(argv[1], "wb");
  FILE *compact = fopen(argv[2], "wb");
  if (!direct || !compact) return 3;
  for (uint32_t state = 0; state < 2; state++)
    for (uint32_t symbol = 0; symbol < SYMBOL_COUNT; symbol++) write_u16(direct, ts_parse_table[state][symbol]);

  CompactGroup *groups = calloc(SYMBOL_COUNT, sizeof(*groups));
  int *next = malloc(SYMBOL_COUNT * sizeof(*next));
  int *terminal_index = malloc((UINT16_MAX + 1u) * sizeof(*terminal_index));
  int *nonterminal_index = malloc((UINT16_MAX + 1u) * sizeof(*nonterminal_index));
  uint32_t *map = calloc(STATE_COUNT - 2, sizeof(*map));
  if (!groups || !next || !terminal_index || !nonterminal_index || !map) abort();
  for (size_t i = 0; i <= UINT16_MAX; i++) terminal_index[i] = nonterminal_index[i] = -1;

  uint32_t offset = 0;
  for (uint32_t state = 2; state < LARGE_STATE_COUNT; state++) {
    map[state - 2] = offset;
    uint16_t group_count = 0;
    for (uint32_t symbol = 0; symbol < SYMBOL_COUNT; symbol++) {
      uint16_t value = ts_parse_table[state][symbol];
      if (!value) continue;
      bool nonterminal = symbol >= TOKEN_COUNT;
      int *index = nonterminal ? &nonterminal_index[value] : &terminal_index[value];
      if (*index < 0) {
        *index = group_count++;
        groups[*index] = (CompactGroup){.value = value, .first_symbol = symbol,
                                        .nonterminal = nonterminal, .head = -1, .tail = -1};
      }
      CompactGroup *group = &groups[*index];
      next[symbol] = -1;
      if (group->tail >= 0) next[group->tail] = symbol;
      else group->head = symbol;
      group->tail = symbol;
      group->count++;
    }
    for (uint32_t i = 0; i < group_count; i++) {
      if (groups[i].nonterminal) nonterminal_index[groups[i].value] = -1;
      else terminal_index[groups[i].value] = -1;
    }
    qsort(groups, group_count, sizeof(*groups), compare_groups);
    write_u16(compact, group_count);
    offset++;
    for (uint32_t i = 0; i < group_count; i++) {
      CompactGroup *group = &groups[i];
      write_u16(compact, group->value);
      write_u16(compact, group->count);
      offset += 2;
      for (int symbol = group->head; symbol >= 0; symbol = next[symbol]) {
        write_u16(compact, symbol);
        offset++;
      }
    }
  }

  for (size_t i = 0; i < sizeof(ts_small_parse_table) / sizeof(uint16_t); i++) write_u16(compact, ts_small_parse_table[i]);
  uint32_t map_offset = (offset * 2 + sizeof(ts_small_parse_table) + 3) & ~UINT32_C(3);
  uint8_t padding[3] = {0};
  uint32_t padding_size = map_offset - (offset * 2 + sizeof(ts_small_parse_table));
  if (fwrite(padding, 1, padding_size, compact) != padding_size) return 6;
  for (uint32_t state = 2; state < LARGE_STATE_COUNT; state++) write_u32(compact, map[state - 2]);
  for (uint32_t state = LARGE_STATE_COUNT; state < STATE_COUNT; state++) {
    write_u32(compact, offset + ts_small_parse_table_map[state - LARGE_STATE_COUNT]);
  }
  if (fclose(direct) || fclose(compact)) return 7;
  validate(argv[2], map_offset);
  printf("%u %zu\n", map_offset, sizeof(ts_parse_table) + sizeof(ts_small_parse_table) + sizeof(ts_small_parse_table_map));
  return 0;
}
