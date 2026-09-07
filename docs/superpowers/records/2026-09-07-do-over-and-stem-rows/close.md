# `DO ... OVER` and the three `Stem` rows -- close

Base `da02f3f1e`. Plan:
`docs/superpowers/plans/2026-09-07-do-over-and-stem-rows.md`.

**Half the goal landed.** The three `Stem` rows are done. `DO ... OVER` is
measured, specified and NOT landed: it was implemented, it agreed with the
oracle everywhere, and it was reverted because it introduces a
garbage-collection defect this crate has no correct place to fix yet. The
implementation is kept at `scratchpad/doover-wip/run.rs.diff` and the
measurements are in the plan.

## What landed

`corpus/collection-arity.tsv`: `agree` 422 to **425**, `send-differs` 10 to
**7**. `corpus/method-bodies.txt`: the same three rows, `loud` to `answers`.
Strict corpus 439 to **440**, adding
`lang/stem_request_and_directory.rex`.

All three `Stem` methods turn on one field: a stem has a VALUE of its own,
separate from its tails, and an unassigned stem's value is its own derived
name. `a.b = 1` then `a.~length` is **2**, not 0 -- measured, both engines
agreeing.

* **`request(class)`** upper-cases its argument; `'ARRAY'` answers the stem's
  `makeArray`, which for a `Stem` is its TAILS in the tail tree's post-order,
  and every other name is forwarded to the value. The missing argument is the
  POSITIONAL 93, not the named 88.
* **`toDirectory`** answers a `Directory` of one entry per tail that has a
  value. Its order is the DIRECTORY's, not the stem's: names go in in tail
  order and come back in the store's, which falls out of Phase 5h rather than
  needing anything here.
* **`unknown`** forwards the message and its arguments to the value.

One unit test changed rather than being fixed:
`a_receiver_with_no_class_here_is_loud` asserted that a `Stem` receiver is
loud "because this phase builds no class for it". `UNKNOWN` is now built, so
the premise is gone and the oracle's own answer replaced it.

## `DO ... OVER`: measured, correct, and reverted

The protocol is fully measured and recorded in the plan.
`RexxInternalObject::requestArray` is two paths keyed on `isBaseClass()`, and
the split is observable four ways -- a subclass of `Table` overriding
`makeArray` answers the override, a subclass of `Array` doing the same also
answers it (so `isArray()` is the PRIMITIVE test, not "an array or a
subclass"), a class overriding `request` answers from `request` with the
argument `ARRAY`, and `makeArray` runs exactly once per loop.

Implemented, it agreed with the oracle on every probe, and it closed **two
silent divergences** that are still open without it:

* `do e over` a two-line string iterates the two lines on the oracle and the
  whole string once here;
* `do e over .nil` is **98.913 at rc 158** on the oracle and iterates `.nil`
  once at **rc 0** here -- a wrong answer, not a missing one.

**Why it is reverted.** `DO OVER`'s items must stay reachable for the loop's
lifetime, and today they always are *through the target*: an array's items are
its own slots, and the engines root the target -- the tree-walker with a
clause-lifetime temp, the IR engine with a register. A `makeArray` result is a
DIFFERENT object that nothing roots. Under
`run_program_collect_every_alloc` the converted array's items are swept and
the loop binds a dead handle; `collect_stress` then panics rendering one.
Rooting it with `push_temp` at header-acceptance time, at `over_items` time,
per item, and array-plus-items were all tried and none holds under the IR
engine. `RootSet::park` would hold it, but its own doc reserves it for
`REPLY` and warns it may be deleted.

So the missing piece is a root whose lifetime is the loop's, and choosing one
is a design decision about the IR engine's loop-value lifetime rather than a
patch. That is the next session's, with the protocol already measured.

## How it was actually found, because I got it wrong repeatedly

I ruled on the mechanism four times -- the fresh array is unrooted, the items
are unrooted, the temps shift the IR register file, the pre-existing
`StringTable` path has the same latent bug -- and each was wrong, twice
producing a "fix" that changed nothing. What localised it was evidence:
a driver built outside the repo that named each program before running it,
then a prefix bisect **with a positive control** that the whole file still
failed.

The first bisect was worthless and looked conclusive: the prefixes cut off a
`::routine` the program calls, so every one of them errored early and every
one read as passing. Six "OK" lines, no information. Only adding the control
-- the untruncated file, which must fail -- exposed that.

The program was `do_over_string_table.rex`, and the line was
`do e over .resources`. **`.resources` in a program with no resources is not
a StringTable at all**: an unresolved dot-variable is the String
`.RESOURCES`, so that line is `DO OVER` a string and took the new path. The
isolated case fails only when the string is a heap object -- `'abc'` is
inline and passes, `.RESOURCES` is not and does not -- which is why the
existing corpus never saw it.

## Gates

The seven, over the commit.
