# Phase 5g Task 9 — the rest of the review

Plan: `docs/superpowers/plans/2026-09-06-phase-5g-ordered-collections.md`.
BASE `5ed5ecbab`. Task 8 acted on the review's first four findings; this is
findings 5 through 17, re-derived against HEAD before being acted on, plus one
I found while auditing my own criterion.

---

## Fixed

**The equality protocol had the operands the wrong way round.**
`ArrayClass::findSingleIndexItem` is `item->equalValue(test)`
(`classes/ArrayClass.cpp:2094`): the value being **searched for** receives the
`==`, and the slot's contents is its argument. `same_item` sent it to the
element. Measured, every line reverses:

| | oracle | crate before |
|---|---|---|
| array of `'plain'`, `hasItem(.K~new)` | `1 1` | `0 .nil` |
| array of `.K~new`, `hasItem('plain')` | `0 .nil` | `1 1` |

`K` defines `::METHOD "==" return 1`. **Task 1's M2 witness cannot see this**:
it puts a `K` in and asks for a `K`, and both directions answer 1. This matters
beyond `Array` because Phase 5h's hash store asks the same question per key.

**A `Queue`'s index bound is two-tier, and I had made it one tier in Task 8.**
`putRexx` validates against the **allocated extent** first — past it is 93.918
— and only then calls `checkInsertIndex` for a position inside the extent but
past the last item, which is 93.966. Task 8 turned every past-`lastItem` index
into 93.966, which was a regression for the far tier. The extent is measured,
not guessed:

| receiver | boundary |
|---|---|
| `.Queue~new`, 1 item | 16 is 93.966, 17 is 93.918 |
| `.Queue~new(5)`, 1 item | 16 / 17 — a request below the default is floored at it |
| `.Queue~new(50)`, 1 item | 50 / 51 |
| `.Queue~new` grown to 20 items | 20 accepted, 21 is 93.918 |

So the extent is `max(requested, 16)` grown to fit the items --
`ArrayClass::DefaultArraySize` is 16 (`classes/ArrayClass.hpp:327`).

**`Array~fill` reached the receiver directly**, so it was the one row that
worked on an `Array` and refused on a subclass of one. Task 6's "one
resolution point for every Array-shaped receiver" was false, by exactly one
function.

**`next`/`previous` bailed on an out-of-range index.** Measured,
`.Array~of('a','b')~previous(5)` answers `2`; the crate answered `.nil`,
because the subscript failed to validate and the body returned early.

**The sort wrote its copy back over a receiver the comparator had changed.**
Upstream sorts in place, so a comparator that empties the array leaves it
empty; this crate restored the pre-sort contents. Guarded by comparing the
item count across the sort — **an approximation, and stated as one**: a
callback that swaps one item for another is not distinguished, and is not
measured.

## Fixed, but not witnessed — say so plainly

**Items are now rooted across the Rexx callbacks** in `ordered_pairs`,
`list_pairs` and `merge_sort`. The review panicked at `not_in_arena`'s
`"a live value"` on three such programs, driving
`run_program_collect_every_alloc` from its own binary, with two controls that
separated unrooting from allocation volume. That evidence is good.

**I could not reproduce it.** `corpus/lang/collection_callback_mutates.rex`
exercises the same shape — five searches whose callback empties the
collection, with items *built* rather than written as literals so nothing else
roots them, and the callback allocating twenty strings after emptying — and
removing the two roots leaves `collect_stress` **green**. Three attempts,
three greens.

So: the rooting stays, on the review's evidence; **nothing in this tree
witnesses it**, and the code comment says that rather than claiming a
measurement I do not have. The witness is not wasted — it is oracle-matching,
it covers callback-mutation behaviour, and it is what caught the sort
write-back above.

## Found by auditing my own criterion, and not this phase's to fix

```rexx
say (.nil == 'The NIL object')
```
Oracle `0`, this crate `1`, on both engines, and `=` likewise. No collection
involved. It is why `collection_item_equality.rex` carries a paragraph saying
which two lines it deliberately does **not** contain: they would be red for a
reason that is not that file's subject.

## Confirmed rather than changed

The review's findings 7, 8, 9, 10, 11, 13 and parts of 14 and 15 are real and
are **not fixed here**: `Supplier` bounds on its items array alone, `List`'s
`unsignedNumberValue` index conversion and its 93.918 for a bad one, `List`
handles re-issuing from zero after `empty`, `list_pairs` dropping entries that
hold nothing, `List~section` answering the subclass where the oracle answers
`List`, a subclass `init` that does not forward, the comparator result needing
`numberValue` with 26.903, the comparison **sequence**, `dimensions` going
stale after `append` on a zero-length array, and `Queue~put`'s argument
position in 93.907. Each is a row-level divergence with a repro in the
review's own directory; none is a crash or a memory-safety defect. **They are
listed here rather than fixed because fixing them is another phase's worth of
work, and leaving them named is better than leaving them found-and-forgotten.**

## Gates

Fast checks: `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` clean, `cargo test --release --workspace
--no-fail-fast` exit 0, 2124 passed, no suite failing. Strict corpus **423 of
423**. Four witnesses added: `collection_item_equality.rex`, `queue_extent.rex`,
`collection_callback_mutates.rex`, and `queue_bounds.rex` extended.

**G1** **G2** **G3** **G4** **G5** **G6** **G7**
