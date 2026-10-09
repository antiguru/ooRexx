# Task 6 re-review: fix round 1 (2bde37127..c1938dfa2)

### Finding Verdicts

- **C1. A line typed at a pre-boundary pause corrupts the SIGL and indent of the condition the paused clause queued** — ADDRESSED.
  - The fix is in `run/interpret.rs:371` and `:398`, which save and restore `clause_state` around `run_debug_fragment`. The `|| self.debug_pause()` exemption is gone from `clause.rs:269-280`.
  - The review's own probes `tv1`, `tw1` and `tu2` now agree with the oracle on stdout, stderr and rc.
  - Variations, all of which agree on the handler's SIGL:
    - `c1n`: CALL ON NOTREADY in a loop, three typed lines including an assignment.
    - `c1f`: CALL ON FAILURE, two typed lines.
    - `c1se`: SIGNAL ON ERROR.
    - `c1sn`: SIGNAL ON NOTREADY.
    - `c1r`: a typed line that raises SYNTAX (`zz = 1/0`).
    - `c1i`: a pause before an INTERPRET fragment.
    - `c2proc`: the pause inside a PROCEDURE.
    - `c2int`: the pause inside an INTERPRET fragment.
    - `c2call` and `c2sig`: a typed `call sub` or `signal sub`. The handler's SIGL is now 3, matching the oracle; 2bde37127 printed 1.
  - HALT queued by the paused clause (`c1h`, the command sends SIGINT to its parent): stdout agrees. stderr differs only in the `RC(-113)` against `RC(-4)` line.
  - The head debug build fires no tripwire on any probe in `t6rev/p` or `t6rr/p`, or on any new witness.
- **I1. CALL ON and CALL OFF pause** — ADDRESSED. The fix is `run/interpret.rs:485`. `con`, `ca`, `ci`, `cr` and `e7` agree, and so does `so` (SIGNAL ON/OFF, CALL ON/OFF ANY).
- **I2. A LEAVE that raises 28.1 or 28.3 pauses before the error** — ADDRESSED. The fix is `run/interpret.rs:224`. `lv1`, `lv3` and `lv2` agree.
- **I3. A NOTREADY from the debug read does not reach a CALL ON NOTREADY trap as the oracle delivers it** — NOT ADDRESSED at the ITERATE/LEAVE pause.
  - The review's probes `di4`, `eo3` and `eo4` agree, and so do the three new witnesses.
  - The pause in `flat_loop_step_escaped` (`ir/drive.rs:2484`) queues the read's NOTREADY and does not deliver it. It is delivered at a later clause's boundary with that clause's SIGL.
  - `dh5`: `do i = 1 to 2; iterate; end`, default `.DebugInput` at EOF. The oracle prints `nr 3, nr 4, nr 4, after, nr 6` and ours prints `nr 3, nr 4, after, nr 6, nr 6`.
  - `dh8` (`iterate i` from an inner loop): ours prints `nr 4` where the oracle prints `nr 5`.
  - The head **debug** build fails the fourth-site tripwire on both, rc 101: "a clause at line 6 began while a condition queued by this activation's clause at line 4 was still waiting".
  - At 2bde37127 this path ended in 43.1, so the defect was hidden there. It is reachable only since this round's `trap_frame` change.
- **I4. A line typed at INTERPRET's pause changes the fragment echo's indent** — ADDRESSED, by C1's restore. `im`, `sk`, `sk2`, `sk3` and `debug_interpret_indent` agree.
- **I5. `=` at the pause after a LEAVE diverges** — ADDRESSED.
  - `e1` now has the same stdout and the same rc (228, Error 28.1) as the oracle.
  - stderr differs only in the indent of the re-run echo and of the error report's echo, which the KNOWN GAP row records.
  - The crate test is `src/tests.rs:492`.
- **M1. Concern 2 is an oracle defect and belongs among the deviations** — ADDRESSED.
  - Deviation 26 is true. The oracle printed Error 10.1 and exited rc 246 on 4 runs of 4 of its program (`z26`). Ours echoes the DO again, prints `end` and exits rc 0.
