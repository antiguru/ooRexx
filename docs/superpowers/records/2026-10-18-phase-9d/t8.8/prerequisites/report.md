# Fresh-worktree prerequisite enumeration

The default workspace test population at `87357a519cdac3cbfc79810dc58026d525c0c6d9` contains 99 tests blocked by absent checkout-local native paths. The run built the workspace once, executed every selected default test binary, and ran workspace doctests separately. It also found 60 missing-source failures, one filesystem precondition failure, and one unrelated stale-table failure. This is an enumeration from completed executions, not a source-search estimate.

| Result | Count |
| --- | ---: |
| Selected and completed test binaries | 154 |
| Binaries returning zero / nonzero | 136 / 18 |
| Binary tests passed / failed / ignored by existing attributes | 3603 / 161 / 5 |
| Native path precondition failures | 99 |
| Missing `ootest` source failures | 56 |
| Missing `oodocs` source failures | 4 |
| Cross-filesystem precondition failures | 1 |
| Unrelated stale-table failures | 1 |
| Doctests passed / failed | 15 / 0 |
| Compile errors / outer timeouts | 0 / 0 |

## Frozen source and execution scope

Before compilation, the worktree was on `codex/p9d-t88-prereq` at the expected revision, `git status --short --untracked-files=all` was empty, and `build/lib` did not exist. The dedicated target directory `/tmp/codex-p9d-t88-prereq-target` also did not exist. Final branch, revision, status, index diff, worktree diff, and path checks confirmed the same state, with `ootest` and `oodocs` absent too. Source, index, HEAD, submodules, checkout-local native builds, and the primary worktree were not changed.

Outer build and test commands ran under `memcap 8G`, with `CARGO_INCREMENTAL=0`, `CARGO_BUILD_JOBS=4`, and the dedicated target directory. Cargo used `--locked` and `-j4`, and each binary ran serially in its package directory with `--test-threads=1 --nocapture`. Original build, binary, and doctest children had 500-second foreground bounds, while listing and verification children had 30-second bounds. Existing interpreter and oracle wrappers ran unchanged, and no oracle libraries were supplied as substitutes for checkout-local test artifacts.

Selection used Cargo JSON artifacts whose profile is a test profile and whose executable is present. All selected artifact targets have `target.test=true`, consisting of 126 integration targets, 17 binary unit harnesses, and 11 library unit harnesses, with no examples selected. All `--list` invocations succeeded, and the manifest `harness=false` settings apply only to benchmark targets outside this selection. Every selected artifact has an empty feature list, and the test profile uses optimization level 3, debug assertions, and overflow checks.

```text
memcap 8G python3 /tmp/codex-p9d-t88-prereq-driver.py build
  cargo test --workspace --no-run --locked -j4 --message-format=json
memcap 8G python3 /tmp/codex-p9d-t88-prereq-driver.py run
  <each selected executable> --list --format terse
  <each selected executable> --test-threads=1 --nocapture
memcap 8G python3 /tmp/codex-p9d-t88-prereq-driver.py doc
  cargo test --workspace --doc --locked -j4 -- --test-threads=1
```

The cold build completed successfully in 310.271 seconds, and binary execution took 1012.565 seconds in total. The longest binary was `collect_stress`, which passed 79 tests in 313.185 seconds, while the default corpus target passed in 151.246 seconds. All 154 binaries produced completed summaries, and their listed populations reconcile exactly with passed, failed, and ignored counts. The installed toolchain was rustc 1.99.0, recorded with full provenance in `rustc-version.stdout`.

No `REXX_*` variables were inherited or enabled by the driver, including corpus, ooTest, calibration, oracle-refresh, or docs-less modes. Existing tests that return early when a gate is disabled retained that behavior, so these results cover default behavior rather than full gated populations. The existing `release_local_reference` fixture test compiled its own C++ fixtures under the owned target temporary directory and passed, while the gated `outer_context` fixture construction stayed disabled. Doctests completed with four executed examples, ten compile-fail examples, and one compile-only example, with no omitted doctest target.

The existing default ignored tests were:

* `target-018-format::format_after_survives_the_full_u32_range`
* `target-018-format::format_before_survives_the_full_u32_range`
* `target-018-format::trunc_accepts_places_at_and_past_the_i32_negation_boundary`
* `target-110-corpus::probe_emit_uncaptured_marker`
* `target-116-suite::fixtures::tests::sysfiletree_group_retains_assertion_and_metadata_context`

## Missing checkout-local native paths

