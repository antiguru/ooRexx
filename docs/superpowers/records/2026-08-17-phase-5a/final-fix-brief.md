# Final fix round -- the whole-branch review

Review: `.superpowers/sdd/2026-08-17-phase-5a/final-review.md`. **CHANGES REQUIRED**: 1 high,
4 medium, 4 low. This is the plan's last fix dispatch; one scoped re-review follows it and then I
adjudicate whatever remains.

Every finding is a cross-task one -- something no single-task diff review could see, because it
takes one task's mechanism and another task's new population.

## H1. Must fix: the REXX_DEFINED lock is not set on the library's own classes

I reproduced it. Three lines, both engines:

    say .Alarm~isA(.Comparable)
    .Alarm~inherit(.Comparable)
    say .Alarm~isA(.Comparable)

    oracle  rc 158, "0", then 98.985 User additions are not allowed to the REXX language classes
    crate   rc 0,   "0 0", stderr empty

**rc 0 with wrong stdout** -- the class this project treats as its worst, and the review reports the
mutation lands as well.

The shape: `rexx_classes::native_classes`'s `build()` sets the flag on every class it builds
(`native_classes.rs:391`-`:398`), which is the `Setup.cpp` half. The `.orx` half has no counterpart.
`Interp::bootstrap_library` closes the bootstrap with `remove_setup_methods` and nothing else, so
`Setup.cpp:1809`'s sibling is modelled and `RexxClass::liveGeneral`'s `setRexxDefined()` under
`PREPARINGIMAGE` is not. **The plan's Task 21 row names both halves of `liveGeneral`** and the
package half *was* carried across the boundary -- `.Comparable~package~name` and `.Alarm~package~name`
are `REXX` on all three interpreters. Only the flag half was missed.

Set it on the classes the bootstrap declares, at the point the bootstrap closes. Verify against the
oracle for a class from each embedded file and for a non-public one, and add a corpus row -- this is
a differential, so it is writable. **Then check the three sentences the review says assert the gap
does not exist**, and correct them; a fix that leaves its own contradiction standing is how this plan
has lost rounds before.

## M1-M4

* **M1** -- the `~publicClasses` refusal's stated ground is measurably gone, in two places. The
  reason it was refused no longer holds; decide whether the refusal or the reason changes, and say
  which.
* **M2** -- `rexx-bench-suite.rs:874`-`:878` **emits a report paragraph**, not a comment, saying
  "This crate has no `CoreClasses.orx` bootstrap yet (Phase 5), so it starts fast by not doing the
  work the oracle does at startup". Task 23 landed the bootstrap. The `Role::Offset` doc at
  `:134`-`:137` says the same. This is the Phase 5 D2 evidence and it now tells whoever runs the
  gate not to make the comparison. The reviewer measured it interleaved, three rounds, with the
  oracle side reproducing the roadmap's pinned median as its control: the crate is about 21 ms
  slower at cold start, still under D2's 50 ms image threshold, so **D2 resolves the same way but now
  on a measurement rather than a formality**. Fix the emitted text and the doc.
* **M3** -- the `DO OVER` `StringTable` order divergence is in no boundary list. It is a real
  licensed divergence and 5b/5c should be handed it.
* **M4** -- `::ATTRIBUTE EXTERNAL` is filed to 5a by the gate table and to Phase 7 by the
  interpreter's own refusal message. One of the two is wrong and they should agree; say which and
  why. Do **not** resolve it by re-filing the gate row to Phase 7 -- Task 24 already declined that,
  because it closes a gate row by narrowing what the gate covers.

## L1-L4

Four sentences Task 23 voided: two laziness justifications, an ownership sentence pointing at a task
that has since run, a "not reachable in this phase" whose premise changed, and
`class_graph.rs:231`-`:238`'s "Nothing inside this crate reads it". Take them from the review.

## Rules for this round

* **Prefer deleting to rewriting.** Every false sentence this plan shipped arrived as an added
  justification whose argument was right.
* **Commit before writing your report section**, not after.
* **Run all five gates at the commit you report**, with `--no-fail-fast`, and **quote the command
  that printed each figure beside the figure**.
* H1 touches `src/`, so a sitting is owed. The prose-only parts do not move codegen; if a commit is
  documentation only, prove it with the `.text` section hash across a forced rebuild.
* `cp` before any mutation and restore from the copy. Never `git checkout --`.

Write your report to `.superpowers/sdd/2026-08-17-phase-5a/final-fix-report.md`.
