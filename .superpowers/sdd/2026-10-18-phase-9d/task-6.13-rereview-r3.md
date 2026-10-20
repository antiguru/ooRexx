# Task 6.13 re-review: fix round 3

Scoped re-review of `cfbeefbed..ead293ab0`: code `518613a71`, records `ead293ab0`. Tree HEAD is `f50e37ca3`, which adds a queue item only.

**Builds:**
* HEAD release `rexx`: exit 0.
* Parent: round 2's code, from a worktree at `cfbeefbed`.
* Round 0: `ooRexx-p9-pin/cg-9d/t6.13/rexx-head`.
* The pin: `ooRexx-p9-pin/rexx-run`.

**How the probes ran:**
* Probes ran from `scratchpad/rev613r3-probes.a9nZ/`, each run in a fresh `mktemp -d`.
* The oracle ran under the standard wrapper; the crate under `memcap 2G timeout 20`.
* stdout, stderr and rc were compared separately.
* SIGINT probes used `halt/sigint.sh` (SIGINT by PID at 0.6 s).
* I did not run the `ra() + ra()` shape on the oracle.

The worktree and target dirs are deleted, and `pgrep -a sleep` and `pgrep -a rexx` are empty.

## Round-2 findings

* **Important 1 (a target that parks): closed for sleeps that run to their end.** `w1`, `w2`, `w4`, `w5`, `w6`, `w9`, `tick2`, `tick2a`, `tick2b`, `tick2c` and `lost` are identical to the oracle in 2 of 2. For example, `lost` gives `O:a|O:b|O:c|done 0 calls 3`. But the mechanism that closes them cuts *other* waits short; see Important 1 below.
* **Important 2 (a held condition outranks a halt): closed for the depth-1 shapes,** following Deviation 12 as ruled:
  * `hA`: `Error 4.1` at line 1, rc 252.
  * `hB`: `O:a|HALT handler rc -4 line 2`.
  * `hC`: `O:a|after -4|HALT handler rc -4 line 3`.
  * `hD`: like `hB`.
  * `hE`: unchanged.
  * `sigint`: `HALT rc -4 stem S.0 S.1 line 4`.

  The oracle gives 98.923 on each, which Deviation 12 licenses. Nested waits lose the halt; see Important 2 below.
* **Minor 3: closed.** `carry` gives `O:a|O:d|O:e|trapped 42.3 calls 3` on both sides, and `lang/command_redirected_target_raises_part_line.rex` is its witness.
* **Minor 4: closed.** Both doc comments are corrected, and the token check stays.
* **Still identical:** `two1`, `order`, `shin1`, `raise1`, `raise2`. `conc` is identical as a multiset.

## (a) The stack of waits

