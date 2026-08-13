# `PARSE` target slots, and the map that charges for them

**Goal:** stop hashing a `PARSE` target's name where the plan already holds its slot, and stop paying a hash to read the slot the plan holds.

**Status:** open, two tasks. The successor entry 32 named and measured, taken up here.

## What is already measured, and by whom

The bare-stem task instrumented `Interp::slot_of` at its own head and ran `samples/rexxcps.rex` under `count=100`/`averaging=100`:

```
3,080,203 slot_of calls, of which
  2,800,003  PARSE targets      (plan-resolved: A1..A4, B1..B3, C1..C3, P1..P8, V1, V2)
    140,199  RESULT             (through extra)
    139,999  SIGL               (through extra)
          2  one-offs at startup
```

**Not one of those calls is a stem or a compound**, which is what closed that family.
`PARSE` targets are the large remainder, and they are resolvable: with the target arm instrumented, **2,800,003 of 2,800,003** come back `by_symbol=Some(n)` with `slot_of` computing that same `n`, on both engines -- `V2 Some(19)/19`, `P8 Some(28)/28`, and so on.
A three-line probe (`parse var zline za zb zc`) reads `Some(1)/1`, `Some(2)/2`, `Some(3)/3` on both engines.

`parse_template.rs:686` passes a literal `None` and says why in its own comment: "No compiler resolved a `PARSE` target".

**And the map that would answer costs a hash of its own.** `hash_one::<&SymbolId>` measured 0.83-0.93% of `rexxcps` self time at that head, which is the same order as `slot_of` itself.
That is the price this family has been paying all along: every slot it saves is bought with a SipHash of a `u32`, because `by_symbol` and `Code::slots` are `HashMap<SymbolId, usize>`.
`plan.rs:208` has said since `180875a9` that this wants an array index and that the accessor to do it landed, and that the swap is "worth its own measurement, not a side effect of an unrelated change".
This plan is that measurement, taken before the change that would otherwise hide it.

## The trap this plan exists to avoid, stated before the tasks

**Replacing a literal `None` at a call site with a value is exactly what the bare-stem task measured at 2 instructions on every pass of every controlled loop**, stem-controlled or not, against same-binary spans in the low thousands.
Reverting that one line put `emptyloop` back to a figure indistinguishable from base.
`parse_template.rs:686` is a literal `None` at a call site.
**So `emptyloop` and `varlookup` are gates on Task 2, not axes it reports afterwards.**

**And the arms that now assert are aimed at this change.** `assign_expr_target`'s stem and compound arms carry `debug_assert!(at.is_none())`, landed by that task's fix round precisely because this is the change most likely to violate them.
A `PARSE` target can be stem-shaped or compound-shaped (`parse var zline za. zb.1`), and those shapes take their slots by the entry route already landed.
**Task 2 passes a slot for a `Variable`-shaped target only.** If an assertion fires, the task has found its own defect, not a false guard.

## Task 1: the id-indexed slot table

**Files:** `rust/crates/rexx-exec/src/plan.rs`, `rust/crates/rexx-exec/src/lib.rs`, `rust/crates/rexx-exec/src/run.rs`, `rust/crates/rexx-exec/src/ir/compile.rs`.

Replace `Plan::by_symbol: HashMap<SymbolId, usize>` with a dense table indexed by `SymbolId::index()`, in the shape `Plan::compounds` already uses in the same struct and `Plan::lines`/`Plan::indents` use beside it.

- [ ] **Step 1: measure the base and every axis's own spread**, `perf stat -e instructions:u`, arms staged at one fixed binary path, from a fresh empty directory. Include `emptyloop` and `varlookup`, which are the control axes for Task 2 and are wanted at base here anyway.

- [ ] **Step 2: size the change from the compiler, not from a search.** `by_symbol` is read through `Code::slots` at `lib.rs:1344` and `run.rs:1138` and directly in `compile.rs` and `plan.rs`. A grep over the type name will overcount: the last time this project budgeted a type change that way, the sites that named the type and the sites that needed changing differed by two orders. Change the field, build, and let the errors be the list.

- [ ] **Step 3: keep the accessor honest.** Whatever replaces `by_symbol`, its read path carries `debug_assert_eq!` against the answer the old three-source resolution gives, then is **inverted** to prove the probe fires. This is the technique that has caught this family at every application and it is not optional.

- [ ] **Step 4: measure.** Every axis. If the swap is below the resolution floor everywhere, **say so and do not take it** -- Task 2 does not depend on Task 1 landing, only on knowing which representation it is being measured against.

- [ ] **Step 5: record it as the next entry of `phase-4f-record.md`, appended.**

## Task 2: the `PARSE` target's own slot

**Files:** `rust/crates/rexx-exec/src/parse_template.rs`, and whatever Task 1 left as the accessor.

- [ ] **Step 1: pass the slot at `parse_template.rs:686`, for a `Variable`-shaped target only.** Stem-shaped and compound-shaped targets keep `None` and take their slots by the route already landed. Correct that call site's comment, which currently states as a fact the thing this task changes.

- [ ] **Step 2: gate on the control axes before believing anything.** `emptyloop` and `varlookup` execute no `PARSE` at all. If either moves up by more than its own span, **the shape is wrong and the win is not real** -- see the trap above, and prefer the shape that leaves every call site's argument constant, as the bare-stem task did.

- [ ] **Step 3: prove which axes can see it, by counting.** Instrument the target arm and run every axis. An axis that executes no `PARSE` target makes its result a bound rather than a zero, and must be reported as one.

- [ ] **Step 4: check the answers did not move**, with the accessor tripwire, then inverted.

- [ ] **Step 5: re-profile `rexxcps` and count `slot_of` again.** Entry 32's count was 3,080,203 with 2,800,003 of them `PARSE` targets. Say what the count is afterwards and what the remainder is, because that decides whether anything nameable is left on this path at all.

- [ ] **Step 6: record it, appended.**

## What these tasks must not do

* **Do not change what any program observes.** Both engines must stay byte-identical to each other and to the oracle, across the corpus. No divergence licence is spent.
* **Do not widen `write_slot` or `control_slot`** to answer for shapes they decline today. Those declines are measured and their doc comments carry the measurements.
* **Do not remove `slot_of`.** `RESULT` and `SIGL` reach it from run-time byte strings with no symbol to hang a slot on, and that is a different kind of change.
* **Do not quote a wall-clock figure.** The instruction counter is this phase's instrument.
* **Do not rewrite an earlier record entry.** The record is appended to.
