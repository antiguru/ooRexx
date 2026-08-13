# `PROCEDURE` first in a `::ROUTINE` panics where the oracle raises 17.1

**Goal:** Refuse `PROCEDURE` where the oracle refuses it, instead of pushing a frame that later trips an invariant and aborts the process.

**Status:** open, unassigned, **and this is a panic rather than a wrong answer**. Found 2026-08-13 by the review of Task 2 of `2026-08-13-compound-name-resolution.md`, while adjudicating an unrelated concern about `exec_procedure`. Reproduced here before filing. Pre-existing: nothing in that plan's diff touches `run.rs` or `activation.rs`, and **both engines** fail identically.

## The transcript

```rexx
call sub
zz = 1
say zz
exit 0
::routine sub
procedure
say 'in sub'
```

Oracle, under the standard wrapper from a fresh empty directory:

```
     6 *-* procedure
     1 *-* call sub
Error 17 running .../p.rex line 6:  Unexpected PROCEDURE.
Error 17.1:  PROCEDURE is valid only when it is the first instruction executed
             after an internal CALL or function invocation.
```
rc 239.

This crate, `REXX_ENGINE=ir` **and** `REXX_ENGINE=tree-walker`, identically:

```
thread 'rexx-interp' panicked at crates/rexx-core/src/roots.rs:409:9:
assertion `left == right` failed: grow_slots on a frame that is not the top one
```
rc 101, and stdout empty -- the `say zz` never runs.

## The mechanism, as the reviewer described it

`Activation::routine` sets `entered_by_call: true`, so this crate **admits** a `PROCEDURE` that the oracle rejects.
`exec_procedure` then pushes a second frame onto an activation whose `owns_frame` is already true; `invoke_call` pops one on the way out; and the caller's next `slot_of` miss reaches `grow_slots` with a frame that is no longer the top one, which is the assertion that fires.

The panic is therefore two steps removed from the cause: the wrong decision is admitting the instruction, and the assertion is a later, correct guard noticing the damage.
**Fix the admission, not the assertion.**

## What the task must establish first

- [ ] **Step 1: the exact rule the oracle applies.** 17.1's own text says `PROCEDURE` is valid only as the first instruction executed after an internal `CALL` or function invocation. Capture the oracle for each case rather than reasoning from the sentence: an internal label called with `CALL`, an internal label called as a function, a `::ROUTINE` called with `CALL`, a `::ROUTINE` called as a function, a `::METHOD`, the main program, and `PROCEDURE` after some other instruction has already run in each of those.
* A `::METHOD` is Phase 5 and out of scope for the fix, but capture it anyway so the table is complete and the exclusion is a decision rather than a gap.

- [ ] **Step 2: find every construction of `entered_by_call`** and say which of the Step 1 cases each one covers. The defect is that one of them claims a property the oracle does not grant.

- [ ] **Step 3: raise 17.1 where the oracle raises it**, with the same clause echo -- note the oracle echoes the `procedure` line and then the `call sub` line, which is the caller's frame being reported, and that ordering is part of the expected bytes.

- [ ] **Step 4: keep the assertion.** It did its job: it caught a corrupted frame stack rather than letting a wrong slot be read. Do not relax it to make the program run.

- [ ] **Step 5: sweep for the same shape.** Any other instruction whose legality depends on how the activation was entered -- and any other place `owns_frame` is set true twice -- wants the same question asked of it. Write down what you checked, including what came back clean.

- [ ] **Step 6: a differential case**, both engines, plus the corpus sweep, and a count of how many corpus programs change.

## Why it matters more than its rarity suggests

A panic aborts the process with no Rexx condition, so a program cannot trap it and a caller sees rc 101 rather than a syntax error.
This project's standing rule is that a not-implemented path fails loudly with a code outside the `256 - major` band precisely so it cannot be confused with a raised condition; a panic is outside that discipline entirely.
