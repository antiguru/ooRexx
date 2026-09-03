## Task 4 -- the `>x` silent divergence

**BASE:** `b4adf8a1e`. Tree clean, all five gates 0 there, table C 5c count **237**.

**Read first:**
1. `.superpowers/sdd/2026-09-02-phase-5c/task-3-report.md` -- it found this and could not fix it
   within its own scope. Its measurement is your starting point, not your conclusion.
2. `docs/superpowers/plans/2026-09-02-phase-5c.md` -- Global constraints.
3. `rust/CLAUDE.md`.

---

## Your subject

**`>x` evaluates to the referenced variable's value instead of to a `VariableReference` object.**
Confirmed by the controller, both engines, three descriptors read separately:

```rexx
vr = 5
o = >vr
say o~class~id
```

```
crate  ir           rc 0   stdout "String"              stderr empty
crate  tree-walker  rc 0   stdout "String"              stderr empty
oracle              rc 0   stdout "VariableReference"   stderr empty
```

**rc 0 on both sides, empty stderr on both sides, different stdout. This is a silent wrong answer** --
the defect class this project treats as the worst, because nothing in any harness goes red for it.
It dates to `8b87195bc` (2026-08-03), so it is not this phase's regression; this phase merely walked
into it.

## What matters here beyond the fix

**Find the edges before you build.** `>x` is a *variable reference*, and the interesting questions
are what the oracle does with one once it exists -- assignment through it, `~value`, `~name`, passing
it to a routine, `>x` over an uninitialised variable, `>x` over a stem or a compound, and `>x` in an
argument position versus an assignment's right-hand side. Measure each on the oracle before deciding
what the object has to be. Task 3 measured only `~class~id`.

**A wrong object is worse than a refusal.** If some part of the surface cannot be made right in this
task, a loud refusal for that part is the correct outcome and a plausible-looking wrong object is
not. Say which parts you made right.

**Look for siblings.** A silent divergence rarely sits alone. `8b87195bc` is the commit that
introduced this behaviour -- read what else it touched, and check whether the same shape produces a
silent wrong answer anywhere near it.

## Done when

* `>x`'s answer matches the oracle byte for byte on three descriptors, both engines, across the
  edges you measured -- or the parts you did not implement refuse loudly and are listed.
* **A corpus witness exists** under `rust/corpus/` that would have caught this: it must fail before
  your fix and pass after. Run it both ways and record both.
* You say what `VariableReference`'s four table C rows now do -- moving them is not required, and
  Task 3's report explains why they may not move.
* Any sibling divergence you find is recorded with its transcript, whether or not you fix it.

## Rules

Every Global constraint in the plan binds you. The ones that bite here:

* **D74**: one sentence plus at most a short paragraph per doc comment. No results, no reasoning, no
  alternatives -- those go in your report.
* Oracle probes from a **fresh empty directory**; three descriptors read separately, never `2>&1`;
  both engines (`REXX_ENGINE=ir`, `REXX_ENGINE=tree-walker` -- `tree` alone is rc 2 and reads like a
  fast success); capture exit status **immediately**, not after an intervening command.
* **Never** use `/usr/bin/time -v timeout ... <binary>` to judge parked vs spinning -- it reports
  `timeout`'s rusage and prints 0.00 for a program burning a core. Sample `/proc/<pid>/stat` fields
  14/15 with the binary launched directly.
* Never run a program in `rust/corpus/oracle-crashes.txt`. Never set `NUMERIC DIGITS` above 1000.
* **Re-derive every generated artifact you invalidate.** `corpus/refusal-sites.tsv` cites
  `crates/rexx-exec/src/*.rs` by line number and has shipped stale once this phase (`486b53da9`);
  derive it from the failing test's own printed scan, never by shifting numbers by hand.
* **Run the three test gates with `--no-fail-fast`.**
* No `unsafe`. `oodocs/`, `ootest/` and the C++ tree are read-only.
* A gate run does not survive the turn that starts it. Tree hash before the first gate and after the
  last. `git commit -F <file>`, paths named explicitly; `Cargo.lock` unstaged.
* Once you have reported and gone idle, the tree is no longer yours unless it is handed back.

Report to `.superpowers/sdd/2026-09-02-phase-5c/task-4-report.md`. **Say plainly what you did not do.**
