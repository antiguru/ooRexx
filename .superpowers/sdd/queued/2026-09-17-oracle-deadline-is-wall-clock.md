**CLOSED 2026-10-30 by Phase 9d Task 8.8c (949a7806b).**

**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** no probe; gate rows time out under load and none of the three directions is chosen, which the Phase 9 close gates meet.

# Queued: a gate verdict that depends on machine load

Found 2026-09-17 while closing fix round 1 of the file-split task. Pre-existing,
unrelated to that task, and not in any path either of its commits touched.

## The observable

`no_row_started_diverging_or_stopped_answering` (`rexx-exec/tests/method_bodies.rs`,
panicking at `tests/gate_tables/mod.rs:169`) failed twice at `f9ffe9a8b` under
`cargo test -j 4 --release -p rexx-exec --no-fail-fast`, exit 101 both times, and
passes in isolation.

It reported **different rows each time**:

* run 1: `DateTime weeksInYear (instance)`: the oracle did not finish: TimedOut
* run 2: `Package findPublicRoutine (instance)` and `Properties items (instance)`,
  same classification
* run 3, the binary alone, framed by a load measurement either side:
  `StackFrame context (instance)`, same classification

**Four distinct rows across three runs, never once repeating.** The same block took
121.15s, then 195.35s, then 146.90s. Load average was 77.51 on a 32-core machine
when first checked, with the one-minute figure above the five- and fifteen-minute
ones; run 3 ran at 36.99 rising to 41.34 and still failed, which is consistent,
since 37 on 32 cores is still oversubscribed and is not a test of the hypothesis.

Non-repeating rows, a 61% swing in block duration, and sustained oversubscription
are three independent observables and they agree: this is not a property of any
row.

## The mechanism, named

`crates/rexx-exec/tests/support/oracle.rs:49`:

    pub const ORACLE_DEADLINE: Duration = Duration::from_secs(10);

A **wall-clock** deadline on the oracle subprocess. A probe that ordinarily
finishes in milliseconds can exceed ten seconds of wall clock when the machine is
oversubscribed, and the harness then classifies the row as a structural failure,
which `gate_tables/mod.rs` makes red in every mode and which `REXX_CORPUS_GATE`
cannot relax. `oracle.rs:361` already carries a comment about avoiding "a flaky
`TimedOut`", so the failure mode was known and the mitigation does not cover this
path.

## Why it matters more than one flaky test

The verdict is only trustworthy in one direction, and nothing in the harness says
so:

* **Green under load is strong evidence.** The probes finished despite contention.
* **Red under load is no evidence at all.** It is indistinguishable from a real
  divergence, and it is reported in the language of one: "the oracle did not
  finish ... not an answer to compare".

Any gate reading taken on a busy machine therefore has an asymmetric meaning that
the reading itself does not carry. Several readings in this project's history were
taken while other agents were building, and none records the load it ran under.

## What the work is

Open. Three directions, none decided:

* Deadline on **CPU time** rather than wall clock, so contention does not count
  against the probe.
* Keep the wall-clock deadline but **record the load beside every verdict**, so a
  red row carries the context that decides whether it means anything.
* **Retry once and report the retry**, never silently.

**"Retry until it passes" is the wrong fix**, for the same reason "make the tests
skip" was wrong on the worktree-precondition item: it converts a real hang into a
slow pass and removes the only signal that would have shown it. Whatever is built,
a probe that genuinely never finishes must still end the run red.

The generalisation worth keeping either way: **frame a failing run with a
measurement of its environment, before and after.** A transient shortage read as a
machine property retires a measurement permanently, and the reverse error accepts a
real failure as noise.

## Confirmed 2026-09-18

The same suite at the same commit on the same machine, framed by load: **exit 0 at
load 4.68, with zero `TimedOut` anywhere in the log**, against four distinct rows
timing out across three runs at load 37 to 77. The hypothesis is no longer an
inference from non-repeating rows; it has a negative control.

This also fixes the asymmetry stated above in place: green at low load is the
reading that means something, and it is now on record for `f9ffe9a8b`.

## Task 8.8c evidence

The controller selected a hard wall deadline with load recorded at timeout. Child-wait and output-drain diagnostics name their probe, configured budget, elapsed duration and load snapshot, while the original single-invocation controls still pass. Pure formatting controls cover available and unavailable load data, and the external retained-log assertion confirms the actual path and phase associations. No retry or load-based pass policy was added, and this closure does not claim a before/after environment history.

Evidence: `.superpowers/sdd/2026-10-18-phase-9d/task-8.8c-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/timing/`.
