# Re-review of the final-review fix round — shared constraints

Range: `e64202ae7..cf92ff4fb` (eleven commits, F1-F11). Worktree
`/home/moritz/dev/repos/ooRexx-rust-rewrite`, branch `plan/rust-rewrite`.

Inputs:
* the brief the implementer worked from: `.superpowers/sdd/2026-09-14-phase-8/final-fix-brief.md`
* the controller's rulings during the round: the last sections of
  `.superpowers/sdd/2026-09-14-phase-8/progress.md` from "Fix dispatch pre-flight" on
* the implementer's report: `.superpowers/sdd/2026-09-14-phase-8/final-fix-report.md`
* the review findings being fixed: `final-review-a-boundary.md`, `final-review-b-integration.md`
  in the same directory
* the implementer's probes and mutants: `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/final-fix/`

## You are read-only, and a gate run is live

* **The controller's phase gates are running in this worktree's `rust/target/` right now.** Do not
  build into `rust/target/`, do not edit, create, delete or format any tracked or untracked file in
  the worktree, and run no `git` command that writes. Build only in your own copy: `git archive
  cf92ff4fb rust | tar -x -C <your scratch>`, with `interpreter/`, `api/`, `build/`, `extensions/`,
  `ootest/` symlinked beside it read-only, and `CARGO_TARGET_DIR` inside your scratch. `-j 4`.
* Write only your report file in `.superpowers/sdd/2026-09-14-phase-8/` and scratch under
  `scratchpad/<your slice name>/`. Do not dispatch subagents.

## Hazards

* The C++ tree, `build/`, `samples/`, `ootest/`, `oodocs/`, `api/`, `testbinaries/` are read-only;
  no extension under `build/` is ever rebuilt (forged extensions in your own scratch are fine).
* Oracle: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
  from a **fresh empty directory**; stdout, stderr, exit status as three files, never `2>&1`.
* Read `rust/corpus/oracle-crashes.txt` before unusual probes; never run its entries. Never
  `NUMERIC DIGITS` above 1000, never `.Package~new` on a repository file, never
  `::OPTIONS TRACE ?<letter>`.
* No `cd X && ...; rest` with paths built from variables. `/bin/grep -a` for counts.

## Rules a finding may cite

* `unsafe` only in `rust/crates/rexx-api/src/ffi.rs` and `src/load.rs`, each block with a
  `SAFETY:` note naming the invariant and who establishes it; none in `tests/`.
* No new dependency beyond `libloading`. No process-global state.
* Byte-identical stdout, stderr, exit status against the oracle.
* Comments minimal; never state a set's size; no em-dashes; every citation lands on its subject
  when printed.

## What a re-review is for

1. **Each named finding is closed**, by running its original probe (the review reports name them)
   against a build of `cf92ff4fb`, not by reading the diff.
2. **Assume this round broke something.** Fix rounds on this project have introduced a defect or a
   false statement more often than not. Hunt regressions in the neighbourhood of each change, and
   new false statements in comments, doc comments, commit messages and the report.
3. **Every control the report claims**: re-run at least the ones that are the only witness of a
   fix, with your prediction written first.

## Report contract

Write the report file first and append as you go. Each finding: severity, file:line, what, the
command and its output, run or inferred. Say per original finding: closed / not closed / partly.
End with "Not reached". When done, reply with the report path, counts by severity, and one line per
Critical or Important. Writing the file alone does not reach the controller.
