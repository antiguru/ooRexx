# Task 16, fix round 2

The re-review of fix round 1 is at
`.superpowers/sdd/2026-08-17-phase-5a/task-16-rereview.md`. **Read it first.** It closes
all seven of round 1's findings on their substance and raises ten new ones, labelled B
through J. Nine are yours; the tenth (Priority 1, the controller's own commit) is already
fixed at `08965a822` and is not your work.

Every defect below is a sentence in a comment or in `task-16-report.md`. There is no
behaviour defect, no dead instrument and no number that gates anything. Do not change
behaviour except where D says to add a corpus program.

## Rulings, already made -- implement these, do not re-litigate them

**B. `Activation::reply`'s "named rather than counted" doc claims a self-check it does
not have.** The rereview offers two ways out. **Ruling: drop the enforcement clause, keep
the first half.** `cargo doc` is not one of the five gates, so turning the four reader
names into intra-doc links would buy a warning that lands among nineteen others and fails
nothing -- that is a check blind to its own subject, which is the hazard this plan keeps
hitting. If you want the links as well, add them, but the sentence must stop claiming an
error that does not exist.

**C. The order-count difference is the program, not the sitting.** The reviewer re-ran the
five-order shape from a third sitting and got five orders again, three rows identical. Fix
`Interp::deferred`'s doc to attribute the difference to how the shape is written. The
conclusion is unchanged and stays: the oracle has no single answer, no figure here is *the*
distribution, and neither program can be a corpus row. **Also fold in the traceability
half** -- the report must carry the program its 18/12 came from, inline, since it is a few
lines long.

**D. 34.902 has no corpus backstop.** **Ruling: add the corpus program**, do not weaken the
sentence. `corpus/lang/method_guard_when_not_logical.rex` -- a method whose `GUARD ... WHEN`
expression evaluates to a non-logical value. Measure it against the oracle under the
standard wrapper from a fresh empty directory, both engines, three descriptors read
separately, and only then add it. The reviewer's reading was oracle rc 222,
`Error 34.902:  Value of expression following GUARD keyword ...`, both engines identical --
confirm that yourself rather than copying it. This makes the guard test's doc sentence true
instead of restating it, and it is real differential coverage rather than prose. The corpus
count moves 185 -> 186; every gate must be green at the new count.

**E. `Interp::exec_guard`'s doc is an unmarked second copy of entry 7.** It quotes the
never-run program inline with a 6-second deadline and no do-not-run warning; round 1 removed
the pointer that used to tie it to the `Loud`. Point it at `corpus/oracle-crashes.txt` entry
7 and drop the 6-second figure. Do not re-measure it, and do not run the program: it is in
the never-run file. The authoritative record is the entry, which states its own deadline.

**F. The marker count is not what its own command prints.** `/bin/grep -c` gives 4 for
`activation.rs`, not 3, because `Activation::reply`'s marker wraps to a second line. The
three *sites* are right and the marking is complete. **Ruling: delete the number, keep the
site list.** The list is the claim's actual subject; a line count cannot see it, because one
wrapped marker pays for one missing marker. Do not ship a corrected count.

## Also fix, all in files you are already touching

**G.** The contribution table tags all four `dispatchclass` cells `min-derived` and the
block says "Recomputed from `min`". True of the tw rows; the ir rows are `changed` **median**
over `base` min. Substantively harmless -- the difference is 0.0006 percentage points and the
reconciliation closes either way -- but it is the wrong label in the one block whose whole
subject is which statistic is being read. State what each arm actually used.

**H.** "19 unresolved-link warnings" is 19 warnings of which 4 are unresolved links, and none
of the 4 comes from this diff. The fact the claim was reaching for -- that the round's new
intra-doc links all resolve -- is true and worth stating in place of the wrong one.

**I.** The fix table says the test's old name "carried a count". It carried a false universal;
that is what finding 7 said about it. "Five more set-size phrases" is also itself a count, in
a report about removing counts, and the reviewer can identify four.

**J.** The `raised_guard_not_logical` LEGALITY paragraph was inserted between the summary line
and the rest, so the `Error_Logical_value_guard` catalogue sentence now continues the LEGALITY
paragraph instead of the summary. Its sibling `raised_if_not_logical` six lines above keeps the
catalogue text in the summary block. Restore the local convention.

## What must not happen

**The set-size sweep must not relocate the shape.** Round 1 was the first on this plan that did
not, and the rereview verified it: four genuine set-size phrases removed, none reintroduced.
Do not introduce a new count anywhere, including in the report's own prose about the fixes.
Where you replace a count, replace it with names, and do not attach a claim that the names are
self-enforcing unless something actually fails when one goes stale.

**Do not add an instrument claim you have not run.** Three of the closed enumerations
(`RootSet::park`, the guard test's doc, `Loud::guard_when_false`) have no instrument and are
parked as such -- leave them. D is the one place where the answer is to build the instrument.

## Gates

`cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast`. Report the corpus
matching line verbatim; it must read 186 of 186 once D lands. No perf sitting is needed --
this round changes no code that runs.
