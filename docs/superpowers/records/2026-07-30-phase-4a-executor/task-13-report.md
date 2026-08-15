STATUS: DONE

# Task 13 report: TRACE

Base: b8b8f16c. Corpus before: 22 of 26 (all four remaining failures TRACE).

## Step 0: reading the inherited pieces before designing anything

- `eval`/`eval_node` split confirmed in `rust/crates/rexx-exec/src/eval.rs` (not yet re-read in
  full at time of writing this line; will cite the exact split point once read).
- `static_indent`/`indent_in_range` (`run.rs:2190`-`2306`) and `pop_search_frame` (`run.rs:1128`)
  exist from Task 11, as the dispatch said. `static_indent` returns **spaces**, not "levels"
  (already multiplied), confirmed by its own doc comment's worked numbers (2 per DO/LOOP, 4 per
  matched IF branch, 2 for a SELECT's own scan, 6 for a matched WHEN's THEN, 4 for OTHERWISE).
- `Interp.trace: Vec<u8>` (`lib.rs:569`) is the sink, separate from `out`, exactly as the dispatch
  said. Nothing writes to it yet outside the placeholder note; `execute`'s error path writes to
  `interp.trace` too (`lib.rs:920`, `interp.trace.extend_from_slice(&raised.report(&site))`) --
  so the error report and TRACE output already interleave on the SAME sink in file order, which
  matters for byte-for-byte comparison against the oracle's own stderr.
- `Raised::report`'s clause-echo line (`error.rs:270`-`300`) is
  `format!("{:>6} *-* ", site.line)` + `site.indent` spaces + `site.text` + `\n` -- this is
  the exact byte shape of the oracle's `trace_prefix_table[TRACE_PREFIX_CLAUSE]` line
  (`"*-*"`), confirmed against `RexxActivation.cpp`'s `TRACE_OVERHEAD` formula (`LINENUMBER=6,
  PREFIX_OFFSET=7, PREFIX_LENGTH=3, INDENT_SPACING=2`): 6-wide right-aligned line number, one
  space, 3-char prefix, one space, then indent spaces, then the clause text, no trailing quote
  (the clause line is the one prefix with no quoted value). TRACE's own `*-*` line reuses this
  exact function; no second formatter for it.
- `Settings` (`rexx-num`, used by `Activation`) currently has **no trace-mode field at all**.
  `TRACE R`/`TRACE I`/`TRACE OFF` need one; `InstructionKind::Trace(Trace)` is not implemented
  (`step` falls to the loud path, confirmed by the corpus run below).

## Step 0.5: corpus baseline, unpiped

```
$ cargo test -p rexx-exec --test corpus
22 of 26 matching -- REPORT MODE, NOT THE GATE
mismatches (4):
  [TRACE] lang/trace_output.rex
  [TRACE] lang/trace_results.rex
  [TRACE] lang/prefix_dotvar_logical_over_label.rex
  [TRACE] lang/trace_numeric_request.rex
```
All four are `TRACE`, confirming the dispatch's own claim. All four programs read
(`rust/corpus/lang/{trace_output,trace_results,prefix_dotvar_logical_over_label,trace_numeric_request}.rex`).

## Step 1: the oracle's prefix table and format constants, read from source

`RexxActivation.cpp:3565`-`3588`, `trace_prefix_table`, all 19 entries (index = `TracePrefix`
enum order at `RexxActivation.hpp:92`-`110`):
`*-*` clause, `+++` error, `>>>` result, `>.>` dummy, `>V>` variable, `>E>` dot-variable, `>L>`
literal, `>F>` function, `>P>` prefix, `>O>` operator, `>C>` compound, `>M>` message, `>A>`
argument, `>=>` assignment, `>I>` invocation, `>N>` namespace, `>K>` keyword, `>R>` alias,
`<I<` invocation-exit.

Format constants (`RexxActivation.cpp:3596`-`3611`): `LINENUMBER=6`, `PREFIX_OFFSET=7`,
`PREFIX_LENGTH=3`, `INDENT_SPACING=2`, `QUOTES_OVERHEAD=2`,
`TRACE_OVERHEAD = 6+1+3+1+2+2 = 15`, `INSTRUCTION_OVERHEAD = 6+1+3+1 = 11`.

Gating (`TraceSetting.cpp:52`-`54`, `RexxActivation.hpp:339`-`369`), read from source, not
guessed:
- `TRACE R` sets `{traceAll, traceLabels, traceResults, traceCommands}`.
- `TRACE I` sets `{traceAll, traceLabels, traceResults, traceIntermediates, traceCommands}`.
- clause echo (`*-*`) fires whenever `tracingAll()` (`tracingInstructions()`) -- true for both R
  and I.
- `>>>` (RESULT) fires whenever `tracingResults()` -- true for both R and I. This is the
  assignment/keyword-instruction's own **computed value**, traced once per instruction, not an
  intermediate.
- `>L>`, `>V>`, `>O>`, `>P>`, `>C>` (as `traceIntermediate`/`traceVariable`/`traceOperator`/
  `tracePrefix`/`traceCompoundName`) fire only under `settings.intermediateTrace` -- **TRACE I
  only, never TRACE R**.
- `>=>` (ASSIGNMENT) fires only under `intermediateTrace` too (`ExpressionVariable.cpp:299`,
  `traceAssignment`, called from a variable's own `assign`) -- TRACE I only.
- `>K>` (KEYWORD) fires under `tracingResults()` (both R and I), from `traceKeywordResult`,
  called by `DO`'s control-clause components (`TO`/`BY`/`FOR`/`WHILE`/`UNTIL`/`COUNTER`,
  `DoBlockComponents.cpp`/`DoBlock.cpp`) and `SELECT CASE`'s `CASE` (`SelectInstruction.cpp:372`)
  -- both 4a-owned constructs, matching the dispatch's reachable-set claim.

## Step 2: capturing real oracle bytes (`cat -A` / Python `repr`, under the mandated ulimit)

### `trace_output.rex` (`TRACE I`), full stderr, byte for byte:

```
     2 *-* x = 1 + 1
       >L>   "1"
       >L>   "1"
       >O>   "+" => "2"
       >>>   "2"
       >=>   X <= "2"
     3 *-* y = x * 3
       >V>   X => "2"
       >L>   "3"
       >O>   "*" => "6"
       >>>   "6"
       >=>   Y <= "6"
     4 *-* if y > 5 
       >V>   Y => "6"
       >L>   "5"
       >O>   ">" => "1"
       >>>   "1"
     4 *-*   then
     4 *-*     say "big"
       >L>       "big"
       >>>       "big"
     5 *-* trace off
```
stdout: `big\ndone 6\n`. rc 0.

Decoded byte layout (`>L>` line, indent 0): `'       >L>   "1"\n'` -- 7 blanks, `>L>`, 3 blanks,
`"1"`, `\n`. Matches `TRACE_OVERHEAD=15` with the quote at offset `13 = TRACE_OVERHEAD-2`.
`>=>` line: `'       >=>   X <= "2"\n'` -- tag `X` **unquoted** (`traceAssignment`'s own call passes
`quoteTag=false`), marker `" <= "` (`ASSIGNMENT_MARKER`), matching `traceTaggedValue`'s read.
`>V>` line: tag unquoted too (`traceVariable` also passes `quoteTag=false`), marker `" => "`
(`VALUE_MARKER`).
`>O>` line: `'       >O>   "+" => "2"\n'` -- tag **quoted** (`traceOperatorValue` always quotes),
marker `" => "`.

**A genuine defect found in the inherited `static_indent`, not a re-derivation of the same
quantity: the THEN/ELSE marker clause's own indent is wrong, and OTHERWISE's is unreachable.**
Measured above: `if y > 5` is at indent 0, its own `then` marker clause is at indent **2**, and
`say "big"` (the THEN's *body*, one instruction later) is at indent **4**. Read against
`ThenInstruction.cpp:130`-`137`: `execute` calls `context->indent()`, THEN traces its own clause,
THEN calls `context->indent()` again for whatever follows -- so the marker clause sits at
`enclosing + 2` and the body sits at `enclosing + 4`, confirmed identical in `ElseInstruction.cpp`
(`context->indent(); traceInstruction(this); context->indent();`, same shape).

`static_indent`/`indent_in_range` (`run.rs:2190`+) currently give **the THEN marker itself indent
4** (it falls into the `if target >= then_start && target < false_target: return 4 +
indent_in_range(...)` branch, and inside that recursive call `target == then_start == pc`
returns 0 immediately, total 4) -- wrong by +2, though harmless for the *body*, which still gets
the right total (4) because the marker occupies only the *first* position of that same range and
everything after it still resolves correctly. The **ELSE marker's own clause is worse: it is
skipped by the `If` arm entirely** (the `target > false_target` check is strict, so `target ==
false_target` falls through to `pc = else_end; continue`, and the outer loop's own `if pc ==
target return 0` then fires when `pc` walks *back* to... no, `pc` is *advanced past* target, so
the ELSE marker's own index is never revisited and the search returns the fallback default; this
needs a differential probe to pin the exact wrong number rather than asserted from reading, see
below) -- and **`WHEN`'s own THEN marker and `OTHERWISE`'s own clause have the identical two
defects**, one via the shared `InstructionKind::Then` node (`instruction.rs:2586`+, `if_instruction`
is reused for both `PendingThen::If` and `PendingThen::When`, confirmed by reading the parser, not
assumed) and one because `indent_in_range`'s `SELECT` arm has **no case at all** for
`target == *otherwise_index` (the `target > *otherwise_index` check is strict, same shape as the
ELSE bug) and falls through to `unreachable!("a resolved SELECT's own range holds only its WHENs
and OTHERWISE")` -- **a real panic risk under TRACE**, not merely a wrong number, since
OTHERWISE's own clause is unconditionally traced (`OtherwiseInstruction.cpp:64`,
`traceInstruction(this)`) whenever `TRACE R`/`I` is on and a `SELECT` reaches its `OTHERWISE`.

This was invisible to Task 10/11 because a marker clause (`THEN`/`ELSE`/`OTHERWISE`) carries no
expression and can never itself raise a condition, so `record_failure_site` never resolves a
`FailureSite` at one of these indices -- the gap exists only for TRACE's own clause echo, which
echoes literally every stepped instruction, markers included.

**This is a defect in inherited shared code, not a second computation of the same quantity, and
I flagged it to the coordinator before touching `static_indent` -- see the message log below.**

## Step 3: message sent to `main`, reply received -- proceed, in this task, four conditions

Coordinator independently re-ran the `else`/`otherwise` probes and confirmed both, plus the exact
panic site (`run.rs:2268`, the `unreachable!` inside `static_indent`'s `SELECT` arm). Ruling:
**land the fix in Task 13** (permitted file, and it is what makes the shipped constructs' clause
echo correct), with four conditions:
1. Separate, bisectable commit landed *before* the trace feature commit.
2. All four get tests; the `OTHERWISE` one must be phrased as "this aborted the process today,"
   not merely "this indent was wrong."
3. Re-run Task 11's twelve-shape indentation table after the change and say so explicitly, since
   narrowing `>=`/`>` and inserting equality cases ahead of them is exactly the shape that hides
   an off-by-one regression.
4. Reconsider the `unreachable!` itself, not just route around it -- this crate's rule (`error.rs`'s
   catalogue-miss path, cited again at `run.rs:536`) is that the diagnostic path never turns a
   reportable gap into a crash, and `static_indent` now feeds trace as well as the error report.

