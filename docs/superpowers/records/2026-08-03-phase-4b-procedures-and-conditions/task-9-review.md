# Task 9 review -- the call trace surface, and three absent value lines

Reviewed: `233fbd8b` on `e72cc19f`. `d2f490bc`/`d24dde78` excluded as instructed.

Method: every claim below is labelled **[ran]** or **[read]**. All oracle runs used
the mandated wrapper from `.../scratchpad/t9rev/{wit,old,ref,tl,ra,pr,gap,rt,deep,four}`,
each `mkdir`'d for this review. stdout, stderr and exit status were captured as three
separate files throughout; no `2>&1`. A baseline binary was built from a detached
worktree at `e72cc19f` for A/B.

Tracked files were mutated six times to test that tests can fail; each was reverted
with `git checkout --` and `git status --porcelain` confirmed clean after each. The
tree is clean and the release binary rebuilt from the committed source.

---

## Verdict 1 -- spec compliance

| requirement | verdict | how |
|---|---|---|
| Step 1: five existing expectations regenerate byte-identical | **met** | [ran] regenerated all five from the live oracle with the module doc's own recipe; `diff` empty for all five |
| Step 2: `>I>`/`<I<` row as a 4c deferral, not unreachability, with Task 3's three measured differences | **met** | [ran] every claim in the row independently reproduced -- see below |
| Step 3: commit new expectations, update `CLAIMED_PREFIXES` | **met** | [read] `CLAIMED_PREFIXES` is 13 and asserted against the witness union |
| Step 4: implement `>A>`, `>F>`, `>R>` | **met** | [ran] all three byte-exact against the oracle on my own probes, correct gates |
| Step 5: activation indent base, without duplicating or contradicting Task 2's clamp | **met, no code needed** | [ran] verified, see below |
| Step 6: close I31, re-verify bound-before-test / `FOR` / `ITERATE`; settle the four both-DIFF probes | **met** | [ran] |
| Step 7: coverage measure, asserted against a committed literal | **met** | [ran] literal is right and the assertion goes red when it drifts |
| Step 8: suite green, KNOWN GAP row removed with a witness in the tree | **met** | [ran] |

**Step 2, re-derived independently.** [ran] `call zorkolo` with a `::routine zorkolo`
runs at rc 0. Its own variable pool: a caller's `nn = 5` reads as `NN` inside. Builtins
shadow it: `call max 1, 9` with a `::routine max` sets `RESULT` to 9 and the routine
never runs. `TRACE` does not cross into it: a caller's `trace r` echoes none of its
clauses. `trace l` in the caller targeting a `::routine` emits nothing.
`::options trace labels` fires both lines, verbatim
`>I> Routine "ZORKOLO" in package "<absolute path>".`. Every sentence in the row is
true as written.

**Step 5, re-derived independently.** [ran] `activation_indent` already existed at
`e72cc19f` and `git diff e72cc19f 233fbd8b` touches only a comment near it. On the
brief's own two-argument transcript the *only* base divergence was `use arg`'s `>>>`
pair; the callee's `+2` indent, the label echo, `procedure` and the doubled return
value were already right. So "Step 5 needed no code" is accurate, not an omission.
Clamp constraint intact: at nesting depth 25 (`deep25.rex`, 25 nested controlled loops)
our stderr is **byte-identical to the oracle with no normalisation**, with `*-*` clamped
(three successive deeper clauses share one column) and `>>>` running on to column 64.
`MAX_CLAUSE_INDENT` is untouched.

**Step 6, the four both-DIFF probes.** [ran] Located `q11`/`y4`/`z6`/`w29` in
`scratchpad/rr4/` (byte-identical to the copies the implementer used), copied to a fresh
directory and A/B'd. At `e72cc19f` the *entire* divergence of each of the four is the
missing `>>>` pre-/post-increment pair on re-tested passes -- nothing else appears in
any of the four diffs. At `233fbd8b` all four match on all three descriptors byte for
byte, no normalisation. The identification as I31 is correct: one gap, not a third.

