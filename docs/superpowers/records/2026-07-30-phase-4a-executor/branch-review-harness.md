# Branch review: harness and gate instruments

STATUS: DONE

Scope: `rust/crates/rexx-exec/tests/` (`corpus.rs`, `assertions.rs`, `coverage.rs`,
`loud.rs`, `collect_stress.rs`, `trace_oracle.rs`, `spike.rs`, `trace_oracle/`),
`rust/scripts/mutate-4a.sh`, `rust/corpus/`, `docs/superpowers/plans/phase-4a-gate.md`.

Single question: which of these instruments cannot fail?

Method: every claim below was executed, not read off.
All mutation was done in a scratch copy of the tree at
`/tmp/.../scratchpad/tree` (its own `target/`, its own `.git`, `interpreter/`
symlinked back for the `rexx-inventory` build script).
The shared worktree was never written to; `git status` on it is unchanged.
Where a scratch result could have been confounded by a stale build artefact
(`CARGO_MANIFEST_DIR` is baked in at compile time and `target/` was copied), the
crate was force-rebuilt and the experiment repeated -- one result was overturned
that way, and the overturned version is not reported as a finding.

---

## Summary

| # | Finding | Rating |
|---|---|---|
| H1 | `mutate-4a.sh` reports "9 of 9 mutations caught" and exits 0 with the oracle absent. It never establishes a baseline. | Critical |
| H2 | 17 of the 29 lines in `phase-4a.txt` can be deleted with the whole `cargo test` suite still green; deleting 3 of them silently takes criterion 6 from 9/9 to 5/9. | Important |
| H3 | `trace_oracle/`'s prefix-to-witness table is prose. A witness can be replaced by a program that emits none of the prefix it is named for, the expectation regenerated from the oracle, and all 5 tests stay green. | Important |
| H4 | Gate criterion 6's wording ("verified for real") is accurate for the guard it names but silent about H1, which is the failure mode the same criterion was rewritten to close. | Important |
| H5 | `assertions.rs` is blind to the row set shrinking; the only pin is in another crate's test. Documented, and the cross-check does fire. | Minor |
| H6 | `lang/no_trailing_newline.rex` is a corpus entry whose oracle output is zero bytes on both channels at rc 0 -- the one differential datapoint `/bin/true` would satisfy. | Minor |
| H7 | Criterion 4's "7 of the 29 subset programs panic" is not reproducible from the shipped harness, which aborts on the first panic. | Minor |
| H8 | `mutate-4a.sh`'s `require_clean` uses `git diff` (worktree vs index), so a staged-but-clean-worktree modification passes it; and it writes to a fixed `/tmp` path. | Minor |

Everything else I attacked held. In particular the four instruments I most
expected to be hollow were not: `corpus.rs`'s uncaptured-report mechanism,
`assertions.rs`'s EXEMPT set, the `tags!` compile-error guarantee, and criterion
4's stress mode all failed on demand when I broke the thing they measure.

---

## 1. `corpus.rs` -- holds, except through its input list

**Oracle absent: cannot pass.** Confirmed as a side effect of H1's experiment.
With `oracle_root()` pointed at `/nonexistent/oracle/build`, `corpus_differential`
FAILS with the intended message; there is no skip path.
`check_case` also panics on a `phase-4a.txt` entry that does not resolve, so a
listed program cannot be silently dropped either.

**The report does reach a plain `cargo test`, and the demonstration is a real
negative control.** Scratch mutation: replaced `emit_uncaptured`'s body with
`eprint!("{text}")` (leaving the subprocess code behind `if false`).
`demonstrate_the_report_reaches_a_plain_cargo_test` FAILED, with the child's
captured output showing `test probe_emit_uncaptured_marker ... ok` and no marker.
On the same run `corpus_differential ... ok` printed nothing at all -- exactly the
"green line means 17 of 26 disagree" shape the file exists to prevent.
So this is one of the few self-checks in the tree that is proved rather than argued.

**STRICT genuinely fails.** Not asserted -- `mutate-4a.sh` runs STRICT nine times
and it went non-zero on all nine, with real per-mutation matching counts
(25, 26, 28, 28, 28, 27, 26, 28, 28 of 29).

**Where it can be made hollow: `phase-4a.txt` itself.** Both `total` and the
witness set are derived from the list, so removing a line shrinks the measurement
silently. See H2.

