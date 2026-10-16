# Phase 9c Task 3 report: the instruction raise sites

Commit `7436ca94e` (base `93de0ca29`): code, the 59 witness rows in
`rust/corpus/errors/parse-errors.tsv`, PENDING shrunk, and the 47.1 test updated.

## What landed

* `Inst::error(msg, inserts)` replaces `error(code, sub)` (`instruction.rs:249`). It and
  `found(msg, token)`, which inserts `display_value` of a token (`None` the end of clause,
  `";"`), are `#[cold] #[inline(never)]`. Every token lookup for an insert happens inside a
  cold helper: `found`, `then_expected`, `unreachable_switch`, `bad_trace_setting`,
  `unmatched_loop_keyword`.
* `required_end(msg: Msg<1>)` names the token that follows (`requiredEndOfClause`,
  `LanguageParser.hpp:205`). `loop_conditional`'s 27.1 is `Msg<0>` and is raised directly.
* `need_variable_class` is a method naming the token (31.2, 31.3).
* `expr.rs`: `parse_expr`, `parse_logical`, `Parser::logical`, `comma_list` take
  `missing: Msg<0>`; `missing_expression` is cold and raises it. `parse_case_when_list` passes
  `E35_934`. `display_value` is `pub(crate)`. `directive.rs:719` passes `E35_936` (its only
  change).
* The message assignment (`instruction.rs:552`) no longer passes 35.1 as a missing sub: it
  parses an optional expression and raises 35.1 naming the `=` or assignment-operator token
  (`InstructionParser.cpp:237`, `:250`). Measured: `o~x +=` gives `found "+="` on both sides.
* `missing_then(which) -> Msg<1>`; `missing_then_sub` stays for `block.rs:660` (Task 4).
* `tests/errors.rs`: the 47.1 test is now `an_interpret_label_error_names_the_label` and
  pins `found "X"` (oracle measured: `code=47.1  message=INTERPRET data must not contain
  labels; found "X".`, rc 0, under a `signal on syntax` trap).

### A switch default the oracle reaches

`do i over x for 1 to 2` reaches `newDoOverLoop`'s default (`InstructionParser.cpp:1494`):
the `FOR` expression stops at `TO` and `BY` (`TERM_CONTROL`), and only `FOR`, `WHILE`,
`UNTIL` have arms. The oracle prints `unmatched DO OVER loop subkey 29`, the
`InstructionSubKeyword` value of `TO` (`Token.hpp:263`), and `DO WITH` prints
`unmatched DO WITH conditional 0`, its still-`SUBKEY_NONE` conditional type (`:1703`).
`unmatched_loop_keyword` (`loops.rs:383`) reproduces both, mapping `BY` to 2 and `TO` to 29.
The other `Error_Interpretation_switch` sites (keyword dispatch, controlled-loop subkey,
`loop_conditional(None)`) go through `unreachable_switch`, which inserts the token's display
rather than an enum value; no row reaches them.

## Site table

Oracle paths are under `interpreter/parser/`, `IP` = `InstructionParser.cpp`. "tok" is
`displayValue` of the named token (`errorToken`, `LanguageParser.cpp:4129`), "next" the token
`requiredEndOfClause` reads.

