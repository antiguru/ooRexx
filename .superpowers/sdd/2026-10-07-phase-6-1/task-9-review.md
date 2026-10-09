# Task 9 review: policies, invariants, decision trace and replay

Base `6358ca7a6`, head `88e818648` (rust/ identical to `296cac74f`). Reviewer: t9-review. Line
numbers are the diff file's (`review-6358ca7a6..88e818648.diff`, "d:N") or the tree's at `88e818648`.
Scratch: `/tmp/claude-1000/p61/t9r/` (`p1/` probes, `inj/` the injection copy, `perf/` the layout
control). Probe binary: the implementer's `head2` (`/tmp/claude-1000/p61/t9/perf/bin/head2/rexx-run`,
sha256 `bef89dcd...`, matching the gate record), run as `memcap 2G timeout 20 $R prog.rex` from a
fresh directory.

### Spec Compliance

- ❌ Issues found:
  - Replay against a different program, or a damaged trace, runs to rc 0 and prints a different
    trace hash with no complaint (Important I1 below; the report's Concern 5).
  - The perf record has no running total against the 6.1 base (Important I3).
- Everything else in the brief is present and does what it says, verified by running (below).
- ⚠️ Cannot verify from the diff: whether any ooTest group Task 10 runs keeps a thousand or more
  activities parked at once (I2 decides how much that matters).

### Checks run

**1. Policies.**

- `pre:d,k=N` takes exactly d preemptions among the first k contended steps. `spin.rex` (two started
  activities, 20,000 iterations each, about 20,000 contended steps), d in {1, 3, 7} by k in {20, 200,
  5000} by seeds 1-3, counting `preempt` lines in the trace: within k is exactly d in all 27 runs,
  beyond k is 0. With d >= k (`pre:5,k=3`) the trace is `preempt 1/2/3`, as documented.
- `uniform:p` rate over about 40,000 contended steps, two seeds each: p=0.05 gives 0.0498 and 0.0516;
  0.2 gives 0.2014 and 0.2014; 0.5 gives 0.4992 and 0.5042; 0.9 gives 0.9017 and 0.9037. All inside
  binomial noise.
- `pct:d,k=N`: three started workers each print their tag six times; the number of runs of equal
  lines in the 18 output lines, seeds 1-30. `pct:1,k=1` gives 3 in all 30 (one block per worker:
  no change points, so the highest priority runs to its end). `pct:2` gives 3 or 4, `pct:3` gives 4
  or 5, `pct:5` gives 4 to 7: never more than 3 + (d-1). `uniform:0.05` gives 8 to 15 for contrast.
  The priority shape and d-1 change points behave as specified.
- The only ready-queue dequeue is `next_runnable`'s (`grep -n "ready\.(pop_front|remove|...)"`:
  `scheduler.rs:1776` and `sim_pick`; the `retain`s at `:1675/1707/1718` drop `me`), so no site
  bypasses pct's pick.
- `order=fifo` against one-event: `sems.rex` (five waiters on one `EventSemaphore`, one post) under
  `fifo`, seeds 1-20: `order=fifo` wakes `12345` in all 20; the default one-event order gives 19
  distinct permutations in 20.
- Fairness floor: `FAIRNESS_FLOOR = 24_000_000 / 58` (d:1958). 24 ms is the oracle's
  `timeSliceLength` (`interpreter/concurrency/ActivityManager.hpp:359`); 58 ns is the gate record's
  `## Task 8` median (17,229,378 clauses/s, `phase-6-1-gate.md:791-792`). `floor=50` on `spin.rex`:
  preemption gaps are 50 in every case under `fifo`, `pre:1,k=10` and `uniform:0` (799 gaps each), and
  50 plus one pct change point under `pct:2,k=100`. The polling program ends with `A ended`, rc 0,
  under `fifo`, `pre:1,k=1`, `uniform:0`, `uniform:0.000001`, `pct:1,k=1` and `pct:3,k=5`, with
  contended=413,796 to 413,797 (one floor); `pre:1,k=1` takes 827,589 (two floors: the preemption at
  step 1 puts the poller ahead of `setFlag` in FIFO, so it is chosen again after the first floor),
  which is the specified behaviour, not a defect.

