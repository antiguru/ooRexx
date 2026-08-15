# Task 3.8 report: errors with number, sub-number and line

Status: complete, including the review round.
Commits `868749b1`, `0e48dc4c`, `e0f0f3f1`, then `fec2b92d`, `01dc673b`, `c85a43f1` for the review's five Importants and six Minors. Base `f60c7c4b` on `plan/rust-rewrite`.

`365` crate tests, `549` workspace, `0` failures.
`cargo clippy --offline --all-targets -- -D warnings` exit 0.
`cargo fmt -p rexx-parse` exit 0.
Zero `allow(dead_code)`, ownerless or otherwise.
`116` mutations, all applied, all caught, none by a compile error.
All `1020` corpus rows re-measured against `rexxc` after the schema change: `0` class, expect or line mismatches.

## What was built

`rust/crates/rexx-parse/src/error.rs` completes `ParseError`: `message()`, `line(&ProgramSource)`, `Display` and `std::error::Error`.
`rust/crates/rexx-parse/src/error/tests.rs` holds sixteen unit tests over the rendering rule.
`rust/crates/rexx-parse/tests/errors.rs` is the phase's error gate, thirty-two tests.
`rust/corpus/errors/parse-errors.tsv` is the corpus it reads: 1020 programs with the answer `build/bin/rexxc` gave for each, and a per-row field saying what kind of answer it was.

The type itself stays in `token.rs`, where Task 3.3 put it with a documented reason.
Moving it would have meant rewriting that reason and re-pointing five modules' imports for no gain.
`error.rs` is its completion, and it is the only module in the crate that knows the message table exists.

## The `subs` decision: removed, not filled

`ParseError.subs` existed and no construction site ever set it.
I removed it.
The measurement is the argument, and it is in `error.rs`'s module doc so it does not live only here.

Of the **202** distinct `(major, sub)` pairs the crate's own tests reach -- 195 from the 567 translation-error corpus rows plus the seven `INTERPRET`-only ones -- **93** have a substitution in their sub-message.
They need three kinds of value:

* the offending token's text, about sixty of them, the `found "&1"` family;
* the line of the construct that is still open: 7.1, 7.2, 10.002, 10.003, 10.004, 10.007, 14.001-14.005, 14.901, 18.001, 18.002;
* a keyword's own spelling: 19.925, 20.929, 25.927, 35.935, 49.002.

Exactly one needs something out of scope outright: 36.901's `&1` is a byte offset within a line, which the brief says not to produce.

So filling is *possible*, and it does not need a new field either -- the values are strings, and the note in `token.rs` about needing the offending position as a second field was about carrying a position, not a rendered value.
What makes it the wrong call is the pairing of cost with verifiability.
`syntaxError` in the C++ is handed the offending token at each call site; this parser's roughly two hundred raise sites are not, so every one would have to name its own offender, and about half of them sit either side of a `next_real()` so "the offender" is genuinely ambiguous per site.
And this phase does not gate substitution values, so all two hundred would land unverified under a gate that cannot see them wrong.
A value that looks right and is wrong is worse than one that is absent.

What is owed and to whom is recorded: Phase 4 has to answer `condition('o')~additional` for a trapped syntax error, which is where the values become observable from Rexx rather than only from a message, and it will need a differential test per substitution.

### What `message()` does instead

The table holds two rows per error, a major and a sub, and the interpreter prints both:

```text
Error 7 running select.rex line 3:  WHEN or OTHERWISE expected.
Error 7.1:  SELECT on line 3 requires WHEN.
```

`message()` returns the **sub-message when it holds no `&N` placeholder, and the major's own text when it does**.
One rule, no truncation heuristics, and an `&1` can never reach a user.

The major's text is always complete.
Measured over the whole generated table: of its 704 rows, exactly one with sub 0 carries a placeholder, `101.000` (`Error &1 running &2, line &3:.`), and 101 is a runtime wrapper this parser cannot raise.
`error/tests.rs` asserts that as a property with 101 as the single named exemption, so a second such major fails rather than being absorbed.

**And the text is byte-exact against the oracle, which was measured rather than assumed.**
Over the 551 rejected programs in the first corpus build, `message()` is byte-identical to one of the two lines `rexxc` prints for that error in 551 of 551 cases: 196 times the sub-message, 355 times the major.
Zero neither.
The reviewer re-ran this over the committed corpus and got 558 of 558 with a 203/355 split, which reconciles exactly: all seven rows added after the first build take the sub-message branch.

The `INTERPRET` measurement confirms the same thing from the other side, through the condition object rather than through stderr: `condition('o')~errortext` is the major's text and `~message` is the sub-message with its substitutions filled.
For 99.908, whose sub-message takes no substitution, `message()` returns `INTERPRET data must not contain EXPOSE.` and that is `~message` byte for byte.
For 47.1, whose sub-message substitutes the label, `message()` returns `Unexpected label.` and that is `~errortext` byte for byte, while `~additional` is `X` and this phase carries nothing.
I asserted the wrong one of those two on the first attempt and the test caught it.
Method: the raw `rexxc` stderr was captured per program during corpus generation, and the check re-derived `message()` from `rexxmsg.xml` offline and compared against both captured lines.
This is *not* gated in the repo -- the scope decision de-scoped message text, and gating it would have meant checking in both oracle lines per row -- but it is the reason the rule above is not merely defensible.

