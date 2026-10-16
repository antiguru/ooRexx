# Phase 9c Task 2 report: the raising API, and the scanner and expression sites

Code commit: `fdd1ca33d` (base `cdbbc1568`). The probe programs, runner and results are
committed in it under `docs/superpowers/records/2026-10-15-phase-9c/task-2-probes/`.

## What landed

* `rexx-inventory/build.rs` generates `Msg<const N: usize>` (private `major`/`sub`,
  `pub const fn code(self) -> (u16, u16)`) and one `pub const E{major}_{sub}: Msg<N>` per
  catalogue row, `N` being the highest `&k` in the row's text (`highest_insert`).
* `ParseError` gains `inserts: Option<Box<Vec<Vec<u8>>>>` (`clippy::box_collection` is
  `#[expect]`ed with its reason: a boxed slice is two words). `ParseError::raise(msg, byte,
  inserts: [Vec<u8>; N])` stores `None` when `N == 0`. `inserts()` answers `&[Vec<u8>]`.
  `code` and `sub` stay public fields. The `token.rs` sentence about a `u32` keeping the error
  two words wide is deleted.
* `ParseError::message()` returns `Vec<u8>`: the (major, sub) row with `&k` replaced by insert
  `k`, a `&k` with no insert left as it is (the oracle's `messageSubstitution` loop runs only
  `additional->size()` times, `Activity.cpp:1246`). No fallback to the major's text, and the
  `has_placeholder` machinery and its tests are gone with it. `Display` decodes it with
  `String::from_utf8_lossy`.
* `From<&ParseError> for Raised` passes `error.inserts().to_vec()` as `additional`.
* Every raise site in `scanner.rs` and `expr.rs` is on `ParseError::raise`, except the one below.

### The site left on `ParseError::new`

`parse_expr`'s and `comma_list`'s absent-expression raise (`expr.rs:145`, `:509`) take the
35 sub from their callers as `missing: u16`, through `missing_expression`. The callers are in
`instruction.rs`, `instruction/loops.rs`, `instruction/address.rs` and `directive.rs`. Every
caller passes an arity-0 sub except `instruction.rs:499`, which passes 35.1 where the oracle
(`InstructionParser.cpp:233`, `:250`) raises 35.1 with the `=` or assignment-operator token as
insert. Changing the parameter to `Msg<0>` forces edits in Task 3 and 4 files and a restructure
of `:499`, so I left it and asked the controller. Arity-0 messages render the same through
`new`, so no PENDING pair depends on it except 35.1 through `:499`.

## Site table

Oracle paths are under `/home/moritz/dev/repos/ooRexx/interpreter/`. "display" is
`RexxToken::displayValue` (`parser/Token.cpp:308`), implemented as `expr.rs`'s `display_value`:
a symbol's upcased name, a literal's value, an operator's `GlobalNames` spelling, an assignment
operator's spelling plus `=`, the punctuation's own spelling, and `;` for the end of the clause.
"position" is `errorPosition` (`parser/LanguageParser.cpp:4114`): the opener's 1-based byte
offset in its line, then its line number.

