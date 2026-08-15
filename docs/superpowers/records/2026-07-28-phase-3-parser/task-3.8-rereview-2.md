# Task 3.8 re-review 2: the fix round dfe495d7..c85a43f1

Scope: the three fix commits (`fec2b92d`, `01dc673b`, `c85a43f1`) only.
HEAD at review time is `c85a43f1`, tree clean, so file:line citations below are the working tree.
Every measured claim in this report was re-measured against `build/bin/rexxc` or `build/bin/rexx` from the session scratchpad, or re-derived from the tree; nothing was taken from the implementer's report.

## Verdicts on the five findings

### I1: fabricated oracle text in a comment. Addressed

`grep -rn "corresponding IF" rust/` and `grep -rn "DO, LOOP or SELECT" rust/` both return nothing.
The corrected comments quote the oracle verbatim and were re-measured:

* `rust/crates/rexx-parse/src/error/tests.rs:30` quotes `Error 8.2:  ELSE has no corresponding THEN clause.`; `rexxc` on `nop` / `else nop` prints exactly that (rc 248).
* `rust/crates/rexx-parse/src/error/tests.rs:47` quotes `Error 10.1:  END has no corresponding DO, LOOP, or SELECT.`; re-measured, byte-identical (rc 246).
* The 25.901, 7.1 and 36.901 comment quotes in the same file were also re-measured, all byte-identical.

### I2: the G12 false survivor baked into the corpus header. Addressed

The corpus header now records the true history at `rust/corpus/errors/parse-errors.tsv:50-57`: the five `::class c` family rows were added "on the mistaken belief that 99.916's second raise site was equally uncovered", that `tests/program.rs` had pinned all five kinds since Task 3.7b, and that the mutation "was scored against a single test target, so a miss there read as a survivor".
Both supporting claims verify:

* `trailing_code_after_a_bodiless_directive_is_99_916_for_every_kind_that_never_has_a_body` (`rust/crates/rexx-parse/tests/program.rs:149`) covers all five kinds, and `git log -S` dates it to `5aeb3255`, the Task 3.7b commit.
* The mutation log now scores `G12` as caught by the gate test.

The harness fix itself is not verifiable from the tree, because the harness is temporary tooling; only its documented output is.

### I3: tests/program.rs's eight error programs absent from the corpus. Addressed

All eight are present, each with the expect and line the file's doc comments claim:
`18.1` line 3 for `say "main"\n::routine r\nif 1 = 1\n`, `37.2` line 2 for `::routine r\nsay )\n`, five `99.916` rows for the bodiless-directive kinds (line 2, and line 5 for the `::resource` case), and `99.938` line 2 for `::constant c\nsay "trailing"\n`.
Verified by grepping the TSV for the exact escaped program bytes; one hit each.

### I4: the eager-scan deviation had no test. Addressed

Three tests now exist in `rust/crates/rexx-parse/tests/errors.rs`:

* `the_eager_scan_deviation_still_deviates` (line 567) pins our 6.2 on line 3 against the oracle's 37.2 on line 1.
* `the_eager_scan_deviation_needs_both_errors_to_appear` (line 595) pins that neither half alone deviates.
* `no_corpus_row_holds_the_eager_scan_shape` (line 609) asserts the exclusion over the corpus, and mutation `J5` shows it live.

The oracle side was re-measured: `say )` / blank / `'unclosed` gives `Error 37 ... line 1` and `Error 37.2` (rc 219), exactly as the test comment records.
The deviation input stays outside the corpus, which is the correct placement the brief argued for.

### I5: the closeable INTERPRET residual. Addressed

`the_interpret_only_errors_match_the_condition_objects_own_code` (`tests/errors.rs:639`) gates all seven fragments, plus two rendering tests at lines 679 and 689 and the accept direction at line 702.
I re-ran all seven fragments through a `signal on syntax` trap under `build/bin/rexx`:
every `code`, `errortext`, `message`, `additional` and `position` value in the comments at `tests/errors.rs:645-655`, `683` and `693-695` is byte-identical to what the oracle answers, including `position=2` for all seven, `message=INTERPRET data must not contain EXPOSE.` for 99.908, and `errortext=Unexpected label.` / `additional=X` for 47.1.

## The breakage hunt

### The pair diff, rebuilt independently

I extracted every `(major, sub)` tuple from all fifteen test files of the crate with my own scan, and diffed against the gate's coverage (195 distinct pairs over the 567 translation rows, plus the seven `INTERPRET` pairs).
Five pairs came out uncovered, and all five are false positives of my regex, not assertions of a parser error:

* `(3, 1)` and `(4, 0)`: loop indices at `src/block/tests.rs:742`.
* `(3, 8)`: an AST leaf span at `src/ast/tests.rs:70`.
* `(18, 3)`: the negative-direction assert of the deviation rule at `tests/errors.rs:546`, which asserts the rule does NOT fire.
* `(32, 303)`: a count tuple at `src/directive/tests.rs:1488`.

