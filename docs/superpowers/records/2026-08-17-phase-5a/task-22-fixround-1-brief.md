# Task 22, fix round 1

Review: `.superpowers/sdd/2026-08-17-phase-5a/task-22-review.md`. **Spec compliance PASS.** Task
quality CHANGES REQUIRED on four majors and seven minors, and **every major is a false sentence, not
a behaviour**. The reviewer could not falsify the implementation: every probe it put against the
registry matched the oracle byte for byte on three descriptors and both engines, including several
you did not write.

## The governing rule, carried over from Task 21

That task spent five rounds and **every false sentence it shipped was an added justification whose
argument was right**. Round 5's rule closed three of four items by deleting: **where an item closes by
striking a clause, strike it; do not replace it with a better reason.** A clause that is not there
cannot rot. Write new prose only where a reader is left with a real gap, and run whatever it asserts
before committing.

## The four majors

**M1. An orphaned doc block.** `lib.rs:6227`-`:6295`. You inserted `pub struct NativeEntryPoint` and
`native_entry_points()` **between `run_program`'s doc comment and `run_program`**. I verified it: the
block now documents the struct, `pub fn run_program` has no documentation at all, and the section
heading `// ---- the public entry point ----` now heads a data struct. Every paragraph in it is false
of what it is attached to. `fmt`, `clippy` and the suite all pass over this; only `cargo doc` sees
it, and that is not a gate. Move the new items below `run_program`, or re-anchor the block.

**M2. A caveat falsified by evidence you were already holding.** `staged_gap`'s doc at
`lib.rs:1626`-`:1628` still says the second row is "reasoned rather than probed" because "no library
in this tree loads". `REXX` loads. You corrected the copy in `phase-4-exclusions.txt` and left the
source copy untouched -- and **the correction you wrote is also false**: it argues only the `::METHOD`
form, while `::ROUTINE` and `::ATTRIBUTE` naming `LIBRARY REXX` are still gaps, still reach the
staged check, and their library still resolves. The reviewer probed the row you both called
unprobeable and got oracle rc 214 against crate rc 120. Keep the row in both tables; strike the
claim that it cannot be measured, and the reason.

**M3. A quoted refusal message this task deleted.** `lib.rs:4739`-`:4742` quotes
`::METHOD EXTERNAL is not implemented (Phase 7)`. I checked: that string has **zero** occurrences in
`lib.rs` now; the program it describes gets
`::METHOD EXTERNAL naming a library other than REXX is not implemented (Phase 7)`.

**Method note, and it matters for how you sweep:** the string is hard-wrapped across two `///`
lines, so a plain `grep` for it over the tree finds nothing. Do the sweep by **collapsing comment
blocks to single lines first and then searching** -- otherwise you will conclude there are no more
instances when the search could not have seen them.

**M4. A "no X below this arm" claim falsified by the arm you inserted below it.**
`dispatch.rs:1440` says "No `blame_native_method` below this arm"; your `Invocable::External` arm at
`:1477` calls exactly that, and your own new comment three lines earlier says so. Two comments
disagreeing inside one `match`. The behaviour is right; narrow or move the older sentence.

## The seven minors

Take them from the review directly -- `MINOR 1` through `MINOR 7`. Two shapes worth naming:

* **MINOR 1**: you swept the invocable-kind count in two places and the reviewer found four more in
  the same file, one of them a whole section. Sweep by running a search over the file, not by fixing
  the instances someone handed you.
* **MINOR 3**: comment set cardinalities. Same rule as always: name the set, not its size.

## How to close

All five gates. If you touch `src/` in a way that can affect codegen, a sitting is owed; if the round
is comments only, prove it the way Task 21's last round did -- forced rebuild, `sha256sum` on
`target/release/rexx-run` before and after -- which is stronger than a sitting and cheaper.
`cp` before any mutation, restore from the copy, never `git checkout --`. Append to your report.
