STATUS: DONE

## Round 2: the coordinator found a real defect in the round-1 fix

After the round-1 commit, the coordinator measured `select` / `when 'x'
then nop` / `end` and found it attributed to the `SELECT`'s own clause
(line 1, `select`) instead of the `WHEN`'s (line 2, `when 'x' `) -- wrong
clause *and* wrong line, not merely the indentation gap.

**Cause, exactly as the coordinator diagnosed it.** Real dispatch for
`When`/`WhenCase` lives entirely in `Select`'s own `step` arm, which reads
a `When`/`WhenCase` node as *data* and evaluates its condition/values
directly (`self.eval_condition(...)`, `self.test_case_when(...)`) --
never through `step_in_temps_frame` for that instruction, because that
instruction's own `step` arm is a pure no-op. Round 1's fix (moving
failure-site resolution into `step_in_temps_frame`, innermost wins) closed
the misattribution for an instruction *inside* a branch's body, which
always does go through `step_in_temps_frame` via `run_bounded`. It did not
close it for a `WHEN`/`WhenCase`'s own *condition*, because there is no
`step_in_temps_frame` call for that at all -- the report's own round-1
text described this as "closed", and it was only half closed. `IF` never
had this defect: `if 'x' then nop` blames the `IF` clause correctly,
because the `IF` *is* the failing instruction and *does* go through
`step_in_temps_frame` for itself.

**Fix.** Factored `step_in_temps_frame`'s resolution logic out into
`record_failure_site(source, instruction)` (first-call-wins guard
unchanged), and made `Select`'s own arm call it directly, against
`when_instruction` (the specific `When`/`WhenCase` node whose condition or
values are being evaluated), on the error path of both
`self.eval_condition(...)` and `self.test_case_when(...)` -- explicit
`match` instead of `?`, so the site is recorded before the error
propagates. `step_in_temps_frame` itself now just calls
`record_failure_site(source, instruction)` unconditionally on any `Err`.

**All five oracle cross-checks the coordinator asked for, verified via
`rexx-run` under the mandated `ulimit`, byte-for-byte except the
already-out-of-scope indentation** (transcripts in
`.../scratchpad/probes/{when_cond_raises,case_expr_raises,
whencase_value_raises,otherwise_raises,second_when_raises}.rex`):

| Scenario | Attributed to | Confirmed |
|---|---|---|
| `select` / `when 'x' then nop` / `end` | the `WHEN`, line 2 | yes (was line 1/`select`, now fixed) |
| `select case (1/0)` / `when 1 then nop` / `end` | the `SELECT`, line 1 | yes -- **confirmed rather than assumed**: `case` is still evaluated directly inside `Select`'s own `step` call with no intervening instruction to blame, so this was already correct and needed no code change |
| `select case 1` / `when (1/0) then nop` / `end` | the `WHEN`, line 2 | yes -- same fix as the plain-`WHEN` case, since `test_case_when`'s error path is symmetric with `eval_condition`'s |
| `select` / `when 1=0 then nop` / `otherwise` / `say 1/0` / `end` | the `say 1/0` clause, line 4 | yes -- already correct before this round too: `OTHERWISE`'s body runs through the *outer* loop's ordinary `step_in_temps_frame`, never through `Select`'s own explicit-match path, so it was never exposed to either defect |
| `select` / `when 1=0 then nop` / `when 'x' then nop` / `end` | the second `WHEN`, line 3 (not the first's line 2) | yes -- the line genuinely moves with the failing `WHEN`, not merely differing from the `SELECT`'s |

Added five unit tests checking `interp.failure_site` directly (line **and**
clause text, not just `raised.number`/`.sub`, which no existing test in
this file checks and which is exactly the blind spot that let the round-1
defect through unnoticed by `cargo test`):
`a_when_conditions_own_failure_is_attributed_to_the_when_not_the_select`,
`the_second_of_two_whens_own_failure_moves_the_line_with_it`,
`a_select_cases_own_expression_failure_is_attributed_to_the_select`,
`a_whencase_values_own_failure_is_attributed_to_the_when_not_the_select`,
`a_raise_inside_an_otherwise_branch_is_attributed_to_its_own_clause`.
`cargo test -q -p rexx-exec --lib`: **102 passed, 0 failed** (97 + 5 new).
`cargo test -q --workspace`: all green. `cargo fmt -p rexx-exec --check`
and `cargo clippy -q -p rexx-exec --all-targets -- -D warnings`: both
clean.

**Corpus, re-run as asked: still 9→12 of 26, unchanged by this fix.** None
of the 26 corpus programs currently exercise a raise inside a `WHEN`'s
condition (the two that raise at all inside a `SELECT`/`WHEN` construct,
`select_when.rex`/`select_when_bodies.rex`, are still blocked on `DO`
before reaching that code at all), so this fix has no observable effect on
the corpus number yet -- it closes a real defect that the corpus does not
currently happen to exercise, not a regression in the count.

## The coordinator's other two decisions, both settled

**Indentation stays out of scope for me; Task 11 owns it.** The
coordinator confirmed my characterisation independently and added one
measurement I had not covered (`if 1=1 then say 2 & 1`, 4 spaces,
confirming `IF`'s `THEN` branch is 2 frames on its own, same as `WHEN`'s).
Restating the full, confirmed rule here in the form Task 11 can implement
from directly, with what was measured versus inferred:

* **Rule**: two extra spaces per currently-open block-stack frame at the
  point the failing clause was parsed (matching `block.rs`'s own `Control`
  stack almost exactly), applied on top of `Raised::report`'s existing
  one-space baseline (`"{:>6} *-* "`).
* **Measured directly** (mine, this round and last, each a `rexx-run`-vs-
  oracle transcript): top level, 0 extra spaces. `SELECT`/`WHEN`/`THEN` (no
  `OTHERWISE`), 6 (3 frames: `SELECT`, `WHEN`, and its own `THEN` branch as
  a *separate* frame from the `WHEN` itself). `SELECT`/`OTHERWISE`, 4 (2
  frames: `SELECT` and `OTHERWISE`'s own block -- `OTHERWISE` has no
  separate `THEN` frame, unlike `WHEN`). `IF`/`THEN` alone (no `ELSE`), 4
  (2 frames: `IF` and its own `THEN` branch). `IF`/`ELSE`, in the `ELSE`
  branch, 4 (2 frames, symmetric with `THEN`). Two full
  `SELECT`/`WHEN`/`THEN` levels nested inside each other, 12 (6 frames,
  confirmed additive with no discount for nesting).
* **Measured by the coordinator, not independently reproduced by me**:
  `if 1=1 then say 2 & 1`, 4 spaces (matches my own `IF`/`THEN` row above,
  so this is a confirmation of the same rule from a different clause
  shape, not a new data point).
* **Stated in the brief, not independently reproduced by me this round**:
  one/two/three enclosing `DO`s give 2/4/6 spaces (one frame per `DO`,
  consistent with the rule above, but I did not build a `DO`-based probe
  myself since `DO` is not implemented on this branch yet); `do i = 1 to
  'x'` gives 0, because the control expression fails before the loop's own
  frame is pushed.
* **Not measured, and worth checking rather than assuming**: whether a
  `WHEN CASE`'s own `THEN` branch counts the same as a plain `WHEN`'s (both
  go through the identical `WhenThen` control-stack frame in `block.rs`,
  so I expect yes, but did not run the oracle transcript for it); whether
  `EndThen`/`EndWhen`-style transient bookkeeping frames (which exist only
  during parsing, never at run time) need any special handling versus
  simply counting real, still-open blocks at the point of failure -- my
  own measurements are all consistent with "count only genuinely open
  blocks", but I have not stress-tested an `ELSE IF` chain's own count.
* Since this is exactly the *static* nesting depth at the point a clause
  was parsed, it looks computable from the AST alone (a per-instruction
  count precomputed once, or a backward walk through the flat instruction
  list counting enclosing block/branch constructs) with no runtime
  block-stack needed -- but this is an inference from the data above, not
  something I verified by building it.

**`INTERPRETER_STACK_BYTES`'s doc comment in `lib.rs`: fourth bullet
added**, this round's only change to that file (one doc comment, as
permitted). States the honest gap rather than a number: `step` now
recurses through `run_bounded` once per source nesting level of `IF`/
`SELECT`, cost per level unmeasured, bounded by program text rather than
data so not D19's unbounded case. The 2,000-level synthetic nested-`IF`
sanity check (≈70ms, no stack issue) is cited as reassurance, not as a
bytes-per-level figure.

