# Task 6 report: `rexx-exec/src/dispatch/*` over the limit

BASE `ed8cb3f03`. There are fourteen code commits, then this report's
commit, then the gate results. Every artifact cited is under
`docs/superpowers/records/2026-09-15-file-split/task-6-files/` (`files/`
below). The tooling is Task 5's, adapted, in `files/tools/`.

## Commits

| # | commit | what moved | from, into |
| --- | --- | --- | --- |
| c1 | `464dbe90f` | `library.rs`'s test module | `dispatch/library.rs` to `dispatch/library/tests.rs` |
| c2 | `168b0caba` | `native.rs`'s test module | `dispatch/native.rs` to `dispatch/native/tests.rs` |
| c3 | `69816b613` | `Package~local`, `Package~name` | `dispatch.rs` to `dispatch/package.rs` |
| c4 | `5f4a421e7` | the two separator entry points | `dispatch.rs` to `dispatch/files.rs` |
| c5 | `fdc3025db` | `String`'s `sign`, `makeString`, `upper`, `length`, `makeArray`, `reverse` | `dispatch.rs` to `dispatch/string.rs` |
| c6 | `96e2ebb5a` | `Directory`/`StringTable`'s `at`/`put`/`unknown` and `hash_index` | `dispatch.rs` to `dispatch/hash.rs` |
| c7 | `bace8f2f8` | the method-argument parsers and `METHOD_ARGUMENT_DIGITS` | `dispatch/buffer.rs` to `dispatch/method_arguments.rs` |
| c8 | `4488f00a6` | `Stem`'s natives, its tail tree, its rows | `dispatch/hash.rs` to `dispatch/hash/stem.rs` |
| c9 | `1cf10137a` | `Relation`'s and `Bag`'s natives and rows | `dispatch/hash.rs` to `dispatch/hash/relation.rs` |
| c10 | `dfe5cf4e7` | `List`'s natives and rows | `dispatch/collection.rs` to `dispatch/collection/list.rs` |
| c11 | `fd4f6e3a2` | `Queue`'s natives and rows | `dispatch/collection.rs` to `dispatch/collection/queue.rs` |
| c12 | `406a1a2b7` | `Supplier`'s natives, state names and rows | `dispatch/collection.rs` to `dispatch/collection/supplier.rs` |
| c13 | `86a31607a` | `Array`'s sort family and its rows | `dispatch/collection.rs` to `dispatch/array/sort.rs` |
| c14 | `ddeeb5640` | the rest of `collection.rs`'s `Array` natives and rows, and the emptied table | `dispatch/collection.rs` to `dispatch/array/surface.rs` |

The order follows the brief: tests-only moves first (c1, c2), then the
residue leaves from `dispatch.rs` (c3 to c6), then `buffer.rs`, `hash.rs`
and `collection.rs`. Each commit message says `git blame -w -C -C -C`
recovers the moved lines.

Line counts use the plan's `find crates -name '*.rs' | xargs wc -l`:

| file | BASE | final |
| --- | --- | --- |
| `dispatch.rs` | 3166 | 2881 |
| `dispatch/hash.rs` | 2992 | 2133 |
| `dispatch/collection.rs` | 2279 | 425 |
| `dispatch/buffer.rs` | 2206 | 1877 |
| `dispatch/library.rs` | 1391 | 776 |
| `dispatch/native.rs` | 1046 | 704 |
| `dispatch/string.rs` | 2222 | 2364 |
| `dispatch/package.rs` | 1032 | 1063 |
| `dispatch/files.rs` | 565 | 585 |
| `dispatch/array.rs` | 702 | 712 |
| new: `hash/stem.rs`, `hash/relation.rs` | | 719, 309 |
| new: `collection/list.rs`, `queue.rs`, `supplier.rs` | | 662, 208, 197 |
| new: `array/sort.rs`, `array/surface.rs` | | 339, 612 |
| new: `method_arguments.rs` | | 352 |
| new: `library/tests.rs`, `native/tests.rs` | | 625, 349 |

## Where things went, and why

* **`Array`'s half of `collection.rs` went under `dispatch/array/`**, as
  `array/sort.rs` (c13) and `array/surface.rs` (c14), not to
  `collection/array.rs`. That leaves `Array`'s methods in one place:
  `dispatch/array.rs` and its children. The alternative would have left
  them in two places, `dispatch/array.rs` and `collection/array.rs`. The
  Array-shaped store the moved bodies work on stays in `collection.rs`
  (`store_of`, `slots_of`, `append_slot`, `queue_bound`, the splice
  helpers), because `Queue`'s and `List`'s bodies use the same store. The
  cost is in visibility: see Departure 2.
