# Task 4 review: `ExprKind::Call`, the internal-function expression form

Reviewed diff `fd9a97ac..c3db2bcc` (commit `c3db2bcc`) against
`task-4-brief.md` and `task-4-report.md`.

## Verdicts

1. **Spec compliance: PASS.** Every requirement the brief states is met, and
   each of the five expansions beyond its Files list is load-bearing rather
   than opportunistic. Details per requirement below.
2. **Task quality: CHANGES REQUESTED.** One Critical: this task makes a
   composition reachable (`say f(1) + g(2)`, `say f(g(1))`, `call sub f(1)`)
   that prints two spaces too much indent for every activation after the
   first *and* for the enclosing expression's own `>>>` -- measured against
   the oracle, and not confined to `TRACE`: a plain program with no trace at
   all reports its error-echo clause at the wrong indent. The corpus cannot
   see it because DEVIATION 0 normalises exactly those spaces, which is also
   why the new corpus witness's own stated reason for existing is false.

Counts: **1 Critical, 2 Important, 5 Minor.**

## Measured results for each claim the brief asked me to verify

Every oracle run below was made from a fresh empty subdirectory I created,
wrapped as `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx probe.rex )`,
with stdout, stderr and exit status read as three separate descriptors. Our
side is `rust/target/debug/rexx-run`, built from the committed tree.

