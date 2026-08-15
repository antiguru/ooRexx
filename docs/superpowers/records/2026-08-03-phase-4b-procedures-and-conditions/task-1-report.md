# Task 1 report: a real `INTERPRET`, replacing the Task 3 spike

Status: **DONE_WITH_CONCERNS**. Everything the brief asked for is implemented and
verified; three divergences from the oracle are disclosed below, two of which this
task made *reachable* rather than created, and one scope judgement is flagged for
the coordinator.

Baseline confirmed before any edit: `cargo test -p rexx-exec` green at
`38fed3a1`, corpus `29 of 29 matching`.

Commit: **`a462e3e9`** (`a462e3e973d7dc9232398aee7d00660088974c5b`), one commit,
15 files, on `plan/rust-rewrite`. This report is not in it: `.superpowers/` is
gitignored.

---

## 1. What changed and why

### The keyword itself

`run_fragment` already did everything Step 6 describes -- parse the text as a
`Fragment`, plan it against the enclosing frame, run its instruction list through
a bounded sub-loop, forward an unowned `Flow` outward. The brief's "a private
fragment-running helper may already do most of what Step 6 describes" is exactly
right, and the whole of the keyword's implementation was deleting the
`if self.interpret_spike` guard on the `InstructionKind::Interpret` arm
(`run.rs`). No new fragment-running code was written.

Deleted with it: the `interpret_spike` field, `Interp::new`'s parameter,
`execute`'s parameter, `run_program_interpret_spike`, and `Loud::interpret_leave`
(replaced -- see §2). `Interp::new` losing its argument touched 205 call sites
across `run.rs`, `eval.rs`, `value.rs`, `stem.rs`, `plan.rs`, `trace.rs` and
`lib.rs`; that edit is purely mechanical and the diff for the five files this
task otherwise does not touch contains nothing but `Interp::new()` lines
(verified with `git diff --unified=0 | grep -v Interp::new`, empty).

### Ownership and the harnesses

* `lib.rs`'s `instruction_owner`: `Interpret` -> `None` ("implemented in this
  crate"), per the brief's note that `None` is the value to use, not a phase
  string.
* `tests/owners.rs`: `Interpret` -> `Owner::InScope`; its `EXPECTED_OUT_OF_SCOPE`
  row deleted; `InstructionKind` counts 20/9 -> 21/8.
* `tests/loud.rs`: the `Interpret` witness row deleted (pinned item 4 -- a
  witness for a variant that moved in scope must be deleted, not left stale);
  `expected_instructions.len()` 24 -> 23; `in_scope_counts_...` 20 -> 21.
* `rust/corpus/phase-4b.txt` created, listing `lang/interpret_dynamic.rex`, with
  a header in `phase-4a.txt`'s shape stating what the subset admits and what it
  still excludes. The exclusion list points at `tests/owners.rs`'s tag tables as
  the authoritative, machine-checked version rather than duplicating a prose list
  that can rot.
* `tests/coverage.rs`: `every_in_scope_variant_is_witnessed_by_the_phase_4a_subset`
  renamed to `..._by_the_phase_subsets` and now reads the **union** of
  `phase-4a.txt` and `phase-4b.txt`. `tests/collect_stress.rs`'s call site reads
  the union too. The `EXPECTED_SUBSET` pin (`coverage.rs:516`) stays
  `phase-4a.txt` only, as instructed.
* `rust/corpus/README.md` gained a "Phase 4b subset" section, and its "Phase 4a
  subset" section lost the clause "while Phase 4a is still the only thing built",
  which this task made false.

### Tests

