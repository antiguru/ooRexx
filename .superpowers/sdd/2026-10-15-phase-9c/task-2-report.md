# Phase 9c Task 2 report: the raising API, and the scanner and expression sites

Commits: `fdd1ca33d` code (base `cdbbc1568`), with the probe programs, runner and results under
`docs/superpowers/records/2026-10-15-phase-9c/task-2-probes/`; `06cf4a4a7` the witness rows in
`rust/corpus/errors/parse-errors.tsv` and PENDING keyed per row. Fix round 1: `a23fdba9e`
(cold error construction, the full arity check, the re-substitution test) and `dd253ad15`
(`ParseError` is one boxed word).

## What landed

* `rexx-inventory/build.rs` generates `Msg<const N: usize>` (private `major`/`sub`,
  `pub const fn code(self) -> (u16, u16)`) and one `pub const E{major}_{sub}: Msg<N>` per
  catalogue row, `N` being the highest `&k` in the row's text (`highest_insert`).
* `ParseError` is `ParseError(Box<ParseErrorFields>)` since `dd253ad15`. `code`, `sub` and
  `byte` are public fields of `ParseErrorFields` and read through `Deref` as before, and the
  inserts are a `Vec<Vec<u8>>` in the box. This replaces the brief's single
  `Option<Box<..>>` field, which cost startup +0.39% (fix round 1 below). **It deviates from the
  brief in one respect: every error allocates its box, including one with no inserts.**
  `ParseError::raise(msg, byte, inserts: [Vec<u8>; N])` and `new` are `#[cold]
  #[inline(never)]`. `inserts()` answers `&[Vec<u8>]`. The `token.rs` sentence about a `u32`
  keeping the error two words wide is deleted.
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
insert. Ruled by the controller: this boundary stays in Task 2, and Task 3 (plan Step 1b) owns
the `Msg<0>` parameter change with its callers, `directive.rs` included, and the `:499`
restructure. Arity-0 messages render the same through `new`.

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

| | old (`cdbbc1568`) | `fdd1ca33d` | `dd253ad15` |
|---|---|---|---|
| `ParseError` | 16 | 24 | 8 |
| `Result<(), ParseError>` | 24 | 32 | 8 |
| `Result<Token, ParseError>` | 40 | 40 | |

The `Result<Token, ..>` row and the `fdd1ca33d` column come from a throwaway test that printed
`size_of` in both trees. `error/tests.rs`'s `a_parse_error_is_one_word` pins 8 for both
`ParseError` and `Result<(), ParseError>`.

## PENDING

At `fdd1ca33d` PENDING was still keyed by (major, sub): 92 pairs before (Task 1, `cdbbc1568`), 81
after. These pairs came out because every corpus row of each pair matched:
`(6, 1)`, `(13, 1)`, `(15, 1)`, `(15, 2)`, `(15, 3)`, `(15, 4)`, `(20, 930)`, `(30, 1)`,
`(35, 901)`, `(36, 902)`, `(99, 943)`. Counted with
`git show <sha>:rust/crates/rexx-parse/tests/errors.rs | sed -n '/^const PENDING/,/^\];/p' | grep -c '^    ('`.

**Re-keyed per row at `06cf4a4a7`** (controller ruling). A pair key exempted every row of a
pair, so the rows a converted site raises were unchecked while any unconverted site shared the
pair (`(35, 1)`, `(36, 901)`, `(31, 2)` and `(31, 3)` here). `PENDING` is now `&[&str]`, holding
each pending row's `program` field exactly as the corpus file spells it (escapes included). That
key survives row insertion, where a line number does not. `Case` carries the field as `source`,
`message_mismatches` keys by it, and the converse test is renamed
`every_pending_message_row_still_differs_from_the_oracle`. Every row outside PENDING must render
the oracle's message. The list was generated from the test's own mismatch output and has 299
entries (`sed -n '/^const PENDING/,/^\];/p' rust/crates/rexx-parse/tests/errors.rs | grep -c '^    r#'`).

The rows my pairs keep pending, all raised at Task 3 or 4 sites: 31.2 `drop 5`, `use local 1`,
`drop .5`; 31.3 `use local .a`, `drop .`; 35.1 `raise syntax`, `a == 1`, and the two label-colon
deviation rows (18.x here, 35.1 on the oracle), which no conversion makes match; 36.901
`::constant c (1+2`.

