# Phase 5g Task 8 — the corrections a review found after the close

Plan: `docs/superpowers/plans/2026-09-06-phase-5g-ordered-collections.md`.
BASE `d4d5e5c31`, which is Task 7's close plus its gate reading.

**Task 7's close was premature.** A review run against `df753e529` — over
which all seven gates were 0 — found shipped divergences in `Array` and
`Queue` that no witness, no derived table and no gate could see. This task is
the correction, and the reason the defects survived is worth stating before
the list: **every witness this phase wrote appends to a dense array**, and a
dense array is the one case where the wrong rule and the right one agree.

---

## The defect, which is one defect

`ArrayClass::append` counts from `lastItem`, the last **occupied** index
(`classes/ArrayClass.cpp:726`), and `ArrayClass::empty` sets that field to
zero (`:672`). This crate counted from the slot count. Measured on the oracle
against the crate as it stood:

| receiver, then `append` | oracle | crate |
|---|---|---|
| `.Array~new(5)` | `1`, size 5 | `6`, size 6 |
| `.Array~of('p','q','r')` after `remove(3)` | `3`, size 3 | `4`, size 4 |
| `.Array~of('p','q','r')` after `empty` | `1`, size 3 | `4`, size 4 |
| `.Array~new` with `[5]` set, then `remove(5)` | `1`, size 5 | `6`, size 6 |
| `.Array~new(4)` with `[2]` set | `3`, size 4 | `5`, size 5 |
| `.Array~of('p','q','r')` (dense) | `4` | `4` |

Only the last agrees, and only the last had a witness.

`insert` counts from the same place, and shifts into slack the array **already
had** rather than growing: `.Array~new(4)~insert('j')` leaves size 4 where a
plain `Vec::insert` leaves 5. The distinction is slack the array had *before*
the insert, not slack the insert created — `.Array~of('x','y')~insert` still
grows to 3, and getting that backwards was the first fix's own bug, caught by
`array_structure.rex` going red.

## What it did to `Queue`

`queue` appends, so the same rule: a queue emptied after holding two items
started again at slot 3 here and slot 1 upstream, which left `peek` and `pull`
reading a hole and answering `.nil`.

```
q~queue('a'); q~queue('b'); q~empty; q~queue('z')
    oracle: items 1 size 1 peek z pull z
    crate:  items 1 size 3 peek The NIL object pull The NIL object
```

**Task 4's report says `.CircularQueue~new(3)` "now works end to end". That
sentence was false past the first `empty`**, and the same probe run before an
`empty` agrees, which is why it read as true.

Two more, both from the same review:

* **`Queue~size` is the item count.** `Setup.cpp:789` maps it to
  `ArrayClass::itemsRexx`, not to `sizeRexx`. This crate answered the slot
  count.
* **A `Queue`'s insertion index is bounded by its last item.**
  `QueueClass::checkInsertIndex` (`classes/QueueClass.cpp:113`) raises
  **93.966** `Incorrect queue index` when the position is past it, and its own
  comment says why: "the position must be location of an existing item within
  the bounds of the queue, unlike an array which can insert at empty slots or
  beyond the existing bounds". Both `insertRexx` and `putRexx` call it. This
  crate extended instead, and where it did refuse it raised 93.918.

The check is class-dependent and the store cannot carry it: a `Queue` and a
user subclass of `Array` are both instances holding a store, so `is_queue`
asks the class graph. Registering a separate `Queue~INSERT` row does not work
— it shares a `MethodId` with `Array`'s, and the later row in the slice wins,
which is how that was found.

## A new oracle crash, four shapes

A `List` entry may hold nothing — `insertRexx` requires no value and `putRexx`
requires only the index, both measured during Task 5 — and `removeItem`,
`hasItem` and `index` all dereference each entry's value without checking.
SIGSEGV, rc 139, 4 runs of 4, from either way of making the empty entry.

Filed in `corpus/oracle-crashes.txt`, same class as the `Stem~index` entry this
phase filed earlier. **This crate answers all four**, so they are rows where a
differential cannot be run at all rather than rows where the two sides agree.
No upstream ticket.

## Witnesses

`corpus/lang/array_append_index.rex` and `corpus/lang/queue_bounds.rex`, filed
in all three places. Between them they carry every row of the table above, the
`empty`-then-`queue` sequence, the item-count `size`, and both 93.966 refusals.
Strict corpus **420 of 420 matching**.

## The part that should change how the next phase is reviewed

**Neither derived table moved a single row for any of these corrections.**
`method-bodies.txt` sends no arguments, so it never builds a trailing hole;
`collection-arity.tsv` sends one real argument list per row, so it never
builds a *sequence*. Both were green over every defect above, before and
after.

The phase's own instrument was built to replace a verdict column that agreed
about arity errors, and it is better than that column — but it shares the
column's shape: **one send to a fresh receiver**. Every defect in this task
needed two sends, or one send to a receiver with a history. That is the gap,
it is not closed, and a witness is the only thing in this tree that currently
covers it.

## Gates

Fast checks: `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` clean, `cargo test --release --workspace
--no-fail-fast` exit 0, 2124 passed, no suite failing. `collect_stress` green.

**G1** **G2** **G3** **G4** **G5** **G6** **G7**
