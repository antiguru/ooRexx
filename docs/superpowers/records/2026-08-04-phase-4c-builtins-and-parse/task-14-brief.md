### Task 14: the compound-`DO` control-variable fix

**Files:** modify `crates/rexx-exec/src/run.rs` (`bind_control`), `crates/rexx-parse/src/ast.rs`, `rust/corpus/keyword-exempt.txt`, `docs/superpowers/plans/phase-4-exclusions.txt`

**Assigned to 4c by the 4b gate's Step 3c ruling.**

**What the divergence is.** `do cv.j = 1 to 5` is legal Rexx and the oracle iterates it, assigning the compound `CV.J` on every pass.
`bind_control` writes the control variable through a flat name-to-slot lookup, so `CV.J` becomes the literal name of one simple variable and no tail is resolved -- while the same executor resolves the same name correctly in `say cv.j` one line later.
**It is not a parse gap**: `cv.j` is a single symbol token and the parser interns `"CV.J"` whole.
The `LEAVE`/`ITERATE`/`END` forms naming a compound all dispatch correctly.

**The recorded cost cites a `rexx-parse` change, and its justification is wrong.**
The recorded phrasing was "`Controlled::control` carrying the `VariableRef` shape an assignment target already does".
Both halves are false: an assignment target is an `Expr` (`ast.rs:708-711`), and `VariableRef` is `Direct`/`Indirect(SymbolId)` with **no tail**, which also contradicts this row's own "it is not a parse gap".

**Re-derived 2026-08-07, and the answer is that no `rexx-parse` change is needed.** `Controlled::control` (`ast.rs:1014`) is already a `SymbolId`, and the parser already interns `CV.J` whole -- which is the same thing `say cv.j` starts from. So the shape the fix needs is already there, and the defect is entirely in how `rexx-exec` resolves that `SymbolId`: `bind_control` (`run.rs:5757`, taking `control: SymbolId`) goes through a flat name-to-slot lookup instead of the compound resolution the expression path uses.

**`ast.rs` is therefore dropped from the file list.** If you find you need it after all, say so before changing it -- that would mean this derivation is wrong.

- [ ] **Step 1: Reproduce all three narrowing probes before changing anything**

- [ ] **Step 2: Fix, then re-run `REXX_KEYWORD_GATE=1`**

**Seven** `base/keyword` bodies turn on this and they are the **only assertion failures in the whole table**. Named, so nobody has to count them again:

```
DO::test_DO_standardTest2A     DO::test_DO_standardTest2Q     ITERATE::test_12
DO::test_DO_standardTest2B     DO::test_DO_standardTest5-69   LEAVE::test_11
DO::test_DO_standardTest2P
```

*(Counted from the data rows 2026-08-07. `keyword-exempt.txt`'s own header block says "6 bodies" and is wrong -- the seventh is unaccounted for in that sentence, not in the table. Correct the header while you are there. This is the file's-header-explains-its-own-values trap in its most dangerous form: the count was inherited into this step, and an implementer asserting "the six" by name leaves the seventh unasserted while removing all seven.)*

When the fix lands, all seven start passing and `the_exempt_set_matches_the_current_failures` goes red until they are removed.

**A red test is not the success signal on its own** -- three different outcomes turn it red, and editing the rows' attribution instead of removing them turns it green again.
**Assert that all seven bodies now pass**, by name, before removing them.

- [ ] **Step 3: Remove the six rows, move the exclusions row to `CLOSED DEFECTS`, run the shared verify block, commit**

---

