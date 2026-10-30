**CLOSED 2026-10-30 by Phase 9d Task 8.8e1 (1d8b03220).**

# Probe-text literals name gate_table_c.rs for code now in tests/table_c/

File-split Task 10 (2026-09-25) moved gate table C's probe derivations from
`rust/crates/rexx-exec/tests/gate_table_c.rs` to `tests/table_c/probes.rs`
and `checks.rs`. String literals that say "the derivation in gate_table_c.rs"
(e.g. `tests/table_c/checks.rs:386`), and the committed probes under
`rust/corpus/gate-tables/{classes,hierarchy,methods}/` derived from them,
still name the old file; they read true only as the test target's name.
Re-word the literal and re-derive the probes in one commit. Also:
`corpus/gate-tables/README.md:95` names `tests/table_c/probes.rs` without the
`crates/rexx-exec/` prefix line 28 of the same file uses.


## Task 8.8e1 evidence

Regenerated probe comments through the existing Table C derivation, verified the path-only diff, and passed the existing Table C checks. See `.superpowers/sdd/2026-10-18-phase-9d/task-8.8e1-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/records/e1/`.