**2. Invariants: the eight uncovered checks and a real baton release.** In the scratch copy
(`/tmp/claude-1000/p61/t9r/inj`, `git archive 88e818648 rust interpreter`) I added nine
`Corruption` variants to the existing hook and one test running each through the existing
`corrupted()` helper. Command: `CARGO_TARGET_DIR=/tmp/claude-1000/p61/t9r/target-inj memcap 8G
cargo test -j 4 -p rexx-exec --lib inj_uncovered_invariants -- --nocapture`, exit 0
(`/tmp/claude-1000/p61/t9r/inj2.log`).

| corruption injected | program | rc | first refusal line |
|---|---|---|---|
| `ActivityId(9999)` pushed on `ready` | THREE_ACTIVITIES | 120 | a ready handle naming no idle activity |
| `idle.push(None)` | THREE_ACTIVITIES | 120 | a handle neither free nor filed |
| first ready handle pushed on `free` | THREE_ACTIVITIES | 120 | a free handle naming an activity |
| running handle pushed on `free` | THREE_ACTIVITIES | 120 | a running activity's handle free |
| first ready activity's record moved into the running slot | THREE_ACTIVITIES | 120 | a running activity filed as idle |
| a lock's owner set to `ActivityId(9999)` | GUARDED | 120 | a guard held by no activity |
| a lock's first waiter queued again | GUARDED | 120 | a guard waiter queued twice or behind itself |
| a lock's first waiter popped, its `Waiting::Guard` kept | GUARDED | 120 | a guard wait missing from its guard's queue |
| `self.baton.release()` before the check, `acquire()` after | THREE_ACTIVITIES | 120 | a switch on a thread not holding the baton |

Every check fires on its corrupt state with its named invariant. The last row is a real release (no
holder at all), so `held_here` reads the baton's actual holder (`baton.rs:104-106`, a thread-id
compare), not only the test flag. No check failed to fire, so there is no finding on reachability.
What the runs show besides (Minor M1): most corruptions are refused twice.

**3. Trace and replay across processes.** Programs `two.rex` (TWO_ACTIVITIES), `deciding.rex`
(DECIDING), `guardwhen.rex` (four `GUARD ON WHEN n > 0` takers, four gives), `sems.rex`; policies
`uniform:0.3`, `pre:2,k=10`, `pct:3,k=20`, `uniform:0.5,order=fifo`, `uniform:0.2,gc=0.05`, `fifo`;
seeds 3 and 11: 48 recordings, each replayed by `sim:replay=FILE` in a second process. All 48 match
rc, stdout and the `trace=` hash; traces run from 1 to 108 lines. Divergence (concern 5) and commas:
see I1 and M2.

**4. Exploration value.** `two.rex` under `pre:1,k=N`, seeds 1-40: k=1 gives `t m1 m2` 40 times;
k=3 gives `m1 m2 t` 19, `m1 t m2` 4, `t m1 m2` 17; k=5 gives 22/10/8. All three interleavings of
the program are reached at k=3, which is more than the two the test asserts (M4).

**5. Perf.** The gate record quotes both callgrind commands and names the base (`6358ca7a6`) with
sha256s; the binaries in `/tmp/claude-1000/p61/t9/perf/bin` still match them. Concern 4 and the
running total, one callgrind run, exit 0, every spread at most 0.0001%:

```
memcap 8G bash rust/bench-programs/callgrind.sh -r 2 -j 6 -o /tmp/claude-1000/p61/t9r/perf/cg1 -p "pingmsg pingguard pingsem alloc alloc4c heapshape rexxcps emptyloop" base=/tmp/claude-1000/p61/t9/perf/bin/base/rexx-run pad5=/tmp/claude-1000/p61/t9r/perf/bin/pad5/rexx-run pad40=/tmp/claude-1000/p61/t9r/perf/bin/pad40/rexx-run head2=/tmp/claude-1000/p61/t9/perf/bin/head2/rexx-run base61=/tmp/claude-1000/p61/t1/bin/base/rexx-run
```

`pad5` and `pad40` are `6358ca7a6` plus `layout-pad.py 5` and `40` (sha256 `56ad9efe...`,
`d1f5cc98...`, one `Compiling rexx-exec` each, own target dirs after touching the tree); `base61` is
Task 1's base binary, `e6af1198b`, sha256 `2ea19b3e...` matching the gate record's `## Task 1`.
head2 against base reproduces the record to the fourth decimal (alloc +0.3552, pingmsg +0.2005,
heapshape +0.1280).

- Layout control: pad5 and pad40 against base are within 0.0000% on all eight programs, so dead code
  in the driver does not move instruction counts here; alloc's +0.36% is not that kind of layout
  noise.