| Rust site | Oracle | Inserts |
|---|---|---|
| `instruction.rs:211` 18.1/18.2 (`then_expected`) | `LanguageParser.cpp:1358` (`errorLine`) | the IF/WHEN clause's line |
| `instruction.rs:472` 47.1 | `IP:158` | tok (label) |
| `:503` 35.1 `symbol ==` | `IP:186` | tok (`==`) |
| `:538` 35.1 message `==`, `:555` 35.1 message with no value | `IP:225`, `:237`, `:250` | tok (`==`, `=`, assignment operator) |
| `:589` 21.901, `:842` 21.909, `:875`/`:879` 21.911, `:916`/`:938` 21.906, `:1447`/`:1455` 21.905, `:1551` 21.903, `:1559` 21.904, `:1628` 21.907/21.908 (`block_name`), `:1666` 25.923 | `IP:2945`, `:2261`, `:3018`, `:3026`, `:4155`, `:4219`, `:4080`, `:4088`, `:1045`/`:4000`, `:1058`/`:4010`, `:2838`, `:3866` | next |
| `:743` 25.17 | `IP:3459` | tok |
| `:830` 49.2 keyword switch (`unreachable_switch`) | `IP:453` | `keyword`, tok (oracle: the keyword enum) |
| `:851`/`:854` 20.905 | `IP:2970` | tok (`;` at the end) |
| `:889` 25.11, `:894` 25.15 | `IP:3036`, `:3052` | tok |
| `:920` 24.1 (`bad_trace_setting`, `:288`) | `IP:4166`, `:4190`; `TraceSetting.cpp` `badOption` | first byte of the value that is not `?` |
| `:932`/`:935` 35.1 `TRACE -`/`+` | `IP:4209`, `:4214` | tok (`;` at the end) |
| `:941` 26.7 | `IP:4224` | the number token's `value()` |
| `:963`/`:968` 25.913, `:977` 25.912 | `IP:2596`, `:2617`, `:2653`, `:2660` | tok |
| `:1017`/`:1060` 25.916 | `IP:2452`, `:2561` | tok |
| `:1114` 35.1 `RAISE SYNTAX` with no value | `IP:3553` (`nextToken()` after the backed-up EOC) | `;` |
| `:1138` 25.906, `:1144`/`:1187` 25.907 | `IP:3595`, `:3608`, `:3721` | tok |
| `:1208` 25.905/25.929 | `IP:4292` | tok |
| `:1242`-`:1248` 20.931 | `IP:4344` | tok |
| `:1264`/`:1309` 46.902 | `IP:4366`, `:4449` | tok |
| `:1348` 99.948 | `IP:2396` | tok |
| `:1507` 25.1-25.4 | `IP:990`, `:3945` | tok (condition) |
| `:1539` 25.914/25.915 | `IP:1029`, `:1034`, `:3984`, `:3989` | tok |
| `:1607` 46.1 | `IP:4535` | tok |
| `:1641`/`:1660` 25.923 | `IP:3827`, `:3853` | tok |
| `:1677`/`:1679` 31.2/31.3 (`need_variable_class`) | `IP:2386`, `:2391`, `:4486`, `:4491` | tok |
| `loops.rs:73`, `:111` 35.1 | `IP:2063`, `:2110` | tok (`==`) |
| `loops.rs:230`, `:301`, `:361` 27.902 | `IP:1309`, `:1328`, `:1346`, `:1472`, `:1606`, `:1624`, `:1681` | tok (the repeated keyword) |
| `loops.rs:310` 20.929 | `LanguageParser.cpp:2318` (`requiredVariable`, `IP:1613`, `:1631`) | `INDEX` or `ITEM` |
| `loops.rs:394` 49.2 (`unmatched_loop_keyword`) | `IP:1494`, `:1703` | as above |
| `loops.rs:414` 27.901 | `IP:4639`, `:4646` | tok |
| `loops.rs:225`, `:413` 49.2 (`unreachable_switch`) | `IP:1372`; `Error_None` at `IP:1365` etc. | what, tok |
| `address.rs:87` 25.934, `:148`/`:151`/`:193` 25.933 | `IP:765`, `:841`, `:895` | tok |
| `address.rs:184` 35.935 | `IP:872`, `:888` | `STREAM` or `USING`, `ADDRESS` |
| `parse.rs:95` 25.12 | `IP:3238` | tok (the token that ended the option loop) |
| `parse.rs:161`/`:222` 38.1, `:247`/`:251` 38.2 | `IP:3317`, `:3431`, `:3345`, `:3362` | tok |
| every other site in these files | as cited in each site's comment | none (`Msg<0>`) |

### Arity the catalogue does not give

