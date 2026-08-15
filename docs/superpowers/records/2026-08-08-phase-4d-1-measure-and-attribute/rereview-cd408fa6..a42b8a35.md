# Scoped re-review: 4d-1 Task 8, fix round 1

Range cd408fa6..a42b8a35

 docs/superpowers/plans/phase-4d-gate.md | 220 +++++++++++++++++++++++++-------
 1 file changed, 172 insertions(+), 48 deletions(-)

## Diff
diff --git a/docs/superpowers/plans/phase-4d-gate.md b/docs/superpowers/plans/phase-4d-gate.md
index 1a9f6b02..d31a4c6a 100644
--- a/docs/superpowers/plans/phase-4d-gate.md
+++ b/docs/superpowers/plans/phase-4d-gate.md
@@ -1,67 +1,110 @@
 # Phase 4d exit gate -- criteria and assessment
 
 Phase 4d measures this crate against the C++ oracle on every axis that can be measured, attributes the gap to named causes carrying numbers, and closes when those axes reach parity.
 This document is the criteria and the current assessment against them.
 It is written at the end of 4d-1, before any optimisation, which is the whole reason it can be trusted: every bar below is downstream of a number already on the record rather than of a fix someone has in hand.
 
 **Nothing in this document optimises anything, and 4d-2 is not planned here.**
 4d-2 is planned from `phase-4d-attribution.md` once this unit closes, one task per cause, and this gate is what it is planned against.
 
-**It is downstream of three documents and adds no measurement of its own.**
+**It is downstream of four committed documents and adds no measurement of its own.**
 
 * `docs/superpowers/plans/perf-baseline.md`, the interleaved two-interpreter baseline -- the **current** section, measured 2026-08-08 at repo commit `d233d1e9`, not the superseded `107febcd` one.
-* `docs/superpowers/plans/phase-4d-attribution.md`, seven named causes, three prototypes published and reverted, measured 2026-08-09 at `c445154c`, with Task 7's allocator diagnostic at `9ce83f14`.
+* `docs/superpowers/plans/phase-4d-attribution.md`, seven named causes, three prototypes published and reverted, measured 2026-08-09 at `c445154c`, with the allocator diagnostic and its feasibility check in that document's own Task 7 section at `9ce83f14`.
 * `docs/superpowers/plans/phase-4d-retention.md`, the unbounded-growth defect, measured 2026-08-08 at `c9a90906`.
+* `docs/superpowers/plans/d1-decision.md`, whose Phase 4 addendum (`:113` onward, 2026-08-09, from `ad7c36f0`) carries the rebuilt full-GC-pause comparison, and whose `:76` carries Phase 2's `arith` parity debt.
 
-Two figures come from task reports rather than from those three, and each says so at its criterion: the full-GC-pause comparison (Task 5, measured 2026-08-09, committed at `c445154c`) and the allocator feasibility check (Task 7).
-
-**Every figure below names the date and the commit it was measured at.**
+**Every figure below names the date and the commit it was measured at, and is quoted from one of those four.**
 A figure that names neither is not a figure, and this project has already shipped one baseline that silently described a different interpreter than the one in the tree.
+Nothing here is sourced to a task report, which is gitignored, or to a scratchpad probe, which is not on the record at all.
 
 **The oracle build is identified by three hashes**, all matching across the baseline, the attribution and the retention document:
 
 | object | sha256 |
 |---|---|
 | `build/bin/rexx` | `bb5bb8ccbb96c376e329b91aafdad891f975ba06c941dbceba82c3848fa13019` |
 | `build/lib/librexx.so.4` | `42136c4038004fe2d5104181e06873301f032ced97c54a0fe84042e006d9b6fb` |
 | `build/lib/librexxapi.so.4` | `3536b76379c23fc7e3c4bce97b57d3c4812291ce5d68e71adc507ee690dc7d66` |
 
 That C++ tree also carries an uncommitted local patch, so these hashes are the only identity that build has.
-The crate side is `rust/target/release/rexx-run` sha256 `c3b2516069a1b5f5d504613986f00b69e0c0fc892c041958212d45a699c0e967`, byte-identical across all three source documents.
+The crate side is `rust/target/release/rexx-run` sha256 `c3b2516069a1b5f5d504613986f00b69e0c0fc892c041958212d45a699c0e967`, byte-identical across the baseline, the attribution and the retention document.
+
+---
+
+## The result, before the criteria: no bar this gate can derive reaches parity on any axis
+
+**Phase 4d does not close today, and the attribution it is built on does not predict that it will.**
+Six axes miss parity, a correctness defect is open, and the platform matrix is one of five.
+
+**The stronger statement is the one that matters to whoever plans 4d-2.**
+Every axis's predicted best ratio, derived below from the named causes' own measured shares, is between 1.62x and 2.80x -- and two axes have no prediction at all, because nothing profiled them by cause.
+**So landing all seven named causes, perfectly, on every axis, does not reach parity anywhere.**
+4d-2 planned solely from those seven runs out of causes before it runs out of gap.
+
+That is not an argument for a lower bar.
+It is the reason the debt mechanism below is spelled out rather than left as an intention, and the reason the stopping rule below says what happens when the causes are exhausted and the phase is still open.
 
 ---
 
 ## The bar, and what it is not
 
 **The bar is parity, unamended.**
 Global Constraints `:39`:
 
 > **Shipping gate (parity).** No phase from 2 onward closes with a Rust subsystem slower than its C++ counterpart on the Phase 0 benchmark suite, measured on Linux and macOS.
 > "Slower" means the criterion point estimate falls outside the C++ baseline's confidence interval on the slow side.
 
 **It is not 1.5.**
 `:40` scopes the 1.5 threshold to Phase 1's viability check for a heap benchmarked without an interpreter, and says the parity gate applies from Phase 2 on.
 4a's R2 criterion reuses that 1.5 and is a Phase 1-shaped bar applied to a Phase 4 measurement; it is not this gate's bar.
 
