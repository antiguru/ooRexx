# Phase 9c Task 4 report: the block, directive and program sites, the coverage test, and the old API gone

Status: DONE. Base `d5ea1577b`. Commits:

* `dbbcdafb0` the conversion, `ParseError::new` and `PENDING` deleted, the coverage test, the rows,
  the exclusions edit, `suite-9d-set.txt`, and the records under
  `docs/superpowers/records/2026-10-15-phase-9c/task-4-{mutants,probes,suite}/`.
* `e1bfac290` the directive messages whose oracle raise passes a token the text does not show
  (`ARITY_OVERRIDES`), with the witness `corpus/lang/parse_error_directive_insert.rex`.
* `16e89ac59` INTERPRET line inserts (ruled by the controller, see Rulings), with the witness
  `corpus/lang/parse_error_interpret_line.rex`.

## Rulings received during the task

1. 11.1 (`MAX_EXPR_DEPTH`) gets a TSV row, and `tests/errors.rs`'s `parse()` runs every program on a
   512 MiB thread as `tests/deep.rs` does, with no length threshold. Applied.
2. INTERPRET line inserts are fixed here, widened to `rexx-exec/src/run/interpret.rs` and
   `rexx-parse/src/source.rs`, message line inserts only. Applied in `16e89ac59`.

## What landed

* `block.rs`: `Block` holds the `ProgramSource`. `line_insert`, `error_line` (`errorLine`,
  `LanguageParser.cpp:4101`), `then_expected`, `block_error` (now takes the `Frame`), `end_mismatch`
  and `misplaced_label` are the insert builders, all `#[cold] #[inline(never)]` apart from
  `line_insert`, which only they call. `match_label` is a method taking the block's index.
  `missing_then_sub` is deleted from `instruction.rs`.
* `directive.rs`: `error<N>`, `found` (display value of the token, `None` the end of clause),
  `unmatched_paren` and `bad_trace_setting`, all cold. `required_end(Msg<1>)` names the next token.
  `next_name` and `next_symbol` answer the token found instead, and `require_name`,
  `require_literal` and `class_reference` take a `Msg<1>` naming it. `require_upper_name` is gone
  (its only caller was `::ANNOTATE`, now `annotated_name`). The per-iteration `let bad =
  self.error(..)` in the `::CLASS`, `::METHOD`, `::ATTRIBUTE`, `::OPTIONS`, `::REQUIRES` and
  `::ROUTINE` loops, which allocated an error for every sub-keyword, is gone. `signed_constant`
  answers `NotSigned { token, value }`, because `::CONSTANT`'s 19.916 names the sign where
  `::ANNOTATE`'s 19.923 names the signed value (`DirectiveParser.cpp:1905` against `:2249`).
* `lib.rs`: 99.914 raises `E99_914`.
* `token.rs`: `ParseError::new` is deleted. `raise` is the only constructor
  (`grep -rn "ParseErrorFields {" rust/crates` finds the struct, the two `Deref` impls and the
  literal in `raise`).
* `rexx-exec/src/error.rs`: `a_parse_error_becomes_the_condition_the_oracle_raises` built its errors
  with `ParseError::new`; it now uses `raise` with `E27_901` and `E35_929` and asserts the pair
  literally. Outside the brief's file list, forced by the deletion; reported to the controller.
* `rexx-inventory/build.rs`: `ARITY_OVERRIDES` entries, each citing its `DirectiveParser.cpp` raise
  sites (`e1bfac290`). Every cited line was printed with `sed -n` and holds the raise.
* `source.rs` (`16e89ac59`): `ProgramSource::interpret(text, line)` and `message_line(line)`, which
  answers the INTERPRET clause's line for an `Interpret` source and `line` otherwise. Every line
  insert goes through it: `block.rs` (14.x, 7.x, 10.x, end-of-body 18.x), `instruction.rs` (18.x),
  `expr.rs` and `directive.rs` (36.x), `scanner.rs` (6.1). `run/interpret.rs` passes
  `clause_state.line()`, the value `raise_parse_failure` already uses for the reported line. The set
  of line-insert messages was derived with
  `grep -n 'line &' interpreter/messages/RexxErrorMessages.h`, each checked to be named in
  `rexx-parse/src` or not: every one is except 101.0.

