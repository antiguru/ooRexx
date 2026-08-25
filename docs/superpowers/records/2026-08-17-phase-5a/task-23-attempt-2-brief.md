# Task 23, attempt 2

Attempt 1's report is `.superpowers/sdd/2026-08-17-phase-5a/task-23-report.md`. It landed the
install half and reported BLOCKED on the prologue, correctly. **Its enumeration of the five missing
mechanisms stands and is the starting point.** What changed is the reading of blocker 1.

## The ruling that resizes this task

Attempt 1 reported blocker 1 as "`DO OVER` a `StringTable`, and the mechanism behind it is
hash-collection iteration order, which nothing in this plan assigns". I carried that up as the
blocker. **Moritz challenged it and was right: matching the oracle's hash order is a different
problem from iterating at all, and the prologue does not need the first one.**

Measured, both loops, `CoreClasses.orx:63` and `StreamClasses.orx:45`:

    do name over publicClasses
       class = publicClasses[name]
       .environment~put(class, name)
       rexxPackage~addPublicClass(name, class)
    end

**Order-insensitive.** Each iteration puts one distinct key into `.environment` and one into the
package. The resulting state is identical under any permutation of the keys, and the prologue prints
nothing, so no output depends on the order. The second loop in `CoreClasses.orx` iterates a literal
array, whose order is given in the source.

**So build `DO OVER` over a hash collection with a deterministic order of your choosing.** Sorted key
order is fine and is what `public_classes_table` already fills in. Do **not** build a bucket model,
and do not spend the task reproducing `HashContents`' order.

**Record the order divergence rather than hiding it.** A user program that iterates a table and
prints its keys will differ from the oracle in order. No corpus program does that today, because the
construct has been rc 120. Say so in the doc comment beside the iteration, name it as a known
divergence class with the reason it is not closed here, and **do not add a corpus row whose output
depends on that order** -- adding one would pin an order the oracle does not share and turn this into
a red row rather than a recorded gap.

Attempt 1's analysis of the oracle's order is not wasted and should be preserved where it is (in the
plan's Task 23 section): it establishes that the order is deterministic rather than random
(`hash = 31*h + byte`, bucket `hash % 17`, `HashContents.cpp:489`), which is what makes the
divergence closable later by whoever needs it. **Correct any sentence that calls it a blocker.**

## The rest of the work, from attempt 1's own enumeration

2. **`String~UPPER`** -- `CoreClasses.orx:73` needs it twice in one clause. Build it. It is
   independently witnessable by an ordinary corpus row, so give it one.
3. **`Class~defineClassMethod` and `Class~inheritInstanceMethods`** -- the bootstrap needs a state in
   which `.Class` has them and a step that removes them when the prologue reaches its `exit`
   (`Setup.cpp:1809`'s `removeSetupMethods()`). `native_classes.rs`'s `REMOVED_BY_IMAGE_SAVE` is
   where the shipped state is modelled.
4. **The `REXX_DEFINED` lock, open during the bootstrap** -- `.string~inherit(.Comparable)` must
   succeed for the prologue and must keep raising `98.985` for a user program.
5. **The REXX package accepting additions during the bootstrap** -- the oracle's `checkRexxPackage`
   (`PackageClass.cpp:470`) refuses when `isInternalCode()`, a flag `TheRexxPackage` does not carry
   while the image is being built.

Rows 3, 4 and 5 are one shape, as attempt 1 said: **bootstrap-time interpreter state that is not the
shipped state**, closed by one step at the prologue's `exit`. Land them together with the bootstrap
that exercises them, which is now possible because blocker 1 is not a wall.

## What attempt 1 already delivered, and you must not redo

* the message scope override send (`8f7a757e4`);
* `rexx-lib` with the three sources and their sha256 pins, drift control inverted both ways;
* `tests/bootstrap_install_oracle.rs`, the install differential -- **byte-identical to the oracle on
  three descriptors, both engines**; I re-verified that `CoreClasses.orx` run directly is now the
  oracle's own rc 159 at line 47 on both engines;
* checks 3 and 5 passing and asserted against the oracle.

## Done when

The brief's five checks, as far as they are reachable, each a number or a named absence. Checks 1,
2 and 4 are the ones this attempt is for. The brief's control (suppress `ACTIVATE`, `.TraceObject
~option` reads `OPTION`) is already run and inverted -- **do not re-run it**; run a control for
whatever you newly build, and say what reddens.

If a sixth mechanism appears that none of this names, **report BLOCKED with the measurement** rather
than absorbing it. That is what attempt 1 did and it was the right call.

Same hard rules as attempt 1's dispatch. Append to `task-23-report.md` under a new heading; do not
rewrite what is there.
