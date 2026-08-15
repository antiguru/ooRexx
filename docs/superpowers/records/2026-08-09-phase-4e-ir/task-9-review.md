# Task 9 review: promote arithmetic, with the patch table as its consumer

Reviewed `e63e8a00..dd0c4dcb`.
`f712de19` is on the branch above it and touches the plan only.

**Spec compliance: PASS.**
**Quality: high.** No correctness defect found.
Three findings, all prose or coverage; none changes an answer.

Everything below was run, not read.
All builds and all mutations were done in a private copy of the tree (`.../scratchpad/tree`), because other agents are live in this session; the repository working tree is untouched and `git status` is clean.

## Gates, re-run from a cold target directory

* `cargo test --workspace`: **1417 passed, 0 failed** (4 ignored), matching the report and the brief's BASE + 6.
* `cargo test --workspace --release`: **1417 passed, 0 failed** -- the distinct `lto = "fat"` gate.
* `cargo fmt --all --check`: exit 0. `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
* No `unsafe` added; no em-dash in any added line.

## 1. The `DIGITS` guard, by construction and by running

By construction: `small_int_arith` (`eval.rs:1085`) rejects on `!within_digits(left, digits) || !within_digits(right, digits)` **before** the operator, and `exact_small_int` re-checks the result.
Both operands are checked.
`value.rs:439`-`:448` already carries the `1000 - 25` reasoning and the `ootest` citation, and this task did not touch `value.rs`.

By running, against the oracle with the wrapper and from fresh empty directories:

* `numeric digits 3 / say 1000 - 25 / say 999 + 6 / say 500 * 500 / say 0.5 - 25` -- oracle `980 / 1.01E+3 / 2.50E+5 / -24.5`, byte-identical on both engines.
* The loop form the brief calls subtler -- `zn = 999; do 6; say zn; zn = zn + 6` at `DIGITS 3` -- oracle `999 / 1.01E+3 / 1.02E+3 / ...`, byte-identical on both engines.
  An implementation carrying the exact sum forward prints `1.01E+3` twice; this one does not.
* `corpus/lang/loop_control_rounding.rex`, the named witness: stdout **and** stderr byte-identical to the oracle on both engines.
  It is listed in `corpus/phase-4a.txt`, so it is inside the dual sweep's corpus population rather than only reachable by hand.
* A site quickened and then widened *and narrowed again* (`zn` 1 -> 1000 -> 2 -> 0.5 -> 7 at `DIGITS 3`): oracle-identical on both engines, so monotone demotion is not observable in output.
* A chunk re-entered under a different `DIGITS` -- one routine called four times with `numeric digits 3` taken in the third call -- oracle-identical on both engines.
  This is the shape where a hint recorded under one precision could be read under another; the re-check is what makes it safe, and it is.

Falsified: dropping the operand half of the check (`if false` in place of the `within_digits` pair) reddens `eval::tests::an_operand_too_wide_for_the_precision_leaves_the_fast_path`, `eval::tests::the_small_int_fast_path_answers_what_the_general_path_answers`, both `ootest` exempt-set assertions, this task's own case file at `ir_dual_cases/arithmetic:104`, and the precondition test.
So the guard is load-bearing, and -- as the report's own honest negative says -- the precondition test is **not** the unique catcher for that particular mutation.
The mutated binary makes `say 1000 - 25` print `975`.

## 2. The patch table's discipline

The op stream is immutable: `Chunk::ops` is written once by `compile` and never rewritten; the only mutable state a running chunk owns is `Chunk::hints` (`ir/mod.rs:700`-`:755`), read only inside `Op::Arith`'s arm (`drive.rs:681`).
No other arm loads or stores a slot, and the table is dense over arithmetic ops rather than parallel to the stream, so no counter walks beside the region loop -- which is what makes "an op that never quickens pays nothing" true of the loop and not only of its arms.
The departure from the Interfaces line's word "parallel" is the shape the brief's own width paragraph prescribes two paragraphs later, and the report says so.

**The hint cannot change an answer.**
Three independent lines of evidence:

* By construction: `arith_small_int` (`eval.rs:736`) re-reads `DIGITS`, re-decodes both operands and re-checks both, on every execution; it has no side effect and returns `None` for every case where the two paths could disagree.
  `arith_general` is correct for every operand the small path accepts.
  A hint chooses the order and nothing else.
* By the most aggressive mutation available: `Hints::tries_small_int` forced to `false` -- every site skips the fast path forever -- leaves **1416 of 1417 tests green**, including the 10391-program engine-vs-engine sweep and every oracle-recorded expectation.
  The one red is the precondition test's skip count.
* By probe: the demotion and re-entry programs above.

I could not construct a program in which the recorded hint makes the two engines disagree, or makes one disagree with the oracle, and the argument above says why one cannot exist.

**Exit criterion 5's switch is one build.**
`const QUICKENING: bool = false` (`ir/mod.rs:633`) is the whole edit: I built and ran it -- **1417 passed, 0 failed**, clippy exit 0 -- so the measuring build needs no second change and no lint repair.
`Hints::next` is kept separately from `slots.len()`, so the op stream a golden reads is the same under either setting and the switch measures the table rather than also moving what compiled.

**The table is read in production**, which is what exit criterion 5 exists to establish before it measures anything.
Two mutations of the mechanism -- `saw_general` storing nothing, and `tries_small_int` answering `false` -- are each caught by `ir::drive::tests::a_quickened_site_falls_through_to_the_general_path` and by **nothing else in the workspace**.
That is the "can fail is not adds coverage" check done the expensive way, and the new test passes it.

## 3. The error paths in the region loop

The region is a labelled block (`drive.rs:509`-`:944`) whose value reaches `leave_stepped_clause` at `drive.rs:945`.
**The block contains no `?` and no `return` at all** -- grepped for both over the exact line range, zero matches -- so every exit is a `break 'region`, and the leave runs on each.
`Op::Arith`'s failure path is `break 'region Err(failure)` (`drive.rs:745`), which is that discipline.

Each of the three raises run from a promoted clause, against the oracle:

* 41.1 from a non-numeric operand mid-chain -- `say za * 2 + zb + 4` under `signal on syntax` traps `41` at `sigl 4`; case-file row, and re-measured.
* 26.8 from `say 2 ** 2.5` -- case-file row, message and line correct.
* Division by zero -- `say za / zb` reports `Error 42.3` against line 3 with rc 214, and `say za % zb` under a trap gives `trapped 42 at 4`.
  Byte-identical to the oracle on both engines.

The leave having run is visible and was checked: under `trace i`, a 41.1 raised inside a promoted clause **inside a loop**, trapped, resumed by `signal`, leaves every later clause at the oracle's own indent, byte for byte.
An unbalanced clause boundary is exactly what would drift there.

## 4. The trace hazard

`Op::TraceOperator` exists (`ir/mod.rs:485`), is emitted immediately behind its `Op::Arith` reading the same register (`compile.rs:812` and `compile.rs:822`), and `compile::assert_operator_echoes_follow_their_op` (`compile.rs:1139`) asserts position, register **and** operator unconditionally at the end of every compile.
`trace_intermediate`'s `Binary` arm covers every binary family, which is why `echo_operator` takes an `Operator` rather than the arithmetic subset; the case file's "every other binary family" row pins `||`, `=`, `&` and blank concatenation still emitting `>O>` from `eval.rs`.

Falsified, at HEAD: with the emission made a no-op, `both_engines_agree_across_every_population` reddens naming **`corpus lang/trace_output.rex`** -- a program that predates this task -- and `both_engines_agree_on_every_case_file` reddens too.
So the catch is not self-supplied, and I established that by reading which program the sweep names rather than by deleting the case file.
A second mutation, the echo ignoring its own gate, reddens four dual tests.

## 5. The sharing rule

Structural: `>O>` has exactly one production emitter (`trace.rs:777`), reached by `echo_operator` (`trace.rs:761`) from `eval.rs:265` and `drive.rs:763` and from nowhere else; `arith_small_int` and `arith_general` each have exactly two production callers, `eval_arithmetic` and `Op::Arith`; `small_int_arith` has one.
`is_arithmetic` (`eval.rs:1162`) is the guard on `eval_node`'s own arm and the compiler asks it, so the promoted set cannot drift from the computed one -- `only_the_arithmetic_operators_promote` is the adjacent-success test for that.

Behavioural, which is the half that matters -- perturb inside the shared code and the two engines must move together:

* Inverting `NUMERIC FORM` inside `arith_general`: 12 recorded expectations red, **`both_engines_agree_across_every_population` green**.
* Loosening the precision inside `arith_small_int` (`digits + 1`): 6 red, **population sweep green**.
* Replacing the tag inside `echo_operator`: 6 red including three trace-oracle transcripts, **population sweep green**.
  `both_engines_agree_on_every_branch_shape` reddens with "the tree-walker's own trace moved", which is its recorded-expectation half, not an engine disagreement.

The control for all three: an IR-only perturbation -- swapping `lhs` and `rhs` in `Op::Arith`'s arm alone -- **does** redden `both_engines_agree_across_every_population`.
So the sweep can see a split implementation, and it does not see one here.

## 6. `Op::LoadConstant`

`drive.rs:593` builds the value as `self.literal(code.symbols.name(*symbol).as_bytes())`, which is `eval_node`'s `Constant` arm (`eval.rs:436`) verbatim.
Its echo is the existing `Op::TraceLiteral`, and `trace_intermediate` does send both node kinds to `echo_literal` (`eval.rs:221`).
`assert_literal_echoes_follow_their_load` was widened to accept either load, and still checks the register.

The cache key did not change: `chunk_for` is still keyed `(BodyKey, ChunkTrace)` and `compile` still takes only the body, the plan and the trace setting.
The `SymbolId` in the op is safe under that key because `BodyKey` carries `ProgramId`, so a cached chunk is only ever run against the symbol table it was compiled from -- the same reason `Op::Load` may carry one.
`Op::Const`'s doc comment was corrected to point at the new op rather than left claiming a `Constant` reaches `EvalExpr`.

The report's reason for needing the op is right, and it is load-bearing: `zw = zv + 1` compiles to `Load / TraceRead / LoadConstant / TraceLiteral / Arith / TraceOperator / Store`, so without it the `1` is unpromotable and the whole expression falls to one `EvalExpr` -- every arithmetic line on every measured axis, measuring as a null result.

`size_of::<Op>() == 12` holds and is still load-bearing: changing the assert to 13 fails the build with `evaluation panicked`.
Nothing was widened; `Chunk::hints` is a side table, which is what the brief prescribed for exactly this.

## 7. The claims the brief asked me to attack

**Monotone demotion is a performance property only.**
Verified by construction and by the two probes above.
One thing the `PatchSlot` doc (`ir/mod.rs:657`) does not say and Task 11 should read it knowing: for a site whose operands *alternate* between small and wide, monotone demotion can make the table **slower than no table at all**, because that site keeps a fast path under `QUICKENING = false` that the hint takes away permanently.
No such site exists on the measured axes -- `t.k = t.k + 1` and `x = x + 1` always succeed, `k = i // tails` and `arith`'s divisions always fail -- so this is a caveat on the instrument, not a prediction change.