-**If an axis cannot reach parity, the outcome is a recorded debt, not a lowered bar** -- named per axis, with its measurement, carried forward, in the shape D1 and Phase 2 already used.
+### A debt is an amendment, and this section supplies the mechanism the intent lacks
+
+**The phase spec states the intent** (`2026-08-08-phase-4d-performance-design.md:24`): if 4d cannot reach parity, the outcome is a recorded debt in the shape D1 and Phase 2 already used -- named per axis, with its measurement, carried forward -- and it is not a lowered bar.
+**That is a statement of intent and a gate needs a mechanism**, because as a bare sentence it is a route to close 4d without parity on any axis: take three runs, record six NOT METs as debts, close, and nothing in the interpreter has moved.
+This gate's own headline finding guarantees that is the route every axis would otherwise take, so the sentence is kept -- the decision was made above this document -- and the three things it does not say are supplied here.
+
+**Who grants a debt: the user, at plan level, never a task and never this document's author.**
+The precedent is Global Constraints `:36`, which makes adding an entry to a known-failure file "a plan-level decision, never a task-level one", for the same reason: a bar that the work being measured may relax is not a bar.
+
+**Whether a debt closes the phase: only through an amendment, which carries both wordings and the reason.**
+Recording a debt in place of parity *is* a change to what closes the axis, so it goes through the amendment rule below rather than around it, with the parity wording and the debt wording side by side and the measurement that forced the change.
+A debt recorded without that is not a debt, it is an undocumented amendment, and this gate does not recognise it.
+
+**On what evidence, and this is the part that stops the degenerate route.**
+A debt for an axis requires all four of:
+
+1. The axis measured NOT MET at the closing measurement under the three-run rule below -- not UNDECIDED, which is not a result.
+2. **A named cause for the residual, carrying a measured share.**
+   An axis whose remaining gap is unattributed cannot carry a debt, because "we do not know why it is slow" is not a debt, it is unfinished measurement.
+   As of today this rules out `strings`, whose residual is five sixths unattributed (criterion 3), and `heapshape`, which nothing profiled by cause (criterion 6).
+3. What would close it, and which phase owns that work.
+4. A named re-measurement point.
+   Phase 2's `arith` debt is the working example: `d1-decision.md:76` recorded 1.22x, said in the same entry it was a lower bound, and named Phase 4 as its re-measurement -- which is criterion 4 below, and which found it had worsened by a factor of about 2.2 exactly as predicted.
+
+So six debts require six attributed residuals, six statements of what would close them, and one amendment per axis granted by the user.
+That is a far higher price than three runs and a table, which is the point.
 
 ### Two bars per axis, and only one of them closes the phase
 
 Each axis below carries two numbers, and confusing them would make this document useless.
 
 * **The closing bar is parity.**
   It is the same on every axis, it comes from `:39`, and it is the only thing that closes 4d.
 * **The derived bar is a prediction, and it is what 4d-2 is measured against.**
-  It is the ratio the attribution's named causes imply if they were removed: base ratio multiplied by one minus the sum of the shares of causes the attribution states do not overlap each other.
+  It is the ratio the attribution's named causes imply if they were removed: base ratio multiplied by one minus the **removable share**.
+  The removable share is a sum where the attribution states the causes are disjoint, and a **band** wherever two named causes overlap, since the union of an overlapping pair is bounded below by the larger share and above by their sum.
+  Two of the four derived bars below are bands for that reason, and in both the **conservative end is the one to plan against**: the optimistic end assumes an overlap of zero, which the attribution's own wording denies.
+  C6, the allocator, is excluded from every removable share in this document, because its own row says its share is not removable as stated.
   Meeting it does **not** close 4d.
   Its job is to be falsifiable: a 4d-2 task that lands its named cause and does not move its axis toward its derived bar has contradicted the attribution, and the stopping rule below fires.
 
 **The derived bar is a prediction about a mechanism, not a target to hit by any route.**
 An axis that reaches its derived bar through some other change has not confirmed the attribution, and the task that did it reports that rather than claiming a confirmation.
 
 **Where an axis has no honest derivation, it gets no derived bar**, and the criterion says what it rests on instead.
 One axis is in that position and it is the worst one; see criterion 3.
 
 ---
 
 ## The gated measure excludes the fixed per-process offset, and the raw ratio must meet the bar too
@@ -79,65 +122,106 @@ Today the two readings differ by at most 0.093 ratio units, on `strings`, becaus
 
 | axis | raw ratio | net ratio | which binds |
 |---|---:|---:|---|
 | `alloc4c` | 2.08x | 2.09x | net |
 | `arith` | 2.70x | 2.72x | net |
 | `compound` | 6.08x | 6.12x | net |
 | `strings` | 10.61x | 10.70x | net |
 | `varlookup` | 4.35x | 4.38x | net |
 
 Net is the binding reading on all five today, because the oracle's offset is larger than this crate's and netting therefore shrinks the oracle's wall time by more.
 Both are reported at every measurement, and a run where the two disagree about a verdict is a finding about the offset, not a choice between readings.
 
+**Under the net reading the oracle's interval is netted too, by the same subtraction.**
+Comparing a netted point estimate against a raw interval would be a category error, and the closing measurement must not have to guess which was meant.
+The offset is a constant shift, so the netted interval is simply both endpoints less that side's own median offset: `[lo - off, hi - off]`.
+Concretely, for `alloc4c` the oracle's raw interval is 1.1205-1.1408 s about a 1.1279 s median, and netting all three by the oracle's 7.645 ms offset gives 1.1129-1.1332 s about 1.1203 s.
+
 ---
 
 ## The instrument cannot decide inside 7.2%, and this gate says so rather than gating on one run
 
 **Between-run variance exceeds the interval the harness reports within a run, and that is measured rather than suspected.**
 
 * `compound` produced **disjoint** intervals across two quiet runs of a byte-identical binary: 6.35x, interval 6.32x-6.39x, and 6.08x, interval 6.02x-6.14x.
   Both are nine-pair sign-test intervals nominally targeting 96.1% coverage.
 * A third run, the attribution's oracle-interleaved base, moved `alloc4c` **-7.2%** (2.08x to 1.93x), `compound` -2.8%, `arith` -2.2%, `strings` -0.7% and `varlookup` +0.2%, on the same byte-identical binary against the same three oracle objects.
   So the instability is not a property of one axis, and re-measuring `compound` alone would not settle it.