## Files touched, round 2

* `rust/crates/rexx-exec/src/run.rs` -- the attribution fix and five new
  tests.
* `rust/crates/rexx-exec/src/lib.rs` -- one doc comment
  (`INTERPRETER_STACK_BYTES`'s fourth bullet), as explicitly permitted this
  round.
* This report.

`eval.rs` untouched this round (round 1's one-line change stands,
unmodified).

## Round 3: review findings ("spec compliance PASS, code quality PASS, no Criticals", two Important items)

Full review at `.superpowers/sdd/2026-07-30-phase-4a-executor/task-10-review.md`.
Mutation testing killed five mutants including the naive `If` fallthrough,
a `SELECT` exit landing on the next `WHEN`, and `run_bounded` swallowing an
unowned `Flow`; the indentation characterisation was checked against the
oracle including both rows I had marked inferred (`SELECT CASE`'s `THEN`
is six extra spaces, an `ELSE IF` chain is eight -- both confirmed, Task 11
can implement from the table as written). Two Important findings, both
fixed this round, `run.rs` only:

**1. A surviving mutant, and my own round-1 defect class again.** Passing
`None` instead of `source` into `run_bounded` left all 102 tests green --
regressing attribution for a raise inside a branch *body* to the enclosing
construct, exactly what round 1 fixed and exactly the shape round 1's
report claimed (too broadly) was closed. The five round-2 attribution
tests cover conditions, values, the case expression and `OTHERWISE`, but
`OTHERWISE` runs through the *outer* loop, so none of them exercised the
source-threading through `run_bounded` that heals a `THEN`/`WHEN` *body*.

Added two tests,
`a_raise_inside_a_matched_whens_body_is_attributed_to_its_own_clause` and
`a_raise_inside_an_ifs_then_body_is_attributed_to_its_own_clause`, both
checking `failure_site`'s line and text. **Confirmed the kill rather than
assuming it**: saved a copy of `run.rs`, changed both of `Select`'s and
`If`'s own `run_bounded` call sites to pass `None` in place of `source`,
re-ran `cargo test -q -p rexx-exec --lib`. Exactly these two tests went
red (both asserting line 1 -- the enclosing `SELECT`/`IF`'s own line --
where 3/2 respectively were expected), all 102 others stayed green,
confirming the two new tests are what defends this path and nothing else
already did. Restored from the saved copy, re-ran: 104/104 green again.

The habit the coordinator named -- "for each fix, ask which mutation the
new test kills" -- is now the standing question I apply before treating
any attribution/control-flow test as done, not only for this pair.

**2. `Flow::Goto`'s doc comment was stale and self-contradicting.** It
said a fragment's `labels` is always empty so "a fragment can never jump
-- `run_fragment`'s own `unreachable!` on this variant still holds", but
this task's own diff deleted that `unreachable!` and replaced
`run_fragment`'s loop with a `run_bounded` call specifically *because*
`IF`/`SELECT` can now jump inside a fragment with no label involved at
all -- `run_fragment`'s own new comment already said so, contradicting
`Flow::Goto`'s. Rewritten to state what is now true: labels still cannot
be jumped to inside a fragment (47.1 unchanged), but `IF`/`SELECT` can
appear and jump anywhere a `Code` body is stepped, a fragment's own
included, and `run_fragment` now runs through `run_bounded` for exactly
that reason.

**Structuring semicolons in the comments I added, fixed**: six instances
across this task's own lines (the module doc comment, the `If` arm's false
path, the `End` arm, `run_bounded`'s own doc comment, `eval_condition`'s
doc comment, `skip_else`'s doc comment, and one test's doc comment) --
each was a semicolon joining what are really two sentences, rewritten as
two sentences or with "otherwise"/"but" replacing the semicolon. Left two
occurrences alone, both inside doc comments on `raised_if_not_logical`/
`raised_select_no_when`: `"...must be exactly \"0\" or \"1\"; found
\"...\""` and `"...are false; OTHERWISE expected."` are verbatim
quotations of `interpreter/messages/rexxmsg.xml`'s own catalogue text --
the semicolon there is the oracle's, not mine to restructure, and
changing it would misquote the message. Did not touch any pre-existing
(round-1-or-earlier-branch) semicolon-in-comment the reviewer flagged as
drift rather than regression, per the explicit instruction not to mix an
unrelated sweep into this diff.

