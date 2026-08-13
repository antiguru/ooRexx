# A trap handler's clauses echo at the wrong indent

**Goal:** Make a handler's clauses echo at the indent the oracle uses, and find out whether one mechanism explains both known instances or two.

**Status:** open, unassigned. The `SIGNAL ON SYNTAX` instance was found 2026-08-13 by Task 4 of `2026-08-12-condition-promotion.md` and reproduced here before filing. The `CALL ON` instance has been recorded since phase 4e.

## Instance one: `SIGNAL ON SYNTAX`, raise inside a callee

Captured 2026-08-13 against the tree at `0d058c086`, under the standard oracle wrapper from a fresh empty directory. `trace r`, `signal on syntax`, `call sub`, and `sub:` does `return 1/0`:

```
oracle                          this crate
  5 *-*   sub:                    5 *-*   sub:
  6 *-*   return 1/0              6 *-*   return 1/0
  7 *-* syntax:                   7 *-*   syntax:
  8 *-* say 'in handler'          8 *-*   say 'in handler'
    >>>   "in handler"              >>>     "in handler"
```

**stdout and the exit code agree.** Only the trace stream differs, and only in indent: the handler's clauses and their `>>>` lines keep the callee's indent where the oracle returns to the program's.

**It needs the callee.** The same trap raised in the main program -- `zz = 1/0` in place of the call -- agrees byte for byte on both engines. So this is `current_value_indent` not being restored when the trap transfers control out of the activation that raised, rather than anything about handlers as such.

**Both engines emit identical bytes**, so it is not the compiled form. Task 4's control probe (`say 1/0` in place of `return 1/0`, nothing promoted on the trapped path) diverges identically, so it is not that task's promotion either.

## Instance two: `CALL ON` at a promoted loop header

Already recorded in `phase-4e-gate.md` and `2026-08-09-phase-4e-ir.md`: a `CALL ON` handler delivered at a promoted loop header's boundary indents its own clauses **two spaces less** than the oracle -- `13 *-*   h:` against `13 *-*     h:`. Found by a Task 7-M3 review, which rebuilt the three files and reproduced identical bytes, so the enter/leave split did not introduce it either.

## The question this task exists to answer first

The two instances are **in opposite directions**: one handler's clauses are too deep, the other's too shallow.
That is the reason to treat them as one investigation and not two fixes.
Either one quantity -- the indent live across a transfer -- is restored in one path and not another, in which case one change fixes both and the directions are explained by which side each path errs on; or they are separate, in which case a fix for one must be shown not to move the other.

- [ ] **Step 1: reproduce both, side by side, and write the transcripts down** before changing anything. Both are pre-existing, so a capture taken now is a baseline that a fix has to move in exactly the two expected places.

- [ ] **Step 2: find where the indent is saved and restored across an activation boundary**, and where a trap transfer bypasses it. `current_value_indent` lives in `clause_state`; `run.rs` already carries a comment about a handler's own indent near the `CALL ON` delivery.

- [ ] **Step 3: decide whether it is one defect.** Say so in the report either way, with the evidence. A fix that lands without answering this is the failure mode -- the second instance has been recorded for weeks precisely because nobody asked whether it generalised.

- [ ] **Step 4: fix, and measure the blast radius.** Corpus sweep, both engines, and a count of how many corpus programs change output. Trace-indent changes are the class most likely to move transcripts far from the construct being fixed.

- [ ] **Step 5: check the third place this could hide.** A raise inside a callee inside a loop body; a `SIGNAL ON` and a `CALL ON` handler in the same program; and a handler that itself calls a routine.

## What this task must not do

Fix any other trace divergence it finds. Write it down. The value here is that a known quantity moves in a known number of places.
