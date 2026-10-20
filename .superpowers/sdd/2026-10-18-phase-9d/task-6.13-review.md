# Task 6.13 review: commands

Reviewed `65a6d0e45..b5d01d7a8` (`0a934624f` code, `b5d01d7a8` records). I built HEAD into the scratchpad (`rev613-target`, `cargo build --release --bin rexx`, exit 0). Every probe ran from `scratchpad/rev613-probes.RckE/`: the oracle under the standard wrapper, the crate under `memcap 2G timeout 20`, comparing stdout, stderr and rc separately. The parent binary is `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.12-r2/rexx-head` (code `71369fa96`, as the report says). One mutant was built in a scratch worktree. The scratchpad is not durable, so the deciding lines are quoted below. Worktree and both target dirs are deleted, and no child process is left running (`pgrep -a rexx`, `pgrep -a sleep`: empty).

## Spec compliance

Partly compliant. All three clusters' tests pass, and the 9d set is empty. The suite run shows no other changed row. But the hostemu refusal is wider than the brief's "at the point where the `hostemu` environment is missing": it also refuses environments that no loader registers (Important 1). Two behaviours newly reachable through the off-baton path differ from the oracle and were not brought forward (Important 2 and 3).

## Strengths

* **The suite claim reproduces.** `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=<scratch> ... --test suite l3_core_suite`: exit 0, `402 green` in `verdicts.tsv`, 254.6 s. My `tests.tsv` is byte-identical (sorted) to the committed `t6.13/suite/tests.tsv`. `SIGNAL TEST_8 - passed passed`, `TEST_SIGNAL_ANY_CONDITION_FAILURE - passed passed`, `TEST_SIGNAL_ANY_CONDITION_HALT Phase10(Extension) passed blocked: rexx-exec: ADDRESS environment "hostemu" a native package loader registers is not implemented (Phase 10 Extension)`. The new `phase-9-exclusions.txt` row matches that captured refusal text exactly, and it sits in C order (`ADDRESS` < `SIGNAL` < `base/rexxutil`). `suite-9d-set.txt` has no non-comment line.
* **The redirected off-baton path matches the oracle on every shape where the oracle's timing is not observable.** Each case below was run 2 or 3 times per side, with identical output:
  * RC, `.rs`, FAILURE (`call on`) and ERROR (`signal on`) in a started activity, with main ticking meanwhile. Both sides: `main 1`, `main 2`, `FAIL handler 127 ... FAILURE`, `after failure rc 127 -1 1 E 0`, `main 3`, `ERR handler 3 ... 1 O 1`, and the same `+++ "RC(127)"` trace.
  * Targets: a stream name, a Stream object, a Queue, `.output`, the same stem for input and output (`same stem 2 x y`), and `.nil` (`SYNTAX 98.996 Object "The NIL object" is not a valid ADDRESS WITH OUTPUT or ERROR target.`).
  * Four started activities running `sleep 0.5; cat; echo errK >&2` with input, output and error stems. `parallel 1` on both, each line `k 0 3 k 2k 3k 1 errk 0`.
  * Input stem changed by main while the child sleeps: `2 a b I.3` on both. The input is taken at the start on both sides.
  * The input-reader SYNTAX across a park (`.Bad` InputStream in a started activity): `trapped 41.1 1 first 41 8` on both.
  * Main exits while the child runs (`thread 0 1 late`, rc 3), and a command after REPLY (`after reply 0 1 r`).
