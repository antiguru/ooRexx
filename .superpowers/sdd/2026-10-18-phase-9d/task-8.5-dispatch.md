# Task 8.5 dispatch: Variables

BASE 6e0bf5e5b. Implementer of Phase 9d Task 8.5 (one batch of queued items, area: variables).

Read first, in order:
1. task-8-brief.md (the per-item template) and the header of task-8-batches.md (closing conventions,
   inventory columns, the perf ruling: callgrind.sh on EVERY program, no -p, against BASE; own delta at
   most +0.05% libc subtracted per program; pingguard/pingsem SPREAD flags reported, not failed).
2. task-6-dispatch-rules.md (running, git, report, semantics rules). Its perf line naming five programs
   is superseded by the batches header. Report file: task-8.5-report.md in this directory.
3. global-constraints.md.
4. The section "## 8.5 Variables" of task-8-batches.md: your items. Each queued file is under
   .superpowers/sdd/queued/; read each fully, including any RULED line at its top.

Extra item: m1 of .superpowers/sdd/queued/2026-10-15-9b1-task-8b-review-minors.md
(`.context~variables` and a `USE ARG >z` alias). Moritz ruled it ("List Z., Deviation", recorded in
2026-10-19-variable-reference-deviation-candidates.md): the crate lists an alias under the name it is
reachable by. Witness it; do not close the minors file (its other entries are owned elsewhere); add a
status line under its first lines naming m1's witness and sha.

Deviations Moritz already ruled (see 2026-10-19-variable-reference-deviation-candidates.md): list
order source-order as an oracle bug (one Deviation covering DROP, EXPOSE, PROCEDURE EXPOSE order and
the PROCEDURE EXPOSE handler split), the alias listed as Z., and a stem-identity sentence for
Deviation 4. For each: witness it (red where the crate currently follows the oracle; if the crate
already answers the rule, a test pinning the rule's answer), and write a Deviation candidate block in
your report (title, rule with rexxref citation, oracle bug cause with C++ file:line where it is one,
measurement with probe paths, witness names). The controller writes the numbered text into
docs/superpowers/plans/phase-4-exclusions.txt; you never edit that file.

An item whose answer departs from the oracle in a way not ruled: stop that item, report the derived
rule and the departure. Never write a numbered Deviation.

Records under docs/superpowers/records/2026-10-18-phase-9d/t8.5/. Build target
CARGO_TARGET_DIR=/tmp/claude-1000/t85-target (delete after reporting). For the perf base build
6e0bf5e5b into /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.5/rexx-base-6e0bf5e5b from a git archive in its own
target dir (archived trees carry commit mtimes; use a fresh target dir per archived commit).

Checks before reporting: fmt, clippy (-p of every crate you touch, -D warnings), the test binaries you
touch plus rexx-exec --lib and collect_stress, the corpus gate (REXX_CORPUS_GATE=1, known mismatch:
rexxutil_file_date_time.rex), the whole suite table compared with t8.4/r1/suite/ (only your items' rows
may change), and callgrind on every program. Also run rexx-exec --test coverage and rexx-parse --test
errors if you add corpus programs or parse paths.

Never go idle waiting on a background command: run long commands in the foreground (timeout up to
600000) or poll them. No subagents. Commit before verifying. Reply with only status, shas, one-line
test summary, concerns.