* **`collection.rs` now holds the shared contents protocol**, meaning the
  Array-shaped store, `same_item`, `item_argument`, `array_of`, and
  `native_collection_of` (`Queue~of`/`List~of`). It also declares its three
  children and has no native table of its own.
* **`hash.rs`** keeps the store, `Directory`/`StringTable`, the shared
  `Table`/`IdentityTable`/`Set` surface, and three class-side constructors,
  as the ruling says. `native_bag_of` stays with `native_set_of` and
  `native_hash_new`, the constructors `dispatch.rs`'s `NATIVE_CLASS_METHODS`
  names as `hash::`. Moving it into `hash/relation.rs` would have needed a
  re-export for that path. The `Relation and Bag's own surface` heading
  stays above `native_bag_of`, which is `Bag`'s own. The `Stem` heading had
  nothing left under it in `hash.rs`, so it moved with `Stem`.
* **`buffer.rs`**: `method_arguments.rs` takes the whole set Task 2 listed,
  which is `optional_length_argument` through `named_pad_argument`,
  `whole_method_argument`, `refuse_method_argument`, `usize_or_refuse` and
  `METHOD_ARGUMENT_DIGITS`. The module is named after
  `runtime/MethodArguments.hpp`, which every one of those functions cites.
  The per-method argument sets that `String` shares with `MutableBuffer`
  (`substr_arguments` and the rest) stay in `buffer.rs` beside the
  `MutableBuffer` bodies that use them.
* **`dispatch.rs`'s residue.** The rows Task 2 kept in `NATIVE_METHODS`
  stay there. Only the functions moved. Each is widened to `pub(super)` in
  its new file and imported into `dispatch.rs`, so every row and every
  `super::` path elsewhere is unedited.
  * The separator entry points (c4) went to `files.rs`. They are `.File`'s
    `LIBRARY REXX` entry points (`Family::File`,
    `streamLibrary/FileNative.cpp`), and `files.rs` holds that family. They
    sit ahead of `case_sensitive`, in native.rs's table order.
  * **`VariableReference`'s natives stay in `dispatch.rs`**: no child
    serves `VariableReference`, and the ruling does not create one.
* **`string.rs` is not split.** It holds one class's primitive methods and
  their table. The operator block (`BINARY_OPERATORS` and the
  `native_string_op_*` bodies) is part of that same set of methods.
* **`stream.rs` is not split.** It holds `.Stream`'s `LIBRARY REXX` entry
  points around one state block. The open, read/write and positioning
  groups all call the same state helpers (`require`, `ensure_open`,
  `standard_*`), so cutting them apart would scatter those helpers.
* **`package.rs` is not split.** It holds `Package`'s readers and writes,
  one class. It also gained `Package~local`/`~name` (c3).
* **`library.rs`, `native.rs`**: tests only. `LIBRARY_REXX_METHODS` stays
  beside its `const fn` row builders.
* `string.rs`'s own `#[cfg(test)]` block did not move.

## Chained native tables

Every commit that moved rows or natives, c3 to c14, has a `natives_dump.py`
verdict identical to BASE's: 536 entries (`files/c<N>/c<N>-natives-verdict.txt`,
BASE in `files/base/natives-base.txt`). The dump comes from a debug build of
the tree that instrument 4 ran on, in the test worktree, with the patch
applied and then removed.

Every table this task moved rows out of was split into contiguous blocks.
`build` chains each block where its rows stood, so the sequence of rows
`build` iterates is unchanged, with one exception: `hash.rs`. Its table ran
`Table`, `IdentityTable`, `Set`, `Relation`, `Directory`, `Stem`, `Bag`. It
is now chained as `hash` (`Table`, `IdentityTable`, `Set`, `Directory`),
then `hash::relation` (`Relation`, `Bag`), then `hash::stem`. The
natives dump is identical, so no reordered row collides. The dump can see
a reorder when one does collide: chaining `collection::queue` ahead of
`dispatch.rs`'s own table resolves `Queue`'s `AT`/`[]`/`ITEMS` ids to
`native_array_*` (`files/final/natives-control-verdict.txt`).

Table headers. item-tool gives neither a header nor a `];` a unit of its
own. Every new slice's header is the one its rows were cut from, token for
token, apart from name and visibility. `files/c<N>/c<N>-table-headers.txt`
shows this for c8 to c14: the command, and the header before and after.
`files/final/pins.txt` lists every table header under `src/dispatch` at
`ddeeb5640`. All the new slices are `const`, like the `hash.rs` and
`collection.rs` tables they came from. c14 removed `collection.rs`'s
emptied table: its header, its two doc lines, its `];` and the blank line
before it, each a declared deletion (`--expect-gone`).

