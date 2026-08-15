### Task 7-M2: Deliver the itemisation, and spike the two removals 7-M did not consider

**Added 2026-08-10 by Moritz, after Task 7-M's review.** The review confirms 7-M's *conclusion* -- most of the residual is structural, promotion stays monotonically worse per clause -- and rejects the evidence as not yet fit to carry the weight the withdrawn criterion puts on it. Its findings are in `.superpowers/sdd/2026-08-09-phase-4e-ir/task-7m-review.md` and are referred to below by their numbers.

**This task does not block Tasks 8, 9 or 10.** Nothing here changes what a promotion emits, and the two spikes are measured and reverted rather than landed. It does block Task 11, because exit criterion 4's falsification clause asks for exactly what this task delivers.

**Run it when the machine is free.** Every step below is a measurement and none of them is valid taken while another task is benchmarking.

### The harness comes first, because this task is its first user

**Decided 2026-08-10 by Moritz.** Finding I7 is the reason: the both-instruments rule was already in the spec when 7-M ran, 7-M reported five axes on instructions alone, and the gap survived to review. Tasks 8, 9 and 10 each end in "measure, record, commit" and would each re-derive the procedure from prose.

**A tool, not a document, and the distinction is measured in this repo rather than preferred.** `corpus/keyword-exempt.txt` derives its facts and polices them in both directions and needed no correction across thirteen tasks; a one-line prose count of the same kind of fact rotted four times. A methodology section is the prose version of a rule that has already failed once.

**What it must make unexpressible.** Each of these is a defect this phase has actually shipped or nearly shipped:

* **A claim on one instrument.** It emits `instructions:u` and `cycles:u` together or it emits nothing (I7).
* **A comparison assembled from two sittings.** The tool owns arm ordering and rotation; it takes a build and a workload, not two result sets.
* **A cross-binary comparison.** Both engine arms come from one build. Task 4c built the same source twice differing by one comment and read 7.8% between them, on a `.text` with the same sha256 -- within one binary is the only valid form here.
* **A single problem size.** Two sizes or it refuses. Separating per-pass cost from fixed cost is what settled "per promoted clause, zero per entry", and it is the question every promotion task asks.
* **A bare number.** Median of k rounds with the spread beside it, so a figure carries its own noise instead of being quoted alone into the next task's brief.

**And it writes a committed baseline file** -- one machine-readable record per landed task, so the next task's brief cites a path rather than ratios retyped out of prose. That is the failure this phase has repeated: a summary of a measurement is a new claim, and restating one is authorship rather than quotation.

**Scope: in-phase, arm against arm.** Phase 4f measures this binary against the oracle on wall clock, which is a different comparison; build so that mode can be added and do not add it here.

- [ ] **Step 0a: Build the harness**, and use it for every measurement below rather than running `perf` by hand.
- [ ] **Step 0b: Give it a negative control before trusting it.** Build a deliberately slowed arm -- a `black_box` spin in the region loop sized to a few per cent -- and confirm the harness reports a regression of about the size introduced. **This is not optional and it is what condemned the previous instrument:** 4b-M's control read +4.3%, +2.5% and +1.8% where it should have read within half a per cent, and a 7% signal sitting above a 6% artifact is not a measurement. A harness whose control passes silently is indistinguishable from one that never ran.

  **Size the control against the floor Task 8 measured, which is the tightest bound this phase has on instructions.** A control build differing only in `eval_node`'s three arms read **+0.74% on `emptyloop`'s IR arm** -- an axis with **no promoted read**, so the change could not have reached it. That is larger than four of the six per-axis moves in Task 8's own table, and its mechanism is not established. **A per-axis move under about 0.75% is therefore not yet distinguishable from this effect**, and the harness either resolves below it or reports that it cannot. Establishing the mechanism is in scope here if it is cheap and is not worth a redesign if it is not; recording the bound is not optional either way.

**The itemisation, which criterion 4 asks for and does not yet have:**