Four oracle sites pass an insert to a message whose text has no `&1`: 89.1 (`IP:4393`),
89.2 (`IP:3421`), 20.922 (`IP:1122`) and 27.1 (`requiredEndOfClause`, `IP:4616`, `:4631`).
Those raise `Msg<0>` with `[]` here. The rendered message is identical. What differs is the
`additional` array a trapped INTERPRET error carries (one item on the oracle, none here). Not
measured on the oracle; no ruling requested since the message is the task's observable.

### Sites no row reaches

Measured by a throwaway build that recorded `Location::caller()` in `ParseError::raise`
through `#[track_caller]` on `Inst::error`, `Inst::found` and `Parser::raise`, run over the
corpus (`tests/errors.rs`, `--nocapture`), then restored and `cmp`'d. Unreached after the new
rows: the INTERPRET-only 99.908, 99.912, 99.915, 99.923, 99.924 and 47.1 (`rexxc` cannot hold
them; `the_interpret_only_errors_match_the_condition_objects_own_code` and the 47.1 message
test cover them), 27.903 (`loops.rs:322`, unreachable through `create_loop`), 89.2
(`parse.rs:217`: `parse_variable_or_message_term` answers `None` only for a non-symbol and
this arm holds a symbol; `parse arg .a` gives 31.3, oracle and crate) and the three
`unreachable_switch` callers.

## Rows added (59)

Appended with `?` in class, expect, line and message, then
`corpus/errors/derive.sh SCRATCH CANDIDATES > NEW` over the whole file (exit 0, three rounds,
the last over all 1181 rows). `diff` of the original TSV against the result: 59 lines added,
0 removed. Rows added for a site the coverage run found unreached, then those at a reached site
that witness an insert: `a~b == 1`, `a~b =`, `a~b +=`, `numeric form value`, `trace -;`,
`trace -(1)`, `forward 'x'`, `forward class (1) class (2)`, `forward class`,
`forward message 'a' message 'b'`, `forward message`, `forward arguments (1) arguments (2)`,
`forward arguments`, `raise 'x'`, `raise user 'x'`, `raise halt 'x'`, `raise halt description
'a' description 'b'`, `raise halt description`, `raise halt additional 'a' additional 'b'`,
`raise halt additional`, `raise halt array 1`, `use arg >`, `use arg >'a'`, `use arg (a)`,
`use arg a = 1 b`, `use local 'a'`, `call ns:'x'`, `call ()`, `signal value`, `call on 'x'`,
`call on error name (x)`, `drop ('a')`, `parse var 'a'`, `parse arg a ()`, `parse arg a )`,
`do label == 1`, `do i == 1`, `do i =`, `do i = 1 to`, `do i = 1 by`, `do i = 1 for`,
`do i over`, `do with index i over`, `do with index 'a' over x`, `do i over x for`,
`address system with 'input'`, `address system with input 'x'`,
`address system with input stem`; witnesses: `trace ?s` (the queued probe, `found "S"`),
`raise 1`, `raise halt 1`, `use arg >1`, `use arg >a b`, `call on 1`, `nop 1`,
`signal on error name lab x`, `do i over x for 1 by 2` (`subkey 2`),
`do with index i over x for 1 to 2` (`conditional 0`), and the accepted row `trace -`
(a trailing `-` continues the clause; measured, `trace -` / `say "after"` is 21.906 naming
`after`, rc 235).

## PENDING

