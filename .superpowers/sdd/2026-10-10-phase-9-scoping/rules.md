# Phase 9 scoping survey: rules for every scout

You are a read-only scout. The controller will write the Phase 9 spec from your report. Phase 9 is
roadmap row 9 of `docs/superpowers/plans/2026-07-27-rust-rewrite.md` (read row 9, and the paragraph
"Why Phase 9's gate is L3-*core*, not L3"). Read `rust/CLAUDE.md` first: its oracle, probe and gate
rules bind you.

* **Read-only tree.** Do not edit, stage or commit anything in the repository. The only files you
  write are your report and scratch files under your own scratch directory, named in your brief.
* **Worktree:** `/home/moritz/dev/repos/ooRexx-rust-rewrite`, branch `plan/rust-rewrite`, HEAD
  `a98238be5`. Check `git rev-parse --short HEAD` first and stop if it differs.
* **Memory caps.** Every process that runs this crate's interpreter or its tests goes under
  `memcap` (`memcap 2G` for a single interpreter run, `memcap 8G` for a test binary or cargo build),
  scripts and loops included. Every oracle run is wrapped as
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.
  An uncapped run has killed this machine's sessions before.
* **Timeouts.** Put `timeout` on every interpreter run, either side.
* **Probes** run from a fresh empty directory you `mkdir` yourself under your scratch directory.
  `ootest/testOORexx.rex` writes fixtures into its working directory, so run it from a scratch copy.
* **Builds.** Use `CARGO_INCREMENTAL=0`. Build only at HEAD, only `--release` unless your brief says
  otherwise, into the existing `rust/target`. Other scouts share it, and cargo's lock serialises you.
* **No `bash -c` / `sh -c` wrappers, no `rm` with a glob, no `git checkout`/`stash`/`reset`.**
  `rm` only with literal absolute paths.
* **Evidence.** Every count or claim in your report carries the command that produced it, run and
  read, unpiped exit status. Do not write a result before the run that produces it. Name sets; when
  you give a count, give the command beside it. Mark anything you inferred rather than ran as
  "inferred".
* **No subagents.** Do the work yourself.
* **Report:** write the full report to the path in your brief. Return only: status, the report path,
  and up to five lines of the findings that most change Phase 9's scope.
