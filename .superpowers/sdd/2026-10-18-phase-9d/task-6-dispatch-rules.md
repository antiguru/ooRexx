# Rules for every Task 6 instance implementer

## Context
* Phase 9d closes the remaining ooTest suite failures of the Rust rewrite of ooRexx
  (`/home/moritz/dev/repos/ooRexx-rust-rewrite`, branch `plan/rust-rewrite`, crate in `rust/`).
* Read `global-constraints.md` in this directory and your clusters' entries in
  `docs/superpowers/records/2026-10-18-phase-9d/clusters.md` (tests, groups, likely files).
* Copy exact commands and conventions from `task-6.7-report.md` in this directory: the suite
  harness (`REXX_SUITE_GATE=1 ... --test suite l3_core_suite`; exit 101 is expected while the
  phase is open), the corpus gate, `derive.sh` for `parse-errors.tsv` rows, the perf check
  (`rust/bench-programs/callgrind.sh -B 0.5 -p "startup interpret parse churn sayloop"` against
  the pin `/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run`, own delta at most +0.05% per program),
  and records under `docs/superpowers/records/2026-10-18-phase-9d/t6.N/`.
* Compare your whole-suite table with the previous instance's recorded table. Only your clusters'
  rows may change. Known flake: SysFileTree `TEST_TREE_DIRECTORY`.

## Oracle bugs and Deviations
* Never copy an oracle bug. Never write a new numbered Deviation: stop that item and report it.
  The controller takes it to Moritz.
* Licence rows under an already-ruled Deviation go in `rust/corpus/suite-licences.txt` only when
  your brief or the controller says so.
* Hash order is unspecified. Never match the oracle's bucket order; compare unordered results as
  multisets.

## Running
* Oracle: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.
* Crate runs always under `memcap 2G timeout 20`. Never run a crate binary uncapped.
* Builds: `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR` as your dispatch names, under `memcap 8G`
  (`memcap 16G` for `cargo test --no-run`), `-j 4`. Delete the target dir after reporting.
* Probes in a fresh `mktemp -d` under the scratchpad
  `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/`.
  Never leave `.rex` files in the scratchpad root: it is on the oracle's search path.
* Per-task checks are bounded to 10 minutes each. No full gates.

## Git and code
* Never `git add -A`, never `git reset --hard`, never `git checkout -- <file>`. `rm` only with
  literal paths. No `bash -c`. No `pkill -f`. Commit before verifying. No unsafe.
* Comments: no em-dashes. A doc comment states the contract. Match the surrounding code.
* Prose in records: no set sizes, no history.
* No subagents.

## Report
Write `task-6.N-report.md` in this directory: what changed per cluster, witnesses, commands run
with results, suite delta, perf, concerns. Commit it with `git add -f`. Return only status (DONE,
DONE_WITH_CONCERNS, BLOCKED or NEEDS_CONTEXT), shas, a one-line test summary and concerns.
