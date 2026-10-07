# Phase 6.1 scouts: common rules

Repository: `/home/moritz/dev/repos/ooRexx-rust-rewrite`, branch `plan/rust-rewrite`, HEAD `be19fd06a`.
Code under `rust/`. Roadmap row 6.1: `docs/superpowers/plans/2026-07-27-rust-rewrite.md:658`.
Queued items: `.superpowers/sdd/queued/<name>.md`. Loud census: `.superpowers/sdd/2026-10-01-phase-6-s2-s5/loud-census.md`.

- Read-only on this checkout: no edits, no git state changes. Build from a copy:
  `git worktree add /tmp/claude-1000/p61/<you>/wt be19fd06a`, `CARGO_TARGET_DIR=/tmp/claude-1000/p61/<you>/target`.
  Builds under `memcap 8G`, `-j 4`. At the end: `git worktree remove --force /tmp/claude-1000/p61/<you>/wt`
  and `rm -rf` your target dir with a literal absolute path.
- Our binary: `target/release/rexx` (check `cargo build --release -p rexx-cli` or the workspace's binary crate name).
  Our switch modes: `REXX_SWITCH_MODE=every`.
- Oracle: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
  from a fresh empty dir (never the scratchpad that holds other .rex files). No named semaphores, nothing touching
  rxapi persistent state. Concurrent programs: at least 5 runs per engine, state counts.
  C++ source (read-only): `/home/moritz/dev/repos/ooRexx/` (interpreter under `interpreter/`).
- Read-only also: samples/, build/, ootest/, oodocs/, testbinaries/, api/ and the C++ tree.
- No `bash -c` / `sh -c` wrappers. `rm` only with literal absolute paths, no globs. Never pkill -f.
  Bounded foreground waits. No subagents.
- A claim that something "cannot be reached" needs the run that tries. Quote the command beside every figure.
- Prose minimal, no em-dashes. Name sets, do not count them in prose (tables may count).
- Return only: report path and a 3-line summary.
