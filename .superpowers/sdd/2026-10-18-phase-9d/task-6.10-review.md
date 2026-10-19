# Task 6.10 review: trace

Reviewed `d2e55befa..aa7e2a079`. I built HEAD (`rev610-target`) and the parent `d2e55befa` (a `git worktree` with
its own target dir, `rev610-ptarget`), both `cargo build --release --bin rexx`, exit 0. For the perf bisection I
also built `c239cd23f` and `e5f54ba1d`. For the Concern 2 experiment I built one scratch variant of HEAD, described
below. All of these were in their own worktrees and target dirs, and no tracked file was edited. Each probe ran
from its own fresh directory: the oracle under the standard wrapper, the crate under `memcap 2G timeout 20`, or
`timeout -s KILL` for the hang probes. I compared stdout, stderr and rc separately. The probes were in
`scratchpad/rev610-probes.x8Xh/`, which is not durable, so the deciding lines are quoted below.

The controller's commits in the range (`2dc239a9d`, `e480a4255`, `b233e2b08`) touch only `.superpowers/` and
`rust/CLAUDE.md`. No new macro was added. `ir/drive.rs` gains six lines inside the existing `region_ops!`.

## Spec compliance

The six clusters are fixed for the shapes their tests and witnesses use. I confirmed the seven witnesses (corpus
gate 1110 of 1110) and the cluster tests that left `suite-9d-set.txt`. Concerns 1, 3 and 4 are true as stated
(below). But the task leaves three things that the project's bar does not accept as they stand:

* the `traceoutput-invalid-destination` fix is correct only for a body compiled under a tracing setting. The
  ordinary shape, a `TRACE` instruction in the body, is the stale case (finding 1);
* the deadlock fix trades a deadlock for a hang that the parent and the oracle do not have (finding 2);
* an unmeasured per-call cost puts `sendloop` over the 0.5% budget (finding 3).

The report's Perf section says "Trace construction and every other new path run only while a line is traced".
That is false for `finish_call` and `exec_expose`.

## Strengths