## Site table

Oracle paths under `interpreter/`; `LP` = `parser/LanguageParser.cpp`, `DP` =
`parser/DirectiveParser.cpp`. "tok" is `displayValue` of the token named, `;` at the end of the
clause. "line" passes through `message_line`.

| Rust site (HEAD) | Oracle | Inserts |
|---|---|---|
| `block.rs:397` 14.1, `:399` 14.2, `:417` 14.4, `:420` 14.5, `:423` 14.901 | `LP:4192`, `:4224`, `:4243`, `:4218`, `:4229` (`blockError`) | line of the unclosed instruction |
| `block.rs:410` 14.3, the source ends after the THEN | `LP:1371`, `:1396` | line of the IF or WHEN (`frame.parent`) |
| `block.rs:415` 14.3, anything else after the THEN | `LP:4238` | line of the THEN |
| `block.rs:443`, `:448`, `:449` 47.3 / 47.4 / 47.2 (`misplaced_label`, `:683`) | `LP:1240`-`1242` | the label's name (`token` after `resetPosition(1)`) |
| `block.rs:857` 47.3 before an ELSE | `LP:1431` | `ELSE` |
| `block.rs:505` 10.2 / 10.4 (`end_mismatch`) | `instructions/BaseDoInstruction.cpp:201`, `SelectInstruction.cpp:201` | END name, block label, block line |
| `block.rs:511` 10.3 / 10.7 | `BaseDoInstruction.cpp:195`, `SelectInstruction.cpp:197` | END name, block line |
| `block.rs:551` 7.1 | `SelectInstruction.cpp:212` | SELECT line |
| `block.rs:822` 7.2 | `LP:1308` (`topDo()`) | SELECT line |
| `block.rs:380` 18.1 / 18.2 at the end of the body (`then_expected`) | `LP:1348` | line of the IF or WHEN |
| `block.rs:612` 10.1, `:851` 8.2, `:869` 9.2 | `LP:1528`, `:1423`, `:1480` | none |
| `lib.rs:203` 99.914 | `LP:1119` | none |
| `directive.rs:422` 25.901, `:1137` 25.901 (second EXTERNAL) | `DP:387`-`490`, `:2606` | tok |
| `directive.rs:537` 25.902, `:647` 25.925, `:1147` 25.903, `:1059` 25.904, `:1000` 25.924, `:1077` 25.926, `:853` 25.928 | `DP:666`-`815`, `:1493`-`1646`, `:2597`-`2639`, `:2803`-`2851`, `:963`/`:1362`, `:2291`/`:2297`, `:2103` | tok |
| `directive.rs:933` 25.11, `:996` 25.935 | `DP:1021`, `:1353` | tok |
| `directive.rs:1026` 25.927 | `DP:1105`-`1299` | the option keyword, `token->value()` |
| `directive.rs:1014`, `:1017` 20.929 | `DP:1086`-`1270` | the option keyword |
| `directive.rs:917` 26.5, `:948` 26.6 | `DP:985`, `:1044` | `token->value()` |
| `directive.rs:190` 24.1 (`bad_trace_setting`) | `DP:1072` | first byte that is not `?` |
| `directive.rs:303` 99.925 (`require_variable_name`) | `LP:2529` | the name |
| `directive.rs:716` 99.917 (`decode_external`) | `DP:1414`, `:1433`, `:1438`, `:2652`-`2748` | the external string |
| `directive.rs:170` 36.901 (`unmatched_paren`) | `LP:1754` (`errorPosition`, `LP:4114`) | the `(`'s offset in its line, then line |
| `directive.rs:762` 21.913, `:1086` 21.914 (`required_end`) | `DP:1915`, `:2311` (`requiredEndOfClause`) | tok after the clause's end |
| `directive.rs:870` 19.925 (`annotated_name`, called at `:840`-`:846`) | `DP:1972`-`2073` | `::ANNOTATE`, the target keyword |
| `directive.rs:880` 20.919 | `DP:2214` | tok |
| `directive.rs:893` 19.923 | `DP:2235`, `:2242`, `:2249` | tok, the sign, the signed value |
| `directive.rs:757` 19.916 | `DP:1891`, `:1898`, `:1905` | tok, the sign, the sign |
| `require_name` callers 19.902 `:469`, 19.903 `:1118`, 19.904 `:1035`, 19.914 `:577`, 19.915 `:742`, 19.917 `:913`, 19.918 `:942`, 19.919 `:955`, 19.920 `:1069`, 19.921 `:1083` | `DP:646`, `:2572`, `:2788`, `:1473`, `:1860`, `:977`, `:1034`, `:1062`, `:2272`, `:2304` | tok (unshown, `ARITY_OVERRIDES`) |
| `require_literal` 19.905 `:498`, `:628`, `:1139` | `DP:694`, `:1610`, `:2612` | tok (unshown) |
| `class_reference` 19.906 `:392`, 19.907 `:404`, 19.913 `:407`, 19.908 `:418`; 19.908 `:415` | `DP:404`, `:439`, `:452`, `:469` through `:293`; `:461` | tok (unshown) |
| 20.924 `:833`, `:851`; 20.925 `:928`; 20.926 `:534`, `:644`; 20.935 `:991` | `DP:1947`; `:1004`; `:807`, `:1638`; `:1336` | tok (unshown) |
| 19.901 `:374`, 20.916 `:341`/`:344`, 20.920 `:1050`, 20.921 `:455`, 19.924 `:885`, 99.916 `:336`/`:350`, 99.944 `:1054`, every `check_directive` 99.93x/99.94x | `DP:340`, `:89`, `:2820`, `:323`, `:2227`, `:82`/`:142`, `:2829`, `checkDirective` `:154` | none |

