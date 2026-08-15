# Final-review fixes: the five prose findings

Scope: findings 1, 3, 4, 5 and 6 of `final-review.md`. Finding 2 is the plan file and was fixed by
the lead; nothing under `docs/` was touched. No behaviour changed -- every hunk is a comment or a
doc comment.

Gates, all run from `rust/`, all unpiped for their status:

* `cargo fmt --all --check` -- exit 0
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0
* `memcap 8G cargo test --workspace --no-fail-fast` -- exit 0, **1464 passed, 0 failed, 4 ignored**

---

## Finding 1 -- `Op::Arith`'s doc comment (`rust/crates/rexx-exec/src/ir/mod.rs`)

**Was:**

> An expression compiles to this op only when *both* its operands compile to native ops too, so
> `za + zb * 4` is six ops and no `eval` recursion, while `length('ab') + 1` is one
> [`Op::EvalExpr`] entire -- a call has no register to arrive in, and `EvalExpr` names an
> expression *slot* of an instruction, which a subexpression is not.

**Now:**

> An expression compiles to this op only when *both* its operands compile to native ops too, so
> `za + zb * 4` runs with no `eval` recursion in it, while a slot holding one node with no op of
> its own -- a `.NIL`, or a call the address does not reach -- is one [`Op::EvalExpr`] entire,
> because `EvalExpr` names an expression *slot* of an instruction, which a subexpression is not.

**Checked against:**

* The call half is gone because `golden_tests.rs`'s `a_call_promotes_at_the_root_and_below_it`
  pins `zz = length('abc') + 1` as `CallExpr path=root.L / TraceFunction / LoadConstant /
  TraceLiteral / Arith / TraceOperator / Store` -- no `EvalExpr` in the stream, and the call
  arrives in register 0.
* The two replacement examples are each pinned rather than reasoned about:
  `the_value_shapes_outside_the_native_set_stay_general` pins `zw = .nil` and `zw = .nil || za` as
  one `EvalExpr` entire, and `a_call_nested_past_the_paths_width_leaves_the_slot_general` pins the
  same for a call one step past the address width.
* "one node with no op of its own ... entire" is `native_shape`'s own contract
  (`compile.rs`): it is asked once at the whole expression and an expression with one
  unpromotable node anywhere stays one `EvalExpr`. It is also how `Op::CallExpr`'s doc already
  states the rule ("a single sibling term with no op of its own leaves the whole slot on
  `Op::EvalExpr`"), so the two paragraphs now say the same thing.
* The count went rather than being corrected, per the no-cardinality rule. For the record of why
  correcting it would have been wrong twice over: the shape's committed stream is
  `precedence_decides_which_operator_is_the_inner_one`, which renders `zw = za + zb * zc` as
  three `Load`s, three `TraceRead`s, two `Arith`s, two `TraceOperator`s and a `Store` -- every
  leaf and every operator emits its echo op, so what "six ops" would have to be corrected to
  depends on where the counter starts.
* "runs with no `eval` recursion in it" is that same test's stream: `Load`/`LoadConstant` and
  `Arith` only, with `eval.rs` entered for nothing in it. `4` is an `ExprKind::Constant`, which
  `Op::LoadConstant`'s own doc states and the golden streams show as `LoadConstant`.

## Finding 3 -- `Op::Load`'s doc comment (`ir/mod.rs`)

**Was:**

> **Only the whole expression, never a symbol inside one.** `x + 1` compiles to [`Op::EvalExpr`]
> entire; nothing here descends into an operator's operands.

**Now:**

> **Emitted wherever the descent reaches a bare symbol -- as a slot's whole expression, and inside
> one.** `zv + 1` compiles to this op and the operator applied to it, because an operator's
> operands are compiled to ops of their own. A symbol the descent never walks to gets none -- a
> call's arguments go back through `eval.rs` -- and neither does one inside a slot that falls to
> [`Op::EvalExpr`] entire, which is the whole slot's choice rather than this node's.

**Checked against:**

* `golden_tests.rs`'s `an_expression_that_only_contains_a_symbol_is_more_than_that_symbols_read`
  pins `zw = zv + 1` as `Load / TraceRead / LoadConstant / TraceLiteral / Arith / TraceOperator /
  Store` -- the symbol inside the expression takes a `Load`.
* `a_say_of_a_bare_symbol_compiles_to_a_native_read` pins the other half, a symbol that *is* the
  slot's expression.
* "wherever the descent reaches" is `push_native`'s `Variable`/`Stem`/`Compound` arms
  (`compile.rs`) reached through its own `Binary` and `Prefix` recursion, with `push_value`
  entering at the root.
* "a call's arguments go back through `eval.rs`" -- `push_native`'s `Call` arm emits `CallExpr`
  and `TraceFunction` and nothing else, and `drive.rs`'s `Op::CallExpr` arm reaches the arguments
  through `eval_call_resolved` on the node. So no argument symbol takes a `Load`.
* "neither does one inside a slot that falls to `Op::EvalExpr` entire" is the `zw = .nil || za`
  row of `the_value_shapes_outside_the_native_set_stay_general`, where `za` gets no `Load`.

## Finding 4 -- the no-cardinality rule, three sites

**`ir/mod.rs`, `Op::Arith`'s summary line.** "for the seven operators `eval::is_arithmetic` names"
-> "for the operators `eval::is_arithmetic` names". The numeral was not doing quantifier duty: the
relative clause bounds the set exactly, so deleting it widens nothing. Same shape as Task 1's
`is_arithmetic` fix in `eval.rs` ("one of the seven operators" -> "one of the operators").

**`ir/mod.rs`, `Op::Arith`'s second paragraph.** "the seven operators' own `rexx-num` calls" ->
"the arithmetic operators' own `rexx-num` calls". Here bare deletion *would* have widened it, so
the set is named instead of counted. Not "each operator's own `rexx-num` call", which was the
first replacement and was withdrawn: `eval.rs`'s `arith_general` sends `Divide`, `IntDiv` and
`Remainder` all to `Number::div` under different `DivOp`s, so "each ... own call" would imply a
distinctness the code does not have.

**`ir/drive.rs`, the `Op::Arith` arm.** "**The third native expression op**" -> "**A native
expression op**". The ordinal was over a set this plan grew and carried nothing else; the rest of
the comment (what it computes, which `Interp` functions it enters, that it emits nothing) is
unchanged.

**Checked against:** `eval::is_arithmetic`'s `matches!` in `eval.rs` is the one enumeration and is
untouched, so nothing that was load-bearing moved into prose; `arith_general`'s operator `match`
is what the `rexx-num` sentence describes.

## Finding 5 -- `Loud::call_op_off_its_node`'s doc (`rust/crates/rexx-exec/src/lib.rs`)

**Was:**

> [`Loud::select_op_off_its_node`]'s reasoning exactly, one instruction over, with the extra step
> that `ir::compile` emits this op only for the `Named` form: reaching it means the op names an
> instruction whose `CALL` is one of the three forms that stay `Op::Generic`.

**Now:** the doc leads with "A compiled op that runs or traces a call does not describe the call it
was emitted for", and lists the ops that report it: `Op::Call` (the op names an instruction that is
not a `CALL` at all, or one whose `CALL` is a form that stays `Op::Generic`), `Op::CallExpr` (its
`slot` and `path` reach no node of the clause, or reach one that is not a call), and
`Op::TraceFunction` (its echo walks the same address and finds nothing at the end of it).

