# Phase 6.1 Task 11b report: deferred items, scheduling and finalization

Base `32d2f4925`. Head `595ba08ef` (code). Paths are under `rust/crates/rexx-exec/` unless they say
otherwise. Scratch: `/tmp/claude-1000/p61/t11b/`. Probes in `probes/`, runs from fresh `mktemp -d`
directories under `runs/`. Crate runs use `memcap 2G timeout -k 5 N`.

## Step 1: a full pool grows past its bound (`51782fcb8`)

* Design: "the pool grows", not a park. `Pool::reserve` (`src/scheduler/pool.rs`) spawns a thread
  beyond the bound when every thread is busy, and `work` ends a thread after its job while the pool
  holds more threads than its bound. The bound moved into the pool's shared state, so `set_bound`
  applies to threads already spawned. The command (`command.rs`), native (`exit_for_native`) and
  stdin (`input.rs`) sites are unchanged: `reserve` now answers `None` only for a bound of 0 or a
  failed spawn, and those still run inline. Reason for growth over a park: a park needs a
  worker-freed post and a retry of an idle activity's native exit with the baton lend from the
  inbox drain, while a thread beyond the bound needs neither; the oracle has a thread per activity.
* Simulation mode: it sets the bound to 0 (`sim.rs`), so it still runs every blocking call inline,
  bounded by `sim_block_bound`. No new scheduling event exists, so the trace and replay are
  unchanged. The seeded gate is green (Step 3).
* Tests (`src/scheduler/tests/pool.rs`, explicit switch points via `SwitchMode::AtClause(1_000_000)`):
  * `a_command_at_a_full_pool_runs_beyond_the_bound`: bound 2, two `sleep 2` nappers, a `timeout 5
    cat` on a fifo whose writer is a started activity. Expect `hi / read 0 / wrote 0` and at least one
    thread beyond the bound (test counter `threads_beyond_bound`).
  * `a_native_call_at_a_full_pool_runs_beyond_the_bound`: bound 1, NAP holds the thread, two
    `MEETATFULLPOOL` calls (patience 2 s). Expect meeting outcome (2, 0).
  * Mutation (growth disabled: the test-only `fixed` check made unconditional): the command test gave
    `read 124 / wrote 124`, the native test (0, 2). Both are red. Restored by exact edit.
  * The inline fallback stays covered. A test-only `set_pool_fixed` stops a pool at its bound, and
    `a_call_on_a_pool_thread_with_no_thread_free_runs_on_its_lend` (now asserting `beyond == 0`) and
    `callbacks::recalls_drained_together_are_each_served` run with it. Their docs say so.
* Scout probe `full.rex` (64 nappers, fifo path `/tmp/claude-1000/p61/t11b/ff`), crate at the Step 1
  tree: 5/5 `hi / read done wrote`, rc 0, stderr empty, about 3.1 s. Oracle, run with the scout's
  stated deviation (`LD_LIBRARY_PATH=... memcap 2G timeout -k 5 20 .../rexx full.rex`, no `ulimit -v`,
  since 64 oracle thread stacks do not fit in 1 GB): 5/5 the same, rc 0.
* `docs/superpowers/specs/2026-09-29-phase-6-concurrency-design.md` 2.7 said a release leaves ready
  activities to the next baton holder "when a spawn fails or the bound is reached". The second half is
  false now and has been rewritten.

## Rooting fix: every activation's own objects (`5251a8984`)

Found in Step 2 and approved by the controller.

* Defect: `Activity::object_roots` (`src/activity.rs`) rooted the context object, `replied`,
  `executable` and the trapped condition's object of each running and suspended activation. It never
  called `Activation::object_roots`, whose only caller was `spawn_continuation`. A stream a builtin
  opened by name, held only in its activation's table, reached no collection's roots. It survived only
  because `Stream` defines `UNINIT`: the collector resurrected it and readied its finalizer.
  `GC('force')` at base then closes a stream still in use.
* Root-set diff command, at base:
  `git show 32d2f4925:rust/crates/rexx-exec/src/activation.rs | sed -n '946,1041p'` against
  `git show 32d2f4925:rust/crates/rexx-exec/src/activity.rs | sed -n '612,652p'`.
  Fields `Activation::object_roots` roots that `Activity::object_roots` missed: `current_case`,
  `notify_message`, `cold.auto_expose` (owner, scope), `streams`, `call_arguments`, `method_identity`
  (scope, receiver), `exposed` (owner, scope). Both roots cover `context_object`, `replied`,
  `cold.condition` object and `cold.executable`.