The 99 failures below divide into 30 loader failures at explicit shared-library paths and 69 failures caused by the absent `build/lib` directory. The explicit loader paths are `build/lib/librxregexp.so` and `build/lib/librxmath.so`, whose missing loads produce the misleading `the extension publishes RexxGetPackage` panic. The directory failures happen before library loading, so the measurement does not establish that each affected test independently needs an on-disk shared object. In particular, `scheduler::tests::pool::two_native_calls_waiting_for_each_other_meet_on_pool_threads` uses a synthetic in-memory library but still reaches the shared directory canonicalization.

| Exact missing precondition | Failed tests | Evidence |
| --- | ---: | --- |
| `build/lib/librxregexp.so` | 24 | `target-016-context`, `target-063-load`, `target-066-invoke` stdout/stderr |
| `build/lib/librxmath.so` | 6 | `target-066-invoke` stdout/stderr |
| `build/lib` directory | 69 | `target-105-rexx_exec`, `target-122-sim_processes` stdout/stderr |

The unit target accounts for 68 directory-precondition failures, and `sim_processes` accounts for the remaining one. One callback test reports `Disconnected` because its unnamed child first panics at `scheduler/tests/pool.rs:239` on the absent directory. A separate exact-test recheck reproduced that child panic and channel failure immediately, without a timeout. `callback-precondition-recheck.stdout` and `.stderr` retain this evidence rather than classifying the channel error as an unrelated failure.

### build/lib/librxregexp.so

`target-016-context`:

* `a_collection_inside_the_call_leaves_the_cself_intact`
* `cself_survives_a_collection_between_two_calls`
* `drop_object_variable_unbinds_the_name`
* `new_pointer_and_set_object_variable_store_the_cself_block`
* `raise_exception0_records_the_condition_and_the_call_finishes`
* `string_data_and_string_length_hand_over_the_subject`
* `string_data_answers_one_address_for_one_object`
* `the_cself_object_is_swept_when_the_receiver_is_not_a_root`
* `the_write_reaches_the_methods_scope_and_no_other`
* `whole_number_to_object_carries_the_position_the_extension_computed`

`target-063-load`:

* `a_method_is_found_without_regard_to_case`
* `a_search_directory_resolves_a_name_the_undecorated_attempt_misses`
* `the_extension_declares_no_routines`
* `the_method_table_is_what_the_extension_declares`
* `the_oracles_extension_loads_and_names_itself`

`target-066-invoke`:

* `a_call_that_raised_still_returns_what_it_wrote`
* `a_missing_required_argument_stops_the_call`
* `a_native_call_that_raises_keeps_its_condition_across_the_halves`
* `a_second_method_reads_the_cself_the_first_produced`
* `an_argument_the_signature_does_not_consume_is_refused`
* `an_extension_that_raises_still_finishes`
* `regexp_init_with_no_arguments_answers_zero_and_allocates_an_automaton`
* `regexp_init_with_one_argument_parses_it`
* `the_extension_publishes_the_signatures_its_source_declares`

### build/lib/librxmath.so

`target-066-invoke`:

* `a_handle_built_during_the_call_resolves_after_collections_between_the_halves`
* `every_rxmath_routine_is_typed`
* `rxcalcsqrt_formats_at_the_callers_digits_when_no_precision_is_given`
* `rxcalcsqrt_reads_a_precision_argument_that_exists`
* `rxcalcsqrt_refuses_a_missing_and_an_extra_argument_before_running`
* `the_records_between_the_halves_cross_threads`

### build/lib (directory)

`target-105-rexx_exec`:

