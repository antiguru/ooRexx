# Task 8 report: `rexxc` and the `.rxo` writer

Status: DONE_WITH_CONCERNS.

## Commits

* `9622c9e89` rexxc and the .rxo writer: `src/compiled.rs`, `src/bin/rexxc.rs`, `launcher::{load, translate, parse_compile, same_file_message, output_unwritable}`, `lib.rs`'s `translate_program` on `Entry::Translate`, the early return in `install.rs`, `rexx.rs` moved onto `launcher::load`, `builtin/convert.rs`'s `base64_digit` made `pub(crate)`.
* `7de9e8cc2` `tests/rexxc.rs` and `tests/rexxc/cases`, `group_runner.rs`'s `interpreter_bin` places `rexxc`, `suite.rs`'s `check_path` and its two tests turned around, `ChildPath.testGroup:16`.
* `483679f2c` `corpus/refusal-sites.tsv` re-derived (`REXX_REFUSAL_SITES_REFRESH=1 ... --test refusal_sites`): `lib.rs` line numbers only. It should have been in `9622c9e89` and was caught by the per-task check.

## Step 1: oracle measurements

Every run from a fresh directory under the scratchpad (`t8/probe1`, `probe2`, `probe3`), through
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexxc ARGS )`,
stdout, stderr and status read separately. Every shape below is now a case in `tests/rexxc/cases`, recorded from the oracle by `REWRITE=1` and compared against the crate.

* No arguments: `SYNTAX: rexxc inputfile [outputfile] [-s] [-e]` and `Without outputfile rexxc only performs a syntax check.` on stdout, rc 255.
* Unknown flag (`ok.rex -x`, `ok.rex o.rxo -x`, `ok.rex -`, a third non-flag word): `The rexxc command parameters are incorrect.` first, then the two lines, stdout, rc 255, no banner.
* The first word is always the input: `rexxc -x` prints the banner, then 3.901 for `"-x"`, rc 253.
* Output equal to input: banner (unless `-s`), then `Output file name must be different from input file name.` on stdout, rc 254. Compared as bytes (`strcmp`).
* Not found: banner, then `Error 3:  Failure during initialization.` / `Error 3.901: ... Program "nope" was not found.` on stderr, rc 253.
* Syntax error: clause, `Error 35 running <abs path> line 1:`, `Error 35.1: ...` on stderr, rc 221 (`256 - major`), with and without `-s`. No output file is created.
* `-s`, `-e`, both, either order, and by second byte only (`-sx -Exyz`): all rc 0. `-e` without an output is a plain check.
* Review Focus 3: an output in a missing directory and a read-only output both answer `Error 3.902:  Error writing output file "<name as given>".`, rc 253, after a successful translation. The read-only file stays 0 bytes. A syntax error wins over an unwritable output, and 3.901 wins over both.
* Review Focus 4: `a b`, `a b.rex`, `c\xffd`, `c\xffd.rex` found with and without `.rex`, outputs `o p.rxo` and `q\xffr.rxo` written. Messages carry the bytes as given (`"z\xFFz"`, `"n\xFF/x.rxo"`).
* Translation binds libraries and resolves nothing else: `::routine r external "LIBRARY x"` is 98.903 rc 158 (line 2 after a `say`), `::method m external "LIBRARY REXX nosuchentry"` is 90.998 rc 166, while `::class a subclass zzz` and `::requires 'nosuch.rex'` are rc 0.
* `::routine r external "REGISTERED x y"`: oracle 90.999 rc 166. Crate: `rexx-exec: ::ROUTINE EXTERNAL naming REGISTERED is not implemented (Phase 10 Rxapi)`, rc 120.
* The oracle image layout: `#!/usr/bin/env rexx\n`, then 48 bytes (`/**/@REXX` NUL-padded to 16, magic 11111, version 43, word size 64, big-endian 0, level 60600, reserved, pad, image size), then the image. `-e` writes `/**/@REXX@\n` and 72-column base64 with a final line end.
* Restore probes for Task 9, with oracle `rexx`: `/**/@REXX say 1` and `#!/x\n/**/@REXX say 1` are source (13.1 on the `@`, rc 243). `/**/@REXX@\nsay 1\n`, `#!..\n/**/@REXX@\n!!!!\n` and an oracle `-e` image with one base64 digit changed to `!` are **3.905** (`Encoded compiled program "<abs path>" cannot be decoded.`), rc 253, not 3.903.

## What landed

* `compiled::write_image(source, encode)`: the oracle's `#!` line and header layout with magic `0x7273` and format version 1, the source length in `imageSize`, then the source. `-e` is the encoded tag and the base64 of header and source in 72-column lines. The encoder is `builtin/convert.rs`'s `encode_base64_bytes`, already in the crate.
* `compiled::restore(bytes) -> Restore { Source, Compiled(Vec<u8>), Rejected, Undecodable }`, following `processRestoreData`: the binary tag is compared with its NUL as `strcmp` does, the encoded tag is decoded by the rules of `StringUtil::decodeBase64` (line ends only between quartets, `=` only in the last two digits, only line ends after it). `Rejected` is any header other than this build's or a source length not equal to what follows it. `Undecodable` is refused base64.
* `launcher::translate(path, text) -> Outcome`: `translate_program` runs `execute_as` with `Entry::Translate`, which bootstraps, loads the program and runs `install_directives` with `translate_only` set. `install_directives` returns after `override_package_settings`, the end of the translation walk. Witnessed: with the return disabled, `cls.rex -s` answers 98.909 and `req.rex -s` 43.901, and `the_command_line_answers_as_the_oracles_does` goes red. The run path pays one `bool` read per `install_directives` call.
* `rexxc`: `RexxCompiler.cpp`'s scan, banner unless `-s`, the same-name check, `launcher::load`, `translate`, then the image. A failed write removes the partial file and answers 3.902.

