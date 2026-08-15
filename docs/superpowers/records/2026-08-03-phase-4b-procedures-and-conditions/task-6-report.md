# Task 6 report: `SIGNAL` to a label, and `SIGNAL VALUE`

Baseline confirmed green before any change: commit `fa534d3e`, `cargo test --workspace`
all green, corpus `36 of 36 matching`, assertions `4224 of 4259`, `cargo fmt --all
--check` and `cargo clippy --workspace --all-targets -- -D warnings` both clean.

Final commit: `53b75f90ebe72a9ba3fff2670e366326fc66d74c` ("Task 6: SIGNAL to a label,
and SIGNAL VALUE").

## Step 1: measurements

Every oracle invocation below is wrapped `( ulimit -v 1048576;
LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib
/home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`, run from a fresh `mkdir`ed
subdirectory of the session scratchpad, one program per directory, stdout/stderr/rc
read as three separate descriptors.

### 1. `SIGNAL` out of a nested block

Directory: `scratchpad/oracle-probes/p1`. Program `t1.rex`:

```rexx
trace r
say 'before'
do i = 1 to 3
  if i = 2 then signal there
  say 'i=' i
end
say 'after loop, not reached'
exit
there:
say 'reached there'
```

`rc=0`. stdout: `before\ni= 1\nreached there\n`. stderr (`cat -A`, `$` marks
end-of-line):

```
     2 *-* say 'before'$
       >>>   "before"$
     3 *-* do i = 1 to 3$
       >K>   "TO" => "3"$
     4 *-*   if i = 2 $
       >>>     "0"$
     5 *-*   say 'i=' i$
       >>>     "i= 1"$
     6 *-* end$
     3 *-* do i = 1 to 3$
       >>>     "1"$
       >>>     "2"$
     4 *-*   if i = 2 $
       >>>     "1"$
     4 *-*     then$
     4 *-*       signal there$
     9 *-* there:$
    10 *-* say 'reached there'$
       >>>   "reached there"$
```

`SIGNAL` abandons the `DO` unconditionally: no search, no name to match (unlike
`LEAVE`), so the loop's later iterations and the clause after `END` never run, and
the `SIGNAL` clause itself traces with no `>>>` line (it produces nothing).

**Surprise found here, not from this program directly but from trying to turn it
into a unit test byte-for-byte:** the second `do i = 1 to 3` re-echo (line 3, second
pass) is followed by two `>>>` lines (`"1"` then `"2"`) that this crate's own
`loop_advance`'s `LoopState::Controlled` arm does not reproduce -- already a
documented, disclosed "KNOWN GAP" in `run.rs` (comment at that function, citing
`DoBlock::checkControl`), unrelated to `SIGNAL` and out of this task's scope. My
first hand-transcribed unit test used `if i = 2` (firing on the *second* pass) and
failed with exactly that two-line gap, even though the implementation was otherwise
correct. Confirmed independently and unrelated to `SIGNAL`: a plain `do i = 1 to
3 / say 'i=' i / end` under `trace r`, no `SIGNAL` anywhere, shows the identical
gap on the oracle. Fixed by choosing a witness shape that fires on the loop's
*first* pass instead (`if i = 1`), both for the unit test and the corpus witness --
not by touching the loop code, which is out of scope here.

### 2. `SIGNAL` to a label not in the current body

Directory: `scratchpad/oracle-probes/p2`. Program `t2.rex`:

```rexx
say 'before'
signal nowhere
say 'not reached'
```

`rc=240`. stdout: `before\n`. stderr:

```
     2 *-* signal nowhere
Error 16 running .../t2.rex line 2:  Label not found.
Error 16.1:  Label "NOWHERE" not found.
```

Error 16.1, "Label not found" -- `rexx_inventory`'s generated catalogue already
carries `Error_Label_not_found` (16, 0, "Label not found.") and its own subcode
`Error_Label_not_found_name` (16, 1, `Label "&1" not found.`), so no catalogue
change was needed, only a new `Raised::label_not_found` constructor
(`error.rs`) mirroring the existing `syntax(...)` helpers.

### 2b. Quoted vs. bare `SIGNAL` targets (not in the brief's list verbatim, but needed
to know what `Signal::Label`'s single `Box<[u8]>` -- no `literal` flag the way
`Call::Named` has one -- actually means for case sensitivity)

Directory: `scratchpad/oracle-probes/p3`. Three one-line programs, `sub:` present
in each:

- `q1.rex`: `signal "sub"\n...` -- `rc=240`, `Label "sub" not found.`
- `q2.rex`: `signal Sub\n...` -- `rc=0`, reaches `sub:`.
- `q3.rex`: `signal "SUB"\n...` -- `rc=0`, reaches `sub:`.

