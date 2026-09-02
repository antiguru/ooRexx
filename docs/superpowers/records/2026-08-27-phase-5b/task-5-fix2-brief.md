# Task 5, second fix round -- brief

**Do not start until `task-2` has committed and the tree is clean.** This touches
`corpus/phase-5b.txt` and `coverage.rs`, which `task-2` is editing.

One finding, IMP-A from `task-5-fix-review.md`, verified independently by the controller and refined.
Everything else in that review is either already closed or a minor recorded below.

## The gap

`Interp::run_termination_uninits` in `crates/rexx-exec/src/dispatch.rs` carries a copy of the
`processing_uninits` interlock. It is load-bearing, and **no committed row can see it** -- all nine
`corpus/lang/uninit_*.rex` plus `corpus/gate-tables/concepts/obdes.rex` produce byte-identical output
on both engines with it removed and with it intact.

**The interlock is two halves and only one is load-bearing.** Establish this yourself before you fix
anything, because the distinction is what makes the witness possible:

* removing the early-return guard `if self.processing_uninits { return Vec::new(); }` **alone**
  changes nothing -- not the ten rows, and not the probe below, which catches the other half. No
  witness has been found for this half. That is "no witness found", not "unreachable"; nobody has run
  it to a conclusion, and deciding which it is belongs to this round.
* removing the assignment `self.processing_uninits = true` is what lets the *nested* ready-sweep run,
  and it produces a silent wrong answer: rc 0, empty stderr, wrong stdout, on both engines.

## The witness to add

The controller's probe, which reproduces the split. It is `uninit_nested_collection`'s question asked
of the **termination** path: the outer finalizer is reached because the program ends, not because a
program-level `GC('force')` readied it. Measured by the controller against the oracle, ten runs, one
distinct stdout, rc 0, empty stderr:

```
start / end / outer fin / inner built / inner ran inline? 0
```

with the interlock intact the crate matches on `ir` and `tree-walker`; with the assignment removed
both engines answer `inner ran inline? 1`.

```rexx
/* The same question as uninit_nested_collection, asked of the TERMINATION
   sweep instead: G's finalizer is reached because the program ends, not
   because a program-level GC('force') readied it.  Inside it, K's instance
   is made unreachable and a collection driven; the interlock must still
   refuse to run K's finalizer inline. */
say 'start'
g = .G~new
say 'end'
exit 0

::class k
::method uninit
  n = .K~mark
::method mark class
  expose ran
  ran = 1
  return 1
::method ran class
  expose ran
  if var('ran') = 0 then return 0
  return ran

::class g
::method uninit
  say 'outer fin'
  o = .K~new
  say 'inner built'
  y1='a'; y2='b'; y3='c'; y4='d'; y5='e'; y6='f'
  y7='g'; y8='h'; y9='i'; y10='j'; y11='k'; y12='l'
  drop o
  call gc 'force'
  say 'inner ran inline?' .K~ran
```

Take that as a starting point, not as the row to commit: name it, comment it to this project's
standard, and satisfy yourself it is the smallest program that asks the question. Add it to
`corpus/phase-5b.txt` and to `EXPECTED_SUBSET_5B` in `coverage.rs` **in the same commit**.

## What you must prove about it

1. **It reddens.** Remove the assignment, rebuild, confirm the new row fails. Then restore and
   confirm it passes. Transcript for both directions.
2. **It adds coverage rather than merely being able to fail.** Run that same mutation against the
   suite *without* your new row and confirm nothing else catches it. The controller has already
   measured this for the ten UNINIT rows; extend it to the corpus.
3. **D61 holds for it.** Ten oracle runs, one distinct stdout, empty stderr. The row must print only
   *whether* the inner finalizer ran inline, never *when* -- the oracle answers "when" two ways
   across runs, which is exactly why `uninit_nested_collection` does not print it.
4. **The guard half.** Either find a witness for it, or state plainly that you looked and did not,
   and say what you ran. Do not delete it on the strength of having found no witness.

## Also in this round, both cheap

* **IMP-B is already done and is not yours.** The controller appended the missing gate status to
  `task-5-fix-report.md` as a clearly-marked note on 2026-08-31, leaving the implementer's words
  intact. Do not redo it.
* **MIN-B, a stale name.** `Heap::clear_uninit` survives at `plans/2026-08-27-phase-5b.md:398` and
  `:410` and `specs/...-instances.md:447`, and `ClassRegistry::uninit_classes_in_sweep_order` at
  `plans:474`, after the round renamed both. Verify each line still says what the review says before
  changing it.
* **MIN-C, a rustdoc warning the gates do not see.** `check_uninit`'s `[`Self::uninit_classes`]` now
  resolves to the private field since the accessor was renamed: 8 warnings at `d708491a7`, 9 at
  `9c0007723`. Re-measure both revisions yourself.

## Standing constraints

All of `global-constraints.md`. Oracle probes from a fresh empty directory; three descriptors read
separately, never `2>&1`; both engines; nothing from `corpus/oracle-crashes.txt`; no `unsafe`. Five
gates from `rust/` at the end, each status read unpiped and never chained with `&&`, plus the
phase-gate command. Message the controller when you finish.

## Three more minors from the same review

* **MIN-A, an argument not carried by the control it cites.** The round's report argues
  `uninit_class_inherit_runtime.rex` is not redundant against `uninit_class_inherited.rex`, but no arm
  it ran separates them -- its own C3 reddens both, and so does the reviewer's equivalent. The
  conclusion is right and the reviewer supplied the missing control rather than only flagging it:
  `sweep.reverse()` before the stable `sort_by_key` reddens `uninit_class_inherit_runtime` **alone**
  of the ten. Re-run that yourself and put it in the report as the row's non-redundancy evidence,
  replacing the arm that does not separate them.
* **MIN-D, an inference the log cannot support.** The void gate-5 log is said to show binaries that
  "completed while the tree was pristine", but it has 36 `Running` lines against 35 `test result`
  lines and carries no timestamps at all, so it cannot date any binary against the corpus edit.
  Either drop the claim or replace it with something the log can actually carry.
* **MIN-E, prose overtaken by the fix.** `plans/2026-08-27-phase-5b.md:481` still says "silent wrong
  answers on this crate today" when both arms now agree, and the round updated one crate line to a
  state that was true only between `d708491a7` and `b4266cbd6`. Check the whole neighbourhood rather
  than these two lines: on this project a fix round's prose reliably rots one sentence further than
  the one anybody names.