## Rows added

All appended with `?` in class, expect, line and message and filled by
`bash rust/corpus/errors/derive.sh SCRATCH` (exit 0). A final full re-derive at HEAD is
byte-identical to the committed TSV (`cmp`, exit 0). One stage-2 candidate,
`::class c inherit a ,`, was accepted by rexxc (the comma is a continuation) and was replaced by
`::class c inherit a +`; another, `::class c subclass rexx:`, duplicated a row and was dropped when
`no_corpus_program_appears_twice` caught it.

| Program | Oracle | Why |
|---|---|---|
| `do i = 1 to 2\nend j\n` | 10.2 `("J")`, `("I")`, line 1 | queued probe |
| `call foo bar(\n` | 36.901 position 13 line 1 | brief witness |
| `do i=1 to 2; signal on syntax name s1; s1: end\n` | 47.2 `"S1"` | ampersand item |
| `nop\n\nif 1 = 1\n\nthen\n\n::routine r\n` | 14.3 "THEN on line 5" | the THEN-not-at-end branch |
| `::class c '&1'\n` | 25.901 `found "&1"` | Focus 1, markup |
| `::requires x 'a"b'\n` | 25.904 `found "a"b"` | Focus 1, quote |
| `do\n'\x80':\nnop\nend\n` | 47.2 `found "\x80"` | Focus 1, non-UTF-8 |
| `do label x\nend <250 a>\n` | 10.2 with the 250-byte name upcased | Focus 1, long symbol |
| `call off junk\n`, `signal off junk\n` | 25.2, 25.4 | pairs no row raised |
| `r = ` + 50,001 `(` + `1` | 11.1 | pair no row raised (Ruling 1) |
| `::class c inherit a +\n`, `::attribute a external nostring\n`, `::attribute a delegate "p"\n`, `::annotate routine\n`, `::annotate method\n`, `::annotate attribute\n`, `::annotate constant\n` | 19.908, 19.905, 20.926, 19.925 x4 | call sites the reach run found unreached |