**Recorded, not fixed** (per the coordinator's own instruction --
scaffolding for a caller that does not exist yet): `self.failure_site` is
never cleared mid-run. It only matters once a condition trap can resume
execution after a raise and keep running (4b's `SIGNAL ON`), at which
point a *second* raised condition in the same run would see the *first*
one's stale, already-`Some` site and never resolve its own. 4a has no such
resumption, so nothing currently observes this. **For 4b**: whoever adds
`SIGNAL ON`'s resume path needs to clear `self.failure_site` at the point
a trapped condition's handler successfully resumes execution (or,
equivalently, wherever an `Outcome`'s taken `failure_site` currently getting
consumed by `execute()` assumes there is exactly one raise per run and
would need revisiting for a run that can raise, resume and raise again).

**Verification, this round**: `cargo test -q -p rexx-exec --lib`, 104
passed (102 + 2 new), 0 failed. `cargo test -q --workspace`: all green.
`cargo fmt -p rexx-exec --check`: clean. `cargo clippy -q -p rexx-exec
--all-targets -- -D warnings`: clean. Corpus re-run:
**still 12/26**, unchanged (no corpus program currently raises inside a
`WHEN`'s or `IF`'s branch *body*, only the two blocked-on-`DO` ones that
would, `select_when.rex`/`select_when_bodies.rex`). Files touched this
round: `run.rs` only, matching the permitted set exactly (`lib.rs`/
`eval.rs` untouched, both stand as committed in rounds 1-2).

# Task 10: `IF`, `SELECT`, `SELECT CASE`

Implementer notes and measurements, written as work proceeds.

## Reading phase

- Read `run.rs` in full (1452 lines): `Flow`, `step`, `step_in_temps_frame`,
  `run_activation`, `run_fragment`. `Flow::Goto(usize)` exists, currently
  `#[allow(dead_code)]`, unused. `step_in_temps_frame` is the only wrapper
  that calls `step`; `run_activation`, `run_fragment` and every test helper
  go through the wrapper, never `step` directly.
- Need to read: `ast.rs` (`If`, `Select`, `WhenCase`, `Instruction` shapes,
  the indices), `plan.rs` (how If/Select instructions get their target
  indices baked in during Phase 3), `eval.rs` (`eval_logical_list`,
  34.1/34.2/34.6 error sites), `error.rs` (`Raised`, `Raised::report`,
  indentation state).

## Baseline corpus measurement