**Both quoted and bare forms search the label table** -- unlike `CALL "name"`, which
never does at all (`a_quoted_call_name_never_reaches_the_label_table`, already in
the tree) -- but case-sensitively against the label's own upcased spelling, so a
lowercase quoted spelling still misses. This settled that `resolve_signal_target`
needs no `literal`/`search_labels` parameter the way `resolve_and_run_call` does:
`Signal::Label`'s single `Box<[u8]>` is already the right shape.

### 3. `SIGNAL` from inside a called routine to a label in the caller's body

Directory: `scratchpad/oracle-probes/p4`. Program `t4.rex`:

```rexx
call sub
say 'after call, not reached if sub signals out'
exit

sub:
say 'in sub'
signal caller_label
say 'sub not reached'
return

caller_label:
say 'caller label reached'
```

`rc=0`. stdout: `in sub\ncaller label reached\n`. stderr: empty.

**Surprising at first, and exactly the shape the brief warned about ("a defect
worth knowing about" and "run it instead").** `caller_label:` reaches from inside
`sub`, and `after call...` never prints. This is *not* `SIGNAL` crossing an
activation boundary on its own: at this phase every internal `CALL` target shares
its caller's exact body and label table (`resolve_and_run_call`'s own D9r comment
already says `Activation::nested(program, selector, ...)` reuses the same
`selector`, since no `::routine` directive gives a callee a body of its own yet).
So `resolve_signal_target`'s "search the running activation's own body" finds
`caller_label:` for the mundane reason that `sub`'s own body *is* the caller's --
the identical mechanism `resolve_and_run_call` already uses for `CALL`, just never
exercised by a `SIGNAL` before. And `after call...` never prints because `SIGNAL`,
unlike `RETURN`, never pops the activation it fires in: once `caller_label:`'s own
code runs out of instructions, the *callee's* activation falls off the end, which
ends the whole program (`Ended::Exited`), not merely the call.

### 4. `SIGNAL` out of an `INTERPRET` fragment

Directory: `scratchpad/oracle-probes/p5`. Program `t5.rex`:

```rexx
trace r
say 'before'
interpret "signal there"
say 'not reached'
exit
there:
say 'reached there'
```

`rc=0`. stdout: `before\nreached there\n`. stderr:

```
     2 *-* say 'before'
       >>>   "before"
     3 *-* interpret "signal there"
       >>>   "signal there"
     3 *-* signal there
     6 *-* there:
     7 *-* say 'reached there'
       >>>   "reached there"
```

Reaches the enclosing label, unlike `LEAVE`/`ITERATE`, whose own search stops dead
at the fragment boundary. The failure twin, directory `scratchpad/oracle-probes/p6`,
program `t6.rex` (no `trace r` this time):

```rexx
say 'before'
interpret "signal nowhere"
say 'not reached'
```

`rc=240`. stdout: `before\n`. stderr:

```
     2 *-* signal nowhere
     2 *-* interpret "signal nowhere"
Error 16 running .../t6.rex line 2:  Label not found.
Error 16.1:  Label "NOWHERE" not found.
```

Both clauses echo, innermost first, both carrying the enclosing `INTERPRET`'s own
line -- the same shape `run_fragment`'s own doc comment already tables for
`LEAVE`/`ITERATE`.

