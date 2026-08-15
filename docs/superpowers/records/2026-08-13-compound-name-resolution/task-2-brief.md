### Task 2: the slot, resolved once

**Files:**
* Modify: `rust/crates/rexx-exec/src/plan.rs` (the table gains the slot per variable piece)
* Modify: `rust/crates/rexx-exec/src/stem.rs` (`tail_key` reads slots; `read_by_name`'s remaining callers)

**Interfaces:**
* Consumes: Task 1's table.
* Produces: each variable tail piece carrying the slot `slot_for` assigned it at build time, so no name is hashed to read a tail.

- [ ] **Step 1: record the slot beside the piece.** `note_compound_name` already calls `slot_for` on each variable piece and discards the answer; keep it.

**Task 1 found a second filler of that table, and it assigns no slots at all.**
`Plan::bind` records the split of any compound-shaped spelling it binds, which is how a `DO` control variable that is itself a compound (`do a.i = 1 to 5`, whose tail `run.rs` re-resolves on every pass) gets an entry.
`bind` deliberately assigns no slot to the stem or to a piece, because the whole dotted name is already bound to one and adding slots for its parts would move every later slot number in the body -- a frame-layout change rather than a split.
So **an entry does not imply its pieces have slots**, and Task 1's measurement of the loop shape (`run::tests::a_compound_control_variables_tail_re_resolves_every_pass`) is where that bites.
Decide explicitly what a piece with no precomputed slot does -- an `Option` on the piece falling back to `read_by_name`, or `bind` starting to assign slots and the frame-layout consequence being measured -- rather than assuming every entry carries one.

- [ ] **Step 2: read the value by slot in `tail_key`**, through the same machinery `read_by_name` reaches after its own lookup, so an unset piece still derives its own spelling and nothing else changes.

- [ ] **Step 3: the three-source rule, which is where this task can be wrong**

`Interp::slot_of` resolves in three steps: the plan's names, then the activation's `extra`, then growth.
A precomputed slot is the *first* source only.
Establish and state whether a compound tail piece can ever be bound in `extra` rather than the plan -- `DROP (v)`'s run-time target and a fragment's new names are what put entries there -- and if it can, the precomputed slot must not shadow it.
**This is the one correctness question in the task**; answer it with a program that reaches the case, not with an argument.

- [ ] **Step 4: the same oracle captures as Task 1 Step 6**, plus: a compound whose tail piece is introduced by an `INTERPRET`, and a compound referenced both before and after a `DROP` of its tail variable.

- [ ] **Step 5: gates, then commit.**

- [ ] **Step 6: measure this task alone**, same instrument as Task 1 Step 8.

---

## Measurement, after both land

`perf stat -e instructions:u` over `rexxcps` and over every `Role::Loop` program in `rust/bench-programs/`, base against head, one run per arm.
The loop axes are the control: `compound` is the only one holding compound variables at all, so a movement anywhere else is codegen drift and belongs in the record as the cost side.

Wall clock only if the instruction counter shows a change worth trying to time, and then under the accept rule with **two** do-nothing controls -- entry 27 records four byte-identical copies of one binary spanning 3.47% because their `argv[0]` basenames differed in length, so every arm is staged at one fixed path.

Record the result as the next entry of `docs/superpowers/plans/phase-4f-record.md`, including the axes that did not move.

## Deliberately out of scope

* Replacing the default hasher. It would cut the same cost by a smaller factor and would hide whether the resolution work was removed.
* Caching a compound's *value* or its tails map. That is a different optimisation with a different invalidation story, and this project has an unmerged prototype of it on the C++ side.
* Anything about `Op` or the compiled stream. This plan is under both engines equally.
