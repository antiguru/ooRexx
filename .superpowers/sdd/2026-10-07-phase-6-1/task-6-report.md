# Task 6 report: interactive debug, pause placement and `.DebugInput`

Base `fe4956b36`. Commits `f062f791e` (the change) and `004312db8` (perf round). Line citations are
at `004312db8`.

## Design

The pause decision lives in one cold function, `debug_pause_after_clause`
(`run/interpret.rs:208`), given the clause and, on a cold exit, the `Flow` it answered. It asks
`pauses_after(kind)` (`run/interpret.rs:461`), the oracle's per-instruction list, against the flags
`TraceSetting::setDebug` derives from the letter: `pauseInstructions` is `tracingAll`, so an
instruction pauses under `?A`/`?R`/`?I` only, a label where labels trace (`pauseLabel`). Under a
negative skip count the flags come from the setting the count saved (`pausing_mode`, `:279`),
since suppression replaces the setting with one whose `all` is clear.

| where | what | `=` |
|---|---|---|
| hot exit, `ir/drive.rs:1887` | every promoted clause, now per kind (THEN no longer pauses) | `$pc = $clause_pc` |
| cold exit before `leave_stepped_clause`, `ir/drive.rs:1900` via `debug_pause_in_region` (`:2430`) | a Flowed region (Op::Exec, PARSE, EXPOSE, Message, unparked CALL, LEAVE, WHEN CASE false, a repeating DO with no passes) and a region ending with traps pending | `RegionEnd::At(clause_pc)` |
| `leave_parked`, `ir/drive.rs:3286` | a parked CALL once the callee returns | `RegionEnd::At(clause_pc)` |
| `settle`'s loop arm, `flat_loop_step_escaped` (`ir/drive.rs:2456`, called at `:3861`) | ITERATE, once the flat loop has stepped and echoed the DO | the ITERATE's op, `LeaveOrigin::index` |
| `exec_flow`'s INTERPRET arm, `run.rs:766` | before the fragment (`InterpretInstruction.cpp:76`) | `Flow::Goto(index)` |
| `settle_command`, `command.rs:1100` | a command the clause traced, after RC and its condition (`RexxActivation.cpp:4525`) | `Flow::Goto(index)` |

The cold-exit pause runs before the clause boundary, as the oracle's `execute` pauses before its
trap check: measured, the pause after `x = f()` comes before the `CALL ON` handler `f` queued
(`debug_pause_before_trap`; base runs the handler first). No pause follows an `Exit`, `Return` or
`Signal` flow, an `Iterate` flow (the loop arm owns it), an IF or SELECT that answers a `Flow`, or a
block `DO` that answers one (a labelled block runs nested, so its `Flow` comes after the body).
Commands are decided by `debug_pause_after_command` (`run/interpret.rs:260`): traced by `all` or
`commands`, or retraced for an error or failure, as `instruction_traced` is in the oracle.

The read (`debug_input_line`, `run/interpret.rs:341`) is `Activity::traceInput`: `LINEIN` sent to
`local_route(b"DEBUGINPUT")` under `pinned!` (`PinKind::TraceWrapper`); an absent entry or a `.nil`
answer is the null string, and a `.nil` entry is sent `LINEIN` as the oracle sends it (97.1). A
failure the read raises returns through `debug_pause_now`'s `Result`; on the cold exit it becomes
the region's failure and `leave_stepped_clause` records the site, and the hot exit records it
itself (without, the report said `0 *-* <no failing clause recorded>`).

## Branch-free attempt (R7) and outcome