**Identical to the oracle (2 of 2):**
* `n2`: two nested redirected commands in one target method, run for each of three outer lines.
* `n3`: a target whose nested command has its own sleeping USING target: `O:ia|O:ka|O:a|O:ib|O:kb|O:b|done 0 calls 2 4`.
* `rec`: a `::routine` re-entering the same command clause three deep through the target.
* `rec2`: the same clause shape in an internal routine and a `::routine`.
* `reply` (REPLY from the target, then a redirected command): `O:a|replied a 1 latea|O:b|done 0|replied b 1 lateb`.
* `start` (START from the target).
* `endmain` (main ends while a worker's nested wait is open) and `endmain2` (main ends on SYNTAX): the worker finishes.
* `hs1` (the target in SysSleep when the halt arrives): oracle and head both give `9 *-* call SysSleep 2` / `Error 4.1`.
* `hs5` (the target traps HALT itself): `target HALT rc -4 fast 1|after -4`. That is Deviation 12's shape.

**Different from the oracle:** Importants 1 and 2.

## (b) A command's ERROR/FAILURE while its CALL ON trap is delayed

rexxref (`condtra.xml`):
* Under ERROR: "raised at the end of the clause that called the command but is ignored if the ERROR condition trap is already in the delayed state". FAILURE has the same wording.
* Under CALL ON: the delayed state "persists until the RETURN from the CALL ... if the condition is raised again, it is either ignored (for ERROR and FAILURE) or ... delayed".

So the rule follows rexxref. Probes, identical to the oracle in 2 of 2:
* `ce1` (`'exit 3'` and `'exit 4'` in separate clauses): the handler runs twice.
* `ce12`: two command clauses on one line.
* `ce11`: three commands in a DO loop.
* `ce10`: the commands in `call g` twice.
* `ce6`: two command clauses in one INTERPRET.
* `ce5`: a command erroring inside the handler. Head gives `handler after rc 6`, with no second handler run, which is the "persists until the RETURN" half.
* `ce2` (the target raises ERROR, the command exits 0, then a separate `'exit 4'`): `handler  rc 9 sigl 3|after 9 0|handler exit 4 rc 4 sigl 5|end 4`.
* `ce7` (FAILURE: the target raises FAILURE, then an unknown command): the handler runs once each.

`rcun2` is the oracle's entry-58 SIGSEGV (2 of 2). Head gives rexxref's answer. SIGNAL ON instead of CALL ON is Important 3.

## Issues

### Critical

None.

### Important

1. **A piece or end of the outer command's output, arriving while the target's method waits on something else, ends that wait early.** The SysSleep, semaphore and guard cases are new in round 3. `Posted::Captured` and `Posted::Unblocked` now find their wait anywhere in the stack and call `unpark(activity)`, which runs `guards.woken` and `make_ready` whatever the activity is parked on.
   * `spur`: the target does `call SysSleep 1` on line `a`, and the child writes `b` at 0.2 s.
     * Oracle, round 0 and the pin: `O:a slept>0.9 1`.
     * Head: `O:a slept>0.9 0`. The sleep is cut short.
   * `sem`: the target waits on an EventSemaphore posted at 1 s.
     * Oracle and the pin: `O:a waited>0.9 1`.
     * Head: `waited>0.9 0`. Round 0 is also wrong here.
   * `guard`: the target object's guard is held for 1 s by `o~start('hold')`. **Mutual exclusion is broken:**
     * Oracle, round 0 and the pin: `released|O:a|O:b|O:c|done 0 elapsed>0.9 1`.
     * Head: `O:a|O:b|O:c|done 0 elapsed>0.9 0|released`. The guarded `lineout` ran while another activity held the guard.
   * In each case the parent waits correctly and then hits round 2's refusal.
   * Fix: unpark only when the activity is parked on that wait, which is the innermost one, or make every other park re-check its own condition when woken.

2. **A halt arriving while a target's method waits on its own nested command is lost.**
   * `hs6` (no trap anywhere; a nested `address system 'sleep 2; ...'` in the target; SIGINT at 0.6 s):
     * Oracle: `98.923`, rc 158.
     * Head: `after -4`, rc 0. The program runs on as though Ctrl-C had not happened.
   * `hs2` (`signal on halt` in main): head `after -4`. The handler never runs.
   * `hs3` (`call on halt`): head `after -4 fast 1`, with no handler line.
   * `hs4` (three deep): head `O3:a rc RC|O1:a rc -4|after -4`, with no handler.

   Likely cause: the inner clause raises the HALT in the target's method. It leaves the method as a SYNTAX 4.1, not a HALT, so the outer clause's `halted && !is_halt(..)` drops it with `clear_failure_levels`, and nothing raises 4.1 at the outer clause. Deviation 12's observable is 4.1 at the clause, so the trapped handler must run.

3. **A target's ERROR, FAILURE, USER or NOTREADY caught by a SIGNAL ON trap is held for the child. The oracle unwinds at once. Introduced by round 2; I missed it then.**
   * `ce3` (`signal on error`; the target does `raise error 9`; the command exits 5) and `ce3b` (exits 0):
     * Oracle, round 0 and the pin: `O:a|signalled  rc 9 sigl 3 calls 1`.
     * Head and the parent: `O:a|O:b|signalled ... calls 2`.
   * The same split holds for `su` (SIGNAL ON USER), `snr` (NOTREADY), `sfa` (FAILURE), `sany` (ANY with ERROR) and `sanyu` (ANY with USER). The oracle gives `calls 1` in 2 of 2; head and the parent give `calls 2`.
   * Only SYNTAX is held by the oracle (`sanyz` and `raise1`: `calls 2` on both sides).
   * Under CALL ON, both sides agree (`cue`, `cnr`, `ce2`).
   * rexxref is silent on the timing, so the round's rule says follow the oracle: hold only SYNTAX, and let any other condition a SIGNAL ON trap catches end the clause at once.
   * This puts back an exit from the wait while the child runs, so `Blocked::end`'s token check becomes reachable again. Witness that too.

### Minor

None new. `qpull`'s deadlock report names different lines (`line 18`/`line 14` against the oracle's `14`/`3`), but the pin gives the same as head, so it is pre-existing and was a faulty probe on my side. Not a finding.

## Tests (HEAD, release, exit status read unpiped)

* `cargo test -p rexx-exec --lib`: exit 0, 1157 passed.
* `--no-fail-fast` over `licensed_divergences`, `concurrency_tests`, `rexx_command`, `signals`, `stdin_contention`, `sim_processes`, `program_end` and `refusal_sites`: exit 0, every binary ok (`signals` 39 passed).
* `REXX_CORPUS_GATE=1 ... --test corpus`: exit 0, `1143 of 1143 matching`.
* I did not re-run the whole suite.

**Verdict: CHANGES REQUESTED.** Important 1 is a round-3 regression. Important 2 is a round-3 gap in the Deviation 12 ruling. Important 3 is a round-2 divergence from the oracle's timing.
