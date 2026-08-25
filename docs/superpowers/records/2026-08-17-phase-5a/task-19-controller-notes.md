# Task 19: what the controller measured before dispatch

**Read this before you plan anything. The brief's crate-side readings are stale, and every claim in
it now matches.** Measured at BASE `9a0f249e9`, fresh directory, absolute paths, three descriptors
read separately, both sides bounded, `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`:

| program | oracle | crate, both engines |
| --- | --- | --- |
| `say .K~c` / `say .K~hasMethod("C")`, `::CONSTANT c 42` | rc 0, `42` then `1` | **identical** |
| `say .K~c`, `::CONSTANT c (1+1)` | rc 0, `2` | **identical** |
| `say .K~method("C")`, `::CONSTANT c 42` | rc 0, `a Method` | **identical** |
| `say .K~method("M")`, class-only `::METHOD m CLASS` | rc 159, `Compiled method "METHOD" with scope "Class".` frame then 97.1 | **identical** |
| floating `::CONSTANT c (1+1)`, no `::CLASS` | rc 157, 99.906 | **identical** |
| floating `::CONSTANT c 42`, no `::CLASS` | rc 0, `main` | **identical** |

The brief records the first three as diverging -- `97.1 rc 159`, `hasMethod` answering `0`, `rc 120`
at `~method`. Those were true when the plan was written and are not true now. **Task 18's install
passes and its `createConstantGetterMethod` work closed them.**

`phase-4-exclusions.txt`'s KNOWN GAP -- a `::CONSTANT` expression cannot send to a class declared
later -- is also closed behaviourally. Its own transcript program is **rc 0 `main` on the oracle and
on both engines** now. Moving that row to CLOSED DEFECTS is real work this task owes; the plan
already assigns it here, in its own commit.

## What this means for the shape of the task, and the trap in it

**Do not invent work, and do not ship decoration.** If the behaviour is already there, say so
plainly and let the task be verification, coverage and bookkeeping. That is an honest outcome and it
is what I expect. What is *not* acceptable is corpus rows that pass the moment they are written and
prove nothing about M7.

**The control is the whole test of whether this task added anything**, and the brief already names
the trap: `.K~c` cannot be the control's subject, because a class-side getter alone answers it `42`
and the control would then behave the same whether or not M7's instance half exists. The control the
brief requires is that **creating only the class-side getter leaves `.K~method("C")` raising 97.1
where the oracle answers `a Method`** -- so that program reddens.

**Run that control.** If it fires, the instance-side getter is real and this task's coverage is real.
If it cannot be made to fire, that is a finding worth more than the task: it would mean
`~method("C")` answers `a Method` for some reason other than the getter being on the instance
behaviour, and the corpus row would be witnessing nothing. Report which, with the mutation and the
corpus count it produced.

## Also owed

* The **instance-side reading is 5b's debt** -- it needs `~new`, which this phase does not have.
  Record it as debt beside the old plan's Task 7 debt. Do not imply `~method("C")` covers it.
* The two existing failure shapes stay green: the expression raising (rc 159, 97.1) and the
  class-less structural refusal (rc 157, 99.906).
* The blame-the-last-installed-class rule the old plan's Task 8 fix round pinned with two programs
  that only work as a pair -- keep the pair.

## Prerequisites confirmed in the tree

Task 18's second pass (`resolve_constants` with its own blame target) landed and its corpus row
`class_constant_expression_later_class.rex` exercises it. Task 9's `~method` is present and reads the
class's own instance dictionary. Corpus stands at **204 of 204** with every gate green at BASE.
