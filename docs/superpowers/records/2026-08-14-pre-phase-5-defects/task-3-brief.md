### Task 3: the `>K>` line a `DO OVER ... FOR` does not print

**Background detail:** `docs/superpowers/plans/2026-08-13-over-for-keyword.md`.

**Files:**
- Modify: `rust/crates/rexx-exec/src/run.rs` (`HeaderRole::keyword()`, and its doc comment in the same edit)
- Verify, and modify only if needed: `rust/crates/rexx-exec/src/ir/compile.rs`
- Re-capture: `ir_dual_cases/loop-header-values`, and any `trace_oracle` transcript holding a `DO OVER ... FOR`

**Interfaces:**
- Consumes: Task 2's committed tree, since both change trace output in `run.rs`.

**The divergence, captured 2026-08-13** on `trace r`, program `zs = 'a b c'` then `do qq over zs for 1`:

```
oracle                              this crate
  >K>   "OVER" => "a b c"             >K>   "OVER" => "a b c"
  >K>   "FOR" => "1"                  (nothing)
```

`HeaderRole::OverFor::keyword()` answers `None`, so no `Op::TraceKeyword` is emitted and the tree-walker echoes nothing either. **Both engines agree with each other**, so this is the role table withholding a keyword, not anything the compiled form does.

**A note on probing this construct:** a probe reaching for `.array~of` hits the declared Phase 4 message-send gap and exits 120, which says nothing about the defect. Build the probe from values Phase 4 can produce -- a blank-delimited string, a number.

**This is a defect and not an exclusion.** `phase-4-exclusions.txt` holds work assigned to a later phase and permanent chosen differences; this is a difference nobody chose, in a construct otherwise in scope and implemented. Do not add a row there.

- [ ] **Step 1: find out whether `OverFor` is alone.**

Capture the oracle for **every** `HeaderRole` variant under both `trace i` and `trace r`, one program per variant, and write the transcripts down before changing anything.

`Initial` is the one other variant documented as echoing no `>K>`, and that is recorded as measured rather than assumed. **Confirm it rather than trusting the comment** -- the comment beside `OverFor` made the same kind of claim and was wrong. A second withheld keyword found here changes the shape of the fix from one variant to a table.

- [ ] **Step 2: write the failing test, run it, confirm it fails.**

- [ ] **Step 3: give the role its keyword, on the tree-walker.**

`HeaderRole::keyword()` in `run.rs`. **Correct its doc comment in the same edit:** it currently says `None` is "for the roles the oracle echoes nothing for", which is the sentence that made this gap invisible.

- [ ] **Step 4: let the compiled engine follow.**

`compile.rs` emits `Op::TraceKeyword` where `role.keyword().is_some()`, so the compiled side should need no change of its own. **Verify that rather than assume it**, and if it does need one, say so in the report.

- [ ] **Step 5: re-capture the transcripts from the oracle, never by hand.**

`ir_dual_cases/loop-header-values` pins the gap on purpose and its rows now change.

- [ ] **Step 6: run the corpus sweep and the gates**, and say in the report how many corpus programs changed output. **A construct this narrow should move few; a surprise there means Step 1 missed something.**

- [ ] **Step 7: the mutation witness**, then commit, as Task 1 Steps 7 and 8.

---

