# Final whole-plan review: expression promotion (ec649c0bc..8a48bbb1d)

Scope: defects that span tasks. Per-task verdicts are not re-litigated. Read-only; nothing in
the tree was edited. HEAD confirmed at `8a48bbb1d`, working tree clean for the reviewed paths.

**Verdict: the code is coherent across all four tasks; two prose defects span tasks, one in the
crate and one in the plan, plus four lower-severity staleness items.**

Everything the lead asked to be checked was checked. What follows is ranked most-severe first.

---

## 1. `Op::Arith`'s doc comment states exactly the claim Task 4 falsified

`rust/crates/rexx-exec/src/ir/mod.rs:514-520`:

> An expression compiles to this op only when *both* its operands compile to native ops too, so
> `za + zb * 4` is six ops and no `eval` recursion, while **`length('ab') + 1` is one
> `Op::EvalExpr` entire -- a call has no register to arrive in**, and `EvalExpr` names an
> expression *slot* of an instruction, which a subexpression is not.

Both halves of that sentence are false in the final tree.

* `golden_tests.rs:90` compiles `zz = length('abc') + 1` and pins the stream
  `CallExpr … path=root.L / TraceFunction / LoadConstant / TraceLiteral / Arith / TraceOperator /
  Store` -- no `EvalExpr` anywhere. `golden_tests.rs:116` uses `zz = length('ab') + length('cd')`,
  the doc comment's own example with a second call.
* "a call has no register to arrive in" is the sentence `tests/ir_dual_cases/arithmetic`'s stanza
  comment and header were corrected for in Task 4 Step 5 (`arithmetic:5`, `:105-107`), and the
  sentence `Op::CallExpr`'s own doc lost in Task 3 (the removed "So `zz = f(1)` promotes and
  `zz = f(1) + 1` does not"). The identical claim thirty lines above `Op::Binary`, in the file
  Task 1 and Task 3 both edited, was not swept.

Separately, the same paragraph's "`za + zb * 4` is six ops" does not match the committed streams:
the identical shape (three leaves, two operators) is pinned at ten region ops by
`precedence_decides_which_operator_is_the_inner_one` (`golden_tests.rs:524-538`), because every
leaf and every operator emits its echo op unconditionally. That half predates the plan.

This is the plan's own declared failure mode -- a task falsifying a premise in code it did not
touch -- landing in the single most-edited file of the diff. No per-task review could see it,
because no task's diff contains the line.

## 2. The plan's Global constraints still forbid what Task 3 did

`docs/superpowers/plans/2026-08-12-expression-promotion.md:34-36`:

> **`const _: () = assert!(size_of::<Op>() == 12)` in `ir/mod.rs` stays and is not relaxed.**
> Every op this plan adds fits inside it; that was checked with the compiler on 2026-08-12 by
> adding the candidate variants and building.
> A variant that trips it is a design error in this plan -- report it rather than widening the
> assertion.

Task 3's own section (`:363-375`) widens it to sixteen and the tree asserts
`size_of::<Op>() == 16` (`ir/mod.rs:79`). The constraints section is the part of the plan that
"binds every task in this plan, in full" (`:32`), and it is the first thing a brief is generated
from -- so the document's most authoritative passage now states a false fact about the tree and
instructs a future task to refuse the change that already landed.

The second sentence is a second problem: the 12-byte fit is cited as "checked with the compiler
… by adding the candidate variants and building", which is the identical method the same plan
later documents at `:377-384` as having produced a false result twice by reading the diagnostic
list through `head -4`. The plan corrects the method in Task 3's body and leaves the claim that
rests on it standing in the constraints.

## 3. `Op::Load`'s doc comment denies that a symbol inside an expression gets a load

`rust/crates/rexx-exec/src/ir/mod.rs:469-471`:

> **Only the whole expression, never a symbol inside one.** `x + 1` compiles to `Op::EvalExpr`
> entire; nothing here descends into an operator's operands.

`golden_tests.rs:445-456` pins `zw = zv + 1` as `Load / TraceRead / LoadConstant / TraceLiteral /
Arith / TraceOperator / Store`. This was already false at the review base (the arithmetic
promotion made it so), so it is not a defect this plan introduced -- but it is the same claim, in
the same enum, three variants above the one this plan rewrote, and it is what a reader arriving
at `Op::Binary` or `Op::Prefix` reads next. The sweep that corrected `Op::CallExpr` passed over it.

## 4. The "no cardinality" rule was applied to `eval.rs` and not to `ir/mod.rs`

