# Scout A: where the ooTest suite stands

Read `rules.md` in this directory first. Scratch directory: `/tmp/claude-1000/p9scope/a/`.
Report: `.superpowers/sdd/2026-10-10-phase-9-scoping/scout-a-report.md`.

Phase 9's gate is "the full suite green against existing baselines, excluding enumerated groups"
(L3-core on the host). Nobody has run the whole suite against this crate. Measure where it stands.

1. **Population.** Enumerate every `*.testGroup` under `ootest/` (and how `testOORexx.rex` itself
   discovers groups). Find what "existing baselines" means in this tree: how the oracle's expected
   results or known failures are recorded, if at all. If there is no baseline file, say so and use
   the oracle's own run as the baseline.
2. **Phase 10's share.** Classify each group as Phase 10's or not, by the roadmap's rules: extension
   groups (`json`, `yaml`, `rxregexp` and the other `extensions/`), groups or tests reaching RXAPI
   (macrospace, external queues, `rxsubcom` registration, `RXFUNCADD`/`RXFUNCQUERY`; see
   `reaching_rxapi` in `rust/crates/rexx-exec/tests/support/group_runner.rs`), and native-API groups
   (derived by `rust/crates/rexx-exec/tests/api_group_partition.rs`). Record which rule put each
   group there. Groups or tests that are Phase 10's only partly: say which tests.
3. **Run.** Build `rexx-run` at HEAD in release. Find how `whole_groups` and `group_runner.rs` drive a
   group through `testOORexx.rex` on this crate and on the oracle, including how `printSummary`'s
   `RXFUNCQUERY` is avoided; reuse that mechanism rather than inventing one. Run every
   non-Phase-10 group once on this crate and once on the oracle, each under memcap/ulimit and a
   timeout (oracle slowest group is ~94 s; give the crate 4x, at least 60 s). At most 8 runs at once.
   Groups the oracle itself crashes on or loops in: record and skip.
4. **Classify.** Per group: tests run, passed, failed, errored on each side; and for every group that
   is not green here, the first cause, binned as one of: loud refusal (quote its text and the phase
   it names), wrong answer, crash/panic, timeout/hang, out of memory, framework could not load the
   group. Give the bin counts and the per-group table (as a TSV file beside the report, plus a
   summary in the report). Group the refusal texts: which refusals block the most tests.
5. Record total wall time, and the commands, so the run can be repeated as a gate.
