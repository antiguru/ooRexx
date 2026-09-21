# Spike report: the op driver's control flow

Run 2026-09-19 against BASE `f9ffe9a8b`. **Nothing is proposed for landing. The
tree is back at BASE and `git status` is clean.**

**Recommendation in one line: reject the control-flow refactor of
`Interp::run_ops_from` -- it costs 2.5%-3.7% of retired instructions on three of
the four axes and buys only an I-cache movement that a padding control
reproduces from dead code alone, and the same control retires the `GRANTING`
merge's -3.83% on `dispatchclass` as unattributable.**

## Environment, because two figures depend on it

* AMD RYZEN AI MAX+ 395 (`/proc/cpuinfo`), 32 cores. **L1i is 32768 B, 64 B
  line, 8-way**, as cachegrind autodetects it: `desc: I1 cache: 32768 B, 64 B,
  8-way associative` in any `--cachegrind-out-file`.
* **`perf` had exactly one hardware counter slot for the whole sitting.**
  `perf stat -e instructions,cycles /bin/true` counts `instructions` and reports
  `<not counted>` for `cycles`; each of `cycles`, `instructions`,
  `l1-icache-loads`, `l1-icache-load-misses` counts alone at 100%. So every
  `perf` figure below is **one event per process invocation**, not four events
  in one run. This is the sandbox boundary already recorded in
  `rust/bench-baselines/README.md`, and it is a property of the moment, not of
  the machine.
* Load average is quoted beside each `perf` sitting. Both `perf` sittings were
  interleaved across builds inside each (event, axis) group.
* Scaled copies of the bench programs were used for every valgrind run, in two
  sizes per axis, so that **a marginal figure (size b minus size a, divided by
  the extra iterations) removes startup**. Sizes: `dispatchclass` 200k/400k,
  `decrender` 20k/40k, `varlookup` 1M/2M, `emptyloop` 1M/2M for the cache runs;
  40k/80k, 4k/8k, 200k/400k, 200k/400k for the instruction-address dumps. The
  `perf` runs used the committed bench programs at their own sizes.

## What the I-cache misses actually are

Before any variant. Marginal I1 misses per iteration, from

```
valgrind --tool=cachegrind --cache-sim=yes --I1=<geometry> --D1=49152,12,64 \
  --cachegrind-out-file=/dev/null rexx-run <axis>-<size>.rex
```