+  **That third run's movement is not purely between-run, and the gate says so rather than letting its own most-quoted number be read as something its source denies.**
+  The attribution states that this run used a different wall-clock harness -- a bash subshell with the `ulimit` builtin timed by `date +%s.%N`, against the suite's `/bin/sh` wrapper timed by `Instant` -- and that this is part of why its base medians land between 7.2% under and 0.2% over the committed section's.
+  So 7.2% is a between-run **and** between-harness movement.
+  The direction is conservative, since it inflates a band that can only withhold a pass, so it opens no hole; it is corrected because later work will quote this document rather than the attribution.
+* **Three byte-identical runs are on the record, not two.**
+  The nine-pair run that the current baseline section replaced (`compound` 6.35x), the current nine-pair run, and the attribution's interleaved run, all of the same `rexx-run` sha256 against the same three oracle objects.
+  `alloc4c`'s 7.2% is the largest absolute movement across all three.
 
 **This matters because `:39`'s verdict is decided on an interval.**
 The oracle's own intervals are roughly ±1% of its median.
 A between-run movement of 7.2% is seven times that window, so an axis sitting near parity would have its verdict decided by which run happened to be taken.
 **The gate's own criterion is therefore decidable by noise at the boundary, and no wording fixes that -- only a better instrument does.**
 
-**Two runs cannot establish a variance model and this gate does not invent one.**
+**Three runs cannot establish a variance model and this gate does not invent one.**
 What it does instead is two things, both conservative, neither claiming a coverage property:
 
-1. **The closing measurement is three independent runs of the whole suite**, each a separate process invocation, separated in time, each with its own machine-quiet check, and an axis is MET only if the verdict holds in **all three**.
+1. **The closing measurement is three independent runs of the whole suite**, and an axis is MET only if the verdict holds in **all three**.
    Three is the smallest count that makes a single lucky run insufficient; it is not a confidence statement and must not be quoted as one.
-2. **An axis whose point estimate lands within the largest between-run movement yet observed of the oracle's interval, on either side, is recorded UNDECIDED rather than MET or NOT MET.**
-   That figure is **7.2%** as of 2026-08-09, from `alloc4c`, and it is a **lower bound** on between-run variance drawn from five paired observations across two runs -- not an estimate of it.
-   At the closing measurement it is recomputed as the maximum absolute between-run movement on the record at that time, so it can only grow.
+2. **An axis whose point estimate lands within the band defined below of its own parity threshold is recorded UNDECIDED rather than MET or NOT MET.**
+
+**"Independent" is defined, because three back-to-back runs satisfy the letter and defeat the purpose.**
+A run counts only if all three of these hold and are recorded in its report:
+
+* **No prior run's residency is still held.**
+  This is measured, not hypothetical: the retention document records a non-interleaved pass reading 18.94 s for a prototype against 9.20 s for the base, which interleaved reads 7.79 s against 9.24 s -- the opposite sign -- because it started while a previous run's 3.7 GB resident set was still resident.
+  So each run starts with no `rexx-run` or `rexx` process alive and no prior run's resident set held.
+* **Nothing else CPU-bound is running**, with the load average recorded before and after each run and quoted in the report, the way Task 4 recorded 0.86-4.25 on 32 cores for the run that produced the current baseline.
+* **The three runs are not one sitting.**
+  Each run's start timestamp is recorded, and at least one full run's duration -- about twelve minutes -- separates consecutive runs, so a single episode of thermal or frequency drift cannot span all three.
+
+**The band is a percentage of the ratio, and it is applied to the parity threshold rather than to the interval's width.**
+The movements it is drawn from are percentage changes in the *ratio*, so that is the scale it is expressed on.
+Write `T` for the axis's **parity threshold ratio**: the oracle's interval upper bound divided by the oracle's median, both on the reading being judged.
+`T` is what `:39`'s verdict reduces to once both sides are expressed as a ratio, and it is a little above 1.00 -- for `alloc4c` on the raw reading it is 1.1408 / 1.1279 = **1.011**.
+With `b` the band and `R` this crate's ratio point estimate:
+
+* `R < T x (1 - b)` reads **MET**.
+* `R > T x (1 + b)` reads **NOT MET**.
+* Anything between reads **UNDECIDED**.
+
+**`b` is 7.2% as of 2026-08-09**, from `alloc4c`, and it is a **lower bound** on between-run movement drawn from three byte-identical runs -- not an estimate of variance.
+At the closing measurement it is recomputed as the maximum absolute between-run movement on the record at that time, **including the three closing runs' own movements against each other**, so the band is computed after those runs and applied to them.
+It can only grow, which is why letting the closing runs feed it cannot manufacture a pass.
+
+**Both readings and all three runs, and the combination rule is total so nothing falls through it.**
+The rule is applied to the raw reading and the net reading separately, in each of the three runs.
+An axis is **MET** only if every one of those six evaluations reads MET; **NOT MET** only if every one reads NOT MET; **UNDECIDED** otherwise.
+Runs that disagree with each other are themselves evidence the instrument cannot decide, which is exactly what UNDECIDED means.
 
 **UNDECIDED is not a pass and does not close 4d.**
 It is a statement that this instrument cannot answer the question, and the axis stays open until something can.
+It is also not a debt: the debt mechanism above requires NOT MET at the closing measurement, precisely so that an axis cannot be carried forward on a verdict the instrument declined to give.
 
 **The cost of this rule is stated rather than hidden: it makes a bare tie unprovable.**
-An axis that genuinely lands at 1.00x reads UNDECIDED, and only beating the oracle by more than the band reads MET.
+An axis that genuinely lands at 1.00x reads UNDECIDED, and MET needs `R < T x (1 - b)`, which on `alloc4c`'s current numbers is 0.939 -- about 6% faster than the oracle rather than level with it.
 That is a strengthening of what this *instrument* can certify, not an amendment to `:39`'s bar, and the way out is a better instrument -- more pairs per run, or enough independent runs to justify a real variance estimate -- rather than a wider band.
 
 ---
 
 ## The criteria
 
 **Every criterion below was checked against one question before it was written: what degenerate execution satisfies this, and would deleting its subject leave it green?**
 This project has shipped criteria satisfied by shrinking their own denominator, and one whose stated falsification procedure was measured and did not falsify.
 Where a criterion cannot see something, that is said at the criterion rather than in a footnote.
 
