# Scout B report: the silent-wrong-answer items of 6.1

HEAD `be19fd06a`, built from a worktree copy: `CARGO_TARGET_DIR=/tmp/claude-1000/p61/sb/target memcap 8G cargo
build --release -j 4 -p rexx-exec --bin rexx-run`. "Ours" is that `target/release/rexx-run FILE`, default switch
mode. The oracle is the command in `scout-common.md`. Every probe ran from its own directory holding only its own
files, through `/tmp/claude-1000/p61/sb/run.sh DIR FILE [STDIN]` (ours, then the oracle, `2>&1`, rc after each).
One run each unless a count is given. Ours writes stdout and stderr as separate buffers, so under `2>&1` the
relative order of `SAY` lines and trace lines differs from the oracle's even where each stream matches; the
comparisons below are made per stream. Probe texts are in the appendix.

## 1. Activity vs activation scope (clock, RANDOM seed, SETLOCAL, and three more)

### Probe results

| probe | ours | oracle |
|---|---|---|
| `c2` routine resets clock | `routine r zero: 0` | `routine r zero: 1` |
| `c5` external program resets clock | `external zero: 0` | `external zero: 1` |
| `int` internal `TIME('R')` after caller read `E` | `0` | `1` |
| `reply` (5 runs each) | `m before 0.050234..0.050630`, `m after 0.080945..0.081604` | `m before 0`, `m after 0.031429..0.031942` |
| `rnd` seeds 7 in main, 11 in a method, 13 in a routine | `876 457 99 70` | `876 457 517 735` |
| `sl1` method `SETLOCAL`, main `ENDLOCAL` | `main endlocal 1` | `main endlocal 0` |
| `sl2` across REPLY (5 runs each) | 5/5 `cont endlocal 0`, `main endlocal 1` | 5/5 `cont endlocal 1`, `main endlocal 0` |
| `sl6` routine ends with an open `SETLOCAL` | `main sees routine` | `main sees orig` |
| `cond1` routine `RAISE PROPAGATE` inside main's handler | rc 214, 42.3 propagates | `routine trapped 98.918`, rc 0 |
| `cond2` main `RAISE PROPAGATE` after a routine handled 42.3 | rc 214, 42.3 reported | `main trapped 98.918`, rc 0 |
| `cond3` internal call in a handler propagates | rc 214, 42.3 | rc 214, 42.3 (same; traceback differs, see below) |
| `ct2` absorbed `WHEN` after a call that ran `SELECT CASE 'zz'` | `other` | `done` |
| `ct4` same, absorbed `WHEN 'zz'` | `done` | `other` |
| `ct3` control, callee runs no `SELECT CASE` | `done` | `done` |
| `dbgcall` `call s` typed at a `trace ?r` pause | `s` runs untraced, no pauses | `s` traced, pauses after each of its clauses (T1..T4 consumed inside `s`) |

`rnd`: the oracle's `517 735` is main's own seed-7 stream continuing; the method's seed 11 and the routine's
seed 13 never reach it. `cond3`'s traceback also differs: ours prints an extra `9 *-* call s` line. That is a
separate traceback defect, not scope.

Side finding, oracle crash: `sl3` (routine `SETLOCAL` left open, then an internal `SETLOCAL` and main's
`ENDLOCAL`) aborts the oracle with `free(): invalid pointer`, rc 134, 5/5 runs
(`for i in 1 2 3 4 5; do run.sh probes/sl3 sl3.rex | grep -c Aborted; done` printed `1` five times). Ours prints
`after internal internal 1`, rc 0. Candidate for `corpus/oracle-crashes.txt`.

### Oracle rule, with citations

- Settings live in `ActivationSettings` (`execution/ActivationSettings.hpp`), one per activation: `elapsedTime`,
  the `elapsedReset` flag, `timeStamp`, `conditionObj`, `traceIndent`, `streams`, `traps`, `ioConfigs`.
