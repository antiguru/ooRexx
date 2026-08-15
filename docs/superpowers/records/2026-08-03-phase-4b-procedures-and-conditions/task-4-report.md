# Task 4 report: `ExprKind::Call` -- the internal-function form

## Baseline confirmation

HEAD at start was `344677e6`, matching the brief. `cargo test --workspace` was
green before any change: 893 passed / 0 failed / 4 ignored, corpus `33 of 33
matching`, assertions `4224 of 4259`.

## Design decision beyond the literal file list

The brief lists `eval.rs` and `tests/owners.rs` as the files to modify. Two
things turned out to require touching more than that, both load-bearing for
correctness rather than incidental:

1. **`run.rs`: `exec_call` was split into a shared `resolve_and_run_call`
   (now `pub(crate)`) plus a thin `exec_call` that only does `CALL`'s own
   `RESULT`/`Flow` translation.** `eval_call` needs the identical resolution
   order, argument-evaluation discard, `MAX_ACTIVATION_DEPTH` guard and
   three-piece indent bookkeeping (`activation_indent`/`indent_offset`/
   `clause_line_override`) that `exec_call` already had -- confirmed to
   matter for the expression form too by measurement (see the `trace r`
   transcript below). Duplicating ~80 lines of that machinery by hand was
   rejected as exactly the copy-paste-drift shape this project's other
   shared tables (`owners.rs`, `phase-4-exclusions.txt`) exist to avoid.
   The refactor is behavior-preserving: `exec_call`'s own ~20-test suite in
   `run.rs` was run before and after and is unchanged, byte for byte, plus
   the corpus differential (`call_return.rex`) stayed matching.
2. **`error.rs`/`lib.rs`: a new `Failure::Exited(Option<ObjRef>)` variant.**
   `EXIT` inside a routine reached through the expression form (or that
   routine falling off its own end) must end the whole program, exactly as
   it does through `CALL` -- measured on the oracle (transcript below). For
   `CALL`, that travels through `Ok(Flow::Exit(..))`/`Ok(Ended::Exited(..))`,
   a success channel `step`/`run_activation` both have room for. `eval`'s own
   return type is a plain `ObjRef` with no such room, so the event needs a
   different channel for the expression form, and `Failure::Exited` is it:
   constructed once in `eval_call`, then propagated by every enclosing `?`
   completely unremarked (the existing generic "an `Err` escaped, seal a site
   and re-throw" paths in `step_in_temps_frame`/`resolve_and_run_call` don't
   need to know the variant exists, since sealing a site nothing ever prints
   is harmless). `execute`'s top-level match (`lib.rs`) is the one place it
   is finally read, and it is handled exactly like an ordinary `Ok(value)`:
   same `exit_code_for`, no stderr report. Verified end-to-end with a
   dedicated unit test (rc 5, empty stdout, empty stderr, matching the
   oracle exactly).

Both changes were run against the *entire* existing test suite before being
considered safe, not just the new tests.

## Oracle transcripts (all captured in a fresh, empty scratch subdirectory, per the probe rule -- never the scratchpad root, which is on the external-routine search path)

Wrapped as `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )` throughout, stdout/stderr/exit read as three separate descriptors.

