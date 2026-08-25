# Task 21, fix round 5 -- the last one

Re-review: `.superpowers/sdd/2026-08-17-phase-5a/task-21-rereview-4.md`. **This is the final round of
this task.** Whatever survives it, I adjudicate and close. The reviewer verified this round's prose by
execution -- it reimplemented both needles and injected each bypass spelling to check your blind-spot
list, and every claim you made about the widening and about F1's exhibit held. Four items are left.

## The governing rule for this round: prefer deleting to rewriting

Four correction rounds on this task have each introduced a false sentence while fixing one, and every
single instance was **an added justification** -- an exhibit, a quantifier, or a reason that did not
survive being run. The arguments were right each time. So:

**Where an item can be closed by striking a clause, strike it. Do not replace it with a better
reason.** A clause that is not there cannot be false. Only write new prose where a reader would be
left with an actual gap, and where you do, run whatever it asserts before committing it.

## The four items

**1. False clause, `dispatch_seam.rs:230`-`:232`.** The comment justifies stripping comment lines
with "the needle is an ordinary phrase and this file's own prose uses it". I verified this is false
under both readings: `source_files()` walks `CARGO_MANIFEST_DIR/src` only, so `tests/dispatch_seam.rs`
is never read -- and this file's own module doc says exactly that at `:93` -- while under `src/` the
only occurrence of `heap.collect(` is real code at `lib.rs:6034`, no comment anywhere. **Strike the
reason.** The decision to strip comments is right and needs no justification beside it; if you want
one, "a comment must not be able to satisfy the assertion" is true prospectively and asserts nothing
about today's tree.

**2. Set cardinality, `dispatch_seam.rs:220`:** "Two spellings that would have escaped a narrower
needle do not escape this one". Both are named in the same sentence, so the count carries nothing,
and the set is open. Strike the number.

The reviewer noted the same file already carries the shape at `:50`, pre-existing and unflagged.
**Ruling: leave `:50` alone.** It is not this task's, and reaching into it is how a correction round
grows. Record it as a follow-on.

**3. `phase-5a.txt:635` names a superset.** "The `class_context_*` rows beside them" -- the glob
matches four rows and one of them, `class_context_package.rex`, is the package row the previous
sentence just described. The sentence is true because "beside them" excludes it, but the set name
picks out a superset containing the row it contrasts against. Name the rows or name the property.

**4. The same paragraph describes five things for six rows.**
`lang/class_package_addition_refused.rex` has no description there, though its twin in `coverage.rs`
does. Pre-existing, not this round's, but the two comments now describe different sets. One clause.

## Not changing, ruled

The past-tense clauses in the corpus headers stay, as ruled in round 4. `dispatch_seam.rs:50` stays.

## How to close

All five gates. This round should touch no `src/` code path -- if it touches only comments, say so and
measure the sitting the way you did for the last two comment-only rounds. Append to the report, and
end it with a short list of what a future task inherits from this one.
