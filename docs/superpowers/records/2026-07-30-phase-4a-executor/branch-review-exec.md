# Branch review: execution core (run.rs, eval.rs, lib.rs)

STATUS: DONE

Scope: `rust/crates/rexx-exec/src/run.rs` (5598 lines), `src/eval.rs` (1543), `src/lib.rs` (1233), range `9f68662a..HEAD`.
Focus per brief: cross-task interactions (run_bounded Goto absorption x block-stack unwinding x trace hooks), set-once `Interp` field hygiene, stale comments and citations, silent-failure paths.

## Findings

### F-EX1 (Important): F3's absorbed-`WhenCase` escape bypasses `leave_select`, so a `LEAVE`/`ITERATE` naming the enclosing `SELECT LABEL` from inside `OTHERWISE` misresolves

Cross-task interaction: Task 11 x Task 13's F3, exactly the class the per-task reviews could not see.
Task 11 deliberately rerouted `OTHERWISE`'s body through `Select`'s own `run_bounded` + `leave_select`
(`run.rs` ~899-942, the comment "used to be a plain `Goto` onto the OTHERWISE marker ... wrong for
LEAVE/ITERATE") so the `SELECT` can recognise its own label.
Task 13's F3 fix (`InstructionKind::WhenCase` arm, `run.rs` ~1069-1092) reintroduces the pre-Task-11 shape on the escape path:
a false absorbed `WhenCase` returns `Flow::Goto(false_target)`, which exits the `Select` arm entirely
(`leave_select`'s `other` arm forwards it), lands on the `Otherwise` marker, and the `OTHERWISE` body then
runs under the *outer* loop, outside any `leave_select`.

Measured (probes in scratchpad, `c_f3_otherwise_leave.rex` / `f_f3_otherwise_iterate.rex`):

```text
select label s case 2 / when 2 then / when 3 then nop / otherwise say 'O' / leave s / ... / end / say 'after'
  oracle: O, after, rc 0        ours: O, then Error 28.3, rc 228
same shape with `iterate s` in the OTHERWISE:
  oracle: Error 28.5, rc 228    ours: Error 28.4, rc 228
```

A second-order effect of the same bypass: the `SELECT` also never applies `pop_search_frame` to a
`Leave`/`Iterate` forwarded past it on this path, so 28.x residual indents can differ from the oracle too
(not separately probed).
Reachability is narrow (SELECT LABEL + CASE + false absorbed `WHEN CASE` + named LEAVE/ITERATE inside
OTHERWISE; no corpus program), but it is a hard behavioral divergence, and the first probe turns a
clean oracle run into an error on our side.

### Verified clean (same probe batch)

* Bare `LEAVE` inside a labelled `Simple` block and inside `SELECT LABEL`: oracle raises 28.1 in both
  (despite the catalogue text naming "labeled block"), ours identical, byte for byte, rc 228.
  `do_body_outcome`'s `None => is_loop` is correct as written.
* Controlled loop `FOR`-exhaustion binding: `do i = 1 to 100 for 3 / end / say i` prints `4` on both
  sides, confirming `loop_advance`'s bind-before-budget-check claim ("for either reason") that the
  comment asserts but the tests only pin for the `TO` bound.
* `step` has exactly one non-test caller, `step_in_temps_frame` (`run.rs:1255`), and it pops the temps
  frame unconditionally around the call. The invariant the six `eval.rs` functions rely on holds.

### F-EX2 (Important): `Flow::Leave(SymbolId)`/`Iterate(SymbolId)` cross the `run_fragment` boundary carrying a fragment-table id

`run_fragment` (`run.rs:2394`) runs a fragment through `run_bounded`, whose catch-all forwards
`Flow::Leave`/`Iterate` out of the fragment (only `Goto` is range-checked; `Exit` is documented to
propagate). A `leave x` inside `INTERPRET` text produces `Flow::Leave(Some(id), ..)` where `id` is
interned in the *fragment's* `SymbolTable`. Every consumer above the fragment (`do_body_outcome`,
`leave_select`, `run_activation`) compares that id against labels interned in the *program's* table
(`label == Some(n)`), and the 28.3/28.4 error path resolves it via `code.symbols.name(n)` with the
program's table. `SymbolId`s are table-relative (`Code::slots`' own doc says a fragment's ids are the
fragment's), so this can silently match the wrong label, miss a right one, or panic/name the wrong
symbol in `raised_leave_no_match`. Additionally, a `LeaveOrigin` born in a fragment has `site: None`,
so a 28.x escaping through `INTERPRET` reports `<no failing clause recorded>` instead of the enclosing
`INTERPRET` clause (`Flow::Leave` is `Ok`, so the INTERPRET's `step_in_temps_frame` never records a
site the way it does for a `Raised`).

Reachable only through `run_program_interpret_spike` today (the `INTERPRET` keyword is Loud under
`run_program`), so no shipped behavior is wrong; but 4b builds `INTERPRET` on exactly this machinery
and will inherit the trap. `run_fragment`'s doc covers only the inward direction (no label targets
*inside* a fragment); the outward direction is undocumented. Recommend at minimum a comment, at best
mapping or refusing named `Leave`/`Iterate` at the fragment boundary.

