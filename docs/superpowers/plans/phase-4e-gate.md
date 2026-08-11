# Phase 4e gate -- the IR is the default engine

Task 11 of `docs/superpowers/plans/2026-08-09-phase-4e-ir.md`, against the exit gate in `docs/superpowers/specs/2026-08-08-phase-4e-ir-design.md`.
BASE `5ab3028f`.

**The phase closes.**
Six of the seven criteria are met, one is met in part, and the part that is missing is an instrument rather than a behaviour.
Every verdict below carries what it could not see, and two of them carry a hole a later phase inherits.

| # | criterion | verdict |
|---|---|---|
| 1 | both engines agree across the suite under STRICT, and the IR is the default | **met** |
| 2 | the corpus differential does not regress | **met** |
| 3 | `rexxcps` runs on the IR and matches the oracle after masking | **met** |
| 4 | the ratios are recorded on both instruments, with the residual itemised | **met** |
| 5 | the patch table has a working consumer, with a win that reverts | **met** |
| 6 | the minimum promotion set is native, and its ops do the work | **met in part** |
| 7 | the trace oracle passes | **met, by a build that nothing re-runs** |

## What the default flip actually changed

Three places name an engine when nobody chose one, and they are not the same place.

* `Invocation::none` -- **flipped to `Engine::Ir`**. This is the one the spec's criterion 1 means, and it is what every in-process harness reaches: `corpus.rs`, `trace_oracle.rs`, `assertions.rs`, `bif_assertions.rs`, `keyword_assertions.rs`, `collect_stress.rs` and `spike.rs` all build their invocation from it.
* `rexx-run`'s `REXX_ENGINE` being unset -- **flipped to `Engine::Ir`**, so the shipped binary and the library agree. A set-but-unrecognised value is still refused rather than defaulted.
* `Interp::new` -- **left at `Engine::TreeWalker`**, and this one is not a body-entry point: `execute` overwrites it from the `Invocation` before any body runs, so it is inert for every production route. What it does decide is the engine for `run.rs`'s own unit tests, which construct an `Interp` directly. See criterion 7.

## Criterion 1: both engines agree, and the IR is the default

**Met.**

`cargo test --workspace`, exit status read unpiped, in four combinations:

| profile | gates | result |
|---|---|---|
| dev | none | 1423 passed, 0 failed, 4 ignored, exit 0 |
| dev | all four STRICT | 1423 passed, 0 failed, 4 ignored, exit 0 |
| release | none | 1423 passed, 0 failed, 4 ignored, exit 0 |
| release | all four STRICT | 1423 passed, 0 failed, 4 ignored, exit 0 |

STRICT is `REXX_CORPUS_GATE=1 REXX_ASSERTIONS_GATE=1 REXX_BIF_GATE=1 REXX_KEYWORD_GATE=1`, and all four harnesses printed their own `mode: STRICT` banner in the run -- read from the uncaptured stream, not inferred from the exit status.

The falsification the spec asks for is in the tree and did not have to be added.
`ir_dual.rs` builds every case once in `populations` and runs each twice, so "both arms saw the same programs" is a property of the code rather than a claim about two lists, and three assertions pin the list against something outside the file: `the_dual_harness_reads_every_phase_subset_file`, `the_sweep_runs_every_ootest_suite_a_sibling_harness_runs` and `every_population_the_tree_calls_for_is_present_and_non_empty`.
That file also reads no gate variable at all, so there is no mode in which it exits 0 having found a divergence.

**What it could not see: three divergences where the two engines agree and the oracle differs from both.**
None fits `KNOWN_DIVERGENCES`, which records engine-against-engine drift, and none fits a corpus exclusions file, which lists programs rather than behaviours.
None blocks this gate and none is a defect this phase introduced.

* `TRACE VALUE expr` emits no `>K>` line. The oracle traces a `TRACE VALUE` clause's own computed setting as a keyword value and this crate does not. Found by Task 6, whose report has the site and the one call that would fix it.
* A `CALL ON` handler delivered at a promoted loop header's boundary indents its own clauses two spaces less than the oracle -- `13 *-*   h:` against `13 *-*     h:`. Found by Task 7-M3's review, which rebuilt `7a7f5849`'s three files and reproduced identical bytes, so the enter/leave split did not introduce it.
* `interpret '::routine zfoo'` reaches the right error, 99.914 at rc 157, and omits the oracle's first echo line. Found by Task 10's review.

