# How much of the assembly is bounds checking, measured 2026-09-21

Asked by Moritz. Measured at `f3974036a` (code `26ccef4ee`), on a binary whose
`.text` hashes `81d2f383dcc0270c45ce1644895fc3b838f9577e7cc7e0cfaef8d24a7d680532`,
identical to the one the re-profile measured.

## Method

`objdump -d` the release binary, then find every conditional branch whose target
block reaches a call to `core::panicking::panic_bounds_check`,
`core::slice::index::slice_index_fail` or
`core::panicking::panic_misaligned_pointer_dereference` without leaving the
block first. That branch is a bounds-check guard; the block it jumps to is the
cold path.

Priced from `valgrind --tool=callgrind --dump-instr=yes`, which gives retired
instructions per address, so the guard branches and the compares feeding them
are counted where they actually executed rather than attributed.

**The control is built into the classification: a real bounds-panic block never
executes in a program that exits 0.** Five blocks out of 958 had a non-zero
count, which means they were misclassified by the window heuristic, and they are
excluded. After the exclusion the cold blocks execute **zero** instructions, on
every axis.

Scripts and dumps in the session scratchpad under `bc-analysis/`.

## Static, in the binary

| | |
|---|---|
| `.text` | 2,525,115 bytes |
| guard branches | 1,295, **7,182 bytes** |
| cold panic blocks | 953, **18,546 bytes** |
| distinct panic call sites | 883 |
| **together** | **25,728 bytes, 1.019% of `.text`** |

The compare that feeds a guard is not counted here: 1,177 of the 1,295 guards
have an adjacent `cmp` or `test`, and the other 118 branch on flags an earlier
instruction already set.

## Dynamic, in retired instructions

| axis | total | at the guard branches | with the adjacent compares |
|---|---|---|---|
| `rexxcps` | 20,291,841,264 | 102,359,742, 0.5044% | **0.9868%** |
| `varlookup` | 17,432,573,007 | 87,444, 0.0005% | **0.0010%** |
| `emptyloop` | 9,923,583,822 | 87,376, 0.0009% | **0.0017%** |

**The narrow axes have no bounds checking left in their hot loops.** Their
87,444 and 87,376 executed checks are startup; the loops themselves run none.
That is `26ccef4ee` working: those programs are almost pure driver, and the
driver's per-op check is the one it removed. It also explains why that commit
was worth -0.864% and -0.750% there and only -0.079% on `rexxcps`.

`rexxcps`'s remaining checks are not in the driver at all. The hottest guards
are in `collect_now`, `alloc_with`, `exec_parse`, `numeric_order`,
`Number::mul`, `concat_values` and `builtin::run`.

## What this retires

**The `core/src/slice/index.rs` attribution said 3.86% of `rexxcps`. The
bounds checks themselves are 0.99%, near enough four times less.** That file's
share covers the whole indexing operation, address computation and iteration
included, and an inlined file's share inside one function was already shown
today to move the wrong way when the function is rewritten.

So the 3.86% never was a bounds-check figure, and the remaining bounds-check
work on `rexxcps` is about one percent of the program, spread across seven
functions, none of them the driver.

## What the figure is not

**0.99% is an upper bound on what removing the checks would save**, not an
estimate of it. Some of those compares are needed for something else and would
survive; against that, removing a check can unlock optimisation downstream. Only
an A/B settles either, and safe Rust offers no switch to take them all out at
once for a control.