* **Every TRACE option matches the oracle** on a probe with an assignment, a command, a CALL to a PROCEDURE,
  stems, PARSE, a DO loop and a SIGNAL. I ran `trace A C E F I L N O R S` with the default route. stdout
  (including `.TraceObject~counter`'s delta) and rc are identical for all ten. The one stderr difference is
  `13 *-*   signal l1` against `13 *-* signal l1`, the clause after a one-line DO loop. The parent has the same
  difference, so it is outside this task. `S` is 24 on both sides.
* **Destinations.** `.traceOutput~destination(.output)`, a `.stream` file, `.error~destination(.output)`, trace in a
  method, trace in INTERPRET, and `.TraceObject~option` `T S F P N` routed to `.output` all match on stdout, stderr
  and rc. The parent differed on stdout here.
* **`P` and NOTIFY.** `.TraceObject~option = 'P'` keeps the line back while the counter still counts it, and NOTIFY
  gets every object. stdout and stderr match, where the parent differed.
* **Interactive debug.** I ran `trace ?r`, then `trace n`, `trace value 'bogus!'`, `.context~package~trace` and
  `trace off`, with lines typed on stdin. stdout, stderr and rc (232) match.
* **REPLY.** Fifty guarded `m: ... reply` sends followed by a guarded send, `if 1 then reply`,
  `do 1; reply; end`, `reply 7; return`, and a REPLY rest that waits on `guard on when` under `trace i`, all match
  the oracle, with and without a Rexx destination. Running `trace_reply_continuation.rex` 10 times on the oracle
  gave one digest. The crate gave that same digest under `wall`, `sim:1`, `sim:7`, `sim:42`, `sim:3,uniform:0.2`,
  `sim:9,uniform:0.2`, `sim:5,uniform:1`, `sim:8,uniform:0.5`, `sim:7,pct:3,k=40`, `sim:11,pct:3,k=200` and
  `sim:2,pre:2,k=40`.
* **The seeded gate.** `REXX_SIM_ONLY=corpus/lang/trace_reply_continuation.rex:program`: exit 0, 70 of 70
  `agrees with 1`. The eight other trace rows the report names (`call_trace_nested`,
  `reply_continuation_error_trace`, `trace_object_activity_fields`, `trace_object_native_caller`,
  `trace_object_guard_fields`, `guard_when_traces_each_evaluation`, `guard_when_trace_object_waiting`,
  `trace_debug_ignores_trace`) each give exit 0, 70 of 70.
* **The deadlock is gone.** In `two1.rex`, two started activities each trace 30 passes under `trace r` into a Rexx
  destination whose LINEOUT bumps a guarded class counter.
  * The parent was killed at 10 s (rc 137) under `sim:3,uniform:0.2`, `sim:9,uniform:0.2`, `sim:5,uniform:1`,
    `sim:7,pct:3,k=40` and `sim:11,pct:3,k=200`.
  * HEAD prints `lines A 30 B 30` under all ten modes, the same as the oracle's 8 of 8.
  * `two2.rex`, the same shape on the default route, gives 366 stderr lines with an identical sorted digest on
    HEAD, the parent and the oracle under every mode. So a thread that traces every clause does not starve
    another thread that also traces every clause.
* **Concerns 3 and 4 are true.**
  * Concern 3 (`probes/untrapped-trace-failure/p.rex`): the oracle prints the later lines, `routine end` and
    `not reached`, then rc 139. HEAD reports 97.1, rc 159.
  * Concern 4 (`probes/noprolog-install`): with routine and package names masked, the only stderr difference is
    the missing `<I<`/`>I>` pair of `pn2`, the NOPROLOG package with a class.
* **Reentrancy terminates.** A destination whose LINEOUT runs `trace r`, and one that calls a traced routine, both
  finish with rc 0 on HEAD, with the nested lines on stderr. The oracle segfaults on both, as Concern 7 says.
* **Tests.**
  * `cargo test --release -p rexx-exec --lib`: exit 0, 1152 passed.
  * `REXX_CORPUS_GATE=1 ... --test corpus`: exit 0, `1110 of 1110 matching`.
  * `-p rexx-parse --test sourceline_oracle`: exit 0.

## Issues

### Critical

None.

### Important

1. **A failed trace delivery lets the traced clause run, in the ordinary shapes too (Concern 2, sized).** Concern 2
   presents the stale chunk as a corner. It is in fact the normal case. A `TRACE R` instruction, the `TRACE()`
   builtin, or a TRACE inside INTERPRET makes the rest of the body stale. Only a body compiled under
   `::OPTIONS TRACE` (or re-entered under the setting) gets `Op::TraceClause`. Every probe below has the caller
   trap SYNTAX and print the counter. The routine's last clause is `.local~c = .local~c + 1` after
   `.traceOutput~destination("")`:

   | routine body | oracle | HEAD |
   |---|---|---|
   | `trace r; ...; .local~c += 1` | `syntax 97.1 c=0` | `syntax 97.1 c=1` |
   | `...; x = trace("R"); .local~c += 1` | `c=0` | `c=1` |
   | `interpret 'trace r; ...; .local~c = .local~c + 1'` | `c=0` | `c=1` |
   | method: `trace r; ...; .local~c += 1` | `c=0` | `c=1` |
   | loop, `if i = 2 then trace r` | `c=1` | `c=2` |
   | internal `r:` routine (stale4) | `c=0` | `c=1` |

   The report's own probe, `stale-clause-runs`, prints `say ran` on HEAD only. The witness was edited in
   `a5266a662` from `say 'r1 say ran'` to `x = 'r1'`, so that the routine r1 no longer shows the divergence.

   **The same class is reachable without staleness.** Only the compiled *echo* abandons its clause. A failed
   `>>>`, `>V>` or `>=>` line is queued to the clause's end, so the rest of the clause runs. In `mid7.rex`, a
   body is compiled under `::options trace r` (not stale) and runs `say f() 'tail'`. `f` is an untraced routine
   that calls `.traceOutput~destination("")`. The oracle's stdout is `syntax 97.1`. HEAD's is
   `5 tail|syntax 97.1`: the SAY ran after its `>>>` line failed. `mid4.rex` gives the same result with a
   destination whose LINEOUT raises 40.1 on `>>>` lines (oracle stdout `syntax 40.1 ...`, HEAD
   `said|syntax 40.1 ...`). The report does not mention this case.

   **Is there a fix that costs nothing on the clause path?** I tried the obvious one. In `clause_region!`, before
   `region_ops!`, I added `if $stale && let Some(f) = self.stale_echo_failure() { break 'cold Err(f) }`, where
   `stale_echo_failure` is `#[cold] #[inline(never)]`.
   * It fixes every stale row above: stdout, stderr and rc all match the oracle.
   * It costs, against HEAD in one callgrind run: `nop` +6.49%, `assign` +5.97%, `emptyloop` +1.94%, `sayloop`
     +0.50%, `parse` +0.39%.
   * The implementer's earlier `$entry.abandoned` form (`c239cd23f`) costs `nop` +4.33%, `assign` +4.88% and
     `emptyloop` +1.29% against the parent. The report's "about 0.5% on every clause" comes from measuring only
     `sayloop` and `parse`. On clause-dense programs the real cost is about ten times that, so the ruling is
     weighing the wrong number.

   I found no zero-cost form inside the driver. Any test between the clause entry and its ops costs, which agrees
   with the implementer's bisection. Two directions keep the clause path untouched, and neither was measured:
   * (a) Make staleness impossible while echoes are on. At the cold sites that change the setting (the TRACE
     instruction, `TRACE()`, debug entry and exit), leave the level and resume it under the chunk compiled for
     the new setting, at the next instruction index. This needs a pc map between the two chunks, and loop frames
     that survive the switch.
   * (b) For the non-echo lines, have the trace line functions return `Result`. Their gates are
     `#[inline(always)]` and the `Err` is produced only after the out-of-line line call. So the untraced path
     folds to `Ok(())`, and the failure propagates at once, as `traceOutput`'s exception does.
   
   (b) is the larger change, but it is the one that also covers `mid7`. Either way, this needs the controller's
   ruling under "correctness over performance". It should not ship as a footnote.

2. **The new yield rule hangs a program that the parent and the oracle finish.** In `st1.rex`, main starts a
   worker that runs `do i = 1 to 100; nop; end` and then sets `.local~flag = 1`. Main's destination LINEOUT (an
   `unguarded` method) waits with `do while .local~flag = 0; end` before writing the line.
   * The oracle prints `done 1`, rc 0, on 8 of 8 runs.
   * The parent prints `done 1` under wall and all nine sim modes.
   * HEAD hangs under wall and under all nine sim modes (`sim:1`, `sim:7`, `sim:42`, `uniform:0.2` at two seeds,
     `uniform:1`, `pct:3` at k 40 and 200, `pre:2`). Each run was killed at 10 s, rc 137.
   * gdb on the hung process shows the interp thread spinning in the destination's `.local~flag` lookup. The
     worker never ran.

   The cause is the rule itself. While `routing_trace` is set, `serve_requests` never takes the pinned yield. A
   delivery runs Rexx clauses, so a deferred slice is never taken until that delivery ends. A delivery that waits
   on another activity therefore never ends. Variants that block do terminate: a destination looping on
   `call syssleep 0.01`, or one that waits with `guard on when`. So this is confined to a destination that
   computes or spins during delivery. But it is a liveness regression, and the report's "a legal schedule" does
   not hold for an unbounded delivery. Suggested direction: narrow the suppression to the window in which the
   TraceObject is built under the class's guard (before the `LINEOUT` to the program's destination), and let the
   destination's own clauses yield as before. Then rerun `two1.rex` and `st1.rex` under the sim modes above.
   Witness `st1` in the sim gate.

   (Separately, and also on the parent: a crate process inside a trace delivery's busy loop ignores SIGTERM. In
   `t1.rex` the destination LINEOUT is `do forever; end`. `timeout` killed the `memcap` wrapper and left `rexx`
   running, on HEAD and on the parent. It is pre-existing, so it is a queue candidate. It is why a bare
   `memcap 2G timeout 20` does not bound `st1` on HEAD.)