- Internal call and INTERPRET: `_parent->putSettings(settings)` copies the caller's whole settings in
  (`RexxActivation.cpp:225`). An internal call gets `timeStamp.valid = false` (`:241`) but keeps the stale stamp
  and the reset flag. Nothing is copied back on return. INTERPRET copies back on return
  (`parent->getSettings(settings)`, `:686`).
- Method (`:130-185`) and routine/program/external (`:268-330`) start from `clearObject()`: `elapsedTime == 0`, no
  `conditionObj`, `debugPause == false`, and `randomSeed = activity->getRandomSeed()` (`:174`, `:312`), which
  advances the activity's generator (`Activity.cpp:469-475`).
- `TIME`: `getTime` anchors a pending reset to the stale stamp on the next fresh read (`RexxActivation.cpp:3391-3413`);
  `getElapsed` anchors lazily when `elapsedTime == 0` (`:3424-3432`).
- `RANDOM`: `getRandomSeed` delegates to `parent` for an internal call or INTERPRET (`isInternalLevelCall`,
  `:3455-3460`), so internal calls share their top-level activation's seed; each top-level activation has its own
  (`RexxActivation.hpp:648`).
- `SETLOCAL`/`ENDLOCAL`: `pushEnvironment`/`popEnvironment` act on the nearest `isTopLevelCall()` activation
  (program, method or external call; `RexxActivation.cpp:4640-4682`, `RexxActivation.hpp:173`). At termination an
  activation with a non-empty list restores the oldest saved environment (`:1494-1500`).
- `RAISE PROPAGATE` reads `context->getConditionObj()` (`RaiseInstruction.cpp:262`, `settings.conditionObj`,
  `RexxActivation.hpp:330`), else 98.918 "No active condition available for PROPAGATE". A `CALL ON` handler's
  activation gets it set (`RexxActivation.cpp:3340`); internal calls copy it; routines and methods start without.
- `SELECT CASE`'s value is on the select's `DoBlock` (`DoBlock.hpp:78` `setCase`), read by
  `RexxInstructionCaseWhen::execute` from `context->topBlockInstruction()` (`WhenCaseInstruction.cpp:146-148`):
  per activation, per construct.
- Debug pause: `debugPause` is a member of each `RexxActivation` (`RexxActivation.hpp:632`), true only on the
  activation `debugInterpret` creates (`RexxActivation.cpp:205-212`, `:2786-2802`). `inDebug()` is
  `isDebug() && !debugPause` (`RexxActivation.hpp:367`), so a routine called from a typed line has
  `debugPause == false` and traces and pauses under the copied `?R`.

### Where activation state lives in ours

`Activation` (`rexx-exec/src/activation.rs:254`, boxed, recycled through `Activity::spare_activations`) already
holds the per-activation settings: `settings`, `trace_mode`, `debug`, `address`, `traps`, `condition`,
`cached_clock`/`clock_stale`, `streams`. Constructors: `Activation::new` (program, external, and the native
`Conversion` frames), `::nested` (internal call and `CALL ON`, `activation.rs:643`, takes `Inherited`,
`activation.rs:931`, built at `run/call.rs:901`), `::routine` (`run/call.rs:946`), `::method` (`dispatch.rs:3171`).
INTERPRET and debug-pause lines run inside the current activation as fragments (`run/interpret.rs:28`, `:255`), so
INTERPRET already shares state, which matches the oracle's copy-in plus copy-back. REPLY moves the replying
`Activation` box to the continuation (`split_level`, `ir/drive.rs:2932-2938`, and `release_method_activation`). Level state that is
activation-scoped but kept on `Activity` is saved and restored per call by `CallTail` (`run/call.rs:66-74`,
`:971-1006`): `clause_state`, `activation_indent`, `indent_offset`, `clause_line_override`, `call_context`.

### Fix sketch

Move each wrong-side field onto `Activation`; P89's three copy lines (`ir/drive.rs:2984-2986`) are then deleted,
because the box travels with the continuation.

