# Task 8 fix round C -- report

Implementer: fix-s8c. BASE `a9fd1a7ec`. Scratch:
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/fix-s8c/`
(below, `$C`). Written first, filled as the work goes.

## Status

Code committed at `f24f208ad`; gates green. `RECORDED` is `FUNCTION.TEST_REXXQUEUE` alone and
`DETAIL_DIFFERS` empty, as before.

## N5 -- the stream-object reader's ends of input

**Cause.** Two, one per end. (a) A tail-less or `EXIT` `RAISE` of a condition other than SYNTAX
ended the raising activation and consulted no trap anywhere (`exec_raise`'s "no trap at all" arm),
so the reader's dispatcher never took a `raise notready` or `raise user stop` in `LINEIN`, and the
send answered nothing. The oracle's `RexxActivation::raiseExit` (`execution/RexxActivation.cpp:1741`)
leaves every internal call and then, at the first activation that is not one (`isTopLevelCall`),
calls `raise`, which offers the condition to that activation's sender exactly as `RAISE ... RETURN`
does (`senderActivation`, `:2739`, stopping at a native frame that traps it: the dispatcher). This is
general, not the reader's: `raise user foo` in a method runs the caller's `CALL ON USER` handler and
fires its `SIGNAL ON USER` (probes `$C/x/ru2`, `ru3`). (b) `read_stream_lines` read a `LINEIN` that
answered nothing as `.nil` and went on; `InputRedirector::readBuffered` stops at a null line. And
untrapped, `raise halt` (and `raise nomethod`) is `reportException` in the raising level
(`:1868`-`:1891`), an ordinary SYNTAX 4.1 (97.1 or 97.5) that a `SIGNAL ON SYNTAX` there traps; here it
was a HALT-named report no trap could see (`ru4`-`ru8`, `ru10`, `ru11`).

**Fix.** `exec_raise`: for no tail or `EXIT`, `top` is the first activation that is not an internal
call (`first_non_internal_depth`), and the condition is offered to the frame above it through the
path `RETURN` already took (dispatcher take, `CALL ON` queued against it, `SIGNAL ON` delivered with
the new `Search::AboveTop`, which passes internal calls and then the top); the flow is `Exit`
instead of `Return`. A queued `CALL ON` joins the caller's `INTERPRET` queue, not the raiser's:
`fragment_owners` records which activation each running fragment runs in, and `fragments_within`
counts those being left. Untrapped HALT is `Raised::halt()`, now SYNTAX 4.1, searched `Top` for no
tail and `Here` for `RETURN`; NOMETHOD likewise 97.1 over `ADDITIONAL`/`DESCRIPTION`, else 97.5.
`read_stream_lines` ends the input at `Ok(None)`.

**Witnesses** (`phase-8.txt`, sourceline files): `input_stream_reader_ends.rex` (NOTREADY, USER and
HALT without a tail, a bare `RETURN`, `RAISE ... EXIT`, an internal call's tail-less raise, and a
`RAISE ... RETURN 'last'` reader, each after a blank line; the issuing activation's traps never
run) and `raise_exit_offered_to_caller.rex` (a raise from an internal call of a method and of a
routine, from `INTERPRET` in a routine, a method's `SIGNAL ON USER`, HALT trapped by `CALL ON HALT`,
untrapped HALT as 4.1 from a method, from an internal call and with `RETURN` under the method's own
trap, NOMETHOD 97.1 and 97.5, and a main-level internal call whose raise nobody traps). Both agree on
three descriptors; `a9fd1a7ec`'s binary (`$C/rexx-run-BASE`, fix-s8b's `rexx-run-FINAL`, the same code)
is killed at 20 s on the first and differs at rc 252 on the second.

**N5's probes**: `sq1`-`sq7`, `sr1`-`sr4` identical on three descriptors. `sq8` (`raise error;`) is a
parse error the oracle reports as 35.1 at rc 221 and this crate refuses at rc 120: the known
parse-error-reporting refusal, not this finding.

### Controls (predictions written 2026-09-28 before any ran)

Each mutation is applied to a copy-checked file, built with `--profile mutation` in `$C/tgt`, and
`REXX_CORPUS_GATE=1 cargo test --profile mutation -p rexx-exec --test corpus` run; the file is
restored from the copy and `cmp`-checked after.

| # | Mutation | Prediction | Result |
|---|---|---|---|
| R1 | `redirect.rs` `input_dispatcher_takes` always answers false (no take at all: the control S5 described and never ran) | corpus red on `input_stream_object_reader.rex` (its first reader, `raise notready ... return("")`, answers `''` forever: did not finish) and `input_stream_reader_ends.rex` (the VALUE reader, `raise user stop return 'last'`: did not finish); nothing else | **red, mechanism different**: the corpus run was OOM-killed at memcap's 8G (the in-process reader appends without bound and no deadline stops it before the cap), so the run named no program. Attributed with the mutated binary (`$C/tgt/mutation/rexx-run`, built by that run): `input_stream_object_reader.rex` and `input_stream_reader_ends.rex` each killed at the 20 s timeout, `raise_exit_offered_to_caller.rex` rc 0. Restored |
| R2 | `redirect.rs` `read_stream_lines`: a `LINEIN` answering nothing is a line again (`Ok(None)` read as `.nil`) | corpus red on `input_stream_reader_ends.rex` alone, the BARE reader never ending (did not finish) | as predicted: 645 of 646, `input_stream_reader_ends.rex` "the crate did not finish" (deadline). Restored |
| R3 | `run/condition.rs` `exec_raise`: `top` always 0 (a tail-less raise offered to the raising activation's own caller, internal levels not left) | corpus red on `raise_exit_offered_to_caller.rex` alone, its second line (`handler USER FOO` after `value ex`) missing, and the `routine rv` handler line; `input_stream_reader_ends.rex`'s DEEP case stays green (the internal call's `Flow::Exit` still ends the reader through the empty answer) | as predicted, one line more than predicted: 645 of 646, `raise_exit_offered_to_caller.rex` alone; both `handler USER FOO` lines missing, and also `hz not reached` printed (the main-level internal call's raise offered to the main program's own `CALL ON`), checked with the mutated binary (`$C/mut/R3-rexx-run`). Restored |
| R4 | `error.rs` `Raised::halt`: search `Nobody` again (set in `exec_raise`'s untrapped HALT arm) | corpus red on `raise_exit_offered_to_caller.rex` alone, ending at rc 252 after `signalled USER BAR 15` | as predicted: 645 of 646, the witness alone, stdout stopping after `signalled USER BAR 15`, rc 252 (`$C/mut/R4-rexx-run`). Restored |
| R5 | `activation.rs` `fragments_within` answers 0 | corpus red on `raise_exit_offered_to_caller.rex` alone, the `handler USER FOO` line after `interpreted 5` missing | as predicted: 645 of 646, that line alone (`$C/mut/R5-rexx-run`). Restored |

## N1 and the I2-shaped residuals -- external program, INTERPRET, directive install

**Cause (N1, residuals 2 and 3).** A program called as a routine or function ran in a `TopLevel`
activation whose frame said `PROGRAM`, and nothing captured its frame or sealed its traceback line
when a failure left it (`run_loaded` had neither step `invoke_call` has), so a trapped condition had
no such level. The oracle runs it as an `EXTERNALCALL` activation, a `ROUTINE` frame named as it
was called (`RexxActivation::createStackFrame`, `execution/RexxActivation.cpp:5006`, `isRoutine`).
Its directives install inside that activation, whose line is then the installing `::CLASS`
directive's, and a `::CONSTANT` expression runs as a `METHOD ::CONSTANT` level; the crate had the
traceback lines of the install but not those frames, nor the program as the origin. Found with it
and fixed because they are the same level: a called program's clauses traced at its caller's
indent (a routine's start at the margin; `$C/q/t1`, `a9fd1a7ec` indented two), it announced no
`>I>`/`<I<` (the oracle's `Routine "<its path>"`), and every `>I>`/`<I<` named the main program as
the package where the oracle names the routine's own (`$C/q/t2`, a `::REQUIRES`d routine).

**Fix.** `read_snapshot`: a `TopLevel` activation entered as `SUBROUTINE` or `FUNCTION` is
`ROUTINE`. `run_loaded` for such a call: on an install failure, `capture_site_frame` builds the
level's ended `ROUTINE` frame from the site the install left (the `::CLASS` line) and seals it; on a
run failure, `capture_activation_frame` before the pop and `seal_site_level` after it; the caller's
indent levels are set aside for the run and put back; `trace_invocation_exit` before the pop.
`resolve_constants` captures the `METHOD ::CONSTANT` frame the same way. `invocation_subject`
names a called program by its path, and the three announcement sites name `package_path` of the
running program. `build_interpret_frame` became `build_ended_frame`, which takes the kind, name and
arguments.

**Witness** `external_program_levels.rex` with `.d/` (`phase-8.txt`, sourceline file): the review's
`q/d2`, `q/d4`, `q/r2` and `q/r3` shapes (a failure two levels into a program called by `CALL` and as
a function, one through a `::REQUIRES`d class's `INTERPRET`, the frame type seen from inside, a
`::CONSTANT` failing while a called program installs), the traced program, a traced `::REQUIRES`d
routine, and an untrapped class `INIT` failure during install (the caller's line). Identical on three
descriptors; `a9fd1a7ec`'s binary differs on stdout and stderr. The review's `q/d1`, `d3`, `d5`-`d7`,
`r2`, `r3` and my `r3b`, `r3c`, `t1`, `t2` are identical.

### Controls, the external program's level (predictions written 2026-09-28 before any ran)

| # | Mutation | Prediction | Result |
|---|---|---|---|
| X1 | `lib.rs` `run_loaded`: the called program's `capture_activation_frame` not called | corpus red on `external_program_levels.rex` alone: the `F ROUTINE EXT 2 1` lines (call and function) and `F ROUTINE EXTFN 1 1` missing | as predicted, and more: 649 of 650, the witness alone, those three frames missing and PROPAGATED `0` for the call and function cases (the level no longer counts as left); `$C/mut/X1-rexx-run` over `$C/q/w1`. Restored |
| X2 | `lib.rs` `run_loaded`: no `seal_site_level` after a called program's run fails | corpus red on `external_program_levels.rex` alone: the caller's `T 5 *-* call ext 5` and `T 8 *-* x = ext(5)` lines missing | **red, mechanism falsified**: 649 of 650, the witness alone, but the caller's lines survive; what goes is the called level's own: its frame (`F ROUTINE EXT`, `F ROUTINE EXTFN`) and its line (`T 2 *-* call lvl n`, `T 1 *-* return ...`), because the unsealed frame and site are overwritten by the caller's rather than blocking them. Restored |
| X3 | `dispatch/context.rs` `read_snapshot`: the `ROUTINE` arm for a called program removed | corpus red on `external_program_levels.rex` alone: `F PROGRAM EXT`, `inside PROGRAM kind.rex`, `F PROGRAM bad.rex` | as predicted but for the last: 649 of 650, `PROGRAM EXT` (twice), `PROGRAM EXTFN`, `inside PROGRAM kind.rex`; the install frame stays `ROUTINE bad.rex`, being built by `capture_site_frame`, not `read_snapshot`. Restored |
| X4 | `install.rs` `resolve_constants`: no `::CONSTANT` frame | corpus red on `external_program_levels.rex` alone: `F METHOD ::CONSTANT 4 0` missing and POSITION `3` for `4` | as predicted: 649 of 650, those two. Restored |
| X5 | `lib.rs` `run_loaded`: the caller's indent levels kept for the called program | corpus red on `external_program_levels.rex` alone: `T 1 *-*   return ...` indented two more, and `tr.rex`'s traced lines on stderr indented | as predicted on the binary (`$C/mut/X5-rexx-run` over `$C/q/w1`: both); the corpus reports stdout alone, its DEVIATION 0 normalisation absorbing trace indent widths on stderr. Restored |
| X6 | `run/settings.rs` `trace_invocation_entry`: the `>I>` names `program_path` again (the `<I<` left as fixed) | corpus red on `external_program_levels.rex` alone, stderr: both `>I>` lines name `main.rex`'s path as the package, the `<I<` lines not | as predicted: 649 of 650, stderr, the two `>I>` lines alone. (The first batch's X6 did not apply: its text matched twice and `run.py` refused before writing; re-run with a unique anchor, the file `cmp`-equal to its copy before.) Restored |

## ~empty under an exposed tail

**Cause.** The oracle's exposed tail is one `CompoundTableElement` two stems share
(`RexxCompoundVariable::expose`); `StemClass::empty` (`classes/StemClass.cpp:1060`) and a tail-less
`PUT` (`bracketEqual`, `:476`) `clear()` the object's tree, so the element leaves the object's stem
and stays with every stem that exposes it, keeping its value. Here an exposing stem names the
object's stem and the tail (`Body::Stem::exposed`), so after the clear it read and wrote the
object's stem afresh (review's `q/r4`). Matched, so no refusal was restored: it is local to the stem
code.

**Fix.** `Interp::stem_exposers` notes, for each stem another stem's `EXPOSE` made tails of, a weak
reference (`Body::WeakRef`, the collector's own) to each exposing stem (`record_exposer`, from
`expose_tail`; dead notes dropped when a list or the table doubles). `detach_exposed_tails`, called
before `~empty` and a tail-less `PUT` clear, gives the stems still exposing tails of it one new stem
holding the tails as they stand (the elements' new home), repointing their `exposed` entries to it.
The cells are roots; the stems are not. Found with it and fixed because it is the other clear: a
tail-less `PUT`/`[]=` stored a tail named by the null string where the oracle makes the value the
default, clears every tail and refuses a stem as the value (98.976) (`$C/x/sp1`-`sp3`).

**Cost.** A weak-reference cell per `EXPOSE` of a tail; `$C/x/em3.rex` (200000 method calls exposing
a tail, an `~empty` every 1000, then 20000 new objects): 0.31 s / 58,332 KB at `a9fd1a7ec`, 0.34 s /
55,872 KB here (wall clock, one run each, noisy). A first version that pruned on every call was
0.52 s against 1.13 s and was replaced.

**Witness** `expose_tail_detached.rex` (`phase-8.txt`, sourceline file): `q/r4`'s shape and further:
two tails, the object's view after, a second method's view of the same tail after (a fresh element),
a `PROCEDURE EXPOSE` of the detached tail, a refill and a second empty, `symbol`, `allIndexes`,
`items`, `hasIndex`, `[]` and the local `~empty` on a stem holding a detached tail, a tail-less `PUT`
under exposure, the tail-less `PUT` and `[]=` on a plain stem, and 98.976. Identical on three
descriptors; `a9fd1a7ec`'s binary differs on all three. The review's `p/ex1`-`ex4` stay identical.

**Record** removed from `phase-4-exclusions.txt` ("AN OBJECT'S STEM EMPTIED WHILE A METHOD'S STEM
EXPOSES ONE OF ITS TAILS").

### Controls (predictions written 2026-09-28 before any ran)

| # | Mutation | Prediction | Result |
|---|---|---|---|
| E7 | `stem.rs` `detach_exposed_tails` returns at once | corpus red on `expose_tail_detached.rex` alone: from `after empty` on it reads the object's stem afresh (`after empty A.1 A.2`, `object changed 1`, ...), and `after put` reads `dflt` | as predicted: 651 of 652, the witness alone (`after empty A.1 A.2`, `object changed 1`, ..., `after put dflt`; `$C/mut/E7-rexx-run`). Restored |
| E8 | `stem.rs` `detach_exposed_tails`: the orphan made but no exposed entry repointed | as E7 (same lines), and nothing else | as predicted: 651 of 652, the witness alone, the same stdout as E7. Restored |
| E9 | `dispatch/hash/stem.rs` `native_stem_put`: the no-tail branch removed | corpus red on `expose_tail_detached.rex` alone: `object dflt 0` and the `0 d d` / `0 e` lines differ, and the final `PUT` of a stem is not 98.976 (rc 0 for 158) | as predicted: 651 of 652, the witness alone, stdout, stderr and rc (`object kept 2`, `2 1 S.2`, `3 3`, rc 0). Restored |
| E10 | `lib.rs` `object_roots`: `stem_exposers`' cells not rooted | the corpus green (no collection runs in the witness's lifetime at its size); `collect_stress.rs`, which collects at every allocation, red on `expose_tail_detached.rex` (a cell freed before the empty, so the element is not detached) or panicking at a dead handle | as predicted, the first branch: corpus 652 of 652; `collect_stress.rs`'s `the_l0_subset_passes_again_under_collect_on_every_allocation` red on `expose_tail_detached.rex` alone, the stress run reading the object's stem afresh (`after empty A.1 A.2`). Restored |


## N2 -- PROPAGATED (with RAISE PROPAGATE) and N3 -- a RAISE SYNTAX in a callback

**Cause (N2).** PROPAGATED was the constant `0` (`condition.rs`: "nothing sets it to 1 yet"). The
oracle's `Activity::raiseException` (`concurrency/Activity.cpp:955`-`:972`) offers a SYNTAX condition
to the raising Rexx activation alone (`raiseCondition` stops at the first `RexxActivation`) and
marks it PROPAGATED the moment that one does not trap it; an internal call and an `INTERPRET`
share their parent's traps in effect (in this crate the parent's table is where they are found), so
leaving one of those does not count. `Activity::reraiseException` (a native boundary, `:1330`) marks
it too. The record at `phase-4-exclusions.txt:4693` confined this to a native re-raise and blamed the
boundary; both were false (review's `pr1`, no native frame).

**Found with it: `RAISE PROPAGATE` out of a method or routine could not be trapped.**
`exec_raise_propagate` sent the condition with `Search::Nobody` everywhere, so a caller's `SIGNAL ON
SYNTAX` never saw it (`$C/x/rp1`, `rp5`, `rp7`) and the untrapped report had no `running ... line N`
(`rp2`). The oracle's `RexxActivation::raise` (`execution/RexxActivation.cpp:1840`) pops the level
and `reraiseException`s **the same object** (frames and traceback as first raised; POSITION,
PROGRAM, PACKAGE the re-raising level's; PROPAGATED 1). Measured: from an internal call it is
trapped neither by that call's parent nor by the first activation that is not an internal call,
but by that activation's caller (`rp7`); at the program's own level it is the positionless report
as before (`rp4`, `rp8`, `rp9`).

**Cause (N3).** A native boundary's re-raise reset the origin (`reraise_failure_levels`) but a
condition raised by `RAISE SYNTAX` carries its own `position`, which `build_condition_object_from`
reads first, so the RAISE's line survived the re-raise.

**Fix.** `failure_propagated` is set when a SYNTAX failure leaves a level that is not an internal
call (`capture_activation_frame`) and by `reraise_failure_levels`; it rides `Unwound` into the
object. `failure_reraised`, set by `reraise_failure_levels`, makes the object ignore the raise's own
position. `exec_raise_propagate`, where an activation above the first non-internal one exists,
re-raises with `Search::Caller` (or `AboveTop` from an internal call), keeps the handler's object
(`reraised_object`, rooted) for the trap to update (`reraise_condition_object`: POSITION, PROGRAM,
PACKAGE of the re-raising level, from `failure_origin` once that level too is left; PROPAGATED 1),
and skips the frames and origin of the levels it leaves (`reraise_leaving`).

**Record.** `phase-4-exclusions.txt`'s "A NATIVE SYNTAX CONDITION RE-RAISED INTO A CALLER LACKS
PROPAGATED" removed: `cond3.rex` and `sendthrow.rex` (the Task 5 forge's probes, with fix-s8b's
`forge/libreach.so`) are identical on three descriptors now (`$C/cmpf.sh`). Its note on the METHOD
group's `forward ... continue` frame was re-measured by the re-review (`rs` probes, agree). The
neighbouring "A RaiseCondition TAKEN BY CALL ON CARRIES POSITION" (`rcond.rex`) still differs and
stays; it is not this round's.

**Witnesses** (`phase-8.txt`, sourceline files): `propagated_leaving_levels.rex` (`pr1`'s six
cases, `RAISE PROPAGATE` from a method's handler and from an internal call's inside a method, the
method's own trap passed over from an internal call, then an internal call trapped by its parent
at 0), `propagate_untrapped_from_method.rex` (the report's `line 4`), `native_reraise_raise_syntax.rex`
with `.env` (`TestSendMessage0` over `RAISE SYNTAX`, tail-less and with RETURN, trapped and
untrapped). `a9fd1a7ec`'s binary differs on each. `nr1` case a1 now agrees; its a4 is N4.

**Not matched, kept out of the witness**: after a `RAISE PROPAGATE` was trapped by the main
program, the oracle's next condition object names the `PROGRAM` frame with an empty name
(`PROGRAM  27`) where this crate names the path; the witness prints type and line only. Not
explained.

### Controls (predictions written 2026-09-28 before any ran)

| # | Mutation | Prediction | Result |
|---|---|---|---|
| P1 | `run.rs` `capture_activation_frame`: `failure_propagated` never set | corpus red on `propagated_leaving_levels.rex` alone: `method error`, `routine error` and `method raise` read `0` | as predicted: 648 of 649, those three lines (`$C/mut/P1-rexx-run`). Restored |
| P2 | `run.rs` `reraise_failure_levels`: `failure_reraised` not set | corpus red on `native_reraise_raise_syntax.rex` alone, POSITION `24` for `6` and `25` for `12` | as predicted, one figure off: 648 of 649, `24` for `6` and `26` (the RAISE's actual line) for `12`. Restored |
| P3 | `run/condition.rs` `exec_raise_propagate`: never re-raises (the old `Nobody`, positionless) | corpus red on `propagated_leaving_levels.rex` (from `own trap` on: nothing traps it, rc 214) and `propagate_untrapped_from_method.rex` (the report has no `running ... line 4`) | as predicted: 647 of 649, both files, those differences. Restored |
| P4 | `run.rs` `capture_activation_frame`: `reraise_leaving` never consulted (the propagating level adds its frame and origin) | corpus red on `propagated_leaving_levels.rex` (the `propagated from a method` and `from an internal call` POSITIONs become the propagate lines in the methods) and `propagate_untrapped_from_method.rex` (report line 12 for 4) | **half falsified**: 648 of 649, `propagated_leaving_levels.rex` alone (POSITION `58`, `66`, `77` for `21`, `27`, `35`); the untrapped report is unaffected, its line coming from the sites, not the origin. The witness that sees P4 is the first file. Restored |
| P5 | `run/condition.rs` `exec_raise_propagate`: always `Search::Caller` (no `AboveTop`) | corpus red on `propagated_leaving_levels.rex` alone: from `w`'s internal `sub`, `w`'s own `mt` takes it (`not reached` printed) | as predicted: 648 of 649, that file alone. Restored |

## N3 -- tail-less RAISE SYNTAX in a callback

In the N2 section above (cause, fix, witness `native_reraise_raise_syntax.rex`, control P2); commit
`2c8abdbdf`.

## N4 -- CallRoutine, and the live stack's native and INTERPRET levels (residual 1)

**Cause.** Two. (a) `RexxThreadInterface.CallRoutine` sent the routine `CALL` or `CALLWITH`, so
its level was a `Routine~callWith` call: a `ROUTINE CALL` frame, a `METHOD CALLWITH` level and a
`Compiled method "CALLWITH"` traceback line the oracle does not have. `CallRoutineDispatcher::run`
(`concurrency/RexxStartDispatcher.cpp:207`) calls the routine directly under the null string.
(b) Every live stack this crate built (`.context~stackframes`, and the frames a condition object
takes of the levels still running) was the activations alone. The oracle's
`Activity::generateStackFrames` walks every `ActivationBase`: a native call's `NativeActivation`
(`nr4`: the called routine sees `METHOD CALLR`) and an `INTERPRET` fragment's own activation
(review's `q/r1`, residual 1).

**Fix.** (a) A `Surface::call_routine` member; this crate's `call_routine_directly` runs the routine
under the null string over the array's items (98.913 for a non-array, as before), the name threaded
through `enter_routine`, `enter_installed_routine` and `call_over_installed_routine`, where
`Routine~call` and `~callWith` keep `CALL`. (b) `live_levels` lists, before each activation, the
native calls it made (`NativeFrame::caller`, set at the push) and the fragments running in it
(`Interp::fragments`, which replaces round N5's owner list and keeps each fragment's parsed text,
its `INTERPRET` line and indent, and the enclosing fragment's clause), each innermost first;
`build_live_frame` builds each, a fragment's traceline from its clause being stepped
(`fragment_clause`, written in the clause region's existing `else` of the override test).
`context_stack_frames` and `condition_frames` both walk it; a native call's own condition still
leads with its frame where no activation called it.

**Witness** `live_stack_frames.rex` with `.env` (`phase-8.txt`, sourceline file): `q/r1`, a nested
`INTERPRET`'s frame count and the enclosing fragment's clause, a trap inside an internal call made
from `INTERPRET`, `TestSendMessage0`'s Rexx method listing its frames, `TestCallRoutine`'s routine
listing its frames and counting them, the review's `nr1` a4 trapped and `nr2` untrapped. Identical on
three descriptors; `a9fd1a7ec`'s binary differs on stdout and stderr. `nr1`-`nr4` and `$C/x/nf1` are
identical now.

**Not matched, found here, left:** `Routine~call` from Rexx has a `METHOD CALL` level on the oracle
(`RoutineClass::callRexx` is an internal method with a frame of its own) and names the routine's
frame by the routine's name (`ROUTINE [rr]`); here `ROUTINE [CALL]` and no `METHOD` level
(`$C/x/if1`). The same absence is why L5's mutant showed no `CALLWITH` level in the live stack: this
crate lists no live frame for a built-in method's level. Not an N4 member (CallRoutine is), not
recorded yet; see Concerns.

### Controls (predictions written 2026-09-28 before any ran)

| # | Mutation | Prediction | Result |
|---|---|---|---|
| L1 | `dispatch/context.rs` `live_levels`: no native row listed | corpus red on `live_stack_frames.rex` alone: the `METHOD SEND0` line inside `m` and `METHOD [CALLR] The NIL object 2` inside the called routine missing, `0 3` becoming `0 2`; the trapped CallRoutine frames keep CALLR (a failure's native level is `blame_native_level`'s) | as predicted: 650 of 651, those three lines (`$C/mut/L1-rexx-run`). Restored |
| L2 | `dispatch/context.rs` `live_levels`: no fragment row listed | corpus red on `live_stack_frames.rex` alone: the `INTERPRET 22` line and `trapped INTERPRET 25` missing, `4` becoming `2` | as predicted, one line more: 650 of 651, those, and the `23 *-*` traceline naming the outer `interpret` (the innermost frame being the routine's once no fragment is listed). Restored |
| L3 | `run.rs`: `fragment_clause` never written | corpus red on `live_stack_frames.rex` alone: the `INTERPRET 22` traceline reads the fragment's first clause, `nop;` | as predicted, one line more: 650 of 651, `nop;`, and the nested case's traceline reading the inner fragment's first clause. Restored |
| L4 | `dispatch/executable.rs` `call_routine_directly`: frame name `CALL` for the null string | corpus red on `live_stack_frames.rex` alone: `ROUTINE [CALL] 1 2` and the trapped `ROUTINE [CALL] 1` | as predicted: 650 of 651, those two. Restored |
| L5 | `rexx-api/src/callbacks.rs` `call_routine`: sends `CALLWITH` again | corpus red on `live_stack_frames.rex` alone: a `METHOD [CALLWITH]` frame in each case, `0 3` becoming `0 4`, and a `Compiled method "CALLWITH"` traceback line trapped and untrapped | **partly falsified**: 650 of 651, the trapped `METHOD [CALLWITH]` frame and both traceback lines, and `ROUTINE [CALL]`; but `0 3` stays and the live listing inside the routine has no `CALLWITH` level (the mutant is a built-in method's level, which the live stack does not list; see the note above). Restored |


## N6 -- a .Stream read once

**Not a defect: the probe's wrapper.** `review-s8/cmp.sh` runs the oracle and then this crate in the
same `mktemp -d` directory, and `sr4.rex` begins by appending two lines to `in.txt`; the crate
therefore read the four lines both runs had written. Each side from its own fresh directory,
`sr4.rex` prints `0 one|two`, `one`, `0 two` on both, rc 0 (`$C/cmp.sh`, fixed to use a directory
per side; `sr5`-`sr11` in `$C/x` isolate it: the doubling appears only with a pre-existing file).
No code change. Witness: none needed for a non-divergence; `sr4.rex` stays in `$C/p`.

## Performance

Callgrind `summary`, each program copied to a fresh directory and run by `$C/cg/run.sh TAG BINARY
PROGRAM` (`valgrind --tool=callgrind`). `b0` is `391242b7e` (surface-8's `base-target`), `a9` is
`a9fd1a7ec` built here from `git archive` into `$C/base-a9` with its own target (`Compiling
rexx-exec` seen), `f24` is `f24f208ad` (`$C/tgt`). The trap loops are the review's `cg2` programs.

| program | b0 | a9 | f24 | f24/a9 | f24/b0 |
|---|---|---|---|---|---|
| `t.rex`, trapped in the raising activation, 5000 | 629,367,288 | 550,726,150 | 552,012,343 | +0.23% | -12.29% |
| `tu.rex`, one internal call up, 5000 | 1,232,668,700 | 1,176,838,238 | 1,180,306,752 | +0.29% | -4.25% |
| `tm.rex`, one method up, 5000 | 642,968,791 | 682,952,244 | 684,776,263 | +0.27% | +6.50% |
| `td.rex`, eleven internal calls up, 2000 | 5,364,191,625 | 5,447,157,559 | 5,484,456,044 | +0.68% | +2.24% |
| `rexxcps.rex` (`rust/bench-rexxcps`) | -- | 19,452,293,687 | 19,469,278,213 | +0.09% | -- |
| `em3.rex` (`$C/x`, 200000 sends exposing a tail) | -- | 2,573,389,777 | 2,802,317,915 | +8.90% | -- |

* Every trap loop is within +10% of the Task 8 BASE `b0` (the I5 ruling's bound); the largest is the
  method loop at +6.50%, which was +6.2% at `a9` in the review's own measurement.
* rexxcps moves +0.09% against `a9`, inside the layout band this project measured. Per-clause work
  added: none on the common path; the clause region's existing `else` of the `INTERPRET`-override
  test now stores the fragment's clause index (taken only inside a fragment).
* `em3` pays for the weak-reference cell each `EXPOSE` of a tail now allocates (`~empty`'s fix);
  measured because it is the new per-send cost, not a gate. A version that pruned on every expose
  was twice as slow on the wall clock and was replaced before commit.
* The intermediate round (`d354a2c3d`, before the `~empty` commit) read t +0.26%, tu +0.36%, tm
  +0.31%, td +0.74%, rexxcps +0.10% against `a9` (`$C/cg/round1.txt`).

## Commits

* `50ec3eb98` Offer a tail-less or EXIT RAISE to the caller, as the oracle's raiseExit does (N5)
* `2c8abdbdf` Set PROPAGATED as the oracle does, and re-raise RAISE PROPAGATE in the caller (N2,
  N3)
* `5b884e03b` Run a program called as a routine as the oracle's ROUTINE level (N1, residuals 2 and
  3)
* `d354a2c3d` List a running stack's native and INTERPRET levels, and call CallRoutine's routine
  directly (N4, residual 1)
* `f24f208ad` Keep an exposed tail with its exposer when the object's stem is emptied (`~empty`)

N6 has no commit (not a defect). Before each commit: `cargo fmt --all --check`, `cargo clippy
--workspace --all-targets -- -D warnings` and `REXX_CORPUS_GATE=1 memcap 8G cargo test --release
--workspace --no-fail-fast` (built first outside the cap; the one run built under it was
OOM-killed at 8G while compiling, `$C/ws2.log`'s first attempt) in `$C/tgt`: 0 failed each time
(`$C/ws1.log` had one failure, `refusal_sites`, re-derived before `50ec3eb98`). The corpus and the
api instrument were also run alone before the P and L mutations (`$C/mut/base2.log`, `base4.log`:
the instrument green, so `RECORDED` and `DETAIL_DIFFERS` unchanged), the corpus and
`collect_stress.rs` before the E mutations (`base5.log`), and the whole gated workspace, instrument
included, in the shared `rust/target` at `f24f208ad` before the gates (`$C/shared.log`: 2752
passed, 0 failed). No `ffi.rs` or `load.rs` change, so no Miri run.

## Gates

`$C/gates.sh` (surface-6's, `S` set to `$C`), run at `f24f208ad` in the background; the tree was not
touched while it ran. `$C/gates/status.txt`:

```
f24f208addebdd34a5c9b00f26e99b6e440d0b5b
started 2026-09-28T21:20:36+02:00
load at start 9.88 11.25 13.34 6/2644 3272806
G1 fmt exit 0
G2 clippy(empty target) exit 0
G3 release build (test --no-run) exit 0
load G4 9.01 11.00 13.22 13/2636 3274808 2026-09-28T21:20:48+02:00
G4 release test exit 0
G4 Compiling lines: 0
load after G4 11.33 16.62 15.60 7/2647 3403297
G5 debug build (test --no-run) exit 0
load G6 12.03 16.14 15.47 7/2657 3408601 2026-09-28T21:27:42+02:00
G6 debug test exit 0
G6 Compiling lines: 0
load after G6 21.38 18.96 17.20 42/2802 3537209
f24f208addebdd34a5c9b00f26e99b6e440d0b5b
finished 2026-09-28T21:34:21+02:00
```

Summed `test result` lines: G4 2752 passed / 0 failed / 4 ignored, G6 2753 / 0 / 4. The corpus reads
`652 of 652 matching` in both, and `every_test_of_the_phase_8_groups_matches_the_oracle_but_the_recorded
... ok` in both. G2's clippy checked `rexx-exec`, `rexx-api` and `rexx-core` from an empty target.

## Concerns

* **A looping reader is not bounded before memory in the in-process corpus harness.** R1's mutant
  OOM-killed the whole corpus run at memcap's 8G instead of being reported as "did not finish": the
  in-process side's deadline did not stop a reader appending lines before the cap. The control was
  still red and attributed with the mutated binary, but a future looping reader would take the
  whole corpus run down rather than name itself.
* **Found, pre-existing, not fixed, not recorded** (outside this round's list; for the lead):
  * `RAISE ... RETURN` inside `INTERPRET` is offered to the activation running the fragment on the
    oracle (`senderActivation` is the interpret activation's parent), here to that activation's
    caller (`$C/x/ri1.rex`: oracle `rr2 handler`, here none).
  * `Routine~call` from Rexx has a `METHOD CALL` level and a routine frame named by the routine on
    the oracle; here `ROUTINE CALL` and no level (`$C/x/if1.rex`). More generally no live frame is
    listed for a built-in method's level.
  * After a `RAISE PROPAGATE` is trapped by the main program, the oracle's next condition object
    names the `PROGRAM` frame with an empty name; not explained (kept out of the witness).
  * `raise error;` (`review-s8/p/sq8`) is the known parse-error-reporting refusal (rc 120 for 221).
* **Scope went past the listed items where the fix needed it, each witnessed**: untrapped HALT and
  NOMETHOD from `RAISE` as SYNTAX errors a trap can take; `RAISE PROPAGATE` re-raised in the caller;
  a called program's trace indent and `>I>`/`<I<`, and every `>I>`/`<I<` naming the routine's own
  package; the tail-less `PUT`/`[]=` on a stem.
* **Control mishaps, in their rows**: R1's mechanism (OOM, not a deadline), X2's (the called level's
  own lines went, not the caller's), X3's `bad.rex` frame, P2's line figure, P4's second file and L5's
  live count were predicted wrong; the first X6 did not apply (text matched twice) and was re-run.