No genuinely asserted pair escapes the gate.
The specific claims also verify: `20.917` has four assertions in `src/expr/tests.rs` and four corpus rows, `20.930` three and three, the three `logical_error` cases are in the corpus as `if <condition>` programs, and `20.923` has no assertion anywhere outside the corpus (only the two raise sites at `src/expr.rs:1109` and `:1112`), which is what the header claims.

### Numeric claims, re-derived

Every one checks:

* 1020 corpus rows: 567 translation + 9 install + 444 accepted, counted from the TSV.
* The nine install rows are exactly the nine programs of the report's raw oracle output, seven `98.903` and two `90.999`; two were independently re-measured (`::method 3 attribute ...` rc 158, `REGISTERED x y` rc 166) and reproduce.
* 195 distinct translation pairs, 202 with the interpret seven (disjoint, checked), 93 of the 202 substitute in their sub-message, 704 table rows, exactly one sub-0 placeholder row and it is `101.000`; all re-derived from the generated table.
* 152 translation rows past line 1, against the test's floor of 140; the full line distribution matches the report's table (415+152 = 567).
* `checked >= 565` is 567 minus the two label-colon rows, exact.
* 116 mutations re-summed from the log (16+3+26+13+12+11+12+8+5+10); "applied and caught" is not reproducible because the harness is not in the tree.
* 365 crate tests, 0 failures, re-run; clippy `--all-targets -- -D warnings` clean, re-run.
* 301 `samples/` files counted; both `.orx` bootstrap files re-measured at rc 0.
* A random sample of 14 rewritten TSV rows (8 translation, 4 accepted, 2 install) re-measured against `rexxc`: 0 mismatches, including a line-9 row.

### The class field

Count is exactly 9, the gate asserts it exactly (`tests/errors.rs:455`, `assert_eq!(checked, 9)` plus the shape test's `assert_eq!(install, 9)` at line 288), and the nine rows are the right nine.
The C++ citations in the header exist: `LibraryDirective::install` in `interpreter/instructions/LibraryDirective.cpp` and `PackageClass::processInstall` at `interpreter/classes/PackageClass.cpp:1227`.
The classification remains a recorded judgement, as the header says; nothing mechanical in the tree can re-derive it, which the header also says.

### What this round did break or leave

No Critical and no Important defect found.
Minors, each real but none load-bearing:

* **M1, a format guard was dropped in the decoder.** The old `cases()` asserted that a program field holds no raw tab (`fields.next().is_none()` after three splits, with the message "a program field must not contain a tab; \t is the escape"). The new `splitn(4, '\t')` at `tests/errors.rs:137` silently folds a raw tab into the program bytes instead of failing. The corpus currently has no such row (checked), and the decoded bytes would equal the file's content either way, but the format's own rule that `\t` is the only spelling of a tab is now unenforced.
* **M2, one test still stacks two measured expectations.** `the_eager_scan_deviation_needs_both_errors_to_appear` (`tests/errors.rs:595`) asserts `(37, 2)` for `say )` and then `(6, 2)` plus line 3 for the literal in one `#[test]`, the exact shape this round split everywhere else because "a shared #[test] hid a wrong expectation twice". Both expectations were re-measured and are correct, so nothing is hidden today.
* **M3, a report claim does not match the code.** The report says the label-colon direction tests are "one assertion each"; `the_label_colon_rule_fires_for_the_pair_it_was_written_for` holds two asserts and `..._does_not_fire_for_a_near_miss` four (`tests/errors.rs:535`, `:541`). The code is fine; these assert a pure local function, so the hiding cost is diagnostic only.
* **M4, an overstated comment.** `src/instruction/tests.rs:1490` says `tests/errors.rs` "re-measures every one through the condition object"; the test asserts against recorded measurements and measures nothing at run time. The measurements themselves are real (I reproduced all of them).
* **M5, the report's residual section is stale against the tree.** It says `message()` drops the sub-message "for 82 of 181 error pairs"; the tree's module doc and my derivation both say 93 of 202. Tree is consistent; report is internally inconsistent.
* **M6, near-duplicate corpus rows.** For example `if then nop` and `if then nop\n` both exist (the pre-existing case-table transcription and the new `logical_error` transcription). Byte-level dedup passes; this is redundancy, not a defect.

### Constraint checks

No `unsafe` anywhere in the diff.
Programs stay `Vec<u8>` end to end; `holds_placeholder` takes `&str` only over table text, which is genuinely UTF-8.
No em-dashes in the changed `.rs` files; the semicolons in comments are inside quoted oracle text.
The added README lines are one sentence per line and carry no em-dash.
The class exception is defined per row by the property that holds, not by a code prefix, and both the header and the module doc name the `98.9xx` trap explicitly.