**This is the measurement that decided `Flow` needed a new variant rather than
reusing `Goto`.** `run_fragment` runs the fragment via `self.run_bounded(&code, 0,
fragment.len(), ...)`, where `code.body` is the *fragment's own* tiny instruction
array, entirely separate from the enclosing program's real body that
`resolve_signal_target` resolves against. A bare `Flow::Goto(target)` escaping from
inside the fragment would have that same `run_bounded` call range-check `target`
against `[0, fragment.len())` -- and since both index spaces start at 0, a
coincidental in-range match is the common case, not the exception. That would
silently resume stepping the *fragment's* unrelated instruction at that position
instead of escaping to the real target -- a wrong-body bug that would not show up
as a crash, only as wrong output, and only for programs whose target happens to
collide with the fragment's own size. `Flow::Signal(usize)` is a distinct variant
for exactly this reason: `run_bounded`'s range check (`Flow::Goto(target) if target
>= start && target <= end`) does not match it, so it always falls to the "other =>
propagate" catch-all in every one of `run_bounded`, `do_body_outcome`,
`leave_select` and `run_fragment`'s own post-processing match -- verified by
reading all four (none needed a new arm) and confirmed by the passing tests below.

**Nesting inside `DO`/`LOOP` or `IF` turned out to need no equivalent care**, which
is worth recording because it looks like it should. Three probes, directory
`scratchpad/oracle-probes/p9`/`p10`/`p11`:

- A label written inside a `DO`/`END` block (even a non-repeating one, used as an
  `IF`'s own body): `rc=209`, `Error 47.2: Labels are not allowed within a DO/LOOP
  block; found "INSIDE".`
- A label as an `IF`'s own `THEN` body, no `DO` wrapper: `rc=209`, `Error 47.3:
  Labels are not allowed within an IF block; found "INSIDE".`

`rexx-parse` already rejects both at parse time (pre-existing behaviour, also
already documented in `corpus/README.md`'s "Things this corpus learned the hard
way" section, which I found only after re-deriving the same fact from the oracle).
So a `SIGNAL` target can never sit strictly inside a range `run_bounded` is
currently absorbing a `Goto` into -- reusing `Goto` there would very likely have
worked by construction. It is the fragment boundary alone that cannot tolerate it.

### 5. `SIGNAL VALUE` where the value is not a label

Directory: `scratchpad/oracle-probes/p7`. Four one-line programs:

- `v1.rex`, `target = 'THERE'` / `signal value target` / ... / `there:` -- `rc=0`,
  reaches `there:`. stderr under `trace r`:
  ```
       2 *-* target = 'THERE'
         >>>   "THERE"
       3 *-* signal value target
         >K>   "VALUE" => "THERE"
       6 *-* there:
       7 *-* say 'value form reached'
         >>>   "value form reached"
  ```
  `SIGNAL VALUE` traces its own `>K>` line, keyword `"VALUE"`, exactly the shape
  `trace_keyword` (already in the tree, used by `WHILE`/`UNTIL`/`FOR`) produces.
- `v2.rex`, `target = 123` -- `rc=240`, `Label "123" not found.`
- `v3.rex`, `target = ''` -- `rc=240`, `Label "" not found.`
- `v4.rex`, `target = 'no such label'` -- `rc=240`, `Label "no such label" not
  found.`
- `v5.rex` (directory `p7`, same run), `target = 'there'` (lowercase) with `there:`
  present -- `rc=240`, `Label "there" not found.` (case-sensitive, same as the
  quoted `Label` form).

**No shape check at all** on the value before the label search -- a number, an
empty string, and an ordinary string all raise 16.1 naming that exact rendered
text, never a different error. Directory `scratchpad/oracle-probes/p8`, program
`v6.rex`, confirms the `>K>` trace's indent one `DO` deep (`current_value_indent`
directly, no `+2` the way `WHILE`/`UNTIL` carry, since `SIGNAL VALUE`'s own value
is evaluated as part of its *own* step, not as part of an enclosing instruction's):

```
     2 *-* target = 'THERE'
       >>>   "THERE"
     3 *-* do i = 1 to 1
       >K>   "TO" => "1"
     4 *-*   signal value target
       >K>     "VALUE" => "THERE"
     8 *-* there:
     9 *-* say 'reached, one do deep'
       >>>   "reached, one do deep"