The Focus 1 rows are compared as bytes: `message_mismatches` compares `Vec<u8>`.

## Every converted site reached

Measured with a throwaway build (restored from saved copies, `git diff --stat` identical before and
after) that put `#[track_caller]` on `ParseError::raise` and on every insert-building helper in
`block.rs` and `directive.rs`, recorded `Location::caller()` in a static set, and ran every corpus
row through a temporary test. Before the reach rows, the unreached call sites were
`directive.rs` `class.inherit.push(self.class_reference(..))`, the attribute's EXTERNAL and DELEGATE,
`annotated_name` for ROUTINE/METHOD/ATTRIBUTE/CONSTANT, and `lib.rs` 99.914. After them only 99.914
is unreached, which is INTERPRET-only (`INTERPRET_ONLY` and the interpret witnesses reach it).

## Mutation evidence

`docs/superpowers/records/2026-10-15-phase-9c/task-4-mutants/`: `mutants.py` (one mutant per
converted call site), `runmut.sh`, `mutants.jsonl`, and `mut-summary.txt`. Each mutant swaps the
message for another of the same arity, or replaces an insert with `MUT`, or shifts a line. Batches
never hold two mutants of one pair, and a mutant counts red when a row of its pair is listed by
`every_secondary_message_matches_the_oracle`; 99.914 counts red when
`the_interpret_only_errors_match_the_condition_objects_own_code` fails. Run:
`CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --profile mutation -p rexx-parse --test errors --no-fail-fast`,
exit 101 for every batch. Every mutant is red (`grep -c " RED " mut-summary.txt` and
`grep -c " GREEN " mut-summary.txt` print 102 and 0). The line numbers in `mutants.jsonl` are of
the tree before `cargo fmt`, which moved some by a line. Three runs were invalid and are not
counted: batch 0 failed to compile twice (`E18_1`, then `E99_908`, not imported where the mutant
put them), and the first line-insert mutant (`line_of(byte + 1)`) moved no line.

The `e1bfac290` sites: every override site was mutated to pass an empty insert in one release build,
and the witness's stdout diffed against the oracle's
(`task-4-probes/override-witness/mutated.diff`): every line of an overridden message differs and no
control line does.

The `16e89ac59` sites: the `e1bfac290` binary, which numbers an INTERPRET string line 1, differs from
the oracle on the witness on every line-insert line
(`task-4-probes/interpret-line-witness/e1bfac290.diff`).

## PENDING

204 entries at `d5ea1577b`
(`git show d5ea1577b:rust/crates/rexx-parse/tests/errors.rs | sed -n '/^const PENDING/,/^\];/p' | grep -c '^    r#'`).
After the conversion every row matched except the two label-colon rows, measured by emptying
`PENDING` and running `every_secondary_message_matches_the_oracle`. `PENDING` and
`every_pending_message_row_still_differs_from_the_oracle` are deleted. `message_mismatches` skips
rows `label_colon_deviation` matches, and
`the_label_colon_rule_covers_exactly_the_two_corpus_rows_it_was_written_for` now pins both rows by
program and by the message this crate renders, and asserts the oracle's recorded message is
`Incorrect expression detected at ":".` and differs from ours.

## The coverage test

`tests/errors.rs`: `messages_named_by_the_parser` walks `rexx-parse/src`, skips `tests.rs` files,
files declared by `#[cfg(test)] mod name;` (`expr/differential.rs`), inline `#[cfg(test)] mod x {`
blocks and `//` comments, and collects every `E{major}_{sub}`.
`every_message_the_parser_names_is_raised_by_a_corpus_row_or_recorded_as_unreached` requires each
pair to be raised by some translation row where this parser raises the same pair, or to be in
`UNREACHED_BY_FILE`, and not both, and every record to name a pair the parser still names.
`every_unreached_reason_still_holds`: `InterpretOnly` must be in `INTERPRET_ONLY` (the list
`the_interpret_only_errors_match_the_condition_objects_own_code` now reads); `NoProgram`'s
`oracle_site` must resolve under `interpreter/` to a line holding the pair's symbolic name, looked
up in `messages/RexxErrorCodes.h`, and every probe must not raise the pair.
`the_scan_finds_the_messages_it_was_written_against` checks the scan itself.