Built `rexx-run` (`cargo build -q -p rexx-exec --bin rexx-run`) and ran every
program in `corpus/phase-4a.txt` through both `target/debug/rexx-run` and the
oracle (under the mandated `ulimit -v 1048576`), comparing stdout+stderr+rc.
Script kept at
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/corpus_check.sh`.

**Baseline: 9 pass, 17 fail.** Matches the brief's stated "Nine match the
oracle today on all three channels."

```
PASS: lang/arith_digits.rex
FAIL: lang/no_trailing_newline.rex
FAIL: lang/select_when.rex
FAIL: lang/stem_compound.rex
FAIL: lang/trace_output.rex
PASS: num/comparison.rex
FAIL: num/notation_thresholds.rex
FAIL: lang/do_loop_forms.rex
FAIL: lang/do_label.rex
FAIL: lang/leave_nested_outer.rex
FAIL: lang/iterate_from_select.rex
FAIL: lang/if_else_chain.rex
FAIL: lang/select_when_bodies.rex
FAIL: lang/select_when_absorption.rex
FAIL: lang/leave_iterate_variants.rex
PASS: lang/drop_stem_tail.rex
PASS: lang/stem_aliasing.rex
PASS: lang/exit_with_value.rex
PASS: lang/exit_no_value.rex
PASS: lang/number_identity.rex
PASS: lang/comparison_families.rex
PASS: lang/deep_nested_expr.rex
FAIL: lang/trace_results.rex
FAIL: lang/prefix_dotvar_logical_over_label.rex
FAIL: lang/comparison_operators_remaining.rex
FAIL: lang/trace_numeric_request.rex
```

Will re-run after implementation.

## AST fields, traced through the actual parser (block.rs), not guessed

Read `ast.rs` (InstructionKind::If/Then/Else/Select/When/WhenCase/Otherwise/End,
EndTarget, EndStyle) and `block.rs` (`flush_control`, `set_false_target`,
`set_then_exit`, `match_select_end`, `match_end`, the WHEN-registration arm,
`resolve_targets`) in full, and hand-traced two concrete programs through the
parser's own code to pin down what the jump fields actually contain, because
prose alone left a real ambiguity (below).

Traced `if n=1 then say "a" else say "b"` / `say "after"`:
`[0:If{false_target:Some(3)}, 1:Then, 2:Say(a), 3:Else{then_exit:Some(5)},
4:Say(b), 5:Say(after)]`. **`false_target` lands exactly ON the `Else`
instruction's own index, not past it.**

Traced `select / when a then X / when b then Y / otherwise Z / end`:
`When(a).false_target` = the index of `When(b)` itself (not past it);
`When(b).false_target` = the index of `Otherwise` itself; every `When`'s
`exit` = `end + 1` (`match_select_end`'s `fixWhen` port, same value for every
`When` of one `SELECT`, confirmed directly in the source).

### The arrival-path problem this trace exposes, and why a naive Goto-chase is wrong

Both traces show a **fallthrough-versus-jump ambiguity that plain
`Flow::Next`/`Flow::Goto` dispatch cannot resolve from `(instruction, pc)`
alone**: in the `If` trace, the true path (condition true, run `Say(a)`) and
the false path (`Goto(3)`) both land on **the same index**, `Else@3` --
one by `pc += 1` after `Say(a)`, the other by an explicit jump. A correct
implementation needs those two arrivals to behave oppositely (skip the
else-body on the true arrival, run it on the false arrival), and `step`
receives no information distinguishing which happened. The same problem
recurs for `SELECT`/`WHEN`: `select_when_bodies.rex` is written exactly to
catch the naive version of this (a wrong exit "would show up directly...
w2-a/w2-b would print twice").

Ruled out: extra `Interp`/`Activation` state (a per-activation block stack)
would resolve it trivially, but I can only write `run.rs`; `Activation`
(`activation.rs`) explicitly does *not* have one yet ("nothing reads it
yet... Task 11 is the first code that will actually walk a block").

**Resolution, confined entirely to `run.rs`: `If` and `Select` each resolve
their *entire* construct inside their own `step` arm, via a bounded
sub-loop over the winning branch's instruction range, and return one
`Flow::Goto` that skips straight past everything else (the other branch, the
remaining `WHEN`s, `OTHERWISE`, and the `END`) to the true resume point.**
This mirrors `run_fragment`'s existing shape in this same file (a local `pc`
loop over a bounded range, calling `step_in_temps_frame` per instruction,
propagating `Flow::Exit` immediately) rather than inventing a new pattern.
Concretely:

* `If`: evaluate `condition`. Chosen branch's instruction range is
  `[condition_index + 1, false_target_or_len)` for true (the `+1` starts at
  the `Then` marker, which just traces), or, for false,
  `[false_target, then_exit_or_len)` if `false_target` names an `Else`
  (else-body, `Else` included since it only traces), or nothing at all if
  there is no `Else`. Run the bounded sub-loop over that range, then return
  `Flow::Goto` to the true resume point (`then_exit_or_len` if there was an
  `Else`, else `false_target_or_len`).
* `Select`: **evaluates `case` at most once**, iterates its own
  `whens: Vec<usize>` directly (not by chasing `false_target`), tests each
  `When`/`WhenCase` in source order, and on the first match runs a bounded
  sub-loop over `[when_index + 1, false_target_or_len)`, then returns
  `Flow::Goto(exit_or_len)` -- skipping every remaining `WHEN`, `OTHERWISE`
  and the `END` in one jump. `false_target` on each `When`/`WhenCase` is
  therefore read only as "where THIS candidate's own body ends" for the
  bound, never chased as a runtime jump.
* No match, `otherwise` present: `Flow::Goto(otherwise_index)`, and the
  **outer** `run_activation` loop runs `Otherwise` and its body normally --
  no bounded sub-loop needed here, because `OTHERWISE` is always the last
  thing before `END`, so its natural fallthrough into `END` is already
  correct (no sibling to accidentally re-enter).
* No match, no `otherwise`: `Flow::Goto(end_index)`, landing **exactly on
  the `END` instruction** and letting the outer loop step it next. This is
  deliberate and is what makes 7.3's clause echo the `END`'s, not the
  `SELECT`'s (`run_activation`'s error path echoes whatever instruction was
  current when the failure escaped) -- confirmed against `EndStyle::Select`'s
  own doc comment: "Reaching this END at run time is error 7.3."

Consequence: `Then`, `Else`, `Otherwise` each become pure no-op/trace step
arms (`Ok(Flow::Next)`, like `Label`) -- they are only ever reached *inside*
a bounded sub-loop (`Then`, `Else`) or via the deliberate no-match `Goto`
(`Otherwise`), never via an ambiguous fallthrough, so "only traces" in the
`ast.rs` doc comment is literally true once `If`/`Select` do the jumping
themselves instead of leaving markers to guess how they were reached.
`End`'s own arm is where 7.3 actually raises, keyed on
`EndStyle::Select` vs `Otherwise`/`LabeledOtherwise` (no error) vs
`Do`/`LabeledDo`/`Loop` (Task 11's, fails loudly as not-yet-implemented).

Nested constructs (an `IF` inside a `WHEN`'s body, etc.) are safe under this
scheme without extra bookkeeping: a nested construct's own jump targets are
always inside its enclosing construct's range by the way `block.rs` assembles
them (an inner branch always closes before the outer one does), so the
bounded sub-loop just sees `step_in_temps_frame` return a `Flow` whose target
is within (or at the exact boundary of) its own range, same as `run_activation`
handles it one level up.

`step`'s only caller remains `step_in_temps_frame`: the bounded sub-loops
inside `If`/`Select` call `self.step_in_temps_frame(...)` recursively (never
`self.step` directly), the same relationship `run_fragment` already has to
`run_activation`.

## SELECT CASE's `==` comparison

Resolved for free by the design above: since `Select`'s own step evaluates
`case` itself, once, before testing any `WhenCase`, there is no need for a
`WhenCase` to look up its enclosing `SELECT` at runtime (I had briefly
considered a backward scan over the instruction list for this; the
"Select resolves everything in one step" design above makes that
unnecessary). Each `WhenCase.values` is compared against the cached `case`
value with `==`, matching on the first equal value (an OR of `==`, not an
AND).

## Open item to verify before implementing 34.1/34.2

`eval.rs` has `logical_value(text: &[u8]) -> Option<bool>` and
`eval_logical_list` (34.6, for `ExprKind::Logical` nodes specifically -- a
comma list). A single-expression condition is *not* wrapped in `Logical`
(confirmed: the spec's own wording is "Logical is the comma list in a
condition"), so `If`/`When`'s own `condition: Expr` needs its own 34.1/34.2
check only when its `kind` is not `ExprKind::Logical` -- reusing
`logical_value` on the evaluated result rather than re-deriving the 0/1 rule.
Need to check `logical_value`'s visibility from `eval.rs`; if it is private
to that file, reusing it from `run.rs` needs either a visibility bump (which
touches a file I am not supposed to write without asking) or a duplicated
check. Will resolve by inspection before writing code, and ask first if a
visibility change turns out to be needed.

## Checked in with the coordinator before implementing

Sent the design above (the arrival-path ambiguity, the bounded-sub-loop
resolution, and the `logical_value` visibility question). The coordinator
independently re-derived the same `block.rs:432` trace and confirmed the
ambiguity is real and structural, approved the bounded-sub-loop design, and
approved bumping `logical_value` to `pub(crate)` (done: one line, `eval.rs`,
staged separately below). Three constraints attached, addressed in the
implementation:

1. **Temps-frame granularity must survive**: the bounded sub-loop must call
   `step_in_temps_frame` per instruction, never open one frame around a
   whole branch. Done -- `run_bounded` is a `while` loop that calls
   `step_in_temps_frame` once per instruction, exactly like
   `run_activation`'s own loop one level up.
2. **An unrecognised `Flow` must propagate outward, not be handled
   locally**, because Task 11's `LEAVE`/`ITERATE` (`leave sel` on a
   labelled `SELECT` exits that `SELECT`) will need to unwind out of a
   nested Rust call the same way `Flow::Exit` already does.
   **What `run_bounded` does with a `Flow` it does not own**: a
   `Flow::Goto(target)` is "mine" only when `start <= target <= end`
   (`end` inclusive: a nested construct's own resume point landing exactly
   on my own boundary is normal completion, not an escape); every other
   case -- `Flow::Exit`, an out-of-range `Goto`, or *any future `Flow`
   variant Task 11 adds* -- falls to one `other => return Ok(other)` arm
   and is returned unchanged. This is deliberately a catch-all, but the
   kind that forwards rather than the kind that discards: a new variant
   never needs a matching arm added here to be handled correctly, only to
   be handled *at all* by whichever level does own it.
3. **The new recursion must not be silent.** `step` calling `step` (through
   `run_bounded`, for a nested `If`/`Select`) means Rust stack depth now
   grows with the source nesting depth of `IF`/`SELECT`, bounded by program
   text rather than by data, so it needs no D19-style counter. I could not
   cheaply bisect a precise bytes-per-level figure the way
   `records_the_stack_cost_of_one_eval_frame` did for `eval` (that recursion
   is driven by a single expression's term count, easy to dial up to
   hundreds of thousands from one generated program; this one is driven by
   *lexical* `IF`/`SELECT` nesting, which no real or corpus program pushes
   past single digits, so there is no natural knob to bisect against a
   4a-shaped program). Sanity check instead: a synthetically generated
   2,000-level-deep nested `IF` chain (`if 1=1 then if 1=1 then ... say
   "deep" ... else nop`, 4,001 lines) runs through `rexx-run` in ~70ms with
   no stack issue, an ample margin over anything a real program's `IF`/
   `SELECT` nesting will reach. **Honest gap**: I did not produce a
   bytes-per-level number, and did not add one to `INTERPRETER_STACK_BYTES`'s
   own doc comment in `lib.rs`, because that file is outside the permitted
   set stated to me (`run.rs`, one line of `eval.rs`, this report). I
   documented the recursion and its bound in `run.rs`'s own module doc
   comment and on `run_bounded` itself instead. **Flagging for the
   coordinator**: should I also add a fourth bullet to
   `INTERPRETER_STACK_BYTES`'s doc comment in `lib.rs` naming this
   recursion (mirroring `eval`/`Plan::note`/AST-drop), or is that out of
   scope for this task and better left as a note for whoever next touches
   that budget?

## The absorbed-`WHEN` case: a second finding, from the coordinator's own follow-up

The coordinator flagged that `Select.whens` only lists a `WHEN` whose
*immediate* enclosing block is that `SELECT` (`ast.rs`'s own doc comment),
so the absorbed-`WHEN` shape (`when 1 = 1 then` / `when 2 = 2 then n = 42`)
means my bounded sub-loop *will* execute a `When` instruction its own
`SELECT` never listed, and asked me to make sure that path is defined
rather than assumed.

Traced this one through `block.rs` too, the same way as the other two.
Concretely, for `n = 0 / select / when 1 = 1 then / when 2 = 2 then n = 42
/ otherwise / n = 99 / end / say n` (indices below are the real, 11-instruction
flat body, `n = 0` occupying index 0):

```
1  Select{whens:[2], otherwise:Some(7), end:Some(9)}
2  When{cond: 1=1, false_target:Some(5), exit:Some(10)}
3  Then
4  When{cond: 2=2, false_target:Some(7), exit:None}   <- absorbed, not in whens
5  Then
6  Assign(n=42)
7  Otherwise
8  Assign(n=99)
9  End{style:Otherwise}
10 Say(n)
```

The absorbed `When` at index 4 is added by `flush_control`'s `WhenThen`-
closing arm (the same generic "the arriving clause closes the pending
branch" path an ordinary single-statement consequence takes), *not* by the
`is_when` arm's `Select.whens.push(...)` -- because by the time that check
runs, `block.top()` is `EndWhen` (just pushed by the outer `When`'s own
closing), not `Select`, so the `frame.kind.is_select()` check fails and the
absorbed `When` is silently skipped. Its own `exit` is therefore never
touched by `match_select_end`'s loop and stays `None` permanently.

**Measured against the oracle: this whole program prints `0`, rc 0** --
*neither* `n = 42` nor `n = 99` runs, even though the absorbed `When`'s own
condition (`2 = 2`) is true. I built the plausible-wrong version mentally
before fixing it (the "watch it fail" step in spirit, since I'd already
derived the design and wanted a discriminating case rather than a redundant
one): if `When`/`WhenCase`'s own `step` arm evaluated its condition like an
independent clause (the natural thing to write if you did not know about
absorption), running When1's bounded body `[3, 5)` = `[Then, When2]` would
call `step_in_temps_frame` on `When2` too, which would test `2 = 2`, find it
true, run *its own* bounded body `[5, 7)` (`Then`, `Assign(n=42)`), and
`Goto(When2.exit.unwrap_or(len))` -- `exit` is `None`, so this lands past
the *entire program*, skipping `say n` outright. Either way (naively
running `n = 42`, or overrunning past `say n`), this diverges from the
oracle's `0`.

**Resolution: `When`/`WhenCase`'s own `step` arm is a pure no-op
(`Ok(Flow::Next)`), exactly like `Then`/`Else`/`Otherwise`.** All real
dispatch lives in `Select`'s own arm, which reads a `When`/`WhenCase` node
as *data* (pattern-matching its `condition`/`values`/`false_target`/`exit`
fields directly) rather than ever calling `step_in_temps_frame` on it for a
decision. A `When`/`WhenCase` instruction is therefore only ever "stepped"
in one of two situations, and in neither does it get to decide anything:
inside a bounded sub-loop as inert filler (the winning branch's own body
happens to contain it, as here), or -- in the case of a `WHEN` that
genuinely *is* one of its `SELECT`'s `whens` -- never at all, because
`Select`'s own arm reads it as data and only ever steps the instructions
*after* it. Confirmed with a dedicated unit test
(`an_absorbed_when_runs_neither_its_own_consequence_nor_otherwise`) and a
direct `rexx-run` transcript against the file (below); the corpus's own
`select_when_absorption.rex` also newly passes end to end.

## A third finding, found while manually verifying against `rexx-run`: nested failures were misattributed

While checking a raise inside a matched `WHEN`'s `THEN` branch against the
oracle by hand (not caught by any unit test, since unit tests check
`Raised.number`/`.sub`, never the clause echo), I found `run_activation`'s
error path recorded the *wrong* clause. Probe:

```rexx
select
  when 1 = 1 then
    say 1/0
  otherwise
    nop