## Corpus counts, both of them

| Set | Count |
|---|---|
| Soundness: distinct Program-kind programs the three named test files raise an error on | **492** |
| ... of which reach `err`/`err_at`/`err_byte` directly | 487 |
| ... plus programs reaching `parse` directly in the 18.1/18.2 line tests | 8 (3 overlap) |
| Accepted Program-kind programs from the same three files | **453** |
| Interpret-kind error programs from the same three files (excluded, see below) | 6 |
| Added from `tests/scanner.rs`'s scanner-error table | 19 |
| Added from `src/expr/tests.rs`'s expression-error table, as `r = <expr>` | 31 (1 deduped) |
| Added after mutation testing found unreached paths | 7 |
| Added from `tests/program.rs`, including the only two inputs that error inside a directive body | 8 |
| Added from `src/expr/tests.rs`'s assertions *outside* its case table, found by the pair diff below | 10 |
| **Corpus file total** | **1020 rows: 567 translation, 9 install, 444 accepted** |
| Completeness also over | **301** `samples/` + `CoreClasses.orx` + `StreamClasses.orx` |
| `INTERPRET`-only errors, measured through the condition object | 7 |

The 567/492 difference is the 75 rows added from the other test files, and the 9 install rows are separate because our own tests assert those programs *parse*.

### The brief's 385 is stale

The brief and criterion 4 both say 385.
The measured number is 492, and the gap is Task 3.7c: 95 of the 492 come from `src/block/tests.rs`, which did not exist when the criterion was written. 385 + 95 = 480, and the remaining dozen are rows 3.7c added to the other two files.
Nothing about the method changed; the number was simply taken before the last task landed.

### Every test file, checked as a diff rather than claimed as a list

Criterion 4 originally named three files parenthetically, and those three raise **none** of the scanner classes (6.x, 13.1, 15.x, 30.1, 99.943) and none of the expression classes (37.x, 36.90x, 35.901, 19.909).
That matters for a specific reason: the eager-scan deviation the criterion tells the gate to be able to see is a *scanner* deviation, so over the three files alone the gate is structurally blind to it.
The criterion now says every test file, and I first satisfied that by naming a longer list -- which is the same mistake one level up.

**The check is a diff now.** Every `(major, sub)` pair any test file asserts, against every pair the gate covers. It found two pairs asserted and gated nowhere:

* **20.917**, `Symbol expected after superclass colon (:)`, four assertions;
* **20.930**, `Simple variable or stem symbol expected after > or < prefix operator`, three assertions.

Both live in `src/expr/tests.rs` in `error(...)` calls *outside* that file's own case table -- and the case table is what I had transcribed.
Ten rows added, each measured: those seven plus the three `logical_error(...)` cases as the `if <condition>` programs they were measured as.
The diff is empty now, and the diff is the artefact worth keeping, because a list of files goes stale silently and a diff cannot.

For completeness, what the diff does **not** flag: `src/convert/tests.rs`'s two `is_err()` assertions are on `check_trace_setting`, a helper returning `Result<(), ()>` rather than a `ParseError`, and its program-level manifestation is 24.1, which the corpus holds eleven rows of.
`src/ast/tests.rs`, `src/clause/tests.rs`, `src/token/tests.rs`, `src/expr/differential.rs`, `tests/sourceline.rs` and `tests/tokens.rs` assert no error at all.

### Multi-error inputs

None to exclude, and that is now **asserted** rather than observed once.
`no_corpus_row_holds_the_eager_scan_shape` takes every row whose error is a scanner class (6.x, 13.x, 15.x, 30.x, 99.943), truncates the program at the failing clause's own byte -- which leaves every complete clause before it -- and re-parses the prefix.
A prefix that also fails means the program holds two errors and does not belong in the corpus.
Zero rows.
Mutation `J5` changes the truncation to the whole program and the check fails, so it is live rather than vacuous.

The deviation itself now has a test, which it did not before: `the_eager_scan_deviation_still_deviates` pins our 6.2 on line 3 against the oracle's measured 37.2 on line 1, and `the_eager_scan_deviation_needs_both_errors_to_appear` pins that neither half alone deviates -- `say )` alone is the oracle's own 37.2, and the unterminated literal alone is 6.2 on its own line.
It is deliberately **outside** the corpus, because a row for it would record our answer as the expected one.

## The disagreements, all eleven, with raw oracle output

Nine of them are the "not a translation error" exception, and **the brief says two**.
It is nine, in **two** error classes, not one: seven `98.903` and two `90.999`.
`90.999` is not a `98.9xx` code at all.