**1. `noval.rex`** (a routine returning no value in expression form -- the
brief's own Step 3 measurement):

```
say f(1)
exit
f: return
```

```
EXIT:212
STDOUT: (empty)
STDERR:
     1 *-* say f(1)
Error 44 running .../noval.rex line 1:  Function or message did not return data.
Error 44.1:  No data returned from function "F".
```

**2. `noval2.rex`** (falling off the routine's own end, no `RETURN` at all --
a *different* event from the above, confirmed by measurement rather than
assumed identical):

```
say f(1)
exit
f: nop
```

```
EXIT:0
STDOUT: (empty)
STDERR: (empty)
```

This ends the whole program silently, exactly like `CALL`'s own documented
"falling off the end ends the program" behaviour -- not an error.

**3. `tr1.rex`** (the D2r indent question under `trace r`, which the brief
does not measure but which the `resolve_and_run_call` refactor needs to get
right):

```
trace r
zz = f(1) + 1
say zz
exit
f: return 41
```

```
EXIT:0
STDOUT: 42
STDERR:
     2 *-* zz = f(1) + 1
     5 *-*   f:
     5 *-*   return 41
       >>>     "41"
       >>>   "42"
     3 *-* say zz
       >>>   "42"
     4 *-* exit
```

Confirms the callee's own clauses echo at the calling clause's own printed
indent plus two (D2r), the same rule `CALL` already carries.

**4. `litform.rex`** (`CallTarget::Literal` never searches the label table --
re-measured independently, matching the brief's own claim exactly):

```
say "f"(1)
exit
f: return 41
```

```
EXIT:213
STDOUT: (empty)
STDERR:
     1 *-* say "f"(1)
Error 43 running .../litform.rex line 1:  Routine not found.
Error 43.1:  Could not find routine "f".
```

4b has not built the builtin/external steps that would distinguish "not a
label" from "not anything at all", so this crate's own answer stays the loud
`4c` fallback (`CallTarget::Literal` never even attempts the label search),
not a fabricated 43.1 -- exactly the same shape `CALL "SUB"` already has.

**5. `resultcall.rex`** (RESULT untouched by the expression form):

```
result = 'before'
zz = f(1)
say result
exit
f: return 99
```

```
EXIT:0
STDOUT: before
STDERR: (empty)
```

**6. `exitcall.rex`** (`EXIT` inside a routine reached through the expression
form -- the scenario `Failure::Exited` exists for):

```
say f(1)
exit 9
f: exit 5
```

```
EXIT:5
STDOUT: (empty)
STDERR: (empty)
```

## The measured error for a routine returning no value in expression form

**Error 44.1, rc 212** ("256 - 44"): major line "Function or message did not
return data.", sub line `No data returned from function "F".` (the resolved
label's own upcased spelling substituted for `&1`). Implemented as
`Raised::no_data_returned(name: &[u8]) -> Raised` in `error.rs`
(`Raised::syntax(44, 1, vec![name])`), reusing the existing generated message
catalogue (`rexx-inventory`, sourced from `interpreter/messages/rexxmsg.xml`
lines 3715-3734) rather than hand-transcribing the text. This is a genuinely
different event from falling off the routine's own end with no `RETURN` at
all, which is `Ended::Exited` (ends the whole program, rc 0, silent) -- the
two were confirmed to differ by running both, not assumed to be the same
"no value" case.

## What was left for 4c at the fallback, and where a 4c implementer finds it

`eval_call` (`rust/crates/rexx-exec/src/eval.rs`) resolves `CallTarget::
Symbol` against the calling activation's own internal labels only.
`CallTarget::Literal` never searches labels at all (symmetric with `CALL
"SUB"`). Either way, a name that does not resolve reaches
`resolve_and_run_call`'s (`run.rs`) `Loud::unresolved_call(name)`, which is
the exact "routine \"NAME\" is not implemented (4c)" fallback `CALL`'s own
unresolved names already produce. The I25 split -- internal routine first
(4b), builtin second (4c), external third (Phase 7) -- is now stated directly
in `eval_call`'s own doc comment in `eval.rs`, not only in
`phase-4-exclusions.txt` and test-file comments as before this task. A 4c
implementer's builtin lookup belongs **between** the label search and the
call to `resolve_and_run_call`'s fallback, not after it -- i.e. inside
`eval_call` itself, resolving a builtin name before ever reaching the loud
path. `resolve_and_run_call` and `Ended`/`Failure::Exited` are already
`pub(crate)`/crate-visible, so nothing about the activation machinery needs
touching again for that.

## Ownership bookkeeping updated (Task 0's Step 5, all five items)

`ExprKind::Call` moved from `Owner::Phase("4b")` to `Owner::InScope`:
`tests/owners.rs`'s `EXPR_TAGS` and `EXPECTED_OUT_OF_SCOPE` (row deleted),
the pinned counts (`EXPR_TAGS`: 10 in scope / 5 phase-owned, was 9/6);
`tests/loud.rs`'s `EXPR_WITNESSES` (the old `Call`/`"say foo(1)"` row
deleted) and its own count assertions; `src/lib.rs`'s `expr_owner` (now
`None` for `ExprKind::Call`, joining the pattern `InstructionKind::Call`'s
own `None` arms already use). `tests/coverage.rs`'s module doc and
`docs/superpowers/plans/phase-4-exclusions.txt`'s "EXPRKIND OWNERSHIP"
section were both updated to state the closure rather than leave a now-false
claim that `ExprKind::Call` is still split.

A new corpus witness, `rust/corpus/lang/call_expression.rex`, was added
(listed in `phase-4b.txt`) to satisfy `coverage.rs`'s
`every_in_scope_variant_is_witnessed_by_the_phase_subsets` -- it exercises
both the D2r indent rule and RESULT-untouched under `trace r`, matching the
oracle byte for byte. This also required regenerating
`rexx-parse/tests/sourceline_oracle/call_expression.txt` (Phase 3's gate,
walks every file in `corpus/lang/`), using the project's documented driver
script from a scratch directory.

## Test output

`cargo test -p rexx-exec --lib eval::` (the new tests, run in isolation):

