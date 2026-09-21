# Queued spike: the op driver's control flow

Requested by Moritz, 2026-09-19, after the llvm-lines and DWARF pass on
`Interp::run_ops_from`: *"spike sorting out control flow. For example, we could
have one place where we return, instead of many, and a variable that holds an
optional error, essentially breaking time-of-raise from time-of-propagation, if
it's safe to do so."*

The "if it's safe to do so" is the whole of the work. Read the safety section
before the design section.

## What the measurements say, and what they do not

Measured 2026-09-18/19 at `f9ffe9a8b` on the shipped binary (release, `lto =
"fat"`, `codegen-units = 1`, `debug = true`).

Per instantiation of `run_ops_from`, from the post-LTO IR and the binary's own
symbol table:

| | `<true>` | `<false>` |
|---|---|---|
| machine code | 15,377 bytes | 15,432 bytes |
| basic blocks | 638 | 632 |
| `phi` nodes | 907 | 901 |
| `switch` | 22 | 22 |
| `alloca` surviving -O3 | 138 | 138 |

**The ratio that motivates the spike: roughly 2,940 machine instructions across
roughly 635 basic blocks is about 4.6 instructions per block, with about 900 phi
nodes.** A driver that branches every four or five instructions is spending its
text on control flow rather than on work.

**Where the text goes, by DWARF line attribution.** 36% of instructions in both
instantiations attribute to std/core rather than to project code: the `?` / `Try`
impl on `Result` (`core/src/result.rs:2176`, `:2192`, `:2177`) is 169
instructions, conversion shims (`core/src/convert/mod.rs:780`) 121, `Option`
(`option.rs:2141`) 67, slice bounds checks (`slice/index.rs:184`) 55. The largest
project cluster is `rexx-core/src/roots.rs:165`, `:166`, `:171` at 141, which is
register and root access. The hottest single project line is `drive.rs:512`, the
loop head `if pc >= stop {`, at 53.

**What this does not say.** None of it is a runtime measurement. Code size and
block count are not cycles, and this project has measured that dead code alone
moves cycles up to 4% per axis with `dispatch` spanning 5.9 pp, non-monotonic in
pad size. The spike's own figures must come from callgrind on the bench axes,
interleaved, each revision in its own `CARGO_TARGET_DIR`.

## Safety: what a raise's position is observable through

`?` guarantees that nothing after it runs. Replacing that with a deferred
`Option<Failure>` removes the guarantee, and in this interpreter the **position**
of a raise is observable, not just the fact of one. At minimum:

* the traceback's reported clause and line
* `SIGL`
* which clause a `SIGNAL ON` / `CALL ON` trap attributes the condition to, and
  `offer_to_trap`'s decision at `drive.rs:404`
* trace output and its ordering, including the `>>>` echo
* the `Loud` refusal paths, which move all three descriptors

So the two designs are not equally safe and must not be conflated:

**(a) Single exit by labelled break.** Replace `return Err(e)` and `?` with
`break 'ops Err(e)`. Control still transfers immediately; only the syntactic exit
point moves. Semantically identical **by construction**, and it is the design the
spike should try first.

**(b) Deferred propagation.** Set `err = Some(e)` and continue to a single exit.
This changes when propagation happens and is only sound where the intervening
region is provably a no-op. It is not a refactor, it is a semantic change, and it
carries a proof obligation **per site**, not once for the function.

A third, cheaper than either:

**(c) Hoist the repeated fallible lookups.** Several arms re-derive the same
chunk and instruction lookups the region already performed. Fewer `?` sites
because there are fewer fallible calls, with no change to when anything
propagates.

And one already recorded elsewhere:

**(d) The bounds-check panic paths.** `slice/index.rs:184` is 55 instructions
here, and asserting the bound before indexing is recorded in project memory as
worth several percent on `Index`'s panic path.

## How the spike is judged

The instruments already exist and were exercised on this function:

1. `cargo llvm-lines` per crate, and the function's own entry.
2. `llvm-nm --print-size` on the shipped binary for each affected function.
3. Post-LTO IR block count, `phi` count, `switch` count and shape, `alloca`
   count.
4. callgrind on the bench axes, interleaved, plus the oracle differential.

**A control-flow change must move instruments 3 and 2. If block and phi counts
fall materially and callgrind does not move, control flow was not the cost** --
which is a result worth having, and is the outcome the spike should be willing to
report.

The corpus differential and the gate tables are the safety arbiter: any change
that moves where a condition is reported shows up as a divergence rather than as
a judgement call. Run them, do not reason about them.

## Do not start from a clean sheet