### Witness rows added

Twelve rows were appended with `?` in class, expect and line, then filled by
`corpus/errors/derive.sh SCRATCH > NEW` (exit 0, 21 s). `diff` against the input showed only those
twelve lines. Some reach a converted site that no existing row reached: `expr.rs:197`, `:239`
and `:794`, and the line insert of `:961`. The others witness an insert behaviour, such as NUL, a
raw byte, markup or a nested comment, at a site that existing rows already reached. The
reviewer's mutants D, F, G and H show which rows those are. Near-duplicates were skipped:

| Program | Pair | Site |
|---|---|---|
| `address x with input stream (1` | 36.901 position 29 | `expr.rs:239` |
| `parse arg a (b` | 36.901 position 13 | `expr.rs:197` |
| `r = >\n` | 20.930 `found ;` | `expr.rs:794` |
| `r = 1 +,\n    (2\n` | 36.901 position 5 on line 2 | `expr.rs:961`, the line insert |
| `x = a \x00 b\n` | 13.1 `""` | `scanner.rs:615` NUL rule |
| `x = a \x80 b\n` | 13.1 one invalid byte | `scanner.rs:622` |
| `x = a \xf0\x9f\x98\x80 b\n` | 13.1 four bytes | `scanner.rs:622` |
| `r = >'80'x\n` | 20.930 raw byte | `expr.rs:806` |
| `r = >'&1'\n` | 20.930 `found &1` | `expr.rs:806`, an insert that looks like markup |
| `::resource '&1' end '&2'\nline\n` | 99.943 | `scanner.rs:956`, the same markup case |
| `say 1\n\nsay 2 /* open /* nested */\nmore\n` | 6.1 on line 3 | `scanner.rs:383` |
| `o = .object~new\ncall o~string('x')\n` | 35.1 at `~` | the queued ampersand probe |

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

After `06cf4a4a7`, which changes only the TSV and `tests/errors.rs`:

| Command | Exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-parse --all-targets -- -D warnings` (after a `touch` of `tests/errors.rs`, `Checking rexx-parse` printed) | 0 |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-parse -p rexx-inventory --no-fail-fast` | 0, every binary ok, `errors` 30 passed |

The corpus gate and the rexx-exec runs were not repeated: no crate source changed.

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
| M11 35.1 missing right operand inserts nothing (`expr.rs:594`) | same | at `fdd1ca33d`: exit 0, 30 passed. At `06cf4a4a7`: exit 101, rows 1061 and 1062 (`r = a +`, `r = a ||`) |
| M12 13.1 NUL rule disabled (`scanner.rs:615`) | same | at `fdd1ca33d`: exit 0, 30 passed. At `06cf4a4a7`: exit 101, new row 1115 (`x = a \x00 b`) renders `"\0"` against `""` |
| M13 20.930 at the clause end inserts nothing (`expr.rs:794`) | same, at `06cf4a4a7` | exit 101, new row 1113 (`r = >`) `found .` against `found ;` |
| M14 35.1 in `parse_constant_expression` inserts nothing (`expr.rs:244`) | same, at `06cf4a4a7` | exit 101, row 199 (`address system with error using x`), which the pair key had exempted |
| M15 a matching row (`r = >\n`) put in PENDING | same, at `06cf4a4a7` | exit 101, `every_pending_message_row_still_differs_from_the_oracle` FAILED, `PENDING entries that now match: ["r = >\\n"]` |

M11 and M12 stayed green at `fdd1ca33d`, under the pair key. The per-row key and the new rows
turn them red.

## Fix round 1

### Startup: the measurement and the fix

The review measured Task 2 at startup +0.3611% against `cdbbc1568`. Moving every
insert-building construction into `#[cold] #[inline(never)]` helpers came first (`a23fdba9e`).
In `scanner.rs` these are `invalid_character`, `unmatched_comment`, `name_too_long`,
`missing_resource_end`, `misplaced_blank` and `invalid_digit`. In `expr.rs` they are
`not_a_variable`, `invalid_at`, `operator_without_term`, `not_a_reference`, `unmatched` and
`raise`, plus `ParseError::raise` itself. `unmatched` takes the opener's token index, and only the
error path reads the token. With all that, startup was still +0.2737% against the base (callgrind,
3 rounds, `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9c-t2-fix1`).