- Attribution of alloc's +0.36% (`callgrind_annotate` on the implementer's
  `cg2/alloc.{base,head2}.r1.cg`): `core::str::converts::from_utf8` is called 18,000,310 times in
  both binaries, from the same site; its self cost rises from 1,029,020,647 to 1,095,021,186 Ir
  (+3.7 Ir per call: the `uint_macros.rs` lines double, 36.1 M to 72.1 M). Same calls, more
  instructions per call: the compiler generated a different `from_utf8` body in the head build. No
  added work on any path; a codegen perturbation from this task's build, inside the budget. Concern
  4 can be closed with that sentence.
- Running total: see I3.

**6. Concern 2 (SysSleep TEST_SLEEP_DURATION).** Reproduced. `call time 'r'; r = SysSleep(d); e =
time('e')` for d = 0.00000001: `0` under `sim:1`, `sim:2,uniform:1` and `sim:3,clock=real`;
default mode 0.000042; the oracle, 5 runs, 0.000051 to 0.000110. d = 0.0000004 reads 0.000001 or 0
in sim depending on the quanta. The oracle formats elapsed time in whole microseconds and prints `0`
for exactly zero (`BuiltinFunctions.cpp:1456-1478`), and the crate's sim does the same. The test's
lower bound is `duration - tooEarly` = 1e-8 (`SysSleep.testGroup:58,107`), which only a measured
elapsed of at least 1 µs satisfies, so it passes on a real system only because nanosleep overshoots
by tens of microseconds. The virtual clock is not wrong: it does what the spec says (a sleep jumps
to its deadline). The test assumes wall time. Task 10's options are a `SIM_EXEMPT` row with this
evidence, or a modelled sleep latency (a seeded overshoot drawn from the oracle's measured range,
sized as the quantum is); the second also makes `time('e')` after any short sleep look like the
oracle's.

### Strengths

