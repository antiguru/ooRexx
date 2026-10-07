# Spec review: Phase 6.1

Spec under review: `docs/superpowers/specs/2026-10-07-phase-6-1-loose-ends.md` (uncommitted draft).
Read `scout-common.md` beside this file for the rules (read-only; worktree builds only if you need a run;
oracle command; no subagents). Inputs the spec cites: the three `scout-*-report.md` beside this file,
the queued items, the roadmap row 6.1, the Phase 6 spec `docs/superpowers/specs/2026-09-29-phase-6-concurrency-design.md`.

Your lens is below. Every finding: the spec's sentence or table row, what is wrong or missing, the
evidence (a file:line, a report line, or a run with command and output), and a proposed fix. Classify:
BLOCKER (the plan would build the wrong thing or an exit criterion cannot be checked), IMPORTANT,
MINOR. Separately list OPEN QUESTIONS that only Moritz can answer (a choice of scope or design, not a
fact you can check), each with your recommendation; include any of the spec's O1-O7 you would answer
differently, and say why.

Write `.superpowers/sdd/2026-10-07-phase-6-1/spec-review-<lens>.md`. Return only the path and the
count per class. Prose minimal, no em-dashes.

## Lenses

- `facts`: every factual claim in the spec against its source: the scout reports, the code at
  `be19fd06a`, the oracle source. Sample the scout reports' own claims too: re-run at least five probes
  across the three reports (pick the ones a disposition rests on: b13's oracle SIGSEGV, b5, c1, D1's
  `rnd` and `sl2`, M11). Check that each roadmap 6.1 queued item and every census (b)/(c) site has a home.
- `dst`: the simulation design (spec section 4, D7, O3-O7) as an engineer who has built deterministic
  simulation testing (FoundationDB, TigerBeetle, Antithesis, timely dataflow / Materialize style) would
  review it. Determinism holes (threads, real time, I/O, allocation order, hash order, the timer thread,
  signals, child processes); whether PCT is applied soundly (Burckhardt et al.: priority change points
  and the depth bound; what "PCT over preemption points with FIFO" does and does not guarantee); whether
  the gate judge can fail on a real bug and stay green on a legal schedule; replay across commits;
  shrinking/minimising a failing seed; how a found bug becomes a regression test. Read
  `rust/crates/rexx-exec/src/scheduler.rs`, `clause.rs`, `timer.rs`, `invocation.rs` as needed.
- `plan`: completeness and checkability: does each exit criterion name a command or a run that decides
  it; does the staging order respect dependencies (e.g. T3 b5 before T6 b13; T1 before T5's
  `debug_pause`; T7 before T8); the performance section (are the right programs named, is the budget
  realistic given D1's per-call stores and the `Activation` size assert); anything in the queued items
  or the census the stages miss; risks the spec does not name.
