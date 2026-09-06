# Phase 5g Task 2 — `Array`'s own surface

Plan: `docs/superpowers/plans/2026-09-06-phase-5g-ordered-collections.md`.
BASE `b1377cb53`.

Thirteen rows bound: `append delete dimensions fill first firstItem insert
last lastItem next previous section`, and `Array~dimension`'s plural sibling.

---

## The pair the task exists to get right

**`delete` and `remove` take the same argument and answer the same item, and
do different things to the array.** Measured on `x,q,y,z,w`: `delete(2)`
answers `q` and leaves `x,y,z,w` at size 4; `remove(2)` answers the item and
leaves a hole with the size unchanged. A witness that reads only the returned
value cannot tell them apart, so `array_structure.rex` prints the whole array
after each.

Same shape one level down: **`first`/`last` answer an INDEX and
`firstItem`/`lastItem` answer the item.** On a sparse array the two cannot
coincide -- an array holding `[1]`, `[3]`, `[5]` answers `first 1 last 5`
against `firstItem p lastItem t` -- which is what `array_navigation.rex` is
built on.

## What the refresh caught that the witnesses did not

`method_bodies.rs` refused the first refresh with **six regressed rows**, and
every one was a real defect:

| row | oracle | this crate, before |
|---|---|---|
| `Array~delete` with no argument | 93.903 | 93.901 |
| `Array~section` with no argument | 93.903 | 93.901 |
| `Array~insert` with no argument | **rc 0** | 93.903 |

`ArrayClass::deleteRexx` opens `requiredArgument(index, ARG_ONE)`
(`classes/ArrayClass.cpp:822`) where the index-validating family raises
93.901, and `sectionRexx` does the same at `:1505`. **`insertRexx` has no
`requiredArgument` for its value at all**: measured,
`.Array~of('x','y')~insert` answers `3` and leaves size 3 with items 2 -- an
empty slot, not a refusal.

Chasing that found a fourth thing neither the witnesses nor the refresh had
asked about. **Four methods, and only four, refuse a multi-dimensional
receiver**: `checkMultiDimensional`'s callers upstream are `APPEND`, `INSERT`,
`DELETE` and `SECTION`. Each raises 93.954 `Method "X" can be used only on a
single-dimensional array.`, a constructor this crate did not have. `fill`,
`first`, `next` and the rest take any shape.

**The check order differs between them and is not guessable.** `append` is
`requiredArgument` then the dimension check; `delete` likewise; `section` and
`insert` check the dimension **first**. Two of the four therefore answer a
different error for the same bad call.

`insert`'s index has a third spelling as well: `.nil` means the front
(`:755`), an omitted index means after the last **occupied** slot rather than
the end of the array, and an index past the end extends -- `insert('far', 9)`
on a size-2 array answers `10`.

## Witnesses

`corpus/lang/array_navigation.rex`, `array_structure.rex`,
`array_structure_refusals.rex`, all filed in `corpus/phase-5c.txt`, in
`EXPECTED_SUBSET_5C`, and as `sourceline_oracle/<name>.txt`. Strict corpus:
**413 of 413 matching**, up from 410.

### Red controls

**M3 -- `delete` clears the slot instead of splicing**, which is `remove`'s
behaviour. Predicted red on `array_structure.rex`. **Confirmed**: 412 of 413,
that file, stdout differing.

**M4 -- `first` answers the item instead of the index.** Predicted red on
`array_navigation.rex`. **Confirmed**: 412 of 413, that file.

Both predictions were written before the runs and both held. Unlike Task 1's
M2, neither witness needed extending afterwards -- the discriminating cases
were in them from the start, because the two pairs above are what the
programs were written around.

## What moved

`corpus/method-bodies.txt`: **28 rows**, 13 `loud` -> `answers` (`Array` 12,
`Queue` 1) and 15 that stay `loud` with new evidence.

`corpus/collection-arity.tsv`: `agree` 38 -> 51, `send-differs` 73 -> 60;
13 rows moved.

`corpus/refusal-sites.tsv` gained `single_dimension_only` and every `error.rs`
line below the insertion shifted by thirteen.

## Gates

Fast checks: `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` clean, `cargo test --release --workspace
--no-fail-fast` exit 0, 2124 passed, no suite failing.

**G1** **G2** **G3** **G4** **G5** **G6** **G7**
