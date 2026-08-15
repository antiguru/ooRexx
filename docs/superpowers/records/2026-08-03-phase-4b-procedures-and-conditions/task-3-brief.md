### Task 3: The body selector, `CALL`, `RETURN`, and the shared variable pool

**Spec:** design spec's "The borrow shape" and D19.

**Files:**
- Modify: `rust/crates/rexx-exec/src/activation.rs` (body selector; a sibling constructor to `Activation::new`; `trace_mode`)
- Modify: `rust/crates/rexx-exec/src/plan.rs` (`BodyKey::directive`)
- Modify: `rust/crates/rexx-exec/src/lib.rs` (delete `Interp::trace_mode`, field at **`:640`**, doc at **`:625-639`**)
- Modify: `rust/crates/rexx-exec/src/run.rs` (`run_activation`'s hardcoded `&program.main`; the `Call` and `Return` arms; the depth counter; `Flow`)
- Modify: `rust/crates/rexx-exec/src/error.rs` (push an activation site)
- Modify: `rust/crates/rexx-exec/tests/owners.rs`

**Interfaces:**
- Produces: a body selector field on `Activation` whose `None` is the main body and whose `Some(i)` is `directives[i]`'s -- the same shape `BodyKey::directive` carries. A sibling constructor to `Activation::new` that inherits `settings` and `trace_mode` from the caller. `Interp::depth: usize`. A `Flow` variant for `RETURN` (`Flow` currently has `Next`, `Goto`, `Exit`, `Leave`, `Iterate` and none of them expresses "unwind to the activation boundary").
- Consumes: `Raised::insufficient_stack()` (`src/error.rs:109`), which **already exists** -- do not write a second raiser.
- Consumes: **three mechanisms Task 2 built for you.** Read each field's own doc comment before using it; they were written for this task and one of them names it.

  * **`Interp::activation_indent` (`src/lib.rs:923`).** **Set, not added**, and `indent_offset` is **zeroed alongside it**, because the enclosing clause's printed indent already contains any escape elevation and leaving it would count that twice -- measured at 16 where the oracle prints 12. Save and restore both around the callee, the way the `Interpret` arm does, so nesting works. Its doc already states your value: **the calling clause's printed indent plus two**, from a measured `CALL` at printed indent 4 into a flat routine echoing the callee at 6.
  * **`Interp::clause_line_override` (`src/lib.rs:987`).** `INTERPRET` uses this because every echo in a fragment stack carries the **enclosing** clause's line. **`CALL` is the opposite case** -- each activation's echo carries **its own** line -- so this task must **not** set it, and must confirm a `CALL` inside a fragment still resolves each line correctly. That interaction is untested today.
  * **`seal_site_level` (`src/run.rs:3023`)**, which moves `failure_site` into the `failure_sites` stack. Call it at each activation boundary. Note `failure_site` is still **first-wins** by design; Task 7 owns clearing it on trap resumption.

**Why:** `run_activation` hardcodes `&program.main`. True for every activation 4a can build, false the moment a callee runs, and the failure is silent and with the right program.

**Read D9r above in full.** The default is a *shared* pool: a callee with no `PROCEDURE` reads and writes the caller's variables and its writes survive the return. **This task implements that default**, and Task 5 adds isolation. A callee without `PROCEDURE` gets **no new slot frame** -- reuse the caller's `SlotFrame`, save and restore `pc`, do not `pop_slots` on return.

**Your witness must have variables in it.** The first revision's witnesses were `sub: say 'callee'` and `f: return 41` -- neither has a variable, so both pass against an implementation that wrongly isolates every callee. That is 4a's own pathology: `Stem` witnessed, arithmetic witnessed, `Stem`-as-arithmetic-operand not, and a process abort shipped.

**Inherited items this task pays for:**

* **I1.** The body selector goes beside `Activation::program`. That field's doc describes the gap; update it rather than leaving a stale note.
* **I2.** `BodyKey::directive` is `Some(index)`-shaped and nothing has ever set it. **Decide it together with I1** or the activation's selector and the plan cache's key will denote different things.
* **I3.** `Activation::new` unconditionally defaults `Settings`. Measured: with `numeric digits 7`, an internal `call sub` sees 7, sets its own to 3, and after `return` the caller still reports 7. Keep two constructors: a fresh top-level run and a nested call begin from different starting settings.
* **I4.** `Interp::trace_mode` moves onto `Activation`. A deliberate 4a-only simplification: 4a has one frame, and measured, a callee's `trace off` does not survive its `return`. The field's doc names this as 4b's first move.
* **I6 and D10.** One Rust frame per activation plus an explicit counter. Measured: unbounded `CALL` recursion gives `Error 11.1`, "Insufficient control stack space", rc 245 -- a reportable condition, not a crash. Do not reopen D19.
* **I34.** The counter protects a sized caller only. On a default 2 MiB thread the native abort arrives at **331 parens or 341 calls**, long before any counter at 50,000 fires. The long-term answer is a documented minimum stack or a sized entry point in `rexx-parse`. This task measures 4b's contribution; it does not fix it.
* **I12's `CALL` half.** Each activation pushes a site entry carrying its own line and the indent base D2r defines.

**Measured semantics this task must reproduce:**

* **`RESULT` is dropped on return, not at the call.** A caller sets `result = 'before'`, calls a no-`PROCEDURE` routine, and the callee prints `inside result= before`. After `sub: return 42`, `RESULT` is `42`; after a bare `return`, `RESULT` reads as the derived name `RESULT`.
* **Omitted arguments.** `call sub 1,,3` with three `USE ARG` targets gives `[1] [Q] [3]` -- the omitted position leaves its target unset, and `arg()` still returns 3.
* **`CALL "SUB"` with `sub:` present is Error 43.1, rc 213**, `Could not find routine "SUB"`. The `literal` flag bypasses the internal label search, so 4b's correct answer is the loud 4c/Phase 7 fallback -- do not wire the literal form into the label table for symmetry.

- [ ] **Step 1: Measure the combined depth budget before writing the counter**

D19 chose per-activation Rust recursion; `run_bounded` already costs a Rust frame per source nesting level; Phase 5's dispatch will add a third. Measure both directions on our binary -- recursion depth to abort with no nesting, and with each activation containing a nested `DO` -- and compare against the oracle's 11.1 depth. **If our native abort arrives before the counter fires, the counter is decoration and the task must say so** rather than ship it silently.

- [ ] **Step 2: Write the failing tests, both with variables**

```rust
#[test]
fn a_routine_without_procedure_shares_the_callers_pool() {
    let out = run_source(
        b"v = 'caller-v'\ncall sub\nsay 'caller sees:' v w\nexit\n\
          sub:\nsay 'callee sees v:' v\nw = 'callee-w'\nreturn\n",
    );
    assert_eq!(out.stdout, b"callee sees v: caller-v\ncaller sees: caller-v callee-w\n");
}

#[test]
fn a_called_label_runs_its_own_clauses_not_the_main_body() {
    let out = run_source(b"call sub\nsay 'main'\nexit\nsub: say 'callee'\nreturn\n");
    assert_eq!(out.stdout, b"callee\nmain\n");
}
```

- [ ] **Step 3: Run them and watch them fail loudly**

- [ ] **Step 4: Add the body selector and the settings-inheriting constructor**

An internal `CALL` targets a **label in the same body**, so it does not exercise `BodyKey::directive`. Add a `::routine` test if one is reachable; **if it is not reachable in 4b, say so in the report** and leave `Some(index)` unset with its doc naming the phase that sets it. Task 9's `>I>`/`<I<` scope decision depends on this answer.

- [ ] **Step 5: Move `trace_mode` onto `Activation`**

The callee inherits the caller's value at call time and does not write back on return.

- [ ] **Step 6: Implement `Call::Named` and `Call::Dynamic`, and `Return`**

`Call` has four forms:

* `Named { name, literal, args }` -- this task's, including the `literal` fallback above.
* `Dynamic { target, args }` -- **this task's.** The first revision named it as 4b's and gave it no step. The target is an expression evaluated at run time, then resolved as a name.
* `Trap(ConditionTrap)` -- **Task 7's.** Keeps failing loudly until then.
* `Qualified { namespace, name, args }` -- **Phase 5's.** Must keep failing loudly with owner `Phase 5`.

Task 0 gave the witness table one row per form, so the last two are still covered after this task. Do not delete their witnesses.

Resolution order for a named call is internal label, then builtin, then external. 4b builds the front; the fallback fails loudly naming `4c`.

`RETURN` needs its own `Flow` variant -- the existing variants do not express "unwind to the activation boundary". Say what happens to a `RETURN` in the **main body** with no active call: measure it, because `loud.rs` may use that shape as its own witness.

- [ ] **Step 7: Implement the depth counter, calling the existing `Raised::insufficient_stack()`**

- [ ] **Step 8: Push an activation site entry using D2r's rule, and verify the three-deep transcript**

Call `seal_site_level` at the activation boundary and set `activation_indent` to the calling clause's printed indent **plus two**, zeroing `indent_offset` alongside it and restoring both on return. Do **not** set `clause_line_override` -- see Interfaces.

Capture the transcript fresh from the oracle. **Include a probe where the caller is lexically nested**, or the expectation cannot distinguish D2r's rule from `2 x depth`, which the first revision of this plan shipped and which agrees with the truth at caller indent 0. Task 2 measured the discriminating case: a call two `DO`s deep into a flat callee gives 6, where `2 x depth` predicts 2.

**One shape nobody has measured: a `CALL` inside an `INTERPRET` fragment, and an `INTERPRET` inside a called routine.** The two mechanisms compose and their composition is untested. Measure both directions and put the transcripts in the report.

**A warning specific to indentation in this phase.** Four separate agents have now got an indent rule wrong here, and every one measured a single shape and generalised: one nesting depth, one caller indent, one field's existing uses, one loop spelling. The rule you are implementing has itself been restated three times. Measure at two depths and two nesting shapes minimum, and if a claim in this brief does not reproduce for you, report that rather than coding to it.

**And there is a live 4a divergence underneath all of this** that is not yours to fix: a repetitive `DO`/`LOOP` that completes a body pass and then ends on a failing control test decrements the oracle's running indent counter. `phase-4-exclusions.txt`'s KNOWN GAP row on the re-tested pass has the measured tables, the scope rule, and a list of shapes still unmeasured. **Keep completed loops out of your witnesses**, or you will be chasing that instead of your own work.

- [ ] **Step 9: Implement `RESULT`, dropped on return**

- [ ] **Step 10: Run the full suite and both gates; 4a's 29 corpus programs must still match byte-for-byte**

If any moved, the site stack or the indent base is wrong. Do not adjust the expectation.

- [ ] **Step 11: Update `tests/owners.rs` and the pinned literals, close the `CALL` half of the KNOWN GAP row, commit**

---