Whoever takes Phase 4f's trace parity should start from that list rather than rediscovering it.

## Criterion 2: the corpus differential does not regress

**Met.**
`REXX_CORPUS_GATE=1`, both profiles: **51 of 51 matching**, which is what Task 10 recorded before the default moved.

*What it could not see:* anything both engines get wrong together, which is the whole of criterion 1's blind spot restated from the other side.

## Criterion 3: `rexxcps` on the IR

**Met.**
`samples/rexxcps.rex` under `ulimit -v 8388608`, from a fresh empty directory, IR arm: exit 0, **empty stderr**, no `Failed` line.
Masked as the spec specifies -- the `Averaged:` calibration line and the clauses-per-second figure -- the IR arm's stdout is **identical to the oracle's**, and identical to the tree-walker arm's, while the raw bytes differ from both.

The address-space cap is worth recording because the documented oracle wrapper does not work here: at `ulimit -v 1048576` this crate aborts with `memory allocation of 402653184 bytes failed`, which is the interpreter stack reservation the tree's own note describes. The bench harness's 8 GiB cap is what both sides ran under.

*What it could not see:* the mask removes two of the five content lines, so what is actually compared is the banner, the `PARSE VERSION` string and the `System is:` line.
Speed is criterion 4's, and the cps ratio is Phase 4f's gate rather than this one's.

## Criterion 4: the ratios, on both instruments, with the residual itemised

**Met.**
The floor was withdrawn at `8e7ce246`; what replaces it is a recording obligation, and this section is the recording.

Every figure below comes from `rexx-arms`: both arms of one build selected through `REXX_ENGINE`, both instruments, two problem sizes, five rounds, all cells rotated inside one sitting.
The rows are in `rust/bench-baselines/phase-4e-arms.tsv` under tasks `11`, `11-spikes` and `11-shipped`; the tables here are read off those rows.

### The head ratios, which are Phase 4f's input state

IR arm over tree-walker arm, within one build, at each axis's committed bound, **on the binary this gate ships** -- the default flipped and `PatchSlot` on a `Cell`.

| axis | `instructions:u` | `cycles:u` |
|---|---:|---:|
| `varlookup` | **0.91619** | **0.88777** |
| `compound` | **0.97214** | 1.01856 |
| `arith` | **0.98167** | 1.01715 |
| `alloc4c` | **0.99682** | 1.01562 |
| `strings` | 1.00504 | **0.99944** |
| `startup` | 1.04501 | 1.00083 |
| `emptyloop` | 1.09223 | 1.00294 |

**Four axes are at or below 1.0 on instructions and two on cycles.**
That is not a criterion any more, and it is recorded as a fact rather than as a pass.

**The shipped binary was measured against `5ab3028f` in a sitting of its own**, because a flipped default and a changed slot type are a different build and this document's own rule is that a ratio belongs to the binary that produced it.
On `instructions:u` the two agree exactly on `emptyloop` (1.09223), `strings` (1.00504) and `alloc4c` (0.99682), and differ on the three arithmetic axes by the `Cell` saving and nothing else: the per-pass gap moves -1810.49 to -1818.50 on `arith`, -320.00 to -321.00 on `varlookup` and -429.66 to -431.75 on `compound`, which is **-8.01, -1.00 and -2.09** against the **-8, -1 and -2** the spike sitting measured on a different pair of builds.
So the default flip itself is measurement-neutral, as it has to be: `rexx-arms` names the arm explicitly in every run.

**`startup`'s cycle figure carries no resolution on either build and nothing should be read off it.**
It is a whole process of about 600,000 cycles; its range here is [0.96639..1.07926] and it read [0.89617..1.58778] in the five-build sitting.
Its instruction figure is stable to four decimal places and is the one that means anything.

### Per commit, because an aggregate cannot say what caused a move

One sitting, five builds, rotated together.
`instructions:u`, IR over tree-walker, at the committed bound -- and on this instrument the columns **are** comparable with each other, because an instruction count does not move with code placement.

