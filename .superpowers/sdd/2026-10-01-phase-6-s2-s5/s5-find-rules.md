# Rules for S5 read-only finding agents

You are one of several agents gathering evidence in parallel for Phase 6 S5 (plan
`docs/superpowers/plans/2026-10-01-phase-6-s2-s5.md`, Tasks 23-26) of the ooRexx Rust rewrite, repo
`/home/moritz/dev/repos/ooRexx-rust-rewrite`, branch `plan/rust-rewrite`. One implementer later
turns all findings into code and records. You write a findings file only.

- **Read-only on the repository.** Do not edit, create or delete files in the tree, and never run
  git commands that change state (no add, commit, stash, checkout, restore, reset, switch). Your
  only writes: your findings file in the workspace, and your own scratch directory.
- **Scratch:** your scratch directory is `/tmp/claude-1000/p6-s5-<your name>/`. To build a
  tree other than the working tree, `git archive <sha> | tar -x -C <scratch>/tree`, then
  `find <scratch>/tree -exec touch {} +`, and use `CARGO_TARGET_DIR=<scratch>/target`; require a
  `Compiling` line for the crate you measure. For the working tree, also use your own
  `CARGO_TARGET_DIR` under your scratch, never `rust/target`.
- **Load:** a full gate run and other agents share the machine. Prefix cargo test/build with
  `memcap 8G`. Run long commands in the foreground (or wait on a PID); never wait on a
  notification that may not come.
- **Shell:** never wrap tool commands in `bash -c '...'` or `sh -c '...'` (they block on manual
  approval for hours). Write steps to a script in your scratch and run `bash /abs/path/script.sh`.
  rm only literal absolute paths, no globs, no variables. Never `pkill -f` a pattern; signal only
  PIDs you started.
- **Oracle:** the C++ interpreter is run as
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
  from a fresh empty directory (leftover .rex files are found as external routines). Never create
  named semaphores or anything that changes rxapi persistent state on the oracle. A concurrent
  program needs 30 oracle runs before a claim about its output.
- **Read-only trees:** the C++ tree, samples/, build/, ootest/, oodocs/, testbinaries/, api/.
- **Evidence:** every figure and every "all/none/every" claim comes with the exact command that
  produced it and its output (or a log path in your scratch copied into the workspace under
  `.superpowers/sdd/2026-10-01-phase-6-s2-s5/s5-evidence/<your name>/`). Do not claim something is
  unreachable or absent without running it.
- **Findings file:** `.superpowers/sdd/2026-10-01-phase-6-s2-s5/s5-find-<your name>.md`. Write it as
  you go (it must survive if you are stopped). Sections: what was measured (commands), results,
  what the implementer must build or change (concrete: file, function, test name), open questions
  for the lead. Prose minimal, no em-dashes.
- **At the end:** delete your scratch with a literal `rm -rf /tmp/claude-1000/p6-s5-<your name>`
  (after copying evidence), and reply with a three-line summary only.
- **No subagents.**
