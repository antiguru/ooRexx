# Task 8 review: `rexxc` and the `.rxo` writer

Diff `6b10e78ef..3db140a2a`. Probes ran on a detached worktree at `3db140a2a`, release build in
`scratchpad/target-rev-8`, crate binaries under `memcap 2G` and `timeout`, oracle by the
global-constraints recipe, every run from a fresh directory. The worktree was removed clean.

## Spec compliance

✅ Spec compliant, with the controller rulings implemented as stated.

* `compiled::write_image`, `compiled::restore` and `enum Restore { Source, Compiled, Rejected, Undecodable }`:
  `rust/crates/rexx-exec/src/compiled.rs:767-776`, `:793-832`.
* `launcher::translate(path, text) -> Outcome` takes the path through `as_bytes`:
  `launcher.rs:1105-1107` (diff lines). `launcher::load` moved from `bin/rexx.rs`, which now calls it.
* Translate-only early return: `install.rs:1015-1019` (diff lines), after `override_package_settings`.
  The run path pays one `bool` read. `translate_only` is set only by `Interp::translate`.
* `Undecodable` is 3.905 and `Rejected` 3.903, per the `Restore` doc comments.
* REGISTERED predicate ignores case: `tests/rexxc.rs` `names_registered` (`eq_ignore_ascii_case`).
  Confirmed against `grep -n -i REGISTERED corpus/errors/parse-errors.tsv`: rows 551, 697, 928, 959, 971.
  Row 959 is the lower-case one.
* Deviations 38 and 39 are licensed as code pairs, then pinned by the exact set of five programs
  (`tests/rexxc.rs`, the `assert_eq!(deviations, [...])`).
* Harness: `interpreter_bin` places `rexxc` (`group_runner.rs`, `place`). `check_path` requires both
  binaries in `bin`. The two tests are turned around. `ChildPath.testGroup:16` drops the `echo $?`.
* D5: `the_oracle_refuses_this_crates_images_as_another_versions` runs oracle `rexx` on both forms
  and gets 3.903, rc 253.

⚠️ I did not re-run the harness claim that 9 of 11 `utilities/rexxc/rexxc` tests pass. It rests on
`docs/superpowers/records/2026-10-15-phase-9c/task-8-suite/`.

## Checks run

### `rexxc` against the oracle on shapes the cases file lacks

Each probe compared rc, normalised stdout, stderr and the directory listing. Every shape below was
SAME unless marked otherwise.

* Odd orders and repeated flags: `-s ok.rex`, `ok.rex -s o.rxo`, `ok.rex o.rxo -s -s`,
  `ok.rex -e -e -s`, `ok.rex o.rxo o2.rxo`, `ok.rex o.rxo -s -e -x`, `ok.rex -s -`,
  `ok.rex o.rxo -S -s -e -E`.
* Directory input: `sub.rex` (a directory) and `sub` give 3.901. A directory named as output
  (`ok.rex subdir`) gives 3.902. Directory input with an output gives 3.901.
* Unreadable input (mode 000), `noread.rex` and `noread`: 3.1 with the absolute path.
* Output in a missing directory, two levels deep (`nodir/x/y.rxo`): 3.902.
* Non-ASCII UTF-8 names: `été` → `é.rxo`, found without the extension. `bé` gives a syntax error
  that reports the path byte for byte. An output under `né/` gives 3.902.
* `-s` with a syntax error on line 3 of 4: line 3 reported, rc 221, with and without an output,
  with and without `-s`.
* `#!` input: `bang.rex` checks clean and writes an image. `bangbad.rex` reports line 3.
  An empty input writes an image.
