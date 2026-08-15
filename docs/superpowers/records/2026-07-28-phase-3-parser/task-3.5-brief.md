## Task 3.5: Expression grammar

**Files:**
- Create: `rust/crates/rexx-parse/src/expr.rs`, `src/ast.rs`
- Test: `rust/crates/rexx-parse/tests/expr.rs`

**Interfaces:**
- Consumes: `Clause` from Task 3.4; `ParseCtx` and `TokenCursor` from Task 3.3.
  Build the cursor with `TokenCursor::new(clause.tokens.clone())`, so an
  expression can never read past the end of its own clause.
- Produces: `Expr` in `ast.rs`, and `parse_expr(&ParseCtx, &mut TokenCursor) -> Result<Expr, ParseError>`.

Built the way Task 3.1 decided. The spike's implementation is a reference, not
a starting point — it was stopped at its first stall or first success, whichever
came sooner, and is not production code.

- [ ] **Step 1: Port the precedence table from the C++**

`LanguageParser.cpp`. Rexx's precedence is not C's: prefix `-` binds **tighter**
than `**`, so `-2 ** 2` is **4**, where C and Python give −4. Verified against
`build/bin/rexx`, and worth stating because the first draft of this plan
asserted the opposite from memory.

Verify every level the same way — against the binary, not the documentation —
and write the probe result into the test file as a comment beside each
assertion, so the next reader can see where the expectation came from.

- [ ] **Step 2: Write the failing tests, one per precedence level**

```rust
#[test]
fn prefix_minus_binds_tighter_than_power() {
    // build/bin/rexx: say -2 ** 2   =>   4      (C and Python give -4)
    assert_eq!(eval_shape("-2 ** 2"), "Power(Prefix(Minus, 2), 2)");
}

#[test]
fn abuttal_concatenation_binds_tighter_than_plus() {
    // build/bin/rexx: a = 1; b = 2; say a b + 1   =>   1 3
    assert_eq!(eval_shape("a b + 1"), "Concat(a, Add(b, 1))");
}
```

- [ ] **Step 3: Run, fail, implement, pass**

- [ ] **Step 4: Differential-test evaluation shape against the interpreter**

A parse tree cannot be compared directly to the interpreter, so compare
*results*: generate expressions over a small operand set, evaluate under
`build/bin/rexx`, and evaluate the parsed AST with a throwaway evaluator. Any
divergence is a precedence or associativity error. This is the Phase 2 method
applied to structure.

**Bound the evaluator, or it becomes Phase 4.** It needs exactly: numeric
literals, string literals, simple variables from a fixed table, the arithmetic
and comparison operators via `rexx-num`, concatenation both explicit and
abuttal, and parentheses. That is enough to catch every precedence and
associativity error. It does **not** need message sends, function calls,
compound variables or control flow — parse those, and assert on their tree
shape directly instead of evaluating them.

- [ ] **Step 5: Commit**

---