`dispatch/tests.rs`'s
`an_unimplemented_method_is_loud_where_an_unknown_one_is_a_condition`
chained `collection::NATIVE_METHODS`. At c14 that line became the five
slices its rows went to, checked by `other_edits.py` (`files/c14/rules`).
The test's name did not change.

## Pinned items

* `mod seam`, the two `method_is_protected(` occurrences and the one
  `seam::clear(` are in `src/dispatch.rs` at every commit
  (`files/final/pins.txt`). `dispatch_seam` passed in instrument 4 at
  every commit.
* **`CLEARANCE_CONSUMERS` gained seven entries**:
  * `hash/stem.rs` (c8), `hash/relation.rs` (c9);
  * `collection/list.rs` (c10), `collection/queue.rs` (c11),
    `collection/supplier.rs` (c12);
  * `array/sort.rs` (c13), `array/surface.rs` (c14).

  Each file holds native bodies whose signature names `Cleared`. Each body
  is dispatch code the list already covered, through `src/dispatch/hash.rs`
  or `src/dispatch/collection.rs`, and none adds a producer of the token.
  `method_arguments.rs`, `library/tests.rs` and `native/tests.rs` name no
  `Cleared` and were not added. The list is live: with `hash/stem.rs`
  removed from it, `the_seam_token_is_named_only_by_the_dispatch_module`
  fails (`files/final/seam-control.txt`).
* **`refusal-sites.tsv` column 3 is asserted, not assumed.** Every commit
  re-derived the table with `REXX_REFUSAL_SITES_REFRESH=1` and diffed every
  column other than column 4, column 3 included, for all 247 rows
  (`files/c<N>/c<N>-refusal-sites.txt`, each showing that the table's mtime
  moved). The result was identical at all fourteen commits. Column 4 did
  not change either: none of the moved code defines a construction site,
  and the one `dispatch/native.rs` row, at line 694, is above the test
  module c2 moved. So nothing needed a ruling.
* Path mentions (`files/path-pins-base.txt`; the brief's two `grep`s,
  before any move). The only file under `crates/*/tests` or `crates/*/src`
  that names a `dispatch/<file>` path is `dispatch_seam.rs`. In the corpus,
  `refusal-sites.tsv` names `dispatch/native.rs:694`, which still holds,
  and `array_make_string.rex` and its recording say "dispatch.rs's
  NATIVE_METHODS". That is still where `Array`'s rows are, since Task 2's
  rows stayed. `run/loops.rs:618`, "the store `hash.rs` owns", is still
  true.

## Departures from the brief

1. **Visibility of depth-two items: `pub(in crate::dispatch)`.**
   `ObjectModel::build` reads each new slice from two levels up, which
   `pub(super)` does not reach. I asked team-lead to choose between
   `pub(crate)` (the ladder as written) and `pub(in crate::dispatch)`,
   which is narrower and which `native.rs` already uses. I said I would
   default to the latter. No ruling came, so the seven new tables,
   `new_supplier` (re-exported from `collection.rs` so that
   `collection::new_supplier` keeps working for `hash.rs`, `hash/*.rs` and
   `introspection.rs`) and `native_array_delete` (called by
   `Queue~delete`) are `pub(in crate::dispatch)`. If the ruling is
   `pub(crate)`, that is a one-token change on nine lines.
2. **Widenings the `Array` placement cost.** Fourteen `collection.rs` store
   helpers went from private to `pub(super)` so that the `array/`
   children, outside `collection`'s subtree, can reach them. Those were
   `store_of` and `slots_of` at c13, and `dimensions_of`, `position_in`,
   `QUEUE_ITEMS`, `store_scope`, `ordered_pairs`, `last_item`,
   `subscript_object`, `occupied`, `queue_bound`, `array_splice`,
   `array_splice_slot` and `array_grow` at c14. The other moves widened
   only what a parent calls back into: `is_list`, `list_insert_at` and
   `list_state` (c10), and eight private parsers for `buffer.rs` (c7).