-**Four protections are shared by criteria 1 to 7 and are stated once here rather than repeated.**
+**Four protections are shared by criteria 1 to 5 and are stated once here rather than repeated.**
+**They do not extend to criteria 6 and 7**, whose numbers come from instruments outside the suite's axis machinery, and each of those two says at its own criterion what protects it and what does not.
 
 * **An axis cannot silently disappear.**
   `rexx-bench-suite.rs`'s `AXES` literal is asserted equal to the contents of `rust/bench-programs/` by `verify_axis_list`, which runs before any measurement and again as a test.
   Deleting a benchmark program is red, and so is an axis in the list with no program.
 * **An axis cannot silently stop being measured while still being reported.**
   An axis declared `Role::Blocked` that stops failing turns the suite red, so a Phase 5 dimension cannot start passing and keep appearing under "axes this crate cannot run", timed by nothing.
 * **The denominator cannot be shrunk by editing a loop bound.**
   `loop_count` reads each program's own `n = <digits>` line out of the source rather than restating it, and both sides run the same file, so a smaller bound moves both sides equally and changes the workload visibly rather than the ratio invisibly.
 * **The oracle is re-measured at gate time, never reused, and is fingerprinted.**
   The three sha256s above are asserted at the closing measurement or the oracle arm is re-run.
   `phase-4-exclusions.txt` records a rebuilt oracle silently repricing every differential result in this project, with a sweep moving from 18 mismatches to 12 and no harness noticing.
 
@@ -158,24 +242,26 @@ With both named causes fully removed the axis is still 2.80x the oracle, so roug
 `varlookup` is also the axis where the near-fitting per-clause constant fits worst (107 ns per clause against 405-585 for the others), which is consistent with its having the largest single named share in the attribution rather than with a hidden constant.
 
 *Vacuity:* cutting process startup is closed by the net reading; shrinking the axis is closed by `verify_axis_list` and the in-program loop bound; a slower oracle is closed by the fingerprint and gate-time re-measurement; a run that did not do the work is closed by the suite's byte-identical-stdout check within and across sides.
 
 ### 2. `compound` -- compound and stem access, at parity
 
 > As criterion 1, for `compound`.
 
 **Derived bar: 1.81x to 2.58x**, and it is a band rather than a point because two of its causes overlap.
 C2 (`/`, `%`, `//`, `**` have no small-integer path) is 41.5% and C7 is 1.8%, both disjoint from everything else named.
 C1 is 14.2% and C4 (a compound access re-derives its tail at run time) is 12.7%, and C4's share *contains* part of C1's, so their union is between 14.2% and 26.9%.
 The band is 6.08x multiplied by one minus 57.5% at the conservative end and by one minus 70.2% at the other.
+**Plan against 2.58x, because 1.81x is strictly unattainable**: the optimistic end is reached only if C1 and C4 overlap by exactly zero, and the attribution's own wording ("contains part of") says they do not.
+The band's true upper end is open, not 1.81x, and nothing measured says where inside it the real union sits.
 
 **Measured checkpoint: 5.91x to 2.86x** in one interleaved run, from prototype P1 alone, which adds `IntDiv` and `Remainder` to the small-integer path.
 That already beats C2's own implied 3.56x, because P1 also removes the operand conversion, the result allocation and the root pushes attributed to their own frames.
 So the band is soft in both directions and is a prediction, not a measurement.
 
 **C4 is D9's own unmet mandate and is named here so 4d-2 does not rediscover it.**
 `plan:389` asks for memoisation built into the stem and compound-variable design from the start rather than porting the slow shape first.
 The slow shape was ported first: the split happens once at plan-build time and the result is discarded, and every access re-splits and re-resolves each tail through the name-keyed map.
 
 *Vacuity:* as criterion 1.
 Additionally, a `compound` result that improved because C2's fix landed is **not** evidence about stems: reading 6.08x as a statement about stem machinery would be wrong by about a factor of two, since `Number::div` alone is 41.5% of the axis and the stem machinery proper is 9.5%.
 
@@ -216,86 +302,108 @@ Two different sets of causes landing on the same number is a coincidence worth r
 Nothing in the attribution separates irreducible decimal-division work from an implementation this crate could improve, and the C++ profile's own `b3_decarith` row puts 73.5% of the oracle's time in arithmetic on a decimal benchmark, so some of it is shared cost.
 
 **This is Phase 2's parity debt's scheduled re-measurement.**
 `d1-decision.md:76` recorded 1.22x and said in the same entry it was a lower bound that would get worse once the Rust side started paying for parsing, dispatch and variable lookup.
 It did, by a factor of about 2.2 as measured today.
 
 *Vacuity:* as criterion 1.
 
 ### 5. `alloc4c` -- allocation throughput, at parity, and **2.08x is a debt rather than headroom**
 
 > As criterion 1, for `alloc4c`.
 
-**Derived bar: 1.70x**, from 2.08x multiplied by one minus C1's 7.0%, C5's 5.2% and C7's 6.0%.
-C4's 4.0% is excluded because it overlaps C1 on this axis.
+**"Debt" in this criterion's heading is the attribution's word, not this gate's.**
+It means a deficit that will get worse, which is the opposite of headroom.
+It is not the *recorded debt* defined under "A debt is an amendment" above, and nothing in this criterion grants one.
+
+**Derived bar: 1.62x to 1.70x, and 1.70x is the end to plan against.**
+C5's 5.2% and C7's 6.0% are disjoint from everything else named and are summed.
+C1's 7.0% and C4's 4.0% overlap on this axis exactly as they do on `compound`, so they get the same union treatment: their union is bounded below by 7.0% and above by 11.0%, giving a removable share of 18.2% to 22.2% and a bar of 2.08x multiplied by one minus each.
+The optimistic 1.62x end assumes the overlap is zero, which the attribution's "contains part of" denies, so like `compound`'s it is strictly unattainable.
 
 **THAT NUMBER MUST NOT BE READ AS "NEARLY AT PARITY ON ALLOCATION", AND IT WILL BE WRONG THE MOMENT A COLLECTOR LANDS.**
 Roughly **83%** of the oracle's time on this axis is its collector marking `tab.`'s growing tail table, which this crate never performs because this crate never collects.
 Measured two ways, on the same tree:
 
 * Profiled: `MemoryObject::newObject` 69.1% total, `MemoryObject::collect` 56.0% total, `CompoundTableElement::live` 20.7% self and the largest self-time function in the oracle's profile.