- [ ] **Step 1: Reconcile the breakdown to its own total (I1, I2).** The published column is `+20, +7, +3, -6, -3, +19, +19, -8, +26, +3` and sums to **80** against a stated total of **78**. Each row's internal sub-arithmetic checks out, and the by-function table below it is internally exact (`47+237+48 = 332`, `66+344 = 410`, `410-332 = 78`), which localises the two-instruction error to the five rows attributed to `run_ops`' arms: those sum to `+21` where the by-function table reads `+19`. Name the row that carries the error.
- [ ] **Step 2: Place the `+26` row (I5).** "Library helpers inlined into each function: bounds-checked indexing and `Result`/`Option` plumbing, 132 against 106" is a third of the whole cost and appears in neither the removed 19 nor the residual bullets. It is the same category the removed 19 came out of, so it is reducible by demonstration rather than by argument. Either put it in the residual with a reason it was not reduced, or reduce it.
- [ ] **Step 3: Supply the five missing cycle readings (I7).** `arith`, `compound`, `strings`, `alloc4c` and `startup` carry an `instructions:u` claim with no `cycles:u` beside it, and the phase's rule is that both are required for any claim -- a rule 7-M itself demonstrated by rejecting a variant on cycles alone. Four arms per axis (base and head, each engine), rotated so no arm keeps a slot, five rounds, medians. **"The IR arm improves on six axes" is unsubstantiated until this exists.**
- [ ] **Step 4: Correct three claims in the report (I3, I8, I9).** Change 1's marginal is given three different values -- 19, 16 and 10 -- and the report uses the only one with no build behind it; measure it. `emptyloop` at head shows the same instructions-fall-cycles-rise signature that `Driving` was rejected for, an order of magnitude smaller, and the report states only the reading that favours the change. "Byte-identical on six of seven axes" is contradicted by the report's own table at `startup` (538,065 against 538,561) and `emptyloop`, and byte-identity is inferred from equal instruction counts, which does not follow; the weaker true claim carries the argument on its own.

**The two spikes, under prototype-publish-revert.** Build, measure, revert without landing. Both arms, both instruments, interleaved within one sitting.

- [ ] **Step 5: Box the `Raised` variant of `Failure` (I4).** The nine-argument call returns through memory because `Failure` is 104 bytes; boxing takes it to 24 and `Result<ClauseRegion, Failure>` to 32. It changes nothing a promotion emits and needs no second implementation, so **the `sret` half of the call boundary is not inherent** whatever the number says. 7-M's stated reason for skipping it contradicts itself -- "a cost on the path both arms take, so it worsens criterion 4 whose denominator falls with it" against "the promoted path carries one more such return per clause", which point opposite ways -- and neither direction was measured. Measure it.
- [ ] **Step 6: Split the clause wrapper into a shared `enter` and a shared `leave` (I6).** `in_stepped_clause_with` and `in_clause` both take the clause's whole body as `impl FnOnce`, which is why the region's ops must live in a callee -- the call boundary at `+20` plus `run_clause_region`'s prologue and epilogue at `+19`, about half the whole cost. An enter/leave pair, or a guard whose `Drop` is the boundary, keeps **one** implementation of the clause unit while letting the driver run a region's ops in its own loop. `in_clause`'s epilogue reads the clause's `Result` and `value.rooted()`, and the failure path has to run the leave from the driver's own `?` site, so this is not free -- but harder is not inherent, and the point of the spike is the number rather than the design.
- [ ] **Step 7: Report both numbers and revert both. Commit the report and the corrected itemisation.** If either spike removes enough to change what 4f should do first, say so; landing it is a separate decision and not this task's.
- [ ] **Step 8: Re-express Task 8's figures through the harness into the baseline file.** Task 8 ran before the harness existed and measured by hand, so its numbers are the one gap in the record the harness is meant to close. Re-running them is cheap and it is also the harness's second negative control: a hand-measured figure it cannot reproduce is a finding about one of the two.

**Do not land either spike in this task.** Tasks 8, 9 and 10 are building on the current driver shape, and a redesign landing underneath them mid-flight is the collision this plan has already paid for once.

---

