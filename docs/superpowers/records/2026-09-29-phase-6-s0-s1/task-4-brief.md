### Task 4: The `Activity`, holding the per-execution fields of `Interp`

**Files:** `rust/crates/rexx-exec/src/lib.rs` (the struct and `object_roots`), a new
`rust/crates/rexx-exec/src/activity.rs`, and every use site (derived).

**Interfaces:** Produces `pub(crate) struct Activity` and `Interp::activity: Box<Activity>` (the one
activity, one indexed load from `Interp`); accessors `Interp::act()`/`act_mut()` or direct field
access through `self.activity`, whichever the measurement favours.

- [ ] **Step 1:** Classify every `Interp` field as interpreter or activity, using the exhaustive
      destructure in `Interp::object_roots` (`lib.rs:2656-2886`) as the checklist. The activity set
      includes at least `value_buffer`, `running`, `suspended`, `spare_activations`, `clause_state`,
      `frames`, `flat_top`, `flat_spares`, `pending_traps`, `active_condition`, `fragments`, `depth`,
      `call_context`, `indent_offset`, `activation_indent`, `random_seed`, `native_handles`,
      `native_spares`, `procedure_permitted`, `region_procedure_permitted`, `trace_cache` and
      `debug_pause`; `clause_countdown` and `deadline` stay with the interpreter. Record the full
      classification table, one row per field with its reason, in the task report.
- [ ] **Step 2:** Move the activity fields into `Activity` in `activity.rs`, with its own exhaustive
      destructure in an `Activity::object_roots` that `Interp::object_roots` calls, so a new field in
      either struct is a compile error until classified. Mechanical: no logic change.
- [ ] **Step 3:** Corpus and collect-stress green; callgrind on every Task 2 program against the base.
      If the running total is over +0.3%, spend up to three rounds on the access path (field order,
      boxing, accessor inlining) before stopping for a ruling.
- [ ] **Step 4:** Gates; commit.