Also asked to probe two shapes outside the four measured (a marker nested two deep; a `THEN` whose
body is a `DO` block), which was already in progress -- see below, done before writing the fix.

## Step 3.5: two more oracle probes, done before writing any fix (a rule from four points plus the
C++ is still a rule fitted to its own sample until something outside the sample confirms it)

### `t13_then_do_block.rex` -- a THEN whose body is a DO block (`trace r`, full stderr):
```
     2 *-* if 1 = 1 
       >>>   "1"
     2 *-*   then
     2 *-*     do
     3 *-*       say 'x'
       >>>         "x"
     4 *-*     end
     6 *-* say 'end'
       >>>   "end"
```
`if`=0, `then`=2, `do` (the THEN's body, itself a DO's own opening clause)=4, `say 'x'` (inside the
DO) = 6 (4 + DO's own +2, unaffected by this fix), `end` (the DO's own End) = 4 (same level as its
own DO, matching the existing model, also unaffected). The marker-is-body-minus-2 rule holds when
the body is itself a block opener, not only a plain clause.

### `t13_nested_two_deep.rex` -- a marker nested two deep (`if` inside `if` inside `do`):
```
     2 *-* do i = 1 to 1
       >K>   "TO" => "1"
     3 *-*   if 1 = 1 
       >>>     "1"
     3 *-*     then
     4 *-*       if 2 = 2 
       >>>         "1"
     4 *-*         then
     4 *-*           say 'x'
       >>>             "x"
     6 *-* end
```
Outer `if` (the DO's body) = 2, its own `then` = 4 (2+2), inner `if` (the outer THEN's body) = 6
(2+4), the inner `if`'s own `then` = 8 (6+2), and `say 'x'` (the inner THEN's body) = 10 (6+4).
The rule holds at every depth with no adjustment: a marker is always exactly its own enclosing
construct's body-indent minus 2, all the way down. Also confirms the earlier `>K>` reading:
`TO => "1"` fires for `do i = 1 to 1` under `TRACE R` alone (`tracingResults()`, not gated on
intermediates), matching the source read.

## Step 4: the fix, `run.rs`'s `indent_in_range`

Four equality cases added, each *before* the existing (now strict-`>`) body-range check, so no
body-content path Task 11 already tested changes at all -- confirmed by condition 3, below.

* `If` arm: `target == then_start` returns `2` (was falling into the body's `4 + recurse`, giving
  `4`). The body check narrows from `target >= then_start` to `target > then_start`; the recursive
  call underneath is untouched (`then_start`, not `then_start + 1` -- the marker's own index is
  simply never revisited by that inner walk once the equality case returns early, exactly like the
  existing `when_index == target` case one arm over).
