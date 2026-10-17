### Task 5: `rexx` beside `rexx-run`, and the A/B

**Files:**
- Create: `rust/crates/rexx-exec/src/bin/rexx.rs`.
- Create: `rust/crates/rexx-exec/src/launcher.rs` (public, the front end's library half).
- Modify: `rust/crates/rexx-exec/src/invocation.rs` (`Invocation` at `:41`), `src/lib.rs`
  (`run_program` at `:3465`, `execute_on` at `:3703` and its callers at `:3468`, `:3488`, `:3505`,
  `:3668`), `src/require.rs` (made reachable from `launcher.rs`).
- Modify: `rust/crates/rexx-exec/tests/support/group_runner.rs:706-740` (`interpreter_bin`),
  `tests/suite.rs:298-328` (`check_path`) and its test at `:1883`.
- Create: `rust/crates/rexx-exec/tests/rexx_command.rs`.

**Produces:**
- `rexx_exec::run_program_named(path: &[u8], text: Vec<u8>, invocation: Invocation) -> Outcome`,
  the byte-named program entry, with `execute_on` taking `path: &[u8]`. `run_program(path: &str,
  ..)` stays and delegates through `path.as_bytes()`, so its callers
  (`grep -rln 'run_program(' rust/crates/*/src rust/crates/*/tests` lists them) are untouched. A
  name holding `'ff'x` reaches `PARSE SOURCE` and the `Error N running <path>` line byte for byte
  (Review Focus 4). A consumer of the path inside `execute_on` that needs a change outside
  `lib.rs` is reported under Shared paths.
- `launcher::resolve_program(name: &OsStr, cwd: &Path, rexx_path: Option<&OsStr>,
  path: Option<&OsStr>) -> Option<PathBuf>`, the oracle's program search built on
  `require::search_entries`, `candidates` and `first_regular` (`require.rs:30`, `:57`, `:129`).
- `launcher::version_banner() -> Vec<u8>` (the ruling above).
- `Invocation::with_default_options(directive: Vec<u8>, count: i64)`, applied as
  `.Package~defaultOptions('D', ...)` then `('C', count)` (`dispatch/package.rs:374-419`) before the
  program is installed.
- `Invocation::without_syscargs()`, for `-e`.

- [ ] **Step 1:** Measure the oracle first, from a fresh directory, and save the transcripts in the
      records directory: no arguments; `-v` and `-V` before and after other options; `-e` with code,
      with no code, with arguments, with a syntax error, with `exit 7`; `-o` and `-od` with and
      without a leading `:`, as the last word, and with a malformed option; an unknown `-x` and its
      effect on `.SYSCARGS` (the array is indexed from `argc_base - 1`, `rexx.cpp:180-189`); a bare
      `-`; a name found through `.rex`, `REXX_PATH` and `PATH`; not found (`Error 3:` then
      `Error 3.901:`, rc 253, no `running` line, measured at plan time); the return-value table of
      scout C section 1; Review Focus 4's names. Read `utilities/rexx/rexx_command.testGroup` for
      what `TEST_PASSINGARGS5` and `TEST_PASSINGARGS7` assert.
- [ ] **Step 2:** Write `rexx.rs` on `launcher.rs`, following `rexx.cpp:80-242` for option scanning,
      usage text (stderr, rc 255), argument joining, `.SYSCARGS` and the exit status. It keeps
      everything `rexx-run.rs`'s `main` does (`src/bin/rexx-run.rs:18-149`): signal handlers,
      `ProgramInput::Stdin`, the transient flags of the real descriptors, the sinks,
      `REXX_SWITCH_MODE` with its `rexx-sim:` lines, the canonical reported path, and the 8-bit exit
      truncation. `-e` runs its text as `INSTORE` (`PARSE SOURCE` answers `LINUX COMMAND INSTORE`).
      Arguments are taken as bytes (`OsStrExt`), never through a lossy conversion, and the
      program runs through `run_program_named` (`rexx-run.rs:109`'s `to_string_lossy` is not
      carried over).
- [ ] **Step 3:** `tests/rexx_command.rs` spawns `CARGO_BIN_EXE_rexx` and the oracle on each Step 1
      shape and compares stdout, stderr and status. The `-v` banner's `Build date:` line and
      copyright year are compile-time stamps on the oracle side (`rust/CLAUDE.md:36` normalises
      `rexxc`'s banner for the same reason): the test normalises those two on both sides and
      compares the rest byte for byte, so it does not break when `build/bin/rexx` is rebuilt on
      another day than `version.rs:24`'s `30 Jul 2026`. Point `interpreter_bin` and
      `check_path`'s identity at `CARGO_BIN_EXE_rexx`, and update
      `the_crate_side_finds_its_own_rexx_and_no_rexxc` (`suite.rs:1883`, its
      `CARGO_BIN_EXE_rexx-run` reads at `:1902` and `:1915`).
- [ ] **Step 4:** Run `utilities/rexx/rexx_command` through the harness: every test of its expected
      set passes. Per-task check. Commit, and read the sha back.
- [ ] **Step 5:** The A/B (spec section 8, Ruled 1). From that commit,
      `CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 --bin rexx --bin rexx-run`, copy
      both binaries to `/home/moritz/dev/repos/ooRexx-p9-pin/` as `rexx-9c-<sha>` and
      `rexx-run-9c-<sha>`, and from the repo's `rust/` run, with the pin first:

      ```
      bench-programs/callgrind.sh -B 0.5 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9c/ab \
        pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run \
        base=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9c-<sha> \
        head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-9c-<sha>
      ```

      Every delta is against `pin` (`callgrind.sh:5-20`). An `OVER` on `head` is a regression
      against the budget: fix rounds under 6.1's stopping rule (three rounds, then the figures go
      to Moritz). Report `head` minus `base` per program as the front end's share. Copy the table
      into `docs/superpowers/records/2026-10-15-phase-9c/ab/`. Commit the record.