Tried first, by reading the mechanism the spec named: lower the flowed ops differently when the
chunk is compiled under `?`, reached through the recompile on trace change. It cannot work. A chunk
is chosen once per level, in `running_level` (`run.rs:423`, its `chunk_for` at `:457` the only
caller, `grep -rn 'chunk_for(' crates/rexx-exec/src`), and a running chunk is never replaced: a
`TRACE ?A` inside a body leaves that body on the chunk compiled before it, which `$stale` exists to
gate. Frames hold op positions of that chunk (`Frame::op_end`, a flat loop's `op_body`), so
switching chunks mid-body would remap every open frame. `pa` (`trace ?a` then `numeric` in one body)
is exactly the case such a lowering never reaches.

Considered and not built: folding the pause into the `pending_traps.is_empty()` test both exits
already make, by keeping a marker in `pending_traps` while debugging. Branch-free in the default
mode, but every reader of `pending_traps` (scheduler, delivery, fragment retains) would have to
skip the marker.

So R7 applied: a `$debugging` test on the cold exit of `clause_region` and of `leave_parked`, and
ITERATE's step behind an out-of-line call in `settle`'s loop arm. Its cost is below.

## Other changes

- `activation.rs:767`, `:1073`, `run/call.rs:902`: an internal call inherits the caller's
  `DebugState` (`putSettings` copies the debug flags and the skip count). Without it the callee's
  first pause printed the prompt again (`dbgcall`'s only stderr difference at base).
- `trace.rs:210`: a command echoed only for its error or failure prints the debug banner first, as
  `traceClause` does (`RexxActivation.cpp:4305`).
- `run.rs:262`: `LeaveOrigin::index`. `command.rs`: `exec_command`, `end_blocked_command` and
  `settle_command` take the clause index.
- `ir/compile.rs:1919`, `:1927`: `assert_analysis_only_narrows` exempts a chunk keyed to a debugging
  setting, for which `trace_flow::analyse` answers `Unknown` by design. Pre-existing: a debug build
  of base panics at `compile.rs:1914` on `trace ?a` / `interpret "say 1"` (rc 101, base debug binary
  built from `fe4956b36`); release builds skip it.
- `clause.rs:271`: `enter_clause`'s tripwire exempts a line typed at a pause, which now runs while
  the paused clause's queued condition waits for its boundary (`debug_pause_before_trap` tripped it).
  Measured that the typed lines do not deliver it: `say 'typed1'`/`say 'typed2'` print before
  `handler` on both engines.
- `tests/collect_stress.rs`: `scope_case_text_debug_line`, `scope_debug_pause_routine`,
  `trace_debug_ignores_trace` and `trace_debug_skip` leave the zero-collection list: each pause now
  sends `LINEIN` through the `.DebugInput` monitor, which allocates.

## Witnesses and oracle agreement

`rust/corpus/phase-6-1.txt` `# Task 6`, each with its SOURCELINE expectation
(`crates/rexx-parse/tests/sourceline_oracle/<name>.txt`, from the sanctioned driver; each count
equals the file's `wc -l`). One run per engine (none concurrent), stdout, stderr and status compared
separately (`/tmp/claude-1000/p61/t6/run.sh`). Each differs from the oracle at base and agrees at
`004312db8` (the release binary measured below):

| witness | scout probe | base vs oracle | head vs oracle |
|---|---|---|---|
| `debug_pause_flowed` | `pa` | stdout | same |
| `debug_pause_kinds` | `pakinds` | stderr | same |
| `debug_pause_loops` | new | stderr | same |
| `debug_pause_commands` | new (`?C`) | stderr | same |
| `debug_pause_command_error` | new (`?E`) | stderr | same |
| `debug_pause_before_trap` | new | stderr | same |
| `debug_call_return` | `dbgcall` | stderr | same |
| `debug_call_pause_caller` | new | stderr | same |
| `debug_reexecute` | new | stdout, stderr | same |
| `debug_reexecute_command` | new | stdout, stderr | same |
| `debug_skip_flowed` | new | stderr | same |
| `debug_input_object` | `di` | stdout | same |
| `debug_input_eof` | `eof` | all three | same |
| `debug_input_nil` | new | all three | same |
| `debug_input_removed` | new | stdout | same |

The pre-existing debug programs (`trace_debug`, `trace_debug_skip`, `trace_debug_ignores_trace`,
`scope_debug_pause_routine`, `scope_case_text_debug_line`) still agree. Stdout and stderr of each
witness were read: each path its header comment names prints (the `pN`/`TN` lines a pause writes to
`.stderr` mark where it paused). `debug_input_object`, `debug_input_eof` and `debug_input_nil` have
no `.stdin`: none reads standard input, and the sidecar control rejects an inert one.
`debug_input_removed` ends in `PARSE PULL`, so its stdin is load-bearing and a pause that read
stdin would change what is pulled.

Crate tests (`src/tests.rs:465`, `:490`): `an_internal_call_does_not_prompt_again` and
`a_command_traced_for_its_error_prints_the_debug_banner`. Their programs run on the base release
binary show both defects (2 prompts; no banner before `2 *-* 'false'`).

## Performance

Builds, each in its own target directory with a `Compiling rexx-exec` line: base `fe4956b36`
(`target-base`), `head` (the worktree release build of `004312db8`'s code before its debug-only
`clause.rs` edit, copied to `bin-head4`), and `nobranch`, the same tree without the three R7 sites
(`target-nb4`; the only difference is `ir/drive.rs`, `diff -r -q`). Under `/tmp/claude-1000/p61/t6/`.

`memcap 8G bash rust/bench-programs/callgrind.sh -r 3 -j 4 -o $T/cg9 -p "emptyloop rexxcps" base=... nobranch=... head=...`,
exit 0, spreads 0.0001% at most:

| program | base | nobranch | head | nobranch % | head % |
|---|---:|---:|---:|---:|---:|
| rexxcps | 17784651519 | 17784933655 | 17793791881 | +0.0016 | +0.0514 |
| emptyloop | 7761304793 | 7761305047 | 7811303828 | +0.0000 | +0.6442 |

Inside the budget without the branch. **The R7 branch, apart: rexxcps +0.050%, emptyloop +0.644%**
(head against nobranch), under the house figure of ~0.5% and ~1.25%. emptyloop's step is 50,000,000
instructions, 2 per pass of its 25,000,000, in `ops_loop_steady`'s codegen; it moved in steps of
that size with unrelated edits (`cg7`, `cg8`: removing the `leave_parked` test alone gave +1.29%).

`PROGRAMS="rexxcps emptyloop" memcap 8G bash rust/bench-programs/wallclock.sh -r 5 -o $T/wall1 base=... nobranch=... head=...`,
exit 0, load average 1.90 at start and 1.55 at end:

| program | base s | nobranch % | head % |
|---|---:|---:|---:|
| rexxcps | 1.945 | +1.44 | +0.21 |
| emptyloop | 0.440 | -0.68 | +2.50 |

All inside ±4%.

Rounds: `f062f791e` measured rexxcps +0.84% (`cg1`; the branch was a test on the Flowed arm after
the boundary, with the clause's values kept live for it, plus a test inline in `settle`); variants
`vA` and `vC` (`cg2`) put +0.50% on the `settle` test and +0.29% on passing the clause's values to
the Flowed-arm call. Round 2 moved the test ahead of `leave_stepped_clause`, which takes the same
values, and the ITERATE step out of line; and the zero-pass DO pause out of `flat_loop_start`: a
no-branch build with it there measured emptyloop +0.6442% (`cg3`), without it +0.0000% (`cg4`).

## Commands and results

- `cargo fmt --all --check`: exit 0. `memcap 8G cargo clippy -j 8 --workspace --all-targets -- -D warnings`: exit 0.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus --test ir_recorded_oracle`
  at `004312db8`'s tree: exit 0, 29 passed 1 ignored, and 21 passed.
- `memcap 8G cargo test -j 4 --workspace --no-fail-fast` at `f062f791e`: exit 0, 3073 passed,
  0 failed, 4 ignored (summed over the `test result` lines, `/tmp/claude-1000/p61/t6/ws2.txt`).
- The same at `004312db8`: exit 0, 3073 passed, 0 failed, 4 ignored (`/tmp/claude-1000/p61/t6/ws3.txt`).

## Concerns

1. `=` at the pause after an ITERATE echoes the ITERATE at its static indent (`10 *-*       iterate`);
   the oracle echoes it at the loop body's (`10 *-*   iterate`). Not witnessed, not fixed.
2. The oracle mishandles `=` at a zero-pass DO's pause: a later valid `END` raises 10.1 (`trace ?a`
   / `do 0` / `say 'never'` / `end` / `do i = 1 to 2` / `nop` / `end`, stdin `=` then empty lines,
   rc 246). Ours re-runs the DO and continues. Not witnessed.
3. REPLY does not pause; the oracle pauses on the continuation's thread.
4. A labelled block `DO` (`do lbl; ...; end`, the nested path) does not pause after its header; the
   oracle does. Unchanged from base.
5. The new pause read runs Rexx (the `.DebugInput` monitor) at every pause, which is why four
   debug programs now collect under the stress mode.
6. The incidental fixes (prompt inheritance, banner before a retrace) are single-threaded; their
   crate tests assert the output, not switch points.

## Fix round 1

Commits `ca3a558b6` (fixes, witnesses, records) and `ac6cb27f6` (the LEAVE crate test counts the
error report's echo). Review: `task-6-review.md`. Probes and runs under `/tmp/claude-1000/p61/t6/fr1/`.

- **C1.** `run_debug_fragment` saves and restores `clause_state` around the typed line
  (`run/interpret.rs:371`, `:398`), so a condition the paused clause queued keeps its SIGL and
  handler indent. Every pause now precedes the boundary: the hot exit takes the cold exit under
  interactive debug (`ir/drive.rs:1886`, `pending_traps.is_empty() && !$debugging`), so the hot
  exit no longer calls the pause itself. The `|| self.debug_pause()` exemption in `enter_clause`
  is gone; the tripwire now keys a waiting condition by activation **and fragment depth**
  (`clause.rs:274`), as `deliver_pending_traps` keys a delivery. Without the depth key it fires on
  every typed line at a pre-boundary pause, the paused clause's condition being in the enclosing
  queue. `debug_pause_before_trap` is green on the debug gate with it.
- **I1.** `pauses_after` answers false for `Call::Trap` (`run/interpret.rs:485`).
- **I2, I5.** A `LEAVE` flow no longer pauses at the cold exit (`run/interpret.rs:224`). It pauses
  in `flat_loop_step_escaped` once the loop has taken it, as ITERATE does (`ir/drive.rs:2472`). So a
  LEAVE that reaches no loop raises 28.1/28.3 without a pause, and `=` runs the LEAVE again outside
  the loop, raising 28.1 at rc 228. The re-run echo's indent differs from the oracle's (the KNOWN
  GAP below), so I5's witness is the crate test `reexecuting_a_leave_raises_outside_its_loop`
  (`src/tests.rs:492`). It fails on `2bde37127`'s code (`after 1`, rc 0).
- **I3.**
  - `raise_notready` queues a CALL ON condition against `trap_frame()`'s activation
    (`run/condition.rs:130`). Under the monitor's non-continuing `FORWARD` the running activation is
    a forwarding phantom, and delivery there looked for the label in the monitor's code (43.1).
  - `debug_pause_in_region` delivers what the pause queued after a DO/LOOP header, which owes no
    boundary of its own (`ir/drive.rs:2449`). The oracle runs the handler before the body's first
    clause.
- **I4.** Fixed by C1's restore (`current_value_indent`).

Witnesses (`phase-6-1.txt`, `# Task 6 fix round 1`). Each was compared with `run.sh` against the
oracle on the `2bde37127`-equivalent release binary (`bin-head4`) and on the fix build. Every one
differs at the first and agrees at the second:

| witness | review probe | old vs oracle |
|---|---|---|
| `debug_pause_sigl_notready` | tv1 | stdout |
| `debug_pause_sigl_error` | tw1 (typed line made visible) | stdout, stderr |
| `debug_pause_call_on` | con | stderr |
| `debug_reexecute_call_on` | e7 | stdout, stderr |
| `debug_leave_no_loop` | lv1 | stderr |
| `debug_leave_unknown_label` | lv3 | stderr |
| `debug_input_eof_call_on` | di4 | all three |
| `debug_input_eof_call_on_loop` | eo3 | all three |
| `debug_input_stdin_eof_call_on` | eo4 | stdout, stderr |
| `debug_interpret_indent` | im | stderr |

All earlier debug witnesses and probes still agree (`runs/w-*`, `runs/p-*`). Of the review's
probes, only `e1` (the KNOWN GAP indent) and `dj2` (the oracle's SIGSEGV) differ.

`debug_pause_sigl_notready`'s SOURCELINE expectation was captured with its own `.stdin` on the
driver's stdin. With `/dev/null` the oracle ran the file's prolog under `.Package~new`, ran the
handler, then died with SIGSEGV (one run, not filed).

Records:
- `rust/corpus/oracle-crashes.txt` entry 31: `.DebugInput~linein` answering no value (review's
  `dj2`, 3 of 3).
- `phase-4-exclusions.txt` Deviation 26: `=` at a zero-pass DO, owner none.
- KNOWN GAP rows for the `=` echo indent after LEAVE/ITERATE, and for a labelled block DO with no
  header pause.

Perf (the hot exit changed):
```
memcap 8G bash rust/bench-programs/callgrind.sh -r 3 -j 4 -o $T/cg-fr1 -p "emptyloop rexxcps" base=$T/target-base/release/rexx-run nobranch=$T/target-nb4/release/rexx-run head4=$T/bin-head4/rexx-run fr1=$T/bin-fr1/rexx-run
```
Exit 0, spreads 0.0001% at most. fr1 against base: emptyloop -0.0013%, rexxcps -0.0472%. No
nobranch control was built for fr1; the total is inside the budget either way. Wall clock
(`wallclock.sh -r 5`, `wall-fr1`): rexxcps +0.15%, emptyloop +2.97%, inside ±4%.

Checks at `ac6cb27f6`:
- `cargo fmt --all --check` exit 0; clippy `-D warnings` exit 0.
- `memcap 8G cargo test -j 4 --workspace --no-fail-fast` exit 0: 3074 passed, 0 failed, 4 ignored.
- Strict corpus + ir_recorded_oracle at `ca3a558b6` exit 0: 29 passed 1 ignored, and 21 passed.
  `ac6cb27f6` changes only `src/tests.rs`.
- At `ca3a558b6` the workspace run failed the new crate test only. It counted 2 `leave` echoes
  where the error report adds a third; fixed in `ac6cb27f6`.

## Fix round 2

Commit `5495d5a08`. Re-review: `task-6-rereview.md`. Probes and runs are under
`/tmp/claude-1000/p61/t6/fr2/`.

**I3 at the ITERATE/LEAVE pause, and the LEAVE regression.** `flat_loop_step_escaped`
(`ir/drive.rs`) now delivers what the pause's read queued right after the pause, as
`debug_pause_in_region`'s DO/LOOP arm does.
- The delivery runs with the LEAVE/ITERATE clause's state, saved before the loop step, so SIGL is
  that clause's.
- Its indent is the loop's: the body's for a pass that goes on, the DO's for a loop that ended
  (`FlatLoop::do_indent` and `loop_indent`, now `pub(crate)`). Measured: the handler echo
  otherwise sat at the LEAVE's or ITERATE's own indent.
- After the delivery the step's clause state is put back.
- A handler that ends the program, with the pass going on, closes the flat loop first
  (`close_flat_top`, `run/loops.rs`). That path is not witnessed.

The re-review's `dh4`, `dh5`, `dh6`, `dh7b` and `dh8` all agree with the oracle on stdout, stderr
and rc.

**Typed-line conditions.** `trap_for` and `trap_at_depth` (`run/condition.rs`) answer no trap
for any condition but SYNTAX when the frame they read is running a line typed at a pause
(`ignored_in_debug_pause`, `RexxActivation.cpp:2475`, `:2600`). `settle_command` (`command.rs`)
returns before RC, `.RS`, the error/failure traces and the condition when the command was typed
at a pause (`RexxActivation.cpp:4442`). Without the second change, `c2re` still differed: it
printed `rc now 3`, which the re-review lists as pre-existing.

**Non-debug fix from round 1.** The `trap_frame` change in `raise_notready` also fixed CALL ON
NOTREADY for any read of standard input at its end (`linein()`, `.stdin~linein`, `charin()`). The
standard input monitors reach `.STDIN` through a non-continuing FORWARD, and the condition was
delivered inside the monitor's `UNKNOWN`, ending in Error 43.1, rc 213, at base and at
`2bde37127`. Witnessed now.

Witnesses (`phase-6-1.txt`, `# Task 6 fix round 2`), each compared once per binary with `run.sh`:

| witness | probe | c1938dfa2 code (`bin-fr1`) | 2bde37127 code (`bin-head4`) | head |
|---|---|---|---|---|
| `debug_input_eof_iterate` | dh5 | stdout, stderr | all three | same |
| `debug_input_eof_leave` | dh7b | stdout, stderr | stdout, stderr | same |
| `debug_input_eof_iterate_outer` | dh8 | stdout, stderr | all three | same |
| `debug_typed_condition_ignored` | c2rn | stdout, stderr | stdout, stderr | same |
| `debug_typed_command_quiet` | c2re | stdout, stderr | stdout, stderr | same |
| `notready_call_on_stdin` | n1 | same | all three | same |
| `notready_call_on_stdin_loop` | n8 (stdin `line1`) | same | all three | same |

The last two pin round 1's non-debug fix, so they agree at `c1938dfa2` and fail before it. Every
earlier debug witness and the round-1 probes still agree. The exceptions are `e1` (KNOWN GAP) and
`dj2` (oracle crash 31).

Records:
- `oracle-crashes.txt` entry 32: `di6`, SIGSEGV 3 of 3, this crate 11.1 at rc 245.
- `phase-4-exclusions.txt` Deviation 27: a non-string `.DebugInput` answer is read by its string
  value.
- A KNOWN GAP row for REPLY under debug: no pause at the REPLY, and the prompt at the
  continuation's first pause.
- The LEAVE KNOWN GAP row now names the error report's echo.

Perf: no change on the driver's hot path. One round of
`memcap 8G bash rust/bench-programs/callgrind.sh -r 1 -j 2 -o $T/cg-fr2 -p "emptyloop rexxcps" base=$T/target-base/release/rexx-run fr2=$T/bin-fr2/rexx-run`
measured emptyloop -0.0013% and rexxcps +0.0441% against fe4956b36.

Checks at `5495d5a08`:
- `cargo fmt --all --check` exit 0; clippy `-D warnings` exit 0.
- Workspace debug run exit 0: 3074 passed, 0 failed, 4 ignored.
- Strict corpus + ir_recorded_oracle exit 0: 29 passed 1 ignored, and 21 passed.