```
######## ::routine r external "LIBRARY x"
     1 *-* ::routine r external "LIBRARY x"
Error 98 running .../8605a95681705261.rex line 1:  Execution error.
Error 98.903:  Unable to load library "x".
rc=158
######## nop / ::routine r external "LIBRARY x"
     2 *-* ::routine r external "LIBRARY x"
Error 98 running .../1d31eea9714e912a.rex line 2:  Execution error.
Error 98.903:  Unable to load library "x".
rc=158
######## ::method m external "LIBRARY foo"
     1 *-* ::method m external "LIBRARY foo"
Error 98 running .../d9e919be2a7c64f7.rex line 1:  Execution error.
Error 98.903:  Unable to load library "foo".
rc=158
######## ::method m external "LIBRARY foo bar"
     1 *-* ::method m external "LIBRARY foo bar"
Error 98 running .../fe065410860a234e.rex line 1:  Execution error.
Error 98.903:  Unable to load library "foo".
rc=158
######## ::method m external "<TAB>LIBRARY<TAB>x"
     1 *-* ::method m external "	LIBRARY	x"
Error 98 running .../c8894b142ea5629b.rex line 1:  Execution error.
Error 98.903:  Unable to load library "x".
rc=158
######## ::method m external "  library   x  "
     1 *-* ::method m external "  library   x  "
Error 98 running .../cd8784b7236f71d7.rex line 1:  Execution error.
Error 98.903:  Unable to load library "x".
rc=158
######## ::method 3 attribute external "LIBRARY x"
     1 *-* ::method 3 attribute external "LIBRARY x"
Error 98 running .../e8cbe08948912df1.rex line 1:  Execution error.
Error 98.903:  Unable to load library "x".
rc=158
######## ::routine r external "REGISTERED x y"
     1 *-* ::routine r external "REGISTERED x y"
Error 90 running .../8172182aefd355f9.rex line 1:  External name not found.
Error 90.999:  Unable to find external routine "y".
rc=166
######## ::routine r external "registered x"
     1 *-* ::routine r external "registered x"
Error 90 running .../ec1cb7217c1b5912.rex line 1:  External name not found.
Error 90.999:  Unable to find external routine "R".
rc=166
```

The other two are the recorded label-colon deviation, exactly two as recorded:

```
######## if 1 = 1 / then: nop            (ours 18.1)
     2 *-* then: nop
Error 35 running .../bb499a315170f639.rex line 2:  Invalid expression.
Error 35.1:  Incorrect expression detected at ":".
rc=221
######## select / when 1 = 1 / then: nop / end   (ours 18.2)
     3 *-* then: nop
Error 35 running .../6520ae3c4f725d59.rex line 3:  Invalid expression.
Error 35.1:  Incorrect expression detected at ":".
rc=221
```

**The install-time exception is no longer a rule at all, and that was a review finding worth taking on principle.**
I had encoded it as `major 98 | 90`, which is the code-prefix shape criterion 4 explicitly warns against -- and the warning exists precisely because a prefix missed a whole class.
It is now a per-row `class` field in the corpus, decided once with its evidence written in the header, and the gate reads it.
A prefix rule can miss a class; a field written per row cannot.

Two cheap discriminators were tried first and neither works, which is why the classification is a judgement recorded once rather than a computation:

```
$ build/bin/rexxc m2load.rex m2load.rxo  ; echo rc=$?   # ::routine r external "LIBRARY x"
rc=158   output file: No such file or directory
$ build/bin/rexxc m2trans.rex m2trans.rxo; echo rc=$?   # ::class c junk
rc=231   output file: No such file or directory
```

`rexxc` produces no output file for either, so "did it compile" does not separate them.

```
$ cat d_load.rex
say "TRANSLATED"
::routine r external "LIBRARY x"
$ build/bin/rexx d_load.rex 2>&1; echo rc=$?
     2 *-* ::routine r external "LIBRARY x"
Error 98 running .../d_load.rex line 2:  Execution error.
Error 98.903:  Unable to load library "x".
rc=158
```

Neither prints `TRANSLATED`, so "did anything run" does not separate them either: the library load happens during package install, and install precedes the leading code section.

What settles it is the C++ tree, checked rather than reasoned: `LibraryDirective::install` (`interpreter/instructions/LibraryDirective.cpp:122`) is reached from `PackageClass::processInstall`, which `PackageClass::install` (`interpreter/classes/PackageClass.cpp:1206`) drives by *calling* a dummy stub routine -- so it runs after translation is complete, by construction.

The label-colon deviation stays a rule, because it is a genuine shape rather than a list, and it keeps its exact count: `the_label_colon_rule_covers_exactly_the_two_corpus_rows_it_was_written_for` asserts 2.
Its both-directions tests are now one assertion each rather than several in one `#[test]`: the rule must fire for `(18.1, 35.1)` and `(18.2, 35.1)`, and must **not** fire for `(18.1, 18.1)`, `(18.1, 35.901)`, `(18.3, 35.1)` or `(None, 35.1)`.

## The line direction

**Zero** disagreements, re-confirmed over all 1020 committed rows.
`line_of(byte)` matches the line the oracle's main message names for every one of the agreeing error inputs, across all three of the conventions the phase measured.

