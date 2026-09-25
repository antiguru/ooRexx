# Scoped re-review: Surface Task 3, fix round 1 (`034c1c7d7..6c96144d8`)

VERDICT: **CHANGES REQUESTED** -- prose only. Every behavioural change in this
round is right where I could measure it, and I found no code change to make.
Four record/comment lines state something the code does not do.

Everything below was run in this session's scratchpad
(`.../scratchpad/t3rr1/`), never in the worktree. Three trees were built from
`git archive` with their own `CARGO_TARGET_DIR`: `e0b30d156` (the commit
before Task 3), `034c1c7d7` (this round's base) and `6c96144d8` (head). The
target directories are deleted.

## The independent check: `gate_table_c` at `e0b30d156`

`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c`
on a pristine `git archive e0b30d156` tree, its own target directory:

```
test result: FAILED. 21 passed; 1 failed
82 row(s) of gate table C owned by a closing or closed phase do not `agree`
```

**82 rows.** The failure predates Task 3, so the report's conclusion holds even
though its argument (a pristine tree at `2d155158c`, this round's own commit)
did not establish it.

The report's *description* of the 82 does not hold. It says they are
"`unanswered` rows of `File`, `Alarm`, `Ticker`, `StreamSupplier` and `Stream`
instance surfaces". The gated 82 are phase 7's alone: `File` instance 50,
`Stream` instance 24, `StreamSupplier` instance 8. `Alarm` instance (7) and
`Ticker` instance (6) are phase 6, which this run does not gate ("6: 13 rows,
13 not yet `agree`"; "7: 94 rows, 82 not yet `agree`"). `Pointer` (5,
never-expected-to-agree) and `StackFrame` (10, deferred) are likewise outside
the 82. Naming two extra families makes the set look like something other than
one phase's backlog.

## Per finding

| # | finding | status |
|---|---|---|
| 1 | fresh allocations in the native call path unrooted | **closed** |
| 2 | native call frame rebuilt per call | **closed**, before this diff |
| 3 | `logical_t`'s `found` is the wrong object | **closed** |
| 4 | `found` never sends `OBJECTNAME`/`DEFAULTNAME` | **closed** |
| 5 | `pointer_string` lacks glibc's `(nil)` | **closed** |
| 6 | the minors | **closed** except three prose items (N1, N2, N3) |
| 7 | `PROCEDURE`/`DEFAULTNAME` trap divergence recorded | **closed**, and the "pre-existing" claim verified |

### 1. Rooting

I built `collect_stress` from the head archive and ran it myself:
`cargo test --release -p rexx-exec --test collect_stress` -> **32 passed, 0
failed**, `the_l0_subset_passes_again_under_collect_on_every_allocation ...
ok`. `phase-8.txt` is in `SUBSET_FILES`, so every Task 3 corpus program runs
under collection at every allocation in that test.

The control is live and the report's cause A is exactly right. With the one
`push_temp(argument)` in `Interp::resolve_stream` deleted from my head tree:

* `run::tests::a_stream_builtins_name_survives_a_collection_at_every_allocation`
  fails, `left: 120, right: 0`;
* the L0 test fails naming exactly five programs, each stress-side rc 120 with
  "a message send to a value whose object is no longer live":
  `address_with_stream`, `executable_context`, `sys_file_functions`,
  `security_manager`, `call_miss_not_cached` -- the five the exclusions entry
  names.

`run.rs` restored from a copy and `cmp`-checked.

**Are those the only unrooted values on the paths the round touched?** I walked
both paths and could not make another one fire.

* `build_condition_object`: every `entries` value is now `push_temp`ed where it
  is made. `frames` and `traceback` were already rooted inside `new_list`
  (`condition.rs:221`), so the two extra pushes are redundant, not load-bearing
  -- harmless. `pending_additional.take()`'s object is pushed immediately.
* The one value still held across allocations is `condition_frames`' third
  return, `frame_line` (`condition.rs:210`, the answer of `send_message(frame,
  b"LINE")`), which `build_condition_object` carries past four allocations
  before rooting it. It is not a hazard: `build_frame` stores `LINE` as
  `interp.counted(snapshot.clause.line)` and `SMALL_INT_MAX` is `(1 << 61) -
  1`, so a line number is always a tagged value with nothing on the heap. Not a
  finding; recorded so the next reader does not have to re-derive it.
* `resolve_stream`'s result `built` is held unrooted across
  `stream_table_mut()` and, at `builtin/stream.rs:200-204`, across
  `text_built`. I tried to make it fire -- `stream(name,'c','query exists')`
  (the `added = false` branch), `stream(name,'s')`, `stream(name,'d')`,
  `lines(name,'C')` -- all `SAME` between a plain run and
  `run_program_collect_every_alloc` (10, 8 and 4 collections in the swept
  runs). The instance `NEW` answers is already a temp of the caller's frame.
* `Interp::refusal` runs *after* `pop_native_frame`, so the `argument` a
  `Refused` carries is no longer rooted by the native frame while
  `native_found` sends `OBJECTNAME`. Measured live rather than argued: a loop
  of `t~int(.D~new)` and `t~logical(.S~new(copies('q',60)))` under
  `run_program_collect_every_alloc` is `SAME` at 1387 collections. The
  argument is a temp of the caller's evaluation, which is what the L0 test's
  green `library_native_object_arguments.rex` also witnesses.

### 2. Frame reuse

Not in this diff: `Interp::native_spares` and `pop_native_frame`'s clearing
landed in `c00d18052`, before this round's base. This round touched only the
three comments that described them, which is item 6 and is correct now: a pop
clears `name`, `arguments`, `argument_list` and `locals`, and leaves `owner`,
`scope`, `receiver` and `method` to be overwritten by the next
`push_native_frame`, which it is (`library.rs:214-218`).

### 3 and 4, and how they compose

Measured on both sides from fresh directories, three descriptors, with a
`::method defaultname` that counts its own runs:

```
t~int(.D~new)      found "a named thing"   DEFAULTNAME ran 2
t~logical(.D~new)  found "a named thing"   DEFAULTNAME ran 1
t~int(d)           found "named"           DEFAULTNAME ran 0   (d~objectName = 'named')
t~logical(d)       found "named"           DEFAULTNAME ran 0
```

Oracle and crate byte-identical on stdout and stderr, rc 0 both. So the two
changes compose: `t~int` runs the name twice (once in the conversion, once in
`native_found`) and so does the oracle; a logical runs it once, in the
conversion, and `native_found` on the converted `Body::Text` does not send.
Neither "twice" nor "never" appears on either side.

**Is the guard faithful, or does it special-case what was measured?** Faithful.
The oracle's classes that override `stringValue()` are `RexxInternalObject`,
`RexxObject`, `StemClass`, `RexxString`, `PointerClass`, `RexxInteger`,
`NumberString`, `MutableBuffer`, `StackFrameClass` and `VariableReference`
(`grep '::stringValue' interpreter/`). Of those, only `MutableBuffer` and
`PointerClass` are a `Body::Instance` in this crate; the rest are `Body::Text`,
`Body::Num`, `Body::Stem`, `Body::Native` or `Body::VarRef` and fall to
`native_found`'s `_ => false`. `NativeState` is `Buffer`, `Stream` and
`Pointer`, and the one the guard lets through, `Stream`, has no C++ override,
so it correctly sends. I probed the shapes below, most of them through both
`t~int` and `t~logical`; oracle and crate are byte-identical on all three
descriptors for every one of them:

```
VariableReference  found "5"            Stem          found "X."
Stream (int)       found "a Stream"     Stream (log)  found "nosuchfile.txt"
StackFrame         found "     4 *-* sf = .context~stackframes[1]"
.environment       found "The Environment Directory"
.local / .methods / .context / Package / Directory / Table / Queue /
Supplier / Message / .string / an Array / a class instance
a DEFAULTNAME answering an Array   found "an Array"   (both sides)
```

The same probe against the `034c1c7d7` binary differs from the oracle on three
of those lines (`t~logical(stream)` was `a Stream`, `t~int(.RN~new)` and
`t~logical(.RN~new)` were `a RN`), so items 3 and 4 each close an observable
divergence, and the only remaining divergences are the two the round recorded:
`t~logical(.S~new(.array~of(1,2)))` and `t~logical(.S~new(.object~new))`,
matching the new exclusions entry word for word.

**Does the new path duplicate the shared helper's logic?** Yes, mildly. See N5.

### 5. The `(nil)` acceptance rule

Checked against real glibc rather than against the report. I compiled a C
program calling `sscanf(text, "0x%p", &p)` (glibc 2.43) and a Rust program
holding `values::pointer_string` verbatim, and ran both over 3851 generated
inputs: 17 `(nil)`-ish spellings x 9 leading-whitespace runs x 5 sign forms x
5 inner-prefix forms, plus 26 non-`(nil)` controls. **Zero divergences**
(1230 accepted, 2621 rejected on both sides), so the rule is glibc's over that
space, not just over the 19 forms the report quotes -- and I found no form the
rule accepts that glibc rejects.

Negative control, predicted before running: deleting both `is_nil_spelling`
arms makes the two sides disagree, and the disagreements are cases glibc
accepts and the rule then rejects. **Confirmed**: 299 differing lines.

### 6. The minors

Checked and correct:

* `RESULT_DIGITS`'s rewritten comment. Measured on both sides:
  `TestDoubleArg('0.6666666666666666')` is `0.666666667` under `NUMERIC DIGITS`
  5, 9 and 20 alike, oracle and crate identical.
* `NO_ALLOCATION_PROGRAMS`'s doc: "the assertion below prints the set it
  observed beside this list whenever the two differ" is true --
  `collect_stress.rs:281` is `assert_eq!(observed, expected, ...)`, which
  prints both sides.
* `refusal_sites.rs`'s `SHARED_ANSWERS` comment: `refusal-sites.tsv:232` gives
  `not_logical`'s witness as `.String~new("abc")~"?"("y", "n")`, whose *receiver*
  is the tested string. The comment now says receiver. Correct.
* The three "emptied" comments (`native_spares`, its `object_roots`
  destructuring, `pop_native_frame`). Correct, see finding 2 above.
* `int_from_native`'s reason at the site is defensible from inside `rexx-api`,
  where `Host::whole_number` may allocate for all the seam knows. (The real
  `Interp::whole_number`, `library.rs:393`, never allocates for a `c_int`, so
  the sentence is about the seam's contract and not about this host. It reads
  as intended.)

Not correct: N1, N2, N3 below.

### 7. The recorded `PROCEDURE`/`DEFAULTNAME` divergence

Verified pre-existing, which the entry asserts. `t~int(.RB~new)` where `RB`'s
`defaultname` does `raise syntax 40.1`: the oracle traps `40.1` and runs on at
rc 0; **both** the `034c1c7d7` binary and the head binary end at rc 216 with a
byte-identical traceback through `OBJECTNAME` and `STRING`. The crate dies in
the *conversion* (`Compiled method "STRING"` is on the traceback), which
pre-dates this round, so the round did not create the shape by making
`native_found` send. NO OWNER is the right call.

## The witnesses

Every behavioural change has a test that fails without it. Three controls I
re-ran or re-derived myself are above (the stream root's unit test, the stream
root's L0 five, the `(nil)` arms). The remaining controls (C1/C1b, C2, C3, C4,
C6) are reported with predictions written first and with the tree restored and
`cmp`-checked, and C1's falsified half and C2's falsified half are both
correctly diagnosed (a host-side mutation cannot reach a stand-in-driven
`rexx-api` test; a logical's `found` never reaches `native_found`).

The new corpus rows are not satisfied by an earlier refusal. Measured, both
sides identical:

```
t~bufferlength('abc')   88.914 Argument 1 must be an instance of the MutableBuffer class.
t~refvalue('abc')       88.914 Argument 1 must be an instance of the VariableReference class.
t~refvalue(.object~new) 88.914 Argument 1 must be an instance of the VariableReference class.
```

The ten new `call pointer` lines distinguish accept from reject:
`TestPointerStringArg` answers `0`/`1` where the conversion succeeds and
88.919 where it refuses, which is what C3 moved.

**The MutableBuffer and VariableReference 88.914 rows are witnessed only on the
refusal side: acceptable, and no different witness was available.** I ran each
of the eight shipped declarations that reach those two rows, one program each,
against the head binary:

```
TestMutableBufferLength      rc 120 loud   TestVariableReferenceName      rc 120 loud
TestMutableBufferCapacity    rc 120 loud   TestVariableReferenceValue     rc 120 loud
TestSetMutableBufferLength   rc 120 loud   TestSetVariableReferenceValue  rc 120 loud
TestGetMutableBufferValue    rc 134 ABORT  TestSetMutableBufferValue      rc 134 ABORT
```

The two aborts are `MutableBufferData` reaching `layout::abort`
(`rexx-api/src/layout.rs:382`), which is by design for a slot that cannot fake
a pointer back into an `extern "C"` frame, and they are identical on the
`034c1c7d7` binary, so they are not this round's. The consequence for the
question asked: a success-side corpus row would either have refused loudly or
aborted the harness, so the refusal half is the only half that could have been
written.

## New findings

### N1 (minor, prose). `rust/crates/rexx-exec/src/dispatch/library.rs:247-249`

```
// `found` is the argument's `stringValue()`, which is not its string
// conversion: measured, oracle, an array is `found "an Array".` for
// 88.921 and 88.905 where its string conversion joins its items.
```

It heads the whole refusal match, and `Refused::NotLogical` at `:271` is the
arm for which the opposite is true: a logical's `found` *is* the string
conversion, and for an array it is `"1\n2"` and not `"an Array"` -- measured on
both sides in this review, which is the very example the comment uses to say it
cannot be. The corpus program's own header (`library_native_object_arguments.rex:1-7`)
already carries the exception; this comment does not.

### N2 (minor, prose). `docs/superpowers/plans/phase-4-exclusions.txt:5099`

"The three in-crate conversion-row witnesses and Task 2's
`library_routine_argument_errors.rex` ... run unchanged under that mode since
the first fix."

They are not in-crate. They are the corpus programs
`lang/library_native_integer_arguments.rex`, `_object_arguments.rex` and
`_special_arguments.rex` -- which the text this diff *deleted* named, at the
same place. Two problems in one sentence: the wrong home, and a bare count
standing in for a set the same edit stopped naming. (The claim itself is true;
my collect_stress run has all three green under that mode.)

### N3 (minor, prose). `rust/corpus/phase-8.txt:205` and `:210-211`

The comment splits the world in two -- "the conversion rows a shipped extension
declares" are the corpus's, "the rows nothing shipped declares" are
`rexx-api/tests/values.rs`'s -- and the split is false. `orxmethod` ships
`TestMutableBufferLength` and `TestVariableReferenceValue`; the corpus
(`library_native_object_arguments.rex:171-172`) witnesses only their 88.914
refusal, and their conversion is unit-test-only. This is the same class of
claim as the "every conversion row" the round set out to fix, narrowed rather
than made derivable.

### N4 (minor, prose). `rust/crates/rexx-exec/src/lib.rs:5694`

"... once the queue has handed it over, and nothing else does." A negative
extent claim over holders that nothing in the tree can run. The walk is already
justified by C6 (delete it and the handler test reddens, the SYNTAX one does
not); the clause adds a claim and no coverage. Dropping it costs nothing.

### N5 (observation). `rust/crates/rexx-exec/src/dispatch/library.rs:627-631`
vs `rust/crates/rexx-exec/src/value.rs:1004-1007`

`native_found` decides "has its own `stringValue`" as `state.buffer().is_none()
&& state.pointer().is_none()`; `Interp::redirect_of` decides the same thing as
`state.buffer().is_some() || state.pointer().is_some()`. Neither matches on
`NativeState` exhaustively, so a fourth variant would be classified by fallthrough
at both sites rather than by decision, and only one of the two would be found by
grepping for the other. Not wrong today -- the two agree, and I checked the
agreement over every shape this review probed -- but it is two copies of one
rule.

### N6 (observation). `docs/superpowers/plans/phase-4-exclusions.txt:5089-5091`

The stream-name root is recorded as "witnessed by `run/tests.rs`'s unit test",
with the withdrawn corpus witness explained beside it. Measured here:
`collect_stress`'s L0 test witnesses it too, and more loudly -- removing the
root turns exactly five L0 programs rc 120. A reader deciding what the unit
test costs should know it is not the only thing standing behind that
`push_temp`.

## Global constraints

No `unsafe` added anywhere in the diff; no `set_var`/`remove_var`/
`set_current_dir`; no em-dash in any added line; no added comment states the
size of a set (the one that did, `NO_ALLOCATION_PROGRAMS`' "Thirty-six
programs", is what this round removed). The C++ tree, `samples/`, `ootest/`,
`oodocs/`, `testbinaries/` and `api/` are untouched.