### Stale comments (measured citation-drift problem, continued)

* **S1 (Important)** `lib.rs:575` ff., `Interp::trace` field doc: "the trace sink ... **which nothing
  in this crate writes yet**. ... Task 13 is the first to write to it." Task 13 shipped in this range;
  `trace.rs` and a dozen `run.rs` sites write to it. The field doc is now false on its central claim.
* **S2 (Important)** `run.rs:45-47` (module doc): "`Then`, `Else`, `Otherwise`, `When` and `WhenCase`
  accordingly step as pure no-ops (like `Label`) -- they are never independently dispatched". False
  since the absorbed-`WHEN` fix and F3: `When`'s arm evaluates its condition (can raise 42.3), and
  `WhenCase`'s arm evaluates, compares against `current_case_text`, and *branches* on the false side.
  The `Select` arm's own doc (`run.rs` ~768-772, "a `When`/`WhenCase` node must never be independently
  stepped for a decision of its own") is contradicted by the same code below it.
* **S3 (Minor)** `eval.rs:1349-1356` (tests, `eval_condition` helper doc): "none of those instructions
  run yet -- Tasks 9-11's `step` dispatch does not reach `IF` -- so a real program can only ever hand
  the parsed condition to this test directly, never through `run`". Tasks 10/11 shipped; `IF`/`WHEN`/
  `WHILE`/`UNTIL` all run and comma lists are reachable through `run` (run.rs's own tests do it).
* **S4 (Minor)** `eval.rs:844-847` (tests, `activate` helper doc): "`step`'s `Assignment` arm only
  handles `ExprKind::Variable` targets today (`Stem`/`Compound` are Task 9's dispatch)". Task 9
  shipped; `run.rs`'s `Assignment` arm handles all three.
* **S5 (Minor)** `run.rs:111`: `Flow::Leave`'s doc cites test
  `leave_and_iterate_survive_a_goto_absorbing_enclosing_if`; the test is named
  `leave_and_iterate_survive_a_do_nested_in_an_ifs_then_iterating_repeatedly` (`run.rs:5300`).
* **S6 (Minor)** `run.rs:2957` (`indent_in_range`'s fallback comment): cites "`run.rs:536` cites
  the same reasoning"; line 536 is now `source: Option<&ProgramSource>,` inside `step`'s signature.
  A line-number self-citation into the same file, already rotted.
* **S7 (Minor)** `run.rs:693-695` (`Label` arm): "nothing in 4a writes to the trace sink yet --
  Task 13's own construct". Stale since Task 13; a `Label` clause *is* echoed now via
  `step_in_temps_frame`.
* **S8 (Minor)** `lib.rs:144-160` (`INTERPRETER_STACK_BYTES` doc, fourth bullet): "`step` recursing
  through `run_bounded` once per source nesting level of `IF` or `SELECT`". Task 11 added `DO`/`LOOP`
  (`run_loop`/`run_repeating`/`run_bounded` per nesting level); the bullet was not updated.
* **S9 (Minor)** `run.rs:2846-2848` (`indent_in_range`'s `Do` arm): `Loop::end` is unwrapped with
  `.expect("an End's closes is only None while its body is still being assembled")` -- the message
  states `End::closes`' invariant, a different field's; `run_loop`'s expect on the same field
  (`run.rs:1549-1551`) has the correct text ("an unclosed DO/LOOP is error 14.1/14.5 ...").

### F-EX3 (Minor): panic-vs-loud inconsistency on the same parser invariant

`Select`'s listed-`WhenCase` path panics on a missing case expression
(`run.rs:872-874`: `.expect("a WhenCase's enclosing Select always carries a case expression")`) while
the absorbed-`WhenCase` arm explicitly refuses to panic on the identical invariant
("kept as a fallback ... rather than an `unreachable!`, on this crate's own rule against turning an
unproven parser invariant into a crash", `run.rs:1059-1068`). One of the two is wrong by the crate's
own rule; the `expect` predates the rule's later application and was never revisited. The same tension
exists at `exec_trace`'s `Trace::Setting` expect (`run.rs:2560-2562`), there with an argued
parse-time-validation justification.

### Set-once `Interp` field audit (brief item 2)

* `failure_site`: every writer (`record_failure_at`, `record_leave_failure`) is first-wins guarded;
  `execute` takes it unconditionally before matching on the result, so it cannot leak across uses
  (and each `execute` owns a fresh `Interp`). Clean. One gap folded into F-EX2: a 28.x whose origin
  is inside a fragment records no site at all and falls to `execute`'s
  `<no failing clause recorded>` fallback.
* `pending_escape_indent`: set only by the absorbed-`WhenCase` false branch; consumed by `.take()` at
  the top of every `step_in_temps_frame`. Between set and consumption only infallible `Goto`
  propagation runs (verified by tracing every path: `leave_select`'s `other` arm, `run_bounded`'s
  return, `Select`'s arm return), so it cannot be attributed to an unrelated failure. The one
  unconsumed path is an escape `Goto` whose target is the end of the body (program simply ends);
  benign, nothing reads it after the run.
* `current_case_text`: set unconditionally by every `Select` step. The disclosed nested-clobber
  limitation (`lib.rs:636-643`) is in fact conservative: an absorbed `WhenCase` is only ever stepped
  through the range `[listed_when+1, listed_false_target)`, which contains nothing but the `Then`
  marker and the absorbed node itself, so nothing can run between the enclosing `Select`'s own set
  and the read. Two nits: the disclosure names only a nested `SELECT CASE`; a nested plain `SELECT`
  clobbers to `None` (fallback arm) rather than to wrong text, a different failure shape. Both appear
  unreachable today.
* `current_value_indent`: ambient rather than set-once; every site that evaluates user expressions is
  preceded by a setter (`step_in_temps_frame` unconditionally; explicit overrides at the `WHEN` scan,
  `OTHERWISE` echo, `WHILE`, `UNTIL`). No stale-read path found.

### Citations verified accurate (all checked by locating the named function, not the named line)

`RexxActivation.cpp:4791-4802` (`evaluateLocalCompoundVariable`, `traceCompoundName`+`traceCompound`
at 4798-4801), `SelectInstruction.cpp:372` (`traceKeywordResult(CASE, ...)` exactly),
`IfInstruction.cpp:140` (`traceResult` in `RexxInstructionIf::execute`), `DoBlock.cpp:182`
(`DoBlock::checkControl` signature), `WhenCaseInstruction.cpp:154`/`158` (the two `traceResult`
calls exactly), `LanguageParser.cpp:1319` (inside the `KEYWORD_WHEN` case), `compare.rs:154`
(`if op.is_strict()` exactly), `RexxActivation::leaveLoop` (exists, `RexxActivation.cpp:1171`),
`RexxInstructionSelect::isLoop` (exists, `SelectInstruction.cpp:154`),
`ThenInstruction.cpp`/`ElseInstruction.cpp` (`indent(); trace; indent();` as claimed),
`OtherwiseInstruction.cpp` (trace at one bump, as claimed),
`RexxInstructionExpression::evaluateStringExpression`, `NumberString::int64Value`,
`Numerics::objectToSignedInteger` (all exist as described). The measured citation-drift problem did
not recur in this slice's external citations; the two rotted references are internal (S5, S6).

### F-EX4 (Minor): temps-frame growth is unbounded across a long-running loop

`step_in_temps_frame` opens one temps frame per *clause*, but since Task 11 a `DO`/`LOOP` resolves its
entire multi-pass execution inside that one clause's frame. Everything `run_repeating` pushes per pass
accumulates until the loop ends: `eval_condition`'s `push_temp` for every `WHILE`/`UNTIL` test
(one `ObjRef` per iteration). A `do while ...` loop running 10^7 passes holds ~10^7 temps in one frame.
Not a correctness defect today (nothing collects mid-run, and under stress-collect the temps are
*rooted*, which is safe), but the doc's "One clause is the right lifetime for a temporary" no longer
describes what a loop clause is, and a future collector gains nothing from roots that should be dead.
Body instructions are unaffected (each gets its own frame).

## Not reached

* `trace.rs`, `value.rs`, `stem.rs`, `plan.rs`, `activation.rs`, `error.rs`: read only where a
  slice-file comment pointed into them; not reviewed (other slices or out of scope).
* F-EX2 was established from the type structure (`SymbolId` is table-relative, `run_bounded`'s
  catch-all forwards `Leave`/`Iterate` out of `run_fragment`) plus the oracle's semantics for
  `LEAVE` through `INTERPRET`; I did not build a harness against `run_program_interpret_spike` to
  execute the mis-resolution, since `rexx-run` cannot reach it (INTERPRET is Loud there).
* TRACE R byte-level output on the F3 escape path (whether the landed-on `END`'s `*-*` echo should
  also sit at the residual indent, not only the `FailureSite`): not probed.
* `static_indent`'s full additive model was not re-derived against the oracle beyond the existing
  test suite; likewise the `Controlled` loop's disclosed two-line `>>>` trace gap
  (`run.rs:2053-2081`) was taken as disclosed, not re-measured.
* The 28.x residual-indent divergence on the F-EX1 escape path (the `pop_search_frame` the bypassed
  `leave_select` never applies) is asserted from code structure; the two probes pin the sub-code
  divergence, not the indent one.
* Deep lexical `DO` nesting stack cost (S8's subject) not re-measured.
* `eval.rs`'s numeric semantics (rounding, DIGITS/FORM interplay) were not re-audited against
  `rexx-num`; prior tasks and the differential corpus own that.

## Probe artifacts

`a_bare_leave_labelled_block.rex`, `b_bare_leave_select_label.rex`, `c_f3_otherwise_leave.rex`,
`d_for_exhaustion_binding.rex`, `e_normal_exhaustion.rex`, `f_f3_otherwise_iterate.rex` in this
session's scratchpad, each run against both the oracle (`ulimit -v 1048576`) and
`cargo run -q -p rexx-exec --bin rexx-run`.