The check is discriminating rather than accidentally satisfied, and that is its own test now: `the_line_check_sees_more_than_one_line_convention` asserts a floor of 140 rows reported past line 1, so a corpus that collapsed to one-line programs fails rather than passing vacuously.

The distribution over the committed corpus's 567 translation rows, counted with `awk` rather than estimated:

```
line 1: 415   line 2: 43   line 3: 32   line 4: 18   line 5: 34
line 6: 1     line 7: 18   line 9: 5    line 11: 1
```

415 + 152 = 567, and nine distinct reported lines.

**My earlier report said 123 and that was a third slice of the data, not a correction of the other two.**
123 was rows where the number *also* agreed, at the first corpus build, before 75 rows were added.
The reviewer's 132 and 142 are two more slices.
The number the test asserts against is 152, which is "translation rows whose recorded line is greater than 1" over the committed file, and it is the only one of the four that names a set precisely.

## Probes, raw

### The two lines and the rc, four programs

```
$ cat a.rex
nop

select

end
$ build/bin/rexxc a.rex 2>&1 1>/dev/null; echo "rc=$?"
     3 *-* select
Error 7 running .../a.rex line 3:  WHEN or OTHERWISE expected.
Error 7.1:  SELECT on line 3 requires WHEN.
rc=249

$ cat b.rex
nop

select

nop

end
$ build/bin/rexxc b.rex 2>&1 1>/dev/null; echo "rc=$?"
     5 *-* nop
Error 7 running .../b.rex line 5:  WHEN or OTHERWISE expected.
Error 7.2:  SELECT on line 3 requires WHEN, OTHERWISE, or END.
rc=249

$ cat c.rex
x = )
$ build/bin/rexxc c.rex 2>&1 1>/dev/null; echo "rc=$?"
     1 *-* x = )
Error 37 running .../c.rex line 1:  Unexpected ",", ")", or "]".
Error 37.2:  Unmatched ")" in expression.
rc=219

$ cat d.rex
say 1
bäc = 2
$ build/bin/rexxc d.rex 2>&1 1>/dev/null; echo "rc=$?"
     2 *-* bä
Error 13 running .../d.rex line 2:  Invalid character in program.
Error 13.1:  Incorrect character in program "ä" ('C3A4'X).
rc=243
```

7.1 reports the `SELECT`'s line and 7.2 the offending clause's, in adjacent programs -- the calibration the brief warned about, confirmed rather than taken on trust.

### The exact message texts the unit tests assert

Two of these corrected assertions I had **reasoned** to and got wrong; the tests caught them on the first run, which is the only reason they are right here.

```
$ cat e82.rex
nop

else nop
$ build/bin/rexxc e82.rex 2>&1 1>/dev/null; echo "rc=$?"
     3 *-* else nop
Error 8 running .../e82.rex line 3:  Unexpected THEN or ELSE.
Error 8.2:  ELSE has no corresponding THEN clause.
rc=248
```

I had written `ELSE has no corresponding IF.`

```
$ cat e101.rex
nop

end
$ build/bin/rexxc e101.rex 2>&1 1>/dev/null; echo "rc=$?"
     3 *-* end
Error 10 running .../e101.rex line 3:  Unexpected or unmatched END.
Error 10.1:  END has no corresponding DO, LOOP, or SELECT.
rc=246
```

I had written `DO, LOOP or SELECT.`, without the comma.
That assertion sat *after* the failing one in the same test, so it never ran on the first pass and would have been a second round trip.

```
$ cat e25.rex
::class c junk
$ build/bin/rexxc e25.rex 2>&1 1>/dev/null; echo "rc=$?"
     1 *-* ::class c junk
Error 25 running .../e25.rex line 1:  Invalid subkeyword found.
Error 25.901:  Unknown keyword on ::CLASS directive; found "JUNK".
rc=231

$ cat e36.rex
r = (a
$ build/bin/rexxc e36.rex 2>&1 1>/dev/null; echo "rc=$?"
     1 *-* r = (a
Error 36 running .../e36.rex line 1:  Unmatched "(" or "[" in expression.
Error 36.901:  Left parenthesis "(" in position 5 on line 1 requires a corresponding right parenthesis ")".
rc=220
```

36.901 is the one substitution the brief says not to produce, and this is it: a byte offset (`position 5`) within a line.
The main message is complete without it, which is what `message()` returns.

### The five bodiless-directive trailing-clause cases, found by mutation `G12`