| axis | `132c3395` 7-M | `9c20430f` T8 | `e63e8a00` 7-M3 | `16077ea1` T9 | `5ab3028f` T10 |
|---|---:|---:|---:|---:|---:|
| `alloc4c` | 1.02002 | 1.01026 | 0.99775 | 0.99769 | 0.99682 |
| `arith` | 1.00862 | 1.00882 | 1.00353 | 0.98315 | 0.98181 |
| `compound` | 1.01548 | 1.01571 | 1.00821 | 0.97379 | 0.97227 |
| `emptyloop` | 1.06539 | 1.06596 | 1.07855 | 1.08713 | 1.09223 |
| `startup` | 1.04383 | 1.04290 | 1.04359 | 1.04482 | 1.04465 |
| `strings` | 1.01697 | 1.01733 | 1.00606 | 1.00591 | 1.00504 |
| `varlookup` | 1.06256 | 1.02118 | 0.98666 | 0.92052 | 0.91645 |

The same sitting on `cycles:u`:

| axis | `132c3395` 7-M | `9c20430f` T8 | `e63e8a00` 7-M3 | `16077ea1` T9 | `5ab3028f` T10 |
|---|---:|---:|---:|---:|---:|
| `alloc4c` | 1.03625 | 1.03286 | 1.00973 | 1.02310 | 1.03507 |
| `arith` | 1.02718 | 1.00909 | 1.01798 | 0.99508 | 0.99258 |
| `compound` | 1.02523 | 1.02843 | 1.01386 | 0.97694 | 1.04129 |
| `emptyloop` | 1.04209 | 1.05582 | 1.11192 | 1.09853 | 1.00946 |
| `startup` | 1.06297 | 1.08466 | 0.95976 | 0.98762 | 1.28693 |
| `strings` | 1.03683 | 1.02896 | 1.02498 | 1.01785 | 1.02716 |
| `varlookup` | 1.04729 | 1.00072 | 1.01342 | 0.90375 | 0.88341 |

**A cycle row is five ratios over five different denominators and a difference along it is not by itself real work.**
`compound` reading 0.97694 at `16077ea1` and 1.04129 at `5ab3028f` is the clearest instance in this table, and the instruction row moves 0.15 of a point across the same boundary.

**`9c20430f` and `e7b8eef8` build a byte-identical `rexx-run`**, sha256 `36c2b61e...`, so they are one column rather than two.
That is Task 7-M2 landing the harness and the baselines and promoting nothing, confirmed by the binary rather than by its commit message -- the same relationship `132c3395` and `9da84dc3` already had.

### The per-pass gap, which is what travels between sittings

IR arm minus tree-walker arm, `instructions:u`, per loop pass.

| axis | 7-M | T8 | 7-M3 | T9 | T10 |
|---|---:|---:|---:|---:|---:|
| `alloc4c` | +320.03 | +163.91 | -35.89 | -36.90 | **-51.05** |
| `arith` | +524.48 | +536.48 | +214.48 | -1727.17 | **-1810.50** |
| `compound` | +239.33 | +243.38 | +127.05 | -406.68 | **-429.86** |
| `emptyloop` | +99.00 | +100.00 | +119.00 | +132.00 | **+140.00** |
| `strings` | +482.00 | +492.00 | +172.00 | +168.00 | **+144.00** |
| `varlookup` | +239.00 | +81.00 | -51.00 | -304.00 | **-320.00** |

### The structural residual, itemised

Task 7-M2 delivered the itemisation at `fd0ea6d1`, per promoted assignment clause, as the `+78` a promotion cost over the same clause run as `Op::Generic`.
Two of its rows have since been removed by a build and the rest stand; the table below is that one carried forward, with the evidence for each change beside it.

| item | 7-M2 | now | evidence |
|---|---:|---:|---|
| the nine-argument call to `run_clause_region` and its `sret` return | +20 | **0** | 7-M3's enter/leave split |
| `run_clause_region`'s prologue, epilogue and outcome mapping | +19 | **0** | the same commit |
| the region loop against `step`'s own single match -- the second dispatch level | +19 | +19 | survives the split, which keeps two matches |
| `stale`, i.e. `chunk.trace() != self.chunk_trace()` | +3 | +3 | unchanged, and D23 needs it |
| the `Op::Clause` arm's payload reads and unattributed remainder | +2 | +2 | unchanged |
| the shared clause unit -- the printed indent, the `SIGL` line | +3 | +3 | unchanged |
| less the `Op::Generic` arm it replaces | -6 | -6 | unchanged |
| less `assign_evaluated` ceasing to be a call of its own | -8 | -8 | unchanged |

