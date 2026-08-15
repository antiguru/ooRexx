### Task 13: Trace

**Spec:** D17, and exit criterion 3.

**Files:**
- Create: `rust/crates/rexx-exec/src/trace.rs`
- Modify: `rust/crates/rexx-exec/src/eval.rs` — the value-line events (`>L>`, `>V>`, `>O>`, `>=>`) are emitted from expression evaluation, which means threading an event through roughly eighteen `eval_node` arms.
- Modify: `rust/crates/rexx-exec/src/run.rs` — the `*-*` clause event, and `InstructionKind::Trace`'s own `step` arm, which no step below mentions and which nothing implements today.
- Test: a `#[cfg(test)] mod tests` inside `rust/crates/rexx-exec/src/trace.rs`, with the committed expectations under `rust/crates/rexx-exec/tests/trace_oracle/`

> **There are zero trace emission points in `eval.rs` or `run.rs` today**, and the only writer to the trace sink is `execute`'s error path. So this task adds the hook to code that already works, rather than finding it in place.
>
> **An earlier version of this note called that a breach of D17 and a process failure. That verdict was wrong and is withdrawn.** D17 does say "emit the event from the start", but its stated purpose is to forbid constant folding and expression fusion so the evaluator is not designed twice, and the shipped `eval_node` does neither, so the harm D17 names never occurred. This plan has also scheduled trace at Task 13 since revision 1, which means Tasks 7 to 9 followed the governing document exactly; a brief carrying all of D17 would not have changed what they built. The note is kept rather than deleted because recording a sound decision as unimplemented invites a later reader to reopen it.
>
> **The retrofit is also far smaller than that note claimed.** It said roughly eighteen `eval_node` arms; there are ten top-level `ExprKind` arms, and more to the point `eval` was already split from `eval_node` precisely so that every exit path including the `?` ones goes through one place. A post-order value event with the value in hand is therefore **one** insertion point in `eval`, not a threading job across arms.
>
> Two things from that note stand on their own merits and are not withdrawn. Sequence this task **after** Task 11, because `run.rs` is the most contended file in the phase, and **read the committed `run.rs`** rather than designing against this plan.

- [ ] **Step 1: Build the prefix table from the oracle's side, not from what we emit**

All 19 prefixes at `RexxActivation.hpp:90`-`110`. Each row is either a witness program 4a emits, or the sub-phase that first emits it. Measured reachable from pure-4a code: `*-*`, `>>>`, `>=>`, `>L>`, `>V>`, `>O>`, `>K>`, `>C>`, and a prefix-operator line.

- [ ] **Step 2: Capture the oracle's exact bytes for each witness** — spacing, quoting and indentation are unspecified anywhere but the oracle. Commit the expectations the way `rexx-parse/tests/sourceline_oracle/` does, with a regeneration command named in the reading test, so `cargo test` alone is the gate.

> **This is the opposite strategy from the corpus runner, deliberately, and both are right.** `tests/corpus.rs` runs the oracle **live** and compares in process, because it exists to track progress across tasks and a committed expectation would have to be regenerated on every task that changes behaviour. Trace expectations are committed instead, because a trace witness is a fixed artefact whose whole value is that it cannot drift silently, and because capturing it requires a running oracle that a machine checking out this tree may not have. Neither section knew about the other when they were written, so the difference looked like an inconsistency; it is a real distinction between an instrument and an expectation. Do not "unify" them.

- [ ] **Step 3: Implement**, emitting to the trace sink (stderr) per evaluation step, gated on the setting.

- [ ] **Step 4: Verify** — `trace r` and `trace i` byte for byte against every committed expectation.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec/src/trace.rs rust/crates/rexx-exec/src/eval.rs rust/crates/rexx-exec/src/run.rs rust/crates/rexx-exec/tests/trace_oracle
git commit -m "Trace value lines, quantified from the oracle's nineteen prefixes"
```

---

