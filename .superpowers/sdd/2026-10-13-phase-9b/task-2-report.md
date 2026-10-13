# Task 2 report

Commits: a0abcc77c (file + test), then the gate record commit (see git log).

* `rust/corpus/suite-9b-groups.txt`: 18 groups, header carries the exact grep command. Derived from
  `/usr/bin/grep -rlawiE` over ootest/ooRexx (20 hits, 19 `.testGroup`; `SysFileCopyMove.sh` is not a group and
  is dropped), minus `base/rexxutil/Macrospace` (every test has a Phase 10 Rxapi row).
* Test `gate_groups_9b::the_9b_group_file_is_what_its_derivation_gives` in `tests/suite.rs` re-derives in Rust
  (ASCII case-insensitive whole-word match, all regular files incl. dotfiles/binary, Phase 10 = every expected
  test has an Owner::Phase10 row) and also asserts the header records the command built from the ROUTINES const.
  `a_word_match_is_grep_dash_w_dash_i` pins the word semantics.
* Red shown: appending `base/rexxutil/Macrospace` made the test fail; reverted with sed, file unchanged vs commit.
* The shell grep and the Rust derivation agreed on first run.
* fmt 0, clippy -p rexx-exec --all-targets 0, whole `suite` test binary 52 passed.
* Baseline run: exit 101 (red as expected), 18 groups 16 red, 29 s wall (26.8 s build, 2.05 s test). Records in
  docs/superpowers/records/2026-10-13-phase-9b/task-2-baseline/, verdicts in docs/superpowers/plans/phase-9b-gate.md.
* Observations: the two windows groups stay (NotRun rows, green). ADDRESS, File, SecurityManager and tilde hit
  routines incidentally and are in the file by the derivation rule. SysFileTree has 17 failed (not blocked) tests,
  SysFileXXX 1, SecurityManager 5 failed.

## Fix round 1

* gate.md: false sentence replaced by the 9b criterion (ruling 1), SecurityManager's 5 failures attributed to
  suite-9d-set.txt:200-204, a table of owning task per red group, TEST_FILEEXISTS_NIL_INVALID note.
* Plan: Task 10 Step 2 now states ruling 1; Task 4 carries ruling 2.
* suite-9b-groups.txt header: "byte (LC_ALL=C) order"; the unchecked revision claim dropped.
* run-output.txt re-captured with the harness command: exit status 101, wall 2 s (build cached), appended as its
  last two lines; verdicts/tests/table unchanged by the re-run. gate.md cites the 29 s first run only as such.
* `cargo test -p rexx-exec --test suite gate_groups_9b`: 2 passed.

## Fix round 2

* gate.md owner table: SecurityManager split into 6 tests blocked on SYSDROPFUNCS (Task 7) and
  TEST_CALL_PASS_THREE_ARGS blocked on SysSearchPath (Task 4). Plan Task 4 Step 3 check gains SecurityManager.
* The "29 s first run" figure removed from gate.md.
