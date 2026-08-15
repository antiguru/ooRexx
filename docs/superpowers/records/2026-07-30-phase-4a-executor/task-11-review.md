STATUS: DONE

# Task 11 review: DO/LOOP, LEAVE/ITERATE, D19 depth limit, clause-echo indentation

Commit under review: `2c9b966c` (single commit, five files, 2,356 insertions).
Reviewer: same as Task 10's, carrying that context.
Verdicts: spec compliance and code quality, findings rated Critical / Important / Minor.

Coordinator pre-verified (not re-run here): 781 workspace tests green, clippy rc 0, fmt rc 0, corpus 22 of 26 (remaining four all TRACE), eleven byte-identical differential probes including indentation at three nesting levels and `leave i` from a nested loop.

## Findings

### Reading phase: the shape of the implementation (established from the diff)

`Flow` gains `Leave(Option<SymbolId>, LeaveOrigin)` and `Iterate(...)`; `run_loop` dispatches per `LoopKind` (`Simple` runs once; `Forever`/`Count`/`Controlled`/`OverOnce` share `run_repeating`); `COUNTER` and `DO WITH` take the loud path before any header evaluation; `run_repeating` is an internal `loop {}` that never returns mid-construct; `do_body_outcome` and `leave_select` consume or forward `Leave`/`Iterate`; `static_indent`/`indent_in_range` compute indentation as a pure function of the flat instruction list; `LeaveOrigin` eagerly captures `(site, static_indent)` at the instruction's own step; 28.1-28.4 hardcode indent zero at the top-level conversion, 28.5 reports the captured indent; `MAX_EVAL_DEPTH = 100_000` checked with `>` in `eval` after the `StackSpan` bookkeeping, with symmetric decrement on the refusal path; `FailureSite` is a named struct replacing the tuple; `Raised::report` prefixes `site.indent` spaces before the clause text only.

Checks that pass by code walk, before any mutation: the depth decrement on `eval`'s refusal path is symmetric (each level decrements exactly once whether `eval_node` ran or not); `whole_nonneg` shares one rule for 26.2/26.3 with the raiser chosen by the caller; `setup_controlled` evaluates header expressions in `ctrl.order` (side-effect order preserved) and checks `initial`/`TO`/`BY` as numeric via `arith_operand` (41.1), never whole; the control variable is bound before the bound test and before the `FOR`-exhaustion check; an iterated pass consumes `FOR` budget; `loop_step` adds `by` under current `DIGITS`.

### Priority 1, verified by mutation: the driver really must not return mid-construct, and the named test bites

Scratch copy synced to `2c9b966c`, baseline `cargo test -q -p rexx-exec --lib` = 155 passed, matching the report.

* **Mutant B**, `run_repeating` returns `Ok(Flow::Goto(body_start))` after each completed body pass (a mid-construct return whose re-walk terminates): **19 tests fail**, `leave_and_iterate_survive_a_do_nested_in_an_ifs_then_iterating_repeatedly` among them (prints 2, expects 9 -- exactly the "loses the running total" signature its doc comment predicts).
* **Mutant D**, `run_repeating` returns `Ok(Flow::Goto(do_index))` (the exact re-entry-as-first-entry shape the brief warned about): the named test **hangs forever** (killed by a 45-second timeout, status 124, no test result line). It cannot pass under the mutant.
* "Ideally only that test goes red" is not physically achievable for this mutant class, and that is a property of the design, not a test gap: all loop state lives in `run_repeating`'s own stack frame, so *any* mid-construct return loses it for *every* repeating loop everywhere -- the top-level `run_activation` loop absorbs the `Goto` exactly as an enclosing `IF`'s `run_bounded` does. Mutant B breaks 19 tests; Mutant D hangs nearly all of them. The named test is aimed at the right mechanism (it is the only one that combines the enclosing `IF` with an accumulating total), and the mechanism is defended in depth besides.

### Priority 4, verified: the off-by-one and the measurement, and an Important bookkeeping failure

* **Mutant E**, `eval`'s `> MAX_EVAL_DEPTH` changed to `>=`: exactly `eval_survives_exactly_max_eval_depth_terms_and_prints_the_oracles_own_answer` fails. The 100,000-term boundary case is pinned and would refuse under the off-by-one, as required.
* **The 1840 measurement reproduces**: re-ran `records_the_stack_cost_of_one_eval_frame -- --nocapture` in the scratch copy and got byte-identical output, `per frame: 1840.0 bytes`, depth 100,000. The figure is real.

