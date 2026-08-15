# Review package: 4d-1 Task 8, the gate

Range e743e05e..cd408fa6

cd408fa6 Write the 4d gate: parity as the closing bar, the attribution as the prediction

 docs/superpowers/plans/phase-4d-gate.md | 488 ++++++++++++++++++++++++++++++++
 1 file changed, 488 insertions(+)

## Diff
diff --git a/docs/superpowers/plans/phase-4d-gate.md b/docs/superpowers/plans/phase-4d-gate.md
new file mode 100644
index 00000000..1a9f6b02
--- /dev/null
+++ b/docs/superpowers/plans/phase-4d-gate.md
@@ -0,0 +1,488 @@
+# Phase 4d exit gate -- criteria and assessment
+
+Phase 4d measures this crate against the C++ oracle on every axis that can be measured, attributes the gap to named causes carrying numbers, and closes when those axes reach parity.
+This document is the criteria and the current assessment against them.
+It is written at the end of 4d-1, before any optimisation, which is the whole reason it can be trusted: every bar below is downstream of a number already on the record rather than of a fix someone has in hand.
+
+**Nothing in this document optimises anything, and 4d-2 is not planned here.**
+4d-2 is planned from `phase-4d-attribution.md` once this unit closes, one task per cause, and this gate is what it is planned against.
+
+**It is downstream of three documents and adds no measurement of its own.**
+
+* `docs/superpowers/plans/perf-baseline.md`, the interleaved two-interpreter baseline -- the **current** section, measured 2026-08-08 at repo commit `d233d1e9`, not the superseded `107febcd` one.
+* `docs/superpowers/plans/phase-4d-attribution.md`, seven named causes, three prototypes published and reverted, measured 2026-08-09 at `c445154c`, with Task 7's allocator diagnostic at `9ce83f14`.
+* `docs/superpowers/plans/phase-4d-retention.md`, the unbounded-growth defect, measured 2026-08-08 at `c9a90906`.
+
+Two figures come from task reports rather than from those three, and each says so at its criterion: the full-GC-pause comparison (Task 5, measured 2026-08-09, committed at `c445154c`) and the allocator feasibility check (Task 7).
+
+**Every figure below names the date and the commit it was measured at.**
+A figure that names neither is not a figure, and this project has already shipped one baseline that silently described a different interpreter than the one in the tree.
+
+**The oracle build is identified by three hashes**, all matching across the baseline, the attribution and the retention document:
+
+| object | sha256 |
+|---|---|
+| `build/bin/rexx` | `bb5bb8ccbb96c376e329b91aafdad891f975ba06c941dbceba82c3848fa13019` |
+| `build/lib/librexx.so.4` | `42136c4038004fe2d5104181e06873301f032ced97c54a0fe84042e006d9b6fb` |
+| `build/lib/librexxapi.so.4` | `3536b76379c23fc7e3c4bce97b57d3c4812291ce5d68e71adc507ee690dc7d66` |
+
+That C++ tree also carries an uncommitted local patch, so these hashes are the only identity that build has.
+The crate side is `rust/target/release/rexx-run` sha256 `c3b2516069a1b5f5d504613986f00b69e0c0fc892c041958212d45a699c0e967`, byte-identical across all three source documents.
+
+---
+
+## The bar, and what it is not
+
+**The bar is parity, unamended.**
+Global Constraints `:39`:
+
+> **Shipping gate (parity).** No phase from 2 onward closes with a Rust subsystem slower than its C++ counterpart on the Phase 0 benchmark suite, measured on Linux and macOS.
+> "Slower" means the criterion point estimate falls outside the C++ baseline's confidence interval on the slow side.
+
+**It is not 1.5.**
+`:40` scopes the 1.5 threshold to Phase 1's viability check for a heap benchmarked without an interpreter, and says the parity gate applies from Phase 2 on.
+4a's R2 criterion reuses that 1.5 and is a Phase 1-shaped bar applied to a Phase 4 measurement; it is not this gate's bar.
+
+**If an axis cannot reach parity, the outcome is a recorded debt, not a lowered bar** -- named per axis, with its measurement, carried forward, in the shape D1 and Phase 2 already used.
+
+### Two bars per axis, and only one of them closes the phase
+
+Each axis below carries two numbers, and confusing them would make this document useless.
+
+* **The closing bar is parity.**
+  It is the same on every axis, it comes from `:39`, and it is the only thing that closes 4d.
+* **The derived bar is a prediction, and it is what 4d-2 is measured against.**
+  It is the ratio the attribution's named causes imply if they were removed: base ratio multiplied by one minus the sum of the shares of causes the attribution states do not overlap each other.
+  Meeting it does **not** close 4d.
+  Its job is to be falsifiable: a 4d-2 task that lands its named cause and does not move its axis toward its derived bar has contradicted the attribution, and the stopping rule below fires.
+
+**The derived bar is a prediction about a mechanism, not a target to hit by any route.**
+An axis that reaches its derived bar through some other change has not confirmed the attribution, and the task that did it reports that rather than claiming a confirmation.
+
+**Where an axis has no honest derivation, it gets no derived bar**, and the criterion says what it rests on instead.
+One axis is in that position and it is the worst one; see criterion 3.
+
+---
+
+## The gated measure excludes the fixed per-process offset, and the raw ratio must meet the bar too
+
+**This is a decision, made here, and it closes a named degenerate path.**
+The spec states it plainly: gate on per-axis ratios that contain process startup while declaring startup itself ungated, and every ratio improves by cutting process startup with nothing landing in the interpreter.
+
+**So the gated measure is each side's median wall time less that side's own measured fixed per-process offset** -- the `iters/s net` column the harness already emits, measured by timing `startup.rex` in the same run, 51 pairs, 5 discarded as warm-up.
+Cutting this crate's startup to zero leaves every net figure unchanged.
+
+**And the raw ratio must meet the bar as well, because netting introduces the opposite degenerate path.**
+The offset is a separately measured quantity that is subtracted; inflating it lowers every net figure.
+Requiring both readings means shrinking the offset can only help the raw one, inflating it can only help the net one, and neither alone can carry a criterion.
+Today the two readings differ by at most 0.093 ratio units, on `strings`, because the offset is small: computed from the current section's own offset line and medians, the oracle's 7.645 ms is at most **0.89%** of any axis (on `strings`, its shortest) and this crate's 1.797 ms is at most **0.077%** (on `alloc4c`, its shortest).
+
+| axis | raw ratio | net ratio | which binds |
+|---|---:|---:|---|
+| `alloc4c` | 2.08x | 2.09x | net |
+| `arith` | 2.70x | 2.72x | net |
+| `compound` | 6.08x | 6.12x | net |
+| `strings` | 10.61x | 10.70x | net |
+| `varlookup` | 4.35x | 4.38x | net |
+
+Net is the binding reading on all five today, because the oracle's offset is larger than this crate's and netting therefore shrinks the oracle's wall time by more.
+Both are reported at every measurement, and a run where the two disagree about a verdict is a finding about the offset, not a choice between readings.
+
+---
+
+## The instrument cannot decide inside 7.2%, and this gate says so rather than gating on one run
+
+**Between-run variance exceeds the interval the harness reports within a run, and that is measured rather than suspected.**
+
+* `compound` produced **disjoint** intervals across two quiet runs of a byte-identical binary: 6.35x, interval 6.32x-6.39x, and 6.08x, interval 6.02x-6.14x.
+  Both are nine-pair sign-test intervals nominally targeting 96.1% coverage.
+* A third run, the attribution's oracle-interleaved base, moved `alloc4c` **-7.2%** (2.08x to 1.93x), `compound` -2.8%, `arith` -2.2%, `strings` -0.7% and `varlookup` +0.2%, on the same byte-identical binary against the same three oracle objects.
+  So the instability is not a property of one axis, and re-measuring `compound` alone would not settle it.
+
+**This matters because `:39`'s verdict is decided on an interval.**
+The oracle's own intervals are roughly ±1% of its median.
+A between-run movement of 7.2% is seven times that window, so an axis sitting near parity would have its verdict decided by which run happened to be taken.
+**The gate's own criterion is therefore decidable by noise at the boundary, and no wording fixes that -- only a better instrument does.**
+
+**Two runs cannot establish a variance model and this gate does not invent one.**
+What it does instead is two things, both conservative, neither claiming a coverage property:
+
+1. **The closing measurement is three independent runs of the whole suite**, each a separate process invocation, separated in time, each with its own machine-quiet check, and an axis is MET only if the verdict holds in **all three**.
+   Three is the smallest count that makes a single lucky run insufficient; it is not a confidence statement and must not be quoted as one.
+2. **An axis whose point estimate lands within the largest between-run movement yet observed of the oracle's interval, on either side, is recorded UNDECIDED rather than MET or NOT MET.**
+   That figure is **7.2%** as of 2026-08-09, from `alloc4c`, and it is a **lower bound** on between-run variance drawn from five paired observations across two runs -- not an estimate of it.
+   At the closing measurement it is recomputed as the maximum absolute between-run movement on the record at that time, so it can only grow.
+
+**UNDECIDED is not a pass and does not close 4d.**
+It is a statement that this instrument cannot answer the question, and the axis stays open until something can.
+
+**The cost of this rule is stated rather than hidden: it makes a bare tie unprovable.**
+An axis that genuinely lands at 1.00x reads UNDECIDED, and only beating the oracle by more than the band reads MET.
+That is a strengthening of what this *instrument* can certify, not an amendment to `:39`'s bar, and the way out is a better instrument -- more pairs per run, or enough independent runs to justify a real variance estimate -- rather than a wider band.
+
+---
+
+## The criteria
+
+**Every criterion below was checked against one question before it was written: what degenerate execution satisfies this, and would deleting its subject leave it green?**
+This project has shipped criteria satisfied by shrinking their own denominator, and one whose stated falsification procedure was measured and did not falsify.
+Where a criterion cannot see something, that is said at the criterion rather than in a footnote.
+
+**Four protections are shared by criteria 1 to 7 and are stated once here rather than repeated.**
+
+* **An axis cannot silently disappear.**
+  `rexx-bench-suite.rs`'s `AXES` literal is asserted equal to the contents of `rust/bench-programs/` by `verify_axis_list`, which runs before any measurement and again as a test.
+  Deleting a benchmark program is red, and so is an axis in the list with no program.
+* **An axis cannot silently stop being measured while still being reported.**
+  An axis declared `Role::Blocked` that stops failing turns the suite red, so a Phase 5 dimension cannot start passing and keep appearing under "axes this crate cannot run", timed by nothing.
+* **The denominator cannot be shrunk by editing a loop bound.**
+  `loop_count` reads each program's own `n = <digits>` line out of the source rather than restating it, and both sides run the same file, so a smaller bound moves both sides equally and changes the workload visibly rather than the ratio invisibly.
+* **The oracle is re-measured at gate time, never reused, and is fingerprinted.**
+  The three sha256s above are asserted at the closing measurement or the oracle arm is re-run.
+  `phase-4-exclusions.txt` records a rebuilt oracle silently repricing every differential result in this project, with a sweep moving from 18 mismatches to 12 and no harness noticing.
+
+### 1. `varlookup` -- variable lookup, at parity
+
+> This crate's point estimate on `varlookup`, net of the fixed per-process offset and raw, falls inside or below the oracle's confidence interval, in three independent runs.
+
+**Derived bar: 2.80x**, from 4.35x multiplied by one minus C1's 32.7% and C7's 2.9%.
+C1 is variable binding resolved through hash maps; C7 is the per-clause binary search of the source line table.
+The attribution states that among C1 to C5 and C7 the only shared call site is C1's with C4, so these two shares do not overlap and may be summed.
+C6, the allocator, is excluded from every derived bar in this document because its own row says its share is not removable as stated.
+
+**Measured checkpoint: 4.36x to 3.92x** in one interleaved run, from prototype P2, which collects only C1's name-keyed half (20.2%) and pays an id-keyed lookup back on every assignment.
+The remaining 12.5% in the id-keyed map is bounded and unprototyped.
+
+**The derived bar does not reach parity and this gate says so rather than deriving a friendlier one.**
+With both named causes fully removed the axis is still 2.80x the oracle, so roughly 64% of what would remain is unattributed.
+`varlookup` is also the axis where the near-fitting per-clause constant fits worst (107 ns per clause against 405-585 for the others), which is consistent with its having the largest single named share in the attribution rather than with a hidden constant.
+
+*Vacuity:* cutting process startup is closed by the net reading; shrinking the axis is closed by `verify_axis_list` and the in-program loop bound; a slower oracle is closed by the fingerprint and gate-time re-measurement; a run that did not do the work is closed by the suite's byte-identical-stdout check within and across sides.
+
+### 2. `compound` -- compound and stem access, at parity
+
+> As criterion 1, for `compound`.
+
+**Derived bar: 1.81x to 2.58x**, and it is a band rather than a point because two of its causes overlap.
+C2 (`/`, `%`, `//`, `**` have no small-integer path) is 41.5% and C7 is 1.8%, both disjoint from everything else named.
+C1 is 14.2% and C4 (a compound access re-derives its tail at run time) is 12.7%, and C4's share *contains* part of C1's, so their union is between 14.2% and 26.9%.
+The band is 6.08x multiplied by one minus 57.5% at the conservative end and by one minus 70.2% at the other.
+
+**Measured checkpoint: 5.91x to 2.86x** in one interleaved run, from prototype P1 alone, which adds `IntDiv` and `Remainder` to the small-integer path.
+That already beats C2's own implied 3.56x, because P1 also removes the operand conversion, the result allocation and the root pushes attributed to their own frames.
+So the band is soft in both directions and is a prediction, not a measurement.
+
+**C4 is D9's own unmet mandate and is named here so 4d-2 does not rediscover it.**
+`plan:389` asks for memoisation built into the stem and compound-variable design from the start rather than porting the slow shape first.
+The slow shape was ported first: the split happens once at plan-build time and the result is discarded, and every access re-splits and re-resolves each tail through the name-keyed map.
+
+*Vacuity:* as criterion 1.
+Additionally, a `compound` result that improved because C2's fix landed is **not** evidence about stems: reading 6.08x as a statement about stem machinery would be wrong by about a factor of two, since `Number::div` alone is 41.5% of the axis and the stem machinery proper is 9.5%.
+
+### 3. `strings` -- string operations, at parity, and **this axis has no derived bar**
+
+> As criterion 1, for `strings`.
+
+**No derived bar, and inventing one would be the most damaging thing this document could do.**
+The named and removable share is C1's 7.6% plus C5's 9.9%, which is 17.5%, implying 8.75x at best.
+The only thing measured against that is the combined prototype's **2.8%**, itself marginal at five repetitions -- a factor of six short, with roughly five sixths of the axis unattributed.
+Everything else is C6 (37.7% self time in the allocator, whose removability the attribution does not establish) and the builtin bodies -- `changestr` 12.2%, `pos` 5.8%, `substr` 5.0% -- which are work the oracle performs too and which nobody has opened.
+
+**What the criterion rests on instead of a derivation** is two measured facts, neither of which is a share of a named removable cause:
+
+* The retention document's trigger-policy prototype measured **16% faster on `strings`**, interleaved at seven repetitions and reverted, which would imply 10.61x to 8.91x.
+  The same prototype moved nothing its data could distinguish from noise on `arith`, `compound` or `varlookup`, and measured nothing at all on `alloc4c`.
+* Task 7's allocator swap measured **-6.1%** on this axis and was reverted; it is a diagnostic, and adoption is not a candidate on quality grounds (see the 4d-2 rules below).
+
+**A 4d-2 task for `strings` cannot take its success criterion from this document.**
+Raising the attributed share above 17.5% is a prerequisite for that axis having a bar at all, and this gate records that as the axis's first piece of work rather than as a gap in a footnote.
+
+*Vacuity:* as criterion 1, plus the specific trap that "the named causes were fixed and the axis improved by their share" is satisfiable here while the axis remains at roughly nine times the oracle.
+
+### 4. `arith` -- decimal arithmetic, at parity
+
+> As criterion 1, for `arith`.
+
+**Derived bar: 2.16x**, from 2.70x multiplied by one minus C3's 14.5%, C1's 4.0% and C7's 1.6%.
+C3 is a number rendering and reparsing itself to classify itself.
+
+**Measured checkpoint: 2.64x to 2.15x** in one interleaved run, from the combined prototype.
+**The near-equality of 2.16x and 2.15x is not confirmation of the derivation, and reading it as such would be wrong.**
+The measured figure comes from a *smaller* set of causes -- C3's 14.5% plus C1's name-keyed 2.4%, 16.9% in total -- and exceeded that set's own share, while C7 was never touched and P1's small-integer path measured -0.7% here, inside noise.
+Two different sets of causes landing on the same number is a coincidence worth recording and not an agreement worth citing.
+
+**`arith`'s 28.7% in `Number::div` is not reachable by the small-integer `//` fix, and a 4d-2 task covering `arith` and `compound` together would half-miss.**
+`arith`'s divisions have non-integer operands, so a small-integer path cannot reach them; that is the prediction P1's -0.7% confirms rather than a disappointment.
+Nothing in the attribution separates irreducible decimal-division work from an implementation this crate could improve, and the C++ profile's own `b3_decarith` row puts 73.5% of the oracle's time in arithmetic on a decimal benchmark, so some of it is shared cost.
+
+**This is Phase 2's parity debt's scheduled re-measurement.**
+`d1-decision.md:76` recorded 1.22x and said in the same entry it was a lower bound that would get worse once the Rust side started paying for parsing, dispatch and variable lookup.
+It did, by a factor of about 2.2 as measured today.
+
+*Vacuity:* as criterion 1.
+
+### 5. `alloc4c` -- allocation throughput, at parity, and **2.08x is a debt rather than headroom**
+
+> As criterion 1, for `alloc4c`.
+
+**Derived bar: 1.70x**, from 2.08x multiplied by one minus C1's 7.0%, C5's 5.2% and C7's 6.0%.
+C4's 4.0% is excluded because it overlaps C1 on this axis.
+
+**THAT NUMBER MUST NOT BE READ AS "NEARLY AT PARITY ON ALLOCATION", AND IT WILL BE WRONG THE MOMENT A COLLECTOR LANDS.**
+Roughly **83%** of the oracle's time on this axis is its collector marking `tab.`'s growing tail table, which this crate never performs because this crate never collects.
+Measured two ways, on the same tree:
+
+* Profiled: `MemoryObject::newObject` 69.1% total, `MemoryObject::collect` 56.0% total, `CompoundTableElement::live` 20.7% self and the largest self-time function in the oracle's profile.
+* Flat-tail A/B, `tab.1 = i` in place of `tab.i = i` and nothing else changed, five interleaved repetitions per cell, all four cells printing `12888896`: **2.06x-2.17x growing against 8.50x-8.79x flat**.
+  Removing the growing table makes the oracle 5.85x faster and this crate 1.49x faster.
+
+So the two sides are not paying for the same thing, and the axis's honest position on the allocation dimension -- the comparison with the collector's work removed from the denominator -- is about **8.5x**, the same neighbourhood as `strings`.
+**The verdict is NOT MET on both readings**, which is why this finding changes what improving the axis means rather than what the table says.
+
+**A collector is a cost here, not a win, and the retention document's prototype never measured it.**
+That prototype covered `strings`, `arith`, `compound` and `varlookup`; `alloc4c` did not exist as an axis when it ran.
+This is the one axis where a collector has a large live set to re-mark and little to reclaim, so whoever lands a trigger policy in 4d-2 measures `alloc4c` before and after and expects the ratio to move the wrong way.
+
+**The dimension is partially covered, and the gate says which part.**
+`alloc.rex`, D9's own allocation program, still exits 120 on a message send and is Phase 5's; `alloc4c.rex` is its 4c-surface analogue and substitutes a compound-variable tail and a `||` concatenation for the array and the `.string~new`.
+Its own header states what it does and does not preserve.
+Parity on `alloc4c` is therefore parity on allocation throughput as a 4c-surface program measures it, and not on `alloc.rex`.
+
+*Vacuity:* as criterion 1, plus the specific trap this criterion exists to block -- a bar of 1.70x on this axis reads as near-parity and is measured against a denominator inflated by work this crate does not do.
+
+### 6. `heapshape` -- full-GC pause, at parity
+
+> The Rust collector's pause on the `heapshape` graph falls inside or below the oracle's measured pause, with both arms building the same object count and the residual shape difference stated.
+
+**No derived bar, because no share-based decomposition of the collect loop exists.**
+Nothing in this phase profiled `Heap::collect` by cause, so there is no share to remove and no ratio to imply.
+What exists instead is a named mechanism, measured directly: `size_of::<Body>()` grew from **32 bytes to 80** between D1's close and this measurement, dominated by `Body::Stem`'s `HashMap` payload, so every value including a plain string now carries a stem's footprint through the mark phase.
+`Heap::collect`'s algorithm is unchanged over that span, confirmed by reading every intervening commit that touches `heap.rs`.
+
+**Measured 2026-08-09 (Task 5, committed at `c445154c`): oracle median 17.966 ms over five runs, Rust 28.5-29.1 ms over two independent criterion runs at n=10, a ratio of 1.58x-1.66x.**
+The previously recorded 26.5 ms / 1.45x figure is **not reproducible**: `a3178cff` replaced `Body::String(String)` -- the exact variant D1's risk analysis names -- and the bench was mechanically updated to keep compiling and never re-run.
+So this axis also now misses the 1.5x Phase 1 debt threshold it was previously inside.
+
+**The two arms are not the same instrument and this criterion says so.**
+The oracle's arm is `TIME('E')` around `GC('F')` in `heapshape.rex`; the Rust arm is a criterion bench at `rust/crates/rexx-core/benches/heap.rs`.
+`rexx-core` has no `Directory` body variant, so the bench approximates the oracle's 1,000-key directory with a flat array holding 1,000 key strings: the same object count, not the same shape, and the residual difference cuts in the oracle's favour.
+It is documented rather than closed.
+
+*Vacuity:* the degenerate execution is a smaller graph, so the criterion requires both arms to build the same object count from `heapshape.rex`'s own construction, and the bench's graph shape is the subject of the comparison rather than a parameter of it.
+Deleting the Rust bench does not leave this green: the criterion has no other instrument and would read NOT MEASURED.
+
+### 7. `rexxcps` -- the oracle's own whole-program benchmark, reported with a verdict
+
+> The internal clauses-per-second ratio is reported at every measurement, with the spread of every run that produced it.
+
+**Not one of D9's eight dimensions.**
+D9 says whole-program benchmarks come from `samples/`, and this is that.
+It is carried because 4a's design spec named it and because it is the only figure in this phase that exercises the interpreter across many constructs at once.
+
+**No derived bar.**
+No cause in the attribution carries a `rexxcps` share, because the profiling was per axis.
+
+**It is the least stable figure in this gate and must not be quoted to three digits.**
+Four measurements of the same tree gave 7.41x (one pair, explicitly not a baseline), 6.21x (a contended run), 7.31x and 7.33x.
+Read 7.33x as the accepted figure for the current measurement and 6.21x-7.41x as the honest range across what has been observed.
+
+*Vacuity:* the benchmark self-calibrates, so the two sides do different amounts of work and their wall times are not comparable; only the per-clause figure is, and each side's `Averaged:` line is quoted at every measurement so the asymmetry is visible rather than inferred.
+
+### 8. `startup` -- cold start, **not comparable at 4d, and nothing is gated on it**
+
+> `startup` is recorded as not comparable, with both sides' figures and D2's absolute target, and no criterion in this document depends on it.
+
+**This crate has no `CoreClasses.orx` bootstrap, so it starts fast by doing none of the work the oracle does.**
+That is Phase 5's.
+Recording 1.797 ms against the oracle's 7.645 ms as a pass would be recording the absence of a feature as a performance result.
+
+**D2's target is absolute, not a ratio: parse and execute 5,203 lines of `CoreClasses.orx` plus `StreamClasses.orx` in under about 55 ms** (`plan:157`, from the oracle's 5.1 ms plus the 50 ms delta threshold).
+D2's decision is already made and is **(a), no saved image** -- "Ship (a) either way" (`plan:151`) -- so the Phase 5 lever is parser and bootstrap throughput, not an image.
+The ratio at that point will look terrible whatever happens, which is precisely why the ratio is not the gate.
+
+**The startup figure has one job in this document and it is not a criterion**: it is the fixed per-process offset that criteria 1 to 5 net out, and it is reported as its own line so a change to it is visible rather than distributed across every axis.
+
+*Vacuity:* this entry is the fix for a criterion that could not fail.
+An axis that starts fast by not existing satisfies any startup criterion, which is why there is none.
+
+### 9. `dispatch` -- method dispatch, **a D9 dimension this phase does not cover**
+
+> `dispatch` is recorded as not covered, owned by Phase 5, and no criterion in this document depends on it.
+
+`dispatch.rex` exits 120 with `rexx-exec: a message send is not implemented (Phase 5)`.
+**A dispatch benchmark that avoids message sends is a different benchmark wearing the same name**, so no 4c-surface analogue was built for it, unlike `alloc`.
+The suite reports it under "axes this crate cannot run" with the status and message it actually produced, and turns red if it stops failing while still declared blocked.
+
+### 10. Unbounded retention is closed, on a **conditional** loop and not only a counted one
+
+> Peak resident set on a loop whose live set is constant does not grow with the iteration count, measured at two counts a factor of eight apart, on **both** `do i = 1 to n` and `do n while zz`; and no benchmark axis aborts with an allocation failure under a stated address-space cap.
+
+**This is a defect, not a performance property, and a user sees it as an abort.**
+At `c9a90906` under the project's standard `ulimit -v 1048576`, `arith.rex` and `strings.rex` die with `memory allocation of 402653184 bytes failed` on stderr, exit 134, and nothing on stdout: no Rexx condition, so `SIGNAL ON SYNTAX` cannot catch it, no traceback, and any partial output lost.
+Raising the cap by exactly this crate's own 512 MiB reservation, to `ulimit -v 1572864`, leaves one axis dying rather than two.
+The ruling rests on the linearity rather than the axis count: growth on every shape measured is linear and unbounded, so any finite limit is reached by a long enough run.
+
+**Two independent causes, and a fix to one of them alone is worse than none.**
+
+* **Cause A, the arena is never collected.** `Heap::collect` has no allocation-count or heap-size trigger; its only production callers are a stress mode and the user-callable `GC('Force')`.
+* **Cause B, a loop header's temps are rooted until the loop ends**, at **two** push sites: `Interp::loop_advance` for a counted loop's control variable and `Interp::eval_condition` for every `WHILE`/`UNTIL` test.
+  This is a root leak a collector cannot reach: the entries are live by definition.
+
+**THE ACCEPTANCE TEST IS `do n while zz; nop; end`, NOT `varlookup.rex`, AND THAT IS THIS CRITERION'S ENTIRE POINT.**
+All four runnable loop axes are `do i = 1 to n`, so a fix closing `loop_advance` alone would take every axis green while leaving `DO WHILE` and `DO UNTIL` growing without bound -- the shape a long-running service loop is actually written in.
+A memory regression test built from the benchmark axes inherits that blind spot exactly, and the first version of the retention document reached the wrong conclusion from those four axes for the same reason.
+
+**What a fix is worth, measured by prototype and reverted:** peak RSS falls 42x on `arith`, 16x on `compound`, 298x on `strings` and 60x on `varlookup`, and `strings` gets **16% faster**.
+Those figures cover cause A and cause B's `loop_advance` site only; the `eval_condition` site was located and measured but not prototyped, so closing it is unquantified.
+
+*Vacuity:* the degenerate execution is measuring only counted loops, which is why the conditional shape is named in the criterion rather than in a note.
+A second degenerate execution is measuring at one iteration count, where a fixed cost is indistinguishable from a linear one, which is why two counts a factor of eight apart are required.
+Deleting the subject does not leave this green: with no retention instrument at all the criterion reads NOT MEASURED, and the defect is independently recorded in `phase-4-exclusions.txt` under KNOWN GAPS so it survives this document.
+
+### 11. Platform coverage -- **the gate is incomplete on this axis and is not silently Linux-only**
+
+> Every criterion above is recorded per platform, with Linux measured and every other platform in the matrix recorded as outstanding.
+
+**`:39` names Linux and macOS. `:35` names five: Linux, macOS 15 arm64, Windows/MSVC, FreeBSD 14.2 and OpenBSD 7.8, and says every phase gate runs on all five.**
+Nothing in this phase ran on any platform but this Linux machine.
+
+**Three specific things are unchecked rather than presumed fine.**
+
+* No baseline exists for macOS, Windows, FreeBSD or OpenBSD, for either interpreter.
+  `perf-baseline.md`'s "What is still missing" says the same and names the work: a CI job per platform that builds the C++ oracle, runs this suite, and either commits numbers or documents why a platform could not produce them.
+* **`rexx-bench-suite` is Linux-only as written.** The address-space cap is a `/bin/sh` builtin and the fingerprints come from `ldd`, `stat` and `sha256sum`.
+  A macOS measurement needs harness work before it needs a machine.
+* The known OpenBSD SIGSEGV in the C++ baseline is pre-existing and may mean that platform produces no baseline at all; that has not been checked from this machine.
+
+**So 4d cannot close on Linux alone**, and this criterion is what stops a Linux-only pass being read as a gate pass.
+Recording Linux as met with the rest outstanding is the honest state; recording the gate as met would not be.
+
+*Vacuity:* the degenerate execution is exactly the one this criterion exists to name -- measure one platform, report per-axis verdicts, and let the reader assume the matrix.
+
+### 12. The measurement's own integrity
+
+> The closing measurement is taken with the same harness, the same statistic and the same wrapper as this one; the oracle is fingerprinted and re-measured rather than reused; both sides alternate within each axis; both sides print byte-identical stdout on every axis; the fixed offset is reported as its own line; and the axis list is pinned against the corpus directory.
+
+**Alternating is not a detail.**
+Frequency drift across minutes on this 32-core part exceeds several of the effects being measured, and a block-per-side layout charges that drift to whichever side ran during it.
+Two separate suite runs on this machine have already invented a 5% effect that was not there, which is why every prototype figure in the attribution is a within-run comparison.
+
+**Both sides alternate and the harness observes the order the children actually ran in**, rather than asserting about its own loop, because nothing in the reported tables could distinguish an alternating run from a side-at-a-time one.
+
+**The address-space cap is stated and applied to both sides on every axis.**
+The current measurements use `ulimit -v 8388608`; the project's standard 1 GiB cap does not clear these workloads on this crate, and criterion 10 is why.
+
+*Vacuity:* a harness that measured nothing would satisfy "the same harness"; the byte-identical-stdout check across sides and within a side is what makes a wall time a statement about the workload, and the axis-list pin is what stops an axis leaving the report.
+
+### 13. Correctness and hygiene do not regress
+
+> `cargo test --workspace --no-fail-fast` from `rust/` is green with no committed suite figure regressed; `cargo clippy --workspace --all-targets -- -D warnings` is clean; `cargo fmt --all --check` from `rust/` is clean; `unsafe_code = "forbid"` stands.
+
+`cargo fmt --all --check`, **not** `cargo fmt --edition 2024 --check`: `cargo fmt` has no `--edition` flag and that spelling exits 2 before doing any work, which reads exactly like a check that passed.
+
+**Stated as a property rather than as numbers copied into prose**, because a count of a mutable in-repo aggregate written into a document rots.
+An optimisation that happens to fix a keyword body *improves* a count while turning a both-directions exempt-set assertion red, so a row that starts passing comes off its exempt file in the same commit.
+
+**Clippy is run from a freshly deleted target directory**, a precedent Tasks 5 and 7 set: a warm target directory can carry a stale result past a change that would have produced one.
+
+---
+
+## Assessment, 2026-08-09
+
+Figures are quoted from the documents named at the top, at the commits named there.
+Nothing in this section was measured by this task.
+
+| # | criterion | result |
+|---|---|---|
+| 1 | `varlookup` at parity | **NOT MET** -- 4.35x raw, 4.38x net; derived bar 2.80x |
+| 2 | `compound` at parity | **NOT MET** -- 6.08x raw, 6.12x net; derived bar 1.81x-2.58x |
+| 3 | `strings` at parity | **NOT MET** -- 10.61x raw, 10.70x net; **no derived bar** |
+| 4 | `arith` at parity | **NOT MET** -- 2.70x raw, 2.72x net; derived bar 2.16x |
+| 5 | `alloc4c` at parity | **NOT MET** -- 2.08x raw, 2.09x net; derived bar 1.70x, **and 2.08x is a debt, not headroom** |
+| 6 | `heapshape` full-GC pause at parity | **NOT MET** -- 1.58x-1.66x; no derived bar |
+| 7 | `rexxcps` reported | **REPORTED, NOT MET** -- 7.33x, observed range 6.21x-7.41x |
+| 8 | `startup` | **NOT COMPARABLE** -- D2's absolute target is about 55 ms; nothing is gated on it |
+| 9 | `dispatch` | **NOT COVERED** -- Phase 5 |
+| 10 | unbounded retention closed | **NOT MET** -- linear unbounded growth from two causes; two axes abort under the standard cap |
+| 11 | platform coverage | **INCOMPLETE** -- Linux measured, macOS/Windows/FreeBSD/OpenBSD outstanding |
+| 12 | measurement integrity | **MET** for the current measurement; re-asserted at the closing one |
+| 13 | correctness and hygiene | **MET** |
+
+**Phase 4d does not close today.**
+Six axes miss parity, one has no derived bar at all, a correctness defect is open, and the platform matrix is one of five.
+That is the state 4d-2 starts from, and it is stated here so that no later document has to reconstruct it.
+
+### Where the derived bars leave parity
+
+**Not one derived bar reaches 1.00x.**
+
+| axis | today | derived bar | still short by |
+|---|---:|---:|---:|
+| `varlookup` | 4.35x | 2.80x | 2.80x |
+| `compound` | 6.08x | 1.81x-2.58x | 1.81x-2.58x |
+| `strings` | 10.61x | none | unquantified |
+| `arith` | 2.70x | 2.16x | 2.16x |
+| `alloc4c` | 2.08x | 1.70x | 1.70x, against an inflated denominator |
+| `heapshape` | 1.58x-1.66x | none | unquantified |
+
+**So the attribution as it stands does not predict parity on any axis**, and that is the single most important thing this gate records.
+4d-2 planned solely from the seven named causes would run out of causes before it ran out of gap.
+Either more attribution is needed before the plan is written, or the debt shape applies and the shortfall is carried forward named and measured; this gate does not choose between those, because choosing is planning 4d-2.
+
+---
+
+## 4d-2's stopping rule and amendment rule
+
+**Stopping rule.**
+4d-2 ends when every axis meets its bar, or when a task's measured result contradicts the attribution -- whichever comes first.
+If 4d-1 names more causes than the plan sized for, 4d-2 re-plans rather than growing.
+
+**What "contradicts the attribution" means concretely**, so the rule is applicable rather than decorative: a task names the cause it addresses and the axes it must move and by how much, from the derived bar above, and reports the measured move against that prediction.
+A task whose measured move matches nothing is a recorded finding about the attribution, not a silently-landed change.
+A task that reaches its axis's derived bar by some other route has not confirmed the attribution and says so.
+
+**Amendment rule.**
+**Any later change to a bar carries both wordings and the reason.**
+`phase-4c-gate.md`'s criterion 4 is the model: its negative control was weakened after a measurement showed the plan's version unsatisfiable, and it survived review **only** because the amendment was recorded with the original wording beside it.
+A criterion that moves after the measurement and does not say so has stopped being a criterion.
+
+**Three rules that constrain 4d-2 and are not criteria.**
+
+* **No allocator adoption is a candidate on quality grounds.**
+  Task 7 measured the fork the spec pre-registered and it fell on the count side: mimalloc moved `alloc4c` -18.4%, `compound` -6.6%, `arith` -6.5%, `strings` -6.1% and `varlookup` **+1.2%**, never recovering more than 46.6% of an axis's own allocator self-time share and losing on the axis with the least allocator involvement.
+  Re-profiling the strongest case showed mimalloc roughly halving its own per-call share and still buying only 18.4% of wall, because the surrounding hashing and copying does not shrink when the allocator changes.
+  D1's pre-registered side byte-arena is the un-replaced candidate.
+  Any allocator proposal must additionally clear the platform matrix: the swap was built on Linux only, and macOS, Windows, FreeBSD and OpenBSD are explicitly unchecked.
+* **A collector is a win on `strings` and a cost on `alloc4c`, and only the first half is measured.**
+  A trigger policy lands with `alloc4c` measured before and after, expecting the ratio to move the wrong way.
+* **The retention fix covers both push sites or it is not a fix**, per criterion 10.
+
+---
+
+## What this gate found
+
+* **The gate has no bar that reaches parity on any axis.**
+  Every derived bar above is a prediction of the ratio the named causes can buy, and the best of them is 1.70x on the axis whose denominator is inflated.
+* **`alloc4c`'s 2.08x nearly became a bar meaning its opposite.**
+  Roughly 83% of the oracle's time on that axis is collector work over a growing tail table that this crate never performs; with the growing table removed the axis reads about 8.5x.
+  A gate reading 2.08x as "nearly at parity on allocation" would have been wrong the moment a collector landed, and would have been read by 4d-2 as a reason to look elsewhere.
+* **`strings`, the worst axis, cannot carry a derived bar**, and saying so is more useful than deriving one anyway: 17.5% named against a 2.8% measured combination win is a factor of six, with roughly five sixths unattributed.
+* **`arith`'s largest single share is not reachable by the fix that transformed `compound`.**
+  `Number::div` is 41.5% of `compound` and 28.7% of `arith`, and the same prototype moved the first by -51.6% and the second by -0.7%, because `arith`'s divisions have non-integer operands.
+  One 4d-2 task for both axes would half-miss.
+* **The cost is allocation count, not allocator quality**, measured rather than argued, with the `varlookup` loss as the internal consistency check that a quality story cannot produce.
+* **The gate's own decision rule is decidable by noise near the boundary**, and the evidence is three runs of a byte-identical binary: `compound`'s disjoint intervals and `alloc4c`'s -7.2%.
+  The response is three independent runs and a declared undecidable band, not a variance model from two runs.
+* **`heapshape`'s recorded Phase 1 figure was not reproducible**, and rebuilding the comparison moved it from 1.45x to 1.58x-1.66x -- out of the 1.5x band it was previously inside -- for a reason unconnected to the string-representation risk D1 named: `size_of::<Body>()` grew from 32 bytes to 80, dominated by a cold `Stem` variant that every value now carries.
+* **This unit could not produce a pre-optimisation baseline, only a current one.**
+  Five speedups landed after Task 2's baseline and before Task 2b's, which contradicts this phase's own no-optimisation rule.
+  It is recorded here as it is in `perf-baseline.md` and `task-2b-brief.md`: the bar-before-optimisation property is partly spent, and no document should claim otherwise.
+
+## What 4d inherits forward
+
+* **`dispatch` to Phase 5**, as a D9 dimension nothing in 4d measures.
+* **`alloc.rex` to Phase 5**, with `alloc4c.rex` covering the allocation dimension on the 4c surface only.
+* **`startup` to Phase 5**, against D2's absolute target of about 55 ms for 5,203 lines and D2's already-made decision of (a), no saved image.
+* **The platform matrix**, which needs harness work before it needs machines: `rexx-bench-suite` is Linux-only as written.
+* **The unbounded-growth defect**, recorded independently in `phase-4-exclusions.txt` under KNOWN GAPS so that it survives whatever happens to this document.
+* **A better instrument, or the honest admission that a tie cannot be certified**, per the undecidable band above.
