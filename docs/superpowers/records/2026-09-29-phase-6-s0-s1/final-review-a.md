# Phase 6 S0/S1 final review, slice A (code)

Range `0d911ab7c..70030c6ed`. Read-only review; no code edits. Builds under
the session scratchpad `p6-final-a/` from `git archive` of each sha, trees
touched, separate `CARGO_TARGET_DIR`s: `tgt-new` (70030c6ed), `tgt-old`
(0d911ab7c), `tgt-pin` (70030c6ed, `--features pinning`, also holds the
in-process harness below), `tgt-cap` (70030c6ed with `MAX_ACTIVATION_DEPTH`
and `MAX_EVAL_DEPTH` both raised to 100,000,000), `tgt-dbg` (debug).
Oracle: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
from a fresh empty directory per probe, stdout/stderr/rc as three files
(`run3.sh`). Oracle timeouts are `timeout -s KILL`: the oracle ignores
SIGTERM while blocked in a GUARD wait (one probe sat 13 minutes under a
plain `timeout 60`).

Status: IN PROGRESS. Sections are appended as they are reached.

## Method

Per-task reviews already covered hunks. This slice combines features across
task boundaries and runs the combination three ways (oracle, head, base),
then runs every probe in-process twice more through a throwaway harness
`tests/review_probes.rs` added to the scratch copy of the head tree:
`run_program` and `run_program_collect_every_alloc` (collect at every
allocation), under `--features pinning`, printing the `PinReport`.

Defect classes hunted, as named in the brief: (1) a path still recursing
on the Rust stack that is not counted as pinned; (2) a value held across
an allocating step without a root; (3) a begin/finish split duplicated
instead of shared; (4) witnesses matching the oracle without running their
claimed paths; (5) the two engines disagreeing within one binary.

For class (1) the instrument is not the report alone (a flat report and an
unpinned recursion print the same `[]`): a generated set of 44 "shapes",
each recursing `f(n) -> f(n+1)` through one construct, is run at depth
1,000,000 on the cap-lifted build (a recursing shape overflows the 512 MiB
interpreter stack; a stackless one finishes) and at depth 3 with
`call SysSleep 0` at the bottom under the pinning build (the report then
names the frames above the park). A shape that overflows AND reports `[]`
is a class (1) defect.

Probe files: `scratchpad/p6-final-a/probes/p01..p27*.rex`, shapes in
`shapes-deep/` and `shapes-pin/` (generator `gen_shapes.py`), outputs in
`out/<probe>/{oracle,new,old}.{out,err,rc}`, harness output
`harness2.txt`, `harness-shapes.txt`, depth runs `deep-cap.txt`,
`cap20.txt`.

## Findings

### Important

**A1 (class 1). An external `.rex` routine call recurses on the Rust stack
and is not pinned; the pinning report calls a park under it flat.**
`rust/crates/rexx-exec/src/lib.rs:1966` `enter_external_program` ->
`run_loaded` (`lib.rs:2105`) runs the callee program as a nested driver
inside the caller's `ops_loop`, with no `pinned!` around it. The same holds
for every other `run_loaded` caller reached from a running program:
`enter_library_program` (`lib.rs:2018`, `Resolved::Library`) and the
`::REQUIRES` prologue runs (`install.rs:677`, `install.rs:968`).

Witness, pinning build (`shapes-pin/s18_external.rex` with `extf.rex` in
the working directory):

    main:  say f(0)   ::routine f  ... return extf(n+1)
    extf.rex: use arg n; if n >= 3 then do; call SysSleep 0; return 'ok'; end; return extf(n+1)

reports `park SysSleep x1 under []`, the same line a stackless path gives.
That it recurses: `shapes-deep/s18_external.rex` (extf self-recursion
200,000 deep) on the cap-lifted build ends in
`thread 'rexx-interp' has overflowed its stack` (rc 134, 8.7 s), where
every S1 stackless shape (call, function, send, ~new/INIT, Message~send,
sendWith, Routine~call/callWith, start~result, plain DO, DO OVER, PARSE
VALUE source, IF/WHEN tests, `[]`, attribute assign, OF factory argument)
reaches 1,000,000 and answers `ok` (`deep-cap.txt`).

Why it matters for S2: the report is the instrument the stage closes on
("a frame is pinned iff it is a Rust frame between scheduler and driver");
an activity parked inside an external routine would be unparkable, and the
instrument says otherwise. Fix direction: push a pin kind (say
`PinKind::Program`) around `run_loaded` when an activation is already
running, add the external-routine case to `FRAME_PROBES` in
`tests/concurrency_tests.rs`, and (separately, see B1) count external
program levels in the activation depth.

### Minor

(none yet)

## Pre-existing on 0d911ab7c, not counted against the stage

Each of these reproduces identically on `tgt-old`; listed because a reader
of the review would otherwise rediscover them.

**B1. External routine recursion has no depth cap: a native stack overflow
instead of Error 11.** `extf.rex` recursing on itself 200,000 deep aborts
both builds with `thread 'rexx-interp' has overflowed its stack` (rc 134);
at 20,000 both answer `ok` where every in-program shape raises 11.1 at the
10,000 cap. `enter_external_program` does not pass the
`MAX_ACTIVATION_DEPTH` check in `begin_call` (`run/call.rs:825`) because
it returns `Begun::Done` before it. D19/I6's whole point was that unbounded
recursion becomes a reportable condition; this path is outside it.