end
```

Oracle: `     3 *-*       say 1/0` / `Error 42 running ... line 3: ...`.
Mine, before the fix: `     1 *-* select` / `Error 42 running ... line 1:
...` -- the *SELECT's own* clause, line 1, not the failing `say 1/0` on
line 3.

**Cause: `run_activation`'s failure-site recording only ever sees the
*outermost* instruction it was stepping.** Task 10 nests
`step_in_temps_frame` calls arbitrarily deep (through `If`/`Select`'s own
`run_bounded`), so when `say 1/0` raises several levels inside a `Select`'s
own `step` call, the `Err` propagates all the way out to `run_activation`'s
one call site, which only has the *original* `instruction` (the `SELECT`)
in hand -- it never sees the actual failing one. This is a real defect my
own design introduced, not a pre-existing gap, since (unlike
`run_fragment`'s fragment, which indexes a genuinely different source) an
`If`/`Select` at any nesting depth shares the *same* `program.source` as
everything around it, so there is no reason it has to be wrong.

**Fix: moved the failure-site resolution from `run_activation`'s own error
arm into `step_in_temps_frame` itself**, threading `source:
Option<&ProgramSource>` through `step`/`step_in_temps_frame`/`run_bounded`.
`step_in_temps_frame` now resolves and records the site itself, guarded by
`self.failure_site.is_none()` so the *innermost* call -- the first, and
only the first, to see the escaping `Err` -- is the one that wins; every
enclosing propagation leaves an already-`Some` site alone. `run_fragment`
passes `None` for `source`, preserving its own pre-existing, deliberate
choice not to resolve a site for an instruction inside an `INTERPRET`
fragment (a genuinely different source, and not this task's to change --
confirmed no existing test in `tests/spike.rs` asserts on that behavior
either way, so nothing else could have silently broken).

Re-measured after the fix, byte for byte against the oracle except for
indentation (next section):

```
oracle: "     3 *-*       say 1/0\n"
mine:   "     3 *-* say 1/0\n"
```

Line number and clause text are now both exactly right.

## Indentation: characterised, not implemented (`error.rs` is Task 12's)

The brief's fourth "silent wrong answer" item: the clause echo is indented
two spaces per block-nesting level, and `Raised::report` does not do this
yet. Task 10 is what first makes this reachable (no block instruction
existed before), so I characterised it against the oracle directly rather
than guessing, but I have **not** touched `error.rs` -- that file is Task
12's, and the brief says to ask first.

Measured (`{:>6} *-* ` prefix has exactly one space before the clause text
with no indentation; every row below is "how many *extra* spaces" beyond
that baseline, from a real `rexx-run`-vs-oracle transcript each time):

| Construct enclosing the failing clause | extra spaces | levels (÷2) |
|---|---|---|
| none (top level) | 0 | 0 |
| `SELECT` / `WHEN ... THEN` (no `DO`) | 6 | 3 |
| `SELECT` / `OTHERWISE` | 4 | 2 |
| `IF ... THEN` (no `ELSE`) | 4 | 2 |
| `IF ... THEN ... ELSE`, in the `ELSE` branch | 4 | 2 |
| `SELECT`/`WHEN`/`THEN` nested inside another `SELECT`/`WHEN`/`THEN` | 12 | 6 |

**The rule: two spaces per currently-open block-stack frame, counting
`block.rs`'s own `Control` stack almost exactly** -- `SELECT` is one frame,
`OTHERWISE` is one frame (so `SELECT`+`OTHERWISE` = 2 levels = 4 spaces),
but a `WHEN`'s own `THEN` branch is **two** frames, not one (`WhenThen` is
its own `Control` variant, separate from the `SELECT` frame it is nested
in -- `block.rs`'s own enum has it), so `SELECT`+`WHEN`+`THEN` = 3 levels =
6 spaces. `IF`'s `THEN`/`ELSE` branch is 2 levels on its own (the `IF`
itself plus the branch, `IfThen`/`Else` each being their own frame) even
with no enclosing block at all. Nesting is additive with no discount: two
full `SELECT`/`WHEN`/`THEN` levels stacked gives exactly 6+6=12 spaces, not
some collapsed or capped number. This is consistent with (not contradicting)
the brief's own DO figures (2/4/6 for one/two/three `DO`s, each contributing
exactly one frame) and with `do i = 1 to 'x'` contributing zero (its control
expression is evaluated, and fails, *before* the block's own frame would be
pushed, matching "the block is not open yet"). Since this is exactly the
static nesting depth at the point a clause was parsed, it looks computable
from the AST alone (a per-instruction nesting count, or a walk backward
through the flat list counting enclosing block/branch constructs) with no
runtime block-stack needed -- but implementing that is Task 12's call, not
mine.

## Implementation

`InstructionKind::If`/`Then`/`Else`/`Select`/`When`/`WhenCase`/`Otherwise`
now have `step` arms; `End` has one for `EndStyle::Select` (raises 7.3),
`Otherwise`/`LabeledOtherwise` (no-op), and `Do`/`LabeledDo`/`Loop` (fails
loudly, Task 11's). `Flow::Goto`'s `#[allow(dead_code)]` and its comment are
gone. New private items, all in `run.rs`:

