# Task 6.13 re-review: fix round 2

Scoped re-review of `7afba3f6d..5e2d0c2f9`: code `54958edd3`, records `5e2d0c2f9`. Tree HEAD is `1bb933348`, which adds records only.

**Builds:**
* HEAD release `rexx`: exit 0.
* Parent: a worktree at `7afba3f6d`, the round-1 code.
* Two HEAD mutants built in a scratch worktree, one at a time.
* Round 0: `ooRexx-p9-pin/cg-9d/t6.13/rexx-head`.
* The pin: `ooRexx-p9-pin/rexx-run`.

**How the probes ran:**
* Probes ran from `scratchpad/rev613r2-probes.K4rO/`, each run in a fresh `mktemp -d`.
* The oracle ran under the standard wrapper; the crate under `memcap 2G timeout 20`.
* stdout, stderr and rc were compared separately, and concurrent output as multisets.
* SIGINT probes used the records' `halt/sigint.sh`, which signals by PID.

Worktrees and target dirs are deleted, and `pgrep -a sleep` and `pgrep -a rexx` are empty.

## Round-1 Important 1: closed

* `raise1.rex`, 3 of 3 on each side. Oracle and head: `lineout b|trapped 42.3 after child 1 calls 2 rc 42|later calls 2|next 0 2 c d`. Parent: `trapped 42.3 after child 0 calls 1`.
* `raise2.rex` (ticking second activity): identical to the oracle, 3 of 3.
* `sigint.rex` no longer gives the round-1 answer, but it now hits a different defect. See Important 2.

## The round's rules, against the oracle (2 runs each; identical unless stated)

**Later lines are offered, and the clause parks until the child ends:**
* `tick` (a 1.6 s child with a ticking activity): `trapped 42.3 after 1 ticks>=4 1 calls 2` / `ticks 8`.
* `conc` (two activities, each with a raising target): identical as a multiset.

**The last condition is raised:**
* `two1`, output raising 42.3 and then error raising 41.1: `O:a|O:b|E:e1|trapped 41.1 sigl 3 calls 2 1 rs .RS`.
* `two2`, error first in time and output raising later: `trapped 41.1`. The error target is written at the end on both sides.
* `order` (timed writes on both streams): `O:o1|O:o2|E:e1|E:e2`.

**A raising line drops the rest of its piece:**
* `errtwice`: `E:e1|trapped 42.3 calls 1 o 1 o`.
* `carry` (`printf "a\nb\nc"; sleep; printf "d\ne\n"`): `O:a|O:d|O:e|trapped 42.3 calls 3`. The part line `c` is not glued onto `d`.

**RC and `.RS` are not set; the condition is raised again at the command's line:**
* `rcun4` (an earlier `exit 7`, then the command): `trapped 42.3 rs 1 sigl 4 pos 4 ctx 1`.
* `rcun5` (CALL ON USER): `handler USER BOOM rc 0 rs 0 sigl 4|after 0 0`.
* `rcun6`: `in z rc 42`.
* `untr` traceback: `19 *-* return 1 / 0`, then `3 *-* address system ...`, then `Error 42 running DIR/untr.rex line 3`.
* `trc` (TRACE C): the same echo on both sides.
* `nr` (NOTREADY via CALL ON): identical.

**No part-line flush and no deferred output:**
* `noflush`: `o 1 a O.2`.
* `noflush2`: identical. The piece split varies between oracle runs.
* `shin1` (input stem = output stem, error raises): `s 2 i1 i2 S.3`. The deferred output is not written.
* `shin4` (an Array as both input and APPEND output): `a 4 i1|i2|i1|i2`.

**Deviation 52 (licensed):**
* `d52`: oracle `ran on 3 calls 2`, head `trapped 42.3 calls 2 rc 42`.
* `d52e` (the error target raises, then exit 3): oracle `ran on 3`, head `trapped 42.3`.

