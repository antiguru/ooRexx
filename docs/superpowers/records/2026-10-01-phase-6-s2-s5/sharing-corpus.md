# criterion 6 over the corpus, at 2a9bbbe12, from rust/:
# REXX_SHARING_LOG=LOG memcap 8G cargo test --release -p rexx-exec --features sharing --test corpus -- --exact sharing_fraction_over_the_corpus --nocapture
# The same command without --release (debug assertions on) writes a byte-identical table.

programs 787

| | objects | shared |
|---|---|---|
| bootstrap | 214064 | 500 |
| program | 306466 | 585 |

| program | bootstrap objects | bootstrap shared | program objects | program shared |
|---|---|---|---|---|
| lang/method_reply.rex | 272 | 9 | 9 | 0 |
| lang/method_reply_twice.rex | 272 | 9 | 8 | 0 |
| lang/method_reply_no_result.rex | 272 | 5 | 5 | 0 |
| lang/method_reply_exit_status.rex | 272 | 9 | 8 | 0 |
| lang/method_reply_chain.rex | 272 | 0 | 14 | 1 |
| lang/class_context_reply.rex | 272 | 4 | 199 | 1 |
| lang/forward_after_reply.rex | 272 | 9 | 18 | 1 |
| lang/object_start.rex | 272 | 0 | 59 | 5 |
| lang/object_start_multidimensional.rex | 272 | 0 | 16 | 2 |
| lang/directive_options_trace_reply.rex | 272 | 9 | 9 | 0 |
| lang/send_resumable_entries.rex | 272 | 0 | 97 | 5 |
| lang/started_waits_for_a_later_send.rex | 272 | 0 | 12 | 3 |
| lang/started_waits_inside_a_call.rex | 272 | 5 | 15 | 3 |
| lang/started_unwaited_says.rex | 272 | 5 | 10 | 2 |
| lang/started_result_message.rex | 272 | 0 | 10 | 2 |
| lang/started_raises.rex | 272 | 10 | 30 | 2 |
| lang/message_wait_then_completed.rex | 272 | 0 | 10 | 2 |
| lang/message_two_waiters.rex | 272 | 0 | 19 | 5 |
| lang/message_start.rex | 272 | 0 | 23 | 3 |
| lang/started_waited_in_a_replied_body.rex | 272 | 5 | 15 | 3 |
| lang/started_waited_in_uninit.rex | 272 | 0 | 15 | 2 |
| lang/context_thread_numbers.rex | 272 | 4 | 32 | 5 |
| lang/context_thread_sequential.rex | 272 | 4 | 19 | 3 |
| lang/context_thread_pool_bound.rex | 272 | 4 | 115 | 27 |
| lang/context_thread_lazy.rex | 272 | 4 | 35 | 6 |
| lang/message_start_error_condition.rex | 272 | 10 | 65 | 12 |
| lang/message_result_reraise.rex | 272 | 10 | 121 | 7 |
| lang/message_result_relayed.rex | 272 | 13 | 55 | 9 |
| lang/message_accessors.rex | 272 | 0 | 89 | 5 |
| lang/message_reuse.rex | 272 | 0 | 131 | 3 |
| lang/started_numeric_address.rex | 272 | 0 | 47 | 5 |
| lang/message_error_condition_after_main.rex | 272 | 5 | 46 | 7 |
| lang/message_send_user_condition.rex | 272 | 0 | 41 | 2 |
| lang/started_primitive_error_condition.rex | 272 | 10 | 70 | 12 |
| lang/message_primitive_send_fails_main.rex | 272 | 5 | 50 | 6 |
| lang/message_send_fails_main_frames.rex | 272 | 5 | 58 | 12 |
| lang/message_halt_wait.rex | 272 | 8 | 127 | 11 |
| lang/message_halt_self.rex | 272 | 13 | 67 | 7 |
| lang/message_halt_untrapped.rex | 272 | 11 | 35 | 5 |
| lang/message_halt_returning.rex | 272 | 14 | 92 | 13 |
| lang/message_halt_loop_step.rex | 272 | 2 | 200 | 16 |
| lang/message_halt_interpret.rex | 272 | 10 | 112 | 12 |
| lang/pinned_busy_wait_interpret.rex | 272 | 0 | 19 | 3 |
| lang/pinned_busy_wait_handler.rex | 272 | 6 | 36 | 3 |
| lang/pinned_busy_wait_external.rex | 272 | 0 | 12 | 2 |
| lang/message_halt_pinned_target.rex | 272 | 2 | 124 | 10 |
| lang/main_ends_in_pinned_yield.rex | 272 | 0 | 14 | 2 |
| lang/main_fails_in_pinned_yield.rex | 272 | 13 | 39 | 4 |
| lang/sleepers_wake_in_deadline_order.rex | 272 | 0 | 21 | 4 |
| lang/sleep_parks_in_if_do_and_argument.rex | 272 | 5 | 14 | 2 |
| lang/sleeper_wakes_busy_main.rex | 272 | 5 | 13 | 2 |
| lang/sleep_in_parse_template_and_when.rex | 272 | 0 | 43 | 6 |
| lang/reply_then_wait.rex | 272 | 4 | 22 | 7 |
| lang/reply_inside_constructs.rex | 272 | 5 | 54 | 6 |
| lang/reply_continuation_error_trace.rex | 272 | 13 | 61 | 7 |
| lang/reply_continuation_error_stderr.rex | 272 | 9 | 16 | 4 |
| lang/reply_continuation_failed_send.rex | 272 | 11 | 52 | 8 |
| lang/reply_split_after_a_trap_at_the_reply.rex | 272 | 9 | 44 | 4 |
| lang/reply_split_after_a_trap_takes_the_failure.rex | 272 | 9 | 38 | 0 |
| lang/reply_as_the_last_clause_numbers_the_replier.rex | 272 | 9 | 39 | 3 |
| lang/context_of_another_activity.rex | 272 | 4 | 110 | 4 |
| lang/context_moved_by_reply.rex | 272 | 0 | 31 | 3 |
| lang/message_reply.rex | 272 | 10 | 138 | 13 |
| lang/message_notify.rex | 272 | 0 | 42 | 14 |
| lang/message_notify_error.rex | 272 | 10 | 109 | 7 |
| lang/message_notify_single_slot.rex | 272 | 13 | 120 | 10 |
| lang/alarm_fires_after_reply.rex | 272 | 4 | 25 | 2 |
| lang/ticker_cancelled_by_its_target.rex | 272 | 0 | 19 | 3 |
| lang/timer_cancel_in_whole_days.rex | 272 | 0 | 39 | 4 |
| lang/timer_post_wakes_every_waiter.rex | 272 | 0 | 23 | 4 |
| lang/timer_post_reads_the_waiters_cancel.rex | 272 | 0 | 32 | 6 |
| lang/timer_remainder_modulo_2_32.rex | 272 | 0 | 22 | 3 |
| lang/trace_object_activity_fields.rex | 272 | 16 | 1057 | 56 |
| lang/uninit_after_every_activity.rex | 272 | 0 | 15 | 4 |
| lang/trace_object_native_caller.rex | 272 | 15 | 344 | 18 |
| lang/guard_serializes_started_sends.rex | 272 | 5 | 34 | 11 |
| lang/reply_seen_by_a_guarded_send.rex | 272 | 0 | 28 | 1 |
| lang/reply_continuation_parks_holding_the_guard.rex | 272 | 0 | 14 | 4 |
| lang/reply_at_nesting_two_reserves_again.rex | 272 | 0 | 12 | 1 |
| lang/guarded_getter_waits_for_reply.rex | 272 | 0 | 9 | 1 |
| lang/guard_on_waits_in_an_unguarded_method.rex | 272 | 5 | 16 | 6 |
| lang/guard_deadlock_two_activities.rex | 272 | 13 | 42 | 4 |
| lang/message_wait_deadlock.rex | 272 | 0 | 54 | 3 |
| lang/method_set_unguarded_is_not_reserved.rex | 272 | 5 | 20 | 3 |
| lang/method_set_guarded_is_reserved.rex | 272 | 5 | 20 | 3 |
| lang/guarded_empty_method_waits.rex | 272 | 5 | 12 | 2 |
| lang/guard_held_before_the_first_start.rex | 272 | 0 | 14 | 2 |
| lang/guard_on_lock_passes_between_activities.rex | 272 | 5 | 23 | 4 |
| lang/guard_when_waits_for_a_store.rex | 272 | 5 | 12 | 2 |
| lang/guard_when_stem_element_store.rex | 272 | 0 | 16 | 4 |
| lang/guard_when_attribute_setter.rex | 272 | 0 | 12 | 3 |
| lang/guard_when_drop_wakes.rex | 272 | 5 | 21 | 4 |
| lang/guard_when_traces_each_evaluation.rex | 272 | 9 | 11 | 2 |
| lang/guard_when_trace_object_waiting.rex | 272 | 13 | 184 | 9 |
| lang/guard_when_replies_take_turns.rex | 272 | 5 | 56 | 11 |
| lang/native_guard_on_when_updated.rex | 272 | 0 | 20 | 3 |
| lang/native_guard_off_keeps_the_count.rex | 272 | 5 | 23 | 5 |
| lang/guard_when_holds_the_lock_again.rex | 272 | 5 | 11 | 2 |
| lang/guard_when_unwatches_at_its_end.rex | 272 | 0 | 12 | 2 |
| lang/event_semaphore_post_wakes_every_waiter.rex | 272 | 0 | 19 | 5 |
| lang/event_semaphore_timed_wait_ended_by_a_post.rex | 272 | 0 | 12 | 3 |
| lang/mutex_semaphore_released_when_its_activity_ends.rex | 272 | 5 | 15 | 3 |
| lang/mutex_semaphore_end_hands_over_to_a_waiter.rex | 272 | 5 | 19 | 4 |
| lang/mutex_semaphore_stays_with_the_replier.rex | 272 | 0 | 17 | 1 |
| lang/mutex_semaphore_timed_acquire.rex | 272 | 5 | 21 | 4 |
| lang/sys_semaphores_across_activities.rex | 272 | 0 | 42 | 5 |
| lang/sys_semaphore_poll_takes_at_its_poll.rex | 272 | 0 | 16 | 2 |
| lang/sys_semaphore_post_waits_for_the_next_poll.rex | 272 | 0 | 12 | 2 |
| lang/alarm_cancel_waits_for_the_timer.rex | 272 | 0 | 24 | 2 |
| lang/alarm_message_target.rex | 272 | 4 | 26 | 4 |
| lang/ticker_fires_on_its_replied_activity.rex | 272 | 8 | 20 | 3 |