3. **Declared edits.** Each is shown in `files/c<N>/c<N>-instrument1.txt`,
   and each file's rule is in `files/c<N>/rules`:
   * c2: in `check_binds` and
     `an_entry_point_resolves_whatever_case_it_is_asked_for`, rustfmt, once
     the de-indent gave it four more columns, drops a match arm's and a
     closure's block braces. Literals are unchanged (instrument 3).
   * c6: the moved `Directory` natives named the store as `hash::owns`,
     `hash::store_at` and so on, seven qualifiers naming the module they
     now sit in. They were dropped. The final re-run with
     `--expect-drop=hash::` proves each unit equals its PRE tokens with
     those qualifiers removed and nothing else
     (`files/final/rerun/c6-drop/c6-instrument2.txt`).
   * c7: `buffer.rs`'s module doc and the comment on `mod buffer;` said the
     argument parsers were there. Both now say "the argument and
     byte-search helpers".
   * c8 to c14: `build`'s chain.
   * c14: the test chain above, and five intra-doc links. `hash.rs`'s table
     now links `collection::list::NATIVE_METHODS`. The three `collection/`
     tables link `super::super::NATIVE_METHODS`, as every other chained
     slice does. `is_list`'s doc links `super::is_queue`; see concern 1.
4. **Import lines added to parents**, each an inserted line, never an edit
   to an existing one. They serve the moved bodies' own `super::` paths,
   which now resolve one level down:
   * `hash.rs`: `use super::collection;`;
   * `collection.rs`: `use super::{INIT, array_size_argument,
     optional_length_argument, unsigned_index};` and `use
     super::{native_array_at_for, positive_index};`;
   * `array.rs`: `use super::{Arity, NativeMethod};` and
     `use super::new_instance;`.

   They count as used, as in Task 5. The same applies to the extra `use`
   line `string.rs` (c5) and `hash.rs` (c6) each gained. `buffer.rs`
   imports its parsers from `super::method_arguments` directly, and
   `dispatch.rs` imports only the parsers other children reach.
5. **`build`'s chain for `hash.rs` is reordered**, as described above, and
   the dump shows the reorder changes nothing.

## The instruments

Per-commit outputs are in `files/c<N>/`, made by `tools/checks.sh` over the
working tree before each commit. `tools/commit_task6.sh` refused to commit
unless the staged `rust/` tree hash equalled the one instrument 4 ran on,
and every commit's hashes matched (`files/instrument4/i4-c<N>.meta`).

| # | 1 (a) unmoved | 1 (b) moved units | 2 tokens | 3 literals | 4 tests | load before / after instrument 4 |
| --- | --- | --- | --- | --- | --- | --- |
| c1 | 775 equal, 1 inserted | 26 of 27, 1 reflowed | 88 of 88 | 88 of 88; 325 literals; 0 control mismatches | identical | 3.90 5.21 4.98 / 1.95 4.66 4.95 |
| c2 | 703 equal, 1 inserted | 13 of 15, 2 declared | 38 of 40, 2 declared | 40 of 40; 325; 0 | identical | 0.83 0.93 1.88 / 2.88 4.70 3.68 |
| c3 | 3135 equal, 1 inserted; dest +31 | 2 of 2 (vis) | 301 of 301 | 301 of 301; 768; 0 | identical | 2.89 4.33 3.62 / 2.25 4.54 4.24 |
| c4 | 3116 equal, 1 inserted; dest +20 | 2 of 2 (vis) | 300 of 300 | 300 of 300; 761; 0 | identical | 3.35 4.39 4.21 / 2.66 4.92 4.75 |
| c5 | 2978 equal, 4 inserted; dest +3, +139 | 6 of 6 (vis) | 299 of 299 | 299 of 299; 757; 0 | identical | 1.67 4.29 4.54 / 3.00 5.01 4.91 |
| c6 | 2869 equal, 1 inserted; dest +1, +113 | 1 of 4, 3 declared | 291 of 294, 3 declared (294 of 294 with `--expect-drop`) | 294 of 294; 744; 0 | identical | 3.26 4.75 4.82 / 4.02 5.77 5.36 |
| c7 | 1868 equal, 7 inserted, 2 declared lines, 1 narrowed | 18 of 18 (8 vis) | 166 of 166 | 166 of 166; 450; 0 | identical | 1.37 3.70 4.62 / 2.40 5.21 5.17 |
| c8 | 2410 equal, 5 inserted | 50 of 50 | 233 of 233 | 233 of 233; 474; 0 | identical | 1.59 3.97 4.72 / 2.53 5.29 5.29 |
| c9 | 2129 equal, 4 inserted | 19 of 19 | 185 of 185 | 185 of 185; 368; 0 | identical | 1.85 4.26 4.92 / 2.39 4.96 5.15 |
| c10 | 1645 equal, 6 inserted | 64 of 65, 1 reflowed | 203 of 203 | 203 of 203; 368; 0 | identical | 2.47 4.11 4.82 / 2.44 5.17 5.24 |
| c11 | 1468 equal, 5 inserted | 23 of 23 | 141 of 141 | 141 of 141; 264; 0 | identical | 2.31 4.43 4.97 / 2.37 5.18 5.25 |
| c12 | 1300 equal, 6 inserted | 18 of 18 | 120 of 120 | 120 of 120; 218; 0 | identical | 2.51 4.53 5.01 / 2.24 5.20 5.34 |
| c13 | 988 equal, 1 vis, 1 widened-reflow, 1 narrowed | 16 of 16 | 103 of 103 | 103 of 103; 185; 0 | identical | 1.25 3.44 4.63 / 2.76 5.10 5.10 |
| c14 | 394 equal, 1 inserted, 7 vis, 5 widened-reflow, 5 declared deletions | 52 of 52 | 88 of 88 | 88 of 88; 154; 0 | identical | 2.60 3.82 4.58 / 3.73 5.38 5.14 |