**The prediction that the table will not pay is sound**, and its withdrawal of the plan's own heading is correct.
I read the benchmark programs: every arithmetic line in `arith.rex` (`a = i / 3`, `b = a * a - 1`, `c = i / 7`, `d = c ** 2 // 5`, `total = total + b + d`) has a fully promotable operand tree, so the operand recursion goes away there exactly as it does in `varlookup.rex`'s `x = x + 1` and `compound.rex`'s `t.k = t.k + 1`.
The plan's "an `arith` that does not move is this task succeeding" was a statement about P1's change to the *operation*, and this task changed the *operands*; the withdrawal in `dd0c4dcb`/`f712de19` is the right correction and it was made in the plan rather than only in the report.
The argument that a hint can only skip a hopeless attempt follows from D22 forbidding it to remove a precondition, and the precondition here is the whole of the specialisation.

## Findings

* **Important (prose only) -- `rust/crates/rexx-exec/src/ir/compile.rs:697`.**
  `push_value`'s doc lists "a constant symbol" among the expressions that fall to `Op::EvalExpr`.
  The same commit made `ExprKind::Constant` compile natively (`compile.rs:744`, `compile.rs:790`), and `golden_tests.rs:482`'s `a_constant_symbol_is_a_native_load` asserts the opposite.
  This is the doc a later task reads to decide what promotes, and it is false; correct or remove it rather than hedge it.
