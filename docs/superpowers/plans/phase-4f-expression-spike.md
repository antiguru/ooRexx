# Phase 4f -- what full expression promotion is worth, measured

**A spike, not a change.** Nothing here landed.
Every figure below was taken on a throwaway build, and the tree was restored from a `cp` copy verified with `sha256sum -c` before this file was written.

**What it answers.** The record's second profiling pass concluded that four queued candidates are one defect -- a fact the parser knew, re-derived at run time -- and named the structural change as extending compile-time resolution from statements to expressions.
It estimated that change at **26.8 points of `strings` and 14.5 of `rexxcps`**, from a sum of sampled shares it said in the same paragraph are not additive.
This spike replaces that estimate with one measured number per axis.

**The headline, first, because one half of it is a refusal.**

| axis | the record's estimate | measured here | verdict |
|---|---:|---:|---|
| `strings` | -26.8% | **-25.25% instructions, -28.24% wall, -28.95% cycles** | **holds** |
| `rexxcps` | -14.5% | **-5.10% instructions, -3.81% cycles, +3.76% clauses per second** | **does not hold, by about a factor of three** |

## Build identity

| | |
|---|---|
| BASE | `f024e902cc14e2fefa8f1ac021165eea53c204cb`, working tree clean before and after |
| BASE `rexx-run` | size=13849736, sha256 `35999021f97f24bb81e8c7a65083340084a8e1107087e11caf34dc8bc7f82141` -- **entry 11's binary reproduced byte for byte**, and reproduced again after the revert |
| spike `rexx-run` | sha256 `bb9da02238b0390afbce2d6172b0efb6759144e22e05afc46999b10570732452` |
| spike, builtin row not cached | sha256 `f2645f3ebb181a4e3a122b0044ce427833cce97bb730441444b65008a05b4933` |
| oracle | untouched and unread by the timed runs; the oracle ratios quoted at the end are entry 11's, marked as constructions where they are used |

**The revert is checked rather than asserted.**
148 source files under `rust/crates` were hashed before anything was edited and re-checked after restoring: 0 mismatches, `git status --porcelain` empty.
`cargo clean -p rexx-exec --release` preceded the final rebuild, because a `cp -a` restore preserves mtimes older than the artefact and cargo will otherwise report "Finished" without rebuilding -- which happened once during this spike and would have left the spike's binary sitting behind a clean `git status`.

## What reaches `Op::EvalExpr` today, counted rather than sampled

A counting build incremented a global counter in `Interp::eval` (once per node evaluated, anywhere in the crate) and in the driver's `Op::EvalExpr`, `Op::Clause` and `Op::Generic` arms, with the `EvalExpr` arm bracketed so that every `eval` entry underneath it -- a call's arguments included -- is attributed to the compiled-expression path.
The AST-derived node count and the run-time bracket agree exactly on both programs, which is what says the attribution is right rather than plausible.

| | `strings` | `rexxcps` |
|---|---:|---:|
| `Op::Generic` clause executions | **0** | 2,660,208 |
| `Op::Clause` (promoted) executions | 15,000,005 | 6,060,622 |
| `eval.rs` node evaluations, total | 54,000,002 | 16,991,356 |
| ... of those, under an `Op::EvalExpr` | **54,000,002 (100%)** | 11,501,055 (67.7%) |
| `Op::EvalExpr` executions | 15,000,002 | 3,240,413 |
| native value ops executed (`Const`, `LoadConstant`, `Load`, `Arith`) | **4** | 2,520,018 |

**On `strings` the IR's native expression ops execute four times in the whole program.**
Every one of the five clauses in its loop body is a promoted clause containing exactly one `Op::EvalExpr` and one `Op::Store`, and 18 AST nodes per iteration are evaluated by `eval.rs` recursion.
The axis the record calls expression-heavy and call-heavy is, at head, running its expressions entirely on the tree-walker's evaluator inside promoted clauses.

**On `rexxcps` a third of the expression work is not reachable from an expression op at all**, because 30.5% of its clause steps are `Op::Generic` -- `PARSE`, `ADDRESS`, `TRACE`, `CALL (expr)` and the rest of the unpromoted residue -- and those evaluate their expressions inside the tree-walker's own clause unit.

**Where the `EvalExpr` executions sit is not where the queue assumed.** Bucketed by root shape:

| root of the expression | `strings` | `rexxcps` |
|---|---:|---:|
| `Call` | 9,000,000 | 140,000 |
| `Binary`, concatenation | 3,000,000 | 280,005 |
| `Binary`, arithmetic | 3,000,000 | 200 |
| `Binary`, comparison | -- | 2,100,002 |
| `Constant` | 1 | 720,004 |
| `Variable` | 1 | 202 |

**The restriction is not only the expression's shape, it is which instruction slots were wired to the promoting path at all.**
`compile` routes an `Assignment`'s value and a `SAY`'s expression through `push_value`; an `IF`'s condition, a `SELECT CASE`'s expression and every `DO`/`LOOP` header value push `Op::EvalExpr` unconditionally.
So `if 1 then` runs `eval.rs` on a bare constant symbol, which is where `rexxcps`' 720,004 `Constant` roots and 2,100,002 comparison roots come from.
That is a bigger share of `rexxcps` than every promotable assignment on the axis, and it is why the first version of this spike measured **-0.40%** there and the second measured -5.10%.

## What the spike is, and what it pays

**It is not a new IR.** It widens `native_shape` and `push_native` in `ir/compile.rs`, adds five op variants and their driver arms, and wires three more instruction slots to `push_value`.

| widened to | emitted as |
|---|---|
| `Binary` with a concatenation operator | `Op::Concat` + the existing `Op::TraceOperator` |
| `Binary` with a comparison operator | `Op::Compare` + `Op::TraceOperator` |
| `Call` with a symbol or literal target and all-native arguments | argument ops into consecutive registers, each followed by `Op::TraceArg`, then `Op::CallExpr`, then `Op::TraceFunction` |
| an `IF`'s condition | the condition's native ops + `Op::CondCheck` |
| a `DO`/`LOOP` header value | `push_value`, as an assignment's value already is |

**The obligations the brief names, and which of them are paid.**

