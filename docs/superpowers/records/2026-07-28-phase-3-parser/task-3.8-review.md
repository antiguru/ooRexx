# Task 3.8 review: errors with number, sub-number and line

Reviewed at `689400e3`, diff `f60c7c4b..e0f0f3f1` plus the criterion amendment in `689400e3`.
Tree was clean at start and is clean at end; every mutation was applied from a `cp` backup and restored from it.

## The three verdicts

1. **Spec compliance: pass.**
   Both files the brief names exist, `subs` is settled rather than inherited, `message()` renders from the generated table, and the gate runs in both directions over a named corpus without needing a built C++ interpreter.
   The brief's two stale facts (385 corpus programs, two non-translation inputs) were measured and corrected rather than copied forward, and the criterion was amended on the findings.

2. **Code quality: good, with two documentation defects.**
   The gate's construction is the best in the phase so far: failures are collected and reported together rather than aborting on the first, deviations are rules with exact pinned counts rather than lists of programs, the placeholder check is implemented twice on purpose, and every floor I tested actually fires.
   Against that, two comments state measurements that do not hold, and one of them is the exact fabricated oracle text the report admits writing.

3. **Gate criterion 4: met in substance, not met as literally worded.**
   The full answer is the last section. Three clauses of the current text are not satisfied by what shipped, and all three are narrow.

## What I re-verified, and how

| Check | Method | Outcome |
|---|---|---|
| Corpus is only the oracle's answers | Re-ran `build/bin/rexxc` on **all 1002 rows**, comparing recorded `expect` and `line` | 444 rc 0 / 558 rejected, **0** `expect` mismatches, **0** `line` mismatches |
| `message()` matches one of the two oracle lines | Real `ParseError::message()` (scratch binary against the crate) vs whole captured stderr, all 558 rejected rows | **558 of 558**: 203 sub, 355 major, **0 neither** |
| The 196 + 355 = 551 arithmetic | Reconciled against the committed corpus | Consistent: the 7 added rows are all sub-branch, 196 + 7 = **203** |
| Line direction | Our `line()` vs the oracle's main-message line, all agreeing rows | **547 of 547 agree, 0 disagreements** |
| Line conventions individually | Whole `rexxc` stderr dumped for 14 rows: 7.1, 7.2, 10.1, 10.3, 10.7, 14.1, 14.901, 18.1, 18.2, 35.934, 47.4, 99.907, 99.913, 99.916 | All 14 agree; 7.1 is the SELECT's line, 7.2 the offender's, 10.x the END's, 18.x the terminating clause's, 99.916 the trailing clause's |
| The floor assertion is not decorative | Dropped every row with `line > 1` (145 rows) and ran the line test alone | Fails: `only 0 corpus errors are reported past line 1` |
| Mutations | Re-applied **6**: A9, A14, C18, E12, G12, H2 | All 6 caught, by the tests the log names |
| No catch is a compile error | `cargo check --offline --all-targets` under E12 | exit 0, no `error[E` |
| The two unreached paths | Both 99.916 raise sites and both 20.923 arms traced; the 7 rows measured against `rexxc` | Rows do reach both, but see Important 2 |
| The INTERPRET residual | Route (c) on all five classes with the full condition object printed | 99.908, 99.912, 99.915, 99.923, 99.924 all confirmed correct |
| The eager-scan deviation | `say )` / `nop` / `x = 'unclosed` through both | Oracle 37.2 line 1, ours 6.2 line 3, exactly as recorded |
| Oracle texts asserted in comments | `rexxc` on 8.2, 25.901, 7.1, 36.901 | 25.901, 7.1, 36.901 correct; **8.2's comment is wrong** |

The corpus verification is the important one and it is total, not sampled: every row of `parse-errors.tsv` reproduces from `build/bin/rexxc` today, including the line column.
**The corpus contains none of our own output.**
The one shape that would have made the gate self-confirming — recording our answer for a program the oracle answers differently — does not occur: rows 378 and 697 record the oracle's `35.1`, not our `18.1`/`18.2`.

## Critical

None.

## Important

### 1. A comment states oracle output the oracle does not produce

`rust/crates/rexx-parse/src/error/tests.rs:29`:

```rust
    //     Error 8.2:  ELSE has no corresponding IF.
```

Measured, on both a two-line and a three-line spelling:

```
Error 8 running .../e8.rex line 2:  Unexpected THEN or ELSE.
Error 8.2:  ELSE has no corresponding THEN clause.
rc=248
```

This is the fabricated text the report confesses to at "What went wrong, honestly" item 1.
The assertion three lines below it was corrected; the comment that says what `rexxc` printed was not.
Failure scenario: Phase 4 owes `condition('o')~message` for a trapped syntax error and will look for recorded oracle text.
This comment is recorded oracle text, it is wrong, and it sits in the one file whose header says every text in it "was printed by `build/bin/rexxc`, not read out of the XML".

### 2. The stated reason for the five 99.916 rows is false, and it is now baked into the corpus header