**The two removed rows are removed by measurement rather than by argument**: `varlookup`'s per-pass gap goes +81 to -51 across `7d9cfb9a` and `alloc4c`'s +163.91 to -35.89, in one sitting, on both instruments.
The split's ledger is wider than the three axes 7-M3 reported: measured here it costs `emptyloop` +19 per pass and gains every other axis -- `alloc4c` -199.8, `arith` -322.0, `strings` -320.0, `compound` -116.3, `varlookup` -132.0.

**What is left that is removable, unchanged from 7-M2's answer except where a build has since settled it:**

* the second dispatch level, +19 -- not separately measured, and it survives the split.
* `eval_chunk_expr`'s `(kind, slot)` re-derivation, 8 -- removable, and out of scope: it changes what a promotion emits and gives `Chunk` a lifetime.
* the register file, about 6 -- not at this level; it is the same fact as "the program counter is an op index".
* `stale`, +3 -- not removable and nearly free; a `TRACE` run by one body clause must be seen by the next.

### A second residual, in a different unit, that nobody predicted

**This is not a row of the table above and must not be added to it.**
That table is per *promoted clause*; this is per loop *pass* of a body that has no promoted clause at all, and the two are different quantities.

`emptyloop`'s loop body is a single `nop`, which compiles to `Op::Generic` and hands its clause straight back.
Its IR-minus-tree-walker gap per pass, across the phase's last four boundaries:

| boundary | gap | delta | what landed |
|---|---:|---:|---|
| `132c3395` 7-M | +99.00 | | |
| `9c20430f` T8 | +100.00 | **+1** | variable reads promoted |
| `e63e8a00` 7-M3 | +119.00 | **+19** | the enter/leave split |
| `16077ea1` T9 | +132.00 | **+13** | arithmetic promoted |
| `5ab3028f` T10 | +140.00 | **+8** | the call forms promoted |

**+19 of the +41 is the enter/leave split, and that one is not a surprise**: it is the cost side of a trade whose gain side is every other axis, priced when it landed.
**The other +22 is, and it has no owner.**
Tasks 8, 9 and 10 each made a body that executes none of what they promoted measurably slower on the compiled arm, while its tree-walker arm moved 1514 to 1518 across the whole five builds.
The mechanism this points at is the driver's own dispatch widening as the op set grows, but **no build here isolates it**, and the honest statement is that the movement is real, per pass, on the IR arm only, and unattributed.

**It is the item 4f should cost first**, because it is the only one on either list that grows with each further promotion rather than shrinking, and Phase 5's sends are the next thing to widen the op set.
Candidates nobody has measured: a jump table, a two-level op encoding, or splitting the hot arms out of the one `match`.

### The instrument checked itself, three times, and one check is sharper than anything the phase had

`5ab3028f` was measured in **three** independent sittings with different companion builds in each.
Every `instructions:u` ratio reproduces: `emptyloop` 1.09223 in all three, `varlookup` 0.91645 in all three, `arith` 0.98181 in all three, `alloc4c` 0.99682 / 0.99682 / 0.99683, `compound` 0.97227 / 0.97228 / 0.97226.
The same binary's `cycles:u` ratios move up to **0.75%** across them -- `alloc4c` 1.03507, 1.02845, 1.02733 is the widest and `compound` 1.04129, 1.04808, 1.04655 the next -- which is the resolution any cycle claim in this document has.

**The sharpest instance of the cross-build cycle artifact this phase has produced is in the shipped sitting, and the work is provably held constant.**
`strings` executes the same instructions on both builds -- ratio 1.00504 on each, per-pass gap 144.00 on each, to the whole instruction -- and its **cycle ratio reads 1.01994 on `5ab3028f` and 0.99944 on the shipped build**, 2.05 points apart, in one sitting, interleaved, five rounds.
Nothing in the shipped build can reach `strings`: it compiles no arithmetic op, so the changed slot type is not in its stream, and the flipped default is overridden per run.
`emptyloop` says the same thing more quietly: instructions identical to the digit on both builds, cycles 1.01066 against 1.00294.
**So a cross-build cycle-ratio difference of about two points is available with the executed instruction stream held identical.**
That is not a stronger version of the byte-identical-`.text` pair the spec cites; it is the complementary one.
That pair holds the *code* constant and bounds repeated runs of one binary. This pair holds the *work* constant across two binaries that genuinely differ, which is the case criterion 4's "comparable within a build and not across builds" rule is actually about, and it had no direct demonstration before now.

