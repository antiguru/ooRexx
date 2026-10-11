# Task 4 review: D3 reasons and the relabels

BASE 052550e85, HEAD 598e9f22c. Reviewer: opus, read-only. No suite was rerun and no test was run;
every check below is a read or a script over the diff or the tree, named where it was used.

## Spec verdict: ✅

D3's shape holds. `Deferral` is the closed set in `rexx-core` (`owner.rs`). `util`, `deferred`/
`deferred_send`, `Loud::internal_routine`, `Loud::redirection` and the `gap` closure take an `Owner`.
`EXCLUDED` gains a reason column, checked against the enum (`phase_ten_reasons.rs`
`every_excluded_builtin_names_a_reason_the_enum_knows`). The `.STDQUE` owed entry is built from
`Owner::Phase10(Deferral::Rxapi)`. The structural test exists and covers the scope D3 gives it.
`owned_message` itself still takes `Option<&'static str>` and gets the reason as a rendered label.
D3's wording ("the reason is a parameter of each constructor shape ... `owned_message`") is satisfied
by the label, so this is not a spec miss. Its consequence is Important 1 below.

Classification matches the brief and the controller's rulings. Macrospace `util` rows, queue
`deferred` rows, `REGISTERED`, `.STDQUE`, the redirect sites and the `EXCLUDED` rows are `Rxapi`.
Non-macrospace `util`, `named_semaphore`, `package_option_write`, `handle_set` and the classic site
are Phase 9. R4's `REFUSING_MEMBERS` and the two `ffi.rs` texts are `Embedding`.
`phase-4-exclusions.txt:5516` reads `OWNER: Phase 10 Rxapi.`

## Quality verdict: Needs fixes

## Findings

### Critical

None.

### Important

1. **`Owner::phase()` is a typed route to a bare `"Phase 10"` that the structural test cannot see.
   The module doc that says otherwise is false.**
   `rust/crates/rexx-core/src/owner.rs:64-70`: `pub const fn phase()` returns `"Phase 10"` for every
   `Phase10(_)`. `owned_message` (`rexx-exec/src/lib.rs:956`) and the owed-entry chain
   (`environment_symbol` `:590`, `environment_entry` `:601`, `unreadable_collection` `:646`,
   `hash.rs:1629` `Owed(Option<&'static str>)`, `environment.rs:260`) still accept any
   `&'static str`. `no_production_literal_names_phase_10` reads literals only, and `rexx-core` is not
   scanned.
   Failure scenario: a Phase 9 task writes `owned_message(what, Some(owner.phase()))`. Nothing
   discourages it, since `phase()` is the method `tests/internal_routines.rs:113` and
   `dispatch/native/tests.rs` already use. The refusal prints `(Phase 10)` with no reason and
   `phase_ten_reasons` stays green. `refusal_carries` catches it only if a gated group reaches that
   site and a row exists for it.
   `rust/crates/rexx-exec/tests/phase_ten_reasons.rs:15-16` says the label "is the only text that
   names Phase 10". `Owner::phase` and `Owner::named`'s `"Phase 10"` arm are two more.
   Fix: make `owned_message` take `Option<Owner>`. This is cheap because the only phase literal left
   in rexx-exec's production sources is `"Phase 9"` at `lib.rs:615` (`native_method`), and
   `instruction_owner`/`expr_owner` answer only `None`. I checked with
   `grep -rn '"Phase \|"4[abc]"' rexx-exec/src rexx-api/src`, test files excluded, and by reading the
   `Some(` arms of `lib.rs:968-1120`. A smaller alternative is to make `phase()` test-only. Then
   delete the false sentence.

### Minor

2. **The R4 relabel has no witness.** I checked `rexx-api/tests/layout.rs:748-760`:
   `a_populated_table_refuses_exactly_the_members_it_names` compares member names only.
   `grep -rn 'Embedding' rexx-exec rexx-api` finds no test. If `REFUSING_MEMBERS`' owners or either
   `ffi.rs:4196`/`:4240` owner went back to `Owner::Phase9`, every test named in the report would
   stay green. One assertion would close it: `refusal_owner("RexxInstanceInterface.Halt") ==
   Some("Phase 10 Embedding")`.
3. **Nothing checks blanking on the real sources.**
   `the_scan_reads_the_test_modules_it_leaves_out` checks that `lib.rs` is *kept* as a file. If
   `without_test_modules` blanked a production file's body, the file would still be "kept" and the
   claim would pass over empty text. The fixture test checks blanking on synthetic input only. Also
   `phase_ten_reasons.rs:50-53`: when no closing line matches, `close` runs to EOF and the rest of the
   file is blanked silently. rustfmt plus `cargo fmt --check` makes this unlikely. A cheap control:
   assert that `production_sources("rexx-exec")`' `lib.rs` text still yields the `"Phase 9"` literal
   at `native_method`.
