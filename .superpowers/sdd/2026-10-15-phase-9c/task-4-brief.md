### Task 4: The block, directive and program sites, the coverage test, and the old API gone

**Files:** `rust/crates/rexx-parse/src/block.rs` (`:356`, `:384`, `:395`, `:442`, `:650-670`),
`src/directive.rs` (helper at `:140`), `src/lib.rs:201` (`ParseError::new(99, 914, ..)`),
`src/token.rs`, `tests/errors.rs`,
`rust/corpus/errors/parse-errors.tsv`, `rust/corpus/suite-9d-set.txt`,
`docs/superpowers/plans/phase-4-exclusions.txt` (the note opened 2026-08-20 at `:5205-5262`).

- [ ] **Step 1:** Convert every remaining site, with the report table. Witnesses: the queued
      `10.2` probe (`end j`, inserts `"J"`, `"I"`, line 1), 36.901 (`call foo bar(`, position 13 on
      line 1), and 47.2 from `2026-10-13-error-position-ampersand.md`. `missing_then_sub` survives
      Task 3 only for `block.rs:660`: convert that 18.x site to `missing_then`, inserting the IF's
      line, and delete `missing_then_sub` (Task 3 finding).
- [ ] **Step 2:** Delete `ParseError::new(code, sub, byte)` and every other path that builds a
      `ParseError` without `Msg<N>`. `PENDING` is empty and is deleted. The two label-colon rows
      (18.x here, 35.1 on the oracle, a licensed Deviation) never match: they leave `PENDING` for
      an exemption that names the Deviation and is checked to still differ (Task 2 finding).
- [ ] **Step 3:** The D6 coverage test in `errors.rs`: scan `rust/crates/rexx-parse/src` (outside
      `#[cfg(test)]` modules and `tests.rs` files) for `E{major}_{sub}` constants passed to the API.
      Each pair needs a `translation` row in the TSV, or a row in
      `const UNREACHED_BY_FILE: &[((u16, u16), Unreached)]`, whose reason is a closed enum and each
      variant is checked:
      - `Unreached::InterpretOnly`: the pair appears in
        `the_interpret_only_errors_match_the_condition_objects_own_code` (`errors.rs:578`);
      - `Unreached::NoProgram { oracle_site, probes }`: the oracle raise site (file:line, checked to
        resolve in the C++ tree) and the probe programs tried, as the `GUARD` rows of
        `rust/corpus/refusal-dispositions.tsv` carry them. The test runs each probe through the
        parser and asserts it does not raise the pair.

      No other variant exists, so no reason is a string nothing re-runs. A pair with neither, where
      a program reaches it, gets a TSV row with `?` fields filled by `derive.sh`. Show the test red
      with one row removed.
- [ ] **Step 4:** Review Focus 1: add rows (`?` fields, filled by `derive.sh`) for an insert holding
      `&1`, a quote, a non-UTF-8 byte and a long symbol, each compared as bytes. Update
      `an_interpret_error_with_a_substitution_renders_the_condition_objects_errortext`
      (`errors.rs:615`) to the oracle's filled message, as a byte literal
      (`b"INTERPRET data must not contain labels; found \"X\"."`).
- [ ] **Step 5:** In `phase-4-exclusions.txt`, delete the sentences of the 2026-08-20 note that R6
      makes false (message text and substitutions deliberately not reproduced). Run
      `base/source.file/incorrectCharacters` through the harness, and remove its line from
      `suite-9d-set.txt` once it passes. Close in the report the queued
      `2026-10-10-translation-error-inserts.md`, the inserts half of
      `2026-10-07-interpret-syntax-traceback-line.md`, and the insert parts of
      `2026-10-13-error-position-ampersand.md` (the `instanceMethod` item there is not 9c's).
- [ ] **Step 6:** Per-task check, plus `cargo test -p rexx-exec --test corpus` gated and the
      INTERPRET witnesses. Commit.