Task 1 removed the counts from `eval.rs` (`is_arithmetic`'s "one of the **seven** operators" ->
"one of the operators"; the module doc's "the twelve comparison operators", "the three binary
logical operators"; `compare_op`'s "the eighteen `Operator` variants"). `ir/mod.rs` kept two:

* `:511` -- "for the **seven** operators `eval::is_arithmetic` names"
* `:524` -- "the **seven** operators' own `rexx-num` calls"

Both sit in `Op::Arith`, immediately above `Op::Binary`, whose newly written prose deliberately
names the set and not its size ("every operator `eval::is_native_binary` names and
`eval::is_arithmetic` does not"). Same rule, adjacent paragraphs, opposite treatment; the count
is currently correct, so this is unevenness rather than falsity. `drive.rs:773`'s "**The third**
native expression op" is the same shape as an ordinal over a set the plan just grew by three.

I found no instance of the converse hazard the lead warned about -- a deleted numeral widening a
claim. `roots.rs`'s rewrite replaced "six functions" with "every such frame is healed here", and
`eval.rs`'s module doc replaced "Six functions here open a temps frame" with a universally
quantified sentence; both are stronger than what they replaced and both are true of the
truncate-to-watermark mechanism.

## 5. `Loud::call_op_off_its_node`'s doc is now an incomplete account of when it fires

`rust/crates/rexx-exec/src/lib.rs:868-880`. The doc says reaching it "means the op names an
instruction whose `CALL` is one of the three forms that stay `Op::Generic`", and the message
reads "a compiled Call op does not name a CALL name of its own body". Task 3 gave it two new
ways to fire (`drive.rs:590`, `drive.rs:650`): a `NodePath` descent that lands on a node with no
such child, and `Op::TraceFunction`'s echo failing the same descent -- neither of which is a
`CALL name` instruction, and the second of which is not a call op at all. The op sharing this
loud with `Op::Call` predates the base; the descent failure mode does not, and the doc's
exhaustive "reaching it means" was not widened for it.

## 6. A stale justification beside the test Task 4 extended

`golden_tests.rs:441-442`: "Neither is arithmetic, so both stay on `Op::EvalExpr` entire", as the
reason `.NIL` and `>zv` stay general. The conclusion is right and the premise is now inert: they
stay general because `native_shape` declines those *term* kinds, and the promotable operator set
is no longer arithmetic. Pre-existing text, but Task 1 widened the operator set and Task 4 added
the call row to the sibling test (`the_value_shapes_outside_the_native_set_stay_general`), so this
is the one neighbour in that family left reading as though arithmetic were the whole set.

---

## What was checked and found clean

**Coherence of the six statements of the promoted set (lead's item 3).** `native_shape`
(`compile.rs:804`), `push_native` (`:863`), `push_value` (`:753`), `golden.rs`'s renderer,
`corpus_shape_tests`' independent restatement and the `Op` doc comments agree, including at the
boundary:

* `native_shape` refuses a call at `path == None` and accepts every other shape regardless of
  depth; `corpus_shape_tests::native` restates that as `depth <= DEEPEST_ADDRESSED_CALL` with
  `DEEPEST_ADDRESSED_CALL = 31`, and `root_of` handles the root call (depth 0) unconditionally.
  `NodePath::child` admits exactly 31 steps before the sentinel would shift out, pinned by
  `ir::tests::a_node_path_carries_thirty_one_steps_and_refuses_the_thirty_second`. The two
  numbers agree, and `corpus_shape_tests.rs:220-229` states the obligation to move them together.
* `push_native`'s `Call` arm's `path.expect(...)` is licensed by `native_shape`'s `Call` arm being
  the only reader of the address; the two are the same predicate.
* `chunk_node_at`'s slot arms (`Assignment`/0, `Say`/0) are exactly the two `push_value` call
  sites (`compile.rs:593`, `:629`), so no emitted address can name a slot the descent declines.
* `Root::of` and `render` are both exhaustive over `Op` with no catch-all; the anti-vacuity list
  in `sweep_every_corpus_body` carries `Binary`, `Prefix` and `CallExpr`.

**The five adjacency assertions (lead's item 4).** All five check position *and* register, and
each checks the tag its echo carries: read (symbol + read kind), operator (operator), prefix
(operator), call (index + slot + path). `assert_operator_echoes_follow_their_op` accepts
`Op::Arith` and `Op::Binary` alike. Every echo op an expression can emit is covered.
`Op::TraceKeyword` is the one echo op with no adjacency assertion, and that is correct rather
than a gap -- its contract is ordering *within* a loop header, not adjacency to a computing op,
and `assert_trace_ops_open_a_clause_region` covers `Op::TraceClause`.

**The `Op` width claims (lead's item 1).** `size_of::<Op>() == 16` is asserted; the doc comment's
summary of record entry 24 matches the entry, including the entry's own disclaimer that it did
not build the change that landed. `Op::CallExpr`'s "widening either to `u32` makes this variant
20 bytes" is arithmetically right (14 bytes of payload at align 4 leaves exactly the two bytes of
tail padding the discriminant needs; either widening takes payload to 16 and forces 20).
`PlanSlot`'s rewritten doc correctly withdraws the claim that the op array forces its
representation.

**The `Engine::DEFAULT` rule (lead's item 2).** Every prose claim about which engine a test
exercises that I could find is now either correct or explicit. `eval::tests::depth_limited`,
`lib.rs`'s stack-span test and `spike.rs`'s `records_the_stack_cost_of_one_eval_frame` all name
`Engine::TreeWalker` and say why. `tests/ir_dual_cases/operators:66-71` goes the other way and
states it correctly: `check_witness` runs `Invocation::none()`, whose engine is `Engine::DEFAULT`,
which is `Engine::Ir`. `Interp::new`'s `engine: Engine::TreeWalker` is documented as not being
the default and as deciding only what in-crate unit tests run on.

**The plan's "deliberately out of scope" section (lead's item 5).** All four claims are true of
the final tree: `ExprKind::Logical` has no `native_shape` arm and falls to `_ => false`;
arithmetic keeps `Op::Arith` (`push_native:942`); a call's arguments are not compiled to
registers (`push_native`'s `Call` arm emits `CallExpr`/`TraceFunction` and nothing else, and the
driver reaches the args through `eval_call_resolved` on the node); `DotVariable` and
`VariableReference` fall to `_ => false` and are pinned by
`the_value_shapes_outside_the_native_set_stay_general`.

**Behaviour.** No cross-task behavioural defect found. `Op::TraceFunction`'s early
`tracing_intermediates` guard is the identical condition `trace_intermediate` opens with, so it
changes what is computed and not what is printed. The `continue` in that arm continues the region
`for` loop, which is the correct next-op step. The rooting story survives Task 4: a nested call's
enclosing operand sits in a register of the region `run_chunk` reserved on the temporaries stack,
so it is a root across the nested activation.
