# Phase 6.1 Task 11b review

Range `32d2f4925..5ac42f73a`, commit `f807ca33e` skipped. HEAD was built from
`git archive 5ac42f73a rust interpreter` into `/tmp/claude-1000/p61/t11br/src` (tree touched first),
with its own target `/tmp/claude-1000/p61/t11br/target`. Command:
`CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx-run`. The log shows
one `Compiling rexx-exec` line. Crate runs used `memcap 2G timeout -k 5 N rexx-run`. Oracle runs used the
standard wrapper (`ulimit -v 1048576`) from a fresh `mktemp -d` directory, except where a run says
otherwise.

## Verdicts

* **Spec compliance: partial.** Step 1, Step 3 and the rooting ruling are met. Step 2 meets its own
  probe, but not the rule it carries out: the rule names `NativeActivation.cpp:1361`, and the oracle
  also drains at an `INTERPRET`'s return (Important 1).
* **Quality: approve with one Important fix.** No Critical findings. The rooting fix is complete for
  every `ObjRef` an `Activation` holds. Each new test goes red under its mutation, and the perf figures
  reproduce exactly.

Counts: Critical 0, Important 1, Minor 3.

## Important

### I1. No drain at an `INTERPRET`'s return or at a native method's return

`run/call.rs:1111` (`finish_call`, labels and routines) and `dispatch.rs:2918` (`finish_send`, methods
with a Rexx body) are the only drain sites. The oracle has two more sites, and on both of them the crate
holds every finalizable object until termination:

* `RexxActivation.cpp:676-705`: an `INTERPRET` activation leaves through the same `RETURNED` block that
  calls `checkUninitQueue`.
* `NativeActivation.cpp:1361`: this is in `NativeActivation::run`, which runs native methods. The
  Stream methods are native methods, so `SAY`, `LINEOUT`, `LINEIN` and `CHAROUT` all reach it.

So the rule "a loop creating finalizable objects runs in bounded memory" holds only when a Rexx label,
routine or method returns inside the loop. The implementer disclosed this gap but gave no measurement.

Evidence: one shape for every probe, with a different last statement in the loop body.

```
.local~n = 0
do i = 1 to 1000
  o = .g~new
  o~s = copies('x', 300000) || i
  <stmt>
end
say 'done' .local~n
::class g
::attribute s
::method uninit
  .local~n = .local~n + 1
```

`/tmp/claude-1000/p61/t11br/run1.sh {crate|oracle} PROBE 60` collects stdout, rc and
`/usr/bin/time -f %M`:

| `<stmt>` | crate | oracle |
|---|---|---|
| none (`attronly.rex`) | `done 0`, 355,860 KB | `done 0`, 375,876 KB |
| `call r` (a `::routine`) | `done 951`, 69,056 KB | `done 999`, 14,420 KB |
| `say ''` | `done 0`, 356,276 KB | `done 998`, 16,668 KB |
| `call lineout '/dev/null', 'x'` | `done 0`, 355,916 KB | `done 996`, 14,064 KB |
| `x = linein('/dev/null')` | `done 0`, 355,860 KB | `done 998`, 18,504 KB |
| `.stdout~charout('')` | `done 0`, 356,344 KB | `done 999`, 14,968 KB |
| `interpret` of the two body lines | `done 0`, 356,600 KB | `done 998`, 24,148 KB |
| `call SysSleep 0` (a native routine, which does not drain) | `done 0`, 356,108 KB | `done 0`, 370,272 KB |

The first and last rows are controls, and the two engines agree on both. A loop that writes output
while it creates finalizable objects is ordinary code. In such a loop the crate's memory grows with
the iteration count, while the oracle's stays flat.

Fix:
* Add the same cold `uninit_ready.is_empty()` check at an `INTERPRET` fragment's normal end.
* Add it after a Stream method or stream builtin returns, which is where the crate does the work of
  the oracle's `NativeMethod`.
* For each site, add a test in the style of `uninit_rounds`, using `say ''` and `interpret` as the
  loop's only return, and confirm each test goes red when its site is removed.
* Measure rexxcps afterwards. It has about 0.07% of margin left.

If Moritz chooses to defer the fix, record the two sites in the Task 12 queue with the table above.

## Minor

### M1. The pool's shrink path has no test