* **Minor -- `rust/crates/rexx-exec/src/ir/compile.rs:709`.**
  The `#[expect(clippy::too_many_arguments)]` reason accounts for eight of the function's nine parameters -- "four emission sinks and the four facts an `EvalExpr` needs" -- and `plan` is named by neither group.
* **Minor -- `rust/crates/rexx-exec/tests/ir_dual_cases/arithmetic`.**
  No division-by-zero row.
  The file records 41.1 and 26.8 from a promoted clause and not the third raise arithmetic can make; I measured 42.3 working through the same `break 'region` path, so this is a missing regression case rather than a defect.

## Unsettled, and what would settle it

* **Whether the 288-line `arithmetic` case file adds coverage no other test has.**
  Every mutation I ran that it caught -- the loosened precision, the dropped operand check -- was also caught by tests that predate this task, and the mutations only it and the sweep caught were IR-only ones the sweep already catches.
  Its distinct value is real but different: it pins *oracle bytes*, which an engine-against-engine comparison structurally cannot.
  Settled by re-running each candidate mutation with the file moved out of `tests/ir_dual_cases/` and recording which rows are the sole catcher, which is the check `rust/CLAUDE.md` prescribes and which was not run for this file.
* **Every performance claim**, deliberately, per the brief.
  Task 11 owes the two falsifications the code is now shaped for: the `QUICKENING` switch (which I confirmed is one build and green) and the `AtomicU32` price against a plain `u32` read (`PatchSlot`'s three methods are the only place the choice appears, so that is also one edit).
  My reading is that the win will survive `QUICKENING = false`, as the report predicts.