## Criterion 5: the patch table has a working consumer, and the win reverts

**Met, and the interesting half is that Task 9 predicted the opposite.**

Deleting the table is one edit -- `QUICKENING = false` -- and it moves the number.
IR arm, `instructions:u` per pass, head against the deleted-table build, in one sitting:

| axis | head | table deleted | the table is worth |
|---|---:|---:|---:|
| `arith` | 60235.35 | 60393.10 | **157.75** |
| `compound` | 15129.01 | 15177.74 | **48.73** |
| `alloc4c` | 16028.72 | 16044.83 | 16.11 |
| `varlookup` | 3518.00 | 3529.00 | 11.00 |
| `emptyloop` | 1658.00 | 1654.00 | -4.00 |

**`arith` and `compound` are the evidence and the other three are not.**
`emptyloop` compiles no arithmetic op at all, so its -4 is what this switch does to codegen and nothing else; `varlookup`'s +11 and `alloc4c`'s +16 are the same order as that artifact against much larger per-pass totals, so neither is evidence about the table.
`arith` and `compound` are an order of magnitude clear of it, and both are axes holding an arithmetic site the small-integer path can never serve -- `i / 3` and `i // tails`.
That is exactly the mechanism Task 9 named: the hint's only power is to skip a hopeless check, so it pays where a site always falls through.

The tree-walker arm reads **62045.85** instructions per pass on `arith` in all four builds of that sitting, to two decimal places, which is the control saying the switch reaches only the compiled arm.

The other half of the falsification -- a test driving the quickened op with operands outside the small-integer range and asserting the general path's answer -- is `a_quickened_site_falls_through_to_the_general_path` in `ir/drive/tests.rs`, and it reads the fall-through count rather than the output, so it cannot be satisfied by an op that never quickens.

**With `QUICKENING = false` the whole suite is 1423 passed, 0 failed, 4 ignored, corpus 51 of 51, exit 0.**
Nothing in the tree runs that configuration, so this reading is a build rather than a standing check.

## Criterion 6: the minimum promotion set is native, and its ops do the work

**Met in part, and the missing part is an instrument.**

The set is native: `Do`/`Loop`, `If`/`Select`, `Assignment`/`Say`, variable access, arithmetic with its quickening consumer, and both call forms all compile to ops of their own, each with golden op-stream tests in `ir/golden_tests.rs`.

**The second clause is met, and the thing that meets it is the measurement rather than a test.**

The counting tests in `ir/drive/tests.rs` are worth less here than they look, and saying so is the point of this paragraph.
`clause_op_entries` separates "the body's clauses reach the compiled stream" from "they do not", each with a negative control -- but the degenerate implementation the spec describes *does* reach the compiled stream, and hands the clause straight back.
So that counter would read the same under it.
`trace_op_echoes` is narrower and does bite: a trace line emitted by the op is a line the delegate would have emitted from `eval.rs` instead, and `a_body_entered_under_trace_r_echoes_its_promoted_clause_from_the_chunk` counts it with `no_trace_op_echoes_without_the_engine_or_without_the_setting` as its control.

**What actually falsifies the degenerate implementation is the instruction count.**
An op that calls the extracted body-runner does everything the tree-walker does *plus* the dispatch that reached it, so it cannot execute fewer instructions for the same work.
Four axes' IR arms execute fewer: `varlookup` by 320 instructions per pass, `arith` by 1810, `compound` by 430 and `alloc4c` by 51.
It is not available for `emptyloop` or `strings`, whose IR arms are still 140 and 144 instructions per pass more expensive, so for the constructs those two axes exercise the second clause rests on the golden streams and the trace counter alone.

**The first clause is not in the tree as the spec words it.**
There is no assertion over the compiled op stream of *every corpus program*.
What exists is a per-construct golden set, `every_instruction_of_an_all_generic_body_compiles_to_one_generic_op` as its negative control, and a corpus-wide assertion that `chunks_refused` is zero on both arms -- which says every body compiled, not what it compiled to.
So a promotion that silently stopped firing for a construct the golden set does not cover would leave every gate here green.
**Building that assertion is Phase 4f's to take**, and it is cheap: the serialiser it needs is `ir/golden.rs`, and the population it needs is `corpus_cases()`.

