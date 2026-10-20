# Task 6.13 re-review: fix round 1

Scoped re-review of `95ac46b37..deea45edb` (code `bdec52856`, `7e42bba78`; records `deea45edb`). I built HEAD into the scratchpad (`rev613r1-target`, exit 0), and one mutant in a scratch worktree. Probes ran from `scratchpad/rev613-probes.RckE/` with the standard oracle wrapper and the crate under `memcap 2G timeout 20`. stdout, stderr and rc were compared separately, and concurrent output as multisets. The round-0 binary is `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.13/rexx-head`. Worktree and target dirs are deleted, and `pgrep -a rexx` and `pgrep -a sleep` are empty.

## Round-0 findings

* **Important 1 (hostemu refusal): closed.** With each library required, `address rxsock`, `RXSOCK`, `rxunixsys` and `RXUNIXSYS` give `rc 30 -1` on both sides. So do `HostEmu` without the library and `address xyzzy` beside every library. `hostemu`, `HOSTEMU` and `"HOSTEMU"` with the library still refuse: `ADDRESS environment "HOSTEMU" a native package loader registers is not implemented (Phase 10 Extension)`, rc 120, where the oracle gives `rc 24 1`. A `hostemu` environment registered through `TestAddCommandEnvironment` after the library is loaded gives `0 0` on both sides.
* **Important 2a (halt drops written output): closed.** SIGINT by PID after 0.6 s:
  * redirected: oracle `HALT rc 98 elapsed>1.5 0 stem 1 first line 15`, head `HALT rc -4 elapsed>1.5 0 stem 1 first line 6`. Round 0 gave `stem 0 S.1`.
  * redirected input: both `stem 0 S.1`.
  * plain: oracle rc -112 (unset status), head -4. This is Deviation 10 as before.
* **Important 2b, the implementer's Deviation 12 claim: partly holds.** Deviation 12 already records this observable departure: "a redirected command under a process-group SIGINT is 98.923 ... where this crate ... raises 4.1 at the same clause". The HALT at the command's clause, where the oracle raises 98.923 first, is therefore not a new candidate. But the same entry rests on a mechanism the round removed. It says a halt "does not end ... a redirected command's wait, which runs on the baton holder's thread ... the halt is served when the wait returns", and its witness name `ctrl_c_halts_a_redirected_command` describes that wait. Now the halt ends the wait, with rc -4 and the child left running, as for an unredirected command. Under PID signalling the oracle also leaves the child running, and `alive after: 1` holds on both sides. So the licensed departure now covers a crate behaviour the ruled text says does not happen. Correcting it is a change to a ruled Deviation's text, so it goes to Moritz with the correction (the report's Concern 1). It does not block the code.
* **Important 3 (targets mid-command): closed.** `observe.rex`, 2 of 2: `mid stem 1 one arr 0` / `end stem 2 one two arr 0` on both sides. More probes:
  * Partial lines across pieces (`printf "ab"; sleep; printf "c\r\nd"; ...`): `mid 1 [abc] [2]`, `end 4 [abc] [de] [] [last] rc 0` on both.
  * One array as both output and error target, merged: `mid both 2 o1|e1`, `end both 4 o1|e1|o2|e2` on both.
* **Minor 4 (rooting witness): closed.** `gc.rex` matches the oracle. I did not rebuild the rooting mutant. The new witness has the shape that killed it in round 0.
* **Minor 5 (TRACE C): closed.** `trc_<t>.rex` covers a bare string, `address system 'lit'`, `address system x 'expr'`, `address value`, ADDRESS WITH, INTERPRET of an ADDRESS command, a command in THEN, `exit 3` / `exit 127`, a DO body, the default environment, an unregistered environment, and two parked redirected commands in a started method. The output is byte-identical to the oracle under `TRACE C`, `E`, `F`, `N`, `A` and `R`. The round-0 binary differs only under `C`. Under `TRACE I` the output is identical as a multiset. Its order varies between oracle runs (3 of 3 differed from the first), because two threads' trace lines interleave.
* **Earlier probes, re-run on HEAD and identical to the oracle:** `conds`, `instem`, `insyn`, `halt` (message halt, 2 of 2), `mainexit`, and `reply`.

## The round's new mechanisms