* Fix: the reuse route. `Activity::object_roots` calls `Activation::object_roots` for each running
  and suspended activation, in place of the four per-field extracts. It roots a superset, and a
  repeated root costs one more pop of the mark loop's work stack. The collector depends on no root
  order: the mark loop dedupes, and `pending_uninit` follows the `uninit` registry's order.
* Other missed fields: under `REXX_SWITCH_MODE=sim:1,gc=1` (a collection at every allocation), these
  base-binary probes all answered correctly: a heap-sized `SELECT CASE` value compared across
  allocating `WHEN`s (`case.rex`), heap-sized routine and method arguments read back after
  allocations (`args.rex`), and an exposed variable read after allocations (`expose.rex`). Those
  fields are rooted on other routes as well, so no failing case exists for them. Only the streams
  rooting has a red test.
* Test: `tests::a_builtins_stream_survives_collections_while_its_table_holds_it`
  (`say linein(f) / call gc 'force' / say linein(f)` on a two-line file). At base it is red: rc 208,
  Error 48.1 "Stream not initialized". Fixed, it prints `one / two`. Witness `probes/streamroot.rex`,
  5 runs per engine: crate and oracle identical, `one / two`, rc 0, stderr empty.
* The collect_stress subset that went red in Step 2's first workspace run
  (`the_l0_subset_passes_again_under_collect_on_every_allocation`: stream_builtins,
  stream_table_sharing, security_manager; `a_stream_builtins_values_survive_a_pinned_yield`):
  `cargo test -p rexx-exec --test collect_stress` gives 37 passed, both with the drain and without it.

## Step 2: pending UNINITs run at activation return (`595ba08ef`)