```
$ printf '::class c\nnop\n' > 1.rex; build/bin/rexxc 1.rex 2>&1 1>/dev/null; echo "rc=$?"
     2 *-* nop
Error 99 running .../1.rex line 2:  Translation error.
Error 99.916:  Unrecognized directive instruction.
rc=157

$ printf '::options digits 9\nnop\n' > 2.rex; build/bin/rexxc 2.rex 2>&1 1>/dev/null; echo "rc=$?"
     2 *-* nop
Error 99 running .../2.rex line 2:  Translation error.
Error 99.916:  Unrecognized directive instruction.
rc=157

$ printf '::requires "nosuch"\nnop\n' > 3.rex; build/bin/rexxc 3.rex 2>&1 1>/dev/null; echo "rc=$?"
     2 *-* nop
Error 99 running .../3.rex line 2:  Translation error.
Error 99.916:  Unrecognized directive instruction.
rc=157

$ printf '::annotate package k 1\nnop\n' > 4.rex; build/bin/rexxc 4.rex 2>&1 1>/dev/null; echo "rc=$?"
     2 *-* nop
Error 99 running .../4.rex line 2:  Translation error.
Error 99.916:  Unrecognized directive instruction.
rc=157

$ printf '::resource d\nbody\n::END\nnop\n' > 5.rex; build/bin/rexxc 5.rex 2>&1 1>/dev/null; echo "rc=$?"
     4 *-* nop
Error 99 running .../5.rex line 4:  Translation error.
Error 99.916:  Unrecognized directive instruction.
rc=157
```

`lib.rs:250` already claimed this was "measured for all five kinds that can never have a body".
It was, during Task 3.7b, and nothing pinned it: `directive/tests.rs` asserts 99.916 only for `::junk` and `:: 5`, both of which reach the *other* raise site.
The `::resource` case reports line 4, so it exercises the line direction too.

### The namespace-qualified-symbol cases, found by mutation `E12`

```
$ printf 'r = a:\n' > 1.rex; build/bin/rexxc 1.rex 2>&1 1>/dev/null; echo "rc=$?"
     1 *-* r = a:
Error 20 running .../1.rex line 1:  Symbol expected.
Error 20.923:  Symbol expected as a name of namespace-qualified symbol.
rc=236

$ printf 'r = (a:)\n' > 4.rex; build/bin/rexxc 4.rex 2>&1 1>/dev/null; echo "rc=$?"
     1 *-* r = (a:)
Error 20 running .../4.rex line 1:  Symbol expected.
Error 20.923:  Symbol expected as a name of namespace-qualified symbol.
rc=236
```

`r = a:` reaches the end-of-clause arm and `r = (a:)` the not-a-symbol arm.
The corpus had **no** 20.923 row at all before this.
Three more spellings were probed and all answer 20.923: `say a:`, `r = a: ` with a trailing blank, and `r = a:;say 1`.

### Both directions of the file-level completeness check

```
$ find samples -name '*.rex' | while IFS= read -r f; do
    build/bin/rexxc "$f" >/dev/null 2>&1 || echo "FAIL $f"; done
checked=301 failures=0
$ build/bin/rexxc interpreter/RexxClasses/CoreClasses.orx   >/dev/null 2>&1; echo rc=$?
rc=0
$ build/bin/rexxc interpreter/RexxClasses/StreamClasses.orx >/dev/null 2>&1; echo rc=$?
rc=0
```

Rust side, `cargo test -p rexx-parse --test errors -- --nocapture`: `files: 301`, `failures: 0`, and both bootstrap files parse.

## Method for the corpus, and how the tree was kept clean

The `ok`/`err` helpers of the three named files were instrumented to dump every program they parse, keyed by an FNV-1a hash of the text so duplicates collapse, with one record file per program rather than an appended manifest -- the first attempt appended with `writeln!` and the parallel test threads interleaved their writes, corrupting 100 of 951 records.
`cargo test -p rexx-parse --lib` collected them.
Then the three files were restored from copies taken before any edit, the one-line `pub(crate) mod tests;` change in `instruction.rs` was reverted, and `git status --short` printed nothing.

`rexxc` was then run on each program, with stdout and stderr captured separately and the whole message retained per program, not a grepped field.
That retention is what made the byte-exactness check above possible after the fact.

## What went wrong, honestly

1. **I asserted two oracle message texts I had reasoned to.** Both were wrong, both were caught by the tests on the first run. `ELSE has no corresponding IF.` and `DO, LOOP or SELECT.` are what I wrote; the oracle says `ELSE has no corresponding THEN clause.` and `DO, LOOP, or SELECT.`. This is the exact failure mode the brief warns about, and the only thing that saved it is that I wrote the assertions before running them rather than after.

2. **The second wrong assertion hid behind the first.** Both were in one test and the 10.1 one sat after the 8.2 one, so it never ran until the first was fixed. Two assertions in one `#[test]` cost a round trip. A test per property would not have.

3. **My first whole-table placeholder test panicked on 101.018.** I had written the module doc claiming the property held for majors this parser raises, then wrote the test over the whole table. The panic was correct and the fix -- one named exemption plus a test asserting it is the only one -- is better than what I had intended.

4. **Five of 106 mutations survived the first pass, and none was an equivalent mutant.** Two were unreached error paths, two were assertions that could not fail, and one was targeted at the wrong test binary. All five are described in commit `e0f0f3f1`. The `A14` survivor is the instructive one: `line_of(self.byte + 1)` gave the right answer for every byte I had tested, because every one of them sat well inside its line.

5. **I guessed the corpus row counts for the test floors twice** instead of counting, and the second guess (563 rejections) failed the test. The real number is 558. Both times a two-second `awk` would have answered it.

