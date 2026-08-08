# Phase 4d -- interpreter performance

**Status:** design, revised after review.
**Entry:** Phase 4c closed.
**Blocks:** Phase 4's exit.
**Planned as two units:** 4d-1 (measure, attribute, write the gate) and 4d-2 (optimise), the second planned only after the first closes.

## The bar

**Parity, unamended.** Global Constraints, `2026-07-27-rust-rewrite.md:39`:

> **Shipping gate (parity).** No phase from 2 onward closes with a Rust subsystem slower than its C++ counterpart on the Phase 0 benchmark suite, measured on Linux and macOS.
> "Slower" means the criterion point estimate falls outside the C++ baseline's confidence interval on the slow side.
> This is the rule everywhere unless a phase says otherwise.

**4d does not say otherwise.** Decided 2026-08-08.

Three consequences the plan must carry rather than discover:

* **The definition of "slower" is a confidence-interval overlap, not a ratio threshold.** Ratios are reported because they are legible; the gate is decided on CI overlap.
* **macOS is in the gate text.** Whether a macOS measurement is obtainable is an open question below, not an assumption.
* **1.5 is not this phase's bar.** `:40` scopes 1.5 to Phase 1's viability threshold for a heap benchmarked without an interpreter, and says the parity gate applies from Phase 2 on. D9 (`:381-390`) states no threshold and points here. An earlier draft of this spec resolved the Phase 4 roadmap row's undefined phrase "the ratio bar" to 1.5 and reported the `rexxcps` miss as 6.7 times; **under the gate that binds it is 10.0 times**, and per axis it reaches 25.

If 4d cannot reach parity, the outcome is a **recorded debt in the shape D1 and Phase 2 already used** -- named per axis, with its measurement, carried forward. It is not a lowered bar.

## What 4d owns

Beyond the cps work, 4d discharges two obligations the master plan scheduled for Phase 4 and that have now survived two phases unmeasured. Decided 2026-08-08.

**D1's Phase 4 re-measurement.** `d1-decision.md:19-22` records the Phase 1 heap result as a debt rather than a pass, "and this must be re-measured at Phase 4 when a real interpreter exists to measure on equal footing."

**D1's pre-registered fix is a named 4d-2 candidate.** `d1-decision.md:50-56`:

> the fix is already specified -- a side byte-arena indexed by `(offset, len)`, which works because `RexxString` is immutable.
> **It was not built, because the gate did not require it.**
> If the Phase 4 re-measurement misses parity, this is the first thing to try, and the plan says explicitly that boxing the enum variants would make it *worse*.

**Phase 2's `arith` parity debt.** `d1-decision.md:76` records that Phase 2's gate asked for parity on `arith` and did not get it (1.22 times), with `:102-105` warning the figure is a lower bound that will get worse rather than better.

This pulls string and value representation into scope, which is consistent with the 2026-08-08 decision that the representation is fair game, and supersedes an earlier draft's deferral of it.

## The axes

D9 (`:385`) names eight dimensions. `rust/bench-programs/` holds eight programs, one per dimension. Seven carry committed wall-clock oracle baselines in `perf-baseline.md`; `heapshape.rex` is D1's GC harness and is measured differently, by timing `GC('F')` directly (`d1-decision.md:61`), which is why it is absent from that table and from `bench-programs/README.md`'s "all seven".

| D9 dimension | program | status today |
|---|---|---|
| variable lookup | `varlookup.rex` | runs |
| compound/stem access | `compound.rex` | runs |
| string operations | `strings.rex` | runs |
| decimal arithmetic | `arith.rex` | runs |
| cold start | `startup.rex` | runs, **not comparable** (below) |
| method dispatch | `dispatch.rex` | `rc=120`, "a message send is not implemented (Phase 5)" |
| allocation throughput | `alloc.rex` | `rc=120`, same |
| full-GC pause | `heapshape.rex` | `rc=120`, same |

**Three D9 dimensions are blocked, not two.** An earlier draft said two, having missed that full-GC pause is D9-named and `heapshape.rex` is its harness -- an undercount written inside the paragraph warning against undercounts.

**Two of the three are 4d's to unblock, and one is not:**