-* Flat-tail A/B, `tab.1 = i` in place of `tab.i = i` and nothing else changed, five interleaved repetitions per cell, all four cells printing `12888896`: **2.06x-2.17x growing against 8.50x-8.79x flat**.
+* Flat-tail A/B, `tab.1 = i` in place of `tab.i = i` and nothing else changed, five interleaved repetitions per cell, all four cells printing `12888896`: **2.17x growing against 8.50x flat**, from medians of 1077.4 ms and 2338.2 ms growing, 184.1 ms and 1564.8 ms flat.
   Removing the growing table makes the oracle 5.85x faster and this crate 1.49x faster.
+  Those are the committed medians in `phase-4d-attribution.md`; a wider pair quoted during review came from an uncommitted scratchpad probe and is not used here, because a figure that names no commit is not a figure.
 
 So the two sides are not paying for the same thing, and the axis's honest position on the allocation dimension -- the comparison with the collector's work removed from the denominator -- is about **8.5x**, the same neighbourhood as `strings`.
 **The verdict is NOT MET on both readings**, which is why this finding changes what improving the axis means rather than what the table says.
 
 **A collector is a cost here, not a win, and the retention document's prototype never measured it.**
 That prototype covered `strings`, `arith`, `compound` and `varlookup`; `alloc4c` did not exist as an axis when it ran.
 This is the one axis where a collector has a large live set to re-mark and little to reclaim, so whoever lands a trigger policy in 4d-2 measures `alloc4c` before and after and expects the ratio to move the wrong way.
 
 **The dimension is partially covered, and the gate says which part.**
 `alloc.rex`, D9's own allocation program, still exits 120 on a message send and is Phase 5's; `alloc4c.rex` is its 4c-surface analogue and substitutes a compound-variable tail and a `||` concatenation for the array and the `.string~new`.
 Its own header states what it does and does not preserve.
 Parity on `alloc4c` is therefore parity on allocation throughput as a 4c-surface program measures it, and not on `alloc.rex`.
 
 *Vacuity:* as criterion 1, plus the specific trap this criterion exists to block -- a bar of 1.70x on this axis reads as near-parity and is measured against a denominator inflated by work this crate does not do.
 
 ### 6. `heapshape` -- full-GC pause, at parity
 
 > The Rust collector's pause on the `heapshape` graph falls inside or below the oracle's measured pause, with both arms building the same object count and the residual shape difference stated.
 
 **No derived bar, because no share-based decomposition of the collect loop exists.**
 Nothing in this phase profiled `Heap::collect` by cause, so there is no share to remove and no ratio to imply.
 What exists instead is a named mechanism, measured directly: `size_of::<Body>()` grew from **32 bytes to 80** between D1's close and this measurement, dominated by `Body::Stem`'s `HashMap` payload, so every value including a plain string now carries a stem's footprint through the mark phase.
 `Heap::collect`'s algorithm is unchanged over that span, confirmed by reading every intervening commit that touches `heap.rs`.
 
-**Measured 2026-08-09 (Task 5, committed at `c445154c`): oracle median 17.966 ms over five runs, Rust 28.5-29.1 ms over two independent criterion runs at n=10, a ratio of 1.58x-1.66x.**
+**Measured 2026-08-09 and committed in `d1-decision.md`'s Phase 4 addendum (`:113` onward, from `ad7c36f0`): oracle median 17.966 ms over five runs, Rust 28.5-29.1 ms over two independent criterion runs at n=10, a ratio of 1.58x-1.66x.**
 The previously recorded 26.5 ms / 1.45x figure is **not reproducible**: `a3178cff` replaced `Body::String(String)` -- the exact variant D1's risk analysis names -- and the bench was mechanically updated to keep compiling and never re-run.
 So this axis also now misses the 1.5x Phase 1 debt threshold it was previously inside.
 
 **The two arms are not the same instrument and this criterion says so.**
 The oracle's arm is `TIME('E')` around `GC('F')` in `heapshape.rex`; the Rust arm is a criterion bench at `rust/crates/rexx-core/benches/heap.rs`.
 `rexx-core` has no `Directory` body variant, so the bench approximates the oracle's 1,000-key directory with a flat array holding 1,000 key strings: the same object count, not the same shape, and the residual difference cuts in the oracle's favour.
 It is documented rather than closed.
 
+**What protects this criterion, and what does not, because the four shared protections above do not reach it.**
+`heapshape.rex` is in `AXES` as `Role::Blocked`, so `verify_axis_list` does pin the oracle arm's program against `rust/bench-programs/` and the suite turns red if that program ever stops failing on this crate while still declared blocked -- which is the case that matters once Phase 5 lands.
+The oracle binary is the fingerprinted one, checked by sha256 at the time of measurement rather than by the harness.
+**Three protections do not apply.**
+`loop_count` is meaningless here, because the figure is a pause and not a throughput.
+The two arms are **not interleaved** and were taken by different tools minutes apart, so this is the one comparison in the gate without the drift protection every other one has.
+And nothing pins the Rust bench's graph to `heapshape.rex`'s: the two constructions are kept in step by hand and by the paragraph above, which is why the residual `Directory` gap is documented rather than asserted.
+
 *Vacuity:* the degenerate execution is a smaller graph, so the criterion requires both arms to build the same object count from `heapshape.rex`'s own construction, and the bench's graph shape is the subject of the comparison rather than a parameter of it.
 Deleting the Rust bench does not leave this green: the criterion has no other instrument and would read NOT MEASURED.
 
