# Task 1 review -- clause line table

Range `9b2d416d3..a8ea4f5be`, code at `5b3be9536`, record at `a8ea4f5be`.
Read-only review; nothing in the repository was modified except this file.

## Verdicts

* **Spec compliance: pass**, with what I could not verify from the diff named under "Could not settle".
* **Task quality: approved**, no critical and no important findings; the minor ones are listed under Q4 and Q5.

## Q1 -- is the tripwire correctly placed and reachable?

Settled from the code: yes, with one bound named at the end.

The tripwire is `rust/crates/rexx-exec/src/plan.rs:358-364`, inside `Plan::line_at`'s `Some(line)` arm.
It compares the cached entry against `source.line_of(instruction.clause_span.start)`, a live recomputation, at the point of use rather than at the build.
The `None` arm falls through to the same recomputation, so a miss cannot answer wrongly and needs no assert.

**Reached by every read of the table.**
`self.lines` is read in production at exactly one place, `plan.rs:356`, inside `line_at`.
The other reads of the field are the new tests reading `plan.lines` directly (`plan.rs:1107`, `plan.rs:1131`, `plan.rs:1226`).
The field is `pub(crate)`, so a crate-internal reader could bypass the accessor; none does today, and `indents` beside it has the same exposure.

`Interp::clause_line_at` (`run.rs:8104`) is the only caller of `line_at`, and its call sites are `run.rs:4937` (`enter_stepped_clause`), `run.rs:6110` (`end_line`) and `run.rs:6140` (`do_line`).
Both engines reach `enter_stepped_clause`: the tree-walker through `in_stepped_clause_with` (`run.rs:4834`) and the compiled engine directly (`ir/drive.rs:462`).
There is no engine-specific read path that skips the accessor.

Paths that bypass the tripwire, all of them correct: `clause_line_at` returns on `source?` and on `clause_line_override` before the table is touched, and takes `source.line_of` directly when `code.plan` is `None` (`run.rs:8125`).
None of the three reads the table.

**The bound.** `debug_assert_eq!` is compiled out in release, so the tripwire does not run in the benchmark or shipped binaries.
That is what the plan asked for and it is this crate's standing technique, not a deviation, but what it buys is "a debug-configuration test run catches it", not "the answer is checked wherever it is used".

The inversion evidence is the implementer's and I did not re-run it: `debug_assert_ne!` reported 403 failures against 0.
Reading the accessor, an inversion firing that widely is what a live assert on a per-clause path looks like, and the figure is consistent with the T1 row's 404.

## Q2 -- can the table answer with a line that is wrong rather than merely absent?

Settled read-only; no probes were needed.

The index means **instruction index into `body.instructions`**, consistently on both sides.

* fill, `plan.rs:430-435`: `body.instructions.iter().map(|i| source.line_of(i.clause_span.start)).collect()`, so entry `n` is instruction `n`;
* read, `plan.rs:356`: `self.lines.get(target)` with `target` the caller's instruction index;
* the callers all held an index already -- `run.rs:4937` passes `enter_stepped_clause`'s own `index`, and `run_repeating` passes `do_index` and `end_index`, which index `code.body.instructions` at `run.rs:6110` and `run.rs:6116`.

Nothing here is a clause index or a byte offset; the byte offset appears only as `line_of`'s argument.

Two independent checks stand between a mispairing and a wrong answer.

* `run.rs:8115-8123` asserts by pointer that `code.body.instructions[index]` is the same object as `instruction`.
  This is load-bearing rather than decorative: `run_repeating` takes `do_index` and `do_instruction` as *separate parameters* (`run.rs:6057-6058`), so their pairing was a caller contract with nothing checking it.
* `plan.rs:359-360` asserts the cached line equals the line of `instruction`'s own span, which additionally catches a plan built from another body or another source.

Fragments get no table: `fragment_plan` calls `Plan::build(..., None)` (`plan.rs:1006`), and `Code::plan` is `None` for a fragment regardless (`lib.rs:1272-1295`).
A body with no plan falls back at `run.rs:8125`; a plan with no entry at `target` falls back at `plan.rs:367`.
Both fallbacks are the pre-change expression verbatim.

**What is left open is the hole the plan named.** A `Plan` cached under one `BodyKey` and reached with a different `ProgramSource` would answer wrongly rather than miss. Nothing in the types prevents it; the tripwire is the whole defence, and only in debug.

## Q3 -- does the INTERPRET override still outrank the table?

Settled from the code: yes, on both engines.

`clause_line_at` (`run.rs:8110-8126`) reads `source?`, then returns `Some(line)` on `clause_line_override` **before** `code.plan` is mentioned.
The table is unreachable while the override is set, so no accessor ordering can bypass it.
That matches `clause_line`'s own order (`run.rs:8081-8083`), the function the new one has to stay identical to.

There is one ordering rather than two: the override is a field of `Interp`, not of either driver, and both engines reach the new accessor only through `enter_stepped_clause`.