- **M2. The oracle crashes when `.DebugInput~linein` answers no value, and related cases** — partly ADDRESSED.
  - Entry 31 is true. Its exact program gave SIGSEGV, rc 139, on 3 runs of 3 (`o31.r1`-`r3`), and ours prints `x`, `y` and exits rc 0.
  - Still recorded nowhere:
    - `di6`: the recursing `.DebugInput`. The oracle gives SIGSEGV and ours rc 245.
    - `dj1`/`di3`: how the crate handles non-string answers.
- **M3. Concerns 1 and 4 are recorded nowhere** — ADDRESSED.
  - The ITERATE row matches `e2`: `3 *-*   iterate` against `3 *-*       iterate`.
  - The labelled-block row matches `g1`:
    - `do label lbl` pauses after its header in the oracle and not in ours.
    - The unlabelled block DO, `do label rl i = 1 to 2`, `loop label fl`, `do label w while 0` and `select label s` all agree.
- **M4. REPLY: the continuation prints the debug banner again** — NOT ADDRESSED. `k3` still differs. It is not recorded: the Task 12 queue line in `progress.md` names "REPLY no pause" and not the banner.
- **M5. Pre-existing divergences observed while probing** — no action expected. `lv4`, `lv4n`, `lv4o`, `cs`, `cc2` and `k2n` still differ as they did at base.

### New Breakage in the Fix Diff

- **Important.** LEAVE's pause moved into `flat_loop_step_escaped` (`ir/drive.rs:2472`, `:2484`; `run/interpret.rs:224`), and that site does not deliver what its read queues. This regresses a case that 2bde37127 got right.
  - `dh7b`: `.local~debuginput = .stdin`, `do i = 1 to 2; leave; end`, one stdin line, so the read hits EOF at the LEAVE's pause.
    - The oracle prints `nr 5`, then `after`, then `nr 7`.
    - 2bde37127 printed `nr 5` first.
    - Head prints `after`, then `nr 7` twice: the LEAVE's condition is reported with SIGL 7.
  - `dh6` and `dh4` show the same with the default `.DebugInput` at EOF (SIGL 6 for 4, and 9 for 7). The head debug build fails the fourth-site tripwire on both (rc 101).
  - One fix covers this and I3's remaining site: deliver what the pause queued after `debug_pause_instruction` in `flat_loop_step_escaped`, as `debug_pause_in_region`'s new DO/LOOP arm does (`ir/drive.rs:2443-2453`). Witness LEAVE, ITERATE and `iterate i` under CALL ON NOTREADY at debug EOF.
- **Minor.** The KNOWN GAP row for `=` after LEAVE says "Everything else agrees". In `e1`, the error report's traceback echo also prints at this crate's indent: `3 *-*   leave` before `Error 28`, where the oracle prints `3 *-* leave`. The crate test's own comment counts that echo. The row should name it.
- **Minor.** The `trap_frame` change in `run/condition.rs:130` also fixes a non-debug defect that the report does not mention, and no witness pins it.
  - Under `call on notready`, a read at EOF from `linein()`, `.stdin~linein` or `charin()` ended in Error 43.1 from `Method UNKNOWN with scope "Monitor"`, rc 213. The same held for stdin `linein()` in a loop.
  - This was true at base fe4956b36 and at 2bde37127 (`n1`, `n8`). Head matches the oracle on both.
  - My search found no corpus program combining CALL ON NOTREADY with a stdin read. A non-debug witness would hold this fix in place.

### Concern checks (run)

- **Item 2: the tripwire keyed on fragment depth.** The key exempts only typed lines. I built debug variants from a `git archive` of c1938dfa2, each in its own target directory with a `Compiling rexx-exec` line:
  - **Variant A (depth key removed).** The tripwire fires only on debug programs that type a line at a pre-boundary pause. These are `ca`, `cc`, `ce`, `ci`, `cr`, `e7`, `tp`, `tq`, `tr3`, `tr4`, `tu2`, `tv1`, `tv1b`, `tw1`, the `c1*`/`c2*` typed-line probes, and the witnesses `debug_pause_sigl_notready`, `debug_pause_sigl_error`, `debug_reexecute_call_on` and `debug_pause_before_trap`.
    - It fires on none of the non-debug INTERPRET probes `i1`-`i8`. These cover:
      - conditions queued by the INTERPRET expression before the fragment;
      - conditions inside the fragment, and nested INTERPRET in a handler;
      - CALL from a fragment into a routine that interprets;
      - RAISE ... RETURN from a function the fragment calls;
      - SIGNAL ON SYNTAX out of a fragment;
      - loops inside a fragment.
    - All eight agree with the oracle.
    - Why non-debug fragments are never exempted: their clauses run on the INTERPRET's line, so the line check already passes. A leftover from an earlier depth-0 clause is caught at the INTERPRET's own depth-0 entry.
  - **Variant D (the DO/LOOP-header delivery disabled).** The tripwire fires on `eo3` and `debug_input_eof_call_on_loop`, so it is live at depth 0 in debug mode.
  - **Variant B (hot-exit delivery skipped).** The earlier `spend_clause_entry` assertion (`clause.rs:415`) fires on `i1`-`i3`, `i5`, `i6` and `i8`.
  - **Variant C (the C1 restore removed).** The debug build raises **no** assertion, while `tv1`, `tw1` and both `debug_pause_sigl_*` witnesses print `handler sigl 1`. The depth key leaves the tripwire blind to the C1 class. The two corpus witnesses are its only guard. They catch it, so this is a trade and not a defect.
