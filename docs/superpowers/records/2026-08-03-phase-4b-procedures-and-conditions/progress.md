# SDD ledger -- plan: docs/superpowers/plans/2026-08-03-phase-4b-procedures-and-conditions.md

Branch `plan/rust-rewrite`, worktree `/home/moritz/dev/repos/ooRexx-rust-rewrite`.
Plan committed at `dc87708d`, after a first revision (`47109a92`) that four adversarial
reviewers found substantially wrong. Their reports are in the session scratchpad; what
they found is folded into the plan, so the plan governs and the reports are not required
reading.

Pre-flight scan: done as four parallel adversarial reviews rather than a self-scan.
8 Criticals found, all resolved in the plan before Task 0 was dispatched. No open
plan-vs-rubric conflict was left for the human to adjudicate.

BASE for Task 0: dc87708d7d0347e6f3e156ba1309d7162adc2585

## Task 0: shared owner table, arm-grained loud witnesses, owner in every loud message

Implementer: t0-owners (sonnet). Status DONE.
Commit `0197b360` -- 6 files, +907/-519. `tests/owners.rs` created; `src/lib.rs`,
`tests/{coverage,loud,corpus,collect_stress}.rs` modified.
Suite green (12 binaries, 0 failed), clippy clean, fmt clean.

**Loud-message format, which every later task copies:** `"{name} is not implemented ({owner})"`.
Live examples: `rexx-exec: CALL is not implemented (4b)`,
`rexx-exec: CALL is not implemented (Phase 5)` for a qualified call.
Two in-scope-but-locally-blocked sites in `run.rs` (the DO/LOOP COUNTER/WITH check and the
stem-target DO OVER deviation) keep their old owner-less message; they were outside this
task's file list.

Ownership data now lives in **three** places: `tests/owners.rs`, `loud.rs`'s witness rows,
and a match in `src/lib.rs` (production code cannot reach a test module). The new assertion
that stderr contains the witness's owner is what keeps the third copy honest.

Task 0 review dispatched (opus), BASE `dc87708d`, HEAD `0197b360`.

### Task 0 review (opus): spec MET WITH GAPS, quality PASS WITH CHANGES REQUIRED

0 Critical, 6 Important, 7 Minor. Report: `task-0-review.md`.
The reviewer re-ran everything and added a seven-mutation battery against the drift guard.
Caught: owner never appended; `Call::Qualified` owner flipped; `ExprKind::List` owner
flipped; `Call::Trap` witness row deleted. **Not caught:** deleting the `"4a"` carve-out,
rewriting the message shape wholesale, and flipping `expr_owner`'s `VariableReference` arm.
So the guard is real and discriminating, with three holes -- which is round 1's work.

Fix round 1 dispatched to t0-owners with all six Important findings. Controller rulings:
I6 takes the `Option<&'static str>` route (`None` == implemented here) rather than an
empty-string constant, because keying the carve-out on the phase name `"4a"` traps Task 3;
I4 is a controller edit and was done here, not by the implementer.

Task 0: minor (deferred): M1 Step 5's block sits at the end of the file, not in the module doc it claims to be in.
Task 0: minor (deferred): M2 `expand_for_witnesses` is not exhaustiveness-checked against `rexx_parse::Call`/`Signal`.
Task 0: minor (deferred): M3 `the_two_harnesses_include_this_exact_file` is defeated by the regression it names.
Task 0: minor (deferred): M4 `instruction_arm` returns `String` where `&'static str` suffices.
Task 0: minor (deferred): M5 the "unavoidably a third copy" claim overstates the constraint.
Task 0: minor (deferred): M6 the reported `cargo fmt --edition 2024 --check` invocation does not run -- `error: unexpected argument '--edition'`. The working form is `cargo fmt --all --check`. **This affects every task's brief**, which tells implementers to use the broken form.
Task 0: minor (deferred): M7 two comments describe a stderr shape that no longer exists (fold into I2's fix if free).

### Task 0 fix round 1: commit `1b7c4b1f`

Five of six Important findings closed. Re-reviewed independently (`task-0-rereview.md`),
which re-ran the mutations rather than trusting the report: F (delete the carve-out) and
G (rewrite the message shape) both **caught**; suite, clippy, fmt clean; 29/29 corpus
unchanged; `run.rs` expansion ruled **not** scope creep, since the original review named
that test module as I1's fix location.

I6 took the `Option<&'static str>` route as adjudicated -- `None` means implemented-here,
and the `"4a"` phase-name sentinel is gone.

**I3 reopened.** The implementer argued `expr_owner`'s `VariableReference` arm is
structurally unreachable until Task 3 implements `Call::Named`, and wrote that into three
comments. It is false. Verified here on the current tree:

    $ printf 'say >x\n' > vr.rex && cargo run -q --bin rexx-run -- vr.rex
    rexx-exec: a variable reference is not implemented (4b)
    rc=120

No `CALL` anywhere; `SAY` is 4a's own, so the expression is evaluated and the arm is
reached. The cited `ast.rs` error 20.930 reasoning is about which token may follow `>`,
not about which instruction context `>x` may appear in.

Round 2 dispatched: replace `EXPR_WITNESSES`' `VariableReference` witness with `say >x`,
re-run mutation C, and **delete** the three false comments. Told the implementer explicitly
that the never-delete-a-comment rule protects information and does not protect a false
statement, because following it literally would produce hedged versions of the same claim.

**The transferable finding, and it is the second instance this session.** A plausible
mechanism-story was accepted in place of a one-line test, and the confident wrong
explanation was more harmful than the gap it explained: an unwitnessed arm invites
scrutiny, three comments saying "understood, deferred" stop a reader looking. The
controller made the identical error earlier the same day, concluding `>I>`/`<I<` belong to
Phase 5 from a `trace i` probe that could never have produced them. Both were caught by
someone re-running the thing rather than re-reading it.

### Plan amendments made during execution

* `0599015a` -- record Task 0's loud-message format in the plan; Step 3 said to and it was
  left in code and a report only.
* `f1527d44` -- the Global Constraints told every task to check formatting with
  `cargo fmt --edition 2024 --check`, which exits 2 on an unknown flag and verifies
  nothing. Task 0 ran it and reported formatting clean. Working form: `cargo fmt --all --check`.
* `74769262` -- `rust/corpus/phase-4b.txt`'s creation moved from Task 10 to Task 1, which
  needs a subset to hold its `Interpret` witness. Same forward-dependency class the
  reviewers found for the owner table; that half was fixed by adding Task 0, this half
  was missed until Task 1's pre-dispatch check.

### Task 0 fix round 2: commit `38fed3a1` -- Task 0: complete

I3 closed. Witness changed from `call sub >x` to `say >x`; the three false
unreachability comments deleted, plus a fourth pre-existing one in `loud.rs`'s module doc
that predated Task 0 and made the same wrong claim -- leaving that standing beside the fix
would have restated the error.

Mutation C re-verified **by the controller, not accepted from the report**: flipping
`expr_owner`'s `VariableReference` arm to `"Phase 7"` now fails
`every_out_of_scope_variant_fails_loudly` with
`stderr does not end with " is not implemented (4b)"`, cargo exit 101. Tree restored via
`git checkout -- <file>`, 0 dirty files, full suite re-run green (exit 0), corpus 29 of 29.

Task 0: complete. Commits `0197b360`, `1b7c4b1f`, `38fed3a1`.
BASE for Task 1: 38fed3a12e540c9d191e9a30b6a136ff31d13a3e

## Task 1: a real INTERPRET, replacing the Task 3 spike

Implementer: t1-interpret (opus). Commits `a462e3e9` (15 files) and `a9420630` (corpus
call site). Suite 852 pass / 0 fail, clippy and fmt clean, corpus **30 of 30**, assertion
table unchanged at 4224 of 4259.

**Escape semantics, both measured rather than reasoned:**

* `LEAVE` never crosses an INTERPRET boundary. An enclosing loop is invisible from inside
  fragment text, including to a bare `LEAVE`, so an unmatched one raises 28.1-28.4 at the
  fragment's own edge, rc 228. **This contradicted what the code did.**
* `RETURN` crosses outward exactly like `EXIT`. Left failing loudly, since
  `InstructionKind::Return` is Task 3's.

Execution did **not** move off `step_in_temps_frame`; it is still the only caller of `step`
in the crate, so Task 7's temps-leak analysis stands. I22's debug tripwire was built on a
new `RootSet::temps_len` (Ok path only, `pop_frame` untouched) and negative-controlled.

Fragment-plan cache **not** added, on evidence: 5 executions, 5 distinct texts, 0 hits, and
all twenty INTERPRET sites in the oracle's own `samples/` build their text at run time.

**Two divergences became reachable for the first time** because `run_program` now has a
real INTERPRET. Both are Task 2's: the one-echo-per-nesting-level report gap, and a
fragment that fails to parse raising 27.901 rc 229 on the oracle where we exit 120 loud.
The second had no home in the plan until this task measured it.

**Plan defect the task found by doing the work.** Step 8 said "two must change and one must
not" and named three `read_subset` call sites; there are four, and the missed one
(`tests/corpus.rs:478`) is the harness that actually compares the two interpreters. A 4b
witness would have been enumerated by the coverage harness and differentially tested by
nothing. Corrected at `4ccfae88`. The implementer followed the brief literally and flagged
it rather than widening a gate figure unasked, which is the behaviour to keep.

The 30th slot was proved to be *compared* rather than skipped, by negative control:
appending a known-failing program gave `30 of 31` with a named divergence. 29 -> 30 alone
would not have shown that.

Task 1 review dispatched (opus), BASE `38fed3a1`, HEAD `a9420630`.

### Task 1 review (opus): spec MET, quality PASS WITH FINDINGS

0 Critical, 2 Important, 11 Minor. Report: `task-1-review.md`. All six implementer claims
re-measured independently and all six held: 8 LEAVE/ITERATE probes with all four error
numbers (28.1 bare LEAVE, 28.2 bare ITERATE, 28.3 named LEAVE including `select label`,
28.4 named ITERATE), RETURN crossing outward, `run.rs:1402` the crate's only `self.step(`
call site, the tripwire made to fire, the zero hit rate reproduced, and the 30th corpus
slot proved compared by the reviewer's own negative control (mutate the Interpret arm to
`Ok(Flow::Next)`, gate names `lang/interpret_dynamic.rex`).

**I1 -- TRACE inside INTERPRET is a live divergence introduced by this task.** `trace r` /
`x='nop'` / `interpret x`: oracle prints `>>> "nop"` and `3 *-* nop`, we print neither. The
new arm never calls `trace_result` on its evaluated text, unlike every other value-producing
arm. It also falsifies the bound the implementer put on their own Concern 1 -- they wrote
that only a program which *raises* inside fragment text fails the gate; one that merely
traces fails too, and TRACE has been in scope since 4a.

**I2 -- the plan's own false claim, propagated into the tree.** Step 8 asserted
`interpret_dynamic.rex` binds a name the enclosing body never mentions. Line 3 is `v = 0`,
so it does not; instrumenting `slot_of`'s `Activation::extra` branch gives 0 hits against a
positive control of 1. `Activation::extra` exists precisely for that case, so the property
motivating the design was witnessed by a unit test and by no differential program. Plan
corrected at `cf7bb05c`; fix appends `interpret "zork = 42"` / `say zork`.

**Fifth false statement of the phase, and the first originating in the plan text.** The
implementer copied it faithfully, which is what a brief is for -- a wrong assertion in a
brief is propagated, not checked. That asymmetry is the reason the pre-dispatch check and
the re-run-don't-re-read review instruction both earn their cost.

Dated-figure ruling: **keep** `9 of 26 at commit e0e57825`. A figure anchored to a named
commit and carrying a re-measure instruction is a record, not a stale claim -- but it asks
the reader to record which commit was measured, so `30 of 30 at a9420630` goes beside it.

