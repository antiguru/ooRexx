# Phase 4e handoff -- three items chased before 4f, time-boxed

**Decided 2026-08-11 by Moritz**, after Phase 4e's final review returned "fit to close" with five unsettled items.
Three of them are chased here rather than inside the optimisation loop; the other two are correctly 4f's.

**Why before 4f rather than in it.**
Phase 4f is a search whose accept rule is a paired comparison against the immediately preceding binary.
Two of the three items below are *unexplained instruction-count moves*, and an unexplained move in the baseline is a corrupted denominator -- which is the class that has produced a wrong ratio six times in this phase.
The third is the missing net that would catch a promotion silently ceasing to fire, and it is worth having **before** a loop starts changing the code that promotes.

**Time-boxed, and the box is the point.**
Each item below names what ends it.
**A named mechanism is the success; a recorded set of ruled-out hypotheses is also a success.** What is not acceptable is a mechanism asserted without a build behind it -- this phase's worst failure was a task stating a verdict its measurements did not reach, and it cost three tasks to unwind.

## The instrument

`rexx-arms` for any ratio or per-pass figure, under the rules the phase established: both instruments, both arms of one build, two problem sizes, one sitting, medians.

**`samply` and `pollard` are the tools for the two attribution items**, and they answer a question `rexx-arms` cannot: `rexx-arms` says *how much* moved, and a profile says *where*.
`samply record` on the axis binary, then `pollard` to load the profile and compare functions between the two builds.
`mcp__pollard__compare_profiles` and `mcp__pollard__compare_functions` are the two that matter; `top_functions` and `stacks_containing` locate before comparing.

**A profile is not a measurement.**
Sampling attributes time, and both items here are *instruction-count* deltas measured exactly. So a profile proposes the candidate and `rexx-arms` or `callgrind` confirms it. A profile alone never closes an item.

---

## Item 1: `arith`'s tree-walker arm moved at the Task 9 boundary

**The fact.** `arith`'s **tree-walker** arm goes 61844.27 instructions per pass at `e63e8a00` to 62030.85 at `16077ea1` -- **+186.6 per pass on an arm Task 9 does not touch.**
Recorded in Task 9's report, absent from the gate document, unexplained.

**Why it matters more than its size.** It is a denominator move. Every criterion-4 ratio on that axis divides by it, so a tree-walker arm that drifts under an optimisation loop makes the loop's own accept rule read the wrong sign. The phase measured the tree-walker arm's cycles-per-instruction moving 3.8% to 6.5% between builds with the work held constant, and that was *cycles*, where code placement explains it. **This is instructions, where placement does not.**

**The obvious candidate, which must be checked rather than assumed.** Task 9 split `eval_arithmetic` into `arith_small_int` and `arith_general` so both engines could enter them. The tree-walker now calls two functions where it had two inline blocks. Task 7 measured exactly this shape before -- extraction for sharing cost the tree-walker arm 0.2% to 1.3% -- and it was accepted deliberately. **If that is the whole of it, this item closes as "already known, now quantified on a third axis", and the sharing rule's price gets one more figure recorded against it.**

* [ ] Build `e63e8a00` and `16077ea1`, profile the **tree-walker arm** of `arith` on both, compare functions.
* [ ] Confirm or refute the extraction hypothesis with an exact instrument, not the profile.
* [ ] If it is the extraction: record the price beside Task 7's, and this item is done.
* [ ] If it is not: name what else moved, or record what was ruled out.

**Ends when** the mechanism is named with a build behind it, or after **three** candidate hypotheses have been tested and refuted. Record either outcome in this file.

### Result: the extraction, confirmed by a build -- but not by the mechanism the item named

Working notes in `.superpowers/sdd/2026-08-09-phase-4e-ir/handoff-perf-report.md`.
Every figure below is a row of `rust/bench-baselines/phase-4e-arms.tsv` under task `item1`, or a `callgrind` reading named as such.

**The fact reproduced, on the phase's own instrument.**
One sitting, both arms, both sizes, five rounds, medians, `instructions:u`: `e63e8a00`'s tree-walker arm is 61843.919 per pass and `16077ea1`'s is 62030.499, so the move is **+186.580** against the recorded +186.6.

**The bisect.**
Three commits in that range carry Rust source the tree-walker reaches, and `16077ea1` itself changes only `tests/ir_dual_cases/arithmetic`.
`b3345d91` -- the split -- is 62014.251, so it carries **+170.332 of the +186.580, or 91%**.
A `callgrind` bisect over all five builds, exact and deterministic, puts the whole of it there: `e63e8a00` 57723.481, `b3345d91` 57884.108, and the three commits after it together -0.746.
The two instruments disagree about the tail, 16.2 against -0.7, and that is recorded as measured rather than explained; they agree on which commit owns the move.

