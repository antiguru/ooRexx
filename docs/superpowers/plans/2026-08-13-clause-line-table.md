# Phase 4g Unit 1: the clause line, from a table rather than a search

**Goal:** stop searching the line index on every executed clause, by keeping the answer beside the one the same upfront pass already keeps.

**Status:** done at `5b3be9536`, one task, accepted. Written 2026-08-13 after the spike below, which is what this plan is built on rather than on the phase document's one-line description of the unit.

**One finding below did not survive the task, and it is left standing here with this pointer rather than rewritten**, because it is what the task was given.
"What survives, and it is the design fact this unit turns on" concludes that the search *depth* is not the cost.
The landed fix measures the opposite: dividing each axis's saving by its own count of removed searches gives 40.0 instructions on a 7-line program rising monotonically to 94.7 on `rexxcps`' 198 lines.
The spike read otherwise from its `rexxcps` arm, which this plan itself disowns as contaminated two paragraphs earlier -- the arm's *table* was disowned and its *design conclusion* was kept, and the conclusion was the part built on it.
The unit's decision is unaffected: a table is preferred to a faster search not because the search is shallow but because a search that is not made costs neither its call nor its depth.
`phase-4f-record.md`'s entry 35 has the numbers.

**Parent:** `docs/superpowers/plans/2026-08-13-phase-4g-structural.md`, Unit 1, ordered first for confidence rather than for size.

## What the spike measured, and the one thing it could not

Run by the controller before this plan existed, `perf stat -e instructions:u`, three interleaved rounds per arm, every arm staged at one fixed binary path, from a fresh empty directory, minimum of each arm quoted.

**`ProgramSource::line_of` is entered once per executed clause on the default engine.**
Instrumented with a call counter and run on `samples/rexxcps.rex` at `count=100`/`averaging=100`, which the program's own `Averaged:` line reports as 100 x 100 iterations of 1000 clauses: **10,000,000 calls against 10,000,000 clauses.**
The compiled engine's promoted clauses do not reach `Interp::enter_clause`, which is what made this worth counting rather than assuming -- but the count says every clause pays anyway.

**Two probe arms, both with deliberately wrong answers, to bound what a fix can buy.**
`flat` returns a constant and removes the function body outright.
`lookup` does one indexed load and no search, which is the shape a precomputed table has.

| axis | base | `lookup` | | `flat` | |
|---|---:|---:|---:|---:|---:|
| `emptyloop` | 27,150,609,654 | 25,625,609,926 | **-5.62%** | 24,525,609,778 | **-9.67%** |
| `rexxcps` | 25,263,153,680 | 24,310,361,501 | -3.77% | 24,731,006,940 | -2.09% |

`emptyloop`'s three rounds span at most 618 instructions on every arm.
`rexxcps` spans 4.03 million on base, 9.83 million on `lookup` and 6.43 million on `flat`.

**The `rexxcps` row is contaminated and this plan does not use it.**
`lookup` removes a strict subset of what `flat` removes, so it cannot save more, and on `rexxcps` it reads 420 million lower.
All three arms self-calibrated to the same 100 x 100 x 1000, checked on each arm's own `Averaged:` line, so the clause count is identical and is not the explanation.
What is left is that both probe arms return **wrong line numbers**, which reach `enter_clause`'s own line comparison and whatever else consults the clause line, and one of the two wrong answers evidently costs less downstream than the other.
**Neither `rexxcps` figure is a bound on anything.** The real number for that axis has to come from the real fix, which returns right answers.

**What survives, and it is the design fact this unit turns on:** the search *depth* is not the cost.
`emptyloop` is 7 lines and `rexxcps` is 198, so `partition_point` runs about three iterations on one and about eight on the other.
They pay the same: `flat` saves 105.0 instructions per pass of `emptyloop`'s loop, which is two clauses a pass, against 52.8 per clause on `rexxcps`.
**So a faster search would be worthless, and the win is in not making the call.**
The honest expectation for this unit is `lookup`'s `emptyloop` figure, about **5.6%** on an axis that does almost nothing else per clause, and less everywhere that does more.

## What is true in the tree

* `Interp::clause_line` (`rust/crates/rexx-exec/src/run.rs:8074`) answers `clause_line_override` when it is set and otherwise calls `source.line_of(instruction.clause_span.start)`.
* `step_in_temps_frame` (`run.rs:4936`) calls it **unconditionally** on every stepped instruction, for the reason its own comment gives: `SIGL` has to stay correct whether or not `TRACE` is on.
* A loop's header and its `END` ask separately (`run.rs:6109`, `run.rs:6139`), which is why a loop pays more than once a pass.
* `ProgramSource::line_of` (`rust/crates/rexx-parse/src/source.rs:266`) is a `partition_point` over `lines`, and is total by construction.
* `Instruction` (`rust/crates/rexx-parse/src/ast.rs:694`) carries `kind` and `clause_span` and no line.