"dest +n" is a block inserted into a destination that already existed.
Instrument 1 (c) checks that such a destination changed by inserted lines
only, each block compared with `cmp`. Instrument 4 gave 1582 test lines in
50 result blocks at BASE and at every commit, compared per result block
and per test (`files/instrument4/`). The flake did not fire at any commit.
c1 to c6 changed no file outside their parent and destinations
(`files/c<N>/c<N>-other-edits.diff` is empty). From c7 on, every other
file has a rule, and `tools/other_edits.py` checked each one
(`files/c<N>/c<N>-other-edits-check.txt`).

**Tooling changes, each with a control.**

* **`item-tool` rows `const` tables** of `(str, str, ..)` tuples, as it
  already rowed `static` ones. `hash.rs` and `collection.rs` declare their
  native tables `const`. Without the change each table was one unit, and
  moving rows out of it would have read as a token change to that one
  unit. Any other `const` stays one unit.
* **`instruments.py`**:
  * asserts that PRE has no duplicate unit keys;
  * `--expect-line` for a corrected module-doc line;
  * `--expect-gone` for the deleted table's lines;
  * `--expect-drop=hash::` for c6's qualifiers;
  * an unmoved unit whose visibility the commit widened and rustfmt
    re-wrapped is compared with its visibility removed, whitespace-blind,
    and listed as `REFLOWED (visibility widened)`;
  * the names an `IMPORTS NARROWED` block reports are now right for a
    deletion inside a declaration. Before, it listed every name of the
    declaration.
* **`other_edits.py`** has three new rules: `imports:OLD=>NEW[:comments=K]`,
  `narrow`/`narrow+insert`, and several `subst` joined with `;;`.
* **The pipeline** (`checks.sh`, `verify.sh`, `prod.sh`): the per-commit
  checks now gate on the other-edits check and on the `cargo doc` warning
  counts being BASE's, with none located in a dispatch file. See concern 1
  for why. `natives_check.sh` runs on the test worktree.
* **`run_tests.sh`** runs with the default target directory, which the
  arity suites need. BASE's first two attempts at instrument 4 failed for
  environmental reasons: the test worktree lacked `build/`, `ootest/` and
  `oodocs/`, and then the arity suites could not find
  `target/release/rexx-run`. `files/instrument4/base-attempt1/` and
  `base-attempt2/` hold them. Only the third BASE run, clean, is the
  comparison baseline. c1's meta had an empty tree line, because `git add`
  exited 1 on the ignored `rust/target` pathspec. It was filled in from
  the same unchanged worktree, with a note in the file, and the commit
  script's tree check used it.

**Controls**, on trees fresh from `git archive`. Task 2's five, Task 3b's
four, Task 4's eight and Task 5's seven ran before the first move
(`files/controls-*-before.txt`), again after the `const`-row change
(`-before-constrows`), and at the end (`files/final/controls-*-final.txt`).
The result was 5 of 5, 4 of 4, 8 of 8 and 7 of 7 every time. Task 5's
three other-edits controls give 3 of 3 at both ends. This task's own
controls:

* `tools/controls_task6.py`, 8 of 8 (`files/final/controls-task6-final.txt`):
  1. a moved `const` row's arity changes: I1b, I2, I3;
  2. a moved row is dropped: I2;
  3. an unmoved `const` row is repointed: I1a, I2;
  4. a declared doc line carries an extra word: I1a;
  5. a token changes inside a widened, re-wrapped unmoved unit: I1a, I2;
  6. an undeclared deletion sits beside the declared ones: I1a;
  7. a declared unit changes beyond the dropped qualifiers: I2;
  8. an unmoved `dispatch.rs` function is renamed: I1a, I2.
