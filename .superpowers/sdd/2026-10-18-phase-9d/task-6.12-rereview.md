# Task 6.12 re-review: fix round 1

This is a scoped re-review of `94982dccc..a0a58dce3`. The code is `174c357be`, `3f3dea5f8` and
`d995b8f17`. I built HEAD into a fresh scratchpad target dir (`rev612r1-target`,
`cargo build --release --bin rexx`, exit 0, sha256 `5eef2f35...`). The comparison binary is round 0's
`/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.12/rexx-head` (`7fba9068c`). Probes ran as in the round-0
review: the oracle under the standard wrapper, the crate under `memcap 2G timeout 20`, each from a fresh
directory, with stdout, stderr and rc compared separately. The probe sources were in
`scratchpad/rev612-probes.lQEo/`, which is not durable, so the deciding lines are quoted below.

## The seven round-0 findings

I reran every round-0 probe against HEAD. All 27 are identical to the oracle on stdout, stderr and rc,
except the STREAM-replacement probes. There the only difference is the oracle's SIGSEGV at exit, which
is entry 54.

| finding | status | deciding probe |
|---|---|---|
| 1 started-message panic | closed | `startpanic`, `startpanic2`, `startpanic3`, `thr_start`, `thr_sendstart`: rc 0, `superclass via start: Object`, `protected rexx via start: foo ran`, `start -> Object events:` with no events, as on the oracle |
| 2 protected `PUT` recursion | closed | `dirput2`/`dirput3`/`dirput4`: `event LOCAL STDOUT` / `event METHOD SUPERCLASS` / `result Object`, rc 0 |
| 3 donated identity | closed for the `isProtected` readback and for new instances (`shared`: `Directory 0 ... Bag 1`; `sharedpriv2`: `table put ok` / `bag 97.2`). Open for instances that already exist: new finding 1 | |
| 4 Directory entry | closed | `dirm`: every read is `m ran`, events `LOCAL D` only |
| 5 STREAM `.nil` | closed | `t_nil`: `nil -> syntax 97.1 Object "The NIL object" does not understand message "LINEOUT".` / `exists: 0`, rc 0 (the oracle has the same lines, then rc 139) |
| 6 REQUIRES NAME | closed | `req`: identical. `nil`, `object` and `missing` each give 3.1 naming the replacement, and `unset` gives 43.901 after one event |
| 7 RC on a manager-raised condition | closed | `nilmx`: `rc nil failure result: failure 1 The NIL object`. `rcabc`: `failure 0 abc` |

All nine `security_manager_*` witnesses (`rust/corpus/lang/`) are identical to the oracle on stdout,
stderr and rc.

## Concern 1: the lifted protection

`invoke_armed_executable` calls `lift_protection(resolution.method)`, which writes `false` into
`special_methods[method].protected`, runs `begin_invoke`, and then writes it back. `special_methods`
is the interpreter-wide table that every send's seam test and every `isProtected` readback read, so
the flag is shared state. Any send of that `MethodId`, and any `isProtected` read of it, sees the
lifted value for as long as the window is open. What I could and could not reach:

* **A Rexx body:** not reachable. The body runs after `begin_invoke` returns, which is after the
  restore. Inside a protected `FOO` run by `Object~run` and recursing into itself,
  `.K~method('FOO')~isProtected` reads 1 on every level (`rec`, `run -> foo 2 1 foo 1 1 leaf`,
  identical to the oracle).
* **SYNTAX in the window:** restored. `lift`/`restore` brackets the call to `begin_invoke`, and its
  `Err` is propagated only after the restore. A native that raises in the window is
  `self~run(.Class~method('DEFINE'), 'I', .Name~new, 'return 1')` (88.909). After it,
  `Class DEFINE protected: 1` (`w1`, identical to the oracle). A Rexx body that raises
  (`viaRunErr`, 40.1) leaves `after: 1 1`. HALT and REPLY are delivered at clause boundaries, and the
  window contains no Rexx clause, so neither can land inside it.
