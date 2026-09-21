# The instructions-per-op profile on rexxcps, measured 2026-09-20

Moritz, 2026-09-20: *"we have to get the instructions per op down."* This is the
denominator and the profile, measured rather than estimated.

Baseline `target/release/rexx-run` at `f9ffe9a8b`, `rust/bench-rexxcps/rexxcps.rex`
(the pinned copy), `valgrind --tool=callgrind --dump-instr=yes`, machine at load
~1.3. Self costs below sum to exactly the run's own `summary:` line,
21,147,250,696, which is how the attribution was checked rather than assumed.

## The denominator, derived not guessed

Op dispatch is a jump table, and its execution count is the number of ops
dispatched. Two tables are live, found by disassembly and counted at their
`jmpq *` addresses:

* `drive.rs:538`, the outer `match op` (`cmpq $0x28` then an indexed jump):
  **19,441,281** in `<true>`, 1,120,000 in `<false>`
* `drive.rs:637`, the region walk inside the `Op::Clause` arm:
  **98,924,837** in `<true>`, 8,400,000 in `<false>`

Two further indirect jumps in each instantiation never execute.

| quantity | value |
|---|---|
| clauses (200 x 100 x 1000) | 20,000,000 |
| ops dispatched | **127,886,118** |
| **ops per clause** | **6.39** |
| **instructions per op** | **165.4** |
| instructions per clause | 1,057.4 |

The outer table firing 19.4M times against 20M clauses is the cross-check: it is
essentially once per clause, as the region architecture implies.

## Where the 165 go

Self Ir, so a function inlined into another is counted in its host.

| share | Ir/op | function |
|---|---|---|
| 20.45% | 33.82 | `Interp::run_ops_from::<true>` |
| 6.77% | 11.19 | `Interp::exec_parse` |
| 4.13% | 6.84 | `Interp::apply_binary` |
| 2.73% | 4.51 | `__memcpy_avx_unaligned_erms` |
| 2.53% | 4.18 | `Interp::assign_expr_target` |
| 2.35% | 3.88 | `Interp::run_ops_from::<true>'2` |
| 2.30% | 3.80 | `rexx_num::compare::numeric_order` |
| 2.21% | 3.66 | `Number::add_signed` |
| 2.03% | 3.36 | `Interp::run_ops_from::<false>` |
| 1.97% | 3.26 | `Interp::append_tails` |
| 1.81% | 2.99 | `Interp::concat_values` |
| 1.70% | 2.81 | `Interp::collect_now` |
| 1.63% | 2.69 | `Interp::to_text` |
| 1.56% | 2.57 | `builtin::run` |
| 1.49% | 2.47 | `Number::mul` |
| 1.49% | 2.46 | `Interp::alloc_with` |
| 1.46% | 2.42 | `Interp::read_at` |
| 1.38% | 2.28 | `Interp::inline_text_to_number` |
| 1.28% | 2.12 | `Interp::heap_to_number` |
| 1.06% | 1.75 | `Interp::run_call_args` |

**The driver's three instantiations together are 5,250,353,399 Ir, 24.8% of the
program, 41.1 Ir per dispatched op.** Roughly 124 Ir per op happen outside it.

## The comparison that frames the target

CREXX's VM costs about **32 host instructions per VM instruction**, measured on
their own binary on this machine: ~4 of dispatch and ~28 of handler body.

**Our driver's own overhead alone, 41.1 Ir per op, already exceeds CREXX's entire
per-instruction cost**, and we spend a further ~124 on top. Their ops are
statically typed with no dynamic name resolution, which is what deleting
`INTERPRET` and `DROP` bought them, so this is not a like-for-like target. It does
size the gap.

## What the profile says to attack

**There is no dominant cost.** After the driver, the largest single item is
`exec_parse` at 11.19 Ir/op, and everything below it is under 7. Halving 165
requires broad wins, not one fix.

Clusters worth naming, since they are spread across several rows:

* **Text/number conversion**: `to_text` 2.69 + `inline_text_to_number` 2.28 +
  `heap_to_number` 2.12 = **7.09 Ir/op**. `bench-programs/README.md` records that
  rexxcps performs 5,580,002 of these conversions.
* **Allocation and collection**: `collect_now` 2.81 + `alloc_with` 2.46 + `free`
  1.52 = **6.79 Ir/op**.
* **Numeric**: `numeric_order` 3.80 + `add_signed` 3.66 + `mul` 2.47 = **9.93
  Ir/op**.
* **`exec_parse` alone is 6.77%**, which independently reproduces the 6.8% figure
  `bench-programs/README.md` already records for it.

**And the other factor is ops per clause, not only cost per op.** 1,057 Ir per
clause is 6.39 ops times 165.4. Emitting fewer ops per clause attacks the same
product from the other side, and the CREXX-derived compare-and-branch fusion
(candidate C, queued separately) is exactly that: it removes two ops from every
`IF`/`WHEN` whose root is a comparison.

