### Task 1: the stem's slot, resolved once

**Files:**
* Modify: `rust/crates/rexx-exec/src/plan.rs` (`CompoundName` gains the slot; `note_compound_name` keeps what it already computes)
* Modify: `rust/crates/rexx-exec/src/stem.rs` (the accessors take it)
* Modify: `rust/crates/rexx-exec/src/eval.rs` (the compound read and write paths pass it)

**Interfaces:**
* Consumes: `CompoundName { stem, tails }` and `Plan::compounds` from the previous plan; `read_stem_at`'s existing `Option<usize>` convention.
* Produces: the stem's slot on the same entry, and `_at` forms of the stem accessors that take it.

- [ ] **Step 1: put the slot where the split already is.**

`note_compound_name` calls `slot_for(stem)` and discards it. Keep it, on `CompoundName`, as an `Option` for the same reason the tail pieces carry one: `CompoundName::split` is entered by a fragment with no plan at all, and `Plan::bind` records a split for a compound `DO` control variable **deliberately assigning no slots**. An entry existing does not imply it carries slots, and that must stay visible in the type rather than remembered in a comment.

- [ ] **Step 2: give the accessors the `_at` form.**

`read_stem_at` is the pattern: take `Option<usize>`, use it, fall back to `slot_of(name)` when it is `None`. **Every `slot_of(stem_name)` in `stem.rs` is in scope** -- find them by reading the file rather than from a list in this plan, and say in the report how many there were and which you changed. A site you leave by choice is a finding to state, not an omission.

- [ ] **Step 3: pass the slot from the callers that have an entry.**

`eval.rs`'s compound read path already looks the entry up for the stem name; the write path is its sibling. A caller with no entry passes `None` and behaves exactly as today.

- [ ] **Step 4: the three-source question, which is the only place this can be silently wrong.**

`Interp::slot_of` resolves through the plan's names, then the activation's `extra`, then growth; a precomputed slot is the first source only.
The previous plan established that a tail piece **can** be bound in `extra` -- `do za.zi = 1 to 3` grows `ZI` into it -- and that a precomputed slot cannot shadow one, because a slot only reaches a piece via `slot_for` putting its name in the plan's map.
**Establish the same for a stem, by running rather than by analogy.** Construct a program where a stem name reaches `extra`, or show it cannot; either answer is a finding, and "it cannot happen" without a program is the failure mode this project has been bitten by repeatedly.

- [ ] **Step 5: prove the answers did not move.**

Build the accessor-level check the previous plan's reviews used: `assert_eq!(precomputed slot, the full three-source resolution)` inside the accessor, run the whole workspace, then **invert it** to prove the probe fires. A dead probe reporting zero firings looks exactly like a live one.

- [ ] **Step 6: oracle captures.** A stem read, a stem write, a bare stem, `DROP` of a stem, `PROCEDURE EXPOSE` of a stem, a stem whose tail changes between references, a compound inside an `INTERPRET`, and a compound `DO` control variable -- the last two being the entries that carry no slots.

- [ ] **Step 7: gates, then commit.**

- [ ] **Step 8: measure.**

`perf stat -e instructions:u` over `rexxcps` and every `Role::Loop` program, base against head, arms staged at **one fixed binary path** -- four byte-identical copies of one binary once spanned 3.47% on wall clock here purely because their `argv[0]` basenames differed in length.
**`compound` and `alloc4c` are subject axes, not controls**: `t.k` and `tab.i = i` are compounds, and the previous plan's own text got this wrong until a measurement caught it.
Measure the instrument's spread on each axis before reporting any difference under a percent.
Then re-profile `rexxcps` with `perf record` and report the hashing bucket, since the whole reason for this task is that the bucket did not fall last time.

---

## Measurement, after it lands

Record the result as the next entry of `docs/superpowers/plans/phase-4f-record.md` -- **appended, never by editing an earlier entry**, which entry 29 exists to restate.
Include the axes that did not move and the cost side if there is one.

## Deliberately out of scope

* Replacing the default hasher. If the stem was the last name-keyed lookup on the hot path, the bucket should fall without it, and doing both at once would hide which one worked.
* The compound `DO` control variable's own tail pieces, which still resolve by name because the whole dotted name holds the slot; the previous plan's report has the argument and its correction.
* Caching a stem's tails map or its values. Different optimisation, different invalidation story.