`drive.rs` carries a measurement comment immediately above `fn run_ops`, reading
"Reproduced across two sittings and two builds, with branch misses at 70,000 out
of 6.3 billion branches either way, so it is not prediction. No mechanism below
'the layout changed' was found, and a change with that profile is not worth 7
instructions."

**That comment names no subject.** Recovered from `git log -S`: it belongs to
commit `132c33955`, whose message says *"Bundling the range-invariant arguments
into a struct is the variant that looks best on instruction counts and is
rejected on cycles; run_ops carries the numbers so the next reader does not
repeat it."* So the comment is about **bundling the range-invariant arguments
into a struct**, a variant already tried and rejected on cycles. As it stands the
comment reads as though it might be about anything nearby, and it nearly caused
the `GRANTING` A/B to be skipped as already done.

Two consequences. The struct-bundling variant is **out of scope**: it has been
measured and rejected. And the comment needs its subject restored, which is a
separate one-line change, not this spike's to make silently.

## Measured before this spike was dispatched, 2026-09-19, and it moves the target

### L1i misses are real, and concentrated

`perf stat`, three interleaved rounds per axis, baseline binary at `f9ffe9a8b`:

| axis | L1i misses per run | miss rate |
|---|---|---|
| `dispatchclass` | 71,526,068 | 14.17% |
| `decrender` | 37,668,996 | 20.35% |
| `varlookup` | 769,393 | 13.23% |
| `emptyloop` | 688,709 | 10.30% |

The rate is high everywhere; the **absolute count** is what separates the axes.
`dispatchclass` and `decrender` miss by tens of millions, `emptyloop` and
`varlookup` by well under a million. The tight loops fit in L1i. The dispatch and
decimal-render paths do not, and that is where this spike's win has to come from.

### The `GRANTING` A/B, and a correction to how it was described

Merging the two instantiations into one function taking a runtime `bool`:

| axis | cycles | L1i misses | Ir (callgrind) |
|---|---|---|---|
| `dispatchclass` | **-3.83%** | **-47.65%** | -0.08% |
| `decrender` | +1.57% | +14.07% | +0.64% |
| `emptyloop` | +2.03% | -9.00% | +0.25% |
| `varlookup` | +5.93% | -20.50% | +4.00% |

Static: the pair is 15,432 + 15,377 = 30,809 bytes; merged, 21,026. Binary
`.text` falls 3,369,325 to 3,351,305.

**The correction.** An earlier reading of this function described `GRANTING` as
specialising a bool "read once and overwritten immediately". That is wrong and
the A/B is what exposed it. `if granting` sits inside the `Op::Clause` arm of the
`'ops:` loop, so it runs **per clause**; and the only write to `granting` is
inside the block that `if granting` guards, so **once false it can never become
true**. With `GRANTING = false` the whole permission block is provably dead and
folds away. That is what `varlookup`'s +4.00% Ir is: the merge reinstating a
per-clause branch and a call that specialisation had deleted outright.

**So the const generic earns its instructions.** The question this spike inherits
is not whether to remove it. It is that ~9.8 KB of duplicated driver is being paid
to keep one per-clause branch folded, and on the axis that is actually I-cache
bound that trade loses by 3.83% of cycles.

### What the spike should now be trying to get

Both halves at once: **keep the specialisation's dead-code elimination, stop
duplicating the arms that are cold.** Concretely, shrink what the hot loop keeps
resident -- `#[cold]` or `#[inline(never)]` on rare op arms, or moving infrequent
handlers out of the driver body -- so that whatever remains duplicated is only
what a hot loop executes. The single-exit and error-deferral designs in the
sections above are worth trying on the same footing and judged by the same
instruments; they reduce block count and epilogue duplication, which is footprint
by another route.

This also reopens a ruling made 2026-09-17 against CREXX's hand-tiered "handler
panel", which was rejected as a layout experiment with no control. The rejection
was reasoned on the wrong axis: 71.5M misses on `dispatchclass` is not layout
noise. The hand-maintained table is still not worth copying; the constraint it
addresses is real.

### The measurement protocol that worked, and its one counterexample

Use all three, because each is blind to something the others see:

* `callgrind --tool=callgrind` for Ir. Deterministic here to five significant
  figures across rounds, and **blind to every I-cache effect**.
* `perf stat -e instructions,cycles,l1-icache-loads,l1-icache-load-misses`, three
  interleaved rounds, base and variant adjacent inside each round. Check the
  multiplex column rather than assuming the counters were not shared.
* `llvm-nm --print-size` on the shipped binary per affected function, and
  `llvm-size` for `.text`.