-### 7. `rexxcps` -- the oracle's own whole-program benchmark, reported with a verdict
+### 7. `rexxcps` -- the oracle's own whole-program benchmark, **reported, and it gates nothing**
 
-> The internal clauses-per-second ratio is reported at every measurement, with the spread of every run that produced it.
+> The internal clauses-per-second ratio is reported at every measurement, with the spread of every run that produced it, and no criterion in this document depends on it.
+
+**It gates nothing, in the same sense as criteria 8 and 9**, and this entry says so rather than letting a criterion whose only requirement is "is reported" sit in a table of MET-or-not verdicts where it could never fail.
+Its figure is quoted in the assessment because a whole-program regression should be visible, not because anything closes on it.
 
 **Not one of D9's eight dimensions.**
 D9 says whole-program benchmarks come from `samples/`, and this is that.
 It is carried because 4a's design spec named it and because it is the only figure in this phase that exercises the interpreter across many constructs at once.
 
-**No derived bar.**
-No cause in the attribution carries a `rexxcps` share, because the profiling was per axis.
+**Two reasons it cannot gate, and either alone is sufficient.**
+No cause in the attribution carries a `rexxcps` share, because the profiling was per axis, so it has no derived bar and no falsifiable prediction.
+And it is the least stable figure in this gate: four measurements of the same tree gave 7.41x (one pair, explicitly not a baseline), 6.21x (a contended run), 7.31x and 7.33x.
+Read 7.33x as the accepted figure for the current measurement and 6.21x-7.41x as the honest range across what has been observed, never to three digits.
 
-**It is the least stable figure in this gate and must not be quoted to three digits.**
-Four measurements of the same tree gave 7.41x (one pair, explicitly not a baseline), 6.21x (a contended run), 7.31x and 7.33x.
-Read 7.33x as the accepted figure for the current measurement and 6.21x-7.41x as the honest range across what has been observed.
+**What protects it, and what does not, because the four shared protections above do not reach it either.**
+It is not in `AXES` -- it is a hardcoded path constant into the read-only C++ tree -- so `verify_axis_list` does not pin it, and a `rexxcps` row silently leaving the report would be caught by nothing.
+`loop_count` cannot apply, because the benchmark self-calibrates its own trial count.
+What does apply: the program lives in a tree this project may not modify, the two sides alternate within the run like every axis, the oracle fingerprint is the same one, and each side's `Averaged:` line is quoted so the work asymmetry is visible rather than inferred.
 
-*Vacuity:* the benchmark self-calibrates, so the two sides do different amounts of work and their wall times are not comparable; only the per-clause figure is, and each side's `Averaged:` line is quoted at every measurement so the asymmetry is visible rather than inferred.
+*Vacuity:* the benchmark self-calibrates, so the two sides do different amounts of work and their wall times are not comparable at all; only the per-clause figure is.
+A criterion over its wall time would be meaningless, and a criterion over its cps ratio would be decided by which of 6.21x and 7.41x was measured.
 
 ### 8. `startup` -- cold start, **not comparable at 4d, and nothing is gated on it**
 
 > `startup` is recorded as not comparable, with both sides' figures and D2's absolute target, and no criterion in this document depends on it.
 
 **This crate has no `CoreClasses.orx` bootstrap, so it starts fast by doing none of the work the oracle does.**
 That is Phase 5's.
 Recording 1.797 ms against the oracle's 7.645 ms as a pass would be recording the absence of a feature as a performance result.
 
 **D2's target is absolute, not a ratio: parse and execute 5,203 lines of `CoreClasses.orx` plus `StreamClasses.orx` in under about 55 ms** (`plan:157`, from the oracle's 5.1 ms plus the 50 ms delta threshold).
 D2's decision is already made and is **(a), no saved image** -- "Ship (a) either way" (`plan:151`) -- so the Phase 5 lever is parser and bootstrap throughput, not an image.
 The ratio at that point will look terrible whatever happens, which is precisely why the ratio is not the gate.
@@ -334,25 +442,26 @@ A memory regression test built from the benchmark axes inherits that blind spot
 
 **What a fix is worth, measured by prototype and reverted:** peak RSS falls 42x on `arith`, 16x on `compound`, 298x on `strings` and 60x on `varlookup`, and `strings` gets **16% faster**.
 Those figures cover cause A and cause B's `loop_advance` site only; the `eval_condition` site was located and measured but not prototyped, so closing it is unquantified.
 
 *Vacuity:* the degenerate execution is measuring only counted loops, which is why the conditional shape is named in the criterion rather than in a note.
 A second degenerate execution is measuring at one iteration count, where a fixed cost is indistinguishable from a linear one, which is why two counts a factor of eight apart are required.
 Deleting the subject does not leave this green: with no retention instrument at all the criterion reads NOT MEASURED, and the defect is independently recorded in `phase-4-exclusions.txt` under KNOWN GAPS so it survives this document.
 
 ### 11. Platform coverage -- **the gate is incomplete on this axis and is not silently Linux-only**
 
 > Every criterion above is recorded per platform, with Linux measured and every other platform in the matrix recorded as outstanding.
 
-**`:39` names Linux and macOS. `:35` names five: Linux, macOS 15 arm64, Windows/MSVC, FreeBSD 14.2 and OpenBSD 7.8, and says every phase gate runs on all five.**
+**`:39` names Linux and macOS.**
+**`:35` names five -- Linux, macOS 15 arm64, Windows/MSVC, FreeBSD 14.2 and OpenBSD 7.8 -- and says every phase gate runs on all five.**
 Nothing in this phase ran on any platform but this Linux machine.
 
 **Three specific things are unchecked rather than presumed fine.**
 
 * No baseline exists for macOS, Windows, FreeBSD or OpenBSD, for either interpreter.
   `perf-baseline.md`'s "What is still missing" says the same and names the work: a CI job per platform that builds the C++ oracle, runs this suite, and either commits numbers or documents why a platform could not produce them.
 * **`rexx-bench-suite` is Linux-only as written.** The address-space cap is a `/bin/sh` builtin and the fingerprints come from `ldd`, `stat` and `sha256sum`.
   A macOS measurement needs harness work before it needs a machine.
 * The known OpenBSD SIGSEGV in the C++ baseline is pre-existing and may mean that platform produces no baseline at all; that has not been checked from this machine.
 
 **So 4d cannot close on Linux alone**, and this criterion is what stops a Linux-only pass being read as a gate pass.
 Recording Linux as met with the rest outstanding is the honest state; recording the gate as met would not be.