* `Else` arm (inside the `Some(InstructionKind::Else { .. })` match on `false_target`):
  `target == false_target` returns `2` (was silently falling through to `pc = else_end; continue`,
  which skips the marker's own index in the outer walk entirely and can never resolve it -- this is
  the "wrong number" bug, not the panic; the panic is `OTHERWISE`'s, next).
* `whens` loop (`SELECT`): `target == body_start` (the `WHEN`'s own `THEN` marker, sharing
  `InstructionKind::Then` with `If`'s) returns `4` (was matching the loop's `target >= body_start`
  and returning `6`, the body's value). Narrowed to `target > body_start` for the body case.
* `otherwise` (`SELECT`): `target == *otherwise_index` returns `2`. This is the one that used to
  reach `unreachable!("a resolved SELECT's own range holds only its WHENs and OTHERWISE")` --
  **a live panic**, confirmed by the coordinator's own independent re-run at `run.rs:2268`, not
  merely a wrong number; the dedicated test for it (below) is phrased as "this aborted the process
  today," per condition 2.

Condition 4, the `unreachable!` itself: **verified it was a live panic before touching it**, not
inferred -- added a temporary `#[test]` calling `static_indent` directly on
`select\nwhen 1 = 0 then nop\notherwise\nsay 'y'\nend`'s own `otherwise_index`, ran it under
`RUST_BACKTRACE=1` against the *unfixed* tree, and got:
```
thread '...' panicked at crates/rexx-exec/src/run.rs:2323:21:
internal error: entered unreachable code: a resolved SELECT's own range holds only its WHENs and OTHERWISE
```
(exactly the message and, at the pre-fix line count, the same site the coordinator's own
independent re-run cited as `run.rs:2268` -- the offset is my earlier If/Else edit adding lines
before it, not a different panic.) Removed the temporary test before writing the permanent one.
Disposition: turned it into a documented fallback (`return 0`, "the enclosing level, nothing
further to add") rather than a second `unreachable!`, per this crate's existing rule that the
diagnostic path never turns a formatting gap into a crash (`error.rs`'s message-catalogue miss,
cited again at `run.rs:536`) -- the arm had already proven itself reachable once, so asserting
its unreachability a second time is exactly the claim that was just falsified. The doc comment
says what a reader should conclude if they see it: indentation shallow by exactly one `SELECT`
construct's own contribution means some `SELECT`-shaped clause position isn't one of the five
now-enumerated cases.

## Step 5: applied, tested, condition 3 re-run, committed separately

* All four equality cases added (`run.rs`'s `If`/`Select` arms of `indent_in_range`), each
  *before* the existing body-range check, narrowed from `>=`/an unconditional strict `>` where
  needed -- no recursive call's own arguments changed.
* **Condition 3, Task 11's fourteen-shape table, re-run**: `the_corrected_28x_indent_rule_matches_
  all_fourteen_probed_shapes` passes unchanged. None of its fourteen shapes targets a marker's own
  index (every one raises inside a body clause), so this is expected, not a coincidence -- stated
  per the coordinator's condition 3 rather than left implicit.
* One new test, `a_then_else_when_then_or_otherwise_markers_own_clause_indents_half_its_bodys`,
  covering all four fixes by calling `static_indent` directly (a marker clause cannot raise, so
  there is no `FailureSite` path to drive it through) -- the `OTHERWISE` case's own comment states
  it aborted the process before this fix, per condition 2, not merely "this indent was wrong".
  Two small helpers (`if_then_start`, `instructions_of`) shared across the sub-cases.
* `cargo test -p rexx-exec`: 96 passed in `run::tests` alone (0 failed), 11 in the crate's other
  integration test, doctests green. `cargo test --workspace`: every `test result: ok`, 0 failed,
  grepped rather than eyeballed. `cargo clippy -p rexx-exec --all-targets -- -D warnings`: clean.
  `cargo fmt --check -p rexx-exec`: clean (ran `rustfmt crates/rexx-exec/src/run.rs` first, per the
  brief's own instruction to use `rustfmt <path>` rather than `cargo fmt -p rexx-exec`).
* Corpus unchanged at 22/26 (expected: this commit fixes indentation, it does not implement
  `TRACE` itself).
* **Committed separately, before the trace feature, per condition 1**: `4ec4884d`,
  "Fix static_indent's marker-clause indentation, found while building TRACE".

## Step 5.5: the sibling `unreachable!`, flagged by the coordinator after reviewing `4ec4884d`

Coordinator verified `4ec4884d` clean (784 tests, clippy/fmt clean, both the new test and Task
11's fourteen-shape table pass, the `OTHERWISE` panic genuinely gone) and then pointed at the
`whens`-loop's own `other => unreachable!("a SELECT's whens holds only When/WhenCase, not
{other:?}")`, one arm above the one just fixed: same function, same diagnostic path, and its
provenance ("Phase 3's own invariant") is no better than the one that just turned out to have
three ways in -- the absorbed-`WHEN` case (this module's own
`when_absorbing_a_when_parses_and_runs_at_rc_0` test) already means a `When` instruction can
execute while its enclosing `SELECT`'s `whens` does not list it, which is the identical shape of
surprise. Asked for the same treatment (a documented fallback), folded into the trace commit
rather than amending `4ec4884d` -- a measured, reproduced defect and a hardening change are not
the same finding, and mixing them would overstate what was actually found for the fixed one.

Done: replaced the `unreachable!` with `_ => continue` (skip this `whens` entry rather than
computing bounds for it), with a comment naming the absorbed-`WHEN` precedent and what a reader
should conclude if they ever see it (a `rexx-parse` defect, not a formatting one -- this function
cannot correct it, only avoid crashing on it). `run_bounded`'s own `unreachable!` (`run.rs:2768`,
its is a control-flow invariant this crate owns end to end, not a diagnostic formatter) is
explicitly left alone, per the coordinator's own scoping of the rule.

Re-verified: `cargo test -p rexx-exec --lib` 158 passed, 0 failed (unchanged count -- this is a
fallback for a case nothing exercises, not new behaviour on any tested path).
`cargo clippy -p rexx-exec --all-targets -- -D warnings` clean. `rustfmt --edition 2024
crates/rexx-exec/src/run.rs` (bare `rustfmt <path>` fails on this file's `let`-chains under the
default edition, a real trap worth naming: it aborts with a parse error and writes nothing, so it
is safe but silently does not format -- `cargo fmt --check` afterward is what caught it, not the
rustfmt invocation itself). `cargo fmt --check -p rexx-exec` clean.

**Held uncommitted, to land inside the `TRACE` feature commit rather than a third small commit**,
per the coordinator's own instruction that this hardening change and the measured `4ec4884d` fix
are different findings and should not be mixed.

## Step 6: a second architecture question, found before writing `trace.rs` -- per-iteration re-echo

Read `run_bounded`'s own loop (`run.rs:1189`-`1206`): `step_in_temps_frame` is called once per
flat instruction *position* it walks, exactly the single insertion point `eval`'s own split has
for value lines -- this is where I planned to hook the `*-*` clause echo, gated on
`self.trace_mode.all`.

**Measured against the oracle first, and it broke that plan.** `trace r` over `do while n < 2 /
n = n + 1 / end` (full stderr, `cat -A`):
```
     2 *-* n = 0
       >>>   "0"
     3 *-* do while n < 2
       >K>     "WHILE" => "1"
     4 *-*   n = n + 1
       >>>     "1"
     5 *-* end
     3 *-* do while n < 2
       >K>     "WHILE" => "1"
     4 *-*   n = n + 1
       >>>     "2"
     5 *-* end
     3 *-* do while n < 2
       >K>     "WHILE" => "0"
     6 *-* say n
       >>>   "2"
```
The `DO`'s own clause (`do while n < 2`) is echoed **again on every iteration**, and so is `end`
-- three times for a loop that runs twice. `do i = 1 to 2` behaves the same way (checked
separately, same shape). This is the oracle's own execution model, not an accident: a `DO`/`LOOP`
instruction's C++ `execute()` genuinely runs once per iteration (it is what tests the condition
and decides whether to continue), so its own `traceInstruction(this)` call fires every time.

**This crate's `run_loop`/`run_repeating` do not have that shape, deliberately.** Task 10/11's
whole design resolves an entire `DO`/`LOOP` construct inside **one** `step` call -- `Flow::Leave`'s
own doc comment states why: any architecture that re-entered the `DO`'s own arm per iteration
risks the `Goto`-absorption trap (a nested construct's `Goto` back to the loop's own top getting
swallowed by an *enclosing* `IF`/`SELECT`'s `run_bounded` instead of reaching the loop). So
`step_in_temps_frame` is called exactly **once** for a `DO`/`LOOP` instruction's own position, no
matter how many times its body runs -- hooking the clause echo only there would echo the header
once at loop entry and never again, which is not what the oracle does under `TRACE R`/`I`.

**None of the four corpus programs this task must close exercises this.** `trace_output.rex`/
`trace_results.rex` have no loop at all; `prefix_dotvar_logical_over_label.rex`'s `do i over
'abc'`/`loop 2` both run under `Trace::Default`/`trace value 'N'`, which this task has already
confirmed produce no trace output whatsoever; `trace_numeric_request.rex` has no loop either.
Reproducing per-iteration re-echo would mean re-plumbing `run_loop`/`run_repeating`'s internal
iteration to re-invoke the clause-echo (and the `>K>` control-keyword trace) on every pass,
touching already-reviewed control-flow code for a formatting concern with no corpus program to
prove it against.

Flagged to the coordinator before implementing anything the loop-iteration side depends on:
proposing to echo a `DO`/`LOOP`'s own clause **once**, at the position `step_in_temps_frame`
already visits, and record the per-iteration re-echo as a **known gap** (this phase's own third
category, `phase-4-exclusions.txt`'s own precedent: "a measured divergence with no owner") rather
than close it inside this task -- and to pick `>K>`'s own committed witness from `SELECT CASE`'s
`CASE` (evaluated exactly once, no iteration) instead of a `DO`'s control clause, sidestepping the
question for criterion 3's own purposes without answering it for real loop programs.

**Rejected, on two grounds, both accepted here as correct.** (1) The `Goto`-absorption constraint
Task 10/11 built binds *returning* mid-construct, not *emitting* a trace line -- a per-pass echo at
the loop driver's own top changes no control flow, so "the single insertion point does not cover
it" was an argument against that insertion point, not against building the feature for real.
(2) Choosing `SELECT CASE`'s `CASE` *because* it cannot see the divergence is the same defect this
project has hit three times before (`/bin/true`, a no-op collect mode, a witness that cannot fail)
wearing better clothes -- a green criterion that never looked at the thing it exists to catch.
Told to build the real per-pass mechanism where it survives the attempt, and where it does not, to
say so and keep a witness that actually exercises the shape rather than one chosen to avoid it.

Also asked, before implementing: whether `DO FOREVER` (`LEAVE`) and `DO`+`ITERATE` re-echo the
same way, and whether `>K>` reappears on a pass whose condition is not re-evaluated.

## Step 7: `DoBlock::checkControl` read directly, and the real mechanism found

Rather than continue inferring from output, read `DoBlock.cpp` (`checkControl`, line 182,
`setCounter`, `DoBlock`'s own constructor) directly. It explains every open question:

* **`Then`/`Else` do it too** (already covered, Step 4): `context->indent(); traceInstruction(this);
  context->indent();` -- confirms the marker-is-half-its-body rule independently of the four probe
  points that first found it.
* **A `DO`/`LOOP` instruction's own C++ object is re-executed once per pass**, and every one of
  its own `execute()` overloads traces its own instruction on entry -- this is *why* the clause
  and `END` re-echo, not a separate design decision to reproduce piecemeal.
* **`checkControl`'s own `increment` flag is `false` on the very first pass and `true` on every
  one after.** `false`: reads the control variable's *already-assigned* value with **no trace at
  all** (the comment: "We've already traced the initial assignment as part of the setup").
  `true`: `result = control->evaluate(...); traceResult(result);` (the pre-increment value) then
  `result = result + by; traceResult(result);` (the post-increment value) then assigns. **This is
  the exact two-line `>>>` pair** measured on every `Controlled` loop's second pass onward, and it
  explains *why* it never appears on the first: nothing traces there in the oracle either.
* **`>K>` for `TO`/`BY`/`FOR`/`COUNTER` is evaluated once, at loop setup** (`DoBlock`'s own
  constructor for `COUNTER`, a `setup`-equivalent for `TO`/`BY`/`FOR` elsewhere) -- matching
  exactly what this crate's own `setup_controlled` already does (evaluate once, at entry), so
  hooking `>K>` there needed no restructuring at all, and produces the right answer *by
  construction*, not by luck.
* **`>K>` for `WHILE`/`UNTIL` is re-evaluated every pass** because the oracle's own condition check
  runs every pass too -- and this crate's `run_repeating` *already* re-evaluates that condition
  every pass (it has to, to decide whether to continue), so hooking `>K>` at that existing
  evaluation point is, again, correct by construction.

Coordinator independently verified the citation against the actual file and line (this project's
now-standard check, after several wrong-function citations elsewhere on this branch) before
accepting it.

### Two more probes, both requested, both done before implementing anything they answer

**`DO FOREVER` with `LEAVE`** (full stderr, `trace r`):
```
     3 *-* do forever
     4 *-*   n = n + 1
       >>>     "1"
     5 *-*   if n = 2 
       >>>     "0"
     6 *-* end
     3 *-* do forever
     4 *-*   n = n + 1
       >>>     "2"
     5 *-*   if n = 2 
       >>>     "1"
     5 *-*     then
     5 *-*       leave
     7 *-* say n
```
Re-echoes `do forever` on every pass, no `>K>` (no control variable at all), and **`END` never
echoes for the pass where `LEAVE` fires** -- only for the pass that falls through to it. This is
the subtlest rule in the whole set and got its own dedicated case, see Step 9.

**`DO i = 1 TO 3` with `ITERATE`**: identical re-echo and `>>>`-pair shape to the plain case; how a
pass ends (fell through vs. a matched `ITERATE`) makes no difference to what traces, since both
reach the same "about to continue" point in `do_body_outcome`'s own answer.

**`DO OVER`, probed properly rather than assumed to match `FOREVER`** (the coordinator's own ask,
since `OVER` is the one `LoopKind` with the least exposure in 4a, restricted to non-stem targets):
```
     2 *-* do i over 'abc'
       >K>   "OVER" => "abc"
     3 *-*   say i
       >>>     "abc"
     4 *-* end
     2 *-* do i over 'abc'
     5 *-* say 'end'
```
`>K> "OVER"` fires once, matching `TO`/`BY`/`FOR`'s own shape -- **no** `>>>` pair on the exit
pass, because `OVER`'s own per-pass advance is an array index, not `checkControl`'s `+BY`
arithmetic. `DO OVER` has no wrinkle beyond what `FOREVER`/`Controlled` already cover between
them.

## Step 8: implemented, verified byte for byte against every probe taken

**Which of the rules above were measured directly versus read from source, stated plainly rather
than left to blend together, per the coordinator's own ask**: the re-echo shape (clause and `END`,
every pass, `END` skipped on `LEAVE`) and the once-vs-every-pass split for `>K>` were **both**
measured independently (this report's own transcripts, taken before `checkControl` was read) **and**
confirmed by the source reading -- the two agree, which is the strongest evidence available here,
not merely convenient. The exact two-line content and order of the `Controlled` pair was **read
from source first** (`checkControl`'s own two `traceResult` calls) and then confirmed against the
independently-taken transcripts; without the source reading, "two `>>>` lines, values `n` then
`n+1`" would have been a guess from four data points.

Landed, this task's own permitted files only:
* `trace.rs` (new): `TraceMode`, `mode_from_setting`/`is_whole_number` (classification),
  `raised_numeric_trace_interactive_only`/`raised_invalid_trace_letter` (24.901/24.1, bare struct
  literals following `raised_select_no_when`'s own precedent in `run.rs`, since `error.rs` is
  outside this task's permitted files), the byte-level formatters (`push_clause`/`push_value`/
  `push_tagged`/`push_operator`), and the `Interp` methods each hook calls
  (`trace_clause`/`trace_result`/`trace_keyword`/`trace_assignment`/`trace_compound_name`/
  `trace_literal`/`trace_variable`/`trace_dotvar`/`trace_operator`/`trace_prefix_op`/
  `tracing_intermediates`).
* `lib.rs`: `mod trace`; `Interp::trace_mode: TraceMode` (deliberately **not** per-`Activation`,
  documented as a stated 4a-only simplification, 4b's first move to undo, mirroring
  `interpret_spike`'s own note); `Interp::current_value_indent: usize`, the one piece of state
  `eval`'s single insertion point needs that its signature does not carry (mirrors the oracle's own
  `settings.traceIndent`, a persistent field rather than a threaded parameter -- avoids the
  eighteen-arm threading job the withdrawn D17 note wrongly predicted for the *clause* retrofit,
  applied here to the *value-line* retrofit instead).
* `run.rs`: `step_in_temps_frame`'s clause-echo hook (the single default insertion point, sets
  `current_value_indent` unconditionally, calls `trace_clause` when `trace_mode.all`);
  `InstructionKind::Trace`'s own `exec_trace` arm (`Default`/`Setting`/`Skip`/`Value`, all four
  forms); `>>>`/`>=>`/`>C>` at `Assignment`'s three target shapes; `>>>` for `Say` (including the
  omitted-expression empty-string case) and for `IF`/plain `WHEN` (`ConditionTrace`, a caller-picked
  enum rather than a decision `eval_condition` makes for itself, since `WHILE`/`UNTIL` want `>K>`
  instead and never a bare `>>>` alongside it); explicit `WHEN`/`WhenCase`/`OTHERWISE` clause echoes
  (none of the three is ever visited by `step_in_temps_frame`, matching `record_failure_site`'s own
  precedent for the identical reason); `test_case_when`'s own `>>>` pair (`SELECT CASE`'s per-value
  comparison, stopping at the first match); `>K>` for `TO`/`BY`/`FOR`/`OVER` (once, at existing
  single-evaluation sites) and `WHILE`/`UNTIL` (every pass, at the existing per-pass evaluation
  site); `run_repeating`'s own per-pass re-echo of the `DO`/`LOOP` clause and (conditionally on
  falling through, not on `LEAVE`) `END`; the `Simple`-block arm's own one-time `END` echo (a
  block never repeats, so this is not `run_repeating`'s job); the documented, disclosed gap comment
  at `loop_advance`'s `Controlled` arm; the two `unreachable!` dispositions from Steps 4-5.5.
* `eval.rs`: `eval`'s own post-order hook (`trace_intermediate`), dispatched on `expr.kind` --
  `>L>` (`Literal`/`Constant`), `>V>` (`Variable`/`Stem`), `>C>`-then-`>V>` (`Compound`, tag = the
  compound's own unresolved source spelling, resolved name = the read-site's own `stem_name` +
  `tail_key`, **not independently correct for an aliased stem** -- the same narrow limitation
  `Assignment`'s own `>C>` has, and for the identical reason: resolving the *object's* own name
  when it differs from the read site's is `stem.rs`'s `stem_get`, outside this task's permitted
  files), `>E>` (`DotVariable`, the spec correction), `>P>` (`Prefix`), `>O>` (every `Binary`, one
  arm for all of arithmetic/comparison/logical/concatenation, reasoned from the arithmetic
  transcript rather than independently probed for every operator family).
* `tests/trace_oracle/` (new) + `tests/trace_oracle.rs` (new): four dedicated witnesses
  (`keyword_while`, `compound_read_write`, `prefix_operators`, `dotvariable_beyond_the_list`) plus
  `trace_output.rex` (read from `rust/corpus/lang/`, not duplicated) -- covers all nine reachable
  prefixes plus the `>E>` bonus. Regeneration command in the module doc comment, so `cargo test`
  alone is the gate, per the brief's own instruction.

## Step 9: verification

* **The `static_indent` panic, reproduced against the unfixed tree before the fix existed** (Step
  4/5, above) -- test-first in the fullest sense this task achieved.
* **Every oracle probe taken during design was re-run against the finished binary** (`rexx-run`)
  after implementation, byte for byte, not merely re-read: `t13_dowhile_trace`, `t13_forever_leave`,
  `t13_doto_iterate`, `t13_do_over`, `t13_select_case_trace`, `t13_dotvar_trace`, `t13_ifelse`,
  `t13_select_otherwise`, `t13_then_do_block`, `t13_nested_two_deep`, `t13_stem_assign_trace`,
  `t13_doto_trace`, `t13_select_when_match` -- every one matches stdout, stderr and exit code
  exactly except the two `Controlled`-loop transcripts (`t13_doto_iterate`, `t13_nested_two_deep`,
  `t13_doto_trace`), which are missing exactly the disclosed `>>>` pair and nothing else (verified
  by reading the diff: every other line, including the re-echoed clause immediately before and
  after the missing pair, matches).
* **Disclosed honestly: this task did not do strict red-green TDD for every hook.** The
  `static_indent` fix did (probe, reproduce, fix, re-verify). The trace feature itself, given its
  breadth (nineteen prefixes' worth of call sites across `Assignment`/`Say`/`If`/`When`/`WhenCase`/
  `Otherwise`/three `LoopKind`s), was built against oracle transcripts taken *before* each piece of
  code and verified against the same transcripts *after* -- broad differential verification rather
  than a failing unit test written first for each individual hook. Where a mismatch surfaced
  (`OTHERWISE`'s own clause missing entirely, a `Simple` block's own `END` missing entirely,
  `OTHERWISE`'s indent wrong by 2), it was found by this same differential process and fixed before
  moving on, not left for a reviewer to find.
* `cargo test -p rexx-exec`: 162 (`--lib`) + 11 (`spike.rs`) + 4 (`corpus.rs`, includes the ignored
  child-process test) + 5 (`trace_oracle.rs`) + doctests, all green, 0 failed.
* `cargo test --workspace`: every crate `test result: ok`, 0 failed (grepped for `FAILED`/`error[`,
  none found).
* `cargo test -p rexx-exec --test corpus`: **26 of 26** in both report mode and
  `REXX_CORPUS_GATE=1` (strict) mode.
* `cargo clippy --workspace --all-targets -- -D warnings`: clean.
* `cargo fmt --check` on this task's own four files plus the two new test files (`rustfmt --edition
  2024 <path>` for each, then `--check`): clean. **Trap worth naming**: a bare `rustfmt <path>`
  (no `--edition`) on `run.rs` fails to parse (this file's `let`-chains need edition 2024) and
  silently writes nothing -- safe, but easy to mistake for "already formatted". `cargo fmt --check`
  is what actually catches it.
* No `unsafe` anywhere in the four files touched (grepped; the three hits are all pre-existing doc
  prose, not code).
* `git status` in this shared worktree shows unrelated, uncommitted changes from other concurrent
  work (`Cargo.lock`, `rexx-exec/Cargo.toml`'s new `rexx-extract` dev-dependency,
  `rexx-extract/src/lib.rs`, `rexx-extract/tests/extract_assertions.rs`, an untracked
  `tests/assertions.rs`) -- confirmed none of it is mine (`git diff` on each), left untouched, and
  excluded from this task's `git add`.

## Known gap, stated once more in full

**A `Controlled` (`TO`-style) `DO`/`LOOP`'s own re-tested pass is missing two `>>>` lines**: the
control variable's pre- and post-increment value (`DoBlock::checkControl`, `DoBlock.cpp:182`).
Root cause: this crate's `current` (in `LoopState::Controlled`) already holds the *next* pass's
value by the time `loop_advance` runs (`loop_step` computed it at the end of the *previous* pass),
so the pre-increment value is gone by the point the trace would need it. Fixing it means moving the
increment out of `loop_step` and into `loop_advance` itself, matching `checkControl`'s own
structure -- a restructuring of the exact split several rounds of review and mutation testing
already certified (`the_corrected_28x_indent_rule_matches_all_fourteen_probed_shapes`, the
`Flow::Leave`/`run_bounded` absorption discipline). Deliberately not attempted under this task's
remaining time rather than risked. Documented at the exact site (`run.rs`, `loop_advance`'s
`Controlled` arm) and not hidden from criterion 3's own table: the `>K>` witness chosen
(`keyword_while.rex`) is a complete, gap-free answer for a *different* `LoopKind` that fully
exercises re-echo and per-pass `>K>`, not a substitute chosen to avoid this one. `TO`/`BY`/`FOR`/
`COUNTER`'s own `>K>` (fires once) and `WHILE`/`UNTIL`'s own `>K>` (fires every pass) are both
unaffected and both verified correct.

Not filed to `docs/superpowers/plans/phase-4-exclusions.txt` directly: that file is outside this
task's permitted files. Flagging here for whoever owns it next (Task 16, per the plan) to fold in.

## Commits

* `4ec4884d` -- the `static_indent` marker-clause fix, separate and before, per the coordinator's
  own condition 1.
* `b3e2e112` -- the trace feature itself: `trace.rs` (new), `eval.rs`, `run.rs`, `lib.rs`,
  `tests/trace_oracle.rs` (new), `tests/trace_oracle/` (new). 14 files, 1454 insertions, 47
  deletions.

Both verified independently before commit: full test suite, clippy, fmt, corpus (report and
strict-gate mode), and the differential probes taken during design re-run against the finished
binary. Working tree otherwise carries unrelated, uncommitted work from other concurrent agents in
this shared worktree (`Cargo.lock`, `rexx-exec/Cargo.toml`, `rexx-extract/*`, an untracked
`tests/assertions.rs`) -- confirmed none of it touched by either commit.

## Files

* `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs` (new)
* `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs`
* `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run.rs`
* `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs`
* `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/tests/trace_oracle.rs` (new)
* `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/tests/trace_oracle/` (new;
  `keyword_while`, `compound_read_write`, `prefix_operators`, `dotvariable_beyond_the_list`, each a
  `.rex`/`.expected` pair)
* This report:
  `/home/moritz/dev/repos/ooRexx-rust-rewrite/.superpowers/sdd/2026-07-30-phase-4a-executor/task-13-report.md`

## Review round: two findings, one Critical, one Minor

**Finding 1 (Critical, the reviewer's own, not this task's own probes): the absorbed-`WHEN`
shape silently swallowed a raising condition.** `select / when 1=1 then / when 1/0 then nop /
otherwise nop / end / say 'after'` -- oracle rc **214**, `Error 42.3` at line 3; ours (before this
fix) rc **0**, prints `after`. Not a trace defect: a silently wrong answer, the exact failure class
D19/the loud-failure discipline exists to exclude, and it survived Task 10's review, Task 11's, and
the first round of this one's, because every existing probe (including this task's own -- see the
existing test `an_absorbed_when_runs_neither_its_own_consequence_nor_otherwise`, still in the tree
unchanged) used a side-effect-free *true* condition, which cannot distinguish "never evaluated"
from "evaluated and discarded." Trace is what surfaced it: building the clause echo required
reading what `InstructionKind::When`/`WhenCase` actually do when stepped ordinarily (the absorbed
case), and the existing comment there asserted "and in neither case does it get to decide anything
on its own" -- true for *deciding*, silently wrong about *evaluating*.

Root cause, confirmed by three probes against the real oracle before touching code:
* `select / when 1=1 then / when 1/0 then nop / otherwise nop / end / say 'after'` -- rc 214, 42.3.
* `select / when 1=1 then / when 2=2 then n=42 / otherwise / n=99 / end / say n` -- prints `0` (the
  existing test's own claim, re-verified, and true regardless of which model is right).
* `select / when 1=1 then / when 2=2 then say 'ABSORBED-RAN' / end / say 'after'` -- prints only
  `after`, **never** `ABSORBED-RAN`, even though `2 = 2` is true. This is the probe that actually
  distinguishes the two models, and nobody had run it before this review round.

All three together pin the real rule: an absorbed `When`/`WhenCase`'s own condition (or, for
`WhenCase`, its own comparison values) **is evaluated for real** -- for side effects and so a raise
escapes -- but it **never gets to take its own branch**, true or false, matched or not. `Select`'s
own arm never sees or touches an absorbed node at all (`ast.rs`'s own doc comment on `whens`,
`LanguageParser.cpp:1319`); a `When`/`WhenCase` reached through ordinary `step_in_temps_frame`
stepping is *always* the absorbed shape, since a *listed* one is fully handled by `Select`'s own
explicit arm without ever calling `step` on itself.

Fix: `InstructionKind::When`'s own arm now calls `eval_condition` (the same function, the same
raiser, `ConditionTrace::Result(self.current_value_indent)` so its own `>>>` traces correctly too,
since this instruction *is* visited by the ordinary clause-echo path unlike a listed `WHEN`) and
discards the boolean, always returning `Flow::Next`. `InstructionKind::WhenCase`'s own arm
evaluates every `values` expression the same way, for the same reason -- **known, disclosed,
narrower gap**: with no `case_text` threaded to an absorbed node, this cannot reproduce
`test_case_when`'s own two-line `>>>` comparison pair for an absorbed `WHEN CASE` specifically; no
corpus or spec example exercises that combination, and the comment at the site says so rather than
claiming it.

Two new tests, `run.rs`:
* `an_absorbed_whens_raising_condition_escapes_even_though_its_own_consequence_never_runs` --
  the mutation it kills is exactly the bug: reverting `When`'s arm to `Ok(Flow::Next)` makes this
  test's `run_source` return `Ok` (rc 0, prints `after`) instead of `Err(Raised { 42, 3 })`.
  Verified by mutation: reverted the fix, this test failed (`unwrap_err()` on an `Ok`), the other
  two absorbed-`WHEN` tests still passed -- confirming this is the one test that could see it, and
  that the fix doesn't disturb the other two claims.
* `an_absorbed_whens_true_condition_still_never_runs_its_own_consequence` -- the companion probe
  that distinguishes the two models by *content* (`ABSORBED-RAN` must never print) rather than only
  by a variable's final value, which the pre-existing `n = 0` test alone cannot do.

Both re-verified against the real oracle through `rexx-run` on the coordinator's own exact program:
stdout, stderr and exit code (214) all byte-for-byte identical.

**Finding 2 (Minor, F1): a bare repeat count (`DO n`, no `TO`/`BY`/`FOR`/`WHILE`/`OVER`) traced no
`>K>` line at all.** Measured: the oracle tags it `FOR`, the same tag an explicit `DO ... FOR n`
gets (`>K>   "FOR" => "2"`), once, at the `DO`'s own level, on the first pass only -- identical
shape to every other single-evaluation `>K>` site. My own earlier verification claim ("every `>K>`
... verified correct") was true of every `LoopKind` *I had actually probed* and silently excluded
`Count`, which the coordinator named as the more useful half of the finding: a verification claim
that quietly narrows its own scope is worse than an admitted gap, because it reads as covering
something it never touched. Corrected here rather than left as written above.

Fix: one `trace_keyword` call in `run_loop`'s `LoopKind::Count` arm, at the existing count-
expression evaluation site, before the `whole_nonneg` validity check (matching every other `>K>`
site's own order: traced as evaluated, not as validated). Verified byte-for-byte against the
oracle through `rexx-run`.

New test, `run.rs`, `a_bare_repeat_count_traces_as_for_the_same_as_an_explicit_one`: asserts
`interp.trace` directly (via `mode_from_setting(b"r")`) against the exact captured bytes, including
the third re-echo pass (the exit check) -- caught **by this test itself** on the first attempt,
where a hand-composed two-pass expectation was one pass short; corrected by reading the real bytes
back rather than guessing a second time. Verified by mutation: removing the new `trace_keyword`
call made this test fail (missing the `>K>` line), with no other test in the file affected.

### Re-verification after both fixes

* `cargo test -p rexx-exec --lib`: **165** passed (162 + 3 new), 0 failed.
* `cargo test -p rexx-exec` (whole crate): every `test result: ok`, 0 failed.
* `cargo test -p rexx-exec --test corpus`: **26 of 26** in report mode.
* `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`: **26 of 26** in strict mode.
* `cargo test --workspace`: every `test result: ok`, 0 failed (grepped for `FAILED`/`error[`, none).
* `cargo clippy --workspace --all-targets -- -D warnings`: clean.
* `cargo fmt --check` (via `rustfmt --edition 2024 <path>` then `--check`, this task's own four
  files): clean.

### Not touched, per the coordinator's own correction

The Controlled-loop `>>>`-pair gap stays a recorded gap (`phase-4-exclusions.txt`, commit
`6539ac11`, outside this task's own edit), with its cost corrected there from "restructuring a
split" to "move the increment from `loop_step` into `loop_advance` behind a not-first-pass flag,
mirroring `checkControl`, roughly twenty lines, real cost is re-verifying bound-before-test/`FOR`/
`ITERATE` semantics, half a day" -- not this task's finding, recorded here only because it is the
other half of the same review and the corrected estimate matters to whoever picks it up next.

## Final commits

* `4ec4884d` -- the `static_indent` marker-clause fix.
* `b3e2e112` -- the trace feature itself.
* `be7cc8ef` -- the review fix round: the absorbed-`WHEN` Critical, and F1 (bare-count `>K>`).

## Re-review round: F3 (Critical, a wrong answer) and F4 (trace-only)

**F4: comma-list conditions traced nothing under `TRACE R`.** Measured: `trace r` / `if 1, 1 then`
gives *three* `>>>` lines from the oracle (one per element, one for the list's own overall result),
one from this crate (only the list's own result, from `eval_condition`'s existing hook). **Not an
architecture problem, reported back before landing anything, per the coordinator's own ask**: the
two missing lines are `trace_result` (results-gated, matching `test_case_when`'s own precedent, the
identical shape `SELECT CASE`'s comma list already has), not a second, competing intermediate-value
computation -- `eval_logical_list` already walks every element and already has each one's own
`text` in hand, so one `self.trace_result(self.current_value_indent, &text)` call, unconditional on
the element's own logical validity (traced *before* the `logical_value` check, matching the oracle:
`if 1, 'x' then` traces the failing `'x'` before raising 34.6), fixes it in the one function all
four keywords already share.

**Re-verifying it exposed a second, adjacent bug this task's own earlier work had introduced: `DO
UNTIL`'s clause re-echo was double-counted on any pass after the first.** The existing top-of-loop,
`first_pass`-gated re-echo (built for `WHILE`/`Controlled`/`Forever`/`OverOnce`, and correct for all
of them) was *also* firing for `UNTIL` loops, on top of `UNTIL`'s own site -- a multi-pass `DO
UNTIL` echoed its clause twice per pass instead of once. Root cause, read from a fresh oracle
transcript rather than assumed: `UNTIL`'s own re-entry (after the body, deciding whether to loop
back) is a *different* decision event from the top-of-loop one, and it is the *only* decision event
an `UNTIL`-only loop has -- there is no separate "test `WHILE`, then maybe run the body" event to
share the top-of-loop site with. Fixed by gating the top-of-loop echo off entirely for `UNTIL` loops
(`is_until_loop`), relying solely on `UNTIL`'s own already-added echo site.

Two new tests, `run.rs`:
* `a_comma_list_conditions_own_elements_each_trace_their_result_under_trace_r` -- kills removing
  `eval_logical_list`'s new `trace_result` call (verified by reverting it: the middle two `>>>`
  lines vanish, the third, pre-existing one does not, confirming the fix and not a coincidence).
* `do_until_re_echoes_its_clause_exactly_once_per_pass_not_twice_or_zero` -- kills *both* directions
  at once: removing `is_until_loop`'s own gate reproduces the double-echo (verified); removing
  `UNTIL`'s own echo site instead reproduces zero echoes on the first pass (verified separately).
  Neither pre-existing `WHILE`/`Controlled` trace test could have caught either mutation, since
  neither exercises `UNTIL` at all.

Re-verified: every `WHILE`/`FOREVER`/`Controlled`/`OVER` transcript taken during the original design
work re-run against the rebuilt binary, all still byte-for-byte identical (no regression from the
`is_until_loop` change).

**F3 (Critical): unlike a plain `WHEN`'s absorbed form, an absorbed `WHEN CASE` genuinely branches
on a false match, and this crate did not.** `select case 2 / when 2 then / when 3 then nop /
otherwise say 'O' / end / say 'after'` -- oracle prints `O` then `after`; before this fix, only
`after`. Not probed and guessed: read the actual parsed AST fields directly (a throwaway debug
binary against `rexx_parse::parse_program`, not inference from output) to find the real mechanism,
because reasoning from output alone had produced two self-contradictory hypotheses first. The real
shape: the *outer*, listed `WhenCase`'s own `false_target` stops *before* the absorbed node's own
index, so `Select`'s own `run_bounded` call for the outer branch never reaches the absorbed node's
own body regardless of what that node's arm returns on a **true** match (confirmed with a dedicated
probe, `t13_f3_true.rex`: a true absorbed `WHEN CASE` still never runs its own consequence, matching
the coordinator's own "matches on both sides" -- this crate's pre-F3 behaviour already agreed with
the oracle here). The one case needing a real branch is **false**: the absorbed node's own
`false_target` points past its own body to whatever follows in the *enclosing* range (`OTHERWISE`,
here) -- outside the bound the outer `run_bounded` call is scoped to, so `Flow::Goto(false_target)`
escapes it unchanged, exactly the way a `LEAVE`/`ITERATE` naming an enclosing construct already does
(`run_bounded`'s own absorption-avoidance, unmodified).

Fixing this needed one new piece of state, `Interp::current_case_text: Option<Vec<u8>>` (`lib.rs`),
mirroring `current_value_indent`'s own field-not-parameter shape: an absorbed `WhenCase` has no
other way to reach the enclosing `SELECT CASE`'s own evaluated `case` text, since a *listed*
`WhenCase` gets it handed directly by `Select`'s own arm and an absorbed one is stepped like any
ordinary instruction, with nothing carrying it along. Set unconditionally by `Select`'s own arm, not
saved/restored across a nested `SELECT CASE` -- a disclosed, narrow limitation (documented at both
the field and the read site), matching this task's own established pattern for the `>C>` aliasing
gap. `InstructionKind::WhenCase`'s own arm now calls `test_case_when` directly (reused, not
duplicated -- it already takes exactly the `(code, values, case_text, indent)` shape needed, and
its own `>>>` comparison-pair tracing comes for free once `case_text` is available at all, closing
the trace-fidelity gap the previous round's comment had flagged as unreachable).

**The plain-`WHEN` sibling of this false path is deliberately untouched and unprobed**, per the
coordinator's own explicit instruction: it is one line away from SF #2018's segfault (`select /
when 1=0 then / when 2=2 then nop / end`), so the oracle cannot answer what it should do, and
probing to find out risks reproducing a crash this project has already filed and left alone. The
new code's own comment says this in place of silence, per the coordinator's own preference.

Two new tests, `run.rs`:
* `an_absorbed_whencases_false_condition_branches_to_its_own_false_target` -- kills reverting
  `WhenCase`'s arm to the old evaluate-and-discard shape (verified: reverting reproduces exactly the
  wrong `after`-only output, while the sibling `When` tests and the true-match `WhenCase` test all
  stay green, confirming the fix is scoped to `WhenCase` alone and does not touch `When`).
* `an_absorbed_whencases_true_condition_still_never_runs_its_own_consequence` -- pins the "matches
  on both sides" half so a future change to the false-path fix cannot silently start running the
  true path's own consequence too.

### Final re-verification, all four review rounds together

* `cargo test -p rexx-exec --lib`: **169** passed (162 base + 3 first round + 2 F1/absorbed-WHEN +
  2 F4/UNTIL + 2 F3), 0 failed.
* `cargo test -p rexx-exec --test corpus`: **26 of 26**, report mode.
* `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`: **26 of 26**, strict mode.
* `cargo test --workspace`: every `test result: ok`, 0 failed.
* `cargo clippy --workspace --all-targets -- -D warnings`: clean.
* `cargo fmt --check` (`rustfmt --edition 2024 <path>` then `--check`, all four of this task's own
  files): clean.
* **Every differential probe taken across this task's entire design and review history re-run
  against the final binary in one sweep** (26 programs: every `LoopKind`/marker/absorption/comma-
  list/`>K>`/`>C>`/`>P>`/`>E>` shape probed at any point in this task) -- all byte-for-byte
  identical to the oracle, none newly broken by a later fix.

## Final commits

* `4ec4884d` -- the `static_indent` marker-clause fix.
* `b3e2e112` -- the trace feature itself.
* `be7cc8ef` -- review round 1: the absorbed-`WHEN` Critical, and F1 (bare-count `>K>`).
* `bca025c2` -- review round 2: F3 (absorbed-`WHEN CASE` Critical) and F4 (comma-list trace, plus the
  `DO UNTIL` re-echo bug F4's own re-verification found).

## Review round 3: the indent inside F3's own new path

**A divergence inside the path F3 itself opened, found by the coordinator's own perimeter probing
of the fix, not by this task.** Same shape as F3's own example but with no `OTHERWISE`, so the
absorbed `WhenCase`'s own false branch lands on `END` and raises 7.3: `select case 2 / when 2 then
/ when 3 then nop / end / say 'after'`. Both sides rc 249, identical error text, but the clause echo
differs by indent alone -- ours `     4 *-* end` (0 spaces), the oracle's `     4 *-*     end` (4).
Checked whether this was pre-existing before touching anything: a plain `SELECT` and a plain
`SELECT CASE` both reaching 7.3 the *ordinary* way (no absorption involved) are byte-identical, so
this is specific to the `Flow::Goto(false_target)` path F3 added, not a latent bug F3 merely
exposed.

**Read as the same "-2" rule already governing every marker in this task, not a new one, per the
coordinator's own instruction to check before writing a special case.** `4` is the absorbed
`WhenCase`'s own condition depth (`6`, already measured correct and unchanged for its own clause)
minus `2` -- the identical "a marker sits at half its own body's indent" arithmetic
`static_indent`'s own doc comment already states for `THEN`/`ELSE`/`OTHERWISE`/a `WHEN`'s own
`THEN`. Confirmed at a second nesting depth before trusting one data point (`t13_f3_nested.rex`,
the same shape one `DO` level deeper): indent 6, not 4 -- the same construct's own depth plus the
loop's own two, matching the rule at every level rather than fitting the one example given.

**Considered, and rejected, growing `Flow::Goto` an indent payload to reuse `pop_search_frame`'s own
machinery directly** -- the literal reading of "the two are one rule and should be written as one."
`Flow::Goto` is the ordinary resume mechanism every `If`/`Select`/`Do` match already uses (`Ok(Flow
::Goto(resume))`, dozens of sites), none of which has any residual indent to carry; giving all of
them one to serve this single escape is the search-frame restructuring the coordinator asked to be
told about rather than done. Used the same *conceptual* rule instead, carried through a field
(`Interp::pending_escape_indent: Option<usize>`, `lib.rs`) mirroring `current_value_indent`/
`current_case_text`'s own already-established shape -- set once, at the one arm that produces this
escape, consumed once, by `step_in_temps_frame` on *every* step (not only a failing one, so a
non-failing landing cannot leak the residual to a later, unrelated failure), and threaded into
`record_failure_site` (which gained an `escape_indent: Option<usize>` parameter, `None` from
`Select`'s own two direct calls for an ordinary `When`/`WhenCase` condition failure, which never
go through a fresh `step_in_temps_frame` of their own and so have nothing to have consumed).

Also corrected a comment `InstructionKind::End`'s own arm carried since before this task: "`Select`'s
own arm above sends every other path around this instruction entirely" was true of every path
`Select` dispatches directly and false of this one, which escapes *through* it rather than being
sent by it -- left in place with the correction rather than silently rewritten, per this task's own
standing rule against deleting a comment's own history.

One new test, `run.rs`: `an_absorbed_whencases_escaping_false_branch_reports_end_at_its_own_residual
_indent`. The mutation it kills: removing the `pending_escape_indent` assignment makes the read-back
indent `0` instead of `4` (verified by reverting); the pre-existing, unrelated
`select_with_no_when_true_and_no_otherwise_raises_7_3` (the *ordinary*, non-absorbed 7.3 path) stays
green either way, confirming the fix is scoped to the escape path alone and does not touch the
common case.

### Final re-verification, all three review rounds together

* `cargo test -p rexx-exec --lib`: **170** passed, 0 failed.
* `cargo test -p rexx-exec --test corpus`: **26 of 26**, report mode.
* `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`: **26 of 26**, strict mode.
* `cargo test --workspace`: every `test result: ok`, 0 failed.
* `cargo clippy --workspace --all-targets -- -D warnings`: clean.
* `rustfmt --edition 2024 <path>` then `cargo fmt --check`: clean on all four of this task's own
  files -- noting for the record, since it bit this round too: a bare `rustfmt <path>` with no
  `--edition` flag defaults to 2015 for this crate and rejects its `let`-chains outright, rather
  than silently under-formatting, so the failure is loud, not a trap that ships quietly.
* **All 28 differential probes taken across this task's entire history, re-swept against the final
  binary in one pass**: byte-for-byte identical to the oracle on every one, including the two new
  F3-perimeter probes (`t13_f3_end_indent.rex`, `t13_f3_nested.rex`).

## Final commits

* `4ec4884d` -- the `static_indent` marker-clause fix.
* `b3e2e112` -- the trace feature itself.
* `be7cc8ef` -- review round 1: the absorbed-`WHEN` Critical, and F1 (bare-count `>K>`).
* `bca025c2` -- review round 2: F3 (absorbed-`WHEN CASE` Critical) and F4 (comma-list trace, plus
  the `DO UNTIL` re-echo bug F4's own re-verification found).
* `50774cd0` -- review round 3: the residual-indent divergence inside F3's own escape path.
* `958a06b9` -- review round 4: the whole-branch review (F-EX1, F-EX2, stale comments, seven Minors,
  F-EX3, F-EX4).
* `556a84f7` -- review round 5: F1/F2/F3 from the branch review's value-model slice.

## Review round 4: whole-branch review (`branch-review-exec.md`, `run.rs`/`eval.rs`/`lib.rs`, `9f68662a..HEAD`)

Not a per-task review this time -- the coordinator's slice covering every task that touched the
execution core in that range, looking specifically for cross-task interactions the per-task reviews
could not see. Two Important findings (F-EX1, F-EX2), two stale comments now false rather than
merely dated (S1, S2), seven Minors (S3-S9), and two more Minors argued from code structure (F-EX3,
F-EX4). Addressed in that order, each verified against the oracle or by mutation as appropriate.

### F-EX1: F3's absorbed-`WhenCase` escape bypassed `leave_select`

Cross-task interaction: Task 11 rerouted `OTHERWISE`'s own body through `Select`'s own `run_bounded`
+ `leave_select` so the `SELECT` can recognise its own label on a `LEAVE`/`ITERATE`; Task 13's F3
fix reintroduced the pre-Task-11 shape on the escape path -- a false absorbed `WhenCase` returned
`Flow::Goto(false_target)` unconditionally, which exits `Select`'s own arm entirely and, when the
target happens to be the `OTHERWISE` marker, runs `OTHERWISE`'s body under the *outer* loop, outside
any `leave_select`.

Measured (oracle, `ulimit -v 1048576`), `select label s / case 2 / when 2 then / when 3 then nop /
otherwise say 'O' / leave s / end / say 'after'`:

```text
oracle: O, after, rc 0        ours (before this fix): O, then Error 28.3, rc 228
```

Same shape with `iterate s` in the `OTHERWISE`: oracle raises 28.5 (named, matched something, not a
loop), ours (before this fix) raised 28.4 (named, matched nothing) -- wrong error family, not only a
wrong indent.

**Fix.** Extracted `Select`'s existing `Some(otherwise_index) => ...` arm into a new method,
`run_otherwise`, shared by both callers: the ordinary "no `WHEN` matched" path, and a new check added
right after the matched-listed-`WHEN`'s own `run_bounded` call --

```rust
if let Some((body_end, resume)) = outcome {
    let flow = self.run_bounded(code, when_index + 1, body_end, source)?;
    if let Flow::Goto(target) = flow
        && *otherwise == Some(target)
    {
        return self.run_otherwise(code, index, *label, target, *end, source);
    }
    return self.leave_select(code, index, *label, resume, flow);
}
```

`run_otherwise` runs `OTHERWISE`'s body through `run_bounded` and hands its own result to
`leave_select`, exactly as the ordinary path already did -- so a `LEAVE`/`ITERATE` naming the
enclosing `SELECT LABEL` from inside an escaped `OTHERWISE` now resolves through the same frame the
ordinary path uses. Verified against the oracle: the `leave s` probe now gives `O`, `after`, rc 0;
the `iterate s` probe now raises 28.5, not 28.4.

**A second bug found while fixing the first, not separately flagged by the review: the escape's own
indent was wrong under nesting.** Getting the `LEAVE`/`ITERATE` resolution right exposed that the
absorbed `WhenCase`'s escape needs every `static_indent` computation past it to carry an additional,
constant offset -- the same "a marker sits at half its own body's indent" gap this task's round 3
already found for the `END` landing shape, now needed at every landing shape reachable through
`run_otherwise` too (`OTHERWISE`'s own marker, `OTHERWISE`'s own body, and the `LEAVE`/`ITERATE`
clause itself via `leave_origin`). Renamed the round-3 field (`pending_escape_indent: Option<usize>`,
one-shot, consumed by `.take()`) to `indent_offset: usize`, ambient like `current_value_indent`,
because a one-shot value cannot survive `run_otherwise`'s own multi-step body.

Two re-verifications, in order, both against the oracle:

1. A full `TRACE R` transcript comparison (`h_trace_otherwise_escape.rex`, scratchpad) still showed
   one wrong line after the `LEAVE`/`OTHERWISE` fix: `SAY`'s own `>>>` value trace. Root cause: eight
   call sites (`Say`, `Assignment`, `If`, `Select`'s own `select_indent`, `Count`'s `FOR` keyword,
   `Controlled`'s setup, `Over`'s keyword, `run_repeating`'s `do_indent`) each independently
   recomputed `static_indent(&code.body.instructions, index)` instead of reading
   `self.current_value_indent`, which `step_in_temps_frame` had already computed *with* the offset
   included. Two computations of one quantity is how they drift -- fixed by replacing all eight with
   a read of the already-correct field.
2. With that fixed, a full 32-probe regression sweep across this task's entire history found one
   more diff, one `DO` level deeper than the first probe (`t13_f3_nested.rex`): indent `8` where the
   oracle wants `6`. The formula in use at that point, `self.current_value_indent.saturating_sub(2)`,
   is *not* constant -- it happened to equal `4` at the top level (matching round 3's own `END`
   finding) and gave `6` one level deeper, where the oracle still wants `4`. Re-measured at two
   nesting depths for all three landing shapes (`END`, `OTHERWISE`'s marker, `OTHERWISE`'s body)
   before trusting a correction a second time:

   | landing shape | top level: ordinary / actual | one `DO` deeper: ordinary / actual |
   |---|---|---|
   | `END`'s own 7.3 | `0` / `4` | `2` / `6` |
   | `OTHERWISE`'s own marker | `2` / `6` | `4` / `8` |
   | `OTHERWISE`'s own body | `4` / `8` | `6` / `10` |

   `actual - ordinary = 4` uniformly -- the offset is the constant `4` (two `indent()` bumps: the
   enclosing listed `WHEN`'s own marker, then its own body entry), not a function of nesting depth.
   Hardcoded `self.indent_offset = 4;` at the one arm that sets it; rewrote both the field's own doc
   comment (`lib.rs`) and the assignment site's own comment (`run.rs`) to state this, including that
   the earlier formula was right for `END` only by coincidence at the top level.

One more probe (`j_trace_nested_otherwise.rex`) showed a diff after the constant-`4` fix; confirmed
by independently re-running the already-disclosed, pre-existing `Controlled`-loop `>>>`-pair gap
(`t13_doto_trace.rex`, this file's own known-gap section above) that this is the *same* accepted
limitation, incidentally exercised because that probe's outer wrapper happened to be a `Controlled`
loop -- not a new regression, no fix applied, correctly out of scope.

Three new tests, `run.rs`: `an_absorbed_whencases_escape_to_end_reports_the_same_constant_offset_
nested` (asserts indent `6` for a nested no-`OTHERWISE` case; mutation killed: reverting to
`current_value_indent.saturating_sub(2)` gives `8`), `an_absorbed_whencases_escape_to_otherwise_
still_finds_the_enclosing_selects_own_label` (asserts `say_output` is `b"O\nafter\n"`; mutation
killed: disabling the escape-redirect check reproduces the pre-fix Error 28.3), `an_absorbed_
whencases_escape_to_otherwise_reports_a_named_iterate_as_28_5_not_28_4` (asserts the raised
condition is `(28, 5)`; same mutation, same kill).

### F-EX2: `Flow::Leave`/`Iterate` carried a fragment-table-relative `SymbolId` across the `run_fragment` boundary

`run_fragment` gives an `INTERPRET` fragment its own fresh `SymbolTable` (`parse_interpret`'s doc);
`run_bounded`'s own catch-all forwarded a `Flow::Leave`/`Iterate` produced inside that fragment
unchanged, so a `leave name`/`iterate name` inside `INTERPRET` text carried a `SymbolId` relative to
the *fragment's* table while every consumer above (`do_body_outcome`, `leave_select`,
`run_activation`) compared it against the *program's* table -- table-relative ids, wrong table,
nothing at the type level to catch it. Concretely: `run_activation`'s own `Flow::Leave(Some(n), ..)`
arm calls `code.symbols.name(n)` on the *program's* table with an id interned in the *fragment's*.

**Cost assessment before fixing anything, per the coordinator's own instruction.** Two ways to make
this correct rather than merely contained were considered and rejected: (a) a new "look up without
creating" method on `rexx-parse`'s `SymbolTable` -- `intern` is the only mutator and always creates
on a miss, `name`/`len` are the only readers, and neither can answer "does this name already exist"
without one -- out of scope, `rexx-parse` is not one of this task's permitted files; (b) changing
`Flow::Leave`/`Iterate` to carry a resolved name instead of an id -- the exact restructuring the
coordinator asked to be told about rather than done, and it would touch every comparison site in
`run.rs`, not only the fragment boundary. Contained instead: refuse the *specific* case that is
wrong (a *named* `Leave`/`Iterate` crossing the boundary) at the one place that still has the correct
table in scope to name it -- `run_fragment` itself, before its `Rc<Fragment>` (and the `SymbolTable`
it owns) goes out of scope. A *bare* `Leave`/`Iterate` carries no `SymbolId` at all and is unaffected.

Added `Loud::interpret_leave(name: &str) -> Loud` (`lib.rs`) and, in `run_fragment` (`run.rs`):

```rust
let flow = self.run_bounded(&code, 0, code.body.instructions.len(), None)?;
match &flow {
    Flow::Leave(Some(id), _) | Flow::Iterate(Some(id), _) => {
        Err(Loud::interpret_leave(fragment.symbols.name(*id)).into())
    }
    _ => Ok(flow),
}
```

This turns a silent wrong-answer (or a possible panic, if the fragment's id happens to be out of the
program table's range) into a loud, honest refusal naming the fragment's own symbol correctly.
Reachable only through `run_program_interpret_spike` today (`run_program`'s own `INTERPRET` arm is
still `Loud::instruction`, unconditionally, before this code ever runs), so no shipped behavior
changes and the corpus is unaffected by construction. 4b, which builds real `INTERPRET` on this exact
machinery, has to replace this refusal with an actual resolution -- documented at both the
`Loud::interpret_leave` and `run_fragment` doc comments, pointing at the same two rejected options
above so 4b does not have to re-derive them.

Two new tests, `run.rs`. `a_fragments_named_leave_refuses_to_cross_the_boundary_rather_than_forward_
an_id`: program `bar = 1\ninterpret "leave foo"\n`, deliberately chosen so the fragment's own table
(one symbol, `FOO`, id 0) and the program's own table (one symbol, `BAR`, also id 0) collide on the
same id with *different* names -- exactly the shape that turns "forward the id" into a wrong answer
rather than a panic. Mutation killed: reverting the `match` to a bare `Ok(flow)` (the pre-fix shape)
changes the result from `Failure::Loud` naming `"FOO"` to `Failure::Raised` 28.3 naming `"BAR"` --
verified directly, error message shown above in this report's own working notes matches
`Raised { condition: "SYNTAX", number: 28, sub: 3, additional: ["BAR"] }`. `a_fragments_bare_leave_
still_forwards_across_the_boundary`: program `interpret "leave"\n` (no name), still becomes the
ordinary exhausted-search `Raised` error at the top. Mutation killed: making the refusal
unconditional (matching on `Flow::Leave(id, _)` and unwrapping unconditionally) turns this into
`Failure::Loud` too, caught by the assertion.

**Left open, disclosed rather than fixed.** A `LeaveOrigin` born inside a fragment always has
`site: None` (`source` is always `None` there, by `run_fragment`'s own established convention), and
unlike a `Raised` condition escaping a fragment -- which the enclosing `INTERPRET` clause's own
`step_in_temps_frame` call gets a fresh, first-wins chance to record as it propagates back up --
`Flow::Leave`/`Iterate` is `Ok`, so it never runs that recording path, and a bare, still-forwardable
`Leave`/`Iterate` that is ultimately unresolved reports `<no failing clause recorded>` instead of the
enclosing `INTERPRET` clause. Fixing this needs `record_leave_failure`'s callers (`run_activation`,
`leave_select`, `do_body_outcome`) to fall back to their own current clause when `origin.site` is
`None`, which touches three call sites rather than one function, and nothing measures what the
oracle actually does here to aim the fix at -- `INTERPRET` is unreachable through the oracle
differential harness today (only through the spike). Documented at `run_fragment`'s own doc comment
rather than fixed, matching the review's own "at minimum a comment" floor for this half of F-EX2.

### Stale comments, S1 and S2 (Important)

* **S1** (`lib.rs`, `Interp::trace` field doc): removed the false "nothing in this crate writes yet"
  claim and "Task 13 is the first to write to it" (Task 13 shipped in this range; `trace.rs` and a
  dozen `run.rs` call sites write to it now). Rewrote in the past tense.
* **S2** (`run.rs` module doc, and `Select`'s own arm doc it contradicted): "`Then`, `Else`,
  `Otherwise`, `When` and `WhenCase` accordingly step as pure no-ops" was true of the first three and
  false of the last two since the absorbed-`WHEN` fix and F3 -- `When`'s arm evaluates its own
  condition (can raise 42.3) and `WhenCase`'s arm evaluates, compares, and branches, for the absorbed
  case. Narrowed both comments to distinguish *listed* (still never independently stepped for a
  decision of its own -- `Select`'s arm dispatches it directly) from *absorbed* (independently
  stepped, the exception both comments now name).

### Seven Minors, S3-S9

* **S3** (`eval.rs`, `eval_condition` doc): "none of those instructions run yet" was true before
  Tasks 10/11 and false after -- `IF`/`WHEN`/`WHILE`/`UNTIL` all run through `run` now. Rewrote to
  say the helper stays for isolating `eval` from `step`'s surrounding dispatch, not because `run`
  cannot reach a comma-list condition.
* **S4** (`eval.rs`, `activate` doc): "`step`'s `Assignment` arm only handles `ExprKind::Variable`
  targets today" was Task 9's own gap, closed by Task 9 itself. Rewrote to say the helper stays for
  the same isolation reason as S3, not to work around a since-closed gap.
* **S5** (`run.rs`, `Flow::Leave`'s own doc): cited a test named `leave_and_iterate_survive_a_goto_
  absorbing_enclosing_if`; the actual test is `leave_and_iterate_survive_a_do_nested_in_an_ifs_then_
  iterating_repeatedly`. Corrected the citation.
* **S6** (`run.rs`, `indent_in_range`'s fallback comment): cited "`run.rs:536` cites the same
  reasoning"; that line number had rotted (now inside `step`'s own signature) even before this
  round's edits shifted every later line further. Replaced the line-number self-citation with a
  named one (`clause_site`'s own fallback, this file), which cannot rot the same way.
* **S7** (`run.rs`, `Label` arm): "nothing in 4a writes to the trace sink yet -- Task 13's own
  construct" was stale since Task 13 shipped; a `Label` clause *is* echoed now via `step_in_temps_
  frame`. Rewrote to say so.
* **S8** (`lib.rs`, `INTERPRETER_STACK_BYTES` doc, fourth bullet): described `step` recursing through
  `run_bounded` "once per source nesting level of `IF` or `SELECT`" without mentioning Task 11's
  `DO`/`LOOP`, which added the identical recursion shape (`run_loop`/`run_repeating` each drive their
  own `run_bounded` calls one level deeper, per lexical nesting level). Added `DO`/`LOOP`/`SELECT
  WHEN`/`WHEN CASE` to the bullet's own list of what costs a level here.
* **S9** (`run.rs`, `indent_in_range`'s `Do` arm): `Loop::end`/`Do::end` was unwrapped with
  `.expect("an End's closes is only None while its body is still being assembled")` -- that message
  states `End::closes`'s own invariant, a different field's. `run_loop`'s own `.expect` on the
  identical field (`body.end`) has the correct text ("an unclosed DO/LOOP is error 14.1/14.5 ...").
  Copied the correct message to the second site.

### F-EX3 (Minor): panic-vs-loud inconsistency on the same parser invariant

`Select`'s listed-`WhenCase` path panicked on a missing `case` expression
(`.expect("a WhenCase's enclosing Select always carries a case expression")`) while the absorbed-
`WhenCase` arm explicitly refuses to panic on the identical invariant (a plain `SELECT` with no
`CASE` at all, which the parser should never produce a `WhenCase` node under -- only `SELECT CASE`
ever builds one). The review's own framing: one of the two is wrong by this crate's own rule against
turning an unproven parser invariant into a crash, and the `.expect` predates the rule's later
application. Rewrote the listed path to match the absorbed one's own fallback: on a missing
`case_text`, evaluate `values` for side effects and treat the `WhenCase` as not matched, never
panicking, exactly the pattern already established one arm below. Not independently unit-tested --
the fallback path is unreachable through the parser on either side (same invariant, same
unreachability), so testing it would need a hand-built `AST` bypassing `parse_program` entirely,
disproportionate for a Minor whose own fix is "stop panicking on an already-unreachable case."

### F-EX4 (Minor): temps-frame growth is unbounded across a long-running loop

Not a correctness defect (nothing collects mid-run, and the temps are rooted throughout), but
`step_in_temps_frame`'s own doc comment claimed "one clause is the right lifetime for a temporary,"
and since Task 11 a `DO`/`LOOP`'s entire multi-pass execution resolves inside that one clause's
frame -- everything pushed per pass (`eval_condition`'s own `push_temp` for every `WHILE`/`UNTIL`
test) accumulates for the loop's whole run, not one iteration's. Documented at the doc comment
itself rather than restructured: a `do while` loop running 10^7 passes now explicitly discloses that
it holds ~10^7 dead-but-rooted temps in one frame, and that "one clause" describes every instruction
here except this one.

### Final re-verification, this round

* `cargo test -p rexx-exec --lib`: **180** passed, 0 failed.
* `cargo test --workspace`: every `test result: ok`, 0 failed, across every crate in the workspace.
* `cargo test -p rexx-exec --test corpus`: **29 of 29**, report mode (reconciles the coordinator's
  own count against this task's own tracking, which had last checked 26 of 26 before other
  concurrently-landing tasks in the shared worktree added corpus entries).
* `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`: **29 of 29**, strict mode.
* `cargo clippy --workspace --all-targets -- -D warnings`: clean.
* `rustfmt --edition 2024 <path>` on all three touched files (`run.rs`, `lib.rs`, `eval.rs`): clean,
  no unexpected reformatting beyond this round's own edits.
* Both new F-EX1 tests and both new F-EX2 tests independently mutation-verified (fix reverted,
  predicted wrong output reproduced exactly, fix restored, green again) -- see each finding's own
  section above for the specific mutation and the exact wrong value it produced.

## Review round 5: three findings from the value-model slice (`branch-review-value.md`, F1-F3)

Same branch review as round 4, different slice -- `rexx-exec/src/{value,stem,plan,activation,error,
trace}.rs` plus `rexx-core`/`rexx-num`/`rexx-parse`/`rexx-extract`, which found three `run.rs`
findings the exec-slice reviewer never saw because they live in a different file's own review scope.
`F4`/`F5` from the same document are in `value.rs`/`stem.rs`, outside this round's permitted files
(`run.rs` only, plus the report), and not addressed here.

### F3: repeat count and `FOR` count validated under a fixed digits 9, not current `NUMERIC DIGITS`

`whole_nonneg` converted via `rexx_num::ARGUMENT_DIGITS` (18) on the reasoning "a loop bound is no
more digits-limited than `EXIT`'s own result is" -- wrong by measurement: `EXIT` genuinely does
convert under `ARGUMENT_DIGITS` (unaffected by this fix, `exit 12345` under `digits 3` still matches
the oracle, rc 57 both sides) but a loop bound does not. The oracle's own `ForLoop::setup`
(`DoBlockComponents.cpp` ~80-100) rounds under the *current* `NUMERIC DIGITS` before asking whether
the result is whole -- `TRACE`'s own rule (`Number::whole_value`'s own doc comment states the
contrast directly), not `EXIT`'s.

Fix: `whole_nonneg` now reads `self.activation().settings.digits()` and converts under that instead
of the fixed constant. Rewrote the function's own doc comment to state the correction and the
citation, rather than leave the corrected code under a rationale that argues the opposite.

Measured against the oracle, both before and after: `numeric digits 3; do 12345; end` is error 26.2
rc 230 on both sides after the fix (ours ran clean, rc 0, before it); `numeric digits 3; do i = 1 to
99999 for 12345; end` is 26.3 rc 230 on both sides after the fix. Also re-checked the review's own
"converse edge" -- `numeric digits 20; do i = 1 to 3 for 123456789; ...; end` -- which the fixed-9
rule would have rejected (`whole_value(9)` fails on a 9-digit count) where the oracle runs it: both
sides now print `1`, `2`, `3`, `after`, rc 0.

One new test, `run.rs`:
`a_repetition_or_for_count_is_validated_under_the_current_digits_not_a_fixed_width`. Mutation killed:
reverting `whole_nonneg` to `rexx_num::ARGUMENT_DIGITS` makes both probes run clean instead of
raising, since `12345` fits comfortably under 18 digits and only fails to fit under 3 -- verified by
reverting.

### F2: `NUMERIC DIGITS`/`FUZZ`/`FORM` with an expression never traced `>K>`

`RexxInstructionNumeric::execute` calls `traceKeywordResult` for `DIGITS`/`FUZZ`/`FORM` alike
whenever an expression is present (`NumericInstruction.cpp:98`/`135`/`174`); `exec_numeric` never
called `trace_keyword` at all. Worth recording *why* this slipped past: this task's own earlier
report enumerated `traceKeywordResult`'s callers as "`DO`'s control-clause components and `SELECT
CASE`'s `CASE`", and the per-task review then checked our behaviour against that enumeration rather
than against the C++ -- an omission in a list one supplies becomes invisible to an audit that trusts
the list. Re-derived the caller set from the C++ directly this round rather than from the old list.

Probed all three settings' bare and expression forms against the oracle before writing anything, to
find the exact gate (not "always traces" or "always doesn't"):

* `numeric digits 9`/`numeric fuzz 2` (expression present): both trace `>K>`.
* `numeric digits`/`numeric fuzz` alone (no expression, "restore the previous value"): neither
  traces anything.
* `numeric form scientific`/`engineering` (a parser-fixed keyword, no expression ever evaluated):
  never traces.
* `numeric form value 'engineering'` (`FormValue`, the one `NUMERIC FORM` shape that *does* evaluate
  an expression): traces `>K>   "FORM" => "engineering"` -- untranslated, matching `set_form_str`'s
  own no-uppercasing rule for this one path.
* `numeric digits 'x'` (expression present, then invalid): traces `>K>   "DIGITS" => "x"` and *only
  then* raises 26.5 -- the trace fires before validation, the same order `setup_controlled` already
  uses for `TO`/`BY`/`FOR`.

Fix: `numeric_operand` (shared by `DIGITS`/`FUZZ`) now takes the keyword string and calls
`trace_keyword` right after evaluating, before returning the text `set_digits_str`/`set_fuzz_str`
validate -- exactly when `expression` is `Some`, never when it is `None`. `FormValue`'s own arm
(never routed through `numeric_operand`, since it has its own error and no-uppercasing handling)
gets the identical call inline, at the identical point, before its own `set_form_str` call.

Verified against the oracle: three probes (all three settings together under `trace r`; the bare
forms; the invalid-`DIGITS` error path) match byte-for-byte on stdout and stderr independently.
(Comparing merged `2>&1` output showed one interleaving difference, `done` printed before its own
trace lines instead of after -- confirmed as the already-disclosed, unobservable stdout/trace-sink
interleaving gap noted in the branch review's own "Not reached" section, not a new defect: stdout
and stderr independently are each byte-for-byte identical.)

One new test, `run.rs`: `numeric_digits_fuzz_and_form_value_trace_k_only_with_an_expression`.
Mutation killed twice, independently: removing `numeric_operand`'s own `trace_keyword` call drops
both `DIGITS`'s and `FUZZ`'s `>K>` lines (they share the function) while `FORM`'s stays; removing
`FormValue`'s own inline call drops only `FORM`'s while `DIGITS`/`FUZZ` stay -- both verified by
reverting in turn and confirming the exact predicted line goes missing.

### F1: controlled-loop `initial`/`to`/`by` stored as exact parses, not rounded at loop entry

`setup_controlled` stored `initial`/`to`/`by` as their exact parse (`arith_operand`, no rounding).
The oracle rounds all three with a prefix `+` (unary plus) at loop entry (`ControlledLoop::setup`,
`DoBlockComponents.cpp:126-166`, `callOperatorMethod(OPERATOR_PLUS, ...)` for each). Masked while
`NUMERIC DIGITS` stays constant (every later use re-rounds to the same width, so exact and rounded
render identically); becomes a wrong answer the moment digits widens inside the loop body, because
the *stored* value is what carries forward, and only the rounded one is supposed to.

Fix: a new free function, `round_via_unary_plus(number: &Number, digits: u64) -> Result<Number,
ArithError>`, `Number::zero().add(number, digits)` -- deliberately not `Number::round_to`, which
rounds but is not what unary `+` means (`round_to`'s own doc comment draws the distinction; `eval.rs`'s
own `PrefixOp::Plus` arm computes the identical sum, for an `ObjRef` this function has no need to
produce, since `LoopState::Controlled`'s own fields are `Number`). `setup_controlled` reads `NUMERIC
DIGITS` once, at entry (nothing between the reads can change it -- a controlled loop's own header
evaluates expressions, never instructions), and applies the rounding to `current` (from `initial`)
and to `to`/`by` when either is actually present, matching exactly where the C++'s own citation
places the three calls. The trace itself is unaffected: measured, `>K>   "TO" => "9.87654"` still
shows the raw, unrounded evaluated text -- the trace fires before this rounding, same as it already
did before F1.

Measured against the oracle, both probes from the review:

* `numeric digits 3; do i = 1.23456 to 3; say i; numeric digits 9; end` -- oracle and ours (after the
  fix) both `1.23`, `2.23`; ours before the fix gave `1.23`, `2.23456` (the exact parse surviving
  into the wider-digits second pass).
* `numeric digits 3; do i = 1 to 9 by 1.2345; say i; numeric digits 9; end` -- oracle and ours (after
  the fix) both `1`, `2.23`, `3.46`, `4.69`, `5.92`, `7.15`, `8.38` (seven values, stopping because
  the eighth, `9.61`, is past the bound `9`); ours before the fix accumulated the exact `1.2345`
  (`1`, `2.2345`, `3.469`, ... diverging further each pass).

A third, full `TRACE R` transcript comparison (`numeric digits 3; do i = 1.23456 to 9.87654 by
1.2345; say i; end`) confirmed the fix does not touch trace output and found no new divergence: the
only remaining diff, both before and after this fix, is the already-disclosed Controlled-loop
`>>>`-pair trace gap (this file's own known-gap section, `t13_doto_trace.rex`) -- confirmed
unrelated to F1 by reverting the fix and observing the identical `>>>`-pair gap present alongside a
*separate*, now-fixed numeric drift (`2.47`/`3.70`/... instead of `2.46`/`3.69`/...) that disappears
once the fix is restored.

One new test, `run.rs`: `a_controlled_loops_header_values_round_at_entry_not_the_exact_parse`, both
sub-cases measured against the oracle before being written into the test, not derived by hand.
Mutation killed twice, independently: removing the `round_via_unary_plus` call on `current` makes
the `to`-probe's second line read `2.23456` (the exact parse) instead of `2.23`; removing the one on
`by` makes the `by`-probe accumulate the exact `1.2345` per pass instead of the rounded `1.23` --
both verified by reverting in turn.

### Final re-verification, this round

* `cargo test -p rexx-exec --lib`: **183** passed, 0 failed.
* `cargo test --workspace`: every `test result: ok`, 0 failed.
* `cargo test -p rexx-exec --test corpus`: **29 of 29**, report mode.
* `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`: **29 of 29**, strict mode.
* `cargo clippy --workspace --all-targets -- -D warnings`: clean.
* `rustfmt --edition 2024 crates/rexx-exec/src/run.rs`: clean, only this round's own edits.
* All three fixes independently verified against the real oracle (not only by unit test) before any
  test was written, using probes taken from the review document's own measured transcripts plus one
  additional converse-edge probe (F3) and one additional full-transcript probe (F1) this round added.

STATUS: DONE.