| field | new home | program / routine / method / external / started | internal call, `CALL ON` | INTERPRET, debug line | REPLY continuation |
|---|---|---|---|---|---|
| `elapsed_anchor`, `pending_elapsed_reset` | `Activation` | fresh (`None`, false) | copied from caller via `Inherited`, together with the caller's stale `cached_clock` (today `nested` sets `cached_clock: None`, `activation.rs:687`, which loses the oracle's lazy-reset anchor) | shared (same activation) | moves with the box |
| `random_seed` | `Activation`, lazily seeded | own, lazily from `initial_seed` mixed with something per activation | none of its own: `next_seed` (`builtin/numeric.rs:560-573`) walks `running` then `suspended` back to the first non-`Entry::InternalCall` activation | shared | moves |
| `locals` | `Activation` | own list | none: push/pop (`lib.rs:2545-2570`) walk to the first non-`InternalCall` activation | shared | moves |
| termination restore of `locals` | activation end | if the ending top-level activation's list is non-empty, restore its oldest entry | n/a | n/a | at the continuation's end |
| `active_condition` | `Activation` (boxed) | none | copied from caller (`Inherited`; `CALL ON` handler gets the trapped one) | shared | moves |
| `current_case_text` | the `SELECT` construct (`ir/drive.rs` `Frame`) or `Activation` | none | none | shared | moves |
| `debug_pause` | `Activation` (flag bit) | false | false, so `TraceCache::of` at `push_activation` (`activation.rs:1137`, `:1175`, `:1201`) sees the callee's own | set on the running activation for the typed line's duration (`run/interpret.rs:256`, `trace.rs:59`) | n/a |

Constraint: `const _: () = assert!(size_of::<Activation>() == 512)` (`activation.rs:497`). Suggested packing:
`elapsed_anchor` as an `i64` with 0 meaning unset (the oracle's own sentinel), `pending_elapsed_reset` and
`debug_pause` as bits in `ActivationFlags` (a `u8` using three bits today), and `random_seed`, `locals` and
`active_condition` in one lazily allocated cold box, allocated on first `RANDOM`, `SETLOCAL` or trap. Whether
8 inline bytes still fit in 512 must be checked with the assert; I did not build it.

Files: `activation.rs`, `activity.rs`, `run/call.rs`, `builtin/datetime.rs` (sites at 777-781, 863-876),
`builtin/numeric.rs`, `lib.rs` (SETLOCAL, root list at 1893), `run/condition.rs` (398, 506-616, 988),
`run/select.rs:57`, `run.rs:900`, `trace.rs`, `run/interpret.rs`, `dispatch.rs:3331`, `ir/drive.rs` (P89 lines),
`scheduler/tests.rs:1058` (rewrite `a_reply_continuation_keeps_the_elapsed_clock_and_the_random_seed`). Size M
overall (each field S).

### Performance risk

- Per call: each constructor writes the new fields; `nested` copies three more words from the caller. The call path
  is hot (`bench-programs/fibcall.rex`, `fibfunc.rex`, `dispatch.rex`, `sendloop.rex`), so measure those. A size
  change of `Activation` past 512 also changes recycling cost; the packing above keeps the inline growth to one word.
- Per activation end: one branch on the cold box (`locals` non-empty) for the termination restore.
- Per clause: none. `debug_pause` is read per clause only under `cfg(debug_assertions)` (`ir/drive.rs:1777`); the
  release path reads `trace_cache`.
- `RANDOM` and `SETLOCAL` from deep internal recursion pay an O(depth) walk per call. Copy-in plus copy-back on
  `finish_call` would avoid that but puts work on every internal return; the walk keeps it off the call path.

### ooTest rows

`base/bif/TIME.testGroup` whole: failing `TEST_VALIDOPT_BIGCHAR_R TEST_VALIDOPT_LITTLECHAR_R TEST_10 TEST_11 TEST_4
TEST_5` (table.txt line 4); derived: `TEST_10 TEST_11 TEST_4 TEST_5` (line 5). `base/keyword/CALL.testGroup` rest and
derived: `TEST_4` (lines 44-45). The queued item attributes `TIME` TEST_4/TEST_10 and `CALL` TEST_4 to the clock,
and the `_R` pair to the per-activity clock (`clock/summary.txt`); I did not re-run them. `TEST_5` and `TEST_11` are
unattributed. No `RANDOM`, `SETLOCAL`, `RAISE PROPAGATE`-scope or `SELECT CASE` row is in `table.txt` or
`alone.txt`; the `RAISE` rows (`alone.txt` 7-9) propagate inside one routine and are not this defect.

## 2. Debug pause skipped after a flowed clause

### Probe results

`pa` (stdin `say 'p1'`, empty, `say 'p2'`, empty, `say 'p3'`, empty): stderr identical; stdout ours
`c1 p1 c3 p2 c4 p3`, oracle `c1 p1 p2 c3 p3 c4`. Reproduces.

The class is wider than `NUMERIC`/`TRACE`. `pakinds` types `.stderr~lineout('pN')` and an empty line at each pause, so
each pause shows up as a line in the trace stream; the clause echoes are byte-identical
(`diff o.txt r.txt` printed `trace-identical`). Pause placement differs (`diff -y o2.txt r2.txt`):

| after clause | ours | oracle |
|---|---|---|
| `numeric digits 9` | no pause | pause |
| `address command` | no pause | pause |
| `drop zz` | no pause | pause |
| `parse pull qq` | no pause | pause |
| `call sub` (after `return`, in the caller) | no pause | pause |
| `interpret "say 'c13'"` | no pause | pause before the fragment's clause |
| `then` (both `IF` and `WHEN`) | pause | no pause |
| `say`, assignment, `nop`, `push`, `do`, `if`, `select`, `when`, label, `.nil~string` | pause | pause |

### Oracle rule

Each instruction's `execute` ends with `context->pauseInstruction()` (`RexxActivation.hpp:380`, a pause when
`pausingInstructions()`), and the set is per instruction class: present in Address, Assignment, Call
(`CallInstruction.cpp:212`, `:349`, `:474`), Drop, Expose, Forward, If (`IfInstruction.cpp:163`), Leave, Message,
Nop, Numeric (`NumericInstruction.cpp:193`), Options, Parse, Procedure, Queue, Reply, Say, Trace in debug
(`TraceInstruction.cpp:162`, `:186`), Use, UseLocal, WhenCase, Constant; absent from Then, Else, End, Exit, Return,
Signal, Raise, Guard, Interpret, Select, Otherwise and the DO family, which use `conditionalPauseInstruction` or none.
Listing command: `grep -rln 'pause\w*()' interpreter/instructions/` against `ls interpreter/instructions/*Instruction*.cpp`.
Where the INTERPRET pause comes from I did not locate.

