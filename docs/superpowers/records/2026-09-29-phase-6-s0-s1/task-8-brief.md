### Task 8: The in-loop driver for calls

**Files:** `rust/crates/rexx-exec/src/ir/drive.rs` (`run_chunk` `:366`, `run_ops`, `run_ops_from`
`:499`, `Op::Clause` `:560`, `Op::CallExpr` `:727`, `Op::CallArgs` `:799`, `Op::Call` `:1560`,
`Op::CallNamed` `:1601`), `run/call.rs` (`call_over_pushed_args` `:360`, `invoke_call_over` `:534`,
`settle_call_result` `:1069`, `invoke_named_call_over_pushed_args` `:1111`,
`call_over_installed_routine` `:496`), `run.rs` (`run_activation` `:382`), `eval.rs` (`enter_eval_node`
`:64`), `ir/drive.rs` frames.

**Interfaces:** Produces the driver's parked state (resume point and pending result register) kept in
the `Activity`, and a separate `#[inline(always)]` resume entry; `begin_call`/`finish_call` halves
whose composition is the recursive path the pinned route keeps.

- [ ] **Step 1:** Read `stackless-spike.patch` and `stackless-spike-findings.md`. Their shape (begin
      and finish halves, `drive`) is the starting point; their cost is what this task must beat: the
      spike's per-clause resume test (+3.5% on emptyloop) and its driver exit and re-entry per call
      (about 470 to 585 Ir per call).
- [ ] **Step 2:** Write the witnesses first: corpus programs for a SYNTAX raised three frames deep and
      trapped by CALL ON in the outermost caller, a SIGNAL out of a callee, TRACE I across nested
      internal calls, and Error 11 at the depth cap for a recursive function and a recursive CALL;
      each identical on the oracle. A collect-stress test that allocates inside a callee while the
      caller holds a value in a register.
- [ ] **Step 3:** Internal CALL, function calls and `::ROUTINE` calls push the callee's frame and
      continue in the same loop iteration; RETURN pops and delivers into the caller's register.
      Resumption lands mid-region through the separate entry; there is no per-clause resume test.
      The Error 11 cap is unchanged; native-stack depth no longer grows per Rexx call (record the
      cap-lifted depth reached, as the spike did).
- [ ] **Step 4:** Corpus, collect-stress, trace oracle tests, `ir/corpus_shape_tests.rs`,
      `ir/compile/invariants.rs`, `golden.rs`, `valid.rs` green (update the structural tests only for
      new shapes, never to hide a changed behaviour). Callgrind against the base; rounds as in
      Task 4, the fib programs being the ones the spike lost.
- [ ] **Step 5:** Gates; commit.