* Change: `Interp::run_uninits_at_return` (`src/dispatch.rs`, `#[cold] #[inline(never)]`) roots the
  returning value, runs `run_ending_uninits` (refusals route as at an activity's end: the first is
  answered, the rest kept for the program's end), then releases the value.
  * A label or routine return calls it at the end of `finish_call` (`src/run/call.rs`), on `Ok` only
    (the oracle's check sits on the RETURNED path).
  * A method return calls it at the end of `finish_send`, after `finish_native_tails`. The first
    version drained inside `finish_call` for methods too, and the probe failed with 91.999
    `Message "NEW" did not return a result`: an `UNINIT`'s own send, run there, found NEW's native
    tail at its call-tail depth and finished NEW with the `UNINIT`'s result.
  * The common path is one `uninit_ready.is_empty()` test at each site. Re-entrancy is
    `processing_uninits`.
* `collect_now`'s comment had a false premise and now names where `runUninits` is reached:
  `GC('force')`, activation return (`RexxActivation.cpp:705`, `NativeActivation.cpp:1361`), the
  activity's dispatch loop (`Activity.cpp:249`) and termination.
* Byte accounting: a drain only clears the uninit flags (`clear_uninit_all`), and the next collection
  sweeps the bodies and releases their bytes through the running figure. The tests below collect
  after a drain in a debug build, so `Heap::collect`'s survivor-sum assertion runs there.
* Tests (`src/tests.rs`, debug assertions on): `pending_uninits_run_as_a_method_returns` (objects
  made by `.f~new`, so `INIT` returns through a send) and `pending_uninits_run_as_a_label_returns`
  (`.g~new` with no `INIT`, attribute setter, `call tick`). Each runs two rounds of 300 finalizable
  objects holding 300 KB strings. It asserts `1 1` (UNINITs ran before the first round ended, and
  more in the second, which needs a collection after a drain) and a peak of at most twice
  `COLLECT_BYTES_FLOOR`. Mutations: deleting the `finish_call` drain turns the label test red (`0 0`).
  Deleting the `finish_send` drain turns the method test red (152,715,185 body bytes at the peak).
  Both were restored from copies.
* Scout probes, head release binary, 3 runs each, `/usr/bin/time -f "%M KB"`:

| probe | crate stdout | crate peak RSS (KB) | oracle stdout | oracle peak RSS (KB) |
|---|---|---|---|---|
| uninit (1000) | `done 950` | 69 036, 69 592, 69 296 | `done 995` | 14 660, 16 472, 16 664 |
| uninit3k (3000) | `done 2966` | 69 448, 69 976, 69 608 | `done 2995` | 20 908, 20 236, 21 084 |
| nouninit (1000) | `done 0` | 52 972, 53 100, 53 368 | `done 0` | 14 408, 14 136, 14 132 |

  The crate's peak is 1.3 times its no-UNINIT control. Before the change: 355 MB and 1.16 GB. The
  count differs from the oracle's (collection timing, licensed).
* Corpus output order: `REXX_CORPUS_GATE=1 ... --test corpus --test ir_recorded_oracle` is green
  with no expectation changed. No corpus program's output changed, so none needed an oracle witness.
* `corpus/refusal-sites.tsv` is re-derived in the same commit. Step 1's `lib.rs` insertions moved the
  `lib.rs` rows' line numbers, so Step 1's commit leaves `refusal_sites` red until `595ba08ef`. The
  diff is line numbers only. Every changed row is in `lib.rs`, and each appears once removed and once
  added once the line number is stripped.

## Step 3: gates and performance

* `cargo fmt`. `cargo clippy --workspace --all-targets -- -D warnings`: 0.
  `cargo clippy -p rexx-exec --all-targets --features pinning,sharing -- -D warnings`: 0. Both ran
  on the Step 2 tree before commit.
* `tools/gates.sh` at `595ba08ef` (status file `gates-status.txt`, first line the sha, ends
  `finished`):
  * `memcap 8G cargo test -j 4 --workspace --no-fail-fast` (debug): 0, 3139 passed, 0 failed.
  * `REXX_CORPUS_GATE=1 ... --test corpus --test ir_recorded_oracle`: 0 (29 passed, 1 ignored; 21
    passed).
  * `REXX_CORPUS_GATE=1 ... --release --test concurrency_tests whole_groups`: 0, 16 passed, including
    `sim_gate::the_seeded_gate`. No `whole_groups` line or sim table needed a change.
* Callgrind (command and table in `docs/superpowers/plans/phase-6-1-gate.md` `## Task 11b`), against
  base61. The worst is rexxcps at +0.4268%, equal to base. The drain check costs rexxcps 12,608 Ir
  over base. The other programs are at most +0.0607% (alloc4c).
* Wall clock, 5 interleaved runs, against base61: rexxcps +0.15%, emptyloop +5.84% (base +4.67%),
  pingsem +1.61%, pingmsg +1.00%, the rest at most 0. emptyloop's Ir equals base, so its gap is
  Task 12's question.
* Queued item `2026-10-09-full-pool-inline-command-hang.md` has its `RESOLVED by` line.

## Concerns

* A failed thread spawn still runs a blocking call inline and can still hang, as before. The oracle
  raises Error 48.1 at `START` in that state.
* The pool has no upper limit under load. Each thread beyond the bound reserves `POOL_STACK_BYTES` of
  address space until its job ends.
* No drain at a native activation's return (`NativeActivation.cpp:1361`) beyond the send path through
  `finish_send`, and none inside `INTERPRET`. The probes reach the drain through `INIT`'s and a
  label's returns.

## Fix round 1

Brief `task-11b-fix1-brief.md`, review `task-11b-review.md`. Head `108bc81fa`.

### I1: drains at an INTERPRET's end and at a native method's return (`be020a36d`)

* `run/interpret.rs` `run_fragment`: at a fragment's normal end (`Flow::Next`, `Return`, `Exit`),
  the `RETURNED` block an `INTERPRET` activation leaves through (`RexxActivation.cpp:676-705`).
  `LEAVE`/`ITERATE` failures and `SIGNAL` out of the fragment do not drain.
* `dispatch.rs` `run_other`: at the return of an `External` (`LIBRARY REXX`, for example the
  `Stream` methods) or `Library` method, which are the oracle's `NativeMethod`s
  (`NativeActivation::run`, `NativeActivation.cpp:1361`). The drain runs on `Ok` only. A generated
  attribute method does not drain, which matches the reviewer's `attronly` control, and neither do
  primitive (`Invocable::Native`) methods or native routines (`sysslp` control).
* `run.rs` SAY: after the direct write that stands in for `.STDOUT`'s `LINEOUT` when `.OUTPUT` is not
  redirected. A routed SAY reaches the `run_other` site through the monitor's send.
* Each site is one `uninit_ready.is_empty()` test before the out-of-line `#[cold]`
  `run_uninits_at_return`. A trace line's direct write does not drain.
* Tests (`src/tests.rs`, `inline_uninit_rounds`: two inline rounds of 300 objects, with no label,
  routine or Rexx method returning in the loop): `pending_uninits_run_as_a_native_method_returns`
  (`call lineout '/dev/null', 'x'`), `pending_uninits_run_as_say_writes` (`say ''`) and
  `pending_uninits_run_as_an_interpret_ends` (`interpret 'nop'`). Each asserts `1 1` as stdout's last
  line and a peak of at most twice `COLLECT_BYTES_FLOOR`. Disabling each site (`if false && ...`)
  turns only its own test red. Restored from copies.
* The reviewer's probes (`/tmp/claude-1000/p61/t11br/probes/`), 1 run per engine, head binary
  `bin/fr1` (sha256 `f82d21a13b024925ef1d4eefc4c180b4c8e9d0edb692df75c987f576b1cb9333`):

| probe | crate | oracle |
|---|---|---|
| attronly (control) | `done 0`, 356,372 KB | `done 0`, 376,028 KB |
| rtncall | `done 951`, 69,284 KB | `done 999`, 16,180 KB |
| b_say | `done 951`, 69,444 KB | `done 998`, 16,328 KB |
| b_callline | `done 951`, 68,532 KB | `done 996`, 14,648 KB |
| b_xlineind | `done 951`, 69,248 KB | `done 998`, 18,272 KB |
| streamsend | `done 951`, 69,164 KB | `done 999`, 15,136 KB |
| interp | `done 951`, 68,720 KB | `done 998`, 24,336 KB |
| sysslp (control) | `done 0`, 355,928 KB | `done 0`, 370,284 KB |

All runs rc 0 with empty stderr. The remaining gap (951 against 995-999) is collection cadence, as
the review's `cadence.rex` shows.

### M1: the pool's shrink has a test (`bec5fe4a7`)

`scheduler::tests::pool::threads_beyond_the_bound_end_after_their_jobs`: a bound-1 pool runs three
held jobs (3 threads), and once they are released it is back at 1 thread within 10 s (test accessor
`Pool::threads`). With the shrink disabled (`if false && state.threads.len() > state.bound`), it is
red: left 3, right 1.

### M2

No change. Controller queues it.

### M3: `ActivationCold` destructured (`108bc81fa`)

`Activation::object_roots` destructures `ActivationCold` and `AutoExpose` exhaustively, naming
`active_condition`, `random_seed`, `locals` and `local` as holding no `ObjRef`. A dummy field added
to `ActivationCold` fails `cargo check` with E0027 at the pattern. It reads the same fields as before,
and only at a collection.

### Checks and perf

* `cargo fmt --all --check` 0. `clippy --workspace --all-targets -D warnings` 0. `clippy -p rexx-exec
  --all-targets --features pinning,sharing -D warnings` 0. `refusal-sites.tsv` re-derived with no
  change.
* `tools/gates-fr1.sh` at `108bc81fa` (status `gates-fr1-status.txt`, `finished`): debug workspace 0
  (3143 passed, 0 failed). Corpus pair 0. `whole_groups` 0, 16 passed, the seeded gate among them.
* Callgrind, `-r 2`, base61 against `bin/head` (`595ba08ef`) and `bin/fr1` (`be020a36d`), output
  `/tmp/claude-1000/p61/t11b/cg2`:

| program | t11b % | fr1 % |
|---|---:|---:|
| pingmsg | -0.0190 | -0.0190 |
| pingguard | -0.5906 | -0.5906 |
| pingsem | -0.4694 | -0.4694 |
| alloc | +0.0118 | +0.0118 |
| alloc4c | +0.0607 | +0.0607 |
| heapshape | +0.0520 | +0.0520 |
| rexxcps | +0.4268 | +0.4268 |
| emptyloop | -0.3191 | -0.3191 |

  rexxcps fr1 is 17,864,345,882 Ir against t11b's 17,864,348,575. M1 and M3 change no non-test
  code on these programs' paths, except `object_roots` at a collection, so they were not measured
  apart.