*What it could not see:* whether Phase 5's dispatch is fast, which is Phase 5's gate.

## Criterion 7: the trace oracle passes

**Met, and the second half of the falsification holds only under a build nothing re-runs.**

The trace oracle harness passes in all four suite combinations, and it now runs on the IR arm: it builds its invocation from `Invocation::none`, which this task flipped.
`ir_dual.rs` diffs raw stderr between the two arms across every population, so indent agreement *between the arms* is asserted there rather than normalised away.

**The `run.rs` unit tests asserting exact stderr are the ones the spec flags, and they were tree-walker-only.**
Measured rather than inferred, with the pair that makes it a measurement: a probe on `run.rs`'s own `run_source` helper reads **1 chunk driven and 2 clauses stepped from it** with `Interp::new` set to `Engine::Ir`, and **0 and 0** with it left at `Engine::TreeWalker`.
So the constructor's value really does decide which engine those tests exercise, and today it is the tree-walker.

With `Interp::new` set to `Engine::Ir` the whole suite is **1423 passed, 0 failed, 4 ignored**, corpus 51 of 51, exit 0.
That discharges "those unit tests running under both engines" -- but by a build taken once, not by a mechanism in the tree, and a configuration nothing runs rots silently.
**The hole is named rather than closed**, and it is the same shape as the two compile-time switches below.

*What it could not see:* the settings the suite never exercises, and trace indent against the *oracle* on the IR arm, which `tests/support/mod.rs` normalises away for every harness that talks to the oracle.

## The predictions, checked against what was measured

A prediction that failed is worth more than one that held, so the failures are first.

### Task 9's predictions: five held, two failed

* **`varlookup` should move, and the plan's earlier text said it would not.** **Held**, and it is the largest move in the phase: the ratio goes 0.98666 to 0.92052 and the IR arm's per-pass cost falls 3780 to 3529 while its tree-walker arm moves 3831 to 3833. The mechanism named -- removing the `eval` recursion around the operands of `x = x + 1` -- is where the instructions went.
* **`compound` should move, by less than 4d-1's -51.6% and by a different mechanism.** **Held**: 1.00821 to 0.97379, with the IR arm falling 15673.50 to 15146.83 per pass.
* **`arith` should move, and an `arith` that does not move is the surprise.** **Held**: 1.00353 to 0.98315, the IR arm falling 62058.75 to 60303.68 -- 1755 instructions per pass, the largest absolute saving on any axis. The plan's older sentence, that an unmoved `arith` would be this task succeeding, is refuted by the build.
* **`alloc4c`, `startup` and `strings` should not move.** **Held.** Both arms of `alloc4c` move together (+28 and +27 per pass) so its gap is flat; `strings`' gap goes 172 to 168; `startup`'s ratio moves inside its own spread.
* **`emptyloop` should not move.** **Failed.** Its gap goes +119 to +132 per pass, entirely in the IR arm (1634 to 1647) with the tree-walker arm at 1515 in both. `emptyloop` compiles no arithmetic op, so this is not the promotion; it is the second residual above, and no build here names its mechanism.
* **The patch table should be at or below noise on every axis, and the win should survive `QUICKENING = false`.** **Failed, on both halves.** The table is worth 157.75 instructions per pass on `arith` and 48.73 on `compound`. The prediction's *mechanism* was right -- it pays where a site always falls through -- and its magnitude claim was wrong, which is the shape a right reason with a wrong number takes.
* **The `AtomicU32` price should be zero or near it.** **Held**: 8 instructions per pass on `arith`, 2 on `compound`, 1 on `varlookup`, 0 on `emptyloop`.

### Task 10's predictions: the flagged risk is what happened