**B2. REPLY continuation runs at program end, so state it writes is not
visible to a later guarded send** (`probes/p04_reply_in_arg.rex`,
`p04b_reply_where.rex`): oracle prints `after argument-value-long`, both
builds `after none`, and the continuation's `say` comes after `end main`.
Same mechanism as the queued reply2 ordering item; recorded here only
because the observable is a value, not an order.

**B3. `Routine~call`/`callWith` run the routine with `.context~name` =
`CALL`** where the oracle answers the routine's own name (`RF`)
(`probes/p22_native_resumables.rex` lines 5-6 of stdout).

**B4. `Message~start(...)~result` after the message's method raised**:
the oracle's traceback has one frame and it also prints the error report on
stderr from the message's thread; both builds give a three-frame traceback
(`Compiled method "RESULT" with scope "Message"` and the two callers) and
nothing on stderr (`probes/p22b_native_errors.rex`, the `h4` block).
S2 territory (a real activity for `start`).

**B5. `Object~setMethod` and `Object~run` are not implemented and fall
through to the receiver's `UNKNOWN`** (every `shapes-pin/*.rex` shows a
second `SysSleep` arrival `under [Unknown]` from `.o~setMethod('DYN', .mth)`
reaching `M~unknown`; `s20_run_method.rex` ends in 97.1). Not Phase 6
scope.

**B6. Hash-collection subclass `~new('string')`** refuses with 93.923 where
the oracle passes the argument to INIT (first version of
`probes/p01_init_mixed.rex`); this is the queued stringtable/hash
`~new` items, seen through `Table`.

**B7. Trace indent after `LEAVE outer` from a nested `DO WHILE`**
(`probes/p10_trace_mixed.rex`): the clause after the left loop echoes two
columns deeper than the oracle on both builds. Matches the recorded
upstream `traceIndent` two-exit-path defect (the oracle under-indents).

## Oracle behaviour met on the way

* `probes/p23_trap_nested.rex`: the oracle segfaults (rc 139) after
  `ha z2 900`. The clause `x = fa(1) + fb(2)` queues `ua` and `ub`, and
  handler `ha` raises `ub` again through `fb(9)` while the queued `ub` is
  still pending, which is the recorded "same CALL ON condition queued twice
  at one boundary" crash. The two-line reductions without the double queue
  (`t4/crash1.rex`, `crash2.rex`) run clean on both sides. Both builds run
  the whole program and agree with each other.
* `GUARD ON WHEN` whose expression sends to the same object
  (`shapes-pin/s42_guard_when.rex`) and `GUARD OFF WHEN` false on a single
  activity (`probes/p05_guard.rex`, first version) deadlock the oracle;
  this crate refuses loudly (`Loud::guard_when_false`, rc 120).

## Probes that agree with the oracle on head

Byte-identical on all three descriptors, head and base alike unless noted:
p01 (Table-subclass INIT, send inside a CALL argument inside a plain DO,
CALL ON user delivered mid-loop, TRACE I), p02/p02b (SIGNAL ON SYNTAX /
USER out of a 300-deep chain mixing function, CALL, send and ~new/INIT,
with `stackFrames~items`), p03 (EXIT in a method entered from a function
argument, RETURN out of a DO in a method, RAISE SYNTAX in INIT reached from
a ~new in a DO under SIGNAL ON), p06 (INTERPRET building calls and sends,
nested INTERPRET in a method, recursive INTERPRET), p07 (PROCEDURE EXPOSE
of stems and indirect lists through label, function and routine-via-method
frames), p08/p08b (condition object, SIGL, stackFrames lines/names/types
and traceback through label, routine, DO and method frames; untrapped
traceback to stderr), p09 (Error 11 through a cycle of function, CALL,
send, ~new and INIT-time recursion), p11 (nested calls/sends in builtin
and routine argument positions, omitted arguments, CALL ON drain of two
conditions in one clause -- base differed, head agrees), p12 (Message~send,
sendWith, send, start~result, Routine~call/callWith, error through
sendWith), p13 (labelled loops with LEAVE/ITERATE across stackless calls
and methods), p14 (NUMERIC/ADDRESS/SIGL/RC inheritance into labels and
routines), p15 (PROCEDURE as first instruction after CALL, function, nested
function, INTERPRET and routine-via-method entries; error 17 traceback),
p16 (CALL ON ERROR/FAILURE with commands in routine, label and method
frames), p17 (`.context` name/line/executable/parentContext across mixed
frames), p18 (EXIT inside INIT, with and without a value), p19 (RESULT and
SIGL after CALLs with and without a value, inside a method), p20 (two
engines: `o~m(f())` vs `o~~m(f())` vs `~~` in an expression vs
`m:super`, with CALL ON drain, TRACE I and error tracebacks through both),
p21 (DO OVER ... FOR f() with the OVER value a fresh array, controlled
loops with allocating TO/BY/WHILE/UNTIL calls), p25 (FORWARD class/
arguments/array/to/continue from stackless-entered methods), p26 (omitted
arguments through sendWith, Message, send, CALL and callWith), p27 (builtin
with function arguments inside a CALL ON handler -- base differed, head
agrees).

Collect-on-every-allocation: every probe above and every shape gives the
same stdout/stderr/rc as its plain run (`harness2.txt`,
`harness-shapes.txt`; collections per program 2 to 58,767, p01 and p11
with 2 and 6 only because they fail early on B6/typo versions -- rerun
figures are in the file).

## Not reached

(filled at the end)
