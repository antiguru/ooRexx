### Task 7: Condition traps, `RAISE`, and `NOVALUE`

**Files:**
- Modify: `rust/crates/rexx-exec/src/error.rs` (`Raised::condition` gains its first reader; delete its `#[expect(dead_code)]`)
- Modify: `rust/crates/rexx-exec/src/run.rs` (the trap table; `SIGNAL ON`/`CALL ON`; the `Raise` arm; `failure_site` clearing)
- Modify: `rust/crates/rexx-exec/src/lib.rs` (`Novalue::Unset`, enum at **`:591`**, produced at **`:914`**)
- Modify: `rust/crates/rexx-exec/tests/owners.rs`

**Interfaces:**
- Consumes: `Raised`, `FailureSite`, `record_failure_site` (**`src/run.rs:1387`**), `Raised::report`, the catalogue and the 256-major exit rule -- all built and byte-verified in 4a.
- Produces: a per-activation trap table; `RAISE` reusing the existing raiser families.

**Why:** `RAISE` is the cheapest instruction in 4b's list because the raiser families already exist. The expensive half is resumption: a trap that transfers control after a raise is the first caller that makes `failure_site`'s never-cleared state matter.

**Inherited items this task pays for:**

* **I10.** `Raised::condition` has no reader. It is a field rather than a hardcoded value because `NOVALUE`, `NOMETHOD` and friends need to set it to something else, and it is `#[expect(dead_code)]` rather than `#[allow]` **on purpose, so the day 4b reads it the annotation asks to be deleted.** Delete it. Verified still present at `0fce4f00` in `error.rs`, reason string "no reader until 4b's SIGNAL ON and condition('c'); expect self-expires". Because it is `expect` and not `allow`, the compiler **errors** the moment you read the field, so the removal is forced rather than remembered -- that is the mechanism working, and reaching for `allow` to quiet it defeats the one thing it was placed there to do.

* **`set_sigl` already exists and traps need it.** `run.rs:2216` sets `SIGL` via `assign_by_name`, added by Task 6 for `SIGNAL` and `CALL`. A trapped condition sets `SIGL` too, so this task is its third caller. **Measure the value the oracle uses on a trap** -- do not assume it is the raising clause's line, and do not assume it matches what `SIGNAL` sets.

* **`Novalue::Unset` is produced at `lib.rs:1417` and read nowhere**, verified at `0fce4f00`. This task is its first reader, exactly as D16 intended.
* **I11.** `Interp::failure_site` is set first-call-wins. A second raise after a trapped first one would report the first site. **This task is the caller that makes it matter.** Clear it on trap resumption, and write a test with two raises where the second is the one reported.

  **The line numbers an earlier revision of this plan gave you are stale, and so are two doc comments in the tree.** Checked at `0fce4f00`: the guard is **not** in the callers and is **not** spelled `is_none()`. It is an early return at the top of `record_failure_site` itself:

  ```rust
  // run.rs:2841
  if self.failure_site.is_some() {
      return;
  }
  ```

  with the assignment three lines below. `record_failure_site` is at `run.rs:2812`; its callers are at `:1248`, `:1270` and `:2771`. Two doc comments -- `run.rs:2674` and `run.rs:2778` -- still say "`self.failure_site.is_none()` is the guard, in both callers", which is wrong about the expression *and* about where it lives. **Correct both while you are in there**, because a task told to clear this field would otherwise go looking in the callers.
* **I13.** `Novalue::Unset` is produced by the read path and read by nothing. D16 required the flag from the start rather than retrofitting a raise into the hottest path. `SIGNAL ON NOVALUE` is its first reader, and 4c's gate program uses `signal on novalue`.
* **I14, the `+++` half.** Measured: a trapped `SIGNAL ON SYNTAX` under `trace r` emits **no `+++` and no error report at all**; the trap label's own clause is echoed as an ordinary `*-*`. Condition traps do not bring `+++` into 4b. `+++` is command errors and failures, Phase 7's under D18.
* **I16, revisited.** 4a concluded `SIGNAL ON SYNTAX` cannot accumulate temps leaks, resting entirely on `step_in_temps_frame` being the single chokepoint: the trap acts at instruction-loop level and the wrapper has truncated before the `Failure` reaches the loop's `Err` arm. **This task makes the trap real.** Re-verify against the implementation; if Task 1 moved execution off the chokepoint, redo the analysis. `.superpowers/sdd/2026-07-30-phase-4a-executor/temps-frame-investigation.md` has the original.

