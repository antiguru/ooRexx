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
