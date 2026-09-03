## Task 8 -- the two dark benchmark axes, and `identityHash`

**BASE:** `a7b72abea`. Tree clean, all five gates 0 there. Table C's 5c count is **110**, the floor;
table D's is **2**. **No row count needs to move in this task** -- both subjects are behaviour, not
rows.

**Read first:** `docs/superpowers/plans/2026-09-02-phase-5c.md` (the Global constraints gained four
entries this phase -- read them), and `rust/CLAUDE.md`.

---

## Part 1 -- the two dark benchmark axes

`bench-programs/alloc.rex` and `heapshape.rex` have **never produced a figure**. Measured at BASE:

```
alloc.rex      rc 120  method "OF" of class "Array" is not implemented
heapshape.rex  rc 120  a message send to a value that is not a hash collection
```

`.string~new` and `.directory~new` landed with Task 2, so `alloc.rex`'s other blocker is gone and
`heapshape.rex` now fails somewhere further in than it did. **Re-measure both before assuming
anything about what is left.**

**Why this matters more than it looks.** These are the **allocation and heap-shape axes**, and this
phase has landed a great deal of object model -- constructors for nineteen classes, a
`VariableReference` with a promotion path that leaks a cell per instance, a `Stem` value kind. Those
are exactly the axes that would show a regression, and they have been dark throughout. A phase that
closes without them has no measurement of its own largest risk.

**There is no pin-relative figure to be had**, and the done-when says so rather than asking for one:
the pinned `rexx-run-f558ea501` refuses both programs and `bench-baselines/phase-5b-arms.tsv` carries
neither axis. What you can produce is an **absolute figure at head**, plus the oracle's for the same
program, which is the comparison that means something.

## Part 2 -- `identityHash` renders wrongly, and silently

Measured at BASE by the controller, three descriptors, on a hashed binary:

```
s. = 1 ; o = s. ; say o~identityHash~length     crate 3   oracle 16
.Object~new~identityHash~length                 crate 3   oracle 16
```

**rc 0 and empty stderr on both sides, different stdout** -- a silent wrong answer, the class this
project treats as worst. It is pre-existing and not this phase's regression, but **three separate
tasks have now hit it** (Task 2's `identityHash` rendering note, Task 5's 65-pair observable sweep,
Task 7's stem sweep), which is enough noting and not enough fixing.

Match the oracle's rendering. Measure what it actually produces -- length, alphabet, and whether it
is stable within a run and across runs -- before choosing a representation. **If it turns out the
oracle's value is not reproducible in a way we can match** (see `oorexx-hash-iteration-order`:
identity-keyed things do not reproduce across runs), then the right outcome is a **licensed
divergence with its measurement**, not a guess that looks right. Say which you concluded and why.

## Done when

* Both bench programs are **rc 0** and their output matches the oracle on three descriptors, both
  engines -- or each remaining blocker is named with its measurement and its owner.
* An absolute figure exists for each of the two axes at head, with the binary's SHA-256 beside it.
* `identityHash` either matches the oracle byte for byte on both engines, or is a committed licensed
  divergence carrying the measurement that justifies it.
* The five gates are 0 and the corpus headline is at its full count.

## Rules

Every Global constraint in the plan binds you. The ones that bite here:

* **D74**: one sentence plus at most a short paragraph per doc comment. No results, no reasoning, no
  alternatives -- those go in your report.
* **Never delete anything, including your own scratch directory** -- the permission prompt cannot be
  answered from a subagent and will park you indefinitely while you still report as running. Name
  what you created; the controller sweeps.
* **`/tmp` is a RAM disk.** Small files there; any isolated build tree on real disk under
  `/home/moritz/dev/repos/claude-build-scratch/task-8/`. Check `df -h /tmp` first.
* **Separate revisions need separate target directories**, and every figure carries the SHA-256 of
  the binary that produced it, read at the time. One `CARGO_TARGET_DIR` gave two revisions the same
  hash this phase, which reads as a clean 1.0000x -- a comparison between a binary and itself.
* **Interleave any A/B comparison.** A non-interleaved run this phase gave 12/15 against 15/15 where
  interleaving gave 16/4 against 16/4 -- the difference was load, not the change.
* **Never wait with `pgrep -f`** -- it matches your own polling shell.
* **Run the three test gates with `--no-fail-fast`.** Gate 5 is slow; that is normal.
* **`directive_options.rs` has a known flake**, roughly 1 in 5, on `directive_options_trace_reply.rex`
  -- two threads trace concurrently and the *oracle's* interleaving varies. It is licensed and
  recorded in the plan. If you see it, it is not yours.
* Oracle probes from a **fresh empty directory**; three descriptors separately, never `2>&1`; both
  engines; exit status captured **immediately**.
* Never run a program in `rust/corpus/oracle-crashes.txt`. No `unsafe`. `oodocs/`, `ootest/` and the
  C++ tree are read-only.
* `git commit -F <file>`, paths named explicitly; `Cargo.lock` unstaged.

Report to `.superpowers/sdd/2026-09-02-phase-5c/task-8-report.md`. **Say plainly what you did not do.**
