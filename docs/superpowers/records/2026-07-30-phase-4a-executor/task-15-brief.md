### Task 15: The `base/expressions` assertion table

**Spec:** "L1, and why it is table-driven", exit criterion 2.

**Files:**
- Modify: `rust/crates/rexx-extract/src/lib.rs`, `src/bin/rexx-extract.rs`
- Create: `rust/crates/rexx-exec/tests/assertions.rs`

**Why:** `rexx-extract`'s current rendering produces programs whose main body is empty, so they execute nothing at all — verified under the oracle, which prints nothing and exits 0. The route is data, not programs.

- [ ] **Step 1: Add an extraction mode emitting one row per assertion** — the expression text, the expected value, and the `NUMERIC DIGITS` in force.

Those files change the setting throughout, from 1 to 100, so the extractor **scans sequentially and carries the setting**. Getting this wrong silently tests the wrong precision and still passes, which is the worst available outcome, so it gets its own test against a file that changes the setting mid-way.

- [ ] **Step 2: Include `PRECEDENCE` (1,226), which is self-contained literal arithmetic.** Phase 2 excluded it because it had no parser; 4a has one.

- [ ] **Step 3: `CONCATENATION` (388) needs a prelude, and adding it naively passes while testing nothing**

Every assertion in that group references variables `a` through `g` assigned at the top of its test method. A row of (expression, expected, digits) cannot carry them, and **part** of the resulting failure is silent: with `a`..`g` unset, each renders as a distinct single-character name, so any row whose expected value happens to match that all-distinct pattern passes while testing nothing. Measured, that is the 56 strict `==` and `\==` rows of the 388. The other 332 use non-strict `=` and turn on real blank-padding equalities between different variables, so they fail visibly instead — line 71 expects `0 0 1 1 0 1 0`, which unset operands do not produce.

Every row still needs the real prelude to mean anything. The reason to state the split precisely is that "all 388 would pass silently" was the original claim, and it is wrong: a reader who checks it, finds 332 loud failures, and concludes the hazard was imagined would then add the group naively.

So a row carries the method's **assignment prelude**, and any assertion whose prelude cannot be represented is listed as blocked rather than quietly included.

- [ ] **Step 4: Compare byte for byte, never numerically** — a numeric comparison would hide the entire created-digits and created-form story across thousands of rows.

- [ ] **Step 5: Prove the table can fail.** Perturb an expected value and confirm that row fails. A table that cannot fail is exactly the defect this criterion already had once, when it quantified over extracted programs that executed nothing.

- [ ] **Step 6: Report the row count and list rows blocked on 4b or 4c** with the sub-phase that unblocks each.

- [ ] **Step 5: Commit.**

---

