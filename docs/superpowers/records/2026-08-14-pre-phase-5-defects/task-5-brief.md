### Task 5: a `SELECT CASE` expression's clause boundary delivers a handler two columns short

**Files:**
- Modify: `rust/crates/rexx-exec/src/run.rs`, at the same call site Task 2 settled for a loop header's boundary
- Test: `rust/crates/rexx-exec/tests/trace_indent.rs`, the raw-stderr harness Task 2 created
- Re-capture: the note in `rust/crates/rexx-exec/tests/ir_dual_cases/loop-header-boundaries` that records this gap becomes false when it closes

**Interfaces:**
- Consumes: Task 2's fix and its `trace_indent.rs` harness. **Do not start this before Task 2 has landed.**

**Found by Task 2, deliberately not fixed there** because it is a different site that the assigned fix does not force, and recorded in commit `1638a0ea4`. Reproduced independently by the controller on that commit.

`SELECT` is a block instruction, so the oracle's level counter is already one deeper when a `SELECT CASE` expression's clause ends. This crate settles that boundary at the level the clause echoed at:

```rexx
trace r
call on user zx name h
zv = 'unset'
select case raiser()
  when 1 then say 'one'
  otherwise say 'other'
end
say 'after' zv
exit
raiser:
raise user zx return 2
h:
zv = 'set'
return
```

```
oracle                              this crate
   12 *-*     h:                       12 *-*   h:
   13 *-*     zv = 'set'               13 *-*   zv = 'set'
      >>>       "set"                     >>>     "set"
   14 *-*     return                   14 *-*   return
```

Both engines agree with each other; stdout and `rc 0` agree with the oracle. Only the trace stream differs, by the same two columns Task 2 fixed for a loop header.

**A trap queued by a `WHEN` condition instead agrees with the oracle**, because a `WHEN` is not a block instruction and its clause ends at the level it echoed. That is the adjacent success and it is what pins the rule to *block* instructions rather than to conditions or to `SELECT`.

- [ ] **Step 1: capture the baseline for the case above and for the `WHEN` case beside it**, both engines, raw un-normalised stderr, all three descriptors, before changing anything. The `WHEN` case must be shown already agreeing -- a fix that moves it has broken something.

- [ ] **Step 2: establish whether `SELECT CASE` is the last of them.**

Task 2 settled `DO`/`LOOP` and left this. Ask the same question of every remaining block instruction the oracle counts a level for, and write down what came back clean as well as what did not. **"Came back clean" does not distinguish a repaired probe from one that never had the defect** -- run each against a build without Task 2's fix as well, which is the control Task 2 used for its own hiding places.

- [ ] **Step 3: write the failing test in `trace_indent.rs`**, which compares raw bytes. A test routed through `tests/corpus.rs` or `tests/trace_oracle.rs` passes before and after, because DEVIATION 0 collapses exactly this signature. Run it and confirm it fails.

- [ ] **Step 4: fix, and confirm the `WHEN` case did not move.**

- [ ] **Step 5: correct the note in `ir_dual_cases/loop-header-boundaries`.** It currently records this gap as measured and open; when it closes, that comment states something false and must be corrected rather than hedged.

- [ ] **Step 6: the mutation witness, the raw blast-radius sweep and the gates**, as Task 2 did them: the raw count is the evidence and the harness count is not.

---