* `tools/controls_other_edits6.sh`, 5 of 5 planted defects caught, with
  the three commits as committed passing
  (`files/final/controls-other-edits6-final.txt`): a name moved to
  another module, a third comment line edited, a code line changed beside
  a narrowing, a second edit in a file with two declared substitutions,
  and a test chain naming a slice twice.
* The native table and seam controls above.

**Re-run with the final tooling** on every commit pair, from `git archive`
trees (`tools/rerun6.sh`, `files/final/rerun-summary.txt`). Every verdict
is PASS, and every output is identical to the committed one. The one
exception is the extra c6 run with `--expect-drop`, which differs where
it should.

**Cumulative** (`files/final/cumulative.txt`): for each parent, every BASE
unit is present exactly once at `ddeeb5640`, in the parent or a
destination, identical modulo visibility, or a declared edit. The declared
edits are c6's three natives, `build`, c2's two tests, and `is_list`'s
doc. **Comments** (`files/final/comments.txt`): across the parents and
every destination, eight comment lines were lost, each one a line listed
above: c7's four, c14's two table-doc lines, and c14's two links. What was
added is the license headers, the module docs, the `mod` comments, the
new tables' docs and the corrected lines.

**`cargo doc --no-deps -p rexx-exec`**: 2 warning lines at every commit;
53 with `--document-private-items`, and 52 with `--cfg test` as well.
From c10 to c13 there was one more of each: see concern 1. c14 puts both
back to BASE's 53 and 52, with 0 located in dispatch files. `tests_links.sh`
found 0 warnings in `library` and `native` and their test files, at BASE
and after c1 and c2. Neither moved test module contains an intra-doc
link (`files/final/intra-doc-links.txt`, with the command).

## Performance

The measure is callgrind's `summary:` minus `libc.so.6` and `ld-linux`, in
two interleaved rounds. BASE was built in its own worktree and target
directory, and `ddeeb5640` in the test worktree's. `.text` sha256 is
`36972bba...` for BASE, the same as Task 5's final since the code is
identical, and `c8244939...` for `ddeeb5640`, whose `.text` is 4432 bytes
smaller (`files/perf/meta.txt`, with the command).

| program | BASE round 1 | BASE round 2 | final round 1 | final round 2 | change |
| --- | --- | --- | --- | --- | --- |
| rexxcps | 17897272755 | 17897272092 | 17899771416 | 17895014232 | +0.00067% |
| nop | 9340093337 | 9340093213 | 9340101473 | 9340100086 | +0.00008% |
| assign | 19540359684 | 19540357175 | 19540361243 | 19540374839 | +0.00005% |
| emptyloop | 9285883079 | 9285884911 | 9285882675 | 9285880563 | -0.00003% |
| varlookup | 14842898905 | 14842894677 | 14842894644 | 14842898955 | +0.00000% |
| arith | 11519334845 | 11519337201 | 11519335483 | 11519334410 | -0.00001% |
| compound | 9234393861 | 9234398882 | 9234397808 | 9234395670 | +0.00000% |
| dispatch | 20471073373 | 20471070051 | 20461081192 | 20461078952 | -0.04881% |
| strings | 17788054908 | 17788057932 | 17737056543 | 17737060153 | -0.28670% |

No axis moves more than the brief's 0.5%, so nothing was bisected.
`compound`, the `Stem` axis, is flat. `strings` (-0.29%) and `dispatch`
(-0.05%) moved by the same amount in both rounds, so the change is real
codegen and not noise. Both are under the threshold, and both went down.
`rexxcps`'s two final rounds differ from each other by 4.8 million,
against 663 between BASE's rounds. Its change is 0.0007%. Every program's
stdout is identical between the binaries except `rexxcps`'s wall-clock
clauses-per-second line (`files/perf/stdout/`). The load average was 1.64
2.58 3.90 before the runs and 8.04 6.67 5.37 after, with 12 runs in
parallel.

## Gates

These ran over `b53917360`, this report's first commit, whose code is
identical to c14. `files/tools/gates.sh` ran them after the commit. The
tree did not change from start to finish: `git status --short` was empty
before and after. The statuses, each read unpiped, are in
`files/gates/status.txt`. G3 to G6 use the default target directory, so
the arity suites ran the binary G3 built.

