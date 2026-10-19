# Rules for every Task 6 reviewer

## Inputs
Your dispatch names the brief, the report, the review package and the range. Also read
`global-constraints.md`, the end of `progress.md` for the controller's rulings on this task, and
`task-6.7-review.md` for the review format. All are in this directory.

## Hunt by running, not by reading
* Build HEAD in `rust/` with `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=<your dir under the scratchpad>
  memcap 8G cargo build --release -j 4 --bin rexx` (add `--bin rexxc` when translation errors are
  in scope). Delete the target dir when done.
* Every crate run is under `memcap 2G timeout 20`. Never run a crate binary uncapped.
* Oracle: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.
* Probes go in a fresh `mktemp -d` under the scratchpad
  `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/`,
  never as `.rex` files in its root.
* Compare stdout, stderr and rc separately. Hash order is unspecified: compare unordered results as
  multisets.
* To compare against the parent commit, use `git worktree add` under the scratchpad with its own
  target dir, and remove both afterwards.
* Probe the edges the change could reach, beyond the cluster's own tests: neighbouring
  instructions and methods, INTERPRET, TRACE, conditions, subclasses, and the hot path's perf
  where the diff touches one. A callgrind spot check is
  `rust/bench-programs/callgrind.sh -B 0.5 -p <program>` against
  `/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run`.
* Run the touched test binaries and `cargo test -p rexx-exec --lib`, bounded to 10 minutes each.

## Rules
* Do not edit tracked files and do not commit. Write only your review file.
* An oracle bug is never something to copy. A divergence that predates the task and lies outside
  its clusters is a queue candidate, not a blocker.
* No subagents.

## Output
Write the review file your dispatch names, in this directory: spec compliance, strengths, issues
by severity (Critical, Important, Minor) with the deciding probe lines quoted (the scratchpad is
not durable), and a verdict of APPROVED or CHANGES REQUESTED. Return only the verdict and a
one-line count of findings.
* A new macro that is not small and mechanical is a finding (rust/CLAUDE.md, Moritz 2026-10-19).
