# D59 was a temporary licence, and Phase 5i exits owing its removal

**Corrected 2026-09-08 by Moritz**, after this phase cited D59 as licensing a divergence:

> it was licensed as a temporary state to unblock the object model work without having weak
> references. it is fine to declare built-in classes with a static lifetime, but all others should
> be collected like regular objects. (no permgen). we can move the fix to after phase 5i, but before
> we start new work.

## What the record said

`docs/superpowers/specs/2026-08-27-phase-5b-instances.md:995` states it as a settled, permanent
decision -- "Class objects are not collected ... This is a decision about this implementation and it
is expected to outlive Phase 5" -- with **D59a** licensing four divergences on the strength of it and
**D60** building a termination-sweep `UNINIT` on the same premise. The spec now carries the
correction beside it; the original is left in place because its measurements are real.

## The decision

**Built-in classes may have a static lifetime. Every other class is collected like any other
object. No permgen.**

## The licence's premise expired inside this phase

D59 was taken because there were no weak references. **Phase 5i's Task 2 made `Body::WeakRef`
constructible and `rexx-core/src/heap.rs`'s weak protocol reachable from a Rexx program for the
first time** -- the protocol was written and unreachable before it. So the reason to hold the licence
went away in the phase that was still citing it, which is exactly the shape a temporary licence
fails in: nothing re-examines it when its premise changes.

## What it costs, and what it turns back into a defect

* **`~subclasses` holds strong references.** Phase 5i Task 5 shipped that row and recorded the
  strong-versus-weak difference as "the question does not arise under D59". **It arises.** The list
  must not retain a class the program has dropped.
* **A `WeakReference` to a class must clear.** D59a licensed it answering a dropped class.
* **`Interp::class_variables` is a permanent root**, so a class-scope instance variable retains
  what it holds for ever -- a real leak rather than a licensed one.
* **A class's `UNINIT` runs later than the oracle's**, which is D60's subject, reopened with D59.
* **The registry is monotone and class identities live outside the arena**
  (`crates/rexx-classes/`). Collecting user classes means moving those identities into the arena and
  giving up the monotone registry, which is structural work in that crate rather than a row-sized
  change.

## When

**After Phase 5i closes and before any new work starts.** Phase 5i does not fix it and does not
cite D59 as licensing anything from 2026-09-08.

## The wider lesson, which is not about classes

A licence taken to unblock work is recorded in the same voice as a decision reached on its merits,
and nothing in the record distinguishes them. D59 read as permanent because it *said* permanent, and
three phases built on it. **A temporary licence needs its expiry condition written into it** -- here,
"until weak references exist" -- so that the thing which retires it is a fact about the tree rather
than someone remembering.
