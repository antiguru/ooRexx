## Task 5 fix round

**BASE:** `d708491a7`. Review: `.superpowers/sdd/2026-08-27-phase-5b/task-5-review.md` -- read it in
full. Your own report is `task-5-report.md`. Also re-read `global-constraints.md` and the 5a
constraints it points at, `rust/CLAUDE.md`, and D60/D61/D69 in the spec.

Verdict: **spec compliance FAIL** on one of your brief's own named arms, quality changes requested.
2 Critical, 3 Important, 7 Minor. Fix all of CRIT-1, CRIT-2, IMP-1, IMP-2, IMP-3 and the Minors.

**The order rule survives review intact and is not in scope here.** The reviewer predicted bucket
order before running on sets your report never used -- including the signed-char half of the fold,
which no transcript in your report or the spec can see -- and every prediction held on the oracle and
on both engines. Do not touch it except where CRIT-1 forces an entry to be made in a different place.

### CRIT-1 -- the arm the brief said must agree does not, and it is silent

Confirmed by the controller, release binary against the oracle, fresh empty directory, three
descriptors, both engines:

```
say 'main'
::class m mixinclass Object
::method uninit class
  say 'uninit on M'
::class k inherit m

  oracle  rc=0  main / uninit on M / uninit on M
  crate   rc=0  main / uninit on M
```

The oracle fires the inherited finalizer **twice** -- once for `K`, which inherits the method, and
once for `M`. Empty stderr on both sides, so this is the silent class. `check_uninit` is never re-run
after an inherit on either path: `dispatch.rs:4300` `native_class_inherit` does not call it, and
`lib.rs:5444` calls it **before** the `class.inherit` loop at `:5447`, while `inherit_mixin`
(`lib.rs:5609`) does not call it at all. Four shapes, all rc 0 with empty stderr: the directive
mixin arm above, its instance-side twin, and both sides of a runtime `.K~inherit(.M)`.

**Your report's sentence "check_uninit is called from the same places the oracle calls it" is false,
and it is the argument the ordering rests on.** Correct the sentence as well as the code, and check
the same way round for `~uninherit`: if inheriting can add a `UNINIT`, uninheriting can remove the
reason for one, and D58's two families make the mutators the place to look.

### CRIT-2 -- a fixed point where the oracle makes one pass

`run_termination_uninits` loops `take_uninit_flagged` to a fixed point; `MemoryObject::runUninits`
is one pass. A `UNINIT` that allocates one instance of a `UNINIT` class per call: bounded at 8, the
oracle prints `u 1` / `u 2` and this crate prints `u 1` .. `u 8`; unbounded, the oracle exits rc 0
and this crate is **killed at 15s, rc 137, with all buffered stdout lost**. A hang is worse than a
wrong answer. Read what `runUninits` actually does about objects flagged during the sweep before you
change the loop, and cite it.

### IMP-1 -- no `processing_uninits` interlock

`RexxMemory.cpp:341`-`:347`. A `GC('force')` inside a `UNINIT` runs the next finalizer inline: oracle
`g uninit / inner built / g done / end / uninit K`, crate `g uninit / inner built / uninit K /
g done / end`. Silent.

### IMP-2 -- the witness that cannot fail, which is why CRIT-1 shipped

`corpus/lang/uninit_class_mixin.rex` gives `K` its own `::method uninit class`, so deleting
`inherit m` from it changes no output on either side; it cannot witness its stated subject and it
reddens in lockstep with `uninit_class_inherited.rex` under every control. **The controller's probe
above is the shape it should have been.** This is the hazard your brief named in its own words --
"check that your witness's value can actually move" -- and it is the third instance in this phase.
When you rewrite it, run the deletion (`inherit m` removed) and show that the program's output moves;
and run it against the suite *without* the new program to show what it adds that nothing else does.

### IMP-3 -- the quadratic table's pre-fix column does not reproduce

Interleaved, three rounds each, debug, `REXX_ENGINE=ir`, the reviewer measured 16,000: 2.6s against
your 0.30; 64,000: 37-45s against your 0.86; 128,000: 155-182s against your 9.67; 200,000: killed at
300s against your 49.55. **Every post-fix cell reproduces exactly and so does the no-`UNINIT`
control**, so load does not explain it -- and your pre-fix figures at 16k and 64k are identical to
your post-fix ones. That is the stale-binary shape: a rebuild that did not happen, so the old binary
answered for the new one. Correct the table, say what the instrument was, and state the fix's real
size, which is larger than you claimed. **Rebuild explicitly between arms and check the binary's
mtime**, and interleave.

### The Minors, all of them

`corpus/README.md:47`'s spliced sentence, which moves what the surrounding paragraph asserts and runs
to 123 columns; the plan's 8/9 figure not describing the committed two-clause program; `uninit_bucket`
folding a lossy `String`, so a non-UTF-8 class id is hashed after mangling (pre-existing, and the
reviewer witnessed it with no `UNINIT` anywhere -- record whether you close it or hand it on, with the
transcript either way); **"a class object is the exception" is a universal with a second member** --
`.nil` hashes to a fixed `0xDEADBEEF`, four runs of four -- so name the set or drop the quantifier;
one measured sequence stated in three places where one of them asserts it; two passages of experiment
narrative that are history rather than justification for the design as it stands; and
`Heap::clear_uninit` now reached only from its own test, which is a fact to act on or to record.

### Gate 5

Your third detached attempt is still running at `scratchpad/task5/g5.status` against `d708491a7`.
**That commit is no longer the end of this task**, so its result is a data point and not the gate.
Run gate 5 to completion on your fix commit and read its status unpiped from a file, detached the way
the third attempt was.

### The performance sitting

Not yours. The controller takes it on a quiet machine; your reading that the failure was contention
rather than a machine property matches the controller's own successful eight-axis sitting earlier the
same day.

### Then

All five gates from `rust/`, each status read unpiped, plus the phase-gate command. Write your report
first and append as you go. Commit with `git commit -F <file>`, naming paths explicitly, never
`git add -A`, never amend. Message the controller when you finish.
