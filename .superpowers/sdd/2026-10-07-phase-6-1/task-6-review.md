# Task 6 review: interactive debug, pause placement and `.DebugInput`

Base `fe4956b36`, head `2bde37127`. Binaries built for this review from `git archive` of each
commit into fresh target directories, each with a `Compiling rexx-exec` line: head release and
debug, base release, and a head debug build with the `clause.rs:271` exemption removed. Probes
are in `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/t6rev/p/`
(`NAME.rex`, stdin `NAME.in`, otherwise `/dev/null`). The runner is `../cmp.sh`: oracle under the
global-constraints wrapper and ours under `memcap 2G`, each from a fresh empty directory, with
stdout, stderr and rc compared separately. Outputs are in `../runs/NAME.TAG/{o,r}`. Every probe
was run once per engine; none is concurrent. The exception is `z1` (concern 2), which was run 3
times.

## Spec Compliance

- ❌ Issues found. Step 2: a NOTREADY that the read raises does not reach a `CALL ON` trap
  correctly (I3). Step 3: the placement has regressions that base did not have. They are CALL ON/OFF
  (I1), a LEAVE that raises 28.x (I2), and a typed line at a pre-boundary pause, which corrupts the
  queued condition's SIGL (C1). The SIGL defect is also masked by the `clause.rs:271` exemption.
- ⚠️ Not verified from the diff: the 3073-test workspace run and the 50-test strict corpus run.
  These were not re-run. The 20 debug witnesses (15 new and 5 pre-existing) agree with the oracle on
  my head build: `w_*` probes, all `out=same err=same rc=same`.

## Strengths

- All 20 debug witnesses match the oracle on stdout, stderr and rc. They agree on a build I made
  myself, not only the implementer's.
- The per-kind table matches the oracle across a wide sweep. Agreeing probes:
  - `k1`: PROCEDURE, EXPOSE, USE ARG, PARSE, OPTIONS, QUEUE, TRACE in debug, RETURN.
  - `k2`: FORWARD CONTINUE, RAISE (pause-wise).
  - `k3`: GUARD ON/OFF WHEN does not pause.
  - `l1`: loops. First, middle and last pass; ITERATE that ends the loop; `iterate k` from an inner
    loop; `leave outer`; DO WHILE, FOREVER, OVER, UNTIL, LOOP n; plain block DO.
  - `s1`: SELECT. WHEN false, OTHERWISE, IF/ELSE, THEN DO, WHEN THEN DO.
  - `s1`: calls. CALL of a builtin, an external file and an internal routine; a function call in an
    assignment; INTERPRET before the fragment and inside a multi-clause fragment.
  - `ca`..`cr`: commands under `?A ?C ?E ?F ?N ?R ?I ?L`, except the I1 site.
  - `tr2`: TRACE letters in debug.
  - `e3`-`e6`, `e8`, `e10`: `=` after PARSE PULL, a message instruction, a parked CALL, WHEN CASE,
    SAY and ITERATE.
  - `sk`, `sk2`, `sk3`, `ds±2`, `ds±4`, `ds6`: skip counts, positive and negative, through
    ITERATE, a command, CALL, INTERPRET, PARSE and an internal call.
  - `eo`, `eo2`: EOF with no trap.
  - `di1`, `di2`, `di5`: `.DebugInput` raising SYNTAX (trapped and untrapped) and a NOTREADY stream
    under SIGNAL ON.
- Every listed claim checks out. Lowering differently under `?` cannot work: `chunk_for`'s only
  production caller is `run.rs:457`. The nobranch tree differs from head in `ir/drive.rs` only at
  the three R7 sites; the debug-only `clause.rs` difference also holds, by
  `diff -r -q /tmp/claude-1000/p61/t6/nb4/rust/crates <head>`. The binary hashes in
  `cg9/binaries.txt` match the gate record. The gate record quotes both commands.
- The compile.rs exemption is sound. The assertion checks an analysis that is not yet wired into
  emission. `as1`, `as2` and the other probes, run on the debug build, raise no panic for any
  program, debug or not.
- `DebugState` inheritance and the banner before an error-only command echo are correct. `ds2`,
  `ds4`, `ds-2`, `ds-4`, `ds6`, `ce`, `cf` and `cn` agree.

## Issues

### Critical (Must Fix)