| axis | steady-state instruction footprint | I1 miss/iter @32K/**8**-way | @32K/**64**-way | @64K/8-way |
|---|---|---|---|---|
| `dispatchclass` | 399 lines, 24.94 KiB | **137.00** | **0.00** | 9.00 |
| `decrender` | 573 lines, 35.81 KiB | **345.82** | **331.70** | 5.01 |
| `varlookup` | 84 lines, 5.25 KiB | 0.00 | -- | -- |
| `emptyloop` | 53 lines, 3.31 KiB | 0.00 | -- | -- |

The footprint column is the set of 64-byte lines whose marginal Ir is at least
one execution per iteration, from `--dump-instr=yes` dumps at the two sizes.

Three things follow, and they are the frame for everything below.

**`varlookup` and `emptyloop` have no steady-state I-cache problem at all.**
Their marginal miss rate is zero to five decimal places; the ~177,000 misses
each reports is startup, and it does not grow with `n`. Any I-cache argument
about those two axes is about startup.

**`dispatchclass`'s misses are conflict misses, not capacity misses.** Its
footprint, 24.94 KiB, fits inside a 32 KiB L1i. Holding capacity at 32 KiB and
raising associativity to 64-way takes the marginal misses from 137.00 to 0.00.
Raising capacity to 64 KiB at 8-way only takes them to 9.00. So what costs
`dispatchclass` 71-79 million L1i misses is **which set** its hot lines land in,
which is address placement.

**`decrender`'s misses are capacity misses.** 64-way at the same capacity leaves
them at 331.70, essentially unchanged; 64 KiB at 8-way collapses them to 5.01.
Its footprint is 35.81 KiB against a 32 KiB L1i, so it is 3.81 KiB over.

## The ceiling on any footprint change to the driver, computed before building one

`run_ops_from::<true>`'s own contribution to the steady-state footprint, and
what a *perfect* hot/cold split of it could recover:

| axis | driver's resident lines | executed bytes in them | packing | ideal lines | best possible saving | as a share of the axis footprint |
|---|---|---|---|---|---|---|
| `dispatchclass` | 56 (3,584 B) | 2,279 | 64% | 36 | 20 lines, 1,280 B | 24.94 -> 23.69 KiB, **-5.0%** |
| `decrender` | 71 (4,544 B) | 2,715 | 60% | 43 | 28 lines, 1,792 B | 35.81 -> 34.06 KiB, **-4.9%** |
| `varlookup` | 41 (2,624 B) | 1,488 | 57% | 24 | 17 lines | |
| `emptyloop` | 20 (1,280 B) | 755 | 59% | 12 | 8 lines | |

And of the 1,282 unexecuted bytes sitting inside `dispatchclass`'s 56 resident
lines, the arms that are cold **by nature** own about 180 (`Queue` 52,
`TraceClause` 31, `SelectCaseText` 28, `Message` 27, `Escape` 23,
`TraceArgument` 19); on `decrender` about 230 of 1,835. The rest is the untaken
side of hot arms, the region-loop's own exit, and code with no line attribution
at all (1,103 B of the function has no `.debug_line` entry: spills, jump tables,
block padding).

So before a line was edited: **a cold-arm extraction can recover of order 200
bytes of resident footprint; a perfect split of the whole function can recover
about 5% of the axis footprint; and `decrender` needs 10.6% to fit in L1i.**

## The variant that was built, and what it did

**V1**: the region loop's by-nature-cold arms moved into one non-generic
`#[inline(never)] fn run_cold_region_op`, and the seven `op_not_driven` arms
collapsed into that function's `_` arm via the existing `undriven_op_name`.
Moved: the eight `Trace*` arms, `Signal`, `Parse`, `Queue`, `Message`, `Expose`,
`Exec`, `Escape`. `Call`, `CallExpr`, `CallNamed`, `WhenTest`, `Prefix` and
`Say` were left in place because they are hot on axes outside this four.

This is design **(a)** applied to a set of arms: control still transfers at the
same point, `break 'cold Err(e)` becomes `return Err(e)` inside the callee and
`break 'cold Err(failure)` at the call site. No raise moved. 381 insertions, 408
deletions, no comment line lost and none re-wrapped (checked by multiset
comparison of every `//` line against the BASE copy).

### Static

`llvm-size` and `llvm-nm --print-size --demangle`:

| | BASE | V1 |
|---|---|---|
| `.text` | 3,369,325 | 3,376,653 (**+7,328**) |
| `run_ops_from::<true>` | 15,377 | inlined into `run_activation` |
| `run_ops_from::<false>` | 15,432 | 12,702 |
| `run_cold_region_op` | -- | 9,965 |
| `run_activation` | 7,776 | 21,389 |

The BASE row reproduces the brief exactly (15,377 / 15,432 / 3,369,325).

**`.text` went up, not down.** The extracted arms measured about 1,900 bytes
inside each instantiation and cost 9,965 as a function of their own: outlined,
their `Loud::*` construction and error tails can no longer be tail-merged with
the driver's. And the smaller driver crossed an inlining threshold, so
`<true>` disappeared into `run_activation`.

### Dynamic, deterministic (valgrind)

Marginal per iteration, BASE -> V1:

| axis | Ir | I1 miss @32K/8-way | I1 miss @32K/**64**-way | steady-state footprint |
|---|---|---|---|---|
| `dispatchclass` | 4376.01 -> 4351.96 (**-0.55%**) | 137.00 -> 57.00 (**-58.4%**) | 0.00 -> 0.00 | 399 -> 391 lines |
| `decrender` | 12464.70 -> 12813.53 (**+2.80%**) | 345.82 -> 405.51 (**+17.3%**) | 331.70 -> 332.23 (**+0.16%**) | 573 -> **588** lines |
| `varlookup` | 919.00 -> 952.99 (**+3.70%**) | 0.00 -> 0.00 | -- | 84 -> **91** lines |
| `emptyloop` | 394.99 -> 405.00 (**+2.53%**) | 0.00 -> 0.00 | -- | 53 -> **54** lines |

The 64-way column is the footprint instrument: it holds capacity fixed and
removes conflict. **On `decrender`, the only capacity-bound axis, it moved by
0.16%.** The change did not reduce the footprint. The direct footprint count
agrees and is worse than that: V1 *grew* the steady-state footprint on three of
four axes, because the call sequence and the callee's prologue are themselves
resident and the remaining driver did not repack.

`varlookup` and `emptyloop` execute **none** of the moved arms and still retire
3.70% and 2.53% more instructions. That is the driver's own codegen changing,
and it is the only part of V1 that is attributable.

### Dynamic, hardware (`perf`)

Three sittings, all interleaved base-adjacent inside each (event, axis) group.
**A** is BASE and V1 only, three rounds, load average 19.17 falling to 12.38.
**B** is all six builds, three rounds, load 10.96 falling to 2.69. **C** is all
six builds, five rounds, cycles and L1i only, load 1.90 to 2.14. All three are
quoted because they agree; the high-load sitting is kept rather than discarded
for exactly that reason.

```
perf stat -x, -e <event> <binary> bench-programs/<axis>.rex
```

| axis | cycles (A / B / C) | l1-icache-load-misses (A / B / C) | instructions (A / B) |
|---|---|---|---|
| `dispatchclass` | -2.61% / -1.87% / -2.66% | -43.8% / -36.7% / -39.2% | **-0.21% / -0.21%** |
| `decrender` | +4.86% / +6.77% / +7.34% | +49.9% / +40.6% / +44.2% | **+2.70% / +2.72%** |
| `varlookup` | +5.18% / +4.42% / +4.69% | +7.5% / +7.4% / +6.5% | **+3.67% / +3.67%** |
| `emptyloop` | +1.60% / +2.48% / +4.10% | +8.0% / +7.9% / +5.7% | **+2.50% / +2.49%** |

The `instructions` column reproduces callgrind to within 0.2 pp on three axes,
which is the cross-check that the two instruments are measuring the same thing.
BASE's own spread across rounds was 0.00%-0.06% on `instructions` and
1.08%-3.60% on `cycles`.

## The padding control: run, and it is the finding

Four control builds. Each takes the **unmodified** BASE `drive.rs` and inserts
dead `#[inline(never)]` arithmetic functions, kept by a `#[used]` static of
function pointers, immediately before the `impl Interp` block. `llvm-nm`
confirms the driver is byte-identical in every one -- `0x3c11` and `0x3c48` in
all four, with only its address moving (`0x139f90` -> `0x13a1a0` -> `0x13a5c0`
-> `0x13abf0` -> `0x13b850`).

Hardware columns are sittings **B / C**, the two that contain the pads.

| build | `.text` | Ir/iter `dispatchclass` | I1 @8-way `dispatchclass` | hardware L1i `dispatchclass` | cycles `dispatchclass` | I1 @8-way `decrender` |
|---|---|---|---|---|---|---|
| BASE | 3,369,325 | 4376.01 | 137.00 | 0.0% | 0.0% | 345.82 |
| pad4 | 3,369,853 | +0.2% | 97.00 (**-29.2%**) | **-56.6% / -54.5%** | -5.07% / -4.51% | -0.2% |
| pad12 | 3,370,909 | -0.2% | 84.00 (**-38.7%**) | **-14.1% / -12.2%** | -2.88% / -2.68% | +13.2% |
| pad24 | 3,372,493 | +0.1% | 118.00 (**-13.9%**) | **+38.3% / +37.3%** | +4.98% / +5.19% | +7.0% |
| pad48 | 3,375,661 | +0.0% | 114.00 (**-16.8%**) | **+66.8% / +65.6%** | +6.38% / +4.81% | +0.7% |
| V1 | 3,376,653 | -0.5% | 57.00 (**-58.4%**) | **-36.7% / -39.2%** | -1.87% / -2.66% | +17.3% |

**What the control says.** With the driver's machine code byte-identical, dead
padding alone moves `dispatchclass`'s hardware L1i misses across a 123
percentage-point range, from -56.6% to +66.8%, **non-monotonically in pad size**,
and its cycles from -5.07% to +6.38% against a BASE round-to-round spread of
2.91%. The same padding moves `Ir` by at most 0.2%, which is the control working:
the code did not change, so the instruction count did not.

V1's -36.7% L1i and -1.87% cycles on `dispatchclass` sit inside that envelope.
**They are not attributable to V1 having removed anything.** The same holds for
`decrender`'s +17.3%: padding alone spans -0.2% to +13.2% there.

**And it reaches backwards.** The brief's motivating figure -- the `GRANTING`
merge at -3.83% cycles and -47.65% L1i on `dispatchclass` -- is inside this same
envelope and on the axis whose misses are 100% conflict. It should be reclassified
as unattributable placement. The attributable half of that A/B is the part the
brief already named: **+4.00% Ir on `varlookup`**, which is the per-clause branch
and call that specialisation had deleted. So **the `GRANTING` const generic earns
its instructions and the argument for removing it was a layout artifact.**

## Designs, and the number each was rejected on

**(a) Single exit by labelled break.** *Already in the tree, and then tried
again as V1.* The region loop's fallible sites are already
`break 'cold Err(...)` rather than `?`. The `?` sites that remain are in the
outer `'ops` loop (`leave_ended_select_branch`, `count_clause_against_deadline`,
`debug_pause_after_clause`, `leave_stepped_clause`, `end_promoted_branch`,
`when_frame`, `otherwise_frame`, `flat_loop_step_top`, `settle`), where a `?`
returns straight out of the function and is already what a single exit would
be. The executed cost of what `?` remains is priced under (c). Extending the same shape to the cold arms is V1: **+2.80% /
+3.70% / +2.53% Ir on three axes, 64-way I1 unmoved at +0.16%, steady-state
footprint up on three of four axes.** Rejected.

**(b) Deferred propagation into an `Option<Failure>`.** Not attempted, and not
because of its proof obligation. It is a *control-flow* change, and the
measurement above prices control flow: the whole `?`/`Try` machinery inside the
driver is 0.70%-2.67% of the driver's own executed instructions, and the driver
is 13.4% of `dispatchclass`'s per-iteration instructions. There is no number
here worth taking a per-site obligation on the traceback clause, `SIGL`,
`offer_to_trap`'s attribution, trace ordering and the `Loud` refusal paths for.

**(c) Hoist the repeated fallible lookups.** Rejected on the executed share of
the `?` implementation. By innermost-line attribution of the marginal Ir inside
`run_ops_from::<true>`: `core/src/result.rs` is **13.00 Ir/iter on
`dispatchclass` (2.22% of the driver, 0.30% of the program), 22.00 on
`decrender` (0.82% / 0.18%), 4.00 on `varlookup` (0.70% / 0.44%), 4.00 on
`emptyloop` (2.67% / 1.01%)**. The brief's 169 static instructions of `Try` code
are almost entirely never executed. The ceiling on (c) is a fraction of 1% of
any axis.

**(d) The bounds-check paths.** *Not rejected -- it is the one candidate the
measurement supports, and it is not a control-flow change.* `core/src/slice/index.rs`
inside the driver is **45.00 Ir/iter on `dispatchclass` (7.69% of the driver,
1.03% of the program), 169.00 on `decrender` (6.33% / 1.36%), 43.00 on
`varlookup` (7.53% / 4.68%), 13.00 on `emptyloop` (8.67% / 3.29%)**. That is
larger than every control-flow item measured here, it is in instructions rather
than in cache behaviour, and instructions are the currency that moved
reproducibly in this whole sitting. Project memory already records asserting the
bound before indexing as worth several percent on `Index`'s panic path.

**Struct-bundling of the range-invariant arguments** was out of scope by the
brief (measured and rejected at `132c33955`) and was not touched.

## What this says the cost actually is

The per-iteration steady-state hot set, by owning function, from the marginal
`--dump-instr=yes` dumps. `run_ops_from` is 13.8% of it on `dispatchclass` and
12.4% on `decrender`; no single function is more than about a fifth.

`dispatchclass`, 399 lines / 24.94 KiB: `run_ops_from::<true>` 55 lines,
`Interp::invoke` 53, `Interp::run_activation` 31, `Interp::eval_node` 23,
`Interp::message_term` 17, `Interp::flat_loop_step_top` 16,
`Interp::loop_advance` 16, `MethodDict::slot` 11.

`decrender`, 573 lines / 35.81 KiB: `run_ops_from::<true>` 71 lines,
`Interp::loop_advance` 49, `Number::add_signed` 45, `Interp::flat_loop_start` 37,
`Number::format_with` 28, `Interp::apply_binary` 26, `builtin::run` 21,
`Interp::flat_loop_step_top` 18.

So the two levers the evidence points at are **not** the driver's control flow:

* `dispatchclass` is conflict-bound. The lever is **placement** -- function
  ordering, alignment, PGO or BOLT-style hot/cold splitting -- and the size of
  the prize is visible in the padding table: a 123-point swing in L1i misses and
  an 11-point swing in cycles is available from address placement alone, and
  every change to this crate is currently drawing from that distribution at
  random. A deliberate placement pass is the only thing here that could take the
  good tail on purpose. This also re-opens the 2026-09-17 ruling against CREXX's
  hand-tiered handler panel on the correct axis: the constraint is real, it is
  conflict rather than capacity, and a hand-maintained table is still not the
  answer to it.
* `decrender` is capacity-bound by 3.81 KiB. The lever is **broad footprint
  reduction across the whole hot set**, and the table above says one function
  cannot deliver it: the driver's entire perfect-packing saving is 1.79 KiB, less
  than half the gap.

## What I could not determine

* **Whether a placement pass is available in this toolchain.** I did not try
  `-C llvm-args` function ordering, a linker order file, PGO, or BOLT. The
  padding table says the prize exists; it says nothing about whether it can be
  taken deliberately.
* **Why `pad12` reports 53.00 marginal I1 misses per iteration on
  `dispatchclass` at 64-way**, where BASE, pad4, pad24, pad48 and V1 all report
  0.00. That geometry is eight 64-way sets holding 512 lines in total, so it
  should show capacity misses only, and BASE's footprint is 399 lines. The most
  likely explanation is that pad12's addresses split more hot instructions
  across line boundaries and land them unevenly across the eight sets, but I did
  not verify it.
* **Whether V1 is correct.** It was checked only by a byte-for-byte diff of
  stdout, stderr and exit status against BASE over the fifteen programs in
  `rust/bench-programs/` (all identical; `heapshape` prints wall-clock timings
  and differs BASE-against-BASE the same way). **The corpus differential and the
  gate tables were not run**, because V1 is not proposed for landing and a
  27-minute gate over a tree that has been reverted measures nothing. If anyone
  wants to revive this shape, the gates are still owed.
* **Whether the `GRANTING` merge would reproduce its own figures today.** I did
  not rebuild it; the reclassification above rests on its published numbers
  falling inside the padding envelope I measured, on the axis I showed to be
  conflict-bound, not on a re-run.
* **`macros.rs:0:28` is 6.4% of the driver's executed instructions on
  `decrender`** and I did not chase which macro it is.

## One thing the spike found and did not fix

The brief is right that the measurement comment above `fn run_ops` names no
subject, and it is still that way -- the brief calls restoring its subject a
separate one-line change and not this spike's to make silently, so it was not
made. It belongs to `132c33955` and is about bundling the range-invariant
arguments into a struct.

## Artifacts

Under this session's scratchpad at `.../scratchpad/cfspike/`: the six binaries
in `bins/` (BASE, four pads, V1), the raw `perf` records `perf-all.tsv` and
`perf-cycles-2.tsv`, the valgrind stderr per run under `m/`, the
instruction-address dumps under `cg/`, and the analysis scripts. Every
`CARGO_TARGET_DIR` was deleted by explicit path once its binary was copied out.
The only in-repository artifact of the spike is this file.