- The policies match the brief exactly, on every measure above, and `pct`'s run structure is the
  textbook PCT shape (at most d blocks beyond the workers').
- Replay is exact wherever the program is the same: 48 of 48 across processes, every decision kind.
- Every invariant check fires on its own corrupt state with its own message, including the eight the
  report left unwitnessed; the `Corruption` hook made adding cases cheap.
- The trace hash is platform-independent (FNV-1a over tag and little-endian value, d:2317-2348) and
  is kept incrementally, so `uniform:1` on a long program does not build a vector unless asked.
- The round 1 perf fix (making `sim_declines_collection` cold) was found by `cgdiff` and explained,
  and the report is candid about what it did not run.

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

**I1. A replay that diverges is silent.** `sim.rs` `Replaying::pick` clamps (`pick.min(last)`,
d:2395-2399) and falls back to the front once the trace is exhausted; `reached` (d:2402-2413) steps
past recorded points it never reached; nothing at the run's end compares what was replayed with what
the file holds. Run:

- `deciding.rex`'s trace (`sim:3,uniform:0.3`, hash `69a0d1c68db3fe90`) replayed on `guardwhen.rex`:
  rc 0, output `done  3 1 4 2`, hash `8d6a6edd34f5a2b3`; on `sems.rex`: rc 0, hash
  `21e812f94d6edd5b`.
- The same trace with `pick 999` inserted after the header, replayed on `deciding.rex` itself: rc 0,
  `c woke` and `b woke` swapped against the recording, hash `36c9b20573b5b02d`.

A replay is how a seeded failure becomes a debugging session; one that quietly takes another
schedule reads as "does not reproduce". The divergence is already visible to the code: in every case
above the replay's own hash differs from the hash of the file's decisions. Fix: under replay, record
each decision the run takes and refuse at the first one that differs from the file's next decision
of that kind (a pick out of range, a preemption the trace has at a step the run did not reach), and
at the end refuse if the file has decisions left; at the least compare `trace_hash(file)` with the
run's hash at the end. Writing the hash into the trace header would also let a reader check a file
by eye.

**I2. The invariant check is quadratic per switch.** `check_invariants` (d:1121-1185) walks every
handle and for each does `free.contains` (O(free)), `readied.contains` (O(ready)) and
`has_wake_source`, whose `holds_park_reason` (d:1189-1204) scans every sleeper and every message's
waiters. Per switch that is O(handles x (free + ready + sleepers + waiters)), and the switches grow
with the activities. Measured, `n` started activities each parked on one `EventSemaphore`, then
posted, `sim:1,uniform:0.5` against `every` (wall seconds): n=250 0.12/0.07, 500 0.48/0.09, 1000
3.07/0.09, 2000 22.4/0.13. `perf record` at n=1000 puts 97.4% in `sim_check_switch`. A Task 10 gate
run with a program of that shape reads as a hang past its deadline. Fix: one pass that builds a
per-handle state (ready, sleeper, waiter, free) in vectors indexed by handle, then checks each handle
in O(1): O(handles + sleepers + waiters) per switch.

**I3. The perf record has no running total, and the running total is over on heapshape.** The
global constraint gates running totals against the 6.1 base (Task 1's base, `e6af1198b`); Task 5's
record carried `base61` for this; Task 9's compares with `6358ca7a6` only (gate record `## Task 9`).
Measured (command in check 5), head2 against base61:

| program | Tasks 1-8 (base vs base61) % | after Task 9 (head2 vs base61) % |
|---|---:|---:|
| pingmsg | -0.4541 | -0.2545 |
| pingguard | -0.5908 | -0.3663 |
| pingsem | -0.4358 | -0.2494 |
| alloc | -0.2804 | +0.0738 |
| alloc4c | -0.8796 | -0.8139 |
| heapshape | +1.0266 | **+1.1559** |
| rexxcps | +0.0230 | +0.0925 |
| emptyloop | -0.3191 | -0.3191 |

heapshape is +1.16% against the 6.1 base, over the +0.5% budget; +1.03% of it was already there
at `6358ca7a6` (no earlier gate named heapshape, so nobody saw it), and Task 9 adds +0.13%. Every
other program is inside. Fix in this task: add the base61 column and the running totals to `## Task
9`, and take heapshape to Moritz as the constraint says (three rounds, then his ruling), naming
which earlier task carried the +1.03% (bisect the 6.1 commits with this command on heapshape
alone). Task 9's own increment is not the cause.

#### Minor (Nice to Have)

**M1. A violation is refused more than once.** In the injection runs every case except `RunningFree`,
`BatonReleased`, `NoWakeSource` and `GuardQueue` prints two refusal lines (`ReadyTwice`: `an
activity ready twice` twice; `RunningReady`: that, then `a ready handle naming no idle activity`),
the committed six included (I re-ran them in the same test), because the run continues on the corrupt state after the first refusal and the
next switch finds it again. The committed test checks `stderr(&broken).contains(...)` (d:3076-3080),
which hides it. Harmless for a gate that fails on the first line; a reader of the stderr sees
duplicated or secondary refusals. Either stop checking once `sim.inconsistent` has been set and
refused, or assert the stderr exactly.

**M2. Trace-file edges.** A comma in a path is refused loudly (`trace=.../a,b.txt` rc 2 "`b.txt` is
not a policy"; `replay=.../c,d.txt` rc 2 but naming the wrong file, `.../c: No such file`). An
unwritable trace (`trace=/nonexistent/dir/t.txt`) and an empty one (`trace=`) both exit 0 with only
`rexx-sim: the trace ...` on stderr, so a harness that asked for a trace and checks the status does
not learn it has none. Two `trace=` items: the last wins silently. Refusing an empty path at parse
and making a failed write change the exit status would close these.

**M3. `pub fn trace_hash` has no caller** (d:2259-2265, re-exported at `lib.rs:68`): nothing in the
workspace calls it (`grep -rn "trace_hash(" crates`). Delete it or use it in I1's check.

**M4. The exploration test asserts two of the three interleavings.**
`pre_1_reaches_both_interleavings_over_a_seed_range` (d:2911-2922) checks `t m1 m2` and `m1 m2 t`;
`m1 t m2` is also reachable at `pre:1,k=3` (4 of 40 seeds, check 4). Asserting all three (widening the seed range if 1-20 misses it) makes the test
witness the policy's full reach.

**M5. Process.** The report's Concern 7 (a `timeout 590 bash -c "until ...; done"` wait) is a
global-constraints breach the implementer self-reported; noted, nothing to fix in the tree.

### Assessment

**Task quality:** Needs fixes

**Reasoning:** The policies, invariants and same-program replay all do exactly what the brief asks,
and every invariant fires on its own corruption; but replay never notices when it is replaying the
wrong thing, and the invariant check's quadratic cost can turn a large sim run into a false hang.
On perf, Task 9's own increments are inside +0.5% and alloc's is codegen, but the record omits the required running total, which is over on heapshape (+1.16% against the 6.1 base, +1.03% of it from earlier tasks).