**C1. A line typed at a pre-boundary pause corrupts the SIGL and handler indent of the condition
that the paused clause queued. The `clause.rs:271` exemption masks it.**
`clause.rs:271`, `ir/drive.rs:2401` (`debug_pause_in_region`), `command.rs:1100`,
`run/interpret.rs` `run_debug_fragment`.

- `tv1` is `call on notready name h; trace ?a; x = linein('/nonexistent/zz'); ...;
  h: say 'handler sigl' sigl`, with stdin `nop` then empty lines. The oracle prints
  `handler sigl 3` and ours prints `handler sigl 1`. That is the typed fragment's line.
- `tw1` (command `'false'` under `call on error`, inside a DO) gives `sigl 4` in the oracle and
  `sigl 1` in ours. The handler also echoes at the wrong indent (`7 *-*   h:` against `7 *-*     h:`).
  `tu2` shows the same in a loop.
- With empty stdin lines all three agree (`ts1`). Without debug they also agree (`ts2`, `ts3`).
- At base, stdout agrees on `tv1`, `tw1` and `tu2`, because the handler ran before the pause. This
  is a regression this task introduced.
- On a debug build with only the exemption removed, the tripwire fires on `tv1`, `tw1` **and the
  committed witness `debug_pause_before_trap`** (rc 101, "a clause at line 1 began while a condition
  queued by this activation's clause at line 3 was still waiting"). The tripwire's own comment
  predicts the symptom: "The handler will now report `SIGL` for the wrong clause". The report's
  justification checks only that the typed lines do not deliver the condition. It does not check
  what the delivery reads afterwards.
- Fix: restore `activity.clause_state` (line and `current_value_indent`) after
  `run_debug_fragment` returns. Then delete the `|| self.debug_pause()` exemption, and add a witness
  whose handler prints `sigl` with a typed line at the pause.

### Important (Should Fix)

**I1. CALL ON and CALL OFF pause. The oracle does not pause after them.**
`run/interpret.rs:1951` (`pauses_after`). `RexxInstructionCallOn::execute`
(`CallInstruction.cpp:570-587`) has no `pauseInstruction`.

- `con` (`call on error / call off error / call on halt name h`): ours takes 3 extra pauses.
- `ca`, `cr` and `ci` take one extra pause at `call on error`. A typed `=` there re-runs CALL ON
  instead of the command, so `e7` prints `err` once where the oracle prints it twice.
- Base agreed on `con`, so this is a regression.
- `InstructionKind::Call(_)` needs to exclude `Call::Trap`. The table's doc names the oracle's
  per-class list, and the list's source is per class, not per keyword.

**I2. A LEAVE that raises 28.1 or 28.3 pauses before the error.**
`ir/drive.rs:1899` (cold-exit pause on `Flowed(Leave)` before `settle` resolves the target).

- The oracle's `RexxInstructionLeave::execute` calls `leaveLoop` before `pauseInstruction`
  (`LeaveInstruction.cpp:110-125`). The error therefore comes first and there is no pause.
- `lv1` (bare `leave` outside a loop) and `lv3` (`leave zz`) take an extra pause in ours. That pause
  consumes a line of debug input. ITERATE 28.4 is correct (`lv2`).
- Base agreed on `lv1`, so this is a regression.

**I3. A NOTREADY from the debug read does not reach a `CALL ON NOTREADY` trap as the oracle
delivers it.** This is Step 2. `run/interpret.rs` `debug_input_line` and `debug_pause_now`.

- With default `.DebugInput` at EOF, `di4` (`call on notready name nr; trace ?a; say 'a'; say 'b'`)
  and `eo3` both fail. The oracle runs the handler at each pause (`nr 3`, `nr 4`). Ours ends at rc
  213: `Error 43.1: Could not find routine "NR"`, raised at `REXX line 1457` (`Method UNKNOWN with
  scope "Monitor"`). The trap is delivered inside the monitor's code.
- With `.local~debuginput = .stdin` (`eo4`), ours runs the handler one clause late, with `SIGL` 5
  where the oracle has 4.
- Only SIGNAL ON is witnessed (`debug_input_eof`).
- Base also differs on `di4` and `eo4`: it read EOF as an empty line and never raised. The head
  failure is in this task's new code path.

**I4. A line typed at INTERPRET's pause changes the indent of the fragment's echo.**
`run.rs:766` (pause) and `run.rs:781` (`base_indent` read from `current_value_indent` after the
pause).

- `im` is `trace ?a; do i = 1 to 1; interpret 'nop'; end`, with a marker line at each pause. The
  oracle prints `3 *-*   nop` and ours prints `3 *-* nop`.
- The same appears in `sk`, `sk2` and `sk3` at every INTERPRET inside a loop.
- With empty lines, ours agrees (`ii2`).
- This is new with the pause before the fragment. It is likely the C1 fix, or reading `base_indent`
  before the pause.

**I5. `=` at the pause after a LEAVE diverges and is not in the report.**

- `e1` is `do i = 1 to 3; leave; end; say 'after' i`, with `=` at the LEAVE's pause.
- The oracle re-executes the LEAVE outside the loop and raises 28.1 (rc 228). Ours re-enters the
  body op, leaves again and prints `after 1` (rc 0).
- The report says `=` re-executes from either exit. This case is neither witnessed nor listed as a
  concern. Base also disagrees, with a different wrong answer.
- Match the oracle, or record it as a deviation.

### Minor (Nice to Have)

**M1. Concern 2 is reproduced, 3 of 3 (`z1`), and it is an oracle defect.**

- Cause: `RexxInstructionBaseLoop::execute` calls `terminate()` for a zero-pass loop, then
  `handleDebugPause(context, OREF_NULL)` (`BaseDoInstruction.cpp:289-299`). On `=`, that runs
  `removeBlockInstruction()` a second time (`:111-124`). The next loop's END then echoes one level
  too deep and raises 10.1, rc 246.
- It exits cleanly, with no crash, exhaustion or block. It therefore does not fit any mode in
  `rust/corpus/oracle-crashes.txt`.
- Under spec D2's DEVIATION disposition it is a numbered deviation, owner none, in
  `docs/superpowers/plans/phase-4-exclusions.txt`, filed the way Deviation 25 records an oracle
  defect.
- `=` after a DO with passes agrees (`z2`, `z3`).

**M2. The oracle crashes when `.DebugInput~linein` answers no value.**

- `dj2` gives SIGSEGV, rc 139, 3 of 3. `Activity::traceInput` casts the answer to `RexxString *`
  unchecked (`Activity.cpp:3252`). This belongs in `oracle-crashes.txt`.
- Non-string answers are type confusion in the oracle (`dj1`: an Array ran as the command `AN ...`).
- The crate's handling of them is not recorded anywhere. It uses `to_text`, and in `dj1` it printed
  `arr` and ran `A ...`.
- `di6` (a `.DebugInput` whose LINEIN itself runs under `trace ?a`) recurses: the oracle segfaults
  and ours stops at rc 245. This is another oracle-crashes candidate.

**M3. Concerns 1 and 4 are reproduced and recorded nowhere a gate sees.**

- Concern 1: `e2` echoes `3 *-*       iterate` where the oracle echoes `3 *-*   iterate`.
- Concern 4: in `l1` line 33, `do label lbl` does not pause after its header; the oracle does.
- Fix them, or file them as deviations.

**M4. REPLY: the continuation prints the debug banner again** (`k3`). The oracle does not. This is
outside concern 3's "does not pause". Base does the same.

**M5. Pre-existing, outside this task, observed while probing.** Each one is the same at base or
appears without debug.

- 28.3 from LEAVE is not trapped by SIGNAL ON SYNTAX at all (`lv4o`, no TRACE: ours rc 228, the
  oracle traps it, rc 0).
- The messages for 10.3 and 24.1 print unsubstituted `&2` and `&1` (`l1` first version, `cs`).
- `trace c` omits the clause echo of `address command 'false'` (`cc2`).
- The `>I>` method-entry line is missing when EXPOSE precedes TRACE (`k2n`).

## Assessment

**Task quality:** Needs fixes

**Reasoning:** The base design holds up: the per-kind table, both region exits, the out-of-line R7
branch and the measured branch cost all check out. But moving the pause ahead of the clause
boundary introduced a SIGL corruption, and the clause.rs exemption hides it from the tripwire built
to catch exactly that. CALL ON/OFF and an erroring LEAVE also regressed against base. A CALL ON trap
on the read's NOTREADY, which Step 2 requires, ends in Error 43.1.

Counts: Critical 1, Important 5, Minor 5. Item 4 (concern 2): reproduced, oracle defect; it goes in
`phase-4-exclusions.txt` as a deviation, not in `oracle-crashes.txt` (M1).