@@ -389,61 +498,73 @@ An optimisation that happens to fix a keyword body *improves* a count while turn
 
 ## Assessment, 2026-08-09
 
 Figures are quoted from the documents named at the top, at the commits named there.
 Nothing in this section was measured by this task.
 
 | # | criterion | result |
 |---|---|---|
 | 1 | `varlookup` at parity | **NOT MET** -- 4.35x raw, 4.38x net; derived bar 2.80x |
 | 2 | `compound` at parity | **NOT MET** -- 6.08x raw, 6.12x net; derived bar 1.81x-2.58x |
 | 3 | `strings` at parity | **NOT MET** -- 10.61x raw, 10.70x net; **no derived bar** |
 | 4 | `arith` at parity | **NOT MET** -- 2.70x raw, 2.72x net; derived bar 2.16x |
-| 5 | `alloc4c` at parity | **NOT MET** -- 2.08x raw, 2.09x net; derived bar 1.70x, **and 2.08x is a debt, not headroom** |
+| 5 | `alloc4c` at parity | **NOT MET** -- 2.08x raw, 2.09x net; derived bar 1.70x (band 1.62x-1.70x), **and 2.08x is a debt, not headroom** |
 | 6 | `heapshape` full-GC pause at parity | **NOT MET** -- 1.58x-1.66x; no derived bar |
-| 7 | `rexxcps` reported | **REPORTED, NOT MET** -- 7.33x, observed range 6.21x-7.41x |
+| 7 | `rexxcps` | **REPORTED, GATES NOTHING** -- 7.33x, observed range 6.21x-7.41x |
 | 8 | `startup` | **NOT COMPARABLE** -- D2's absolute target is about 55 ms; nothing is gated on it |
 | 9 | `dispatch` | **NOT COVERED** -- Phase 5 |
 | 10 | unbounded retention closed | **NOT MET** -- linear unbounded growth from two causes; two axes abort under the standard cap |
 | 11 | platform coverage | **INCOMPLETE** -- Linux measured, macOS/Windows/FreeBSD/OpenBSD outstanding |
 | 12 | measurement integrity | **MET** for the current measurement; re-asserted at the closing one |
 | 13 | correctness and hygiene | **MET** |
 
 **Phase 4d does not close today.**
-Six axes miss parity, one has no derived bar at all, a correctness defect is open, and the platform matrix is one of five.
+Six axes miss parity, two of them have no derived bar at all, a correctness defect is open, and the platform matrix is one of five.
 That is the state 4d-2 starts from, and it is stated here so that no later document has to reconstruct it.
 
 ### Where the derived bars leave parity
 
-**Not one derived bar reaches 1.00x.**
+**Not one derived bar reaches 1.00x**, which is the result stated at the top of this document and quantified here.
+The last column is what would still be owed with the named causes perfectly removed, expressed as work remaining rather than as the bar repeated.
 
-| axis | today | derived bar | still short by |
-|---|---:|---:|---:|
-| `varlookup` | 4.35x | 2.80x | 2.80x |
-| `compound` | 6.08x | 1.81x-2.58x | 1.81x-2.58x |
-| `strings` | 10.61x | none | unquantified |
-| `arith` | 2.70x | 2.16x | 2.16x |
-| `alloc4c` | 2.08x | 1.70x | 1.70x, against an inflated denominator |
-| `heapshape` | 1.58x-1.66x | none | unquantified |
+| axis | today | derived bar | what is still owed at the derived bar |
+|---|---:|---:|---|
+| `varlookup` | 4.35x | 2.80x | 2.8x the oracle, with about 64% of what would remain unattributed |
+| `compound` | 6.08x | 2.58x (band to 1.81x, unattainable end) | 2.6x the oracle, from four named causes fully removed -- the deepest cut any axis has and still not close |
+| `strings` | 10.61x | none | unquantifiable: 17.5% named against a 2.8% measured win, five sixths unattributed |
+| `arith` | 2.70x | 2.16x | 2.2x the oracle, and `Number::div`'s 28.7% is outside the derivation entirely |
+| `alloc4c` | 2.08x | 1.70x (band to 1.62x, unattainable end) | 1.7x on today's comparison, and about **8.5x** on the collector-free comparison the flat-tail control measures -- this row understates the shortfall by a factor of five, which is the whole finding |
+| `heapshape` | 1.58x-1.66x | none | unquantifiable: nothing profiled the collect loop by cause |
 
-**So the attribution as it stands does not predict parity on any axis**, and that is the single most important thing this gate records.
-4d-2 planned solely from the seven named causes would run out of causes before it ran out of gap.
-Either more attribution is needed before the plan is written, or the debt shape applies and the shortfall is carried forward named and measured; this gate does not choose between those, because choosing is planning 4d-2.
+**So the attribution as it stands does not predict parity on any axis.**
+4d-2 planned solely from the seven named causes runs out of causes before it runs out of gap, and stopping-rule condition 3 above is the terminus that predicts.
+The two outcomes available there -- attribute further, or take a debt under the mechanism above -- are both deliberate acts; this gate does not choose between them, because choosing is planning 4d-2.
 
 ---
 
 ## 4d-2's stopping rule and amendment rule
 
