### Task 2b: Normalise leading indentation in the differential harnesses

**Run this after Task 3 lands, not before** -- Task 3 is live in `rexx-exec`'s source and this touches the harnesses that grade it. It has no dependency on Task 3's output.

**Files:**
- Modify: `rust/crates/rexx-exec/tests/corpus.rs` (the stderr comparison in `check_case`)
- Modify: `rust/crates/rexx-exec/tests/trace_oracle.rs` (the committed-expectation comparison)
- Create: a small set of pinned indent witnesses that compare **without** normalisation

**Why:** `phase-4-exclusions.txt`'s DEVIATION 0, which this task implements. Read the row in full first; it carries the C++ citations and the exact scope.

**The scope is narrow and the narrowness is the point.** Collapse the run of spaces between a line's prefix field and its content, on **both** sides, for **stderr only**. Exit status, stdout, catalogue text, clause text, line numbers, value-line **content**, and the presence, absence and **order** of every line all stay byte-exact. A reviewer should be able to read the comparison function and see that it cannot hide a missing line, a reordered line, or a changed value.

**Why indentation and nothing else.** Indentation is *derived* from the clause sequence and block structure already being compared byte-for-byte, so matching it proves nothing further -- except that we have reproduced a C++ counter defect, which `BaseDoInstruction.cpp:161` against `:377` documents. Value lines are the opposite case: `>>>`, `>V>`, `>O>`, `>L>`, `>F>` and `>A>` carry intermediate results and evaluation order that nothing else in the output exposes.

**What must still fail.** Keep a small set of indent witnesses at nesting depth <= 3 containing **no completed loop**, compared without normalisation, so running a `WHEN` body at the wrong level still goes red.

**They already exist -- name and pin them rather than writing new ones.** `src/run.rs`'s unit tests assert exact `interp.trace` bytes with literal leading spaces (16 such assertions). `one_two_and_three_enclosing_dos_indent_by_two_four_and_six` is precisely DEVIATION 0's shape; `the_corrected_28x_indent_rule_matches_all_fourteen_probed_shapes` and `an_absorbed_whencases_escaping_false_branch_reports_end_at_its_own_residual_indent` are the others worth naming. They are unit tests, so normalisation cannot reach them -- the property to record is that this is **deliberate**, not that it happens to be true.

**One of them is a weak witness and the row should say so.** `the_indent_after_a_loop_has_already_exited_is_not_left_over_from_it` runs at top level, where the oracle's counter is already clamped at 0 and the correct and incorrect models agree. It is the test that *looks* like it covers the loop-decrement gap and does not. Do not count it toward the pinned set.

**Where the comparisons are:** `tests/corpus.rs:336` (`if rust.stderr != cpp.stderr`) is the single corpus site. `tests/trace_oracle.rs` compares committed `.expected` files in an `RC n` / `===STDOUT===` / `===STDERR===` format whose regeneration command is in its module doc -- normalise on the comparison path only, never on regeneration, or the committed expectations stop being oracle bytes.

**What this does not close.** The re-tested pass's two missing `>>>` lines are a **content** difference, not an indent one. Normalisation does not touch them and Task 9 still owns them.

- [ ] **Step 1: Write the normalising comparison, and a test that it still catches a missing line, a reordered line and a changed value**

Three negative controls, each a mutation of an expected transcript. If any passes under normalisation, the function is too permissive.

- [ ] **Step 2: Add the pinned no-normalisation indent witnesses**

- [ ] **Step 3: Confirm every corpus program still matches, and that the count did not move**

- [ ] **Step 4: Report how many previously-failing shapes now pass**

If the answer is zero, say so plainly -- it would mean the deviation bought nothing yet, which is a fact worth having rather than hiding.

- [ ] **Step 5: Commit**

---

