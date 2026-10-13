#include "tree_sitter/array.h"
typedef struct { unsigned value; } Item;
int main(void) {
  Array(Item) items = array_new();
  for (unsigned i = 0; i < 1024; ++i) {
    Item item = {i};
    array_push(&items, item);
  }
  array_reserve(&items, 2048);
  for (unsigned i = 0; i < 1024; ++i) assert(array_get(&items, i)->value == i);
  Item tail[] = {{1024}, {1025}};
  array_extend(&items, 2, tail);
  assert(items.size == 1026);
  array_erase(&items, 0);
  assert(array_front(&items)->value == 1);
  assert(array_back(&items)->value == 1025);
  array_delete(&items);
  assert(items.contents == NULL && items.size == 0 && items.capacity == 0);
  return 0;
}