The per-function diff put the rest on `rexx_parse::parse`'s self cost, where `scan` is inlined:
+444,597 Ir. The token `Result`s were unchanged at 40 bytes. An experiment kept every helper and
removed only the `inserts` field (a throwaway build with `inserts()` answering `&[]`). One
callgrind round against the base gave startup -0.1136% and interpret -0.0769%, against +0.2737%
and +0.0561% for `a23fdba9e`. So the field itself was the cost. A candidate with the fields boxed
behind one word gave -0.2810%, -0.0901% and -0.0199% in one round. Committed as `dd253ad15`: its
release binary is byte-identical (`cmp`) to the candidate's.

Command, from `rust/`:
`bash bench-programs/callgrind.sh -B 0.5 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9c-t2-fix1b -p 'startup interpret parse' pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run base=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9c-t2-cdbbc1568 head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9c-t2-dd253ad15`,
exit 0, 3 rounds. The base was built in a `git worktree add --detach` copy at `cdbbc1568` after
touching every file (`Compiling rexx-parse` printed once), with
`CARGO_TARGET_DIR` under the scratchpad. The worktree and that target dir were removed afterwards.

| program | pin | base `cdbbc1568` | head `dd253ad15` | base vs pin | head vs pin | **head vs base (Task 2)** | spreads (pin/base/head) |
|---|---|---|---|---|---|---|---|
| startup | 119,095,681 | 119,494,272 | 119,166,974 | +0.3347% | +0.0599% | **-0.2739%** | 0.0000 / 0.0042 / 0.0034% |
| interpret | 4,207,371,545 | 4,219,634,339 | 4,215,836,639 | +0.2915% | +0.2012% | **-0.0900%** | 0.0000 / 0.0001 / 0.0000% |
| parse | 1,654,598,893 | 1,653,123,818 | 1,652,794,456 | -0.0892% | -0.1091% | **-0.0199%** | 0.0000 / 0.0002 / 0.0000% |

All three are under the +0.05% target. The libc-subtracted deltas, head against base, are
-0.6013%, -0.2303% and -0.0224% in the one-round candidate run. The full table, with libc-subtracted
columns, is in `cg-9c-t2-fix1b`'s `summary.tsv`.

### Minors

* `an_insert_is_not_substituted_again` now uses `E99_943` with `[b"&2", b"&1"]`. Mutant ME
  substitutes twice and turns it red (exit 101, along with `an_insert_reaches_the_message_unconverted`).
* `build.rs` also generates `ARITIES: &[((u16, u16), usize)]`, one entry per constant, built from
  `Msg::arity()` (`pub const fn`, answering `N`).
  `every_constants_arity_is_its_texts_highest_placeholder` checks every entry against the text
  `lookup` returns, and that `ARITIES` is as long as `MESSAGES`. Mutant M16 caps each insert number
  at 3. That spares every sampled constant and turns the test red (exit 101).
* The false "no existing row reached" sentence is replaced, above.
* `previous()` is gone. See the startup section.

### Checks at `dd253ad15`, from `rust/`, exit statuses unpiped

| Command | Exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-inventory -p rexx-parse -p rexx-exec --all-targets -- -D warnings` | 0 |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-parse -p rexx-inventory --no-fail-fast` | 0, no failures |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --lib error::` | 0, 13 passed |
| the six refusal-table binaries, as above | 0 |
| `REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus` | 0, `1062 of 1062 matching` |

## Concerns

1. **`tests/errors.rs`'s `an_interpret_error_raised_without_its_insert_keeps_the_marker`
   pins `found "&1"`** for 47.1, raised at `instruction.rs:424` with no insert. The oracle prints
   `found "X"`. Task 3 has to change the expectation when it converts that site.
2. **The two label-colon deviation rows stay in PENDING under any conversion.** We raise 18.x
   there and the oracle raises 35.1. Task 4 empties PENDING and has to decide what happens to them.
3. **`ParseError` is boxed whole, and every error allocates its box** (`dd253ad15`), where the
   brief has a message with no inserts allocate nothing. The brief's shape measured startup
   +0.39% over this one. The controller should rule on whether that trade stands.
