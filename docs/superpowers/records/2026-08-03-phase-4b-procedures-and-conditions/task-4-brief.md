### Task 4: `ExprKind::Call` -- the internal-function form

**Files:**
- Modify: `rust/crates/rexx-exec/src/eval.rs` (a new `ExprKind::Call` arm)
- Modify: `rust/crates/rexx-exec/tests/owners.rs`

**Interfaces:**
- Consumes: Task 3's activation machinery, unchanged.
- Produces: the `ExprKind::Call` arm, whose fallback is where 4c hangs the builtin table.

**Why:** I25. `ExprKind::Call`'s owner string is `4b`, and the owner named is the phase after which the variant stops failing loudly *for some target*. Whichever sub-phase runs first builds the arm; 4b runs first.

**The target field is checked, and the answer changes what this task owes the owner tables.** `ExprKind::Call { target: CallTarget, args: Vec<Option<Expr>> }`, and `CallTarget` has exactly two forms -- `Symbol(SymbolId)` and `Literal(Box<[u8]>)` -- **both of which are 4b's**. There is no later-phase arm hiding inside it.

That makes this variant unlike `InstructionKind::Call`, which keeps its out-of-scope tag because `Call::Trap` and `Call::Qualified` are still loud and carry their own arm-grained witness rows. **`ExprKind::Call` has no such survivor**, so this task moves it fully in scope: delete its loud witness rather than splitting it, tag it `None` in `tests/owners.rs`, update `EXPECTED_OUT_OF_SCOPE` and the pinned counts, and provide a witness program in a corpus subset or `every_in_scope_variant_is_witnessed_by_the_subsets` fails.

**The two forms differ semantically and the difference is measured.** `CallTarget::Literal`'s own doc says it is never an internal label; confirmed on the oracle in a clean directory: `say "f"(1)` with an internal `f:` label present is **Error 43.1, rc 213, "Routine not found"**, while `say f(1)` runs the label and prints its value. So the literal form's 4b answer is the loud 4c/Phase 7 fallback, exactly as `CALL "SUB"` was in Task 3 -- do not wire it into the label table for symmetry.

**Measure that yourself in a fresh empty directory.** The first attempt at this measurement was taken in the scratchpad root, found a leftover `f.rex` on the external-routine search path, ran it, and reported 44.1 rc 212 instead. See the probe rule in Global Constraints.

**I25's split goes into `eval.rs`'s arm as a comment**, not only into `phase-4-exclusions.txt`. It is currently in the exclusions file and two test-file comments, and a 4c implementer reading `eval.rs` sees none of them. The comment says: internal routine first (4b), builtin second (4c), external third (Phase 7), and that a name reaching the fallback fails loudly naming `4c`.

**Measured:** the expression form does **not** touch `RESULT`. A function call at `trace i` emits `>F>   F => "2"` at the caller's indent before the enclosing expression's own `>>>`; `call sub 1` emits no `>F>`. Both emit `>A>` per argument at the call site. Task 9 implements the prefixes; this task must not emit them.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn an_internal_function_returns_its_value_into_an_expression() {
    let out = run_source(b"say f(1) + 1\nexit\nf: return 41\n");
    assert_eq!(out.stdout, b"42\n");
}
```

- [ ] **Step 2: Run it and watch it fail loudly**

- [ ] **Step 3: Implement the arm**

A routine that returns no value is an error in the expression form. **Measure the oracle's error number and text**; do not guess.

- [ ] **Step 4: Verify the fallback still fails loudly for a builtin name**

```rust
#[test]
fn a_builtin_name_still_fails_loudly_naming_4c() {
    let out = run_source(b"say length('abc')\n");
    assert_eq!(out.exit, NOT_IMPLEMENTED_EXIT);
    assert!(String::from_utf8_lossy(&out.stderr).contains("4c"));
}
```

This assertion depends on Task 0 having put the owner into the message. If Task 0's format differs from `4c`, use Task 0's format.

- [ ] **Step 5: Run the suite, update `tests/owners.rs` and the pinned literals, commit**

---

