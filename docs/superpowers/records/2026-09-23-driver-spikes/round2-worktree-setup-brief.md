# Worktree setup, round 2 (binds both round-2 tasks)

1. `git rev-parse HEAD`. The worktree tool cuts from the master lineage, not
   from `plan/rust-rewrite`. You are pre-authorised to run
   `git switch -c <your-branch> b5dd351d6` in your worktree when HEAD is not
   `b5dd351d6` and `git status --short` shows nothing but `.claude/`. Verify
   HEAD afterwards. Anything else unexpected: stop and report.
2. Tests need untracked fixtures that the worktree lacks. From the worktree
   root, symlink them from the main checkout (read-only use only):
   `ln -s /home/moritz/dev/repos/ooRexx-rust-rewrite/build build`, and the same
   for `ootest`, `oodocs`, `testbinaries`. Some tests look for the release
   binary under `rust/target/release`: point your `CARGO_TARGET_DIR` at a
   directory in your scratch and symlink `rust/target` to it. Never commit
   the symlinks; remove them before your final `git status` check.
3. Common rules, the measurement contract and the report format are in
   `../spikes/common.md` beside this directory, with these differences:
   base is `b5dd351d6` (its `rust/` tree is byte-identical to `0ba0f3876`,
   whose base `.text` hash was
   `748b2f0679dcddb9cf65d2e30d5c50a18e4f90095ecb634f8d018c4bfcce4c32`; confirm
   yours matches), and your scratch directory is
   `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round2/<your-branch-basename>/`.
4. Round 1's reports are in `docs/superpowers/records/2026-09-23-driver-spikes/`
   on your base. Read the one your brief names.
