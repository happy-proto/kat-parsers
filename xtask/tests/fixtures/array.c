#include "tree_sitter/array.h"
typedef struct { unsigned value; } Item;
typedef Array(Item) Items;
static unsigned delete_calls;
static Items *counted(Items *items) { delete_calls++; return items; }
int main(void) {
  Items items = array_new();
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
  array_delete(counted(&items));
  assert(delete_calls == 1);
  assert(items.contents == NULL && items.size == 0 && items.capacity == 0);
  typedef Array(Items) Stack;
  Stack stack = array_new();
  Items nested = array_new();
  Item item = {1};
  array_push(&nested, item);
  array_push(&stack, nested);
  array_delete(&array_pop(&stack));
  assert(stack.size == 0);
  array_delete(&stack);
  return 0;
}
