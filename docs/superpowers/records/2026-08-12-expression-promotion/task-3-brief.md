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
So `NodePath`, `NodeAddr` and `Chunk::nodes` are all withdrawn, and with them the reserve call, the `ChunkTooLarge` for addresses past `u16`, and the table lookup on the driver's hot path.

What lands instead:

* `const _: () = assert!(size_of::<Op>() == 16)`, with its doc comment saying what entry 24 measured rather than restating the old budget's argument.
* `Op::CallExpr { index: u32, slot: u16, path: NodePath, site: u16, dst: u16 }`.
* `Op::TraceFunction { index: u32, slot: u16, path: NodePath, src: u16 }`.
* `NodePath` stays -- it is the encoding, not the table -- and so does `chunk_node_at`, taking `slot` and `path` as two arguments rather than a `NodeAddr`.

**The width was checked with the compiler on 2026-08-12 and both ops fit**: with `path: u32` on each, `size_of::<Op>() == 16` holds, and the same build panics at 15 and at 20.

**Read that check's own trap before running any variant of it.** A first attempt added the fields, saw no `E0080`, and read that as the assertion passing. It was not evaluated at all: four `E0063`/`E0027` field errors stood in front of it, and const evaluation does not run while they do -- setting the assertion to a value that cannot be true produced no panic either, which is what exposed it. Fix every field error first, then read the assertion, and prove it is live by making it fail on purpose once.

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

`NodeAddr` is `{ slot: u16, path: NodePath }`, `Copy`.

`Chunk` gains `nodes: Vec<NodeAddr>` with the same "dense over the ops that need it" doc argument `Hints` and `Calls` carry, and `fn node(&self, at: u16) -> Option<NodeAddr>`.
`compile` builds it alongside `hints` and `calls`; pushing past `u16::MAX` is `ChunkTooLarge { what: "expression addresses past u16" }`.

**The descent.**
In `run.rs`, `Interp::chunk_call_at` and `Interp::chunk_expr_at` are replaced by one function:

```rust
/// The expression an op's address names: expression `addr.slot` of
/// `instruction`, then `addr.path`'s steps down from its root.
pub(crate) fn chunk_node_at(instruction: &Instruction, addr: NodeAddr) -> Option<&Expr> {
    let mut node = match (&instruction.kind, addr.slot) {
        (InstructionKind::Assignment { value, .. }, 0) => value,
        (InstructionKind::Say { expression: Some(expression) }, 0) => expression,
        _ => return None,
    };
    for right in addr.path.steps() {
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

