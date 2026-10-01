# Task 4 report: the `Activity`

Status: DONE_WITH_CONCERNS

## Classification

Test for an unlisted field: would a second concurrently running activity need its own copy for
correctness? Checklist: the exhaustive destructure in `Interp::object_roots` at 027ef469c.

### Activity (`rexx-exec/src/activity.rs`)

| field | reason |
|---|---|
| `value_buffer` | brief; holds one clause's builtin argument values, cleared at clause start |
| `locals` | `SETLOCAL` snapshots live on the oracle's top-level activation, so one stack per activity |
| `running` | brief; the activation stack |
| `trace_cache` | brief; mirrors the running activation's `TRACE` setting |
| `suspended` | brief; the activation stack |
| `spare_activations` | brief; reuse pool fed and drained by this activity's push/pop |
| `thread` | spec 2.4: each activity gets its own thread context (it records the innermost native activation); it is an `Rc` handle, so libraries that keep it are unaffected |
| `native_handles` | brief; spec 2.4 per-activity native-handle stack |
| `native_spares` | brief; reuse pool of that stack |
| `clause_state` | brief; the running clause's line and value indent |
| `flat_loops` | the op driver's open flat loops, same stack as `flat_top` |
| `flat_top` | brief |
| `flat_spares` | brief |
| `frames` | brief; the op driver's open constructs |
| `pending_traps` | brief; traps queued for this activity's activations |
| `active_condition` | brief |
| `pending_additional` | a raise's ADDITIONAL between the raise and its condition object: one raise per activity in flight |
| `reraised_object` | `RAISE PROPAGATE` in flight on this activity |
| `reraise_leaving` | same |
| `pending_result` | same as `pending_additional`, for an extension-raised RESULT |
| `pending_rc` | same, for a command handler's RC |
| `current_case_text` | the innermost running `SELECT CASE`'s value |
| `indent_offset` | brief |
| `activation_indent` | brief |
| `failure_site` | the failure unwinding this activity's levels |
| `failure_sites` | same |
| `failure_frame` | same |
| `failure_frames` | same |
| `failure_origin` | same |
| `failure_propagated` | same |
| `failure_reraised` | same |
| `native_reraise` | the failure an extension call on this activity is answering |
| `outer_caller` | whether this activity's `running` is a swapped-in outer caller |
| `input_dispatch` | the send a redirection reader is running, on this activity |
| `input_dispatch_trapped` | same |
| `input_dispatch_syntax` | same |
| `clause_line_override` | set while an `INTERPRET` fragment on this activity runs |
| `fragment_depth` | this activity's `INTERPRET` nesting |
| `fragments` | brief |
| `fragment_clause` | same as `fragments` |
| `debug_pause` | brief |
| `routing_trace` | a trace delivery in progress on one activity must not divert another's trace lines to the buffer |
| `depth` | brief; eval recursion depth of this activity's chain |
| `max_depth` | the deepest `depth` reached, compared against `depth` |
| `stack_entry` | Rust-stack address of the chain `depth` counts |
| `stack_first` | same |
| `stack_deepest` | same |
| `procedure_permitted` | brief |
| `region_procedure_permitted` | brief |
| `call_context` | brief |
| `random_seed` | brief; per activity in the oracle (`Activity.hpp:426`) |
| `elapsed_anchor` | `TIME('E')` anchor; the oracle keeps it on the activation (`ActivationSettings`) |
| `pending_elapsed_reset` | same (`elapsedReset`) |
| `requires_installing` | the oracle's `Activity::requiresTable` |
| `pins` (feature `pinning`) | the pinned-frame stack records frames on one activity's Rust stack; the report stays on `Interp::pinning` |

### Interpreter (`Interp`)

| field | reason |
|---|---|
| `heap`, `roots` | the one heap and its root set (per-activity roots are Task 5) |
| `key_buffer`, `parse_buffers`, `result_buffer` | lent by `take`/`pop` and handed back; a nested borrower gets a fresh empty buffer, so sharing is correct |
| `text_scratch` | `to_text` returns a borrow of it, which borrowck forbids holding across any `&mut Interp` call |
| `text_numbers` | a parse cache keyed by value |
| `env`, `library_search`, `cwd` | the process environment the oracle shares between threads |
| `next_activation_id`, `next_invocation` | identities unique across the interpreter |
| `programs`, `package_options`, `security_managers`, `plans`, `chunks`, `chunks_refused` | loaded programs and caches keyed by body |
| `deadline`, `clause_countdown` | brief; interpreter |
| `deferred` | REPLY's owed method bodies: a queue of work, not one execution's state |
| `routines`, `package_public_routines`, `merged_public_routines`, `merged_public_classes`, `rexx_class_cache`, `package_namespaces`, `package_locals`, `object_model`, `environment`, `class_variables`, `package_classes`, `package_public_classes`, `class_packages`, `empty_arguments`, `package_objects`, `program_routine_objects`, `package_tables`, `routine_objects`, `library_routine_objects`, `package_imports`, `package_parents`, `constant_values`, `annotations`, `compiled_methods`, `method_objects`, `method_bodies`, `compiled_method_names`, `table_method_bodies`, `executable_sources`, `generated_methods`, `native_externals`, `library_externals`, `external_packages`, `library_codes`, `library_code_rows`, `library_code_keys`, `library_routine_codes`, `rexx_routine_rows`, `rexx_routine_objects`, `defined_library_codes`, `package_routines`, `package_routine_codes`, `routine_generation`, `special_methods` | the package, class, method and routine model, shared by every activity |
| `library_bootstrap`, `collections_before_program`, `library_programs` | interpreter start-up |
| `pinning` (feature) | the arrival report, aggregated over activities |
| `object_methods`, `stem_exposers`, `method_flag_writes`, `message_outcomes` | keyed by heap objects |
| `global_references`, `kept_strings`, `kept_names`, `kept_holders`, `kept_carried`, `kept_carried_limit` | interpreter-wide handle table and copies; `kept_holders` counts calls across all activities |
| `libraries`, `command_handlers`, `library_open_attempts` | loaded native libraries |
| `out`, `trace`, `sinks`, `standard_transient` | the output streams |
| `stress_collect`, `uninit_ready`, `processing_uninits`, `collect_at` | the collector |
| `bootstrap_stderr`, `bootstrap_stdout`, `route_generation`, `store_generation`, `output_route` | route caches over `.local`, which is interpreter-wide |
| `queue`, `input` | the external data queue and `.input`'s position are per session |
| `command_words`, `program_path`, `required_paths`, `required_packages`, `untranslated` | loaded packages and the command line |
| `reqstr_armed`, `lostdigits_armed` | process-wide latches |