- **Item 3: NOTREADY outside debug.** `n1`-`n8` agree with the oracle on stdout, stderr and rc. They cover:
  - `.stdin`, `linein()` and `charin()` at EOF;
  - stream objects with `linein`, `charin` and `lines` in loops;
  - a method's own trap;
  - a non-continuing FORWARD to a method with and without its own trap;
  - FORWARD CONTINUE;
  - PROCEDURE and `::ROUTINE` traps;
  - SIGNAL ON NOTREADY;
  - stdin `linein()` in a loop.

  `n1` and `n8` differed at base and at 2bde37127 (see the third breakage item).
- **Item 4: records.**
  - Entry 31, Deviation 26 and the two KNOWN GAP rows are true, with the LEAVE row's wording exception above.
  - The ten new witnesses agree with the oracle on head, compared with their `.stdin` files. Each differs at 2bde37127, as the report's table states.
- **Item 5: perf.** The record quotes the command (`task-6-report.md:242`). Its base is `target-base`, which is fe4956b36 per the report's Performance section.
  - The report figures are fr1 against base: emptyloop -0.0013% and rexxcps -0.0472%. They match `cg-fr1/*.row`.
  - My own head release build (a fresh target directory with a `Compiling` line, one round) reproduces them: emptyloop 7761207073 against 7761305343 (-0.0013%), and rexxcps 17776273346 against 17784659886 (-0.0472%).
  - `bin-fr1` has no build log in `/tmp/claude-1000/p61/t6/`. The figures are cumulative against fe4956b36, not against the 6.1 base 04b5f0ff8.

### Out-of-Scope Observations

These are outside the fix diff. All were present at 2bde37127.

- **Task 6 regression against base.** A condition raised by a typed line itself is trapped. The oracle's `RexxActivation::trap` ignores every condition except SYNTAX during a debug pause (`RexxActivation.cpp:2475-2480`, `:2600-2607`).
  - `c2rn` (typed `y = linein('/nonexistent/bb')` under CALL ON NOTREADY): ours runs the handler with SIGL 1. The oracle does not run it, and base did not.
  - `c2re` (typed `'exit 3'` under CALL ON ERROR): ours runs the handler an extra time, with SIGL 1. The oracle does not.
- **Pre-existing at base.** A typed command updates RC and traces `+++ RC(3)`. The oracle leaves RC and `.RS` alone during a debug pause (`RexxActivation.cpp:4442`). Probes: `c2re`, which prints `rc now 3` where the oracle prints `rc now 1`, and `c1re`.
- **Pre-existing at base.** A typed `signal lbl` does not transfer control. `c1sig` and `c2sig`: the oracle runs the handler, then `at sub 3`; ours continues with `after`.
- **Pre-existing at base.** A typed `call sub` gives the callee SIGL 1 where the oracle gives the paused clause's line (`c2call`: `in sub 1` against `in sub 3`).
- **Pre-existing at base.** A NOTREADY raised by a DO header's own expression under `trace ?a` runs its handler before the header's pause. The oracle pauses first (`dh1`). Without debug it agrees (`dh1n`).
- **Oracle side, not ours.** `c2mult`: after a typed `do 2; nop; end`, the oracle echoes the rest of the pass two columns shallower. This is consistent with the oracle's traceIndent under-indent after a completed loop. Stdout agrees.

### Verdict

**Fix round:** Findings remain open.