**Important: the re-measured figure was never recorded in the tree, and the report claims it was.**
The report says "`lib.rs`'s own doc comment on the constant now carries this row rather than replacing the ones before it."
It does not: `grep` for `1840`, `291,777` or `291777` over `lib.rs` (and `spike.rs`) finds nothing.
Nothing was overwritten, which honours half the instruction, but the add never happened, so `lib.rs`'s doc comment still presents ~1600 bytes/level and ~335,000 survivable levels as the current figures, and `eval.rs`'s `MAX_EVAL_DEPTH` doc quotes "~1600 bytes/level ... re-measured at implementation time below" with nothing below carrying the re-measurement.
The fifth revision of this figure exists only in the report file.
Fix is the row the coordinator asked for: one paragraph beside the ~1600 one, with date, method (the spike test's own printout) and the 1840/~291,777 pair.

### Priority 3, verified behaviourally: `IF` is transparent to the search

* `if 1 = 1 then leave`: oracle raises 28.1 (rc 228) -- an `IF` is not a leavable block. Ours raises 28.1, same rc, same clause and line (the indent differs, see below).
* `select label s / when 1 = 1 then if 1 = 1 then leave s / otherwise nop / end / say 'after'`: `leave s` crosses the `IF` and exits the `SELECT LABEL`; both sides print `after`, rc 0.
* `do i = 1 to 3 / if 1 = 1 then iterate / say 'unreached' / end / say 'done' i`: the bare `ITERATE` crosses the `IF` and is consumed by the loop; both sides print `done 4`, rc 0.

The C++-absence argument holds behaviourally on all three shapes.

### Priority 2, part 1, verified: the static quantity holds where the eleven probes did not look

* `do i = 1 to 2 / do while 1/0 / ...`: oracle indents the `do while` echo **4** (line 2, the inner `do` clause); ours 4, same attribution.
* Same with `until`: oracle echoes the inner **`end`**, line 4, indent **4**; ours identical.
* **`TRACE` with an `IF`** (`trace r / if 1 = 1 then say 5`): the oracle's `*-*` line for `say 5` indents **4**, exactly `static_indent`'s `IF`-branch rule. The report measured only the plain-`DO` trace case and said so honestly; this probe extends the match to `IF`, which strengthens the shared-quantity claim Task 13 inherits. (The trace also echoes `then` at indent 2 -- a marker line the error path never prints; Task 13's problem, noted for it.)

### Priority 2, part 2 -- **Important: the 28.x indent rule is overfit, and seven probes diverge from the oracle byte-for-byte**

The report's rule is "28.1-28.4 always report indent zero; 28.5 reports the instruction's full lexical depth."
Every probe behind that rule nested the instruction only in constructs of one kind.
Probing mixed shapes falsifies both halves (extra spaces beyond the `*-* ` baseline; "ours" verified by running `rexx-run` from the identical sources):

| probe | shape | family | oracle | ours |
|---|---|---|---|---|
| p5 | `if 1=1 then leave` | 28.1 | **4** | 0 |
| p9 | `if 1=1 then do / leave / end` | 28.1 | **6** | 0 |
| p12 | `do / leave / end` | 28.1 | **2** | 0 |
| p8 | `if 1=1 then do i=1 to 3 / leave zz / end` | 28.3 | **4** | 0 |
| p2 | `do label x / select / when 1=1 then iterate x / ...` | 28.5 | **2** | 8 |
| p10 | same, `iterate x` inside a `do` inside the `WHEN` | 28.5 | **2** | 10 |
| p13 | `do label x / do label y / iterate x / ...` | 28.5 | **2** | 4 |
| p1 | `do label x / if 1=1 then do / iterate x / ...` | 28.5 | 8 | 8 ✓ |
| p11 | `select label s / when 1=1 then iterate s / ...` | 28.5 | 6 | 6 ✓ |

Everything else on the report lines is identical (clause text, line number, both error lines, rc 228 -- confirmed in full for p12); the divergence is indentation only.

**The rule all fourteen data points (mine plus the report's own seven) obey**: the search walks a frame stack to which every `SELECT` and every `DO`/`LOOP` *except an unlabelled `Simple` block* contributes a frame; each frame the search pops **restores the indent to the value saved when that frame was pushed** (its own construct's `static_indent`); the error reports whatever indent is left.
So 28.1-28.4 report zero only when the outermost popped frame sits at top level (the report's probes all did), and report the residual `IF`/unlabelled-block contribution otherwise; 28.5 reports the instruction's own depth only when no pushed frame lies between it and the match (the report's probes all had only unlabelled-`Simple` or `IF` intervenors, which push nothing).
Two consequences worth stating explicitly:

* **The quantity is still fully static.** Every number above is a pure function of the instruction list and the search outcome, so the static design survives intact; only the two-family shortcut is wrong.
* **The fix fits the existing propagation exactly**: when a construct that owns a frame (`Select` always; `Do`/`Loop` when `is_loop` or labelled -- precisely the arms that already inspect the `Flow`) forwards an unconsumed `Leave`/`Iterate` outward, update `origin.indent` to that construct's own `static_indent`. An unlabelled `Simple` block forwards without touching it. Checked against all fourteen points on paper: every one reproduces, including the report's own seven.

**Severity: Important, not Critical.** Error-path stderr indentation only; values, control flow, exit codes, clause attribution and lines all match; the corpus (22 of 26) and all eleven of the coordinator's differential probes are unaffected. But these are real, shipped, oracle-diffable divergences, and the report's written rule would mislead Task 13 the same way the brief warned Task 10's inferred rows could have.

On the eager capture itself: eager capture of the *site* (line, text) is correct and cannot be stale (a lexical position does not move).
The captured single `indent` is the piece that cannot express the oracle's restore-on-pop behaviour, which is exactly what the seven divergent probes show.

### Minor: `run_loop`'s `OVER`-stem comment documents the opposite of what the code does

The comment says a stem reached indirectly, "`over (a.)`", "is not detected here".
Measured: `do v over (a.)` on our side takes the loud path (`DO is not implemented`, rc 120) -- the parenthesised stem *is* detected, because the parser resolves `(a.)` to the same `ExprKind::Stem` a bare `a.` gets.
The behaviour is the safe direction (loud, never a silent divergence) and better than documented; the comment is what is wrong.
No misfire on a non-stem is possible: only a bare trailing-dot symbol parses as `ExprKind::Stem`, and a compound (`a.b`) is `ExprKind::Compound`.

### Verified: the remaining sanctioned and swapped items

* **`DO WITH` and `COUNTER` fail loudly**, checked before any header expression is evaluated (`run_loop`'s first line), each with a dedicated test (`do_with_takes_the_loud_path`, `do_counter_takes_the_loud_path_regardless_of_which_other_kind_it_rides_on`).
* **`CALL` is genuinely out of 4a's reach**, not merely unimplemented today: `run.rs` has no `Call` arm (it falls to the loud catch-all), the Task 13 brief (the only remaining implementation brief) never mentions `CALL`, Task 12 and the Task 14 family are complete, and `lib.rs`'s own doc comments assign the fragment-consuming keywords to 4b. The spike witness swap is sound.
* The depth tests live in `eval.rs`'s `#[cfg(test)]` and drive `run_program`, per the coordinator-resolved conflict in the report; the subject at the limit is public surface, so the private-subject rule is not violated.

## Verdicts

**Spec compliance: PASS with two Important findings.**
Every loop form, the full 26.2/26.3/28.1-28.5/34.3/34.4/41.1 family, `LEAVE`/`ITERATE` semantics including the automatic control-variable label and the `SELECT LABEL` rules, `ITERATE`'s bottom-of-iteration semantics, the `UNTIL`-blames-`END` discriminator, and D19's two-sided depth boundary are implemented, measured, and mutation-defended; the corpus went 12 to 22 of 26.
The two compliance gaps: the 28.x error-report indentation diverges from the oracle in seven probed nested shapes (rule overfit, above), and the re-measured depth figure was recorded nowhere in the tree despite the report claiming it was.

**Code quality: PASS.**
The `run_repeating` never-return-mid-construct discipline is real and mutation-verified from both directions (terminating and hanging mutants both die).
`static_indent` as a pure function is the right call and the report's argument for it (no state to desync) is borne out by its own named test.
Reuse is consistent (`arith_operand`, `saturate_digits`, `eval_condition`, `compare_decoded` via `numeric_less` with a correct argument for the empty byte slices).
One comment documents the opposite of its code (the `over (a.)` Minor above).

## Not reached

* Did not re-run clippy, fmt, the 781 workspace tests, the corpus count, or the eleven differential probes the coordinator already verified.
* Did not probe the full `TO`/`BY`/`FOR` written-order side-effect matrix against the oracle; verified by code walk (`ctrl.order` drives evaluation) and the report's table only.
* Did not probe `OVER ... FOR` on a non-stem against the oracle; the report names it as a judgement call and the test doc comment repeats that honestly, so it stays a flagged judgement, not a verified fact.
* Did not probe very large repeat counts (`do 1e20`) or the `whole_nonneg` `u64` ceiling against the oracle.
* Did not chase the C++ source to confirm the restore-on-pop mechanism I inferred; the fourteen oracle data points pin the behaviour, not the implementation.
* Did not re-litigate the report's self-disclosed `git checkout --` rule violation; it is disclosed, no work was lost, and it is the coordinator's call.

## Re-review of the fix round (`2c9b966c..1ff7ba61`)

Scope: both Importants and the Minor.
Method: read the 839-line diff in full, mutated the new logic four ways in a scratch copy synced to `1ff7ba61` (baseline 157 lib tests green), independently oracle-probed the implementer's own new table rows, and byte-compared our fixed outputs on the previously-divergent shapes.
This round changed control-flow logic; I hunted for what it broke and found nothing behavioural.

### Verified: a matching search stops without resetting

By code walk: in `leave_select` the two matching arms (guarded on `label == Some(name)`) precede the two pop arms, so a match consumes before any reset can run; in `do_body_outcome` the `matched` branches consume (`Goto(resume)`, `Ok(None)`, or the 28.5 `Err`) without touching `origin`, and 28.5 records the residual as-is, which is correct because any intervening pops already ran on the way in.
Behaviourally: the table's `p1`/`p11` rows are exactly the immediate-match shapes (full depth reported, nothing reset), and under mutants M1/M2 every consumed-`LEAVE`/`ITERATE` behaviour test stayed green (155 and 156 of 157 passing respectively), confirming `pop_search_frame` is reachable only on the not-consumed path and can only ever move `FailureSite.indent`, never control flow.

### Verified: the twelve-row table test can fail, and fails on the right rows

* **M1**, `pop_search_frame` returns `origin` unchanged (the old rule resurrected): 2 tests fail, the table dying on **p8**, the first row where a pop matters (`origin` 6, correct 4), while the no-pop rows `p5`/`p9`/`p12` before it pass. `leave_no_match_through_two_real_loops...` dies too.
* **M2**, pop resets to zero: table dies on **p8** (correct 4, mutant 0), the first row whose saved indent is nonzero.
* **M3**, `owns_frame = is_loop` (drops the labelled-`Simple` frame): exactly one test fails in the whole suite, the table, on exactly **p13**, the one row that discriminates that clause of the rule.

The table encodes the oracle, not the implementation: I had independently measured nine of its rows against the oracle before this round existed, and the assert-per-row messages name the row that dies.

### Verified: the independent agreement is genuine

I oracle-probed the implementer's two genuinely new shapes myself (not previously in my table): `n1` (bare `LEAVE` through an unlabelled `SELECT`) reports indent 0, and `n3` (named `ITERATE` crossing a real loop and a `SELECT`, matching neither) reports indent 0, both matching the table.
Our side, rebuilt at `1ff7ba61`, is byte-identical to the oracle's echo line on `n1`, `n3`, and the three previously-divergent shapes I re-ran (`p2` now 2, was 8; `p9` now 6, was 0; `p12` now 2, was 0).

### Verified: the depth row landed correctly

The 1840 paragraph sits beside Task 7's ~1600 text in `lib.rs` (both intact), gives the method (the exact `cargo test ... -- --nocapture` command plus its verbatim output line), the survivable-depth arithmetic (~291,777, ~2.9x headroom), and lists the full lineage (820, 850, 783/784, 1600, 1840) with "each was correct for the code it measured" -- prior figures read as correct-for-their-commit, not as errors.
`eval.rs`'s stale "re-measured at implementation time below" is genuinely gone, replaced by the figure itself (1840) and a pointer to `lib.rs`.
`tests/spike.rs` carries the figure beside the test that produces it.
One wording nit, not worth a round: `eval.rs` says the row is recorded "with the date"; the row is task-anchored ("at Task 11") like every other row in that table, with no calendar date. Consistent with its neighbours, just not literally what the sentence says.

### Verified: the `over (a.)` test pins the loud path

**M4**, deleting the `ExprKind::Stem` check: exactly the two OVER-stem tests fail (`do_over_a_stem_target_takes_the_loud_path` and `do_over_a_parenthesised_stem_target_is_also_caught`), 153 others filtered/passing.
Without the check the program runs to completion and `unwrap_err` panics, so the test cannot pass for an unrelated reason; the only loud-capable step in its program is the `Do` arm's stem detection.
The corrected comment now states the true mechanism (a single parenthesised sub-expression collapses to its own `ExprKind`) and names the remaining genuine gap (a function call returning a stem, 4b's problem).

### Re-review verdict

Both Importants and the Minor are fixed, the fix generalises past the data it was fitted to (the coordinator's two unlisted shapes plus my two spot-checks all match), the new logic is mutation-defended row-by-row, and nothing behavioural moved.
No new findings beyond one wording nit.
**Task 11 can close.**
