## Task 6: the licensed divergences (D59a)

**Goal.** D59's licence has a witness the harness runs, and the fifth divergence Task 5 opened is
decided rather than left unstated.

**BASE:** `6eb234598`, the commit named in your dispatch. Read
`.superpowers/sdd/2026-08-27-phase-5b/global-constraints.md` and the 5a constraints it points at,
`rust/CLAUDE.md`, the plan's Task 6 section
(`docs/superpowers/plans/2026-08-27-phase-5b.md:567`-`:640`), and D59 and D59a in the spec
(`docs/superpowers/specs/2026-08-27-phase-5b-instances.md:562`-`:640`).

**Task 5's delivery 3 has landed**, so this task's precondition holds: `Interp::run_termination_uninits`
exists at `dispatch.rs:2641` and `lib.rs:7071` drains it. Written before that, this task's DEVIATION
would have pinned "never runs"; written now it pins "runs, but at the wrong point".

---

## What the controller measured. Every figure below is a claim to re-measure, not a premise

Three rules of this phase apply to this section in particular: a brief's "what is already there" is a
claim; a witness must be run against a control that makes it fail; and no check may rest on an order
the oracle does not reproduce. All figures below are 3 runs; take yours to 5. Standard oracle wrapper,
fresh empty run directory, three descriptors read separately, both crate engines.

### Divergence A -- the one D59a's table assigns to 5b

A class object's `UNINIT` runs at a driven collection on the oracle and in the termination sweep here.

```rexx
k = .Object~subclass('K', .MyMeta)
say 'before'
drop k
call gc 'force'
say 'after'

::CLASS MyMeta SUBCLASS Class
::METHOD uninit
  say 'class uninit'
```

```
oracle        rc 0, stderr empty   before / class uninit / after
ir            rc 0, stderr empty   before / after / class uninit
tree-walker   rc 0, stderr empty   before / after / class uninit
```

Silent: rc 0 and empty stderr on every side, stdout differing only in the *position* of one line.
Both crate engines agree with each other, so this is a crate-against-oracle row and not an
engine-against-engine one.

**Why the metaclass and not `~setMethod`.** `k~setMethod('UNINIT', ...)` from a program context is
`97.2 ... cannot accept private message "SETMETHOD"` at rc 159 on the oracle -- Task 3's
restricted-private check, reached again here. A metaclass carrying an instance-side `UNINIT` is the
route that works. It does **not** override `NEW`, so it is not the shape
`corpus/oracle-crashes.txt` entry 8 forbids; check that yourself before you run anything
metaclass-shaped.

### Divergence B -- the fifth, which this task has to decide

The oracle holds recently allocated objects out of a driven collection
(`Memory.hpp:107` `SaveStackSize = 10`; `RexxMemory.cpp:306` `collectAndUninit(bool clearStack)`,
and `GC('force')` passes `false`). This crate has no such hold, so its forced collection reaches an
object the oracle's cannot.

```rexx
say 'start'
o = .K~new
<N padding clauses, each `zI = 'padI'`>
drop o
call gc 'force'
say 'after-gc'

::class k
::method uninit
  say 'uninit ran'
```

```
N        oracle                            ir and tree-walker
0..8     start / after-gc / uninit ran     start / uninit ran / after-gc
9..      start / uninit ran / after-gc     start / uninit ran / after-gc
```

rc 0 and empty stderr on all three sides at every N measured (0, 1, 2, 8, 9, 10). Silent again.

**The window counts allocations, not clauses, and this is the controller's own finding rather than
Task 5's.** Replacing each padding clause with `zI = 'p'||'q'||'r'||'s'` moves the threshold from 9
clauses to 3:

```
3-concat clauses   N=1  start / after-gc / uninit ran      N=3  start / uninit ran / after-gc
                   N=2  start / after-gc / uninit ran      N=4  start / uninit ran / after-gc
```

Re-measure this before you rest anything on it. It is load-bearing for the decision below.

### A near miss you should not repeat

The controller's first two reconstructions of Divergence B **did not reproduce it** and would have
supported a "the plan's table is wrong" ruling against a true finding. Both put a `say` inside
`::method init`, or an extra `say` between `~new` and `drop`. Either allocates enough to evict the
new object from the ten-slot save stack, so the divergence vanishes and the oracle agrees at every
N. The shape is exact: nothing between `~new` and the padding, and the padding is the only thing
that moves the window. `corpus/lang/uninit_instance_collected.rex`'s own header comment carries the
three-row margin table and is the authority on the shape.

---

## Build

### 1. The DEVIATION rows, in a file you name and justify

The plan offers `docs/superpowers/plans/phase-4-exclusions.txt`'s DEVIATIONS section (rows 0..4
today, in an IMPLEMENTED / SCOPE / WHY shape, DEVIATION 4 being the closest precedent) or a Phase 5
successor file you name. **Say which and why in your report.**

One fact for that decision, which is a claim to re-measure: the only test that reads that file is
`rust/crates/rexx-exec/tests/builtin_status.rs`. It reads the whole text and searches it for
`KNOWN GAP: <NAME>` markers (`:696`-`:710`); it never parses sections and never counts DEVIATION
rows. The `excluded == 15` assertion at `:497` counts *status rows*, from
`rexx_inventory::builtins::wholly_excluded()`, not anything in the file. So adding a DEVIATION row
there cannot redden that gate. That makes the Phase 4 file *safe*; it does not by itself make it
*right*, which is the part you decide.