* The three fragment-lifetime tests moved from `tests/spike.rs` into `lib.rs`'s
  `#[cfg(test)] mod tests`, doc comments verbatim. They call `run_program` --
  the trade `run_program_interpret_spike`'s doc asked 4b to re-make now settles
  the other way, because the argument that favoured the integration test ("a unit
  test with privileged access to private internals proves less about the shape
  callers actually get") no longer applies: these tests need no privileged access
  at all now that `INTERPRET` is reachable through the front door.
* Step 4's new test, `interpret_binds_a_name_the_enclosing_body_never_mentions`,
  added in the same module. Run before the implementation, it failed with
  `NOT_IMPLEMENTED_EXIT`; after, it passes.
* `tests/spike.rs`: `the_interpret_keyword_still_fails_loudly` **deleted** rather
  than re-pointed. Its subject is the property "4a does not ship `INTERPRET`",
  which this task deliberately makes false; keeping the name over a different
  witness would have made the test lie about what it checks.
  `the_loud_failure_code_cannot_be_confused_with_a_rexx_error` moved off
  `call "sub"` (Task 3 would have taken it) onto a message send, `q~append(1)`,
  with the exact expected stderr taken from a run:
  `rexx-exec: a message send is not implemented (Phase 5)`. The comment there
  records why a message send and not `PARSE`/`ADDRESS`: those are 4c's and would
  break again in weeks.
* `run.rs`'s two F-EX2 tests replaced -- see §2, the measurement invalidated one
  of them.
* `rexx-core` gained `RootSet::temps_len`, and `step_in_temps_frame` gained the
  debug tripwire I22 scheduled in 4a and left unbuilt. `pop_frame` is unchanged,
  as I22 requires.

---

## 2. The two escape questions, measured

Both measured on the oracle before anything was implemented, wrapped as
`( ulimit -v 1048576; LD_LIBRARY_PATH=…/build/lib …/build/bin/rexx FILE )`, with
stdout, stderr and rc read as three separate descriptors.

### Question 1: `LEAVE` naming a loop that encloses the `INTERPRET`

**Answer: the search never crosses the boundary.** An enclosing loop is
invisible from inside fragment text, and an unmatched `LEAVE`/`ITERATE` is the
exhausted search at the fragment's own edge -- 28.1/28.2/28.3/28.4, rc 228.

The bare `LEAVE` row is the one that decides it, and it is the opposite of what
"the fragment runs inside the enclosing activation" predicts. Before this
measurement the code forwarded a bare `Flow::Leave` outward and the enclosing
`DO` consumed it, which was wrong.

Program (`p1`):

```rexx
do label outer while 1
  say 'in-loop'
  interpret "leave outer"
  say 'unreached-inside'
end
say 'after-loop'
```

Oracle -- stdout `in-loop\n`, rc 228, stderr:

```text
     3 *-*   leave outer
     3 *-*   interpret "leave outer"
Error 28 running /…/p1_leave_named_label.rex line 3:  Invalid LEAVE or ITERATE.
Error 28.3:  Symbol following LEAVE ("OUTER") must either match the label of a current loop or block instruction.
```

Program (`p3`), the bare case:

```rexx
do kk = 1 to 3
  say 'pass' kk
  interpret "leave"
  say 'unreached-inside'
end
say 'after-loop'
```

Oracle -- stdout `pass 1\n`, rc 228, stderr:

```text
     3 *-*   leave
     3 *-*   interpret "leave"
Error 28 running /…/p3_leave_bare.rex line 3:  Invalid LEAVE or ITERATE.
Error 28.1:  LEAVE is valid only within a repetitive loop or labeled block instruction.
```

The remaining four probes, same shape, same rc 228:

| program | fragment text | enclosing construct | oracle |
|---|---|---|---|
| `p2` | `leave idx` | `do idx = 1 to 3` | 28.3, `("IDX")` |
| `p4` | `iterate outer` | `do label outer idx = 1 to 3` | 28.4, `("OUTER")` |
| `p9` | `iterate` | `do kk = 1 to 3` | 28.2 |
| `p10` | `leave` | nothing (top level) | 28.1 |
| `p11` | `leave choose` | `select label choose` | 28.3, `("CHOOSE")` |

`p11` is what shows the rule is about the `INTERPRET` boundary and not about
loops specifically: a labelled `SELECT` is equally invisible.

The control, `p5`, confirms this is a statement about *crossing* and not a
blanket refusal:

```rexx
interpret "do jj = 1 to 5; say 'frag' jj; if jj = 2 then leave; end"
say 'after-interpret'
```

Oracle -- `frag 1\nfrag 2\nafter-interpret\n`, empty stderr, rc 0. A loop written
inside the fragment consumes its own `LEAVE` normally.

**Implemented.** `run_fragment` now raises 28.1/28.2/28.3/28.4 at its own
boundary, in the same four-arm shape `run_activation` already uses for the
program's boundary, resolving the name against `fragment.symbols` -- which is
also the correct home for F-EX2's finding (the `SymbolId` is interned in the
fragment's own fresh table and is meaningless above this function). F-EX2's loud
refusal is gone, replaced by the condition the oracle actually raises, at the
same call site and for the same reason.

Differential after implementing, all eight probes: **stdout MATCH, rc MATCH,
stderr differs by exactly one line** -- the innermost clause echo, see Concern 1.

Two existing `run.rs` tests were replaced because this measurement invalidated
them: `a_fragments_named_leave_refuses_to_cross_the_boundary_rather_than_forward_an_id`
(the refusal is now a raise) and `a_fragments_bare_leave_still_forwards_across_the_boundary`
(whose name states the falsified premise; the assertion `Failure::Raised(_)`
would still have passed, which is why the *name and doc* mattered more than the
assertion). Three tests stand in their place, all with mutation-kill notes:
`a_fragments_leave_or_iterate_never_reaches_the_enclosing_loop` (all four
families, table-driven), `a_fragments_named_leave_is_resolved_against_the_fragments_own_table`
(keeps F-EX2's deliberate `FOO`/`BAR` id-collision design), and
`a_leave_inside_the_fragments_own_loop_is_consumed_there`.

### Question 2: `RETURN` inside a fragment

**Answer: it crosses the boundary outward, exactly like `EXIT`.** In a called
routine it returns *from the routine*; in the main body it ends the program with
the value as the exit code.

`p6`, main body:

```rexx
say 'before'
interpret "return 7"
say 'after'
```

Oracle -- stdout `before\n`, empty stderr, **rc 7**. `after` is not printed.

`p7`, inside a routine reached as a function:

```rexx
say 'main-start'
rv = sub()
say 'rv=' rv
exit 0
sub:
  say 'sub-start'
  interpret "return 11"
  say 'sub-after-interpret'
  return 22
```

Oracle -- stdout `main-start\nsub-start\nrv= 11\n`, empty stderr, rc 0. The
fragment's `RETURN` returned 11 from `sub`; neither `sub-after-interpret` nor
`return 22` ran. (The probe values are deliberately distinct -- 11 versus 22
versus 7 -- so that "returned from the routine", "ended the fragment only" and
"ended the program" each print different bytes.)

`p8`, bare `RETURN` in a routine reached by `CALL`: stdout
`main-start\nsub-start\nmain-after-call\n`, rc 3. Same conclusion.

**Left failing loudly, as the brief instructs**, because `InstructionKind::Return`
is Task 3's and this task must not invent it. Verified it *is* still loud rather
than silently wrong: `p6` under `rexx-run` gives `before` on stdout,
`rexx-exec: RETURN is not implemented (4b)` on stderr, rc 120. The
`Failure::Loud` propagates out of `run_fragment` through the ordinary `?` path
with nothing fragment-specific needed.

**For Task 3:** `RETURN` must forward out of `run_fragment` the way `Flow::Exit`
already does, and in the main body it must produce the same exit code `EXIT`
would. The `Flow` channel is the right vehicle; nothing about the fragment
boundary needs a special case, which is the opposite of the `LEAVE` answer above
and is why both had to be measured separately.

---

## 3. Did execution move off the `step_in_temps_frame` chokepoint?

**No.** I16's claim is intact.

`step_in_temps_frame` is still the only caller of `step` in the crate --
`grep -n "self\.step(" *.rs` finds one call site (`run.rs:1402`) plus three
occurrences inside doc-comment prose. The `Interpret` arm calls `run_fragment`, which calls `run_bounded`, which
steps every instruction through `step_in_temps_frame` exactly as `If`/`Select`/
`Do` already did. No new stepping path was introduced and none was bypassed, so
Task 7's analysis of `SIGNAL ON SYNTAX` and temps-leak accumulation rests on the
same single chokepoint it did before this task.

I also built the debug tripwire I22 scheduled and 4a did not (`RootSet::temps_len`
plus a `debug_assert` in `step_in_temps_frame`). `pop_frame` is untouched, per
I22. It asserts on the `Ok` path only: on `Err`, the six `eval.rs` sites'
unreached `pop_frame`s are healed by the outer truncation, and asserting there
would fire on a correct program's ordinary error path -- the exact reason
`pop_frame`'s own doc forbids an assertion inside it.

**Negative-controlled rather than assumed.** A tripwire that cannot fire is
worse than none, so I temporarily added `RootSet::negative_control_pop_temp` and
called it from `step`'s `Nop` arm, then ran `do idx = 1 to 2 / nop / end`:

```text
thread 'rexx-interp' panicked at crates/rexx-exec/src/run.rs:1406:9:
step popped below its own temps watermark (2 -> 1), so it discarded roots it did not push
```

Both the accessor and the call were reverted immediately afterward
(`grep negative_control` over both crates is empty). The suite is green with the
tripwire live, so it fires on a real under-pop and does not fire on the tree as
it stands.

---

## 4. The fragment-plan cache: not added, and the measurement

**Decision: no cache**, which is I8's default. The permission was conditional on
measuring a hit rate on a real program, and the measurement says the hit rate is
zero, so the condition is not met.

**Measured, by instrumentation.** I temporarily added an `eprintln!` of the
fragment text at the top of `run_fragment`, built `rexx-run`, ran every corpus
program containing `INTERPRET`, and counted total executions against distinct
texts. Instrumentation reverted afterward (`grep FRAGTEXT` is empty).

| program | fragment executions | distinct texts | hits |
|---|---|---|---|
| `corpus/lang/interpret_dynamic.rex` | 5 | 5 | **0** |
| `corpus/num/settings.rex` | 0 | 0 | n/a |
| `corpus/num/errors.rex` | 0 | 0 | n/a |

The two `num/` programs reach no fragment at all yet: both need `CALL` (Task 3),
`PARSE` (4c) and `SIGNAL ON` (Task 7) first, and stop at
`a function call is not implemented (4b)`. Their hit rate is derivable from their
source without running them, and is also zero: `settings.rex` calls its `t`
routine with twelve distinct literal strings, `errors.rex` calls `try` with six.
I have labelled that as derived rather than measured, because it is.

**Corroborating evidence from real programs**, since three corpus files is a thin
basis. The oracle's own tree has twenty `INTERPRET` sites across ten files
(`samples/`, read-only). Every one of them builds its text from a run-time value:
sixteen concatenate a variable into the text (`interpret "resultOfCall = target~"method`,
`interpret 'arg_array['i'] =' arg_array[i]`, `interpret "self~Param"i"='.NIL'"`),
and the remaining four interpret a bare variable that is a REPL input line or a
recursion's line (`rexxtry.rex:152`, `calculator.rex:162`). None is loop-invariant.

So the withdrawn `(enclosing body, fragment id)` key was rightly called "sound and
useless", and a text key is not better in practice: it would retain an entry per
distinct fragment text -- unbounded in exactly the REPL-shaped programs that use
`INTERPRET` most -- for no hits. Fragment plans are built, used and dropped, and
stay that way.

---

## 5. Test output

```
cargo fmt --all --check                                  -> exit 0
cargo clippy --workspace --all-targets -- -D warnings     -> exit 0
cargo test --workspace                                    -> exit 0
```

852 tests pass, 0 fail, 1 ignored (`probe_emit_uncaptured_marker`, by design).
`rexx-exec`'s own lib tests went 184 -> 188 (four added in `lib.rs`, two removed
and three added in `run.rs`); `tests/spike.rs` went 11 -> 7.

Corpus differential: **`29 of 29 matching`**, unchanged. The assertion table is
`4224 of 4259`, byte-identical to the figure at `38fed3a1` -- confirmed by
stashing this work, re-running, and unstashing, rather than assumed.

`lang/interpret_dynamic.rex`, the new 4b witness, was run through both
interpreters directly: **stdout MATCH, stderr MATCH, rc MATCH (0)**.

---

## 6. Concerns

**Concern 1 -- the nested clause echo is now a live divergence, not a latent
one.** The oracle prints one `*-*` echo per `INTERPRET` nesting level, innermost
first, each carrying the *enclosing* clause's line number; this crate prints only
the enclosing one. All eight `LEAVE`/`ITERATE` boundary probes match the oracle on
stdout, on rc, on both error numbers and on the two `Error …` lines, and differ by
exactly that one missing line. `run_activation`'s own comment recorded this as a
known gap in 4a and said 4a's only nesting was "the fragment spike that 4b
deletes" -- which was true then and is false now, so I corrected that comment.
Fixing it needs `FailureSite` to become a stack and the fragment's clause text to
be resolved against `Fragment::source` while its line number comes from the
enclosing `INTERPRET`; that is a `Raised::report` change, not an instruction-loop
one, and it is outside this task. ~~**Any future corpus program that raises inside
`INTERPRET` text will fail the byte-for-byte gate until it is fixed.**
`interpret_dynamic.rex` avoids it only by raising nothing.~~

**That bound is wrong, and the review (I1) falsified it by measurement.** Struck
through rather than deleted, because a wrong bound on a known gap is the kind of
error worth leaving visible. The missing echo is not confined to the error path:
the same `source: None` suppresses the fragment's clause under `TRACE` too, where
nothing raises at all, and `TRACE` has been in scope since 4a. The correct bound
is **any program that traces *or* raises inside fragment text diverges by one line
per fragment clause**, and `interpret_dynamic.rex` avoids it by doing neither. The
fix-round section below has the transcript. What I got wrong was reasoning about
the gap's extent from the path I had happened to exercise, rather than asking what
else reads the same `source` argument -- the same shape as the `LEAVE` error, one
finding earlier in the same task.

**Concern 2 -- a fragment that does not parse is loud, not a condition, and that
is now reachable too.** Measured: `interpret "do forever then"` gives the oracle
27.901 at rc 229; this crate gives `rexx-exec: INTERPRET text did not parse:
27.901: Invalid DO or LOOP syntax.` at rc 120. Loud rather than silent, so
criterion 5 holds, but not byte-identical. This is `execute`'s standing position
on parse errors ("wrong in the details on purpose and right in never being
mistaken for success") and applies equally to a top-level syntax error today; the
fix is one `ParseError`-to-`Raised` conversion serving both, which should be built
once rather than here for one caller. Recorded in `Loud::parse`'s doc.

**Concern 3 -- `tests/corpus.rs`'s `read_subset` call site was left at
`phase-4a.txt`, and that is a judgement call I want reviewed.** The brief names
three call sites ("two of them must change and one must not") and `corpus.rs:478`
is a fourth it does not mention. I followed the brief literally: changing it would
have made the differential harness report `30 of 30` rather than the `29 of 29`
the coordinator's instructions pin, and widening a gate figure is not a decision
this task should make silently. The cost is that `phase-4b.txt`'s witness is read
by `coverage.rs` (parse coverage) and `collect_stress.rs` (stress equivalence) but
**not** by the harness that compares against the oracle. I closed that gap by hand
for this task -- `interpret_dynamic.rex` matches byte for byte, reported above --
but the next 4b task adding a witness will not have that done for it. If the
intent is that the differential harness reads the union, that is a one-line change
and should be made deliberately, with the report figure expected to move.

**Concern 4 -- `Interp::new`'s parameter removal touched five files this task's
brief does not list** (`eval.rs`, `value.rs`, `stem.rs`, `plan.rs`, `trace.rs`).
It is unavoidable given the instruction to delete the constructor parameter, and
it is mechanical: the diff for those five files contains nothing but
`Interp::new()` call lines.

---

# Addendum: the coordinator's rulings, and the follow-up commit

The coordinator ruled on all four concerns after the first commit. Concerns 1 and
2 were assigned to Task 2 (the one-echo-per-nesting-level gap was already its
work; the fragment parse error became its new Step 5b, carrying the measurement
below and the note that a top-level syntax error wants the same
`ParseError`-to-`Raised` conversion, so it gets built once). Concern 4 needed no
action. The plan is corrected at `4ccfae88`. Concern 3 was granted, and this
addendum records the change it asked for.

## `tests/corpus.rs` now reads the union

The brief said "two of them must change and one must not" and named three
`read_subset` call sites; there are four, and the missed one is the harness that
actually runs both interpreters and compares them. Leaving it pinned to
`phase-4a.txt` produced a witness the coverage harness enumerates and no
differential harness ever reads -- a witness in name only, which the next 4b task
adding one would have inherited silently.

Changed: the call site itself, `read_subset`'s doc (which claimed a one-element
slice), the module doc, the report's own title line, and the STRICT-mode assertion
message, which said "phase-4a corpus programs" of a set that is no longer only
those. `EXPECTED_SUBSET` is untouched and still pins `phase-4a.txt` alone.

Nothing pinned the program count, as the coordinator checked: the figure is
computed from `subset.len()`.

**Corpus is now `30 of 30 matching`**, up from 29 of 29, the added program being
`lang/interpret_dynamic.rex`.

## `interpret_dynamic.rex` verified through the harness, not by hand

Two checks, because a count rising from 29 to 30 does not on its own prove the
30th program was compared rather than skipped:

1. **STRICT gate mode.** `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus
   corpus_differential` exits 0 and prints `mode: STRICT (the gate)` and
   `30 of 30 matching`. STRICT asserts `mismatches.is_empty()`, and `check_case`
   compares stdout, stderr and exit code as three separate descriptors, so this
   is the harness itself certifying all three for all thirty.
2. **A negative control on the new file.** I temporarily appended
   `num/settings.rex` -- a program that stops at `a function call is not
   implemented (4b)` -- to `phase-4b.txt`, and the harness reported:

   ```text
   30 of 31 matching -- REPORT MODE, NOT THE GATE
     [a function call] num/settings.rex: stdout, stderr, exit code differ (loud failure: a function call is not implemented; rust rc 120, oracle rc 0)
   ```

   So the runner genuinely reads `phase-4b.txt` and genuinely compares what it
   finds there; the 30th slot is not being silently passed. Reverted immediately
   (`phase-4b.txt` is back to its one entry).

## Re-run gates after the change

Each read unpiped:

```
cargo fmt --all --check                                -> FMT_EXIT=0
cargo clippy --workspace --all-targets -- -D warnings   -> CLIPPY_EXIT=0
cargo test --workspace                                  -> TEST_EXIT=0
```

852 tests pass. Confirmed precisely rather than by eye: zero lines containing
`FAILED`, zero containing `failures:`, no `test result` line that is not `ok`, and
all 68 test binaries reporting `0 failed`. The assertion table stays
`4224 of 4259`.

## The `LEAVE` finding was measured, not reasoned about, and the code was wrong

Recording this at the coordinator's request, because a worked counterexample is
more use to the next implementer than the rule restated.

Before the measurement, this crate forwarded a bare `Flow::Leave` out of a
fragment for the enclosing `DO` to consume. That behaviour follows from a true
premise, stated in `run_fragment`'s own doc comment and correct in every other
respect: the fragment runs *inside* the creating activation -- same frame, same
slots, no activation pushed. If the enclosing loop's frame is right there, a
`LEAVE` reaching it is what you would predict, and I would have predicted it.

The oracle does the opposite. `interpret "leave"` inside `do kk = 1 to 3` is
error 28.1, "LEAVE is valid only within a repetitive loop", at rc 228 -- the
enclosing loop is invisible. Six probes, four condition families, both `DO` and
`SELECT`, all agree. Ten seconds of running settled what an afternoon of
reasoning from a correct premise would have got wrong, and the plan's first
revision asserted one of these two answers rather than measuring it.

The same shape has now cost this phase time three times: two confident
unreachability arguments (one of which this task deleted from
`run_activation`, where it claimed the nested-echo gap could not be reached) and
this one. The distinguishing feature in every case is that the reasoning was
*sound* and the premise it started from was *true*; what was missing was a
second premise nobody knew to look for. That is not a failure mode more care
fixes. It is one that running the program fixes.

## Follow-up commit

**`a9420630`** (`a942063022d8f9664cdb6b0b664af5fdf07ff5bd`), read back with
`git log` after committing, one file: `rust/crates/rexx-exec/tests/corpus.rs`.

So this task is two commits: `a462e3e9` (the implementation) and `a9420630`
(this follow-up), with the coordinator's plan correction `4ccfae88` between
them.

---

# Fix round 1: the two Important findings, and two Minors

Commit: **`ebbfb3d7`** (`ebbfb3d742673a473409af737bf56991b4af8529`), seven files,
read back with `git log` after committing.

Review verdict was spec **MET** / quality **PASS WITH FINDINGS**, 0 Critical, 2
Important, 11 Minor. This round fixes I1, I2, M1 and M2 only; the other nine
Minors are deferred to the whole-branch review as instructed.

## I1 -- TRACE inside `INTERPRET`

**(a) fixed.** The `Interpret` arm now calls
`self.trace_result(self.current_value_indent, &text)` before `run_fragment`,
matching `Say` and `Assignment`. It was the only value-producing arm in the crate
that traced nothing.

Re-measured on the oracle myself rather than taken from the review, and it
reproduces exactly. `trace r` / `zz = 'nop'` / `interpret zz`:

```text
     2 *-* zz = 'nop'
       >>>   "nop"
     3 *-* interpret zz
       >>>   "nop"
     3 *-* nop
```

Also measured one `DO` deeper, which the review did not cover and which is where a
plausible wrong fix still fails: the oracle's `>>>` picks up that construct's own
two spaces (`       >>>     "nop"`), because the value indent is the enclosing
clause's. After the fix, all three probes match the oracle on stdout and rc, and on
stderr differ only by the fragment's own clause echo -- one line each.

New test `interpret_traces_the_text_it_is_about_to_run` (`run.rs`) pins both the
byte-exact top-level transcript and the indented case. Both mutation-kills in its
doc were carried out, not asserted:

* deleting the `trace_result` call -> FAILED;
* moving it after `run_fragment` -> FAILED, with the output showing exactly why
  (`>>>   "nop"` at indent 0 instead of `>>>     "nop"` at 2, because the fragment's
  own stepping has overwritten `current_value_indent` by then). That is the
  "before, not after" claim in the arm's comment verified rather than assumed.

**(b) deferred to Task 2, as the coordinator allowed, and the reason is
structural.** `run_fragment` passes `source: None`, which is what suppresses the
fragment clause echo. Passing `Some(&fragment.source)` does not fix it and makes
something else worse: the echo would carry the *fragment's* line number where the
oracle prints the enclosing `INTERPRET`'s, and the fragment's clause would win
`record_failure_site`'s first-wins race, moving the error report off the line the
oracle names. Both need the echo stack. Disclosed at three sites now -- the
`Interpret` arm, `run_activation`'s comment, and the corrected Concern 1 above.

**A pre-existing gap found while measuring this, and it is not mine.** `t3`'s
transcript is also missing the `>>>` control-variable lines a `DO` re-emits on each
iteration. Checked against the same program with the `INTERPRET` removed
(`trace r` / `do kk = 1 to 2` / `nop` / `end`): it diverges identically. So that is
a 4a `DO`/`TRACE` gap, untouched by this task and reported here only because I
found it.

## I2 -- the witness, and why the prescribed fix was itself vacuous

The finding is right: `phase-4b.txt`'s header claimed the witness binds a name the
enclosing body never mentions, and it did not.

**The prescribed fix does not fix it, measured.** The review and the dispatch both
say to append `interpret "zork = 42"` and `say zork`. I applied exactly that,
instrumented `slot_of`'s growth branch, and got **zero** `Activation::extra` hits --
the same result as before the fix. The cause is that `say zork` sits in the
*enclosing* body, so `ZORK` is in that body's own plan and `activation.plan.slot_of`
finds it; `extra` is never reached. The review's own positive control was
`interpret "zork = 42"` / `interpret "say zork"`, with **both** sides inside
fragments, and the prescription silently dropped that property when it turned the
second line into a bare `say`.

Verified in both directions before concluding, since an instrumentation that prints
nothing is indistinguishable from a branch that is never taken:

| program | `Activation::extra` hits |
|---|---|
| the witness before this fix | 0 |
| the witness + `interpret "zork = 42"` + `say zork` (as prescribed) | **0** |
| the witness + `interpret "zork = 42"` + `interpret "say zork"` | **1, `ZORK`** |

So the appended lines are `interpret "zork = 42"` / `interpret "say zork"`. The
differential still matches the oracle byte for byte on all three descriptors, which
is the property the review verified for its own variant and which holds for this one
too. The review's oracle-match claim was correct; only its coverage claim failed.

`phase-4b.txt`'s header now states the property with the condition that makes it
true -- both the write and the read inside fragments -- and says outright that the
two spellings are indistinguishable in the output, so the next editor re-runs the
check instead of assuming it survived.

**Two pins had to move with the program, both deliberately.**

* `rexx-parse`'s `the_corpus_programs_parse` pins an instruction count per corpus
  program: 8 -> 10, with a note saying what the two new instructions are for.
* `rexx-parse`'s `sourceline_matches_the_interpreter_for_every_corpus_program`
  reads a captured expectation file. I **regenerated it from the oracle** rather
  than editing the count by hand, using the driver the module doc documents -- run
  against a scratch **copy** of the file, because that driver calls `.Package~new`
  and the standing constraint forbids that on a file inside the repository. Oracle
  answered `count 10` and its ten lines; the regenerated file's line block is
  byte-identical to the corpus program (`cmp`, not eye).

## M1 and M2

* **M1**, `tests/loud.rs`: the doc said "19 entries" for a 23-row list, counting
  coarse tags while describing the expanded one. It now says 23, shows the
  arithmetic (19 + 3 + 1), and records that the line has now been wrong twice with
  two different numbers.
* **M2**, `tests/corpus.rs`: "each of the two tasks still to land" was 4a plan
  state and is gone; the 3-of-26 and 9-of-26 figures are kept and labelled as a
  dated record of 4a.

## The dated figure

Kept, and a second dated row added beside it rather than replacing anything:
`Expected result at commit a9420630: 30 of 30 matching`. That is what the original
row's own instruction asks for -- re-run and record which commit was measured --
and the review was right that leaving it untouched satisfied the letter and not the
intent.

## Gates

Each read unpiped:

```
cargo fmt --all --check                                -> FMT_EXIT=0
cargo clippy --workspace --all-targets -- -D warnings   -> CLIPPY_EXIT=0
cargo test --workspace                                  -> TEST_EXIT=0
REXX_CORPUS_GATE=1 ... corpus_differential              -> STRICT_EXIT=0
```

**853 passed, 0 failed, 4 ignored** -- 853 rather than 852 because of the new
trace test, and **4** ignored, which corrects report §5's "1 ignored" (review M11:
1 is the `-p rexx-exec` figure, the other three are `rexx-num`'s `--ignored`
multi-gigabyte tests). Assertion table unchanged at `4224 of 4259`.

## The corpus figure is 30 of 30, not 31 of 31

The dispatch said to confirm `31 of 31`. It is **30 of 30**, and that is correct
rather than a shortfall: the fix grows an existing program by two clauses, and the
subset still names thirty programs (29 in `phase-4a.txt`, 1 in `phase-4b.txt`).
Reaching 31 would mean adding a thirty-first program file, which neither the review
nor the dispatch asked for and which I have not done on my own initiative.
STRICT mode certifies all thirty on all three descriptors.