-**Stopping rule.**
-4d-2 ends when every axis meets its bar, or when a task's measured result contradicts the attribution -- whichever comes first.
+**Stopping rule, and it says which bar it means, because the phase spec's wording does not.**
+The spec reads "4d-2 ends when every axis meets parity, or when a task's measured result contradicts the attribution -- whichever comes first", and this gate carries that with one gap closed.
+**"Its bar" is the closing bar, parity, never the derived bar.**
+Reaching every derived bar is not a stopping condition, because this gate's own result is that no derived bar reaches parity -- so a rule read the other way would end 4d-2 with the phase still open and say nothing about what happens next.
+
+So 4d-2 ends on the first of **three** conditions, and only the first closes the phase:
+
+1. **Every axis meets the closing bar, parity, under the three-run rule.** 4d closes.
+2. **A task's measured result contradicts the attribution.** 4d-2 stops and re-plans; the phase stays open.
+3. **The causes are exhausted -- every named cause has landed and axes still miss parity.**
+   This is the terminus this gate predicts, and it is not closing.
+   Two outcomes are available and both are deliberate acts: attribute further and re-plan 4d-2 from a wider attribution, or take a debt per axis under the mechanism above, which is an amendment the user grants and not something 4d-2 can decide for itself.
+
 If 4d-1 names more causes than the plan sized for, 4d-2 re-plans rather than growing.
 
 **What "contradicts the attribution" means concretely**, so the rule is applicable rather than decorative: a task names the cause it addresses and the axes it must move and by how much, from the derived bar above, and reports the measured move against that prediction.
 A task whose measured move matches nothing is a recorded finding about the attribution, not a silently-landed change.
 A task that reaches its axis's derived bar by some other route has not confirmed the attribution and says so.
 
 **Amendment rule.**
 **Any later change to a bar carries both wordings and the reason.**
 `phase-4c-gate.md`'s criterion 4 is the model: its negative control was weakened after a measurement showed the plan's version unsatisfiable, and it survived review **only** because the amendment was recorded with the original wording beside it.
 A criterion that moves after the measurement and does not say so has stopped being a criterion.
 
 **Three rules that constrain 4d-2 and are not criteria.**
@@ -453,35 +574,38 @@ A criterion that moves after the measurement and does not say so has stopped bei
   Re-profiling the strongest case showed mimalloc roughly halving its own per-call share and still buying only 18.4% of wall, because the surrounding hashing and copying does not shrink when the allocator changes.
   D1's pre-registered side byte-arena is the un-replaced candidate.
   Any allocator proposal must additionally clear the platform matrix: the swap was built on Linux only, and macOS, Windows, FreeBSD and OpenBSD are explicitly unchecked.
 * **A collector is a win on `strings` and a cost on `alloc4c`, and only the first half is measured.**
   A trigger policy lands with `alloc4c` measured before and after, expecting the ratio to move the wrong way.
 * **The retention fix covers both push sites or it is not a fix**, per criterion 10.
 
 ---
 
 ## What this gate found
 
 * **The gate has no bar that reaches parity on any axis.**
-  Every derived bar above is a prediction of the ratio the named causes can buy, and the best of them is 1.70x on the axis whose denominator is inflated.
+  Every derived bar above is a prediction of the ratio the named causes can buy, and the best conservative end of them is 1.70x -- on the axis whose denominator is inflated by collector work this crate does not do.
+* **The one route by which this phase could close without parity had an intent and no mechanism.**
+  The spec's debt sentence, taken alone, is satisfied by three runs and a table of NOT METs.
+  It now requires a named attributed residual per axis, a statement of what would close it, a re-measurement point, and a user-granted amendment carrying both wordings -- so an axis whose residual is unattributed cannot carry a debt at all, which today rules out the two axes with no derived bar.
 * **`alloc4c`'s 2.08x nearly became a bar meaning its opposite.**
   Roughly 83% of the oracle's time on that axis is collector work over a growing tail table that this crate never performs; with the growing table removed the axis reads about 8.5x.
   A gate reading 2.08x as "nearly at parity on allocation" would have been wrong the moment a collector landed, and would have been read by 4d-2 as a reason to look elsewhere.
 * **`strings`, the worst axis, cannot carry a derived bar**, and saying so is more useful than deriving one anyway: 17.5% named against a 2.8% measured combination win is a factor of six, with roughly five sixths unattributed.
 * **`arith`'s largest single share is not reachable by the fix that transformed `compound`.**
   `Number::div` is 41.5% of `compound` and 28.7% of `arith`, and the same prototype moved the first by -51.6% and the second by -0.7%, because `arith`'s divisions have non-integer operands.
   One 4d-2 task for both axes would half-miss.
 * **The cost is allocation count, not allocator quality**, measured rather than argued, with the `varlookup` loss as the internal consistency check that a quality story cannot produce.
-* **The gate's own decision rule is decidable by noise near the boundary**, and the evidence is three runs of a byte-identical binary: `compound`'s disjoint intervals and `alloc4c`'s -7.2%.
-  The response is three independent runs and a declared undecidable band, not a variance model from two runs.
+* **The gate's own decision rule is decidable by noise near the boundary**, and the evidence is three runs of a byte-identical binary: `compound`'s disjoint intervals and `alloc4c`'s -7.2%, the latter part between-run and part between-harness.
+  The response is three independent runs and a declared undecidable band expressed as a percentage of the parity threshold ratio, not a variance model from three runs.
 * **`heapshape`'s recorded Phase 1 figure was not reproducible**, and rebuilding the comparison moved it from 1.45x to 1.58x-1.66x -- out of the 1.5x band it was previously inside -- for a reason unconnected to the string-representation risk D1 named: `size_of::<Body>()` grew from 32 bytes to 80, dominated by a cold `Stem` variant that every value now carries.
 * **This unit could not produce a pre-optimisation baseline, only a current one.**
   Five speedups landed after Task 2's baseline and before Task 2b's, which contradicts this phase's own no-optimisation rule.
   It is recorded here as it is in `perf-baseline.md` and `task-2b-brief.md`: the bar-before-optimisation property is partly spent, and no document should claim otherwise.
 
 ## What 4d inherits forward
 
 * **`dispatch` to Phase 5**, as a D9 dimension nothing in 4d measures.
 * **`alloc.rex` to Phase 5**, with `alloc4c.rex` covering the allocation dimension on the 4c surface only.
 * **`startup` to Phase 5**, against D2's absolute target of about 55 ms for 5,203 lines and D2's already-made decision of (a), no saved image.
 * **The platform matrix**, which needs harness work before it needs machines: `rexx-bench-suite` is Linux-only as written.
 * **The unbounded-growth defect**, recorded independently in `phase-4-exclusions.txt` under KNOWN GAPS so that it survives whatever happens to this document.
