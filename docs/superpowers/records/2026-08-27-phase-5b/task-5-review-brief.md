# Task 5 review brief

Review `c65b51641..d708491a7` on branch `plan/rust-rewrite` in the worktree
`/home/moritz/dev/repos/ooRexx-rust-rewrite`. Four commits: `3ff1055de` (the deliveries and the order
rule), `cf85bfd67` (the sweep made linear), `eda2a0b94` (citation corrections and corpus/README.md),
`d708491a7` (comment rewrapping).

Diff package: `.superpowers/sdd/2026-08-27-phase-5b/review-c65b51641..d708491a7.diff`

## Read first

* `docs/superpowers/specs/2026-08-27-phase-5b-instances.md`, especially D59, D59a, D60, D61, D69
* `docs/superpowers/plans/2026-08-27-phase-5b.md`, Task 5's section (edited by this task)
* `.superpowers/sdd/2026-08-27-phase-5b/task-5-brief.md` and `task-5-report.md`
* `.superpowers/sdd/2026-08-27-phase-5b/global-constraints.md` and the 5a constraints it points at
* `rust/CLAUDE.md`

## Two hard constraints on how you work

1. **The worktree is read-only to you.** A debug corpus gate is running in it; a write of yours
   voids it. The only file you may create is `.superpowers/sdd/2026-08-27-phase-5b/task-5-review.md`.
   No `git add`/`commit`/`checkout`/`stash`. To test a mutation, `git archive <rev> | tar -x` into
   scratch gives a mutable copy -- the Task 1 reviewer used exactly that to run both controls.
2. **Set `CARGO_TARGET_DIR` to your own scratch directory on every cargo invocation**, so you do not
   block on the gate's target lock.

## What to check

**The order rule is the task's biggest claim and the controller has already verified its citations**
-- `HashCollection.hpp:130` `DefaultTableSize = 17`, `new_identity_table()` using it,
`HashContents::iterateNext` walking buckets in ascending index order with the chain first, and
`RexxClass::getHashValue` returning `id->getHashValue()`. **Do not re-verify those; verify the
implementation matches them.** Does this crate's sweep actually reproduce `strhash(class id) % 17`
ascending with entry order as the tie-break, or does it reproduce the *transcripts* the report
quotes? Construct class-id sets the report does not use and predict the order before running.

**D61.** The task claims D61 lapses over class-against-class order because the oracle's order is now
characterised, and stays in force over anything involving an instance. Check every new corpus program
against that: does any of them depend on an order the oracle does not reproduce? The instance side is
measured at 19/20 vs 1/20, which is exactly the shape D61 exists for.

**The five controls.** The report gives each a corpus figure and a reduced arm. Re-run at least C1,
C3 and C5 yourself in a scratch copy. C3 matters most: it claims `obdes` stays green under a sweep in
entry order, i.e. the task's own gate row cannot see the ordering rule -- if true, the ordering
witnesses are load-bearing and must be checked hard.

**The quadratic fix (`cf85bfd67`).** The report claims 49.55s -> 2.48s on a 200,000-instance program
and 1.27s vs 1.28s for the same program with no `UNINIT`, i.e. the ordinary path untouched. Re-time
it. **The machine is busy; interleave your timings rather than running one build then the other.**

**Silent wrong answers.** This task turns a silent absence into a delivery, and delivery is
order-visible and timing-visible. Probe past the rows: `UNINIT` raising, `UNINIT` on a subclass whose
parent also defines one, an object resurrected by its own `UNINIT`, `UNINIT` during a collection
inside a `UNINIT`, and a class whose `UNINIT` is defined after instances exist.

**The divergence left open.** The report records a new one -- the oracle's `SaveStackSize = 10` save
stack holding recent allocations out of a driven collection, so with 0..8 padding clauses the oracle
answers `a c uninit` and this crate `a uninit c`. It is written into the plan's Task 6 for a
decision. Confirm the measurement and the boundary at 9, and say whether you agree it is a collector
decision rather than a `UNINIT` one.

**Quality.** `rust/CLAUDE.md`'s comment rule of 2026-08-27; no comment states the size of a set;
ASCII only, no em-dashes, no historical framing. Check rustdoc intra-doc links resolve
(`/bin/grep -n` the item name). Check the plan and `corpus/README.md` edits for factual claims,
especially universal quantifiers -- record the exact pattern beside any negative claim you verify.

## Output

`.superpowers/sdd/2026-08-27-phase-5b/task-5-review.md`. Spec compliance verdict; quality verdict
(Approved / Approved with comments / Changes requested); findings as Critical / Important / Minor
with file:line, what is wrong, and the fix. Quote the command beside every figure. Say which
acceptance criteria you re-measured and which you could not.

Write the file first and append as you go. Message the controller when you finish.