6. **One of those five survivors was my harness lying to me, and I acted on it.**
`G12` read as a survivor because it was scored against one test target while `tests/program.rs` covered it.
I responded by adding five corpus rows for a path that had been pinned since Task 3.7b, then wrote a corpus-header note stating as fact that no row reached it.
A false survivor is worse than a missed mutant, because it produces confident work in the wrong direction.
Both halves are fixed, in the harness and in the header.

7. **The confession did not reach the code.**
I found two fabricated oracle message texts, reported them in this file, corrected the *assertions*, and left the fabricated text sitting in the comment above one of them.
That is where the next reader would have believed it.
Reporting a defect is not fixing it.

8. **I asserted a message text by reasoning for a third time**, in the new `INTERPRET` test, and got the *branch* wrong rather than the text.
99.908's sub-message takes no substitution, so `message()` returns the sub-message and not the major I had written.
Caught on the first run.
The lesson is narrower than "measure the text": measure which branch applies, because both branches produce real oracle strings and only one of them is the answer.

9. **I satisfied "every test file" by naming a longer list of files.**
That is the same error the criterion had just been rewritten to remove, one level up, and it hid two ungated error pairs.
A diff over the pairs is what actually checks it.

## Residual concerns

* **`message()` drops the sub-message for 82 of 181 error pairs.** That is the scope decision working as designed, but it is a *visible* difference: a Rexx program trapping a syntax error and reading `ERRORTEXT` or `MESSAGE` sees the generic sentence, and one reading `ADDITIONAL` sees nothing at all rather than an empty array. Phase 4 will have to build this, and it will have to do it at roughly two hundred raise sites.

* **The corpus is generated by a method that is documented but not scripted.**
The instrumentation is temporary by design, so regenerating the corpus means redoing it from the header's description.
`corpus/expr/precedence.tsv` has the same property.
It is fine while the corpus only grows by hand-measured rows, and it would not be fine if the corpus ever needed a wholesale rebuild.
Re-*verification* is scripted and cheap, though: eight seconds for all 1020 rows.

* **The `class` field is a human judgement and nothing in the tree can check it.**
Two mechanical discriminators were tried and neither separates a translation error from an install failure, as the raw probes above show, so the field records an argument rather than a measurement.
The exact count of 9 is what stops it drifting.
A tenth row cannot be classified `install` without that assertion failing and the argument being made again.

* ~~**Interpret-kind errors are outside the gate entirely.**~~ **Closed by the review round.**
All seven -- 47.1, 99.908, 99.912, 99.914, 99.915, 99.923, 99.924 -- are measured through the condition object and gated by `the_interpret_only_errors_match_the_condition_objects_own_code`, with the raw output above.
They stay out of the corpus *file* because `condition('o')~position` is the `INTERPRET` instruction's own line, 2 for every one of them, and not a position inside the fragment, so there is no oracle line for a fragment to record.
The number and sub-number are what the criterion gates and both are differential now.

* **`for_each_variable_name`'s forward hazard from 3.7c is untouched** and still stands: a new *field* on an existing instruction variant silently escapes the 99.913 check. Nothing here changes that, and this gate would not catch it either, because the corpus records only what the oracle says about programs that already exist.

## Mutation log

116 mutations. All applied -- the harness refuses a pattern that does not occur exactly once and reports it as `NOT APPLIED` rather than skipping it -- all caught, and none by a compile error, which the harness checks for by scanning the output for `error[E` and `could not compile`.

**The harness had a defect of its own, and it cost a review round.** A mutation was scored against one test target, so a mutation covered by a *different* target read as a survivor: `G12` is covered by `tests/program.rs` and was scored against `--test errors`, and I acted on the false survivor by adding five corpus rows for a path that had been pinned since Task 3.7b. Two fixes. Anything that survives its own narrow target is now re-run against the whole crate, and only a clean whole-crate run counts as a survivor. And the whole-crate run passes `--no-fail-fast`, because cargo otherwise stops at the first failing binary and the *attribution* is incomplete even when the verdict is right -- which is exactly what made `J10` look as though the new `INTERPRET` gate test did not cover it. Verified by hand afterwards: applying `J10` and running `--test errors` alone fails two of the new tests.

That is the fourth mutation-harness defect in this phase, after the first-`test result:` line bug, the stale patterns, and the equivalent mutants.

Targets: `--test errors` unless noted, `--lib error::` for group A, and the whole crate for `F2` and `J10`.

### Group A -- `error.rs`: the rendering rule, `line`, `Display`