- I3, at the ITERATE/LEAVE pause.
- New Important: the LEAVE pause no longer delivers what its read queues. This regresses `dh7b` against 2bde37127, and the debug build fails the tripwire on `dh4`, `dh5`, `dh6` and `dh8`.
- Minors open: M2 (`di6`, and the handling of non-string answers), M4, and the two new Minors.

Counts: Critical 0, Important 2 (I3 open, 1 new), Minor 4 (M2 part, M4, 2 new).

Probes are in `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/t6rr/p/` (`NAME.rex` with stdin `NAME.in`) and in the first review's `t6rev/p/`. Runs are in `t6rr/runs/NAME.TAG/{o,r}`, where `rel` is head, `old` is 2bde37127 and `base` is fe4956b36. Debug variant runs are in `t6rr/dbgruns/`. Every probe was run once per engine unless stated; none is concurrent.

## Fix round 2 (c1938dfa2..ac54b40f2)

The binaries were built from a `git archive` of ac54b40f2 into a fresh target directory (`t6rr/t2`): one release build and one debug build, each with a `Compiling rexx-exec` line. Probes and their stdin files are in the scratchpad: `t6rr/p` and `t6rr/p2`, plus the first review's `t6rev/p`. Runs are in `t6rr/runs/NAME.r2`, and the debug runs are in `t6rr/dbg2`.

### Finding Verdicts

- **I3 at the ITERATE/LEAVE pause** — ADDRESSED.
  - `flat_loop_step_escaped` now delivers what the pause queued (`ir/drive.rs:2475-2526`).
  - `dh5` and `dh8` agree with the oracle on stdout, stderr and rc.
  - `x6` agrees: `loop i = 1 to 3` with `if i = 2 then iterate`.
- **New Important (round 1): the LEAVE pause did not deliver what its read queued** — ADDRESSED.
  - `dh4`, `dh6` and `dh7b` agree.
  - `x7` agrees: `leave i` from an inner loop.
  - The debug build fires no tripwire on any of `dh4`-`dh8`.
- **Typed-line conditions (c2rn, c2re; ruled in scope)** — ADDRESSED.
  - The fix is in `run/condition.rs` (`ignored_in_debug_pause`, used by `trap_for` and `trap_at_depth`).
  - `c2rn`, `c2re`, `c1rn` and `c1re` agree on all three channels.
  - Variations, all of which agree:
    - `t3`: a typed `call sub`. When the routine raises NOTREADY under the inherited trap and under its own CALL ON, both handlers run as in the oracle.
    - `t4b`: a typed `raise user u` under CALL ON USER is ignored.
    - `t6`: a typed `raise halt` under CALL ON HALT is ignored.
- **Typed command sets no RC/.RS and traces nothing** — ADDRESSED. The fix is `command.rs:1049`.
  - Agreeing probes:
    - `t1`: typed `'exit 3'`, a FAILURE command and `address system 'false'` under CALL ON ERROR and CALL ON FAILURE, then `say rc .rs`.
    - `t2`: a typed `call sub` whose routine runs `'exit 4'`. The routine sets RC, as the oracle's routine activation does.
    - `t5`: `trace ?e` with a typed `'exit 5'`.
- **M2 (`di6`, non-string answers)** — ADDRESSED.
  - Entry 32's exact program gave SIGSEGV, rc 139, on 3 runs of 3 (`o32.r1`-`r3`). Ours stops at Error 11.1 with rc 245, as the entry says.
  - Deviation 27 matches `dj1` and `di3` as measured. The one exception is a wording point, below.
- **M4 (REPLY)** — ADDRESSED. The REPLY KNOWN GAP row matches `k3`.
  - The oracle prints the banner and pauses after `17 *-* reply 5`. Ours prints the banner and pauses after `18 *-* say 'after reply'`, so every typed line runs one clause late.
  - Stdout is the same.
- **Minor: LEAVE KNOWN GAP wording** — ADDRESSED. The row now names the error report's echo, as `e1` shows.
- **Minor: the non-debug stdin NOTREADY fix was unwitnessed** — ADDRESSED.
  - `notready_call_on_stdin` and `notready_call_on_stdin_loop` (`n1`, `n8`) agree, as do `n2`-`n7`.

### New Breakage in the Fix Diff