### Fix sketch

`ir/drive.rs` has one pause site, the hot exit of `clause_region!` (`:1905`); the `RegionEnd::Flowed` arm
(`:1930`) never pauses, and the hot exit pauses for every promoted clause, `THEN` included. Make the decision per
instruction kind: a `pauses_after(kind)` table from the oracle list above, asked at both exits (under the existing
`$debugging`), plus the caller-side pause on a `CALL`'s return (`run/call.rs` `finish_call`) and the INTERPRET one.
`=` (re-execute) must work from the Flowed arm too. Files: `ir/drive.rs`, `run/call.rs`, `run/interpret.rs`, a table
in `ir/` or `trace.rs`. Size S to M.

### Performance risk

The Flowed arm runs for every `Op::Exec` clause. Adding `if $debugging && ...` there is a per-clause branch on a
register-held bool, which the record prices at ~0.5% rexxcps for any conditional in the `Op::Clause` arm
(`oorexx-per-clause-branch-costs`). The `pauses_after` lookup must sit behind `$debugging`. Removing the `THEN`
pause moves the lookup onto the hot exit's existing `$debugging` branch only.

### ooTest rows

None attributable on their own; the `TRACE` debug rows also read `.DebugInput` (item 3).

## 3. Debug input does not reach pauses

### Probe results

`di` (stdin `/dev/null`): stderr identical; stdout ours `after`, oracle `typed after`. Reproduces.