```
running 36 tests
test eval::tests::a_builtin_name_still_fails_loudly_naming_4c ... ok
test eval::tests::a_literal_call_target_never_reaches_the_label_table ... ok
test eval::tests::a_routine_returning_no_value_in_expression_form_raises_44_1 ... ok
test eval::tests::an_exit_inside_a_routine_reached_by_expression_call_ends_the_whole_program ... ok
test eval::tests::an_internal_function_returns_its_value_into_an_expression ... ok
test eval::tests::an_internal_functions_expression_form_does_not_touch_result ... ok
[... 30 pre-existing eval.rs tests, all still ok ...]
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 184 filtered out
```

`cargo test --workspace` (full suite, final): **899 passed, 0 failed, 4
ignored** (baseline 893/0/4; +6 new tests, none removed). Corpus: **34 of 34
matching** (baseline 33 of 33; +1 new witness, no existing program changed).
Assertions: **4224 of 4259** (unchanged from baseline -- the 35 not-passing
rows are all `XRANGE`/message-send blocked, unrelated to this task).
`cargo fmt --all --check`: clean. `cargo clippy --workspace --all-targets --
-D warnings`: clean.

## What is NOT in this task

`TRACE I`'s own `>F>`/`>A>` lines (Task 9's) -- `eval`'s `trace_intermediate`
hook still has no arm for `ExprKind::Call`, deliberately. `USE ARG`/`ARG()`
(still loud) -- argument values are evaluated (for their side effects/
failures) and discarded, the same shape Task 3's `CALL` already established.
Builtins and external routines (4c, Phase 7) -- the loud `4c` fallback,
described above.

## Fix round 1 (commit `068373c5`, parent `ffb21c2f`)

Review (`task-4-review.md`): 1 Critical, 2 Important, 5 Minor. `Failure::
Exited` and the `exec_call`/`resolve_and_run_call` split were both ruled
sound and kept unchanged. Fixed: C1, I1, I2, M4. M1/M2/M3/M5 deferred to the
final whole-branch review, per the reviewer's own instruction.

**C1 (Critical) -- `current_value_indent` not restored across a nested
activation.** `resolve_and_run_call` saved and restored `activation_indent`/
`indent_offset`/`clause_line_override` on the way out of a nested activation,
but not `Interp::current_value_indent`, which `run_activation` overwrites on
every clause the callee steps. Before Task 4 at most one activation could be
entered per clause, so the next clause's own `step_in_temps_frame` re-set the
field before anything read it -- the gap was unobservable through `CALL`
alone. `say f(1) + g(2)` enters two activations in one clause, and without
the restore `g`'s own base indent (and everything computed from it) is
derived from `f`'s last clause instead of the caller's own. Not confined to
`TRACE`: an ordinary, untraced program with a raise inside the second call
reports that error's clause echo at the wrong indent, which the reviewer
measured directly.