**Does the guard test earn its lines?**
`clause_line_at_answers_what_clause_line_answers_and_the_override_still_wins` (`plan.rs:1151`) sets `clause_line_override` on a live `Interp` and asserts the answer is the override at every index.
That is a real assertion against real behaviour, not a comment pretending to be a test: the implementer's T3 mutation fails exactly this test and nothing else.
Its weakness is the reverse of the usual one -- the rule it pins is currently unreachable from any program the implementer could write, so inverting the ordering would change nothing a user sees today.
It guards a future arrangement, which is what the implementer says, and it should stay.
It also does something the mutation table does not credit: its third block pins `source: None` answering `None` even with the override set, the arm where the two accessors could most easily come apart.

## Q4 -- prose that is now false

Four findings, all minor, all documentation.

1. **The plan's "What this task must not do" still asserts the refuted premise, and the pointer does not cover it.**
   `docs/superpowers/plans/2026-08-13-clause-line-table.md`, the first bullet: "Do not make the search faster. The spike says depth is not the cost."
   The new pointer at the top of the file names only the section headed "What survives, and it is the design fact this unit turns on".
   A later reader who lands on the constraint bullet directly gets the refuted claim with nothing beside it.
   The instruction itself is still right; only its stated reason is not.
   Fix: add the pointer to that bullet, or restate the bullet's reason as the one entry 35 gives (a search not made bounds any faster search).

2. **"two paragraphs earlier" is off by one.**
   The plan's new pointer says the contaminated arm is disowned "two paragraphs earlier"; the disowning paragraph is lines 36-40 and the finding begins at line 42, so it is the paragraph immediately above.
   Confined to the plan file: entry 35 and the `plan.rs` field doc both say "disowns as contaminated" with no distance, and are correct.
   Dropping the count fixes it and also satisfies the no-cardinality rule.

3. **`Code::plan`'s doc comment is now an incomplete enumeration.**
   `lib.rs:1272-1274` says the field holds "its clause indents (`Plan::indents`) and how each of its compound names splits (`Plan::compounds`)".
   It now also carries `Plan::lines`, and this change did not update the sentence.
   The same comment's "each reader falls back to computing its own answer -- what `printed_indent` and `Code::compound` both did" now has a third reader, `clause_line_at`.

4. **A set counted in prose.** The `Plan::lines` field doc (`plan.rs:263`) says "about 53 on the three 12-to-14-line ones".
   That quantifies a set of axes rather than naming it; it sits on the measurement boundary, so it is the weakest of the four.

**Checked and correct**, against the code beside them:

* "1-based" in the field doc and in `line_at`'s -- `ProgramSource::line_of`'s own doc (`rexx-parse/src/source.rs:255`) says 1-based and `partition_point(...).max(1)` is that.
* `plan_for`'s new claim that both production callers take body, symbols and source out of one `Rc<Program>` reached through the id the key carries -- `run.rs:4341` takes `routine_program` from `self.programs[installed.program.0]` and keys on `installed.program`; `lib.rs:2215` keys on the id it just pushed.
* `clause_line_at`'s "the shape `printed_indent` already has for `Plan::indents`" -- `run.rs:5599-5602` is that exact `match code.plan` shape.
* `Plan::build`'s "the only such caller is `fragment_plan`" -- the only production `Plan::build(..., None)` is `plan.rs:1006`; the other is the new fallback test.
* The field doc's "`enter_stepped_clause` asks for the answer on every stepped clause" -- correct at `run.rs:4937`, and it quietly corrects the plan's own "What is true in the tree", which attributed the call to `step_in_temps_frame`.
* Entry 35 does not restate entries 26 through 34; it cites entry 27 for the no-wall-clock rule, entry 31 for the control-axis ask and entry 32 for the staging method, and each citation matches that entry's own subject.
* Entry 35's 1487-passed figure is entry 34's 1483 plus the tests this task adds, which is a consistency check the entry does not claim and passes anyway.
* No em-dash anywhere in the diff.

## Q5 -- test value

Four tests are added, all in `plan.rs`.

**Fail if the table is wrong:**

* `line_at_answers_what_line_of_answers_at_every_index` (`plan.rs:1089`) -- asserts `answered == expected` *and* `plan.lines == expected`, so it separates "the accessor recomputed" from "the table is right", which the accessor's own tripwire cannot. Its distinctness assertion defeats the table-answers-its-first-entry-everywhere degenerate case.
* `build_fills_what_line_of_computes_for_every_corpus_program` (`plan.rs:1199`) -- the corpus counterpart, with a `compared > 40 && positions > 500` floor that defeats the empty-sweep vacuity this project has been bitten by.
* `clause_line_at_answers_what_clause_line_answers_and_the_override_still_wins` (`plan.rs:1151`) -- see Q3; sole catcher of the precedence rule.

**Exercises rather than pins, mostly:**

* `line_at_falls_back_when_the_plan_was_built_without_a_source` (`plan.rs:1127`) -- it does assert two real things (a `None` source leaves the table empty, and the fallback arm still answers what `line_of` answers), and it would catch a fallback that returned a constant. But it cannot catch a wrong table, because there is no table, which is why it is absent from the T2 catcher list.

