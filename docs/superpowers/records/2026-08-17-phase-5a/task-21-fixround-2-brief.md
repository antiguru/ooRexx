# Task 21, fix round 2

Re-review: `.superpowers/sdd/2026-08-17-phase-5a/task-21-rereview.md`. Verdict CHANGES REQUIRED, but
**all eleven original findings are ADDRESSED** and verified against the tree, and your two new corpus
rows match the oracle byte for byte on both engines. This round is one regression plus prose.

## NEW-1. `GC('force')` bypasses the sweep your C1 fix relies on. Must fix.

I reproduced it. Both engines identical:

    say .context~objectName
    say gc('force')
    say .context~objectName

Oracle rc 0, `a RexxContext` / `1` / `a RexxContext`. Crate rc 120 after the second line:
`a message send to a value whose object is no longer live is not implemented (Phase 5)`.

`builtin/state.rs:222` calls `interp.heap.collect(&interp.roots)` directly, so the context-object
sweep you added to `collect_now` never runs and an activation's cached object is collected out from
under it. **This is a regression this round introduced** -- at `7d1544f84` every `.context` minted a
live object, so the program was byte-correct.

Take the reviewer's first option: route the builtin through `interp.collect_now()`. It closes the
class rather than the instance, and it also stops a forced collection leaving `collect_at`
unadjusted. Add the corpus row that becomes writable once it is fixed -- `gc(` and `.context`
currently appear in disjoint sets of programs, which is why nothing caught this.

**Two sentences in the tree are falsified by NEW-1 and must move with the fix:**

* `activation.rs:756`-`:760` -- "Those are the two states an activation can be in and neither reaches
  the other's mechanism." The states are two; the mechanism hangs off the *collection site*, and one
  site had neither. (While you are there: the sweep lives in `Interp::collect_now`, not
  `collect_if_due`, which only calls it.)
* `dispatch.rs:970`-`:971`, pre-existing and now false -- "A handle whose slot is gone. Not reachable
  from a running program." The three-line probe above reaches it from a running program.

**And one latent window worth a sentence, not a change:** `resume_reply` calls
`self.roots.release(parked)` before `self.push_activation`, so between those two lines the context
object is rooted by neither mechanism. Nothing in that window allocates today. Say so beside the
`release` so the next person to add an allocation there sees it.

## Concern 4. Ruling: take the deletion.

I am overriding your deferral, and the reviewer's argument is why. Your rule -- do not rewrite
another task's measured prose on the strength of one clause -- is right and is the rule this plan has
most needed. It does not cover this case: the clause is not merely unwitnessable, it is **false and
measured false** (a fresh-object-per-send build answers `1` to that `=` row, not `0`), and **the
remedy is a deletion**. A deletion cannot introduce a false statement, which is the entire argument
for deferring. Strike the `0` and its row in `Interp::method_object`'s doc; leave every other
measurement in that paragraph untouched and un-re-taken. The `a Method` half is sound.

## Prose defects

* **Historical framing**, `environment.rs:822`-`:823`: "Measured before this check existed:
  `.K~defineMethods(.local)` was rc 0 against the oracle's rc 163." Strike the framing.
  `Loud::unreadable_collection` already makes the point without it.
* **A corpus comment that claims something the plain run does not do.**
  `corpus/lang/class_context_identity.rex` says the loop "allocates enough to run the collector". It
  does not: `COLLECT_FLOOR` is 65,536 slots and 300 iterations are three orders short. The reviewer
  bracketed the first collection at about 65k by peak RSS. Keep the next sentence, that
  `collect_stress.rs` is what makes that half real; fix or strike this one.
* **One more sixth-decimal slip, same shape as m5**, in your re-measurement section: "ir/large reads
  1.017380 on both builds" -- the TSV has 1.017380 out-of-line and 1.017378 inlined. Changes no
  conclusion.

## How to close

All five gates, plus re-run the NEW-1 probe on both engines. Invert the NEW-1 fix as a control and
say what reddens. `cp` before any mutation and restore from the copy; never `git checkout --`.
Append to the report.