**(a) Per-wait token and re-park.**
* `many.rex`: 8 activities × 3 redirected commands, each writing 40 to 160 lines with sleeps between them, the error stream going to an Array, and per-line order checked. `bad 0` in 3 of 3 runs on both sides.
* `two.rex`: 6 of 6 identical on head. The oracle hung in 3 of 6 runs (rc 124, `4.1` at the command), an oracle defect for the oracle-crashes list.
* The token guard is load-bearing. A mutant without the token check in `Blocked::end` gives `next 0 0 O.1 O.2` in 4 of 4 runs of `command_redirected_target_raises.rex`. HEAD gives `next 0 2 c d` in 4 of 4.
* Program end while a raised target's child is still running: both sides exit after the child, at 1.01 s and 1.08 s, and the child's `touch` file exists at exit.

**(b) Targets prepared first.**
* `begin.rex`: a bad `APPEND` stem as output (`E1`), as error (`E2`), and as output that is also the input (`E3`) gives `26.904 ran 0` on both sides. The command never runs, as the report says.
* Matching the oracle: an array that is both input and output (`2 i1 i2`), the same with APPEND (`2 q1,q1`), a named stream that is both input and output (`f1 f1 more`), REPLACE with no output keeping the default (`0 dflt`), and APPEND with no output (`3`).
* Two cases differ, and in both the oracle is at fault while HEAD equals round 0 and the parent:
  * input stem `t.` with output APPEND `t.` and error `t.`: oracle `t.0` 8, head 5. rexxref says "incremented by the total number of output lines", which gives 2 + 3 = 5.
  * input stem `u.` that is also the error stem: oracle `[U.]`, head `[in]`. The variant with no output target is **SIGSEGV on the oracle (rc 139)**: `w.0 = 1; w.1 = 'in'; address system 'cat >&2' with input stem w. error stem w.`. Queue it for oracle-crashes.

**(c) TRACE C.** Covered above. Untraced cost: callgrind `-r 1 -B 0.5`, exit 0. With libc subtracted, head minus round 0 is rexxcps -0.0016%, parse 0.0000%, startup 0.0000% and sayloop -0.0003%. The echo check runs only on the command path.

## Issues

### Critical

None.

### Important

1. **An output target that raises now ends the clause while its child still runs, and the child's later output is discarded. The oracle finishes the command first and keeps writing.** rexxref is silent on when this happens. The round's own rule for Important 3 was to follow the oracle where the documentation is silent, and round 0 matched the oracle's timing. The probe is `raise1.rex`: `.Bad subclass OutputStream`, whose first `lineout` raises 42.3 and whose later calls print `lineout <l>`. The command is `'echo a; sleep 0.6; echo b; exit 0' with output using (b)`, under `signal on syntax`. Three runs each:
   * oracle: `lineout b`, `trapped 42.3 after child 1 calls 2 rc 42`
   * head: `trapped 42.3 after child 0 calls 1 rc 42`

   With a ticking second activity (`raise2.rex`) the result is the same. In the halt variant (`sigint.rex`: SIGINT 0.6 s into the raising command, followed by a second redirected command), the oracle is still in the first command (`HALT rc 98 stem S.0 S.1 line 5`). Head has already trapped and moved on (`trapped SYNTAX 42.3`, `HALT rc -4 stem 1 x line 8`). `end_blocked_command` returns `written`'s error at once, and the token then drops the rest of the child's output and its end. Deviation 52 covers the rc dependence only, not this timing. Either keep the clause parked until the child ends, holding the first condition and still offering later lines to the target (the oracle's `ApiContext` catch, then `handleError` after the handler returns), or take "a target's condition ends the clause at once" to Moritz as a Deviation candidate.

### Minor

2. **Deviation 12's text is now false where it says a redirected wait is not ended by a halt** (above). It goes to Moritz with the correction, as the report's Concern 1 says. The licensed observable is unchanged.
3. **Oracle defects to queue:** the SIGSEGV with an error stem that is also the input stem; the oracle hanging in 3 of 6 runs of four concurrent redirected commands (`two.rex`); and the oracle's `t.0` 8 and `[U.]` above. None is copied.

## Tests

* `cargo test -p rexx-exec --lib`: exit 0, 1155 passed.
* `--no-fail-fast --test concurrency_tests --test rexx_command --test signals --test stdin_contention --test sim_processes --test program_end --test licensed_divergences --test refusal_sites`: exit 0, every binary ok. `the_table_holds_the_parts_the_spec_names` is green.
* `REXX_CORPUS_GATE=1 ... --test corpus`: exit 0, `1140 of 1140 matching`.
* I did not re-run the whole suite this round.

**Verdict: CHANGES REQUESTED** (Important 1: fix it, or take it to Moritz as a Deviation candidate. Minors 2 and 3 are for the controller.)
