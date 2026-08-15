# Task 9 report: promote arithmetic, with the patch table as its consumer

**Status: DONE_WITH_CONCERNS.** Everything the brief asked for is built and green. Two things it did not ask for are in the commits and are the reason for the qualifier: an op the task could not be done without, and a withdrawal of the plan's prediction for this task.

BASE `e63e8a00`. Four commits, read back from `git log`:

| hash | what |
|---|---|
| `b3345d91` | Split arithmetic where a compiled op needs to enter it, and guard the seam |
| `ea17a699` | Compile an arithmetic operator into the stream, operands and all |
| `f0d12ebe` | Let an arithmetic site remember which path was worth trying |
| `dd0c4dcb` | Withdraw a prediction made about a change this task does not make |

Suite **1417 passed, 0 failed** (BASE 1411; +1 precondition test, +5 golden tests). `cargo fmt --all --check` clean, `cargo clippy --workspace --all-targets -- -D warnings` clean warm **and** from a cold `CARGO_TARGET_DIR` -- a separate directory rather than wiping `target/`, since other agents are live in this session. `chunks_refused` asserted 0 across the corpus by the dual harness, unchanged.

---

## What was built

**The sharing rule.** `eval_arithmetic` was two blocks; it is now two functions, entered from both engines and never copied:

* `Interp::arith_small_int` -- the small-integer path, which reads `DIGITS`, decodes both operands and calls the existing free `small_int_arith`;
* `Interp::arith_general` -- the `rexx-num` path, operand conversion, the seven operators, 41.1 and `**`'s 26.8.

`eval_arithmetic` is now the frame, the two operand evaluations, and those two in order. `Op::Arith` calls the same two, in the same order, with no frame -- its operands are already rooted by the registers they came from, which is `arith_general`'s stated contract on its caller.

`>O>` got the treatment `>L>` and `>V>` already had: one `Interp::echo_operator` in `trace.rs`, called by `trace_intermediate`'s `Binary` arm and by `Op::TraceOperator`. It takes an `Operator` rather than the arithmetic subset, because every binary family traces `>O>`.

**One enumeration, not two.** `is_arithmetic` moved into `eval.rs` as the guard on `eval_node`'s own arithmetic arm, and `compile` asks it. A compiler promoting one operator more than the interpreter computes would run `arith_general` on a concatenation; there is now no second list to drift.

**Two new ops, plus one the brief did not name.** `Op::Arith { op, hint, lhs, rhs, dst }` and `Op::TraceOperator { op, src }`. `size_of::<Op>()` is still 12 and the assert is untouched: `Arith` is `u32 + 3×u16 + u8 + tag` exactly. The left operand lands in `dst` and only the right takes a register, so a chain holds at two registers however long it runs (asserted: `zw = za + zb + zc + zd` reserves 2, `zw = za + zb * zc` reserves 3).

**The patch table.** `Chunk::hints`, dense over the ops that can specialise and keyed off the op's own `hint` field the way `consts` is -- **not parallel to the op stream**, and that is a deliberate departure from the Interfaces line, which says "parallel". A stream-parallel table needs the op's position, and a region's ops are walked as a slice (`ops_in`), so finding it would mean a counter incremented for every op in every region -- paid by exactly the ops D22 says must pay nothing. The brief's own width paragraph prescribes this shape in the same breath ("a side table keyed off the op the way `Chunk::consts` already is"), so the two sentences disagree and I followed the second.

A slot holds `TRY_SMALL_INT` or `GENERAL`. It decides only which path is *tried first*; both are correct for every operand, the small-integer one re-decodes and re-checks `DIGITS` every execution, and the state moves one way only, so the store happens at most once per site.

## The falsifications

**The trace hazard, run and confirmed at HEAD.** With `Op::TraceOperator`'s emission made a no-op, `both_engines_agree_across_every_population` reddens naming `corpus lang/trace_output.rex`, and `both_engines_agree_on_every_case_file` reddens too. **It stays red with this task's own case file (`ir_dual_cases/arithmetic`) removed from the directory entirely**, so the catch is not self-supplied. Restored.