**Checked against:** the four `Loud::call_op_off_its_node()` sites in `drive.rs` -- two in the
`Op::Call` arm (the instruction not being an `InstructionKind::Call`, and its `Call` not being
`Call::Named`), one in the `Op::CallExpr` arm (`chunk_node_at(...)` answering `None`, or a node
whose kind is not `ExprKind::Call`), and one in the `Op::TraceFunction` arm (`chunk_node_at(...)`
answering `None` -- that arm accepts a node of any kind, so only the descent can fail it). The
`Named`-only emission is `compile.rs`'s `InstructionKind::Call(call) if matches!(&**call,
Call::Named { .. })` guard. The old text's "three forms" count went with the rewrite.

**What I did not change: the message string.** It still reads "a compiled Call op does not name a
CALL name of its own body", which is narrower than the ops that now report it -- a
`TraceFunction` echo whose descent fails prints a sentence about a `Call` op. Widening it changes
output, which this round was told not to do, so it is reported rather than fixed. If it is wanted,
"a compiled call op does not name a call of its own body" covers all four sites, and no test or
corpus file pins the current text (checked by searching the tree for the message).

## Finding 6 -- the stale justification in `golden_tests.rs`

**Was:** "Neither is arithmetic, so both stay on [`super::Op::EvalExpr`] entire."

**Now:** "Neither is a term `native_shape` accepts, so both stay on [`super::Op::EvalExpr`]
entire."

**Checked against:** `native_shape`'s `match` in `compile.rs` -- `ExprKind::DotVariable` and
`ExprKind::VariableReference` have no arm and fall to `_ => false`, so the refusal is about the
term kind and not about the operator set. The conclusion is unchanged and is pinned by this test's
own `.nil` and `>zv` cases, and by `the_value_shapes_outside_the_native_set_stay_general`.

---

## Same class, found and deliberately not fixed

These are what I noticed in the regions I read while making the five fixes. It is not a sweep, and
I did not go looking -- an unbounded one was ruled out for this round.

* **`ir/drive.rs`, the `Op::Load` arm: "**The second native expression op**"**, and "**The phase's
  first native expression op**" at the `Op::Const` arm, at `Op::Const`'s doc comment in
  `ir/mod.rs`, and at `an_assignment_of_a_literal_compiles_to_a_constant_load_and_a_store` in
  `golden_tests.rs`. Same ordinal-over-a-growing-set shape as the one Finding 4 named. Fixing only
  the third leaves the sequence reading first, second, (none), which is worse than either
  consistent state; I judged that better than widening the round past its list, but it is the one
  item here I would fix next.
* **`golden_tests.rs`, the doc of
  `an_expression_that_only_contains_a_symbol_is_more_than_that_symbols_read`**: "`.NIL` and `>zv`
  are **the two** expressions that look like a bare symbol read and are not one". A count, and an
  exhaustiveness claim over an in-repo set, in the sentence immediately before Finding 6's fix. I
  changed the clause the review named and left this one.
* **`eval.rs`, `is_strict_compare`'s doc**: "Whether `op` is one of **the eight** strict comparison
  operators". Same rule, in the file Task 1 swept for exactly this.
* **`ir/compile.rs`, above the `Op::Call` emission guard**: "The other **three** `Call` forms fall
  to `Generic` below" -- a count of a set the phases are still growing, and the same fact the
  `Loud` doc above carried until this round.
* **`eval.rs`, `arith_general`'s `unreachable!` message**: "eval_node only dispatches the seven
  arithmetic operators here". A count, but inside a code string rather than a comment; the rule
  exempts enumerations in code, and this is neither cleanly one nor the other.
