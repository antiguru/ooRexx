# Phase 4f -- the optimisation loop

> **For agentic workers:** this is **not** a task list, and it must not be executed as one.
> It defines a loop, its record, its accept rule and its stopping condition.
> Read [Why this is a loop](#why-this-is-a-loop-and-not-a-task-list) before doing anything else.

**Supersedes 4d-2.** Decided 2026-08-09.
**Entry:** Unit 0 below closed, and Phase 4e landed.
**Exit:** parity on every classic-Rexx axis, or the loop's stopping rule fires with the shortfall recorded.
**Phase 4 closes when this closes.**

## Why this is a loop and not a task list

4d-2 was specified as one-cause-per-task, planned from `phase-4d-attribution.md`'s seven causes.
That shape cannot work, and the evidence is in the attribution itself.

**An attribution has a shelf life measured in landed changes.**
The attribution measures `compound` as C2 41.5%, C1 14.2%, C4 12.7% of that axis's time.
It also measures that landing the small-integer `//` fix moves `compound` by **-51.6%**.
Land it, and every remaining share on that axis is wrong, because the denominator moved.
The derived bars for `compound`'s other causes are stale the moment the first task lands, and a linear plan would spend its second task optimising against a model it had already invalidated.

That is not a defect in the attribution. It is what optimisation is: measure, hypothesise, try, re-measure, keep or discard, **re-profile**, repeat -- with an unknown iteration count, where most attempts fail, and where the candidate queue is *regenerated* rather than consumed.

**The distinction that decides the machinery.**
Phase 4e is **construction**: it has deliverables, interfaces and a definition of done, so the linear SDD process fits it, and that process is what produced 4d-1.
This phase is **search**. It has no deliverables, only a stopping condition.
Applying task-list machinery to search produces exactly the false precision this phase exists to avoid -- a plan that looks complete because its tasks are done, while the thing it was for is unmet.

## The bar

**Within noise of the oracle, or better, on every classic-Rexx axis.**
Decided 2026-08-09, and it is stricter than what `phase-4d-gate.md` currently permits, not looser.

**A recorded debt is not available on a classic-Rexx axis.**
`arith`, `compound`, `strings`, `varlookup`, `alloc4c` and `rexxcps` close on the bar or the phase stays open.
The debt mechanism survives only for work that genuinely belongs to a later phase -- `dispatch`, `alloc.rex`, `startup` -- where it is a scoping statement rather than a concession.

**The reason is structural, not perfectionism.**
Performance work that lands after a phase is declared done may change the implementation structurally, and then it is a patch on top of something already certified.
Anything that would restructure the interpreter has to happen before the gate closes, not after.

## Unit 0 -- characterise the noise, then reduce it

**Nothing in this phase is decidable until this closes, and it blocks the gate as much as it blocks the loop.**

`phase-4d-gate.md` currently declares an undecidable band of 7.2%.
That figure is **the largest of three observations**, adopted as a lower bound because three runs cannot support anything better.
A 7.2% band means a change worth 5% is unmeasurable, and most individual optimisations are worth less than that.
It is a broken instrument, not a fact about the machine.

**Step 1: characterise.**
30 to 50 repetitions per side per axis, same binary, same program, same machine state, so the *distribution* of the ratio is known rather than its observed extreme.
Report the distribution, not a single number: median, spread, and the shape, because a long tail and a wide symmetric spread need different responses.

**Step 2: reduce.**
Most of the variance is likely attackable and none of it requires touching the interpreter:

* pin to a core with `taskset`, so a migration mid-run stops being a measurement,
* fix the CPU governor, so frequency scaling is not part of the signal,
* measure cycles rather than wall time where the comparison allows it,
* control residency and page-cache state between runs,
* remove the harness difference `phase-4d-attribution.md` records between the suite and the ad-hoc scripts, so one harness produces every number.

**Step 3: re-derive the band from the reduced distribution**, and amend `phase-4d-gate.md` with both wordings and the reason.

**The target is a band under one per cent**, because the loop below cannot accept a change it cannot measure.

**Do not flip UNDECIDED into a pass while the band is wide.**
"Within noise" is the right bar only when noise is small; with a 7.2% band it would mean 7.2% slower passes, and a worse instrument would make the gate easier. That is the vacuity shape this project has shipped before. The band shrinks first.

## The loop

### The record

One committed file, appended to, never rewritten: every attempt, its hypothesis, its measured effect, and its disposition.

**Failures are recorded with the same weight as successes**, because the record's main job is stopping the next person retrying a dead end.
`smallvec` for `Number::digits` was measured and rejected once already; that result is worth more than most accepted changes, and it survived only because someone wrote it down.

Each entry carries: the cause or hypothesis, the axes it predicted it would move and by how much, the measured move, whether it was accepted, and the commit.

### The accept rule

**A change lands only if its measured effect exceeds the characterised band, interleaved, against the oracle.**

Below the band it is discarded regardless of how good the theory is.
This is the rule that makes Unit 0 a prerequisite rather than a nicety.

**A change that reaches its axis's target by a route other than its stated hypothesis has not confirmed the hypothesis**, and says so in the record.

### The re-profile cadence

**After every accepted change, re-profile the axes it moved.**
The shares are now different, and the next candidate is chosen from the new profile, not from the entry attribution.

The entry attribution is the loop's **starting point**, not its plan.
When re-profiling stops proposing candidates above the band, that is a stopping condition, not a prompt to work harder on the list.

### The stopping rule

The loop ends on the first of:

1. **Every classic-Rexx axis within noise of the oracle or better.** This is the only ending that closes Phase 4.
2. **Re-profiling yields no candidate above the band on an axis still short.** Record the shortfall, the profile that produced no candidate, and what would be needed. This is a finding, not a failure, and it is what tells us a structural change is required rather than another local fix.
3. **A measured result contradicts the model badly enough that continuing would be guessing.** Stop and re-derive.

## What this phase does not do

* **It does not plan Phase 4e.** The IR is construction and has its own spec.
* **It does not decide representation questions on its own authority.** `size_of::<Body>()` at 80 bytes, dominated by a cold `Stem` variant, puts D1's recorded guidance against boxing enum variants in tension with D1's own Phase 4 re-measurement. That is a decision to be taken deliberately, with both wordings recorded, not absorbed into an optimisation attempt.
* **It does not touch `strings` or `heapshape`'s bars until they have attributions.** Both currently have none -- `strings` has 17.5% named against a 2.8% measured win, and `heapshape` has a named mechanism with no share decomposition. Attributing them is loop work; giving them bars before that would be inventing numbers.

## Open questions

* **Whether cycle counts can replace wall time** for the accept rule. They are far more reproducible, but the oracle side must be measured the same way and the gate's own text is written against wall-clock ratios.
* **The platform matrix.** `rexx-bench-suite` is Linux-only as written, and `:35` names five platforms. Harness work precedes machine access.
* **Whether the band can be made per-axis.** `varlookup` reproduced at 0.94% within a run while `strings` reached 5.47%; one global band is set by the worst axis and may be needlessly strict on the others.