## Tests

* `cargo test -p rexx-exec --lib` (debug, `memcap 8G`): 1150 passed, 0 failed, including the `compiled::tests` and `launcher::tests` added.
* `--release --test rexxc`: 4 passed.
  * `the_command_line_answers_as_the_oracles_does`: every case in `tests/rexxc/cases` agrees on all three descriptors and on the first 31 bytes of each output.
  * `rexxc_agrees_with_every_recorded_parse_error_row`: every TSV row agrees except the exempt and the licensed ones (below), rows run in parallel, about 1 s.
  * `registered_is_refused_with_the_rxapi_reason`: the line-551 program fails and its stderr carries `(Phase 10 Rxapi)`.
  * `the_oracle_refuses_this_crates_images_as_another_versions`: the oracle's `rexx` answers 3.903, rc 253, on both the binary and the encoded image.
* Harness: `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=utilities/rexxc/rexxc,utilities/rexx/rexx_command REXX_SUITE_TABLE=docs/superpowers/records/2026-10-15-phase-9c/task-8-suite/table.tsv ... --test suite l3_core_suite`, exit 101. `rexx_command` is green. `utilities/rexxc/rexxc`: 9 of 11 pass, and `TEST_REXXC_ENCODING` and `TEST_REXXC_TWO_ARGS` fail, which await Task 9 (the brief names them). The oracle passes 11. Table and verdicts in `docs/superpowers/records/2026-10-15-phase-9c/task-8-suite/`.
* `--release --test suite -- rexxc the_crate_side_bin_holds`: 3 passed (`the_crate_side_finds_its_own_rexx_and_rexxc`, `a_command_the_crate_side_runs_finds_its_rexx_and_rexxc`, the class-files test).
* `--release --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test rexx_command`: all pass after the re-derivation.
* `REXX_CORPUS_GATE=1 --release --test corpus --test sim_processes`: 29 passed 1 ignored, and 4 passed.
* `cargo fmt --all --check`: 0. `cargo clippy -p rexx-exec --all-targets -- -D warnings`: 0.

## TSV agreement

The exempt predicate matches `REGISTERED` **case-insensitively** in the decoded program. The brief's byte match misses row 959, `::routine r external "registered x"` (install, 90.999), which this crate refuses with the same `Rxapi` reason. The spec's wording is "whose program names `REGISTERED`", which that row does.

Deviations 38 and 39 (`phase-4-exclusions.txt`, pinned in `rexx-parse/tests/errors.rs`) show up here too: three `DO OVER`/`DO WITH ... FOR n TO|BY` rows (ours 25.927, oracle 49.2) and the two `then: nop` rows (ours 18.1/18.2, oracle 35.1). The test licenses exactly those code pairs and asserts the deviating programs are exactly those, so a new deviation or a fixed one goes red. Every other row agrees, including every `LIBRARY` install row (98.903, 90.998, 90.999 through `LIBRARY REXX`).

## Rulings

Ruled by the controller during the task:

1. `Restore` has a fourth variant, `Undecodable`, for the 3.905 the oracle answers on refused base64. `Rejected` stays 3.903. The controller amends Task 9 and Review Focus 2.
2. A narrow early return in `install.rs` for `translate`, witnessed above, with no change to the run path but one `bool` read.
3. `rexx.rs`'s `load` moves into `launcher::load`, and both binaries use it.

Taken by me, for the controller to confirm:

4. The `REGISTERED` predicate is case-insensitive (see TSV agreement).
5. Deviations 38 and 39 are licensed in the TSV agreement, as `rexx-parse/tests/errors.rs` already does. Spec criterion 4 names only the `REGISTERED` exemption.

## Concerns

* Files outside the task's list: `install.rs` (ruled), `bin/rexx.rs` (ruled), `builtin/convert.rs` (`base64_digit` made `pub(crate)` so the file decoder reuses it).
* `rexxc` on an existing `.rxo` input translates its bytes as source. Task 9's restore is what decides that path.
* `.superpowers/sdd/2026-10-15-phase-9c/progress.md` was modified in the working tree by someone else and is untouched by these commits.

## Fix round 1

Ruling (controller, from `task-8-review.md`): the `REGISTERED` exemption covers only rows whose class is `install` and whose decoded program names `REGISTERED` in any case. `exempt(row)` in `tests/rexxc.rs` is that predicate. It selects lines 551 and 959, derived by `awk -F'\t' '$1=="install" && toupper($5) ~ /REGISTERED/ {print NR}' corpus/errors/parse-errors.tsv` from `rust/`.

* `rexxc_agrees_with_every_recorded_parse_error_row` now also asserts that some checked row names `REGISTERED`, and that this crate refuses every exempt row with `(Phase 10 Rxapi)` in stderr. An exemption wider than the refusal goes red.
* Widened back to `names_registered(&row.program)` alone: red on "a translation row naming REGISTERED is checked". With that guard also disabled: red with `exempt rows not refused with the Rxapi reason, at lines [697, 928, 971]`. Both mutants were reversed by exact edits, and `grep -n 'row.class == "install"'` confirmed the restored line.
* Lines 697, 928 and 971 now run and agree. Run by hand through this crate's `rexxc FILE -s` from a fresh directory, each gives `Error 99.917:  Incorrect external name specification "..."` with the recorded insert, line 1, rc 157.
* `--release --test rexxc`: 4 passed. `cargo fmt --all --check`: 0. `cargo clippy -p rexx-exec --tests -- -D warnings`: 0.