* `dispatch::library::tests::a_held_library_is_not_replaced_by_a_later_answer`
* `dispatch::library::tests::a_library_call_answers_the_same_under_a_collection_at_every_allocation`
* `dispatch::library::tests::a_library_named_twice_is_opened_once`
* `dispatch::library::tests::a_library_routine_answers_the_same_under_a_collection_at_every_allocation`
* `dispatch::library::tests::a_loaded_library_outlives_the_finaliser_sweep`
* `dispatch::library::tests::a_search_directory_written_after_the_start_is_not_searched`
* `dispatch::library::tests::a_version_refused_library_raises_once_and_is_held`
* `dispatch::library::tests::the_conversion_rows_answer_the_same_under_a_collection_at_every_allocation`
* `dispatch::library::tests::the_native_finaliser_answers_the_same_under_a_collection_at_every_allocation`
* `dispatch::library::tests::the_search_path_is_what_makes_the_name_resolve`
* `dispatch::library::tests::unloaders_run_in_the_oracles_table_order`
* `run::tests::routines::a_library_backed_routine_installs_and_runs_when_it_is_called`
* `run::tests::routines::a_routine_a_required_library_exports_runs_rather_than_answering_43_1`
* `run::tests::routines::an_extension_reading_the_instance_reaches_its_table`
* `run::tests::routines::an_extensions_raise_with_substitutions_is_the_calls_condition`
* `scheduler::tests::an_ended_activitys_thread_context_is_kept`
* `scheduler::tests::callbacks::a_callback_from_an_abandoned_call_answers_nothing`
* `scheduler::tests::callbacks::a_callback_from_another_thread_is_refused_in_the_simulation_mode`
* `scheduler::tests::callbacks::a_callback_from_another_thread_takes_the_baton_and_raises_98_983`
* `scheduler::tests::callbacks::a_callback_from_the_calls_own_thread_runs`
* `scheduler::tests::callbacks::a_callback_is_served_at_a_pinned_holders_next_cold_visit`
* `scheduler::tests::callbacks::a_callback_waits_for_a_contended_guard_on_its_own_thread`
* `scheduler::tests::callbacks::a_callback_while_the_holder_is_pinned_is_answered`
* `scheduler::tests::callbacks::a_foreign_callback_while_the_calls_thread_is_in_a_callback_raises_98_983`
* `scheduler::tests::callbacks::a_kept_call_context_used_from_another_threads_call_raises_98_983_there`
* `scheduler::tests::callbacks::a_kept_context_used_from_another_threads_call_raises_98_983_there`
* `scheduler::tests::callbacks::a_variable_set_from_an_abandoned_call_sets_nothing`
* `scheduler::tests::callbacks::an_abandoned_calls_completion_does_not_complete_the_next_call`
* `scheduler::tests::callbacks::recalls_drained_together_are_each_served`
* `scheduler::tests::callbacks::two_activities_in_native_calls_with_interleaved_callbacks`
* `scheduler::tests::lent::a_buffer_changed_in_place_under_a_call_stays_shared`
* `scheduler::tests::lent::a_buffer_grown_under_a_call_keeps_the_storage_the_call_holds`
* `scheduler::tests::lent::a_held_handle_carried_string_survives_another_activitys_prune`
* `scheduler::tests::lent::a_held_heap_string_survives_another_activitys_collection`
* `scheduler::tests::lent::an_abandoned_calls_frame_ends_with_its_completion`
* `scheduler::tests::lent::finishing_a_buffer_string_keeps_its_kept_string_in_place`
* `scheduler::tests::native::a_callback_during_an_off_baton_call_takes_the_baton`
* `scheduler::tests::native::a_callbacks_wait_on_a_call_below_it_is_refused`
* `scheduler::tests::native::a_failure_ending_a_native_park_abandons_the_call`
* `scheduler::tests::native::a_function_whose_call_left_its_driver_and_answered_nothing_is_44_1`
* `scheduler::tests::native::a_guarded_library_method_runs_after_its_guard_wait`
* `scheduler::tests::native::a_library_routine_called_again_reaches_the_row_its_name_finds`
* `scheduler::tests::native::a_lone_activitys_native_call_keeps_its_driver`
* `scheduler::tests::native::a_native_call_in_the_simulation_mode_spawns_no_pool_thread`
* `scheduler::tests::native::a_native_call_only_another_activity_can_end_is_refused_after_its_bound`
* `scheduler::tests::native::a_native_call_where_another_activity_lives_leaves_its_driver`
* `scheduler::tests::native::a_pinned_wait_is_satisfied_by_an_activity_returning_from_a_native_call`
* `scheduler::tests::native::collect_stress_across_a_native_calls_driver_exit`
* `scheduler::tests::native::completed_native_calls_repeat_independently_of_host_descheduling`
* `scheduler::tests::native::only_a_timed_out_native_watch_refuses_a_completed_call`
* `scheduler::tests::notifier_failures_too_many_to_nest_are_each_reported`
* `scheduler::tests::pool::a_call_on_a_pool_thread_with_no_thread_free_runs_on_its_lend`
* `scheduler::tests::pool::a_command_at_a_full_pool_runs_beyond_the_bound`
* `scheduler::tests::pool::a_lone_call_keeps_the_baton_after_its_callback_starts_an_activity`
* `scheduler::tests::pool::a_long_command_does_not_stop_another_activitys_output`
* `scheduler::tests::pool::a_native_call_at_a_full_pool_runs_beyond_the_bound`
* `scheduler::tests::pool::a_native_panic_on_a_pool_thread_unwinds_under_a_lend`
* `scheduler::tests::pool::a_panic_on_a_pool_thread_reaches_the_interpreter_thread`
* `scheduler::tests::pool::a_panic_under_a_lend_reaches_the_lender_before_it_runs_on`
* `scheduler::tests::pool::a_pinned_wait_blocks_on_the_inbox_while_a_pool_thread_runs_the_call`
* `scheduler::tests::pool::a_redirected_command_does_not_stop_another_activity`
* `scheduler::tests::pool::a_spawn_failure_leaves_ready_activities_to_the_next_baton_holder`
* `scheduler::tests::pool::an_off_baton_commands_output_keeps_its_place`
* `scheduler::tests::pool::bounded_callback_recursion_nests_under_one_lend`
* `scheduler::tests::pool::callback_recursion_on_a_pool_thread_reaches_the_depth_cap`
* `scheduler::tests::pool::deep_pinned_recursion_on_a_pool_thread_raises_11`
* `scheduler::tests::pool::the_translator_on_a_pool_thread_has_the_interpreter_threads_stack`
* `scheduler::tests::pool::two_native_calls_waiting_for_each_other_meet_on_pool_threads`

