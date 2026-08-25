# Task 21, fix round 3

Re-review: `.superpowers/sdd/2026-08-17-phase-5a/task-21-rereview-2.md`. **The regression is closed
as a class, not just at the one door** -- the reviewer enumerated from the type: one function frees a
slot, one caller of it in `rexx-exec`, and every route to it now sweeps. The corpus row discriminates
in both directions. Nothing behavioural is left. Every finding below is prose or an instrument.

## Must fix

**1. `Outcome::collections`'s doc is false in three clauses** (`lib.rs:374`-`:381`). I verified it:
it says "Always `0` under `run_program`" and "nothing else in this crate calls `collect` at all",
while `collect_policy.rs:121` asserts `outcome.collections > 0` from a plain `run_program` and its
own doc names the measured counts. Pre-existing, not yours -- but it is the paragraph a reader
consults to answer exactly the question this task spent two rounds on, so it is in scope. Nothing to
re-measure; the numbers are in `collect_policy.rs`'s doc.

**2. Historical framing: you struck one and added two.** `activation.rs:763`-`:768` and
`state.rs:226`-`:229` both date themselves against the defect ("`GC('Force')` **was** such a door",
"**Reaching past it left** ..."). **Ruling: strike the dating, keep every word of the evidence.**
The reviewer is right that the constraint's own test settles it -- strike the framing and both
sentences say the same thing about the code as it is, which makes the framing decoration. It is also
the same shape you were told to strike from `environment.rs` in that very commit. Use the shape
`Interp::alloc_with`'s doc already uses: a measured negative control on a design that is not this one
("a door that calls `Heap::collect` directly frees the running activation's own context object --
measured against such a build, ..."), not a date stamp. I know the tree has precedent both ways; this
is my ruling, and the before-state lives in the report and the ledger.

While you are there: "was oracle rc 0 twice" is loose. A process has one exit status.

**3. Concern 4's deletion left its own opening sentence unsupported.** `environment.rs:1008`-`:1014`
promises identity is observable "through **two** of its own methods" and now exhibits one, because
the struck row was the second. That is both the falsified-neighbour pattern and a set cardinality in
a comment, which is what m3 was about. Either name the one method it now shows, or restore the second
exhibit as the `==` comparison that does discriminate.

**4. Two group comments do not cover the row inserted under them.** `corpus/phase-5a.txt`'s last
block and `coverage.rs:1143` both enumerate what the block holds, and `class_context_gc.rex` went in
without joining either sentence. One clause each. This is the insertion-orphans-a-doc-block shape
that has bitten this plan repeatedly.

## Take both recommendations

**5. Make the collection-site class a check rather than a paragraph.** This is the one I most want.
The whole shape of NEW-1 was that the mechanism hangs off the site, and a paragraph saying "check a
new collection site against this paragraph" is prose, which this project has measured to be a
non-control. `crates/rexx-exec/tests/dispatch_seam.rs` already has the walker and the
`occurrences(needle)` helper and the precedent argument in its module doc;
`rexx-core/tests/unsafe_sites.rs` is the second. Assert that `heap.collect(` occurs once in
`crates/rexx-exec/src/` and that the occurrence is in `lib.rs` inside `collect_now`. Then invert it
to prove it can fail.

**6. Add the `REPLY` + `.context` corpus row.** No corpus program contains both, which is the same
disjoint-sets shape that let NEW-1 through, one mechanism over: the parked route has no differential
row at all. The reviewer measured that such a row is writable now -- its probe came back
byte-identical to the oracle on all three descriptors **and line order** on both engines. Write it.

## How to close

All five gates. Both engines for the new rows. Invert the new lexical assertion and say what it
reports when inverted. `cp` before any mutation, restore from the copy, never `git checkout --`.
A sitting is owed only if you change `src/` in a way that can affect codegen; comments in a
`debug = true` profile do change the binary, so say which you did and measure if you are unsure.
Append to the report.