**The confirming build.**
`e63e8a00`'s pre-split body restored inline in `eval_arithmetic` at `16077ea1`, with `arith_small_int` and `arith_general` left in place for `Op::Arith`, reads **61868.915** -- recovering **161.584 of the 186.580, or 87%**, and landing 24.996 above `e63e8a00`.
The same restore one commit earlier lands 23.045 above it on `callgrind`, so the residual belongs to `b3345d91` too and survives giving the tree-walker its body back.

**The item's own account of the mechanism is wrong, and the correction matters for the sharing rule's price.**
Neither half is a call: `nm -C` finds no `arith_small_int` and no `arith_general` symbol in any binary of the range, and `callgrind` records no call edge to either.
Both are inlined into `eval_arithmetic` outright, and `eval_arithmetic` carries +137.67 of the +160.63 `callgrind` reads.
So what the tree-walker pays is **the code LLVM emits for the shared shape, not two calls** -- which is why the price cannot be predicted from the call count, and why Task 7's measured band is the only thing that bounds it.

**A refuted candidate, recorded so nobody retries it.**
`arith_small_int` reads `activation().settings.digits()` before deciding, so the general path reads it twice where the pre-split body read it once.
Moving that read inside the `SmallInt` arm removes exactly the second one and is semantically identical.
It costs **78.05 more instructions per pass**, not fewer.

**Closed as the item predicted: already known, now quantified on a third axis.**
186.580 on 61843.919 is **0.302%**, inside the 0.2% to 1.3% band Task 7 measured for this shape.

**What a fix would cost and buy, as 4f's input.**
The only fix the measurements support is undoing the sharing for the tree-walker, which is exactly the build above: it buys back 161.584 per pass, 0.26% of that arm, and costs two copies of the arithmetic that have to be kept in step -- the defect the sharing rule exists to prevent.
Not recommended here; recorded as the trade.

---

## Item 2: no corpus-wide assertion over the compiled op stream

**The hole, in the gate's own words.** There is no assertion over the compiled op stream of *every corpus program*.
What exists is a per-construct golden set with a negative control, plus a corpus-wide assertion that `chunks_refused` is zero on both arms -- **which says every body compiled, not what it compiled to.**
So a promotion that silently stopped firing for a construct the golden set does not cover would leave every gate green.

**This is the only item here that builds something rather than explaining something**, and the gate calls it cheap because both halves already exist: the serialiser is `ir/golden.rs` and the population is `corpus_cases()`.

**The design question this must answer, and it is not obvious.** A committed golden op stream per corpus program is thousands of lines that churn on every promotion, which is a maintenance cost the phase should not hand 4f. **The cheaper shape is an invariant rather than a transcript**: assert over each program's compiled stream that no instruction the minimum promotion set covers emitted `Op::Generic`. That catches a promotion ceasing to fire, needs no committed bytes, and does not churn when an unrelated op is added.

* [x] Decide between the transcript and the invariant, and **write the reason down** -- if the invariant is chosen, say what it cannot see that a transcript would.
* [x] Build it over `corpus_cases()`.
* [x] **Prove it can fail**: pick a promotion, make `compile` fall through to `Op::Generic` for it, confirm the assertion reddens and names the program. Restore, rebuild.
* [x] **Then prove it adds coverage**: apply that same mutation with the new assertion removed, and record what else caught it. If the existing suite catches it identically, say so -- the assertion may still be worth having as a cheaper and more direct signal, but that is a different claim from "it adds coverage".

**Ends when** the assertion exists with both proofs recorded, or when the design question resolves to "not worth building" **with the reason written down**.

### Result: built, as an invariant, with both proofs recorded

`rust/crates/rexx-exec/src/ir/corpus_shape_tests.rs`, one test.
Working notes in `.superpowers/sdd/2026-08-09-phase-4e-ir/handoff-item2-report.md`.

**The shape: the invariant, at both levels the promotion set has, with expectations derived from each body's own parse tree.**
An instruction of a covered kind must have opened an `Op::Clause` region and must not have emitted `Op::Generic`; an instruction outside the set carries no claim, so a future promotion cannot redden it.
For an `Assignment` or a `SAY` -- the two constructs whose value expression can compile natively -- the last value-producing op in the clause region must be exactly what the expression's shape calls for, `Const`/`LoadConstant`/`Load`/`Arith`/`EvalExpr`.
That second half is **bidirectional**, which is what lets it see arithmetic promoting where it should not as well as ceasing to promote.

**The transcript was rejected for the cost the item names.**
**The op-kind count summary was judged on the same terms and also rejected**: it is still a committed artefact with a regeneration step, and it churns whenever any op is added, removed or split for a program even when the promotion set has not moved.
What it buys over the invariant is a changed count of the trace ops, and `ir_dual.rs` already diffs raw stderr between the two arms over the corpus population -- executed bytes rather than compiled shape, which is the stronger instrument.

