# The `>K>` line a `DO OVER ... FOR` does not print

**Goal:** Print the `FOR` keyword echo that a `DO name OVER expr FOR expr` owes, on both engines, and establish whether any other header role withholds one.

**Status:** open, unassigned. Found 2026-08-12 by Task 2 of `2026-08-12-condition-promotion.md`, whose oracle captures crossed it; not fixed there, because the fix moves trace output on both engines and that needs its own differential rather than a corner of a promotion task.

## The divergence, measured

`HeaderRole::OverFor::keyword()` answers `None`, so no `Op::TraceKeyword` is emitted and the tree-walker echoes nothing either.
The oracle echoes.
Captured 2026-08-13 on `trace r`, program `zs = 'a b c'` then `do qq over zs for 1`, both under the standard oracle wrapper from a fresh empty directory:

```
oracle                              this crate
  >K>   "OVER" => "a b c"             >K>   "OVER" => "a b c"
  >K>   "FOR" => "1"                  (nothing)
```

Task 2 reproduced the same on `trace i` and on `do qq over 4.5 for 2`, and the review reproduced both independently.
**Both engines agree with each other**, so this is the role table withholding a keyword and not anything the compiled form does.
`ir_dual_cases/loop-header-values` holds a transcript, which is what stops the gap moving unnoticed while it is open.

## Why it is a defect and not an exclusion

`phase-4-exclusions.txt` holds work assigned to a later phase and permanent chosen differences.
This is neither: it is a difference nobody chose, in a construct that is otherwise in scope and implemented.
Its own preamble warns that a file which absorbs whatever turns out hard is an artifact of the phase it gates, so the row does not go there.
The standing rule is that the two engines agree with the oracle byte for byte, with `MAX_EVAL_DEPTH` the one accepted divergence, and this is a second one until it is fixed.

## The task

- [ ] **Step 1: find out whether `OverFor` is alone.**

Capture the oracle for every `HeaderRole` variant under both `trace i` and `trace r`, one program per variant, and write the transcripts down before changing anything.
`Initial` is the one other variant documented as echoing no `>K>`, and that is recorded as measured rather than assumed -- confirm it rather than trusting the comment, since the comment beside `OverFor` made the same kind of claim and was wrong.
A second withheld keyword found here changes the shape of the fix from one variant to a table.

- [ ] **Step 2: give the role its keyword, on the tree-walker.**

`HeaderRole::keyword()` in `run.rs`. Correct its doc in the same edit: it currently says `None` is "for the roles the oracle echoes nothing for", which is the sentence that made the gap invisible.

- [ ] **Step 3: let the compiled engine follow.**

`compile.rs` emits `Op::TraceKeyword` where `role.keyword().is_some()`, so the compiled side should need no change of its own -- verify that rather than assume it, and if it does need one, that is worth a note in the report.

- [ ] **Step 4: move the transcripts.**

`ir_dual_cases/loop-header-values` pins the gap on purpose; its rows now change. Any `trace_oracle` transcript holding a `DO OVER ... FOR` changes too. Re-capture from the oracle, never by hand.

- [ ] **Step 5: run the corpus sweep and the gates**, and say in the report how many corpus programs changed output. A construct this narrow should move few; a surprise there means Step 1 missed something.

## What this task must not do

Fix any other trace divergence it happens to find.
Write it down and leave it: the value of this task is that one line of output changes and the differential says exactly which programs saw it.