**The vacuity hazard specific to this task.** A trap criterion asserting "the handler ran" by checking that the program exited 0 is satisfied by a program that never raised. Assert a **value the handler sets**, and pick one that is neither the flag's derived name nor its unset rendering -- an unset read yields the derived name, so a flag left unset renders as plausible data.

**A question this task inherited, now answered by Task 6 -- do not re-derive it.** Task 4's Critical was that `resolve_and_run_call` failed to restore `Interp::current_value_indent` across a nested activation. Task 6 then shipped the identical defect on a new field, `current_clause_line`, producing a wrong `SIGL`: `say f(1) + g(2)` gave the oracle's 1 against our 5, the first callee's last line.

**Both are fixed, and the fix is structural rather than another remembered rule.** The two fields are bundled into one `Copy` struct, `ClauseState`, and `resolve_and_run_call` saves and restores it as a single struct copy. A third field of this shape is therefore restored the moment it joins `ClauseState`, with no second edit anywhere. **If this task adds per-clause state that a nested activation can overwrite, put it in `ClauseState`** -- that is the whole mechanism, and adding a bare field to `Interp` instead is how the first two got missed.

**The `INTERPRET` arm needs no equivalent restore, measured rather than argued.** Task 6 ran three programs byte-for-byte against a live oracle -- two calls inside one fragment clause, and a call whose callee uses `INTERPRET` internally -- with zero diff on all three. The structural reason: an `INTERPRET` is always a whole clause of its own, so the arm never creates the "two things share one clause" shape that makes the omission observable, and `resolve_and_run_call`'s own restore covers call/fragment nesting in either direction.

**What is still open, and it is yours.** A condition trap that resumes execution mid-clause is the one shape that could still create two activations within one clause by a route neither Task 4 nor Task 6 could test, because neither had traps. Construct it and compare. If it diverges, the fix is to put the affected state in `ClauseState`, not to add another restore.

- [ ] **Step 1: Measure the trap transcripts, stdout, stderr and rc separately**

At minimum: `SIGNAL ON SYNTAX` with a raise, trapped; `CALL ON ERROR NAME handler` with no command (measured, handler not invoked, rc 0); `SIGNAL ON NOVALUE` reading an unset variable; `RAISE SYNTAX 40.4`; `RAISE ... RETURN` and `RAISE ... EXIT`; `RAISE PROPAGATE` from inside a trap; a trap in a **caller** for a condition raised in a **callee**; `SIGNAL ON` inside an `INTERPRET` fragment.

- [ ] **Step 2: Write the failing tests, asserting handler-set values rather than exit codes**

- [ ] **Step 3: Add the trap table to `Activation`, with dispatch decided by the Step 1 measurements**

Whether traps inherit into a callee, and whether a condition raised in a callee propagates up to a caller's trap, are both measured in Step 1 rather than assumed. Do not carry over the first revision's guess.

- [ ] **Step 4: Implement `SIGNAL ON`/`OFF` and `CALL ON`/`OFF`**

`SIGNAL ON` transfers control and does not return; `CALL ON` calls the handler and resumes. The two differ in exactly the way that makes `failure_site` clearing necessary.

- [ ] **Step 5: Implement `Raise`**

`Raise` carries `condition`, `propagate`, `rc`, `description`, `additional`, `array` and `result`. `RaiseResult { exit, value }` is the `RETURN`/`EXIT` tail.

- [ ] **Step 6: Wire `Novalue::Unset` to `SIGNAL ON NOVALUE`; delete `Raised::condition`'s `#[expect(dead_code)]`**

- [ ] **Step 7: Clear `failure_site` on trap resumption, with a two-raise test**

- [ ] **Step 8: Re-verify the temps-frame conclusion against the real trap**

Report the verification with the program used. "It still holds" without a program is not a verification.

- [ ] **Step 9: Run the suite and the corpus gate, update `tests/owners.rs`, commit**

---