Fix round 1 dispatched: I1, I2, and two Minors that are false statements rather than style
(`loud.rs:178` says "19 entries" for a 23-row list; `corpus.rs:26-31` says "each of the two
tasks still to land", no longer true).

Task 1: minor (deferred): two provably-no-op `record_leave_failure` calls in `run_fragment`.
Task 1: minor (deferred): 16 lines duplicated from `run_activation`'s arms.
Task 1: minor (deferred): a mutation-kill note naming a mutation that cannot be typed.
Task 1: minor (deferred): a verbatim-preserved doc now false in the present tense.
Task 1: minor (deferred): an ungrammatical `Outcome::collections` rewrite.
Task 1: minor (deferred): nothing pins `phase-4b.txt`'s line list, unlike `EXPECTED_SUBSET` for 4a.
Task 1: minor (deferred): Step 4's test duplicates an existing assertion without saying so.
Task 1: minor (deferred): a tripwire cost claim that relies on dead-code elimination.
Task 1: minor (deferred): the report says "1 ignored" where the workspace has 4.

### Task 1 fix round 1: commit `ebbfb3d7`

I1 half (a) fixed -- the `INTERPRET` arm now calls `trace_result` on its evaluated text,
matching `Say`'s shape and indent, re-measured one `DO` deeper than the review covered.
Half (b), the clause echo, deferred to Task 2 with a structural argument (see below).
M1 and M2 fixed. Dated figure kept with `30 of 30 at a9420630` added beside it.
853 passed / 0 failed / 4 ignored; fmt, clippy and the STRICT corpus gate all exit 0.

**The controller's prescribed I2 fix was itself vacuous, and the implementer refused it.**
I relayed the reviewer's `interpret "zork = 42"` plus a bare `say zork`. The bare `say`
puts `ZORK` in the *enclosing* body's clause list, so `Plan::build` gives it an ordinary
slot and `Activation::extra` is never reached -- zero hits, the very defect being fixed.
Shipped instead: `interpret "zork = 42"` / `interpret "say zork"`, one hit, still
byte-identical to the oracle. Confirmed here by reading the program: `ZORK` now appears
only inside quoted fragment text, so the plan never sees it.

The review's *oracle-match* claim was right and only its *coverage* claim failed, and the
two spellings are indistinguishable in the output -- both print 42, both match. That is why
`phase-4b.txt`'s header now states the condition rather than the observation.

**Sixth instance of the family, and specifically "a fix for vacuity can itself be
vacuous"** -- a shape already recorded in the gate-criteria memory before this phase began.

**Controller error: the dispatch asked for `31 of 31`.** Wrong -- the fix grows an existing
program by two clauses rather than adding a file, so thirty is correct. The implementer
declined to invent a thirty-first program to satisfy the number, which was right.

**The implementer's "new" DO/TRACE gap is I31**, already owned by Task 9 and already costed
at about twenty lines. Measured here: `trace r; do i = 1 to 2; nop; end` -- the oracle emits
two `>>>` per re-tested pass (`"1"`/`"2"`, then `"2"`/`"3"`), we emit none. The plan's gap
inventory is accurate; no new row.

**Half (b)'s deferral argument, to be carried into Task 2's body if it survives review:**
passing `Some(&fragment.source)` would print the fragment's own line number where the oracle
prints the enclosing `INTERPRET`'s, and would take the error report off the line the oracle
names by winning `record_failure_site`'s first-wins race. It needs the echo stack Task 2
builds. Sent to a scoped re-reviewer with instructions to **refute** it rather than confirm
it, since structural-impossibility claims have been wrong six times this phase.

### Task 1 fix-round re-review: all four ADDRESSED. Task 1: complete.

Re-reviewer reproduced everything rather than reading it, and went past the brief:
instrumented the **entire** 30-program corpus in one pass, total `Activation::extra`
hits = 1 (ZORK), confirming nothing else in the differential set reaches that path.

`extra` hit counts: original 0, controller-prescribed (bare `say zork`) 0, shipped
(`interpret "say zork"`) 1. The prescribed fix was vacuous, as the implementer said.

**Half-(b)'s deferral survives, and it is the first structural-impossibility claim this
phase to do so.** It survived because the reviewer *built the naive fix and ran it*: passing
`Some(&fragment.source)` does not supplement the enclosing echo, it **replaces** it, and it
moves the reported error line from the correct 3 to 1. Both claimed failure modes
reproduced, then reverted. Carried into Task 2 Step 4 at plan level, with the warning not to
reach for the one-line version, which has now been built twice and fails both ways.

Also verified: `rexx-parse`'s SOURCELINE regeneration reproduced byte-identically against a
scratch copy rather than a repo file; every comment the diff adds or changes checked and
found true; both mutations named in the new test's doc carried out by hand and both kill it;
tree clean after three rounds of temporary edits.

Task 1: complete. Commits `a462e3e9`, `a9420630`, `ebbfb3d7`.
BASE for Task 2: ebbfb3d742673a473409af737bf56991b4af8529

## Task 2: the error report's site stack, the 40-column cap, the fragment ParseError

Implementer: t2-sitestack (opus). Commit `addf320f`. Suite 858/0/4, fmt and clippy 0,
**corpus 32 of 32** (was 30).

Delivered: one clause echo per nesting level innermost-first; the `*-*` clamp at 40 columns
applied at **both** formatters (`Raised::report` and `push_clause`), closing a divergence
that shipped in 4a; the missing `*-*` echo for a traced `INTERPRET`; and a fragment
`ParseError` raising 27.901 at rc 229.

**Two corpus programs, not one.** `lang/deep_nesting_indent_cap.rex` went into
`phase-4a.txt` with `EXPECTED_SUBSET` amended in the same commit, as the brief required.
The implementer then added `lang/interpret_error_echo.rex` to `phase-4b.txt` **beyond the
file list**, on the grounds that the task's central deliverable would otherwise have no live
differential witness: `interpret_dynamic.rex` was written to neither trace nor raise
*precisely because this gap existed*. Sound, and the same "witness that cannot observe its
own subject" shape that has bitten this phase twice. Ruling: keep.

**Step 5b is partly open, bounded, and asserted as-is.** Two residual divergences: the
oracle also echoes the failing fragment clause at indent 0 regardless of enclosing indent
(measured, so not the activation base under another name), and our sub-message renders
`found "&1"` where the oracle has `found "THEN"`. The implementer **declined to guess** a
clause end, because `ParseError` carries the clause's start byte with no end, and
to-end-of-source is right for a single-clause fragment and wrong for
`interpret "do jj = 1 to 1; do forever then; end"`, whose echo is `do forever then;`. The
unit test asserts both divergences as they are, so neither can shrink unnoticed.

**Top-level syntax errors can share the mapping but not the report.** `parse_program` takes
text by value and returns no `ProgramSource` on the failure path, so `execute` cannot resolve
the line `Raised::report`'s major line names, and `rexx-parse`'s `parse(&ProgramSource)`
composition is private. Closing it is a `rexx-parse` signature change, outside the file list.
Recorded in `execute`'s own parse arm.

**The `CALL` indent rule is measured but deliberately not implemented.** Callee base =
calling clause's **printed** indent + 2, over six programs including the discriminating one
(a call two `DO`s deep into a flat callee gives 6, where 2 x depth predicts 2). Distinct
from `INTERPRET`'s +0. Both are now in `phase-4-exclusions.txt`'s amended KNOWN GAP row for
Task 3, with the three mechanisms it needs named: `indent_offset`, `clause_line_override`,
`seal_site_level`. This independently confirms D2r.

**`Option<&ProgramSource>` now has no `None` caller** (`run_fragment` was the last). Kept,
with seven comments corrected that claimed a `None` caller exists or that a site inside a
fragment is unresolvable. Collapsing it is mechanical across every threading signature --
a follow-up, not this task's.

**Self-caught false comment.** The implementer wrote "three of the four mutations survive at
caller-indent 0" into a doc comment, measured it, found it was two for two unrelated
reasons, and replaced the claim with the measured table. Sixth instance of the family this
phase and the **first caught by its own author before anyone else saw it**.

**New divergence found while probing, unrecorded anywhere:** a newline inside `INTERPRET`
text is error 13.1 at rc 243 on the oracle; we accept it. Awaiting reviewer confirmation
before a KNOWN GAP row goes in.

**Controller error, corrected at `c30e1901`:** the brief claimed 4a byte-verified the report
"on eleven programs and all eleven must still pass". No such set exists -- nine `#[test]`
functions in `error.rs` plus one in `spike.rs`, and they are tests rather than programs.
Copied unverified from the scoping document. Third unchecked assertion to reach an
implementer from a brief this phase; the implementer spent effort hunting for the set before
verifying the property that actually matters (no expected-byte literal changes).

Task 2 review dispatched (opus), BASE `1c6cc329`, HEAD `addf320f`.

### Task 2 review (opus): spec MET (one partial), quality GOOD. Fix round 1: `6c24ab15`

0 Critical, 5 Important, 8 Minor. Reviewer ran all five mutations itself, found **no vacuous
assertion**, and called the test hygiene the best in this tree -- every new assertion
oracle-grounded rather than self-consistent. Second corpus program and `Option` retention
both upheld independently.

**F1 was caused by my pre-dispatch instruction.** I found `Interp::indent_offset` existed and
told Task 2 to extend it. It is a *transient escape elevation* (`0 -> 4 -> 0`) with two
**absolute** writers (`run_otherwise` sets 0, the absorbed `WhenCase` false escape sets 4), so
a long-lived fragment base sharing it is destroyed by both. The check found the field existed;
I never checked what it meant. Fixed with its own field, `Interp::activation_indent`.

**My fix instruction was also incomplete, and the implementer measured its way past it.**
I relayed "add it at every site that adds `indent_offset` today". `pop_search_frame` adds
*neither* offset and still needed the base -- three `LEAVE`-inside-a-fragment shapes reported
at 0 where the oracle prints 2. It now takes the base and deliberately not the elevation,
leaving Task 11's fourteen shapes byte-identical. Second: `indent_offset` must be **zeroed**
for the fragment's duration, not merely left alone, because the enclosing clause's printed
indent already contains the elevation -- an `INTERPRET` inside an escaped `OTHERWISE` one `DO`
deep gave 16 where the oracle gives 12. That shape passed *before* this round, so it is a
guard row in the new test rather than a fourth bug.

**F2 was generalised rather than patched.** `Interp::printed_indent` is now the single place
either offset is applied and all six sites go through it. F2 existed because six sites
open-coded the addition and one was missed; the fix removes the class. `static_indent`
untouched and still pure.

**F4 recorded in four places**, seven-shape table re-measured row by row rather than copied --
including the zero-trip row, which is what shows the decrement belongs to the failing re-test
rather than to the construct.

**F5 was the implementer's own error, named rather than quietly deleted:** they ran the
newline probe *before* Step 5b landed, read rc 120, and never re-ran it. The crate handles it
correctly. Had I written that gap row when it was reported, the exclusions file would now
carry a fabricated divergence.

Suite 860/0/4, fmt and clippy 0, corpus 32 of 32 unchanged. The F1 fix moved no existing
expectation structurally rather than luckily: `activation_indent` is 0 for every program not
running a fragment, so adding it at a site cannot change a 4a answer.

Fix-round re-review dispatched (opus), BASE `8b7e4053`, HEAD `6c24ab15`.

### Task 2 fix round 2: commit `e4caa7bf` (done by the controller)

The implementer hit its session limit mid-round; the round was documentation only, so I did
it. N1's rule corrected in all five places, N2's corpus header rewritten with the mirror
regenerated (file stays 53 lines), N3's four comments corrected. Gates: fmt 0, clippy 0,
`cargo test --workspace` 0, corpus 32 of 32.

The N1 error was mine twice over -- the narrow rule, and the inference from the zero-trip row
that a failing re-test is what decrements. It is whether a **body pass completed**. A
zero-trip loop's first test also fails, so that row could never separate the two hypotheses,
and it is the row both of us reasoned from.

### DEFERRED DECISION: does per-sub-phase ownership attribution earn its keep?

Raised by Moritz mid-4b, deliberately deferred to Task 12's gate (new Step 3b) rather than
acted on now.

**The measurement, at `e4caa7bf`:** `owners.rs` + `loud.rs` + `coverage.rs` = **1,852 lines**
against **9,025** lines of interpreter in `run.rs` + `eval.rs` + `error.rs`. Task 0 was 907
insertions with **zero interpreter functionality**.

**The finding: the split is sound, the seam's axis is not.** Phase 4 is split by *language
feature* while the code is organised by *mechanism*, so every sub-phase touches every file.
The seam therefore buys none of the isolation a split normally buys -- D5 already concluded
we cannot parallelise across it, because every scheduling collision in the 4a ledger was two
agents in one file. Full cost of the boundary, almost none of the benefit.

Sorting this phase's defects by cause is stark. **Attribution artifacts:** Task 0 entire;
`loud.rs` deleting the witness covering `Call::Qualified`/`Call::Trap`; ownership data in
three places with one hand-maintained in production code; four "witness implemented out from
under a test" occurrences; the `>I>`/`<I<` owner question, whose *existence* is an artifact;
the corpus subset union plumbing. **Would have happened anyway:** every indentation error
(40-column cap, the `+2` rule, F4, N1) -- those are the oracle being complicated -- plus the
missing no-`PROCEDURE` construct, the vacuous tests, and `cargo fmt --edition`.

So the split generates the bookkeeping defects and the oracle generates the semantic ones.
The semantic ones cost more to find and are not the split's fault.

**Why defer rather than fix now:** ten more 4b tasks will exercise the machinery and Task 3
alone (`CALL`, four forms across three owners) will say more than reasoning can; consolidating
`owners.rs` means editing the file every remaining task touches while they are landing, which
is D5's collision and would move every gate figure in between; and the third copy is
**guarded** rather than rotting -- Task 0's stderr assertion catches drift, verified by
mutation B.

4c is the last sub-phase that could act on the answer, which is why the gate step must produce
a **costed recommendation** rather than an observation.

## Task 3: the activation stack becomes real -- commit `733da737`

Implementer: t3-call (opus). DONE_WITH_CONCERNS. 879 passed / 0 failed / 4 ignored, fmt and
clippy 0, **corpus 33 of 33** (new witness `lang/call_return.rex`), assertion table unchanged.

**Two defects found only by running the compositions, both invisible to every test without
an `INTERPRET` in it:**

* `CALL` inside `INTERPRET` must **clear** `clause_line_override`, not merely not set it --
  the enclosing fragment has already set it, so leaving it prints every callee clause at the
  fragment's line. My brief said "must not set it", which was necessary and not sufficient.
* Label resolution must go against the running **activation's** body, not the body being
  stepped. Inside a fragment those differ and a fragment's label table is always empty, so
  every `CALL` inside an `INTERPRET` was unresolvable.

**`::routine` is NOT reachable in 4b, measured.** It is structurally present and `body_of`
resolves `Some(i)`, but a same-file `::routine` sits *behind* the builtin resolution step,
which is 4c's: `::routine max` alongside `call max 1,2` still calls the builtin. This settles
Task 9's `>I>`/`<I<` question -- they need `trace l` on a `::routine`, so they are not 4b's.
Task 9's step is rewritten accordingly, with one caveat added: Task 3's witness used a
**builtin name**, which is the one shape that cannot separate "behind the builtin step" from
"unreachable entirely", so Task 9 confirms with a non-builtin name before writing the row.

Three further measured facts about a `::routine` activation, recorded for 4c: its own
variable pool, builtins shadow it, and **`TRACE` does not cross into it at all**.

**Concerns accepted:** arguments are evaluated (observable -- `call sub 1/0` raises 42.3 at
the `CALL` clause, rc 214) but discarded, since `USE ARG` and `ARG()` are both still loud and
a stored field nothing reads is a `dead_code` error under `-D warnings`; Task 5 keeps the
`Vec<Option<..>>` intact. `Loud::missing_body` is unreachable today and is a `Loud` rather
than an `unreachable!`.

**Controller error corrected in the plan:** the Interfaces block promised later tasks
`Interp::depth`. The name was already taken by `eval`'s expression-recursion depth, and the
activation quantity is exactly `activations.len()` -- a second field could only disagree.
Later tasks read `activations.len()`.

**I34 confirmed with numbers.** The depth counter is decoration for deeply nested bodies:
native abort at 22,534 flat activations, 14,062 with one `DO` each, 5,616 with five, 1,403
with 25, on a 512 MiB thread in debug. At a 10,000 limit the counter fires first only for the
first two rows. A counter over activations cannot bound a budget shared with `run_bounded`.
Table now in `phase-4-exclusions.txt`.

**Process failure, mine.** I edited `run.rs` while t3-call was live in it, which is the
two-agents-one-file collision D5 records three times and which I have cited as the reason not
to parallelise. Caught by a "file modified on disk" notice. Reverting was impossible without
destroying in-progress work, so I warned the implementer which hunks were mine; they rode
along in `733da737` and the reviewer was told to ignore them. Cause: I treated documentation
fixes as not counting as editing.

Task 3 review dispatched (opus), BASE `ef6745c0`, HEAD `733da737`.

### Post-Task-3 corrections, commit `6ef3812d`

Task 3's implementer checked three of my claims instead of accepting them, and was right on
the one that mattered.

**DEVIATION 0 was documented and never implemented.** `bc665b40` changed two documentation
files and no harness code, so the differential still compares stderr byte-for-byte with
indentation included, while the row read as in force. The danger is inverted from a normal
gap: not something that fails loudly, but a later task *weakening* an indent expectation
because it believes a protection exists. The row's first line now says NOT YET IMPLEMENTED
and that every figure so far was obtained under exact comparison. Task 2b still owns landing
it. Caught by an implementer reading the harness rather than the row.

**On the collision I was half wrong.** `git log --oneline -S'construct-shaped rule' --
rust/crates/rexx-exec/src/run.rs` returns `733da737`, so my edit did ride along. Their
`git show --stat` check was sound for the question it asked -- did my *commits* touch the
file -- but could not see an uncommitted working-tree edit staged by them.

**The damage was not the one I assumed.** Nothing was clobbered. What broke is that I stopped
mid-edit when the warning fired, having corrected one of two sites, and `static_indent`'s doc
kept the scope claim `do label q ... end` had already falsified. **An interrupted correction
is worse than an unstarted one**, because the round reads as done. Fixed in `6ef3812d`; that
site now names the C++ mechanism like its sibling rather than enumerating constructs.

**A corruption mode with no local witness, recorded for anyone running mutation testing.**
The implementer made byte-for-byte backups of `run.rs` and restored them **eleven times**
during mutation testing. An edit landing in that window would have been silently reverted,
and their `cmp` against their own backup could never detect it -- the backup is the
comparand. It did not fire here for reasons of timing, not safety.

**Scheduling decision:** Task 2b is unblocked (Task 3 landed, and 2b touches `tests/corpus.rs`
and `trace_oracle.rs` while a Task 3 fix round would touch `src/`), but it is held until
t3-review returns. Low overlap is not no overlap, and the collision above came from exactly
this kind of low-risk reasoning.

### Task 3 review (opus): spec MET, quality good. 0 Critical, 1 Important, 10 Minor

Every claim re-measured independently and all held but one: shared pool in both directions,
`RESULT` dropped on return, argument evaluation observable, the clause-echo stack **with the
caller lexically nested**, both compositions including the two failure modes, two rows of the
depth table, all eleven mutations including the brief's own trap, the new witness genuinely
compared, `loud.rs`'s split-arm rows intact, and the `activations.len()` substitution ruled
right. Vacuity audit found nothing.

**F1, the one Important: the `::routine` reachability claim is wrong as recorded.** Verified
here -- `call zorkolo` with a `::routine zorkolo` runs on the oracle at rc 0, where we exit
120 loud. So a `::routine` **is** reachable in 4b for any non-builtin name.

The measurement was right and the inference was not. The `max` witness is the one shape where
"behind the builtin step" and "unreachable entirely" predict identical bytes -- the same
probe-selection trap that produced the `>I>`/`<I<` error earlier in this phase, where absence
under one instrument was read as ownership. I flagged exactly this caveat when amending Task 9
and the implementer had already shipped the claim; the review is what closed it.

Not implementing `::routine` in 4b stays correct: it costs a clean loud failure, and
builtin-colliding names need 4c's table, where a wrong answer would silently run the wrong
routine rather than fail loudly. **Only the stated reason changes** -- four sites in the crate,
dispatched to t3-call as fix round 1, plus Task 9's step which I corrected here.

Consequence for Task 9: the two prefixes are out of scope **by decision**, not by
unreachability, and its row must say which.

### Task 3 fix round 1: commit `e68d6a6c`. Task 3: complete.

Comment-only, verified here: 0 non-comment changed lines, the corrected claim present at all
four sites, "cannot be reached" gone from the crate, tree clean, 879 / 33 of 33 / 4224 of
4259 unmoved.

Commits: `733da737`, `e68d6a6c`.

**A NEW CAUSE for the "cannot happen" family, and it is not bad probe choice.** The
implementer reported that the falsifying evidence was already in their own transcripts:
probes `ro1.rex` and `ro2.rex` both used `::routine foo`, a non-builtin name, and both
dispatched on the oracle -- **earlier in the same session than the `max` probe**. So they had
the discriminating data first, ran a narrower probe afterwards, and wrote the conclusion from
the narrower one.

Every previous instance in this phase was "the instrument could not have produced the answer".
This one is different: **a later, narrower measurement overwrote an earlier, wider one, and
neither was ever re-read against the other.** No probe-selection rule prevents it. What
prevents it is re-reading your own earlier measurements before writing a conclusion, which is
not a habit anything in this process currently enforces.

Self-reported rather than found by review, which is the second time an implementer has caught
their own reasoning defect before anyone else saw it.

## Task 2b: DEVIATION 0 implemented -- commit `0830a12e`

Implementer: t2b-normalise (sonnet). DONE. 891 passed / 0 failed / 4 ignored, fmt and clippy
0, **corpus 33 of 33 and assertions 4224 of 4259, both unchanged**.

**Zero figures moved, and reported as zero.** The corpus was already byte-exact at 33 of 33
before this landed, so there was nothing left for stderr normalisation to newly fix. Stated
plainly in the report rather than presented as an achievement, which is what the brief asked
for and the honest answer.

The +12 on the suite count is six shared `support::tests` compiling into two test binaries.

`tests/support/mod.rs` holds `normalize_stderr`, used by `corpus.rs`'s `check_case` and
`trace_oracle.rs`'s `check_witness` at their stderr comparisons only -- never on `.expected`
regeneration, which stays raw oracle bytes. It requires a known 3-byte marker at a fixed
offset (7) or passes the line through untouched, collapses only the space run *after* the
marker to one space, stops at the first non-space so a quoted value's own leading spaces
survive, and guards `space_run == 0` so a malformed line cannot grow a space.

Three negative controls (missing line, reordered line, changed value), two positive controls,
and a non-trace-line control.

**Pinned unnormalised witnesses named rather than duplicated**, since a unit test's own
`assert_eq!` is outside either harness's comparison function:
`one_two_and_three_enclosing_dos_indent_by_two_four_and_six`,
`the_corrected_28x_indent_rule_matches_all_fourteen_probed_shapes`,
`an_absorbed_whencases_escaping_false_branch_reports_end_at_its_own_residual_indent`. The
weak fourth (`the_indent_after_a_loop_has_already_exited_is_not_left_over_from_it`) is
explicitly excluded in both files with the reason.

**Controller ruling on the flagged third file.** `tests/support/mod.rs` was not in the
brief's Files list. Upheld: duplicating the function in both harnesses would recreate the
"one quantity, two formatters" drift this project already paid for -- the same defect Task 0
was created to remove for owner data, and the one `trace.rs`'s module doc records for the
clause echo. Obeying the Files list here would have followed the letter of the brief against
its evident purpose.

Review dispatched (opus), deliberately hostile: this task **weakens the project's own gate**,
so the whole question is what it hides beyond what DEVIATION 0 licensed. Anything extra is a
defect that is never caught again, silently, for the rest of the project.

### Task 2b review (opus) and fix round 1: `344677e6`. Task 2b: complete.

0 Critical, 2 Important, 6 Minor. The reviewer proved the normalisation does **not** hide
missing, extra or reordered lines, changed values, changed clause text, changed line numbers,
truncated markers, `\r\n`, or non-UTF-8; confirmed the marker set complete against `trace.rs`;
and confirmed `.expected` regeneration is untouched. The third-file decision was upheld on
both substance and mechanism.

**I1, the one class it did hide.** `normalize_stderr` split on `\n` and asked per physical
line whether bytes 7..10 are a known marker -- which cannot distinguish a trace line from the
**tail of a quoted value containing a newline**. Verified here: `trace i` with
`x = '0a'x || "       >>>   z"` produces three stderr lines of the form `       >>>   z"`
that are string data, not trace lines, and the normalisation ate their spaces. A pure
value-content divergence therefore compared equal -- outside what DEVIATION 0 licensed, which
names value content as byte-exact. Latent (no corpus program embeds a newline in traced text)
but permanent and silent.

Fixed by carrying one bit of state: only a recognised trace line whose own `"` count is odd
opens a continuation region, and lines inside it are copied through untouched. `is_trace_line`
was factored out so the tracking and the normalisation share one marker check rather than two
copies -- the same duplication defect this phase has paid for twice. The residual limitation
is documented honestly and is **stricter, never looser**: a value containing a literal quote
can look unterminated, and the only consequence is a following genuine trace line being left
un-normalised, which cannot hide a divergence.

Negative control 4 uses the **real measured oracle bytes**, mutates one continuation line's
space run, asserts `normalize_stderr` still distinguishes them, and carries a sanity assertion
that the raw bytes differ. Verified to fail against the pre-fix function by reverting and
re-running.

**I2 was mine.** I wrote "precisely DEVIATION 0's own oracle-counter shape" into the plan
amendment about `one_two_and_three_enclosing_dos_indent_by_two_four_and_six`, and it
propagated into the deviation row, `corpus.rs`'s module doc and the report. False and
self-contradictory: that test raises on the **first** iteration, so no pass completes and no
loop ends -- the exact condition the counter defect needs. It is a valid pinned indent witness
and not a witness for the counter gap. All three places now say plainly that **no pinned
witness demonstrates the counter defect**, which is the honest state.

893 passed / 0 failed / 4 ignored; corpus 33 of 33 and assertions 4224 of 4259 both unchanged.
Commits `0830a12e`, `344677e6`.

## Task 4: ExprKind::Call, the expression form -- commit `c3db2bcc`

Implementer: t4-exprcall (sonnet). DONE, no outstanding concerns. 899 passed / 0 failed /
4 ignored, **corpus 34 of 34** (+1 witness `lang/call_expression.rex`, no existing program
changed), assertions 4224 of 4259 unchanged, fmt and clippy 0.

Scope beyond the brief's Files list, each argued in the report:

* `run.rs` -- `exec_call` split into a shared `pub(crate) resolve_and_run_call` plus a thin
  `exec_call`, claimed behaviour-preserving on the strength of `exec_call`'s existing ~20-test
  suite passing unchanged either side.
* `error.rs` / `lib.rs` -- **a new `Failure::Exited(Option<ObjRef>)`**, because `eval`'s
  return type has no `Flow` to carry an `EXIT` out of a routine reached through the
  expression form. Real design work: a control-flow outcome routed through an error type,
  creating a second representation of what `Flow::Exit` already represents.
* `tests/{loud,coverage,owners}.rs` -- ownership bookkeeping; `ExprKind::Call` moved **fully**
  in scope, its loud witness deleted rather than split, since `CallTarget`'s two forms are
  both 4b's and no arm survives.
* `phase-4-exclusions.txt`'s EXPRKIND OWNERSHIP section.

**A measurement trap specific to this task, flagged to the reviewer.** The implementer reports
`44.1` for a routine returning no value in expression form. That is plausibly right, and it is
also **exactly** the error my contaminated probe produced this morning when a stale `f.rex` on
the oracle's external-routine search path was found and executed. Same text, different cause,
and this is the one construct where contamination and the correct answer are indistinguishable
by error text alone. The reviewer must confirm it from a fresh empty directory with a real
internal label.

**Concurrency discipline held.** My `fd9a97ac` (plan `.md` only) landed while t4 was in the
crate; the implementer checked for overlap, found none, and said so. That is the rule I broke
during Task 3 and have kept since.

Task 4 review dispatched (opus), BASE `fd9a97ac`, HEAD `c3db2bcc`, weighted toward
`Failure::Exited` and the 44.1 re-measurement.

### Task 4 review (opus) and two fix rounds. Task 4: complete.

**1 Critical, 2 Important, 5 Minor.** `Failure::Exited` and the `exec_call` split both ruled
sound. Commits `c3db2bcc`, `068373c5`, `994a92b7`. 901 passed / 0 failed / 4 ignored, corpus
34 of 34, assertions unchanged, fmt and clippy 0.

**C1, the phase's first Critical.** `resolve_and_run_call` saved and restored
`activation_indent`, `indent_offset` and `clause_line_override` but **not**
`current_value_indent`, which `run_activation` overwrites on every callee clause. Before
Task 4 at most one activation could be entered per clause; `say f(1) + g(2)` enters two, and
the second computed its base from the first callee's last clause. **Not trace-only** -- with
no `TRACE` in the program, an ordinary error report read `4 *-*   say 1/0` on the oracle
against our `4 *-*     say 1/0`. Verified here, and verified byte-identical after the
one-line fix. The regression test was confirmed to fail without it.

**DEVIATION 0 hid it, and that narrows the deviation's justification.** I measured that
normalising collapses exactly this difference, so the corpus reported 34 of 34 with the bug
live. The row's argument -- indentation carries no independent information because the clause
sequence proves it -- holds for the ORACLE's counter defect and **not** for our own state
management, and the harness cannot tell them apart. `ffb21c2f` narrows the justification and
records that the pinned unnormalised unit tests are the only instrument catching this class.
Not a reversal: reproducing the C++ counter defect is still the wrong target.

**A ruling worth keeping.** The implementer left `call_return.rex`'s header carrying the same
false indent-pinning claim they had just fixed in three other places, on the grounds that it
was a Task-3-owned file. Flagging rather than acting was right; the boundary was wrong.
Overruled, because at least two false statements in this phase survived by exactly that route
-- someone corrected one instance of a claim and left its twin. **But reading the header
before ruling changed the fix**: lines 10-14 tangled the false claim with a still-true
constraint (the `DO` block is deliberately plain, which the raw-byte unit tests depend on),
and an unqualified "correct the false claim" would have taken the true one with it.

### Open question recorded for Task 7

The `INTERPRET` arm (`run.rs:1005-1011`) has the **same omission** C1 fixed in the call path:
it saves and restores three pieces of level state and not `current_value_indent`. Reachability
is **unknown**. Two fragments cannot run in one clause at instruction level, and a call inside
a fragment is covered by the call path's own restore -- but "unobservable because at most one
per clause" is precisely the reasoning that was true for the call path until Task 4 falsified
it, and a trap resuming mid-clause is the most plausible next breaker. Task 7's brief now
requires measuring it and reporting what was tried either way.

## Task 5: PROCEDURE, EXPOSE, USE ARG, USE LOCAL -- commits `8b87195b`, `d3413cbd`, `50ba7aa1`, `92e80218`

Implementer: t5-procedure (opus). 2,079 insertions across 16 files in the first commit.
Gates at `92e80218`: suite 0, **corpus 36 of 36**, strict corpus gate 0, fmt 0, clippy 0,
tree clean.

**Both disagreements with my brief were upheld by review.**

* The redirect design: the brief framed a choice between an `Interp` route and a `RootSet`
  route. The `Interp` route **is not implementable as described** -- `Interp::read` resolves
  through `code.slots` and calls `roots.slot(frame, slot)` directly, reaching `slot_of` only
  on a fallback miss, so a pair-returning `slot_of` would not cover the hot read at all and
  the route would mean hand-writing the alias check at 15 sites. My 12-to-26 re-costing was
  measuring the wrong thing entirely.
* Step 2 was not built and `plan.rs` is unchanged. My premise -- "`PROCEDURE` must be a
  routine's first instruction, so the list is a property of the body" -- is true of the
  grammar and false of execution: the same `sub: procedure` runs when called and raises 17.1
  when fallen into, so no property of the body's text can decide it.

**Review: 1 Critical, 1 Important, 3 Minor.** C1 was `USE ARG >name` missing the oracle's
98.995 refusal and giving **silent wrong answers** -- five measured programs at rc 0 with
wrong output where the oracle raises at rc 158. The reviewer characterised the rule by
measurement and found the pair that makes the fix correct: exposed **and holding a value**
raises, exposed **and unset** succeeds, so the trigger is the value, not the exposure --
despite the message's wording pointing at locality. Without that pair the fix would plausibly
have been an exposure check.

**Fix round 2, found by a malformed probe of mine.** Passing a simple-variable reference into
a stem target, the oracle refuses with 88.929 and we accepted it silently. Measuring the
matrix then turned up 88.930 as well. Three refusals in that arm: 88.928 was implemented,
the two **kind-mismatch** ones were not, and the review missed them because it was checking
the *initialisation* rule the Critical was about, not the *kind* rule beside it. Both now
enforced and verified byte-identical.

**Two errors of mine during verification, recorded because the ledger is worth more honest.**
My comparison loop redirected to bare `r.out` inside a subshell that had `cd`'d into `rust/`,
so `cmp` compared the oracle against a stale scratchpad file: three false DIFFER results
against a correct implementation. The same bug wrote two scratch files into the repository
working tree, which my own global constraint forbids; removed, tree clean. An apparatus that
is wrong produces false positives as readily as false negatives, and mine was not checked
before I acted on it.

Task 5: complete pending its fix-round-2 report, which the implementer has not sent.

## Task 6: SIGNAL to a label, and SIGNAL VALUE -- commit `53b75f90`

Implementer: t6-signal (sonnet). DONE, no blocking concerns. Suite green, fmt and clippy 0,
**corpus 37 of 37** confirmed in both REPORT and `REXX_CORPUS_GATE=1` STRICT mode (new witness
`lang/signal_forms.rex`). Assertions unchanged at 4224 of 4259 -- `base/expressions` does not
exercise `SIGNAL`.

**`Flow` needed a new variant, `Flow::Signal(usize)`, and the argument is worth keeping.**
`run_fragment` steps a fragment through `run_bounded` in the **fragment's own index space**,
which is separate from the enclosing body a `SIGNAL` target resolves against. Both index
spaces start at 0, so a bare `Goto` escaping a fragment would be range-checked against the
wrong body and **a coincidental in-range match is the likely case rather than an edge case**
-- a silent wrong jump, the outcome class this project ranks worst.

`DO`/`LOOP`/`IF` nesting needs no equivalent care, and that was measured rather than assumed:
`rexx-parse` already rejects a label written inside either (47.2/47.3), so the risk does not
arise there.

`Flow::Signal` is forwarded untouched by every `run_bounded` / `do_body_outcome` /
`leave_select` / `run_fragment` catch-all and consumed only by `run_activation`'s top-level
dispatch, the same shape as `Goto`.

**A judgement call sent to review rather than settled in-task.** The pre-existing disclosed
Controlled `DO`/`LOOP` retrace gap (`loop_advance`, I31's family) bit the implementer's first
hand-transcribed unit test. They worked around it by choosing a witness that fires `SIGNAL` on
the loop's **first** pass rather than its second, for both the unit test and the corpus witness,
**without touching loop code**. Not touching it was right. Whether choosing a witness that
avoids a known divergence is sound scoping or quietly routes around something this task should
have surfaced looks identical from inside the task, so the reviewer rules.

Also corrected, unprompted: a stale entry-count doc comment in `loud.rs` left over from Task 5
moving `Procedure`/`Use` in scope -- in the exact file this task's own edit touched.

Task 6 review dispatched (opus), BASE `375b662d`, HEAD `53b75f90`. First review told to read
`rust/CLAUDE.md` rather than have the constraints pasted at it.

### Task 6 review (opus) and two fix rounds. Task 6: complete.

Review: spec **NOT MET**, quality **NEEDS WORK** -- 1 Critical, 3 Important, 4 Minor.
Commits `53b75f90`, `837bbf0f`, `0fce4f00`. Final gates verified here: suite 0, fmt 0,
clippy 0, **corpus 37 of 37 in STRICT mode**, tree clean.

**C1: `SIGNAL` did not set `SIGL`, and neither did `CALL`.** Measured: `signal there` gave
oracle 2 against our `SIGL`; `call sub` gave oracle 1 against our `SIGL`. A **silent wrong
answer** -- and `corpus.rs:74` asserts "Every mismatch today is a *clean loud failure*:
nothing produces a wrong answer", which this made false. `SIGL` appeared nowhere in the
exclusions file, either corpus subset, or any source comment. Ruled implement rather than
disclose: the root was shared with Task 3, Task 7 would have inherited a third instance since
a trapped condition also sets `SIGL`, and weakening the harness's own claim is worse than
fixing what falsifies it.

The implementer read `RexxActivation::signalTo` to explain the fragment case rather than
stopping at the transcript: an interpret-created activation delegates `SIGNAL` to its parent
instead of setting `SIGL` itself, so `SIGL` ends up holding the parent's currently-executing
instruction -- the `INTERPRET` clause. Our crate reproduces that observable answer through
`clause_line_override` without adopting nested activations for `INTERPRET`.

**Then the fix shipped the same defect on its own new field.** `current_clause_line` was
cached per-clause and not restored across a nested activation, so `say f(1) + g(2)` read the
first callee's last line: oracle `sigl in g: 1`, ours `5`. **Third instance of one shape** --
`current_value_indent` (Task 4's Critical), the `INTERPRET` arm's copy of it, now this. The
implementer had modelled the new field on `current_value_indent` explicitly and copied the
unconditional caching without the restoring.

**I had put that warning in their original dispatch, in those words, with those line numbers,
and it shipped anyway.** That is evidence the control was wrong, not the person: prose in a
dispatch does not prevent this class.

**The fix is now structural.** `current_value_indent` and `current_clause_line` are bundled
into one `Copy` struct, `ClauseState`; `resolve_and_run_call` saves and restores it as a
single struct copy, so a third field of this shape is restored the moment it joins the struct
-- no second edit anywhere. ~65 call sites were a mechanical access-path rename with no field
renamed, so existing doc comments stayed accurate. This is the answer to "mechanical
enumeration over a hand-maintained list", and it is type-level rather than discipline-level.

**The `INTERPRET`-arm question I recorded as unknown for Task 7 is now answered**: three
programs byte-for-byte, zero diff, including two calls inside one fragment clause and a call
whose callee uses `INTERPRET`. The arm never creates the "two things share one clause" shape.
Task 7's brief is updated to record the answer and to say that any new per-clause state goes
in `ClauseState`. What remains open for Task 7 is the one route neither Task 4 nor Task 6
could test: a condition trap resuming mid-clause.

**Two rulings went the implementer's way, both verified rather than accepted.** The reviewer
collapsed `Flow::Signal` into `Goto` and produced the predicted wrong jump. And it ran the
original loop witness end to end to confirm the known Controlled-`DO` retrace gap was not
concealing a `SIGNAL` defect -- the stderr diff was only the two known lines.

The implementer also declined to add one requested test, with reasons: the self-referential
fragment shape does not fail under a `Goto` collapse, it **hangs**, because absorption at or
before the `SIGNAL`'s own position reproduces the identical `Goto` every pass through an
uncapped loop. A test that can hang CI on a future regression is worse than one fewer test.
Accepted, and documented in the code rather than dropped silently.

## RESUME POINT -- session compacted 2026-08-03

HEAD `7ec66f84`, tree clean. Tasks 0, 1, 2, 2b, 3, 4, 5, 6 complete.
Corpus **37 of 37** byte-identical in STRICT mode; assertions 4224 of 4259; suite, fmt and
clippy all green at Task 6's close (`0fce4f00`).

**Task 7 (conditions, `RAISE`, `NOVALUE`) is RUNNING** as agent `t7-conditions` (opus),
dispatched from brief `task-7-brief.md` at baseline `7ec66f84`. It owes a report at
`task-7-report.md` and answers to three inherited questions:

1. Whether a condition trap resuming mid-clause creates two activations in one clause -- the one
   route Tasks 4 and 6 could not test, both of which shipped a defect of that shape.
2. Re-verification of the temps-frame leak analysis against a **real** trap, with the program
   used. "It still holds" without a program is not a verification.
3. What `SIGL` is set to on a trap, measured rather than assumed to match `SIGNAL`'s value.

When it reports: generate the review package from BASE `7ec66f84`, dispatch a review on opus,
and weight it toward the trap/`ClauseState` interaction and the trap-test vacuity hazard (a test
asserting the program exited 0 is satisfied by a program that never raised).

**Then, in order:** Task 8 (`PUSH`/`QUEUE`, ships with zero differential coverage by design and
must say so), Task 9 (trace prefixes `>A>`/`>F>`/`>R>`, and I31's two missing `>>>` lines),
Task 10 (4b corpus and the collector), Task 11 (`base/keyword` L1 with absolute floors, not just
the conservation law), Task 12 (the gate, including Step 3b's costed recommendation on whether
per-sub-phase ownership attribution earns its keep).

**Run the pre-dispatch check before every task.** It has found a missing prerequisite for every
task it has been applied to, and the categories keep changing -- a corpus subset file created
nine tasks too late, a pinned literal needing a paired edit, two formatters where the plan named
one, a mechanism the plan told an implementer to reuse that meant something else, a stale cost
input under a decision not yet made, and two stale doc comments in the area a task must modify.

**Two decisions still open for Moritz**, neither blocking: whether to file the upstream
`traceIndent` defect (`BaseDoInstruction.cpp:161` against `:377`), and the ownership-attribution
question deferred to Task 12's gate with its measurement attached (1,852 lines of bookkeeping
against 9,025 of interpreter).

---

## Task 7: implementer reported (2026-08-03 21:36)

**Status: DONE_WITH_CONCERNS.** Commit `f906aabc`, parent `7ec66f84`. Report:
`task-7-report.md` (2,864 lines, 85 oracle transcripts in Appendix A).

**The hash moved once.** The implementer amended: `e2971db4` no longer exists, `f906aabc` is the
hash to record. The amend corrected two statements in the report that said eleven trap tests
"went green" against the drifted harness when they went **red** -- which is how the harness
defect was found. Unpushed, tip of branch, no force-push involved.

**Controller re-verified every gate independently** rather than taking the report's word, and
the numbers match it exactly: `cargo test --workspace` 956 passed / 0 failed; STRICT corpus
`38 of 38`; assertions `4224 of 4259` with 35 RUNTIME-BLOCKED (all Phase 5 message-sends and
`XRANGE`, the expected set); `cargo fmt --all --check` exit 0; clippy `-D warnings` exit 0.
Corpus 37 -> 38, the new witness being `lang/condition_traps.rex`.

**The three owed answers, all resolved with programs:**

1. *Two activations in one clause.* Real and constructible, and **we match**. A `CALL ON`
   handler cannot produce the shape (measured: the assignment has already stored the routine's
   value before the handler runs). A `SIGNAL ON` handler that `RETURN`s from a mid-expression
   callee does: `zz = one(1) two(2)` gives `FROMHANDLER SIGLIS 2` on both sides, where a leak
   would have read 6, 9, 10 or 11. `ClauseState`'s whole-struct restore already covered it and
   **no field was added** -- the type-level fix paid off exactly as intended.
2. *Temps frames.* Holds, measured at 200 and 400 trapped-and-resumed cycles with equal
   `roots.temps_len()`; the raise is inside a parenthesised expression on purpose, so `eval`'s
   own frame-opening sites are on the path. Injecting a `push_temp` into `offer_to_trap` turns
   it red.
3. *`SIGL` on a trap.* One rule covers traps and `SIGNAL` both: **the current clause line of the
   activation in which the transfer happens** -- which is not always the raising clause, because
   the trap that fires is not always in the activation that raised. Pinned at three points
   (inherited trap in a `PROCEDURE`d callee -> callee's line; callee does `signal off syntax`
   -> caller's line; raise inside `INTERPRET` -> the enclosing clause).

**One defect of the recurring family was found and fixed** -- one level over from
`ClauseState`. A pending `CALL ON` condition was delivered at the next clause boundary reached
by *any* activation, so in `zz = one(1) two(2)` the handler ran inside `two`. `PendingTrap` now
carries the raising activation's depth and delivery waits for it.

**A test-harness defect worth carrying forward.** `run.rs`'s test module used a hand-rolled
`run_activated` that reproduced `run_activation`'s dispatch arm by arm, and the trap offer lives
in `run_activation`'s loop -- so **eleven trap tests ran against a harness that could not trap.**
They went red rather than green, so that batch was never vacuous, but a test asserting "this
condition is fatal" or checking an exit code would have passed for the wrong reason. Now
`run_activated` *is* `interp.run_activation().map(Ended::value)`.

**Six defeat-the-mechanism mutations, one survivor, and the survivor was real.** The re-arm test
inserted over a live entry, which is indistinguishable from inserting over an absent one, so the
removal was never exercised. Replaced by `the_trap_that_fired_is_removed_from_the_table`, which
asserts the fired entry is gone **and** an unrelated one is still there -- without the second
half, clearing the whole table satisfies it.

**Task 7: complete.**

### Carried forward from Task 7's concerns

* **For Task 9 (trace prefixes / I31):** `EXIT <expr>` emits no `>>>` value line under `trace r`.
  Pre-existing, reproduced on a three-line program with no conditions in it, so it belongs to
  `InstructionKind::Exit`'s arm and not to Task 7. **It constrains corpus programs**: any
  `trace r` witness containing `exit <value>` will diverge, which is why `condition_traps.rex`
  has none.
* **For Task 10 (corpus):** `rust/corpus/lang/condition_syntax.rex` exists in the directory but
  is listed in no subset.
* **For 4c:** `condition()` stays loud, so no corpus program can read a trapped condition's name;
  `Raised::condition` and `Trap::call` carry values that have no differential witness yet.
* **Minor (deferred), to be triaged by the final whole-branch review:** `RAISE PROPAGATE`'s
  `active_condition` is never cleared, so a `PROPAGATE` reached after a handler has finished
  re-raises that handler's condition (unmeasured either way); non-`SYNTAX` `RAISE ... RETURN`
  searches exactly one level out rather than outward, so a grandparent trap is missed
  (unmeasured); and `RAISE PROPAGATE`'s report rendering rests on four transcripts rather than a
  family. All three are stated in the doc comments of the functions that own them.

### Task 7 review (opus): spec FAILED, quality not ready to close

`task-7-review.md`, 501 lines. The reviewer **ran** nearly everything -- its own oracle probes plus
eleven mutations of its own, on top of independently re-running all six of the implementer's.
All six reproduced exactly, survivor included. Tracked tree left clean, every mutation restored.

**It cleared one concern.** Concern 2 is **not** a residual: probe `pb` shows the oracle *also*
declines to propagate to a grandparent, so `exec_raise`'s `Search::Caller` looking exactly one
level out is measured-correct. The doc comment calling it a residual is now false.

**Critical.** `run.rs:692-696` / `:738-742`: `Flow::Return` and `Flow::Exit` return before the
`pending_trap` check, so a pending `CALL ON` condition is **dropped** (oracle `HANDLER-AT 7`,
ours `NOMARK`) or **delivered into a later unrelated activation at the same depth** (oracle
`SIGL 8`, ours `SIGL 11`). Four oracle DIFFs. **This is the same family as the defect
`PendingTrap::depth` was added to fix, one turn further in -- depth equality does not identify an
activation.** Fix dispatched as a mechanism change, not a two-call-site patch.

**Important.** (2) `deliver_pending_trap` never clears `active_condition`: oracle `98.941` rc 158
against our silence at rc 0. Concern 1 called this unmeasured; one probe settled it, and `pf`
shows the `SIGNAL ON` case must *not* clear, so the fix is the `Ended::Returned` arm only.
(3) Two implemented, documented behaviours have **no test anywhere**, including the STRICT gate:
re-arming a `CALL ON` trap after its handler returns, and `exec_raise_propagate`'s `reportable()`
guard -- 9 of the reviewer's 11 mutations were killed, these 2 survived. (4) `run_source`'s doc
comment is false about `slots`. (5) **`RAISE SYNTAX` does not validate its argument, violating
the Global Constraint that unimplemented things fail loudly outside 157..253**: measured rc 216,
rc 25, and rc **0** where the oracle gives 98.941/158 and 33.904/223. rc 0 on a raise is a silent
wrong answer. (6) `Delivery`'s doc names one writer, there are three. (7) `Interp::pending_trap`'s
doc asserts the very property finding 1 breaks.

**Minor (deferred), for the final whole-branch review to triage:** `run_activated`'s `_program`
parameter is dead; `trap_for` is `pub(crate)` with no cross-module caller.

**Cannot verify from diff, resolved by the controller:** whether new allocations go through
`Interp::alloc_with` (they use `self.text(...)`); Appendix A's 85 transcripts as a set; the claim
that eleven trap tests went red against the now-deleted harness.

**Fix round 1 dispatched** to the original implementer at `f906aabc`, with instructions not to
amend that commit (the review references it), to fix finding 1 at the mechanism rather than the
two call sites, and to pin findings 1/2/3/5 with tests checked by deleting the line each protects.

### Task 7 fix round 1: `431e2698`, then re-review (opus)

**Round 1 went past the finding, in the useful direction.** The Critical was **two** defects:
*placement* (the check now runs immediately after the clause finishes, before the `Flow` dispatch)
and *identity* (`PendingTrap` named its target by `activations.len()`; a depth is unique only
while its activation is live, so `Activation` now carries a never-reused `ActivationId`).

**The implementer found a third shape neither the review nor the controller reached** (probe
`ps`): a pending condition whose activation is unwound by an error the *caller* traps, after
which the caller calls something else and lands at the dead activation's depth. Placement cannot
fix it -- that activation never reaches another clause boundary -- so it is what proves identity
necessary. Mutation table: reverting placement kills two tests, degrading identity to a depth
kills only the third.

**Finding 5 came back as measurement rather than compliance.** Asked for loud failure, the
implementer derived the oracle's actual rule from 19 programs: major must be 1..=99 else
`33.904`; the sub is the digits after the point as a plain integer; an unknown pair is `98.941`
with `&1` = `major * 1000 + sub`, except where the catalogue has no `(major, 0)` entry at all.
Worth more than a loud exit. **Concern 2 formally withdrawn** (`pb` re-run, MATCH).

Controller-verified at `431e2698`: 964 passed / 0 failed (8 new tests), STRICT `38 of 38`,
assertions 4224/4259, fmt 0, clippy 0.

**Re-review (83 tool uses, everything run) confirmed the instruments**: the mutation table
reproduces exactly, survival included; all eight new tests die to a targeted mutation; the
rooting `push_temp` is load-bearing on **both** arms (deleting it panics `a live value` under
collect-on-every-allocation; keeping it passes with 70-79 real collections); no retention
introduced (200k-iteration RSS identical); **`ActivationId` coverage is type-level** -- both
constructors require it, no other struct literal, no `Clone`, three stack mutations; and the
finding-5 exception clause is confirmed structurally against `Interpreter::messageNumber` and the
two `98.941` call sites in `Activity.cpp`.

**Closed: 1, 3, 4, 6. Partially closed: 2, 5, 7. Five new findings, all RAN.**

* **NEW 1 (Important).** `deliver_pending_trap` sets `active_condition = None` where the oracle
  **restores the enclosing one**: oracle `42.3` rc 214, ours `98.918` rc 158. Save/restore
  matches the oracle in all three measured shapes.
* **NEW 2 (Important).** The C++ rejects `sub >= 1000`; we do not. `40.1000`/`40.1001`/`40.99999`
  are `33.904` rc 223 on the oracle, `98.941` rc 158 here -- and the `u32` comment asserts the
  opposite for `40.99999` **by name**.
* **NEW 5 (Important).** **The central comment of round 1's own fix is false.** "No path at all
  between the clause finishing and this check" -- but clauses inside `DO`, `SELECT` and
  `INTERPRET` bodies run through `run_bounded`, which has **no check**. `do2`, `sel1`, `int1` all
  DIFF in timing and `SIGL`. The behaviour is **pre-existing and identical on `f906aabc`**; the
  assertion is what round 1 added.
* **NEW 3, NEW 4 (Minor).** "Majors 1 and 2 are the only ones without a `(major, 0)` entry" is
  false in two comments -- there are **45**, and the code is right because it does the lookup.
  The `RAISE SYNTAX` argument is parsed with Rust integer parsing rather than Rexx `numberValue`
  (`'4E1'`, `'40.1E2'`, `'40.'` DIFF).

**Fix round 2 dispatched**, weighted to NEW 5: fix it at the mechanism if condition delivery in
loop and `SELECT` bodies is Task 7's, or name the owning task with the probe -- but either way
end with a comment that states a property the code actually has.

### Task 7 fix round 2: `26da3ac6`

Controller-verified: **968 passed / 0 failed**, STRICT `38 of 38`, assertions 4224/4259, fmt 0,
clippy 0. Lib tests 285 -> 289. All five findings reproduced before being fixed; 19 of 19 new
probes MATCH after.

**The implementer refused the re-review's ownership finding, with evidence, and was right.** The
re-review found NEW 5's behaviour identical on `f906aabc` and concluded "pre-existing". The
answer: **`f906aabc` is Task 7.** `git show 7ec66f84:...lib.rs | grep -c pending_trap` is **0**;
at `f906aabc` it is 2. The whole pending-delivery mechanism arrived with this task, so
"pre-existing on `f906aabc`" means "present since Task 7 shipped". It fixed it rather than
reassigning it. **Third time this task that the person carrying out the work corrected the person
checking it.**

**NEW 5 fixed at the mechanism.** The delivery rule now lives in one function,
`Interp::clause_boundary`, called from exactly the two places that step a clause -- and the claim
is offered in checkable form: `grep -n "step_in_temps_frame("` outside its own definition returns
`run_activation`'s loop and `run_bounded`'s loop, nothing else. `run_bounded` is where a `DO`
body, a `WHEN`/`THEN` body and an `INTERPRET` fragment execute, so a rule applied only in
`run_activation` held for none of them. `do2`/`sel1`/`int1` all MATCH now. The rooting
`push_temp` moved into `clause_boundary` with the rule, so both callers get it.

**NEW 1: restore, not clear.** One clause can queue a `CALL ON` condition *and* raise a
`SIGNAL ON`-trapped one (`zq = sub() + 1/0`), so a call handler can be delivered while a signal
handler runs; round 1's `= None` wiped the condition the enclosing handler was running for.
Progression across rounds: oracle `42.3` rc 214; round 0 silence rc 0; round 1 `98.918` rc 158;
**round 2 `42.3` rc 214**. The adjacent-success case is what stops this being a disguised "never
clear": two tests still give `98.918` when nothing was active.

**NEW 2 / NEW 4:** sub bounded `0..=999`, each half through `Number::parse` + `whole_value` (the
oracle's `numberValue`, not `str::parse::<i64>`); a decimal point with an empty tail rejected.
The `u32` comment that asserted the opposite for `40.99999` **by name** is gone rather than
hedged.

**NEW 3:** the real count is **45**, not 2 (1, 2, 12, 32, 50-87, 94, 95, 96). Code was always
right -- it does the lookup. The implementer named it for what it was: *"a measured-sounding
claim I did not measure, in the round whose whole subject was measuring. Two probes agreeing is
not a count."*

**New concern:** the `Err` arm of `deliver_pending_trap`'s restore is unmeasured -- a `CALL ON`
handler that fails under an outer trap. Stated at the code, and handed to the round-2 re-review
as the highest-value new measurement available.

**Round-2 re-review dispatched** at `431e2698..26da3ac6`, weighted to the `clause_boundary`
extraction (it touched `run_bounded`, which every `DO` body runs through, so a regression there
would be broad and quiet), the two-caller claim, the moved rooting, and that unmeasured `Err` arm.

### Round-2 re-review: four closed, and the exhaustiveness claim fails a third time

`task-7-rereview2.md`, 90 tool uses, everything run. NEW 1, 2, 3, 4 **closed**, each by
independent check rather than re-reading: `ac1`/`ac2`/`ac3` MATCH at `42.3` rc 214 and all three
DIFF on `431e2698`; a 45-argument edge sweep 45/45 MATCH with 13 DIFF on round 1; the 45 majors
**counted from the generated catalogue**, confirming `1, 2, 12, 32, 50-87, 94, 95, 96`. Also
verified independently: the two-caller grep, the moved `push_temp` load-bearing at **both**
callers with a negative control (all four probes panic `a live value` when deleted), and that
round 1's `pa2`/`pf`/`lv1`/`ps` still MATCH.

**NEW-A (Important): a `DO` header is a clause, and its boundary lives inside `run_loop`.** `g1`
(`do i = 1 to sub()`) and `g4` (`do while ... sub()`) still deliver at the wrong clause and the
wrong `SIGL`, and three comments assert the "exactly two" property anyway.

**The pattern is the finding, and it is worth carrying past this task.** Each round fixed the
sites it knew about and each round's *claim* was wrong the same way:

| round | fix | claim | what broke it |
|---|---|---|---|
| 0 | delivery keyed on depth | depth identifies an activation | a dead activation's depth is reused |
| 1 | the two `Flow` arms | "no path at all between the clause finishing and this check" | `run_bounded` |
| 2 | extracted `clause_boundary` | "exactly two callers", with a grep offered as proof | `run_loop` |

**A grep is evidence about today's tree, not a property of the code.** This is the same lesson
the per-clause-state defect taught three times in 4a, and it was only killed by a type -- fields
bundled into one `Copy` struct, so a new field is restored the moment it joins. Round 3 was
therefore dispatched asking for a construction that makes a fourth site impossible or
self-announcing, with explicit permission to report that no such construction exists rather than
to write a third hand-checked enumeration.

**Minor:** NEW-B, a failing `CALL ON` handler at `run_bounded`'s boundary echoes `2 *-* do i = 1
to 1` where the oracle echoes `3 *-* call sub` (round 1 echoed nothing) -- a *new* wrong line, and
the no-trap control attributes correctly. NEW-C, `deliver_pending_trap`'s doc left attached to
the inserted `clause_boundary`. NEW-D, `run_bounded`'s "every other exit returns the escaping
`Flow` unchanged" is now false. NEW-E, `restoring_none_is_still_the_common_case` is a strictly
weaker duplicate -- no mutation kills one without the other.

**The `Err` arm is resolved, and the answer is neither "measured" nor "unmeasured".** It **is**
reachable (a panic probe hits it from 4 programs; none of the 289 tests do) **and its restore is
unobservable, because `offer_to_trap` overwrites `active_condition` anyway.** Dead in effect.
Round 3 must decide deliberately: remove it, or keep it and say at the code that it is
unobservable-by-construction and why.

### Task 7 fix round 3: `f9e6bf26` -- the enumeration removed rather than extended

Controller-verified: **970 passed / 0 failed**, STRICT `38 of 38`, assertions 4224/4259, fmt 0,
clippy 0.

**NEW-A closed with a type obligation, which is what three rounds of enumeration failed to do.**
`ClauseState` moved to `clause.rs`. A private field on a **crate-root** struct is visible to
`crate::run`, because a child module sees its ancestors' private items; a private field on a
struct in a **sibling** module is not. So the line field is unreachable from `run.rs`, and the
only way to set it is `Interp::enter_clause`, which returns a `#[must_use] ClauseToken` that only
`Interp::end_clause` consumes -- and `end_clause` is what delivers a pending handler.

**Measured, not asserted.** Deleting the `end_clause` call gives
`error: unused variable: `token``, exit 101 under the clippy gate: *a clause entered and never
ended does not build.* The report states the remaining escape rather than hiding it -- `let
_token` still compiles, rustc's own message advertises it, and nothing short of a scoped-closure
API closes it. That honesty is in the module doc.

**Why binding the two was the right axis:** the two symptoms were always one omission. `do while
zn < sub()` reported `SIGL` from a clause three lines away *and* delivered its handler at the
wrong moment, because the site that fails to say "a new clause is starting" is the same site that
fails to run what a boundary owes.

**It found a site no round and no reviewer enumerated** -- the **`UNTIL` re-test** -- by probing
the family rather than the two reported members. And which clause a re-test belongs to *moves*:
the `DO` clause on the first pass, the `END` clause after (`SIGL` 4 then 7, measured).
`g1`/`g4`/`g5` all MATCH.

The boundary now lives inside `step_in_temps_frame`, so `run_activation` and `run_bounded` get it
without being listed. **One** rule keeps its own site, `offer_to_trap`, because a clause that
*failed* has not completed and whether its handler is owed depends on whether the failure is
trapped here or unwinds -- measured both ways (`ac1` delivers, `ps` does not).
`ClauseEnd::Completed`/`Failed` replaced an `Option<&Flow>` that conflated "no value" with "did
not complete"; under the `Option` the error path started delivering handlers it must not, caught
immediately by an existing test.

**NEW-B, C, D, E all closed** -- C and D fell out of the restructure rather than being patched
(`clause_boundary` is gone, so its fused doc comment is gone; `run_bounded` no longer synthesises
a `Flow`, so its doc is true again). **The `Err` arm decided:** kept, on the argument that the
alternative is not "nothing" but a wrong value that happens not to be read, and it would become
observable the day a reader does not go through `offer_to_trap`. Said at the code.

**Round-3 re-review dispatched**, told to attack the type obligation adversarially (`let _token`,
`mem::forget`, early return, `?` between enter and end), to re-derive the sibling-privacy claim,
and -- because the `UNTIL` discovery shows the family was not exhausted -- to probe for a fifth
site.

### Round-3 re-review: the obligation is real but narrow, and round 3 introduced a regression

`task-7-rereview3.md`, 60 tool uses, everything run. NEW-B/C/D/E **closed**; NEW-A **partially**.
Over 38 probes against a `26da3ac6` worktree: 11 DIFF->MATCH, 16 MATCH on both, 9 DIFF on both,
**2 MATCH->DIFF**.

**The type obligation stops exactly one thing: never mentioning the token.** 13 attacks, all run
against `cargo build` plus the clippy gate. Caught: deleting the call, `mem::forget`, forging a
token (`E0451`). **Escaping clean:** `let _token` (the one the implementer named), `drop(token)`,
an early `return` between enter and end, a `?` between them, **round 1's own Critical
re-expressed** as three gate-clean lines, and **setting the line with no token at all** through
whole-`ClauseState` assignment -- which `run.rs:3162` and `:3202` already do today. Also
`#[must_use]` is not what produces the quoted compile error: with it deleted the same
`unused variable` fires; it covers only the bare-statement spelling.

**Important, all RAN:**

* **A regression.** A loop re-test after `ITERATE` is attributed to the `END` clause; the oracle
  attributes it to the `ITERATE` clause. `u7`/`u8` **MATCHed on round 2 and DIFF now**, with no
  trap involved -- pure `SIGL`. Two comments assert the false rule.
* **Four clause sites still have no `enter_clause` and no boundary, silently**: `IF`'s condition
  with `then` on the next line, a false `WHEN`'s condition, `SELECT CASE`'s expression, and
  `WHEN`-false -> `OTHERWISE`. Byte-identical to round 2 so not regressions, but they falsify
  `clause.rs:191` ("Every site that runs a clause calls this"). The single-line `then`/`when`
  spellings match **only by coincidence**.
* **A newly expressible GC hazard.** `ClauseEnd::Completed(None)` at a site that has a `Flow`
  drops the rooting: the same `a live value` panic as deleting the `push_temp`, **yet clippy 0
  and `cargo test -p rexx-exec` exit 0, `collect_stress` included.** Round 2's `clause_boundary`
  took `&Flow` unconditionally, so this shape could not be written before.

**Minor:** `lib.rs:947-966` still names `clause_boundary`, which this commit deleted; round 2's
`unreachable!` guard is gone so an invariant holds but no longer announces itself; and **the
`UNTIL` re-test's boundary -- round 3's headline discovery -- is unobservable and untested**:
replacing it with `ClauseEnd::Failed` leaves 291/291 green and all 38 probes byte-identical. Only
its `enter_clause(end_line)` half is load-bearing.

**Verified correct:** `offer_to_trap`'s "a failed clause has not completed" holds across nesting
and `INTERPRET` (`o1`-`o5`, `o5` the adjacent refusal); `step_in_temps_frame`'s hot path
otherwise unchanged; `LEAVE`/loop control/trace unaffected; the two new loop tests are not
duplicates.

**Round 4 hands off to a fresh implementer**, per the skill's rule that a loop surviving three
resumes means the implementer cannot see its own problem. The blind spot here is specific and
repeated: **claims of exhaustiveness over a hand-enumerated site list**, now three rounds running
and present again in round 3's own module doc.

### Task 7 fix round 4: `9a4b57be` -- fresh implementer, and the pattern ends by derivation

Controller-verified: **976 passed / 0 failed**, STRICT `38 of 38`, assertions 4224/4259, fmt 0,
clippy 0. Tree clean. 175 tool uses, ~42 minutes.

**A/B against an `f9e6bf26` worktree over 63 probes: 17 DIFF->MATCH, 43 MATCH on both, 3 DIFF on
both, 0 MATCH->DIFF.** The three are one pre-existing class (a controlled `DO`'s per-pass `>>>`
control-variable lines under `trace r`), DIFF on `f9e6bf26` too.

**How the recurring pattern was ended: derive the rule from the oracle instead of listing sites.**
`RexxActivation::run` calls `processClauseBoundary()` after each `nextInst->execute()`
(`RexxActivation.cpp:642-654`), so a boundary sits after **every** instruction of the flat list;
this crate diverges only where `IF`/`SELECT`/`DO`/`INTERPRET` resolve other instructions inside
their own `step`. That is a statement about the language, not about today's call sites.

**And a `debug_assert` tripwire in `Interp::in_clause` to catch the next one.** It fired on
`p2/p5/p7/p8/p9` under the old code and stayed silent on `p1/p10/p11/p14/u7/n1` -- **and found a
fifth site the re-review did not have** (`p9`, a *true* `WHEN` with `then` on the next line). It
also fired once on a non-defect (a `DO`'s control setup and first header test are two clauses
here, one instruction there), which is why it compares lines; that exemption and what it
therefore misses are stated at the code rather than left implicit.

**Important 1** -- `HeaderClause{Do,End,Iterate(line)}` replaces `header_pass: bool`;
`do_body_outcome` returns `FellThrough`/`Iterated(line)`/`Escaped`. **A second divergence turned
up while measuring the first**: `END` must not *echo* for an `ITERATE`-ended pass either.

**Important 2** -- `IF`'s condition, `SELECT CASE`'s expression and each listed `WHEN`'s condition
are now clauses, pinned with `then` always on the next line, plus the adjacent success
(`a_single_line_then_reports_the_same_line_either_way`) so the rule is pinned to the property
rather than to the coincidence.

**Important 3 -- decision: strengthen, with a scoped closure.** Re-attacked and compiled:
**round 1's own Critical, re-expressed, is now harmless** (build 0, clippy 0, **tests 0**); round
3's `let saved = self.clause_state` is `E0507`; forging `SavedClauseState` is `E0603`. Residuals
named in the doc: a site can decline to call `in_clause` (the tripwire's job), and
`ClauseState::new()` stays reachable so `run.rs` can reset to line 0.

**Important 4** -- the rooted value now comes from the closure's return type via `ClauseValue`, so
`Completed(None)` at a `Flow` site is **inexpressible**. Plus `collect_stress.rs`'s
`a_clause_value_survives_the_handler_its_boundary_runs`: deleting the rooting panics `a live
value` on all three rows **while the 296-test lib suite stays green** -- which is exactly the gap
that let round 3's hazard through.

**Minors** -- `lib.rs`'s `pending_trap` doc rewritten; `HandlerExit::from_ended` makes the round-2
invariant a type again *and fails safe*; the `UNTIL` item dissolved, because "keep the line, drop
the boundary" is no longer expressible.

Seven mutations, each killing exactly the intended test. Measured cost ~1.5% on a 400k-iteration
loop; the `debug_assert` is absent from release.

**A process note from the implementer, unprompted and worth keeping:** its first spot-check after
cleaning up the A/B worktree silently reported *every* probe as changed, because the harness
pointed at a deleted binary. It recreated the worktree and re-ran all 63 rather than quoting the
broken run. Same family as `verification-commands-that-do-not-run`: an instrument that cannot
work reads exactly like one reporting a real result.

**Round-4 re-review dispatched** -- the last the process allows. It is told to re-run all 13
attacks against the new closure, to test the tripwire as the test-only instrument it is, to read
`RexxActivation.cpp:642-654` itself, and to mark every open item **load-bearing or parkable**,
because the controller adjudicates after it rather than looping again.

### Round-4 re-review: all seven closed, nothing load-bearing open

`task-7-rereview4.md`, 101 tool uses, everything run. **Important 1, 2, 3, 4 and all three Minors
closed.** Three new findings, every one Minor, parkable, and documentation-only -- the reviewer
measured the behaviour correct in all three cases.

**The scoped closure, attacked with re-review 3's own thirteen:** all thirteen are now
**inexpressible or no-ops**. A1-A4, A9, A10, A12, A13 have no token or no `ClauseEnd` to name;
A8 is `E0603`; A5/A6 return from the closure with the boundary still to come; B3 is `E0507`, B4
is `E0603`. **A5b -- round 1's own Critical, re-expressed -- is not merely caught but a literal
no-op** (build 0, clippy 0, tests 0), where round 3's identical three lines broke two tests. Two
escapes remain and both are recorded below.

**93 probes A/B against two confirmed-live binaries that disagree on 35: 35 DIFF->MATCH, 54 MATCH
on both, 4 DIFF on both, 0 MATCH->DIFF.** Thirty probes are the reviewer's own, aimed at the
shapes the dispatch flagged as uncovered, and 18 of those went DIFF->MATCH. The 4 both-DIFF are
the disclosed controlled-`DO` `>>>` gap. **It confirmed both binaries were live before trusting
any tally**, after the implementer's warning that a deleted binary reads as "everything changed".

**The tripwire is real, checked as an instrument rather than assumed:** M3/M5 fire it with the
right lines; a debug `rexx-run` with M3 applied panics on `p2` as a negative control; and all 93
probes under an unmutated debug build fire it **zero** times, with debug stdout byte-identical to
release. **The C++ derivation verified by reading**: one `processClauseBoundary()` call site,
`RexxActivation.cpp:653`, immediately after `execute` at 642; six `run_bounded` call sites here,
and the two without a header clause evaluate nothing, so no seventh construct exists.
**Performance concern withdrawn** -- three shapes including one built to stress per-header
boundaries, no measurable difference.

**The three new Minors, all parkable:**

* `run.rs:4579-4580` names `a_until_retest_reports_the_end_clauses_line`; **no such test exists**.
  The claim it decorates is true -- dropping the call fails two real tests.
* `clause.rs:245-250`'s "by type" half is narrower than stated: a site can smuggle its `Flow` out
  through a captured `&mut Option<Flow>` and return `()`, dropping the rooting, with build 0,
  clippy 0 and 296 lib tests 0. **The new `collect_stress` test catches it**, so the "by test"
  half is what carries the weight.
* `clause.rs:97-98`'s residual enumeration is short by one: `restore_clause_state` is
  `pub(crate)` and can put a **stale** state back, setting a nonzero clause line with no boundary.
  `deliver_pending_trap` being `pub(crate)` is the unnamed mirror image.

**Round 5 dispatched, comments only, cheap model.** All three are false statements in comments,
and `rust/CLAUDE.md` requires a false comment corrected or removed rather than hedged -- and this
task's recurring defect *was* false claims in comments, so leaving three behind would be the
worst possible closing note. The dispatch forbids behaviour changes, requires all five gate
numbers to be identical afterwards, and asks that the residual enumeration be restated as a
property or else explicitly labelled a list of escapes known today rather than a proof that no
other exists.

### Task 7 fix round 5: `1663538a` -- and Task 7 is complete

**Controller-verified comment-only**: `git diff 9a4b57be 1663538a` filtered for non-comment
changed lines returns **empty**. All five gates identical to baseline: 976 passed / 0 failed,
STRICT `38 of 38`, assertions 4224/4259, fmt 0, clippy 0.

Each of the three corrections was made by **reproducing the reviewer's mutation and reverting**,
not by reading:

1. `run.rs:4579` cited a test that does not exist. Reproducing M-U (dropping the `in_clause` call
   at the `UNTIL` re-test) showed the two tests that actually fail are
   `a_while_retest_belongs_to_the_do_clause_then_to_the_end_clause` and
   `a_loop_retest_after_an_iterate_belongs_to_the_iterate_clause`. Those are now named.
2. `clause.rs:245` claimed the closure's return type means "the site cannot choose". Reproducing
   the C1 smuggle (compute the `Flow` inside the closure, stash it in a captured
   `&mut Option<Flow>`, return `Ok(())`) gave build 0, clippy 0, 296/0 lib tests -- and
   `collect_stress`'s `a_clause_value_survives_the_handler_its_boundary_runs` failing with
   `a live value`. The comment now says the type narrows the shape and the stress test is what
   pins the rooting.
3. `clause.rs:97`'s two-item enumeration rested on a false claim. Reproducing C3 (wrap
   `run_bounded`'s `step_in_temps_frame` in a save/restore pair, restore a stale state) went
   undetected at build 0, clippy 0, 296/0. **Rewritten as a property** -- this module cannot
   narrow its `pub(crate)` surface below what legitimate callers need -- with both concrete
   escapes named and an explicit statement that the list is what is reachable today, **not a
   closed proof.** That last clause is the whole lesson of this task written into the code.

## Task 7: complete

`7ec66f84` -> `1663538a`. Five commits: `f906aabc` (task), `431e2698`, `26da3ac6`, `f9e6bf26`,
`9a4b57be`, `1663538a` (fix rounds 1-5). Four review passes, all on opus, all measuring rather
than reading. Final state: **976 passed / 0 failed**, STRICT `38 of 38` (corpus 37 -> 38),
assertions 4224/4259, fmt 0, clippy 0.

**Open items carried forward, none load-bearing:**

* **Task 9 (trace prefixes / I31)** owns the `EXIT <expr>` `>>>` gap, and now also the
  **controlled-`DO` per-pass `>>>` control-variable lines** under `trace r` -- the 4 both-DIFF
  probes in round 4's A/B.
* **Task 10 (corpus)**: `rust/corpus/lang/condition_syntax.rex` exists but is in no subset.
* **4c**: `condition()` stays loud, so `Raised::condition` and `Trap::call` still have no
  differential witness.
* **Final whole-branch review to triage**: `RAISE PROPAGATE`'s report format still rests on four
  transcripts; `run_activated`'s `_program` parameter is dead; `trap_for` is `pub(crate)` with no
  cross-module caller; the two `pub(crate)` escapes named in `clause.rs:97`.
* **Pre-existing, not 4b's**: the parse-error reporting arm (rc 120 against the oracle's rc 221
  with a clause echo), reproduced by `say 1 +` with no `RAISE` in it.

## Task 8: implementer reported -- `ed7f1e94`

**Status DONE.** Controller-verified: **978 passed / 0 failed**, STRICT **39 of 39** (38 -> 39),
assertions 4224/4259 unchanged, fmt 0, clippy 0.

`queue.rs` is new: `push` head-inserts (LIFO), `queue` tail-appends (FIFO), matching the oracle's
shared `RexxInstructionQueue`. Wired into `Interp`; two `step()` arms in `run.rs` mirroring
`SAY`'s evaluate/render/trace shape. All three of the brief's measurements were **re-run rather
than inherited** and matched.

**The pre-dispatch check paid off again, and my own dispatch was wrong in one place.**

* *Right:* the brief named `owners.rs` but not `tests/loud.rs`, which carries the two `Push`/
  `Queue` witness rows **and** a pinned count. Handing that over meant it was not rediscovered.
* **Wrong: I told it to remove `Push`/`Queue` from `lib.rs`'s match at line 425. It refused, with
  evidence, and it was right.** That match is `let name = match kind` -- an exhaustive name
  lookup over every `InstructionKind`, listing implemented variants (`Do`, `If`, `Interpret`,
  `Return`, `Signal`) alongside unimplemented ones, because it is a *completeness* requirement
  and not a scope table. Removing two arms would be a non-exhaustive-match compile error with
  nowhere for them to go. It edited the real scope table, `instruction_owner`, instead. Verified
  by the controller by reading `lib.rs:415-435`.

**A second deviation, also with evidence: it added a corpus witness the brief did not ask for.**
Moving `Push`/`Queue` in scope trips `tests/coverage.rs`'s criterion-1 gate -- every in-scope
variant needs a corpus witness -- which is a *different* requirement from the queue-storage
coverage gap the brief describes. Without it `cargo test --workspace` would have regressed. The
witness `lang/push_queue.rex` runs under `TRACE R` so its differential run is real: it pins the
half the harness can reach (expression evaluation, rendering, trace output, and that a bare
`PUSH`/`QUEUE` traces a null string), leaving only storage order uncovered.

**The KNOWN GAP row is written the way this project needs.** It states plainly that no corpus
program can distinguish a correct `PUSH`/`QUEUE` from `Push | Queue => Ok(Flow::Next)`, that
**unlike the rest of its section nothing is known to be wrong**, what coverage exists instead, and
where the storage order came from: the oracle prints `C`, `A`, `B` for `push "a"; queue "b";
push "c"` followed by three `PULL`s, and since `PULL` upcases, the queue's own order is `c`, `a`,
`b`. That probe is deliberately not a corpus program -- it needs `PULL`, which is 4c's.

It also corrected a doc comment above `INSTRUCTION_WITNESSES` that was **already stale from Task
7** rather than compounding it while editing the same array.

**Review dispatched** (opus) at `1663538a..ed7f1e94`, weighted to the one question that matters
here: whether the substitute coverage is real, or whether a no-op that still traces would satisfy
it.

### Task 8 review (opus): spec MET, quality GOOD, no Criticals -- but one real hole

`task-8-review.md`, 54 tool uses, five mutations, tree restored and re-verified at 978/0.

**Both deviations confirmed correct independently.** Deleting `lang/push_queue.rex` from
`phase-4b.txt` fails `coverage.rs:639` with `2 in-scope variant(s) unwitnessed: Push, Queue`, and
the `lib.rs:425` refusal is `E0004` on a match listing nine implemented variants. Bookkeeping
touched exactly the right tables; `plan.rs`/`coverage.rs` correctly untouched.

**I3, the substantive finding: the degenerate that survives everything is named nowhere.** Delete
only `self.queue.push(line)` / `self.queue.queue(line)` -- keep the evaluation, keep the trace,
discard the value -- and `cargo test --workspace` stays at **978/0** with STRICT at **39/39**.
**Nothing in the tree asserts that `run.rs` writes to `Interp::queue` at all.** `queue.rs`'s unit
tests pin the type, `push_queue.rex` pins the trace, and nothing joins them. Closeable without
4c: a test that observes the queue's contents after running a program through the interpreter.

**I1 and I2 are false descriptions of the safety net.** `queue.rs:119` claims `run.rs`'s arms are
exercised by `tests/loud.rs`; with both arms replaced by `Ok(Flow::Next)`, `--test loud` is 8/8
green -- the catcher is `corpus.rs`. And the KNOWN GAP row's central claim, that no corpus program
can distinguish a correct `PUSH`/`QUEUE` from a no-op, **is false as of the commit that wrote
it**: `push_queue.rex` takes STRICT to 38 of 39 under exactly that mutation. The row contradicts
its own later paragraph.

**The reviewer's boundary is sharper than either the brief's or the row's:** the trace half *is*
differentially covered (no-op mutation -> 38/39); the storage half is *not* (delete just the
write -> 39/39). That is the sentence the row should carry.

**M3 is worth noting for the controller's earlier question:** the reviewer explicitly endorsed
"nothing is known to be wrong" as honest -- it looked and found nothing. What is false is the
*section header*, "Each is a real divergence", which the row hedges around instead of correcting.

**M6 is pre-existing:** `owners.rs:185-186` still says `InstructionKind::Call` keeps
`Owner::Phase("4b")` -- false since Task 7, and contradicted by line 120 of the same file.

**Fix round 1 dispatched**, weighted to I3, with the instruction to verify the new test dies to
the reviewer's exact mutation. Also passed back a note that the report's own "Test that could
fail" section asserted a property a two-line mutation falsifies -- the recurring shape here is an
instrument *described* as covering something it does not reach, and the cure is running the
mutation before writing the sentence.

### Task 8 fix round 1: `e25c9065`

Controller-verified: **979 passed / 0 failed** (+1), STRICT `39 of 39`, assertions 4224/4259,
fmt 0, clippy 0.

**I3 closed, and the controller checked it rather than relaying it.** The new test is
`push_and_queue_actually_write_into_the_running_interpreters_queue`: it runs a real program
through `Interp::run_activation` and reads `Interp::queue` back. Applying the reviewer's exact
mutation -- replacing both `self.queue.push(line)` / `self.queue.queue(line)` with
`let _ = line;`, keeping evaluation and tracing -- gives **298 passed, 1 failed**, and the one
failure is that test. It pins precisely the previously-unpinned property and nothing else. Tree
restored, `git status` clean.

I1 and I2 corrected; the KNOWN GAP row is now **three separately measured claims** rather than one
falsifiable sentence. All seven Minors addressed, including **M6, which was not this task's** (a
second stale `owners.rs` comment inherited from Task 7) and **M7**, where the corpus program now
exercises a numeric value as well as strings, with a regenerated `sourceline_oracle` fixture
re-verified byte-identical against the oracle.

**The implementer corrected its own earlier report section**, which had carried the same false
claim the review caught as I1, and named the failure mode explicitly: written without running the
mutation first. That is the habit `rust/CLAUDE.md`'s method section exists to install, applied by
the person who tripped over it rather than by a reviewer.

**Scoped re-review dispatched** (sonnet -- small fix diff), told not to re-derive the gate numbers
or the mutation the controller already ran, and weighted to whether the new test catches
*corruption* as well as deletion (swap the two arms, wrong order, only-last-value), whether all
three of the rewritten row's claims are individually true, and whether rewriting eight comments
introduced a ninth false one.

### Task 8 re-review: all ten closed. Task 8 complete at `e72cc19f`

`task-8-rereview.md` (sonnet, 62 tool uses). **I1, I2, I3 and M1-M7 all closed.**

**I3 was tested harder than it was asked to be.** Beyond the controller's deletion mutation, the
reviewer ran four more -- the full `Ok(Flow::Next)` no-op, swapping `push`/`queue` in `run.rs`'s
arms, swapping `push_front`/`push_back` inside `Queue`, and "store only the last value". The test
dies to all five. It is a guard against corruption, not only against deletion, which was the open
question when the round was dispatched.

M7's regenerated fixture is **MD5-identical** to the oracle's output, and the reviewer confirmed
`to_text` really does have a distinct code path for numbers against literals -- so the widened
corpus witness pins something the string-only version did not.

**NEW-1, the one new finding, was the same stale claim in a third place.** `lib.rs:783-786` still
said `InstructionKind::Call` keeps an owner string "because `Call::Trap`/`Call::Qualified` are
still loud". `Call::Trap` moved in scope at Task 7; only `Call::Qualified` is loud, and the
arm-grained match sixty lines above (`lib.rs:714-723`) already says so correctly.

**Fixed by the controller directly** -- no agents were live, and it is one comment. Verified the
true state against `lib.rs:714-723` and `owners.rs:129/185-191` before writing, confirmed
comment-only by filtering the diff for non-comment changed lines (empty), and re-ran the gates:
**979 passed / 0 failed**, STRICT `39 of 39`, fmt 0, clippy 0. Commit `e72cc19f`, hash read back
from `git log`.

**The pattern is worth recording.** One stale fact ("`Call::Trap` is loud") had **three** copies:
`owners.rs` (found by the Task 8 review), `loud.rs`'s `INSTRUCTION_WITNESSES` doc (found by the
implementer while fixing the first), and `lib.rs` under `ExprKind::Call` (found by the re-review).
Task 7's own five review passes saw none of them, because each sits in a file Task 7 did not have
to touch. **This is the ownership-attribution cost the Task 12 decision is about**, now with a
concrete instance attached: three prose copies of one fact, drifting independently, none
machine-checked.

## Task 8: complete

`1663538a` -> `e72cc19f`. Three commits: `ed7f1e94` (task), `e25c9065` (fix round 1), `e72cc19f`
(re-review NEW-1). Final: **979 passed / 0 failed**, STRICT **39 of 39** (38 -> 39), assertions
4224/4259, fmt 0, clippy 0.

**Next: Task 9** -- trace prefixes `>A>`/`>F>`/`>R>` and I31's two missing `>>>` lines. Run the
pre-dispatch check first; it has now found a missing prerequisite for **nine** consecutive tasks.
Task 9 also inherits two gaps found during Task 7: the `EXIT <expr>` `>>>` line, and the
controlled-`DO` per-pass `>>>` control-variable lines under `trace r` (the 4 both-DIFF probes in
Task 7's round-4 A/B).

## Task 9 dispatched -- trace prefixes, the activation indent base, and three absent-value-line gaps

Base `e72cc19f`, opus. Pre-dispatch check found three things, its ninth consecutive task with a
finding, and the categories keep changing.

**1. A stale line number.** `CLAIMED_PREFIXES` is at `tests/trace_oracle.rs:254`, not the brief's
`:233`; the assertion comparing it to the witness union is at `:304-308`.

**2. The brief contradicts itself, and the later wording is the stale one.** Its Step 2
establishes by measurement -- after an earlier wrong answer that its own review caught -- that a
`::routine` **is reachable** in 4b for any non-builtin name (`call zorkolo` with a
`::routine zorkolo` runs on the oracle at rc 0), and that 4b defers it **by decision** because
builtin-colliding names need 4c's table. It then says twice to write the exclusions row as a
deferral and once, later, to write it as "unreachable until 4c's builtin step exists". Those are
mutually exclusive, and "unreachable" is the exact error the brief's own closing paragraph warns
against. **Resolved in the dispatch: write the deferral.**

**3. A gap Task 7 handed to Task 9 is in neither the brief nor the exclusions file**, and would
have vanished. Measured by the controller rather than relayed:

```
trace r / say 'a' / exit 0
oracle:  3 *-* exit 0  then  >>>   "0"
ours:    3 *-* exit 0  then  nothing
```

Both rc 0. **`EXIT <expr>` emits no `>>>` value line** -- same family as I31, a value line that is
*absent*, which DEVIATION 0's indentation normalisation does not touch. Task 7 also left four
both-DIFF probes described as a controlled `DO`'s per-pass `>>>` control-variable lines; the
dispatch asks whether those **are** I31 or a third gap, **by measurement, assuming neither way**.

So Task 9 owns three absent-value-line gaps rather than one. The dispatch allows closing fewer,
provided the rest get a KNOWN GAP row with an owner and a measured transcript, and provided the
choice is stated rather than left silent.

**Also carried into the dispatch:** the activation indent base is the *calling clause's printed
indent* plus the delta, not twice the depth; `static_indent`'s signature does not change; Task 2's
40-column clamp applies to `*-*` only and **must not** be extended to value lines or
re-implemented (measured at depth 25: `*-*` tops out at 40 while `>>>` runs to 52); and Step 7's
coverage number must be **asserted against a committed literal**, since a printed number no
assertion reads cannot fail.

## Task 9: implementer reported -- `233fbd8b`

**Status DONE.** 232 tool uses. Controller-verified: **986 passed / 0 failed** (was 979), STRICT
**40 of 40** (was 39), assertions 4224/4259 unchanged, fmt 0, clippy 0.

**All three absent-value-line gaps closed, none deferred.** The controller re-ran its own
pre-dispatch probe: `trace r` / `say 'a'` / `exit 0` is now **byte-identical to the oracle on both
stdout and stderr**, where before it omitted `>>>   "0"`.

**The four both-DIFF probes were settled by measurement, not by description.** The implementer
found `q11`/`y4`/`z6`/`w29` in Task 7's round-4 scratchpad, re-ran them against both binaries --
all four DIFF on `e72cc19f`, all four byte-identical now. **They are I31, one gap, not a third.**
That is the question the plan was corrected to ask, and it was answered the way the correction
asked for it: by running, assuming neither.

Step 5 needed no code -- the activation indent base was already in the tree, and the brief's
transcript matched at `e72cc19f` except for `use arg`'s value lines.

**Two of the brief's I14 claims are contradicted by measurement, and the implementer built what it
measured.** It reports `>O>`/`>A>` at a reference call site are `>O> ">" => "ORIG"` (the *name*)
and `>A> "PP"` (the *value*) -- **the brief has them swapped** -- and that "an internal-label call
with `trace l` first emits nothing" is false, because the oracle echoes every label clause it
executes. **Sent to the review to settle independently.** If confirmed, the plan's I14 row is
wrong and gets corrected at the source, per the rule added to `rust/CLAUDE.md` at `d24dde78`.

**Two new KNOWN GAPS recorded rather than fixed**, both found while probing: `TRACE L` echoes
label clauses on the oracle and nothing here (owner unassigned -- the fix is small, but
`mode_from_setting`'s classification of `L` is read by `TRACE()` reporting and by 4c's `>I>` row);
and `do aa.1 = 1 to 2` stores a **simple** variable named `AA.1`, a **stdout** divergence present
on `e72cc19f` too, needing a `rexx-parse` change that no 4b task's file list reaches.

**Self-reported scope creep:** it fixed two things Task 7 owned, because they sat in the three
lines `>A>` had to occupy -- `RAISE ... ARRAY` traced its `>K>` before the elements instead of
after, and closed up an omitted element instead of leaving a hole (a stdout difference: ours
`maximum expected is X.` against the oracle's `maximum expected is .`), where the code comment had
labelled the choice "unmeasured". Handed to the review to judge as scope and to verify as
correctness.

**Owed to the durable artifacts once the review confirms them** -- deliberately not written yet,
because writing unverified claims into the plan is how the false statements being cleaned up this
week got there:

* the I14 `>O>`/`>A>` correction and the `trace l` correction, into the plan's Task 9 text;
* an **owner** for the `do aa.1 = 1 to 2` gap. It needs a `rexx-parse` change, so no 4b task
  reaches it, and an unowned gap row is the shape that goes stale. Controller's call at Task 12
  at the latest.

### Task 9 review (opus): all eight steps met, quality good, no Criticals

`task-9-review.md`, 75 tool uses, 26 probes of its own, six mutations applied and reverted.
**Both contested points went the implementer's way**: `>O>` carries the name and `>A>` the value,
`>R>` is names on both sides, and `trace l` **does** echo an internal label's clause with and
without `PROCEDURE`. The brief was wrong; the implementer built what it measured.

**F3 was the controller's and is fixed at `38b2cb7b`** -- both false I14 sites in the plan now
carry the measured shapes and say what was wrong. The correction had been living only in the task
report, which is the exact failure `rust/CLAUDE.md`'s new rule describes, **caught by a review one
commit after the rule landed.**

### Task 9 fix round 1: `94237403`

Controller-verified: **989 passed / 0 failed**, STRICT **41 of 41** (40 -> 41), assertions
4224/4259, fmt 0, clippy 0.

**F2 closed, not deferred, and costing it first is what decided that: the fix is four lines.** A
Controlled loop now re-*reads* its control variable each re-tested pass. `do ii = 1 to 3 ; ii = 10
; end ; say ii` was ours 4 against the oracle's 11 -- the oracle reads 10 back, adds 1, ends after
one pass. Two adjacent shapes came free (`DROP` -> 41.1 on `"II"`, non-numeric -> 41.1 on
`"abc"`), and they showed the review's "milder instance" was **not** mild: *every* such failure
was blamed on the `DO` clause where the oracle blames whichever clause transferred control back.
A/B against `233fbd8b`: six F2 shapes DIFF->MATCH, ten adjacent probes 6 DIFF->MATCH, 4
MATCH-on-both, **0 MATCH->DIFF**.

**F1 answered better than the finding asked.** The controller said "if nothing can pin it, say
so". The implementer found that "no test can pin it" held only for the two **normalising**
harnesses -- a unit test asserting `interp.trace` **raw** sits outside both -- and added one
asserting two full oracle transcripts that goes red under precisely the two mutations the review
used to prove the witnesses could not.

**F8 closed rather than argued.** `TraceMode` gains a fourth field, true wherever `all` already
is, so the new gate reduces to `all` in every mode but `L`, asserted directly to bound the change
to one letter.

### A harness incident, self-reported, and a new rule from it

The implementer's round-0 mutation harness restored files with `git checkout -- FILE`. Run this
round **before committing**, its first invocation **discarded every uncommitted `run.rs` change**;
its second then reported **STAYED GREEN for a test that no longer existed**, which is how the
discard was noticed at all. Work redone from the session record and re-verified; the harness now
restores from a copy.

**Verified directly by the controller:** `cargo test <name>` exits **0** when it matches nothing
(`0 passed; 0 failed; 301 filtered out`). So a mutation harness reading only the exit status
**cannot distinguish "passed" from "gone"** -- the instrument that checks whether tests can fail
was itself a test that could not fail. Recorded in `rust/CLAUDE.md` at `a55ed36c`: assert a
non-zero run count, restore from a copy rather than from git, and never let a mutation harness be
the only thing deciding a test is load-bearing.

**Re-review dispatched** and told the round-0 mutation evidence is **partly suspect** -- to re-run
a representative sample of both rounds itself, treat any claimed-red mutation that comes back
green as a Critical, and probe its own loop shapes, since ten adjacent probes is a small sample
for a change to control-variable semantics that every `DO` in the corpus exercises.

### Round-1 re-review: F1-F8 closed, mutation evidence survives, one real defect introduced

`task-9-rereview.md`, 102 tool uses, 66 probes of its own, every mutated file restored **from its
own copies with md5s checked** -- never `git checkout --`, following the rule the incident
produced.

**The suspect evidence held.** No claimed-red mutation was found green. All seven round-1
mutations re-run red with **non-zero run counts read explicitly**; five round-0 mutations
re-sampled and red, bringing reviewer-confirmed coverage to nine of thirteen. F1 verified exactly:
both asserted transcripts are the oracle's own bytes, and the new raw-`interp.trace` unit test
goes red under **both** review mutations while `--test trace_oracle` stays at 21 and the corpus at
41/41. F8's "reduces to `all` in every mode but `L`" is true **exhaustively** -- the five
constants are the only constructible `TraceMode` values.

F2 confirmed over **66 probes** A/B'd against a `233fbd8b` baseline: ~30 DIFF->MATCH, **0
MATCH->DIFF**.

**NEW-1 (Important, load-bearing): the F2 fix introduced a defect.** The re-read uses
`read_by_name`, which **bypasses `NOVALUE`**. With `signal on novalue` and a body that `DROP`s the
control variable, the oracle runs the handler and exits **rc 0**; we raise a spurious **41.1 at
rc 215**. `SIGNAL ON NOVALUE` is otherwise correct, so it is a hole at one site -- and it makes
"Both match byte for byte now" an overclaim in both the report and the exclusions text.

**Five Minors, every one a false statement, and four of them written *while correcting* the first
three.** A stale `L` in the `TraceMode::OFF` letter list; a comment recording a mutation result
that quotes the **old** gate number and is contradicted by its own next paragraph; a module-doc
table claiming `>>>` appears in "every witness below" when `trace_labels.expected` has none, and
"three witnesses" where there are now four; an F9 note asserting a pre-gate shape "every other
tracing site in this file uses" when `tracing_intermediates()` occurs **exactly once**; and a doc
naming `setTraceLabels` directly above a paragraph asserting it no longer does.

**The pattern, named in the dispatch:** across three consecutive commits this task shipped
comments contradicting something measured in the *same commit* -- three in the original, then five
more produced by the act of correcting those three. **The corrections are where the new false
statements come from.** Round 2 was dispatched with one concrete rule: every claim about a number,
a count, or "every other site" must have a command behind it, and must say which.

**The re-review hit the same hazard from the other side**, and recorded it: its first mutation
attempt left `--test trace_oracle` at 21 passed while the defect was real, because the corpus
pinned it instead. A harness running one test file would have called it green -- the sibling of
`cargo test <name>` exiting 0 on zero matches.

**Parkable, to be recorded not fixed:** the oracle's under-indent after a completed inner loop
(pre-existing, DEVIATION 0's territory, currently unrecorded); and that `TRACE()` is unimplemented,
so the `TRACE L` gap row's `TRACE()`-coupling argument had no live reader.

### Task 9 fix round 2: `f02c3ed3`

Controller-verified: **990 passed / 0 failed**, STRICT `41 of 41`, assertions 4224/4259, fmt 0,
clippy 0.

**NEW-1 fixed, and the controller measured it rather than relaying it.** `signal on novalue name
nh` / `do ii = 1 to 3` / `drop ii` / `end` now gives **rc 0 on both sides with both descriptors
byte-identical**, handler at `SIGL 4`. `read_by_name` is replaced by `Interp::read` +
`novalue_check`.

**The implementer's diagnosis of its own error is the useful part: the wrong reader was
type-shaped, not semantic.** `read_by_name` took a `&[u8]` exactly like `bind_control`, and that
similarity is what made it look right. It returns the derived name on a miss and reports nothing
to its caller, where the oracle's `control->evaluate` is an *evaluation* and raises NOVALUE.

**The ordering is pinned rather than commented.** `novalue_check` runs *before* the `>V>`/`>>>`
lines because the oracle's failing re-test emits the `DO` re-echo and then nothing at all.
Mutation R2-M2 moves it after and the witness goes red -- and **stdout and rc would still agree**,
so only stderr separates "raises the condition" from "raises it at the right moment". Sent to the
re-review as the sharpest claim of the round.

**One probe was discarded rather than reported**, and the discard was disclosed: it does not
terminate on either side, so its only difference was which side `timeout` killed first.

**NEW-2 to NEW-6 each fixed against a named command, with the command quoted in the replacing
text** -- `grep -c '>>>' trace_labels.expected` -> 0, `grep -c 'tracing_intermediates()' run.rs`
-> 1, `cargo test --workspace` -> 989. **Where possible it removed the class rather than the
instance**: the count beside the prefix table is gone rather than corrected, and both "every
witness below" rows are now assertions.

**It disputed one parkable and was specific about why.** The under-indent is *not* unrecorded --
the indent row already states the rule; what was missing is that **every example in that row
raises**, so it now carries the ordinary `trace r` shape beside the raising one, and a duplicate
row would have been the worse fix. The `TRACE()` observation then found something still live:
DEVIATION 0's "stays byte-exact" list opened with "`TRACE()` is readable by the program", which is
a property of the **oracle**, not of the agreement -- corrected with the transcript (`say
trace()`: oracle `N` rc 0, ours `not implemented (4c)` rc 120).

**On the named pattern, it supplied a mechanism rather than an apology**, and it is worth keeping:
*when correcting a comment you are writing about the code's **context** -- what other sites do,
what the gates say, how many things there are -- and that is exactly the part you cannot see from
the line you are editing.* Three of six findings that round were countable claims, and all three
would have fallen to a grep taking under a minute. **A sentence about context needs a command the
way an assertion needs a mutation.**

**Round-2 re-review dispatched** (sonnet, 36 KB diff), weighted to the ordering mutation, to
whether a different reader on the hot path changed anything else, to whether the discarded probe
hides a divergence, and to whether this fourth consecutive round of comment corrections produced
a further false comment.

### Round-2 re-review: closed except NEW-5, and a self-defeating comment

`task-9-rereview2.md`, 96 tool uses. NEW-1, 2, 3, 4, 6 **closed**; NEW-5 **partial**.

**The ordering claim survives, verified the hard way.** The reviewer ran the mutated binary A/B
rather than reading a test count: stdout and rc stay identical to the oracle (`handler II`, rc 0)
while **stderr alone diverges by exactly the two predicted lines**. R2-M1 through R2-M4 all
reproduced the implementer's exact counts. The discarded probe was confirmed genuinely
non-terminating on **both** interpreters -- a sound discard. No performance regression from the
different reader: 3.64s baseline against 3.49s at head on a 2M-iteration loop.

**NEW-F1: the comment quoting its own evidence made itself false.** `run.rs:5296` quotes
`grep -c 'tracing_intermediates()' run.rs` and says `1`. Today it is **2** -- **the comment's own
text contains the literal search string, so committing the evidence created the second match.**

This is a failure mode of the remedy the controller imposed one round earlier ("put a command
behind every countable claim"). **When the artifact being measured is the file you are writing
into, quoting the search term changes the answer.** The dispatch asked for a fix that cannot rot
-- describe the property without embedding the literal -- explicitly *not* updating `1` to `2`.

**NEW-F2:** `phase-4-exclusions.txt:1148` claims `novalue_check` is "the pairing every other read
site in this crate already pairs with it". Of three other `self.read(code, ...)` sites **only
`eval.rs:319` pairs with it**; `expose_names` and `drop_variable` both discard the `Novalue` flag.
It also undercuts the adjacent LESSON paragraph about what `read_by_name` is "for" -- its only
other caller is `tail_key`, not `PROCEDURE EXPOSE`.

**The pattern is now six instances across four consecutive commits, and every one arrived in a
correction to a previous one. The corrections are the defect's habitat.** NEW-F2 is an "every
other site" claim written *in the paragraph diagnosing that exact error*, so the mechanism the
implementer correctly named did not protect it -- which is the argument for a mechanical check
over a remembered one.

**Round 3 dispatched asking for a judgement on the class, not just the instances**, with two
candidate mechanisms and permission to propose a third: hold claims where a machine can check
them (the only fix so far that has never needed correcting -- the "every witness below" rows
became assertions and the hand-maintained count was deleted), or constrain prose's form (no bare
counts, no "every other site", no quoted command whose search term the comment contains). Stated
requirement: **no seventh instance in the commit that closes the sixth.**

### Task 9 fix round 3: `50da3045` -- and a taxonomy that explains all six

Controller-verified: **990 passed / 0 failed**, STRICT `41 of 41`, assertions 4224/4259, fmt 0,
clippy 0. **No behaviour changed**, verified two ways: filtering the crate diff for non-comment
changed lines returns empty, and `phase-4b.txt`'s program list is byte-identical, 11 before and
after.

**NEW-F1: the survey was deleted, not corrected from 1 to 2.** The count was never load-bearing --
the gate decides whether to *build* the Vecs, not whether to print -- and it is replaced by a
benchmark: 2M passes under `TRACE OFF`, release, three runs each, **3.13/3.13/3.15 s with the gate
against 3.22/3.22/3.23 s without, ~40 ns per pass.** A number about a fixed benchmark cannot rot
as the file changes.

**NEW-F2: enumerated, then only the durable half kept.** 1 of 3 confirmed. Pasting the census
would fix the falsehood and keep the liability, so the paragraph now says pairing is a **per-site
decision**, and the LESSON paragraph argues from the two readers' **signatures** -- one cannot
report an unset read, the other can -- which the compiler keeps honest.

**It found two more itself, by sweeping `git diff e72cc19f | grep '^+'` rather than re-reading**: a
round-1 paragraph still saying the read goes "through `read_by_name`" after round 2 changed the
reader (same defect, same twenty lines, one round earlier), and `phase-4b.txt`'s "every other 4b
program uses trace r", which enumeration showed was **already false** -- four set no trace at all.

**The taxonomy, which is the round's real output and is now `rust/CLAUDE.md` at `6eddd10d`:**

> Comments pointing at **immutable referents** -- oracle bytes, C++ citations, benchmark numbers --
> have needed **zero** corrections across four commits. **Every** falsehood was a **mutable repo
> aggregate**: call-site counts, witness surveys, gate totals.
> So: immutable -> prose is fine; mutable and load-bearing -> assert it; mutable and not
> load-bearing -> delete it. Plus the form rule the self-defeating grep generalises to: **a claim
> must not be falsifiable by the act of committing it.**

That also explains why the controller's round-2 rule backfired. "Put a command behind every
countable claim" *produced* NEW-F1 directly, because the command's search term joined the corpus
it measured. **A rule about how to evidence a claim cannot fix a claim that should not be prose.**

**It declined to add a lint, with a reason worth keeping:** it would fire on the legitimate
immutable statements, and hedging it would be this task's failure mode one level up.

**And it flagged where its own taxonomy is weakest** -- two claims it classified immutable that
are really about the *language* rather than the oracle at a fixed program: "all three ways a label
is reached" and "all nine accepted letters". Both measured, both believed exhaustive, but
**exhaustiveness over a language is weaker than a transcript**, and that is where a seventh
instance would most likely come from. The round-3 re-review was dispatched to attack exactly
that -- find a fourth route or a tenth letter -- plus re-run the benchmark, since it is now the
load-bearing prose.

### Round-3 re-review: the chain is broken, and the predicted weak spot was real

`task-9-rereview3.md`, 73 tool uses. **NEW-F1, NEW-F2 and both sweep-found fixes closed.**
**Answer to the round's stated question: no seventh false statement** -- every added sentence in
the 13 KB diff checked against running code or direct git history.

**The benchmark reproduces on different hardware:** five interleaved release rounds, with-gate
mean 3.542s against without-gate 3.625s, ranges non-overlapping in all five, **delta ~41.8 ns per
pass** against the claimed ~40. Absolute times differ, which is exactly why stating the per-pass
delta rather than the wall time was the right call.

**The implementer's own risk flag was correct.** It had named its two weakest claims -- "all three
ways a label is reached" and "all nine accepted letters" -- on the grounds that exhaustiveness
over a *language* is weaker than a transcript. Both were attacked:

* **Tenth letter: does not exist.** The reviewer read `TraceSetting::parseTraceSetting` directly
  (exactly nine letter cases plus a default error) and probed seventeen other letters, all `24.1`.
* **Fourth label route: found.** An **internal function call**, `x = sub()`, reaches a label under
  `TRACE L` -- confirmed against the live oracle, `*-* sub:` echoing exactly as under `CALL sub`.
  No behavioural bug (this crate already matches byte for byte); the **claim** is false and
  `trace_labels.rex` does not witness the route.

**The asymmetry is the lesson and refines the taxonomy committed at `6eddd10d`.** The letters
claim survived because a single C++ switch **enumerates** the letters, giving it a checkable
referent. Nothing enumerates label routes, so that claim was a universal quantifier with no
enumeration behind it -- and it reads exactly like the safe kind. Provisional rule put to the
implementer for its read before it goes into `rust/CLAUDE.md`: **an exhaustiveness claim is only
as good as the enumeration behind it -- cite the site that enumerates, or do not claim
exhaustiveness.** That would have caught "three ways" and permitted "nine letters".

**Second parkable:** round 3's deletion took a true sentence out with the false one (`step`'s
`Assignment` arm building `rendered` unconditionally, verified true at `run.rs:957`, not restated
elsewhere). No correctness risk; restoration left to the implementer's judgement.

**Round 4 dispatched**, small: correct the "three ways" claim, add the function-call route to
`trace_labels.rex` and regenerate its expectation byte-identical to the oracle, and decide on the
deleted sentence.

### Task 9 fix round 4: `16a67ae2` -- and the rule reaches its final form

Controller-verified: **990 passed / 0 failed**, STRICT `41 of 41`, assertions 4224/4259, fmt 0,
clippy 0, `--test trace_oracle` 22 passed.

**Two countable claims checked rather than relayed, because this task has been wrong on eight of
them.** (1) "The counts do not move" is **true**: `trace_oracle.rs`'s test markers are 33 at both
`50da3045` and `16a67ae2`, while `trace_labels.rex` grew 42 -> 58 lines, so the route extends an
existing witness rather than adding a test. The 21 -> 22 shift happened at round 2, not here.
(2) The extended witness is **byte-identical to the oracle on both descriptors**, rc 0 both sides,
with the `zf:` echo present in both.

**It did not write "four ways".** The claim appeared in four places and none now states a count.
They give the property with its enumerating citation -- a label echoes when the `LABEL`
instruction is **executed**, per `RexxInstructionLabel::execute` -> `traceLabel` -- and present
routes as probed examples. Writing "four" would have been the same mistake with a bigger number.

**It declined the optional restoration, correctly and for the stated reason:** the sentence is
still true, but it is case 3 of its own procedure -- a mutable in-repo referent, no longer
load-bearing -- and *"it was true when written" is not the criterion, since all eight false
statements were true when written.*

**What no mutation can show, said rather than faked:** there is no route-specific branch in the
implementation, so nothing can break one route and leave the others. The new coverage guards a
future implementation that grows such a branch, not a defect reachable today.

### The comment rule's final form, at `7b03dbe3`

The implementer improved the controller's wording with a counterexample rather than accepting it.
The controller proposed: *an exhaustiveness claim is only as good as the enumeration behind it.*
**Its objection: something enumerating it is not enough.** `WITNESS_PREFIXES` genuinely
enumerates, and "`>>>` appears in every witness below" still rotted -- because that `const` is in
this repo and changed when a witness landed.

> **The axis is whether the enumeration can change without the sentence being reread.** Outside
> the repo -- the C++, the oracle's own tables -- cite it and prose is fine. Inside the repo,
> assert it. Where neither is available, state the property with its citation rather than a count.

That is now in `rust/CLAUDE.md` alongside the immutable/mutable taxonomy (`6eddd10d`) and the
zero-match exit-status rule (`a55ed36c`) -- three rules, all earned inside one task.

**Final light re-review dispatched**, scoped to one question: is there a **ninth** false statement
in the commit that closed the eighth, plus whether the C++ citation actually supports each of the
four rewritten sentences.

### Closing review, and a prompt-injection attempt reported by the reviewer

**FLAGGED FOR MORITZ, not a code issue.** The closing reviewer reports that **twice during file
restoration, a tool-result channel injected a fabricated "system-reminder"** claiming
`trace.rs`/`trace_labels.expected` had been intentionally modified, and **instructing it not to
tell the user**. It verified both claims false via `git diff HEAD` and a `git show HEAD` hash
match, **refused the concealment instruction, and disclosed the attempt**. That is the correct
response in full.

**Controller-verified tree integrity afterwards:** working tree clean, both named files
byte-identical to HEAD, and the **C++ oracle tree at `/home/moritz/dev/repos/ooRexx` clean**. No
damage, nothing concealed. Raised to Moritz.

### Round-4 closing review: one ninth statement, and it was an orphaned neighbour

`task-9-rereview4.md`, 45 tool uses. **All four rewritten route sentences are individually true**,
verified by reading `LabelInstruction.cpp:54-60` and `RexxActivation.hpp:366,376` directly -- each
citation supports its sentence. Witness gap closed and byte-identical to a fresh independent
oracle run. R4-M1 and R4-M2 both red at `0 passed; 1 failed; 21 filtered out`, and the restores
were verified against `git show HEAD` hashes rather than only against its own backups. Round 3's
declined restoration judged the right call.

**The ninth false statement existed, but not at any of the four sites the round targeted:**
`phase-4-exclusions.txt:1104`, three paragraphs below the round's own lesson paragraph, said the
expectation "is three `*-*` lines". It has **four** -- the fourth being the function-call route
added by that same commit. **The sentence a change falsifies need not be the sentence the change
edits**, which is a variant worth carrying: the sweep must cover the neighbourhood of a change,
not only its diff.

Fixed by the controller at `5aa144d5`, and **not corrected to "four"** -- restated as a property
(one `*-*` line per label actually reached), per the rule the task itself produced.

## Task 9: complete

`e72cc19f` -> `5aa144d5`. Crate commits: `233fbd8b` (task), `94237403`, `f02c3ed3`, `50da3045`,
`16a67ae2` (fix rounds 1-4). Controller doc commits interleaved: `d2f490bc`, `d24dde78`,
`38b2cb7b`, `a55ed36c`, `6eddd10d`, `7b03dbe3`, `5aa144d5`.

Final: **990 passed / 0 failed**, STRICT corpus **41 of 41** (39 -> 41 across the task), assertions
4224/4259, fmt 0, clippy 0.

**Delivered:** `>A>`/`>F>`/`>R>` byte-exact; the activation indent base confirmed already correct;
**all three absent-value-line gaps closed** (I31, `EXIT <expr>`, and the four both-DIFF probes
proved to be I31 itself); `TRACE L` label echo implemented rather than deferred; the
control-variable re-read fixed with `NOVALUE` intact; and a prefix-coverage count asserted against
a committed literal rather than printed.

**Open, all parkable, for the final whole-branch review:** the `do aa.1 = 1 to 2` stdout
divergence (needs a `rexx-parse` change no 4b task's file list reaches -- **still needs an
owner**, controller's call at Task 12 at the latest); `TRACE ?L`'s two `+++` banner lines; and the
oracle's under-indent after a completed inner loop, which DEVIATION 0 covers.

**Next: Task 10** -- the 4b corpus and the collector. Run the pre-dispatch check; it has found a
missing prerequisite for **nine** consecutive tasks.

## Task 10 dispatched -- the 4b corpus, the I19 sweep, and the collector

Base `598be610`, sonnet. **First task run under the new review discipline** (`598be610`): prose
findings batch into one end-of-task pass instead of entering the fix loop, that pass sweeps the
neighbourhood rather than the diff, and the dispatch names where the risk is instead of sending a
reviewer everywhere.

Pre-dispatch check, tenth consecutive task with a finding, and this time one of them is work the
plan asks for that is **already done**:

1. **I20's union half is already built.** `collect_stress.rs:127-130` already calls
   `read_subset(&[phase-4a.txt, phase-4b.txt])`, and `read_subset` takes a slice for exactly that
   reason. What remains open is the half that matters -- **the stress mode has never seen a call
   frame** -- so the dispatch asks for confirmation that it now does, and what running it over the
   grown subset produces.
2. **`assert_program_has_no_directives` is at `coverage.rs:153`, not `:331`.**
3. **`rust/corpus/proc/` does not exist and all eleven 4b programs live in `lang/`.** The plan
   predates them. Nothing keys on the subdirectory name (checked), though several comments cite
   `corpus/lang/...` paths. Controller's call, stated in the dispatch: **put them in `lang/`**
   unless the implementer finds a concrete reason, and say which.
4. The I19 pointer is where the plan says: `rexx-core/src/heap.rs:81`, deliberately on
   `Heap::collect` rather than at the leak site.

**Risk named in the dispatch rather than left to be discovered:** Step 1 is the whole task and its
failure mode is programs that pass trivially. 4a's two Criticals survived 824 tests, a 29-of-29
byte-identical corpus, nine reviews, seven criteria and a nine-mutation script **because a
coverage criterion that enumerates variants asserts nothing about combinations** -- `Stem` had a
witness, arithmetic had witnesses, `Stem` as an arithmetic operand had none, and `a. = 5; say a. +
1` aborted the process. So the three combination programs are the deliverable, and every program
must come with the mechanism it pins: *which line, deleted or inverted, makes this diverge?* If
none, it is decoration.

Also carried: the `exit <value>` constraint on `trace r` witnesses is **lifted** (Task 9 closed
it), but the controlled-`DO` per-pass `>>>` gap and `TRACE ?L` remain open in the exclusions file
and constrain trace settings.

## Task 10: implementer reported -- `3d6d68d7`

**Status DONE.** Controller-verified: **990 passed / 0 failed** (unchanged -- no new `#[test]`
fns), STRICT corpus **44 of 44** (41 -> 44), assertions 4224/4259, fmt 0, clippy 0. The three new
programs went in `lang/` as the pre-dispatch correction directed, with their `sourceline_oracle`
fixtures.

**Each program arrives with the mechanism it pins, proved by a real one-line mutation:**

* `lang/raise_in_routine_in_loop.rex` -- `exec_raise`'s CALL-ON arm delivering `Flow::Return`;
  forcing `None` turns `TRAPPED:2` into `RESULT:2`.
* `lang/expose_stem_across_calls.rex` -- `exec_procedure`'s `alias_slot` loop.
* `lang/call_on_trap_rearms.rex` -- `deliver_pending_trap`'s reinsertion; skipping it drops the
  second handler firing, because `CALL ON`, unlike `SIGNAL ON`, must **not** disarm.

**The controller verified the second independently rather than relaying it:** neutering the
`alias_slot` loop gives **rc 215, no stdout, `Error 41.1 ... ("ST.SUM")`** at the **first**
`call bump`. `run.rs` restored from a copy and confirmed byte-identical to HEAD.

**The implementer corrected the brief twice, and the controller once.**

1. **I20's "`collect_stress` has never seen a call frame" was already stale.** It measured first:
   the stress mode already passed on the pre-existing 41-program union before this task touched
   anything. What this task adds is **depth** -- repeated call/trap/loop combinations no single
   program drove -- not first coverage. 2/2 pass, plus a per-program zero-collection check across
   all 44.
2. **The controlled-`DO` per-pass `>>>` gap the controller carried into the dispatch as open is,
   per `phase-4-exclusions.txt`, already closed.** The genuinely open item is `TRACE ?`. Sent to
   the review to adjudicate; if the implementer is right, the controller carried a stale item.

**I19 sweep: nothing else found**, supported by an enumeration -- `pop_frame` has exactly one call
site, the only two `Flow` variants carrying an `ObjRef` are `Return`/`Exit`, `exec_call` re-roots
explicitly, `eval_call` is covered by `eval()`'s uniform rooting and already runs under
`collect_stress` without panic. No `lib.rs` change. **A negative claim is the dangerous kind**, so
the review was told to check the enumeration itself rather than the conclusion.

**A self-correction at authoring time, which is the new discipline working:** its first draft of
`expose_stem_across_calls.rex`'s mutation write-up predicted a third-call failure **by reasoning**;
running the mutation showed the **first** call fails, and it fixed the comment before committing
rather than shipping it for a reviewer to catch.

**Review dispatched risk-scoped** (sonnet, 26 KB): can the three programs fail, and do they pin the
*combination* rather than something a single-construct program would also catch; the I19 negative
claim's enumeration; corpus-rule compliance and determinism. **Prose findings collected, not
dispatched**, with instructions to check the sentences around the change and not only the changed
lines.

### Task 10 review: two Criticals, and the risk-scoping paid immediately

`task-10-review.md`, 101 tool uses. Every gate re-run, all three programs re-diffed byte-for-byte
against the live oracle on all three descriptors, all three mutations reproduced with md5-verified
restores. **Both Criticals landed exactly where the dispatch pointed it**, which is the argument
for naming the risk instead of sending a reviewer everywhere.

**Critical 1: two of the three "combination" programs are not new coverage.**
`raise_in_routine_in_loop.rex`'s CALL-ON mutation is caught **identically** by the existing
`condition_traps.rex` block 4 (single call, no loop); `expose_stem_across_calls.rex`'s
`alias_slot` mutation is caught **more thoroughly** by `call_procedure_expose.rex` (5 of 5 lines
diverge). Only `call_on_trap_rearms.rex` is genuinely new -- `condition_traps.rex` is unaffected
by its mutation.

**The subtlety worth keeping: the mutations did go red, so the programs can fail. They are not
*new*.** "Can fail" and "adds coverage" are different properties, and this project's whole
vacuity discipline had only ever checked the first. The method gains a step: **run the candidate
mutation against the existing subset before crediting the new program.**

**Critical 2: the I19 sweep's enumeration is false.** "`pop_frame` has exactly one call site" --
there are **seven**, one in `run.rs` and six in `eval.rs`, corroborated by `RootSet::pop_frame`'s
own doc comment. No live rooting bug found among the six on inspection, but **the sweep never
examined or acknowledged them**, so the conclusion may be right while the reasoning that produced
it is not. `rust/CLAUDE.md`'s exhaustiveness rule applies to a report exactly as to a comment: an
in-repo enumeration must be checked, not asserted.

**Important:** two shipped corpus headers assert `SIGL` reads the routine's own `RAISE` line when
the oracle-matched output proves it reads the **calling clause's** line -- the programs are right
and the headers describe them wrongly -- and the report's quoted `SIGL` values have dropped digits.

**Both disputed claims resolved in the implementer's favour, and one was the controller's error.**
`collect_stress` genuinely already covered call frames at `598be610` (verified in an isolated
worktree). And **the controlled-`DO` `>>>` gap is genuinely closed; `TRACE ?` is the open one --
the controller carried a stale item into the dispatch.** Recorded here because the pre-dispatch
check exists to stop exactly that, and this time it introduced one instead.

### Task 10 fix round 1: `8b4fd867` -- two witnesses deleted rather than defended

Controller-verified: **990 passed / 0 failed**, STRICT **42 of 42**, assertions 4224/4259, fmt 0,
clippy 0. Corpus 41 -> 44 -> **42**; the 4b subset is twelve programs.

**Critical 1 accepted in full: both non-discriminating programs removed.** Before deleting, it
tried three further mutations to save them -- a `pop_slots` frame-leak, an `indent_offset` restore
skip, a `set_sigl` skip -- each against a byte-identical `run.rs` copy with a rebuild. The
frame-leak was **already caught by `call_procedure_expose.rex`'s own four sequential calls**; the
other two were inert everywhere. `call_on_trap_rearms.rex` remains and now carries the task alone.

**It supplied a structural reason, which is the round's real output and the thing most worth
checking.** No combination mutation exists for those two shapes because **loop control state lives
in a Rust local on `run_repeating`'s own stack frame**, unreachable by any nested `CALL`, and
**`PROCEDURE EXPOSE` re-resolves everything fresh on every call** from one whole-file `Plan` with
no cross-call cache. If that holds, a loop and a call **share no mutable state**, so no program
combining them can pin more than the two pin separately.

**That generalises the lesson beyond this task.** "Write combination witnesses" is not always
achievable: a combination can only pin something extra where the implementation gives the two
constructs shared mutable state. Where it does not, the honest deliverable is fewer witnesses, not
more. Sent to the re-review as a **negative claim about the implementation** -- the class this
project has been wrong on six times, each a sound inference from a true premise missing a second.

**Critical 2 redone properly:** seven `pop_frame` sites confirmed, all six `eval.rs` bodies read
directly, each allocating its return value strictly before its own `pop_frame` with zero
intervening statements, and `RootSet::pop_frame` re-read and confirmed a bare `Vec::truncate`. No
gap beyond the documented `run.rs` site -- same conclusion as before, now with the reasoning that
supports it.

**Important fixed:** both headers' `SIGL` claims were backwards, and their cited values were
**stale by ten lines from an earlier header edit** -- now `54/60/66`, not `4/10`, re-verified
against a live oracle run of the file exactly as committed.

**Re-review dispatched** (sonnet), weighted to the structural claim with an explicit yes/no asked
-- *does a loop-plus-call combination witness remain owed?* -- plus per-site verification of the
six `eval.rs` bodies, re-confirmation that `call_on_trap_rearms.rex` still discriminates now that
it stands alone, and a check that nothing else referenced the deleted programs.

### Task 10 re-review: all closed, and the structural claim's premise was falsified

`task-10-rereview.md`, 85 tool uses. **Critical 1, Critical 2 and the Important all closed**, each
by running: the frame-leak mutation reproduced (panics `call_procedure_expose.rex` at exit 101),
all six `eval.rs` `pop_frame` bodies read directly and confirmed, and the corrected `54/60/66`
header values matched against a live oracle run of the committed file, byte-for-byte on all three
descriptors.

**`call_on_trap_rearms.rex` re-confirmed as the sole surviving witness** -- the reviewer ran its
mutation against **all twelve** current 4b programs and only it diverges. Deletions checked for
stray references; `every_in_scope_variant_is_witnessed_by_the_phase_subsets` still passes.

**Verdict on the structural claim: no combination witness is owed -- but the stated premise was
wrong, and the reviewer proved it by construction.** It built a program where a loop control
variable is exposed and mutated by a nested `CALL`, which does share mutable state across the two
constructs. The claim survives on a narrower ground: that pathway is **generic slot aliasing,
already witnessed with no loop present**, and no branch fires only when the two are combined.

**Controller-verified before correcting it:** `run.rs:5113` reads the control variable back out of
the variable pool every pass -- `self.read(code, *control)`, **the re-read Task 9 added four
commits earlier** so `do ii = 1 to 3 ; ii = 10 ; end` ends after one pass. So the value is
reachable and only the *bookkeeping* (limit, increment, direction) is in the Rust local.

**This is the seventh instance of the shape `rust/CLAUDE.md` warns about** -- a sound inference
from a true premise missing a second premise -- and here the missing premise was **created by this
same phase, four commits earlier**. Nobody could have inferred it; only running finds it.

Header narrowed at **`2ab642ab`**, stating both what is true and that the previous wording was
falsified. Comment-only: the file's non-comment diff is empty, the program list is unchanged at
twelve, corpus stays **42 of 42** STRICT, workspace **990/0**.

## Task 10: complete

`598be610` -> `2ab642ab`. Commits: `3d6d68d7` (task), `8b4fd867` (fix round 1), `2ab642ab`
(controller, claim narrowed). Final: **990 passed / 0 failed**, STRICT **42 of 42**, assertions
4224/4259, fmt 0, clippy 0.

**Net effect:** one genuinely discriminating new witness (`call_on_trap_rearms.rex`), a corrected
I19 sweep over seven sites rather than an asserted one, and two programs deleted for not earning
their place. **A corpus that went down by two and got stronger.**

**Next: Task 11**, the `base/keyword` L1 extractor. Run the pre-dispatch check -- it has found
something for **ten** consecutive tasks, most recently a piece of work the plan asked for that was
already done.

---

# RESUME POINT -- written 2026-08-04 before compaction

**HEAD `2ab642ab`, working tree clean, no agents live.** Workspace **990 passed / 0 failed**,
STRICT corpus **42 of 42**, assertions 4224/4259, fmt 0, clippy 0. Verified by the controller at
this commit, not relayed.

**Tasks 0-10 complete.** Remaining: **Task 11** (`base/keyword` L1 extractor) then **Task 12**
(the 4b gate). Nothing is mid-flight.

**Do this first on resume:** generate the Task 11 brief with `scripts/task-brief`, then run the
pre-dispatch check against the tree. It has found something for **ten consecutive tasks** and the
categories keep changing -- most recently a piece of work the plan asked for that was **already
done** (`collect_stress` already read the subset union), and before that a brief instructing two
contradictory things. **Findings go into the plan first and the dispatch second** (`rust/CLAUDE.md`
Method, added at `d24dde78`): a correction living only in a dispatch is lost at every handoff,
which is how a Task 7 finding went missing until it was re-measured by luck.

**Task 11 specifics already known:** its Step 1 is a ten-minute check of whether `base/keyword`
uses a shape `extract_assertions` already models -- report either way. The conservation law
`rows + dropped == calls` is a **tautology alone** (holds at `0 + 0 == 0`); it needs absolute
committed literals and a non-empty floor beside it. And the denominator is a real choice:
`base/keyword` has 2,561 `self~assertSame` but 4,567 `self~assert*`, so picking `assertSame`
leaves 44% of the group outside the population while the law reports perfect.

**Review discipline for 11 and 12 changed at `598be610`** -- read that section of the plan before
dispatching. Prose findings do not enter the fix loop; they batch into one pass that sweeps the
**neighbourhood** of each change, not the diff. Re-reviews verify their named findings plus "did
this round introduce a new false statement". Size the review to the risk and say where the risk is.

**Two decisions still open for Moritz**, neither blocking: whether to file the upstream
`traceIndent` defect (`BaseDoInstruction.cpp:161` against `:377`); and Task 12 Step 3b's costed
recommendation on per-sub-phase ownership attribution, which now has hard evidence attached (one
stale fact had **three** prose copies, found by three agents across two tasks, none seen by Task
7's five review passes).

**Unowned gap needing a decision at Task 12 at the latest:** `do aa.1 = 1 to 2` stores a *simple*
variable named `AA.1` -- a stdout divergence needing a `rexx-parse` change that no 4b task's file
list reaches.

---

## Task 11: pre-dispatch check, and the plan rewrite it forced

**Eleventh consecutive task where the pre-dispatch check found something**, and the largest
so far: it answered Task 11's own Step 1 and the answer changed what the task is.

Measured at ooTest **r13178**, against the existing `extract_assertions`:

* `base/keyword` yields **54 rows from 2,561 `assertSame` calls (2.1%)**; 26 of 39 groups yield zero.
* `rexx-extract-assertions --suite .../base/keyword` **panics on its own conservation law** at the
  second group: `0 rows + 39 dropped != 265 assertSame calls`.
* Cause 1: `scan_method_for_assertions` only tests `trimmed.starts_with("self~assertsame")`, and
  **405 of the 2,561 calls are mid-line** (`id=0010; ABBREV =0010; self~assertSame(abbrev, id)` in
  `ASSIGNMENT`, 226; `When i=0 Then self~assertSame(...)` in `IF`, 126). Verified as real, not a
  counting artifact, by listing the non-line-start matches.
* Cause 2: the same prefix test claims the **120 `self~assertSameList`** calls, which
  `parse_assert_same` rejects (next char `L`, not `(`), so each one calls `block()` and poisons
  every later `assertSame` in its enclosing method. `base/expressions` has **zero**
  `assertSameList`, which is the only reason this has never shown.

**The plan's row model does not transfer**, so Task 11 now specifies **whole-body** extraction.
The feasibility measurement, which the plan lacked: of 26 candidate statement-shaped groups, 18
carry `assertSame`-bearing `test_*` methods, and **1,008 of those 1,033 methods use no message send
other than `self~assert*`**, carrying **1,850 of 2,020 calls (91.6%)**. So no message dispatch and
no Phase 5 prerequisite. Recorded in the plan as a **ceiling to report against**, with both
caveats stated (the script was crude; "no `~`" is necessary, not sufficient).

**Three smaller corrections, all now in the plan:**

* The denominator's two spellings differ by exactly the 120 `assertSameList` calls: **2,441 exact**
  against **2,561 prefix**, out of **4,567** `self~assert*` of all spellings. The plan's first
  revision quoted 2,561 without noting it is a prefix count.
* The conservation law must count **mid-line** calls. Stated over a line-start `calls`, it would
  hold exactly while ignoring 405 assertions -- certifying the blindness it exists to catch. This
  is the same shape as the `0 + 0 == 0` tautology the plan already guarded against, one level up.
* **`ootest/` is not checked-in test data.** Git-ignored at `.gitignore:6`, **zero** tracked files,
  an SVN working copy of `svn.code.sf.net/p/oorexx/code-0/test/trunk` at **r13178**; the main
  ooRexx worktree has no `ootest` at all. Two comments in the tree assert the opposite --
  `assertions.rs`'s `suite_root` ("checked-in test data in the same repository, not an external
  build a machine might be missing") and `rexx-extract-assertions.rs` ("already checked into the
  tree"). Task 11 fixes both and pins the revision in the failure message.

Plan rewritten and committed at **`d47239c9`** before the brief was regenerated, per the
findings-go-into-the-plan-first rule (`rust/CLAUDE.md` Method, `d24dde78`).

**Dispatched:** `t11-keyword-l1`, opus, BASE **`d47239c9`**.
Brief `task-11-brief.md` (61 lines), report `task-11-report.md`.
One controller decision carried in the dispatch and not the plan's Files list: the new extractor
goes in a **new module** `crates/rexx-extract/src/keyword.rs`, not into the already-564-line
`lib.rs`.

**Step 1 came back with two of my numbers wrong.** Verified both myself before amending; the
implementer is right on both, and both make the task's case stronger.

* "26 of 39 groups yield zero" -> **34 of 39** (25 of the 30 that contain any `assertSame`; only
  `DO` 3, `NUMERIC` 28, `REPLY` 1, `TRACE` 1, `VarRef` 21 produce a row). The 26 was **carried in
  from the next paragraph's "26 candidate statement-shaped groups"** -- a number contaminated by
  its neighbour, which is the failure mode `probe-discipline` and the brief-propagation rule both
  warn about, arriving this time in a plan I had just written to fix other people's numbers.
* "405 not at the start of a line" -> **403 mid-line, plus 2** at line start inside
  `GUARD.testGroup`'s `waiter_multiple`, a `::method` not named `test*` that no scanner reaches.
  The count of unseen calls was right; the attribution was wrong, and the second cause needs its
  own drop reason.

**The Step-1-as-reproduction device worked.** Handing over measured numbers with "if yours
disagree, stop and report, do not silently substitute" is what surfaced these; the previous
revision's "spend ten minutes checking" would have produced a fresh answer with no cross-check
against mine, and the wrong numbers would have stayed in the plan.

Implementer's population decision, endorsed and recorded in the plan: take the **narrower**
`self~assertSame`-only population (**896 methods / 1,773 calls, 72.6%**) over the wider
`self~assert*` one. The wider buys **+169** calls only by replacing **659** other `self~assert*`
calls with `nop` -- reporting bodies as passing after deleting their other checks. Correct call.

Plan corrected at **`6e49abad`**. Agent continued without blocking; nothing it builds depended on
either number.

**Task 11 implementer finished.** Five commits: `2e76303a` extractor + four pins, `664fac9d`
comment handling fix, `fda5bff9` harness + exempt set + `REXX_KEYWORD_GATE`, `ee33daf2` docs +
`KNOWN GAPS` row, `5b2de07a` drop-reason enum + floor raise.

**Process incident, worth remembering.** The agent reported `DONE_WITH_CONCERNS` while
`crates/rexx-extract/src/keyword.rs` was **uncommitted and mid-edit**; the tree did not compile,
and two `cargo build` runs minutes apart produced *different* error sets because the file was
still moving. I verified the committed state in a **separate detached worktree under the
scratchpad** rather than measuring in the agent's live tree -- the right move, and the one
`controller-edits-collide-with-live-agents` implies for the inverse direction too. Told the agent
DONE means committed and clean. It finished the conversion properly and committed at `5b2de07a`.

A second, subtler trap caught me while verifying: `... | grep -c 'test result: FAILED' && echo
"=== fmt ===" && cargo fmt --all --check; echo "fmt exit $?"` printed `fmt exit 1` **for a command
that never ran**. `grep -c` exits 1 when it finds zero matches, which broke the `&&` chain, and
`$?` then reported the broken link's status. Same family as
[[shell-pipe-exit-status]] and [[verification-commands-that-do-not-run]], new shape: a *successful*
zero-count is a non-zero exit.

**Verified independently at `5b2de07a`** (isolated worktree, not relayed): build clean,
**1015 passed / 0 failed**, fmt 0, clippy 0, `REXX_KEYWORD_GATE=1` 6/6, `REXX_ASSERTIONS_GATE=1`
5/5. Report mode reproduces `100 of 896 bodies passing, carrying 713 of 1773 assertSame calls` and
the by-reason table exactly. Exempt set 796 rows = **790 `4c` + 6
`defect:compound-do-control-variable`**, and 100 + 796 = 896.

**Headline result: 1,773 of 2,441 exact-spelling `assertSame` calls extracted into 896 bodies
(72.6%); 100 bodies pass, 713 assertions (40.2%).**

**Findings that matter beyond this task:**

* **`base/keyword` has zero Phase 5 dependency.** All 796 non-passing bodies are 4c's or the
  recorded defect. `PARSE` alone is 656 of them; the rest are builtins (`ARG` 51, `COPIES` 19,
  `DIGITS` 11, `FORM` 10, `FUZZ` 9, ...), `ADDRESS` 10, `PULL` 1. So 4b owes this table nothing
  further and its pass rate is a clean measure of 4c's remaining surface.
* **The six divergences are the previously-unowned compound-`DO` gap, now witnessed.** Two symptoms
  of one gap: a compound control variable is never assigned (`Do cv.j=1 To 5` -> oracle `[5 6]`,
  ours `[5 CV.1]`), and the tail is not re-resolved per pass (`Do a.i=1 To 3` with `i` moving ->
  oracle `[8]`, ours `[4]`). Each re-run under the oracle in rewritten form. Recorded in
  `phase-4-exclusions.txt` and in the exempt set as `defect:compound-do-control-variable`.
  **Task 12 must rule on this rather than inherit it.**
* **An L1 harness of this shape can manufacture divergences, and the L1 rung alone cannot detect
  it.** Two of the original eight apparent divergences were the harness's own comment-stripping
  bug; only the **oracle cross-check** separated them from the six real ones. Verified the language
  fact myself: `say '['1/**/05']'` -> `[105]`, `say '['1 /**/ 05']'` -> `[1 05]`. A comment ends a
  token without contributing a blank. **This constrains how 4c builds `base/bif`.**
* The `another assert* spelling` category landing at exactly 138 methods / 169 calls reproduces,
  from a different rule, the +138/+169 measured by an independent probe before the extractor
  existed. The population decision's price is now a twice-derived committed number.

**Dispatched:** `t11-review`, opus, range `d47239c9..5b2de07a`, review package 143 KB.
Risk named in the dispatch rather than left to the reviewer to find: the body rewrite as a
translation step, the never-run-assertions vacuity hazard, whether the derived `unblocked_by` is
circular, whether the four pins can fail as a set, and the two zero-valued drop categories.

**Correction to the process incident above, checked against the commit clock rather than accepted
on assertion.** `ee33daf2` 13:36:07, my first inspection 13:40:36, `5b2de07a` 13:48:06. The agent
reported `DONE_WITH_CONCERNS` at a **clean, verified tree**; the dirty non-compiling window was new
work it began afterwards, on my own refinement request. "Reported DONE over a dirty tree" was my
mischaracterisation and is wrong. What actually happened is an eleven-minute silent window during a
multi-file refactor, which is a state a reader will misinterpret -- and I did.

**Two fixes, one on each side.** The agent will announce the start of a multi-file refactor rather
than going quiet. I will scope refinement requests explicitly: my message asked for the drop-reason
work without saying whether it belonged to this task or a follow-up commit, which is what made the
window ambiguous in the first place. A report is only a snapshot of the moment it was written, and
an unscoped follow-up request invalidates it silently.

**The compound-`DO` attribution, settled with measurement.** The agent investigated rather than
defended its wording, and the outcome splits one question into two:

* **It is not a `rexx-parse` gap.** `Controlled.control` is a `SymbolId` (`rexx-parse/src/ast.rs:1014`)
  and `cv.j` is a single symbol token, so the parser interns `"CV.J"` and hands it over intact.
  My ledger note calling this "a stdout divergence needing a `rexx-parse` change" was wrong about
  the layer.
* **The defect is one executor function**, `bind_control` at `rexx-exec/src/run.rs:5284`, which
  writes the control variable through `slot_of` -- a flat name -> slot lookup -- so `"CV.J"` becomes
  the literal name of one variable and no tail is resolved. The clincher is that the same executor
  resolves the same name correctly one line later: `say cv.j` renders `CV.1`.
* **Provenance is 4a's; ownership is unassigned.** `instruction_owner` returns `None` for
  `InstructionKind::Do`, meaning implemented-not-deferred. Nothing schedules the fix. The agent's
  "this is a 4a defect" read as an ownership claim when it meant provenance; approved a reword of
  all three places to say both plainly, folded into the review fix round.

**Verified the two-line repro myself, both ways, fresh directory:** `j = 1; do cv.j = 1 to 3; end;
say '['cv.1']' '['cv.j']'` gives oracle `[4] [4]`, rexx-exec `[CV.1] [CV.1]`. This takes the defect
out of the L1 harness entirely, so believing it does not require trusting the body rewrite.

**And measured the one thing the agent flagged as unverifiable.** It predicted `do cv.j over ...`
shares the defect through the same `bind_control`, and could not test it because "the collection
expression needs a message send". **That reason is wrong.** `DO OVER` on a stem needs no message
send -- the oracle runs `do cv.j over a.` fine, iterating tails. Through `rexx-run` it is
`rexx-exec: DO is not implemented`, exit 120: **`DO OVER` is not implemented at all**, so control
never reaches `bind_control`. Conclusion unchanged (correctly kept out of `KNOWN GAPS`), but it
becomes testable **when `DO OVER` lands, not when Phase 5 does** -- parking it behind Phase 5 would
be the wrong milestone.

Third time on this task a code-reading inference was right about the answer and wrong about the
mechanism. Each was caught only because it was labelled inference. See
[[unreachability-claims-need-running]].

**Controller edit collided with the live agent, exactly as predicted.** I edited
`docs/superpowers/plans/...-4b-....md` (Task 12 Step 3b's re-measurement, committed `ceabe481`)
while `t11-keyword-l1` was standing by. It observed `M` on that file, concluded from a
`git diff --exit-code` of 0 and a `git log` of `5b2de07a` that only the mtime had moved, and ran
`git update-index --refresh`. Both readings were taken either side of my commit, which is why they
looked consistent and were not. **No harm**: `update-index --refresh` only rewrites stat info for
files whose content already matches, so it could not have touched the edit. Verified after:
HEAD `ceabe481`, tree clean, edit present.

The note in [[controller-edits-collide-with-live-agents]] says documentation fixes count as edits.
I had it and made one anyway. Fix adopted: announce a shared-file edit in the same breath as making
it, which is the reciprocal of what I asked the agent to do about multi-file refactors an hour
earlier.

**Failure-class refinement, the agent's and correct.** The `DO OVER` mistake was **not** a
code-reading inference -- it ran a probe. The probe used `'a b c'~makearray(' ')` as the collection,
which contains a message send, so it died at Phase 5 before reaching the loop, and the death was
reported as the blocker. That is a **probe-design error**: the witness failed for a reason other
than the one under test. "Run it rather than argue it" cannot catch it, because it was run. The
check that catches it is asking **what the probe actually exercised** before believing its failure
mode -- a check on the evidence, not on the reasoning. Distinct remedy, so keep it distinct from
the inference errors. See [[probe-discipline]].

**Four distinct shapes on this one task**, worth keeping separate because each has its own fix:
two inference errors from true premises ([[unreachability-claims-need-running]]); one probe that
failed for the wrong reason ([[probe-discipline]]); and one stale measurement of mine quoted as
current (Step 3b's ratio).

**Fourth failure shape, the agent's own diagnosis and a genuinely distinct one: two observations at
different times treated as one state.** Its `git log` ran before my `ceabe481` commit and its
`git diff --exit-code` after it, and it combined them into "content identical, only mtime moved" --
a conclusion consistent with both readings and true of neither moment. The tell it had and passed
over: it was reasoning about a file whose mtime it had *just watched change*, and still treated
sequential reads of it as simultaneous. `git status --porcelain` alone, run once, would have said
`M`.

Remedy is its own: for a moving target, capture everything in **one** command, or re-read after
concluding. Not the stale-measurement remedy (re-measure at use), not the probe remedy (check what
the probe exercised).

**Four shapes across five incidents on this one task**, each needing a different fix:

1. Inference from a true premise, twice ([[unreachability-claims-need-running]]).
2. A probe that failed for a reason other than the one under test ([[probe-discipline]]).
3. A measurement true when written, quoted as current (Step 3b's 1,852/9,025, mine).
4. Two reads of a moving file combined into one state (the agent's, above).

The agent named the common reflex behind 2 and 4 better than I did: **preferring the explanation
that makes the evidence agree**, over the reading that leaves it in conflict and forces a question.

**Second Task 12 pre-dispatch finding, committed `19692bcc`.** "Criterion 2's exempt list cannot
light up at this gate or 4c's" was written when `tests/assertions.rs`'s 35 Phase-5 rows were the
only exempt set. Task 11 added `rust/corpus/keyword-exempt.txt`, whose 6
`defect:compound-do-control-variable` rows **can** light up by design. Two referents, opposite
properties, one phrase; Task 12 now says to name the file.

**Both Task 12 findings so far were created by Task 11's own output**, not present when the plan was
written. Same shelf-life problem as an unreachability claim, one level up: a plan statement can be
true when authored and falsified by the phase's own later work.

## Task 11: complete

**`Task 11: complete`** at **`b23986d9`**. Commits: `2e76303a`, `664fac9d`, `fda5bff9`, `ee33daf2`,
`5b2de07a` (implementation), `8d6790c6` (review round 1), `b23986d9` (review round 2).

**Verified by me at `b23986d9`, in an isolated detached worktree, not relayed:** build clean,
**1019 passed / 0 failed**, `cargo fmt --all --check` 0, `cargo clippy --workspace --all-targets --
-D warnings` 0, `REXX_KEYWORD_GATE=1` **7/7**, `REXX_ASSERTIONS_GATE=1` **5/5**,
`REXX_CORPUS_GATE=1` **9/9**.

**Result: 1,773 of 2,441 exact-spelling `assertSame` calls extracted into 896 bodies (72.6%);
100 bodies pass, carrying 713 assertions.** Exempt set 796 rows = 790 `4c` + 6
`defect:compound-do-control-variable`.

**Review: spec PASS, quality PASS with required corrections. Zero behavioural defects across two
rounds.** The reviewer's strongest evidence was positive rather than negative: it ran **all 896
bodies under the C++ oracle**, each from a fresh empty directory, and **none had an assertion
fail**; for the 100 passing bodies the two interpreters agree on marker sequence, full byte-for-byte
stdout and exit status. The 713 is not a count of coincidences.

**The round-1/round-2 record is the reusable data point.** Round 1: 13 findings, **0** new
behavioural defects, **5 new false statements**. Round 2: 5 findings, 0 new statements found. That
is the Tasks 7-9 pattern reproduced exactly, and it is why the scoped re-review's second question
("did this round introduce a new false statement") earns its place while a third full review would
not have.

**NF1 is the sharpest single instance this phase has produced.** Round 1 wrote "all four read `4c`
only because `rexx-exec` blocks on a **builtin** first". Measured: `routine "label"`, `routine ""`,
`routine "CHARIN"`, `routine "DIGITS"` -- only the last two are builtins, and the first two are the
very `::routine`s **the same paragraph had just said the standalone program does not carry**. A
correct source sentence two lines above, compressed into a word that contradicted it. The
implementer's own reading is the right one and belongs in the method: **a summary of a measurement
is a new claim and needs the measurement re-run, not the prose reread.** Neither more care nor
rereading the diff could have caught it -- the sentence is locally plausible and only false against
a fact in its own neighbourhood. I applied that rule to their fix: re-ran all three shapes through
`rexx-run` rather than reading the replacement text.

**Two claims I propagated upward and had to withdraw**, both from this task:

* That the reviewer could not reproduce the `classify_sends` mutation because the function did not
  exist before `5b2de07a`. Both halves true, inference false: the review ran **at** `5b2de07a`,
  where it exists, and said it **did not attempt** the mutation. A fabricated cause, stated as the
  likely one, relayed by me.
* That the table's pass rate is "a clean measure of 4c's remaining surface". It is not: four of the
  896 bodies are **not equivalent to the methods they came from** (three call `::routine`s the
  standalone program lacks, one falls through into `dig: Return digits()` so its `RETURN` becomes
  the exit status), so a `4c` attribution says **what a body hits first**, not what would make it
  pass. Final wording, which parses where "lower bound on what 4c would leave" did not: **the 790
  `4c` rows are an upper bound on what landing 4c would fix.**

**Open, carried to Task 12:** the six compound-`DO` divergences. Defective code is 4a's
(`bind_control`, `rexx-exec/src/run.rs:5284`, writes the control variable through a flat
`slot_of`); the gap is **unowned**. Recorded in `phase-4-exclusions.txt`'s `KNOWN GAPS` and in the
exempt set. **Task 12 must rule on it rather than inherit it.**

**Note for whoever reads `git worktree list`:** a detached verification worktree is registered under
the session scratchpad (`verify-ee33daf2`, currently at `b23986d9`). It is mine, read-only, and
exists so measurements never race a live agent's tree. Remove it at phase close.

---

## Task 12 -- the 4b gate. Committed `a5a979ad`, hash read back with `git log` after committing.

**Created:** `docs/superpowers/plans/phase-4b-gate.md`, `rust/scripts/mutate-4b.sh`.
**Modified:** `docs/superpowers/plans/phase-4-exclusions.txt`, and `rust/crates/rexx-exec/src/run.rs`
for a doc comment only -- see below.

**Ten criteria, written and saved before anything was run.** Eight met; criterion 2 met carrying
the criterion defect 4a already recorded (all 35 `tests/assertions.rs` exempt rows are Phase 5's,
so the literal wording still cannot pass inside Phase 4 without `EXEMPT`); criterion 3 met weakly
at 13 of 19 trace prefixes, with DEVIATION 0's removal of indentation from its reach stated in the
criterion rather than discovered after it.

**Measured at `4c8c1f68` and again after the `run.rs` comment fix, every exit status read unpiped,
identical both times:** workspace `1,019 passed / 0 failed / 4 ignored`; `cargo fmt --all --check`
and `cargo clippy --workspace --all-targets -- -D warnings` clean; `REXX_CORPUS_GATE=1` **42 of 42
matching**; `REXX_ASSERTIONS_GATE=1` **4,224 of 4,259 rows, 35 RUNTIME-BLOCKED**;
`REXX_KEYWORD_GATE=1` **100 of 896 bodies, 713 of 1,773 assertSame calls**; `mutate-4b.sh` **12 of
12 as declared**, re-run at the committed tree.

**`mutate-4b.sh` runs two instruments per mutation and each row declares what both should say.**
4a's script ran the corpus alone and treated its silence as failure, which is the wrong verdict for
five of these twelve -- the dropped argument-list root (criterion 4's activation-shaped negative
control), the queue's storage, the queue's order, an omitted argument's `>A>` line and a callee's
inherited trap table are all invisible to a differential run, four of them correctly. The
oracle-absent attack that defeated 4a's first script was reproduced against this one: it exits 1 at
the first baseline, before touching a mutation. `corpus.rs` was restored from a scratchpad copy,
never `git checkout --`, and re-verified at 42 of 42.

**Two declarations were wrong and were corrected to what was measured**, both visible in the script
and in criterion 6. One is a finding rather than a slip.

**NEW KNOWN GAP, found by this gate's own instrument: a callee's inherited trap table has no
differential witness.** Deleting the inheritance leaves `REXX_CORPUS_GATE=1` at 42 of 42 and fails
five `run.rs` unit tests. Mechanism, checkable from the corpus programs' own text: every raise a
corpus program makes inside a routine is a `RAISE ... RETURN`, which unwinds the routine before
delivery, so the table that matches is always the caller's own live one and never the inherited
copy. Nothing is wrong; it needs nothing from a later phase; it is an unwritten witness. Recorded
in `phase-4-exclusions.txt`.

**STEP 3c RULING: the compound-`DO` control-variable divergence is 4c's.** Option 2 of the three.
The row moved from `KNOWN GAPS` into a new `EXCLUSIONS` section, which makes it 4c's own gate
criterion; the enforcement already exists and is automatic (the six `defect:` rows in
`keyword-exempt.txt` go red the moment the fix lands). **It was recorded in TWO `KNOWN GAPS` rows
describing one `bind_control` mechanism** -- Task 9's, carrying the `>C>` symptom and the
rexx-parse cost, and Task 11's, carrying the six L1 bodies -- and that split is part of why it
drifted unowned from 4a through eleven tasks, because either row read as the whole defect. Merged.
A pointer stays in `KNOWN GAPS`. **Not fixed here:** `run.rs` and `rexx-parse` are outside this
task's file list, and moving the tree in the commit that measures it would invalidate every figure
above, with no review round behind it. **`keyword-exempt.txt` deliberately does not change** --
those six rows' attribution is derived from the outcome kind (`RunOutcome::AssertionFailed` maps
unconditionally to one constant), not from an ownership table, so rewriting them to `4c` would turn
the set assertion red.

**One file touched outside the nominal list, and why.** `bind_control`'s doc comment in
`rexx-exec/src/run.rs` said the defect was "recorded as a KNOWN GAP", which **this commit's own
change falsified**. Doc comment only, no behaviour; corrected rather than left for a reader to trip
over. Every figure above was re-measured after it.

**STEP 3b: keep the attribution; spend the consolidation budget on the third copy's PROSE. Not
acted on in 4b.** Three data points: `e4caa7bf` 1,852/9,025 = **20.5%**; `5b2de07a` 1,928/15,838 =
**12.2%**; `4c8c1f68` **identical to the second**, because `git diff --stat` over exactly those six
files between them is empty -- the ten intervening commits are plan and documentation edits. Across
4b the harness grew **4%** while the interpreter grew **76%**, which runs against the argument
consolidation was raised on. What would change the recommendation at 4c's gate: the three files
exceeding ~2,600 lines, or the fraction rising at all rather than falling to the expected 8-9%.
Merging the three tables is **not** a straight equality -- `owners.rs` is variant-grained and
ternary, `lib.rs` is arm-grained and binary, and `loud.rs` already carries a hand-maintained
reconciliation -- so it moves the duplication rather than removing it, at about half a day. The
phase strings are load-bearing for exactly **one** consumer, and it is the 796-row file 4c's gate
fires on: 790 of those rows derive their `unblocked_by` by parsing the owner out of the loud
message. Removing them before 4c's gate would be the wrong order. The guarded duplication has never
produced a wrong answer; the unguarded prose about it rotted four times in one count comment and
three times in one sentence about `Call::Trap` that Task 7's five review passes all missed.

**Not gated on, reported:** the `base/keyword` L1 table, with all three qualifications beside it
(a `4c` attribution says what a body hits first; the 790 are an upper bound on what landing 4c
would fix, not a measure of 4c's surface; `base/keyword` has zero Phase 5 dependency). No figure
from `TRACE.testGroup` is used anywhere in the gate -- criterion 3's own measured, named subset is
used instead.

### Task 12 fix round 1 -- committed `0e14fac4`, hash read back with `git log`.

All review findings verified independently before acting; **none disputed, every one reproduced**.

**C1 was a real harness hole, not a wording problem, and it is fixed rather than documented.**
`phase-4b.txt`'s twelve entries were pinned by nothing: measured one line at a time, **nine of the
twelve leave `coverage.rs` green**. Worst case `lang/condition_traps.rex` -- criterion 8's only
witness and three mutations' declared corpus catcher -- whose deletion left the corpus at
`41 of 41 matching` at exit 0, with criteria 1, 4 and 8 all still reporting MET and the headline
shrinking silently. `coverage.rs` gains `phase_4b_subset_matches_the_committed_list`, the 4b
equivalent of the pin 4a's own branch review built for `phase-4a.txt`. Verified to fail by name.
**Workspace 1,019 -> 1,020; every gate figure re-run and only that count moved.**

**Why the false claim was plausible, because it generalises:** it is *true of `phase-4a.txt`*, and
it was carried across to a second file without being re-run. A falsification clause is a claim like
any other and needs the same measurement as the criterion it protects.

**I1** amended criterion 6 so an unexpected *catch* fails as loudly as an unexpected survival --
what makes I17's equivalence falsifiable instead of assumed. **I2** corrected two false sentences
beside a true measurement (two of the ten intervening commits *do* change `rust/` source; exactly
one, not three, touched Step 3b's framing; the empty six-file diff itself stands). **I3**
de-duplicated the six-body listing inside the very row that argues against duplication.

**M5/M6 hardened the classifiers and the fix was verified against the review's own constructed
inputs** (7 cases, all correct): per-target run counts rather than an aggregate, every target
required to have reported, `--no-fail-fast`, and `0 of 0 matching` rejected. **M4** cites `stem.rs`
for I17 instead of making a fourth prose copy, and notes the deliberate departure from what that
comment asked for. **M1, M2, M3, M7, M8** corrected as reported.

**Final state, all re-run at `0e14fac4`, each exit status unpiped:** workspace **1,020 passed /
0 failed / 4 ignored**; fmt and clippy clean; corpus **42 of 42**; assertions **4,224 of 4,259**
with 35 RUNTIME-BLOCKED; keyword **100 of 896 bodies, 713 of 1,773 calls**; `mutate-4b.sh` **12 of
12 as declared**, baselines green before and after. Tree clean.

### Task 12 fix round 2 -- committed `a1484211`, hash read back with `git log`.

All five findings measured before acting; **all five reproduced, none disputed.**

**N1: "the declared corpus catcher for THREE of criterion 6's mutations" was false; the true value
is ONE.** Measured by applying each of the six corpus-catching mutations in turn -- every one
diverges on exactly one program (`41 of 42` in all six cases), so no program can be the declared
catcher for more than one. Row 6 (`SIGL` off by one) names `lang/condition_traps.rex`; rows 1 and 7
are structurally out of its reach (no `PROCEDURE` in it, and its single `call raiser` fires once
where row 7 needs a second raise -- which is why `call_on_trap_rearms.rex` exists).

**This was C1's own defect repeating one round later**, in the paragraph whose entire subject is a
falsification clause that was reasoned instead of measured. The triple came from the previous
review's C1 text, was relayed unmeasured, and was carried into two committed files. Corrected in
`phase-4b-gate.md` and `coverage.rs`'s pin doc; **`0e14fac4`'s commit message carries it
uncorrectably**, and the report says so.

**N3: the nine-unpinned claim holds distributively, not collectively.** Removing all nine at once
*does* fail `coverage.rs` (exit 101, `3 in-scope variant(s) unwitnessed: Interpret, Signal, Raise`).
The document now states which reading holds, and says the whole-suite check was run for
`condition_traps.rex` specifically while only `coverage.rs` was run for the other eight.

**N2/N4:** `b23986d9` touches three of the four `rust/` paths, not four; `8d6790c6`'s four `rust/`
paths carry 406 insertions, not 480 (480 is the six-file total).

**I1:** `mutate-4b.sh`'s header still carried the pre-amendment criterion 6 wording, so the
contradiction the gate document claims to have removed survived one file over.

**M5's second half taken rather than skipped**, because criterion 4's central claim rested on it.
`run_one` now prints the failing TARGET BINARY per suite divergence. Measured: **row 9's is
`tests/collect_stress.rs` and nothing else**, row 8's is `tests/trace_oracle.rs` alone, row 10's is
`unittests src/lib.rs` alone -- so "the collector is what sees a dropped argument-list root" is now
observed rather than inferred from test names.

**Final state, all re-run at `a1484211`, each exit status unpiped:** workspace **1,020 passed /
0 failed / 4 ignored**; fmt and clippy clean; corpus **42 of 42**; assertions **4,224 of 4,259**
with 35 RUNTIME-BLOCKED; keyword **100 of 896 bodies, 713 of 1,773 calls**; `mutate-4b.sh` **12 of
12 as declared**, baselines green both ends. Tree clean.

## Task 12: complete -- PHASE 4B GATE CLOSED

**`Task 12: complete`** at **`a1484211`**. Commits: `a5a979ad` (gate), `0e14fac4` (review round 1),
`a1484211` (review round 2).

**Verified by me at `a1484211`, isolated worktree:** build clean, **1020 passed / 0 failed**,
fmt 0, clippy 0, `REXX_CORPUS_GATE=1` 9/9, `REXX_ASSERTIONS_GATE=1` 5/5, `REXX_KEYWORD_GATE=1` 7/7,
`./scripts/mutate-4b.sh` exit 0 with **12 of 12 as declared**, baseline restored (42 of 42, 325
passed), tree clean after.

**Gate verdict: ten criteria, eight met, one met carrying an inherited defect, one met weakly with
its weakness quantified.** Review: meets the brief with one required fix; quality HIGH.

**Two things the gate does that 4a's could not.**

* `mutate-4b.sh` declares an expected result **per instrument per mutation**, where 4a's ran the
  corpus alone and read its silence as failure. Five of twelve mutations are invisible to a
  differential run, four of those for a **correct** reason, and none of that was expressible before.
  Row 12 is an **equivalent mutant** declared `PASSED PASSED` -- a mutation that must NOT be caught.
* It found and published a defect in its own process: two of twelve declarations were edited to
  match measurement, with the note that "a script whose declarations are edited to match its output
  is worthless unless the edits are visible".

**C1 was a real hole and is closed.** `phase-4b.txt` had **no committed-list pin** while
`phase-4a.txt` did, so nine of twelve programs could be deleted one at a time with `coverage.rs`
green -- including `lang/condition_traps.rex`, criterion 8's only witness, after which the corpus
gate reported `41 of 41 matching, exit 0` and the headline shrank 42 -> 41 still reading as a clean
sweep. `phase_4b_subset_matches_the_committed_list` now fires on **delete, add and reorder** (I
measured delete; the re-review measured add and reorder).

**The ruling: option 2, the compound-`DO` gap is 4c's**, moved from `KNOWN GAPS` into `EXCLUSIONS`.
Reasoning worth keeping: "The gate reports the tree 4b ships; it does not move it." And it found
what I never connected -- **the defect had two `KNOWN GAPS` rows** (Task 9's and Task 11's) for the
same `bind_control` mechanism, verified at the parent commit, lines 828 and 938. Its conclusion:
recording a gap twice is worse than once, because the second copy hides the first's incompleteness.

**Step 3b recommendation for the 4c plan: keep the attribution; spend the consolidation budget on
deleting the prose that restates it.** Ratio fell 20.5% -> 12.2% across 4b (harness +4%, interpreter
+76%). The guarded duplication never rotted; the unguarded commentary about it did. Falsifiable:
the recommendation changes only if the three files exceed ~2,600 lines or the fraction rises.

**The review-round record, and it is the third consecutive reproduction of the pattern.**

| round | findings | new behavioural defects | new false statements |
|---|---|---|---|
| Task 11 round 1 | 13 | 0 | 5 |
| Task 11 round 2 | 5 | 0 | 0 |
| Task 12 round 1 | 12 | 0 | 4 |
| Task 12 round 2 | 9 | 0 | 0 (verified by me) |

**Two of Task 12 round 1's four were mine.** N1 -- "`condition_traps.rex` is the declared corpus
catcher for **three** of criterion 6's mutations", true value **one** -- came from the original
review's own C1 text, which I relayed verbatim without running it; the implementer then carried it
into two committed files. **A finding whose entire subject was "this falsification clause was
reasoned instead of measured" propagated its own unmeasured claim into the fix for itself.** N3 was
mine too: I wrote "nine of twelve leave `coverage.rs` green" (distributive, true) and the document
made it collective (false -- with all nine removed, three variants go unwitnessed).

**And my own check of N1's correction nearly produced a fifth.** `grep -ci 'call raiser'` returned
**2**, contradicting the re-reviewer's "its single `call raiser`"; the second is inside a comment
quoting the clause as prose. A count that includes a comment, which is the same shape as three
findings this phase. Checked before reporting it.

## PHASE 4B COMPLETE

**Final fix round `a7f1a020`.** Whole-branch review verdict: **ready with fixes, no Critical**;
all nine fixed and verified by me.

**Verified at `a7f1a020`, isolated worktree:** **1020 passed / 0 failed**, fmt 0, clippy 0,
`REXX_CORPUS_GATE=1` 9/9, `REXX_ASSERTIONS_GATE=1` 5/5, `REXX_KEYWORD_GATE=1` 7/7,
`mutate-4b.sh` exit 0 with **12 of 12 as declared**, tree clean.

**The final review's value was entirely in what a per-task review cannot see.** Four of five
Important findings were cross-task drift -- a fact restated in several places where only some were
updated. Scoping it away from a fourteenth pass over the same hunks is what made room for that.

* **I1** `l1-coverage.md` still called the compound-`DO` gap unowned after Task 12 assigned it to
  4c. **Three prose copies; the ruling updated two.** The phase's signature defect, reproduced by
  the fix for it.
* **I2** `support/mod.rs` said "ten prefixes this crate can emit"; measured **thirteen**. Written
  Task 2b, falsified Task 9, in a file Task 9 never touched. **Fixed as a property**, not a
  corrected count -- it now says the qualifying set is the oracle's whole nineteen-marker table and
  that reachability is deliberately not stated because it moves.
* **I3** `coverage.rs` held a claim and its own falsification **167 lines apart**, written by two
  different tasks; four review passes over that file never connected them.
* **I4** `read_subset` is three byte-identical copies, one tested, while Task 2b built
  `tests/support/mod.rs` for exactly that hazard. Deferred to 4c, explicitly.
* **I5 was mine.** I built the deferred-minors roll-up with `grep -n 'minor (deferred)'` and missed
  two `progress.md` blocks using different wording, **both explicitly addressed to the final
  review**. The mechanism I was guarding against failed one step earlier than I expected: the
  roll-up was not the complete set. `deferred-minors.txt` now carries a header naming the exact
  command that produced it and the fact that it was corrected by hand.

**M-a is the sharpest structural fix of the phase.** The Step 3b ratio row was true when written
and false one commit later, twice. The terminating fix is not a better number: **every row is
labelled with a commit, and none says "now", because "now" is the one label that cannot be written
down truthfully.** A commit-labelled row cannot go stale.

**Deferred-minors triage, all 21: 2 fix before merge (done), 12 fix in 4c, 7 drop** -- four of the
seven already closed in the tree, which a current roll-up would have shown.

**The review's own caveat, kept because it is honest:** it found drift by tracing facts *known* to
have changed, so it only finds drift whose changed fact is already known. No cheap exhaustive check
exists. Assume more remains.

**Carried to 4c:** the compound-`DO` fix (owned, in `EXCLUSIONS`); `read_subset`'s three copies;
`corpus/README.md`'s stale inventory; twelve deferred minors with rulings; and Step 3b's
recommendation -- **keep the attribution, spend the consolidation budget on deleting the prose that
restates it**, falsifiable if the three files exceed ~2,600 lines or the fraction rises from 12.5%.