| Claim | Measured |
| --- | --- |
| Value-less internal routine in expression form | Oracle `say f(1)` / `exit` / `f: return` in a fresh dir: **rc 212**, empty stdout, `Error 44 ... Function or message did not return data.` + `Error 44.1: No data returned from function "F".` Ours: byte-identical, rc 212. The report's `44.1` is **confirmed, not the contamination**. |
| Literal form | Oracle `say "f"(1)` with `f:` present: **rc 213**, `Error 43 ... Routine not found.` / `Error 43.1: Could not find routine "f".` Ours: loud `4c` fallback, rc 120, `rexx-exec: routine "f" is not implemented (4c)`, stdout empty -- the label did **not** run, so the literal was not wired into the label table. Correct for this phase. |
| `ExprKind::Call` moved fully in scope | Loud witness row deleted (not split); `EXPECTED_OUT_OF_SCOPE` row removed; `EXPR_TAGS` 9/6 -> 10/5; `loud.rs`'s `expected_exprs.len()` 6 -> 5 and `in_scope_counts` 9 -> 10; `lib.rs`'s `expr_owner` -> `None`. Verified non-vacuous: deleting `lang/call_expression.rex` from `phase-4b.txt` makes `every_in_scope_variant_is_witnessed_by_the_phase_subsets` fail with `ExprKind: 1 in-scope variant(s) unwitnessed ...: Call`. |
| Corpus 34 of 34, and the new slot genuinely compared | `34 of 34 matching` on the committed tree. Genuinely compared: appending `say length('abc')` to `call_expression.rex` gives `33 of 34` naming `lang/call_expression.rex: stdout, stderr, exit code differ`, and under `REXX_CORPUS_GATE=1` the gate fails. So the slot is exercised -- **but see I1 for what that comparison cannot see.** |
| What is left for 4c at the fallback | `eval_call`'s doc states the order (internal 4b / builtin 4c / external Phase 7) and that the fallback names `4c` -- a 4c implementer reading `eval.rs` *will* find the split. But the placement instruction is wrong; see I2. |
| No pre-existing expected-byte literal changed | Confirmed. The diff adds two files (`corpus/lang/call_expression.rex`, `rexx-parse/tests/sourceline_oracle/call_expression.txt`) and changes no existing expectation. The new `.txt`'s body is byte-identical to the `.rex` and its `count 28` matches. All 33 previously-passing programs still match (34 of 34, none previously passing lost). |
| Suite / lints | `cargo test --workspace`: **899 passed, 0 failed, 4 ignored** (report's figure confirmed by summing every `test result:` line). Assertions `4224 of 4259`. `cargo fmt --all --check`: exit 0. `cargo clippy --workspace --all-targets -- -D warnings`: exit 0. |

Additional oracle probes I ran that the report did not (all byte-identical to
the oracle on the committed tree, stdout + stderr + rc):

* `EXIT` inside a function reached from inside an `INTERPRET` fragment -> rc 5, silent, the clause after the `INTERPRET` never runs.
* `EXIT` inside a function called by a function (`g: return h(2)` / `h: exit 7`) -> rc 7.
* `EXIT` inside a function used as a `DO` control bound (`do i = 1 to f(1)`) -> rc 3.
* `EXIT` inside a function used as an `IF` condition -> rc 4.
* `EXIT` inside a function passed as a `CALL` instruction's *argument* (`call sub f(1)`, `f: exit 3`) -> rc 3, `RESULT` untouched.
* A routine falling off its own end in expression form (`f: nop` at end of file) -> rc 0, silent; and falling through into a following label (`f: nop` / `g: return 3`) -> prints `3`, rc 0.
* A raise inside a routine reached by the expression form -> the two-level echo chain and 42.3 report match byte for byte.
* Recursion through the expression form (`f: return f(1)`, and the same guarded by an `IF`) -> rc 245 (11.1), no native abort. `MAX_ACTIVATION_DEPTH = 10_000` still holds despite the expression path costing three more Rust frames per activation than `CALL`'s.

## Ruling: `Failure::Exited`

**Sound, and the right shape. Keep it.**

* It does not duplicate `Flow::Exit`; it is the same event on the one channel
  that has room for it. `eval` returns `Result<ObjRef, Failure>` with no
  `Flow`, so an `EXIT` reached through an expression has nowhere else to go.
  Duplicating it as a sentinel `ObjRef` or a side-channel field on `Interp`
  would be strictly worse.
* **They cannot disagree.** Both carry `Option<ObjRef>` and `execute`
  (`rust/crates/rexx-exec/src/lib.rs:1479`) collapses them into one arm,
  `Ok(value) | Err(Failure::Exited(value)) => interp.exit_code_for(value)`,
  so there is no second exit-code rule to drift from.
* **Every consumer of `Flow::Exit` was checked against every producer of
  `Failure::Exited`.** The only paths that catch a `Failure` rather than
  `?`-ing it are `step_in_temps_frame` (`record_failure_site`),
  `run_fragment` (`seal_site_level`), `resolve_and_run_call`
  (`seal_site_level`), `run_repeating`'s `WHILE`/`UNTIL` and `Select`'s
  `When`/`WhenCase` (`record_failure_at`/`record_failure_site`). All five
  record a site and re-throw; none reports, none swallows. `failure_sites` is
  read only under `Err(Failure::Raised(..))`, so the recorded sites are
  genuinely inert -- the doc's "sealing a site nothing ever prints is
  harmless" is true today.
* The two hard cases the brief named were measured, not reasoned: `EXIT`
  inside a function inside an `INTERPRET` fragment, and `EXIT` inside a
  routine called by another routine. Both match the oracle exactly (rc 5,
  rc 7, both silent). I added three more (`DO` bound, `IF` condition, `CALL`
  argument) and all match.
* No `From<..> for Failure` can produce it, no test's `let Failure::Raised(..)
  else { panic }` can be reached by it, and `Loud`'s owner machinery never
  sees it.

One gap, Minor: the variant has **two** producers -- `EXIT`, and a routine
running off the end of the program -- and only the first has a unit test.
See M4.

## Ruling: the `exec_call` split

**Correct, necessary, and behaviour-preserving where it matters.** Extracting
~80 lines of resolution / argument evaluation / depth guard / three-piece
indent bookkeeping beats a hand-copied second version, and the shared
function is exactly where 4c's builtin lookup should go (contra the doc; see
I2).

I diffed the old `exec_call` against the new pair line by line. There is
exactly **one** semantic difference, and it is an improvement the report does
not mention: `base_indent` used for the caller-side `RESULT` `>>>` is now
captured *before* the arguments are evaluated (`run.rs:1800`), where the old
code captured it after (old `run.rs:1729`). That matters only because Task 4
makes an argument able to contain a call, and capturing after would read the
callee's polluted indent. Measured: `trace r` / `call sub f(1)` / `sub:
return 5` prints the caller's `>>>   "5"` at the correct indent because of
this. Good call -- but the *other* reader of the same field was not fixed,
which is C1.

---

# Findings

## Critical

### C1 -- `current_value_indent` is not restored across a nested activation, and Task 4 is what makes that observable

`rust/crates/rexx-exec/src/run.rs:1748` and `:1762-1764`.

`resolve_and_run_call` saves and restores three pieces of level state
(`activation_indent`, `indent_offset`, `clause_line_override`) but not
`Interp::current_value_indent`, which `run_activation` -> `step_in_temps_frame`
(`run.rs:1942`) overwrites on every clause the callee runs. Before this task
that was unobservable: at most one activation could be entered per clause,
and the next clause's own `step_in_temps_frame` re-set the field. Task 4
makes two or more activations in a *single* clause reachable, and every one
after the first computes its base from the previous callee's last clause.

Measured, oracle vs ours on the committed tree (four shapes, fresh dirs):

```
trace r ; say f(1) + g(2)        oracle: g's clauses at 2, inner >>> at 4, outer >>> at 2
                                 ours:   g's clauses at 4, inner >>> at 6, outer >>> at 6
trace r ; say f(g(1))            oracle: f's clauses at 2, >>> at 4 / 2
                                 ours:   f's clauses at 4, >>> at 6 / 6
trace r ; call sub f(1)          oracle: sub's clauses at 2
                                 ours:   sub's clauses at 4
```

**Not a trace-only defect.** With no `TRACE` anywhere in the program:

```
say f(1) + g(2)
exit
f: return 1
g: say 1/0
return 2
```

oracle stderr `     4 *-*   say 1/0`, ours `     4 *-*     say 1/0`. That is
an ordinary error report, the thing a user sees.

**Fix (verified):** one line, beside the other three restores at
`run.rs:1764`:

```rust
self.clause_line_override = saved_line;
self.current_value_indent = base_indent;
```

With it, all four shapes above and the error-report shape are **byte-identical
to the oracle** (`diff` against captured oracle stderr: no output). The whole
workspace suite stays green with the fix applied -- 899 passed / 0 failed / 4
ignored, corpus 34 of 34, `cargo fmt --all --check` and
`cargo clippy --workspace --all-targets -- -D warnings` both exit 0 -- which
is also the proof that *nothing today covers this*: no existing test
distinguishes the two versions.

A regression test is needed and cannot live in the corpus (see I1). Put it in
`run.rs`'s own unit tests, beside
`a_callees_clauses_echo_at_the_calling_clauses_indent_plus_two`, asserting
the exact stderr of `trace r` / `say f(1) + g(2)`; those tests compare bytes
and are not reachable by `normalize_stderr`.

## Important

### I1 -- the new corpus witness's stated reason for existing is false, twice over

`rust/corpus/lang/call_expression.rex:11-14`, repeated verbatim in
`rust/corpus/phase-4b.txt:91-95` and in the report.

The header says:

> a version that skipped the indent bookkeeping for the expression form would
> still pass every eval.rs test (none of them inspect stderr this closely)
> and would only diverge here.

Both halves of "would only diverge here" are wrong, and I measured each:

1. **It does not diverge here.** `corpus.rs`'s DEVIATION 0 runs both sides
   through `support::normalize_stderr`, which collapses the run of spaces
   between a trace line's 3-byte marker and its content -- which is *exactly*
   what the D2r indent rule produces. Changing `base_indent + 2` to
   `base_indent` in `resolve_and_run_call` (i.e. deleting the D2r rule
   outright) leaves the corpus reporting **34 of 34 matching**, gate mode
   included. Independently: mutating this witness into the C1 shape (`say
   f(1) + g(2)`) also reports 34 of 34 despite the demonstrated divergence.
2. **It is not the only thing that would catch it.** The same mutation fails
   six `run.rs` unit tests
   (`a_callees_clauses_echo_at_the_calling_clauses_indent_plus_two`,
   `a_call_inside_a_fragment_echoes_each_activations_own_line`,
   `a_returned_value_traces_in_the_callee_and_again_in_the_caller`,
   `a_callees_trace_setting_does_not_survive_its_return`,
   `an_interpret_inside_a_callee_runs_at_the_callees_own_level`,
   `the_report_echoes_one_clause_per_activation_innermost_first`). And since
   `eval_call` and `exec_call` share `resolve_and_run_call`, an
   "expression-form-only" version of the bug is not even constructible.

This is the shape the project keeps finding in its own instruments: a witness
whose justification names a property the harness provably cannot observe. The
witness is still worth keeping -- it does pin stdout byte-exact (`42`, `in f`,
`result: before`, which genuinely discriminates a wrongly-settled `RESULT`;
the `LENGTH` mutation above proves stdout differences are caught) and the
clause *sequence* and *line numbers* on stderr, which normalisation does not
touch.

**Fix:** replace the indent paragraph in both files with what the witness
actually pins -- stdout byte-exact including `result: before`, and the stderr
clause sequence and line numbers -- and state explicitly that the *indent* is
pinned by `run.rs`'s unit tests, not here, because DEVIATION 0 normalises it.
The same false claim sits in Task 3's `call_return.rex` entry
(`phase-4b.txt:80-88`, "Its whole value is the `trace r` transcript on
stderr ... pins at 6 where a '2 x depth' rule would say 2"); it is
pre-existing, but this task copied it forward instead of checking it, and
both should be corrected together.

### I2 -- the 4c hand-off points at the wrong function, and following it literally would close only half the gap

`rust/crates/rexx-exec/src/eval.rs:408-416`, and the corresponding paragraph
in `task-4-report.md`.

The doc reads:

> A name that reaches neither **this function's own label search** nor (once
> it exists) 4c's builtin table fails loudly naming `4c` ... and the builtin
> lookup belongs *between* the label search and that call, not after it.

and the report elaborates: "inside `eval_call` itself, resolving a builtin
name before ever reaching the loud path."

`eval_call` has no label search. The label search is in
`resolve_and_run_call` (`run.rs:1663-1670`), one function away and shared
with `exec_call`. A 4c implementer who does what this says has two bad
options: duplicate the label search into `eval_call` so they have somewhere
to sit "between" (the exact duplication this task's own refactor exists to
prevent), or wire builtins into the expression form only. The second is the
likely one, and it is measurably incomplete -- `call length 'abc'` is a
builtin call through the *instruction* form, and on the committed tree it is
still `rexx-exec: routine "LENGTH" is not implemented (4c)` while the oracle
prints `3`.

**Fix:** say that the builtin lookup goes in `resolve_and_run_call`, between
the `activation_body.labels.get(name)` miss and `Loud::unresolved_call`, so
both call forms close together; and keep the ordering statement (internal /
builtin / external) where it is. If `eval_call` is genuinely meant to own an
expression-only step, say which step and why `CALL` does not need it.

## Minor

### M1 -- stale cross-reference to "the five judgement calls"

`rust/crates/rexx-exec/tests/loud.rs:49`: "in particular the five `ExprKind`
assignments that are a Task 16 gate-time judgement call". After this task
four judgement-call assignments remain (`QualifiedCall`, `ClassResolver`,
`List`, `VariableReference`) -- `Call` closed. Worse, `coverage.rs`'s newly
written text uses "five" for a *different* set ("The five that remain
(`QualifiedCall`, `ClassResolver`, `List`, `VariableReference`, `Message`)"),
which includes `Message`, the one assignment that is a spec citation and not
a judgement call. A reader following the pointer lands on a set of five that
contradicts the sentence that sent them. **Fix:** `loud.rs:49` -> "the four
remaining `ExprKind` assignments that are a Task 16 gate-time judgement
call".

### M2 -- the exclusions section heading now contradicts one of its own rows

`docs/superpowers/plans/phase-4-exclusions.txt:217`: "EXPRKIND OWNERSHIP --
the expression forms 4a/4b do not evaluate". The section still pins
`VariableReference 4b`, i.e. a form 4b *does* evaluate -- that is what the
owner string means. The pre-change heading ("the six expression forms 4a does
not evaluate") was accurate. **Fix:** "EXPRKIND OWNERSHIP -- the expression
forms 4a does not evaluate, and who owns each".

### M3 -- garbled sentence in the same section

`docs/superpowers/plans/phase-4-exclusions.txt:288-292`: "`Call`'s own row is
kept rather than deleted outright, the same "CLOSED DEFECTS" convention below
applies to a row whose own history is worth a reader tripping over rather
than losing to a quiet deletion -- removing it needs the same amendment
closing a KNOWN GAP does." Two independent clauses spliced by a comma, and
the middle clause has no readable subject. The referenced section does exist
(line 737), so only the wording is at fault. **Fix:** split into two
sentences -- "`Call`'s own row is kept rather than deleted outright. The
"CLOSED DEFECTS" convention below applies: a row whose history is worth
tripping over is not lost to a quiet deletion, and removing it needs the same
amendment closing a KNOWN GAP does."

### M4 -- `Failure::Exited`'s second producer has no test

`rust/crates/rexx-exec/src/eval.rs:454` produces `Failure::Exited` for
`Ended::Exited`, which `run_activation` returns for **two** distinct events:
an `EXIT` instruction, and the callee running off the end of the program
(`run.rs:650`). `error.rs`'s `no_data_returned` doc leans on the distinction
("**Not** the same path as running off the end of the routine with no
`RETURN` at all"), and the report measured it on the oracle (rc 0, silent),
but no test covers it. I verified it matches (`say f(1)` / `exit` / `f: nop`
-> rc 0, empty stdout, empty stderr both sides; and `f: nop` followed by `g:
return 3` prints `3` on both). **Fix:** one more test beside
`an_exit_inside_a_routine_reached_by_expression_call_ends_the_whole_program`.

### M5 -- the module doc lists a form that never evaluates

`rust/crates/rexx-exec/src/eval.rs:22-23`: "(Task 4, 4b) `ExprKind::Call`,
the internal-function form (`f(...)`/`"f"(...)`)". `"f"(...)` never
evaluates in this phase -- it is unconditionally the loud `4c` fallback. The
forward pointer to `eval_call`'s doc softens it, but the list reads as
"implemented". **Fix:** "the internal-function form (`f(...)`; the
`"f"(...)` literal form stays loud, see `eval_call`)".

## Requirement-by-requirement spec check

| Brief requirement | Verdict |
| --- | --- |
| New `ExprKind::Call` arm in `eval.rs` | Met (`eval.rs:399`, `eval_call` at `:435`) |
| `tests/owners.rs` modified | Met |
| Variant moved *fully* in scope, not split | Met -- `Owner::InScope`, `expr_owner` -> `None` |
| Loud witness **deleted**, not split | Met (`loud.rs`'s `Call`/`say foo(1)` row removed) |
| `EXPECTED_OUT_OF_SCOPE` updated | Met |
| Pinned counts updated | Met -- `EXPR_TAGS` 10/5, `loud.rs` 10 and 5, plus the two comment blocks that carry copies |
| Witness program in a corpus subset | Met, and verified non-vacuous by removing it (`Call` reported unwitnessed) |
| Literal form not wired into the label table | Met -- `search_labels = false`, oracle 43.1/213 re-measured, our answer is the loud `4c` fallback and stdout stays empty |
| Step 3's error measured, not guessed | Met -- 44.1 rc 212, `Raised::no_data_returned` built from the generated catalogue |
| I25's split in `eval.rs`'s arm as a comment | Met in letter; placement guidance wrong (I2) |
| Step 1 test | Met (`an_internal_function_returns_its_value_into_an_expression`, adapted to `run_program`) |
| Step 4 test naming `4c` | Met, and non-vacuous: before this task the same program named `4b` |
| Must not touch `RESULT` | Met -- measured on the oracle and pinned by a unit test and the corpus witness's stdout |
| Must not emit `>F>`/`>A>` | Met -- `trace_intermediate`'s match has no `Call` arm and falls to `_ => {}` |
| Suite / fmt / clippy / commit | Met |
| No `unsafe`, C++ tree untouched | Met |

Expansions beyond the Files list, each judged: `run.rs`'s split (necessary,
behaviour-preserving, one deliberate improvement -- see the ruling above);
`error.rs`/`lib.rs`'s `Failure::Exited` (necessary and correct -- see the
ruling above); `tests/loud.rs`, `tests/coverage.rs`, `tests/owners.rs`
(required by the in-scope move, all three consistent); `phase-4-exclusions.txt`
(required by the plan-amendment rule; M2/M3 are wording); the corpus witness
and its `sourceline_oracle` expectation (required by
`every_in_scope_variant_is_witnessed_by_the_phase_subsets`; I1 is its
justification, not its existence). Nothing in the diff is outside what the
task needed.

## Tree state

Every experiment above was reverted. `git status --porcelain` is empty; HEAD
is `c3db2bcc`.