| gate | command | result | load average (1, 5, 15 min) |
| --- | --- | --- | --- |
| G1 | `cargo fmt --all --check` | exit 0 | 2.48, 4.57, 4.77 before the run |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings`, empty target dir | exit 0 | |
| G3 | `cargo build --workspace --all-targets --release`, no cap | exit 0 | |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast` | exit 0; 135 result blocks; 2663 passed, 0 failed, 4 ignored; `corpus_differential` 604 of 604, STRICT | 16.07, 9.76, 6.66 before (G3's build had just finished); 3.13, 7.19, 6.64 after |
| G5 | `cargo build --workspace --all-targets` (debug) | exit 0 | |
| G6 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (debug) | exit 0; 135 result blocks; 2664 passed, 0 failed, 4 ignored; `corpus_differential` 604 of 604, STRICT | 5.92, 7.70, 6.80 before; 2.20, 6.51, 6.93 after |

Every figure matches BASE's. The counts come from `tools/test_results.py`
run over each gate's own log (`files/gates/test-release.results`,
`test-debug.results`). The 604 comes from that log's differential report
(`files/gates/corpus-differential-*.txt`).
`introspection_arity::every_unstable_row_is_really_unstable` passed in
both runs, so nothing was re-run.

## Concerns

1. **c10 to c13 carried a broken private intra-doc link.** c10 moved
   `is_list`, whose doc links `[`is_queue`]`, out of `is_queue`'s module.
   `docs.sh` counted the extra warning (54 and 53 instead of 53 and 52) at
   c10, c11, c12 and c13, and I did not read the count. c14 fixed the link.
   From c14 on, the pipeline refuses a commit whose warning counts differ
   from BASE's, or which has any warning located in a dispatch file. The
   public `cargo doc` did not see the break: 2 warnings throughout.
2. **Files still over the trigger**: `dispatch.rs` (2881), `string.rs`
   (2364), `hash.rs` (2133), `stream.rs` (1925), `buffer.rs` (1877),
   `package.rs` (1063). `class_protocol.rs`, `object_protocol.rs` and
   `dispatch/tests.rs` were out of scope. The reason each of the six stays
   is given above. `dispatch.rs`'s remaining non-model code is
   `VariableReference`'s natives (about 130 lines), which no existing
   child serves.
3. **`stream.rs`'s module doc was already false at BASE.** It says the
   reading and writing halves "are separate work", yet `charin`, `linein`,
   `charout` and `lineout` are in the file. This task did not make it
   false, so it is not edited here. Likewise, `files.rs`'s doc says each
   entry point is "a function of that path alone", which
   `case_sensitive`, `list_roots` and `temporary_path` already were not
   at BASE; the two separators c4 moved there are not either.
4. **`dispatch.rs`'s comment on `mod string;`** says `String`'s rows are
   chained "rather than merged into" `NATIVE_METHODS`. Six of the
   functions `string.rs` now holds have their rows in `NATIVE_METHODS`, by
   the ruling that those rows stay. The comment still describes
   `string.rs`'s own table correctly, so it was left alone.
5. **The test chain in `dispatch/tests.rs` lists `hash::NATIVE_METHODS`,
   but not `hash::relation` or `hash::stem`.** c8 and c9 did not need to
   touch it, and it filters to `String` rows, which neither slice has.
   c14 replaced `collection::NATIVE_METHODS`, which it did have to touch,
   with all five of its successors.
6. **`pub(in crate::dispatch)` was used before the ruling came**
   (Departure 1). The ruling that followed accepted it; see below.
7. The rules and spec files behind each commit's other-edits check
   (`files/c<N>/rules`, `spec.json`) are committed with this report and
   not with their commits.

## Rulings after the report

team-lead ruled on four points after the gates ran.

**Q1: `pub(in crate::dispatch)` is accepted** for items two levels down
that are read from outside their subtree. Each such item, and what reads
it (`/bin/grep -rn 'pub(in crate::dispatch)' src/dispatch`, `native.rs`'s
two older ones left out):