## Commits

- `0a4c5f46a` Move per-execution state from Interp into an Activity (`Box<Activity>`; round 1)
- `8f0437be3` Hold the Activity inline in Interp (round 2)
- `edacf562d` Record Task 4's instruction counts (`docs/superpowers/plans/phase-6-perf.md` `## Task 4`)

Shape: `rexx-exec/src/activity.rs` holds `pub(crate) struct Activity` with `pub(crate)` fields,
`Activity::new` and `Activity::object_roots` (its own exhaustive destructure; the root-extending
code for its fields moved with their comments). `Interp::object_roots` destructures `activity` and
calls `activity.object_roots(out)` first. Access is direct field access, `self.activity.X`
(524 rewritten sites plus 12 line-broken ones); disjoint borrows through it compile unchanged.
Doc paths `Interp::<moved field>` became `Activity::<moved field>`; no comment dropped or re-wrapped.
Pinning: `Pinning` keeps the report on `Interp`; the new `PinStack` (the frame stack) is
`Activity::pins`; `pinned!`/`pin_enter!`/`pin_leave!` use `$interp.activity.pins`, `park_point!`
passes it to `Pinning::park`, and `Outcome` takes the report with that stack's depth as
`unbalanced`.

## Performance

Base 1754a3b5a built fresh: `.text` sha256 `ad20a4d1a1cc2f1f...`, identical to Task 2's base
binary; the whole-file sha256 differs (`fed193bc...` vs recorded `e500b628...`), as the build path
differs. Its counts differ from Task 2's table by ~150 Ir per program; every delta below is against
this build.

Round 1 (`Box<Activity>`): +0.01% (startup) to +9.65% (nop), rexxcps +4.03%. Over budget.
Round 2 (`Activity` inline): every program within +/-131 Ir (largest +0.0002%, startup); rexxcps
-16,859, a decrease inside the base spread Task 2 recorded (20,659). Full table in phase-6-perf.md `## Task 4`.
Two rounds used.

Pinning off: `nm -C` of the r2 `rexx-run` finds 0 symbols matching
`pinning|PinStack|PinKind|ParkKind` (base 0; control: 1905 `rexx_exec::` symbols).

## Gates

At 8f0437be3 (`S=$S bash .../p6-gates/gates.sh`), `status.txt` verbatim:

```
8f0437be39fcea04f763a999d83c4361fe4ce42d
started 2026-09-29T23:24:37+02:00
load at start 1.90 7.74 10.11 1/1591 4029485
G1 fmt exit 0
G2 clippy(empty target) exit 0
G3 release build (test --no-run) exit 0
load G4 12.56 17.31 15.05 9/1644 4034565 2026-09-29T23:32:22+02:00
G4 release test exit 0
G4 Compiling lines: 0
load after G4 1.94 8.87 12.22 1/1612 4163699
G5 debug build (test --no-run) exit 0
load G6 4.29 8.97 12.18 3/1602 4168378 2026-09-29T23:38:41+02:00
G6 debug test exit 0
G6 Compiling lines: 0
load after G6 1.82 5.54 9.75 1/1602 102997
G7 clippy --features pinning exit 0
G8 pinning self-tests exit 0
8f0437be39fcea04f763a999d83c4361fe4ce42d
finished 2026-09-29T23:45:22+02:00
```

Test totals from the logs: G4 2783 passed 0 failed, G6 2784 passed 0 failed, G8 4 passed.
0a4c5f46a also ran green (G1-G8 exit 0, finished 2026-09-29T23:24:14+02:00).

## Concerns

1. **Inline, not `Box`.** The brief's interface names `Interp::activity: Box<Activity>`; boxed cost
   up to +9.65%. Inline costs nothing measurable. Ruling P3's "one indexed load" is met trivially
   (zero loads). Several activities will need either a box/arena slot again (and this cost back,
   unless the running activity's hot fields are reached another way) or a swap of the inline value.
   Needs a ruling before Task 5 builds on it.
2. Commit 8f0437be3's message says the fields "sit where they sat on Interp"; the layout is a
   nested struct, not the same offsets. The measurement is what holds.
3. Beyond the brief's list I moved: `locals`, `thread`, `flat_loops`, `fragment_depth`,
   `fragment_clause`, `clause_line_override`, `current_case_text`, the raise/failure fields,
   `native_reraise`, `outer_caller`, `input_dispatch*`, `max_depth`/`stack_*`, `elapsed_anchor`,
   `pending_elapsed_reset`, `routing_trace`, `requires_installing`. Reasons per row above. `thread`
   follows spec 2.4; it is an `Rc` clone at every use, so libraries holding it are unaffected now.
4. Root order handed to the collector changed (activity roots first). Mark order is not observable
   in a mark-sweep; G4/G6 include the collect-stress corpus and are green.
