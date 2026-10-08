### Task 3: USE ARG targets, `.context~condition`, `CONDITION('D')`, stem attributes, OPTIONS, USE LOCAL, objects in DO/FORWARD/RAISE

**Files:** `run.rs` (`use_target_name` `:1856`, `OPTIONS` `:1197`, `USE LOCAL` `:1620`, `FORWARD`
`:2281`), `dispatch/context.rs:322`, `builtin/state.rs:360`, `dispatch.rs` (`:3059`, `:3078`),
`run/loops.rs` (`:716`, `:2233`), `run/condition.rs:733`, `lib.rs` (constructors `instruction_owner`
`Options` arm `:1014`, `use_local_in_a_method`, `object_position`, the accessor and delegate
constructors), pins listed in scout A section 3 (`dispatch/tests.rs:487`, `:491`, `:497`, `:1734`,
`eval/object_operand_tests.rs:188`, `:240`, `:337`, `tests/spike.rs:78`, `tests/loud.rs`,
`tests/concurrency_tests.rs` USE LOCAL and CONDITION "O" lines), `tests/owners.rs` (`:154`, `:324`,
`:468-473`), `corpus/refusal-sites.tsv`, corpus programs.

**Interfaces:** None new; reuses `assign_expr_target` (`run.rs:2510`), `Interp::condition_copy`
(`condition.rs:512`), `operator_message_receiver` (`eval.rs:1184`), `request_array`
(`dispatch/array.rs:647`).

- [ ] **Step 1:** Corpus programs from scout A's probes for c2 (`c_use_arg_msg`, `c2_use_arg_bracket`,
      `c2_use_strict_arg_msg`, `c2_use_arg_msg_default`, `c2_use_arg_msg_method`), c3
      (`c_context_condition`, `c_context_condition_call`), c4 (`c_condition_d_raise6`), c5/c6
      (`c_attr_compound`, `c_attr_stem`, `c_delegate_compound`, `c_delegate_stem`), b1 (`b_options`,
      `b_options_expr`), b2 (`b_use_local_method`, `b_use_local_method_expose`), b10 (`b_do_to_class`,
      `b2_do_to_class`, `b3_do_to_array`, `b2_do_by_inst`, `b3_do_by_ctx`, `b_do_ctrl_class`,
      `b_do_ctrl_env`, `b3_do_ctrl_inst`, `b3_do_ctrl_array`, `b3_forward_args_env`,
      `b3_forward_args_inst`, `b_raise_additional_env`, `b3_raise_additional_inst`,
      `b3_raise_additional_env`, `b3_raise_additional_class`, `b3_raise_additional_class_trace`). Each
      refuses today.
- [ ] **Step 2:** One commit per group: c2 through PARSE's message-term arm; c3 answers
      `condition_copy`'s Directory; c4 answers `''` for `(None, NOVALUE)`; c5/c6 bind the attribute or
      delegate to the object variable pool's stem or compound; b1 evaluates and traces the expression
      and does nothing else (remove `Options` from `instruction_owner`, its tag from `owners.rs`'s `INSTRUCTION_TAGS` and its
      `:324` row, and set the Phase 5 count assertion `:468-473` to 0; `tests/loud.rs` reads that
      table);
      b2 marks listed names local and exposes every other name on first touch; b10 routes DO header
      values and control variables through the operator send (97.1 when the receiver lacks the
      operator) and FORWARD/RAISE through `request_array` (98.939 without MAKEARRAY). Delete each
      refusal's constructor and its pins.
- [ ] **Step 3:** Witnesses agree; the per-task check. Commit.

