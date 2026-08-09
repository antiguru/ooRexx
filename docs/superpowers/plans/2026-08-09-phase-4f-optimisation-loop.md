# Phase 4f -- the optimisation loop

> **For agentic workers:** this is **not** a task list, and it must not be executed as one.
> It defines a loop, its record, its accept rule and its stopping condition.
> Read [Why this is a loop](#why-this-is-a-loop-and-not-a-task-list) before doing anything else.

**Supersedes 4d-2.** Decided 2026-08-09.
**Entry:** Phase 4e landed. Unit 0 below is a set of rules, not a measurement to complete first.
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

## Unit 0 -- the two measurement rules

**Withdrawn 2026-08-09: the upfront noise characterisation.** Both wordings kept.

It required 30 to 50 repetitions per side per axis to characterise a distribution, then interventions to reduce it, targeting a band under one per cent before anything else could start.
A run was begun and stopped at 314 rows.

**It conflated two different measurement problems and priced the harder one upfront.**
Deciding "did this change help" and deciding "are we at parity" are not the same question, need different instruments, and cost wildly different amounts.
And the expensive one is only expensive **near the boundary**: no amount of instrument precision changes the verdict on an axis sitting at 10.61x.

### The loop's accept rule: paired, same sitting, cheap

**A candidate is accepted if a paired interleaved comparison against the immediately preceding binary shows a reproducible gain.**

Both binaries run alternately within one loop, on one machine state, so frequency, residency, page cache and thermal state hit both arms and cancel.
That is why this is cheap: a handful of alternating pairs resolves a few per cent, and it needs no knowledge of any absolute noise band.

The measurement is **relative to our own previous state**, never to the oracle. The oracle does not enter the accept rule.

**Interleave within one loop; never read across two separate runs.** That rule stands and is the reason this works.

### If a few runs do not show it, there is none

**The working model, and it is a hard rule for candidates: an improvement that a handful of paired runs cannot show does not exist.**
Discard it. Do not add runs, do not add pairs, do not re-run it later hoping for a better sitting.

Three reasons it holds here, and the first is the one that matters:

* **Adding runs until a candidate passes is searching for a result rather than measuring one.** The loop tries many candidates; if each may be re-run until it looks good, some will look good by chance, and the record will fill with accepted changes that do nothing. The cost is not one bad change -- it is that the record stops being evidence.
* **A change too small to see in a few runs is too small to matter at this distance.** The axes are between 2.08x and 10.61x from the bar. Closing that needs changes worth tens of per cent, and dozens of one-per-cent wins is not a plan.
* **Throughput is the loop's scarce resource.** Every hour spent rescuing a marginal candidate is an hour not spent finding a large one, and the large ones are what the bar needs.

**This does not contradict the gate's escalation rule, and the difference is worth stating because it will otherwise read as licence to re-run a failing candidate.**
Escalation applies to **one pre-specified question asked once** -- is this axis at parity -- where spending more measurement is legitimate because nothing is being selected.
It does **not** apply to candidate selection, where re-running until something passes is exactly the failure above.
A candidate gets its few runs and a verdict. An axis at the boundary gets more runs and no choice about which answer it wants.

### The gate's verdict: escalate only when the answer is close

**Measure cheaply first, and spend more only if the result is near 1.0.**

* If the ratio is far from 1.0 relative to the spread the runs themselves show, it is decided. Six axes between 2.08x and 10.61x need no further measurement to be called NOT MET, and none of them ever did.
* If it lands near 1.0, that axis alone earns more runs -- targeted, on the axis in question, until the interval separates from 1.0 or demonstrably will not.

The escalation is **adaptive and per axis**, not a global constant established in advance.
There is no need to know "the noise profile"; there is a need for enough runs to separate a particular number from 1.0, and that requirement scales with how close it already is.

**Sensitivity is demonstrated when it is claimed, not before.** When an axis escalates, a control pair differing by a known small amount is run alongside it, so a tight interval is shown to be sensitivity rather than blindness.

### If an axis cannot be certified at parity

**Then it must show measured, reproducible relative improvement instead**, against a named prior state, with the shortfall from parity recorded.
That is a weaker claim than parity and is labelled as one -- but it is a *measured* claim, which "within an unmeasurable band" is not.

### What was kept from the withdrawn unit

* **The harness unification.** `rexx_bench::child` now holds one capped, directory-pinned child wrapper shared by `rexx-bench-suite` and the band tool, so the `/bin/sh`+`Instant` against bash+`date` split recorded in `phase-4d-attribution.md` no longer exists in tooling. That split contaminated the 7.2% figure and removing it is worth having on its own.
* **The control programs**, for the escalation path above.
  `rust/bench-control/alloc4c-101.rex` is `bench-programs/alloc4c.rex` with the loop bound raised by exactly 1% and nothing else changed, and a test asserts both halves of that so the pair cannot quietly stop being the known amount it is used as.
* **One measured reading from the 314 rows before the run was stopped**, set out below rather than summarised, because the summary this bullet first carried -- that pinning widened the spread -- is not what the per-axis rows say.

### The reading the stopped run left

**Three or four passes per arm per axis. It decides nothing, and it is written down only so the next person does not repeat it blind.**
Measured 2026-08-09 08:10:55 to 08:32:49 at `77679f75` plus the harness landed with this reading, `rexx-run` sha256 `c3b2516069a1b5f5d504613986f00b69e0c0fc892c041958212d45a699c0e967` against the three oracle objects `phase-4d-gate.md` names, load average 1.01 to 1.56 throughout, every axis printing one constant value across all 314 rows.

| axis | unpinned median ratio | pinned median ratio | shift | unpinned max deviation | pinned max deviation |
|---|---:|---:|---:|---:|---:|
| `alloc4c` | 2.0029x | 2.0018x | -0.06% | 0.63% | 0.58% |
| `arith` | 2.7072x | 2.6548x | -1.94% | 0.75% | 0.58% |
| `compound` | 5.8641x | 5.7723x | -1.57% | 0.48% | 1.20% |
| `strings` | 10.5219x | 10.0853x | -4.15% | 0.77% | 1.88% |
| `varlookup` | 4.3231x | 4.1053x | -5.04% | 1.24% | 0.48% |

Max deviation is the largest deviation of a pass's ratio from the median over that arm's passes on that axis.
**Three dispersion statistics are available from these rows and the next paragraph turns on which one is read**, so the table names its own rather than calling itself the envelope: max deviation from the median, as above; the full range, `(max - min) / median`; and the interpolated 2.5/97.5 half-width.

**What the rows support: pinning moves the ratio.**
Four of five axes shifted in the same direction, by up to 5.04% on `varlookup` -- larger than either arm's dispersion there on any of the three statistics.
So `taskset` is not a control that leaves the quantity alone while tidying its variance, and a pinned measurement is not comparable with an unpinned one.

**One figure disagrees with a commit message and the rows settle it.**
`c0ad58b7`'s message puts `varlookup`'s shift at 4.77%; recomputed from the rows it is **5.04%**, which is what this table and every sentence around it carry.
A commit message cannot be edited, so the disagreement is recorded here rather than left for a reader to find and have to adjudicate.

**What the rows do not support: that pinning widened the spread.**
That was the controller's summary and it is wrong as a general claim.
Which axes narrowed depends on which of the three statistics is read, and exactly one axis is sensitive to the choice.
`arith` narrows by max deviation (0.75% to 0.58%) and by interpolated half-width (0.72% to 0.54%), and **widens** by full range (0.81% to 0.88%).
`alloc4c` and `varlookup` narrow under all three, `compound` and `strings` widen under all three.
So the disagreement is one axis wide, and the table's column is the first of the three.
The worst-axis figure rose from 1.24% to 1.88% -- max deviation, as the table -- through a change of which axis is worst, `strings` overtaking a `varlookup` that pinning more than halved.
At three or four passes none of this is separated from sampling noise.
The usable statement is that pinning did not visibly reduce the spread while it did visibly move the ratio, so core migration is not obviously the variance source and pinning is not free.

**Adding `perf stat` moved the ratio again, in the same direction, on all five axes**: -0.33%, -1.32%, -1.57%, -1.28% and -0.58% on `alloc4c`, `arith`, `compound`, `strings` and `varlookup`.
That is the direction a fixed per-process cost produces on a ratio whose sides differ in absolute duration -- the oracle's medians run 0.88 s to 1.25 s here and this crate's 2.31 s to 8.95 s -- but nothing measured `perf stat`'s startup cost directly, so the mechanism is a candidate rather than a finding.

**The CPU governor could not be fixed, and that is a property of the machine rather than an omission.**
`/sys` does not exist in this environment: `ls /sys` reports no such file, `/proc/mounts` carries no `sysfs` line, and every `scaling_governor` is therefore absent -- the same absence `rust/CLAUDE.md` records as the reason no cgroup limit can be created here.
Frequency does vary, 1.479 to 2.972 GHz across the 32 logical CPUs at rest.
A later reader on a machine with sysfs should try it rather than assume it was weighed and dropped.
Counting cycles is the substitute available here, and it is a substitute rather than an equivalent: cycles ignore frequency scaling, but a cycle ratio equals a wall-clock ratio only if both sides run at the same average frequency.

### What this means for the gate's 7.2% band

`phase-4d-gate.md`'s band was derived as the largest of three observations and used as a global threshold.
Under the rules above it is not needed as a global constant, and the gate is amended to say so: the band is replaced by the escalation rule, which spends measurement where the answer is close and nowhere else.

## The loop

### The record

One committed file, appended to, never rewritten: every attempt, its hypothesis, its measured effect, and its disposition.

**Failures are recorded with the same weight as successes**, because the record's main job is stopping the next person retrying a dead end.
`smallvec` for `Number::digits` was measured and rejected once already; that result is worth more than most accepted changes, and it survived only because someone wrote it down.

Each entry carries: the cause or hypothesis, the axes it predicted it would move and by how much, the measured move, whether it was accepted, and the commit.

### The accept rule

**A change lands only if a paired interleaved comparison against the immediately preceding binary shows a reproducible gain**, per Unit 0.

Against our own previous state, not against the oracle -- the oracle is what the *gate* compares to, and it does not enter the accept decision.
A change whose effect does not survive the pairing is discarded regardless of how good the theory is, and is **not** re-run with more pairs -- see [If a few runs do not show it](#if-a-few-runs-do-not-show-it-there-is-none).
"Reproducible" means the sign holds across the alternations, not that a single pair favoured it.

**A change that reaches its axis's target by a route other than its stated hypothesis has not confirmed the hypothesis**, and says so in the record.

### The re-profile cadence

**After every accepted change, re-profile the axes it moved.**
The shares are now different, and the next candidate is chosen from the new profile, not from the entry attribution.

The entry attribution is the loop's **starting point**, not its plan.
When re-profiling stops proposing candidates whose effect survives the pairing, that is a stopping condition, not a prompt to work harder on the list.

### The stopping rule

The loop ends on the first of:

1. **Every classic-Rexx axis within noise of the oracle or better.** This is the only ending that closes Phase 4.
2. **Re-profiling yields no candidate whose effect survives the pairing on an axis still short.** Record the shortfall, the profile that produced no candidate, and what would be needed. This is a finding, not a failure, and it is what tells us a structural change is required rather than another local fix.
3. **A measured result contradicts the model badly enough that continuing would be guessing.** Stop and re-derive.

## What this phase does not do

* **It does not plan Phase 4e.** The IR is construction and has its own spec.
* **It does not decide representation questions on its own authority.** `size_of::<Body>()` at 80 bytes, dominated by a cold `Stem` variant, puts D1's recorded guidance against boxing enum variants in tension with D1's own Phase 4 re-measurement. That is a decision to be taken deliberately, with both wordings recorded, not absorbed into an optimisation attempt.
* **It does not touch `strings` or `heapshape`'s bars until they have attributions.** Both currently have none -- `strings` has 17.5% named against a 2.8% measured win, and `heapshape` has a named mechanism with no share decomposition. Attributing them is loop work; giving them bars before that would be inventing numbers.

## Open questions

* **Whether cycle counts can replace wall time** for the accept rule. They are far more reproducible, but the oracle side must be measured the same way and the gate's own text is written against wall-clock ratios. The reading above adds a second cost to check: turning `perf stat` on moved the measured ratio on all five axes, so a counted run and an uncounted one are not interchangeable even when only the wall time is read off.
* **The platform matrix.** `rexx-bench-suite` is Linux-only as written, and `:35` names five platforms. Harness work precedes machine access.
* **How many runs an axis near 1.0 actually needs.** Reproducibility differs by axis -- `varlookup` reproduced at 0.94% within a run while `strings` reached 5.47% -- so the escalation cost is per axis and is not known in advance for any of them. It is answered when an axis first escalates, not before.
