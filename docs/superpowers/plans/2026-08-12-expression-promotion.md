# Finishing expression promotion in the IR implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** promote the expression shapes `ir::compile` still leaves to `Op::EvalExpr` -- the non-arithmetic binary operators, the prefix operators, and a call nested inside a larger expression -- so that a clause built from them runs from registers with `eval.rs` not entered.

**Architecture:** three additions to the op stream and one address widening.
`Op::Binary` computes concatenation, comparison and logical operators from two registers, through one `Interp::apply_binary` that `eval_node` also enters.
`Op::Prefix` does the same for `+`/`-`/`\`, through `Interp::apply_prefix`.
A call's op stops naming an expression *slot* alone and starts naming a slot plus a bit-encoded route down to the node -- so a call anywhere inside a promotable expression gets an op, where today only a call that **is** the whole slot does.

**Tech stack:** Rust 2024, the `rexx-exec` crate, `cargo test`/`clippy`/`fmt` from `rust/`.

## Context

Phase 4f is the optimisation loop.
`docs/superpowers/plans/phase-4f-record.md` is its running record; entry 23 landed `Op::CallExpr` for a call at the root of a value and recorded that three of `strings.rex`'s four calls are promoted, not four.
This plan is what closes that.

The two benchmark axes this is aimed at:

* `bench-programs/strings.rex` -- `joined = piece || changed` is a concatenation at the root of a value, and `total = total + length(joined)` holds a call one operator down.
* `bench-programs/alloc4c.rex` -- `s = "item" || i` is a concatenation, and `total = total + 3 + length(s)` holds a call two operators down.

Neither line has any op today: each falls to `Op::EvalExpr` entire.

## Global constraints

* **The oracle tree at `/home/moritz/dev/repos/ooRexx/` is read-only.**
  Never modify `interpreter/`, `samples/`, `build/`, `ootest/`.
* **`rust/CLAUDE.md` binds every task in this plan**, in full: the oracle wrapper, the probe rules, the gate rules, the repository hygiene rules and the comment rules.
  Read it before starting.
* **`const _: () = assert!(size_of::<Op>() == 12)` in `ir/mod.rs` stays and is not relaxed.**
  Every op this plan adds fits inside it; that was checked with the compiler on 2026-08-12 by adding the candidate variants and building.
  A variant that trips it is a design error in this plan -- report it rather than widening the assertion.
* **No second implementation of anything `eval.rs` already owns.**
  A native op enters the same function `eval_node` enters, with the operands already in hand.
  This is the rule `Op::Arith`, `Op::Load` and `Op::CallExpr` were each built under, and their doc comments state it.
* **A computing op emits nothing; its trace line is a separate op immediately behind it.**
  `Op::TraceLiteral`, `Op::TraceRead` and `Op::TraceOperator` are the precedent, and `compile` asserts the adjacency.
* **`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `memcap 8G cargo test --workspace --no-fail-fast` must all be clean at the end of every task**, run from `rust/`, each exit status read unpiped.
* **Commit at the end of each task**, with `git commit -F -`, staging the exact paths changed.
* Comments state the contract at the top and the reasoning at the decision point.
  No task numbers, no "used to", no counts of mutable in-repo aggregates.
  `--` rather than an em-dash.

## What is deliberately out of scope

Stated here so that a reviewer does not read an omission as a gap.

* **`ExprKind::Logical(items)`, the comma-separated conditional list.**
  It short-circuits (`eval_logical_list`), so it is not an operator over two registers, and it keeps `Op::EvalExpr`.
* **Arithmetic keeps `Op::Arith`.**
  Its site carries a quickening hint no other operator has, and `**`'s exponent is not evaluated the way its base is (`eval_arithmetic`'s own doc comment) -- so it does not share the operand prologue the other three families do.
* **A call's arguments are not compiled to registers.**
  `Interp::eval_call_resolved` evaluates them through `eval`, with the `>A>` lines, the omitted-argument rule and `USE ARG >name`'s reference capture inside it; a register-based argument path would be a second copy of all of that.
* **`ExprKind::DotVariable` and `ExprKind::VariableReference`** keep `Op::EvalExpr`.

## Files

