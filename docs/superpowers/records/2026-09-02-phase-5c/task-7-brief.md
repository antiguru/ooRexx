## Task 7 -- `Stem`, the last 5c-addressable block

**BASE:** whatever HEAD is when you start (`git log --oneline -1`); confirm the tree is clean first.
Table C's 5c count is **137** and the phase's floor is **110**, so these 27 rows are the whole of
what is left to move.

**Read first:**
1. `docs/superpowers/plans/2026-09-02-phase-5c.md` -- **"The target"** and the Global constraints
   (four were added this phase; read them rather than skimming).
2. The spec's **"What table C actually measures -- the correction"**.
3. `.superpowers/sdd/2026-09-02-phase-5c/task-2-report.md` -- **it built `Stem` and backed it out.**
   Read why before designing anything.
4. `.superpowers/sdd/2026-09-02-phase-5c/task-1-report.md` -- the construction route you would use.
5. `rust/CLAUDE.md`.

---

## Your subject, measured by the controller at `82a8ae86d`

```
s. = 'dflt' ; o = s. ; say 'a' o          crate  rc 120  after printing "a dflt"
                                          oracle rc 0    "a dflt"
  then o~class~id                         crate  rc 120  a message send to a stem is not implemented
                                          oracle "Stem"
  then s.1 = 'one' ; o[1] ; o~at(1)       oracle "one one"

o = .Stem~new ; say 'a' o ; say o~class~id
                                          crate  rc 120  method "NEW" of class "Stem"
                                          oracle rc 0    "a" (empty) then "Stem"
```

**This is not a missing constructor, and that is the whole point of the task.** Two things are
missing and the second is the larger:

1. `.Stem~new`.
2. **A `Stem` object model.** Its string value is the stem's *default value*, not `a Stem` -- that is
   the silent wrong answer Task 2 found and backed out over. And *any* message send to a stem is
   refused today, so `~class~id`, `[]`, `~at` and the rest all stop at one gap.

A stem also aliases a live variable: `s.1 = 'one'` after `o = s.` is visible through `o[1]`. Whatever
you build has to keep that, so a snapshot copy is the wrong shape.

## Scope, and an explicit stopping point

**Move the 27 rows** -- `stem__instance.rex` byte-identical to the oracle on three descriptors, both
engines -- and no more.

**If this turns out to need a general stem object model rather than a bounded one, stop and report
rather than half-building it.** That is a real possible outcome and it is a good one: 27 rows is not
worth a rushed object model, and the phase closes at 137 perfectly well with `Stem` named as a
handover beside `StackFrame`. Say what you found, what it would take, and stop. **Do not** implement
method bodies to move a row -- a row agrees when the probe constructs and the class answers the
documented name set, not when the methods work.

## Done when, either way

* Either `stem__instance.rex` is rc 0 and byte-identical to the oracle on three descriptors on both
  engines, and table C's 5c count reads **110**; or `Stem` is recorded as a handover with the
  measurement showing what it needs, and the count still reads 137.
* If you moved it: the string-value question is answered by a committed program, not by prose --
  `say o` for a stem with a default and for one without, both compared to the oracle.
* If you moved it: the aliasing question likewise -- a write to `s.1` after `o = s.` seen through `o`.
* Count reported **before and after**.

## Rules

Every Global constraint in the plan binds you. The ones that bite here:

* **D74**: one sentence plus at most a short paragraph per doc comment. No results, no reasoning, no
  alternatives -- those go in your report.
* **Never delete anything, including your own scratch directory.** The permission prompt cannot be
  answered from a subagent and will park you indefinitely while you still report as running -- it
  cost three hours this phase. Name what you created; the controller sweeps.
* **`/tmp` is a RAM disk.** Small files there; any isolated build tree on real disk under
  `/home/moritz/dev/repos/claude-build-scratch/task-7/`. Check `df -h /tmp` first.
* **Never wait with `pgrep -f`** -- it matches your own polling shell's command line.
* **Rebuild before any sweep that follows a revert or a mutation run.** `cargo test --release`
  relinks `target/release/rexx-run`, so a mutated binary can outlive its revert with `git status`
  clean; two tasks lost a sweep to this. Read the binary's SHA-256 and state it beside any figure.
* **Re-derive every generated artifact you invalidate**; `corpus/refusal-sites.tsv` cites source line
  numbers and has shipped stale once. Derive it from the failing test's own printed scan.
* **Run the three test gates with `--no-fail-fast`.** Gate 5 is slow -- tens of minutes per binary in
  a debug build. Normal, not a hang.
* Oracle probes from a **fresh empty directory**; three descriptors separately, never `2>&1`; both
  engines (`REXX_ENGINE=ir`, `REXX_ENGINE=tree-walker`); exit status captured **immediately**.
* Never run a program in `rust/corpus/oracle-crashes.txt`. No `unsafe`. `oodocs/`, `ootest/` and the
  C++ tree are read-only.
* `git commit -F <file>`, paths named explicitly; `Cargo.lock` unstaged.
* Once you report and go idle, the tree is no longer yours unless handed back.

Report to `.superpowers/sdd/2026-09-02-phase-5c/task-7-report.md`. **Say plainly what you did not do.**