At `scheduler/pool.rs:262`, a thread finishing a job ends if the pool holds more threads than its
bound. No test detects this path. The mutant `if false && state.threads.len() > state.bound {`, which
keeps every extra thread as idle forever, leaves the scheduler tests green:
`memcap 8G $T scheduler::` gives `test result: ok. 123 passed; 0 failed`. Fix: add a test-only exit
counter, or read back `threads.len()` after a burst beyond the bound, and assert that the pool is back
at its bound.

### M2. A failed spawn still runs inline and hangs, and the reservations make that easy to reach

This was disclosed, and it predates the task. Pool threads reserve `POOL_STACK_BYTES`, which equals
`INTERPRETER_STACK_BYTES` (512 MiB). Spec 2.7 says pool stacks should be smaller.

* With growth, `cmd1000.rex` runs 1000 started activities, each `address system 'sleep 1'`. It ends
  rc 0 in 1.45 s, with 77,120 KB peak RSS, 2003 threads and 544,662,028 KB VSZ. That is fine under the
  heuristic overcommit this machine uses.
* With the address space capped, the scout's 64-napper fifo probe hangs. The command was
  `( ulimit -v 3145728; memcap 2G timeout -k 2 12 rexx-run full.rex )`. It gave rc 137 with empty
  stdout, and `timeout`'s TERM did not end it. Without the cap, the same probe gives
  `hi / read done wrote`, rc 0, in 5 of 5 runs. The oracle raises Error 48.1 at `START` in that state.

Fix: queue for later. On a failed spawn, park or raise rather than running inline, and size pool stacks
as spec 2.7 states.

### M3. `Activation::object_roots` reaches `ActivationCold` by field

`activation.rs:947-1050` destructures `Activation` exhaustively, but reads `cold.condition`,
`cold.auto_expose` and `cold.executable` by field. If an `ObjRef` is added to `ActivationCold`, it
would go unrooted with no warning. Today `active_condition` is the only cold field that is not rooted.
It holds a `Raised` (`error.rs:73`), and that type holds no `ObjRef`. This predates the task, but since
the rooting fix it carries every running and suspended activation's root set. Fix: destructure
`ActivationCold` exhaustively, as the method already does for `TrappedCondition`.

## What was checked and held

### (a) Rooting

I did the field diff by hand. The old `Activity::object_roots` rooted `context_object`, `replied`,
`cold.executable` and `cold.condition.object`. `Activation::object_roots` adds `current_case`,
`notify_message`, `cold.auto_expose` (owner and scope), `streams`, `call_arguments`, `method_identity`
(scope and receiver) and `exposed` (owner and scope). This matches the report.

The fields `Activation::object_roots` leaves out hold no `ObjRef`:
* `io_configs`, `traps` and `debug`, as their own comments say.
* `cold.active_condition`: `Raised` and `FailureSite` contain no `ObjRef` (`grep -n ObjRef error.rs`).
* `frame` is rooted through the slot stack.

Idle activities and REPLY continuations reach the same code through `Idle::object_roots` and then
`Activity::object_roots`.

I hunted for missed roots under `REXX_SWITCH_MODE=sim:S,gc=1` with seeds 1 to 5, which collects at
every allocation. Every finalizable object printed `U name` from its `UNINIT`. In all 5 seeds, no
`U name` line came before that object's last use. The two probes:
* `probes/roots.rex`: an argument used after the callee's own calls return; a `SELECT CASE` value; a
  REPLY'd method's local used after the reply; an object held only by a `CALL ON` condition's
  `ADDITIONAL`; a routine's return value; a `DO OVER` snapshot; a receiver held only by its send; a
  `~start` argument.
* `probes/temps.rex`: argument-list temporaries across function calls; a receiver with pending
  arguments; `.array~of` temporaries; a stem tail; a variable inside `INTERPRET`.

Without `sim`, stdout matches the oracle line for line on both probes. Only the order of the `U` lines
at termination differs, which is licensed GC ordering.

In `finish_call`, nothing allocates between `pop_activation` and the drain. During the drain,
`run_uninits_at_return` parks the returning value.

### (b) Drain semantics

* Which activity runs the drain: the returning one, as `runUninits` uses
  `ActivityManager::currentActivity`.
