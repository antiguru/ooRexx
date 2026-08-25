## Task 2: the reading ledger

**Goal.** Commit `docs/superpowers/plans/phase-5-reading-ledger.md`: which authorities have been read
end to end, by whom, to which stopping point, at which revision.

**Why here.** The spec's enumeration has no denominator -- it is a hand-made list of mechanisms and no
check can say it is complete. The only instrument that has ever found a missing mechanism on this
project is reading an authority end to end, and two of the three things that instrument owes are
recorded in the spec as `not done`. Doing this before the row sets are frozen means a mechanism found
by reading can still become a row rather than a surprise.

**Read, and record the stopping point per row:**

* `fundclasses.xml`, `collclasses.xml`, `utilityclasses.xml`, `streamclasses.xml` -- **every `cls*`
  section's own prose**, not every `mth*`.
* `rexxpg/en-US/classes.xml` -- every section of "A Closer Look at Objects".
* `ootest/` -- **one row per 5a mechanism in the spec's enumeration**, naming the test group that
  pins it or recording that none does. **The stopping point is stated this way because the obvious
  phrasing has an empty referent, measured 2026-08-21: "every `testGroup` the spec's enumeration
  names" resolves to nothing, because the enumeration's `authority` column cites `provide.xml`,
  `dire.xml` and `fundclasses.xml` and never `ootest`.** The word `testGroup` occurs twice in the
  whole spec and neither occurrence is in an enumeration row; the spec's own "What I could not check"
  says "every `ootest` citation in the enumeration above is inherited", and there are none to inherit.
  That sentence is wrong, the spec is adopted and this plan does not edit it, so the correction is
  recorded here and belongs in the ledger this task commits.
  **Derive the mapping instead of reading a list that does not exist** -- which is more work than the
  original phrasing implied and is the work the row was always for: **this reading is what makes the
  three-signal rule's second signal checkable**, and until it exists no task may declare an oracle
  defect. A mechanism with no test group is a real and useful answer; record it as one rather than
  leaving the row blank.
* the C++ -- the bounded stopping point is **every `file:line` the spec cites, verified**. Of those
  sampled while the spec was written, four were wrong and two file-ambiguous.

**Also produced, because both are readings and neither has another home:**

* **the unexercisable-class list, re-derived rather than inherited.** The spec carries it as a
  carried-and-corrected open question: `Alarm` and `Ticker` stay unexercisable because their `init`
  reaches a native D37 defers, not because of `REPLY`/`GUARD`. An earlier claim that the list is
  smaller than the old spec assumed is unsupported. Derive it and name each member's reason.
* **the corrections.** A citation this reading finds wrong is recorded **in the ledger**, with the
  right one. This plan does not edit the adopted spec; the ledger is where a later reader learns a
  citation moved.
* **the deferral table, read as an authority rather than as implementation detail.** The spec asks
  for this by name: `RemoveMethod`/`HideMethod` reached the enumeration because a reviewer read
  `rexx-classes/src/native_classes.rs`'s `DEFERRALS` and asked what its deferred classes wait on, and
  the spec calls that "the only instrument that has produced a member of this class without someone
  reading an authority end to end". So the ledger carries a row per deferral: the concrete missing
  mechanism the row names, and either the enumeration row that owns it or the statement that none
  does. Every row in that table is a mechanism somebody hit and nobody enumerated, which is exactly
  the shape the reading instrument exists to find -- and it costs one file rather than a book.

**Verification.** The committed ledger, one row per authority with reader, stopping point and
revision -- `oodocs` at **r13198**, `ootest` at **r13178**, the C++ at its git revision. A row that is
`not done` is not a defect; **a row whose stamp is older than the checkout is**. That staleness check
is Task 3's, run against the same stamps, because it needs the checkout present and D56's docs-less
rule.

**What this task cannot see.** Its own denominator is a hand-made list of authorities, so it narrows
the hole the spec names and does not close it. Say that in the ledger's own header rather than in a
report nobody rereads.

**Done when** the ledger is committed, every authority row carries a stopping point and a revision,
and every `DEFERRALS` row is matched to the enumeration row that owns its mechanism or recorded as
matching none. No code, no sitting; the five gate commands still run.

---