**The precedent is exact, and it is this crate's own.**
`Plan::indents` (`rust/crates/rexx-exec/src/plan.rs:244`) is the same move for the same reason: a constant of the source text that `step_in_temps_frame` asked for on every clause, computed once by `all_indents` in `Plan::build` (`plan.rs:344`), read through `Plan::indent_of` (`plan.rs:292`) which falls back to `static_indent` when the plan has no answer, and pinned by `indent_of_answers_what_static_indent_answers_at_every_index` (`plan.rs:940`).
That doc comment records what it was worth: `indent_in_range` had been the single largest self-time function in the profile at 8.0%.
**This unit is that shape, a fourth time**, after the compound-name family's own applications.

**The obstacle, named so the task does not discover it as a surprise.**
`Plan::build(body, symbols)` (`plan.rs:331`) has no `ProgramSource`.
Its production callers are `Interp::plan_for` (`plan.rs:810`) and `Interp::fragment_plan` (`plan.rs:904`).
A fragment's clauses all read the enclosing `INTERPRET` clause's line through `clause_line_override`, and `Code::plan` is `None` for a fragment, so the fragment path needs no table and must not grow one by accident.

## What the task must establish by running, not by reading

* **Whether the table is ever consulted with the wrong source.** `plan_for` caches by `BodyKey`; a table of line numbers is only valid for the source it was built from. If one `BodyKey` can be reached with a different `ProgramSource`, the cache does not merely miss, it **lies** -- which is the failure this family risks and the reason the accessor carries a tripwire rather than the test suite carrying a hope.
* **Whether the loop header and `END` sites can read the table too**, or whether they hold an instruction index at all at those points.
* **What the fix actually costs on `rexxcps`**, since the spike could not say.

## Steps

- [x] **Step 1: measure the base, and each axis's own spread, before any change.** Every axis, `perf stat -e instructions:u`, arms staged at one fixed binary path, from a fresh empty directory. Record the spread beside the base so a difference under a percent can be read later. Count the `line_of` calls per axis at base, the way the spike did for `rexxcps`, because an axis that never calls it makes its own result a bound rather than a zero.

- [x] **Step 2: add the table, in `indents`' own shape.** A `Box<[usize]>` on `Plan`, one entry per instruction, filled by a walk in `Plan::build`, read through an accessor that falls back to `source.line_of` when there is no plan. Read `plan.rs:244` through `plan.rs:300` for the pattern and follow it rather than inventing a second one. `Plan::build` needs the source; give it the source rather than moving the answer into `rexx-parse`, which would put a field on `Instruction` and touch every construction site for a constant only the executor reads.

- [x] **Step 3: put the tripwire in the accessor, then invert it to prove it fires.** `debug_assert_eq!(cached, source.line_of(span.start))` inside the accessor, whole workspace green, then inverted and re-run so the zero is a live zero. This is the technique that has caught this family twice and it is not optional here: a wrong line is silent in every program that does not raise a condition or trace.

- [x] **Step 4: answer the `BodyKey` question above by running.** Construct the case if you can -- an external routine, a `::ROUTINE`, an `INTERPRET`, the same body reached from two programs -- and say what happened. If you cannot construct it, say that instead of concluding it cannot happen.

- [x] **Step 5: measure, on every axis, and say what the profile looks like afterwards.** If the result is below this phase's resolution floor on every axis that can see it, **say so and do not take the change**: a unit that cannot be measured is not banked, and the phase document's order is about confidence, which a null result also supplies.

- [x] **Step 6: record it as the next entry of `phase-4f-record.md`, appended.** Carry the spike's contaminated-arm finding into the entry rather than only the clean numbers, because the next unit will want to know that a wrong-answer probe arm is not a bound.

## What this task must not do

* **Do not make the search faster.** The spike says depth is not the cost. A better search is a change with a measurement attached to the wrong quantity.
* **Do not put a line on `Instruction`.** That is the parse-side design, it touches every construction site, and the executor is the only reader.
* **Do not change what any program observes.** `SIGL`, the `*-*` trace line and every condition's reported line must be byte-identical, on both engines, across the corpus. This unit spends none of the phase's divergence licence.
* **Do not quote a wall-clock figure.** The phase's instrument is the instruction counter, for the reason its own section gives.
* **Do not rewrite an earlier record entry.** The record is appended to.
