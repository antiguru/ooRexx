STATUS: DONE

# Coherence audit: coordinator edits, rulings, and source comments

Scope: (1) section-scoped contradictions at the four named seams; (2) whether Task 11
is safe to implement from its own section; (3) four rulings, attacked; (4) facts
duplicated across plan / spec / exclusions / source comments / progress.md.

Read-only. Numbers are `audit-claims`'s job; this is about coherence.

Headline: **Task 11 is not safe to build from its own section.** Three of the four
things the coordinator believes Task 11 owns or inherits are absent from Task 11's
body, and Step 4 demands a unit test that cannot pass as specified. The D17
process-failure verdict does not survive contact with D17's own text or with the
shipped `eval`/`eval_node` split.

---

## Critical 1: the clause-echo indentation has four recorded owners and zero real ones

Four documents, four different owners, no two agreeing:

* `docs/superpowers/plans/2026-07-30-phase-4a-executor.md:745` — inside **Task 10's**
  section, and it says "**this task is what makes it reachable**". Task 10 is closed.
* `.superpowers/.../task-10-report.md:593` — heading "Indentation: characterised, not
  implemented (`error.rs` is **Task 12's**)", and the section closes "implementing that
  is **Task 12's** call, not mine." Task 12 is closed.
* `progress.md` — "Every residual difference is now purely the indentation, which is
  **Task 11's**", repeated three times, including in the list of three things Task 11
  inherits.
* **Task 11's own section: no mention of indentation at all.** Not in Files, not in a
  step, not in the prose. `grep -i indent` over the plan returns exactly two hits: line
  745 (Task 10) and line 906 (Task 13's byte-capture step).

Because briefs are the task section verbatim (verified: `task-9-brief.md` and
`task-11-brief.md` are byte-copies of their plan sections plus a trailing `---`), the
implementer of Task 11 will never see this requirement. It is currently owned by two
closed tasks and mentioned in a progress log that is not extracted.

This is the fifth instance of the extraction-seam mistake, not the fourth, and it is
the most expensive one so far, because the requirement is *reachable as of Task 10* —
line 745 says so itself — and the corpus is already running: the first corpus program
that fails inside an `IF` or `SELECT` diverges on stderr now, not later.

**Fix:** the requirement has to be written into whichever task actually gets it, in that
task's own body, with a pointer to `task-10-report.md`'s measured table (which is the
only place the verified rule exists). Leaving line 745 where it is also leaves a false
statement in the plan about a closed task.

## Critical 2: Task 11 Step 4's demanded unit test cannot pass as specified

Step 4: "a **unit test that reaches the 11.1 raise**, since no differential program can
cross our limit without also crossing the oracle's cliff, and without that test the depth
path is untested by construction."

Step 3 sets the limit at 100,000 and the Files section puts the test in
`#[cfg(test)] mod tests` inside `eval.rs`.

The sized stack exists in exactly one place in the workspace:

    rust/crates/rexx-exec/src/lib.rs:800   std::thread::Builder::new().stack_size(INTERPRETER_STACK_BYTES)

That is the public entry point. `grep -rn 'stack_size\|Builder::new'` over
`rexx-exec/src`, `rexx-exec/tests` and `rexx-parse/src` returns that one hit. A libtest
thread gets 2 MiB by default. At the ~1600 bytes/level that Task 11's own text quotes,
`eval` dies natively at roughly **1,300** levels — two orders of magnitude short of the
limit the test is supposed to trip. The test as written aborts with SIGABRT and no
message, which is the precise outcome D19 exists to exclude, and the cheapest-looking
"fix" available to an implementer under pressure is to lower the limit, which is the
divergence D19 forbids.

The warning exists — D19: "A `cargo test` thread has a default stack far smaller than
the one a depth limit is calibrated against, and the failure mode there is exactly the
silent native overflow D19 exists to exclude" — **in the spec, in a bullet about where
the sized thread lives.** Task 11's body carries neither that sentence nor any
instruction to spawn a sized thread in the test. Task 11's Files section does not name
`lib.rs`, so an implementer who works out that the test needs the entry point also finds
the file it lives in outside their permitted set.

**Fix:** state in Task 11's body that the depth test must run on a thread it sizes
itself (or through the sized entry point), and either add `lib.rs` to the Files set or
name the helper it may use.

## Critical 3: `run_bounded` absorbs any in-range `Goto`, and nothing tells Task 11

`run.rs:826` — `Flow::Goto(target) if target >= start && target <= end => pc = target`.

An enclosing `run_bounded` claims **any** jump landing inside its own range, forwards
and backwards, without notifying the arm that produced it. Task 11's `LEAVE`/`ITERATE`
are jumps. Concretely: a `DO` loop inside an `IF`'s `THEN` branch, with `ITERATE` in the
loop body. The loop top is inside the `IF`'s range, so if Task 11 implements loops as
`Flow::Goto` against a per-activation block stack (which `activation.rs:15-20` says is
Task 11's design), the `IF`'s `run_bounded` sets `pc` to the loop top directly and the
`Do` arm is re-entered as a *first* entry — re-initialising the control variable, or
leaving a stale `Block` on the stack. That is exactly criterion 6's "`LEAVE` unwinding
one block too few" mutation, arriving by a route the mutation list does not describe.

The invariant Task 11 needs is: **a `Flow::Goto` may be absorbed by any enclosing
`run_bounded` frame without any intermediate arm running, so all block-stack unwinding
must be complete before the `Goto` is returned.** `run_bounded`'s doc comment states the
ownership rule correctly and even anticipates `LEAVE`, but it states it as a *propagation*
guarantee ("a `Flow` variant this function does not recognise ... propagates outward"),
which is the case that does *not* bite. The case that bites is an in-range `Goto`, which
does not propagate. Task 11's body says only "Read Task 10's committed code before
designing the block stack", which is right but not sufficient: this is one conditional
buried in a 45-line doc comment.

Note also that `Activation`'s planned `blocks: Vec<Block>` and Task 10's Rust-recursion
ranges are two control mechanisms that must agree, and `LEAVE`/`ITERATE` is the only
thing that crosses between them. Task 11's body does not say which mechanism owns loops.

## Critical 4: Task 11's three inheritances from Task 10 are absent from Task 11

`progress.md` records that Task 11 "carries three things this task produced": the
oracle-verified indentation rule in implementable form; the guarantee that `run_bounded`
forwards an unowned `Flow` so `LEAVE` can unwind out of a nested Rust call; and `error.rs`
in its permitted set for both the 11.1 catalogue test and the indentation work.

In Task 11's section: (1) absent entirely (Critical 1); (2) absent entirely — the section
instead still poses it as an open contingency, "**if Task 10 ships jump targets alone**,
this task retrofits block bookkeeping onto Task 10's arms in the same file immediately
afterwards", a conditional whose answer has been known and verified since commit
`a9997e55`; (3) present in Files but justified only by "the 11.1 catalogue-family test",
so an implementer has no reason to touch `Raised::report`.

Same section also contains a statement that its own edit falsified: "**That last one is a
coordination point with Task 10 and neither task's text mentions the other's stake.**"
Task 11's text now does mention it, in that very sentence. Task 10's still does not.

---

## Important 1: Task 16's Files section still orders the file Step 4 forbids

Task 16, lines 986-988:

    **Files:**
    - Create: `rust/crates/rexx-exec/tests/coverage.rs`, `tests/loud.rs`, `rust/scripts/mutate-4a.sh`
    - Create: `docs/superpowers/plans/phase-4-exclusions.txt`

Step 4, line 1004: "**`phase-4-exclusions.txt` ALREADY EXISTS and is ahead of this step.
Do not write it, and do not follow this step literally.** ... Writing it from this step
would **regress** it."

The Step 4 correction landed; the Files line it contradicts survived. This is the third
occurrence of the pattern the coordinator named, in the task most likely to be executed
by scaffolding read off the Files list. The plan's own "File structure" block (line 53)
lists the same path under files to create, which compounds it — though that one is
outside every task section and therefore invisible anyway.

Task 16's Files section also omits `docs/superpowers/plans/phase-4a-gate.md`, which
Step 5 creates, and `rust/corpus/phase-4a.txt`, which Step 1's set assertion reads.

Step 4's markdown is also malformed: a bare `>` on line 1005 opens a blockquote that
continues into line 1006, inside a checkbox list. It will render as an orphaned quote.

## Important 2: Task 13's Files section and its Step 5 disagree about where the tests live

Files: "Test: a `#[cfg(test)] mod tests` inside `rust/crates/rexx-exec/src/trace.rs`,
with the committed expectations under `rust/crates/rexx-exec/tests/trace_oracle/`".

Step 5: `git add rust/crates/rexx-exec/src/trace.rs rust/crates/rexx-exec/tests/trace.rs
rust/crates/rexx-exec/tests/trace_oracle`.

`tests/trace.rs` is an integration test the Files section does not name and the Global
Constraint at line 15 argues against for a private subject. The same defect is still
live in Task 9's Step 5 (`tests/run_basic.rs`) and was in Task 11's brief as generated
(`tests/run_loops.rs`); Task 11's plan section has since been corrected to stage
`run.rs eval.rs error.rs` and is now clean. So the coordinator fixed this in one place
of three. `tests/run_basic.rs` does not exist, which means Task 9 shipped by deviating
from its own commit block.

## Important 3: the phase now has two harnesses with opposite oracle strategies, and neither task knows

Task 14 Step 3 (plan): "Build the harness as a `cargo test` **with the oracle's expected
output committed**, so `cargo test` alone is the gate and a script regenerates", and the
Files section creates `tests/corpus_oracle/`.

As shipped (`tests/corpus.rs`, commits `19e9e286`/`3363b278`): the harness **runs the
oracle live as a subprocess** and fails when it is missing. `tests/corpus_oracle/` does
not exist. The module doc argues the choice explicitly — a committed expectation "would
let a stale binary answer for the current oracle with nothing to [catch it]".

That is very likely the better design, but the plan still instructs the other one, and
Task 13 Step 2 instructs the *committed-expectation* pattern ("the way
`rexx-parse/tests/sourceline_oracle/` does") for trace, in the same phase, for the same
oracle. Task 13's implementer should be told that Task 14's runner rejected committed
expectations on stated grounds, and asked to decide rather than inherit. Right now
neither section mentions the other.

Task 14's Files section also still says "Create" for `rust/corpus/phase-4a.txt` and
`tests/corpus.rs`, both of which exist. The coordinator fixed the file-existence problem
for Tasks 13 and 16 and left it in Task 14.

## Important 4: two counters raise 11.1 at two limits, and the plan never relates them

`rexx-parse::expr::MAX_EXPR_DEPTH = 50_000` raises 11.1 for the parenthesis and
nested-call axes. Task 11 is to raise 11.1 from `eval` above 100,000. Both use the same
error number, and the exclusions file DEVIATION 2 is the only document that explains
that one is parity and the other a deviation.

Task 11's body never mentions `MAX_EXPR_DEPTH` or the 50,000 figure. The trap for
Step 4's "unit test that reaches the 11.1 raise" is direct: a test written with nested
parentheses, or with nested calls, reaches 11.1 at 50,000 from `rexx-parse` without
`eval`'s counter ever incrementing, and looks exactly like a pass. Given Critical 2
(a flat 100,001-term chain will overflow a libtest thread first), a parenthesis-based
test is the *likely* path an implementer takes to make the test go green.

**Fix:** Task 11 must say the test has to assert `eval`'s counter fired, not merely that
11.1 was raised, and must name `MAX_EXPR_DEPTH` as the decoy.

## Important 5: a KNOWN GAPS reason in `phase-4-exclusions.txt` was falsified by Task 10

The one-clause-echo gap row closes: "The gap is unreachable from any corpus program
today, because **the only construct that nests an instruction loop inside another is the
INTERPRET fragment spike**, which 4b deletes and which `run_program` does not expose."

Since `addf89b1`, `run_bounded` nests an instruction loop inside `run_activation` for
every `IF` and `SELECT`, and Task 11 will nest more. The conclusion may well survive —
the oracle's multi-echo is per activation/interpret nesting, not per block — but the
stated reason is now false, and it is the sentence a reader would rely on to decide the
row is still dormant. Correcting the reason is a one-line edit; leaving it is how the row
stops being reviewed.

---

## Ruling 1 (deferring the `EXIT` under-rooting window): **defensible, and criterion 4 could not have caught it either way**

The deferral is sound, for a reason stated in exactly one place and absent from the
ruling itself.

The window runs from `step_in_temps_frame`'s unconditional `pop_frame` to
`Interp::exit_code_for` (`lib.rs:695`). What runs in it: `run_activation`'s teardown,
`execute`, then `exit_code_for` → `to_number`. I checked `to_number` (`value.rs:199`):
`SmallInt` goes through `Number::parse(&n.to_string())`, a Rust allocation; `Body::Text`
fills `num.get_or_insert_with(...)`, a `Box<Number>` on the Rust heap, in place. Neither
calls `Heap::alloc_with`. **There is no Rexx-heap allocation in the window.** A faithful
collect-on-every-allocation collects at allocation sites, so it never fires inside the
window, and criterion 4 cannot expose the bug. So: you have not deferred the thing most
likely to fail your own gate.

Three things are wrong around the ruling, though, and one of them matters more than the
ruling:

* **Criterion 4 is the one criterion of the seven with no way to fail.** Its whole text
  is "**The named L0 subset passes again under collect-on-every-allocation.**" The gate
  section's own opening sentence promises "each criterion names the set it quantifies
  over, **each can fail**, and no criterion's anti-vacuity requirement lives in prose
  beside it." Criteria 1, 2, 3, 5 and 6 each carry an explicit anti-vacuity device
  (macro-generated no-wildcard match, falsification perturbation, prefix table taken
  from the oracle's side, the unapplied-pattern guard). Criterion 4 carries none, and
  `Heap::alloc_with` never collects, so "under collect-on-every-allocation" is
  satisfiable by a mode that is a no-op. Task 14 Step 4 — "Run it, and separately under
  collect-on-every-allocation" — adds nothing. This is the same shape as the `/bin/true`
  finding criterion 6 was rewritten to close. It needs a negative control: a value
  deliberately left unrooted must be collected under the mode, or the mode must report a
  nonzero sweep count from inside the corpus run.
* **The load-bearing argument lives only at the leak site.** `run.rs:443-459` carries it:
  "Benign only because nothing on that path allocates or collects". `Heap::collect`'s new
  comment does *not*: it says "one known under-rooted window is **waiting for the day
  something does** [call this]", which reads as "this will break when you wire a
  collector". Whoever implements criterion 4 will open `Heap::collect`, conclude
  criterion 4 is blocked on fixing `EXIT`, and either do unnecessary work or defer the
  criterion. The precise claim needs to be at `Heap::collect` too, and criterion 4 should
  name it.
* The commit message for `c67dd343` reproduces the weaker framing, so the strongest
  argument you have for this ruling is not in the ruling, not in the commit, not in the
  spec, and not in the gate criterion.

## Ruling 2 (`If`/`Select` resolving inside their own `step` arm): **right outcome, and the stated reasoning does not dispose of the alternative that actually competed**

The design as built is sound and well documented, and I could not construct a case where
`run_bounded`'s range confinement gets a jump wrong on its own terms. `block.rs`
assembling inner targets to close before outer ones is the right invariant to lean on.

But the ruling was framed as nested-bounded-loop *versus repairing Phase 3's elided
end-of-branch markers*, and rejected the repair as "a special case with no analogue for
`WHEN`". `run_bounded`'s own doc names a third option and dismisses it on availability,
not merit: "**a per-activation block stack would resolve it trivially but does not exist
yet** (`Activation`'s own doc comment: Task 11's)". That is the option that actually
competed, and Task 11 builds it one task later regardless. So the recorded justification
argues against the weakest alternative and is silent on the strongest.

Cost to Task 11, concretely:

1. Critical 3 above. Two control mechanisms in one file, and `LEAVE`/`ITERATE` is the
   only thing that crosses between them.
2. The indentation quantity is now split across both mechanisms. Task 10's measured rule
   is "two spaces per currently-open block-stack frame", where `IfThen`, `Else`,
   `WhenThen` and `Otherwise` are each their own frame — and under Task 10's design none
   of those has a runtime frame, because `If`/`Select` push nothing. So whoever
   implements the indentation must derive a depth that Task 10's design deliberately
   does not maintain. Task 10's report saw this and concluded the quantity is *static*
   and computable from the AST alone; that is probably right, and it means the
   indentation does not want the block stack at all (see Ruling 3).
3. `End`. Task 10 gave `End` an arm for `EndStyle::Select` only; Task 11 must extend it
   for `Do`/`Loop`. Task 11's body never mentions `End` or `EndStyle`, and criterion 1
   notes "`EndStyle` has never been gated by any phase".

Nothing here argues for reopening the decision — Task 10 has shipped, reviewed, with the
`Flow`-forwarding hinge verified by mutation. The correction needed is to Task 11's body,
not to Task 10's code.

## Ruling 3 (indentation to Task 11, one mechanism shared with Task 13): **the sharing is right, the assignment is wrong, and as written it is unassigned**

*Sharing*: correct. Both consumers need one quantity — nesting depth at a clause — and
Task 10 verified the same rule against the oracle for both the `*-*` clause echo on the
error path and (by construction) the trace prefix. Two formatters, one quantity.

*Assignment*: wrong, on three counts.

1. It is not in Task 11's body, so it is assigned to nobody (Critical 1).
2. The reason to give it to Task 11 — that it needs the block stack — is contradicted by
   the only measurement anyone has taken of it. `task-10-report.md`: "this is exactly the
   static nesting depth at the point a clause was parsed, it looks computable from the
   AST alone ... with no runtime block-stack needed". If that holds, Task 11 owns it for
   no reason beyond file contention on `run.rs`, and it does not even need `run.rs`.
3. Task 11 is already the largest task in the phase: six `LoopKind` variants, `COUNTER`,
   `OVER ... FOR`, `WHILE`/`UNTIL` with 34.3/34.4, `LEAVE`/`ITERATE` with the 28.1-28.4
   family, the 26.2/26.3/41.1 measurement step, the D19 depth counter, three files. It
   is also the task blocking everything. Adding an oracle-characterisation deliverable
   against `Raised::report`'s formatting to it is the wrong place to put schedule risk.

Better shape: its own small task, sequenced anywhere before Task 13, with Files =
`error.rs` plus whichever of `plan.rs`/`rexx-parse` carries the per-instruction depth, and
a single deliverable — the depth quantity plus the `Raised::report` consumer — leaving
Task 13 to add the second consumer. If it stays in Task 11, it needs a paragraph in Task
11's body, a pointer to `task-10-report.md:593`'s table, and `Raised::report` named in the
`error.rs` Files justification.

One further coherence problem in the plan's version of the rule (line 745): "This is the
same indentation `TRACE` applies, so Task 13 shares it." Nothing has verified the *trace*
half. Task 10 measured the error-report echo only; `task-10-review.md` confirms six probes,
all of them "a raising `say 2 & 1` inside the construct". Whether `trace r`'s `*-*` lines
carry the same count is asserted, not measured, and it is asserted in a closed task's
section that Task 13's brief will not contain.

## Ruling 4 (D17 was not implemented; Task 13 is the retrofit D17 forbade): **overturned in substance**

D17's full decided sentence is "the dispatch loop emits a trace event per evaluation step
from the start, and 4a formats the value lines", so "from the start" is really there and
you did not invent it. But three things follow from reading the rest of the block and the
shipped code.

**1. The harm D17 names has not occurred.** D17's entire stated rationale is: "Emitting an
event per evaluation step forbids constant folding and expression fusion, which is
accepted ... and the alternative is designing the dispatch loop twice." The thing D17
protects against is a dispatch loop built with folding or fusion that then has to be
redesigned. The shipped `eval_node` (`eval.rs:100-222`) does no folding and no fusion:
every node dispatches, every operand goes through `eval`. So the decision's substance is
intact and the retrofit is not the one D17 was written to avoid.

**2. The plan itself scheduled trace at Task 13, from revision 1.** Tasks 7, 8 and 9
complied with the governing document exactly. Their Files sections do not include
`trace.rs`; their steps do not ask for an event. Even a brief containing all of D17
verbatim would not have changed what they built, because they were told what to build and
it did not include trace. So "Tasks 7 to 9 were in breach" is not supportable — if anything
was in breach it was the plan's decomposition, and it was visible at authoring time rather
than discovered by Task 13. The lesson you drew ("a decision recorded only in the spec
reaches nobody") is a real and separate lesson, and it is well evidenced elsewhere in this
audit; it just is not what happened here.

**3. The scope claim is overstated, and checkably so.** Task 13's Files entry says the
value lines mean "threading an event through roughly **eighteen** `eval_node` arms".
`eval_node`'s match has **ten** top-level `ExprKind` arms (`Binary` split across four),
and — decisively — `eval` is a thin wrapper that already exists *for exactly this
property*: it was split out "so that the depth is decremented on every exit path including
the `?` ones" (`eval.rs:68`). Every node's value returns through
`let value = self.eval_node(code, expr);` at `eval.rs:95`. A post-order trace event with
the value in hand is **one** insertion point there, matching on `expr.kind` to pick the
prefix. So the "most contended file in the phase" alarm applies to the `*-*` clause event
in `run.rs` (real, and small), not to the value lines.

What I would keep from the ruling: sequencing Task 13 after Task 11 (right, for
`run.rs` contention), and reading committed `run.rs` rather than the plan (right). What I
would drop: the process-failure verdict, and the eighteen-arms scope estimate. Recording
"D17 was not implemented" as a breach also creates a hazard of its own — it invites a
future reader to conclude the *decision* was unsound and reopen it.

---

## Part 4: facts recorded twice

Handled well, and worth saying so:

* `tests/corpus.rs:451`'s "9 of 26 matching" against `progress.md`'s "12 of 26" is not
  drift. The comment is anchored ("at commit `e0e57825`") and explicitly instructs
  "the fix is to re-run and record which commit was measured, **not** to adjust this
  comment to match a stale number". That is the right pattern and the only place in this
  phase where a duplicated figure defends itself.
* D19's per-level byte figure is duplicated between spec and plan and both carry the
  "re-measure, do not quote a predecessor" instruction plus the full history of the four
  values. Perishable data marked perishable.

Drifting or one-sided:

* **The `EXIT` window's safety argument** exists only in `run.rs`'s `Exit` arm.
  `Heap::collect`'s comment, `c67dd343`'s message, criterion 4 and Task 14 Step 4 all
  describe the same window and none carries the "nothing on that path allocates" clause
  that makes the deferral safe. Highest-consequence duplication problem I found, because
  the four weaker copies all imply criterion 4 is blocked.
* **The exclusions file's dormancy reason** for the one-clause-echo gap (Important 5) is
  falsified by shipped code.
* **Trace prefixes**: spec criterion 3 lists nine reachable prefixes ending in `>P>`;
  Task 13 Step 1 lists the same eight strings plus "**a prefix-operator line**". The
  paraphrase is correct (`TRACE_PREFIX_PREFIX` is index 8 in the enum at
  `RexxActivation.hpp:92`-`110`) but it means Task 13's brief is the only copy without
  the literal byte string, in the one task whose deliverable is literal byte strings.
* **Indentation ownership**: four documents, four owners (Critical 1).
* **`corpus_oracle`/committed-expectation strategy**: plan says one thing, shipped code
  does the other, Task 13 is about to be told the plan's version (Important 3).

## Non-findings, checked

* `eval.rs:70-71` really does say "Task 11 adds the limit check to this function", so
  Task 11's Files justification for `eval.rs` is accurate.
* Every `LoopKind` variant Task 11 names is real (`ast.rs:986`: `Simple`, `Forever`,
  `Count`, `Controlled`, `Over`, `With`), and `counter`/`for_count` exist
  (`ast.rs:971`, `:999`), so "`COUNTER` and `OVER ... FOR` are in the AST" holds.
* `run_bounded`'s catch-all really is the forwarding kind (`other => return Ok(other)`),
  so the `LEAVE`-unwinds-out-of-a-nested-call guarantee is true as stated. It is the
  in-range case, not this one, that is undocumented for Task 11.
* Task 11's Files entries all name existing files; the two added entries (`eval.rs`,
  `error.rs`) are both justified and both in Step 5's `git add`.
* The 19-prefix count and the `:92`-`:110` span in the spec are exact
  (`TRACE_PREFIX_CLAUSE` at 92, `TRACE_PREFIX_INVOCATION_EXIT` at 110, 19 entries).
  The plan's `:90`-`:110` includes `typedef enum`; harmless.
* The exclusions file's DEVIATION 2 and the plan's Task 11 agree on the two-sided bound,
  the fire-above-100,000 off-by-one, and the parity-vs-deviation split.

## Not reached

* Tasks 1-8 and 12's bodies, except where a later task cites them.
* Task 15's section, beyond noting it has two steps numbered 5.
* The `plan.rs` and `stem.rs` comment surfaces.
* Whether the indentation rule's *trace* half matches the oracle (asserted in the plan,
  measured by nobody).
* Any re-measurement. All oracle numbers here are taken as given; I ran no probes.

STATUS: DONE
