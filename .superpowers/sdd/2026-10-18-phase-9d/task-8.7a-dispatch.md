# Task 8.7a dispatch: Crash, builtins, collections

BASE 81f4bf250. Implementer of Phase 9d Task 8.7a (one batch of queued items, area: a TRACE rooting crash, builtins, collections).

Read first, in order:
1. task-8-brief.md (the per-item template) and the header of task-8-batches.md (closing conventions,
   inventory columns, the perf ruling: callgrind.sh on EVERY program, no -p, against BASE; own delta at
   most +0.05% libc subtracted per program; pingguard/pingsem SPREAD flags reported, not failed).
2. task-6-dispatch-rules.md (running, git, report, semantics rules). Its perf line naming five programs
   is superseded by the batches header. Report file: task-8.7a-report.md in this directory.
3. global-constraints.md.
4. The section "### 8.7a Crash, builtins, collections" (under "## 8.7") of task-8-batches.md: your items. Each queued file is
   under .superpowers/sdd/queued/; read each fully, including any RULED line at its top. The gate line
   phase-9a-gate.md:480 is closed with a line in your report and a row in t8.7a/closed.tsv as the
   batches header says.

Notes:
* The first item is an rc 101 panic (an unrooted traced value); fix it first and witness it under the
  collect_stress harness (rust/crates/rexx-exec/tests/collect_stress.rs).
* 2026-10-19-sortwith-on-foreign-class.md's shape segfaults the oracle: add it to
  rust/corpus/oracle-crashes.txt in that file's format and never run it on the oracle again.
* An item the triage or a Task 6 cluster already fixed: re-run its probe; if it agrees with the oracle,
  close it `closed:probe agrees` with the probe path.
* An item whose answer departs from the oracle in a way not ruled: stop that item and report the derived
  rule and the departure, with a Deviation candidate block. Never write a numbered Deviation.

Records under docs/superpowers/records/2026-10-18-phase-9d/t8.7a/. Build target
CARGO_TARGET_DIR=/tmp/claude-1000/t87a-target (delete after reporting). For the perf base build
81f4bf250 into /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7a/rexx-base-81f4bf250 from a git archive in
its own fresh target dir (archived trees carry commit mtimes).

Checks before reporting: fmt, clippy (-p of every crate you touch, -D warnings), the test binaries you
touch plus rexx-exec --lib and collect_stress, the corpus gate (REXX_CORPUS_GATE=1, known mismatch:
rexxutil_file_date_time.rex), rexx-exec --test coverage and rexx-parse --test errors if you add corpus
programs, the whole suite table compared with t8.6/r1/suite/ (only your items' rows may change), and callgrind on every program. Run the workspace test once
at the end (`cargo test --release --no-fail-fast --workspace` from rust/, with rust/target/release/rexx
built first) and name every failure; known flakes: datetime_zone (DST), the redirected-target
collect_stress row, rexxutil_process reaped-child under load, SysFileTree TEST_TREE_DIRECTORY.

Never go idle waiting on a background command: run long commands in the foreground (timeout up to
600000) or poll them in a bounded loop; never use pgrep -f or pkill -f. No subagents. Commit before
verifying. Reply with only status, shas, one-line test summary, concerns.