### H2 (Important) -- 17 of 29 subset entries are silently droppable

For each of the 29 entries I removed that one line and ran the only other consumer
of the list that can object, `tests/coverage.rs`. 12 entries are load-bearing for
variant coverage; the other 17 are not:

```
no_trailing_newline, trace_output, leave_nested_outer, iterate_from_select,
if_else_chain, select_when_bodies, select_when_absorption, drop_stem_tail,
stem_aliasing, exit_with_value, exit_no_value, number_identity,
comparison_families, deep_nested_expr, trace_results,
mutation_digits_at_render, mutation_form_at_render, mutation_controlled_order
```

`corpus.rs` and `collect_stress.rs` cannot object by construction -- they report
"N of N" against a total they read from the same file.

The three Task 16 mutation witnesses are in that list, which is the sharp end.
Removing exactly those three lines:

* `cargo test -p rexx-exec` -- fully green (170 lib + 5 assertions + 1
  collect_stress + 2 corpus + coverage + loud + trace_oracle + spike, no failures);
* `bash scripts/mutate-4a.sh` -- **5 of 9 caught**, exit 1. Mutations 3
  (`Loop::end` off-by-one), 4 (`Controlled::order` reversed), 8 and 9
  (created-vs-current DIGITS/FORM) all go NOT CAUGHT.

So a three-line edit to a `.txt` file halves criterion 6's power and the test
suite says nothing. The only thing that notices is a hand re-run of a script that
is not part of `cargo test`. Note the asymmetry: *adding* a program to
`corpus/lang/` is guarded (`rexx-parse`'s `sourceline_oracle` walks the directory
with `read_dir` and demands a committed expectation per file), *removing one from
the subset list* is not guarded by anything.

Cheapest fix in the same spirit as the rest of the tree: assert the subset list
against a committed literal, the way `coverage.rs` already pins
`EXPECTED_OUT_OF_SCOPE` and `assertions.rs` pins `EXEMPT`.

### H6 (Minor) -- one vacuous differential datapoint

`lang/no_trailing_newline.rex` produces, under the oracle, zero bytes of stdout,
zero bytes of stderr, rc 0. It is a legitimate Phase 3 artefact (its point is the
missing final newline, for `SOURCELINE`), and it does still discriminate an
`IF`/`ELSE` mis-wiring that would print `unreached`. But as a member of a
"29 of 29 matching" count it is the one entry whose match a do-nothing
implementation would also earn. Worth a sentence in the corpus README rather
than a change.

---

## 2. `assertions.rs` -- the EXEMPT set holds in both directions

Four attacks, all in the scratch copy.

**(a) Delete one `EXEMPT` entry.** `the_exempt_set_matches_the_current_blocked_rows`
FAILED: `left: 35, right: 34`.

**(b) Corrupt one `EXEMPT` entry's `expr` text** (identity is
`group`+`method`+`occurrence`+`expr`+`expected`, so this desynchronises the lookup
without changing the count). Both `the_exempt_set_matches_...` and, under
`REXX_ASSERTIONS_GATE=1`, `assertions_differential` FAILED, with
`Literals::test_hexadecimal occurrence 1 is not passing and is not on the
committed EXEMPT list` in the report.

