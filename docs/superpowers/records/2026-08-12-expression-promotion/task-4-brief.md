### Task 4: a call nested inside an expression

**Files:**

* Modify: `rust/crates/rexx-exec/src/ir/compile.rs`
* Modify: `rust/crates/rexx-exec/src/ir/golden_tests.rs`
* Modify: `rust/crates/rexx-exec/src/ir/corpus_shape_tests.rs`
* Modify: `rust/crates/rexx-exec/tests/ir_dual_cases/operators`
* Modify: `rust/crates/rexx-exec/tests/ir_dual_cases/arithmetic`

**Interfaces:**

* Consumes: Task 3's `NodePath` and its `Op::CallExpr`/`Op::TraceFunction` `path` field, and Tasks 1 and 2's widened `native_shape`.

**The change.**
`native_shape` accepts `ExprKind::Call` -- at the root and at any depth the path can carry.
It therefore has to know the depth, so it takes the address of the node it is asked about:

```rust
fn native_shape(expr: &Expr, path: Option<NodePath>) -> bool {
    match &expr.kind {
        ExprKind::Literal(_) | ExprKind::Constant(_)
        | ExprKind::Variable(_) | ExprKind::Stem(_) | ExprKind::Compound(_) => true,
        // A call needs an address, and a node past the width has none. The
        // whole slot then falls to `Op::EvalExpr`, which is the answer it
        // already had.
        ExprKind::Call { .. } => path.is_some(),
        ExprKind::Prefix { operand, .. } => native_shape(operand, descend(path, false)),
        ExprKind::Binary { op, left, right } => {
            is_native_binary(*op)
                && native_shape(left, descend(path, false))
                && native_shape(right, descend(path, true))
        }
        _ => false,
    }
}

fn descend(path: Option<NodePath>, right: bool) -> Option<NodePath> {
    path?.child(right)
}
```

**Only the `Call` arm reads the address, and that is the whole of the design.**
Every other shape here is computed into a register the node above names and is addressed by nothing, so a depth past the width costs it nothing.

**An earlier version of this section charged the depth at every node** -- `path: NodePath`, each operator arm answering `path.child(..).is_some_and(..)` and the `Call` arm answering `true` unconditionally -- and that is a defect rather than a simplification.
It de-promotes any *call-free* expression nesting more operators than the width carries, which today compiles to native ops entire.
`rust/corpus/lang/deep_nested_expr.rex` is exactly such a program: a single assignment of three thousand `1 +` terms and no call anywhere in it.
Measured 2026-08-12, its main body's chunk is **12004 ops, 2999 of them `Arith` and none of them `EvalExpr`, before and after this task**; under the charge-everywhere version the whole assignment would have become one `Op::EvalExpr`, which is a plan for an optimisation making a corpus program slower.

`push_native` gains a `Call` arm, and takes the path alongside the expression.
The arm is what `push_value` does today for a root call, with the address coming from the path rather than being `ROOT`:

```rust
ExprKind::Call { .. } => {
    ops.push(Op::CallExpr { index, slot, path, site: calls.reserve()?, dst });
    ops.push(Op::TraceFunction { index, slot, path, src: dst });
}
```

`push_value`'s own special case for a root call is then **deleted**: `native_shape` accepts it and `push_native` emits it, which is the same stream by a shorter route.
`push_native` needs `index`, `slot` and `calls` for the address and the resolution site, so thread them through; both it and `push_value` then carry a `clippy::too_many_arguments` expectation, which this file already precedents.

**One thing threading them costs, measured rather than predicted.**
`compile`'s expression walk takes a stack frame per operator, and the wider parameter lists make each frame bigger.
`ir::corpus_shape_tests` compiles `deep_nested_expr.rex` directly on a libtest thread, and measured 2026-08-12 that sweep was passing with under 128 KiB of a 2 MiB stack to spare *before* this task -- it aborts at `RUST_MIN_STACK=1966080` and passes at `2097152` -- so the wider frames tipped it into a stack overflow.
No other harness noticed, because every other one reaches `compile` through `Interp`, which runs on a thread with `INTERPRETER_STACK_BYTES`.
The fix is for that sweep to compile on the same stack the interpreter compiles on rather than on a libtest thread's; it is not a reason to keep the parameter lists narrow.

**The register discipline is unchanged and must stay so.**
A nested call writes to the `dst` its parent gave it, exactly like a `Const` or a `Load`, and takes no register of its own.
`za = f(1) + g(2)` therefore compiles to: `CallExpr -> dst`, `TraceFunction`, alloc `rhs`, `CallExpr -> rhs`, `TraceFunction`, `Arith`, `TraceOperator`, and two registers total.