**Exit criterion 5's switch, built both ways.** `const QUICKENING: bool` in `ir/mod.rs` is the whole of the table's removal. With it `false`: nothing is allocated, no slot is loaded or stored, every site tries the small-integer path and falls through -- which is what the op does with no table at all. I built and ran that configuration: **1417 passed, 0 failed, clippy clean**. The op stream a golden reads is byte-identical under either setting (`Hints` keeps its index counter separately from the slot vector), so the switch measures the table and not also what compiled.

**Exit criterion 3c is a three-line edit in one place.** `PatchSlot` wraps the `AtomicU32` behind `new`/`get`/`set`; those three lines are the only place the choice appears.

**The precondition test carries a witness that it is not vacuous.** Its program runs one pair of sites four times, and every pass but the first falls through for a different reason -- operand too wide (`1000 - 25` is `980`, measured on the oracle, not `975`), result too wide, operand not an integer -- each behind a pass that took the fast path at the identical site. Because both paths print the same bytes, output alone cannot say the table was read, so the test also asserts the skip count. Two mutations of the new mechanism -- a `saw_general` that stores nothing, and a hint that always says general -- are each caught by **this test alone** across the whole workspace.

**One honest negative on coverage.** Dropping `small_int_arith`'s operand check is caught by three tests that already existed (`eval::tests::an_operand_too_wide_for_the_precision_leaves_the_fast_path` among them), so the precondition test adds no coverage *for that* mutation. What it uniquely covers is the quickened site, which is new code.

## Concerns

**1. `ExprKind::Constant` had to be promoted, and the brief did not foresee it.** An unquoted number parses as `ExprKind::Constant`, not `ExprKind::Literal` -- so `zx + 1` contains nothing promotable and would have fallen to `EvalExpr` entire. Without `Op::LoadConstant` this task would have emitted no arithmetic op for `x = x + 1`, `t.k = t.k + 1` or `a = i / 3`, which is every arithmetic line on every benchmark axis: it would have measured as a no-op and looked like a null result rather than a missing case. `Op::Const`'s existing doc comment had already recorded why `compile` cannot intern a constant's bytes (they are in the symbol table, which `compile` deliberately does not take), and the answer is the shape `Op::Load` already uses: the symbol travels in the op, the spelling is read where `code` is in hand. Its echo is the existing `Op::TraceLiteral`, because `trace_intermediate` sends both node kinds to the same `echo_literal`. Nothing about the cache key changed.

**2. The plan's prediction for this task was about a different change, and I have withdrawn it in the plan (`dd0c4dcb`) rather than only here.** The exact sentence that inverts: **"An `arith` that does not move is this task succeeding."** It is sound about prototype P1, which changed which arithmetic *operation* ran; this task changed how the *operands* are evaluated, which `i / 3` has exactly as much of as `t.k + 1` does. `arith` should move, and an `arith` that does not is now the surprise. The heading sentence **"Predicted movement: `compound` only"** is wrong for a second reason: it rules out `varlookup`, whose `x = x + 1` is the promoted shape exactly and is half its loop body. And `compound`'s cited -51.6% came from `k = i // tails` leaving the `Number` path -- `//` still takes that path after this task, so that figure is not a target here in either direction. The full replacement prediction is in the plan under "Task 9's recorded prediction".

**3. The patch table is predicted not to pay, and I would rather say so now than have Task 11 find it.** D22 forbids a hint from removing a precondition. For small-integer arithmetic the precondition *is* the specialisation -- decode both operands, check both against `DIGITS`, apply the operator -- so there is nothing left for a hint to skip except that same check. The only thing it can usefully do is skip a *hopeless* attempt, which is the direction implemented. So the table should pay slightly on `arith`, whose divisions never take the fast path, and cost a relaxed load on `compound` and `varlookup`, whose sites always do. **I expect exit criterion 5 to find the win surviving `QUICKENING = false`.** That is not a defect in the code and does not condemn D22's schema; it says small-integer arithmetic is a poor consumer for it, and Task 11 should record that either way.

**4. D22's justification for `AtomicU32` is weaker than the true one, and the true one is stronger.** The plan says `Cell` is not `Sync` and notes that reason does not bite yet. What bites today and was not stated: a chunk is reached as `&Chunk` through an `Rc`, so a site cannot record anything at all without interior mutability. `Cell<u32>` would serve for that; the atomic is a bet only on the `Sync` half. Recorded in the plan beside the prediction.