The report (line 332) and `parse-errors.tsv` (lines 17-22) both say the 99.916-on-a-non-directive path was claimed measured at `lib.rs:250` while "nothing pinned it: `directive/tests.rs` asserts 99.916 only for `::junk` and `:: 5`".

That is wrong. `rust/crates/rexx-parse/tests/program.rs:148` is

```rust
fn trailing_code_after_a_bodiless_directive_is_99_916_for_every_kind_that_never_has_a_body()
```

and it loops over exactly the five kinds, `::CLASS`, `::OPTIONS`, `::REQUIRES`, `::ANNOTATE`, `::RESOURCE`.
It predates this task and the diff does not touch it.
Re-applying mutation G12 against `--test program` alone:

```
---- trailing_code_after_a_bodiless_directive_is_99_916_for_every_kind_that_never_has_a_body stdout ----
test result: FAILED. 22 passed; 1 failed
```

So `lib.rs:250`'s comment was true and pinned all along, and G12 "survived" only because the harness scored it against `--test errors`.
The generalisation the report draws — that a comment asserted a measurement that did not exist — holds for 20.923 (I grepped; nothing outside `expr.rs` and the two new corpus rows mentions it) and not for 99.916.
Failure scenario: a per-target mutation harness reports false survivors, and the conclusions drawn from them get written into a permanent artefact. The five rows are still worth having, because they put the five kinds under the oracle differential instead of only under a hand-written number, but the header's stated reason should say that instead.

### 3. `tests/program.rs`'s error programs are absent from the corpus

Criterion 4 now says the corpus is the programs the crate's own tests assert an error on, "**across every test file and not a named subset**".
`tests/program.rs` asserts errors on eight Program-kind programs. None is in the corpus:

```
ABSENT  say "main"\n::routine r\nif 1 = 1\n            (18.1)
ABSENT  ::routine r\nsay )\n                          (37.2)
ABSENT  ::class c\nsay "trailing"\n                    (99.916, and four siblings)
ABSENT  ::constant c\nsay "trailing"\n                 (99.938)
```

By contrast the scanner table is 19 of 20 present (the absentee, `x = 1\ny = #`, is a duplicate of `say 1\nx @ y` in class and line convention) and the expression table is 31 of 31.
So the omission is one file, and it is the file that raises errors *inside a directive body* — a distinct path, since a body gets its own `translate_block` call.
Failure scenario: a regression that reports the wrong line for an error inside a `::ROUTINE` body passes the gate, because `tests/program.rs` pins the number and nothing pins the line there.

### 4. The eager-scan deviation has no test anywhere

Criterion 4 lists two recorded exceptions and says "each with a test pinning both directions".
The label-colon one has `a_then_or_when_label_is_a_missing_then_here_and_a_bad_expression_there` with four near-miss assertions, and its count is pinned at 2 by `each_deviation_covers_the_number_of_corpus_cases_it_was_written_for`.
The eager-scan one has nothing: no `Deviation` variant, no test, and by policy no corpus row. Grepping the crate for it finds only a module doc comment at `src/scanner.rs:34-39`.

The deviation is real and I reproduced it:

```
$ printf "say )\nnop\nx = 'unclosed\n" > m.rex
$ build/bin/rexxc m.rex 2>&1 1>/dev/null
     1 *-* say )
Error 37 running .../m.rex line 1:  Unexpected ",", ")", or "]".
Error 37.2:  Unmatched ")" in expression.
rc=219
ours: 6.2  line 3  Unmatched single quote (').
```

**The gate can still observe it**, which was the reviewer's question: no rule waives it, so a multi-error row added to the corpus fails the soundness test rather than being absorbed.
What is missing is the assertion that it *is* this shape and only this shape.
Failure scenario: the deviation widens — a new masking shape at a clause boundary the scanner now reaches earlier — and nothing in the crate notices, because multi-error inputs are excluded from the corpus by policy and no test names the deviation.

### 5. The INTERPRET residual needs recording, and it is closeable

Five classes are outside the gate with no oracle-differential coverage at all: 99.908, 99.912, 99.915, 99.923, 99.924.
They rest on the table in `src/instruction/tests.rs:1211-1215`.
I measured all five through route (c) and all five are correct:

```
######## interpret "reply 1"
CODE=99.924   MSG=Translation error.   ADD=INTERPRET data must not contain REPLY.
######## interpret "forward to 1"    CODE=99.923
######## interpret "guard on"        CODE=99.912
######## interpret "use local a"     CODE=99.915
######## interpret "expose a"        CODE=99.908
```

The report calls this "not a gap this task could close without route (c) machinery".
The machinery is five lines of Rexx and this crate already uses it twice — `tests/program.rs:257` and `:270` say so for 47.1 and 99.914.
So this is a gap that was declinable, not one that was unreachable.
Recommendation: either add an `interpret`-kind flag to the corpus for these six programs, or name the residual in criterion 4 so it is a recorded exclusion rather than an unstated one.
Failure scenario as it stands: a regression in any of the five is caught only by a hand-written number in a unit test, which is precisely the coverage class this phase built the gate to replace.

## Minor