* **The new rooting is load-bearing, and a probe can see it.** `gc.rex` gives a started activity's command two `Collect subclass Array` targets that only the `IoContext` holds, while main churns allocations. HEAD and the oracle print `out a`, `out b`, `err c`, `0 1`. A mutant with `blocked.object_roots(out)` removed from `Activity::object_roots` gives `rexx-exec: a message send to a value whose object is no longer live is not implemented`, rc 120, in 3 of 3 runs.
* **The simulation-mode refusal is deliberate and predates the task.** `REXX_SWITCH_MODE=sim:1` on the witness gives `a command in the simulation mode that waits longer than its bound is not implemented`, rc 120, on HEAD and on the parent alike. The ticking probe (`conds.rex`) runs to completion in sim mode on both. `collect_on_baton`'s doc states the design: the pool bound is 0, so a wait past `block=` is abandoned, its group killed, and refused. `phase-6-1-gate.md:864` records the same `block=` refusal design for native calls. `refusal-sites.tsv:309` lists it. This is not a new hole.
* **`.context~rs` matches on every route except INTERPRET.** The routes probed were: an unregistered environment (`-1 -1 30`), the default environment, a `call on error` handler (`in handler 1 1`, `handler after 0 0`, caller `1 1`), a SYNTAX from an invalid target (rs unchanged, rc 98), `exit 7` from an external `::routine` (rs unchanged), a started method (`The NIL object 1 1`), a `::routine` (`The NIL object .RS`), and a dead internal-routine context (`98.981 Target RexxContext is no longer active.`). stdout and stderr are identical on both sides. Only commands set the status, as `RexxActivation::setReturnStatus` has one caller (`RexxActivation.cpp:4471`).
* **Concern 1 is confirmed.** `rsi.rex` on the oracle: `in interp, none before The NIL object 1`, `in interp 0 1`, `in interp2 1 0 1`. On HEAD: `1 1`, `1 1`, `0 0 0`. Outside the INTERPRET the two sides agree (`after interp 1 1`). The documentation, `utilityclasses.xml` (rs method): "Returns the context current return status value. If no host commands have been issued in the current context, .nil is returned. This is equivalent to using the .RS environment symbol." The crate follows that. The oracle's `.context~rs` inside an INTERPRET disagrees with its own `.rs`. This is a Deviation candidate for Moritz, as the report says. Separately, `xintdeb.xml:97` says commands run from interactive-debug input do not set RC or `.RS`. That is for the debugger phase (11.1) and is not reachable here.
* **Concern 3 is confirmed, with one correction.** HEAD gives `test result: FAILED. 51 passed; 1 failed` in `concurrency_tests`. The failure is `the_table_holds_the_parts_the_spec_names`. The `have`/`want` diff holds exactly five programs on the `have` side only: `trace_reply_continuation`, `trace_destinations_two_threads`, `trace_destination_waits_on_thread`, `trace_destination_waits_calling_a_method` and `trace_destination_waits_calling_a_label`. All five are in `sim-gate.tsv` and `phase-9.txt` but not in `phase-6.txt`, which the test's `want` reads (`concurrency_tests.rs:5117`). The correction: the first of these rows came in at `f7eacd9b3` (Task 6.10 initial), then two more at `85b3ddd06` (round 1) and two at `24da8a29b` (round 2). By that reading the test has been red since Task 6.10's first commit, not since round 2. I derived this from the rows and did not run it at `f7eacd9b3`.
* **Queued item not closed, confirmed.** `bgcmd.rex` on HEAD under `timeout 3`: `Error 4.1: Program interrupted with HALT condition.`, rc 124. No leftover process.
* **Perf reproduces.** `callgrind.sh -r 1 -B 0.5 -p "rexxcps startup sayloop"`, exit 0. rexxcps: parent -libc +0.5193%, head -libc +0.5318%, so the task's own delta is +0.0125% (the report gives +0.0126). startup's own delta is +0.0004% and sayloop's +0.0003%.
* **Tests.** `cargo test -p rexx-exec --lib`: exit 0, 1155 passed. `--no-fail-fast --test rexx_command --test signals --test stdin_contention --test sim_processes --test program_end`: all ok (1, 37, 1, 4, 8 passed).

## Issues

### Critical

None.

### Important

1. **The refusal fires for any package whose loader ran, including rxsock and rxunixsys, which register no environment. This is a regression from rc 30.** `Libraries::loader_named` checks `has_hook(Loader)` and a matching package name. rxsock (`rxsock.cpp:703`, loader `loader`, only `WSAStartup` on Windows) and rxunixsys (`rxunixsys.cpp:1740`) both have loaders. Probe `address rxsock 'hello'; say 'rc' rc .rs` with `::requires "rxsock" LIBRARY`:
   * oracle: `+++ "RC(30)"`, `rc 30 -1`, rc 0
   * parent: `rc 30 -1`, rc 0
   * HEAD: `rexx-exec: ADDRESS environment "RXSOCK" a native package loader registers is not implemented (Phase 10 Extension)`, rc 120

   `address RXSOCK`, `address rxunixsys` and `address RXUNIXSYS` behave the same way. `address xyzzy` beside each library stays rc 30, and so does `address rxmath` (rxmath has no loader). Concern 2's own enumeration names the class correctly: hostemu is "the one non-Windows extension calling `RexxRegisterSubcom*`". But the rule keys on "has a loader", which is a wider set, so the report's claim that unregistered names keep rc 30 is false for these two packages. No suite test reaches them, which is why the suite stays green. Fix: key the refusal on the packages whose loader registers a subcommand environment, which is the set Concern 2's grep derives, rather than on any loader. Add a witness or probe line for `address rxsock` after its `::requires` (rc 30 on both sides).