* **A native body that sends the same method:** I found no protected primitive whose native body
  runs user code here.
  * `define`'s name argument does not call a user `string` method (88.909 on both sides).
  * Pending UNINITs run in the window would show up as `isProtected` read as 0 from inside an UNINIT.
    In 277 UNINITs run while `self~run(.Class~method('SUBCLASSES'))` was repeated, the count was 0
    (`u2`, `.local~z` 0).
  * `defineMethods` with a user Directory subclass whose `supplier` sends `DEFINE`: the oracle calls
    that `supplier` *inside* the native body (`user supplier ran`,
    `plain -> done events: METHOD DEFINEMETHODS a COLL | METHOD DEFINE INNER,return 2`). The crate
    never calls it (`plain -> done events: METHOD DEFINEMETHODS a COLL`), and round 0 behaved the same
    way. This is a pre-existing divergence, and a queue candidate. Once it is fixed, user code runs
    inside the window. A recursive `defineMethods` from that `supplier` would then skip the manager.
* **Another thread:** in `g2`, an unguarded `viaRun` runs a guarded protected `FOO` with
  `Object~run` while another activity holds `FOO`'s guard. Meanwhile that activity's managed code sends
  `FOO` 40 times and reads `isProtected` each time. Every probe answered `1X`, meaning protected and
  intercepted, with 40 or 80 `METHOD FOO` events, under the default mode and under `sim:1`, `sim:2`,
  `sim:3,uniform:0.2`, `sim:11,uniform:0.2`, `sim:21,uniform:0.2` and `sim:13,pct:3,k=40`. So the
  guard wait is not inside the window. `sim:5,pct:3,k=40` and `sim:29,pct:3,k=40` stop with rc 120,
  `a pinned wait for a guard that only an activity pinned below it can end is not implemented`. Round 0
  does the same on those seeds, so this is the known pinned-guard limitation and not this round.

So I found no program today that observes the window. It is still a defect, as the dispatch says: the
decision "this run is not a send" belongs to one call, but it is stored on the method, where every
other send and readback in the process can see it. It also turns unsafe as soon as a protected native
calls back into Rexx, which the oracle's `defineMethods` already does. Finding 2 below says what to do.

## Concern 2: no RESULT on a manager-raised condition

rexxref's security chapter says only that a FAILURE or ERROR entry raises the condition, and names RC
as the command return code. `condtra.xml`'s condition-object list defines `RESULT` as "Any result
specified on the RETURN or EXIT options of a RAISE instruction", and adds "&nil; is returned for any
entry not available to the condition". A manager-raised command condition comes from no RAISE, so the
documented condition object has no `RESULT`. The round's behaviour therefore follows the documented
rule, as well as the oracle. Probe `c2`, identical on both sides:
`manager failure: 7 0 The NIL object 0`. That is `rc` 7, `hasIndex('RESULT')` 0, `~result` `.nil`, and
the variable `RESULT` unset.

The ordinary command path is the one that departs from the documentation. On both sides it carries
`RESULT` = rc (`FAILURE rc 127 hasResult 1 result 127`, `ERROR rc 3 hasResult 1 result 3`). It
predates the task, lies outside its clusters, and is not a defect of this round. It is worth taking to
Moritz with the next batch of Deviation candidates, not as a fix here.

## Issues

### Critical

None.

### Important

1. **Instances that already exist keep the donated identity, so `setProtected` skips the manager for
   them and `setPrivate` no longer applies to them.** `unshared_identity` installs a fresh copy in
   the class's dictionary and repoints the method objects. Instances created before the call still
   dispatch the old `MethodId`. Probe `cache2`: `b0`, `b1` and `b2` are made before
   `.Bag~method('PUT')~setProtected`, and `b1` and `b2` are also sent a message first. Then a managed
   routine sends `put` to each:
   ```
   oracle: .b0~put(2) events: METHOD PUT   (same for .b1, .b2)
   HEAD:   .b0~put(2) events:              (same for .b1, .b2)
   both:   x = .Bag~new; x~put(1) events: METHOD PUT
   ```
   Probe `cache3`: `pb = .Bag~new` before `.Bag~method('PUT')~setPrivate`, then `pb~put(1)` from the
   program. The oracle and round 0 give `97.2`, while HEAD gives `existing bag put after setPrivate:
   ok`. A new Bag is 97.2 on HEAD too. `.Bag~method('ITEMS')~setProtected` gives the same split:
   `.pb~items` sends no event on HEAD and `METHOD ITEMS` on the oracle. A `MyBag` instance made before,
   and an Array made before `.Array~method('APPEND')~setProtected`, both do ask. The defect is
   specific to the classes whose identity is unshared, here Bag. For protection this is a bypass:
   every object an agent already holds escapes the manager. For `setPrivate` it is a regression from
   round 0. Fix: the existing instances' behaviours must see the copy. Either rebuild or repoint the
   behaviours that resolve `PUT` through the asked class, or mark the copy by name in the dispatch
   tables those behaviours use, not only in the class dictionary. Add a witness line with an instance
   made before each setter.