* **`alloc` -- unblock, non-optional.** Allocation throughput needs no message send. Concatenation, compound-variable creation and numeric temporaries all allocate, and 4c-surface Rexx has all three. The ordering argument is decisive: 4d is the phase where representation changes land, and **landing them with the allocation axis unmeasured is the worst available ordering.**
* **`heapshape` -- unblock, because D1 requires it.** The C++ arm is `TIME('E')` around `GC('F')`; the Rust arm is a criterion bench at `rust/crates/rexx-core/benches/heap.rs`. That bench needs rebuilding rather than re-running: commit `a3178cff` replaced `Body::String(String)` -- the variant D1's risk analysis names -- with `Body::Text`, so today it measures a different representation than the recorded figure. **The recorded number is not reproducible as a like-for-like comparison, and 4d-1 rebuilds the comparison rather than quoting it.**
* **`dispatch` -- to Phase 5.** A dispatch benchmark that avoids message sends is not measuring method dispatch. A 4c-compatible variant would be a different benchmark wearing the same name.

**`startup` is not comparable at 4d and must not be recorded as passing.** This crate has no `CoreClasses.orx` bootstrap -- that is Phase 5 -- so it starts fast by doing none of the work the oracle's startup does. D2's actual target is absolute: parse and execute 5,203 lines of `CoreClasses.orx` plus `StreamClasses.orx` in under about 55 ms (`plan:157`). D2's decision is also already made and it is **(a), no saved image** -- "Ship (a) either way" (`plan:151`); the image is a conditional fallback, so the Phase 5 lever is parser and bootstrap throughput, not an image. 4d records `startup` as **not comparable yet**, names the 55 ms target, and gates nothing on it. The finer instrument already exists and needs no building: `rust/crates/rexx-bench/src/bin/rexx-time.rs`, 50 runs after 10 warmups, which produced the oracle's 5.119 ms median.

## Provisional sizing, and the reading that may overturn it

Single run per side, release build both sides, this machine, no spread, taken 2026-08-08 with other work running.
**Sizing figures for planning only.** 4d-1 supersedes them.

| axis | oracle | rust | ratio |
|---|---|---|---|
| `varlookup` | 1.22 s | 30.79 s | 25.2 |
| `strings` | 0.88 s | 12.94 s | 14.7 |
| `compound` | 1.13 s | 15.61 s | 13.8 |
| `rexxcps` | -- | -- | 10.02 (5 runs each side, 0.9% spread) |
| `arith` | 1.17 s | 4.48 s | 3.8 |

**The competing hypothesis, which 4d-1 must settle before 4d-2 is decomposed.**
Converting these to approximate absolute throughput using each program's own loop counts puts the **Rust side in a narrow band of roughly 0.9 to 1.9 million clauses per second, while the oracle spans roughly 3 to 47 million.**
On that reading the 3.8-to-25.2 spread is largely a property of the *denominator*, there is one dominant cause -- a fixed per-clause interpreter overhead that swamps what the clause does -- and a per-axis decomposition would produce several tasks attacking the same thing.

The clause counts behind that are estimates from reading the programs and are not load-bearing to the third digit; the shape is what matters. **4d-1 therefore reports absolute throughput on both sides, not only ratios.** A ratio hides its denominator, which is the same defect as a denominator that shrinks.

An earlier draft drew the opposite conclusion from `arith` at 3.8 -- that it was evidence for `rexx-num`. It does not follow: `arith` is the axis where the *oracle* does the most work per clause, so a flat Rust overhead predicts it as the best ratio regardless of how good the numeric core is. That inference is withdrawn.

## The two units

### 4d-1 -- measure, attribute, write the gate

Planned now, in full.

**Deliverables.**

1. **The measurement.** Every runnable axis plus `rexxcps`, both interpreters, interleaved, with **absolute throughput and ratio** per axis and the spread of each. `alloc` and `heapshape` made measurable as part of this. The committed axis list is a literal pinned against the corpus directory the way `corpus.rs`'s file list is, so an axis cannot silently drop out at 4d-2.
2. **The attribution.** Named causes, each a falsifiable claim carrying a number: what share of an axis's self time it accounts for, and the ratio predicted if it were removed. **A cause is the unit, not an axis** -- benchmark programs are not axis-pure (`varlookup.rex`'s inner loop is `x = x + 1`, which is 19 million decimal additions inside the axis named "variable lookup"), so causes are many-to-many with axes and each cause carries the set of axes it predicts it will move.
3. **`phase-4d-gate.md`**, carrying per-axis parity criteria, the CI-overlap decision rule, each bar's **derivation** and not only its value, and the oracle fingerprint.

