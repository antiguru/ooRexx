# Scout E1: concurrency and memory deferred items

Binary: `git archive f7170918c rust interpreter` into `/tmp/claude-1000/p61/se1/src`, tree touched,
`CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/claude-1000/p61/se1/target memcap 8G cargo build --release
-j 4 -p rexx-exec --bin rexx-run` (log shows `Compiling rexx-exec v0.1.0 (/tmp/claude-1000/p61/se1/src/...)`).
`rexx-run` sha256 `83a0a85c16f90c7f868c319c9d15227696b2f57ad23984751f7d89a9e93de445`. Line numbers
below are at f7170918c.

| Item | Verdict | Size | Files a fix touches | Risk to 6.1 work |
|---|---|---|---|---|
| 1. full-pool inline command hang | STILL REPRODUCES (crate 5/5 hang, killed rc 137; oracle 5/5 rc 0, but only outside `ulimit -v`) | M | `command.rs` (:715-717), `scheduler.rs` (`exit_for_native` :653-683, `exit_for_block` :687), `input.rs` (:371-373), `scheduler/pool.rs` | Sim mode: `collect_on_baton` already bounds the inline wait there (`sim_block_bound`), and any park-for-a-worker path adds a scheduling event the seeded gate would record. Native inline path holds `PinKind::NativeApiCallback`. |
| 2. `TIME('E')` loop hang | CHANGED: an empty loop body still hangs (plain `WHILE`, controlled `WHILE`, `UNTIL`, inside `INTERPRET`); a body with any clause (`nop`, assignment) now ends | S | `ir/drive.rs` (`Op::LoopNext` arm :3738), `run/loops.rs` (`flat_loop_step` :1897, `run_repeating` :1330) | One store per loop pass on the hottest loop path. Measure loop benchmarks (emptyloop, rexxcps); see the per-clause-branch memory. Not related to the sim mode or R9/R10. |
| 3. `Message~halt` after a `~send` | STILL REPRODUCES, and wider than the item: any `~send`, replied or not (crate 5/5 `0`, rc 0; oracle 5/5 `1` and 4.1, rc 252) | S | `dispatch/object_protocol.rs` (`dispatch_held_message` :988), `scheduler.rs` (`halt_message` :2316), `lib.rs` (a message-to-activity record beside `started_messages` :1407, pruned at collection) | Halt delivery and wake order under sim mode (`halt_all`'s `sim_order_event`). The new record keys on message objects and must be pruned or rooted in `collect_now` (heapshape round edits `heap.rs` and the collection path). |
| 4. `EXIT` in a handler run at the `REPLY` clause's end | STILL REPRODUCES (crate 5/5 `got v`, rc 0; oracle 5/5 91.999, rc 165) | S | `dispatch.rs` (`release_method_activation` :3294, the `None` continuation arm :3327-3336), the handler-`EXIT` path that leaves `Activation::replied` set | REPLY split accounting (`splits_owed`) and ruling P41's REPLYASSERT allowance. No sim/R9/R10 contact. |
| 5. REPLY continuation state visibility | FIXED by 9f5c68ed5 (S3 Task 11, guard locks; `guards.rs` `transfer_on_reply`). The original `p04_*` probes are lost; three reconstructions match the oracle byte for byte, 5/5 per engine and 3/3 under `REXX_SWITCH_MODE=every` | none | none | none |
| 6. pending `UNINIT` objects never finalized mid-run | STILL REPRODUCES, grows linearly: 1000 objects 355 MB vs oracle 15 MB; 3000 objects 1.16 GB vs 21 MB; no-UNINIT control 53 MB vs 14 MB | M | `lib.rs` (`collect_now` :3010-3016, whose comment states a false premise), the activation-return paths (`dispatch.rs` `release_method_activation`, `run/call.rs` routine/internal returns), `dispatch.rs` `run_ready_uninits` :3478 | Changes when `UNINIT` Rexx code runs: visible in corpus output (oracle `done 995` vs crate `done 0`, currently licensed GC timing), in sim-mode schedules and the seeded gate. Interacts with R10 (resurrected bodies count live in `bytes_due`) and the heapshape round (collection cadence). Adds a check on every activation return (perf budget). |

## 1. Full native pool runs a blocking command inline

Probe `full.rex` (from `/tmp/claude-1000/p61/t8rev/p3/full.rex`, fifo path changed; fifo made with
`mkfifo /tmp/claude-1000/p61/se1/ff`):

```rexx
do i = 1 to 64
  s.i = .s~new~start('NAP')
end
call SysSleep 0.5
w = .w~new~start('WRITE')
address system 'cat /tmp/claude-1000/p61/se1/ff'
say 'read done' w~result
::class s
::method nap
  address system 'sleep 3'
::class w
::method write
  call SysSleep 0.1
  address system 'echo hi > /tmp/claude-1000/p61/se1/ff'
  return 'wrote'
```

Control `fifo.rex` (same without the 64 nappers): crate and oracle both `hi` / `read done wrote`,
rc 0, stderr empty.

Crate, 5 runs, each from a fresh directory:
`memcap 2G timeout -k 5 15 /tmp/claude-1000/p61/se1/target/release/rexx-run /tmp/claude-1000/p61/se1/probes/full.rex`,
followed by `timeout 2 tee /tmp/claude-1000/p61/se1/ff </dev/null` to release any blocked reader.
5/5: stdout empty, rc 137 (the TERM at 15 s did not end it; the KILL 5 s later did). stderr:

```
     6 *-* address system 'cat /tmp/claude-1000/p61/se1/ff'
Error 4 running /tmp/claude-1000/p61/se1/probes/full.rex line 6:  Program interrupted.
Error 4.1:  Program interrupted with HALT condition.
    10 *-* address system 'sleep 3'
Error 4 running ... line 10:  Program interrupted.
... (the same pair for the started activities)
memcap: ... Killed ...
```

Oracle under the standard wrapper (`( ulimit -v 1048576; LD_LIBRARY_PATH=... timeout -k 5 20 .../rexx FILE )`),
5/5: rc 208, stdout empty, stderr:

```
       *-* Compiled method "START" with scope "Object".
     2 *-*   s.i = .s~new~start('NAP')
Error 48 running /tmp/claude-1000/p61/se1/probes/full.rex line 2:  Failure in system service.
Error 48.1:  Failure in system service: ERROR CREATING THREAD.
```

The 1 GB virtual limit cannot hold 64 oracle thread stacks. Deviation from the standard command,
stated: the oracle run again with `memcap 2G` in place of `ulimit -v`
(`LD_LIBRARY_PATH=... memcap 2G timeout -k 5 20 .../rexx FILE`), 5/5: stdout `hi` / `read done wrote`,
stderr empty, rc 0. The oracle has one thread per activity and no pool, so this hang has no oracle
counterpart.

Root cause: `command.rs:715-717`. When `exit_for_block` finds no free pool worker it answers
`Err(block)` and the command is collected by `collect_on_baton` on the baton thread with no bound
outside the sim mode, so the started writer can never run and the deadline is never checked. The same
inline fallback exists for native calls (`scheduler.rs:669-683`, `call.run` on the baton) and stdin
reads (`input.rs:371-373`).

Size M: three fallback sites and one design choice. A `Block` touches no island value, so the command
and stdin sites can run on an unpooled overflow thread (as `collect_on_baton`'s sim branch already
does); a native call needs the baton lent to its thread, so it either parks until a worker frees or
lends to an overflow thread.

## 2. `TIME('E')` loop hang

Probe `elapsed.rex` (the item's text):

```rexx
call time 'r'
do while time('e') < 0.3
end
say 'empty body ended'
call time 'r'
do while time('e') < 0.3
  nop
end
say 'nop body ended'
```

Command: `memcap 2G timeout -k 5 10 .../rexx-run elapsed.rex` (crate), standard oracle wrapper.
Crate: stdout empty, rc 124, stderr

```
     3 *-*   end
Error 4 running /tmp/claude-1000/p61/se1/probes/elapsed.rex line 3:  Program interrupted.
Error 4.1:  Program interrupted with HALT condition.
```

Oracle: `empty body ended` / `nop body ended`, stderr empty, rc 0.

Splitting the probe shows what changed since the item was written:

| Probe | Crate (timeout 5 s) | Oracle |
|---|---|---|
| `elapsed_nop.rex`: `do while time('e') < 0.3; nop; end` | `nop body ended`, rc 0 | same, rc 0 |
| `elapsed_asg.rex`: body `x = 1` | `assign body ended`, rc 0 | not run |
| `elapsed_until.rex`: `do until time('e') >= 0.3; n = n + 1; end` | `until ended`, rc 0 | not run |
| `elapsed_forever.rex`: `do forever; if time('e') >= 0.3 then leave; end` | `forever ended`, rc 0 | not run |
| `elapsed_ctl.rex`: `do i = 1 while time('e') < 0.3` / `end`, then `do until time('e') >= 0.3` / `end` | hangs, rc 124, 4.1 at line 3 | `ctl ended 1` / `until empty ended`, rc 0 |
| `elapsed_untilempty.rex`: `do until time('e') >= 0.3` / `end` | hangs, rc 124, 4.1 at line 3 | not run |
| `elapsed_interp.rex`: `interpret "do while time('e') < 0.3; end"` | hangs, rc 124, 4.1 at line 2 | `interp ended`, rc 0 |

Root cause: the clock cache is invalidated only in `Interp::enter_stepped_clause` (`run.rs:2849`,
`clock_stale = true`). A pass boundary (`ir/drive.rs:3738` `Op::LoopNext`, then
`run/loops.rs:1897` `flat_loop_step`) counts the header clause against the deadline but does not mark
the clock stale, so a loop with no body clause re-reads the cached `TIME('E')` forever. The oracle
invalidates after every instruction it executes, the loop's own `DO`/`END` included
(`RexxActivation.cpp:647`).

Size S: set `clock_stale` at the pass boundary (`flat_loop_step`, and the non-flat
`run_repeating` at `run/loops.rs:1330`).

## 3. `Message~halt` after a `~send`

Probe `pend4.rex` (the item's text). Crate, 5 runs, `memcap 2G timeout -k 5 10`: 5/5 identical,
rc 0, stderr empty:

```
v
0
rest 1
rest 2
rest 3
main end
```

Oracle, 5 runs: 5/5 rc 252, stderr

```
     5 *-* say m~halt
Error 4 running /tmp/claude-1000/p61/se1/probes/pend4.rex line 5:  Program interrupted.
Error 4.1:  Program interrupted with HALT condition.
```

stdout `v, 1, rest 1, rest 2, rest 3` in 3 runs and `v, rest 1, 1, rest 2, rest 3` in 2.

The reply is not needed. `sendhalt.rex`:

```rexx
m = .message~new(.k~new, 'm')
say m~send
say m~halt
say 'main end'
::class k
::method m
  return 'v'
```

Crate: `v` / `0` / `main end`, rc 0. Oracle: `v` / `1`, then the 4.1 at line 3, rc 252.

Root cause: `MessageClass::send` records the sender's activity as the message's `startActivity`
(`MessageClass.cpp:432`), and `halt` asks that activity (`:811-816`). The crate's `Message~send`
(`dispatch/object_protocol.rs:988`, `dispatch_held_message`) records nothing, and
`Interp::halt_message` (`scheduler.rs:2316`) answers false for any message not in `started_messages`
and finds a target only by `root_then == Then::Started(message)`.

Size S: record the sending activity for a sent message and let `halt_message` target it.

## 4. `EXIT` in a `CALL ON` handler run at the `REPLY` clause's end

Probe `hexit.rex` (the item's text). Crate, 5 runs: 5/5 rc 0, stderr empty, stdout
`handler on main 1` / `got v` / `main end`. Oracle, 5 runs: 5/5 rc 165, stdout `handler on main 1`,
stderr

```
     2 *-* say 'got' .k~new~m
Error 91 running /tmp/claude-1000/p61/se1/probes/hexit.rex line 2:  No result object.
Error 91.999:  Message "M" did not return a result.
```

Root cause: `exec_reply` (`run.rs:2142`) leaves the split owed until the replier's next clause
(`ir/drive.rs:2602-2611`). The handler's `EXIT` ends the method first, and
`release_method_activation` (`dispatch.rs:3294`) takes the `None`-continuation arm (`:3327-3336`) and
answers the `REPLY` value. The oracle's `EXIT` overwrites the result before the split, so the sender
gets none.

Size S: an `EXIT` that ends a replied activation before its split answers the `EXIT`'s value
(here none) rather than `Replied::value`.

## 5. REPLY continuation state visibility

The item's probes (`p6-final-a/probes/p04_reply_in_arg.rex`, `p04b_reply_where.rex`) are not on disk
(`find / -name p04_reply_in_arg.rex` finds nothing). The item carried the remaining cause (the
continuation not holding the replier's guard) to S3 Task 11 as `reply-guard-transfer-witness`. The
guard transfer (`guards.rs:791` `transfer_on_reply`) first appears in 9f5c68ed5
(`git log -S transfer_on_reply`).

Reconstructions:

`replyvis.rex`: a guarded `m` replies `'r'`, loops 20000 times, then sets `v` to its argument;
main then sends guarded `get`.

```rexx
o = .k~new
say 'got' o~m('argument-value-long')
say 'after' o~get
say 'end main'
::class k
::attribute v
::method init
  expose v
  v = 'none'
::method m
  expose v
  use arg a
  reply 'r'
  do i = 1 to 20000; end
  v = a
  say 'continuation wrote' v
::method get
  expose v
  return v
```

`replyvis2.rex`: `reply f('argument-value-long')` then `v = 'cont-' || v`, read back through the
`v` attribute. `replyvis3.rex`: the replying send inside a `CALL` argument
(`call show o~m('argument-value-long')`).

Both engines, 5 runs each, all rc 0 with empty stderr, crate stdout byte-identical to the oracle's
first run in every run (`cmp`):

```
replyvis:  got r / continuation wrote argument-value-long / after argument-value-long / end main
replyvis2: got argument-value-long / after cont-none / end main
replyvis3: got r / after argument-value-long / end main
```

`REXX_SWITCH_MODE=every` (read by `rexx-run.rs:63`): replyvis and replyvis2 3/3, replyvis3 5/5
identical to the oracle.

## 6. Pending `UNINIT` objects never finalized mid-run

The Task 5a review's probe text is not recorded; reconstructed from its description. `uninit.rex`:

```rexx
.local~n = 0
do i = 1 to 1000
  o = .f~new(copies('x', 300000) || i)
end
say 'done' .local~n
::class f
::method init
  expose s
  use arg s
::method uninit
  .local~n = .local~n + 1
```

`uninit3k.rex` is the same with 3000; `nouninit.rex` drops the `uninit` method.

Commands, `/usr/bin/time -v` directly on the interpreter:
`memcap 2G timeout -k 5 60 /usr/bin/time -v .../rexx-run PROBE` and
`( ulimit -v 1048576; LD_LIBRARY_PATH=... timeout -k 5 60 /usr/bin/time -v .../rexx PROBE )`.
3 runs each; all rc 0, stderr only `time -v`'s report.

| Probe | Crate stdout | Crate peak RSS (KB) | Crate wall | Oracle stdout | Oracle peak RSS (KB) | Oracle wall |
|---|---|---|---|---|---|---|
| uninit (1000) | `done 0` | 356 500, 356 392, 355 420 | 0.72-0.74 s | `done 995` | 16 004, 14 460, 16 228 | 0.63 s |
| uninit3k (3000) | `done 0` | 1 155 504, 1 155 040, 1 155 596 | 2.20-2.65 s | `done 2995` | 21 060, 20 264, 20 728 | 1.91 s |
| nouninit (1000) | `done 0` | 52 444, 53 120, 52 728 | 0.65-0.84 s | `done 0` | 14 180, 18 916, 14 120 | 0.62-0.63 s |

The crate holds every finalizable object's 300 KB string until termination, about 385 KB per object
(800 MB more for 2000 more objects); the oracle stays flat.

Root cause: `lib.rs:3010-3016` (`collect_now`) queues resurrected objects on `uninit_ready` and says
the oracle's `runUninits` is reached only from `GC('force')` and termination. That premise is false:
`MemoryObject::checkUninitQueue` (`RexxMemory.hpp:123`, `if (pendingUninits > 0) runUninits()`) is
called at every Rexx activation's return (`RexxActivation.cpp:705`), at native activation return
(`NativeActivation.cpp:1361`), and in the activity loop (`Activity.cpp:249`, `:324`, `:3440`). The
crate drains `uninit_ready` only from `GC('force')` (`builtin/state.rs:128`), an activity's end
(`scheduler.rs:1599`) and termination (`lib.rs:3484`).

Size M: a drain at activation return, guarded by an emptiness check on a hot path, with
`run_ending_uninits`-style routing of refusals raised inside an `UNINIT`, and re-entrancy through
`processing_uninits`. Expected effect: the 1000-object probe should fall to roughly the `nouninit`
figure (53 MB); not built or measured here.