* Input that is an image (the crate's binary and encoded `.rxo`): both sides read it as source,
  13.1 on line 2, rc 243.
* Output naming the input by a different spelling (`./ok.rex ok.rex`): both sides overwrite the
  source with an image, rc 0.
* Translation boundary: a `::REQUIRES` of a file with a syntax error is rc 0. `::CONSTANT (1/0)`
  without a class is 99.906, and with a class it is rc 0. `LIBRARY rexxutil NoSuchFn` is 90.999.
  A duplicate `::ROUTINE` is 99.903.
* **Diverged:** `::class c` / `::method m attribute external "LIBRARY x"` / `::options digits 0`.
  The oracle answers 98.903 (line 2, rc 158) and the crate 26.5 (line 3, rc 230). The crate's `rexx`
  diverges the same way, and so does the shorter `::routine r external "LIBRARY x"` /
  `::options digits 0`. The divergence predates this task (see Minor 1).

### Header bytes and encoded lines

* `xxd` of the crate's `bang.rex` image: `#!` line (20 bytes), `/**/@REXX` and 7 NULs, magic
  `73 72`, version `01 00`, wordSize `40 00`, bigEndian 0, level `b8 ec 00 00` (60600), reserved
  and pad zero, imageSize `1a` (26, the source with its `#!` line), then the source. This matches the
  ruling and `ProgramMetaData.hpp:78-94` field for field. The oracle's image differs only in magic
  (`67 2b`), version (`2b 00`), size and data.
* `-e` on a 707-byte source: every line is 72 columns, the last ends in `\n`. The oracle has the
  same 72-column lines and final line end.

### TSV agreement test can fail

Mutations to the review worktree's `parse-errors.tsv`, each run with the built `rexxc` test binary
and `rexxc_agrees` as the filter:

* Baseline: ok.
* Row 199 main line `1` → `2`: FAILED, `1 of 1104 rows disagree`, the row named.
* Row 199 message `"X"` → `"Y"`: FAILED, `1 of 1104 rows disagree`.
* Reversed by exact edit. `git diff --stat` was empty and the test was ok again.

### `compiled::restore` fallbacks

Read against `ProgramMetaData::processRestoreData` (`ProgramMetaData.cpp:287-352`) and
`StringUtil::decodeBase64` (`StringUtil.cpp:1743-1871`).

* Truncation inside the header is `Rejected` (`split_at_checked`). A length past the end or shorter
  than the data is `Rejected` (exact `!=`). Both are covered by unit tests.
* Tag tests match the oracle's: binary `length > 9 && strcmp`, encoded `length > 11 && memcmp`.
  A `#!` line with no line end is `Source`. The oracle's `return false` also means source.
* `decode` matches the oracle's rules: `=` only at digits 2 and 3, `==` at digit 2, line ends only
  at digit 0, only line ends after padding, and an incomplete last quartet accepted.
* A truncated encoded image decodes short, so it is `Rejected` by length. This holds by reading the
  code. No unit test covers it, and Task 9 pins it.

## Strengths

* `restore` follows the oracle's decision procedure closely, down to the NUL in the `strcmp` tag
  and the `>` length guards. The decoder reuses `base64_digit` rather than copying a table.
* `rexxc` builds the image before translating and writes it only after success. A failed write
  removes the partial file. Every Review Focus 3 shape I tried agrees with the oracle.
* The cases file records oracle output and compares all three descriptors plus the image prefix.
  The TSV test checks rc, the secondary line and the main line. The mutations show each of those
  checks bites.
* `place` in `group_runner.rs` removes the duplicated link logic instead of copying it per binary.

## Issues

### Critical

None.

### Important

None.

### Minor

1. Pre-existing divergence, inherited by `rexxc`. `::routine r external "LIBRARY x"` followed by
   `::options digits 0` answers 26.5 (rc 230) on the crate's `rexx` and `rexxc`. The oracle's `rexx`
   and `rexxc` answer 98.903 (rc 158). The oracle binds the library before it validates
   `::OPTIONS`. No TSV row covers the pair. This is outside Task 8's diff. Queue it for the
   install-order owner.
2. `compiled.rs`, `a_foreign_or_damaged_header_is_rejected`: an encoded image whose decoded bytes
   lack the tag is `Rejected` (3.903). Measured, oracle `rexx`: `#!..\n/**/@REXX@\n` plus the base64
   of `say 1;` ×40 runs. It prints `1` forty times and then runs the leftover base64 text as
   commands, rc 0. `validate` fails without raising, and the parser reads the buffer that was
   decoded in place. The crate's answer is the sensible one, but it is an unrecorded divergence.
   Task 9 should license it explicitly and not inherit it silently.
3. `tests/rexxc.rs` `names_registered`: the exemption covers five rows, but only rows 551 and 959
   are the Rxapi gap. Rows 697, 928 and 971 are 99.917 translation errors, and the crate agrees on
   them (checked on 697 and 971: 99.917, rc 157). Exempting them hides any regression there from
   this test. The brief mandated the predicate, so this is plan-mandated. A narrower predicate
   would exempt only rows the crate actually refuses, for example rows whose expected class is
   `install`.
4. `compiled.rs` tests: no unit test truncates the encoded form (inside a line, or inside a
   quartet). Behaviour is correct by reading. Task 9 is assigned to pin it.

## Assessment

**Spec compliance:** ✅ Spec compliant, with one item to verify (the harness pass count).

**Task quality:** Approved.

**Reasoning:** Every probed command-line shape agrees with the oracle byte for byte, and the header
matches the ruling field for field. Mutating the TSV turns the agreement test red. The findings are
either pre-existing or belong to Task 9.
