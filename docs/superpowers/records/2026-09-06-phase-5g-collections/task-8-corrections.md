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

Seven gates over `8aa53c8c3`, from `scratchpad/gates-5g8.status`: G1 0, G2 0,
G3 0, G4 0, G5 0, G6 0, G7 0, `failed-suites=0` on each suite-running gate.

---

## What I checked myself, after the review

The review's findings above were re-derived before being acted on. These are
the rest of its brief, checked here rather than taken:

**Every report sentence the review was asked to attack is CONFIRMED**, on both
engines against the oracle, in one program (`claims.rex` in the verification
directory): `Queue~remove` shifts and `Array~remove` leaves a hole at the same
size, `Array~delete` shifts; `Array~empty` and `List~empty` both answer the
receiver; a section of a `Queue` is a `Queue`, of a `List` a `List`, of an
`Array` an `Array`; `of` answers the receiver's own class on all three; and
`sort` is not numeric -- `.Array~of(10,9,2,100,1)~sort` is `1,10,100,2,9`.

**"Exactly four methods refuse a multi-dimensional receiver" is CONFIRMED, and
the check that confirms it is not the obvious one.** Sending all 32 documented
`Array` instance methods to a `2x3` receiver, the oracle raises on seven, and
both engines match it on all 32. Five of the seven raise `93` and two raise
`98`, so counting by major code would give the wrong answer. By decimal:
`APPEND`, `INSERT`, `DELETE` and `SECTION` raise **93.954**; `PUT` raises
**93.925** (not enough subscripts) and the two sort names raise **98.975**
(the receiver was sparse). Four, as claimed.

**The instrument has a blind spot, and it is currently empty.** Its
`every_row_is_sent_something_its_arity_needs` check can only see rows whose
upstream arity is a number, so the twelve rows sent nothing whose body is Rexx
-- `CircularQueue`'s `makeArray size sort stableSort supplier`, `List` and
`Queue`'s `sort`/`stableSort`, `Supplier`'s `allItems allIndexes supplier` --
are held only by the oracle-completes-the-send rule. Measured: the oracle
refuses an extra argument on all ten that can be sent one, so every row in the
hole genuinely takes none. **The hole is real and nothing sits in it**; closing
it would mean asserting that refusal per row.

## A pre-existing divergence this phase made reachable

Found while auditing the blind spot above, and it is **not this phase's and
not the collections'**:

```rexx
call one
say 'returned'
exit
one: procedure
  signal on syntax name oops
  v = .DateTime~new~addYears('notanumber')
  say 'accepted'
  return
oops:
  say 'refused' rc
  return
```

Oracle rc 0, `refused 88` then `returned`. This crate, both engines: **rc 168
and no output at all.** The same send with the trap at the top level rather
than inside a subroutine matches on both sides.

**A condition raised inside a library (`CoreClasses.orx`) Rexx method does not
unwind to a `SIGNAL ON SYNTAX` trap in an outer subroutine frame.** It is not
`FORWARD` -- a user class forwarding inside a subroutine traps correctly -- and
it is not the collections: `DateTime~addYears` and `TimeSpan~fromHours`
reproduce it and both predate this phase. `CircularQueue~string` and
`~makeArray` reach it too, which is how it was found, and that is the whole of
this phase's relation to it.

**No witness is added for it**, because a witness would be red. It is recorded
here with a repro that names no collection, so whoever owns library frames can
take it without reading this phase.