**Stage 2 may prototype, and must publish and revert.** The falsification rule below requires having a fix in hand to confirm an attribution, which would otherwise hand stage 3 the very number it is supposed to write independently. So a prototype's measured win is published in the attribution document and the prototype is reverted; the bar is then visibly downstream of a number already on the record.

**Swap the global allocator early, as a diagnostic rather than as an optimisation.** Decided 2026-08-08.

A smoke profile of `arith` -- the axis that is *best* against the oracle -- puts roughly a third of self time in the glibc malloc family (`_GI___libc_malloc`, `int_malloc`, `int_free_chunk`, `_GI___libc_free`, `realloc`), with `Heap::alloc_with_uncollected` a further 5% on top.
One 4-second run of one axis, so the figure is indicative and 4d-1 re-measures it properly.

The value of an allocator swap here is that **it separates two hypotheses that the profile alone cannot.**
If a `#[global_allocator]` change recovers most of that third, the cost is allocator *quality* and the fix is adoption.
If it recovers little, the cost is allocation *count*, and the fix is not to call the allocator -- which is D1's side byte-arena, already nominated as the first thing to try.
Run it before deciding between them, because it is a few lines and it bounds the answer.

Two constraints on *adopting* it, as opposed to measuring with it:

* This is a runtime behaviour change, not a build setting, so unlike `lto` it does **not** go into the baseline. It is measured, published and reverted per the rule above, and 4d-2 decides adoption against the bar.
* Adoption is gated on the platforms the parity gate and CI already name. An allocator that does not build everywhere the interpreter ships is not a candidate, and that must be checked rather than assumed.

Note what the comparison is really telling us: the oracle does not call libc malloc per object at all -- it allocates from its own pools (`MemoryObject`, `DeadObjectPool`) -- so a per-value malloc is a difference in kind, and a faster malloc narrows it without removing it.

### 4d-2 -- optimise

**Planned only after 4d-1 closes**, from the attribution. It cannot be planned now: its tasks are one-cause-each and no cause exists yet, so any task written today would be written from the provisional table this spec has already disowned.

