## Task 2 -- surface A, the constructors

**BASE:** `486b53da9`. Tree clean, all five gates 0 at that commit, corpus 331 of 331.

**Read first:**
1. `docs/superpowers/plans/2026-09-02-phase-5c.md` -- the premise paragraph and Global constraints.
   This is the SECOND version of the plan; the first had a false premise.
2. The spec's **"What table C actually measures -- the correction"** section. M2-M6 there are
   SUPERSEDED; do not build from them.
3. `.superpowers/sdd/2026-09-02-phase-5c/task-1-report.md` -- Task 1 built the construction route you
   will use, and its report says what it did not finish.
4. `rust/CLAUDE.md`.

---

## Your subject

**The constructors behind the 30 rc-120 probes.** Table C's 5c not-yet-agreeing count is **815** at
BASE (down from 868 after Task 1's 53). Your job is the largest remaining block.

The distinct `~NEW` blockers, from sweeping all 96 probes:

```
Bag  Class  Directory  EventSemaphore  IdentityTable  List  Message  Method
MutableBuffer  MutexSemaphore  Package  Queue  Relation  RexxContext  Routine
Set  Stem  StackFrame  String  Supplier  Table  VariableReference  WeakReference
```

**`Pointer` is deliberately not in that list.** It is `unreachable` under D73 -- the reference says
instances come only from native code -- and **must not get a constructor**. Its probe names `~new`
because the row set expects the refusal.

**Four of the names above have no `~new` at all** -- `RexxContext`, `StackFrame`,
`VariableReference` and `RexxInfo` -- and are **Task 3's**, not yours. They raise `93.967` or `97.1`
and are reached by a Rexx-level route instead (`.context`, `.context~stackFrames[1]`, `(>x)`,
`.RexxInfo`). Leave them; if you touch one, say so and why.

So your list is the remaining **19**.

## What "done" means here, and what it does not

A row agrees when the probe **constructs** and the class **answers the documented name set**. It does
**not** require the methods to work -- `'abc'~hasMethod('abbrev')` is 1 while `'abc'~abbrev('a')` is
rc 120, and that row agrees. **Do not implement method bodies.** If you find yourself writing one to
move a row, stop: either the row does not need it, or you have found something the plan is wrong
about, and the second is worth reporting.

## Known blocked, do not fight these

Task 1 left `Alarm` and `Ticker` `not-covered` and they are **not yours to unblock**: `Alarm~init`
needs the `+` operator applied to a user-class instance and `Ticker~init` needs `String~sign`, both
of which are method-body work this phase does not own. If another class turns out to need a method
body the same way, treat it the same and record it rather than reaching for the body.

## Done when

* Each of the 19 constructors' probes is **rc 0 and matches the oracle on three descriptors, both
  engines** -- or is recorded as blocked with the measurement that shows why.
* `Pointer` still refuses, and you say what you ran to confirm it.
* Table C's 5c not-yet-agreeing count is reported **before and after** (815 at BASE).
* Any class you moved to `covered` went through Task 1's construction route, not around it.

## Rules

Every Global constraint in the plan binds you. The ones that bite here:

* **D74**: one sentence plus at most a short paragraph per doc comment. No results, no reasoning, no
  alternatives -- those go in your report. `964dc68ba` is the worked trim.
* **Re-derive every generated artifact you invalidate.** `corpus/refusal-sites.tsv` cites
  `crates/rexx-exec/src/*.rs` by line number; `486b53da9` exists because a doc-comment-only change
  moved 41 of those and shipped a red gate for two commits. `fmt`, `clippy` and one test binary do
  not catch it.
* **Run the test gates with `--no-fail-fast`.** Without it `cargo test` halts at the first failing
  binary and everything sorting after it is unmeasured.
* Oracle probes from a **fresh empty directory**; three descriptors read separately, never `2>&1`;
  both engines (`REXX_ENGINE=ir`, `REXX_ENGINE=tree-walker` -- `tree` alone is rc 2 and reads like a
  fast success); capture exit status immediately, not after an intervening command.
* `/bin/grep -a` for counts over data; a `grep -c` over Rust source misses multi-line entries.
* Gate-table report output goes to **stderr**, and `cargo test` captures it without `-- --nocapture`.
* A gate run does not survive the turn that starts it. Tree hash before the first gate and after the
  last.
* No `unsafe`. `oodocs/`, `ootest/` and the C++ tree are read-only.
* `git commit -F <file>`, paths named explicitly; `Cargo.lock` unstaged.

Report to `.superpowers/sdd/2026-09-02-phase-5c/task-2-report.md`. **Say plainly what you did not do.**
