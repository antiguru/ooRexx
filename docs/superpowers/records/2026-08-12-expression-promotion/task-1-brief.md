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

`is_comparison` is a new free function in `eval.rs` beside `is_arithmetic`, enumerating the eighteen operators `eval_node`'s comparison arm lists today, moved out of that arm rather than copied.

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
The measured behaviour each of the three collapsed arms documented -- `Blank` inserting exactly one space, the eighteen comparison operators, "both operands always evaluated, never short-circuited" -- must survive as doc comments on the functions that now own it.
Deleting a true comment to make the change easier is forbidden.

`Loud::binary_operator` is a new loud message for the one `Operator` no family claims, `Operator::Backslash` (the prefix `\` token, which the parser never builds an `ExprKind::Binary` from).
Follow the shape of the loud messages already in `loud.rs`.
**Do not write `unreachable!` here**: this project has shipped six wrong "cannot be reached" claims.

**The promotion.**
`eval::is_native_binary(op) -> bool` is `is_arithmetic(op) || is_concatenation(op) || is_comparison(op) || is_logical(op)`, spelled as a positive enumeration of the four families so that an operator added to `rexx_parse::Operator` does not silently become promotable.

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

* all three concatenation spellings, `||`, abuttal and blank, including that `Blank` inserts exactly one space however much whitespace separated the terms;
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