4. **A REHOME row for a Phase 10 constructor cannot be written.** `refusal_dispositions.rs:61`
   `OPEN_PHASES` admits `"Phase 10"`. `:678` then requires `Owner::Phase(row.phase)` to equal the
   constructor's owner, which is now `"Phase 10 Rxapi"` (`owner_label`), so no row passes both checks.
   No REHOME row exists today (`grep -n REHOME corpus/refusal-dispositions.tsv`, header lines only).
   The failure appears the first time one is needed.
5. `phase_ten_reasons.rs` `a_phase_10_label_carries_the_variant_name` repeats `rexx-core`'s
   `owner.rs` unit test `a_phase_10_label_is_the_phase_then_the_reason`.

## Focus items, as checked

* **Focus 1, the structural test.** It can fail. The report's mutant (`named_semaphore` back to
  `Some("Phase 10")`) is the right shape, and `names()` matches `"Phase 10"` as a whole word inside any
  literal, including longer texts. The scan covers `rexx-exec/src` and `rexx-api/src`
  (`SCANNED`). The exclusion is no wider than D3 allows:
  * `tests.rs` files are excluded by name. Every `mod tests;` in either crate sits under
    `#[cfg(test)]` (19 directly above, 3 with an `#[expect]` between: `sim.rs`, `time_support.rs`,
    `scheduler.rs`; checked with `grep -B2`).
  * `#[cfg(test)] mod NAME;` files and directories are excluded.
  * Inline `#[cfg(test)] mod` bodies are blanked.

  A `#[cfg(test)] fn` stays in scope (fixture). The scanner move is unchanged: a Python comparison of
  `source_files`, `shown`, `literals`, `starts_word` and `names` between BASE's `closed_phases.rs` and
  `tests/support/literals.rs` found each function identical apart from `pub`. Gaps: Important 1 and
  Minor 3.
* **Focus 2, classification.** I checked the report's oracle citations in the oracle checkout:
  * `StreamClasses.orx:175-177` calls `!handle_set` for `HANDLE:`.
  * `StreamNative.cpp:2576-2582` `handle_set` calls `setHandle(fh)`.
  * `LibraryPackage.cpp:279-281` wraps `ROUTINE_CLASSIC_STYLE` in a `RegisteredRoutine`.
  * `Interpreter.cpp:638-649` `getCurrentQueue` sends `GET` to `.STDQUE`.
  * `StreamClasses.orx:445-459`: `init` for `SESSION` and `get` touch no daemon.

  None fits a D3 reason, so both are Phase 9. The controller has ruled on the classic site.
* **Focus 3, `refusal-sites.tsv`.** The file carries no refusal text: the one `Phase` in it is prose
  in a record column (row 215, "Phase 7 ledger"). So the tsv refresh cannot witness Review Focus 5 in
  either direction. It is a site/line table only. A script over `git diff -U0` paired the 34 removed
  rows with the 34 added rows. Every difference was in column 4, and the path before the last `:` was
  the same, so only line numbers moved. The report's claim holds. Focus 5 is carried instead by the
  expected-text edits in the unit and integration tests, which I read in the diff:
  * `dispatch/tests.rs`, `eval/tests.rs`, `run/tests/directives.rs`, `scheduler/tests/native.rs`,
    `spike.rs`, `suite.rs` `MACRO_REFUSAL`, and the four `phase-9-exclusions.txt` refusal rows: each
    changes only the parenthesised label.
  * The `ffi.rs` texts keep their shape through `layout::refusal`
    (`"{what} is not implemented ({label})"`).
  * `refusal_text` is unchanged in format.
* **Focus 4, the reason vocabulary.** `suite_rows` re-exports `rexx_core::Deferral`, and
  `Owner::parse` goes through `Deferral::named`, so there is one vocabulary. `refusal_carries`
  requires `"(" + label + ")"`, so the closing parenthesis rejects `Rxapi2`. The new FAILING fixtures
  cover a bare `(Phase 10)`, the wrong reason and a near-miss.
* **Focus 5, edits outside the brief.** Each one is a required consequence:
  * `bif-exempt.txt`: the attribution is read from the refusal text.
  * `bif_assertions` `PHASES`: the same vocabulary.
  * `coverage.rs` and `owners.rs`: `EXCLUDED`'s new column.
  * `refusal_dispositions.rs`: `Owner` forms, and `COMPUTED_OWNERS` now names `excluded_owner`.

  The repointed `closed_phases.rs` controls still witness what they claim:
  * `the_scan_finds_an_open_phase_it_is_not_asked_about` now uses the same literal walk over all of
    `rexx-exec`, test modules included, instead of the old line grep. That is closer to the scan it
    controls.
  * The `rexx-api` control now proves that `ffi.rs` is read, because a literal opening
    `RexxInstanceInterface.AttachThread` is found. Detecting a phase inside a longer text stays
    covered by the fixture in the same test, which expects lines 4, 5 and 7.
* **Focus 6, prose.** No em-dash in added lines (`grep '—'` over the added side of the diff). No set
  size in new comments. The `EXCLUDED` doc drops its old counts. One false sentence:
  `phase_ten_reasons.rs:15-16` (Important 1).