* `run_bounded(code, start, end, source)` -- the bounded sub-loop, doc
  comment carries the full design argument above.
* `eval_condition(code, condition, raise)` -- the 34.1/34.2-vs-34.6 split:
  a comma list (`ExprKind::Logical`) is evaluated through `eval`'s own
  dispatch to `eval_logical_list`, which already validates every element
  and answers exactly `b"0"`/`b"1"`, so this reads that result back rather
  than re-checking it; a single expression gets its own check via
  `logical_value` (now `pub(crate)` in `eval.rs`) and the keyword-specific
  raiser on failure.
* `test_case_when(code, values, case_text)` -- `WhenCase`'s `==`: **not**
  routed through `eval_compare`/`Operator::StrictEqual` (which would have
  needed a second `eval.rs` visibility bump beyond the one already
  approved). The design doc's own strict-family rule ("no padding and the
  shorter string is less") reduces, for equality specifically (no "less" to
  fall back on), to exact byte-for-byte equality with no numeric awareness
  at all -- so this is a plain `Vec<u8>` `==`, needing no `rexx-num` call.
  Matches the measured example both ways (`select case '007'` vs `when 7`:
  no match; `select case 7` vs `when 7`: match).
* `skip_else(code, target)` -- if `target` names an `Else`, its own
  `then_exit` (defaulted); otherwise `target` unchanged. Shared by `If`'s
  true path to compute its resume point.