3. **Every call and method send costs 15 to 19 more instructions, and `sendloop` is over budget.** In one
   callgrind run (`-r 3`, spread 0.0000) against the pin, the parent and HEAD:
   * `sendloop` 14595033646 → 14670038732, +0.51% against the parent and **+0.5836% against the pin, OVER the
     0.5 budget**. That is 75.0M instructions over 5M sends, 15 per send.
   * `dispatch`: +0.43% against the parent, 19 per send.
   * `fibcall`: +0.33% against the parent, about 17 per internal CALL.

   The bisection (`-r 1`) puts all of it on `e5f54ba1d`: `sendloop` c239 14720040215 → e5f5 14790038891, and
   `dispatch` 22260825550 → 22350824602. That commit moves the `<I<` and guard release ahead of the call-tail pop
   in `finish_call`. It adds a `call_tails.last()` lookup and, for methods, a `split_owed()` call (which runs
   `running_activation()` and reads `replied`) on every return. It also adds a check to every EXPOSE. The
   report's perf table covers `startup`, `interpret`, `parse`, `churn` and `sayloop`, none of which sends a
   message, so this was never seen. It needs a fix (for example, read the kind from the tail the pop already
   returns, and gate `split_owed` on a flag the REPLY sets), then a callgrind run that includes `sendloop`,
   `dispatch` and `fibcall`.