| Rust site | Oracle site | Inserts |
|---|---|---|
| `scanner.rs:383` 6.1 | `parser/Scanner.cpp:232` | line the comment opened on (`startLine`, read after stepping over `/*`) |
| `scanner.rs:622` 13.1 | `parser/Scanner.cpp:1207` | the bad UTF-8 sequence (1 to 4 bytes, one byte where invalid, empty for NUL as `snprintf("%s")` stops there); its bytes as uppercase hex |
| `scanner.rs:766` 30.1 | `parser/Scanner.cpp:1524` | the symbol upcased |
| `scanner.rs:821` 6.2 / 6.3 | `parser/Scanner.cpp:1650`, `:1654` | none (chosen at run time between two `Msg<0>`) |
| `scanner.rs:956` 99.943 | `parser/DirectiveParser.cpp:2337` | end marker, resource name (`token->value()`) |
| `scanner.rs:1015` 15.1 | `parser/Scanner.cpp:453` | position `i` from the right-to-left scan |
| `scanner.rs:1017` 15.5 | `parser/Scanner.cpp:459` | none |
| `scanner.rs:1045` 15.3 | `parser/Scanner.cpp:546` | the bad character, raw |
| `scanner.rs:1071` 15.2 | `parser/Scanner.cpp:607` | position `i` |
| `scanner.rs:1073` 15.6 | `parser/Scanner.cpp:613` | none |
| `scanner.rs:1105` 15.4 | `parser/Scanner.cpp:696` | the bad character, raw |
| `expr.rs:197` 36.901 (`parse_paren_expression`) | `parser/LanguageParser.cpp:2705` | position of the `(` the caller consumed |
| `expr.rs:239` 36.901 (`parse_constant_expression`) | `parser/LanguageParser.cpp:2675` | position of the `(` |
| `expr.rs:244` 35.1 (`parse_constant_expression`) | `parser/LanguageParser.cpp:2663` | display of the token |
| `expr.rs:308` 31.2 / 31.3 (`need_variable`) | `parser/LanguageParser.cpp:894`, `:898` | the symbol's name (chosen at run time between two `Msg<1>`) |
| `expr.rs:567` 35.1 dyadic `\` | `parser/LanguageParser.cpp:2921` | display (`\`) |
| `expr.rs:581` 37.2 | `parser/LanguageParser.cpp:2984` | none |
| `expr.rs:582` 37.901 | `parser/LanguageParser.cpp:2991` | none |
| `expr.rs:584` 35.1 (assignment op, anything else) | `parser/LanguageParser.cpp:2975`, `:2998` | display |
| `expr.rs:594` 35.1 missing right operand | `parser/LanguageParser.cpp:2963` | the operator's spelling |
| `expr.rs:623` 35.1 non-prefix operator | `parser/LanguageParser.cpp:3672` | display |
| `expr.rs:631` 35.901 | `parser/LanguageParser.cpp:3655` | the prefix operator's spelling |
| `expr.rs:719`, `:721` 19.909 | `parser/LanguageParser.cpp:3385`, `:3348` | none |
| `expr.rs:763`-`:782` 20.917 | `parser/LanguageParser.cpp:3407`, `:3348` | none |
| `expr.rs:794`, `:806` 20.930 | `parser/LanguageParser.cpp:3729` | display of the token after the prefix, `;` at the clause end |
| `expr.rs:826`, `:949` 11.1 | `concurrency/Activity.cpp:2308` | none |
| `expr.rs:865` 36.901 / 36.902 (`arg_list_inner`) | `parser/LanguageParser.cpp:3133`, `:3139` | position of the `(` or `[` consumed just before the list (the oracle's `firstToken`, every caller at `:3231`, `:3281`, `:3317`, `:3431`, `InstructionParser.cpp:2543`, `:3672`) |
| `expr.rs:958` 35.1 `()` | `parser/LanguageParser.cpp:3788` | display (`(`) |
| `expr.rs:961` 36.901 | `parser/LanguageParser.cpp:3793` | position of the `(` |
| `expr.rs:1018` 35.1 operator as a term | `parser/LanguageParser.cpp:3871` | display |
| `expr.rs:1020`, `:1021`, `:1024` 37.2, 37.901, 37.1 | `parser/LanguageParser.cpp:3878`, `:3886`, `:3882` | none |
| `expr.rs:1027` 35.1 | `parser/LanguageParser.cpp:3891` | display |
| `expr.rs:1035`, `:1038` 20.923 | `parser/LanguageParser.cpp:3267` | none |

## `size_of::<ParseError>()`

Measured by compiling each struct definition alone with `rustc --edition 2021` and printing
`size_of` (old: `git show HEAD:rust/crates/rexx-parse/src/token.rs`'s struct at `cdbbc1568`):

| | old | new |
|---|---|---|
| `ParseError` | 16 | 24 |
| `Result<(), ParseError>` | 24 | 32 |

`error/tests.rs`'s `a_parse_error_is_three_words` pins 24.

## PENDING

Before (Task 1, `cdbbc1568`): 92 pairs. Removed, each because every corpus row of the pair now matches
(`every_pending_message_pair_still_differs_from_the_oracle` named them):
`(6, 1)`, `(13, 1)`, `(15, 1)`, `(15, 2)`, `(15, 3)`, `(15, 4)`, `(20, 930)`, `(30, 1)`,
`(35, 901)`, `(36, 902)`, `(99, 943)`. After (`fdd1ca33d`): 81. Counted with
`git show <sha>:rust/crates/rexx-parse/tests/errors.rs | sed -n '/^const PENDING/,/^\];/p' | grep -c '^    ('`.

The pairs my sites raise that stay, with the rows that keep them (all from Task 3 or 4 sites):
`(31, 2)` rows 361, 480, 800 (`drop 5`, `use local 1`, `drop .5`); `(31, 3)` rows 142, 359
(`use local .a`, `drop .`); `(35, 1)` rows 510, 660 (`raise syntax`, `a == 1`) and 442, 761 (the
recorded label-colon deviation, 18.x here against 35.1); `(36, 901)` row 377
(`::constant c (1+2`).

## Probes

`docs/superpowers/records/2026-10-15-phase-9c/task-2-probes/`: `mk.py`, `mk2.py`, `mk3.py` write
the programs, `run.sh` runs each from its own probe directory on
`build/bin/rexx` (under `ulimit -v 1048576`, `timeout -k 5 20`) and on `rust/target/debug/rexx-run`
built from this tree (under `memcap 2G`, `timeout -k 5 20`), comparing stdout, stderr and rc
separately. `results.txt` has every program's rcs, both comparisons and both sides' `Error N.M:`
line through `cat -v`. Every program is one run per side; none is concurrent.

All 108 programs: stdout identical, stderr identical, rc identical (`grep -c DIFF results.txt`
answers 0, and `run.sh` answered `yes` for every program). stdout is empty for every error
program. The ones the brief names, crate stderr secondary lines:

* scout C 35.1 at `"("`, `say ()`: `Error 35.1:  Incorrect expression detected at "(".`, rc 221.
* scout C 13.1, `b\xc3\xa4c = 2` on line 2: `Error 13.1:  Incorrect character in program "M-CM-$" ('C3A4'X).`, rc 243.
  Also `\x80` (`"M-^@" ('80'X)`), NUL (`"" ('00'X)`), `\x01` (`"?" ('01'X)`, the report's
  `displayable`), a 4-byte sequence (`('F09F9880'X)`), an overlong `\xc0\xaf` (`"M-@" ('C0'X)`)
  and a `\xc3` at the end of the file (`"M-C" ('C3'X)`).
* `2026-10-13-error-position-ampersand.md`, `call o~string('x')`: `Error 35.1:  Incorrect expression detected at "~".`, rc 221.

Inserts that look like markup reach the output once, unsubstituted: `x = >'&1'` gives
`found &1.`, and `::resource '&1' end '&2'` gives `Missing ::RESOURCE end marker "&2" for
resource "&1".` A raw byte from a literal: `x = >'80'x` gives `found M-^@.` on both sides.

Probes that did not reach the path they were named for, read from their stdout and stderr:
`hex_group` and `bin_group` translate (`'1 23'x` and `'1 0101'b` are legal), `colon_in_expr` and
`qualified_bad` are run-time 98.987, `prefix_missing` (`x = -`) is 35.918, `say_prefix` (`say -`)
runs with rc 0, `e39` (`r = a = = 1`) runs. `hex_bad_group`, `bin_bad_group`, `not_prefix`
(35.901) and `e42` (`r = +`, 35.901) cover the paths those missed.

## Commands, exit statuses unpiped

From `rust/`, all at the committed tree:

| Command | Exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-inventory -p rexx-parse -p rexx-exec --all-targets -- -D warnings` | 0 |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-parse -p rexx-inventory --no-fail-fast` | 0 (rexx-parse lib 261 passed, `errors` 30, inventory `errors` 7, `oracle_agreement` 1, no failures) |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --lib error::` | 0 (13 passed) |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --no-fail-fast` | 0 (8, 6, 5, 3, 5, 3 passed) |
| `REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus` | 0, `1062 of 1062 matching`, 29 passed 1 ignored |

No ooTest groups are named for this task, so none were run.

## Mutation evidence

Each mutant applied by exact replacement (`mutate.py`, kept in the session scratchpad), run, and
restored from a copy, with `git diff --stat -- rust` compared against the pre-mutation baseline
after each batch (identical every time).

| Mutant | Run | Result |
|---|---|---|
| M1 `highest_insert` caps each digit at 1 | `-p rexx-inventory --test errors` | exit 101, `a_constants_arity_is_its_texts_highest_placeholder` FAILED |
| M1b `highest_insert`'s `unwrap_or(0)` to `unwrap_or(1)` | same | exit 101, E0308 at `let _: errors::Msg<0> = errors::E8_2` |
| M2 `raise` stores `Some` for `N == 0` | `-p rexx-parse --lib error::` | exit 101, `a_message_with_no_inserts_carries_none` FAILED |
| M3 `substitute` indexes `digit - b'0'` | same | exit 101, five tests FAILED, among them `a_token_insert_fills_the_sub_message` and `an_insert_reaches_the_message_unconverted` |
| M4 `From<&ParseError>` passes `Vec::new()` | `-p rexx-exec --lib error::` | exit 101, `a_parse_errors_inserts_render_the_same_secondary_line_through_raised` FAILED |
| M5 13.1 hex lowercase | `-p rexx-parse --test errors` | exit 101, row 1044 `('c3a4'X)` against `('C3A4'X)` |
| M6 6.1 line plus one | same | exit 101, row 1036 `on line 2` against `on line 1` |
| M7 36.x position without the `+ 1` | same | exit 101, row 1074 `position 5` against `position 6` |
| M8 20.930 insert emptied | same | exit 101, row 1105 `found .` against `found A.B.` |
| M9 30.1 insert not upcased | same | exit 101, row 1054 |
| M10 99.943 inserts swapped | same | exit 101, row 1053 |
| M11 35.1 missing right operand inserts nothing (`expr.rs:594`) | same | **exit 0, 30 passed** |
| M12 13.1 NUL rule disabled (`scanner.rs:615`) | same | **exit 0, 30 passed** |

M11 and M12 survive: no corpus row covers a NUL source byte, and every 35.1 row is exempt
while `(35, 1)` stays in PENDING. The same holds for every expr.rs 35.1 site and for 36.901,
31.2 and 31.3. The probes witness them (results above), but no test does.

## Concerns

1. **The `missing: u16` boundary** (section above). I sent the question to the controller and
   had no answer when I committed.
2. **No test gates the expr.rs 35.1, 36.901, 31.2 and 31.3 inserts, or the 13.1 NUL rule**
   (M11, M12). The durable witness would be `parse-errors.tsv` rows, which is Task 4's file. I
   asked the controller whether to add them here and had no answer when I committed. The
   `results.txt` shapes are ready to become rows.
3. **`tests/errors.rs`'s `an_interpret_error_raised_without_its_insert_keeps_the_marker`
   pins `found "&1"`** for 47.1, raised at `instruction.rs:424` with no insert. The oracle prints
   `found "X"`. Task 3 has to change the expectation when it converts that site.
4. `Result<(), ParseError>` grows from 24 to 32 bytes: the `Option<Box<..>>` niche is spent on
   the `Option`. Parse-time only, and no perf gate ran in this task.