**Input stem shared with the error stem:**
* `shin3` and `sh3g` (input stem `s.` also the `error append stem s.`): the oracle stops offering output lines after the raise (`O:i1|trapped 42.3 ... calls 1`). Head keeps offering them (`O:i1|O:more|... calls 2`).
* With the error stem unshared (`sh3f`) or with no raise (`sh3n`), the two sides agree.
* This is an oracle defect, not to be copied (Minor 5): `CommandIOContext::resolveConflicts`, `CommandIOContext.cpp:131`. When the *error* target conflicts with the input, it does `output = new BufferingOutputTarget(output);` where `error` was meant. So the output target is buffered until cleanup, where its first raise drops the rest.

## Issues

### Critical

None.

### Important

1. **An output USING target whose `lineout` parks loses the command's later output, or hangs the clause. A regression since round 1, still open.**

   Shapes:
   * `w1`: `address system 'echo a' with output using (o)`, where `lineout` does `call SysSleep 0.1`.
     * Oracle: `O:a|done 0 calls 1`, rc 0.
     * Head and parent: `O:a|rexx-exec: a wait that nothing left to run can end is not implemented`, rc 120.
   * Every other variant whose `lineout` parks fails the same way:
     * `w2` and `w9`: two lines.
     * `w4` and `w5`: the target runs `address system 'true'`.
     * `w6`: a nested redirected command.
     * `tick2`, `tick2a`, `tick2b`, `tick2c`: with or without a raise or a ticking activity.
   * `w3` (an error target) and `w7`/`w8` (`SysSleep 0`) are fine.
   * With no hang, lines are lost silently. `lost`: `'echo a; sleep 0.2; echo b; sleep 1.5; echo c'`, with `lineout` sleeping 0.6 s on `a`.
     * Oracle, round 0 and the pin: `O:a|O:b|O:c|done 0 calls 3`.
     * Head and parent: `O:a|O:c|done 0 calls 2`, rc 0.

   **Cause:** `end_blocked_command` takes `self.activity.blocked` out of the record while the target writes run, and puts it back only afterwards. A `Posted::Captured` or `Posted::Unblocked` that arrives while the target is parked therefore finds no `blocked` (`scheduler.rs:1538`, `:1564`) and is dropped. For an end, `in_flight` is decremented too, so the wake is lost and the clause later parks with nothing to end it. Round 0 and the pin run every shape. The fix is to keep the record's `Blocked` reachable during the writes, or to queue posts for it.

2. **A condition held when a halt arrives outranks the halt. The HALT is not raised at the clause, and a trapped HALT handler never runs.**

   Deviation 12's licensed observable is "raises 4.1 at the same clause" in place of the oracle's 98.923. Under the round's own rule the last condition wins, and the interruption is the last event (the oracle raises 98.923, not the target's 42.3). The probes send SIGINT at 0.6 s into `'echo a; sleep 1.13; echo b' with output using (.Bad~new)`, where the target raises on `a`:
   * `hB` (`signal on halt`):
     * Oracle: `98.923 ... rc 158`.
     * Head: `O:a`, then `10 *-* return 1 / 0` / `Error 42.3`, rc 214. The HALT handler never runs.
     * `hE` is the same program with the target not raising. Head gives `HALT handler rc -4 line 2`, which is Deviation 12's shape.
   * `hA` (untrapped), `hC` (`call on halt`) and `hD` (error stem added): the same, head 42.3, rc 214.
   * `sigint.rex` (`signal on syntax` and `signal on halt`):
     * Parent: `trapped SYNTAX 42.3|HALT rc -4 stem 1 x line 8`.
     * Head: `0 *-* <no failing clause recorded>|Error 0 running ... <no message 0.0 in the catalogue>`, rc 0. That is the queued HALT-at-a-SIGNAL-label defect (Task 8). The report says that defect needs no command to reach it, which is true. But this round is what sends an ordinary command shape there: the held SYNTAX is trapped and the pending HALT is taken at the label.

   When `abandon` ends a wait that holds a condition, the clause should answer as Deviation 12 does (4.1) and drop the held condition. Otherwise the held-wins shape goes to Moritz as a Deviation 12 amendment.