**Each task** names the cause it addresses, the axes it must move and by how much (from the attribution's prediction), and reports the measured move against that prediction. A task whose measured move matches nothing is a recorded finding about the attribution, not a silently-landed change.

**Stopping rule.** 4d-2 ends when every axis meets parity, or when a task's measured result contradicts the attribution -- whichever comes first. If 4d-1 names more causes than the plan sized for, 4d-2 re-plans rather than growing.

**Amendments.** Any change to a bar after 4d-1 closes carries both wordings and the reason, in the style `phase-4c-gate.md` used for its criterion 4, whose negative control was weakened after measurement and survived review only because the amendment was recorded.

## Measurement discipline

* **Pin the harness before measuring anything.** Name N, the statistic, and the tool, and adopt Global Constraints `:39`'s CI-overlap definition rather than inventing a third. Three statistics already exist in this tree -- `rexxcps` used 5 runs and `(max-min)/mean`, `perf-baseline.md` used criterion at `sample_size(10)` with a 95% interval, and the parity gate is defined on a criterion interval -- and stage 5 must be comparable to stage 1.
* **Pin `[profile.release]`.** `rust/Cargo.toml` has **no `[profile.release]` section at all**, so defaults apply: `lto = false`, `codegen-units = 16`, `debug = false`. Two consequences. `lto = "fat"` with `codegen-units = 1` would move every axis with no interpreter change, and is the first thing an optimiser reaches for, so whether it is in scope must be stated rather than left silent. And `debug = false` means **samply profiles a binary with no line information** -- 4d-1's named instrument does not work on the binary it measures. Decide whether profiling uses a separate profile, and whether measurement runs use the profiling one. Any profile change is its own task with its own before and after.
* **Fingerprint the oracle.** `phase-4-exclusions.txt` records that a rebuilt oracle "silently reprices every differential result in this project", that Task 6's sweep moved from 18 mismatches to 12 with no code change and no harness noticing, and that adding a fingerprint was **considered and declined** because rebuilds are rare and the user's own. That was reasonable for the corpus. It is not reasonable for a phase whose entire output is ratios against that binary, measured twice with all of 4d-2 in between. Record size, mtime and sha256 at 4d-1 and assert them at the gate, or re-run the oracle arm.
* **Interleave the runs.** Not all-oracle-then-all-rust. This is a 32-core AMD part and frequency drift across minutes exceeds several of the effects being chased.
* **State the timing window.** `perf-baseline.md:249` attributes 0.6 to 0.7 s of the Rust side's `rexxcps` wall time to process startup and parse that the internal timer does not see. A fixed offset of that size is about 15% of `arith`'s 4.48 s and about 2% of `varlookup`'s 30.79 s -- it inflates cheap axes more than expensive ones, which is the same shape as the spread being explained. **The degenerate path is explicit: gate on per-axis ratios that contain startup while declaring startup itself ungated, and every ratio improves by cutting process startup with nothing landing in the interpreter.** 4d-1 reports the fixed per-process offset as its own line so a change to it is visible rather than distributed.
* **Never report a single figure.** The 10.02 is credible because it arrived with a 0.9% per-side spread; the sizing table above is not, and says so.
* **A speedup claim is falsified like a bug fix.** Revert the change and show the number moves back. Without that it is a coincidence with a commit message attached.
* **Correctness is not a tradeoff.** No committed suite figure regresses, per `phase-4c-gate.md`'s recorded values. Stated as a property rather than as numbers copied into prose, because `rust/CLAUDE.md` identifies mutable repo aggregates in prose as the class that rots -- and because an optimisation that happens to fix a keyword body *improves* a count while turning the both-directions exempt-set assertion red, so a row that starts passing comes off its exempt file in the same commit.

## Scope

**In.** Anything the attribution names, including the heap and value representation, plus D1's and Phase 2's re-measurements and the side byte-arena as a named candidate.

**The guardrail on representation changes is a recording rule, not a restriction.** Whatever lands, the measurement that justified it is written into `d1-decision.md` itself -- a live document, untouched since 2026-07-28, that schedules its own Phase 4 re-measurement. A commit message is not that document.

**Out.**

* `Rc`/`Arc` and any refcounted value representation adopted for its own sake. This does **not** exclude the side byte-arena, which is a different mechanism and is explicitly in.
* The seven SIGABRT programs and the 512 MiB address-space reservation at `rexx-exec/src/lib.rs:309`. That reservation is a per-process startup cost and may become relevant to `startup`; if the attribution names it, it re-enters. It is *not* "D19's 512 MiB reservation" -- D19 decides a sized thread and deliberately records no number, and `perf-baseline.md:262` carries the same mis-citation.
* `dispatch`, to Phase 5.

**On the `Cow` copy.** This spec's own assumption, stated as such because it is not written down anywhere else: the deferred copy work was expected not to move `rexxcps`, on the grounds that `rexxcps` uses only short strings. That may still hold for `rexxcps` and is now much weaker as a general claim -- `strings` sits at 14.7. The mechanism, corrected: **every `.into_owned()` on a `to_text` result copies the whole byte string when the argument is already text**, and `required_string` (`builtin/mod.rs:761-766`) and `optional_string` (`:768-776`) both do it -- `datetime.rs` and `state.rs` use only the latter. It is not "every string builtin call regardless of size": `LENGTH` (`builtin/string.rs:94-104`) calls `.len()` on the `Cow` and never materialises it, and a `SmallInt` argument returns `Cow::Owned` (`value.rs:131`) so owning it is a move.

## Open questions for the plan

* **macOS.** The parity gate names Linux and macOS. Is a macOS measurement obtainable for 4d, and if not, is the gate met on Linux with macOS recorded as outstanding? This must be answered before 4d-1 writes the gate, not after.
* **The master plan contradicts this phase's existence.** `plan:473` says "The Phase 4 row closes when 4c closes", `plan:442`'s roadmap row uses "the ratio bar" without defining it, and 4d appears nowhere. 4d-1 amends all three.
* Does `varlookup` at 25 share a cause with `compound` at 13.8, or are they separate? Under the flat-overhead hypothesis the question dissolves; under the per-axis reading it drives the decomposition. 4d-1 answers it.
* **Was D9's compound-memoisation mandate met?** D9 `:389` says to build memoisation into the Rust stem design "from the start rather than porting the slow shape first and optimising later". `compound` is at 13.8. Whether that was done is 4d-1's first question for that axis.
* D9 `:390` also says to read the existing performance profile rather than re-derive it. **Both of D9's cited design inputs live outside both git trees**, in `/home/moritz/.claude/projects/-home-moritz-dev-repos-ooRexx/memory/`, and both describe the **C++** interpreter -- `CompoundVariableTable`'s BST, `MemoryObject::newObject`, `DeadObjectPool::findFit`. They constrain where to look; they are not a profile of this crate, and 4d-1's job is to profile this crate.
