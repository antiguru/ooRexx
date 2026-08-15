# Task 8 report: write the gate

Status: DONE.
Deliverable: `docs/superpowers/plans/phase-4d-gate.md` (new, 13 criteria, modelled on `phase-4c-gate.md`).
Base commit `e743e05e`; committed as `cd408fa66fb8549c82ec370a39e33bf5a94768d8` (`cd408fa6`), read back with `git log -1 --format=%H`.
The report itself is not in the commit: `.gitignore:19` ignores `.superpowers/`, as it did for every prior task in this phase.
Nothing was measured by this task; every figure is quoted from `perf-baseline.md` (current section, 2026-08-08 at `d233d1e9`), `phase-4d-attribution.md` (2026-08-09 at `c445154c`, Task 7 at `9ce83f14`), `phase-4d-retention.md` (2026-08-08 at `c9a90906`), and two task reports (Task 5's `heapshape` rebuild, Task 7's feasibility check).

## The structural decision the gate rests on: two bars per axis

The brief says the bar is parity, unamended, **and** that each bar must carry a derivation of the form "the ratio implied by removing cause C's measured self-time share".
Those are two different numbers and collapsing them would have produced either a trivial gate (every bar is 1.00x, derivation ":39") or an amended one (a bar of 2.80x on `varlookup`, which is a lowered bar wearing a derivation).

So the gate carries both, explicitly labelled:

* **The closing bar is parity**, identical on every axis, from Global Constraints `:39`, and it is the only thing that closes 4d.
* **The derived bar is a prediction**, the ratio the attribution's named causes imply if removed, and it is what 4d-2's tasks are measured against under the stopping rule.
  Meeting it does not close 4d.
  Its job is to be falsifiable: a task that lands its cause and does not move its axis toward the derived bar has contradicted the attribution.

This is exactly the spec's own design for 4d-2 (`:119-125`), so the two-bar shape is the spec's, not an invention here.

**The headline result of doing this honestly: not one derived bar reaches 1.00x.**
The attribution as it stands does not predict parity on any axis, and the gate says so in its own section rather than leaving it to be discovered when 4d-2 runs out of causes.
It deliberately does **not** choose between "more attribution is needed before 4d-2 is planned" and "the debt shape applies" -- choosing is planning 4d-2, which is out of scope.

## The per-axis bars and their derivations

| axis | today (raw / net) | derived bar | derivation |
|---|---|---|---|
| `varlookup` | 4.35x / 4.38x | 2.80x | 4.35x x (1 - C1 32.7% - C7 2.9%); the attribution states C1's only overlap among C1-C5,C7 is with C4, so these two may be summed. Measured checkpoint: P2 took 4.36x to 3.92x in one run, collecting only C1's name-keyed half. |
| `compound` | 6.08x / 6.12x | 1.81x-2.58x | 6.08x x (1 - C2 41.5% - C7 1.8% - the C1/C4 union), and a band because C4's 12.7% is contained in C1's 14.2%, so the union is 14.2%-26.9%. Measured checkpoint: P1 alone took 5.91x to 2.86x, already beating C2's implied 3.56x. |
| `strings` | 10.61x / 10.70x | **none** | Cannot be derived. Named removable share 17.5% (C1 7.6 + C5 9.9) against a measured combination win of 2.8%, a factor of six, with five sixths unattributed. What it rests on instead: the retention prototype's 16% (implying 8.91x) and Task 7's -6.1%, neither a share of a named removable cause. |
| `arith` | 2.70x / 2.72x | 2.16x | 2.70x x (1 - C3 14.5% - C1 4.0% - C7 1.6%). The combined prototype's measured 2.15x is **not** confirmation: it came from a smaller set (C3 + C1's name-keyed 2.4% = 16.9%), exceeded that set's share, and never touched C7. |
| `alloc4c` | 2.08x / 2.09x | 1.62x-1.70x, plan against 1.70x, **a debt not headroom** | C5 5.2% + C7 6.0% summed, C1 7.0% and C4 4.0% unioned (7.0%-11.0%) exactly as on `compound`. Flagged in capitals: ~83% of the oracle's time here is collector work over a growing tail table this crate never performs (flat-tail A/B, committed medians: **2.17x** growing vs **8.50x** flat), so the honest allocation-dimension position is ~8.5x and the bar is invalid past a collector landing. |
| `heapshape` | 1.58x-1.66x | **none** | No share-based decomposition of `Heap::collect` exists. Named mechanism instead, measured: `size_of::<Body>()` 32 -> 80 bytes, dominated by a cold `Stem` variant every value now carries; the collect algorithm is unchanged since D1. |
| `rexxcps` | 7.33x (range 6.21x-7.41x) | none | Not a D9 dimension; no cause carries a `rexxcps` share. Reported with its four-measurement range rather than to three digits. |