Red with one row removed: `call off junk` deleted from the TSV, then
`cargo test -p rexx-parse --test errors every_message_the_parser_names`: exit 101,
`25.2 (.../src/instruction.rs)` (`task-4-mutants/cov-red.txt`). Restored, `cmp` exit 0.
Red reasons: a 25.2 record citing `InstructionParser.cpp:1` fails with
`does not raise Error_Invalid_subkeyword_calloff` and the coverage test with `is raised by a corpus
row and also recorded` (`reason-red.txt`); with the right site (`:990`) and probe
`call off junk`, it fails with `(25, 2) is reached by "call off junk\n"` (`reason-red2.txt`).

### UNREACHED_BY_FILE

| Pair | Reason | Probes |
|---|---|---|
| 47.1, 99.908, 99.912, 99.914, 99.915, 99.923, 99.924 | `InterpretOnly` | |
| 27.903 | `parser/InstructionParser.cpp:1650` | `do with`, `do with index`, `do with item i`, `do with over x`, `do with x over y` |
| 37.1 | `parser/LanguageParser.cpp:3882` | `x = ,`, `x = 1 + ,`, `x = (1,)`, `x = (,1)`, `say f(1,,2)`, `x = a[,]` |
| 49.2 | `parser/InstructionParser.cpp:453` | `do i = 1 to 2 by 1 to 3`, `do i = 1 for 2 while 1 until 2`, `do i over x for 1 to 2`, `do with index i over x for 1 by 2`, `do forever while 1 until 2`, `loop 3 while 1 until 2` |
| 89.2 | `parser/InstructionParser.cpp:3421` | `parse arg .a`, `parse arg 5`, `parse var x .a`, `parse value 1 with .`, `parse arg a.b. c` |

(each loop probe followed by `end`).

## Step 4, Step 5

