# Task 6.7 report: collections

Clusters `identity-collection-equality`, `collection-set-operations`,
`collection-argument-error-numbers`, `array-array-index`, `queue-removeitem`, `nil-comparison`
and `bag-hasitem-nil`. Every test of these clusters passes except the Bag test, which is
licensed under Deviation 45, and IdentityTable `TEST_AT01`, `TEST_[]` and `TEST_HASINDEX`,
which stay red under Deviation 4 (Concerns). The whole suite changes in exactly the rows of the
tests that pass and the licensed one.

## Commits

* `599f663ae` Phase 9d Task 6.7: collections. The fixes, the four witnesses with their
  `sourceline_oracle/` files and `phase-9.txt` lines, the licence row, and the closed tests out
  of `suite-9d-set.txt`.
* `b55626842` a comment-only correction in `eval.rs` (the concatenation with `.nil` reads the
  string value, which raises a trapped NOSTRING). The checks below ran on `599f663ae`; `cargo fmt
  --all --check` was rerun on `b55626842`, exit 0.
* The records commit that follows holds `docs/superpowers/records/2026-10-18-phase-9d/t6.7/` and
  this report.

## Root causes and fixes

### identity-collection-equality

`hash.rs`'s `hasItem`, `index` and `removeItem` compared items with `==` for every hash
collection. The oracle's `HashContents::isItemEqual` (`HashContents.hpp:190`) is identity, and
only `EqualityHashContents` (`:434`, which `StringHashContents` inherits) uses `equalValue`. So
an `IdentityTable` holding a `.File` found an equal `.File` (`TEST_BUG_1468`). Fix:
`same_held_item(keys, wanted, item)`, identity under `Keys::Identity` and `same_item` otherwise,
in those three methods.

`TEST_HASITEM` and `TEST_INDEX` now pass too: their unequal probe is `"1223"~substr(2,1)`
against the integer 2, which is a different handle here.

`TEST_AT01`, `TEST_[]` and `TEST_HASINDEX` stay red. Each fails at an assertion that two
separately built equal two-byte strings are different objects (`coll~at("2"||"2")`,
`coll["2"||"2"]`, `hasIndex("1223"~substr(2,2))`). Deviation 4 names exactly these assertions
as its L2 cost. `t6.7/probes/identity/identity_rest.rex` runs every other assertion of those
tests on both sides, and each holds; only the Deviation 4 questions answer differently. No licence row is written: see Concerns.

### queue-removeitem and collection-set-operations

One cause. `Array~removeItem` cleared the slot. The oracle's `ArrayClass::removeItem` calls the
virtual `remove`, which `QueueClass::remove` (`QueueClass.cpp:222`) redefines as `deleteItem`, so
on a Queue the items after it close up. Queue `TEST_16` read a hole at index 2. The Rexx
`difference` and `xor` remove items from a copy of the receiver, so their Queue and
CircularQueue answers had indexes `1 4`, `5 6 7`, and so on, and `isCollectionEqual` compares
`allIndexes`. Fix: `native_array_remove_item` splices the slot out when `is_queue(receiver)`.
Measured before the fix (`t6.7/probes/setops/`, oracle and parent outputs): every Queue and
CircularQueue difference and xor matched the oracle in items and differed only in indexes;
intersection and union already matched. The task's output is identical to the oracle's.

### array-array-index and collection-argument-error-numbers

`Array~insert` and `Array~section` read their index with `positive_index`, so an Array index was
93.907. `ArrayClass::validateIndex` (`ArrayClass.cpp:1211`) spreads a single Array argument into
its subscripts. Fix: `single_subscript` in `array/surface.rs`:
* not an Array: `positive_index` as before;
* the first `items` slots of the Array, as `validateIndex` takes them: none is 93.901 with the
  argument's position, one is that item read by `positive_index`, a hole goes through
  `position_in`'s existing refusal, and more than one on a fixed-dimension receiver is 93.926;
* a Queue: `QueueClass::validateIndex` (`QueueClass.cpp:98`) does not spread, so the Array is
  read as `requestString` reads it, its items' lines. A single positive item converts, and
  anything else is 93.907 found "an Array". Measured on the oracle: `insert('a', .array~of(1))`
  works on a Queue, and `.array~new`, `.array~of('x')` and `.array~of(1,2,3)` are 93.907.

`section`'s count is now `optional_length_argument` (`ArrayClass.cpp:1512`), 93.923 for `.nil`,
`.string`, `-1`, `1.5` and `'x'`. It was `array_size_argument`'s 93.906.

`Queue~insert` checked the index with `queue_bound`, whose first tier (past the extent) is
`putRexx`'s 93.918. `insertRexx` validates under `IndexUpdate`, which extends instead of
refusing, and then `checkInsertIndex` raises 93.966. Fix: insert raises 93.966 for any position
past the last item, and no longer calls `queue_bound`.

