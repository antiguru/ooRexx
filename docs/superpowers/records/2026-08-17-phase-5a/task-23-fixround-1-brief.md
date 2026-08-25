# Task 23, fix round 1

Review: `.superpowers/sdd/2026-08-17-phase-5a/task-23-review.md`. **Spec compliance PASS.** Task
quality CHANGES REQUIRED: 2 major, 2 moderate, 7 minor. The bootstrap itself is not in question --
the reviewer swept the bootstrapped state far past the corpus and found the five checks sound.

## M1. Ruling: FIX it, do not record it.

I reproduced it. `say .Validate~number('LENGTH', 'abc')`:

    oracle   3700 *-*  Method NUMBER with scope "Validate" in package "REXX" (no source available).
             Error 88 running REXX line 3700:  Invalid argument.
    crate    3700 *-*  raise syntax 88.902 array(name, number)
             Error 88 running /abs/path/to/v.rex line 3700:  Invalid argument.

The reviewer offered me the choice of closing it or recording it as a licensed divergence. **Close
it.** Three reasons, in order:

1. It is **a refusal that became a wrong answer** -- before this task `.Validate` was an unbuilt
   `.environment` name and every one of these programs was a loud rc 120. That is the class the
   global constraints single out as the one no gate can see.
2. **It leaks an absolute filesystem path into stderr.** That output is machine-dependent, so the
   program could never be a corpus row on any checkout. A divergence I cannot write a row for is a
   worse thing to license than one I can.
3. The breadth is 52 of 73 `::METHOD ... CLASS` pairs across the two embedded files. That is not an
   edge.

The mechanism is known and the reviewer verified it: an image-saved package carries no source, so
`PackageClass.cpp:589` calls `RexxActivation::formatSourcelessTraceLine`
(`RexxActivation.cpp:5057`), rendering `Message_Translations_sourceless_method_invocation`
(`messages/rexxmsg.xml:6525`): `Method &1 with scope "&2" in package "&3" (no source available).`
The error line then names the package rather than a file. Mark an embedded library package
source-less, render that frame for a frame inside one, and answer `REXX` as the program name.
**Verify each substitution against the oracle rather than against the message file**, and add corpus
rows -- they are writable once the path stops appearing.

**Not yours, and do not fix it here:** the rc/condition half the reviewer separated out (40.3/40.4
`Incorrect call to routine` where the oracle raises 93.901/93.902 `Incorrect call to method`) is a
pre-existing `USE STRICT ARG` defect, proven on the pinned pre-5a binary with a purely user-declared
class method. Record it as inherited, with the reviewer's minimal reproduction, and say that this
task made 52 programs reach it. Do not widen into it.

## M2, D1, D2

* **M2** -- the code is right and the *documentation is backwards in three places*. The reviewer
  verified the C++ distinction holds exactly as you stated it, so this is prose only. Fix all three.
* **D1** -- `environment.rs:1422` says "This phase loads one program, so its path is the running
  program's". Every run now loads four. The behaviour is right (`record_package_class`'s
  `library_programs` skip keeps `.Alarm~package~name` answering `REXX`, verified); the comment a
  reader would rely on says the opposite.
* **D2** -- `rexx-lib`'s recursion argument rests on "neither non-entry file contains the word `call`
  at all", which is false: `/bin/grep -in call` on `StreamClasses.orx` answers 2, both in comments.
  The property still holds; the evidence does not. Also note the reviewer's sharper point: the
  assertion is case-sensitive and prefix-anchored, and the windows `PlatformObjects.orx` you
  correctly declined to embed contains exactly `  call 'orexxole.cls'`. Make the check as wide as
  the claim.

## The minors, with two worth doing properly

* **m1 -- correct the attribution, keep the observable.** Every figure reproduces from the TSV and
  the reviewer confirmed +148.4M per run independently with `perf stat`. Two things to fix: the
  control puts `strings` at 5,406.48 against 5,433.48, which is **-0.50%**, not "within a rounding
  step" -- 27 instructions a pass, and your own table prints both numbers. And "so this is the
  collector" is an inference, not an attribution: the control separates *the bootstrap ran* from
  *the new code exists*, not the collector from a larger resident heap's other costs, and no
  collection count was read. State the observable and stop there, or read the count.
* **m5 -- take the before-figure. It was available.** The reviewer found
  `target/release/deps/gate_table_c-88c89472c7b48c4f`, built between your base commit and your first
  code commit, and ran it: **before 15 agree / 35 diverge-both / 13 diverge-stdout, after 62 agree /
  1 diverge-both**, so the delta is **+47 `agree` and the elimination of all 13 silent-wrong-answer
  rows**. That is a far better statement of what this task did than "I could not take it". Record it
  with the reviewer's own provenance caveat: the binary's source state is inferred from its mtime.
* m2, m3, m4, m6, m7 -- take them from the review directly.

## How to close

All five gates. M1 changes `src/`, so a sitting is owed; the bootstrap's own cost is already
recorded and is not what that sitting is measuring. Invert whatever control you build for M1.
`cp` before mutating, restore from the copy, never `git checkout --`. Append to the report.