* The Focus 1 rows are above. `an_interpret_label_error_names_the_label` (Task 3's rename of the
  brief's `an_interpret_error_with_a_substitution_renders_the_condition_objects_errortext`) already
  pins `br#"INTERPRET data must not contain labels; found "X"."#`, a byte literal, so nothing changed.
* `phase-4-exclusions.txt`: the 2026-08-20 note is now at `:5249` (located by its title). Its bullet
  "The message text, the substitutions, the clause echo and the exit status are a deliberate
  deferral ..." is deleted.
* incorrectCharacters, from `rust/`:
  `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/source.file/incorrectCharacters REXX_SUITE_TABLE=<repo>/docs/superpowers/records/2026-10-15-phase-9c/task-4-suite/table.tsv CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite`,
  exit 0. `tests.tsv`: `TEST_INCORRECT_CHARACTERS` passed on the oracle and the crate, verdict green.
  Its line is removed from `suite-9d-set.txt`, and the header's removal note names it.

## Queued items closed

* `2026-10-10-translation-error-inserts.md`: `trace ?s` (Task 3) and `e103` (`do i = 1 to 2` /
  `end j`): oracle and release `rexx-run` identical stdout, stderr and rc 246, `("J")`, `("I")`,
  `on line 1` (`task-4-probes/results-final.txt`).
* The inserts half of `2026-10-07-interpret-syntax-traceback-line.md`: `interpret "x=1; y=(; z=2"`
  untrapped, identical stdout, stderr (`at "("`) and rc 221. The REPLY-inside-INTERPRET half was
  not probed.
* The insert parts of `2026-10-13-error-position-ampersand.md`: 35.1 at `"~"` (Task 2) and 47.2
  `"S1"`, identical, rc 209. The `instanceMethod` item is not 9c's.

Every probe is one run per side, none concurrent, each side from the same fresh directory path in
turn (`task-4-probes/run.sh`, `run2.sh`, `run3.sh`; outputs under the session scratchpad).

## Checks, from `rust/`, exit statuses unpiped, at `16e89ac59`

| Command | Exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-inventory -p rexx-parse -p rexx-exec --all-targets -- -D warnings` | 0 |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-parse -p rexx-inventory --no-fail-fast` | 0, 424 passed, 0 failed (summed over the binaries) |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --lib error::` | 0, 13 passed |
| `REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus` | 0, `1065 of 1065 matching`, 29 passed 1 ignored |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test ir_recorded` | 0, 28 passed |
| the suite run above (at `dbbcdafb0`) | 0 |

No `Loud` site changed, so the refusal tables were not run. INTERPRET witnesses: `interp_trap`,
`interp_tb` and `interp_line` (`task-4-probes/`) and the two new corpus programs, all identical to
the oracle on the final binary.

## Callgrind

Base built in a `git worktree add --detach` copy of `d5ea1577b` with every `.rs` under `rust/crates`
touched and `CARGO_TARGET_DIR` under the scratchpad (`Compiling rexx-parse` and `Compiling
rexx-exec` printed), copied to `/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9c-t4-d5ea1577b`;
worktree and target dir removed. Head built at `16e89ac59` after touching `rexx-parse/src/lib.rs`
(`Compiling rexx-parse` printed), copied to `rexx-run-9c-t4-16e89ac59`. From `rust/`:

`bash bench-programs/callgrind.sh -B 0.5 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9c-t4c -p 'startup interpret parse' pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run base=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9c-t4-d5ea1577b head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9c-t4-16e89ac59`,
exit 0, 3 rounds. `summary.tsv` copied to
`docs/superpowers/records/2026-10-15-phase-9c/task-4-callgrind-summary.tsv`.

| program | pin | base | head | base vs pin | head vs pin | **head vs base (Task 4)** | head vs base, libc subtracted | spreads (pin/base/head) |
|---|---|---|---|---|---|---|---|---|
| startup | 119,095,681 | 119,177,103 | 119,150,282 | +0.0684% | +0.0458% | **-0.0225%** | -0.0176% | 0.0002 / 0.0002 / 0.0002% |
| interpret | 4,207,371,545 | 4,215,653,343 | 4,192,592,440 | +0.1968% | -0.3513% | **-0.5470%** | +0.0197% | 0.0001 / 0.0000 / 0.0000% |
| parse | 1,654,598,893 | 1,652,804,693 | 1,652,779,512 | -0.1084% | -0.1100% | **-0.0015%** | -0.0006% | 0.0000 / 0.0000 / 0.0000% |

All three whole-process deltas are under +0.05%. The `interpret` whole-process delta is almost all
in libc (23.4M Ir fewer there, +0.35M outside it); the libc-subtracted delta is the one to read for
the parser's own cost. What moved libc was not investigated. Earlier heads, same command:
`dbbcdafb0` -0.0089% / +0.0033% / -0.0009% (`cg-9c-t4`), `e1bfac290` -0.0104% / +0.0070% /
-0.0007% (`cg-9c-t4b`).

## Concerns

1. `phase-4-exclusions.txt` still holds false statements this task's work makes false, outside the
   note the brief names: the KNOWN GAPS row "A SOURCE THAT DOES NOT PARSE RENDERS ITS MESSAGE'S
   PLACEHOLDERS LITERALLY" (`:2292`), whose removal the section says needs an amendment, and the
   paragraph "Also still unsubstituted: a parse error's sub-message renders its catalogue template
   with &1 in it" (`:2485`). The 2026-08-20 note's remaining bullet about `NOT_IMPLEMENTED_EXIT` and
   its "This crate exits NOT_IMPLEMENTED_EXIT" sentence are stale from an earlier phase, not R6.
   Left for the controller.
2. `docs/superpowers/plans/2026-07-28-phase-3-parser.md:16` ("Parse errors are deliberately not
   reproduced 1:1 ... error 36's byte-position substitution is not produced at all") is a historical
   plan, not edited.
3. `rexx-exec/src/error.rs`'s test changed (forced by deleting `ParseError::new`), outside the
   brief's files.
4. `interpret`'s whole-process callgrind figure moved -0.55% through libc, unexplained (above).

## Fix round 1

Review: `task-4-review.md`.

* **I1** (ruling (b)): the 49.2 `NoProgram` record keeps its `InstructionParser.cpp:453` site and
  drops `do i over x for 1 to 2` and `do with index i over x for 1 by 2`, which are Deviation 38's
  shapes. It gains probes aimed at the keyword switch: `then nop`, `else nop`, `otherwise nop`,
  `when 1 then nop`, `end`. Every remaining probe went through the oracle with
  `corpus/errors/derive.sh` (`rexxc -s`, one run each): `do i = 1 to 2 by 1 to 3` 27.902,
  `do i = 1 for 2 while 1 until 2` / `do forever while 1 until 2` / `loop 3 while 1 until 2` 27.1,
  `then nop` 8.1, `else nop` 8.2, `otherwise nop` 9.2, `when 1 then nop` 9.1, `end` 10.1. None is
  49.2.
* **M1, the finding.** No numbered Deviation exists. The label-colon divergence is ruled in the
  Phase 3 plan, `docs/superpowers/plans/2026-07-28-phase-3-parser.md`: the paragraph headed "A named
  deviation, of the same class as the accepted eager-scan one" (`:1169`), and the exit criterion's
  "Recorded exceptions" (`:2557`), with the reason in Task 3.6's report
  (`records/2026-07-28-phase-3-parser/task-3.6-report.md:117`). `grep -n "Deviation [0-9]"
  phase-4-exclusions.txt` finds none for 18.x against 35.1. `label_colon_deviation`'s doc and the
  test now cite the Phase 3 plan and say there is no number. Measured shape (corpus rows): oracle
  `if 1 = 1` / `then: nop` 35.1 line 2 `Incorrect expression detected at ":".`, crate 18.1 `IF
  instruction on line 1 requires matching THEN clause.`; oracle `select` / `when 1 = 1` /
  `then: nop` / `end` 35.1 line 3, crate 18.2 `WHEN instruction on line 2 ...`. Both reject. If a
  numbered Deviation is wanted, that is for Moritz.
* **M2**: the "One from each raising file" comment is deleted.
* **M3**: deleted from `phase-4-exclusions.txt`: the KNOWN GAPS row "A SOURCE THAT DOES NOT PARSE
  RENDERS ITS MESSAGE'S PLACEHOLDERS LITERALLY", the paragraph "A fragment that fails to PARSE ...
  without a rexx-parse signature change" (with "We emit the second line only"), the paragraph "Also
  still unsubstituted ...", and in the 2026-08-20 note "Every syntax error diverges at the rexx-run
  level.", "This crate exits NOT_IMPLEMENTED_EXIT and prints one line ...", "Both halves are
  already decided and recorded." and the NOT_IMPLEMENTED_EXIT bullet. The tests that read the file
  (`coverage`, `loud`, `owners`) pass.
* **M5**: `Block::line_insert` takes a byte. `error_line`, `then_expected` and `end_mismatch` all
  build their insert through it.

### Checks, from `rust/`, exit statuses unpiped

| Command | Exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-inventory -p rexx-parse -p rexx-exec --all-targets -- -D warnings` | 0 |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-parse -p rexx-inventory --no-fail-fast` | 0, 424 passed, 0 failed |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --lib error::` | 0, 13 passed |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --test coverage --test loud --no-fail-fast` | 0 (closed_phases 8, coverage 21, loud 9, owners 6, phase_ten_reasons 5, refusal_dispositions 3, refusal_sites 5, representation_refusals 3 passed) |
| `REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus` | 0, `1065 of 1065 matching` (`Compiling rexx-parse` printed) |

The `line_insert` change is on cold error paths only. Callgrind was not re-run.
