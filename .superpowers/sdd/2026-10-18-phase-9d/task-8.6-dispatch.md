# Task 8.6 dispatch: Streams, paths, commands

BASE 0fd10a074. Implementer of Phase 9d Task 8.6 (one batch of queued items, area: streams, paths, commands).

Read first, in order:
1. task-8-brief.md (the per-item template) and the header of task-8-batches.md (closing conventions,
   inventory columns, the perf ruling: callgrind.sh on EVERY program, no -p, against BASE; own delta at
   most +0.05% libc subtracted per program; pingguard/pingsem SPREAD flags reported, not failed).
2. task-6-dispatch-rules.md (running, git, report, semantics rules). Its perf line naming five programs
   is superseded by the batches header. Report file: task-8.6-report.md in this directory.
3. global-constraints.md.
4. The section "## 8.6 Streams, paths, commands" of task-8-batches.md: your items. Each queued file is
   under .superpowers/sdd/queued/; read each fully, including any RULED line at its top. The gate line
   phase-9c-gate.md:146 points at 2026-10-13-lossy-path-sites.md: close it with a line in your report
   and a row in t8.6/closed.tsv as the batches header says.

Notes:
* Rexx strings are bytes. Path sites (lossy-path-sites) must pass names as bytes (OsStr/OsString via
  std::os::unix::ffi), never through a lossy UTF-8 conversion; witness with a name containing 'ff'x in
  a fresh directory, and clean it up.
* stdout write errors: witness under a closed stdout (`>&-`) and a closed pipe, in a test harness that
  spawns the binary; never close the test runner's own stdout.
* The background-child item: the oracle returns from the command at once; derive what "the command
  completes" means from rexxref (ADDRESS, commands) before choosing when to stop reading the child's
  output, and keep the existing command witnesses green.
* An item whose answer departs from the oracle in a way not ruled: stop that item and report the
  derived rule and the departure, with a Deviation candidate block. Never write a numbered Deviation.

Records under docs/superpowers/records/2026-10-18-phase-9d/t8.6/. Build target
CARGO_TARGET_DIR=/tmp/claude-1000/t86-target (delete after reporting). For the perf base build
0fd10a074 into /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.6/rexx-base-0fd10a074 from a git archive in
its own fresh target dir (archived trees carry commit mtimes).

Checks before reporting: fmt, clippy (-p of every crate you touch, -D warnings), the test binaries you
touch plus rexx-exec --lib and collect_stress, the corpus gate (REXX_CORPUS_GATE=1, known mismatch:
rexxutil_file_date_time.rex), rexx-exec --test coverage and rexx-parse --test errors if you add corpus
programs, the whole suite table compared with t8.5/r6/suite/ if present else the latest t8.5 suite
table (only your items' rows may change), and callgrind on every program. Run the workspace test once
at the end (`cargo test --release --no-fail-fast --workspace` from rust/, with rust/target/release/rexx
built first) and name every failure; known flakes: datetime_zone (DST), the redirected-target
collect_stress row, rexxutil_process reaped-child under load, SysFileTree TEST_TREE_DIRECTORY.

Never go idle waiting on a background command: run long commands in the foreground (timeout up to
600000) or poll them in a bounded loop; never use pgrep -f or pkill -f. No subagents. Commit before
verifying. Reply with only status, shas, one-line test summary, concerns.
