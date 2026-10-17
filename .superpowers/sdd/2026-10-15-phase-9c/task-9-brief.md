### Task 9: The restore surface

**Files:** `rust/crates/rexx-exec/src/lib.rs` (`execute_on`'s `ProgramSource::new` at `:3712`),
`src/run.rs:3304`
(`Interp::parse_file`, which serves `::REQUIRES` at `install.rs:715`, the external call at
`lib.rs:2365`, `newFile` at `install.rs:1928` and `options.rs:584`), `src/install.rs:986`
(`package_from_source`, which `.Package~new(name, array)` at `src/dispatch/package.rs:1062` and
`~loadPackage(name, array)` at `:1007` reach), `src/dispatch/class_protocol.rs:339`
(`parse_source_lines`),
`rust/crates/rexx-api/src/callbacks.rs:1697` and `:1759` (`LoadPackageFromData`, `NewRoutine`),
`rexx-parse` (`ProgramSource`, for a program that has no source to show),
`docs/superpowers/plans/phase-4-exclusions.txt:5679-5687`. Create
`rust/crates/rexx-exec/tests/compiled.rs` and a fixture directory
`rust/crates/rexx-exec/tests/compiled/` holding an oracle-written `.rxo`.

**Consumes:** `compiled::restore`, `compiled::write_image` from Task 8.

- [ ] **Step 1:** Measure the oracle on its own `.rxo`, from a fresh directory, and save the
      transcripts: `SOURCELINE()` (0), `SOURCELINE(1)` (40.34, "the number of lines in the program
      (0)"), `PARSE SOURCE` (names the `.rxo`), `trace r` and `trace i` output, an error's traceback
      (`Program in "<path>" (no source available).`), `~source` of the restored routine and package,
      and each route below. Measured at plan time: `SOURCELINE()` 0, `PARSE SOURCE` naming the `.rxo`,
      40.34 on `SOURCELINE(1)`, no clause lines under `trace r`.
- [ ] **Step 2:** Every route calls `compiled::restore` where the oracle restores. The oracle's
      calls are `grep -n 'Class::restore(' interpreter/parser/LanguageParser.cpp`, which prints
      `108, 166, 205, 242, 267, 292, 324, 378, 4349, 4394`, and `classes/MethodClass.cpp:430`
      (`MethodClass::restore`, which `:108` and `:166` reach). By enclosing function and route:
      - `:108` `createMethod(name, array, ctx)`: `.Method~new` with an array whose second item is
        `/**/@REXX@` (joined with LF);
      - `:166` `createMethod(name, ctx)`: `.Method~newFile`;
      - `:205` `createRoutine(name, array, ctx)`: `.Routine~new(name, data)` with the same array;
      - `:242` `createRoutine(name, ctx)`: `.Routine~newFile`;
      - `:267` `createRoutine(name, buffer, ctx)`: the native `NewRoutine`
        (`api/ThreadContextStubs.cpp:547`), before its data is split into lines;
      - `:292` `createProgram(name, buffer)`: the native `LoadPackageFromData`
        (`ThreadContextStubs.cpp:344`, `package/PackageManager.cpp:880`), likewise;
      - `:324` `createProgram(name, array, ctx)`, with the same array test, reached through
        `createPackage(name, array, ctx)` (`:398`): `.Package~new(name, array)`
        (`classes/PackageClass.cpp:228`) and `~loadPackage(name, array)`
        (`PackageManager.cpp:854`);
      - `:378` `createProgramFromFile`: `rexx FILE` (`concurrency/RexxStartDispatcher.cpp:132`,
        `:237`), `::REQUIRES` (`createPackage(filename)`, `:431`), and an external `CALL`
        (`execution/RexxActivation.cpp:3138`);
      - `:4349` (`processInstore`) and `:4394` (`restoreFromMacroSpace`) are `RexxStart`'s
        instore and macrospace paths, Phase 10's.

      Measured for this revision on the oracle, from a fresh directory with a `rexxc -e` image of
      a one-routine package: `.Package~new('pa', <the file's lines>)` and
      `.context~package~loadPackage('lp', <the file's lines>)` both restore (one routine,
      `sourceSize` 0, the routine callable); `.Package~new('ps', <the whole file as one string>)`
      does not (no routine, `sourceSize` 1), because a string source is one line and never has the
      second item. A binary image passed as an array of lines is source, and raises 13.1 at the
      `@` (`base/class/Routine.testGroup:238`). `Undecodable` raises 3.905 and `Rejected` raises 3.903, both rc 253, with the oracle's
      text.
      An encoded image whose decoded bytes lack the tag is `Rejected` (3.903). The oracle's `rexx`
      instead runs the buffer it decoded in place, then the leftover base64 text as commands, rc 0
      (Task 8 review, Minor 2). Record it as a numbered Deviation in `phase-4-exclusions.txt`
      (the oracle bug is not copied), and pin the crate's 3.903 in `tests/compiled.rs`. Also pin
      an encoded image truncated inside a line and inside a base64 quartet.
- [ ] **Step 3:** A restored program is parsed from its embedded source and answers the Step 1
      observables as the oracle's compiled program does. The no-source property lives on the
      program's own source object, not in a side map.
- [ ] **Step 4:** `tests/compiled.rs` (criterion 4): it spawns `CARGO_BIN_EXE_rexxc` and the oracle's
      `rexxc` explicitly, one per side, asserts each compile's rc 0 and the 29-byte prefix
      `#!/usr/bin/env rexx\n/**/@REXX`, and runs one program per restore route and per observable on
      each side's own output, comparing stdout, stderr and status. The routes include
      `.Package~new(name, array)` and `~loadPackage(name, array)`, and the string form of
      `.Package~new` as the converse. The committed oracle-written `.rxo`
      answers 3.903, rc 253, here. Add Review Focus 2 and 5.
- [ ] **Step 5:** Run through the harness `API/oo/METHOD`, `base/class/Method`, `base/class/Routine`
      and `utilities/rexxc/rexxc`: `TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA`, both
      `TEST_NEW_ARRAY_FROM_FILE` and both `TEST_NEW_FILE_COMPILED` pass, and the `rexxc` group passes
      whole. Close the `phase-4-exclusions.txt` row with a DELIVERED note citing `tests/compiled.rs`
      and the harness run, and correct `:5391`'s reference to it. Per-task check, plus
      `rexx-api`'s tests. Commit.

