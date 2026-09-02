## Task 7: `~copy`, `~run`, `~send`/`~sendWith`, `~start`/`~startWith`

**Goal.** Each has a witness in `corpus/phase-5b.txt` and agrees.

**BASE:** `cabc32418`, the commit named in your dispatch. Tree clean. Read
`.superpowers/sdd/2026-08-27-phase-5b/global-constraints.md` and the 5a constraints it points at,
`rust/CLAUDE.md`, the plan's Task 7 section
(`docs/superpowers/plans/2026-08-27-phase-5b.md:670`-`:719`), and D67 and D68 in the spec.

---

## What is already there: nothing of this task's. Verified, not assumed

The native method table in `crates/rexx-exec/src/dispatch.rs` registers, for `Object`: `CLASS`,
`DEFAULTNAME`, `HASMETHOD`, `IDENTITYHASH`, `INIT`, `ISA`, `ISNIL`, `NEW`, `OBJECTNAME`,
`OBJECTNAME=`, `REQUEST`, `SETMETHOD`, `STRING`, `UNSETMETHOD`. **None of `COPY`, `RUN`, `SEND`,
`SENDWITH`, `START`, `STARTWITH` is present, and there is no `Message` class at all** -- not the
class, not `RESULT`, `COMPLETED` or `HASERROR`. `Array` has `AT`, `ITEMS`, `MAKESTRING`, `SIZE`,
`TOSTRING` and `[]`, and no `[]=`, `OF` or `NEW`.

So `Message` is the largest unnamed piece of this task. The plan says "plus enough of `Message` for
`~result`, `~completed` and `~hasError`", which reads like an addendum and is a new class. Scope it
before you start and say in the report what "enough" turned out to mean.

Re-derive the list above yourself before you rely on it; a table read by eye is a claim like any
other.

---

## A Done-when criterion cannot be met as written, and the plan needs correcting

The plan's acceptance says the `send` witness carries "the array form's starting-class override --
measured, oracle rc 0, `o~send('M')` answers the subclass's method and `o~send(.Array~of('M',
.Base))` the base's".

**The behaviour is real. The spelling is not available.** Measured by the controller, oracle,
standard wrapper, fresh empty directory, rc 0 and empty stderr:

```rexx
o = .Sub~new
say 'plain' o~send('M')
say 'array' o~send(.Array~of('M', .Base))
::class Base
::method M
  return 'base'
::class Sub subclass Base
::method M
  return 'sub'