Fixed with the one-line restore the reviewer specified
(`self.current_value_indent = base_indent;`, `run.rs`, beside the other
three restores). Added `current_value_indent_is_restored_after_a_nested_
expression_call` (`run.rs`'s own unit tests, beside `a_callees_clauses_
echo_at_the_calling_clauses_indent_plus_two`), asserting the exact oracle
transcript for `trace r` / `say f(1) + g(2)` against `f: return 1` / `g:
return 2`. **Verified the test fails without the fix**: temporarily
commented out the restore line, re-ran the test, got the exact wrong-indent
mismatch (`g`'s clauses and the trailing `>>>` at indent 4/6 instead of
2/4), then restored the fix and confirmed the test passes again and the
whole workspace suite stays green.

The oracle transcript for the regression test's expected bytes was measured
with a leading `trace r` clause (since `run_source_traced` sets trace mode
externally and the test source carries no such clause), then every line
number decremented by one -- verified as the correct transformation by
independently re-deriving `a_callees_clauses_echo_at_the_calling_clauses_
indent_plus_two`'s own already-checked-in expected bytes the same way, from
its own source with a real `trace r` prepended, and confirming an exact
match before trusting the same method for the new test.

**I2 (Important) -- the 4c hand-off pointed at the wrong function.**
`eval_call`'s own doc said a 4c implementer's builtin lookup belongs "between
this function's own label search" and the loud fallback -- but `eval_call`
has no label search; it lives in `resolve_and_run_call` (`run.rs`), shared
with `exec_call`. Corrected `eval_call`'s doc comment (`eval.rs`) to say the
builtin lookup belongs inside `resolve_and_run_call`, between
`activation_body.labels.get(name)`'s miss and `Loud::unresolved_call`, and
that `eval_call` itself owns no expression-only resolution step -- the only
thing specific to it is what happens *after* `resolve_and_run_call` returns.

**I1 (Important) -- the new corpus witness's stated reason for existing was
false.** The witness's header (and its `phase-4b.txt` entry) claimed the D2r
indent rule as pinned by the differential run against the oracle. It is not:
`corpus.rs`'s DEVIATION 0 (`support::normalize_stderr`) collapses the run of
spaces between a trace line's marker and its content, which is exactly what
D2r's indent produces -- confirmed directly by reading `normalize_stderr`'s
own scope statement (collapse the space run at `PREFIX_OFFSET`, nothing
else) and by the reviewer's own mutation test. Rewrote both `call_expression.
rex`'s own header and its `phase-4b.txt` entry to state what the differential
run actually pins (stdout byte-exact, including `result: before`, which
discriminates a wrongly-settled `RESULT`; stderr's clause sequence and line
numbers) and that the indent itself is pinned by `run.rs`'s unit tests
instead, outside `normalize_stderr`'s reach. Also corrected the identical
pre-existing false claim in `call_return.rex`'s own `phase-4b.txt` entry
(Task 3's), per the reviewer's instruction to fix both together -- left
`call_return.rex`'s own internal header comment (which makes the same claim)
untouched, since the review named only its `phase-4b.txt` entry and touching
a Task-3-owned corpus file's content was outside this fix round's specific
asks; flagging it here as a fast-follow rather than fixing it opportunistically.

Editing `call_expression.rex`'s header changed its line count, so
`rexx-parse/tests/sourceline_oracle/call_expression.txt` was regenerated
against the real oracle (same driver script and recipe as the original
task), giving `count 38` (was `count 28`).

**M4 (Minor, fixed) -- `Failure::Exited`'s second producer had no test.**
`Ended::Exited` (and so `Failure::Exited`) has two producers: an `EXIT`
instruction, and a routine falling off its own end (`run_activation`'s own
`Ok(Ended::Exited(None))` when its instruction loop runs out) -- only the
first had a test. Added `a_routine_falling_off_its_own_end_in_expression_
form_also_ends_the_whole_program` (`eval.rs`, beside the `EXIT` test),
matching the oracle exactly (`say f(1)` / `exit` / `f: nop` at end of file:
rc 0, empty stdout, empty stderr).

**M1/M2/M3/M5 -- deferred**, per the reviewer's own instruction to defer them
to the final whole-branch review.

### Test output (fix round 1)

`cargo test --workspace`: **901 passed, 0 failed, 4 ignored** (round-0
baseline 899/0/4; +2 new tests, the C1 regression test and the M4 test).
Corpus: **34 of 34 matching**, unchanged from round 0 -- confirming no
pre-existing expectation moved and the slot count is stable. Assertions:
**4224 of 4259**, unchanged. `cargo fmt --all --check`: clean (after running
`cargo fmt --all` once to fix two lines the new tests introduced).
`cargo clippy --workspace --all-targets -- -D warnings`: clean.

Commit `068373c5`, parent `ffb21c2f` (a concurrent, non-overlapping commit
from another agent narrowing DEVIATION 0's own justification in
`phase-4-exclusions.txt`, itself prompted by this review's C1 finding -- no
overlap with this fix round's files).

### Follow-up: `call_return.rex`'s own header (commit `994a92b7`)

I had flagged, rather than fixed, that `call_return.rex`'s own internal
header comment (as opposed to its `phase-4b.txt` entry, which I1 already
corrected) makes the identical false indent-pinning claim, reasoning that
it was a Task-3-owned file outside this fix round's specific asks.

**Ruling from the team lead: fix it.** "Task-3-owned corpus file" is not a
real constraint -- corpus files are not owned by the task that created them
once landed, and the standing rule ("a comment that states something false
must be corrected or removed, not hedged or left") applies regardless of
which file the false claim sits in. This phase's dominant recurring defect
has been exactly this shape: a false claim corrected in one place while its
twin survives in a neighbouring file (cited: Task 2's round-4 correction
reaching one of two sites and not the other, caught by the next review).

Rewrote `call_return.rex`'s header with the same correction applied to
`call_expression.rex`: states what the differential run actually pins
(stdout byte-exact RESULT reads, stderr's clause sequence and line numbers)
rather than the D2r indent, which DEVIATION 0 normalises away and which
`run.rs`'s unit tests pin instead. Also reworded the paragraph explaining
why the `DO` block is plain rather than repetitive -- the real reason is
avoiding an unrelated known gap entering the comparison, not "pinning
indents", which this program no longer claims to do.

The header's line count moved (54 -> 63 lines), so
`rexx-parse/tests/sourceline_oracle/call_return.txt` was regenerated
against the real oracle (`count 63`), using the same driver script and
recipe as every other regeneration in this task.

**Verification:** `cargo test --workspace` -> **901 passed, 0 failed, 4
ignored** (unchanged from fix round 1 -- no test code touched this round).
Corpus: **34 of 34 matching**, unchanged -- confirms no pre-existing
expectation moved. Assertions **4224 of 4259**, unchanged. `cargo fmt --all
--check` and `cargo clippy --workspace --all-targets -- -D warnings`: both
clean (no Rust source touched this round; corpus/oracle files only).

Commit `994a92b7`, parent `068373c5`.