`target-122-sim_processes`:

* `a_seed_gives_one_trace_hash_in_two_processes`

## Additional missing source prerequisites

The ignored `ootest` checkout is absent, and 56 default tests fail before they can inspect their intended groups or framework copies. The missing paths include `ooRexx/base/{expressions,keyword,bif}`, `ooRexx/API`, `ooTest.frm`, and `framework/OOREXXUNIT.CLS`, with exact raw errors retained per target. The ignored `oodocs` checkout is also absent, and four extractor tests fail with an actionable SVN checkout message. No checkout was initialized, no docs-less flag was set, and failures hidden behind these missing inputs were not inferred as completed assertions.

### Missing ootest source

`target-003-extract_assertions`:

* `base_expressions_expect_syntax_conversion_counts`
* `base_expressions_yields_the_measured_row_and_blocked_counts`
* `every_assert_same_in_base_expressions_is_a_row_or_an_accounted_for_drop`

`target-004-extract_keyword`:

* `base_keyword_yields_the_measured_counts`
* `every_assert_same_is_a_row_or_an_accounted_for_drop`
* `the_committed_group_list_matches_the_checkout`
* `the_drop_reasons_account_for_every_call_outside_the_population`
* `the_population_choices_price_is_reproduced_by_an_independent_rule`
* `the_row_floor`

`target-009-extract_bif`:

* `base_bif_yields_the_measured_counts`
* `every_call_is_a_row_or_a_counted_drop`
* `the_drop_reasons_account_for_every_call_outside_the_population`
* `the_row_floor`

`target-082-bif_assertions`:

* `bif_assertions_differential`
* `the_exempt_set_matches_the_current_failures`
* `the_falsification_proof`

`target-092-api_group_partition`:

* `a_group_whose_library_changes_changes_phase`
* `each_api_group_belongs_to_the_phase_its_libraries_import_from`

`target-098-keyword_assertions`:

* `keyword_assertions_differential`
* `the_exempt_set_matches_the_current_failures`
* `the_falsification_proof`

`target-112-assertions`:

* `assertions_differential`
* `digits_and_form_are_carried_not_defaulted`
* `the_exempt_set_matches_the_current_blocked_rows`
* `the_falsification_proof`
* `the_raise_falsification_proof`

`target-116-suite`:

* `fixtures::tests::a_child_refusing_on_the_inherited_stderr_leaves_the_run_ended_normally`
* `fixtures::tests::a_command_a_test_leaves_running_is_killed_with_the_run`
* `fixtures::tests::a_command_the_crate_side_runs_finds_its_rexx_and_rexxc`
* `fixtures::tests::a_refusal_at_load_is_blocked_at_load_after_one_run`
* `fixtures::tests::a_refusal_ends_the_run_and_the_next_run_goes_past_it`
* `fixtures::tests::a_run_that_starts_no_test_after_an_earlier_one_did_is_not_blocked_at_load`
* `fixtures::tests::a_test_that_hangs_is_renamed_out_and_the_next_run_proceeds`
* `fixtures::tests::discovery_finds_groups_at_every_depth_in_path_order`
* `fixtures::tests::the_crate_side_bin_holds_the_oracle_bins_class_files_and_no_other_binary`
* `fixtures::tests::the_expected_set_follows_test_classes_through_the_file_as_the_framework_does`
* `fixtures::tests::the_expected_set_is_what_the_framework_starts_whatever_the_file_comments_or_quotes`
* `fixtures::tests::the_marker_file_names_the_test_a_kill_at_the_bound_ended_in`
* `fixtures::tests::the_relative_path_tests_pass_on_the_oracle_with_their_neighbours_copied`
* `fixtures::tests::the_run_cap_stops_a_group_that_hangs_on_every_test`
* `fixtures::tests::the_suite_revision_is_the_recorded_one`
* `fixtures::tests::without_memcap_the_crate_run_is_under_the_ulimit_fallback`
* `gate_groups_9b::the_9b_group_file_is_what_its_derivation_gives`
* `row_checks::each_owner_check_passes_its_rows_and_fails_the_others`
* `row_checks::the_rxapi_rows_are_reaching_rxapis_output`
* `the_port_binding_groups_run_alone_and_the_rest_together`
* `the_row_files_hold_only_checked_rows`