Precedent to follow, not to copy: DEVIATION 4 is licensed with a date and a named licensor. You do
not have that authority and neither does the controller; write the row with its measurement and its
reasoning, and say in your report that it is landing subject to Moritz's veto. The controller has
put the recommendation to him.

### 2. The runnable witness

`docs/superpowers/plans/phase-4-exclusions.txt` is prose nobody executes -- DEVIATION 4 carries
sixteen transcripts and nothing re-runs any of them. Build the executable half in the shape of
`ir_dual.rs:1075` `KNOWN_DIVERGENCES` and `:1105`
`the_known_engine_divergences_still_diverge_exactly_as_recorded`, whose doc states the property:
red if either side's answer moves in either direction, including a fix, which should delete the row
rather than update it. Reuse its non-empty-table assertion; an empty table asserts nothing.

**That table is not the harness you need, and copying it directly would build the wrong instrument.**
Its two rows compare *tree-walker against ir*, both in process. Yours compares *crate against
oracle*, so it needs `tests/support/oracle.rs` -- `locate()`, the `Oracle` run method, `CppOutcome`,
`ORACLE_DEADLINE`, `wrapped_exit_code`, `descriptor_diffs` -- and it must run **both** crate engines
and assert they agree with each other while both differ from the oracle in exactly the recorded way.
A row that checked one engine would go green on a build where the two engines had drifted apart.

Assert all three descriptors on both sides, not just the differing one: rc 0 and empty stderr are
what make these silent, and a row that stopped asserting them would still pass if the crate started
raising.

### 3. Decide Divergence B

The plan gives two ways out -- license it with a runnable witness, or build the hold (a ring of
recently allocated handles rooted from `Interp::alloc_with`) and pay the sitting -- and says this
task picks one and must not leave it unstated.

**The controller's recommendation is to license it, and the allocation-counting measurement is why.**
Building a ten-deep hold does not buy agreement: the oracle's window is ten *allocations*, and
reproducing the threshold means this crate allocating arena objects in one-to-one correspondence with
the C++ interpreter's, which it does not do and is not going to. A hold would move the divergence, not
close it -- the same "WHY EXACT AGREEMENT IS NOT AVAILABLE" shape DEVIATION 3 already uses -- while
changing collection *reachability* for every object on the hottest path in the interpreter, with a
`collect_stress` interaction, for an observable that needs a driven `GC('force')` to see at all.

**This is a recommendation and not a ruling.** If your own measurements contradict it -- in
particular if the threshold turns out to track something the crate *could* match -- say so and
license nothing until it is settled. Either way the outcome is a row: leaving it unstated is the one
answer the plan forbids, because a silent wrong answer with no row anywhere near it is the defect
class this phase is most exposed to.

If you license it, the row records that **no corpus program is on the diverging side** --
`corpus/lang/uninit_instance_collected.rex` deliberately pads, and its comment says why -- and that
row needs a witness in the same shape as Divergence A's.

### 4. What this task does not own

The other three consequences of D59a are not yours. `~subclasses` keeping a dropped class and a
`WeakReference` still answering are **5c's**, because 5c lands those methods. The OOM asymmetry is
stated in D59a with its measurement and needs no runnable witness, because a witness that OOM-kills a
gate run is worse than the prose. Record them where they are owed; do not build them.

---

## Done when

* The DEVIATION exists in a named file, with the file choice justified.
* The harness runs its witness for Divergence A, and for Divergence B if you license it.
* **The control is recorded as run, with its transcript.** For Divergence A that is: deliver the
  class's `UNINIT` from the driven collection instead of the termination sweep, and the DEVIATION's
  assertion reddens. That is the applicable mutation -- "make the crate match the oracle" is not one
  anybody can apply without undoing D59. Name and run the equivalent control for Divergence B's
  witness if you build one.
* The five gates each exit 0, statuses read unpiped, and the phase gate
  (`REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c
  --test gate_table_d --no-fail-fast`) is reported with its table C and table D not-agree counts.
  Table C 5b has one red row today, `methodsbyclass`, which is Task 8's and not yours.

## Rules

* Correct the plan or spec file where you find it wrong; do not correct your brief or your report
  around it. If Divergence B's table in the plan is wrong in any particular, fix the plan.
* Never run a program in `rust/corpus/oracle-crashes.txt` and never construct one of those shapes.
* No `unsafe`. Stop and say so rather than reach for it.
* Oracle probes run from a fresh empty directory. Read stdout, stderr and exit status as three
  separate descriptors; never `2>&1`.
* Commit with `git commit -F <file>`, naming paths explicitly. Never `git add -A`, never amend,
  never a bare `git stash`, never `rm` with a star glob, never `git checkout --` on a file you have
  edited. **Re-read `git diff --cached --stat` immediately before committing**: `Cargo.lock` has
  been rewritten by a gate run between review and commit on this plan before.
* Write your report to `.superpowers/sdd/2026-08-27-phase-5b/task-6-report.md`, and say plainly what
  you did not do.
