# Final whole-branch review: Phase 6 S2-S5

Range `c812f0cd0..d1abee6ea` on branch `plan/rust-rewrite` in
`/home/moritz/dev/repos/ooRexx-rust-rewrite`. Code under `rust/`; the rest is records.

- Spec (binding): `docs/superpowers/specs/2026-09-29-phase-6-concurrency-design.md`.
- Plan: `docs/superpowers/plans/2026-10-01-phase-6-s2-s5.md` (Global Constraints, Review Focus, and
  the exit criteria; read task text only where you need it).
- Gate record: `docs/superpowers/plans/phase-6-gate.md` sections `## S2` onward; `phase-6-pinning.md`,
  `phase-6-perf.md` `## S2-S5 gate`.
- Ledger of rulings: `.superpowers/sdd/2026-10-01-phase-6-s2-s5/progress.md` (lines `Ruling P..`).
  A ruled departure is not a finding unless the ruling is wrong on the spec.
- Final gates: all green at `42b29493c` (`bg/42b29493c/status.txt` in the same dir). Do not re-run them.

## Scope: a mutation hunt, not a read

Every task was reviewed alone. You look for what no single task review could see:
1. Defects at task seams: state that one stage adds and a later stage's path does not carry. The
   latest instance: a `REPLY` continuation was built without the elapsed clock and `RANDOM` seed
   (P89, `ir/drive.rs` `split_level`). Hunt this class: every field of `Activity` /
   activation state, and every place a new activity or continuation is built or a baton passes.
   Name each field and whether each builder copies, resets or shares it, against the oracle.
2. Invariants the spec states (single owner, pinning, the baton, timer-thread slice, signal
   delivery, GUARD WHEN wakeups, semaphores) that only a test suite's absence keeps unbroken:
   mutate the code and see whether anything goes red.
3. `unsafe` outside `ffi.rs`, `load.rs`, `bytes.rs`, `frame.rs`, `island.rs`, `signal.rs`;
   mutable globals beyond the registry, the timer thread and the P70 test statics; new
   dependencies beyond loom and libc; `Op::Generic`; `size_of::<Op>() != 16`.
4. Records: a sentence in the gate/pinning/perf records the evidence it cites does not support.
   Sample, do not read all.

A finding needs a run: a probe against the oracle, or a mutation that stays green. Claims that a
path "cannot be reached" need the line that tries.

## Rules

- Read-only on this checkout: no edits, no git state changes. Build copies via
  `git worktree add /tmp/claude-1000/p6-final/wt d1abee6ea` (remove it with `git worktree remove`
  at the end) or `git archive`; scratch and `CARGO_TARGET_DIR` under `/tmp/claude-1000/p6-final/`
  only; delete it at the end. Builds under `memcap 8G`, `-j 8` at most.
- Mutation runs: `cargo test --release -p rexx-exec --no-fail-fast`; restore every mutant.
- Oracle: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
  from a fresh empty dir. No named semaphores, nothing touching rxapi persistent state. Concurrent
  programs: run each at least 5 times per engine and state the counts.
- No `bash -c` / `sh -c` wrappers. `rm` only with literal absolute paths, no globs. Never pkill -f.
  Foreground commands with bounded waits; never end a turn waiting for a notification.
- No subagents.

## Report

Write `.superpowers/sdd/2026-10-01-phase-6-s2-s5/final-review.md`: Strengths (short), Issues as
Critical / Important / Minor each with file:line, the failing run (command, expected, got), and fix;
the seam table from item 1; a "Declined to judge" list; verdict (ready / with fixes / no). Prose
minimal, no em-dashes. Return only verdict, counts per severity, and the report path.
