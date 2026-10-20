# Task 6.13 re-review: fix round 4

Scoped re-review of `2434a77af..86f2a43b2`: code `8e365dcbe` and `41dd12f9c`, records `86f2a43b2`.

**Builds:**
* HEAD release `rexx`: exit 0.
* Parent: round 3's code, from a worktree at `2434a77af`.
* Round 0 and the pin, as before.

**How the probes ran:**
* Probes ran from `scratchpad/rev613r4-probes.Qq3N/`, each run in a fresh `mktemp -d`.
* The oracle ran under the standard wrapper; the crate under `memcap 2G timeout 20`.
* stdout, stderr and rc were compared separately.
* SIGINT was sent by PID at 0.6 s (`sigint.sh`).

The worktree and target dirs are deleted, and `pgrep -a sleep` and `pgrep -a rexx` are empty.

## Round-3 Importants: all closed

1. **Posts for an outer wait.** Oracle and head are identical on each, 2 of 2:
   * `spur`: `O:a slept>0.9 1|O:b slept>0.9 0|O:c slept>0.9 0|done 0`.
   * `sem`: `O:a waited>0.9 1|...`.
   * `guard`: `released|O:a|O:b|O:c|done 0 elapsed>0.9 1`.
2. **A halt at a nested wait** (head; the oracle gives 98.923 in place of 4.1, which is Deviation 12):
   * `hs6` (no trap): `O:b inner rc RC`, `8 *-*` / `2 *-*`, `Error 4.1`, rc 252. Round 3 gave `after -4`, rc 0.
   * `hs2` and `hs3`: the same shape at lines 10 and 3.
   * `hs4` (three deep): `O3:a rc RC|O2:n rc RC|O1:b rc RC`, traceback 13, 13, 3.
   * `hs1`: identical to the oracle.
3. **A non-SYNTAX condition under SIGNAL ON.** Identical to the oracle (`calls 1`): `ce3` (`O:a|signalled  rc 9 sigl 3 calls 1`), `su`, `snr`, `sfa`, `sany`, `sanyu`. `sanyz` (SYNTAX) is still `calls 2` on both sides.

## Sweep of the round-1 to round-3 probe sets (156 programs, 2 runs each side, plus 25 SIGINT programs)

* 124 are byte-identical to the oracle on all four runs.
* Every other one is a known shape, unchanged from earlier rounds:
  * entry 56 and the `CommandIOContext.cpp:131` family (`begin`, `errin`, `sh3e`, `sh3g`, `shin2`, `shin3`);
  * Deviation 51 (`rsi`);
  * Deviation 52 (`d52`, `d52e`, `rcun3`);
  * entry 58 (`rcun2`);
  * the oracle's `lineout` buffering (`buf`, `noclose`, `noclose2`);
  * `conc`, identical as a multiset;
  * the pre-existing `qpull` report lines.
* The round-1 library probes (`env_*`) match round 1's result when run with the oracle's library path. The three `hostemu` refusals are the Phase 10 refusal round 1 recorded.
* Every SIGINT program is identical to the oracle where Deviation 12 does not apply, and is Deviation 12's 4.1 shape where it does. `hA`–`hE`, `sigint`, `sigtok` and `sigtok2` are unchanged from round 3.

## The implementer's concerns

**Concern 1: upheld.**
* rexxref (`condtra.xml`, "Action Taken when a Condition Is Not Trapped"): "For HALT and NOMETHOD, a SYNTAX condition is raised", and SYNTAX propagates to each caller.
* So a halt served in a target's method, untrapped there, reaches main as SYNTAX 4.1, which main's SIGNAL ON HALT does not take.
* `q1` (`signal on halt` in main, a method sleeping) gives `9 *-* call SysSleep 3`, `3 *-* x = .M~new~go`, `Error 4 ... line 9`, rc 252 on both sides.
* My round-3 expectation that the trapped handler runs at every depth was wrong. The real defect there was the lost halt (rc 0), and that is closed.

**Concern 2 (`hs5`): upheld.**
* Head: `target HALT rc -4 fast 1|O:b inner rc RC|after 0`.
* Oracle: 98.923 untrapped.
* This is `hB`'s depth-1 shape one level down, under Deviation 12's sentence. No new candidate.

## Hunt of the design note's event/state pairs (oracle identical unless stated)

* **Not innermost, parked on a guard that then releases** (`deep2`): a nested command's target blocks on a guard another activity holds for 1.2 s while both waits' pieces arrive. Result: `released|T2:i|T2:j|T1:a|T1:b|T1:c|done 0`, 3 of 3.
* **The child ending while the target's method is mid-halt** (`hs7`): the target traps HALT and its handler sleeps 0.6 s while the outer child ends. Result: `tgt halt a|tgt done a|O:b|after 0`, identical.
* **The same, with the handler running its own redirected command** (`hs8`): head gives `tgt halt rc -4|tgt handler cmd 0 1 hh|O:b RC|after 0`. The oracle gives 98.923, which is Deviation 12.
* **REPLY from a target method:**
  * `reply2` (guarded `lineout` replies and then sleeps): `R:a|R:b|done 0 slow 1|R:c`.
  * `reply3` (unguarded, replies, then a redirected command with its own target): `done 0|Q:na|R:a 0|Q:nb|R:b 0`.
  * The round-3 `reply` and `start` probes also match.
* **SIGNAL out of a target, then `parked_block`:**
  * `sigdeep3`: SIGNAL ON ERROR in T1's method; T2, a nested command's target, raises ERROR. T1's handler runs a fresh redirected command while the nested child still runs. Result: `T1 signalled rc 9 sigl 9|T1 next 0 1 z|T1:b|...|done 0`, 3 of 3.
  * `sigdeep4`: the handler sleeps 1.5 s while the abandoned nested child's posts and the outer pieces arrive. Result: `T1 signalled|T1 slept|T1:b|T1:c|done 0`, 3 of 3.
  * `sigmid`: main's SIGNAL ON ERROR runs a second redirected command whose target sleeps while the first child's posts arrive. Identical.
* **Program end with a nested wait open** (`endmain`, `endmain2`): identical.

## Issues

### Critical

None.

### Important

None.

### Minor

1. **A queue candidate outside this round's change: the "running line" of a HALT re-raised through `~result`.** `hs9`: a halt while a target waits on a started message's `result`.
   * Oracle: `Error 4 running ... line 9`, then `line 1`.
   * Head: `line 15`, then `line 9`. Both sides show the same traceback lines and `O:b`.
   * This is the same pattern as `qpull`'s deadlock report, which the pin shares with head (`qpull` is pre-existing).
   * For `hs9` itself, the pin and round 0 differ from the oracle in other ways: no `O:b`, and a single report.
   * Append it to the error-report-line queue item rather than fix it here.

## Tests (HEAD, release, exit status read unpiped)

* `cargo test -p rexx-exec --lib`: exit 0, 1157 passed.
* `--no-fail-fast` over `licensed_divergences` (22), `concurrency_tests` (54), `rexx_command`, `signals` (40), `stdin_contention`, `sim_processes`, `program_end` and `refusal_sites`: exit 0, every binary ok.
* `REXX_CORPUS_GATE=1 ... --test corpus`: exit 0, `1146 of 1146 matching`.
* I did not re-run the whole suite.

**Verdict: APPROVED.** The Minor is a queue candidate for the controller.
