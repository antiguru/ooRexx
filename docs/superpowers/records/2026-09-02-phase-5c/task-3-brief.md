## Task 3 -- surface C, the classes with no `~new`

**BASE:** the commit at the tip when you start (`git log --oneline -1`); the tree is clean and all
five gates are 0 there. Table C's 5c not-yet-agreeing count is **280**.

**Read first:**
1. `docs/superpowers/plans/2026-09-02-phase-5c.md` -- the premise paragraph and Global constraints.
2. The spec's **"What table C actually measures -- the correction"**. M2-M6 there are SUPERSEDED.
3. `.superpowers/sdd/2026-09-02-phase-5c/task-1-report.md` -- it built the construction route you
   will use.
4. `.superpowers/sdd/2026-09-02-phase-5c/task-2-report.md` -- **read its two unanticipated findings
   in full.** They are the shape of your task too: matching the oracle's refusal makes a probe
   byte-identical without moving a single row, because a `not-covered` group reads `unanswered` by
   design. Rows move only through Task 1's construction route.
5. `rust/CLAUDE.md`.

---

## Your subject

**`RexxInfo` `RexxContext` `StackFrame` `VariableReference` -- 57 rows.** All four raise on `~new`
and all four have a working Rexx-level route instead. Measured on the oracle, rc 0:

```
.context~class~id                 ->  RexxContext
.context~stackFrames[1]~class~id  ->  StackFrame
.RexxInfo~class~id                ->  RexxInfo
x = 5 ; say (>x)~class~id         ->  VariableReference
```

and their `~new` answers:

```
.RexxContext~new        rc=163  93.967 NEW method is not supported for the RexxContext class.
.StackFrame~new         rc=163  93.967   (same)
.VariableReference~new  rc=163  93.967   (same)
.RexxInfo~new           rc=159  97.1   Object "a RexxInfo" does not understand message "NEW".
```

Three of the four are already byte-identical to the oracle on all three descriptors -- their probes
match the refusal. **That is why their rows have not moved**, and why your job is the construction
route, not the refusal.

## These are NOT D73 `unreachable`, and the distinction is written down

`crates/rexx-extract/src/docs/classes.rs`'s `UNCONSTRUCTIBLE` already carries all four as
`Status::NotCovered`, on the rule stated in its own doc comment:

> A row whose sentence says the user cannot construct one **and names a Rexx-level route** to obtain
> one makes a different claim; those are `not-covered`.

**Read that doc comment before classifying anything.** `Buffer` and `Pointer` are the `unreachable`
pair and are not yours.

## What to be careful about

* **A route that returns the wrong thing is a silent wrong answer.** Task 2 built `Stem`, found a
  plain instance answers `a Stem` where the oracle answers the stem's value, and **backed it out**.
  Check what your route's object renders as, not just its `~class~id`.
* `.context` and `.context~stackFrames[1]` are evaluated **inside the probe**, so what they describe
  depends on where the probe stands. A route whose answer changes with the probe's own shape will
  not compare stably. Prefer one whose answer does not.
* Do **not** implement method bodies to move a row. If a row seems to need one, that is a finding.

## Done when

* Each of the four is either `covered` through a committed construction route with its probe **rc 0
  and byte-identical to the oracle on three descriptors, both engines**, or is recorded as blocked
  with the measurement showing why.
* Table C's 5c not-yet-agreeing count reported **before and after** (280 at BASE).
* Any route you commit is shown to construct -- Task 1's `OracleShape::Exactly` will catch one that
  does not, and saying you relied on that is fine, but say it.

## Rules

Every Global constraint in the plan binds you. The ones that bite here:

* **D74**: one sentence plus at most a short paragraph per doc comment. No results, no reasoning, no
  alternatives -- those go in your report.
* **Re-derive every generated artifact you invalidate**, `corpus/refusal-sites.tsv` especially: it
  cites `crates/rexx-exec/src/*.rs` by line number and has shipped stale once this phase
  (`486b53da9`). Re-derive it from the failing test's own source-scan output, not by shifting numbers
  by hand.
* **Run the three test gates with `--no-fail-fast`**, or everything after the first failure is
  unmeasured.
* Oracle probes from a **fresh empty directory**; three descriptors read separately, never `2>&1`;
  both engines (`REXX_ENGINE=ir`, `REXX_ENGINE=tree-walker` -- `tree` alone is rc 2 and reads like a
  fast success); capture exit status immediately, not after an intervening command.
* **Never run a program in `rust/corpus/oracle-crashes.txt`.** It gained an entry this phase --
  `Message~result` before the send blocks the oracle indefinitely.
* `/bin/grep -a` for counts over data; a `grep -c` over Rust source misses multi-line entries.
* Gate-table report output goes to **stderr**; `cargo test` captures it without `-- --nocapture`.
* A gate run does not survive the turn that starts it. Tree hash before the first gate and after the
  last. No `unsafe`. `oodocs/`, `ootest/` and the C++ tree are read-only.
* `git commit -F <file>`, paths named explicitly; `Cargo.lock` unstaged.

Report to `.superpowers/sdd/2026-09-02-phase-5c/task-3-report.md`. **Say plainly what you did not do.**