`startup` is recorded **NOT COMPARABLE** with D2's absolute ~55 ms target for 5,203 lines and D2's already-made (a)-no-image decision, gating nothing.
`dispatch` is recorded **NOT COVERED**, Phase 5, with the reason a 4c-surface analogue was not built (unlike `alloc`): a dispatch benchmark that avoids message sends is a different benchmark.

## The three hardest calls

**1. The gated measure excludes the fixed per-process offset, and the raw ratio must meet the bar as well.**
The brief demanded a decision on this and named the degenerate path (cut startup, improve every ratio, land nothing in the interpreter).
Netting closes that path -- but netting introduces the mirror-image path, since the offset is a separately measured quantity that gets subtracted, so inflating it lowers every net figure.
Requiring **both** readings means shrinking the offset can only help raw and inflating it can only help net, so neither alone carries a criterion.
Net is the binding reading on all five axes today (the oracle's offset is the larger, so netting shrinks its wall time by more); the two readings differ by at most 0.093 ratio units.

**2. Between-run variance: three independent runs, plus a declared undecidable band, and no variance model.**
The evidence is three runs of a byte-identical binary against three byte-identical oracle objects: `compound`'s disjoint intervals (6.32-6.39 vs 6.02-6.14) and the attribution run's `alloc4c` -7.2%.
The oracle's own intervals are about +/-1% of its median, so a 7.2% between-run movement is seven times the window `:39`'s verdict is decided in.
The gate states plainly that its own criterion is decidable by noise at the boundary and that no wording fixes that, then responds with two conservative rules: the closing measurement is **three independent runs** and the verdict must hold in all three; and an axis landing within the **largest between-run movement yet observed** (7.2% as of 2026-08-09, recomputed as a maximum at closing time, explicitly a lower bound and not an estimate) of the oracle's interval reads **UNDECIDED**.
UNDECIDED is not a pass and does not close 4d, so it cannot be used as an escape hatch -- it only ever withholds a pass.
The cost is stated rather than hidden: this makes a bare tie unprovable and requires beating the oracle by more than the band to read MET, which is a limit on what the *instrument* can certify, not an amendment to `:39`.

**3. `strings` gets no bar, and that is written as the finding rather than as a gap.**
The criterion is still parity like every other axis; what is missing is any derivation that predicts a reachable ratio.
The gate adds one forward-looking rule: a 4d-2 `strings` task cannot take its success criterion from this document, and raising the attributed share above 17.5% is that axis's first piece of work.

## Vacuity test

Applied to every criterion, with the answer written at the criterion.
Four protections are shared by the seven axis criteria and are stated once: the `AXES` literal is asserted against `rust/bench-programs/` by `verify_axis_list` (so deleting an axis is red); a `Role::Blocked` axis that stops failing turns the suite red; `loop_count` reads each program's own `n = <digits>` line so a denominator cannot be shrunk invisibly; and the oracle is fingerprinted with three sha256s and re-measured at gate time rather than reused.

Named degenerate executions blocked, beyond the two the brief supplied:

* **Criterion 10 (retention)** names `do n while zz; nop; end` as the acceptance test rather than `varlookup.rex`, because all four runnable loop axes are `do i = 1 to n` and a fix closing `loop_advance` alone would take every axis green while `DO WHILE` and `DO UNTIL` keep growing.
  It also requires two iteration counts a factor of eight apart, since one count cannot distinguish a fixed cost from a linear one.
* **Criterion 6 (`heapshape`)** requires both arms to build the same object count from `heapshape.rex`'s own construction, since the degenerate execution is a smaller graph; the residual shape gap (no `Directory` body variant, approximated by a flat array, cutting in the oracle's favour) is stated at the criterion.
* **Criterion 8 (`startup`)** is the *fix* for a criterion that could not fail: an interpreter that starts fast by not having a bootstrap satisfies any startup criterion, so there is none.
* **Criterion 11 (platform)** exists precisely to block "measure one platform, report per-axis verdicts, let the reader assume the matrix."