**Step 7.** [ran] `WITNESSED_PREFIX_COUNT = 13` is right (4a's ten plus `>A>`/`>F>`/`>R>`),
nothing is printed, and the chain is real: `PREFIX_COVERAGE` set == `support::TRACE_PREFIXES`
(19), its `Witnessed` subset == `CLAIMED_PREFIXES`, and `CLAIMED_PREFIXES` is already tied
by `every_witness_still_emits_every_prefix_it_is_named_for` to bytes present in the
committed `.expected` files, which `check_witness` compares against our own output.
Mutating the literal 13 -> 12 goes red on `witnessed count`. `OWNER_PHASES`'s "same four
strings minus 4b" claim checks out against `coverage.rs:46`.

**Gates, re-run by me** [ran]: `cargo test --workspace` 986 passed / 0 failed;
`REXX_CORPUS_GATE=1 ... --test corpus` 40 of 40, mode STRICT; `cargo fmt --all --check`
exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0.

**Committed expectations are real oracle bytes** [ran]: all five new `.expected` files
regenerate byte-identical from the live oracle. The new corpus program
`raise_array_substitution.rex` is byte-identical to the oracle on all three descriptors,
including the error report's absolute path.

**Negative control, re-run by me** [ran]: all six new witnesses diverge on stderr against
the `e72cc19f` binary and match at `233fbd8b`. Confirmed.

---

## Verdict 2 -- task quality

The implementation is right where I could measure it, and I measured a lot of it: 26
probes of my own across the restructured loop path, the call path and the reference
path, plus the six witnesses, the five old witnesses and a depth-25 clamp probe. One
probe found a pre-existing, unrecorded divergence (F2 below) and one found a hole in
what the harness can see (F1). Neither is a regression.

**The brief is wrong on both contested points and the implementer is right.** [ran]

* `trace i` / `orig = 'PP'` / `call sub >orig` / `use arg >qq` gives
  `>O>   ">" => "ORIG"` (the **name**, upcased) then `>A>   "PP"` (the **value**), then
  `>R>     "ORIG" => "QQ"` (names on **both** sides). The brief has `>O>`/`>A>` swapped
  and states `>R>`'s left operand as a value. The implementer implemented what it
  measured, which is correct.
* "An internal-label call with `trace l` first emits nothing, with or without
  `PROCEDURE`" is false. With `PROCEDURE` the oracle emits `4 *-*   sub:`; without it,
  `9 *-*   sub:`. The implementer's KNOWN GAP row is correct.
* `<orig` does trace the literal `">"`, as reported.

**Scope creep -- the `RAISE ... ARRAY` work.** [ran] Two halves, judged separately.
The `>K>`-after-elements ordering and the doubled `>A>` per element are *Task 9's*:
`>A>` is its deliverable and its position cannot be right without the ordering. The
substitution-hole fix is genuine creep into Task 7's construct, self-disclosed, and it
is **correct**: the oracle prints `maximum expected is .` and never uses `'X'`; at
`e72cc19f` we printed `maximum expected is X.`; at `233fbd8b` the whole run is
byte-identical to the oracle. `RaiseInstruction.cpp:229-237` reads as cited. Acceptable
-- it closed a comment that had labelled its own choice unmeasured, and it is witnessed
live (corpus 39 -> 40, disclosed).

**Both new KNOWN GAP rows are real and correctly described.** [ran]
`trace l`: our stderr is empty where the oracle emits `5 *-* there:` and `9 *-*   sub:`,
stdout and rc matching -- and the row's note that the callee's label sits at the calling
clause's indent plus two is right. Compound control variable: `do aa.1 = 1 to 2 ; nop ;
end ; say aa.1` gives oracle `3`, ours `AA.1`, and the base binary `AA.1` too, so
"not introduced by this task" holds; under `trace i` the oracle's `>C>` before every
control-variable line is indeed missing from ours while `>V>`/`>>>`/`>=>` are present.
Both are correctly scoped as gaps rather than defects Task 9 should have closed.

**Comment accuracy** is high overall -- five stale comments were genuinely corrected,
and the C++ citations I spot-checked (`RexxActivation.cpp:3655`, `UseInstruction.cpp:167`,
`RexxInstruction.cpp:144-162`, `RaiseInstruction.cpp:229-237`) are all accurate. Four
comments are wrong; see F1, F4, F5, F6.

**Dead code:** none. `loop_step` is gone (only comments name it), `stepped` has one
reader and one writer, `do_indent` is used.

**No GC hazard** from the moved increment: `to_text` does not allocate through
`alloc_with`, `previous` is `push_temp`'d before rendering, and the `number` ->
`bind_control` window is the same one the pre-Task-9 code had.

---

## Findings

### Critical

None.

### Important

**F1. The `do_indent`/`loop_indent` split is pinned by nothing, and two comments claim
it is.** [ran]
`rust/crates/rexx-exec/src/run.rs:4994` -- replacing
`let bind_indent = if re_tested { loop_indent } else { do_indent };` with
`let bind_indent = loop_indent;` leaves `cargo test --workspace` at **986 passed /
0 failed** and the corpus gate at **40 of 40**, while the resulting binary diverges from
the oracle byte for byte (`>=>   JJ <= "2"` against the oracle's `>=>     JJ <= "2"`).
The same holds for `>F>`: `rust/crates/rexx-exec/src/eval.rs:301` changed to
`trace_function(indent + 2, ...)` produces **zero** test failures and 40 of 40.
Cause: DEVIATION 0's `normalize_stderr` collapses exactly the space run these lines
differ in, so no `.expected` comparison and no corpus comparison can see a trace line's
indent. That is a permanent chosen deviation, not a defect -- but two comments state the
opposite and are false:
* `rust/crates/rexx-exec/tests/trace_oracle.rs:83-87`: "it also pins a `DO OVER`'s own
  single `>=>` -- the neighbouring *passing* case, which is what separates 'traces the
  control variable' from 'traces it at the right indent'". It does not; that is the one
  distinction the comparison erases.
* `rust/crates/rexx-exec/tests/trace_oracle.rs:253-256`: "two expression-form calls in
  one clause, so the second one's own line has to come back to the *caller's* indent
  after the first callee moved it." Nothing makes it have to.

The implementer knew the mechanism -- `tests/trace_oracle/exit_value.rex`'s own header
says "It does not pin the indent: DEVIATION 0 normalises the space run this file's own
comparison sees" -- which makes these two the errors rather than the exception. The
behaviour is correct against the oracle (I checked at depth 25 and on the witnesses with
raw `cmp`); what is wrong is the claim that a test holds it there. Correction, not code,
is what this needs; the honest statement is that trace-line indent is verified only by
raw A/B probes and by the `lib.rs` unit tests that compare unnormalised strings.

**F2. A pre-existing stdout divergence in the exact arm Task 9 restructured, unrecorded,
and Task 9's new `>V>` line now asserts the wrong value out loud.** [ran]
`rust/crates/rexx-exec/src/run.rs:4931-4999` (`LoopState::Controlled`).
`trace i` / `do ii = 1 to 3 ; ii = 10 ; end ; say ii`: oracle stdout `11`, ours `4`,
`e72cc19f` also `4` -- so pre-existing, not a regression. The oracle's `checkControl`
**re-reads the control variable** (`control->evaluate`, which is why it traces `>V>`);
ours renders `LoopState::Controlled::current`, a value the body cannot reach. Before
Task 9 the divergence was two absent lines plus wrong stdout; now we emit
`>V>     II => "1"` where the oracle emits `>V>     II => "10"`, i.e. a line that
positively states a value the oracle contradicts.
The comment at `run.rs:4943` correctly describes the oracle's `control->evaluate` but
does not say our line does not read the variable, which is exactly the point a reader
of that arm needs. This shape is not in the exclusions file, not in any code comment,
and not in the report -- it is a gap dropping silently, in the code this task owns, and
it is the closest adjacent shape to the ones Step 6 asked to be re-verified. It needs a
KNOWN GAP row (stdout-affecting, like the compound-control-variable one it sits beside)
and one sentence at `run.rs:4943`.
A second, milder instance of the same "we model the header, the oracle models the
clause" split, also pre-existing and also unrecorded:
`do ii = 9E999999999 by 9E999999999 to 9E999999999` overflows in the increment and the
oracle attributes it to `line 4` (the `END`, which it echoes) where we attribute it to
`line 2` (the `DO`); `e72cc19f` also says line 2.

**F3. The plan still carries both statements this task falsified.** [read] + [ran]
`docs/superpowers/plans/2026-08-03-phase-4b-procedures-and-conditions.md:208` and
`:979` both still say `>O>   ">" => "PP"` / `>A>   "orig"` / `>R>     "PP" => "Q"`, and
`:979` still says "an internal-label call with `trace l` first emits nothing, with or
without `PROCEDURE`". I measured both wrong (above). The `trace l` half is corrected in
`phase-4-exclusions.txt`; the `>O>`/`>A>`/`>R>` half is corrected **only in
`task-9-report.md`**, which is the one place `rust/CLAUDE.md` says a correction is lost.
Any regenerated Task 9 brief -- a fix round, say -- hands the next worker the same two
false transcripts, and the prefixes are the task's deliverable. Mitigating: the rule was
written into `rust/CLAUDE.md` at `d24dde78`, *after* this commit, so it was not binding
at the time. Fix the plan.

### Minor

**F4.** `rust/crates/rexx-exec/tests/trace_oracle.rs:72` -- "the union across all five
must be exactly the ten prefixes claimed". There are now **ten** witnesses and
**thirteen** claimed prefixes; both numbers in that sentence are stale. [read]

**F5.** `rust/crates/rexx-exec/src/trace.rs:522-527` (`trace_alias` doc) -- "and under
`trace l` shows nothing at all" is false: the same program under `trace l` emits
`6 *-*   sub:` on the oracle. What is true is that `>R>` does not appear. The same
sentence appears in `task-9-report.md` section "`>R>`". [ran] Note this is the very
divergence the implementer recorded as a KNOWN GAP two sections later, so the comment
contradicts the task's own finding.

**F6.** `rust/crates/rexx-exec/src/run.rs:3446-3449` (`eval_argument` doc) -- "the value
is computed here for both variants, by evaluating the inner expression through the
ordinary path" is contradicted by the Task 9 paragraph twelve lines below it ("by
evaluating the reference node itself, **not** its inner variable") and by the code at
`:3483`. One of the two must go; the older sentence is the wrong one. [read]

**F7.** The commit message calls the `RAISE`-substitution-hole fix "a stdout content
difference". [ran] The untrapped 40.4 report is written to **stderr**; the report body
and the corpus program's own header use the hedged "stdout/stderr", which is right.
Cosmetic, but it is the sentence a later reader will quote.

**F8.** The `TRACE L` KNOWN GAP row is filed with "Owner unassigned" while
`mode_from_setting` -- the site it names -- is inside `trace.rs`, one of the four files
this task's brief listed. The coupling argument given (D17's three-field `TraceMode`,
`TRACE()`'s reported setting, 4c's `>I>`/`<I<` row) is sound and there is precedent for
an owner-unassigned row (`TRACE ?`), so this is a judgement call rather than an error --
but it is the one row here that could have been closed inside the task's own file list.

**F9.** `bind_control` (`run.rs:5150`) guards `trace_assignment` with
`if self.tracing_intermediates()`, which `trace_assignment` already does internally.
Harmless (it avoids two allocations) and not a behaviour difference -- noted only
because a duplicated gate is where a future divergence between the two hides.

### Cannot verify from the diff

* The report's 20-probe A/B table (18 DIFF -> MATCH, 2 MATCH on both, 1 DIFF on both)
  cannot be reproduced from the diff; the probe files were not committed. I ran 26
  probes of my own instead, over the same shape families, and reached the same
  conclusion in direction: no MATCH -> DIFF that I could find, and the one DIFF-on-both
  the report names (a `SIGNAL ON SYNTAX` handler reached from inside a callee printing
  two columns in) reproduces exactly as described and is indeed indent-only and
  pre-existing, so DEVIATION 0 does cover it and the decision not to file a row is
  defensible.
* Of the thirteen claimed mutations I re-ran four -- M2 (`>R>` gate), M4b (`EXIT`'s
  `trace_result`), M9 (`13 -> 12`), M13 (omitted element closes up). All four went red,
  each naming the test the table names. The other nine were not re-run.
* Whether the `>A>` twice-per-element shape holds for array element counts other than
  three, and whether `>A>` fires at an `ExpressionList` site, are untested here; the
  report flags the latter as Phase 5's and it fails loudly today.

---

## Summary

Spec compliance: **all eight steps met**, including Step 5, which correctly needed no
code, and Step 7, whose number is asserted rather than printed and whose literal is
right. The three absent-value-line gaps are all closed and all three closures are
byte-verified against a live oracle by this review, not taken from the report.

Task quality: **good**. The implementation is correct everywhere I could measure it, the
two points where the implementer contradicted the brief are both points where the brief
is wrong, the negative control and the mutation sample both hold up, and the two new
KNOWN GAP rows are real and honestly scoped. The two things worth acting on are F1 (two
comments claim a witness pins an indent that DEVIATION 0 makes invisible -- correct the
comments, the code is right) and F2 (a pre-existing stdout divergence in the restructured
arm that nothing records, and which Task 9's new `>V>` line now states out loud). F3
(fix the plan) is cheap and prevents the next Task 9 brief from carrying the same two
false transcripts.