**The counterexample that must be respected.** The same variant binary that halved
`dispatchclass`'s misses made `decrender`'s **worse by 14%**. Same footprint
reduction, opposite sign, so placement and alignment are doing at least as much
work as size. A footprint argument therefore needs a padding control before it is
believed, exactly as the project's own layout-attribution result requires.

## Spike result, 2026-09-19: rejected, and the control retired a published number

Report: `.superpowers/sdd/2026-09-19-driver-control-flow-spike-report.md`. Nothing
landed; tree clean at `f9ffe9a8b`.

**The padding control is the finding.** Four builds with the driver byte-identical
(`llvm-nm` confirms `0x3c11` and `0x3c48` in all four, only the address moving),
padded with dead `#[used]` functions, on `dispatchclass`:

| build | hardware L1i | cycles | Ir |
|---|---|---|---|
| pad4 | -56.6 / -54.5% | -5.07 / -4.51% | +0.2% |
| pad12 | -14.1 / -12.2% | -2.88 / -2.68% | -0.2% |
| pad24 | +38.3 / +37.3% | +4.98 / +5.19% | +0.1% |
| pad48 | +66.8 / +65.6% | +6.38 / +4.81% | +0.0% |

A 123-point swing in L1i and an 11-point swing in cycles **from dead code alone**,
non-monotonic in pad size, with `Ir` flat throughout. That flat `Ir` column is the
control working: nothing about the executed path changed.

**This retires the `GRANTING` merge's headline.** The -3.83% cycles and -47.65%
L1i reported for it on `dispatchclass` sit inside that envelope and were never
attributable. Its attributable half is the **+4.00% `Ir` on `varlookup`**, which
is a cost. **The const generic keeps its instructions and stays.**

**Conflict against capacity, which is the sharpest result here.** From cachegrind
at two sizes and two associativities, marginal misses per iteration:
`dispatchclass` has a 24.94 KiB steady-state footprint and goes to **zero** misses
at 32 KiB 64-way, so its misses are essentially all **conflict**. `decrender` is
**capacity**-bound, 35.81 KiB against 32. `varlookup` and `emptyloop` have no
steady-state I-cache problem at all: their misses are startup and do not grow
with `n`.

So on the axis that misses most, shrinking the hot loop was never the lever.
Placement is, and placement is what this project has no reliable way to steer.

**V1**, cold arms moved to one `#[inline(never)]` non-generic helper, was rejected
on retired instructions: `+2.80%` `decrender`, `+3.70%` `varlookup`, `+2.53%`
`emptyloop`, with 64-way I1 unmoved at +0.16%. `.text` went **up** 7,328 bytes,
because the extracted arms cost 9,965 standalone against roughly 1,900 inlined in
each instantiation.

**(d) bounds checks is the one item not rejected**, and it is now the largest
measured candidate in this area. `slice/index.rs` inside the driver is 45 / 169 /
43 / 13 retired instructions per iteration across the four axes: **4.68% of
`varlookup`, 3.29% of `emptyloop`**, 1.36% `decrender`, 1.03% `dispatchclass`.
That is in retired instructions, the currency the padding control does not wash
out.

**(b) and (c) were rejected on executed cost, not on principle.** The whole
`?`/`Try` machinery is 0.70-2.67% of the driver's *executed* instructions and
`core/src/result.rs` is at most 0.44% of any axis, so the brief's 169 static `Try`
instructions are almost never executed. A static instruction count was the wrong
instrument for choosing among these, which is worth remembering the next time a
`cargo llvm-lines` table suggests a target.

### The hot set, which is why the rejections were foregone

Per-iteration steady-state hot set by owning function, from marginal
`--dump-instr=yes` dumps. **`run_ops_from` is 13.8% of it on `dispatchclass` and
12.4% on `decrender`, and no single function exceeds about a fifth.**

`dispatchclass`, 399 lines / 24.94 KiB: `run_ops_from::<true>` 55 lines,
`Interp::invoke` 53, `Interp::run_activation` 31, `Interp::eval_node` 23,
`Interp::message_term` 17, `Interp::flat_loop_step_top` 16, `Interp::loop_advance`
16, `MethodDict::slot` 11.

`decrender`, 573 lines / 35.81 KiB: `run_ops_from::<true>` 71 lines,
`Interp::loop_advance` 49, `Number::add_signed` 45, `Interp::flat_loop_start` 37,
`Number::format_with` 28, `Interp::apply_binary` 26, `builtin::run` 21,
`Interp::flat_loop_step_top` 18.

So the driver's control flow was never the lever, whatever the refactor had
measured. The two real levers are different from each other:

* `dispatchclass` is **conflict**-bound, and the lever is **placement**: function
  ordering, alignment, PGO, BOLT-style splitting. The padding table is the size of
  the prize, 123 points of L1i and 11 of cycles, **and every change to this crate
  currently draws from it at random.**
* `decrender` is **capacity**-bound by 3.81 KiB, and the lever is **broad
  footprint reduction across the whole hot set**, which no one function can
  deliver: the driver's entire perfect-packing saving is 1.79 KiB, under half the
  gap.

### Left open, and stated as the spike stated it

* **Whether a placement pass is available in this toolchain at all.** No attempt
  was made at `-C llvm-args` ordering, a linker order file, PGO or BOLT. The
  padding table says the prize exists; nothing yet says it can be taken
  deliberately rather than drawn at random.
* **The `GRANTING` reclassification is an inference, not a re-measurement.** Its
  published figures fall inside a padding envelope measured on an axis shown to be
  conflict-bound. The test that would settle it is rebuilding that variant across
  several pad sizes and watching its advantage fail to persist. Not run. Nobody
  should read "retired" as "re-measured".
* **V1's correctness is unverified beyond the bench programs.** Byte-identical
  stdout, stderr and exit status against BASE over the fifteen
  `rust/bench-programs/` files, `heapshape` excepted for its wall-clock output.
  The corpus differential and the gate tables were **not** run, deliberately,
  since V1 was never proposed for landing. If the shape is revived, the gates are
  owed.
* `pad12` reports 53.00 marginal I1 misses per iteration on `dispatchclass` at
  64-way where BASE, pad4, pad24, pad48 and V1 all report 0.00. Unexplained;
  the standing guess is uneven splitting of hot instructions across line
  boundaries and sets.
* `macros.rs:0:28` is **6.4% of the driver's executed instructions on
  `decrender`** and was not chased to a macro.

## rexxcps, measured 2026-09-20, and it was missing from everything above

The spike and the `GRANTING` A/B both omitted `rexxcps` and used `decrender` as a
stand-in, on the strength of `bench-programs/README.md` calling it "rexxcps' inner-loop
shape". A pinned, deterministic copy exists for exactly this purpose at
`rust/bench-rexxcps/rexxcps.rex`, with its own README; it was not used.

### Hardware, three interleaved rounds, load ~1

| | baseline | `GRANTING` variant |
|---|---|---|
| instructions | 21,174,094,626 | 21,254,058,991 (+0.38%) |
| cycles | 6,108,539,539 | 6,207,711,252 (+1.62%) |
| L1i loads | 1,770,837,600 | 1,721,642,127 |
| L1i misses | **158,829,145** | 216,533,473 (+36.3%) |
| miss rate | 8.97% | 12.58% |

**`rexxcps` misses more than twice `dispatchclass` (71.5M) and more than four times
`decrender` (37.7M).** Its miss *rate* is the lowest of the four because it issues
1.77B icache loads; the rate flattered it and the absolute count is what costs.

The variant is worse on all three counters here, so on the target that matters the
merge is a plain loss.

### Conflict or capacity: capacity, decisively

`valgrind --tool=cachegrind --cache-sim=yes`, varying `--I1` only:

| geometry | I1 misses | vs 32K/8-way |
|---|---|---|
| 32K, 8-way | 724,354,711 | -- |
| 32K, **64-way** | 684,714,031 | **-5.5%** |
| 64K, 8-way | 365,921,150 | **-49.5%** |
| 128K, 8-way | 74,595,329 | **-89.7%** |

Associativity at unchanged capacity buys 5.5%. Doubling capacity halves the misses;
quadrupling removes nine tenths. **`rexxcps` is capacity-bound and placement is not
its lever**, which is the opposite of `dispatchclass` and the same direction as
`decrender`.

**But the magnitude is nothing like `decrender`'s.** That axis was 3.81 KiB over a
32 KiB cache. `rexxcps`'s hot set does not fit in 64 KiB either, and only largely
fits at 128 KiB. So the proxy got the class right and the size wrong by an order of
magnitude, and every footprint number reasoned from it understates the gap.

**Simulation does not transfer to absolute hardware counts.** Cachegrind reports
724M misses at 32K/8-way where hardware reports 158.8M, a factor of 4.6. The
simulator models no op cache, and this CPU has one. Use these rows for the
conflict-versus-capacity classification and the ratios between geometries, never as
predicted hardware counts.

### What this changes

The spike concluded that the capacity-bound lever is broad footprint reduction across
the whole hot set, which no single function delivers. That conclusion survives and
gets sharper: for `rexxcps` the gap to close is tens of KiB, not the 1.79 KiB the
driver's perfect packing could ever have saved. V1-style outlining was directionally
right for this axis and numerically irrelevant to it.
