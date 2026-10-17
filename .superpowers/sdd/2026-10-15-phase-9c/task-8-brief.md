### Task 8: `rexxc` and the `.rxo` writer

**Files:**
- Create: `rust/crates/rexx-exec/src/bin/rexxc.rs`, `rust/crates/rexx-exec/src/compiled.rs`.
- Modify: `rust/crates/rexx-exec/src/launcher.rs`, `src/lib.rs` (the translation entry beside
  `execute_on`).
- Modify: `tests/support/group_runner.rs` (`interpreter_bin` also places `rexxc`), `tests/suite.rs`
  (`check_path` requires `rexxc` inside `bin`, and `the_crate_side_finds_its_own_rexx_and_no_rexxc`
  at `:1883` is turned around: its `resolve("rexxc", ..) == None` at `:1913` and its copied-`rexxc`
  failure at `:1915-1916` become the converse), `tests/suite/fixtures/ChildPath.testGroup:16`.
- Create: `rust/crates/rexx-exec/tests/rexxc.rs`.

**Produces:**
- `compiled::write_image(source: &[u8], encode: bool) -> Vec<u8>` (the header ruling above).
- `compiled::restore(bytes: &[u8]) -> Restore`, with
  `enum Restore { Source, Compiled(Vec<u8>), Rejected }`: `Source` when the bytes after an optional
  `#!` line carry neither tag, `Compiled(source)` for a header this build writes, `Rejected` for any
  other header or a damaged image. Task 9 uses it.
- `launcher::translate(path: &Path, text: Vec<u8>) -> Outcome`: parse, bind `EXTERNAL "LIBRARY"`
  names (spec criterion 4), and report a failure exactly as `rexx` would, without running anything.
  The path reaches messages through `OsStrExt::as_bytes`, as Task 5's entry takes it.

- [ ] **Step 1:** Measure the oracle (`RexxCompiler.cpp:89-200`): no arguments (`SYNTAX:` lines on
      stdout, rc 255); an unknown flag (`The rexxc command parameters are incorrect.` first); output
      equal to input (rc 254, after the banner); not found (banner, then `Error 3:`/`3.901`, rc 253);
      a syntax error with and without `-s` (stderr report, rc `256 - major`); `-s`, `-e`, both, in
      either order; Review Focus 3 and 4. The messages come from the catalogue by message number
      (`Error_REXXC_*`). The banner's build date and copyright year are compared normalised, as
      in Task 5 Step 3.
- [ ] **Step 2:** Write `compiled.rs` (with its own base64), `rexxc.rs` on `launcher::translate`, and
      unit tests: write then restore round-trips both forms; a header with the oracle's magic, a
      wrong version, or a source length past the end is `Rejected`; a source starting with
      `/**/@REXX` text but no header is handled as Step 1 of Task 9 measures.
- [ ] **Step 3:** `tests/rexxc.rs`:
      - every Step 1 shape against the oracle, comparing stdout, stderr and status;
      - the D6 agreement: for every TSV row, `CARGO_BIN_EXE_rexxc FILE -s` from a fresh directory
        gives rc `256 - major` and the recorded secondary line, or rc 0 and empty stderr for `ok`
        rows (rows run in parallel). The rows whose program names `REGISTERED` are exempt per
        spec criterion 4, selected by a predicate that reads the decoded `program` field for the
        bytes `REGISTERED` (today `parse-errors.tsv` lines 543, 689, 920 and 963, by
        `grep -n REGISTERED rust/corpus/errors/parse-errors.tsv`), never by a list of rows;
      - the exemption's witness: line 543's program (`::routine r external "REGISTERED x y"`)
        through `CARGO_BIN_EXE_rexxc FILE -s` fails, and its stderr carries the refusal text that
        `Owner::Phase10(Deferral::Rxapi)` renders (`(Phase 10 Rxapi)`, the gap at
        `rexx-exec/src/directives.rs:79-82`). When Phase 10 implements `REGISTERED`, this test and
        the predicate go red together;
      - the crate's `.rxo`, binary and encoded, run by the oracle's `rexx` answers 3.903, rc 253
        (D5: the header fails the oracle's `validate`, never crashes it).
- [ ] **Step 4:** The harness's `bin` holds `rexxc`, and `check_path` asserts it resolves there.
      Run `utilities/rexxc/rexxc` through the harness: every test of its expected set passes,
      which needs Task 9 for `TEST_REXXC_TWO_ARGS` and `TEST_REXXC_ENCODING`. Until then, record
      which tests pass. Per-task check. Commit.