* Modify `rust/crates/rexx-exec/src/eval.rs` -- the operand/operator split, and the shared `apply_binary`/`apply_prefix`.
* Modify `rust/crates/rexx-exec/src/ir/mod.rs` -- the new `Op` variants and `NodePath`.
* Modify `rust/crates/rexx-exec/src/ir/compile.rs` -- `native_shape`, `push_native`, `push_value`.
* Modify `rust/crates/rexx-exec/src/ir/drive.rs` -- the new arms, and the two arms whose address changes.
* Modify `rust/crates/rexx-exec/src/ir/golden.rs` -- the test-only renderer, which is exhaustive over `Op`.
* Modify `rust/crates/rexx-exec/src/ir/golden_tests.rs` -- the committed streams.
* Modify `rust/crates/rexx-exec/src/ir/corpus_shape_tests.rs` -- `Root`, `root_of`, `native`.
* Modify `rust/crates/rexx-exec/src/run.rs` -- the address resolution `chunk_call_at`/`chunk_expr_at` do today.
* Add `rust/crates/rexx-exec/tests/ir_dual_cases/operators` -- a **datadriven case file**, not a directory and not a `.rex` file, holding dual-engine programs for the newly promoted shapes.
* Modify `rust/crates/rexx-exec/tests/ir_dual_cases/arithmetic` -- two of its comments state that the other operator families are unpromoted and that a call cannot sit in an operand register, and this plan falsifies both.

---

### Task 1: the non-arithmetic binary operators as one native op

**Files:**

* Modify: `rust/crates/rexx-exec/src/eval.rs`
* Modify: `rust/crates/rexx-exec/src/ir/mod.rs`
* Modify: `rust/crates/rexx-exec/src/ir/compile.rs`
* Modify: `rust/crates/rexx-exec/src/ir/drive.rs`
* Modify: `rust/crates/rexx-exec/src/ir/golden.rs`
* Modify: `rust/crates/rexx-exec/src/ir/golden_tests.rs`
* Modify: `rust/crates/rexx-exec/src/ir/corpus_shape_tests.rs`
* Add: `rust/crates/rexx-exec/tests/ir_dual_cases/operators`
* Modify: `rust/crates/rexx-exec/tests/ir_dual_cases/arithmetic`

**Interfaces:**

* Produces: `Interp::apply_binary(&mut self, op: Operator, left: ObjRef, right: ObjRef) -> Result<ObjRef, Failure>`, `eval::is_native_binary(op: Operator) -> bool`, `Op::Binary { op: Operator, lhs: u16, rhs: u16, dst: u16 }`.
* Consumes: nothing from a later task.

**The split.**
`Interp::concat`, `Interp::eval_compare` and `Interp::eval_logical` in `eval.rs` share one operand prologue exactly:

```rust
let frame = self.roots.push_frame();
let left_value = self.eval(code, left)?;
self.roots.push_temp(left_value);
let right_value = self.eval(code, right)?;
self.roots.push_temp(right_value);
/* ... the operator's own work ... */
self.roots.pop_frame(frame);
```

Split each at that seam.
The operator's own work becomes a method taking two already-rooted values:

* `fn concat_values(&mut self, left: ObjRef, right: ObjRef, separator: &[u8]) -> Result<ObjRef, Failure>`
* `fn compare_values(&mut self, op: Operator, left: ObjRef, right: ObjRef) -> Result<ObjRef, Failure>`
* `fn logical_values(&mut self, op: Operator, left: ObjRef, right: ObjRef) -> Result<ObjRef, Failure>`

Each keeps its existing body verbatim from the first line after the prologue to the line before `pop_frame`, and its existing doc comment, minus the sentences that were about evaluating operands.
**The rooting contract moves to the caller and must be stated at each one**: these allocate, so their arguments have to be rooted by the caller -- which is what `push_temp` did inside and what a register does in the driver.
This is `Interp::arith_general`'s existing contract and `Op::Arith`'s driver arm cites it; say the same thing here.

`Interp::apply_binary` is the one dispatch:

```rust
pub(crate) fn apply_binary(
    &mut self,
    op: Operator,
    left: ObjRef,
    right: ObjRef,
) -> Result<ObjRef, Failure> {
    match op {
        Operator::Concatenate | Operator::Abuttal => self.concat_values(left, right, b""),
        Operator::Blank => self.concat_values(left, right, b" "),
        op if is_comparison(op) => self.compare_values(op, left, right),
        Operator::And | Operator::Or | Operator::Xor => self.logical_values(op, left, right),
        op => Err(Loud::binary_operator(op).into()),
    }
}
```

`is_comparison` is a new free function in `eval.rs` beside `is_arithmetic`, enumerating the operators `eval_node`'s comparison arm lists today, moved out of that arm rather than copied.

`eval_node`'s three arms (concatenation, comparison, logical) collapse into **one** arm that runs the prologue and calls `apply_binary`:

```rust
ExprKind::Binary { op, left, right } if is_native_binary(*op) && !is_arithmetic(*op) => {
    let frame = self.roots.push_frame();
    let left_value = self.eval(code, left)?;
    self.roots.push_temp(left_value);
    let right_value = self.eval(code, right)?;
    self.roots.push_temp(right_value);
    let result = self.apply_binary(*op, left_value, right_value);
    self.roots.pop_frame(frame);
    result?
}
```

Place it **after** the arithmetic arm, so arithmetic still reaches `eval_arithmetic`.
The measured behaviour each of the three collapsed arms documented -- `Blank` inserting exactly one space, the comparison operators it names, "both operands always evaluated, never short-circuited" -- must survive as doc comments on the functions that now own it.
Deleting a true comment to make the change easier is forbidden.

