# Task 8.2 report: conditions and tracebacks

Status: DONE_WITH_CONCERNS. Base `35db1719c`. Code `4acd606e9`, `12ef5a35c`, `918e67f90`,
`9f8be0ec2`, `c28564d1e`, `a52fa561b`; records and this report in the commit that adds it.
Records: `docs/superpowers/records/2026-10-18-phase-9d/t8.2/` (`probes/<item>/` holds each probe
with its `base`, `head` and `oracle` descriptors).

## Items

### 2026-10-10-leave-iterate-error-not-trappable.md (closed, `4acd606e9`)

A `LEAVE`/`ITERATE` matching no loop reached the activation boundary as `Flow::Leave`, and
`apply_flow`'s 28.x went straight out of the driver. The driver now offers it to the traps as any
clause's error (`ir/drive.rs`). Witness `corpus/lang/leave_iterate_trapped.rex`: 28.1-28.4 in the
program, in an internal call and inside an `INTERPRET`, SIGL at the clause. Red at base (rc 228),
identical to the oracle at head.

### 2026-10-10-raise-novalue-return-into-call-on-any.md (closed, `4acd606e9`)

`trap_at_depth`'s `ANY` fallback now applies `TrapHandler::canHandle`: a `CALL ON ANY` does not take
SYNTAX, NOVALUE, LOSTDIGITS, NOMETHOD or NOSTRING (`run/condition.rs`). Measured besides the item:
`raise nomethod return` into a `CALL ON ANY` is 97.5 on the oracle, as it now is here. Witness
`corpus/lang/raise_return_call_on_any.rex` (NOVALUE, LOSTDIGITS, NOSTRING skipped; USER, NOTREADY
taken; NOMETHOD 97.5). The KNOWN GAP row is removed from `phase-4-exclusions.txt`.

### 2026-10-10-directory-setmethod-unknown-traceback-line.md and gate:phase-9a-gate.md:471 (closed, `a52fa561b`)

`processUnknown` calls the primitive UNKNOWN without a frame for a Stem (any class),
a VariableReference, and a Directory or StringTable of the base class (`StemClass.cpp:277`,
`VariableReference.cpp:241`, `HashCollection.cpp:989`). `unknown_or_nomethod` runs that primitive
directly for those (`dispatch.rs` `frameless_unknown`); a Directory subclass and an explicit
`~unknown` send keep the level, as on the oracle. Witness `corpus/lang/unknown_frameless_traceback.rex`
(tracebacks read through a trap); red at base. The gate line is in `t8.2/closed.tsv`.

### 2026-10-02-while-condition-failure-traceback.md (closed, `a52fa561b`)

A WHILE test failing on a re-test is blamed on the END or ITERATE that re-entered the loop, for the
test's own failure (34.x, which also differed: `probes/while-retest/w4`, `w6`, `w8`) and for a
routine it calls; the WHILE, COUNTER and UNTIL header paths make the END the frame readers' clause
(`run/loops.rs`), so `.context~stackframes` and a trapped condition's traceback name it too
(`probes/while-retest/t1.rex` showed `4 *-* nop` at base). Witness
`corpus/lang/while_retest_blame.rex` (DO, END and ITERATE blame, program and INTERPRET); red at base.

### 2026-10-10-propagated-condition-extra-traceback-line.md and 2026-10-10-reply-propagated-condition-traceback-line.md (closed, `12ef5a35c`)

The oracle reports an untrapped `RAISE PROPAGATE` from the condition object's TRACEBACK
(`Activity::display`). Here the levels from the propagate to the trapping level (the outermost
internal call sharing the active condition) now seal no site, a RETURN re-raise at or below the
trapping level names its own clause's line in the report header, and after a REPLY the object's
lines beyond the trapping level are appended (`run/condition.rs`, `run.rs` `seal_site_level`).
Witnesses `corpus/lang/raise_propagate_traceback.rex` (internal call plus INTERPRET plus ::routine,
header line), `raise_propagate_return_line.rex`, `reply_propagate_traceback.rex` (identical to the
oracle in 8 of 8 runs; the probe `rf1d.rex` gives the wall run's bytes under `sim:1`, `sim:7,fifo`,
`sim:3,uniform:0.2` and `sim:5,pct:3,k=40`). All red at base. Every shape in `probes/propagate/` is
identical at head.