```

## Did `Flow` need a new variant, and why

Yes -- `Flow::Signal(usize)`. Measurement 4 above is the direct reason: reusing
`Goto` would have `run_fragment`'s own bounded execution of the fragment
range-check an escaping `SIGNAL` target against the *fragment's* own tiny index
space rather than the real body `resolve_signal_target` resolved it against, since
both spaces start at 0 and a coincidental match is the likely case, not an edge
case. Measurements at the end of section 4 (47.2/47.3) rule out the symmetric
worry for `DO`/`LOOP`/`IF` nesting -- `rexx-parse` already forbids a label there, so
`run_bounded`'s own range absorption for those constructs can never see a `SIGNAL`
target inside its range in a legally parsed program. The new variant is forwarded
untouched by every `run_bounded`/`do_body_outcome`/`leave_select`/`run_fragment`
catch-all (each already has an `other => ...` arm; none needed editing) and
consumed only by `run_activation`'s own top-level dispatch
(`Flow::Signal(target) => self.activation_mut().pc = target`, the identical
treatment `Goto` gets there). The one test-only exhaustive match over `Flow`
(`run.rs`'s own `run_activated`, used by `run_source_traced`) needed restructuring
from a single `run_bounded` call into a small loop, since real `run_activation`
loops on `pc` and this test miniature previously did not.

`resolve_signal_target` (new, `run.rs`) mirrors `resolve_and_run_call`'s own label
resolution exactly -- reading `self.activation().program`/`.body`, not `code.body`
-- for the identical reason: inside a fragment the two differ, and a fragment's
own `labels` is always empty (47.1). No fallback exists the way `CALL`'s
builtin/external search does: `SIGNAL` raises `Raised::label_not_found` (16.1)
directly when nothing matches, which the generated catalogue already carries text
for.

## Step 4 note: `current_value_indent`

The brief asked me to check `SIGNAL` against the four pieces of level state
`resolve_and_run_call` saves/restores (`activation_indent`/`indent_offset`/
`clause_line_override`/`current_value_indent`), since Task 4 found a real gap
there. `SIGNAL` creates no nested activation and enters no fragment of its own, so
none of that machinery applies to it directly. I did find that the *`INTERPRET`
arm's own* three-piece save/restore (`activation_indent`/`indent_offset`/
`clause_line_override`, `run.rs`, the `Interpret` step arm) does **not** restore
`current_value_indent` the fourth way `resolve_and_run_call` does -- but tracing
through it, every consumer of `Flow::Signal` re-derives `current_value_indent` from
scratch via `step_in_temps_frame` before it is next read (the same reasoning that
made the gap invisible for `CALL` before two calls could land in one clause), and
measurements 4/4b above (`interpret_error_echo.rex`-shaped, `signal_forms.rex`)
found no divergence from the oracle. I did not change the `Interpret` arm; if a
future task finds a shape that does expose it, it is the same class of fix
Task 4's C1 was, in the same arm.

## Tests written (Step 2), then Step 3

Eight new unit tests in `rust/crates/rexx-exec/src/run.rs` (`tests` module),
directly transcribing measurements 1-5 above (with the KNOWN-GAP-avoiding `if i =
1` correction to measurement 1's witness):

- `signal_unwinds_a_nested_do_and_lands_on_its_label`
- `signal_to_an_undefined_label_raises_16_1`
- `a_quoted_signal_label_searches_case_sensitively_unlike_a_quoted_call`
- `signal_from_a_called_routine_reaches_a_label_in_the_shared_body_and_never_returns`
- `signal_escapes_an_interpret_fragment_to_reach_an_enclosing_label`
- `signal_to_an_undefined_label_inside_a_fragment_reports_both_clauses`
- `signal_value_traces_its_own_keyword_line_and_then_searches_like_label`
- `signal_value_targets_that_match_no_label_all_raise_16_1_naming_the_rendered_text`

**Verified these can fail, not just pass**, per the phase's own recurring finding
about vacuous tests: I temporarily replaced the `InstructionKind::Signal` step arm
with the pre-Task-6 fallback (`Err(Loud::instruction(&instruction.kind).into())`)
and re-ran just these eight --

```
running 8 tests
test run::tests::a_quoted_signal_label_searches_case_sensitively_unlike_a_quoted_call ... FAILED
test run::tests::signal_to_an_undefined_label_raises_16_1 ... FAILED
test run::tests::signal_from_a_called_routine_reaches_a_label_in_the_shared_body_and_never_returns ... FAILED
test run::tests::signal_value_targets_that_match_no_label_all_raise_16_1_naming_the_rendered_text ... FAILED
test run::tests::signal_unwinds_a_nested_do_and_lands_on_its_label ... FAILED
test run::tests::signal_value_traces_its_own_keyword_line_and_then_searches_like_label ... FAILED
test run::tests::signal_to_an_undefined_label_inside_a_fragment_reports_both_clauses ... FAILED
test run::tests::signal_escapes_an_interpret_fragment_to_reach_an_enclosing_label ... FAILED

