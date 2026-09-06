# Phase 5g Task 3 — the sort family

Plan: `docs/superpowers/plans/2026-09-06-phase-5g-ordered-collections.md`.
BASE `f148122f9`.

Four rows: `sort sortWith stableSort stableSortWith`.

---

## One algorithm, not two

`Setup.cpp` maps `Sort` and `StableSort` onto `ArrayClass::stableSortRexx` and
both `With` spellings onto `stableSortWithRexx`, so the unstable names are the
stable bodies. The witness asserts that rather than assuming it: the two
spellings answer the same thing on the same data.

The sort is a stable merge sort written out rather than handed to
`slice::sort_by`, because **the comparison runs Rexx and can raise** and a
`Result` cannot travel through a `bool` comparator.

## The plan's open question, answered before any code

The plan said to check that a native body can call back into a Rexx method
*before* planning the sort, since if it could not, that would be the task.
Measured on the oracle first, then on this crate: `sortWith` over a
`::CLASS MyCmp SUBCLASS Comparator` whose `compare` is Rexx answers `3,2,1`,
and the default order over a user class's own `compareTo` orders by its
values. The mechanism was already there -- Task 1's `same_item` sends `==` the
same way -- so the sort is the task after all.

## What the order actually is

**Not numeric.** `ArrayClass::BaseSortComparator::compare` is
`first->compareTo(second)` (`classes/ArrayClass.cpp:2891`), and `compareTo` on
a string or an integer is a *string* comparison. Measured:
`.Array~of(10,9,2,100,1)~sort` answers `1,10,100,2,9`, where a numeric sort
would answer `1,2,9,10,100`. That one line is the difference and the witness
prints both, the second through `.NumericComparator`.

`sortWith` sends `COMPARE` with two arguments (`:2897`) and raises 91.999 if
the comparator answers nothing -- measured, byte-identical on both sides, and
now the witness for `refusal-sites.tsv`'s `no_result` row, which moved from
`off-send-surface` to `body+send` because this task made it reachable.

The sort is **in place** and answers the receiver.

## The refusal that is not a skip

**A hole is a refusal.** Measured at rc 158: an array holding `[1]`, `[3]`,
`[5]` answers `~sort` with `98.975 Missing array element at position 2.`,
where `allItems` skips the same holes without complaint. The witness prints
`allItems` first and then lets the sort refuse, so the two behaviours sit
beside each other. A new constructor, `missing_array_element`.

A **multi-dimensional** array sorts, flattened -- the sort family is not among
the four methods `checkMultiDimensional` guards.

## Witness and red controls

`corpus/lang/array_sorting.rex`, filed in all three places. Strict corpus:
**414 of 414 matching**, up from 413.

**M5 -- the merge's tie-break `<= 0` becomes `< 0`**, which makes the sort
unstable. Predicted red on `array_sorting.rex`. **Confirmed**: 413 of 414,
stdout differing.

**M6 -- `dense_items` skips holes instead of raising.** Predicted red on the
same file, and on all three descriptors because the program's last line is the
refusal. **Confirmed**: 413 of 414, `stdout, stderr, exit code differ`.

Two cases in the witness were rewritten before it was filed because they did
not discriminate: every `K` renders identically so the `compareTo` order had
to be read back value by value, and the original caseless data sorted the same
way case-sensitively. Both were caught by reading the oracle's own output
rather than by a mutation.

## What moved

`corpus/method-bodies.txt`: 4 rows, all `Array`, all `loud` -> `answers`.
`corpus/collection-arity.tsv`: `agree` 51 -> 55, `send-differs` 60 -> 56.
`corpus/refusal-sites.tsv` gained `missing_array_element` and moved
`no_result` onto the send surface.

## Gates

Fast checks: `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` clean, `cargo test --release --workspace
--no-fail-fast` exit 0, 2124 passed, no suite failing.

Seven gates over `eaae6abf4`, from `scratchpad/gates-5g3.status`: G1 0, G2 0,
G3 0, G4 0, G5 0, G6 0, G7 0, `failed-suites=0` on each suite-running gate.
