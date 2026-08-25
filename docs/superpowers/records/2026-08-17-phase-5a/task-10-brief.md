## Task 10: the send path -- `resolve`'s inputs, and D24's three surviving constraints

**Goal.** Open the send path's shape once, before three tasks in a row widen it.

**Build:**

* `resolve` gains the **caller's scope** and the **caller's package** (D27's amendment, D53). Task 13
  is what uses them; landing the signature here keeps Tasks 12 and 13 from rewriting each other.
* **D24's three constraints, which the spec carries as still-5a's and undesigned**: selectors interned
  at compile time; a `SmallInt` behaviour arm; a receiver in the calling convention. Each is a
  structural decision that is expensive to retrofit after `UNKNOWN`, the access scopes and the
  required-string protocol have all added call sites.
* **The roadmap's patchable-slot sentence** (`2026-07-27-rust-rewrite.md:492`, which the superseded
  spec cites as `:482`) claims the IR "founds OO dispatch for Phase 5 by making a call site a
  patchable slot". D28 sends nothing through that slot. Amend the sentence or record why it survives;
  do not leave it standing unexamined.

**Verification, and its weakness stated rather than hidden.** This task is meant to move **no row in
either table** and no corpus program. That is a weak differential: a green run is what a no-op also
produces. The instruments that can fail are therefore named individually --

* an in-crate test per constraint that fails when the constraint is dropped: selector identity after
  interning; the `SmallInt` arm being taken for a small integer receiver rather than the general path;
  the receiver arriving through the calling convention rather than being re-derived.
* `ir_dual` and the corpus, which pin that behaviour did not change.
* the sitting, which is this task's real acceptance: it is the one task in the plan whose subject is
  the shape of the hottest path.

**Done when** the three constraints have a failing-when-dropped test each, both tables' 5a row counts
are unchanged, and the sitting is recorded with `instructions:u` per axis. Sitting required, and at or
above 1% this task says whether it is the change or the layout with an interleaved control.

---

