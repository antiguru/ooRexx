# SDD ledger -- plan: docs/superpowers/plans/2026-08-13-compound-name-resolution.md

## Task 1 -- the split, computed once

Landed. Report: `task-1-report.md`.

* `Plan::compounds`, a `Box<[Option<CompoundName>]>` indexed by `SymbolId::index`, filled by the pass that already split every compound name.
* `Interp::tail_key` and the new `Code::stem_name` read it; a body with no plan (an `INTERPRET` fragment) splits its own spelling through the same `CompoundName::split`.
* Six call sites stopped calling `compound_parts` for the stem: two in `eval.rs` (the brief's), three in `run.rs`, one in `parse_template.rs`.
* Gates green; **1476 -> 1477** passed, 0 failed, 4 ignored.
* `perf stat -e instructions:u`, one run per arm at one fixed binary path: `rexxcps` **-7.19%** (2.057 billion), `compound` **-17.70%** (6.577 billion), four untouched axes +0.04% to +0.60%.
* Ten oracle-measured stanzas added as `rust/crates/rexx-exec/tests/ir_dual_cases/compound-names`.

Written into Task 2's own text: `Plan::bind` also fills the table and assigns no slots to the pieces, so an entry does not imply its pieces have slots.

## Task 2 -- the slot, resolved once

Not started.

## Task 1: the split, computed once -- COMMITTED b3e8cf4d2
1477 passed, 0 failed. Review: spec PASS, quality PASS WITH CHANGES (three comment claims, no code change required).
Measurement reproduced independently: rexxcps -7.12% instructions (report said -7.19%; its base figure was its own high outlier), compound -17.69%. Both arms staged at one fixed binary path.

Behaviour preservation was established the strongest way available: the reviewer put `assert_eq!(entry, CompoundName::split(symbols.name(id)))` INSIDE `Code::compound`, so every cache access checked itself against a fresh split -- 757 tests, zero firings across corpus and population sweeps -- then inverted it to assert_ne! to prove the probe was live. That technique belongs in any future caching change.

Fix round QUEUED behind Task 2 (same files):
1. The new unit test's doc says a missing OR MISADDRESSED entry is not a wrong answer. Measured false: keying each entry under (id.index()+1) % len reddens 13 tests, twelve at output level. The fallback argument covers a MISSING entry only -- a misaddressed one is a different failure mode and the two were conflated.
2. "the three syntactic positions a compound-shaped symbol reaches the pass through" -- names a set size, and the size is wrong: PARSE VAR a.i is a fourth, verified against the oracle.
3. The case file header claims unique coverage for rows where it was not measured; only the fragment row was.
4. **A measured regression to record, not to fix**: INTERPRET-saturated loops are +1.0% (against +0.45% drift on the same loop without INTERPRET), because fragment_plan now builds and discards a compound table per execution. Worth it against -7.12% on rexxcps, but it goes in the record entry.
5. Minor: `bind` records splits for stem-shaped names nothing reads.

Also from the review, for the future: a mutation that corrupts a stem name can send a corpus program non-terminating, so mutation sweeps want a per-test timeout. One run was killed at ~25 minutes for exactly this.