`eof` (`signal on notready`, `trace ?a`, stdin `/dev/null`): ours runs to the end, rc 0; the oracle's debug read
raises NOTREADY, enters the handler (`NOTREADY trapped STDIN`), rc 3.

### Oracle rule

`doDebugPause` reads with `activity->traceInput(this)` (`RexxActivation.cpp:4244`); `Activity::traceInput`
(`Activity.cpp:3240-3263`) calls the `RXSIODTR` exit, else sends `LINEIN` to `.local~DEBUGINPUT`, mapping `.nil` to the
null string. The default `.DebugInput` monitor's destination is `.INPUT` (ours sets the same,
`environment.rs:848-851`), whose `.STDIN` raises NOTREADY at end of input, which the `eof` probe shows reaching a
`SIGNAL ON NOTREADY` trap.

### Fix sketch

In `debug_pause_after_clause` (`run/interpret.rs:202`, the read at `:230`) replace `self.input_line()` with a reader shaped like
`linein_line` (`input.rs:310-325`): `local_route(b"DEBUGINPUT")`, send `LINEIN` under `pinned!`, `.nil` or no answer to
empty. A condition raised by the read (NOTREADY) must propagate into the activation's traps as the `eof` probe shows,
so the read's `Failure` returns through `debug_pause_after_clause`'s `Result`. The send runs Rexx code from the pause
site, as `run_debug_fragment` already does there. A missing `RXSIODTR` exit is out of scope. Size S.

### Performance risk

None: only at a pause.

### ooTest rows

`TRACE` TEST_TRACE_NUMERIC_DEBUG (`alone.txt:37`) per the queued item. The other `TRACE` rows whose bodies set
`.DebugInput~destination(...)` (`ootest/ooRexx/base/keyword/TRACE.testGroup` lines 1003, 1050, 1065, 1091, 1110, 1125,
1152): TEST_TRACE_?, TEST_TRACE_IGNORED, TEST_TRACE_?_OPTION, TEST_TRACE_?A, TEST_TRACE_?R, TEST_TRACE_?I
(`alone.txt` 28-32, 36), all failing on ours. These are candidates only: they also depend on item 2's pause
placement, and none was run with a fix.

## 4. INTERPRET syntax traceback line

### Probe results

`interp` (`interpret "x=1; y=(; z=2"`): ours omits `1 *-* y=(;` and says `at "&1"`; the oracle prints the line and
`at "("`. Both rc 221. Reproduces.

The defect covers every fragment parse error. `i2.rex N` under `signal on syntax`, printing the condition
object's message and traceback:

| N | text | ours | oracle |
|---|---|---|---|
| 1 | `y=(` | 35.1 `at "&1"`, 1 traceback line | 35.1 `at "("`, extra line `4 *-* y=(` |
| 2 | `x=1; y=(  ; z=2` | as 1 | `5 *-* y=(  ;` (to the end of the token where the error was detected) |
| 3 | `x=1` `'0a'x` `y=(` ... | 13.1 `"&1" ('&2'X)` | 13.1 with the byte and `'0A'X`, line `x=1` plus newline |
| 4 | `do forever then` | 27.901 `found "&1"` | `found "THEN"`, line `do forever then` |
| 5 | `say 'abc` | 6.2, no line | 6.2, line `say 'abc` |
| 6 | `x = 1 +` | 35.1 `"&1"` | `"+"`, line `x = 1 +` |
| 7 | `if then` | 35.929, no line | 35.929, line `if then` |
| 8 | `end` | 10.1, no line | 10.1, line `end` |

`interpreply` (REPLY in INTERPRET, 99.924): the oracle prints `4 *-* reply 5;`, ours omits it; message text matches.

### Oracle rule

A translation error raises with the parser's compile frame on the stack; `LanguageParser::createStackFrame`
(`LanguageParser.cpp:866-876`) builds a `FRAME_COMPILE` frame whose traceback is `package->traceBack(clauseLocation)`.
`clauseLocation` is the clause being scanned, from its first token to the last one scanned
(`Scanner.cpp:231`, `:438`, ...), which is why `y=(  ;` keeps the blanks and the `;`. Inserts come from the
raising call: `errorToken` passes `token->displayValue()` (`LanguageParser.cpp:4129-4132`), and the `error(code,
value...)` overloads pass values (`:4141-4171`).