**What the invariant cannot see.**
It reads an op's *kind* and never its fields, so a wrong register, jump target, region `end`, constant index or `SymbolId` passes; a transcript would catch those.
It says nothing about the ops it does not name, so a dropped `TraceLiteral`, `TraceRead`, `TraceOperator`, `EndBranch` or `Jump` passes.
It is a claim about what compilation emitted, not about what running it does.
The item's own wording -- "it cannot see an op emitted wrongly, only one not emitted at all" -- holds at the instruction level and is too pessimistic at the expression level, and holds for operands at both.

**One correction to the item's premises.**
`ir/golden.rs` is **not** ungated: `ir/mod.rs` declares `mod golden;` under `#[cfg(test)]`.
So `render` is unreachable from an integration test and the assertion cannot sit beside `corpus_cases()` in `tests/ir_dual.rs`; it is a unit-test module inside the crate, matching on `Op` variants directly, and uses `render` only to print the offending stream in the failure message.
The population is taken from the `corpus/phase-*.txt` directory listing rather than from a third copy of `SUBSET_FILES`, and `::ROUTINE`/`::METHOD`/`::ATTRIBUTE` bodies are swept as well as `main`.

**Anti-vacuity, asserted rather than described**: the population is non-empty and larger than one, the body count is at least the program count, and every construct of the minimum promotion set and every expression root is observed at least once by the sweep.

**Proof 1, it can fail.**
`CALL name`'s match guard made unreachable so it falls to `Op::Generic`.
Red, naming the program: `lang/address_env.rex main: instruction 31 (CALL name) is in the minimum promotion set and compiled to Op::Generic`.
Restored from `cp`, `sha256sum -c` OK, rebuilt, green.

**Proof 2, and the two mutations give opposite answers.**

* **Against a promotion that stops firing outright, it adds nothing.**
  With the assertion removed and the same `CALL name` mutation applied, `--no-fail-fast`, the existing suite catches it in three `ir::golden_tests` tests and one `ir::drive::tests` test, and `tests/spike.rs` aborts on a stack overflow.
  Recorded as what it is: a more direct signal, not new coverage.
* **Against a promotion that stops firing conditionally, nothing else in the workspace sees it.**
  `native_shape`'s recursion bounded at depth 8 -- a change 4f might reasonably want, since `push_native` recurses once per operator.
  With the assertion: 1427 passed, **1 failed**, the new test alone.
  Without it: **1427 passed, 0 failed, exit 0**.
  Every promoted expression in the golden set is hand-written and shallow, `corpus/lang/deep_nested_expr.rex` is one assignment nesting three thousand terms on purpose, and a bound between them is invisible to every hand-written witness.

Both mutations are recorded in the file's own module doc, so the limit travels with the code rather than only with this document.

**Gates.**
`cargo fmt --all --check` exit 0, `cargo clippy --workspace --all-targets -- -D warnings` exit 0.
`cargo test --workspace` dev: 1428 passed, 0 failed, 4 ignored, exit 0.
`cargo test --workspace --release` under all four STRICT gates: 1428 passed, 0 failed, 4 ignored, exit 0, with four `mode: STRICT` banners read from the uncaptured stream.
BASE is 1427 in both profiles, so the delta is the one test added.

**No profiler, benchmark or oracle run was taken**, and nothing here makes a performance claim.
The one performance question this leaves open, if anyone wants it settled: the sweep parses and compiles every corpus body on each `cargo test` run, and whether that is worth naming is a `cargo test` wall-clock question rather than an interpreter one.

---

## Item 3: an all-`Generic` body pays 22 more instructions per pass

**The fact.** `emptyloop`, whose body reaches no promoted clause at all, pays about **+22 instructions per pass** across the phase, and its ratio ends at **1.09223** -- the worst axis in the gate.
Its own prediction failed too: Task 9 predicted no movement and it moved +13 per pass, entirely in the IR arm, on an axis that compiles no arithmetic op.

**The gate refuses to attribute it and names one candidate: the driver's `match` widening.**
Every op variant added to `Op` widens the dispatch that *every* op pays, including bodies that promote nothing.
**If that is the mechanism, it is a real design cost with a known shape** -- the gate's own suggestion is a jump table or a two-level encoding -- and it gets worse with every future promotion, which makes it worth knowing now rather than after Phase 5 adds message sends.

* [ ] Profile `emptyloop`'s IR arm at the phase's start and at head; compare functions.
* [ ] Test the dispatch-width hypothesis with a build that isolates it. **A spike, measured and reverted, not an argument.**
* [ ] If confirmed: record what the encoding change would cost and what it would buy, as 4f's input. **Do not build it here.**
* [ ] If refuted: record what was ruled out.

