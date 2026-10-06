# Task 24 implementer brief (condensed from the S5 refusals and verify findings)

Requirements: `task-24-brief.md` (the plan's task text) plus what follows. The plan's Global
Constraints section (`docs/superpowers/plans/2026-10-01-phase-6-s2-s5.md`, read only that section)
binds every step. Spec: `docs/superpowers/specs/2026-09-29-phase-6-concurrency-design.md` section 9
criterion 8.

Findings to build from: `s5-find-refusals.md` (evidence `s5-evidence/refusals/`) and, for u5,
`s5-find-verify.md`. Rulings P75-P79 in `progress.md` (lines starting "Ruling P7").

## Step 1, criterion 8

1. Both enumerations (findings section 1 and 2) re-run at your final head and committed under
   `docs/superpowers/records/2026-10-01-phase-6-s2-s5/` with their commands; each shown empty
   (the method probe: no oracle Message/EventSemaphore/MutexSemaphore method answers the generic
   not-implemented refusal).
2. **P78.** Reword `run.rs:2375` and `handle.rs:50` (re-find them; line numbers move) so no comment
   names Phase 6 as an owner.
3. **Divergence rows** (findings section 3): add each missing row with owner none and its reason,
   using the row text the findings give, plus **P75** (u5, from `s5-find-verify.md`) and **P77**
   (the unsatisfiable wait; program end P39/P40; scheduling licences P38/P41; Error 11 depth on a
   pool thread, spec :551-553). Not P53's leak.
4. **P76.** Each new row gets a witness test where an existing harness reaches the case (signals
   tests, scheduler tests, group runner, corpus); show each witness red once by a mutation or by
   reverting the behaviour it pins (report only), run with `--no-fail-fast`. A row with no witness
   says so and why.
5. Findings section 3's edits to existing rows (`:4077` GUARDED is real; `:4739` "or Phase 6").

## Step 2

6. **P79.** Fix `closed_phases`' word tokenizer so an identifier containing `CLOSED` (e.g.
   `CLOSED_PHASES`) is not the resolution word; show the false pass first (the findings' rename
   experiment), then give the Alarm/Ticker row (`:5667`) its real resolution (DELIVERED note: Alarm
   and Ticker gate table C rows agree at HEAD; cite).
7. `closed_phases` gains Phase 6; the negative control (`closed_phases.rs` ~:335) counts
   `"Phase 10"` alone. Move the Phase 6 corpus witnesses from `phase-8.txt` to `phase-6.txt` (P17;
   findings section 4 lists the range and the hard-coded phase-file lists and `gate_tables/mod.rs`
   `CLOSED_PHASES` that need `6`).

## Gate record

Add a criterion 8 subsection to `## S5` in `docs/superpowers/plans/phase-6-gate.md` (Task 23 created
the section), quoting both enumerations' commands and empty outputs and listing the divergence rows
by exclusions-file line.
## Checks (P51)

`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
    `cargo test --workspace --release`, `cargo test -p rexx-core --doc`. No full gates.

## Rules

- unsafe only in ffi.rs, load.rs, bytes.rs, frame.rs, island.rs, signal.rs. No new deps beyond loom
  and libc (rustix features OK). No mutable globals beyond the registry, the timer thread and P70's
  test statics. No Op::Generic; Op::Clause stays discriminant 0; `size_of::<Op>() == 16`.
- Read-only: the C++ tree, samples/, build/, ootest/, oodocs/, testbinaries/, api/.
- Oracle: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
  from a fresh empty dir; no named semaphores; 30 runs before a claim about a concurrent program.
- Git: commit with `git commit -F <file>`, message ending with the two lines
  `Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>` and
  `Claude-Session: https://claude.ai/code/session_0157HT3JqnRRbgiMDb84iGSD`. Add files by path; never
  `add -A`, amend, force, reset, checkout, restore, stash. No push.
- Shell: never `bash -c`/`sh -c` wrappers in tool commands (they block on approval); script files
  run as `bash /abs/path.sh`. rm only literal absolute paths, no globs. Never `pkill -f`. Long runs
  in the foreground or wait on a PID; never wait for a notification. Scratch and target dirs under
  `/tmp/claude-1000/p6-t24/`; `memcap 8G` on cargo test; delete scratch at the end.
- Prose: minimal, no em-dashes, no set sizes or counts in prose unless quoted from a cited log
  beside its command, nothing forward-looking.
- No subagents.

## Report

Write `task-24-report.md` (same dir) as you go. Return: status (DONE / DONE_WITH_CONCERNS / BLOCKED /
NEEDS_CONTEXT), commits, one-line test summary, concerns.
