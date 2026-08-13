# A bare stem's slot already exists and nothing uses it

**Goal:** Stop hashing a bare stem's name where the plan has already assigned it a slot, in the three places that still do -- and find a workload that can see the one which pays per iteration.

**Status:** open, unassigned. Found 2026-08-13 by the stem-slot task of `2026-08-13-stem-slot-resolution.md`, which corrected the comments that denied it and deliberately did not take the optimisation; the third site was found by that task's review. **Confirmed real by both.**

## What was believed, and what is true

Three comments in the tree said a bare stem write has no slot to resolve ahead of the write, so it must go by name.
Measured on `zs. = 'one'`, both engines: `by_symbol[id]` is `Some(0)` and equals the slot `stem_assign` then hashes for.
Measured on `do cv. = 1 to 3`, both engines: `by_symbol=Some(0)`, `slot_of=0`, on all four writes.
The slot exists. `Plan::bind` assigned it when it walked the body; nothing reads it back.

This is the same shape as the three applications already landed -- `compound_parts` re-splitting an interned name, a tail piece re-hashing its own, a stem half re-hashing its own -- and it is the last member of the family that is known about.

## The three sites

* **`stem_assign`**, reached from `run.rs`'s `assign_expr_target` for `zs. = value`. Per statement.
* **`replace_stem`**, the other bare-stem operation `stem.rs` leaves resolving by name. Per statement.
* **`bind_control`'s `NameShape::Stem` arm**, where `control_slot` answers `None` **by choice** and the slot it could answer is `Some(0)`. **Reached on every pass of a stem-controlled `DO`, so this one pays per iteration.**

The first two were named by the implementer; the third by its reviewer, and it is the one worth the task.

## What makes this a task rather than a patch

**No current benchmark axis would show it.** `compound` and `alloc4c` hold compounds, not bare stem writes; nothing in `rust/bench-programs/` runs a stem-controlled `DO`.
So the first step is a workload, and the measurement discipline this project now holds itself to says an axis that cannot see a change makes its result unstatable rather than zero.

- [ ] **Step 1: write the workload before the fix.** A stem-controlled `DO` over enough iterations to be measurable, and a bare-stem-write loop beside it. Decide whether either belongs in `rust/bench-programs/` permanently -- if it does, it is an axis every later entry inherits, so say why rather than adding it quietly. Measure the base with `perf stat -e instructions:u`, arms at one fixed binary path, and record each axis's own spread before quoting any difference.

- [ ] **Step 2: take the slot, in the same shape as the three that landed.** `Option<usize>` with a `slot_of` fallback, the difference visible in the type. Read `5eaa3c8e8` for the pattern and `a93bfb548` for the tail-piece version.

- [ ] **Step 3: answer the `extra` question again, by running.** It has been yes all three times -- `do za.zi = 1 to 3` grows `ZA.`, `interpret "zq.1 = 7"` grows `ZQ.`, `zn='ZR.1'; drop (zn)` grows `ZR.` -- and a fourth site is not exempt because the first three were. Instrument `slot_of`, run, and include a control that resolves through the plan.

- [ ] **Step 4: check the answers did not move** with the technique that has caught this family twice: `assert_eq!(precomputed, full three-source resolution)` inside the accessor, whole workspace, then **inverted** to prove the probe fires.

- [ ] **Step 5: measure, and re-profile `rexxcps`.** The hashing bucket is at about 5.3% after three applications, down from about 7.7%. Say what is left in it and whether anything nameable remains, because that is what decides whether this family is finished.

- [ ] **Step 6: record it as the next entry of `phase-4f-record.md`, appended.**

## What this task must not do

Take the general fix of replacing the default hasher. If the four applications have removed the name-keyed lookups from the hot paths, the bucket should fall without it; doing both at once hides which one worked.