2. **A halt during a redirected command drops the lines the command already wrote, and the HALT-versus-98.923 difference was not brought forward (Concern 4).** In `sigint.rex`, `address system 'echo first; sleep 2.13; echo late' with output stem s.` is preceded by `s.0 = 5; s.1 = 'old'`, with `signal on halt; signal on syntax`. The process gets `kill -INT` after 0.6 s:
   * oracle: `HALT rc 98 elapsed>1.5 0 stem 1 first line 15`. The read fails with EINTR, raising `98.923 Address command redirection failed (Interrupted system call)` (seen untrapped in the first run). Its SYNTAX trap fires, and the pending HALT is taken at the handler's first clause. `first` is already in the stem, because `OutputRedirector::writeBuffer` writes as data arrives.
   * HEAD: `HALT rc -4 elapsed>1.5 0 stem 0 S.1 line 6`
   * parent: `HALT rc 0 elapsed>1.5 1 stem 2 first line 6` (it waited out the child on the baton)

   Two separate points:
   * (a) `Blocked::abandon` sets `out` and `err` to empty, so `first`, which the pool thread had already received, is lost and the stem is emptied. The documented rule (`instrc.xml`, WITH OUTPUT: "the number of output lines is stored in stem.0 and individual lines are stored as stem.i") gives no reason to drop lines the command produced. The oracle keeps them, and so did the parent. Fix: hand what was collected so far to the targets on abandonment.
   * (b) HALT with rc -4 where the oracle raises 98.923 then HALT. HALT is arguably the documented answer, and the oracle's EINTR leak is an oracle defect. Deviation 10's `INTERRUPTED` text covers an interrupted `waitpid` for the unredirected wait, not this redirected read. Under Ruled 13 this is a new divergence candidate that goes to Moritz before it is written. The report's "Not compared with the oracle" left it unmeasured.

   A message halt (`m~halt` from main) does not abandon the command on either side. Both wait the child out and print `HALT rc 0 elapsed>1.5 1 line 14` / `stem 2 first arr 0`.

3. **Other activities now observe redirect targets as stale for the whole command. The oracle empties them first and fills them as output arrives.** This was not observable before the task, because the baton was held. `observe.rex`: a started activity runs `'echo one; sleep 1; echo two' with output stem s. error using (a)` on a stem and an array shared with main, and main reads them at 0.5 s:
   * oracle (2 of 2): `mid stem 1 one arr 0`, `end stem 2 one two arr 0`
   * HEAD (2 of 2): `mid stem 7 old arr 2`, `end stem 2 one two arr 0`

   The oracle calls `ioContext->init()` before the handler runs (`CommandHandler.cpp:140`). For REPLACE, `StemOutputTarget::init` empties the stem and sets `stem.0` to 0, and `CollectionOutputTarget::init` sends `empty`. Then each `WriteOutputBuffer` takes the lock and writes lines as they arrive. HEAD does both at `command_ended`. rexxref describes the ordering ("emptied ... before any output lines are added", "truncated to zero length before any output lines are written") but not when these happen relative to other activities. So this is either matched (at least the REPLACE emptying at the start, which is cheap) or taken to Moritz as a Deviation candidate ("targets are written when the command ends"). It should not be left unrecorded. The witness cannot see this, because its main never reads the target.

### Minor

4. **The rooting fix has no committed witness.** The mutant above, with the parked targets unrooted, would pass both new corpus witnesses, which hold their target in a variable (`out.`) or in an unparked `address ""`. `gc.rex` (quoted above: an `IoContext`-only subclass target plus allocation churn in main) catches it 3 of 3. It would make a cheap `phase-9.txt` witness.
5. **Pre-existing, outside the clusters: an `ADDRESS env command` clause is not traced under TRACE C.** `trace c; address system 'echo a' with output stem o.; address system 'echo b'`: the oracle prints `2 *-* address system 'echo a' with output stem o.` before `>>> "echo a"`, while HEAD and the parent print only the `>>>` lines. A bare `'echo b'` clause is traced correctly on both. This is a queue candidate.
6. **Perf note.** The report's "every whole-process figure against the pin is under +0.5%" is true of the raw column only. rexxcps -libc is +0.5318% against the pin (parent +0.5193%). The task's own share is +0.0125%, but the cumulative -libc drift on rexxcps is past 0.5%, and that is worth one line in the controller's ledger.
7. **Stale gate prose.** `phase-6-gate.md:472` and P64 still describe the off-baton path as "an `ADDRESS` command with nothing redirected". That is a closed phase's record, so a pointer in the next gate is enough.

## Assessment

**Task quality:** The off-baton change itself is sound: conditions, RC, `.rs`, input SYNTAX, GC rooting, concurrency and program end all match. `.context~rs` is right outside INTERPRET, and the suite claim holds. But the hostemu rule refuses two real extensions that answered rc 30 before, and the redirected wait opens two oracle differences (halt output loss, target visibility) that went unmeasured instead of being fixed or taken to Moritz.

**Verdict: CHANGES REQUESTED** (Important 1, 2a; 2b and 3 are either fixed or taken to Moritz as Deviation candidates. Minor 4 can be folded in; 5 to 7 can be queued or noted.)
