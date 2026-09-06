# Phase 5g Task 5 — `List`

Plan: `docs/superpowers/plans/2026-09-06-phase-5g-ordered-collections.md`.
BASE `c1bc7c912`.

Twenty-seven rows, and unlike `Queue` none of them was already registered.

---

## Why `List` shares nothing

Measured before any code, and it is the whole reason `Queue` and `List` are
separate tasks:

```
.Queue~new~allItems   ->  a message send to a value that is not an array
.List~new~allItems    ->  method "ALLITEMS" of class "List" is not implemented
```

The first is `Array`'s body reaching a `Queue` and finding no store; the second
is no body at all. Upstream says the same: `Setup.cpp` gives `Queue`
`InheritInstanceMethods(Array)` and gives `List` twenty-two `ListClass::*`
entry points of its own.

## The index is a handle

Every other ordered collection here is addressed by where an item sits. A
`List` is addressed by a token that stays with the item as the list is
inserted into and deleted from, and **an implementation that makes it a
subscript passes every single-element test and fails the first insert.**

Measured: appending `a`, `b`, `c` answers `0 1 2`; inserting after the first
answers a fresh `3` and leaves every old handle reaching the item it always
did; removing the second leaves the first and third valid.

**Handles are recycled, last-freed-first.** Removing handles 1 and then 3 from
a five-item list makes the next three appends answer **3, 1** and a fresh
**5** — so the free list is a stack, not a queue, and not "the lowest free
handle" either. That is one measurement, it is in the witness, and mutation M9
is exactly it.

The store is three pool entries — the items, their handles, and the stack of
freed handles — under the `List` class as scope, which is `Supplier`'s and
`Queue`'s arrangement and traced by the collector for the same reason.

## Four defects the refresh caught and the witness did not

`method_bodies.rs` refused the first refresh with six regressed rows:

| row | oracle | this crate, before |
|---|---|---|
| `~at` with nothing | 93.903 **argument 2** | 93.903 argument 1 |
| `~put` with nothing | 93.903 **argument 2** | 93.903 argument 1 |
| `~insert` with nothing | **rc 0**, answers `0` | 93.903 |
| `~empty` | answers the receiver | no result, 91.999 |

**The argument number is two whatever the position is.** `ListClass` passes
`ARG_TWO` to `validateIndex`, `requiredIndex` and `validateInsertionIndex`
alike (`classes/ListClass.cpp:349`, `:381`, `:454`), so a bare `~at` reports
`argument 2 is required` even though `at` takes one argument. Nothing but the
oracle would have said so.

And on `put` it is the **index** that is required, not the value: measured,
`l~put(, handle)` succeeds and leaves the entry holding nothing.

`~empty` answering the receiver is the same defect Task 1 fixed on `Array`,
found the same way, in a body written after that fix. The lesson did not
transfer because the witness called `~empty` as a statement again.

## Witness and red controls

`corpus/lang/list_operations.rex`, filed in all three places, and extended
with the four cases above after the refresh found them. Strict corpus:
**416 of 416 matching**.

**M9 — the free list pops the front instead of the back**, making it a queue.
Predicted red on the handle-reuse line. **Confirmed**: 415 of 416.

**M10 — `list_position` reads the handle as a position.** Predicted red.
**Confirmed**: 415 of 416. This is the mutation that proves the witness tests
the property the class is about.

## What moved

`corpus/method-bodies.txt`: **28 rows, all `List`**, all `loud` -> `answers`.
`corpus/collection-arity.tsv`: `agree` 145 -> **183**, `setup-differs`
193 from 231.

## Gates

Fast checks: `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` clean, `cargo test --release --workspace
--no-fail-fast` exit 0, 2124 passed, no suite failing.

**G1** **G2** **G3** **G4** **G5** **G6** **G7**