**(c) Both directions at once -- the documented Task 15b control, reproduced.**
Deleting `program_for`'s prelude-writing loop gives **3,923 of 4,259 rows passing**
(336 not passing against `EXEMPT`'s 35) and **345 EXEMPT-set violations**, split
exactly as `task-15b-report.md` records it: 22 rows that "now PASSES but is still
listed in EXEMPT", 323 unlisted rows that started failing. The documented figure
reproduces to the row.

The set assertion is a genuine bijection, not a one-sided containment: the
identity key includes `occurrence`, which is unique per row within a
group+method, so "every not-passing row has an entry" plus "the lengths match"
does force "every entry has a not-passing row". That reasoning is sound.

**(d) A row that silently stops being evaluated -- H5 (Minor).** Moving
`CONCATENATION.testGroup` (388 rows) out of the suite leaves `assertions.rs`
**fully green in STRICT mode**, at 3,871 rows instead of 4,259: `still_blocked`
is unchanged because all 35 exempt rows live in `Literals`. The file declines to
pin the row count on purpose and says why (`collect_all`'s doc comment), delegating
to `rexx-extract`'s `base_expressions_yields_the_measured_row_and_blocked_counts`.
I verified that delegate actually fires -- with the group removed it FAILS,
`left: 3871, right: 4259`, as does `base_expressions_expect_syntax_conversion_counts`.

One caveat on that verification, recorded because it nearly became a false finding:
on the first attempt the extractor test passed with the file removed, because
`rexx-extract` had not been recompiled in the scratch tree and its baked-in
`CARGO_MANIFEST_DIR` still pointed at the real repository. After
`touch`ing its sources the test fails as it should. The pin is real.

Residual: the delegation means a shrinking suite is caught only by a
*workspace* run, and only by a count in a different crate. That is a documented,
deliberate choice, not a defect -- Minor.

**Not attacked:** I did not try to make an exempt row genuinely pass by
implementing part of Phase 5; (c) covers that direction adequately.

---

## 3. `coverage.rs` and `loud.rs` -- the compile-error claim is true, verified

The claim is that the `tags!` macro has no wildcard arm so a new variant is a
compile error. I added variants rather than believing it.

**`PrefixOp::ZzzReviewProbe`.** `rexx-parse` compiled unchanged; three production
sites in `rexx-exec` (`eval.rs:230`, `eval.rs:403`, `lib.rs:504`) refused. With
those patched, the next and only error was
`crates/rexx-exec/tests/coverage.rs:142: error[E0004]: non-exhaustive patterns:
&PrefixOp::ZzzReviewProbe not covered`.

**`InstructionKind::ZzzReviewProbe`.** Production sites `rexx-parse/src/ast.rs:898`,
`rexx-parse/src/block.rs:776`, `rexx-exec/src/plan.rs:133` and
`rexx-exec/src/lib.rs:394` refused first. With those patched:

```
crates/rexx-exec/tests/loud.rs:88:      error[E0004] ... not covered
crates/rexx-exec/tests/coverage.rs:142: error[E0004] ... not covered
crates/rexx-exec/tests/coverage.rs:399: error[E0004] ... not covered
```

Both files, both enums. Verified rather than asserted.

Two secondary observations, neither a defect:

* `lib.rs:394` is the loud-failure namer, so a new `InstructionKind` also forces a
  decision *there* -- the same variant cannot be added without someone choosing
  whether it is loud. That is a stronger guarantee than the gate document claims.
* `loud.rs` checks only `exit_code == NOT_IMPLEMENTED_EXIT`, never that the stderr
  message names the variant under test. For single-construct witnesses that is
  fine; for `VariableReference` (necessarily inside a `CALL`) the file already
  says the loudness is not attributable to the variant itself. Correctly disclosed,
  no action.

---

## 4. `mutate-4a.sh`

### H1 (Critical) -- the script reports full coverage against an absent oracle

`run_one` calls `run_subset` (`REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test
corpus`) and treats **any** non-zero exit as "caught". There is no baseline run
before the first mutation, and no check that the unmutated tree passes.

Experiment: scratch copy, `oracle_root()` repointed at `/nonexistent/oracle/build`,
nothing else changed. `bash scripts/mutate-4a.sh`:

```
=== 1..9 ===
caught: the subset diverges under this mutation.
test corpus_differential ... FAILED
the oracle binary is missing at /nonexistent/oracle/build/bin/rexx. ...
==============================================================================
9 of 9 mutations caught by rust/corpus/phase-4a.txt
==============================================================================
exit 0
```

Nine for nine, exit 0, having compared nothing at all. Any condition that makes
the corpus test fail for an unrelated reason -- a missing oracle, a broken
`LD_LIBRARY_PATH`, an unrelated compile error, a corrupted `phase-4a.txt` --
converts this instrument into an unconditional pass. This is the same class as
the `/bin/true` defect that criterion 6 was rewritten to escape, arriving through
the back door: the old wording had a control too weak to fail, this one has a
control that cannot distinguish its own failure from the mutation's.

Fix is three lines: run `run_subset` once before `require_clean` returns and abort
if it does *not* succeed, and abort again at the end if the restored tree does not
pass. That also catches an interrupted revert.

### The stale-pattern guard is real

Replaced mutation 3's OLD string with one that cannot match. Output:

```
=== 3. off-by-one on Loop::end ===
UNAPPLIED PATTERN: found 0 times (need exactly 1) in crates/rexx-exec/src/run.rs.
FATAL: mutation '3. ...' could not be applied ...
exit 1
```

It aborts rather than skipping, mutations 4-9 never run, and the `EXIT` trap
restored `run.rs`/`eval.rs`/`value.rs` byte for byte (`git diff --quiet` clean
afterwards). Confirmed.

### No mutation is caught by a compile error

I ran a variant of the script whose only change was `run_subset` doing
`cargo build --offline -p rexx-exec --tests` instead of the corpus test.
**All nine compile cleanly** -- "0 of 9 caught" by the compiler. Combined with the
real run's per-mutation divergence counts (25/26/28/28/28/27/26/28/28 of 29), every
one of the nine is caught by an observed output difference against the oracle, as
the gate document claims.

### H8 (Minor)

`require_clean` uses `git diff --quiet -- <3 files>`, which compares the worktree
to the *index*; a change that is staged but matches the worktree passes the guard,
and the `cp` backup then captures it. `git diff HEAD --quiet` would close it.
Separately, `run_subset` writes to a fixed `/tmp/mutate-4a-subset-output.txt`, so
two concurrent runs interleave; `mktemp` would be free.

---

## 5. `trace_oracle/` -- the expectations are genuine; the table around them is not enforced

**The committed expectations reproduce exactly.** I re-ran the regeneration recipe
from the module comment against the live oracle for all five witnesses
(`keyword_while`, `compound_read_write`, `prefix_operators`,
`dotvariable_beyond_the_list`, and `corpus/lang/trace_output.rex`) and diffed:
**all five byte-identical**. Nothing here was generated from our own output. The
reviewer's earlier confirmation stands.

### H3 (Important) -- a witness can stop witnessing and nothing notices

The prefix-to-witness table lives only in the module doc comment. `check_witness`
compares this crate's output to the committed file and nothing else. So:

Scratch experiment -- replaced `keyword_while.rex` with `trace r / n = 0 /
n = n + 1 / say n` (no loop, therefore no `>K>` anywhere), regenerated
`keyword_while.expected` from the oracle with the file's own documented recipe,
and re-ran: **5 passed, 0 failed**, with `grep -c '>K>' keyword_while.expected`
returning 0. The test named
`keyword_while_covers_a_re_evaluated_keyword_across_every_pass` passes while
covering no keyword at all.

Today the prefixes are all really there -- measured:

| expectation | prefixes present |
|---|---|
| `trace_output` | `*-*` `>>>` `>=>` `>L>` `>V>` `>O>` |
| `compound_read_write` | `*-*` `>>>` `>=>` `>L>` `>V>` `>C>` |
| `keyword_while` | `*-*` `>>>` `>K>` |
| `prefix_operators` | `*-*` `>>>` `>L>` `>P>` |
| `dotvariable_beyond_the_list` | `*-*` `>>>` `>E>` |

Union is exactly the ten claimed. A ten-line assertion (each witness's expectation
must contain the prefixes the table says it does) would turn the doc table into an
instrument. Without it, the table is a comment.

This sharpens the gate document's criterion 3 wording rather than contradicting it
-- see H4 below.

---

## 6. `collect_stress.rs` -- criterion 4's instrument is sound, and I reproduced its control

This is the criterion the gate calls out as built on the last day, and it is the
one I most expected to be inert. It is not.

**Negative control reproduced.** Deleted `eval.rs`'s
`self.roots.push_temp(left_value);` in `eval_arithmetic` (the site the gate document
names) in the scratch copy:

* `cargo test -p rexx-exec --test collect_stress` -- **FAILED**, `panicked at
  crates/rexx-exec/src/value.rs:205: a live value`.
* `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` on the *same* tree --
  **29 of 29 matching, ok**.

So the rooting defect is entirely invisible to criterion 1 and visible only to
criterion 4. That is the strongest single result in this review: the mode is a
genuinely independent instrument, not a restatement of the corpus run.

The two anti-vacuity assertions (per-program `collections > 0`, and the aggregate)
are the right shape, and the per-program one is the load-bearing half.

**H7 (Minor).** The gate document reports "7 of the 29 subset programs panic"
under this control. The shipped harness cannot produce that number -- it aborts on
the first panic, as my run did. The figure must have come from running programs
individually, which is fine, but it is not re-derivable by re-running the control
as documented. Either say so, or catch the panic per program so the count is
reproducible.

---

## 7. `spike.rs`

Not a gate instrument -- Task 3's design artefact, kept deliberately. Read, not
attacked. Two notes:

* `the_stack_span_does_not_depend_on_what_else_the_program_evaluated` asserts
  *equality* of two spans rather than a bound, and its own comment explains that a
  bound would pass for both the fixed and the broken version. That is the right
  instinct and rare.
* `records_the_stack_cost_of_one_eval_frame` carries a deliberately loose bound
  (`> 4096`) plus a `println!`; its own comment says the tight number will move.
  Its `survivable > 100_000.0` check is the only real assertion, and the comment
  records that Task 11's `MAX_EVAL_DEPTH` removed the ability to re-confirm the
  extrapolation through the public entry point. Disclosed, not hidden.

I did not re-capture spike.rs's eleven oracle transcripts against the live oracle.

---

## 8. The gate document, criterion by criterion

| Criterion | Verdict claimed | My judgement |
|---|---|---|
| 1. L0 subset + variant coverage | MET | **Supported, with H2.** Both halves do what they claim; the differential half fails on an absent oracle and the coverage half is a real compile-time guarantee. Neither guards the subset *list*, and the document does not raise that. |
| 2. Assertion table | MET, with a recorded criterion defect | **Supported.** The defect described (all 35 blocked rows are Phase 5's, which the criterion never licensed) is exactly what `EXEMPT` records, and both directions of the set assertion fire. The claim "both were checked for this document; neither fired" is consistent with what I measured. |
| 3. Trace, byte for byte | MET, weakly | **Supported, but weak in a slightly different place than described.** The document says the trace *surface*'s coverage is unmeasured, which is true. H3 shows the *committed table's own claimed coverage* is also unmeasured -- a witness can quietly stop witnessing its prefix. That is a second, narrower weakness the "weakly" verdict does not name. |
| 4. Collect on every allocation | MET, on a mode built today | **Supported and then some.** Every caveat the document raises is honest, and the negative control reproduces. H7 is a small reproducibility nit on one figure. |
| 5. Loud failure | MET | **Supported.** Verified by adding a variant, not by reading the macro. |
| 6. Mutation control | MET | **Not fully supported -- H1/H4.** Every specific claim in the section is true: the guard was verified for real (I reproduced it), all nine are caught by a running divergence and none by a compile error (I reproduced both). What the section does not say is that the instrument producing "9 of 9" reports 9 of 9 against an absent oracle, and that three deletable lines in a `.txt` take it to 5 of 9. For a criterion whose entire purpose is to be the control on the other criteria, that gap belongs in the verdict. |
| 7. unsafe / clippy / fmt / spike | MET | Not independently re-run (already verified elsewhere this session). |

### H4 (Important) -- what criterion 6's section should say

The section's own framing is "this replaces `/bin/true`, which satisfied the old
wording". The replacement inherits a different version of the same problem: a
control whose success signal is "the subject failed", with no check that the
subject can succeed. Two sentences and a three-line script change close it.

---

## What I did not reach

* The full workspace run (824 tests) in the scratch copy -- taken as already
  verified this session, and everything I ran in `rexx-exec`, `rexx-extract` and
  `rexx-parse` was green unmutated.
* `rust/corpus/expr/precedence.tsv` and `rust/corpus/errors/parse-errors.tsv` --
  Phase 3 instruments consumed by `rexx-parse`, adjacent to my slice. I checked
  only that `sourceline_oracle.rs` walks `corpus/lang/` with `read_dir` (relevant
  to H2's asymmetry).
* `spike.rs`'s eleven oracle transcripts were not re-captured.
* I did not try to make a listed `EXEMPT` row pass by implementing Phase 5
  behaviour; the prelude-deletion control covers that direction.
* I did not attempt to falsify `coverage.rs`'s `EXPECTED_OUT_OF_SCOPE` or
  `variant_counts_match_the_audited_split` by hand-editing them -- they are
  literal-vs-derived comparisons whose failure mode is mechanical.
* No timing or flakiness analysis; every experiment was run once unless it
  contradicted a documented figure, in which case it was repeated.
