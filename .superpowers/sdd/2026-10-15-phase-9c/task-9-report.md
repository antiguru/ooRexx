# Task 9 report: the restore surface

Status: DONE_WITH_CONCERNS.

## Commits

* `230b59655` every restore route restores a compiled image: `compiled::{program_source, lines_source}`, `ProgramSource::compiled` and `is_traceable`, the routes, the observables, `Surface::compile_buffer`.
* `ec20c18b3` `tests/compiled.rs`, `tests/compiled/cases/restore`, `tests/compiled/programs/`, `tests/compiled/oracle.rxo`.
* `caa9de0f7` Deviation 41, the `TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA` row DELIVERED and its reference corrected, `api_group_tests.rs`'s `RECORDED` entry removed, suite records.
* `06c5cc082` unit tests for the restore helpers, `corpus/refusal-sites.tsv` re-derived (line numbers only: `diff <(git show HEAD:rust/corpus/refusal-sites.tsv | sed -E 's/\.rs:[0-9]+/.rs/' | sort) <(sed -E 's/\.rs:[0-9]+/.rs/' corpus/refusal-sites.tsv | sort)` is empty).
* The commit carrying this report and `docs/superpowers/records/2026-10-15-phase-9c/task-9-step1/`.

## Step 1: oracle measurements