**5. Monotone demotion is a deliberate simplification with a cost.** A site whose operands leave the exact-integer range once and return keeps the general path for the rest of the chunk's life. Re-arming needs a policy -- a counter, an interval -- and there is no measurement here to choose one from. Documented on `PatchSlot`.

## Scope I did not take

`small_int_arith` still covers `+`, `-` and `*` only. Extending it to `/`, `%`, `//` and `**` is what P1 did and what `compound`'s -51.6% measured, and it would have to land in the shared function -- changing the tree-walker's semantics path too. That is a different task from "promote arithmetic", and doing it inside the IR's quickened arm alone would be a second implementation of arithmetic, which the sharing rule forbids. If the phase wants that win, it is a separate change to `eval.rs` benefiting both engines, and it should be scheduled as one.

---

# Fix round 1

`bb0f7e80` (three fixes) and `16077ea1` (the sole-catcher study and what it found). Suite **1417 passed, 0 failed**; fmt and clippy clean.

**`push_value`'s doc.** The review is right and the shape is the one this repo has paid for before: I rewrote that paragraph in `ea17a699` and carried its stale list of what falls to `EvalExpr` forward without re-checking it against the change the same commit made. The list is deleted rather than corrected -- `native_shape` is the enumeration, and a prose copy is one that stops agreeing with it.

**The lint reason.** No longer a count of anything, so it cannot be off by one again.

**The zero divisor.** Two rows, measured: untrapped `6 // zd` for the clause the report echoes, and a trapped run across `/`, `%` and `//` for where `SIGL` lands on each. It is the only one of arithmetic's three raises that comes out of `rexx-num` rather than out of converting an operand.

## The sole-catcher study

Eleven mutations attempted, **eight valid**, each re-run with `tests/ir_dual_cases/arithmetic` moved out of the directory, `--no-fail-fast`.

| mutant | caught without the file by | sole? |
|---|---|---|
| dropped `>O>` echo | population sweep + another case file | no |
| every binary operator promotes | 5, incl. this task's own golden | no |
| operands swapped | 7, incl. three goldens | no |
| operand `DIGITS` guard dropped | 5, two predating this task | no |
| result `DIGITS` ignored | 6, incl. two predating this task | no |
| `>O>` tag fixed to `"+"` | 5, incl. three `trace_oracle` tests | no |
| both `>O>` gates removed | 5 dual-engine tests | no |
| **`**` exponent via the operand conversion** | **nothing, in either configuration** | **yes, after the row below** |

**Result: one row of the file is a sole catcher, and it did not exist before this check.** The review's reading is confirmed -- the file is thin as a mutation instrument.

The one finding is worth more than the tally. `**`'s exponent goes through `to_number` and not `arith_operand`, and routing it through the latter is **invisible to a `2 ** 2.5` row**: `2.5` converts, and then `Number::pow` raises the identical 26.8 for it. Only a *nonnumeric* exponent separates the two paths, and no row had one -- so the mutation was invisible to the entire workspace, this file included. `2 ** 'x'` (26.8), `'x' ** 2` (41.1) and `'x' ** 'x'` (41.1, the base winning) close it. That asymmetry is documented in `eval_arithmetic`'s own doc comment and was untested.

**Three invalid mutants, named so nobody counts them as evidence.** Writing the result to `lhs` cannot be observed, because `lhs` is always `dst` by construction. Making every constant load `0` sends corpus programs into non-termination rather than failing. Removing either gate on the `>O>` path alone changes nothing, since the emission is gated twice -- only removing both is observable.

**I have not deleted rows, and the reason is on the file rather than only here.** Both arms of an engine-against-engine comparison are this crate, so a shared arithmetic path that is wrong the same way twice passes it; these rows record what the C++ interpreter prints for shapes no corpus program covers. That is the justification the review identified, it is different from mutation-catching, and the header now says so together with the measured negative result -- so the next reader inherits the finding instead of re-deriving it.

**Nothing to disagree with.** All three fixes were real; the one I would defend is not among them.