* **`emptyloop`, `varlookup`, `arith`, `compound` and `startup`: no movement, either arm, either instrument.** **Failed on four of the five**, on `instructions:u` with disjoint five-round ranges: `emptyloop` 1.08713 to 1.09223, `varlookup` 0.92052 to 0.91645, `arith` 0.98315 to 0.98181, `compound` 0.97379 to 0.97227. `startup` held. **The moves are small and they do not point one way**: every tree-walker arm got slower by 3 to 15 instructions per pass, while the IR arms split -- `emptyloop` +11, `varlookup` -11, `compound` -17.9, `arith` -68.3. None of these axes executes a call of either kind, so none of it is the call work, and nothing here names what it is.
* **`strings` and `alloc4c`: flat or very slightly faster on both arms, with the ratio unchanged.** **Failed in sign.** Both arms of both axes got slower: `strings` +135 per pass on the tree-walker arm and +111 on the IR arm, `alloc4c` +43 and +29.
* **"The axis I would expect to go the wrong way first, if one does, is `strings`, and the mechanism is the extra call boundary."** **Held, and it is the most useful prediction in the phase.** `strings` is the largest absolute mover, it moved the wrong way, and it makes four builtin calls per pass: 135 instructions over four calls is **33.75 per builtin call** on the tree-walker arm. `alloc4c` makes one call per pass and paid 43, which does not reconcile to the same per-call constant, so the call boundary is not the whole of what moved.
* **The resolution cache executes zero times on every axis.** **Held, and measured rather than assumed.** With `CALL_SITE_CACHE = false` the IR arm reads **exactly** its head figure on `emptyloop` (1658.00), `arith` (60235.35), `varlookup` (3518.00) and `compound` (15129.01), and 16028.88 against 16028.72 on `alloc4c`. So no part of Task 10's movement is attributable to the cache in either direction, which is what the brief required and what nothing had checked.

## The `Sync` decision: `PatchSlot` becomes a `Cell<u32>`

**Decided and landed, rather than left in place by default.**

The bet the `AtomicU32` was making cannot be collected, and Task 10 is what voided it.
`assert_sync::<Chunk>()` fails at head naming **exactly one** culprit, `Cell<Option<Resolved>>` in `CallSite`, so a `Chunk` is not `Sync` whatever `PatchSlot` is.
Two further reasons stack on that: the cache hands out `Rc<Chunk>`, and an `Rc` is neither `Send` nor `Sync` whatever it holds; and a `Resolved` is two `usize`s plus a tag, too wide for a lock-free atomic here, so `CallSite` cannot follow `PatchSlot` even if someone wanted it to.

**The price was measured before the decision, not after it**: 8 instructions per pass on `arith`, 2 on `compound`, 1 on `varlookup` and 0 on `emptyloop`, IR arm, two builds of one sitting.
Small, but paid for a property nothing can observe, so the slot is now a `Cell<u32>` and the type's doc comment says what it costs to change back.
`CallSite`'s own doc carried the sentence "an atomic costs it nothing", which the measurement falsifies; it is corrected rather than left standing.

**What a later phase has to redo either way.** If a chunk ever crosses a thread, `CallSite`, the `Rc` and the slot all move together, and D22's schema is revisited whole. Nothing here is load-bearing for that; it is one field made consistent with its neighbour.

## The two compile-time switches, and the third configuration

Three configurations were run that nothing in the tree runs, and each is a build rather than a standing check.

| configuration | suite | corpus |
|---|---|---|
| `QUICKENING = false` | 1423 passed, 0 failed, 4 ignored, exit 0 | 51 of 51 |
| `CALL_SITE_CACHE = false` | 1423 passed, 0 failed, 4 ignored, exit 0 | 51 of 51 |
| `Interp::new` on `Engine::Ir` | 1423 passed, 0 failed, 4 ignored, exit 0 | 51 of 51 |

The suite count does not move under either switch because `ir/drive/tests.rs` reads them -- `let expected = if QUICKENING { 4 } else { 0 }` and the same shape for the call cache -- so the two tests that would otherwise contradict a disabled table adapt instead.
That is the right design and it is also why a green run under the switch says less than it looks: the tests that *depend* on the switch are the ones that change what they expect.

**All three rot silently, and 4f inherits that.**
The cheapest fix for the first two is a `#[cfg]`-gated second run in the suite; for the third it is a dual-engine helper in `run.rs`'s test module, which is a larger change because `Interp::new` has well over a hundred callers.

## The enter/leave trade at final promotion coverage

**Recorded as far as a build can say, and the counterfactual is not available.**

