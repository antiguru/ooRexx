# Phase 5g-D — the divergences Task 9 confirmed and did not fix

Base: `5ac6f69e7`. Source: the "Confirmed rather than changed" section of
`docs/superpowers/records/2026-09-06-phase-5g-collections/task-9-review-findings.md`,
each row re-measured against the oracle and against this crate's `5ac6f69e7`
binary before being planned.

**One row on that list no longer diverges.** `Queue~put`'s argument position
in 93.907 was restored by Task 9 itself. Re-measured at five shapes --
`put('Y',2)`, `put('Y',5)`, `put('Y',99)`, `put('m',-1)` and `put('m')` --
oracle and crate agree line for line: 93.966, 93.966, 93.918, `93.907 ...
argument 2 ... found "-1"`, 93.903. It is struck from the list rather than
worked, and the re-measurement is the evidence.

Twelve remain, grouped into seven tasks. Every "oracle" line below was run;
every "crate" line is `target/release/rexx-run` at `5ac6f69e7`.

---

## D1 -- a `List` index is converted, not compared

Oracle: `l = .List~of('a','b','c')` and every one of `l~at(' 1')`,
`~at('01')`, `~at(1.0)`, `~at('+1')`, `~at(1e0)`, `~at(' 1 ')` answers `b`.
Crate: all six answer `The NIL object`, because `list_position` compares the
argument OBJECT to the stored handle with `==`.

`ListClass::validateIndex` (`classes/ListClass.cpp:195`) is
`index->unsignedNumberValue(item_index, Numerics::ARGUMENT_DIGITS)`, and
`ARGUMENT_DIGITS` is 18 on this build (`runtime/Numerics.hpp:90`). A value
that will not convert raises **93.918**; one that converts but names no live
entry answers the NoLink path, which is `.nil` for `at` and 0 for `hasIndex`.

Measured boundaries: `hasIndex('1e1')` is `0` (converts to 10, absent),
`hasIndex('1000000000')` is `0`, `hasIndex('-0')` is `1`, `hasIndex('  3  ')`
is `0`; `at('abc')`, `at(-1)`, `hasIndex(1.5)`, `hasIndex('1e300')` and
`hasIndex('')` are all 93.918 naming the argument as written.

Work: an `unsigned_index` beside `whole_index` in `dispatch.rs` -- the same
`rexx_num::ARGUMENT_DIGITS` conversion, admitting 0 -- and `list_position`
rebuilt on it.

The requiredIndex paths already agree and are not touched: `insert('z',99)`,
`put('z',99)` and `section(99)` are 93.918 on both, and `index`/`hasItem`
agree.

## D2 -- an entry that holds nothing is still an entry

`l~put(, 1)` leaves entry 1 holding nothing. Oracle: `items` 3, `allIndexes`
`0,1,2`, `at(1)` `.nil`, `hasIndex(1)` `0`, `next(0)` `1`, `previous(2)` `1`,
`next(1)` and `previous(1)` `.nil`, `firstItem` over an empty first entry
`.nil`, `section(0,3)` 3 items, `section(0,2)` 2, `section(1,1)` 93.918.

Crate: `list_pairs` is a `filter_map` that drops any entry whose item is
`None`, so the count is 2 and every walk is off by one per empty entry.

Work: `list_pairs` answers `Vec<(ObjRef, Option<ObjRef>)>`; `items`,
`allIndexes`, `isEmpty`, the four end methods, `next`/`previous`, `section`,
`allItems` and `makeArray` follow it; `hasIndex` is false for an entry that
holds nothing while the chain still walks through it.

**A residue this task does not close, and says so.** `l~allItems~items` is 3
on the oracle against an array whose slot 2 is unoccupied -- `hasIndex(2)` on
that array is `0`. `ListContents::allItems` appends `OREF_NULL` for the empty
entry (`classes/support/ListContents.cpp:672`) and
`ArrayClass::setArrayItem` increments `itemCount` whenever the slot was not
occupied, without looking at the value (`classes/ArrayClass.cpp:512` and
`ArrayClass.hpp:267`). This crate derives an array's item count from its
occupancy, so it answers 2. It is an upstream counter that disagrees with the
array's own `hasIndex`, reachable only through a `List` holding an empty
entry; it is named here and left, not silently absorbed.

## D3 -- `empty` re-issues handles from zero, and `section` answers a `List`

