### Task 1: `PROCEDURE` first in a `::ROUTINE` panics where the oracle raises 17.1

**Background detail:** `docs/superpowers/plans/2026-08-13-procedure-in-routine-panic.md`. Read it first; it carries the mechanism as the reviewer described it.

**Files:**
- Modify: `rust/crates/rexx-exec/src/run.rs` (`exec_procedure`, and wherever `PROCEDURE` admission is decided)
- Modify: `rust/crates/rexx-exec/src/activation.rs` (`Activation::routine`, `entered_by_call`)
- Do NOT modify: `rust/crates/rexx-core/src/roots.rs:409` -- that assertion is correct and stays
- Test: a corpus program plus a differential case, both engines

**Interfaces:**
- Produces: nothing later tasks consume. This task is self-contained.

**The reproduction, verified on `328c51fcf`:**

```rexx
call sub
zz = 1
say zz
exit 0
::routine sub
procedure
say 'in sub'
```

Oracle: echoes the `procedure` line, then the `call sub` line, then

```
Error 17 running .../p.rex line 6:  Unexpected PROCEDURE.
Error 17.1:  PROCEDURE is valid only when it is the first instruction executed after an internal CALL or function invocation.
```

at **rc 239**. This crate, on **both** engines: panics at `crates/rexx-core/src/roots.rs:409` with `grow_slots on a frame that is not the top one`, **rc 101, stdout empty** -- the `say zz` never runs.

**The mechanism:** `Activation::routine` sets `entered_by_call: true`, so this crate admits a `PROCEDURE` the oracle rejects. `exec_procedure` pushes a second frame onto an activation whose `owns_frame` is already true; `invoke_call` pops one on the way out; the caller's next `slot_of` miss reaches `grow_slots` with a frame that is no longer the top. **The wrong decision is admitting the instruction. Fix the admission, not the assertion.**

- [ ] **Step 1: capture the oracle's actual rule, case by case.**

Do not reason from 17.1's sentence. Capture one oracle transcript per case, into a fresh empty directory, and write the table into your report:

  * an internal label reached by `CALL`, `PROCEDURE` first
  * an internal label reached as a function call, `PROCEDURE` first
  * a `::ROUTINE` reached by `CALL`, `PROCEDURE` first
  * a `::ROUTINE` reached as a function call, `PROCEDURE` first
  * the main program, `PROCEDURE` first
  * each of the above with some other instruction executed before the `PROCEDURE`
  * a `::METHOD`, `PROCEDURE` first

A `::METHOD` is Phase 5 and out of scope for the fix. **Capture it anyway**, so the exclusion is a recorded decision rather than a gap.

- [ ] **Step 2: find every construction of `entered_by_call` and map each to a Step 1 case.**

`/bin/grep -rn "entered_by_call" rust/crates/`. For each construction site, say in your report which Step 1 case it covers and whether the oracle grants the property it claims. The defect is that one of them claims a property the oracle does not grant.

- [ ] **Step 3: write the failing differential test.**

Add the reproduction above as a corpus program, or as a test that runs it under both engines and compares against the oracle bytes you captured in Step 1. It must fail before the fix with the panic, and it must check stdout, stderr and exit status separately.

Run it and confirm it fails with `rc 101`.

- [ ] **Step 4: raise 17.1 where the oracle raises it.**

The clause echo ordering is part of the expected bytes: the oracle echoes the `procedure` line and **then** the `call sub` line, which is the caller's frame being reported.

- [ ] **Step 5: run the test and confirm it passes**, on both engines, byte for byte against the oracle on all three descriptors.

- [ ] **Step 6: sweep for the same shape, and write down what came back clean.**

Any other instruction whose legality depends on how the activation was entered, and any other place `owns_frame` is set true twice. Report what you checked including the clean results -- a sweep that reports only its finds is indistinguishable from one that stopped early.

- [ ] **Step 7: the mutation witness.**

Break the fix deliberately (re-admit the instruction) and confirm the new test goes red. Restore from a `cp` backup, `touch`, `sha256sum -c`, rebuild, re-run.

- [ ] **Step 8: gates, corpus sweep, commit.**

`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `memcap 8G cargo test --release --workspace --no-fail-fast`. Report how many corpus programs changed output. Commit with `git commit -F -`, then read the hash back with `git log`.

---