### nil-comparison

`apply_binary` answered the six equality operators against `.nil` without reading it, and sent
every other comparison through `required_string_operand`, which raises NOSTRING for `.nil`. The
oracle's `String`, `Integer` and `NumberString` comparison methods all answer before reading
(`CompareOperator`, `StringClass.cpp:1248`; `IntegerClass.cpp:1237`ff;
`NumberStringClass.cpp:3338`ff): `.false`, except `\=`, `<>`, `><` and `\==`. Fix: the early
answer covers `is_comparison(op)` rather than `is_equality(op)`; `is_inequality` already names the
four true ones. A concatenation with `.nil` still reads its string value, and raises a trapped
NOSTRING on both sides.

### bag-hasitem-nil

Licence row in `rust/corpus/suite-licences.txt`:
`base/class/Bag	TEST_HASITEM_VALUE_NOTEQUAL_INDEX	Deviation 45	the test's one assertion
expects the oracle's .nil from hasItem with an index unequal to the item`. The test's body is
that one assertion (`Bag.testGroup:833`-`834`), so no other assertion is left to show passing.
The suite reports the row as `Deviation(45)` and the Bag group green.

## Witnesses, red then green

Corpus programs in `rust/corpus/phase-9.txt`, each with its `sourceline_oracle/` file from that
test's oracle driver. Parent is `01db1850f` built from `git archive` (with `interpreter/` and
`extensions/`, which `rexx-lib`'s build script reads). `t6.7/witnesses/summary.tsv`, run by
`t6.7/witnesses/run3.sh`:

| program | oracle | parent | t6.7 |
|---|---|---|---|
| `collection_identity_table_items` | rc 0 | rc 0, stdout differs | identical |
| `collection_queue_remove_item` | rc 0 | rc 0, stdout differs | identical |
| `collection_array_index_argument` | rc 0 | rc 163, 93.907 at line 10 | identical |
| `string_compare_nil` | rc 0 | rc 0, NOSTRING at line 12 | identical |

Read stdout, each path printing:
* `collection_identity_table_items`: hasItem, index and removeItem on an IdentityTable answer
  `0`, `.nil`, `.nil` for an equal K, and a Table answers `1`, `x`, the item. Two equal items
  under two indexes stay distinct.
* `collection_queue_remove_item`: indexes `1 2 3` after removeItem, `at 4` is `.nil`, a missing
  item answers `.nil`, a CircularQueue closes up, and difference, reversed difference, xor and
  intersection on Queue and CircularQueue each print contiguous indexes. Items print sorted.
* `collection_array_index_argument`: the Array-index successes on Array and Queue, then one
  `try` line per refusal, each printing its code: 93.926, 93.901 and 93.907 for the Array forms,
  93.923 for each count, 93.966 for each queue insert, 93.907 for the queue Array forms.
* `string_compare_nil`: every comparison operator against `.nil` for the `.nil` string, a word,
  an integer, a decimal, the null string and a String subclass instance, then NOSTRING at the
  concatenation.

## The tests

`suite-9d-set.txt` loses these lines, and each passes or is licensed in the whole-suite run:
`base/class/Array` `TESTINSERT`, `TESTMULTIINDEXINSERT`, `TEST_SECTION`, `TEST_SECTION_SYNTAX5`,
`TEST_SECTION_SYNTAX6`; `base/class/Bag TEST_HASITEM_VALUE_NOTEQUAL_INDEX` (licensed);
`base/class/CircularQueue TEST_DIFFERENCE`; `base/class/IdentityTable` `TEST_BUG_1468`,
`TEST_HASITEM`, `TEST_INDEX`; `base/class/Queue` `TEST_DIFFERENCE`, `TEST_XOR`;
`base/class/QueueRGF` `TEST_DIFFERENCE`, `TEST_INSERT_ERROR2`, `TEST_XOR`;
`base/class/String/comparisonOperators TEST_NIL`; `base/class/collections/queue TEST_16`.

It keeps `base/class/IdentityTable` `TEST_AT01`, `TEST_HASINDEX` and `TEST_[]`.

No queued item closes: `queued-inventory.tsv` holds none of these clusters' items (its
`2026-10-18-upstream-bag-hasitem.md` row is already `not-a-divergence`). No exclusion row is
touched.

## Changed rows

Against `t6.6/r2/suite/tests.tsv`, by the command at the top of `t6.7/suite/changed-rows.txt`:
exactly the tests above. Each but the Bag test goes `failed` to `passed` with no row, and the
Bag test keeps `failed` with its row now `Deviation(45)`. No other row changed, and the
`SysFileTree TEST_TREE_DIRECTORY` flake did not show. Suite: 402 groups, 17 red (24 at 6.6 r2).

## Checks

