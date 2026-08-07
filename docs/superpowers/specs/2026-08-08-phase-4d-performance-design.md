# Phase 4d -- interpreter performance

**Status:** design, not yet planned.
**Entry:** Phase 4c closed.
**Blocks:** Phase 4's own exit gate, which requires `samples/rexxcps.rex` to run "within the ratio bar".

## Goal

Close the measured performance gap between this crate and the C++ oracle, across every benchmark axis that can be measured today, before Phase 5 multiplies the call sites.

Phase 4 does not close until this does.
The Phase 4 row's exit gate reads "`samples/rexxcps.rex` runs clean and within the ratio bar", and the ratio bar is failed by 6.7 times.

## Why this is a phase and not a cleanup

The gap is large, it is not uniform, and the shape of it is not yet attributed to anything.

`rexxcps` reports a single headline ratio of 10.02.
Treating that as "the interpreter is ten times slow" would be wrong: the per-axis spread below runs from 3.8 to 25.2, and an average over that spread describes none of it.
A single-benchmark target would also invite optimising the benchmark, which is why the gate is per axis.

Doing this before Phase 5 rather than after is a cost argument, not an aesthetic one.
Phase 5 executes `CoreClasses.orx` and brings 32 classes at once; every hot path fixed afterwards is fixed against a larger surface, and every hot path *not* fixed becomes a property of the class library's measured behaviour.

## What can be measured, and what cannot

`rust/bench-programs/` holds eight programs, one per D9 dimension, of which seven carry committed oracle baselines in `docs/superpowers/plans/perf-baseline.md`.

**Three of the eight do not run under this crate today**, all stopping on the same cause:

| program | axis | status |
|---|---|---|
| `dispatch.rex` | method dispatch | `rc=120`, "a message send is not implemented (Phase 5)" |
| `alloc.rex` | allocation throughput | `rc=120`, same |
| `heapshape.rex` | heap shape | `rc=120`, same |

**This must be stated in the gate rather than quietly reducing the denominator.**
Two of the three -- method dispatch and allocation throughput -- are axes D9 names explicitly, so 4d closes with two of D9's dimensions unmeasured and unmeasurable, and Phase 5 inherits them.
An earlier phase's gate reported coverage over a subset that silently shrank; the mechanism there was the same, and the fix is to name what is missing.

## Provisional sizing, and what it is not

Single run per side, release build both sides, this machine, no spread, taken 2026-08-08 while other work was running.
**These are sizing figures for planning, not the measurement.**
Stage 1 produces the measurement, interleaved and with variance, and stage 1's numbers supersede these.

| axis | oracle | rust | ratio |
|---|---|---|---|
| `varlookup` | 1.22 s | 30.79 s | 25.2 |
| `strings` | 0.88 s | 12.94 s | 14.7 |
| `compound` | 1.13 s | 15.61 s | 13.8 |
| `rexxcps` | -- | -- | 10.02 (5 runs each, 0.9% spread) |
| `arith` | 1.17 s | 4.48 s | 3.8 |
| `startup` | ~0.00 s | ~0.00 s | below this instrument's resolution |

Three observations that shape the plan, each falsifiable at stage 2:

* **`varlookup` is the worst axis at 25 times, and it is the most fundamental operation there is.**
  Plain variable lookup sits under every other axis, so a fix here moves everything, and a failure to explain it undermines any attribution built on top of it.
* **`arith` at 3.8 is the best**, which is evidence for `rexx-num` rather than against it, and suggests the gap is in the interpreter rather than in the numeric core.
* **`strings` at 14.7 may contradict a prediction made when this phase was proposed.**
  That prediction was that the deferred `Cow`/copy work would not move performance, on the grounds that `rexxcps` uses only short strings.
  It still holds for `rexxcps`, but `required_string` renders every string argument through `to_text(...).into_owned()`, which is a `malloc` plus `memcpy` on **every string builtin call** regardless of size.
  If stage 2 attributes the `strings` axis to that copy, then the deferred memory work is a performance item after all, and this spec's scope boundary needs revisiting rather than defending.

## The five stages

