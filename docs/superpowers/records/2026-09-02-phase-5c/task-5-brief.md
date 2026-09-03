## Task 5 -- the reflection classes

**BASE:** `0b8c9c7ce`. Tree clean, all five gates 0 at `989c307e2`, table C 5c count **233**.

**Read first:**
1. `docs/superpowers/plans/2026-09-02-phase-5c.md` -- **"The target"** and the Global constraints.
2. The spec's **"What table C actually measures -- the correction"**. M2-M6 there are SUPERSEDED.
3. `.superpowers/sdd/2026-09-02-phase-5c/task-1-report.md` -- built the construction route you use.
4. `.superpowers/sdd/2026-09-02-phase-5c/task-2-report.md` -- **its two unanticipated findings are
   the shape of your task**, and it is the task that measured why your four classes are hard.
5. `.superpowers/sdd/2026-09-02-phase-5c/task-3-report.md` -- how `RexxInfo` and `RexxContext` were
   moved through a Rexx-level route, which is the pattern nearest to yours.
6. `rust/CLAUDE.md`.

---

## Your subject -- 106 rows, the largest remaining block

```
Package 38    Class 31    Method 17    StackFrame 10    Routine 8    TraceObject 2
```

**These are not blocked on a bare `~new`.** Task 2 measured each and left them precisely because each
needs a real thing to reflect over: `Class` a class clone, `Method` a compiled executable, `Package`
a loaded package, `Routine` a compiled executable. Read its measurements before designing a route --
it did the work of finding out *why* each refuses, and repeating that is waste.

`StackFrame` is Task 3's leftover with a measured cause: the crate refuses both documented routes,
`.context~stackFrames` at rc 120 and `condition('O')` at rc 120 (it answers a `Directory`, which is
unimplemented). Task 3 also corrected itself here -- its first finding used `[1]` on what is a
`List`, and `condition('O')~stackFrames` **does** reach a `StackFrame` on the oracle. The route needs
a **trap**, not an expression, so its construction cell may need more than one clause.

**`Stem`'s 27 rows are deliberately not yours.** Task 2 built them and backed them out: a plain
instance renders `a Stem` where the oracle renders the stem's value, which is a silent wrong answer.
If your work makes them easy, say so and stop -- do not take them.

## What "done" means, and what it does not

A row agrees when the probe **constructs** and the class **answers the documented name set**. It does
**not** need the methods to work: `'abc'~hasMethod('abbrev')` is 1 while `'abc'~abbrev('a')` is rc
120, and that row agrees. **Do not implement method bodies.** If a row seems to need one, that is a
finding and a stopping point, not a licence -- `Alarm` and `Ticker` are already parked for exactly
that reason.

**A route that answers the wrong object is worse than no route.** Task 2's `Stem` is the worked
example. Check what your route's object *renders as*, not only its `~class~id`.

## Done when

* Each of the six is either `covered` through a committed construction route with its probe **rc 0
  and byte-identical to the oracle on three descriptors, both engines**, or is recorded as blocked
  with the measurement showing why.
* Table C's 5c not-yet-agreeing count reported **before and after** (233 at BASE). The phase's floor
  is 100; if you land all six it should read 127.
* Any class you moved went through Task 1's construction route, not around it.
* Anything you had to leave is named with its owner, so the residual set stays fully attributed.

## Rules

Every Global constraint in the plan binds you -- **read them, they gained four entries this phase**.
The ones that bite here:

* **D74**: one sentence plus at most a short paragraph per doc comment. No results, no reasoning, no
  alternatives -- those go in your report.
* **The scratchpad is a RAM disk.** Small files there; **any isolated build tree on real disk under
  `/home/moritz/dev/repos/claude-build-scratch/task-5/`**. Check `df -h /tmp` first.
* **Never delete anything, including your own scratch directory** -- the permission prompt cannot be
  answered from a subagent and will park you indefinitely while you still report as running. Name
  what you created; the controller sweeps.
* **Kill what your probes start and confirm with `kill -0`** if anything schedules.
* **Never wait with `pgrep -f`** -- it matches your own polling shell.
* **Re-derive every generated artifact you invalidate**; `corpus/refusal-sites.tsv` cites source line
  numbers and has shipped stale once. Derive it from the failing test's own printed scan.
* **Run the three test gates with `--no-fail-fast`.** Gate 5 is genuinely slow -- tens of minutes per
  binary in a debug build. That is normal, not a hang.
* Oracle probes from a **fresh empty directory**; three descriptors separately, never `2>&1`; both
  engines (`REXX_ENGINE=ir`, `REXX_ENGINE=tree-walker`); exit status captured **immediately**.
* Never run a program in `rust/corpus/oracle-crashes.txt`. No `unsafe`. `oodocs/`, `ootest/` and the
  C++ tree are read-only.
* `git commit -F <file>`, paths named explicitly; `Cargo.lock` unstaged.
* Once you report and go idle, the tree is no longer yours unless handed back.

Report to `.superpowers/sdd/2026-09-02-phase-5c/task-5-report.md`. **Say plainly what you did not do.**
