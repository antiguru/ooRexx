# Phase 5g-D — close

Plan: `docs/superpowers/plans/2026-09-07-phase-5g-d-divergences.md`.
BASE `5ac6f69e7`. Subject: the divergences Phase 5g's Task 9 confirmed and
left named rather than fixed.

## What the phase was for

Task 9's report ends with a list headed *Confirmed rather than changed*: real
row-level divergences, each with a repro in the review's own directory, left
unfixed because "fixing them is another phase's worth of work". This was that
phase. Every row on the list was re-measured against the oracle and against
this crate's `5ac6f69e7` binary before being planned, which is what caught the
one that was no longer true.

## Struck before any work

**`Queue~put`'s argument position in 93.907.** Task 9 restored the two-tier
queue bound and, with it, this row; the report listed it anyway. Re-measured
at five shapes -- `put('Y',2)`, `put('Y',5)`, `put('Y',99)`, `put('m',-1)`,
`put('m')` -- oracle and crate agree line for line: 93.966, 93.966, 93.918,
`93.907 ... argument 2 ... found "-1"`, 93.903. No code changed.

## What moved, one task each

| task | subject | before | after |
|---|---|---|---|
| D1 | a `List` index is converted, not compared | `at(' 1')` and five other spellings `.nil` | all six `b`; a spelling that cannot convert is 93.918 |
| D2 | an entry holding nothing is still an entry | `items` 2 of 3, every walk off by one | chain and count carry it; only lookups read it absent |
| D3 | `empty` resets the free chain; `section` is a `List` | appends after `empty` `2 1 0 3`; subclass section `MYLIST` | `0 1 2 3`; `List` |
| D4 | `Supplier` bounds, and `requestArray` | bounded by the shorter array; a `List` argument refused | bounded by ITEMS; converted |
| D5 | the contents belong to `new` | `of` sent `APPEND`; a non-forwarding subclass `INIT` had no store; a second `INIT` wiped it | `INIT` with no arguments and nothing else; built on demand; kept |
| D6 | the sort | `'1.0'` left the array unsorted, `'abc'` ran on; sequence was a textbook merge | 26.903/26.902; upstream's sequence, comparison for comparison |
| D7 | `dimensions` after `append` | `.Array~new(0)~append('q')~dimensions` `0` | `1` |

Seven witnesses, one per task, all filed in the three places and all
byte-identical to the oracle on both engines:
`list_index_conversion.rex`, `list_empty_entry.rex`,
`list_empty_and_section.rex`, `supplier_bounds.rex`,
`collection_construction.rex`, `sort_comparisons.rex`,
`array_dimensions_append.rex`. Strict corpus 424 at BASE, 431 at close.

## What is still open, named rather than absorbed

**`allItems~items` over a `List` holding an empty entry.** Oracle 3, this
crate 2, both agreeing the array's size is 3 and that its slot 2 answers
`hasIndex` 0. `ListContents::allItems` appends the empty entry's `OREF_NULL`
(`classes/support/ListContents.cpp:672`) and `ArrayClass::setArrayItem`
increments `itemCount` whenever the slot was unoccupied without looking at the
value (`classes/ArrayClass.cpp:512`), so upstream's own count disagrees with
its own `hasIndex`. This crate derives the count from occupancy. Closing it
means giving `Body::Array` a stored count that can disagree with its slots,
which is a change to the array representation and not to `List`.

**`unconverted_array_argument`'s refusal is still loud** where upstream's
`arrayArgument` raises `Error_Execution_noarray` for a receiver whose
behaviour has no `MAKEARRAY`. D4 added the conversion for receivers that have
one; the other limb is unchanged and was not on Task 9's list.

## A defect I shipped inside this phase, and how it was found

D4 inserted `request_array` immediately above `unconverted_array_argument` and
silently took that function's doc block with it. `cargo fmt` and `cargo
clippy` both pass over it -- neither can see it -- and no test executes a doc
block that carries no code. It was found by reading the file after the commit,
not by any instrument, and fixed in `ccf55c3e3`.

The same insertion shape occurs four more times in this phase
(`unsigned_index`, `array_of_slots`, `whole_comparison`, and the sort's four
functions). Each was then checked against the file rather than assumed: all
four end their replacement at the item they follow, so the next doc block
still sits on the item it names.

## A hazard met while writing a witness, and not a divergence

A SYNTAX condition raised inside an internal routine declared `PROCEDURE` runs
the CALLER's `SIGNAL ON SYNTAX` trap in the CALLEE's variable pool. Measured:
with `n = 7` set before the call, the trap prints `n` as `N` and `symbol('n')`
as `LIT`; drop the `procedure` keyword from the same routine and the trap
prints `7`.

The corpus's `signal next` retry idiom then re-raises on its own uninitialised
counter forever. Measured, both interpreters spin -- the oracle wrote 961,910
lines in ten seconds, this crate spun silently -- and **neither answers
SIGTERM**, so `timeout` without `-s KILL` does not stop them.

**This crate and the oracle agree**, on both engines, so nothing here changes.
It is not in `corpus/oracle-crashes.txt` either: that file is for programs the
oracle cannot survive, and this one is survived by neither, which makes it a
programming hazard rather than an oracle property. It is why every refusal in
`sort_comparisons.rex` is raised inline in the main routine.

## Gates

Two seven-gate runs, both all green.

Over `96d376350`, the phase through D4: G1 0, G2 0, G3 0, G4 0, G5 0, G6 0,
G7 0, `failed-suites=0` on each suite-running gate
(`scratchpad/gates-d4.status`).

Over `c69a2f688`, the phase complete: the same seven zeros and the same five
`failed-suites=0` (`scratchpad/gates-d7.status`).

Per-task fast checks were `cargo fmt --all --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, and `cargo test --release --workspace
--no-fail-fast` under `REXX_CORPUS_GATE=1` -- which is G1 through G4 exactly,
so what the two gate runs add over the per-task checks is G5's debug build and
the two phase gates.