Every run from a fresh directory under the scratchpad, the oracle through `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )` (and `bin/rexxc`), stdout, stderr and status read separately, 1 run a side. The transcripts are the recorded blocks of `rust/crates/rexx-exec/tests/compiled/cases/restore` (the oracle's output, re-run on every test run) and `docs/superpowers/records/2026-10-15-phase-9c/task-9-step1/`.

* `SOURCELINE()` 0. `SOURCELINE(1)` 40.34 naming 0 lines. `PARSE SOURCE` names the `.rxo`. `.context~package~source` and a routine's `~source` are empty, `~sourceSize` 0, `~sourceLine(1)` the null string (`observe.rxo`, `levels.rxo`).
* `TRACE R` and `TRACE I` print nothing for the program's own clauses. An `INTERPRET` string is traced with its own text, a call from it back into the program is not, and `SOURCELINE()` inside it is still 0 (`levels.rxo`, `interpret.rxo`).
* `TRACE ?R` pauses only inside an `INTERPRET` string, and the `+++ "LINUX COMMAND ..."` banner prints at the first traced clause there (`debug.rxo` with `typed.txt` on stdin).
* Traceback lines: `Program in "<path>" (no source available).` for the program and an internal call, `Routine NAME in package "<pkg>" ...` for a `::ROUTINE`, an external call (the name as called: `fail.rxo`, `FAIL.RXO`), a `::REQUIRES` prologue (named for the package path) and a `.Package~new` prologue (named for the package name), `Method NAME with scope "C" in package ...` for a method. The same lines appear in `condition('o')~traceback` and in `StackFrame~traceLine`, and an install-time `::CLASS` failure blames `Program in` (`levels.rxo`, `observe.rxo`, `external.rex`, `requires.rex`, `prologue.rex`, `frames.rxo`, `install.rxo`).
* Each route restores: `rexx FILE`, external `CALL`, `::REQUIRES`, `.Routine~newFile`, `.Method~newFile`, `.Routine~new(name, lines)`, `.Method~new(name, lines)`, `.Package~new(name, lines)` and `~loadPackage(name, lines)` of an `-e` image (`routes.rex`). The string form of `.Package~new` is source (no routine, `sourceSize` 1). A binary image as an array of lines is source, 13.1 at the `@` (`binary.rex`).
* A restored executable inherits no context: a restored routine or method calling a routine its caller defines raises 43.1 (`context.rex`, `method.rex`).
* 3.903 and 3.905, rc 253, on every route, naming the resolved path for `rexx`, `CALL` and `::REQUIRES` and the name as given for `newFile` and the array forms (`task-9-step1/refusals/`).
* `rexxc` given a `.rxo`, binary or encoded, translates it as source: 13.1 at the `@`, rc 243, no output written. The crate already did this, so `rexxc` is unchanged, and `execute_as` keeps `Entry::Translate` off the restore.
* Review Focus 5: `sub/p.rxo` run from its parent directory, and called from there, finds `::REQUIRES 'helper.rex'` and the bare `call ext` in `sub/`.
* Review Focus 2's source case: a file whose first line is `/**/@REXX say 1` is 13.1 at the `@`, rc 243.
* Deviation 41's image: the oracle answers 13.1 on `'00'X`, rc 243 (`task-9-step1/untagged/`).

## Result per route

All in `tests/compiled.rs`, each side's `rexxc` compiling and each side's `rexx` running its own images, three descriptors equal:

| route | how |
|---|---|
| `rexx FILE` | `lib.rs` `execute_as` via `compiled::program_source`, positionless 3.903/3.905 |
| `::REQUIRES`, external `CALL`, `newFile` (both), `::OPTIONS` string | `Interp::parse_file` |
| `.Package~new(name, array)`, `~loadPackage(name, array)` | `install.rs` `package_from_source` via `compiled::lines_source` |
| `.Method~new`, `.Routine~new` and every `newMethodObject` caller with an array | `class_protocol.rs` `parse_source_lines` |
| `NewRoutine`, `LoadPackageFromData` | `Surface::compile_buffer`, restoring the whole buffer; harness only |
| `NewMethod` | never restores (`LanguageParser::createMethod(name, buffer)`): lines `.Method~new` would restore are parsed as source |

The no-source property is `ProgramSource::is_traceable`, on the program's own source object. Readers: `SOURCELINE`, `Package~source*`, executable `~source`, `Interp::untraceable_code` (gates `route_trace_line_as`, the debug banner and `debug_pause_now`), `record_failure_at`'s sourceless site, `StackFrame~traceLine`, and directive blame. The plain-source path pays `compiled::restore`'s tag check. The trace gate is read only where a trace line is routed, the debug banner prints or a pause is due.

## Tests

* `cargo fmt --all --check`: 0.
* `cargo clippy -j 4 -p rexx-exec -p rexx-api -p rexx-parse --all-targets -- -D warnings`: 0.
* `cargo test -j 4 -p rexx-exec --lib`: 0, 1150 passed before the two new unit tests; `--lib compiled` after them: 15 passed.
* `cargo test -j 4 -p rexx-api`: 0, all suites ok. `cargo test -j 4 -p rexx-parse`: 0.
* `--release --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --test compiled --test rexxc --test rexx_command`: 0 after the re-derivation (`refusal_sites` was red on line numbers before it). `-p rexx-core --test unsafe_sites`: 0.
* `REXX_CORPUS_GATE=1 --release --test corpus --test sim_processes --test program_end --test trace_oracle`: 0 (29 passed 1 ignored, 4, 8, 41).
* `REXX_CORPUS_GATE=1 --release --test api_group_tests`: 0, 24 passed, after removing the delivered test from `RECORDED`.
* `tests/compiled.rs` is live: with `route_trace_line_as`'s gate disabled, `observe.rxo`, `observee.rxo`, `levels.rxo`, `debug.rxo` and `interpret.rxo` diverge; reversed by an exact edit, `git diff --stat` empty.

## Harness

`REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=API/oo/METHOD,base/class/Method,base/class/Routine,utilities/rexxc/rexxc REXX_SUITE_TABLE=docs/superpowers/records/2026-10-15-phase-9c/task-9-suite/table.tsv CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite`, exit 0. Every group green on both sides: `API/oo/METHOD` 320 of 320, `base/class/Method` 56 of 56, `base/class/Routine` 60 of 60, `utilities/rexxc/rexxc` 11 of 11. `TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA`, both `TEST_NEW_ARRAY_FROM_FILE`, both `TEST_NEW_FILE_COMPILED`, `TEST_REXXC_ENCODING` and `TEST_REXXC_TWO_ARGS` pass. No failing test.

## Rulings

Taken by me, for the controller to confirm:

1. A restored executable takes no parent package on any route, the context argument included, as measured (`context.rex`, `method.rex`) and as `createRoutine(name, context)` and `createMethod(name, context)` read.
2. `NewMethod` never restores, as the oracle's `createMethod(name, buffer)`. It reaches `Surface::compile_buffer`, which parses lines `.Method~new` would restore as source and builds the method directly.
3. Sourceless traceback lines follow `formatSourcelessTraceLine`: `isRoutine` is this crate's `ROUTINE` frame kind (`Entry::Routine`, or `TopLevel` with `SUBROUTINE`, `FUNCTION` or `REQUIRES`), a prologue is named for its package as its stack frame is.

## Concerns

* A restored routine called through `~call` (`.Routine~newFile('fail.rxo')~call`) shows `Routine CALL in package ...` here where the oracle shows `Routine <path of the source rexxc translated> ...` (`task-9-step1/routine-object/`). Two causes. This crate's routine-object activation is named for the message (`CALL`), a pre-existing difference visible for source routines too in `StackFrame~name` (oracle `ROUTINE sfn.rex` then `METHOD CALL`, this crate `ROUTINE CALL`). And the image does not carry the translated program's name, so a faithful fix needs the format to hold it. Not in `tests/compiled.rs`.
* Files outside the brief's list: `rexx-api/src/callbacks.rs` and `callbacks/fake.rs` (the `compile_buffer` trait member), `tests/api_group_tests.rs` (`RECORDED` loses the delivered test, without which its gate goes red), `environment/route.rs`, `trace.rs`, `run/interpret.rs`, `dispatch/{context,executable,package,library/surface}.rs`, `builtin/state.rs`, `error.rs`, `launcher.rs` (`positionless` made `pub(crate)`).
* Two of my builds ran without `CARGO_TARGET_DIR` and wrote `rust/target/` (debug check, then `release/rexx` and `release/rexxc` at 19:27 from the uncommitted tree). Anything reading `rust/target/release` should rebuild.
* `.superpowers/sdd/2026-10-15-phase-9c/progress.md` is modified in the working tree by someone else and untouched by these commits.

## Fix round 1

From `task-9-review.md`, with the three rulings confirmed.

1. `surface.rs`: `held`'s doc is back on `fn held`, and `compile_buffer_source` has its own.
2. `surface.rs` `compile_buffer`'s `Method` arm tests the second line directly through the new `compiled::encoded_lines`, which `lines_source` uses too. It no longer decodes an image to throw it away. One narrow difference from before: lines whose second line is the tag but whose joined text is source to `restore` (a first line that is neither `#!` nor a tag) now parse as source directly instead of going through the `.Method~new` send. Both parse them as source, and the direct path takes no context, as the oracle's `createMethod(name, buffer)` does.
3. `error.rs`: `sourceless_routine_line`'s doc uses a colon in place of ` -- `.
4. `tests/compiled.rs`: `boundary.rxo`, the encoded image cut 8 digits into its first line, answers 3.903. Mutation: in `compiled::decode`, `if digit == 0 && !text.is_empty() && text.last() != Some(&b'\n') && index == text.len() { return None; }` before the tail check. `a_foreign_or_damaged_image_is_refused` went red on `boundary.rxo` (3.905 for 3.903), and `line.rxo` and `quartet.rxo` stayed green. Reversed by an exact edit, and `git diff --stat crates/rexx-exec/src/compiled.rs` showed only the `encoded_lines` change.
5. The `CALL` naming, ruled (c): `.superpowers/sdd/queued/2026-10-17-routine-call-frame-name.md` (the source-routine witness, the missing `METHOD CALL` frame, the restored-routine traceback, `RoutineClass.cpp:209`, `:148`, `ProgramMetaData.cpp:190-191`, and that the image must then carry the routine's name), and a KNOWN GAP row with `OWNER: none.` after Deviation 41 in `phase-4-exclusions.txt`.

Checks, all with `CARGO_TARGET_DIR=.../scratchpad/target-impl-9`: `cargo fmt --all --check` 0; `cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` 0; `--release --test compiled --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --test licensed_divergences --test api_group_tests` all ok; `-p rexx-exec --lib compiled` 15 passed.
