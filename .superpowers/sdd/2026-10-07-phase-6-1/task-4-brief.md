### Task 4: Method objects and setMethod

**Files:** `dispatch/class_protocol.rs` (`:274`, `:777`, `:782`, `:1115`), `dispatch/object_protocol.rs`
(`:488`, `:531`, `:728`), `dispatch/executable.rs` (`:581`, `:622`), `directives.rs` (`:437-449`),
`environment/identities.rs` (`:201`, `ExecutableRecord`), `dispatch/context.rs:427`, `dispatch.rs`
(`:2082-2083`; a `NEW` row for VariableReference beside `:1102`), `run.rs:1536`, `lib.rs`
(`method_from_source`, `object_method`, `method_body`, `setup_method`, `expose_receiver`), pins in
`run/tests/directives.rs:264`, `:270`, `scheduler/tests/native.rs:301`, corpus programs.

**Interfaces:** Produces a body for every Method object this crate hands out (directive, constant,
attribute, delegate, native row), so `setMethod`, `run`, `enhanced` and `define` accept it.

- [ ] **Step 1:** Corpus programs from scout A's probes: b4 (`b_method_new_object_source`,
      `b7_method_new_bad_types`, `b7_routine_new_object_src`, `b7_setmethod_object_source`,
      `b7_run_object_source`, `b7_define_object_source`), b5 (`b10_setmethod_from_plain_method`,
      `b9_setmethod_from_directive_method`, `b10_run_constant_method`, `b10_enhanced_constant_method`,
      `b9_setmethod_from_attr_method`, `b9_setmethod_from_delegate`, `b3_send_run_items_array_subclass`,
      `b6_setmethod_primitive_array_subclass`, `b6_enhanced_with_primitive`,
      `b7_subclass_enhancing_primitive`, `b6_define_array_subclass_with_primitive`,
      `b10_define_constant_method`, `b10_define_attr_method`), b6 (`b7_subclass_enhancing_src`,
      `b7_mixinclass_enhancing_src`), b7 (`b11_ctx_exec_method_new`, `b11_ctx_exec_enhanced`,
      `b11_ctx_exec_floating_method`, `b_identities_scope_gone`), b8 (`b6_context_executable_routine_obj`),
      b14 (scout D's `cls_setm`, `cls_setmobj`, `cls_unset`;
      oracle `42 42`, `42 5`, `ok`), VariableReference `NEW` (93.967 naming the receiver's id, base
      and subclass).
- [ ] **Step 2:** b4 raises 93.961 from `Method~new`/`Routine~new` and 93.974 from `setMethod`/`run`/
      `define`; b5 gives every handed-out Method object a body record; b6 compiles class-method source as
      `:1115`'s instance path does; b7/b8 answer the Method or Routine object; b14 gives a class object
      OBJECT-scope methods and an EXPOSE pool; VariableReference gets its `NEW` row. Delete each refusal
      and its pins.
- [ ] **Step 3:** If `b6_define_array_subclass_with_primitive`, `b10_define_constant_method` or
      `b10_define_attr_method` still refuse with a Phase 9 label after b5, record them under row 9 in
      the gate record rather than fix them. A probe that a borrowed native row on a wrong-type receiver now reaches a
      `receiver_class` guard (scout A b13) and refuses, where the oracle crashes; it is Task 7's
      DEVIATION evidence, not a corpus program; save its text and both engines' output in the gate
      record. Witnesses agree; the per-task check. Commit.