**Ends when** the mechanism is named with a build behind it, or after **three** hypotheses have been tested and refuted.

### Result: three hypotheses refuted, including the one the gate named -- and the axis moves 15 per pass under code that never runs

Working notes in `.superpowers/sdd/2026-08-09-phase-4e-ir/handoff-perf-report.md`.
Figures are rows of `rust/bench-baselines/phase-4e-arms.tsv` under task `item3`, or `callgrind` readings named as such.

**The fact reproduced, and the two instruments agree.**
`instructions:u`, one sitting, five rounds: `e63e8a00` has tw 1515.002 and ir 1634.002 for a gap of **119.000**, and head has tw 1518.002 and ir 1658.002 for a gap of **140.000** at ratio **1.09223**.
`callgrind` reads 119.004 and 140.001 for the same two gaps, which is what licenses the bisect below to carry the same weight.

**Where the +21 lands.**
The IR arm's own cost splits exactly into a part that scales with the loop body's clause count and a part that does not, and `callgrind` confirms exactly one `run_ops` entry per pass whichever body runs.
At `e63e8a00` it is 22.0 per `Op::Generic` plus 97.0 per entry; at head it is 31.0 plus 109.0.
`emptyloop` pays one of each per pass, which is why it is the worst axis in the gate: **it has no second clause to amortise the 109 over**.
Per commit, the two halves move as +7/+2 at `ea17a699`, +0/+4 at `f0d12ebe`, +2/+6 at `6b5fac3b`, and every one of those lands in `run_ops`' own self cost.

**Hypothesis 1, the gate's candidate: the driver's `match` widening. Refuted, and in the wrong direction.**
`Op` was given twelve and twenty-four further variants, each with an arm in both of `run_ops`' exhaustive matches, emitted by `compile` behind a condition never true at run time and not foldable at compile time, with `size_of::<Op>()` held at 12 by the crate's own assertion.
The widening is real and demonstrable: `run_ops`' symbols grow from 12228 bytes to 13387, 14716 and 28354, and the spike's environment-variable name is in each spike binary and in no other.
The gap per pass goes 140.001 to 136.005, 137.002 and 125.004 -- **cheaper every time**, confirmed on `rexx-arms` at the committed length, where the twelve-arm build reads a gap of **125.000 and a ratio of 1.08234** against the control's 140.000 and 1.09223.

**Hypothesis 2, the op stream's stride. Ruled out without a build, by an assertion already in the tree.**
`ir/mod.rs` carries `const _: () = assert!(size_of::<Op>() == 12);` at every commit in the range, so the width the driver indexes the stream at never moved.

**Hypothesis 3, `Chunk` gaining fields.**
`hints` arrives at `f0d12ebe` and `calls` at `6b5fac3b`, which are two of the three commits that move the axis, and `f0d12ebe` adds no `Op` variant at all.
Two dead fields of exactly those shapes, constructed and never read, cost **zero**: 31.004 against 31.002 per op, 108.988 against 108.988 per entry.
Refuted.

**What is left, and it is the finding worth carrying into 4f.**
The cost is real, it is IR-only, and it is not a function of anything the design controls: it is what LLVM emits for `run_ops` as that function grows, and it is not monotone in the function's size either.
**A build differing from head only by code that never executes moves this axis by -15.000 per pass** -- 71% of the whole +21 the phase move is being attributed from, and 11% of the gap itself.
So `emptyloop`'s instruction count carries a between-build codegen sensitivity larger than most of the individual steps it is used to measure, and a paired comparison on this axis has to clear that before its sign means anything.

**What a fix would cost and buy, as 4f's input. Not built here.**
The gate's own suggestions -- a jump table, a two-level encoding -- are aimed at the dispatch, and the dispatch is not what costs.
The lever the numbers do support is the 109 per entry: it is paid because a promoted `DO` body re-enters `run_ops` once per pass, so a driver that keeps the loop inside would remove a term that is 78% of this axis' whole IR-only cost at head.
That is a change to how a loop body is driven rather than to how an op is encoded, and its price on the axes that do promote is unmeasured.

---

## What is not chased here

Two of the final review's five unsettled items stay with 4f, deliberately:

* **The enter/leave trade at final promotion coverage.** Answering it needs a hand-written pre-split driver carrying the current op set, and nothing cheaper answers it. That is a large build for a question 4f will answer as a side effect of its own measurements.
* **Whether the exact-stderr trace tests exercise promoted clauses under `Interp::new = Engine::Ir`.** Discharged once by a build; closing it properly means a mechanism in the tree, which is a test-architecture change rather than an investigation.

## The record

Append results to this file, under each item.
**A refuted hypothesis is recorded with the same weight as a confirmed one**, which is Phase 4f's rule adopted early: the record's main job is stopping the next person retrying a dead end.