### 2026-10-21-lineout-nil-names-file.md (closed, `12ef5a35c`)

Not a rendering of `.nil`: the oracle reports a condition raised inside a native activation by its
package and with no line (rexxref `condtra.xml:769-774`, POSITION and PROGRAM are where the condition
was raised; a native method has no line, and the Stream methods' package is REXX). Two defects:
`external_package_path` answered the running program's path for a method of the REXX library
package (`install.rs`), and the write-string refusal was not a marshalling raise
(`dispatch/stream.rs`). Witness `corpus/lang/stream_boundary_report.rex` (88.909, 88.922, 88.907,
inside `Stream~say`); red at base. Measured besides the item: `.stdout~description(1)`,
`~qualify(.nil)` and `~lineout('x', 1, 2)` named the program too (`probes/lineout-nil/`).

### 2026-10-20-call-on-delayed-reraise.md (closed, `918e67f90`)

rexxref (`condtra.xml:443-461`): while a CALL ON trap is delayed, the same condition raised again
"is either ignored (for ERROR and FAILURE) or (for the other conditions) any action ... is delayed
until ... A RETURN is made from the subroutine ... and the subroutine is called again". So the
item's USER shape running the handler twice was right, and ERROR and FAILURE were wrong. A RAISE
RETURN/EXIT of ERROR or FAILURE into a caller whose trap is delayed is now ignored, as a command's
own already was (`call_trap_delayed_at`). The oracle segfaults on the shape, so the witness is the
unit test `a_condition_raised_again_while_its_call_trap_is_delayed` (ERROR, FAILURE once; USER,
NOTREADY twice), red at base on ERROR and FAILURE (`probes/delayed/`).

### 2026-10-20-halt-after-signal-on-syntax.md (closed, `918e67f90`)

The halt request was raised inside the SIGNAL ON SYNTAX trap, where `deliver_pending_traps` runs the
failed clause's CALL ON handlers, after the failure levels were cleared. That delivery now leaves a
halt request queued, and the label's clause raises it (`run/condition.rs` `deliver_pending`). `h5`,
`h6`, `h7` (CALL ON HALT), `h8` (a USER and the halt in one clause) and `h9` are identical to the oracle under SIGINT (`probes/halt/`). Witness `tests/signals.rs`
`sigint_pending_at_a_syntax_trap_is_raised_at_the_label` (both shapes, the oracle's bytes). `h1`,
`h3` and `sigint2` still differ, identically at base: each is a halt during a redirected command,
where the oracle raises 98.923 and this crate abandons the wait (Deviation 12).

### 2026-10-19-condition-deviation-candidates.md (closed, `9f8be0ec2`)

RAISE RESULT implemented: a non-SYNTAX `RAISE ... RETURN|EXIT expr` sets RESULT under a CALL ON
and a SIGNAL ON trap (`run/condition.rs`). ooTest is unchanged by it (suite below).
`corpus/lang/raise_result_and_sender.rex` pinned the oracle's missing entry; its
`show` routine now prints `(not compared)` for a non-SYNTAX condition that was not propagated
(`c28564d1e`, sourceline file regenerated). The rulings on the failed handler call and on the 16.1 frame were already followed;
each has a unit test (below).

### 2026-10-01-message-start-result-traceback.md (open, reported)

Stopped: the oracle's answer is a data race. Probe `probes/message-start/msgstart.rex`
(`o~start('boom')~result`, `boom` does `say 1/0` on line 6). The oracle's outcome is a race whose
frequency depends on the machine: here 40 of 40 runs (20 serial, 20 four at a time,
`t8.2/r1/message-start/`) gave line 3 in both reports, interleaved or not; the review measured 12 of
20 interleaved with line 3 twice and 8 of 20 sequential, line 6 then line 3, byte-identical to this
crate. Head, 20 runs, one outcome: line 6, then `result`'s re-raise names line 3. `MessageDispatcher::handleError` calls
`message->error(c)`, which wakes the `result` waiter, before `activity->error` displays the same
condition object; the waiter's `Activity::reraiseException` writes POSITION 3 into that object
first. rexxref (`fundclasses.xml:1656-1661`) says only that `result` raises the error again; POSITION
is "the line number ... at which a condition was raised" (`condtra.xml:769-771`), which for the
started activity's report is 6. This crate follows that. Listed below as a new oracle defect.

## Deviation candidates (for the controller; none written)

**1. A failed trap handler call taken by SIGNAL ON SYNTAX signals at once.**
Rule: the CALL a CALL ON trap makes runs at the end of the raising clause; if it fails, the SYNTAX
condition is raised then, SIGNAL ON SYNTAX ends that clause, and SIGL names it. No later clause runs.
Measurement (`probes/condition-candidates/a.rex`, `a2.rex`, `a3.rex`): oracle `line4` then
`s 40 4` (and `s 43 4` for a missing label, `s 43 4 43.1` for NOTREADY); this crate `s 40 3`,
`s 43 3`, `s 43 3 43.1`. rexxref `condtra.xml:438-440` ("the CALL trapname proceeds in the usual
way when the instruction completes") and `condtra.xml:468-471` ("the condition is raised immediately
upon detection. If SIGNAL ON traps the condition, the current instruction is ended").
Witnessed by `run::tests::conditions::a_failed_trap_handler_call_signals_at_once`, which asserts
`s 3` for a handler naming a builtin and for one naming no routine.

**2. A non-SYNTAX RAISE RETURN or EXIT value is the condition object's RESULT (oracle bug).**
Rule: RESULT holds "Any result specified on the RETURN or EXIT options of a RAISE instruction".
Measurement (`probes/condition-candidates/b.rex`, `r.rex`): oracle `mh hasResult 0 The NIL object`
for RETURN, EXIT and a SIGNAL ON trap; this crate `mh 1 first`, `mh 1 second`, `sh 1 third`, and no
entry for a tail with no value; SYNTAX already agreed (`s.rex`, `1 x` on both). A second
observable follows: a tail-less `RAISE PROPAGATE` of the trapped USER condition returns the original
RETURN value to the caller (`t8.2/r1/pn.rex`: oracle `h 0 The NIL object` / `back RESULT`, this
crate `h 1 w1` / `back w1`), since the propagate keeps the condition's RETURN information (rexxref
`instrc.xml:2913-2917`; the oracle's own propagate branch reads RESULT back,
`RexxActivation.cpp:1801-1804`). Inside an INTERPRET the entry is set too (`t8.2/r1/ut2.rex`: oracle
`oh 0 The NIL object`, this crate `oh 1 from-interpret`). rexxref `condtra.xml:738-741`. Oracle cause: `RexxActivation::raise` passes the member `result`, not
`resultObj`, to `createConditionObject` (`RexxActivation.cpp:1827`; the SYNTAX branch at `:1851`
passes `resultObj`).
Witnessed by `run::tests::conditions::a_raise_tail_value_is_the_condition_objects_result`, which
asserts the entry, the propagate's returned `w1` and the INTERPRET shape;
`corpus/lang/raise_result_and_sender.rex` leaves the affected lines uncompared.

**3. A 16.1 from a SIGNAL trap inside INTERPRET keeps the PROGRAM frame's file name.**
Rule: a frame of type PROGRAM is the main program's, whose name is its file; only an INTERPRET
frame's name is empty.
Measurement (`probes/condition-candidates/c.rex`): oracle `sf PROGRAM 4 []`, this crate
`sf PROGRAM 4 [<path>/c.rex]`. rexxref `utilityclasses.xml:9595-9601`.
Witnessed by `run::tests::conditions::a_missing_signal_label_inside_interpret_keeps_the_programs_name`,
which runs the program as `/w/c.rex` and asserts `sf PROGRAM 4 [/w/c.rex] 16.1`.

## Other departures from the oracle (not Deviations; for a ruling)

* **Message~start report position** (item above). Oracle: a race between line 3 in both reports
  and line 6 then line 3. Here: always 6 in the started activity's, 3 in `result`'s. rexxref `condtra.xml:769-771`, `fundclasses.xml:1656-1661`.
* **An UNKNOWN a Stem subclass defines.** `x = .ss~new('S.'); say x~nosuch` with
  `::method unknown` on `ss`: oracle 97.1 (it never looks the override up, `StemClass::processUnknown`
  forwards to the value), here the override runs (`mine NOSUCH`, `7`). rexxref `provide.xml:514-519`
  and `:527-532`: the object's UNKNOWN method is called. Unchanged by this task
  (`t82-scratch/unk/sov.rex`, not in the records).
* **ERROR or FAILURE raised twice while delayed** (item above): the oracle segfaults, so it has no
  answer; this crate follows `condtra.xml:443-461`.

## New oracle defects

1. **Message~start's error report reads a position another thread wrote.** Repro
   `probes/message-start/msgstart.rex`. Oracle: the started activity's report says
   `running ... line 3`, the line of `say m~result`, when the waiter's re-raise wins the race (40 of
   40 runs on this machine, 12 of 20 in the review's), and line 6 otherwise. Expected: line 6, where
   the condition was raised. Cause: `MessageDispatcher::handleError` (`MessageDispatcher.cpp:64-72`) calls
   `message->error(c)` before `activity->error(...)`, so the waiting `result` runs
   `Activity::reraiseException` (`Activity.cpp:1330-1341`), which puts its own POSITION, PROGRAM and
   PACKAGE into the same condition object the started activity then displays.
2. **A Stem subclass's UNKNOWN is never run.** Repro above. Oracle: 97.1. Expected (rexxref
   `provide.xml:514-519`): the subclass's UNKNOWN answers. Cause: `StemClass::processUnknown`
   (`StemClass.cpp:277-281`) sends to the stem's value without the method lookup
   `RexxObject::processUnknown` makes; `StringHashCollection::processUnknown` guards the same bypass
   with `isBaseClass()`.

## Found, not fixed

* The traceback line of the clause running an INTERPRET (`interpret arg(1)`) takes the fragment's
  nesting indent when the failing fragment clause is nested, in a trapped condition's TRACEBACK:
  oracle `15 *-*   interpret arg(1)`, here `15 *-*     interpret arg(1)`
  (`probes/while-retest/i1.rex`, identical at base). `clause_of` reads the live
  `current_value_indent` for the running activation; `FragmentLevel::indent` holds the INTERPRET
  clause's. The witness collapses blanks there for that reason.
* Inside a Directory subclass's UNKNOWN-run entry method, `.context~stackframes` lacks the
  `METHOD UNKNOWN` frame the oracle lists (`probes/unknown-frame/sf.rex`, identical at base).
* A plain controlled loop's re-test (no WHILE, UNTIL or COUNTER) keeps the last body clause as the
  frame readers' clause: storing the END at every pass cost churn +0.27% and sayloop +0.29%
  (`callgrind-run2.txt`). No probe showed a difference: the advance of a numeric control variable
  runs no Rexx, and its failure's report is blamed separately. An object-controlled loop whose `+`
  method reads `.context~stackframes` was not measured.

## Commands

All from `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t82-target`, at
`a52fa561b`'s tree.

| command | result |
|---|---|
| `cargo fmt --all --check` (`checks-fmt.txt`) | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-exec -p rexx-parse --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test collect_stress --test signals --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions` (`checks-touched.txt`) | 101: `--lib` 1173 passed; `signals` 42 passed; `collect_stress` 71 passed, 1 failed on `call_site_routine_copy_lifetime.rex` and `call_site_fresh_routine_raises.rex`, as at base; every other binary passed |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 0, 29 passed, 1 ignored |
| `memcap 16G cargo test -j 4 --release -p rexx-parse --test sourceline_oracle` (`checks-srcl.txt`) | 0 |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t8.2/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` (`suite/`) | 0; against `t8.1/suite/tests.tsv` the one changed row is `base/rexxutil/SysFileTree TEST_TREE_DIRECTORY`, oracle column `failed` (the known flake). A run at `c28564d1e` gave no changed row. |

## Perf

`callgrind.txt`: `bench-programs/callgrind.sh -B 0.5 -j 4 -o .../cg-9d/t8.2-run3 -p "startup
interpret parse churn sayloop" pin=.../rexx-run parent=.../cg-9d/t8.1/rexx-head
head=.../cg-9d/t8.2/rexx-head3`, exit 0. Own delta (head minus parent, against the pin):

| program | whole process | libc subtracted |
|---|---|---|
| startup | +0.0048 | +0.0019 |
| interpret | -0.0008 | -0.0022 |
| parse | +0.0003 | 0.0000 |
| churn | +0.0326 | +0.0530 |
| sayloop | +0.0032 | +0.0010 |

Against the pin every program stays under +0.5% (largest churn +0.1870%). churn's delta is
`begin_invoke` (+201K instructions in `cgdiff.py`), which this task does not touch. The first run
(`callgrind-run1.txt`, exit 1: churn +0.5171% against the pin) had `invocable` outlined from
`begin_invoke` (+1.99M) and the per-pass END store; the second (`callgrind-run2.txt`) still had the
outlining. `a52fa561b` removes both. Superseded by Fix round 1's figures: the remaining churn delta
was the second `seam::clear` call site this task added.

## Concerns

1. (Closed in Fix round 1.) churn's own delta was +0.0530% with libc subtracted, from this task's
   second `seam::clear` call site changing `begin_invoke`'s inlining.
2. `corpus/lang/raise_result_and_sender.rex` was edited so its non-propagated USER lines no longer
   compare RESULT;
   the controller may prefer a licensed-divergence row once Deviation candidate 2 is written.
3. The message-start item stays open (oracle race), and the Stem subclass UNKNOWN departure is
   unchanged; both need a ruling.
4. The INTERPRET clause's traceback indent and the subclass UNKNOWN stack frame above are
   pre-existing and not queued.

## Fix round 1

Review `task-8.2-review.md`; brief `task-8.2-fix-r1.md`. Code `77fded0db`; records `t8.2/r1/`.

* **I1.** The RESULT unit test now also asserts the tail-less `RAISE PROPAGATE` returning `w1` to the
  caller and a raise inside an INTERPRET; Deviation candidate 2 above states both, with both answers.
* **I2.** `corpus/lang/stream_boundary_arity.rex`: `.stdout~description(1)` untrapped. Identical to
  the oracle at head; with `install.rs`'s library-package check reverted it reports
  `Error 88 running <path>:` for `Error 88 running REXX:` (`r1/mutants/A-install-rs.*`).
* **I3.** Re-measured (`r1/message-start/outcomes.txt`, `outcomes-parallel.txt`): oracle 20 of 20
  serial and 20 of 20 four-at-a-time runs with line 3 twice; head 20 of 20 line 6 then line 3. The
  item, the departure and the oracle-defect entry above now describe a race with both measured
  frequencies.
* **M1.** The frameless UNKNOWNs no longer call `seam::clear`: each primitive's work is split from
  its native wrapper (`hash_unknown`, `stem_unknown`, `reference_unknown`) and `unknown_or_nomethod`
  runs it directly, as `processUnknown` does without a send. Round 1's second `seam::clear` call also
  broke `tests/dispatch_seam.rs`' one-chokepoint count, which round 1 did not run; it passes now.
  Own deltas (`r1/callgrind.txt`, head minus parent against the pin), whole process / libc
  subtracted: startup +0.0062 / -0.0001, interpret -0.0007 / -0.0023, parse +0.0004 / 0.0000, churn
  +0.0012 / 0.0000, sayloop +0.0041 / -0.0001. Against the pin all stay under +0.5% (largest churn
  +0.1550%).
* **M2.** The mask in `raise_result_and_sender.rex` covers only a non-SYNTAX condition that was not
  propagated, the ruled difference; the INTERPRET raise is witnessed by the unit test (I1).
* **M3, M4.** The doc block is back on `sigint_halts_after_a_long_builtin`; the condition tests'
  docs carry no task numbers.
* **M5.** The `run_repeating` edits are reverted: no repeating WHILE loop falls back to that path
  (mutating its END arm left `while_retest_blame`, and a labelled-loop variant of it written for the
  purpose, identical to the oracle; `r1/mutants/B-*` records the first). The flat UNTIL path's END store, which round 1 left unwitnessed, has
  `corpus/lang/until_retest_blame.rex`: identical to the oracle at head, and red without the store
  (`r1/mutants/C-*`: the routine sees `nop` and `if i = 1` as its caller's clause).

| command | result |
|---|---|
| `cargo fmt --all --check` (`r1/checks-fmt.txt`) | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-exec -p rexx-parse --all-targets -- -D warnings` (`r1/checks-clippy.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test collect_stress --test signals --test dispatch_seam --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions` (`r1/checks-touched.txt`) | 101: `--lib` 1173 passed; `collect_stress` fails only on `call_site_routine_copy_lifetime.rex` and `call_site_fresh_routine_raises.rex`, as at base; every other binary passed |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`r1/checks-corpus.txt`) | 0, 29 passed, 1 ignored |
| `memcap 16G cargo test -j 4 --release -p rexx-parse --test sourceline_oracle` (`r1/checks-srcl.txt`) | 0 |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t8.2/r1/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` (`r1/suite/`) | 0; no changed row against `t8.1/suite/tests.tsv` |

## Closing add-on

Brief `task-8.2-addon.md`; witnesses only, no behaviour change. Records `t8.2/addon/`.

* **Re-review Minor 1.** `corpus/lang/unknown_forward_unasked.rex` (the review's `u4.rex`, the
  security manager answering `0`): identical to the oracle at head; at `9ce1d8384` (scratch worktree
  build) it prints `sm METHOD UNKNOWN` before `len 3` (`addon/u4.9ce1d8384.txt`).
* **Deviation 67.** `run::tests::conditions::a_started_messages_report_names_the_line_that_raised`
  (`crates/rexx-exec/src/run/tests/conditions.rs`) asserts the whole stderr of `msgstart.rex` run as
  `/w/m.rex`: the started activity's report at line 6, then `result`'s at line 3; 15 of 15 repeated
  runs pass. Mutation: `report_started_failure`'s report made positionless (`scheduler.rs:2158`,
  `if false && ...`) turns it red (`addon/mutant-started-report.txt`). A first mutation, dropping
  the unsealed site there, left it green: the site was already sealed.
* **Deviation 68.** Probe `addon/stem_subclass_unknown.rex` (oracle 97.1, head `mine NOSUCH` / `7`,
  `addon/stem_subclass_unknown.head-and-oracle.txt`). Witness
  `run::tests::conditions::a_stem_subclasss_unknown_method_answers` (same file). Mutation: the
  UNKNOWN lookup in `unknown_or_nomethod` answering `None` (`dispatch.rs:3813`) gives the oracle's
  97.1 and turns it red (`addon/mutant-stem-unknown.txt`).

Checks, from `rust/`: `cargo fmt --all --check` 0; `memcap 16G cargo clippy -j 4 -p rexx-exec
--all-targets -- -D warnings` 0 (`addon/checks-clippy.txt`); `memcap 16G cargo test -j 4 --release
--no-fail-fast -p rexx-exec --lib --test corpus --test dispatch_seam` 0 (lib 1175 passed) and
`-p rexx-parse --test sourceline_oracle` 0 (`addon/checks-touched.txt`).