| Mutation | Caught by |
|---|---|
| A1 message drops the placeholder filter | `error::tests::display_does_not_zero_pad_a_three_digit_sub_number` (+4 more) |
| A2 message inverts the placeholder filter | `error::tests::a_second_sub_message_that_needs_no_substitution_is_also_the_message` (+7 more) |
| A3 message looks the sub-message up as a major | `error::tests::a_second_sub_message_that_needs_no_substitution_is_also_the_message` (+2 more) |
| A4 message falls back to the sub-message row | `error::tests::a_sub_message_that_would_substitute_a_token_falls_back_to_the_majors_text` (+4 more) |
| A5 row answers the symbolic name instead of the text | `error::tests::a_second_sub_message_that_needs_no_substitution_is_also_the_message` (+6 more) |
| A6 row swaps major and sub in the lookup | `error::tests::display_does_not_zero_pad_a_three_digit_sub_number` (+7 more) |
| A7 row looks up the next sub-number | `error::tests::display_does_not_zero_pad_a_three_digit_sub_number` (+6 more) |
| A8 has_placeholder answers false on a hit | `error::tests::a_sub_message_that_would_substitute_a_line_falls_back_to_the_majors_text` (+6 more) |
| A9 has_placeholder inverts the digit test | `error::tests::a_sub_message_that_would_substitute_a_line_falls_back_to_the_majors_text` (+6 more) |
| A10 has_placeholder stops after the first ampersand | `error::tests::display_does_not_zero_pad_a_three_digit_sub_number` (+6 more) |
| A11 has_placeholder answers true when there is none | `error::tests::a_second_sub_message_that_needs_no_substitution_is_also_the_message` (+5 more) |
| A12 has_placeholder scans for the wrong byte | `error::tests::a_sub_message_that_would_substitute_a_token_falls_back_to_the_majors_text` (+6 more) |
| A13 line ignores the byte | `error::tests::a_byte_past_the_end_clamps_to_the_last_line` (+2 more) |
| A14 line reports one byte later | `error::tests::a_byte_on_a_terminator_belongs_to_the_line_that_terminator_ends` |
| A15 Display swaps major and sub | `error::tests::display_names_the_number_the_sub_number_and_the_message` (+1 more) |
| A16 Display prints the major instead of the message | `error::tests::display_does_not_zero_pad_a_three_digit_sub_number` (+1 more) |

### Group B -- `token.rs`: the constructor

| Mutation | Caught by |
|---|---|
| B1 new discards the byte | `the_eager_scan_deviation_needs_both_errors_to_appear` (+2 more) |
| B2 new swaps major and sub | `a_then_label_is_a_missing_then_here_and_a_bad_expression_there` (+10 more) |
| B3 new discards the sub-number | `a_when_label_is_a_missing_then_here_and_a_bad_expression_there` (+7 more) |

### Group C -- `block.rs`: the thirteen block errors

| Mutation | Caught by |
|---|---|
| C1 unclosed DO becomes the loop number | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C2 unclosed SELECT becomes the DO number | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C3 dangling THEN becomes the ELSE number | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C4 dangling ELSE becomes the THEN number | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C5 unclosed loop becomes the DO number | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C6 unclosed OTHERWISE takes its table position | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C7 the unclosed-block byte is discarded | `the_reported_line_matches_the_oracles_main_message` |
| C8 the unclosed-block major becomes 18 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C9 a label in a THEN takes the SELECT number | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C10 a label in a SELECT takes the THEN number | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C11 a label in a DO takes the SELECT number | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C12 the label byte is discarded | `the_reported_line_matches_the_oracles_main_message` |
| C13 a named END mismatch takes the no-name number | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C14 a named SELECT END mismatch takes the no-label number | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C15 an unlabelled END mismatch takes the labelled number | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C16 an unlabelled SELECT END mismatch takes the labelled number | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C17 the END-mismatch byte is discarded | `the_reported_line_matches_the_oracles_main_message` |
| C18 a WHEN-less SELECT takes 7.2 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C19 a WHEN-less SELECT reports against the END | `the_reported_line_matches_the_oracles_main_message` |
| C20 an unmatched END takes 10.2 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C21 a missing THEN takes major 17 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C22 a missing THEN reports against the IF, not the terminating directive | `the_reported_line_matches_the_oracles_main_message` |
| C23 a non-WHEN inside a SELECT takes 9.2 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C24 a stray ELSE takes 8.1 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C25 a label before an ELSE takes 47.4 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| C26 a stray OTHERWISE takes 9.1 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |

### Group D -- `scanner.rs`

| Mutation | Caught by |
|---|---|
| D1 an unclosed comment takes 6.2 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| D2 the unclosed-comment byte is discarded | `the_reported_line_matches_the_oracles_main_message` |
| D3 a bad character takes 13.2 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| D4 an over-long symbol takes 30.2 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| D5 an unclosed literal takes major 7 | `the_eager_scan_deviation_still_deviates` (+2 more) |
| D6 an unterminated resource takes 99.944 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| D7 the unterminated-resource byte is discarded | `the_reported_line_matches_the_oracles_main_message` |
| D8 a leading blank in a hex literal takes 15.2 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| D9 a bad hex digit takes 15.4 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| D10 a bad binary digit takes 15.3 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| D11 a leading blank in a binary literal takes 15.1 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| D12 a misplaced hex blank takes 15.6 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| D13 a misplaced binary blank takes 15.5 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |

### Group E -- `expr.rs`

