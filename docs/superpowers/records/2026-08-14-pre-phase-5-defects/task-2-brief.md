### Task 2: a trap handler's clauses echo at the wrong indent

**Background detail:** `docs/superpowers/plans/2026-08-13-handler-clause-indent.md`. Read it first -- it carries a hypothesis stated specifically so that Step 3 can refute it.

**Files:**
- Modify: `rust/crates/rexx-exec/src/run.rs` (`clause_state.current_value_indent` save/restore across a trap transfer)
- Possibly modify: `rust/crates/rexx-exec/src/ir/drive.rs`
- Test: transcripts under `rust/crates/rexx-exec/tests/`, both engines

**Interfaces:**
- Consumes: nothing from Task 1.
- Produces: nothing Task 3 consumes, but **both tasks change trace output and both touch `run.rs`**, so Task 3 starts from this task's committed tree.

**Instance one, verified on `328c51fcf`:** `trace r`, `signal on syntax name bad`, a `call sub` whose callee raises. The handler's clauses echo **four spaces too deep**:

```
oracle                          this crate
   11 *-* bad:                     11 *-*     bad:
   12 *-* say 'trapped'            12 *-*     say 'trapped'
```

**It needs the callee.** The same trap raised in the main program agrees byte for byte on both engines. Both engines emit identical bytes here, so it is not the compiled form.

**Instance two, recorded since phase 4e** in `phase-4e-gate.md` and `2026-08-09-phase-4e-ir.md`: a `CALL ON` handler delivered at a promoted loop header's boundary indents its own clauses **two spaces less** than the oracle -- `13 *-*   h:` against `13 *-*     h:`.

**Instance three, found by Task 1 and reproduced independently on `3cf9fcaba`: no trap is needed at all.** A plain `SIGNAL` inside a `CALL`ed label, with no condition handler anywhere in the program:

```rexx
trace r
call sub
exit 0
sub:
signal onward
onward:
zz = 1 / 0
```

Both engines echo the clauses after the `SIGNAL` **two spaces too deep**, and agree with the oracle on stdout and on `rc 214`:

```
oracle                          this crate
    6 *-* onward:                   6 *-*   onward:
    7 *-* zz = 1 / 0                7 *-*   zz = 1 / 0
```

**This is the sharpest form of the defect and it reframes Step 3's hypothesis**: handlers are not the subject. A `SIGNAL` that leaves a called label does not restore the indent, whether or not a condition raised it. Start from this case -- it has the fewest moving parts of the three.

**The instances are in different directions**, which is why they are one investigation and not separate fixes.

### The harnesses cannot see this defect, and that governs how the task is tested

`phase-4-exclusions.txt`'s **DEVIATION 0** is the one normalisation the differential harnesses apply to `stderr`, shared by `tests/corpus.rs` and `tests/trace_oracle.rs`: it collapses the run of ASCII spaces between a trace line's prefix marker and its content down to a single space. **That is exactly this defect's signature.**

Two consequences, both binding:

* **The test for this fix must compare raw, un-normalised `stderr`.** A test that goes through the shared harness will pass before the fix and after it, which is the "test that cannot fail" this project treats as a defect.
* **The corpus-sweep count in Step 5 will read zero changed programs even if the fix moves many transcripts.** Report the raw count as well, by comparing un-normalised bytes across the corpus before and after. A sweep count taken through the normalising harness is not evidence here and must not be reported as though it were.

Do not change DEVIATION 0 or its normalisation in this task. It is a recorded, accepted deviation with its own justification; whether it should survive this fix is a question for the report, not an edit.

- [ ] **Step 1: reproduce both, side by side, and write the transcripts down before changing anything.**

Both are pre-existing, so a capture taken now is the baseline a fix has to move in exactly the two expected places. Capture under `trace r` and `trace i`, both engines, all three descriptors.

- [ ] **Step 2: find where the indent is saved and restored across an activation boundary, and where a trap transfer bypasses it.**

`current_value_indent` lives in `clause_state`. `run.rs` already carries a comment about a handler's own indent near the `CALL ON` delivery.

- [ ] **Step 3: decide whether it is one defect, and say so in the report either way, with the evidence.**

The hypothesis to refute, from the detail file: *this crate keeps whatever indent is live at the transfer, where the oracle sets it from the kind of transfer* -- `SIGNAL ON` unwinds so the handler belongs at the program's indent; `CALL ON` invokes the handler as a subroutine so its clauses belong one level deeper. It predicts that a fix which merely restores an indent fixes the `SIGNAL` case and leaves the `CALL ON` case exactly as wrong.

**A fix that lands without answering this is the failure mode.** The second instance has been recorded for weeks precisely because nobody asked whether it generalised.

**Read the transcripts against the possibility that the oracle is wrong.** This project has found a genuine upstream trace-indent defect already: two loop-exit paths in the C++, one restoring the indent and one bare-decrementing it, so a completed loop under-indents later clauses. That one is not this one, but "the oracle is wrong here" is live and must be decided from the transcripts rather than assumed away.

- [ ] **Step 4: write the failing test, run it, confirm it fails.**

Both engines, all three descriptors, against oracle bytes captured in Step 1.

- [ ] **Step 5: fix, and measure the blast radius.**

Corpus sweep, both engines, and a count of how many corpus programs change output. **Trace-indent changes are the class most likely to move transcripts far from the construct being fixed**, so the count is the finding, not a formality.

- [ ] **Step 6: check the third place this could hide.**

A raise inside a callee inside a loop body; a `SIGNAL ON` and a `CALL ON` handler in the same program; and a handler that itself calls a routine. Report each, including the ones that came back clean.

- [ ] **Step 7: the mutation witness**, then gates and commit, as Task 1 Steps 7 and 8.

---