299 entries at `93de0ca29`, 204 at `7436ca94e`
(`git show <sha>:rust/crates/rexx-parse/tests/errors.rs | sed -n '/^const PENDING/,/^\];/p' | grep -c '^    r#'`).
The 95 removed are the entries `every_pending_message_row_still_differs_from_the_oracle`
listed as now matching (94) plus `do i over x for 1 to 2\nend` after the switch-default fix.
Every remaining entry was listed with PENDING emptied in a scratch copy (restored, `cmp`'d):
all are raised in `block.rs` (7.x, 10.x, 14.x, 47.2-47.4, the EOF 18.x through `block.rs:660`)
or `directive.rs` (19.92x, 20.919, 20.929, 21.913/914, 24.1 and 25.11 of `::options`, 25.90x,
25.92x, 26.5/26.6, 36.901 of `::constant`, 99.917, 99.925), plus the two label-colon
deviation rows (18.x here, 35.1 there).

## Mutation evidence

`scratchpad/9c-t3/mutants.py` holds one mutation per new row (a different same-arity message,
or the insert token replaced by `None`, or the inserted value changed), each assigned to batch
A or B; `mutate.py` applies a batch by exact line-anchored replacement and restores from
copies. `git diff --stat -- rust` was `cmp`'d against the pre-mutation baseline after each
batch (identical). Runs:
`CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --profile mutation -p rexx-parse --test errors --no-fail-fast`.

| Batch | Mutants | Exit | Result |
|---|---|---|---|
| A | 23 lines (24 rows) | 101 | `every_secondary_message_matches_the_oracle` and `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` FAILED; all 24 A rows red, no B row red, 4 older rows red |
| B | 34 lines (34 rows) | 101 | the same two FAILED; all 34 B rows red, no A row red, 11 older rows red |

The first B attempt aborted on a wrong line anchor before writing any file, ran unmutated
and was green (exit 0); it is not counted. The accepted row `trace -` has no raise site and
no mutant.

## Commands, exit statuses unpiped

From `rust/`, at the tree committed as `7436ca94e` (the gates ran before the commit on the same
tree; nothing changed between):

| Command | Exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-inventory -p rexx-parse -p rexx-exec --all-targets -- -D warnings` | 0 |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-parse -p rexx-inventory --no-fail-fast` | 0, 419 passed, 0 failed (`errors` 30) |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --lib error::` | 0, 13 passed |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --no-fail-fast` | 0 (8, 6, 5, 3, 5, 3 passed) |
| `REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus` | 0, `1062 of 1062 matching`, 29 passed 1 ignored |

No ooTest groups are named for this task, so none ran. End to end on the head binary against
the oracle, one run each from fresh directories, stdout, stderr and rc each identical: the
47.1 INTERPRET trap, `trace ?s` (rc 232, `found "S"`), `o = .object~new` / `o~x +=` (rc 221,
`found "+="`).

## Callgrind

`git diff dd253ad15 93de0ca29 -- rust/crates` is empty, so the base is
`rexx-run-9c-t2-dd253ad15`. Head: `cargo build --release -p rexx-exec --bin rexx-run` at
`7436ca94e` after touching `rexx-parse/src/lib.rs` (`Compiling rexx-parse` printed), copied to
`/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9c-t3-7436ca94e`.

From `rust/`:
`bash bench-programs/callgrind.sh -B 0.5 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9c-t3 -p 'startup interpret parse' pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run base=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9c-t2-dd253ad15 head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9c-t3-7436ca94e`,
exit 0, 3 rounds.

| program | pin | base | head | base vs pin | head vs pin | **head vs base (Task 3)** | spreads (pin/base/head) |
|---|---|---|---|---|---|---|---|
| startup | 119,095,681 | 119,166,974 | 119,178,156 | +0.0599% | +0.0693% | **+0.0094%** | 0.0000 / 0.0002 / 0.0000% |
| interpret | 4,207,371,545 | 4,215,836,904 | 4,215,771,825 | +0.2012% | +0.1997% | **-0.0015%** | 0.0000 / 0.0000 / 0.0000% |
| parse | 1,654,598,893 | 1,652,794,456 | 1,652,805,602 | -0.1091% | -0.1084% | **+0.0007%** | 0.0000 / 0.0000 / 0.0000% |

All three are under +0.05%. The Task 3 column is computed from the medians above; the
libc-subtracted columns are in `cg-9c-t3/summary.tsv`.

## Concerns

1. The four sites in "Arity the catalogue does not give": the oracle's `additional` array has
   one item where ours has none. The message is identical.
2. The unreachable `Error_Interpretation_switch` sites insert the token's display where the
   oracle inserts an enum value. No program reaches them.
