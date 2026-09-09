# Phase 5j Task 4 — the negative controls

Four mutations, each run. The first two were written as predictions before running; the last two
were observed during development, and are recorded as what they were rather than dressed up as
planned controls.

## A — the expunge removed from `collect_now`

**Predicted:** `a_class_nothing_refers_to_is_collected` and
`an_instance_keeps_its_class_alive_and_nothing_else_does` redden; the `UNINIT` witness stays green,
because the finalizer path does not depend on the expunge; `refusal-sites` may redden purely from
the line shift, which would be unrelated.

**Result: CONFIRMED**, and exactly those two reddened. The `UNINIT` witness stayed green.
`refusal-sites` did **not** fire — the removed block does not sit above any line the table names,
so that part of the prediction was not observable rather than confirmed.

## B — a program's class allocated immortal again

**Predicted:** all three witnesses redden, the `UNINIT` one because the finalizer moves back to the
termination sweep.

**Result: CONFIRMED**, all three and nothing else.

## C — `Interp::flag_class_uninit` absent

Observed while building the task, not planned: with a class collectable and nothing flagging it in
the heap, `class-uninit-at-driven-collection` printed **`before / after`**. The finalizer did not
run at all, because `rexx-classes`' pending list held a handle to a freed slot. This is what made
Task 2 keep classes immortal.

## D — `run_one_uninit`'s drop from the pending list absent

Also observed rather than planned: the finalizer ran **twice**, `before / class uninit / after /
class uninit`. A class is on two lists — the heap's flag, which a collection readies, and
`rexx-classes`' pending list, which the termination sweep drains — and running it from one must
remove it from the other.

## What the controls do not cover

Nothing here witnesses a class that is collected *while* a `WeakReference` names it, because the
weak reference is not a root and the class dies at the same collection; `weakref_class.rex` covers
the live-class direction only. Stated rather than left as a gap for someone to assume closed.
