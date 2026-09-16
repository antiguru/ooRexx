# Task 1 report: the survey, before anything moves

**Status: complete.** No source file was read-modified. `git status --porcelain`
was empty before and after the survey work; the only paths this task adds are
its own two documents.

## Where the survey is

`.superpowers/sdd/2026-09-15-file-split/task-1-survey.md` is the live copy, and
the committed one is
`docs/superpowers/records/2026-09-15-file-split/task-1-survey.md`.

The brief said to commit the survey at its `.superpowers/` path. That path
cannot be committed: `.gitignore:30` ignores `.superpowers/`, confirmed with
`git check-ignore -v`. The 2026-08-15 ruling recorded in
`docs/superpowers/records/README.md` is that the live `.superpowers/sdd/` stays
ignored and the record is committed under
`docs/superpowers/records/<plan-basename>/`, and the Phase 8 plan's records were
committed there in slices during execution rather than only at close. So the
committed copy went to the records tree, with the live copy left where the
skill's resume logic looks for it. This report is committed the same way, and
for the same reason the README gives: a report that lives only in an ignored
directory is what the ruling exists to prevent.

Nothing else in the tree was touched. `progress.md` and `task-1-brief.md` remain
the controller's to copy across.

## Method

Two enumerations, both with their commands in the survey.

1. **By length**, the plan's own `find | wc -l | awk '$1 > 1000'`, re-run at
   BASE `ec7824e4a`. Its src/tests decomposition reproduces the plan's figures
   at `5d84dd8cb`, so nothing crossed the trigger in between.
2. **By responsibility**, over every `.rs` from 400 lines up: module doc header
   plus a column-0 item outline, read file by file. That is what found the
   candidates the trigger misses and the files where a split would hurt.

A third measurement turned out to matter more than either: splitting each
candidate's line count into its inline `#[cfg(test)] mod` span and the rest. It
reorders the list and changes the recommendation for a large part of it, because
several files trip the trigger only on their test module.

**One measurement was wrong before it was right.** The first test-span script
accepted a `#[cfg(test)]` on a *function* as the start of a test module and
scanned forward to the next `mod`, which gave `lib.rs` 5713 test lines instead
of 976 and put four other files out by hundreds. Every figure the corrected
script disagreed with was checked back against `grep -n '^#\[cfg(test)\]'` on
the file itself before the table was written. Every command quoted in the survey
was then re-run with the machine's plain `grep` and reproduced the figure
printed beside it.

## Coverage

The length enumeration is in the survey's table, one row per candidate, every
one read. The responsibility enumeration covered every `.rs` from 400 lines up;
of those under the trigger, the ones whose doc header named more than one
subject were opened and checked, and the survey names what each turned out to
be.

## Top three recommended splits

1. **`rexx-exec/src/dispatch.rs`, 10710 lines**: the object model and the
   `resolve`/`invoke` seam are one thing; `MutableBuffer`, the `Object` and
   `Class` protocol natives, the array natives, the constructors and the
   required-string protocol are each another. Proposed children follow the
   pattern the file already uses for `string`, `collection`, `hash` and
   `package`.
2. **`rexx-exec/src/run/tests.rs`, 8328 lines**: 285 tests in one flat file,
   already grouped by construct. Nearly as much relief as Rank 1 at the lowest
   risk in the plan, because no production item moves.
3. **`rexx-exec/src/run.rs`, 9229 lines**: one `impl Interp` from 764 to 8300
   holding CALL, the condition traps, RAISE/SIGNAL, the loops, SELECT,
   INTERPRET, TRACE and the settings instructions, plus the pure branch
   arithmetic and the `raised_*` constructors below it.

## What later tasks assume that does not hold

Full list in the survey's closing section. The ones that change a task:

* **Task 8 cannot do what it says for `ffi.rs`.** "only by moving safe code out"
  has no safe code to move: the `#[cfg(test)]` stub region and `mod tests` both
  contain `unsafe` in exactly the forms `unsafe_sites.rs` matches (15 and 6
  occurrences, measured with that test's own predicate). Any child file holding
  them turns the test red, and `mod tests` alone would leave 848 lines anyway.
  Recommend dropping `ffi.rs` from the task.
* **The plan's D-U1 statement is incomplete**: it omits `rexx-core/src/bytes.rs`
  from the `unsafe` uses list and `rexx-core/src/lib.rs` from the opt-in list.
* **Three things are pinned to `src/dispatch.rs` that the plan does not
  mention**: `mod seam`; `method_is_protected`, whose occurrences must both sit
  in a path containing the literal `dispatch.rs`, which a `dispatch/` child does
  not; and the one `seam::clear(` call inside `Interp::invoke`.
* **`collect_now` is pinned to `src/lib.rs`** by name and indentation, which
  Task 4 must plan around.
* **`refusal-sites.tsv` column 3 is path-derived**, not just column 4. It stays
  stable across a dispatch split only because `/dispatch/` carries the same tag
  as `dispatch.rs`; that is worth stating in the task rather than relying on.
* **Two records the plan does not name** hold a source path:
  `corpus/docs/class-set.txt` and `corpus/docs/class-methods.txt` each carry a
  `module is src/docs/classes.rs` header, compared in both directions by
  `tests/extract_docs.rs`.
* **Task 10's list is mostly leaves.** `rexx-num/tests/format.rs`,
  `coverage.rs`, `native_classes_wiring.rs`, `method_bodies.rs` and
  `ir_recorded.rs` are the plan's own "one set of test cases for one function"
  exception. `rexx-bench-suite.rs` and `gate_table_c.rs` are the ones worth
  doing.
* **Task 5's `error.rs` and `activation.rs` come back leave-or-nearly.**

## Raised rather than folded in

`ir_recorded.rs`'s `LOOP_CASES` and `BRANCH_CASES` are inline `const` case
tables in Rust source, lines 60 to 688. A recorded preference of Moritz's is
that inline-case tables belong in a data file the test reads. That is a
behaviour-visible change, outside this plan's pure-move architecture, so it is a
note for the controller rather than a proposal here.

## Ordering suggestion

The plan runs `dispatch.rs` first, which is its riskiest file and the one where
the performance constraint bites hardest. `run/tests.rs` and
`rexx-bench-suite.rs` move no production code and would exercise the four
instruments, the `cargo doc` pass and the interleaved performance procedure
before `dispatch.rs` has to depend on all of them working.