## Caveats

* Self cost attributes an inlined callee to its host, so the driver's 33.82 Ir/op
  includes whatever LLVM inlined into it. It is not a claim about the dispatch
  loop in isolation.
* The whole-program denominator includes startup and the empty-loop calibration.
  Both are small against 20M clauses but they are in the numerator.

## Ways to decrease ops per clause, measured 2026-09-20

The executed op mix, from the two dispatch tables' own execution counts. Targets
decoded from the binary's jump tables; counts from the callgrind instruction dump.

**Inner region dispatch** (`drive.rs:637`), 103,424,837 at known targets:

| op | executions | share |
|---|---|---|
| **trace ops** (`TraceArgument`/`TraceFunction`/`TraceLiteral`/`TraceRead`/`TraceOperator`/`TracePrefix`, one shared arm) | **38,461,666** | **37.2%** |
| `LoadConstant` | 9,880,016 | 9.6% |
| `Binary` | 7,860,011 | 7.6% |
| `Load` | 7,560,421 | 7.3% |
| `JumpUnless` | 5,900,006 | 5.7% |
| `Condition` | 5,900,006 | 5.7% |
| `Const` | 5,060,417 | 4.9% |
| `PushArg` | 4,760,417 | 4.6% |
| `Jump` | 4,500,000 | 4.4% |
| `Store` | 3,120,256 | 3.0% |
| `Parse` | 2,240,002 | 2.2% |
| `CallArgs` | 1,960,207 | 1.9% |
| `Arith` | 1,380,205 | 1.3% |

**Outer dispatch** (`drive.rs:538`), 19,441,281: `Clause` 84.0%, `EndBranch` 4.3%,
`LoopNext` 3.1%, `EndWhen` / `EnterWhen` / `SelectCaseText` 2.9% each.

### One TRACE instruction taints the whole chunk

Minimal, rendered with `rexx-ir`:

* `zw = 1` / `zv = zw + 1` / `say zv` compiles to **11 ops, no trace ops**.
* The same three clauses with `trace off` prepended compile to **18 ops, with
  `TraceLiteral`, `TraceRead` and `TraceOperator` throughout**.

**Writing `TRACE OFF` makes the compiler emit trace ops for the rest of the
chunk**, though its setting is a compile-time literal. Emission is per-chunk, so
one `TRACE` anywhere taints all of it. `rexxcps` has three, one inside its timed
1000-clause body.

### The A/B, and the constant it yields

`rexxcps` against a scratch copy with all three `TRACE` instructions replaced by
`nop`, same binary, callgrind:

| | baseline | no TRACE | delta |
|---|---|---|---|
| instructions | 21,147,250,696 | 20,325,640,274 | **-3.89%** |
| ops dispatched | 127,886,118 | 86,884,257 | **-32.1%** |
| ops per clause | 6.39 | **4.34** | |
| Ir per op | 165.4 | **233.9** | |
| Ir per clause | 1,057.4 | 1,016.3 | -3.89% |

**Removing a third of the ops removed 3.89% of the instructions**, and `Ir/op`
*rose*, because the ops removed were the cheapest in the stream.

The removed ops did no work, so the delta prices dispatch itself:

> **821,610,422 instructions / 41,001,861 ops = 20.0 instructions to dispatch one
> op that does nothing.**

That constant sizes every op-fusion proposal without measuring it:

* `Condition` + `JumpUnless` always co-occur, both at exactly 5,900,006. Fusing:
  5.9M ops, **~0.56%**.
* Folding `Binary` in where the condition's root is a comparison (the queued
  CREXX-derived candidate C): up to 11.8M ops, **~1.1%**.
* Constant loads, `LoadConstant` 9.88M + `Const` 5.06M = 14.9M, with `Clause
  LoadConstant Store` and `Clause Const Store` among the commonest static clause
  shapes. A fused store-constant: **~1.4%**.
* 25 of 129 static clauses have an **empty region**: the `Clause` op is the whole
  clause, and it still pays an outer dispatch and a region entry.

### What this says about the target

**`Ir per op` is the wrong thing to minimise, and op count is a weak lever.**
Dispatch costs 20 instructions; the other ~214 per op at the no-TRACE figure is
the work inside the ops. Every op-fusion candidate above is worth about one
percent, and all of them together are a few percent.

The trace-op fix is the largest single item at 3.89% and is the only one that is a
**defect rather than a design trade**: a program that turns tracing off should not
pay for tracing. Two shapes, and the machinery for the second already exists:
a statically known `TRACE` setting should select the untraced emission for the
clauses after it, and a dynamic `trace value x` should recompile once the setting
is observed, which is what the staleness check at `drive.rs:568` is for.