1. **`error.rs:49-50`'s load-bearing measurement does not reproduce.**
   It says "Of the **181** distinct `(major, sub)` pairs the error corpus raises, **82** have a substitution".
   Measured over the corpus in the tree: **195** distinct pairs, **93** on the substituting branch.
   No population I tried gives 181/82 — 194/93 excluding the seven added rows, 193 using our own error pairs — so the number predates the scanner and expression tables.
   The argument is unaffected (93 of 195 is still about half) but the numbers are the recorded justification for removing a field and they should be the corpus's numbers.

2. **The lesson about assertion granularity was diagnosed and not applied.**
   Report item 2 says two assertions in one `#[test]` cost a round trip and "a test per property would not have".
   `a_sub_message_that_needs_no_substitution_is_the_message` still holds both, and `a_sub_message_that_would_substitute_falls_back_to_the_majors_text` holds three independently measured oracle texts.
   Demonstrated by breaking both assertions in the first test at once:

   ```
   panicked at crates/rexx-parse/src/error/tests.rs:31:5:
     left: "ELSE has no corresponding THEN clause."   right: "WRONG A"
   ```

   The second breakage is invisible. `display_names_...` (2 texts), `a_then_or_when_label_...` (18.1 then 18.2, different code paths) and `a_load_failure_...` (a loop that panics on the first program) hide the same way.
   The corpus-driven tests do **not**: they collect into `wrong` and report every failure. So the gate itself is unaffected and this is confined to the unit tests.

3. **`deviation()` classifies the non-translation exception by major number.**
   `(None, Some((98 | 90, _)))` at `tests/errors.rs:178` is a code-prefix rule, which is the shape the amended criterion says to avoid because a prefix is how the earlier draft missed a whole class.
   It is safe here only because a third class fails rather than being absorbed and because the count is pinned at exactly 9. Worth a comment saying that is the reason, since the criterion above it argues the opposite.

4. **The report's "123 rows reporting past line 1" contradicts its own table.** The distribution printed two paragraphs below sums to 132 past line 1, and the committed corpus gives 142. The asserted floor is 100, so nothing depends on it.

5. **`src/instruction/tests.rs:1208` says "All four are measured at RUN time" above a list of five.** Pre-existing, not this task's, but it is the comment backing four of the five ungated classes in Important 5.

6. Four structuring semicolons in comments in the new files. The crate already has 31, so this is consistent rather than novel; noted only because the convention forbids them.

## On the `subs` decision

I scrutinised it rather than accepting it, and I endorse it.

The brief is internally contradictory here — "no field added or removed" in the Interfaces block, then "either fill `subs` ... or remove it" three paragraphs later — and the later paragraph is the operative one.
The removal argument holds on its strongest point, which is not cost: values landing at roughly two hundred sites under a gate that cannot see them wrong would be unverified data wearing the appearance of verified data.
The per-site ambiguity is real too; I confirmed the shape at `expr.rs:1109`/`1112`, where the two 20.923 arms differ precisely in whether a token exists to name.

What replaced it is better than a fallback: I measured `message()` against the whole oracle stderr for all 558 rejected rows and it is byte-identical to one of the two lines every time.
That is a stronger property than the report claims for it, and it is worth being precise about which claim was verified.
The report says "`message()` is byte-identical to one of the two lines `rexxc` prints", over 551 rows.
The population that gives 551 comparisons is the oracle's own `(major, sub)` pairs, not our parser's errors — there are only 542 rows in the first build where our parser both errors and agrees.
Using our parser's own error instead, it is 547 of 549, and the two misses are rows 378 and 697, the recorded label-colon deviation, where our number differs so our message describes a different error.
Neither reading is "zero neither" without saying which. The substance stands either way, and the design is right.

## Criterion 4

**Met in substance. Not met as literally worded.** Reword or close three things:

* **"across every test file and not a named subset"** — `tests/program.rs`'s eight error programs are not in the corpus (Important 3). Either add them, which is the better fix because they are the only errors raised inside a directive body, or say which file is excluded and why.
* **"Recorded exceptions, each with a test pinning both directions"** — the eager-scan exception has no test in the crate (Important 4). Either add one, or state that this exception is pinned by exclusion rather than by a test and that the gate surfaces it as a failure.
* **The INTERPRET residual is unstated** (Important 5). Six programs and five error classes sit outside the gate. The criterion should name them, because an unstated exclusion is the failure mode that made the previous two wordings defective.

Everything the criterion demands substantively is delivered and I verified it independently rather than reading it loosely:

* Soundness holds over 558 oracle-rejected programs with the count reported (492 from the instrumented helpers), number and sub-number matching for all 547 non-deviating rows.
* Completeness holds in both directions: 0 programs the oracle accepts and we reject, and exactly the 9 non-translation rejections we accept, in the two classes the amended text names.
* The line is the oracle's main-message line for 547 of 547, across the three conventions the criterion warns about, and the discriminating floor fails when the corpus loses its multi-line rows.
* The corpus is the oracle's output and nothing else, verified row by row for all 1002 rows.

The criterion is no longer vacuous and no longer blind. What remains is that it over-promises coverage in three specific places, and each is a sentence to fix or a handful of rows to add.
