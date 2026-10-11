# Task 4 re-review (fix round 1, 598e9f22c..77c6d6569)

Read-only; reads and one grep. No suite run. The compile and test status of the fix is the implementer's claim.

## Per item
1. Important 1 (bare "Phase 10" route): ADDRESSED.
   * `owner.rs`: `Owner::phase()` deleted; `label()` is the only accessor.
   * `lib.rs:953` `owned_message(name, Option<Owner>)`; `environment_symbol`, `environment_entry`, `unreadable_collection`, `instruction_owner`, `expr_owner`, `excluded_owner` all `Option<Owner>`; `hash.rs` `Owed(Option<Owner>)`; `environment.rs` `owed` array and `owed_entry_owner`/`unbuilt_collection_owner` likewise.
   * All remaining `owned_message(` callers pass `None`, `Some(Owner::...)` or an `Owner` value (sim.rs x4 and lib.rs:866 pass `None`).
   * A production path now cannot build "(Phase 10)" bare: the only Phase 10 label comes from `Owner::label()`, which always appends the reason, and `Owner::Phase10` requires a `Deferral`. `grep '"Phase 10"' rust/crates/*/src` finds only `owner.rs` (parse arm, unit test), `native/tests.rs:150,160` (test module vocabulary) and `rexx-inventory` (the EXCLUDED rows, which carry the reason column).
   * False sentence at `phase_ten_reasons.rs:15-16` replaced by a true one (no mention of the label as the only naming text).
2. `suite-9d-set.txt` rename: ADDRESSED. Diff is a 100% similarity rename; plan Task 3 files line and `phase-9a-gate.md:175` updated. One grep (`grep -rIn phase-9d-set . --exclude-dir=target --exclude-dir=.git`, minus `.superpowers/`) returned nothing.
3. `phase-4-exclusions.txt:5284` reads `OWNER: Phase 10 Embedding`: ADDRESSED.
4. Minors:
   * R4 Embedding pinned: ADDRESSED. `rexx-api/tests/layout.rs` `every_refusing_member_is_owed_by_phase_10_for_embedding` compares `refusal_owner` (now returns `Option<Owner>`) with `Owner::Phase10(Deferral::Embedding)` for every row plus `RexxInstanceInterface.Halt`. It covers the REFUSING_MEMBERS owners. The two `ffi.rs` owner texts are not covered by it (they went through `layout::refusal`, unchanged); that part of review Minor 2 stays open, Minor.
   * REHOME match on phase name with or without reason: ADDRESSED (`refusal_dispositions.rs` `names_it`: equal or `starts_with("{phase} ")`); fixture rows `rxapi REHOME Phase 10` (accepted) and `hostemu REHOME Phase 9` (rejected) exercise both directions.
   * Duplicate test dropped: ADDRESSED (`a_phase_10_label_carries_the_variant_name` removed; the owner.rs unit test now compares against the literal `"Phase 10 {name}"`, which is the stronger form).
   * Blanking checked on production text: ADDRESSED (`the_scan_reads_the_test_modules_it_leaves_out` now finds the literal "a package settings write" in the kept `lib.rs`; the unclosed-module case now asserts instead of blanking to EOF). That literal exists at `lib.rs` `unreadable_collection`'s sibling, so the control is live.

## New defects
None Critical or Important.
* Minor: `native/tests.rs:157-160` and `tests/internal_routines.rs:113` replace `phase()` with an inline `match owner { Phase9 => "Phase 9", Phase10(_) => "Phase 10" }`. Test-side and correct, but a copied pattern; harmless.
* Minor: `refusal_owner` now returns `Owner`, so the `ffi.rs` text keeps its shape only through `refusal(what, owner)`; unchanged, no issue found.