Oracle: `.List~of('a','b','c')`, `~empty`, then four appends answer `0 1 2 3`.
Crate answers `2 1 0 3`, because `native_list_empty` takes the entries one at
a time and each `list_take` pushes its handle onto the free stack. The
free-stack path itself is right and stays: `remove(1)`, `remove(0)`, then
three appends answer `0 1 3` on both.

Oracle: `.MyList~of('a','b','c')~section(0,2)~class~id` is `List`, where the
same question of a `Queue` subclass answers `MYQ` and of an `Array` subclass
answers `MYARR`. `ListClass::section` is `new ListClass`
(`classes/ListClass.cpp:429`), an unconditional plain list, while the other
two build from the receiver's class. Crate answers `MYLIST`.

## D4 -- `Supplier` is bounded by its items, and converts its arguments

`SupplierClass::available`, `next`, `item` and `index` all test `position >
items->size()` (`classes/SupplierClass.cpp:180`, `:217`, `:254`) -- the
INDEXES array bounds nothing. Oracle, over items `('a','b')` and indexes
`(1)`: after one `next`, `available` is `1`, `item` is `b`, and `index` is
`.nil`. Crate: `available` is `0` and `index` raises 93.937, because
`supplier_remaining` takes `items.min(indexes)`.

`SupplierClass::initRexx` is `arrayArgument`, which is `requestArray()` --
a `MAKEARRAY` send for anything that is not already an array
(`runtime/MethodArguments.hpp:683`). Oracle: `.Supplier~new(.List~of('a'),
.List~of('h'))` answers `1 a h`. Crate refuses: `method "MAKEARRAY" of class
"List" is not implemented (Phase 5)`, because `array_argument` reads slots
directly and has no conversion step.

Work: drop the `min`; `index` past the end of the indexes array answers
`.nil`; `array_argument` gains the `MAKEARRAY` send. That send is why the
refusal existed -- it was written when no receiver answered `MAKEARRAY` --
and it can only replace a Loud refusal with an answer.

## D5 -- construction: `of`, a subclass `INIT` that does not forward, `INIT` twice

Oracle, with an `Array`, a `List` and a `Queue` subclass each overriding
`INIT`, `APPEND` and `PUT`: `.Watch~of('x','y')` prints `Watch INIT 0` and
nothing else. `OF` sends `INIT` with no arguments and then fills the
collection without sending anything. Crate: the `Array` subclass is right,
and the `List` and `Queue` rows send `APPEND` per item.

Oracle: a `Queue` subclass whose `INIT` does not forward still works --
`mq~queue('a')` then `mq~items` is `1`; and `q~init` on a `Queue` already
holding two items leaves `items` at 2. Crate: the first refuses with `a
message send to a value that is not an array`, and the second answers `0`.

Work: the store is created on demand rather than only by `INIT`; `INIT` on a
receiver that already has one leaves it alone; `OF` fills natively.

## D6 -- the sort

Two rows.

`WithSortComparator::compare` (`classes/ArrayClass.cpp:2898`) takes the
COMPARE result through `numberValue`, raising **26.903** for one that is not
a whole number and **91.999** for a method that answers nothing. Oracle: a
comparator answering `'-1.0'`, `'1.0'`, `'0.0'` sorts `b,a,c` into `a,b,c`;
one answering `'abc'` is `Error 26.903: Result of a COMPARE method call did
not result in a whole number; found "abc".`; one that returns no value is
`Error 91.999: Message "COMPARE" did not return a result.` Crate: the first
leaves the array unsorted at `b,a,c` and the second runs to completion.

The comparison **sequence** is upstream's merge sort with an insertion sort
below eleven elements and an exponential-search merge above it
(`classes/ArrayClass.cpp:2619`, `:2669`, `:2773`). Measured, `.Array~of(3,1,2)`
compares `1 3`, `2 3`, `2 1`; this crate compares `1 2`, `3 1`, `3 2`. Nine
elements gives twenty-three comparisons in a fixed order, and that order is
observable through any comparator with an effect.

Work: port `mergeSort`, `merge` and `find` as upstream writes them, and take
the result through the crate's whole-number conversion.

## D7 -- `dimensions` after `append` on a zero-length array

Oracle: `.Array~new(0)~append('q')` then `~dimensions~makeString('L',',')` is
`1`, and `.Array~of()` behaves the same. Crate answers `0`: the fixed
dimension list is not updated when the append grows the array.

---

## Gates

Fast checks per task: `cargo fmt --all --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test --release --workspace
--no-fail-fast`, and the same suite under `REXX_CORPUS_GATE=1`. The seven
gates run over D4 and again over D7.