| item | file | read by |
| --- | --- | --- |
| `NATIVE_METHODS` | `hash/relation.rs` | `ObjectModel::build` (`dispatch.rs`) |
| `NATIVE_METHODS` | `hash/stem.rs` | `ObjectModel::build` |
| `NATIVE_METHODS` | `collection/list.rs` | `ObjectModel::build`; the test chain in `dispatch/tests.rs` |
| `NATIVE_METHODS` | `collection/queue.rs` | `ObjectModel::build`; the test chain |
| `NATIVE_METHODS` | `collection/supplier.rs` | `ObjectModel::build`; the test chain |
| `NATIVE_METHODS` | `array/sort.rs` | `ObjectModel::build`; the test chain |
| `NATIVE_METHODS` | `array/surface.rs` | `ObjectModel::build`; the test chain |
| `new_supplier` | `collection/supplier.rs` | through `collection.rs`'s `pub(super) use supplier::new_supplier`: `hash.rs`, `hash/stem.rs`, `hash/relation.rs` (as `super::collection::new_supplier`), `introspection.rs`, `array/surface.rs`; within its subtree, `collection/list.rs` |
| `native_array_delete` | `array/surface.rs` | `collection.rs` imports it (`use super::array::surface::native_array_delete`) for `collection/queue.rs`'s `native_queue_delete` |

Each parent declares its children `pub(super) mod`: `hash.rs` (`relation`,
`stem`), `collection.rs` (`list`, `queue`, `supplier`) and `array.rs`
(`sort`, `surface`).

**The hash chain reorder is accepted**, on the condition that
`natives_dump` is identical to BASE's at that commit. It is at c8 and at c9
(`files/c8/c8-natives-verdict.txt`, `files/c9/...`), and at every commit
since. **No MethodId appears in more than one hash-family slice.** A
scratch build of `ddeeb5640`, whose `build` printed each row's class,
method name and MethodId as it filed it, was never committed.
`files/final/hash-family-method-ids.txt` records it:
* `hash::NATIVE_METHODS` has 32 rows, `hash::relation` 10 and `hash::stem`
  19.
* No MethodId is filed by two of these slices.
* No MethodId of theirs is filed by any row outside them.
* The only MethodIds filed twice are twelve inside `hash::NATIVE_METHODS`:
  `Table`'s rows and `IdentityTable`'s, which share an identity. Each pair
  names the same function, and c8 and c9 did not change their order.

So the reorder cannot change what `build` files, and the identical dump
agrees.

**`dispatch/tests.rs:123` is accepted** as a declared path edit. The
test's name is unchanged, and instrument 4 at c14 is identical.

**`Array` under `dispatch/array/` is accepted.**

## Fix round 1

This round addresses the task review
(`.superpowers/sdd/2026-09-15-file-split/task-6-review.md`), findings I1,
M1 and M3. It is one commit and changes comments only. The chain the
comments now describe is `build`'s at `dispatch.rs:903-917`:
`string`, `hash`, `hash::relation`, `hash::stem`, `collection::list`,
`collection::queue`, `array::sort`, `array::surface`,
`collection::supplier`, `rexx_info`.

* **I1**, `collection.rs`. c14 deleted collection.rs's own table, which
  made all three comments false. Each now names its real neighbour:
  * `List`: "chained into `ObjectModel::build` ahead of this module's own"
    now reads "ahead of `Queue`'s".
  * `Queue`: "ahead of this module's own" now reads "ahead of `Array`'s".
  * `Supplier`: "after this module's own" now reads "after `Array`'s".
* **M1**, `dispatch.rs`, the comment on `mod string;`. c5 made it
  incomplete, and a clause now qualifies it: "whose rows are chained into
  `ObjectModel::build` beside `NATIVE_METHODS` rather than merged into it,
  except the rows `NATIVE_METHODS` itself holds."
* **M3**, `hash.rs`, the doc on `hash.rs`'s table. "beside
  [`super::collection::list::NATIVE_METHODS`]" now reads "ahead of
  [`relation::NATIVE_METHODS`]", the slice `build` chains directly after
  it.

**Checks:**
* `tools/noncomment_tokens.py` takes each file's text with every comment
  removed, as `rustlex.py` finds comments, and compares it with the same
  file at `808c675c3`. All three files are IDENTICAL
  (`files/fix1/noncomment-tokens.txt`). A control plants one token,
  changing `pub(super) mod queue` to `pub(crate)`, and the tool reports
  DIFFER (`files/fix1/noncomment-tokens-control.txt`).
* `cargo fmt --all --check` exits 0.
* `cargo clippy -p rexx-exec --all-targets -- -D warnings` exits 0, run
  with the default target directory and 39G free in `/tmp`.
* `cargo doc --no-deps -p rexx-exec` gives 2 warning lines. With
  `--document-private-items` it gives 53, and with `--cfg test` as well
  52. These are BASE's figures, with 0 located in dispatch files, so the
  new `relation::NATIVE_METHODS` link resolves.

M2, `files.rs`'s doc, is left for a documentation pass, as the review
says.