### Minor

4. **The cost of a Rexx-built TraceObject.** I ran `trace r` over a 10000-pass `x = i + 1` loop, which is 60002
   stderr lines, identical to the oracle's byte for byte.
   * Wall clock, median of 5: HEAD 1789 ms, parent 81 ms, oracle 513 ms.
   * With a Rexx destination: HEAD 2021 ms, parent 1958 ms, oracle 379 ms.
   * Callgrind at 1000 passes (6002 lines), startup subtracted: HEAD about 221k instructions per line, parent
     2.4k, oracle 52.7k. With a Rexx destination: HEAD 244k, parent 234k, oracle 40k.

   So the default route is about 3.5x the oracle's time and about 22x the parent's. About a third of HEAD's
   instructions are the recursive `ClassRegistry::is_a` under `native_hash_put`'s `class_is_one_of` (34.2%
   inclusive), and 18.7% are `frame_table`. This does not block the task, but it should be queued as a perf item
   with these numbers.
5. **An untrapped trace failure's error report goes to stdout.** On `probes/untrapped-trace-failure/p.rex`, HEAD
   writes the traceback and the `Error 97 running REXX line 1457` lines to stdout, and nothing to stderr. A
   report that cannot be delivered should go to the error stream. The oracle gives no comparison because it
   segfaults. Small; can be folded in or queued with Concern 3.
6. **The stability record predates the final code.** `t6.10/witnesses/stability.txt` was measured at `306331c9b`,
   before `7540e967f` and `a5266a662`. My runs at HEAD agree with it, but the record should name the commit it
   stands for.

## Assessment

**Task quality:** Good cluster work, with one correctness gap that is wider than reported, one liveness
regression, and one budget overrun.

**Reasoning:** The TraceObject, NOTIFY/VARIABLE, interactive-debug, prologue and REPLY fixes match the oracle on
every shape I constructed, in the sim modes as well. The deadlock the sim mode found is really gone. But a failed
delivery after an ordinary `TRACE R` still runs its clause, and so does a failed non-echo line in any body. The
yield rule that fixed the deadlock hangs a destination that waits on another thread, under every schedule. And
`e5f54ba1d` costs every call, unmeasured, past the budget.

**Verdict: CHANGES REQUESTED.** Findings 2 and 3 need fixes. Finding 1 needs either a fix or the controller's
ruling with the real cost (nop +4 to +6%), not the 0.5% figure. Findings 4 to 6 can be queued.
