# A bare stem's slot already exists and nothing uses it

**Goal:** Stop hashing a bare stem's name where the plan has already assigned it a slot, everywhere that still does -- and find a workload that can see the site which pays per iteration.

**Status:** done, 2026-08-13. Found by the stem-slot task of `2026-08-13-stem-slot-resolution.md`, which corrected the comments that denied it and deliberately did not take the optimisation; the per-iteration site was found by that task's review. **Confirmed real by both**, and taken here.

**Corrected by the task, after measuring: Step 2's route is the wrong one, and taking it costs more than the whole optimisation is worth.**
The obvious shape -- carrying the slot in `Option<usize>` from `control_slot` through `bind_control` into `assign_expr_target`, and from `write_slot` through `Op::Store` -- replaces a compile-time `None` at `bind_control`'s stem call site with a value, and that alone costs **2 instructions on every pass of every controlled loop**, stem-controlled or not: `emptyloop` +50,000,012, `varlookup` +38,000,036, where the same binary against itself spans 1,348 and 1,576. Attributed by partial revert: undoing that one line and nothing else puts `emptyloop` back on base exactly, and recomputing the slot inside the arm instead costs +100,000,000. Why the generated code changes was not established.
What works is to take the slot from the plan's own `CompoundName` entry -- `Plan::bind` records it there for a stem-shaped name, since such a name is its own stem half -- leaving every call site's argument constant. That serves **both** engines, where the compiled-op route serves one. See entry 32 of `phase-4f-record.md`.

## What was believed, and what is true

The comments in the tree said a bare stem write has no slot to resolve ahead of the write, so it must go by name.
Measured on `zs. = 'one'`, both engines: `by_symbol[id]` is `Some(0)` and equals the slot `stem_assign` then hashes for.
Measured on `do cv. = 1 to 3`, both engines: `by_symbol=Some(0)`, `slot_of=0`, on all four writes.
The slot exists. `Plan::bind` assigned it when it walked the body; nothing reads it back.

This is the same shape as the applications already landed -- `compound_parts` re-splitting an interned name, a tail piece re-hashing its own, a stem half re-hashing its own -- and it is the last member of the family that is known about.

## The sites

* **`stem_assign`**, reached from `run.rs`'s `assign_expr_target` for `zs. = value`. Per statement.
* **`replace_stem`**, the other bare-stem operation `stem.rs` leaves resolving by name. Per statement.
* **`bind_control`'s `NameShape::Stem` arm**, where `control_slot` answers `None` **by choice** and the slot it could answer is `Some(0)`. **Reached on every pass of a stem-controlled `DO`, so this one pays per iteration.**

The first two were named by the implementer; the third by its reviewer, and it is the one worth the task.

## What makes this a task rather than a patch

**Corrected 2026-08-13, after re-profiling: one existing axis does see the two per-statement sites.**
`samples/rexxcps.rex` writes `avar.=1.0''loop` inside its `do loop=1 to 14` body -- a bare stem write, fourteen times per iteration -- so it exercises `stem_assign` through `assign_expr_target`, which stands at 1.57% of that profile.
The first version of this section said no axis could see any of it, which was wrong and was written without reading the program.

**What still has no axis is `bind_control`'s.** `bind_control`'s stem arm is reached only by a *stem-controlled* `DO`, and every loop in `rexxcps` and in `rust/bench-programs/` is controlled by a simple variable.
That one pays per iteration, so it is the site most worth measuring and the only one needing a workload written for it.

The measurement discipline this project holds itself to says an axis that cannot see a change makes its result unstatable rather than zero -- which is why the workload comes before the fix rather than after it.

- [x] **Step 1: write the workload before the fix.** A stem-controlled `DO` over enough iterations to be measurable, and a bare-stem-write loop beside it. Decide whether either belongs in `rust/bench-programs/` permanently -- if it does, it is an axis every later entry inherits, so say why rather than adding it quietly. Measure the base with `perf stat -e instructions:u`, arms at one fixed binary path, and record each axis's own spread before quoting any difference.

- [x] **Step 2: take the slot** -- by the entry route, not the parameter route; see the correction above. `Option<usize>` with a `slot_of` fallback, the difference visible in the type. Read `5eaa3c8e8` for the pattern and `a93bfb548` for the tail-piece version.

- [x] **Step 3: answer the `extra` question again, by running.** It has been yes all three times -- `do za.zi = 1 to 3` grows `ZA.`, `interpret "zq.1 = 7"` grows `ZQ.`, `zn='ZR.1'; drop (zn)` grows `ZR.` -- and a fourth site is not exempt because the first three were. Instrument `slot_of`, run, and include a control that resolves through the plan.

- [x] **Step 4: check the answers did not move** with the technique that has caught this family twice: `assert_eq!(precomputed, full three-source resolution)` inside the accessor, whole workspace, then **inverted** to prove the probe fires.

- [x] **Step 5: measure, and re-profile `rexxcps`.** The hashing bucket is at about 5.3% after three applications, down from about 7.7%. Say what is left in it and whether anything nameable remains, because that is what decides whether this family is finished.
  **Answered: the family is finished on this path, and the sampled share was the wrong instrument for saying so.** The bucket's arms overlap (5.55-6.50% against 5.95-7.14%), so it is unresolvable at this resolution; counting `slot_of` calls directly instead, `rexxcps` at head enters it 3,080,203 times and **not one is a stem or a compound**. What is left is 2,800,003 `PARSE` targets -- every one of which already has its slot in `by_symbol`, measured -- plus `RESULT` and `SIGL` through `extra`. `PARSE` targets are this family's successor and are worth their own plan.

- [x] **Step 6: record it as the next entry of `phase-4f-record.md`, appended.** Entry 32.

## What this task must not do

Take the general fix of replacing the default hasher. If the four applications have removed the name-keyed lookups from the hot paths, the bucket should fall without it; doing both at once hides which one worked.