**1. Measure.**
All runnable benchmark programs plus `rexxcps`, both interpreters, one machine, one session, interleaved.
Output is a ratio per axis with its spread, plus an explicit list of the axes that could not run and why.

**2. Attribute.**
`samply` on the Rust side, analysed through `pollard`.
The deliverable is a named cause per axis, not a flame graph.
D9's two design inputs are read before profiling and not re-derived: the compound-variable memoisation prototype that measured -24% on stem-heavy workloads, and the existing performance profile of where interpreter time actually goes.

**3. Write the gate.**
Numeric bars per axis, into `docs/superpowers/plans/phase-4d-gate.md`, **before any optimisation lands**.
This is the step that makes stage 1 a measurement rather than a target.
A bar written after the optimisation is a description of what happened.

**4. Optimise.**
One named cause per task.
Each task carries a before and after with spread, and lands with the full suite green.

**5. Gate.**
Re-measure every axis, record against the bars written at stage 3, and state which of D9's dimensions remain unmeasured.

## Measurement discipline

A performance phase is unusually easy to fake, so the instrument's own rules are part of the spec.

* **Interleave the runs.** Not all-oracle-then-all-rust. This is a 32-core AMD part and frequency drift across minutes is larger than several of the effects being chased.
* **Never report a single figure.** N runs and the spread. The 10.02 is credible precisely because it arrived with a 0.9 per cent per-side spread; the table above is not, and says so.
* **Release build on both sides.** The oracle is a CMake `Release` build; a debug comparison is not a comparison.
* **Correctness is not a tradeoff.** The full suite stays green at every commit: 1,289 tests, corpus 50 of 50, keyword 888 of 896 bodies, assertions 4,224 of 4,259 rows, `base/bif` 4,920 of 4,999 value rows and 184 of 186 raise rows.
  R1's rule applies at every step, not only at the `rexxcps` gate: an interpreter that computes something different is not faster.
* **A speedup claim is falsified like a bug fix.** Revert the change and show the number moves back. Without that, the claim is a coincidence with a commit message attached.
* **`startup` needs a finer instrument than the one used above**, or it is recorded as unmeasurable at this resolution. It is also the axis whose fix is D2's saved-image decision, scheduled for Phase 5, so it is reported rather than gated.

## Scope

**In scope.** Anything inside the interpreter that the attribution names, including the heap and value representation.

**The representation is fair game** -- decided 2026-08-08.
The guardrail is not a restriction on what may change but on what must be recorded: whatever representation change lands, the measurement that justified it goes in the commit, so D1's record says why the heap changed rather than only that it did.

**Out of scope**, and staying deferred:

* the `Cow`/copy work and the fallible-own fix, **unless stage 2 attributes an axis to it**, per the `strings` observation above
* `Rc`/`Arc` and any refcounted value representation adopted for its own sake
* the seven SIGABRT programs and D19's 512 MiB reservation
* the three Phase 5 axes, which cannot be measured here at all

4d may record what it learns about any of these; it fixes none of them by default.

## Risks

| risk | mitigation |
|---|---|
| Optimising `rexxcps` specifically | Per-axis bars. A fix that moves one axis and nothing else is a special case, and the gate says so. |
| A "structural" fix that is a benchmark-specific special case | Each optimisation task names the cause it addresses and shows the axis or axes it moves. |
| Performance work breaking behaviour subtly | The differential suite is unusually strong here, and green is required at every commit rather than at the end. |
| The bar written at stage 3 turns out unreachable | Stage 3 writes it from stage 2's attribution, so it is grounded in named causes. An unreachable bar is then a finding about the attribution, and is escalated rather than lowered. |
| Two of D9's dimensions unmeasurable | Named in the gate, handed to Phase 5. Not silently dropped. |

## Open questions for the plan

* Does `varlookup` at 25 times share a cause with `compound` at 13.8, or are they separate? They plausibly share variable resolution, and the answer changes the task decomposition.
* Is `startup` worth a finer instrument in 4d, given its fix is a Phase 5 decision?
* Should the three blocked programs get 4c-compatible variants so their axes are measurable now, or is a variant that avoids message sends no longer measuring the axis it is named for?