**One weakness worth naming.** Both table tests compute `expected` with the same expression the fill uses, `source.line_of(instruction.clause_span.start)`.
They pin the transcription, not the rule: if `clause_span.start` were the wrong span to report a line from, every one of them would agree and stay green.
The differential gates are what cover the rule, and the corpus test's own doc comment says as much, so this is a stated limit rather than an oversight.

**Minor, DRY.** The pointer assert at `run.rs:8115-8123` is a near-copy of `debug_assert_names_the_clause` (`ir/drive.rs:1728`), which does the same pointer comparison with the same purpose.
Reusing it would need the helper moved out of `ir::drive`, so leaving it is defensible; a comment naming the twin would stop the next reader adding a third.

**Minor, and Concern 5 raises it honestly.** `Interp::clause_line` survives with one caller, `clause_site` (`run.rs:8050`).
Two accessors for one question is a live hazard: a future per-clause caller reaching for the wrong one silently gets the search back.

## Q6 -- measurement claims

No benchmarks re-run; arithmetic and framing only.

**Every difference is larger than both its arms' spans**, and by a wide margin.
The tightest is `rexxcps` at 962,606,940 against a base span of 11,497,720, which is 83.7 -- exactly the ratio the report quotes as its tightest -- and the entry's "at least a factor of eighty" is therefore the true minimum rather than a round number chosen for comfort.
The other six are between roughly 280 and 10^5.
Each percentage recomputes from its own row.

**One observation on the spans themselves.** Step 1's same-binary spans and Step 5's base spans disagree by up to four orders of magnitude on the same axis (`strings` 36,001,118 against 1,317; `rexxcps` 19,581 against 11,497,720).
That is not an error -- they are different sittings and the interleaved one is the one that counts -- but it means "83.7 times its own wider span" is a statement about that sitting, not a stable bound.
It does not put any axis at risk: the smallest absolute difference, `arith`'s 209 million, is still five times the worst span observed anywhere in either sitting.

**"No figure here is a bound" is earned.**
The claim needs every axis to execute the changed code, and that was established by counting rather than by reading -- a counter on `Interp::clause_line`'s call into `line_of`, at base, on both engines.
One caveat the report does not state: that counter also counts `clause_site`'s calls (`run.rs:8050`), which this change does *not* remove, so the divisor is "searches at base" and not strictly "searches removed".
On these axes the two coincide -- `clause_site` runs only when an echo is gated on or a failure is recorded, and `emptyloop`'s 50,000,005 decomposes exactly as two per pass of 25,000,000 with five left over, leaving no room for anything else.

**The depth finding is better supported than the report argues.**
The report shows the per-search saving is monotone in the body's *line count*, which invites the objection that a longer program differs in more than search depth.
The figures are in fact linear in `log2(lines)`, which is search depth and nothing else: 3 halving steps for 7 and 8 lines, 4 for 12 and 14, 6 for 51, 8 for 198, against savings of 40.0/42.0, 53.5/52.9/52.4, 75.3 and 94.7 -- about 11 instructions per halving step at every point, fitting all seven axes.
A cache-footprint story would not produce that.
Stating it in the log2 form would make the refutation harder to argue with, and would answer the confound the report leaves open.

**Concern 1 is stated as the weakness it is, not argued away.**
The report says "It is weaker than a control and I am not calling it one", and entry 35 repeats it under "What this entry does not claim".
One thing neither says: the stand-in offered for the missing control -- monotonicity in the depth removed -- is the same data the new design finding rests on, so the two lean on each other rather than checking each other.
The log2 fit above is what would make that stand-in independent enough to be worth the weight put on it.

## Constraint checks

* No em-dash in the diff.
* `phase-4f-record.md` appended only; entry 35 neither restates nor contradicts entries 26 through 34.
* No wall-clock figure in entry 35; the one time-shaped number is `rexxcps`' own printed clauses-per-second line, reported as an output difference and labelled as one.
* No line added to `Instruction`; the source is threaded into `Plan::build` as the plan directed.
* One prose set-cardinality slip, Q4 finding 4.

## Could not settle

Nothing was left open for want of effort; these are things the diff cannot answer.

* **Every measured figure** -- the seven-axis table, the profile shares, the mutation counts, the byte-for-byte corpus comparison and the `BodyKey` probe sweep. They are the implementer's, they are internally consistent where they overlap (1483 + the new tests = 1487; the tightest span ratio equals the quoted 83.7; `emptyloop`'s count decomposes exactly), and I did not re-run any of them.
* **Whether a `BodyKey` can be reached with a different `ProgramSource`.** Not attempted, per the brief. The standing answer is the tripwire, and the tripwire is correctly placed.
* **Whether the T3 state is reachable at all.** The implementer instrumented for it and found nothing; that is a negative from a sweep, not a proof, and both the report and the entry say so in those terms.