* **The GC temps frame and rooting at expression depth: paid, by construction.** Registers are an indexable region of the temporaries stack and are already roots the collector walks, so every intermediate a promoted expression produces is rooted exactly as an `Op::Arith` result already is. The spike therefore does **not** re-push each call argument as a temp the way `invoke_call`'s own loop does, and does not push `eval_condition`'s frame around a condition value.
* **Failure-site resolution: paid, unchanged.** Every new op sits inside an `Op::Clause` region, whose `enter_stepped_clause`/`leave_stepped_clause` pair resolves the failing clause's site exactly as before. A failing op leaves the region carrying its `Failure` rather than taking a `?` past the boundary that owes it a site.
* **The trace echo each node kind owes: paid, and two of the lines are new ops.** `>L>` (`TraceLiteral`), `>V>`/`>C>` (`TraceRead`) and `>O>` (`TraceOperator`, now reached by concatenation and comparison as well as arithmetic) already existed. `>A>` is new (`Op::TraceArg`, between one argument's value ops and the next argument's, which is where `invoke_call`'s loop emits it, and with an omitted position tracing an empty value line rather than none). `>F>` is new (`Op::TraceFunction`). An `IF`'s `>>>` is emitted by `Op::CondCheck`.
* **The clause boundary's own work: paid, untouched.** `Op::Clause` still opens a temps frame, sets the `SIGL` clause line through `clause_line` -> `ProgramSource::line_of`, takes the `PROCEDURE` permission and delivers a queued `CALL ON` handler at its end. **This spike removes none of that**, which matters for the estimate and is the subject of the next section.
* **The op array's width: paid, and the assertion is what says so.** `const _: () = assert!(size_of::<Op>() == 12)` is unchanged and still compiles, so the five new variants cost nothing in stream width. `Op::CallExpr` carries `{ site: u32, dst: u16 }` and its argument base, count and omitted-position mask live in the chunk's own call-site table, which is where they belong anyway: they are compile-time constants.
* **The sharing rule: kept by splitting, not by copying.** `invoke_call` became `eval_call_arguments` + `invoke_call_with_arguments`, and the call op enters the second; `concat` and `eval_compare` became wrappers over `concat_values` and `compare_values`, which the ops enter. There is one implementation of each.

**What is not paid, costed rather than skipped silently.**

* **"Compile-time" builtin resolution is really first-execution-per-site caching.** `compile` cannot see a symbol's spelling today, so the site resolves on its first execution and keeps the answer, exactly as `Op::Call` already does. A landed version that resolved at parse time would be **at least** as cheap as this, so the figure is not inflated by it. The spike does hand `compile` the `SymbolTable` in order to intern a call target's name, which contradicts `Op::LoadConstant`'s own doc comment and `chunk_for`'s cache-key-completeness argument; that is a design cost to settle, not a run-time cost to subtract.
* **The two rootings the spike skips.** If a landed version turns out to need `invoke_call`'s per-argument `push_temp` and `eval_condition`'s frame after all, that is about one store per argument and about three per `IF`. On `strings` that is 27 million stores -- nine arguments per iteration over three million iterations -- against 15.9 billion instructions removed, **0.17%**. On `rexxcps` the argument side is negligible (140,000 calls) and the `IF` side is bounded above by every promoted clause being an `IF`: 6,060,622 x 3 against 1.89 billion removed, **at most 0.97%**. Both are at or inside the instrument's own spread on their axis, so the headline figures are quoted unsubtracted with those bounds stated beside them.
* **The promotion is capped at 64 arguments per call**, because the omitted-position mask is a `u64`. Nothing in either axis or the corpus reaches it.
* **No unit test ran.** `chunk_for` grew a fifth parameter and its test call sites pass four, so the `#[cfg(test)]` golden op-stream tests and `corpus_shape_tests` would not build; only `cargo build --release --bin rexx-run` was ever run, and `cargo fmt` and `cargo clippy` were not. **The spike's entire correctness evidence is the differential below**, which is weaker than a landed change's would be and is stated as such.

## Correctness, which is what says the number was taken on a working interpreter

* **The corpus, BASE against the spike.** 70 programs from `rust/corpus`, each from a fresh directory it makes itself, under both engines, stdout and stderr and exit status read separately: **420 descriptor comparisons, all byte-identical.**
* **That sweep can fail, and it did.** The first spike read an omitted argument position out of a register nothing had written, so `say format(12345.6789, , 3)` answered 93.942 where BASE answers a value. `corpus/num/format_trunc.rex` went red on all three descriptors on the IR arm and stayed green on the tree-walker arm, which is exactly the shape a promoted-path defect has. The bitmask above is the fix, and the sweep is the witness for it rather than an argument.
* **Eleven probes against the oracle**, 66 descriptors, **all byte-identical on both engines**: `POS`/`SUBSTR`/`CHANGESTR`/`LENGTH`/`COPIES`/`WORD`/`MAX`/`ABS` calls, quoted targets, internal labels and their `RESULT`, nested calls, concatenation in all three spellings, ordinary and strict comparison, omitted argument positions, `TRACE I` over `>L>`/`>V>`/`>A>`/`>F>`/`>O>`, `TRACE R` over `>>>` for `IF` and loop headers, a comma-list condition, a non-logical `IF` under `SIGNAL ON SYNTAX` (34.1), 40.3/40.5, 42.3, 43.1, and a loop header raising on its `TO` value.
* **What none of that can see.** No collector stress run, no `PROCEDURE EXPOSE` or `INTERPRET` interaction with the register file, and no assertion over the emitted op stream. A landed version needs all three.

## The measurement

Two instruments on every axis, as the brief requires, and the instruction count is the one trusted where they disagree.
`perf stat -e instructions:u,cycles:u`, six runs per side, arms alternating, medians, with each side's own full range printed beside the movement -- entry 10's rule, that a movement below about a per cent needs that axis's own range measured in the same run.
Wall clock separately, seven rounds, base/spike order rotated every round.
**Every sitting was gated on six consecutive five-second `/proc/stat` samples at or above 90% idle**; all four gates opened at 98.8% to 99.3%.

### `strings`

| instrument | base | spike | change | base range | spike range |
|---|---:|---:|---:|---:|---:|
| `instructions:u` | 63,131,754,247 | 47,193,245,001 | **-25.25%** | 0.125% | 0.115% |
| `cycles:u` | 16,257,248,988 | 11,551,348,417 | **-28.95%** | 4.10% | 1.05% |
| wall, median of 7 | 5.4827 s | 3.9344 s | **-28.24%** | | |

Per round on wall: -29.10%, -28.24%, -27.61%, -27.69%, -28.18%, -26.78%, -28.35%.
**The sign holds in 7 of 7**, and the instruction movement is two orders of magnitude above either arm's own range.
Both arms printed the identical stdout hash in all twelve counted runs.

### `rexxcps`

| instrument | base | spike | change | base range | spike range |
|---|---:|---:|---:|---:|---:|
| `instructions:u` | 36,956,355,874 | 35,070,797,451 | **-5.10%** | 0.284% | 0.326% |
| `cycles:u` | 11,350,360,360 | 10,917,802,165 | -3.81% | 0.629% | 1.148% |
| clauses per second, median of 5 | 2,594,581 | 2,692,253 | **+3.76%** | | |

Per round on cps: +2.56%, +4.38%, +4.31%, +4.91%, +3.28%, **ahead in 5 of 5**, with both arms self-calibrating to the identical `100 x 100` in every round.
`rexxcps` is not read as wall time, for entry 1's reason.
The instruction movement is 15 times its own range and is resolved; the cycle movement is about 3 times its own range and is weaker.

### The half of `strings` that is one thing

A third binary, identical to the spike except that the call site's kept builtin row is withheld so `builtin::dispatch` redoes `is_builtin` and the 58-entry table scan on every call.
Three arms, one gated sitting, six runs each, `instructions:u`:

| arm | median | against base |
|---|---:|---:|
| base | 63,143,082,064 | -- |
| full promotion, builtin row **not** cached | 55,258,683,380 | **-12.49%** |
| full promotion | 47,168,020,270 | **-25.30%** |

**About half the win on `strings` is the builtin's resolution and about half is everything else.**
8.09 billion of the 15.98 billion instructions removed are the name resolution alone -- 12.8 points of the axis, against the 17.4 points entry 11's sampled share gave it.
The other 7.88 billion are the `eval` wrapper's depth bookkeeping and post-order trace hook, the per-node match, the id-keyed slot map every expression read went through, and the argument evaluation loop.

## What full promotion cannot reach

Measured, on the spike build with the counter applied on top of it.

| | `strings` | `rexxcps` |
|---|---:|---:|
| `Op::EvalExpr` executions remaining | **0** | **1** |
| `eval.rs` node evaluations remaining | **0** | 5,490,308 |
| ... against BASE's | 54,000,002 | 16,991,356 |

**On `strings` `eval.rs` is never entered at all.** The single remaining `EvalExpr` on `rexxcps` is one `IF` whose condition uses a logical operator.

**So the residual is not expression shapes. It is three other things, and naming them is the part a plan most needs.**

* **Expressions inside unpromoted instructions.** 5,490,308 node evaluations survive on `rexxcps` -- **32.3% of the axis's total** -- and every one of them is reached from an `Op::Generic` clause or from a promoted op that evaluates expressions itself. `PARSE`, `INTERPRET`, `ADDRESS`, `TRACE`, `DROP`, `NUMERIC` and `CALL (expr)` are the instruction side; a `WHILE`/`UNTIL` test re-evaluated per pass is the op side. **Widening the expression compiler reaches none of it**; promoting more *instructions* is what does, and that is a different change.
* **Expression shapes that must stay on `eval.rs`, and they are cheap to enumerate because they are rare.** `&`/`|`/`&&` (`eval_logical` validates each operand as a Rexx logical value and raises on the left first, and the order is measured); `ExprKind::Logical`, the comma list, which short-circuits where the dyadic operators do not; `Message`, `List`, `QualifiedCall`, `ClassResolver` and `DotVariable`, which are Phase 5's; and `VariableReference`, whose value in an argument position is a reference rather than a value. The first two are promotable with more ops; the rest are not this phase's.
* **The costs that live *inside* the shared functions a promoted op enters, which promotion cannot touch by construction.** This is the finding that contradicts the record. A promoted `Op::Load` for a compound still calls `Interp::read_symbol`, which still calls `tail_key` -> `compound_parts` and re-splits the tail from the source spelling. A promoted clause still calls `enter_stepped_clause` -> `clause_line` -> `ProgramSource::line_of`, a binary search per clause. A promoted `Op::Load` carries a plan slot for a simple variable and for a stem, and **not** for a compound, whose read goes through the stem's slot and a tail key resolved at the read site.

## Whether entry 11's estimate holds

**On `strings` it does, and the arithmetic of why it does is not the arithmetic the entry used.**
Entry 11's 26.8 points are 17.4 (builtin resolution) + 6.9 (the id-keyed map) + 2.5 (the line table).
**The line table is not subsumed** -- `line_of` is a per-clause cost and expression promotion does not go near it -- so the change measured here is aimed at 24.3 points and moved 25.25% of instructions and 28.24% of wall.
That is the same overshoot entries 3 and 9 recorded and gave the same reason for: a sampled share counts the block, and the change also removes the machinery around it.

**On `rexxcps` it does not, and the entry's own per-axis table is what shows why.**
Its 14.5 points are 3.0 (builtin resolution) + 6.7 (the variable maps) + 4.8 (the line table).
The line table's 4.8 is not subsumed, for the reason above.
And of the 6.7, entry 11's own text says 5.8 is `Interp::slot_of` reached from "`tail_key`'s `read_by_name` (31%), `stem_get` (20%), `assign_expr_target` under `exec_parse`'s `assign_targets` (34%) and `stem_set` (10%)" -- and **not one of those four is an expression read**.
`exec_parse` is an unpromoted clause; `tail_key` and `stem_get` sit inside `read_symbol`, which a promoted `Op::Load` enters by the same door.
Only `read_at`'s 0.9 is expression-read work.

**So the entry's own decomposition predicts 3.0 + 0.9 = 3.9 points for `rexxcps`, and the measurement is +3.76% clauses per second.**
The shares were right; the sum was wrong, and it was wrong by attributing to expression promotion two costs that survive it.

**The oracle-ratio landings, marked as constructions rather than measurements.**
Entry 11's head ratios multiplied by this spike's paired movement, which crosses a sitting boundary and is not an oracle comparison of its own:

| axis | entry 11 head | entry 11's predicted landing | this spike's landing |
|---|---:|---:|---:|
| `strings` | 6.2666x | 4.59x | **4.50x** by wall, 4.68x by instructions |
| `rexxcps` | 6.4436x | 5.51x | **6.21x** |

## Recommendation on scope

**It is one phase on `strings` and it is not a phase at all on `rexxcps`.**

* **Land it, scoped by the measurement rather than by the slogan.** A quarter of `strings` is the largest single movement any candidate on the current queue has a measured number for, the correctness surface is narrow, the op array does not grow, and the sharing rule is kept by splitting three functions rather than by copying any.
* **Land the call op first if risk is to be staged.** Half the `strings` win -- 12.8 points of the axis -- is one mechanism: a call site keeping its resolved builtin row. That is queue candidate 4, it needs an `Op::CallExpr` and a site cache and nothing else about expressions, and `Op::Call` has already built both for the statement form. The other half needs the full widening.
* **Do not carry `rexxcps` in the same phase, and do not expect the two remaining queue items to come with it.** Candidate 3 (the compound tail re-split) and candidate 6 (the line table) are **not** subsumed by this change and were not moved by it; they are separate edits to shared functions, they are still worth their measured shares, and the record should stop describing them as part of this one.
* **Entry 11's "necessary and not sufficient" survives on `strings` and is refuted on `rexxcps`.** `strings` lands at about 4.5x with every expression native and `eval.rs` never entered, so what remains there is the object model, exactly as entry 11 said. `rexxcps` lands at about 6.2x, and what remains there is **not** the object model alone: a third of its expression work is inside unpromoted instructions, and 30.5% of its clause steps never enter a promoted region at all. The next thing worth measuring on that axis is promoting `PARSE` and the `CALL (expr)` form, which is instruction promotion and belongs to whatever finishes Phase 4e's minimum set -- not to an expression compiler.

**The strongest single sentence for the plan.** Removing **two thirds** of every `eval.rs` node evaluation on `rexxcps` bought **5% of its instructions**; removing **all** of them on `strings` bought **25%**. The `eval.rs` recursion is not a fixed per-node tax that shows up wherever expressions do -- it is expensive in proportion to how many nodes a clause evaluates, and `strings` evaluates **3.600** nodes per promoted clause where `rexxcps` evaluates **1.898**.

## What this document cannot say

* **It is one host, unpinned, on a machine whose governor cannot be fixed.** Four gated sittings, six or seven rounds each.
* **It carries no oracle ratio of its own.** `rexx-bench-suite` was not run; every figure is against the immediately preceding binary, and the two landing figures above are constructions from entry 11's ratios across a sitting boundary.
* **The spike has no unit-test evidence at all**, for the reason given above, and the `cycles:u` column on `strings` sits inside a base-arm range of 4.10% that no other column here shows.
* **It measured the IR arm only.** The tree-walker was run for correctness and never timed.
* **The counting build is not the timing build.** The counters are atomics on the hot path and were never present in any timed run; a count and a time in this document come from different binaries and are never combined into one figure.
* **The residual counts describe these two programs.** The claim that `Op::Generic` clauses are where `rexxcps`' remaining expression work lives is a count on `rexxcps`, not a property of Rexx.
