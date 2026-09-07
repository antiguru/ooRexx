# Phase 5h — close

Plan: `docs/superpowers/plans/2026-09-06-phase-5h-mapped-collections.md`.
BASE `c69a2f688`, which is Phase 5g-D's close.

## (a) The receiver overrides

Nine added, one per class in scope, each a single expression because that is
what a receiver override is: `~~` answers its receiver, so the puts chain.
`Set` and `Bag` use `of`, which is native for those two alone; the rest
inherit `MapCollection~OF`, which spec D100 blocks.

**They moved two rows, and that is the right answer rather than a
disappointment.** A populated receiver sharpens a probe only where an empty
one could not discriminate, and by the time this step ran every body in the
phase already answered either way. The two that moved are
`Stem~hasItem` and `Stem~index`, from `answers` to `uncomparable`: both
segfault the oracle when the stem holds a tail, so the override is what
finally reached the crash the empty receiver hid.

That transition needed one rule change. The table's regression guard treats
`answers -> uncomparable` as a row losing its evidence, which it usually is;
here nothing in the tree changed and the receiver did. The guard now exempts
that transition for a pair named in `ORACLE_CRASHING_SENDS`, and only that
transition, so a row losing its evidence for any other reason is still red.

## (b) The instrument

`corpus/collection-arity.tsv`, re-derived at every task:

| | BASE | close |
|---|---|---|
| `agree` | 183 | 401 |
| `send-differs` | 56 | 31 |
| `setup-differs` | 193 | **0** |
| `exempt` | 1 | 1 |

**`setup-differs` is zero.** Every one of the 433 rows now has a receiver the
instrument can build, which is what the phase was for: at BASE, 193 of them
could not be measured at all because the class had no store.

All 31 `send-differs` rows are named, and none is a store gap:

| rows | what blocks them |
|---|---|
| `Stem`'s `allIndexes`, `allItems`, `makeArray`, `supplier`, and `difference`, `disjoint`, `intersection`, `putAll`, `subset`, `union`, `xor` built on them | the stem's own tail order, which is a different table geometry -- see Task 5 |
| `Stem~request`, `~toDirectory`, `~unknown` | spec section 7 puts them outside the protocol |
| `subset` on `Table`, `IdentityTable`, `StringTable`, `Directory`, `Properties` | `DO ... OVER` over an instance of a user class |
| `setMethod`, `unsetMethod` on `Directory` and `Properties` | `Method~new` |
| `Properties~save` | a `LIBRARY REXX` entry point |
| `Properties~setLogical` | `ARG` option `A` |
| `Stem~union`, `~xor` | `Array~copy`, which is a missing body rather than an aliasing bug -- Task 2 |

## (c) What moved, by diffing the table rather than by summing the reports

`corpus/method-bodies.txt` against BASE: **130 rows moved, 128 of them from
`loud` to `answers`**, and the other two are the `Stem` pair above.
**Nothing moved to `diverge`.**

| class | rows moved |
|---|---|
| `Bag` | 17 |
| `Directory`, `Properties`, `Relation`, `StringTable` | 16 each |
| `Set` | 13 |
| `IdentityTable`, `Stem`, `Table` | 12 each |

Whole-table verdicts: `loud` 345 to 215, `answers` 919 to 1047,
`unanswered` 71 unchanged, `diverge` 2 unchanged.

## What each task found that the plan did not have

* **Task 1** -- the store's GEOMETRY is observable. Iteration is bucket order,
  and the same eight keys answer three different orders depending on
  insertion order and initial capacity. Ported from the C++ and validated by
  simulation against four oracle cases before any Rust was written.
* **Task 1** -- one method identity serves seven classes, because `Setup.cpp`
  donates `IdentityTable`'s rows onward. Registering a body for `Table`
  registered it for `Bag`, `Set`, `Relation` and `Directory` too; the
  method-body table reported 38 rows regressing before the ownership guard
  went in.
* **Task 2** -- `copy` handed back an instance sharing the receiver's store,
  so `SetMixin~union` mutated its receiver. The same defect was already
  shipped for `Queue` and `List`.
* **Task 4** -- a `Directory` does not upper-case its keys and the entry
  family does, which settles the plan's question: the existing map is the
  environment's.
* **Task 4** -- `Class~enhanced` and `~defineMethods` walk a method table
  with the environment's reader, and three corpus programs went red the
  moment the table a program hands them stopped being one.
* **Task 5** -- the stem's tail order is a different geometry again, and four
  rows are left loud rather than shipped wrong.

Three of those five defects were found by a derived table rather than by a
witness, and the fourth by running the mixin rows the plan told each task to
run first.

## Still open, with an owner

* **`RexxQueue`** is the only surveyed class still wholly loud, and spec D93
  makes it Phase 7's.
* **The stem's tail table** -- four rows, named above.
* **`Array~copy`** -- a missing body, which two `Stem` rows wait on.
* **`Directory~setMethod`/`~unsetMethod`** -- the plan says `setMethod` is
  not a store operation and gets its own task if it does not fit; it did not
  fit.
