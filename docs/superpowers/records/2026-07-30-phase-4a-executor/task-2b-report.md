# Task 2b report: `RootSet` cannot express an unset slot

Status: **DONE.** Commit `4ce5aae8`, "Let a slot be cleared, so DROP on a simple
variable has somewhere to go". BASE `4b63b94d`.

Tests: `rexx-core` **41 passed / 0 failed** (was 38, three added); workspace
**622 passed / 0 failed / 3 ignored**; clippy clean workspace-wide; fmt clean.
Every exit status read directly rather than through a pipe.

## Step 1: the failing test, and the measurement behind it

Measured first, because the whole task rests on `ObjRef::NIL` not being able to
stand in for "unset", and that is an oracle question rather than a design
preference. `( ulimit -v 1048576; build/bin/rexx … )`:

```rexx
a = 5
drop a
say a        /*  A                */
x = .nil
say x        /*  The NIL object   */
y = .nil
drop y
say y        /*  Y                */
```

The third case is the one that settles it, and it is the case a shorter probe
would have missed. `x = .nil` shows `.nil` is a value with its own rendering;
`drop y` on a variable *already holding* `.nil` goes back to the derived name.
So the two states are observationally different and a bare `ObjRef` slot has no
spare value left to mean "no value". That is the same argument D16 used when it
rejected storing slots in `temps`, arriving at `set_slot` instead.

Two tests written before any implementation existed, and they failed to compile
on `no method named clear_slot`, which is the correct red state:

* `a_cleared_slot_is_unset_and_differs_from_one_holding_nil` — sets one slot to
  a value and another to `ObjRef::NIL`, clears both, asserts both read `None`
  and that the `.nil` one read `Some(ObjRef::NIL)` beforehand.
* `a_cleared_slot_stops_being_a_root` — collects with the slot set (nothing
  swept), clears, collects again, asserts the value is swept and
  `heap.get(v)` is `None`.

## Step 2: `clear_slot`, and why not an `Option`-taking `set_slot`

Both were viable. I chose `clear_slot` beside `set_slot`, and the reason is
about call sites rather than about `roots.rs`:

* `DROP` is a construct the language has. A Task 9 call site spelling it
  `clear_slot(frame, i)` says what it means; `set_slot(frame, i, None)` reads
  at a glance like a caller that forgot to compute a value, which is a real
  hazard in a file where every other write is a value.
* The `Option` shape is **already visible on the read side**, since `slot`
  returns `Option<ObjRef>`, so keeping the common write monomorphic hides
  nothing.
* Every existing `set_slot` call stays untouched. The alternative was a
  mechanical `Some(...)` wrap across the crate for no gain, and mechanical
  edits are where a wrong one hides.

The reasoning is in `clear_slot`'s doc comment, as the task asked, together
with the oracle transcript.

Clearing an already-unset slot is a no-op rather than an error, because `DROP`
on a never-assigned variable is legal Rexx and does nothing.

### The half a naive wrapper gets wrong, and proof the test catches it

`iter` already filters on the `Option`, so writing `None` makes a cleared slot
stop being a root for free. That is easy to assert and hard to trust, so I
built the wrapper that gets it wrong and ran the suite against it: a `cleared:
HashSet<usize>` beside the storage, consulted by `slot`, with `slots` and
`iter` left alone.

```
test a_cleared_slot_is_unset_and_differs_from_one_holding_nil ... ok
test a_cleared_slot_stops_being_a_root ... FAILED
test result: FAILED. 12 passed; 1 failed
```

So the first test **passes** against the broken implementation and only the
second catches it. That is worth recording precisely: a suite containing only
the obvious test — "does the read answer unset?" — would have shipped an object
kept alive by nothing, invisible until a collection happened to land later at
an unrelated moment. The task text predicted this failure mode; this is the
demonstration that the test written for it actually fires.

## Step 3: the neighbours

**`grow_slots` never recycles a cleared index**, and I have pinned it with a
test rather than leaving it as an observation about the current implementation.

It is a requirement, not a nicety. A cleared slot still *belongs to its name*:
the plan maps an upcased name to an index once, and `DROP a` empties `A`'s slot
rather than retiring it, so a later `a = 1` must land back in the same place.
If growth recycled that index for the next name it allocated, two variables
would alias one slot and the symptom would be one variable's assignment
silently changing another's value. `grow_slots` appends at
`slots.len() - frame.start` and clearing writes in place without shrinking, so
the indexes cannot collide today; `growth_does_not_recycle_a_cleared_slot`
is what keeps that true.

**`pop_slots` needs no equivalent.** It truncates `slots` to `frame.start`, so a
cleared slot leaves with its frame and cannot be observed afterwards. There is
no interaction to guard.

Neither neighbour's existing invariant changed: the two `should_panic` tests
for non-top-frame `grow_slots` and `pop_slots` still pass untouched.

## Step 4: verification

All from `rust/`, at `4ce5aae8`, each exit status read directly:

* `cargo test -p rexx-core` -> **41 passed, 0 failed**, exit 0. Three tests
  added to the 38 that were there.
* `cargo test --workspace --no-fail-fast` -> **622 passed, 0 failed, 3
  ignored**, exit 0.
* `cargo clippy -p rexx-core --all-targets -- -D warnings` -> exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` -> exit 0.
* `cargo fmt -p rexx-core -- --check` -> exit 0, after one reflow that `cargo
  fmt` applied to a new `assert_eq!` of mine. Folded into the same commit
  rather than split out, because it touches only lines this task added, so
  there is no pre-existing formatting churn for it to hide.

Staged exactly two paths, both `rexx-core`. `rexx-exec` (Task 5) and
`rexx-parse` (under review) were not touched.

## What this does and does not unblock

**Unblocks** plain `DROP a` on a simple variable in Task 9, where the read
afterwards must yield the derived name, and 4b's `NOVALUE`, which needs the
same distinction to fire on.

**Was never blocking Task 5**, and that remains true: a stem's `DROP` replaces
the object with `default: None, tails: {}`, which is observationally identical
to never-touched, measured as `drop x.` leaving both `x.1` and `x.` rendering
derived names. Task 5 needs nothing from this commit.

One thing a later task should not have to re-derive: `Novalue` in `rexx-exec`
already returns "was this set?" alongside the value on the read path, per D16's
instruction not to retrofit it. `clear_slot` is the write-side counterpart, so
the two halves of `NOVALUE` now both exist before 4b needs either.