test result: FAILED. 0 passed; 8 failed; 0 ignored; 0 measured; 244 filtered out
```

-- then restored the real implementation and re-ran:

```
running 8 tests
test run::tests::signal_from_a_called_routine_reaches_a_label_in_the_shared_body_and_never_returns ... ok
test run::tests::signal_to_an_undefined_label_raises_16_1 ... ok
test run::tests::signal_escapes_an_interpret_fragment_to_reach_an_enclosing_label ... ok
test run::tests::signal_value_traces_its_own_keyword_line_and_then_searches_like_label ... ok
test run::tests::signal_to_an_undefined_label_inside_a_fragment_reports_both_clauses ... ok
test run::tests::signal_value_targets_that_match_no_label_all_raise_16_1_naming_the_rendered_text ... ok
test run::tests::signal_unwinds_a_nested_do_and_lands_on_its_label ... FAILED   <- first attempt, see below
test run::tests::a_quoted_signal_label_searches_case_sensitively_unlike_a_quoted_call ... ok
```

The one failure on the first real-implementation run was the KNOWN-GAP collision
described in measurement 1 (my own hand-transcription bug, not an implementation
bug -- confirmed by decoding the byte diff and cross-checking against the
oracle's live output for a plain, `SIGNAL`-free `do i = 1 to 3` loop, which showed
the identical two-line gap). After switching that one witness to fire on the
loop's first pass, all eight pass:

```
running 8 tests
test run::tests::signal_escapes_an_interpret_fragment_to_reach_an_enclosing_label ... ok
test run::tests::signal_to_an_undefined_label_inside_a_fragment_reports_both_clauses ... ok
test run::tests::signal_unwinds_a_nested_do_and_lands_on_its_label ... ok
test run::tests::signal_value_traces_its_own_keyword_line_and_then_searches_like_label ... ok
test run::tests::signal_to_an_undefined_label_raises_16_1 ... ok
test run::tests::signal_from_a_called_routine_reaches_a_label_in_the_shared_body_and_never_returns ... ok
test run::tests::a_quoted_signal_label_searches_case_sensitively_unlike_a_quoted_call ... ok
test run::tests::signal_value_targets_that_match_no_label_all_raise_16_1_naming_the_rendered_text ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 244 filtered out; finished in 0.00s
```

## Ownership changes

- `owners.rs`: `InstructionKind::Signal(_)` stays `("Signal", Owner::Phase("4b"))`
  -- unchanged data, comment added explaining why (mirrors `Call`'s own comment,
  since `Signal::Trap` keeps the coarse tag phase-owned exactly as `Call::Trap`
  does for `Call`). `INSTRUCTION_TAGS`'s `InScope`/`Phase("4b")` counts (24/5) are
  unaffected.
- `loud.rs`: `instruction_arm` splits `Signal::Label`/`Signal::Value` into distinct
  names (matching `Call`'s own fully-arm-grained pattern); `expand_for_witnesses`
  shrinks `"Signal"`'s expansion from `["Signal", "Signal::Trap"]` to
  `["Signal::Trap"]`; the combined `"Signal"` witness row is deleted, leaving only
  `"Signal::Trap"`; `INSTRUCTION_WITNESSES` goes from 18 to 17 entries.
  **Also corrected a pre-existing, unrelated staleness**: the `INSTRUCTION_
  WITNESSES` const's own doc comment said "18 coarse tags" (true only through
  Task 3) and was never updated when Task 5 moved `Procedure`/`Use` in scope and
  shrank the coarse count to 16 -- `assert_witness_set_is_complete`'s own copy of
  the same arithmetic *was* kept correct at the time. Fixed both numbers together
  since Signal's own count is the one this task's edit already touches.

## Test summary

`cargo test --workspace`: all green (no failures). `cargo fmt --all --check`:
clean. `cargo clippy --workspace --all-targets -- -D warnings`: clean. Corpus:
**37 of 37 matching**, both REPORT mode and `REXX_CORPUS_GATE=1` STRICT mode.
Assertions: unchanged at 4224 of 4259 (SIGNAL is not exercised by
`ootest/ooRexx/base/expressions`).

`cargo test -p rexx-parse --test sourceline_oracle` initially failed (no oracle
expectation for the new `signal_forms` corpus program); regenerated **every**
`sourceline_oracle/*.txt` file per the documented recipe (`sourceline_oracle.rs`'s
own module comment) rather than hand-picking the new one -- confirmed idempotent
(`git status` showed only the new `signal_forms.txt` file, every existing file
byte-identical to what was already committed).

## Corpus witness

`rust/corpus/lang/signal_forms.rex`, added to `rust/corpus/phase-4b.txt` (with its
header's "still excludes" paragraph updated to drop `SIGNAL` and its arm-grained
paragraph updated to name `SIGNAL` alongside `CALL`). Not added to
`corpus/README.md`'s own tables: confirmed via `git log` that Tasks 1, 3, 4 and 5
did not add their own new corpus files there either (`interpret_error_echo.rex`,
`call_return.rex`, `call_expression.rex`, `call_procedure_expose.rex`,
`use_arg_forms.rex` are all likewise absent from that file), so skipping it here
matches established practice rather than introducing a one-off inconsistency.

All 37 corpus programs byte-identical to the oracle; no pre-existing
expected-byte literal changed.

## Concerns for the team lead

- None blocking. The one thing worth flagging explicitly: `run.rs`'s
  `Controlled`-loop retrace "KNOWN GAP" (pre-existing, already disclosed in the
  tree, unrelated to this task) is now something a `SIGNAL` witness has to know to
  route around, the same way `interpret_error_echo.rex` already had to for a
  different construct. Nothing about this task makes the gap worse or better; it
  is just one more shape of program that has to avoid it until it is fixed.

## Fix round 1 (review at `53b75f90`)

Review verdict: spec **NOT MET** (one Step 1 shape skipped -- `SELECT`; one
behaviour of the implemented construct missing -- `SIGL`), quality **NEEDS
WORK**. 1 Critical, 3 Important, 4 Minor (the four Minors deferred to the final
whole-branch review per the team lead). Full review at `task-6-review.md`.
Commit: `837bbf0f6b79a16c2640368583a448e2d1ba3576`.

### C1 (Critical): `SIGL`

Neither `SIGNAL` nor `CALL` (nor `ExprKind::Call`'s expression form) set `SIGL`,
a silent wrong answer the corpus's own module doc claims this tree does not
produce. Fixed properly rather than disclosed: both `SIGNAL` step arms and
`resolve_and_run_call` now call a new `set_sigl(line)` at the point of
transfer, `line` read from a new `Interp` field, `current_clause_line`.

**Why a field and not a parameter.** `resolve_and_run_call` is reached by
`eval_call` (`ExprKind::Call`, `f(1)`) from arbitrarily deep inside an
expression tree, which has no `source`/`instruction` of its own -- `eval`'s own
signature carries neither. Threading them through `eval`/`eval_node`'s entire
recursive call graph is exactly the "every arm in `eval.rs`" retrofit
`current_value_indent`'s own doc comment already declined for the identical
reason, so `current_clause_line` is cached the same way: set unconditionally by
`step_in_temps_frame` (previously the line was only computed when tracing;
`clause_site` is refactored into a new `clause_line` helper shared by both the
always-on cache and the trace-gated text extraction, so the override logic that
makes fragment clauses attribute to the enclosing line is written once).

**Measured, not assumed, including the two cases the brief called out by name:**

*Fragment case* (`before:` then `interpret "signal there"` then `there:`):
oracle and this crate both give `SIGL` = **2**, the enclosing `INTERPRET`
clause's own line, not anything internal to the fragment. Confirmed at two
levels of fragment nesting (`interpret 'interpret "signal there"'`, still 2) and
for `CALL` inside a fragment (`interpret "call sub"`, callee reads `SIGL` = 2).
Read the oracle's own C++ directly to understand *why*, not only *that*:
`RexxActivation::signalTo` (`execution/RexxActivation.cpp`) delegates a
`SIGNAL` fired inside an interpret-created activation straight to its parent
(`if (isInterpret()) { stopExecution(RETURNED); parent->signalTo(target); }`),
so what ends up in `SIGL` is the *parent's* own currently-executing
instruction -- the `INTERPRET` clause itself. This crate does not create a
nested activation for `INTERPRET` (a deliberate, existing design choice), so
`current_clause_line`'s reuse of `clause_line_override` reproduces the same
*observable* answer through a different mechanism, exactly the way trace/error
echo already do for the identical architectural gap.

*Nested case* (`call outer` / `outer: call inner` / `inner: return` / `return`):
oracle and this crate both give, in order, outer's own `SIGL` = 1 (the first
`CALL`'s line), inner's own `SIGL` = 7 (the second `CALL`'s line), outer's
`SIGL` *after* inner returns = 7 (unchanged -- an ordinary shared-pool
variable, never restored at the activation boundary), and the main body's own
`SIGL` after outer returns = 7 as well. `PROCEDURE` isolation measured
separately and also matches: a `PROCEDURE`d callee's own `SIGL` starts
uninitialised (`SIGL`, the derived name) regardless of what the caller's was,
and the caller's own `SIGL` (already set by the `CALL` itself, before the
callee ran) is untouched by whatever the isolated callee does with its own copy
afterward.

*Value creation*: `self.text(line.to_string().as_bytes())`, not `self.number`.
Measured: a `SIGL` of `22` renders as `22` under `NUMERIC DIGITS 1`, where an
arithmetic result of the same magnitude would round to `2E+1` -- matching the
oracle's own `new_integer(lineNum)`, an integer object that always renders in
full decimal.

*Ordering*: arguments are evaluated under whatever `SIGL` was already in force,
and only then is it overwritten -- measured (`signal there` / `there: call sub
sigl` into `sub: use arg a`): the argument reads back `1` (the `SIGNAL`'s own
line), not `3` (the `CALL`'s own line), matching `internalCall`'s own C++
ordering (it receives arguments already evaluated by its caller).

Five shapes pinned in a new unit test, `sigl_is_set_at_every_control_transfer`;
the corpus witness (`signal_forms.rex`) now prints `sigl` at every successful
transfer too, and the whole file is re-verified byte-for-byte against a live
oracle run from a fresh directory (zero diff on stdout or stderr, no
normalisation needed).

### I1 (Important): the `Flow::Signal`-vs-`Goto` decision had no failing test

Added `signal_out_of_a_fragment_does_not_collide_with_the_fragments_own_index_
space`: `say 'A'` / `interpret "nop; signal here; say 'WRONG BRANCH RAN'"` /
`here: say 'landed correctly'`. `here:` sits at enclosing body index 2, the
fragment has 3 instructions, `2 <= 3` collides under `run_bounded`'s own
absorption guard. Verified by actually collapsing both `Ok(Flow::Signal(
target))` sites to `Ok(Flow::Goto(target))`, rebuilding, and confirming
`rexx-run` prints `A` / `WRONG BRANCH RAN` / `landed correctly` -- then
reverting before running anything else.

**No second, self-referential ("g2") test was added, and this is a deliberate
deviation from the letter of the ask, stated plainly rather than silently
skipped.** I worked through every way to make a "bounded" variant of the
reviewer's own self-referential shape (a label at enclosing index 0, a
one-instruction fragment `"signal top"`) and could not find one: any absorption
landing *at or before* the `SIGNAL`'s own position inside the fragment
reproduces the identical deterministic `Goto` on the next pass through
`run_bounded`'s `while pc < end` loop, which has no iteration budget -- so it
does not fail, it spins forever. Moving the target *past* the `SIGNAL`'s own
position (this task's actual test) is what makes the wrong run terminate at
all; that is not a weaker version of the self-referential case, it is the only
member of the family that can be tested live without risking a hang the next
time this exact regression guard itself regresses. I judged a test that could
hang `cargo test`/CI indefinitely to be a worse outcome than one fewer test,
consistent with this project's own precedent (`MAX_ACTIVATION_DEPTH`, D19/I6:
convert an unbounded case into a bounded, reportable one rather than accept an
unbounded one) -- and documented the reasoning in both `Flow::Signal`'s own doc
comment and the passing test's, rather than leaving the gap unexplained.

### I2 (Important): two false/unsupported claims in `Flow::Signal`'s doc comment

Both removed. "`interpret "signal there"` ... only holds if nothing along the
way can mistake the escaping jump for one of its own" was false: measured, it
holds under the `Goto` collapse too (that witness's own target sits past its
one-instruction fragment, so it never collides) -- meaning that measurement was
never actual evidence for the design decision, and the comment now says so and
points at the program that is (I1's new test). "Coincidental overlap is the
common case, not the exception" was unmeasured and false for every program
either the report or the doc comment cited; replaced with the actual,
narrower, true mechanism (`run_bounded`'s guard is `target <= end` with
`start == 0`, so overlap depends only on the target's index versus the
fragment's length, unrelated to where the label actually sits). Also folded in
M2's own finding while the same paragraph was already being rewritten: `SELECT`
(and 47.4) added alongside `DO`/`LOOP`/`IF` in the "needs no equivalent care"
list, and `If`'s own true-branch forwarding arm named alongside the other four
catch-alls.

### I3 (Important): `SIGNAL` out of a `SELECT` had no measurement, test, or witness

Measured (directory `scratchpad/sigl-probes/s13`, program with a leading
`trace r`): `say 'before'` / `select` / `when 1 = 1 then signal there` /
`otherwise say 'not reached'` / `end` / `say 'after select, not reached'` /
`exit` / `there:` / `say 'reached there sigl:' sigl` -- oracle rc 0, stdout
`before\nreached there sigl: 4\n` (line 4 is `when 1 = 1 then signal there`,
all three of `WHEN`/`THEN`/`SIGNAL` sharing that one source line), `after
select` never runs, and this crate's own build matches byte for byte,
including `SIGL`. Pinned as `signal_out_of_a_select_unwinds_it_and_lands_on_
its_label` (trace-based, exact bytes, line numbers decremented by one to match
`run_source_traced`'s externally-set mode as usual) and folded into
`signal_forms.rex` as a `SELECT` block between the `DO` and `SIGNAL VALUE`
sections.

### Measured `SIGL` values requested for the report

- **Fragment case**: `interpret "signal there"` on line 2 -> `SIGL` = `2` (the
  enclosing `INTERPRET`'s own line), both at one and two levels of fragment
  nesting, and for `CALL` inside a fragment too.
- **Nested case**: `call outer` (line 1) into `outer: call inner` (line 7) ->
  outer's `SIGL` = `1`, inner's `SIGL` = `7`, and `7` survives back up through
  both returns to the main body -- never restored, an ordinary shared-pool
  variable. A `PROCEDURE`d callee's own `SIGL` starts uninitialised regardless,
  and the caller's own value (already set by the `CALL` itself) is unaffected
  by the isolated callee's copy.

### Test summary after fix round 1

`cargo test --workspace`: 68 `test result: ok` lines, 0 failures (verified via
`grep -c "test result: FAILED"` on the raw log, not by eye). `cargo fmt --all
--check` and `cargo clippy --workspace --all-targets -- -D warnings` both
clean. Corpus: **37 of 37 matching**, both REPORT mode and
`REXX_CORPUS_GATE=1` STRICT, re-verified from a fresh directory against a live
oracle run with **zero** raw byte diff on stdout or stderr (no `DEVIATION 0`
normalisation needed at all) after extending `signal_forms.rex`. Regenerated
every `sourceline_oracle/*.txt` again per the documented recipe; only
`signal_forms.txt` changed (idempotent, confirmed via `git status`).

### Concerns

None blocking. The one explicit deviation from the letter of the review (no
live self-referential `Flow::Signal`-vs-`Goto` test) is stated above with the
reasoning; happy to discuss if the team lead weighs the hang risk differently.

## Fix round 2 (review at `837bbf0f`)

One finding: `current_clause_line` was set unconditionally per clause by
`step_in_temps_frame` (modelled explicitly on `current_value_indent`'s own
doc comment) but never saved and restored around a nested activation in
`resolve_and_run_call` -- the third instance of this exact shape
(`current_value_indent` itself, Task 4's C1; the `INTERPRET` arm's own
still-open omission, recorded for Task 7; and now this field). Commit:
`0fce4f00b74e7c343b2142a9580666b7e84f9851`.

### Measured, reproducing the review's own finding

Directory `scratchpad/fixround2/p1`:

```rexx
say f(1) + g(2)
exit
f:
say 'in f'
return 1
g:
say 'sigl in g:' sigl
return 2
```

Oracle: `in f` / `sigl in g: 1` / `3`. Before this fix: `in f` / `sigl in g: 5`
/ `3` -- line 5 is `return 1`, `f`'s own last clause. After: matches the
oracle exactly.

### Fix

Bundled `current_value_indent` and `current_clause_line` into one `Copy`
struct, `ClauseState`, with `Interp` carrying a single `clause_state: 
ClauseState` field rather than two separate ones. `resolve_and_run_call`'s
own save/restore becomes one struct copy each way (`let saved_clause_state =
self.clause_state; ...; self.clause_state = saved_clause_state;`) instead of
a per-field pair. `ClauseState`'s own doc comment states the property that
decides membership -- set per clause by `step_in_temps_frame`, and read
somewhere that can run after a nested activation has already run and
returned within the same clause (`say f(1) + g(2)` is what makes the second
half observable at all) -- and explains why the other four pieces of level
state `resolve_and_run_call` already saves (`activation_indent`/
`indent_offset`/`clause_line_override`/`call_context`) do not share it: they
are level state *for the callee*, set once per call to a value the callee
computes, never refreshed per clause the way `ClauseState`'s own fields are.

Every `self.current_value_indent`/`self.current_clause_line` call site
(~65 across `run.rs`/`lib.rs`/`eval.rs`) is a mechanical access-path rename
to `self.clause_state.current_value_indent`/`.current_clause_line`; neither
field itself was renamed, so every existing doc comment naming either field
is still accurate, and the rename was verified safe by compilation plus the
full existing test suite (a pure access-path change with no logic change).

**Is this the mechanical enumeration, or a hand-maintained list?** The
mechanical enumeration: adding a third field of this shape now means adding
it to `ClauseState`'s own definition, which is restored by the existing
`self.clause_state = saved_clause_state;` assignment with no second edit
required anywhere. A field added directly to `Interp` instead would still
need someone to have read the struct's own doc comment first, but there is
now exactly one place (the struct itself) where that property is stated,
rather than a comment at each nested-activation boundary repeating it.

### `INTERPRET` arm: measured, not assumed

The brief specifically warned that the `clause_line_override` interaction is
not obvious. Three programs, all under `trace r`, all compared byte for byte
on both stdout and stderr against a live oracle run from a fresh directory,
all **zero diff**:

1. `interpret "say f(1) + g(2)"` (directory `scratchpad/fixround2/p2`,
   untraced first, then `scratchpad/fixround2/p3` with `trace r` and full
   transcript comparison) -- two calls inside one *fragment* clause.
2. `say f(1) + g(2)` where `f`'s own body is `interpret "nop"` (directory
   `scratchpad/fixround2/p4`) -- a call whose own callee uses `INTERPRET`
   internally, nested one level inside `resolve_and_run_call`'s own restore.

No change was made to the `INTERPRET` arm. `INTERPRET` never creates the
"two things share one clause" shape that makes the omission observable in
the first place: it is always a whole clause on its own, never combined with
a second call or `SIGNAL` the way an expression can be (`say f(1) + g(2)`
has two calls in one clause; there is no equivalent "two `INTERPRET`s in one
clause"). `resolve_and_run_call`'s own restore already covers any call that
happens to run inside a fragment, or a fragment that happens to run inside a
call, regardless of nesting, because it saves and restores around the whole
nested `run_activation` call irrespective of what that call does internally.

### Test added

`current_clause_line_is_restored_after_a_nested_expression_call`, `current_
value_indent`'s own sibling test, same shape (`say f(1) + g(2)` / `f: say
'in f' return 1` / `g: say 'sigl in g:' sigl return 2`), asserting `stdout`
is `in f\nsigl in g: 1\n3\n`. Verified both ways per this project's own
"pair a refusal with its adjacent success" rule: narrowing the restore back
to `current_value_indent` alone makes this new test fail (`sigl in g: 5`)
while its neighbour (`current_value_indent`'s own test) keeps passing
unaffected; with the real fix in place, both pass.

### Test summary after fix round 2

`cargo test --workspace`: 68 `test result: ok` lines, 0 failures (`grep -c
"test result: FAILED"` on the raw log). `cargo fmt --all --check` and `cargo
clippy --workspace --all-targets -- -D warnings` both clean. Corpus:
**37 of 37 matching**, both REPORT mode and `REXX_CORPUS_GATE=1` STRICT.
Assertions unchanged at 4224 of 4259. No corpus `.rex` file was touched this
round, and `cargo test -p rexx-parse --test sourceline_oracle` passed with no
regeneration needed; `git status` after the run showed no changes under
`sourceline_oracle/`.

### Concerns

None.
