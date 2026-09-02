## Task 1 fix round

**BASE:** `c245dc418`. The review is `.superpowers/sdd/2026-08-27-phase-5b/task-1-review.md`; read it
in full. Your own prior report is `task-1-report.md`. Also read
`.superpowers/sdd/2026-08-27-phase-5b/global-constraints.md`, the 5a constraints it points at, and
`rust/CLAUDE.md`.

Verdict was CHANGES REQUESTED: 2 Critical, 4 Important, 3 Minor. Fix C1, I1, I3, I4, M1 and M3, plus
the missing self-review. C2, I2 and M2 are ruled below and are **not** yours.

### C1 -- the one that matters. A named instance is an operand.

Verified twice, by the reviewer and independently by the controller, release binary against the
oracle from a fresh empty directory, three descriptors, both engines:

```
o~objectName = '123'; say o + 1        oracle rc=159 (97.1)   crate rc=0 out=124
o~objectName = '123'; say (o = 123)    oracle rc=0   out=0    crate rc=0 out=1
```

The second is rc 0 with empty stderr on both sides and different stdout: the silent wrong answer this
phase calls its worst defect, shipped by this task. The cause is the `Body::Instance { name: Some(..) }`
arm of `heap_to_number` (`value.rs:1160`-`:1163`); an *unnamed* instance is correct everywhere. Note
that `eval.rs:1486`'s own `debug_assert` says this cannot happen, and a debug build panics at rc 101
on the comparison probe -- so the tree also has a debug/release split here.

The reviewer validated the deletion: with the arm gone `datatype(o)` still answers NUM, `+` and `=`
return to the licensed loud refusal, and `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec
--no-fail-fast` is `270 of 270 matching`. **Do not take that on trust -- re-measure it.** The logical
and prefix-`\` surfaces are a second, separate change: the reviewer's reading is that the gap check
has to run before `logical_value`. Cover all four families -- arithmetic, comparison, logical, prefix
-- and **probe past them**: concatenation, `||`, abuttal, `SELECT WHEN`, `IF`, `DO WHILE`, a numeric
`DO` control, and anything else that takes an operand.

**This surface needs a committed witness**, in `corpus/phase-5b.txt`. No gate row sends an operator to
an instance, which is why the defect shipped. A named instance whose name is numeric is the shape;
make sure the witness covers the silent arm (`=`) and not only the loud ones.

**Reconcile C1 with M1 before you write either.** M1 says `text_len_inner` gained a `Redirect` arm and
no `name: Some` body arm, so a named instance reaching it falls to an `unreachable!`; the reviewer
could not construct a program that gets there, because `reqstr_armed` converts first. Deleting
`heap_to_number`'s named arm moves what a named instance does on the numeric paths, so work out what
`Redirect::of` and `text_len_inner` do afterwards and make the four sibling functions consistent.
Assert the property in a test rather than describing it in a comment.

### I1 -- the plan's Task 9 list is short by one

A third divergence is measured and the plan's Task 9 paragraph records only two. `::METHOD defaultName`
returning `overridden`, then `o~zzz`: oracle stderr `Object "overridden" does not understand message
"ZZZ".`, crate `Object "a DN"`. Same program without the override agrees byte for byte. Add it beside
the trace one, in the plan file.

### I3 -- two rooting sentences are wider than what holds

The plan's new Task 1 paragraph says "both engines enter a method body through that one function", and
`pool_owner`'s new doc (`run.rs:3073`-`:3077`) says the temporary `message_term` takes "is" the root.
Both are false as written: `send_message` has other callers, four of them added by this commit, and
**your own witness is the counterexample** -- the `INIT`-side `SELF` clobber reaches `pool_owner` with
`message_term`'s temporary holding the class, not the object, which is what control 4 shows. Narrow
the plan's sentence to a program's message term, and make `pool_owner`'s doc name both roots.

### I4 -- the ordinary-send half is argued, not witnessed

The reviewer removed `message_term`'s `push_temp(receiver)` (`dispatch.rs:2473`) in a scratch copy and
`collect_stress` stayed 8 passed / 0 failed. So the claim the plan now asserts has no witness under the
harshest collector setting this project has, and the `go`-side `SELF` clobber contributes nothing to
the rooting question. Your report applied exactly this rule to the third root you deleted. Apply it
here: either add a case that reddens when that line goes, or say in the plan that the ordinary-send
root is argued rather than witnessed. **Do not delete the line** -- absence of a witness is not
evidence it does nothing.

### M3 -- the commit message's rendering sentence

It says the derived rendering "leaves only this crate's own renderings on the wrong side of that
line"; `TRACE`'s value lines are on the wrong side too and are the oracle's rendering shown to the
user. Your code comment at `value.rs:803`-`:810` has it right. You cannot amend the commit, so correct
it in the fix commit's own message and in the plan file if it repeats there.

### Ruled not yours

* **C2** (a debug build aborts once a class defines `UNINIT`: `native_new` sets `Object::has_uninit`
  and `Interp::collect`'s `debug_assert` still says nothing does). Real and verified. It is the seam
  the plan designed, and Task 5 delivery 1 is literally "replace `Interp::collect`'s assertion with
  the delivery". **Ruling: Task 5 runs immediately after this fix round.** Leave the assertion alone;
  do not paper it over. Do not add a program with that shape to the corpus.
* **I2** (`objcla` is now a silent `diverge-stdout`). Task 2's, already written into the plan.
* **M2** (an `objectName` override answering an object: oracle rc 251 exhausts, crate rc 0 answers).
  Record it in your report as a known divergence and add it to the plan's Task 9 list beside I1's;
  do not build recursion to match an oracle stack overflow.

### The self-review

Your report has no self-review section and every other report in this phase does. Add one.

### The performance sitting, which is now takeable

Your report recorded the sitting as unmet because the machine exposed one usable counter and
`perf stat` multiplexed the `cycles:u,instructions:u` pair to about 50% each. **The machine has since
rebooted and the pair now schedules whole** -- controller-measured,
`perf stat -x, -e cycles:u,instructions:u -- /bin/true` reads `100.00` on both rows. So the absence
was transient. **Take the sitting for this fix commit**, with the command your report already
identified (the eight axes against `bench-baselines/pinned/rexx-run-15a1ffa98`, interleaved,
`--baseline bench-baselines/phase-5a-arms.tsv`), and re-verify the pin's `sha256sum` against
`PINNED.md` first. Confirm the pair reads 99.995 or better in the run itself; if it multiplexes again,
say so with the transcript and frame it with the probe before and after, as your report did.
`dispatch.rex` owes a first baseline row rather than a comparison.

### Then

All five gates from `rust/`, each status read unpiped, plus the phase-gate command. The corpus figure
should move by the witnesses you add. Commit with `git commit -F <file>`, naming paths explicitly,
never `git add -A`, never amend. Write your report first and append as you go; message the controller
when you finish.