* Re-entrancy: `processing_uninits` blocks it.
* Errors: a SYNTAX error or an `EXIT` inside a drained `UNINIT` is swallowed silently on both engines.
  For `probes/uerr.rex` and `probes/uexit.rex`, both engines print `done 1`, rc 0, with empty stderr.
  This matches `UninitDispatcher::handleError`.

`done 950` against `done 995` comes from collection cadence alone. `probes/cadence.rex` prints the
iteration at which the count changes:
* The crate collects every 56 objects (`56:54 112:110 ... 952:950`). That is the 32 MiB
  `COLLECT_BYTES_FLOOR` over about 600 KB of garbage per iteration. The last 48 objects are allocated
  after the last collection and are only finalized at termination.
* The oracle collects every 3 or 4 iterations (`7:5 10:8 14:12 ...`).

Each crate step lands at the same iteration as its collection, so the drain itself is prompt. This is
licensed GC timing. Peak RSS is 69,008 KB against the no-UNINIT control's 52-53 MB.

Corpus programs with `UNINIT`: `cmp5.sh` ran 5 runs per engine, comparing stdout, stderr and rc
separately.
* It covered every `corpus/lang/*.rex` that mentions `uninit` (`grep -rli uninit --include=*.rex lang`) and the four
  `gate-tables/` files.
* The `library_*` files ran with `LD_LIBRARY_PATH` set to the oracle's `build/lib` on both sides.
* All of them give the same single signature on both engines, 5 of 5 runs, so no output-order change
  exists to witness.

### (c) Pool growth

Simulation mode sets the bound to 0 (`sim.rs:956`). `reserve` answers `None` before it considers
growth, so no scheduling event is new, and the seeded gate cannot see growth. Growth does not touch
pinning.

The scout's 64-napper probe ends `hi / read done wrote` with rc 0 in 5 of 5 crate runs. The oracle
gives the same in 5 of 5 runs under `memcap 2G` without `ulimit -v`, which is the scout's stated
deviation.

A realistic program can spawn hundreds of threads (M2). The oracle has one thread per activity as
well, so I accept growth itself.

### (d) Byte accounting

A drain only clears flags (`clear_uninit_all`). The next sweep subtracts the bodies through the
running figure. Under the debug test profile, `pending_uninits_run_as_*` print `1 1`, which requires a
collection after a drain, so `Heap::collect`'s survivor-sum `assert_eq!` (`heap.rs:365-370`) ran after
a drain and passed. `bytes_a_body_stops_holding_leave_the_live_figure` passes too.

### (e) Mutations

Each mutant was built into the debug test binary in the scratch copy and restored from a file copy.
Afterwards, `cmp` against `git show 5ac42f73a:<file>` matched all four files.

| mutation | test | result |
|---|---|---|
| `activity.rs:630` back to the base's four extracts | `a_builtins_stream_survives_...` | red: rc 208, Error 48.1 "Stream not initialized" |
| `call.rs:1112` `&& false` | `pending_uninits_run_as_a_label_returns` | red: `0 0` |
| `dispatch.rs:2918` `&& false` | `pending_uninits_run_as_a_method_returns` | red: 152,715,185 body bytes. Stdout still prints `1 1` from the `round` label's returns, so only the peak assertion catches this mutant. |
| `pool.rs:147` `fixed` made unconditional | both `..._at_a_full_pool_runs_beyond_the_bound` | red: `(0, 2)`, and `read 124 / wrote 124` |
| `pool.rs:262` shrink disabled | every scheduler test | green (M1) |

### Perf

The callgrind command and its output, against base61:

```
memcap 8G bash bench-programs/callgrind.sh -r 1 -j 4 -o /tmp/claude-1000/p61/t11br/cg -p "rexxcps alloc emptyloop pingmsg" base61=/tmp/claude-1000/p61/t1/bin/base/rexx-run head=/tmp/claude-1000/p61/t11br/target/release/rexx-run
```

```
rexxcps    17788420306  17864340314  +0.4268
alloc      20345537190  20347942190  +0.0118
emptyloop   7786099689   7761255136  -0.3191
pingmsg     1676421116   1676102651  -0.0190
```

These are identical to the gate record. emptyloop's wall-clock gap therefore comes with flat Ir. It
belongs to Task 12, as the carry rules.