### Fix sketch

Two parts. (a) The traceback line: `ParseError` (`rexx-parse/src/token.rs:28`) carries only `code`, `sub` and the
clause's start `byte`; add the end of the last scanned token, and have `run_fragment` (`run/interpret.rs:28-37`)
record the fragment clause as an inner failure site (line = the INTERPRET clause's line, as the oracle prints) before
returning the `Raised`. Size M (rexx-parse token/scanner plumbing plus one exec site). (b) The inserts: `impl
From<&ParseError> for Raised` (`error.rs:1836-1840`) passes `Vec::new()`. `ParseError` needs a substitution list
filled at each raising site; `grep -rn 'error(\s*[0-9]\+,\s*[0-9]\+)\|ParseError::new(' --include=*.rs
crates/rexx-parse/src | grep -v tests | wc -l` printed 213 sites. Only sites whose message has `&N` inserts need
values. Size L, mechanical. A top-level program parse error is still loud (`lib.rs:3196-3230`) and shares (b).

### Performance risk

None at run time. A larger `ParseError` widens the parser's `Result`; box it if parse benchmarks move.

### ooTest rows

None in `table.txt` or `alone.txt`.

## Full `Activity` field table

`rexx-exec/src/activity.rs`, struct at line 26. "Activity" = activity-scoped or transient across levels in the oracle
too. "Activation, saved" = activation-scoped in the oracle and kept correct in ours by a per-call save/restore or by
an activation tag. "WRONG" = activation-scoped in the oracle, activity-scoped in ours, with a probe.

| field (line) | oracle home | classification |
|---|---|---|
| value_buffer (28) | evaluation stack scratch | Activity |
| locals (34) | `RexxActivation::environmentList` (hpp:645), top-level activation | **WRONG**: `sl1`, `sl2`, `sl6` |
| running (37), suspended (48) | `Activity` activation stack | Activity |
| trace_cache (40) | cache of `running`'s setting | Activity (derived; see debug_pause) |
| spare_activations (58), spare_exposed (61) | allocation reuse | Activity |
| native_handles (66), native_spares (69) | native activation stack | Activity |
| thread (73) | `Activity` thread context | Activity |
| clause_state (78) | per activation | Activation, saved (`CallTail::saved_clause_state`; moved at REPLY) |
| flat_loops (85), flat_top (88), frames (91), flat_spares (115) | `doStack` per activation (hpp:629) | Activation, saved (split per level; moved at REPLY) |
| call_tails (94), parked_calls (97), parked_levels (99), replied_level (102), native_tails (105) | driver machinery | Activity |
| pending_traps (117) | `conditionQueue` per activation (hpp:647) | Activation, saved (each entry tagged with its activation) |
| active_condition (120) | `settings.conditionObj` | **WRONG**: `cond1`, `cond2` |
| pending_additional (130), pending_result (134), pending_rc (139), reraised_object (142), reraise_leaving (145) | raise in flight | Activity |
| current_case_text (158) | `DoBlock` case value (`DoBlock.hpp:78`) | **WRONG**: `ct2`, `ct4` |
| indent_offset (168), activation_indent (173) | `settings.traceIndent` | Activation, saved (`CallTail` saved_offset/saved_base) |
| failure_site (176), failure_sites (179), failure_frame (182), failure_frames (186), failure_origin (205), failure_propagated (209), failure_reraised (211), native_reraise (191) | unwinding in flight | Activity |
| input_dispatch (195), input_dispatch_trapped (197), input_dispatch_syntax (201) | command redirection in flight | Activity |
| clause_line_override (215) | INTERPRET activation's line | Activation, saved (`CallTail` saved_line) |
| fragment_depth (218), fragments (220), fragment_clause (223) | INTERPRET activations | Activation, saved (tagged with `owner`) |
| debug_pause (228) | `RexxActivation::debugPause` (hpp:632) | **WRONG**: `dbgcall` |
| routing_trace (233) | trace delivery in flight | Activity |
| depth (235), max_depth (236), stack_entry (239), stack_first (242), stack_deepest (243) | native stack measurement | Activity |
| procedure_permitted (247), region_procedure_permitted (250) | `procedureValid` flag | Activation, reset at every entry |
| call_context (252) | argList/messageName per activation | Activation, saved (`CallTail` saved_context; `replyargs` matches 5/5) |
| last_name (255) | name cache | Activity |
| random_seed (258) | `RexxActivation::randomSeed` (hpp:648) | **WRONG**: `rnd` |
| elapsed_anchor (267), pending_elapsed_reset (274) | `settings.elapsedTime`, `elapsedReset` | **WRONG**: `c2`, `c5`, `int`, `reply` |
| requires_installing (277) | `Activity::requiresTable` (Activity.hpp:375) | Activity |
| pin_depth (280), driver_pins (282), native_park (285), native_call (288), blocked (291), resuming (294), first (296), root_then (298), drive_floor (301), sliced (304), root_end (306) | scheduler | Activity |
| number (309), spawner (312) | activity identity | Activity |
| splits_owed (316), failed_sends (319) | REPLY / send in flight | Activity |
| guard_waits (322), guarded_send (325), guard_exec (327), guard_posted (330), when_parked (332) | `Activity::guardSem` (Activity.hpp:399) | Activity |
| asleep (334), woken_by_halt (336), semaphore_wait (339), waiting_traced (342) | waits | Activity |
| pins (344), sharing_tag (347) | feature-gated bookkeeping | Activity |

`ClauseState` (`clause.rs`) is a field of `Activity` and is covered by `clause_state`'s row.

Beyond the three known, three fields sit on the wrong side: `active_condition`, `current_case_text`, `debug_pause`.
`locals` makes four with the known three; the queued items already name it.

## Appendix: probes

Each in `/tmp/claude-1000/p61/sb/probes/<dir>/` (removed with the worktree). Run: `run.sh DIR FILE [STDIN]`.

`c2/c2.rex`:
```
call time 'r'
x = 0
do i = 1 to 300000; x = x + 1; end
call r
exit
::routine r
t = time('r')
say 'routine r zero:' (t = 0)
```
`c5/c5.rex` (with `c5/extr.rex` = `t = time('r'); return t`):
```
call time 'r'
x = 0
do i = 1 to 300000; x = x + 1; end
t = extr()
say 'external zero:' (t = 0)
```
`int/int.rex`:
```
call time 'r'; call SysSleep 0.3; x = time('E'); call s
say (time('e') >= 0.3); exit
s: tt = time('R'); return
```
`reply/reply.rex`: the queued item's REPLY probe, verbatim.

`rnd/rnd.rex`:
```
say random(1,1000,7)
say .t~new~m
say random()
call s
say random()
::class t
::method m
  return random(1,1000,11)
::routine s
  call random 1,1000,13
```
`sl1/sl1.rex`:
```
o = .t~new
o~m
say 'main endlocal' endlocal()
::class t
::method m
  say 'm setlocal' setlocal()
```
`sl2/sl2.rex`:
```
o = .t~new
say 'got' o~m
call SysSleep 0.2
say 'main endlocal' endlocal()
::class t
::method m
  call setlocal
  call value 'P6X', 'yes', 'ENVIRONMENT'
  reply 1
  say 'cont endlocal' endlocal()
```
`sl6/sl6.rex`:
```
call value 'P61X', 'orig', 'ENVIRONMENT'
call r
say 'main sees' value('P61X',,'ENVIRONMENT')
exit
::routine r
  call setlocal
  call value 'P61X', 'routine', 'ENVIRONMENT'
  return
```
`sl3/sl3.rex` (oracle abort):
```
call r
say 'main sees' value('P61X',,'ENVIRONMENT')
call s
say 'after internal' value('P61X',,'ENVIRONMENT') endlocal()
exit
s:
  call setlocal
  call value 'P61X', 'internal', 'ENVIRONMENT'
  return
::routine r
  call setlocal
  call value 'P61X', 'routine', 'ENVIRONMENT'
  return
```
`cond/cond1.rex`:
```
signal on syntax
say 1/0
exit
syntax:
  call r
  say 'back'
  exit
::routine r
  signal on syntax name inner
  raise propagate
  say 'no raise'
  return
inner:
  say 'routine trapped' condition('O')~code
  return
```
`cond/cond2.rex`:
```
call r
signal on syntax
raise propagate
say 'main: no raise'
exit
syntax:
  say 'main trapped' condition('O')~code
  exit
::routine r
  signal on syntax name h
  say 1/0
h:
  say 'routine handled' condition('O')~code
  return
```
`cond/cond3.rex`:
```
call outer
say 'main after outer'
exit
outer:
  signal on syntax
  say 1/0
  return
syntax:
  call s
  say 'no propagate from s'
  return
s:
  raise propagate
```
`casetx/ct2.rex` (`casetx3/ct4.rex` is the same with `when 'zz'` in the absorbed position; `casetx3/ct3.rex` has
`g: return 'a'`):
```
select case 'a'
  when g() then when 'a' then say 'absorbed matched a'
  otherwise say 'other'
end
say 'done'
exit
g:
  select case 'zz'
    when 'zz' then return 'a'
    otherwise return 'y'
  end
```
`dbgcall/dc.rex`, stdin `call s`, then alternating `.stderr~lineout('Tn')` and empty lines:
```
trace ?r
x = 1
say 'main'
exit
s:
  y = 2
  say 'in s'
  return
```
`replyargs/ra.rex` (5 runs, identical on both):
```
o = .t~new
say 'got' o~m('A1', 'A2')
call SysSleep 0.2
::class t
::method m
  reply 1
  say 'cont args' arg() arg(1) arg(2)
  use arg x, y
  say 'cont use' x y
  parse arg p
  say 'cont parse' p
```
`pa/pa.rex` and `di/di.rex`: the queued items' probes, verbatim. `eof/eof.rex`, stdin `/dev/null`:
```
signal on notready name nr
trace ?a
say 'a'
say 'b'
exit
nr: trace off; say 'NOTREADY trapped' condition('D'); exit 3
```
`pakinds/pakinds.rex`, stdin 60 pairs of `.stderr~lineout('pN')` and an empty line:
```
trace ?a
say 'c1'
numeric digits 9
say 'c2'
address command
say 'c3'
drop zz
say 'c4'
say 'c5'
push 'q'
parse pull qq
say 'c6'
x = 1
say 'c7'
nop
do i = 1 to 1
  say 'c8'
end
if x = 1 then say 'c9'
select
  when x = 1 then say 'c10'
  otherwise nop
end
call sub
say 'c11'
.nil~string
say 'c12'
interpret "say 'c13'"
say 'c14'
signal lbl
lbl:
say 'c15'
exit
sub: return
```
(An `options 'X'` line was removed after ours refused it: `rexx-exec: OPTIONS is not implemented (Phase 5)`, rc 120.)

`interp/interp.rex`: `interpret "x=1; y=(; z=2"`. `interp2/i2.rex`, run as `i2.rex N` for N in 1..8:
```
signal on syntax
parse arg n
select
  when n = 1 then interpret "y=("
  when n = 2 then interpret "x=1; y=(  ; z=2"
  when n = 3 then interpret "x=1" || '0a'x || "y=(" || '0a'x || "z=2"
  when n = 4 then interpret "do forever then"
  when n = 5 then interpret "say 'abc"
  when n = 6 then interpret "x = 1 +"
  when n = 7 then interpret "if then"
  when n = 8 then interpret "end"
  otherwise nop
end
exit
syntax:
  say 'code' condition('O')~code 'msg:' condition('O')~message
  say 'pos' condition('O')~position 'traceback' condition('O')~traceback~items
  do l over condition('O')~traceback; say '  tb:' l; end
```
`interpreply/ir.rex`:
```
say .t~new~m
::class t
::method m
  interpret "x=1; reply 5; y=2"
  say 'after'
```