**The depth divergence, stated rather than discovered.**
`Op::CallExpr`'s arm enters `enter_eval_node` once, so a promoted nested call's arguments start at the depth the call itself sits at rather than at the depth the tree-walker would have reached by recursing through each enclosing node.
This is not new and it is measured: on 2026-08-12, at head before this plan, `zv = 1` followed by 100,001 `+0` terms answered **rc 245 on the tree-walker and rc 0 on the compiled engine**, because a promoted operator chain is flat ops with no `eval` recursion for `MAX_EVAL_DEPTH` to count.
Every task in this plan widens the set of expressions that reach it -- Task 1 to concatenation, comparison and logical, Task 2 to the prefix operators, Task 4 to a nested call.
`MAX_EVAL_DEPTH` is a guard on *this crate's* Rust stack rather than an oracle behaviour (`eval.rs`'s own doc comment: the oracle's cliff is far lower and it segfaults above it), and the compiled engine has no run-time recursion to guard -- checked to 700,000 terms, rc 0.
So the divergence is the guard not firing where it has nothing to protect, not a wrong answer.

**It is accepted, and the decision is Moritz's, taken 2026-08-12.**
Not returning a stack overflow is a property of a different evaluation strategy; there is no formal semantics here for either engine to be measured against, so the two are allowed to diverge on it, and managing the divergence is worth some effort because the alternative is forcing a shape on the compiled engine to reproduce a limit that exists only to protect the other one's Rust stack.

**The licence is that narrow and does not generalise.**
What is accepted is the depth guard not firing where the compiled engine does not recurse.
Everything else the two engines do must still agree byte for byte, which is what `tests/ir_dual.rs` and the corpus sweep are for, and a divergence found anywhere else is a defect rather than an instance of this.

Record it in the record entry; do not add a mechanism for it, and do not let a task quietly re-pin a test to one engine without saying which of the two the test is now about.

**Steps:**

- [ ] **Step 1: write the failing golden tests**

Replace `a_call_at_the_root_of_a_value_takes_its_own_op_and_a_nested_one_does_not`, whose second half this task falsifies, with the pair that pins the new rule:

```rust
/// A call promotes wherever it sits in a value, and the address it carries is
/// the route to it: the same call at the root and one operator down get the
/// same op with different addresses.
#[test]
fn a_call_promotes_at_the_root_and_below_it() { /* full rendered streams */ }

/// A call the address cannot reach leaves the whole slot general, which is the
/// answer it had before there was an address at all.
#[test]
fn a_call_nested_past_the_paths_width_leaves_the_slot_general() {
    /* thirty-two nested operators with a call at the bottom -> Op::EvalExpr */
}
```

The second test is the one that can only pass for the right reason: build the source by generating `zb + (zb + (... f(1) ...))` to a depth just past the width, and assert the *adjacent* success at one step shallower in the same test.
Pair a refusal with its neighbouring success -- that is what pins the bound to the width rather than to something coincidental.

- [ ] **Step 2: run them and watch them fail**, reading the run count.

- [ ] **Step 3: widen `native_shape` and `push_native`, delete `push_value`'s special case.**

- [ ] **Step 4: update `corpus_shape_tests.rs`.**

`root_of`'s `ExprKind::Call` arm keeps answering `Root::CallExpr`; `native` gains a `Call` arm and the depth bound, restated independently of `compile`.
The corpus contains `rust/corpus/lang/deep_nested_expr.rex`, which nests three thousand terms on purpose -- confirm what that program's assignment now compiles to and that the expectation agrees.

- [ ] **Step 5: add nested-call stanzas to `tests/ir_dual_cases/operators`**

Under Task 1's rules for that file.
Cover a call as an operator's operand, a call as a prefix operator's operand, two calls in one expression, a call inside a call's argument, a call whose own argument is an operator over a call, and a call that raises from a nested position -- with a `trace i` stanza, so the `>F>` line's position relative to the surrounding `>O>` and `>V>` lines is compared.

`tests/ir_dual_cases/arithmetic` has a stanza commented "an operand that is a call, which no compiled operand register can hold: the whole expression is evaluated the way it was before there were arithmetic ops".
This task falsifies that sentence.
Correct it and keep the stanza, whose oracle bytes are unaffected.

- [ ] **Step 6: gates.**

- [ ] **Step 7: commit.**

---

## After the plan

The controller, not a task, does these.

* An interleaved paired comparison per increment, under the accept rule (plan §164): wall clock, `ulimit -v 8388608`, `REXX_ENGINE=ir`, a fresh empty working directory per run, arm order rotated every round, seven rounds per axis, behind the host-idle gate.
  Increment 1 is Tasks 1 and 2; increment 2 is Tasks 3 and 4.
  The axes that can move are `strings` and `alloc4c`; `varlookup`, `compound`, `arith` and `emptyloop` are the controls, and a regression on any of them is a finding.
* `perf stat -e instructions:u,cycles:u` alongside, because the wall-clock floor on these axes is about seven points and an instruction count resolves below it.
* A record entry per increment in `docs/superpowers/plans/phase-4f-record.md`, including the honest decomposition: how much of any win is the promotion and how much is anything else that rode along.