## Platform

Recorded **INCOMPLETE**, not silently Linux-only.
`:39` names Linux and macOS; `:35` names five and says every phase gate runs on all five; nothing here ran anywhere but this Linux machine.
Three things are recorded as unchecked rather than presumed fine: no baseline exists on the other four platforms for either interpreter; `rexx-bench-suite` is Linux-only as written (the cap is a `/bin/sh` builtin, the fingerprints come from `ldd`/`stat`/`sha256sum`), so macOS needs harness work before it needs a machine; and the known OpenBSD SIGSEGV may mean that platform produces no baseline at all.
Task 7's mimalloc feasibility is likewise Linux-built only, and the gate carries that into the allocator rule.

## 4d-2's rules, recorded in the gate

**Stopping rule**, with an applicability clause so it is not decorative: 4d-2 ends when every axis meets its bar, or when a task's measured result contradicts the attribution, whichever comes first; a task names its cause, the axes it must move and by how much from the derived bar, and reports the measured move against that prediction; a task that reaches a derived bar by another route has not confirmed the attribution and says so.

**Amendment rule:** any later change to a bar carries **both wordings and the reason**, in the style of `phase-4c-gate.md`'s criterion 4, whose weakened negative control survived review only because the amendment was recorded with the original wording beside it.

**Three constraints that are rules rather than criteria:** no allocator adoption is a candidate on quality grounds (Task 7's fork fell on the count side; `varlookup` got 1.2% *slower*, which a quality story cannot produce), and any allocator proposal must clear the platform matrix; a collector is a win on `strings` and a cost on `alloc4c` and must be measured on `alloc4c` before and after; the retention fix covers both push sites or it is not a fix.

**Stated explicitly in the gate: 4d-2 is not planned here.**

## Assessment recorded

| # | criterion | result |
|---|---|---|
| 1-5 | `varlookup`, `compound`, `strings`, `arith`, `alloc4c` at parity | NOT MET |
| 6 | `heapshape` full-GC pause | NOT MET (1.58x-1.66x; also outside the 1.5x band it was previously inside) |
| 7 | `rexxcps` | REPORTED, NOT MET |
| 8 | `startup` | NOT COMPARABLE |
| 9 | `dispatch` | NOT COVERED |
| 10 | unbounded retention closed | NOT MET |
| 11 | platform coverage | INCOMPLETE |
| 12 | measurement integrity | MET for this measurement, re-asserted at closing |
| 13 | correctness and hygiene | MET |

**Phase 4d does not close today**, and the gate says so in one sentence so no later document has to reconstruct it.

## Verification

* `cargo fmt --all --check` from `rust/`: **exit 0**, read unpiped.
* `cargo clippy --offline --workspace --all-targets -- -D warnings` from `rust/` with a **fresh, empty `CARGO_TARGET_DIR`**: **exit 0**, read unpiped, stdout and stderr captured to separate files.
  A fresh directory rather than a deleted `rust/target/` deliberately: deleting it would destroy `rexx-run`, whose sha256 three committed documents cite as their measured binary.
  All eight workspace crates appear in the build log, so the clean-target property is real rather than a cache hit.
* Working tree clean before the commit; the commit contains exactly the gate and this report.
* No `git add -A`, no `git reset --hard`, no `git checkout --`, no force-push.

## Review round 1 -- commit `a42b8a357fd99a29d22503f1f4e9c95c8033680d` (`a42b8a35`)

Spec compliance PASS, quality sound but not yet safe to optimise against; eleven items, all addressed in one commit.
Nothing was re-measured.

**Critical: the closing bar was not binding.**
The spec's debt sentence (`2026-08-08-phase-4d-performance-design.md:24`), carried into the gate as one line, supplied a route to close 4d without parity on any axis -- and the gate's own headline finding guarantees that is the route every axis would take.
The sentence is kept, since the decision was made above this document, and now carries the three things an intent lacks: **who** grants a debt (the user, at plan level, precedent Global Constraints `:36` on known-failure files), **whether** it closes the phase (only through an amendment carrying both wordings, so it goes through the amendment rule rather than around it), and **on what evidence** (NOT MET at the closing measurement, a named attributed residual with a measured share, what would close it and which phase owns that, and a re-measurement point in the shape of Phase 2's `arith` debt).
The evidence clause is what closes the degenerate route: an axis whose residual is unattributed cannot carry a debt, which today rules out `strings` and `heapshape`, the two axes with no derived bar.

**Major: the stopping rule did not say which bar.**
"Its bar" is now the closing bar, parity, never the derived bar, and the rule names **three** termini rather than two -- the third being the one this gate predicts, causes exhausted with axes still missing parity, which is explicitly not closing.
The two outcomes available there (attribute further, or take a debt under the mechanism above) are named as deliberate acts.

**Major: the undecidable rule could not be executed as written.**
It now reads: `T` is the parity threshold ratio (the oracle's interval upper bound over its median, on the reading being judged, about 1.011 on `alloc4c`); with band `b`, `R < T x (1 - b)` is MET, `R > T x (1 + b)` is NOT MET, between is UNDECIDED.
The band is a percentage of the **ratio**, which is the scale the observed movements are on.
It is applied to raw and net **separately** in each of three runs, with a total combination rule (MET only if all six evaluations read MET, NOT MET only if all six read NOT MET, UNDECIDED otherwise) so disagreement across runs lands in UNDECIDED rather than falling through.

**Major: the 7.2% floor was mis-characterised.**
The attribution says that run also changed harness (bash subshell + `date +%s.%N` against the suite's `/bin/sh` wrapper + `Instant`) and that this is part of why its medians moved.
Now stated as between-run **and** between-harness, with the note that the direction is conservative so it opens no hole, and corrected because later work will quote the gate rather than the attribution.
Also corrected: three byte-identical runs are on the record, not two.

**Major: the four shared protections did not hold for criteria 6 and 7.**
Scoped to 1-5, with per-criterion protection notes added.
`rexxcps` is a hardcoded path constant outside `AXES`, so `verify_axis_list` does not pin it and `loop_count` cannot apply to a self-calibrating benchmark; what does apply is the read-only tree, the alternation, the fingerprint and the quoted `Averaged:` lines.
`heapshape`'s `Role::Blocked` entry pins the `.rex` this crate cannot run, not the bench that produces the number; the two arms are also the one comparison in the gate **without** interleaving, which is now said.

**Medium, all addressed:** criterion 7 now gates nothing and says so in the manner of 8 and 9 (assessment row REPORTED, GATES NOTHING); the flat-tail A/B cites the committed medians 2.17x and 8.50x and drops an uncommitted scratchpad pair, with a sentence saying why; `heapshape` is cited to `d1-decision.md`'s Phase 4 addendum rather than a gitignored task report, and the preamble no longer claims any figure comes from a task report; the oracle's interval is netted when the net reading is judged, with a worked example; "independent" and "machine-quiet" are defined against the retention document's measured 18.94-vs-9.20 s residency contamination, and the closing runs' own movements do feed the band (which can only widen it); the central negative result moves to its own section above "The bar, and what it is not"; and the "still short by" column becomes "what is still owed at the derived bar", with `alloc4c`'s cell stating that the row understates the shortfall by a factor of five.

**Minors:** `compound`'s 1.81x end is now marked strictly unattainable, with 2.58x named as the end to plan against; `alloc4c` becomes a band (1.62x-1.70x) under the same union rule rather than a point, which resolves the inconsistency instead of explaining it; the derived-bar definition now covers unions and bands, not only sums; the two-sentence line at the platform criterion is split.
One thing the review did not raise and I added: criterion 5's heading word "debt" now collides with the newly defined recorded-debt term, so it carries a sentence saying it is the attribution's word for a deficit that will worsen and grants nothing.

Verification re-run identically: `cargo fmt --all --check` exit 0, and clippy from a second fresh `CARGO_TARGET_DIR` exit 0 with all eight workspace crates in the log.

## Review round 2 -- commit `c790e3a76aa12afa9fefa43095fb751a3c99380c` (`c790e3a7`)

Scoped re-review: all eleven round-1 items ADDRESSED, and the round introduced no new false or self-contradicting statement.
Three presentation minors, all fixed; nothing measured and no figure changed.

* **The worked example did not reproduce from its own displayed digits.**
  `T` was shown to three decimals (1.011) and the answer derived from it to three (0.939), but `1.011 x 0.928 = 0.938`; only unrounded `T` gives 0.9386.
  `T` is now **1.0114**, the example carries its intermediate (`1.0114 x 0.928 = 0.9386`), and the four-decimal convention is stated where `T` is defined.
  Checked both ways: 1.0114 x 0.928 = 0.9386 and (1.1408/1.1279) x 0.928 = 0.938614.
* **The debt section's closing sentence read as though all six missing axes were available for a debt**, immediately after requirement 2 rules two of them out.
  It now says which: two are barred until someone attributes them, and the other four would each cost the full four-part price.
* **Two table inconsistencies.**
  The assessment table's `compound` and `alloc4c` rows now present their bands the way the later table does -- the end to plan against, with the other marked unattainable -- and `alloc4c`'s row spells out that "not headroom" means a deficit that worsens when a collector lands, **not** the recorded-debt mechanism defined two sections above.
  Criterion 5's heading keeps the word "debt", because the disambiguating sentence directly under it quotes that heading; the risk the reviewer named was the table row read on its own, and that is where the fix went.

Verification re-run identically: `cargo fmt --all --check` exit 0, clippy from a third fresh `CARGO_TARGET_DIR` exit 0 with all eight workspace crates in the log.

## Concerns

* **The gate's central negative result is that no derived bar reaches parity.**
  If 4d-2 is planned straight from the seven causes it will exhaust them at roughly 1.7x-2.8x on four axes and with no prediction at all on two.
  The gate records this and deliberately does not resolve it, because resolving it is planning 4d-2.
* **The undecidable band is a rule, not a model, and it makes a tie unprovable.**
  It is the conservative choice and it can only withhold a pass, never grant one, but it is a real tightening of what can be certified and a later reader should see it as such rather than as `:39` restated.
* **`heapshape` and `rexxcps` carry criteria with no derived bar**, for the same reason `strings` does: nothing profiled them by cause.
  Three of the eight things this gate measures therefore cannot predict their own improvement.
* **The two-bar structure was an interpretation of the brief, and review tested it and found it sound**, agreeing that collapsing it gives either a trivial gate or an amended one.
  What was unsafe was not the structure but two sentences around it, both now fixed; recorded so a later reader does not reopen the structure looking for the defect.
* **Executing the undecidable band makes its cost concrete**: MET on `alloc4c`'s current numbers needs a ratio below 0.939, about 6% faster than the oracle rather than level with it.
* **The debt mechanism is the part most likely to be argued with**, because it makes closing an axis without parity expensive on purpose -- the price being attribution work rather than paperwork.
  If a later reader wants it cheaper, the amendment rule applies to it exactly as it applies to any other bar in the document.