* `raised_if_not_logical`/`raised_when_not_logical` (34.1/34.2) and
  `raised_select_no_when` (7.3) -- built directly via `Raised{condition,
  number, sub, additional}` literals, matching this file's own existing
  convention (`raised_symbol_expected` etc.) rather than adding
  constructors to `error.rs`. Numbers/subs/substitution shapes cross-checked
  against `interpreter/messages/rexxmsg.xml` directly (`Error_Logical_value_if`
  = 34.1, one substitution; `Error_Logical_value_when` = 34.2, one
  substitution; `Error_When_expected_nootherwise` = 7.3, no substitutions).

`step`/`step_in_temps_frame`/`run_bounded` all gained an `index: usize`
parameter (a nested call from inside `run_bounded` is stepping an
instruction the activation's own `pc` is not pointing at, so `step` needs
its own position handed in to compute `index + 1` for a branch's start) and
a `source: Option<&ProgramSource>` parameter (the failure-site fix above).
`run_fragment` was rewritten to call `run_bounded(&code, 0, len, None)`
instead of its own hand-rolled loop with `Flow::Goto(_) =>
unreachable!("...cannot jump (47.1)")` -- that `unreachable!` stopped being
true the moment `IF`/`SELECT` could appear and jump *inside* an `INTERPRET`
fragment with no label involved at all, which is a real, if currently
untested, latent bug this task would otherwise have introduced. Every
jump such a construct computes stays within `[0, len)` of the fragment's
own body by construction (`resolve_targets` clamps everything else to
`None`, defaulting to the body's own length), so `run_bounded` always
"owns" it. The test helper `run_source` was likewise rewritten to go
through `run_bounded(&code, 0, len, Some(&program.source))` instead of a
`for` loop that could not follow a `Goto` at all (its own `unreachable!`
comment, "no test program in this module branches", stopped being true the
moment this task added tests that do).

`step`'s only non-test caller remains `step_in_temps_frame`: verified by
inspection (`grep -n 'self\.step('` / `\.step(` in `run.rs`) that every call
site is `step_in_temps_frame` itself, `step_in_temps_frame`'s own callers
(`run_activation`, `run_bounded`, the test helper), or the doctest's own
unrelated miniature `step` method (a different type entirely, `struct
Interp` inside the doctest text, not this crate's).

## Tests

18 new `#[test]` functions in `run.rs`'s existing `mod tests`, covering:

* `if_then_else_runs_exactly_one_branch` / `if_else_if_chain_takes_exactly_one_link`
  -- the discriminating IF/ELSE shape (true and false paths have different
  side effects; a 4-way `ELSE IF` chain run once per branch, `n = 1..4`).
* `select_when_wrong_exit_would_land_in_a_later_whens_multi_instruction_body`
  -- the discriminating `SELECT`/`WHEN` shape from the brief, rebuilt
  *without* `DO` (Task 11's): each `WHEN`'s consequence is a **nested
  `SELECT`** spanning several of its own flat instructions, so a wrong exit
  landing even one instruction into a later `WHEN`'s span is visible. Run
  for `n = 1..4`.
* `an_absorbed_when_runs_neither_its_own_consequence_nor_otherwise` /
  `when_absorbing_a_when_parses_and_runs_at_rc_0` -- the absorbed-`WHEN`
  case, both the observable-output shape and the accepted-at-rc-0 shape the
  brief names directly.
* `select_case_compares_with_strict_equality_not_numeric_equality` --
  `'007'` vs `7`.
* `whencase_comma_is_an_or_of_equals_the_opposite_of_a_plain_whens_and` --
  the central `WhenCase` rule, both directions in one test (`select case 2`
  / `when 1, 2` hits; plain `select` / `when 1, 2` is 34.6).
* `if_condition_that_is_a_comma_list_raises_34_6_not_34_1` -- `if 'x', 1
  then` is 34.6, not 34.1 (built the plausible-wrong version mentally
  first: a version that re-checks `eval_logical_list`'s already-validated
  `b"0"`/`b"1"` result against `logical_value` again would still pass this
  *specific* assertion by coincidence, since re-checking a valid `0`/`1`
  never fails -- what actually discriminates the two implementations is
  `eval_condition`'s own branch on `ExprKind::Logical`, which the corpus's
  `if 'x', 1` case cannot exercise differently from a hand-written re-check;
  I did not find a test that discriminates "check via the `ExprKind::Logical`
  guard" from "re-run `logical_value` on an already-valid result" for the
  *success* path, because both give the identical answer whenever the list
  is actually valid -- only a bad list (which already takes the correct
  34.6 path either way) or a subtly-wrong AND/OR mixup would show it, and
  the `WhenCase`-vs-plain-`WHEN` comma test above is exactly that mixup
  case).
* `select_with_no_when_true_and_no_otherwise_raises_7_3` /
  `select_with_no_when_true_and_an_otherwise_runs_it_without_error`.
* `if_condition_that_is_not_0_or_1_raises_34_1` /
  `when_condition_that_is_not_0_or_1_raises_34_2`.
* `a_labelled_select_with_otherwise_runs_normally`,
  `a_select_nested_inside_an_ifs_then_branch_is_fully_resolved_before_resuming`,
  `if_then_with_no_else_falls_through_on_false`,
  `a_true_comma_list_condition_is_an_and_of_its_parts`,
  `select_when_runs_exactly_the_first_matching_when`,
  `select_case_evaluates_its_own_expression_and_runs_exactly_the_matching_when`.

`cargo test -q -p rexx-exec --lib`: **97 passed, 0 failed** (79 pre-existing
+ 18 new). `cargo test -q -p rexx-exec` (whole package, including the
in-crate corpus differential harness): all green.

Two requirements from the brief that a `#[cfg(test)] mod tests` unit test
genuinely cannot check, verified instead by hand against `rexx-run` and the
oracle directly (transcripts above): the 7.3 clause-echo-is-the-END's-not-
the-SELECT's requirement (unit tests only see `Raised.number`/`.sub`, never
`Raised::report`'s output, since `failure_site` is `run_activation`-only
state a bounded-sub-loop test helper never populates the way a real file
run does) -- confirmed byte for byte including the line number:

```
     3 *-* end
Error 7 running /abs/path/p1.rex line 3:  WHEN or OTHERWISE expected.
Error 7.3:  All WHEN expressions of SELECT are false; OTHERWISE expected.
rc=249
```

and the "several instructions long" `SELECT`/`WHEN` discriminating shape,
which needed the nested-`SELECT`-as-consequence trick above precisely
because `DO` (the shape the brief's own prose assumes) is not implemented
yet.

## Corpus

`corpus/phase-4a.txt`, before and after, via `rexx-run` against the oracle
(script: `.../scratchpad/corpus_check.sh`) and cross-checked against the
in-crate differential report test (`cargo test -q -p rexx-exec`, which
independently reports the identical count):

**Before: 9/26. After: 12/26.** Newly passing: `lang/no_trailing_newline.rex`
(a bare `IF`/`ELSE`, no `DO`), `lang/select_when_absorption.rex` (the
absorbed-`WHEN` case), `lang/comparison_operators_remaining.rex` (a
labelled `SELECT` with `OTHERWISE`, `EndStyle::LabeledOtherwise`). The
remaining 14 failures are all `DO` (10, Task 11's: `select_when.rex`,
`stem_compound.rex`, `notation_thresholds.rex`, `do_loop_forms.rex`,
`do_label.rex`, `leave_nested_outer.rex`, `iterate_from_select.rex`,
`if_else_chain.rex`, `select_when_bodies.rex`, `leave_iterate_variants.rex`)
or `TRACE` (4, Task 13's: `trace_output.rex`, `trace_results.rex`,
`prefix_dotvar_logical_over_label.rex`, `trace_numeric_request.rex`) --
matching the brief's own "thirteen of the seventeen failures are blocked on
IF, SELECT or DO... do not expect all thirteen" (one of those thirteen,
`select_when_absorption.rex`, needed neither `DO` nor `TRACE` and now
passes; the rest are genuinely `DO`-blocked, confirmed by reading each one).

## Verification

* `cargo test -q -p rexx-exec` -- all green (97 lib tests + the in-crate
  corpus report + doctests + integration tests).
* `cargo fmt -p rexx-exec --check` -- clean (exit 0). Note: the bare
  `rustfmt --check crates/rexx-exec/src/run.rs` the brief names fails
  outright with a *parse* error ("let chains are only allowed in Rust 2024
  or later") unless invoked with `--edition 2024` explicitly -- the crate's
  own edition (workspace `Cargo.toml`, `edition = "2024"`) is not picked up
  by the bare binary the way `cargo fmt` picks it up automatically. Used
  `rustfmt --edition 2024 <path>` throughout instead, then confirmed with
  `cargo fmt -p rexx-exec --check` as the authoritative gate. Separately
  confirmed (via `git stash` and re-running) that bare `rustfmt --check` on
  `eval.rs` *also* reports spurious import-order diffs on code I never
  touched, for the identical reason -- a pre-existing environment quirk,
  not something this task introduced or should "fix" by reordering
  unrelated imports.
* `cargo clippy -q -p rexx-exec --all-targets -- -D warnings` -- clean.
* `git diff --stat`: `eval.rs` 1 line changed (the approved visibility
  bump, verified via `git diff` to be exactly and only that line); `run.rs`
  867 insertions / 89 deletions.
* Oracle cross-checks beyond the corpus and unit tests, all via `rexx-run`
  under the mandated `ulimit -v 1048576`: the 7.3 echo transcript above,
  the nested-failure line-attribution fix (before/after), the indentation
  table's six measurements, and the absorbed-`WHEN` transcript. All scratch
  probe files under
  `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probes/`.

## Files touched

* `rust/crates/rexx-exec/src/run.rs` -- this task's own file.
* `rust/crates/rexx-exec/src/eval.rs` -- one line, `logical_value` to
  `pub(crate)`, explicitly approved.
* This report.

## Open items for the coordinator (not blocking, flagging as asked)

1. `INTERPRETER_STACK_BYTES`'s doc comment in `lib.rs` -- add a fourth
   consumer (this task's `step`-inside-`step` recursion) or leave it for
   whoever next touches that budget? See "the new recursion must not be
   silent" above for what I found and did not find.
2. Indentation (previous section) is characterised and written up in full,
   not implemented -- `error.rs` is Task 12's file, and the brief says to
   ask first before touching it. Every corpus program whose failure sits
   inside an `IF`/`SELECT` body will diverge on stderr indentation until
   that lands (none currently do, since the only two `SELECT`/`WHEN`
   corpus programs that raise at all are still blocked on `DO` for
   unrelated reasons).