- **Minor.** Deviation 27's heading says a non-string answer "is read by its string value". Its body, and the crate's behaviour, contradict that for an object with a STRING method.
  - `dj1`'s second answer is an instance of class `s` whose STRING method answers `say 'from string'`. Ours runs the default string instead: `/bin/sh: 1: A: not found`. The body says so too ("runs its default string, as `to_text` gives it").
  - The heading should say what is read: the default string or the joined lines, not the string value.

### Checks run

- **Every probe, on release.** I ran 171 programs on the round-2 release build: the first review's `t6rev/p`, round 1's `t6rr/p`, and the 17 round-1 and round-2 witnesses with their `.stdin` files. The `p2` probes were run separately, below.
  - Every difference from the oracle is already recorded, or was listed as pre-existing or out of scope in round 1. Those are: `as1`, `cc`, `cc2`, `cs`, `di3`, `di6`, `dj1`, `dj2`, `ds5`, `e1`, `e2`, `k2`, `k2n`, `k3`, `l1`, `lv4*`, `z1`, `z26`, `o31`, `c1call`, `c1sig`, `c2call`, `c2sig`, `c1h`, `c2mult`, `c2proc`, `dh1` and `g1`.
  - All 17 round-1 and round-2 witnesses agree (10 from round 1, 7 from round 2).
- **Every probe, on debug.** The round-2 debug build, on the same 171 programs: no panic, and stdout is identical to the release build. The `p2` probes also run without a panic on debug.
- **LEAVE or ITERATE handler ends the program (`close_flat_top`).**
  - These programs agree with the oracle on stdout and rc:
    - `x1`: EXIT in the handler at an ITERATE pause, with the pass going on.
    - `x2`: the same in a nested loop.
    - `x3`: the same at a LEAVE.
    - `x4`: DO OVER inside a PROCEDURE.
    - `x5`: SIGNAL then EXIT from the handler.
    - `x8`: the handler's EXIT returns from a method that the caller's loop calls twice.
    - `x9`: the same through an external function.
  - `x4`, `x8` and `x9` differ on stderr only. `x4` lacks the `+++ "LINUX COMMAND"` header and `x8`/`x9` the `+++ "LINUX METHOD"`/`"LINUX FUNCTION"` headers, the pre-existing header class.
  - Reachability: a scratch build with `panic!` in place of the call panics on `x1`, `x4` and `x5`.
  - Necessity: a scratch build with the call deleted still passes `x1`-`x5`, because the program ends anyway. It fails the debug assertion "a LoopNext op ended a pass of a loop other than the one it names" on `x8` and `x9`, where the caller's loop goes on.
  - So `x8` and `x9` are the programs that make `close_flat_top` observable. Neither is in the corpus, so the path stays unwitnessed as the report says. Adding `x8` as a witness would close that.
- **Records.**
  - Entry 32 is true: 3 of 3 SIGSEGV, ours Error 11.1 with rc 245.
  - Deviation 27 is true apart from its heading.
  - The REPLY KNOWN GAP row is true (`k3`).
  - The LEAVE row is true (`e1`).
- **Perf.** My own build, one callgrind round against fe4956b36 (`t6rr/cg2`):
  - emptyloop: 7761207114 against 7761305564 (-0.0013%).
  - rexxcps: 17792493520 against 17784651245 (+0.0441%).
  - Both match the report.
  - Round 2 against round 1's build is +0.0919% on rexxcps, which is inside the budget. The report's "no change on the driver's hot path" refers to code; nothing here attributes the 0.09%.

### Out-of-Scope Observations

These are pre-existing at fe4956b36 and at c1938dfa2, and outside this diff.

- A typed `raise user u` with no USER trap, followed by more typed lines (`t4`), behaves differently in the two engines.
  - The oracle runs none of the later typed lines. It runs `say 'end'` with no pause after it, and traces nothing further.
  - Ours runs `say 'mid'`, reports the typed `raise syntax 40.1` and runs `say 'mid2'`.
- A traced method or external function prints no `+++ "LINUX METHOD/FUNCTION ..."` header in ours (`x8`, `x9`). An internal PROCEDURE prints one in ours and not in the oracle (`c2proc`, `x4`).

### Verdict (fix round 2)

**Fix round:** All findings addressed, and no new Critical or Important breakage. One new Minor: the heading of Deviation 27.

Counts: Critical 0, Important 0, Minor 1.