`Loud::binary_operator` is a new loud message for the one `Operator` no family claims, `Operator::Backslash` (the prefix `\` token, which the parser never builds an `ExprKind::Binary` from).
Follow the shape of the loud messages already in `loud.rs`.
**Do not write `unreachable!` here**: this project has shipped six wrong "cannot be reached" claims.

**The promotion.**
`eval::is_native_binary(op) -> bool` is `is_arithmetic(op) || is_concatenation(op) || is_comparison(op) || is_logical(op)`, spelled as a positive enumeration of the families so that an operator added to `rexx_parse::Operator` does not silently become promotable.

In `ir/compile.rs`:

* `native_shape`'s `Binary` arm changes from `is_arithmetic(*op) && ...` to `is_native_binary(*op) && ...`.
* `push_native`'s `Binary` arm keeps its register discipline exactly (left into `dst`, right into a scratch above it, both read before `dst` is written) and chooses the op: `Op::Arith` when `is_arithmetic(*op)`, otherwise `Op::Binary { op, lhs: dst, rhs, dst }`.
  Only `Op::Arith` reserves a hint slot; `Op::Binary` must not call `hints.reserve()`, because a slot reserved and never read shifts every later site's index.
* The `Op::TraceOperator { op, src: dst }` behind it is unchanged -- every binary operator traces `>O>`, which `trace_intermediate`'s own `ExprKind::Binary` arm already states.

In `ir/drive.rs`, `Op::Binary`'s arm mirrors `Op::Arith`'s: the same two `debug_assert!`s on the registers, both operands read before the destination is written, `self.apply_binary(*op, left, right)`, `break 'region Err(failure)` on failure, `self.roots.set_temp(registers, *dst as usize, value)`.
Add the matching `Loud::op_not_driven("Binary")` arm in the same place `Op::Arith`'s sits.

**Steps:**

- [ ] **Step 1: write the failing golden test**

In `ir/golden_tests.rs`, beside `only_the_arithmetic_operators_promote`, add:

```rust
/// A concatenation, a comparison and a logical operator each compile to
/// `Op::Binary`, and arithmetic still compiles to `Op::Arith`.
///
/// **The pair is the test.** Either half alone is satisfied by a compiler
/// that emits one op for every operator; together they pin the split, which
/// is that only arithmetic carries a quickening hint.
#[test]
fn every_binary_operator_but_arithmetic_compiles_to_one_op() {
    let chunk = compile_for_test(b"za = zb || zc\n").expect("compiles");
    assert!(render(&chunk).contains("Binary"), "{}", render(&chunk));
    let chunk = compile_for_test(b"za = zb + zc\n").expect("compiles");
    assert!(render(&chunk).contains("Arith"), "{}", render(&chunk));
}
```

Then replace that `contains` pair with the **exact** rendered streams once `golden.rs` renders `Op::Binary`, in the style every other test in that file uses (an `assert_eq!` against a full stream).
`only_the_arithmetic_operators_promote` states the opposite of what this plan builds: rewrite it to name what still does **not** promote -- an `ExprKind::Logical` comma list, a `DotVariable` -- rather than deleting it.

- [ ] **Step 2: run it and watch it fail**

Run: `cd rust && cargo test -p rexx-exec every_binary_operator_but_arithmetic`
Expected: FAIL, and the run count is non-zero.
A `cargo test` filter that matches nothing exits 0 -- read the count, not the status.

- [ ] **Step 3: split `eval.rs` and add `apply_binary`**

As above.
No `ir/` change yet; the workspace must still be green after this step alone, which is what proves the split is behaviour-preserving on the tree-walker.

- [ ] **Step 4: run the workspace**

Run: `cd rust && memcap 8G cargo test --workspace --no-fail-fast`
Expected: green, except the new golden test.

- [ ] **Step 5: add `Op::Binary` and drive it**

`ir/mod.rs` (variant plus its doc comment, in the style of `Op::Arith`'s), `ir/compile.rs`, `ir/drive.rs`, `ir/golden.rs`.
`golden.rs` and `corpus_shape_tests.rs::Root::of` are both exhaustive over `Op` and will not compile until the new variant is classified in each.
`corpus_shape_tests`' `Root` gains a `Binary` variant, and its `root_of`/`native` restate the widened set **independently** -- spelling the operators out rather than calling `eval::is_native_binary`, which is the whole reason that file can fail.

- [ ] **Step 6: add the dual-engine cases**

**Read `tests/ir_dual.rs` and the header of `tests/ir_dual_cases/arithmetic` before writing a byte of this.**
The cases are `datadriven` stanzas in an extensionless file under `tests/ir_dual_cases`, one `program` directive per stanza, expected output tagged `rc>`/`out>`/`err>`.
`REWRITE=1` is refused by the harness on purpose.

**Every expected byte is the C++ oracle's**, captured with

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx /abs/path/PROGRAM.rex </dev/null )
```

run from a **fresh empty directory you `mkdir` yourself**, with absolute paths: the scratchpad is on the oracle's external-routine search path and holds stale `.rex` files that a probe will call.
Never run `select; when 1 = 0 then; when 2 = 2 then nop; end`, `say date('M','0','D')`, or `NUMERIC DIGITS` above 1000 -- each crashes the oracle.

Write `tests/ir_dual_cases/operators` with a header saying what the file is for, and stanzas covering:

* every concatenation spelling -- `||`, abuttal and blank -- including that `Blank` inserts exactly one space however much whitespace separated the terms;
* a comparison from the strict family and one from the non-strict family, plus one under a narrowed `NUMERIC DIGITS` and one under a non-zero `NUMERIC FUZZ`, which is where a comparison reads settings a concatenation does not;
* `&`, `|` and `&&`, including the measured case that **both** operands are checked even when the first decides the answer;
* an operand that is itself an operator, so the nesting is covered;
* a failing case per family, including `say ('y' & 'x')` -- which reports the **left** operand's text -- so the failure path and its substitution are compared;
* a `trace i` stanza and a `trace r` stanza, because the `>O>` line is where a native op diverges first and only an exact stderr comparison sees it.

**Each stanza must be measured to produce identical bytes on both engines before the promotion lands as well as after**, which is what says the row could ever have failed -- the `arithmetic` file's own header states this requirement and it applies here.

- [ ] **Step 6b: correct the prose this task falsifies**

`tests/ir_dual_cases/arithmetic`'s header says "only arithmetic is promoted, so these rows are what says the unpromoted families still emit theirs from `eval.rs`".
That is false once this task lands.
Correct it -- do not hedge it and do not delete the rows, which still pin oracle bytes.

- [ ] **Step 7: gates**

Run each from `rust/`, reading each exit status unpiped:
`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `memcap 8G cargo test --workspace --no-fail-fast`.

- [ ] **Step 8: commit**

Stage the exact paths.
`git commit -F -` with a message saying what the op is and what it shares with `eval.rs`.

---

### Task 2: the prefix operators as a native op

**Files:**

* Modify: `rust/crates/rexx-exec/src/eval.rs`
* Modify: `rust/crates/rexx-exec/src/ir/mod.rs`
* Modify: `rust/crates/rexx-exec/src/ir/compile.rs`
* Modify: `rust/crates/rexx-exec/src/ir/drive.rs`
* Modify: `rust/crates/rexx-exec/src/ir/golden.rs`
* Modify: `rust/crates/rexx-exec/src/ir/golden_tests.rs`
* Modify: `rust/crates/rexx-exec/src/ir/corpus_shape_tests.rs`
* Modify: `rust/crates/rexx-exec/tests/ir_dual_cases/operators`

**Interfaces:**

* Consumes: Task 1's `Op::Binary`, and its widened `native_shape`.
* Produces: `Interp::apply_prefix(&mut self, op: PrefixOp, value: ObjRef) -> Result<ObjRef, Failure>`, `Op::Prefix { op: PrefixOp, src: u16, dst: u16 }`, `Op::TracePrefix { op: PrefixOp, src: u16 }`.

**The split.**
`Interp::eval_prefix` has the same seam, with one operand:

```rust
let frame = self.roots.push_frame();
let value = self.eval(code, operand)?;
self.roots.push_temp(value);
let result = /* the operator's own work */;
self.roots.pop_frame(frame);
Ok(result)
```

The operator's own work becomes `apply_prefix`, keeping the existing `match op` body verbatim and the measured facts in its doc comment.
`eval_prefix` keeps the prologue and calls it.
State the same rooting contract as Task 1: `apply_prefix` allocates, so its argument is rooted by the caller.

**Why the trace op is separate from `Op::TraceOperator`.**
A prefix operator does **not** trace `>O>`.
`trace_intermediate`'s `ExprKind::Prefix` arm calls `trace_prefix_op` with the spelling `+`, `-` or `\`, which is a different line from `echo_operator`'s.
So the echo needs its own op carrying a `PrefixOp`.

**The promotion.**
`native_shape` gains `ExprKind::Prefix { operand, .. } => native_shape(operand)`.
`push_native` gains a `Prefix` arm: emit the operand into `dst`, then `Op::Prefix { op, src: dst, dst }`, then `Op::TracePrefix { op, src: dst }`.
The operand lands in `dst` itself for the reason a binary operator's left operand does, and it is read before `dst` is written.

`compile.rs` has a family of adjacency assertions (`assert_literal_echoes_follow_their_load`, `assert_read_echoes_follow_their_load`, `assert_operator_echoes_follow_their_op`).
Add `assert_prefix_echoes_follow_their_op` in the same shape, checking position, register **and** operator -- an echo behind the wrong op lands in the right place with the wrong tag in it -- and call it beside the others.

**Steps:**

- [ ] **Step 1: write the failing golden test**

```rust
/// A prefix operator compiles to its own op and its own echo, and the echo
/// carries the operator: `\` and `-` trace different lines from one value.
#[test]
fn a_prefix_operator_compiles_to_a_native_op_and_its_own_echo() {
    /* assert_eq! against the full rendered stream of `za = -zb` */
}
```

- [ ] **Step 2: run it and watch it fail** -- `cargo test -p rexx-exec a_prefix_operator_compiles`, reading the run count.

- [ ] **Step 3: split `eval_prefix`, add `apply_prefix`, run the workspace green.**

- [ ] **Step 4: add both ops, drive them, extend `golden.rs`, `corpus_shape_tests.rs` and the adjacency assertion.**

`Root` gains a `Prefix` variant; `root_of` and `native` restate the widened set independently.

- [ ] **Step 5: add prefix stanzas to `tests/ir_dual_cases/operators`**, under Task 1's rules for that file -- oracle-captured bytes, a fresh empty directory, no `REWRITE` -- covering `+`, `-` and `\`, a prefix applied to an operator's result, a prefix applied to a prefix, `\` on a non-logical value, and `-` on a non-numeric value, with a `trace i` stanza so the `>P>` line's position is compared.

- [ ] **Step 6: gates, as Task 1 Step 7.**

- [ ] **Step 7: commit.**

---

### Task 3: a call's op names an address rather than a slot

**Files:**

* Modify: `rust/crates/rexx-exec/src/ir/mod.rs`
* Modify: `rust/crates/rexx-exec/src/ir/compile.rs`
* Modify: `rust/crates/rexx-exec/src/ir/drive.rs`
* Modify: `rust/crates/rexx-exec/src/ir/golden.rs`
* Modify: `rust/crates/rexx-exec/src/ir/golden_tests.rs`
* Modify: `rust/crates/rexx-exec/src/run.rs`

**Interfaces:**

* Consumes: nothing from Tasks 1 and 2.
* Produces: `ir::NodePath`, `Op::CallExpr { index: u32, slot: u16, path: NodePath, site: u16, dst: u16 }`, `Op::TraceFunction { index: u32, slot: u16, path: NodePath, src: u16 }`, `Interp::chunk_node_at(instruction: &Instruction, slot: u16, path: NodePath) -> Option<&Expr>`.

**This task promotes nothing new.**
It is the address widening on its own, so that Task 4 is a change to `native_shape` and `push_native` alone.
Every existing test must stay green with no expectation edited except the two rendered field names.

**The table is withdrawn: put the path in the op and widen the assertion to sixteen.**
This task was designed around a side table because `Op::CallExpr` at `{ index: u32, slot: u16, path: u16, site: u16, dst: u16 }` is twelve bytes of payload and breaks `size_of::<Op>() == 12`.
Moritz then offered sixteen bytes, and the record's entry 24 measured it: nine rounds, three arms, and **the width column is negative on every axis** while the cost a two-arm reading found belongs to *having an extra variant*, which this change does not do.
So `NodeAddr` and `Chunk::nodes` are withdrawn, and with them the reserve call, the `ChunkTooLarge` for addresses past `u16`, and the table lookup on the driver's hot path. `NodePath` is the encoding rather than the table and stays.

What lands instead:

* `const _: () = assert!(size_of::<Op>() == 16)`, with its doc comment saying what entry 24 measured rather than restating the old budget's argument.
* `Op::CallExpr { index: u32, slot: u16, path: NodePath, site: u16, dst: u16 }`.
* `Op::TraceFunction { index: u32, slot: u16, path: NodePath, src: u16 }`.
* `NodePath` stays -- it is the encoding, not the table -- and so does `chunk_node_at`, taking `slot` and `path` as two arguments rather than a `NodeAddr`.

**The width was checked with the compiler on 2026-08-12 and both ops fit**: with `path: u32` on each, `size_of::<Op>() == 16` holds, and the same build panics at 15 and at 20.

**Read that check's own trap before running any variant of it, and note that the trap is not the one the first two attempts wrote down.**
A first attempt added the fields, saw no `E0080`, and read that as the assertion passing.
It was not: the build output was piped through `head -4`, the four `E0063`/`E0027` field errors filled those lines, and the `E0080` was the fifth.
Setting the assertion to a value that cannot be true produced no visible panic either, for the same reason, which is what made the artifact look like a mechanism.
That produced a false explanation -- "const evaluation does not run while field errors stand" -- which reached this plan and a commit message; a later probe then refined it to "`E0063`/`E0027` suppress it, `E0308` does not", which is false in the same way and for the same reason.
**Measured 2026-08-12, unpiped:** adding a field to `Op::CallExpr` and leaving every site unedited gives `E0063`, `E0027` **and** `E0080` together.
Const evaluation runs.
So the only rule needed is: **never read a diagnostic list through `head`**, prove the assertion is live by making it fail on purpose once, and read the whole output when it does.

**`NodePath`.**
In `ir/mod.rs`:

```rust
/// The route from an expression slot's root down to the node one op names.
///
/// **One bit per step, most significant first, behind a sentinel `1`.** Every
/// descent `push_native` makes is into one of at most two children -- a binary
/// operator's left or right, a prefix operator's only operand -- so a step is a
/// bit, and a `u32` holds thirty-one of them before the sentinel falls off the
/// top. `NodePath::ROOT` is the slot's own expression, which is what a call
/// that *is* the whole slot carries.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct NodePath(u32);
```

with:

* `const ROOT: NodePath = NodePath(1);`
* `fn child(self, right: bool) -> Option<NodePath>` -- `None` when the sentinel would be shifted out, which is the thirty-second step.
* `fn steps(self) -> impl Iterator<Item = bool>` -- the bits below the sentinel, outermost first.

(A `NodeAddr` struct, a `Chunk::nodes` table, its reserve call and its `ChunkTooLarge` stood here.
They are what the "table is withdrawn" paragraph above withdraws: the path travels in the op, so there is nothing for a table to hold and nothing on the driver's hot path to look up.
Removed 2026-08-12, during Task 3, because the paragraph and this passage contradicted each other and a task briefed from the passage would have built the table the paragraph deletes.)

**The descent.**
In `run.rs`, `Interp::chunk_call_at` and `Interp::chunk_expr_at` are replaced by one function:

```rust
/// The expression an op's address names: expression `slot` of `instruction`,
/// then `path`'s steps down from that slot's root.
pub(crate) fn chunk_node_at(
    instruction: &Instruction,
    slot: u16,
    path: NodePath,
) -> Option<&Expr> {
    let mut node = match (&instruction.kind, slot) {
        (InstructionKind::Assignment { value, .. }, 0) => value,
        (InstructionKind::Say { expression: Some(expression) }, 0) => expression,
        _ => return None,
    };
    for right in path.steps() {
        node = match (&node.kind, right) {
            (ExprKind::Binary { left, .. }, false) => left,
            (ExprKind::Binary { right, .. }, true) => right,
            (ExprKind::Prefix { operand, .. }, false) => operand,
            _ => return None,
        };
    }
    Some(node)
}
```

The `None` answers are what `Loud::call_op_off_its_node` already reports; keep that failure.

**The two arms.**
`Op::CallExpr`'s driver arm resolves its node through `chunk_node_at`, then matches `ExprKind::Call { target, args }` for the pair it needs -- the match `chunk_call_at` used to do, moved to the one caller that wants it.

`Op::TraceFunction`'s arm gains an early `if !self.tracing_intermediates() { }` guard **before** the address is resolved, so an untraced run pays no descent.
`trace_intermediate` already returns immediately under the same condition, so the guard changes no output; it changes only what is computed before the call.
Confirm `tracing_intermediates` is reachable from `drive.rs` and widen its visibility if not.

**Steps:**

- [ ] **Step 1: add `NodePath` with its own unit tests**, in `ir/mod.rs`'s test module or a new one:

```rust
/// A path round-trips the steps it was built from, and refuses the step past
/// its width rather than silently dropping the sentinel.
#[test]
fn a_node_path_carries_thirty_one_steps_and_refuses_the_thirty_second() {
    let mut path = NodePath::ROOT;
    assert_eq!(path.steps().count(), 0);
    for step in 0..31 {
        path = path.child(step % 2 == 0).expect("thirty-one steps fit");
    }
    assert_eq!(path.steps().count(), 31);
    assert!(path.child(false).is_none());
}

/// The steps come back outermost first, which is the order the descent walks
/// them in -- a path read innermost first would land on the wrong node in
/// every asymmetric expression.
#[test]
fn a_node_paths_steps_come_back_outermost_first() {
    let path = NodePath::ROOT
        .and_then(|p| p.child(true))  /* adjust to the real API */
        ;
    /* assert the sequence is [true, false], not [false, true] */
}
```

- [ ] **Step 2: run them and watch them fail.**

- [ ] **Step 3: widen the assertion, add `path` to both ops and `chunk_node_at`; change their arms; delete `chunk_call_at` and `chunk_expr_at`.**

`compile`'s `push_value` passes `NodePath::ROOT`, which is exactly today's behaviour written in the new terms.
The assertion becomes `size_of::<Op>() == 16`, and **its doc comment has to be rewritten rather than have its number edited**: the paragraph above it argues the budget from "every op in every chunk pays for the widest variant", and what entry 24 measured is that the width itself was free on these axes while *adding a variant* was not. State what was measured and cite the entry.

- [ ] **Step 4: update `golden.rs` and every golden expectation whose rendered `CallExpr`/`TraceFunction` fields changed.**

Nothing else in the expectations may move.
If any other test's expectation changes, stop: the refactor was not behaviour-preserving, and that is the finding.

- [ ] **Step 5: gates.**

- [ ] **Step 6: commit.**

---

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