`target-145-api_group_tests`:

* `the_tests_reaching_rxapi_run_on_neither_side`

`target-152-ir_recorded`:

* `every_population_runs_without_a_refusal`
* `every_population_the_tree_calls_for_is_present_and_non_empty`

`target-153-concurrency_tests`:

* `byte_exact_copies::mark_starts_keeps_a_byte_that_is_not_utf_8`
* `byte_exact_copies::skip_changes_only_the_renamed_methods_line`
* `byte_exact_copies::without_rxfuncquery_keeps_a_byte_that_is_not_utf_8`
* `group_runs::whole_groups::sim_gate::the_table_holds_the_parts_the_spec_names`
* `the_derived_list_holds_the_groups_the_spec_names`
* `the_derived_list_is_the_committed_one`

### Missing oodocs source

`target-006-extract_docs`:

* `every_method_rows_origin_line_names_its_section`
* `every_row_set_is_exactly_what_its_extractor_derives_today`
* `every_row_set_is_stamped_with_the_revision_the_checkout_is_at`
* `skipping_the_comment_blanking_fires_the_argutil_assertion`

## Separate failures

The filesystem fixture requires its worktree temporary directory and the process temporary directory to be on different filesystems. Both directories have device ID 44 here, and the test observed a successful rename where it explicitly requires `EXDEV`. The stale refusal-site table disagrees only with committed source locations for the reported simulation refusal rows, as detailed in its raw failure output. Neither failure was repaired, retried into a pass, or counted as a missing-native-artifact failure.

* `target-105-rexx_exec::builtin::rexxutil::files::tests::a_move_across_file_systems_copies_and_unlinks` at `crates/rexx-exec/src/builtin/rexxutil/files/tests.rs:66:5`.
* `target-134-refusal_sites::the_table_holds_every_constructor_the_source_defines` at `crates/rexx-exec/tests/refusal_sites.rs:522:5`.

## Smallest common precondition proposal

Use one documented test-entry preflight that validates checkout-local native prerequisites and exits once with their paths and the provisioning command before workspace tests start. Reuse the existing CMake targets and existing loader checks, retaining the separate oracle location and adding no Rust dependency. The shared scheduler helper should stop imposing a directory-existence requirement on synthetic-library fixtures that do not load checkout-local libraries. The gate policy in `rust/CLAUDE.md` should require this preflight without converting missing prerequisites into passing or skipped tests.

Source inspection after the executions identifies `rxregexp`, `rxmath`, `orxmethod`, `orxfunction`, and `rxsock` as the checkout-local library targets reached through the affected helpers. Their existing definitions are in root `CMakeLists.txt:1296`, `:1318`, `:1336`, and `testbinaries/CMakeLists.txt:64-65`. The candidate command below follows those definitions, but was not run because provisioning the missing artifacts would erase the measured fresh-checkout condition. A later implementation owner must verify that provisioning command and its actual native build prerequisites before presenting it as a proven setup recipe.

```sh
memcap 8G cmake -S . -B build -DCMAKE_POLICY_VERSION_MINIMUM=3.5 -DCMAKE_BUILD_TYPE=Release
memcap 8G cmake --build build --target rxregexp rxmath orxmethod orxfunction rxsock -j4
```

## Evidence and limits

`preflight.json` and `final-*` files record the source and path checks, while `artifacts.json` preserves Cargo selection and feature/profile metadata. `targets.json`, `tests.json`, `failures.json`, `doctest-results.json`, and `summary.json` provide machine-readable results, and each command has separate `.stdout`, `.stderr`, and `.json` files. The checked-in sources stayed frozen, and every foreground session completed, leaving no live command or enumeration blocker. These results establish the failing default population in this fresh worktree on this machine, while native leaf behavior beyond failed directory checks, missing-source downstream behavior, gated populations, other features, and other platforms remain outside the measured claim.
