# Task 23, fix round 2 -- the last one

Re-review: `.superpowers/sdd/2026-08-17-phase-5a/task-23-rereview.md`. **This is the final round of
this task.** Whatever survives it I adjudicate and close.

**The code is right and is not in question.** The reviewer verified M1's fix is *the oracle's rule*
rather than a shape fitted to the probes -- `Activity::generateProgramInformation`
(`concurrency/Activity.cpp:1090`-`:1123`) takes the first frame whose `getPackage()` is not null and
uses that one frame for both `POSITION` and `PROGRAM`, which is exactly what taking the innermost
line-bearing site does, because the crate's line-less site corresponds to `InternalActivationFrame`
whose `getPackage()` is `OREF_NULL`. It then tried to break it across seven nestings -- two library
frames under a program clause, under an internal `CALL`, under a user class method, under an
`INTERPRET`, a native frame innermost inside a library frame, `raise propagate` through
`signal on syntax`, and mixed-case scope ids -- and every one is byte-identical on both engines.
D1, D2, m2, m4, m5, m6 and m7 are closed. Every figure in m5 reproduces.

What is left is prose, and one of the items is a false statement about your own document.

## Must fix

**1. The breadth number is 52, not 51, in all three places -- and the sentence explaining the
difference is false.** Your own arithmetic gives it away: 7 + 14 + 51 is 72, and you had 73 pairs.
Re-derived twice, with two call shapes, at HEAD: **7 match, 14 loud, 52 diverge**, both times.

Delete "The one pair that moved from diverging to matching outright is the `.Validate~number`
shape". **Nothing moved.** The review measured 7/14/52 before your fix and the re-review measures
7/14/52 after it. `.Validate~number` *with no arguments* goes through `USE STRICT ARG` like the rest
and is one of the 52; the `('LENGTH', 'abc')` case that now matches is the one with arguments, and
it is a corpus row rather than a member of that sweep.

**State the finding that is actually there, because it is stronger than the one you claimed**: the
classification did not move at all, and the *content* of all 52 divergences narrowed to the two
`Error ` lines. Stripping every line beginning `Error ` and comparing the remainder is **52 of 52
identical**. Every traceback frame and every program name matches.

**2. The attribution is not withdrawn, and the retraction misdescribes your own text.** Report line
537 still reads, unqualified, "So this is the collector: ...". Line 555 then says "an earlier draft
of this section claimed it did". **It is not an earlier draft -- it is eighteen lines above, in the
same shipped section.** So the bullet claiming the sentence is withdrawn is false about the text as
it stands. Delete line 537's sentence, or re-word the retraction so it does not describe a live
sentence as a dead one.

**3-6, from the reviewer's own list:**

* `native_classes.rs:117`-`:118` -- add `Queue`, or name the set the six belong to rather than
  counting them.
* `native_classes.rs:371` -- `donate_instance_methods`.
* `environment.rs` -- "The two public-class steps need `::REQUIRES`, which is Phase 5c's" is false
  for the second step. `TheRexxPackage`'s public classes come from the image build, not an import:
  `MemoryObject::completeSystemClass` (`memory/Setup.cpp:199`-`:206`) puts every system class into
  `TheEnvironment` **and** into `TheRexxPackage` in the same two lines. Your `.environment` step is
  the substitute for that step, not a step you skip. The decision is right; the reason is wrong.
  Note the reviewer's supporting sweep: `::class K subclass <X>` over seventeen library class names,
  public and non-public, is 17 of 17 byte-identical, which is what makes "no program can see the
  difference today" survive.
* `error.rs` -- the 101.25 sentence's conclusion is wider than its premise.

**7. Optional, take if cheap:** `ClassClass.cpp:560`-`:563`, and strike the two historical clauses in
`rexx-lib`.

## The rule for this round, carried from Task 21

**Prefer deleting to rewriting.** Every false sentence across the last three tasks arrived as an
added justification whose argument was right. Where an item closes by striking a clause, strike it.
A clause that is not there cannot rot.

## How to close

All five gates, run at the commit you are reporting, with `--no-fail-fast`, and **quote the command
that printed each figure beside the figure** -- that is the rule you added yourself after the false
green, and this is its first use. If the round is comments only, prove it with the `.text` section
hash across a forced rebuild rather than a sitting.