2. **The protection lift writes shared state for a decision that belongs to one call.**
   `lift_protection`/`restore_protection` mutate `special_methods`, which every send and every
   `isProtected` read in the process consults. No reachable observable was found (see Concern 1).
   But the window becomes visible to any code that runs inside a native body, such as the user
   `supplier` the oracle's `defineMethods` calls, and to anything that switches activity mid-call.
   Fix: make it a per-call decision that the seam reads, and never mutate the method. For example,
   give `invoke_armed_executable` its own entry into `begin_invoke` that carries "not a send", so the
   seam skips its `PROTECTED` question for this call alone. That could be a `Resolution` or
   invocation flag that the seam tests after `security_armed`, or a `begin_invoke_run` that clears the
   call without the seam. The cold armed path already pays for the branch, so the unarmed send path
   is unchanged.

### Minor

3. **Entry 55's heading is wider than what was measured.** Entry 55 says "A security manager's
   REQUIRES `NAME` replacement that is not a string" crashes the oracle. Its own program does: I got
   SIGSEGV 4 of 4. But my round-0 `req` probe sets the same `.object~new` replacement inside a loop
   under `signal on syntax`, after a `.nil` round, and there the oracle answers
   `syntax 3.1 Failure during initialization: File "an Object" is unreadable.`, rc 0, 3 of 3. That is
   the crate's answer. The entry should name the shape it measured, and say that a trapped run can
   answer 3.1 instead. The cause line (the unchecked cast) can stay.

### Queue candidate (pre-existing)

* `Class~defineMethods` given a collection subclass does not send the collection's `supplier` here.
  The oracle does, and the sends made from that `supplier` reach the manager
  (`METHOD DEFINE INNER,return 2`).

## Checks

From `rust/`, with `CARGO_INCREMENTAL=0`, the fresh target dir and `memcap 8G -j 4 --release`:
* `cargo test -p rexx-exec --lib`: exit 0, 1155 passed.
* `-p rexx-exec --test security_replacements --test dispatch_seam`: exit 0 (6 and 2 passed).
* `-p rexx-parse --test sourceline_oracle`: exit 0.
* `REXX_CORPUS_GATE=1 -p rexx-exec --test corpus`: exit 0, `1132 of 1132 matching`.
* Perf: `callgrind.sh -r 1 -B 0.5 -p "sendloop dispatch rexxcps startup" pin=... r0=<round 0>
  head=<this build>`, exit 0. With libc and ld-linux subtracted, head minus round 0 is: sendloop -467
  Ir in 13.79 G, dispatch +481 in 20.64 G, rexxcps -3056, and startup -733 in 57.3 M (-0.0013%).
  This reproduces the report's "own 0.0000" rows.

## Assessment

**Task quality:** Needs a small fix round.

**Reasoning:** All seven round-0 findings close on the probes that found them, plus the round's own
witnesses. Concern 2 follows the documented rule. But the unsharing that closes finding 3 leaves every
existing instance of the unshared class on the old identity. For protection that is a manager bypass,
and for `setPrivate` it is a regression from round 0. The protection lift is a per-call decision stored
in global state. It cannot be observed today, but the fix is cheap, and it should not wait until a
native callback makes it observable.

**Verdict: CHANGES REQUESTED** (findings 1 and 2. Finding 3 is prose.)