| Mutation | Caught by |
|---|---|
| E1 a bad variable name takes major 32 | `every_error_the_corpus_raises_renders_a_non_empty_message` (+2 more) |
| E2 an expression error loses its clause byte | `the_reported_line_matches_the_oracles_main_message` |
| E3 the unmatched-opener sub-numbers are swapped | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| E4 a stray right parenthesis after a term takes 37.901 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| E5 a stray right bracket after a term takes 37.2 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| E6 a prefix operator with no operand takes 35.1 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| E7 a missing message name takes 19.908 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| E8 a stray right parenthesis where a term was expected takes 37.901 | `the_eager_scan_deviation_needs_both_errors_to_appear` (+1 more) |
| E9 a stray right bracket where a term was expected takes 37.2 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| E10 a missing expression takes major 34 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| E11 an empty parenthesised expression takes 35.901 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| E12 a qualified symbol with no token takes 20.924 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |

### Group F -- `instruction.rs`

| Mutation | Caught by |
|---|---|
| F1 an instruction error loses its clause byte | `the_reported_line_matches_the_oracles_main_message` |
| F2 a label inside INTERPRET takes 47.2 | `instruction::tests::a_label_in_interpret_text_is_47_1` (+2 more) |
| F3 a missing loop symbol takes 20.910 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| F4 a repeated DO keyword takes 27.901 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| F5 DO WITH with no OVER takes 27.903 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| F6 a bad TRACE letter takes 24.2 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| F7 a non-whole TRACE number takes 26.5 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| F8 a missing PARSE WITH takes 38.2 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| F9 an empty PARSE template takes 38.1 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| F10 a bad PARSE position takes 38.1 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| F11 an extra USE ARG token takes 46.1 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |

### Group G -- `directive.rs`

| Mutation | Caught by |
|---|---|
| G1 a directive error loses its clause byte | `the_reported_line_matches_the_oracles_main_message` |
| G2 a duplicate directive keyword takes 99.926 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| G3 an unknown ::CLASS keyword takes 25.902 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| G4 an unknown ::METHOD keyword takes 25.901 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| G5 an unknown ::ATTRIBUTE keyword takes 25.926 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| G6 a bad external specification takes 99.918 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| G7 an unmatched directive parenthesis takes 36.902 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| G8 a missing ::ANNOTATE value takes 19.909 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| G9 the trailing-clause byte is discarded | `the_reported_line_matches_the_oracles_main_message` |
| G10 a bare :: takes 20.917 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| G11 a non-symbol ::ANNOTATE name takes 20.925 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| G12 a clause that is not a directive takes 99.917 | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |

### Group H -- the corpus file itself

| Mutation | Caught by |
|---|---|
| H1 a recorded sub-number is changed | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| H2 a recorded line is changed | `the_reported_line_matches_the_oracles_main_message` |
| H3 a recorded major is changed | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| H4 a rejected program is recorded as accepted | `the_corpus_holds_at_least_the_rows_it_was_measured_with` (+2 more) |
| H5 an accepted program is recorded as rejected | `the_corpus_holds_at_least_the_rows_it_was_measured_with` (+2 more) |
| H6 a non-UTF-8 escape is mangled | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` (+1 more) |
| H7 a recorded line moves past the end of its program | `every_recorded_line_is_a_line_its_program_has` (+1 more) |
| H8 an expression case's sub-number is changed | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |

### Group I -- the gate's own decoding and checks

| Mutation | Caught by |
|---|---|
| I1 the newline escape decodes to a blank | `the_corpus_escaping_decodes_every_escape_the_format_defines` (+7 more) |
| I2 the tab escape decodes to a blank | `the_corpus_escaping_decodes_every_escape_the_format_defines` |
| I3 the backslash escape decodes to a slash | `a_doubled_backslash_does_not_swallow_the_byte_after_it` (+3 more) |
| I4 the hex escape is read in the wrong base | `the_corpus_escaping_decodes_every_escape_the_format_defines` (+13 more) |
| I5 the placeholder scan in the gate never fires | `the_gates_own_placeholder_check_finds_a_placeholder` (+1 more) |

### Group J -- the class column, the exclusion check, and the INTERPRET paths

| Mutation | Caught by |
|---|---|
| J1 an install-time row is read as a translation error | `a_rejection_that_is_not_a_translation_error_is_accepted` (+2 more) |
| J2 a translation row is read as install-time | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` (+4 more) |
| J3 an accepted row is read as a translation error | `every_recorded_line_is_a_line_its_program_has` (+11 more) |
| J4 soundness reads the install rows instead of the translation rows | `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` |
| J5 the eager-scan exclusion checks the whole program, not the prefix | `no_corpus_row_holds_the_eager_scan_shape` |
| J6 the corpus header row is not skipped | `every_error_the_corpus_raises_renders_a_non_empty_message` (+11 more) |
| J7 the program field is split on its first tab | `every_error_the_corpus_raises_renders_a_non_empty_message` (+11 more) |
| J8 a directive inside INTERPRET takes 99.915 | `the_interpret_only_errors_match_the_condition_objects_own_code` |
| J9 the INTERPRET directive check runs for a program instead | `the_interpret_only_errors_match_the_condition_objects_own_code` (+5 more) |
| J10 EXPOSE inside INTERPRET takes 99.909 | `instruction::tests::expose_is_rejected_inside_interpret` (+3 more) |