### Minor

3. **The "a raising line drops its piece's part line" half of the rule has no witness.**
   * A mutant without `self.carry.borrow_mut().clear()` (`redirect.rs:271`) gives `carry`: `O:a|O:cd|O:e`, where head and the oracle give `O:a|O:d|O:e`.
   * The mutant passes both `lang/command_redirected_target_raises_held.rex` and `lang/command_redirected_target_raises.rex` (run directly): in the held witness the part line follows the raising line in the *last* piece, where the held raise skips the flush anyway.
   * Witness: a part line followed by a later piece, as `carry` does.

4. **Concern 3: `Blocked::end`'s token check is reachable by no program I could build. Two doc comments are now false.**
   * Halts are filtered before `end` is called: `abandon` pushes the token onto `abandoned_blocks`, and the `Unblocked` arm drops it (`scheduler.rs:1556`). A token-check mutant (`if false && ...`) matches head on `sigtok` and `sigtok2`, where a second redirected command runs in the HALT handler while the first child is still running: `HALT rc -4 stem 1 a|second rc 0 t 1 x T.2`.
   * The other exit is a non-Raised failure from a target write in `end_blocked_command`. That covers a Loud, an Exited or a Deadline, and each ends the run. `tokloud` puts a Loud in a started activity: rc 120 for the whole program, and SIGNAL ON ANY does not trap it.
   * That exit also neither pushes onto `abandoned_blocks` nor decrements `in_flight`, which is harmless only because the run ends.
   * The check is one compare and guards the same mechanism Important 1 must rework, so keep it as a defence rather than delete it.
   * Two doc comments to correct:
     * `Blocked::end`: "A clause whose target write raised has left its wait" is no longer true.
     * `Blocked::raised`: "The first condition a target raised". The code holds the last one, and the round's rule says last.

5. **Oracle items, none to copy:**
   * Entry 56 reproduces: rc 139 on 3 of 3, nothing written, and the probe text equals the entry's.
   * Entry 57 reproduces: an indefinite block, rc 124, in 2 of 6 runs (the entry says 3 of 6), with none or only the first of the results. Head gives the four results and `parallel 1` in 3 of 3.
   * The root cause of entry 56 and of the `[S.]`/`[U.]` answers is the `CommandIOContext.cpp:131` line above. With no output target it wraps NULL. That belongs in entry 56's text.
   * New candidate (`rcun2`): an output target whose `lineout` does `raise error 9`, under `call on error`, with the command exiting 5. The oracle prints the command's ERROR handler line and then segfaults, rc 139, 2 of 2. Head runs the handler twice (the target's ERROR, then the command's), rc 0.
   * Pre-existing and outside this cluster (`buf`, `noclose`): the oracle buffers `lineout` to a named stream until close, so a file read mid-program is empty. The crate writes through. A queue candidate only.

## Tests (HEAD, release, exit status read unpiped)

* `cargo test -p rexx-exec --lib`: exit 0, 1155 passed.
* `--no-fail-fast` over `licensed_divergences`, `concurrency_tests`, `rexx_command`, `signals`, `stdin_contention`, `sim_processes`, `program_end` and `refusal_sites`: exit 0, every binary ok (`licensed_divergences` 22 passed).
* `REXX_CORPUS_GATE=1 ... --test corpus`: exit 0, `1141 of 1141 matching`.
* I did not re-run the whole suite.

**Verdict: CHANGES REQUESTED.** Important 1 is a regression since round 1. Important 2: fix it, or take it to Moritz as a Deviation 12 amendment. Minors 3 and 4 go with the fix; Minor 5 is for the controller.
