#!/bin/bash
# The criterion 3 ThreadSanitizer run, from rust/. Usage:
#   bash tsan.sh TARGET_DIR LOG_DIR
# Clean means every command exits 0 and LOG_DIR holds no tsan.* file.
export CARGO_TARGET_DIR=$1
L=$2
mkdir -p "$L"
export RUSTFLAGS="-Zsanitizer=thread"
export TSAN_OPTIONS="log_path=$L/tsan suppressions=$PWD/tsan.supp allocator_may_return_null=1 second_deadlock_stack=1"
T="cargo +nightly test -Zbuild-std --target x86_64-unknown-linux-gnu"
$T -p rexx-api --lib > "$L/api.txt" 2>&1; echo "api exit $?"
# Tests that measure recursion against the native stack: TSan enlarges every
# frame, and its shadow call stack holds 65536 frames.
DEPTH="eval::tests::a_native_chain_past_the_eval_limit_runs
eval::tests::eval_raises_11_1_exactly_one_term_past_max_eval_depth
eval::tests::eval_survives_exactly_max_eval_depth_terms_and_prints_the_oracles_own_answer
ir::drive::tests::recursion_by_function_call_keeps_the_native_stack_flat
ir::drive::tests::recursion_by_new_into_init_keeps_the_native_stack_flat
ir::drive::tests::recursion_by_send_keeps_the_native_stack_flat
scheduler::tests::a_stack_within_the_margin_is_refused
scheduler::tests::nested_pinned_waits_are_bounded_by_the_stack_remaining
scheduler::tests::pool::callback_recursion_on_a_pool_thread_reaches_the_depth_cap
scheduler::tests::pool::deep_pinned_recursion_on_a_pool_thread_raises_11
scheduler::tests::pool::the_translator_on_a_pool_thread_has_the_interpreter_threads_stack
scheduler::tests::recursion_is_bounded_by_the_stack_remaining
tests::the_stack_span_does_not_depend_on_what_else_the_program_evaluated"
$T -p rexx-exec --lib -- --exact $(for t in $DEPTH; do echo --skip $t; done) > "$L/lib.txt" 2>&1
echo "lib exit $?"
$T -p rexx-exec --test signals --test stdin_contention --test program_end --test concurrency_tests \
  --no-fail-fast -- --exact --skip only_the_interpreter_and_its_waits_take_the_halting_signals \
  > "$L/int.txt" 2>&1
echo "int exit $?"
ls "$L" | grep '^tsan' || echo "no tsan log"