```
```
oracle   plain sub / array base
```

But `.Array~of` is not implemented in this crate, and it is **not this task's** -- the plan puts it
on Task 9's list. A witness written with it would be red for a reason that is not Task 7's.

**The array literal is an equivalent route and it works today on all three sides.** Same program with
`o~send(('M', .Base))` is oracle `plain sub / array base`, rc 0, empty stderr; and
`a = ('M', 'x') ; say a~items ; say a[1]` answers `2` / `M` identically on the oracle and on this
crate's `ir` engine. So the witness is reachable at BASE without `.Array~of`.

Use the literal spelling and **correct the plan's Done-when to match**, rather than correcting your
brief or your report around it. Check the tree-walker too; the controller measured `ir` only on the
literal.

---

## Measured in the plan, and every line of it is a claim to re-measure

Two of the plan's own measurements do not discriminate what you build, and it says so; the other
figures in it have not been re-measured by the controller and four such paragraphs have been wrong on
this plan already, always in the direction of making the task look smaller.

* **`~copy` needs a write-through witness.** The copy's initial values and `(o == c)` read the same
  whether the pools were copied or shared, so a witness of those alone is met by a wrong
  implementation. The plan's shape: after `c~set('changed')`, `o~get` is `orig` and `c~get` is
  `changed`. Without both reads the named control cannot redden anything.
* **`~run`'s refusal is not its behaviour.** `o~run(...)` from a program context is `97.2` at rc 159,
  which fires at dispatch before any body runs -- so a witness of the refusal alone agrees the moment
  `RUN` is a restricted-private name with no body behind it. Witness the allowing arm too: from a
  method, `self~run('use arg x; return "ran" x', 'I', 5)` answers `ran 5`.
* **The D67 interaction the plan states for `~run`:** the run method's `EXPOSE` reaches the object's
  *float* pool, not the class's, so `self~run('expose v; return v*10')` raises 41.1 at rc 215 where
  the class's `v` is 7. Re-measure this one specifically before you build to it.

**`~run`'s option surface is the largest in this task and nobody has enumerated it.**
`oodocs/rexxref/en-US/fundclasses.xml`, `mthObjectRun` (the anchor exists; the controller confirmed
the id, not its content): a Method object, a source string, or an Array of source strings; times
`Individual`, `Array`, or neither; first letter only. Enumerate it from the document before building
and say in the report which arms this phase owes and which it defers, with the reason.

`send` and `sendWith` also take an array whose first item is the name and whose second is a class to
start the method search from -- the dynamic form of the `~m:scope` override 5a built statically,
documented at `mthObjectSend`/`mthObjectSendWith`.

**D68 bounds what may be asserted about `~start`.** Its interleaving with the program that started it
is not reproducible (twenty runs, two orders), and `~completed` sampled *before* `~result` is part of
that interleaving (forty runs, thirty/ten). A corpus program may assert `~result`'s value and
`~completed` after it, and nothing else about timing. What `~start` does with a method that **raises**
is not measured and must not be asserted.

---

## Done when

* Each of the four paths has a witness in `corpus/phase-5b.txt` that agrees on three descriptors on
  both engines.
* The `~start` witness asserts `~hasError` after `~result`, and the `send` witness carries the array
  form's starting-class override in the literal spelling above. Both are mechanism-set items the
  task's acceptance had not covered.
* **The control is recorded as run, with its transcript**: making `~copy` share the receiver's scope
  pools instead of copying them reddens the `~copy` witness at rc 0 -- which it can only do with the
  write-through lines. Name and run a control for each of the other three too, or say why one does
  not exist.
* The five gates each exit 0, statuses read unpiped, and the phase gate reported with both tables'
  not-agree counts. At BASE that is table C 5b **1** not-agree (`methodsbyclass`, Task 8's) and table
  D 5b **0**; anything else red is yours.

## Rules

* Correct the plan or spec where you find it wrong -- starting with the `.Array~of` Done-when above.
* Never run a program in `rust/corpus/oracle-crashes.txt` and never construct one of those shapes.
  Read that file first. Note entry 7: `GUARD ... WHEN` blocks indefinitely, and `~start` work is
  close enough to concurrency that you should know the entry before you write a probe.
* No `unsafe`. Stop and say so rather than reach for it.
* Oracle probes run from a fresh empty directory. Three descriptors read separately, never `2>&1`.
  Both engines, always.
* **A gate run does not survive the turn that starts it.** Task 6 lost three runs to this, one dying
  mid-gate-4 with three green statuses on disk and no fourth, while looking in flight from inside the
  turn. Write each gate's status to a file as it finishes, arm a waiter that exits on the process
  vanishing as well as on completion, and read the statuses in the same turn you commit.
* Take a hash of `git status --porcelain` and `git diff` before the first gate and after the last, so
  a run spanning a write is detected rather than promised against. Hash the repository's view, not a
  list of paths: a path list has to be kept in sync with the commit and eventually will not be.
* Commit with `git commit -F <file>`, naming paths explicitly. Never `git add -A`, never amend, never
  a bare `git stash`, never `rm` with a star glob, never `git checkout --` on a file you have edited.
  Re-read `git diff --cached --stat` immediately before committing and confirm `Cargo.lock` is absent.
* Write your report to `.superpowers/sdd/2026-08-27-phase-5b/task-7-report.md` (git-ignored, like
  every sibling report). Say plainly what you did not do.
