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

---

## Item 2: no corpus-wide assertion over the compiled op stream

**The hole, in the gate's own words.** There is no assertion over the compiled op stream of *every corpus program*.
What exists is a per-construct golden set with a negative control, plus a corpus-wide assertion that `chunks_refused` is zero on both arms -- **which says every body compiled, not what it compiled to.**
So a promotion that silently stopped firing for a construct the golden set does not cover would leave every gate green.

**This is the only item here that builds something rather than explaining something**, and the gate calls it cheap because both halves already exist: the serialiser is `ir/golden.rs` and the population is `corpus_cases()`.

**The design question this must answer, and it is not obvious.** A committed golden op stream per corpus program is thousands of lines that churn on every promotion, which is a maintenance cost the phase should not hand 4f. **The cheaper shape is an invariant rather than a transcript**: assert over each program's compiled stream that no instruction the minimum promotion set covers emitted `Op::Generic`. That catches a promotion ceasing to fire, needs no committed bytes, and does not churn when an unrelated op is added.

* [ ] Decide between the transcript and the invariant, and **write the reason down** -- if the invariant is chosen, say what it cannot see that a transcript would.
* [ ] Build it over `corpus_cases()`.
* [ ] **Prove it can fail**: pick a promotion, make `compile` fall through to `Op::Generic` for it, confirm the assertion reddens and names the program. Restore, rebuild.
* [ ] **Then prove it adds coverage**: apply that same mutation with the new assertion removed, and record what else caught it. If the existing suite catches it identically, say so -- the assertion may still be worth having as a cheaper and more direct signal, but that is a different claim from "it adds coverage".

**Ends when** the assertion exists with both proofs recorded, or when the design question resolves to "not worth building" **with the reason written down**.

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

---

## What is not chased here

Two of the final review's five unsettled items stay with 4f, deliberately:

* **The enter/leave trade at final promotion coverage.** Answering it needs a hand-written pre-split driver carrying the current op set, and nothing cheaper answers it. That is a large build for a question 4f will answer as a side effect of its own measurements.
* **Whether the exact-stderr trace tests exercise promoted clauses under `Interp::new = Engine::Ir`.** Discharged once by a build; closing it properly means a mechanism in the tree, which is a test-architecture change rather than an investigation.

## The record

Append results to this file, under each item.
**A refuted hypothesis is recorded with the same weight as a confirmed one**, which is Phase 4f's rule adopted early: the record's main job is stopping the next person retrying a dead end.