From `rust/`, `CARGO_TARGET_DIR=<scratchpad>/t67-target`, `CARGO_INCREMENTAL=0`, each exit
status read unpiped, on `599f663ae`:

* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-exec --lib`: exit 0, 1152 passed.
* `memcap 16G cargo test -j 4 --release -p rexx-exec --test refusal_sites --test owners --test
  closed_phases --test phase_ten_reasons --test representation_refusals --test
  refusal_dispositions --test coverage --test rexxc`: exit 0, no failures. No `Loud` site was
  added or removed, so `refusal-sites.tsv` is unchanged.
* `memcap 16G cargo test -j 4 --release -p rexx-parse`: exit 0 (includes `sourceline_oracle`
  and `program`).
* `memcap 16G cargo test -j 4 --release -p rexx-core --test unsafe_sites`: exit 0, 2 passed.
* `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus`: exit 0,
  1091 of 1091 matching.
* `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=docs/superpowers/records/2026-10-18-phase-9d/t6.7/suite/table.tsv
  memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite`: exit 101 as
  expected, 402 groups, 17 red, 4m40s.

## Perf

`t6.7/callgrind.txt`: `bench-programs/callgrind.sh -B 0.5 -p "startup interpret parse churn
sayloop" -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.7-run pin=.../rexx-run
parent=.../cg-9d/t6.6-r2/rexx-head head=.../cg-9d/t6.7/rexx-head`, exit 0, 3 rounds. The parent
binary is `c5b40df87`, whose code `01db1850f` carries unchanged. Own delta is this figure against
the pin minus `t6.6/r2/callgrind.txt`'s head figure, the last perf-checked task.

| program | vs pin | t6.6 r2 vs pin | own delta (ledger) | vs parent, same run |
|---|---|---|---|---|
| startup | +0.0419 | +0.0434 | -0.0015 | -0.0014 |
| interpret | -2.1629 | -2.1633 | +0.0004 | +0.0005 |
| parse | -0.0861 | -0.0860 | -0.0001 | -0.0001 |
| churn | +0.1111 | +0.1083 | +0.0028 | +0.0028 |
| sayloop | +0.0457 | +0.0473 | -0.0016 | -0.0014 |

Every own delta is under +0.05%, and every figure against the pin is under +0.5%. The
"vs parent" column is head d% minus parent d% in the same run. No error construction was added
outside the existing `Raised` constructors.

## Concerns

* **IdentityTable `TEST_AT01`, `TEST_[]` and `TEST_HASINDEX` need a ruling.** Their failing
  assertions are the ones Deviation 4 names ("THE L2 COST", `phase-4-exclusions.txt`), and every
  other assertion passes (`t6.7/probes/identity/`). `suite-licences.txt` says a Deviation row is
  added only by Moritz's ruling, and the brief rules only Deviation 45's row, so the three stay in
  `suite-9d-set.txt` unlicensed. Proposed: three `Deviation 4` rows, reason "asserts that two
  separately built equal strings are different IdentityTable indexes".
* **A failed `Queue~insert` extends the queue on the oracle.** `insertRexx` validates under
  `IndexUpdate`, which extends the array to the position before `checkInsertIndex` refuses it.
  Measured (`t6.7/probes/queue-insert-extends/`): after a failed `insert('a', 99)` on an empty
  queue, `put('m', 50)` is 93.966 on the oracle and 93.918 here, where a fresh queue is 93.918 on
  both. A refused insert changing its receiver looks like an oracle defect, so it is not copied
  and not witnessed. It needs a ruling.
* **An Array index on an empty, unfixed Array.** `.Array~new~insert('a', .array~of(1,2))` answers
  3 on the oracle and makes the receiver two-dimensional, a shape `insert` itself refuses, and
  `.Array~new~section(.array~of(1,2))` reads a start `validateIndex` never set. This crate keeps
  93.907 for both (`t6.7/probes/index-outside-cluster/q2.diff`, its last two lines). An oracle
  defect by the same reading; it needs a ruling.
* **Queue `put`, `at`, `delete` and `section` with an Array index still differ**, outside these
  clusters' tests and unchanged by this task: `put('a', .array~of(9))` is 93.966 on the oracle
  and 93.907 here, `put('a', .array~of(20))` 93.918 there and 93.907 here, `at(.array~of(1,2))` and `delete(.array~of(1,2))` are 93.907 there and 93.926
  here, `at(.array~new)` 93.907 there and 93.901 here (`q2.diff`). The cause is the one
  `single_subscript` handles for `insert` and `section` (a queue does not spread an Array), but
  `put`, `at` and `delete` validate through `array_position`, which sees the store and not the
  queue. Proposed as a queued item.
* In `t6.7/probes/identity/`, this crate finds a second `"2"||"2"` as the IdentityTable index
  the first one made, while keeping both apart from the literal `"22"`. That is inside Deviation 4's unspecified answers; the mechanism was not traced.