The trade was landed on three axes, with `emptyloop` costed at 24 instructions per pass -- which was 7-M2's *spike* figure (+100 to +124), not the landed split's.
**The landed split costs `emptyloop` 19**, measured at its own boundary in this sitting (+100 to +119), and 7-M3's own report gives the same 19.
Measured across six axes here rather than three, the cost side is `emptyloop` alone and the gain side is every other axis: `arith` -322.0, `strings` -320.0, `alloc4c` -199.8, `varlookup` -132.0, `compound` -116.3 instructions per pass.
So the trade was better than the axes it was landed on showed, and slightly cheaper than the figure it was landed against.

**Whether it is better or worse *now* cannot be answered from this sitting, and saying otherwise would be the phase's worst failure repeated.**
The counterfactual is head with the split reverted, and `git revert --no-commit 7d9cfb9a` at `5ab3028f` **conflicts in `ir/drive.rs`**, which Tasks 9 and 10 rewrote around the split.
Resolving it by hand would be writing a fourth driver, not reverting a commit, and a measurement of that is a measurement of what I wrote.

What the sitting does say, in the direction the trade turns on: the population that pays -- an all-`Generic` body -- has got **21** instructions per pass more expensive since the split for reasons that are not the split, and the population that gains has grown, with `arith` and `compound` joining `varlookup` and `alloc4c` below 1.0.
**Both terms moved, and they push the answer opposite ways**, so the direction is not derivable from them.
The build that would settle it is a hand-written pre-split driver carrying the current op set, and it belongs to whoever revisits the second dispatch level.

## The cache check that costs one boolean

**Recorded, not built, per the brief.**

Task 10's review proposes a `cfg`-gated assertion that re-resolves at every call-site cache hit and compares against what the slot kept, turning each sweep program into a cache check.

**The reason it is worth building is structural rather than incremental.**
Both arms of the dual-engine sweep are this crate, so a resolution that is wrong in the same way on both arms is invisible to it, and the corpus differential sees only what reaches output.
A cache that returned a stale `Resolved` for a name whose meaning changed would therefore pass every gate in this document.
Nothing in this phase can close that by running more programs; only an instrument that looks at the cache can.

**What it costs and what to watch.** One boolean per hit under `cfg`, and a resolution per hit -- so the gated build is much slower and must never be the one measured. It also needs a decision this phase does not make: whether a hit that re-resolves *differently* is a defect or a legal invalidation, which is the question a send cache asks and this one is supposed not to.

## Phase 4f's input state

**This section supersedes `phase-4e-anchor.md`'s figures as the starting point for 4f**, and the two are not the same measurement.

The anchor is **tree-walker against the oracle, on wall clock**, taken at `1b3efeb1` before `Engine::Ir` existed.
This gate is **IR against tree-walker, on instructions and cycles, within one binary**.
Neither converts into the other, and combining them would be the cross-sitting comparison this phase's own method forbids.

* The anchor's oracle ratios stand as the last tree-walker-against-oracle figures and are **not** re-measured here. No oracle ran in any sitting behind this document.
* The head table above is what 4f starts from on the engine axis.
* `phase-4e-anchor.md`'s per-task sections (Task 4a, Task 4c, Task 7) keep their wall-clock figures, and every one of them is **retired as a magnitude** by the spec's own rule about cross-build claims under 10%.
* `rexxcps`: the oracle prints 16,929,196 clauses per second, the IR arm 2,288,610 and the tree-walker arm 2,351,589, in the same session from a fresh directory. Those are wall-clock figures and therefore 4f's instrument rather than this phase's; they are recorded so 4f has a starting point and not as a result of this gate. **One run each, no interleaving and no rounds**, so the two arms' 2.7 per cent difference says nothing.

**The item 4f should cost first** is the +22 instructions per pass an all-`Generic` body now pays across Tasks 8, 9 and 10 for promotions it never executes, because it is the only one on either residual list that grows with each further promotion rather than shrinking.

## What this gate did not evaluate

* **Five platforms.** The inherited gate names Linux and macOS, Global Constraints `:35` requires five, and no platform runs the Rust suite automatically. Everything here is one Linux host. A dual-engine gate doubles whatever manual process exists, and that is unchanged from the spec's own open question.
* **Wall clock.** Not an in-phase instrument by the spec's decision at `8e7ce246`. The `rexxcps` figures above are the exception and are labelled as 4f's.
* **The oracle, on anything but `rexxcps` and the corpus.** No benchmark sitting here ran the C++ interpreter.
* **Whether the IR is faster than the tree-walker in wall-clock terms**, which is what a reader will want from the head table and which the table does not say.
