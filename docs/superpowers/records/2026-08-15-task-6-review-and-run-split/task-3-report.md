# Task 3: the two false passages in `loop-header-boundaries`

Commit `d0b7504a50d4564261909997835154859b50d391`, on `plan/rust-rewrite`, one file changed.

## Where the brief's four passages actually were

The brief warned its line numbers were stale. They were not.
`git diff 56d9d1c86..HEAD -- rust/crates/rexx-exec/tests/ir_dual_cases/loop-header-boundaries`
was **empty** at the tree I found, whose `HEAD` was `d5bc4b8a0`. The three commits that landed
after `56d9d1c86` did not touch this file, so every cited number was still exact:

| Brief's citation | Passage | Where it actually was |
| --- | --- | --- |
| `:44-53` | I1's zero-pass bullet | `:44-53` |
| `:62-67` | I1's "Re-measured" paragraph | `:62-67` |
| `:39-42` | the section framing | `:39-42` |
| `:89-101` | I2's "runs the other way" paragraph | `:89-101` |

I found each by its quoted text and then confirmed the number, rather than the reverse.

## Step 1: the transcripts

Probes ran from a fresh `mkdir`'d directory, with absolute paths for the program and every
redirect, `</dev/null` on stdin, and stdout, stderr and exit status captured as three separate
descriptors. Oracle wrapper as prescribed. Commit for the crate runs: `d5bc4b8a0`, the tree as
found, before any edit.

### Program A: I1's zero-pass loop

Reconstructed from the bullet's own numbers, and it is the row above it with `raiser` returning 0
instead of 1. `do` on line 3, `end` on 5, `say 'after'` on 6, `g:` on 12:

```rexx
call on user zx name h
call on user zy name g
do i = 1 to raiser()
  nop
end
say 'after'
exit
raiser:
raise user zx return 0
h:
raise user zy return 1
g:
say 'G ran' sigl
return
```

| Run | rc | stdout | stderr |
| --- | --- | --- | --- |
| oracle | 0 | `after` / `G ran 6` | empty |
| `REXX_ENGINE=tree-walker`, `d5bc4b8a0` | 0 | `after` / `G ran 6` | empty |
| `REXX_ENGINE=ir`, `d5bc4b8a0` | 0 | `after` / `G ran 6` | empty |

**The divergence the bullet records is closed.** Both engines agree with the oracle byte for byte
on all three descriptors. The reviewer's reconstruction reproduced, and so did their finding: the
bullet's `G ran 3` / `after` is no longer what this crate prints.

### Program B: I2's "runs the other way" reconstruction

Step 1b's `DO UNTIL` program, without the `H ran` marker that the plan's CORRECTED note says was
added later. `do until` on line 4, `nop` on 5, `end` on 6, `say 'after' zv` on 7:

```rexx
call on user zx name h
call on user zy name g
zv = 'unset'
do until raiser1() > 0
nop
end
say 'after' zv
exit
raiser1:
raise user zx return 5
h:
zv = 'set'
raise user zy return 1
g:
say 'G ran' sigl
return
```

| Run | rc | stdout | stderr |
| --- | --- | --- | --- |
| oracle | 0 | `after set` / `G ran 7` | empty |
| `REXX_ENGINE=tree-walker`, `d5bc4b8a0` | 0 | `after set` / `G ran 7` | empty |
| `REXX_ENGINE=ir`, `d5bc4b8a0` | 0 | `after set` / `G ran 7` | empty |

Both recorded lines are wrong, exactly as the brief said. The oracle prints `after set`, not
`after unset`; and this crate no longer prints `G ran 6` / `after set`, it prints what the oracle
prints.

### Program C: the `DO UNTIL` row's own program

Not asked for, but my replacement sentence for I2 makes a claim about that row's bytes, so I
measured the row's exact program rather than relying on the file's own header. It is Program B
with `say 'H ran' sigl` inserted as the first clause of `h:`.

| Run | rc | stdout | stderr |
| --- | --- | --- | --- |
| oracle | 0 | `H ran 6` / `after set` / `G ran 7` | empty |
| `REXX_ENGINE=tree-walker`, `d5bc4b8a0` | 0 | `H ran 6` / `after set` / `G ran 7` | empty |
| `REXX_ENGINE=ir`, `d5bc4b8a0` | 0 | `H ran 6` / `after set` / `G ran 7` | empty |

The row's expected block records those three lines, so the row's bytes are the oracle's.

## Step 2: I1, deleted and promoted

**Deleted**, per the brief, and not rewritten.

* The bullet at `:44-53` is gone in full.
* The paragraph at `:62-67` is gone in full, including its opening
  "**They do not share a cause, and only one of the two has an established one.**" That sentence
  is *about* the pair, so with one bullet left it had no subject; deleting the paragraph whole is
  what the brief asked for and is also what keeps a sentence from standing under a heading that
  makes it false.
* The program is now a row, placed immediately after the one-pass row it contrasts with, so the
  two sit adjacent and a reader compares them by looking down the page.

The row's expected block is the oracle's bytes as measured above, captured from the oracle and not
regenerated. Its comment makes only claims a reader can check against the program printed directly
beneath it:

> The row above with `raiser` returning 0 instead of 1, so the loop runs no pass and no body clause
> stands between the two deliveries. SIGL 6 names the `say` after the loop, not the `end` on line 5
> and not the `do` on line 3, and `after` prints before `G ran 6` rather than after it. The row
> above, with one pass, reports `G ran 4` and prints it first, so it does not cover this shape.

Every number in it is a line of the program below it or a line of the expected block above it. I
wrote a first version that said the second delivery is "owed to the first real clause after the
loop" and replaced it: that phrasing invited the reader to work out which clause that is, and
naming line 5 and line 3 as the ones it is *not* removes the step where the reader could be wrong.

## Step 3: I2, deleted with one anchored sentence kept

The brief asked which half got which treatment.

**The transcript pair, both lines: deleted.** Deletion loses nothing. The oracle half was wrong
when written; the crate half records a divergence that is closed, and the shape it records is now
pinned twice over by bytes in this same repository -- the `DO UNTIL` row further down this file,
and `corpus/lang/do_clause_boundaries.rex` under the live oracle. Prose recording a closed
divergence cannot be made true by correcting its numbers; it can only be made true by ceasing to
record a divergence.

**The framing sentence "Every other shape recorded in this file has this crate delivering *later*
than the oracle. This one delivers *earlier*, on both engines": deleted.** It is false of the
current tree in both halves. The direction it names is not lost: the closed-family section further
down still records it, as "That shape runs the opposite way to every other one here, which is why
it is named rather than left to follow from the others", where it is correctly framed as a
property of the defect that was fixed.

**One sentence's worth of content was load-bearing and was rewritten, anchored:** the paragraph's
real service to the reader was keeping the `DO UNTIL` row from being mistaken for the withdrawn
`UNTIL` bullet's program. That hazard is *worse* now, not better: this file holds a bullet saying
an `UNTIL` requeue diverges and, below it, a `DO UNTIL` row that agrees, and nothing else would
tell a reader those are different programs. So the paragraph now reads:

> **The `DO UNTIL` row below is not this bullet's program and is not offered as one.** That row
> prints `H ran 6`, `after set` and `G ran 7`, which is what the oracle prints for it; this bullet
> records `G ran 5` for the oracle against `G ran 6` for this crate. So a `DO UNTIL` row agreeing
> below neither confirms this bullet nor closes it.

This is the anchoring shape rather than a rewrite of the claim: checking it is a comparison between
two places in the same file -- the row's expected block and the bullet's own numbers -- with no
arithmetic and no external reference. I deliberately dropped the old paragraph's citation of
`docs/superpowers/plans/2026-08-14-pre-phase-5-defects.md`, because the row's program is Step 1b's
program *plus* the `H ran` marker, so "its program is named there" was a claim I would have had to
qualify, and qualifying it is exactly the kind of rewrite this plan has measured going wrong.

## Step 4: the section framing, re-read against what remains

The framing at `:39-42` is unchanged. Its sentences, each checked against the file as it now
stands:

1. **"LOOP-HEADER BOUNDARIES THAT DIVERGE FROM THE ORACLE ARE NOT ROWS HERE, because a row's
   expected block is what this crate prints and these are places where that is not what the oracle
   prints."** True. Exactly one item remains under it, the `UNTIL` requeue bullet, and it is not a
   row. The program I promoted is not a counterexample: measured above, it agrees with the oracle
   on both engines, so it is not a boundary that diverges.
2. **"Each was measured on both engines, and the engines agree on each"** -- true as far as it can
   be checked, and see the concern below. It is a historical claim about the one remaining bullet.
   It does not contradict that bullet's own caveat, which says the observation cannot be *re*-run
   and that its program was not recorded; it does not say the observation was never made.
3. **"none moved when the header was flattened"** -- unaffected by this task. It is a claim about
   `de05ea58`, which my edits do not touch.

I also re-read the sentences that reference the deleted material, for dangling subjects:

* `:68-70` "The bullet stays, because it records a real observation someone made; its numbers are
  left alone, because choosing which program produced them would be adding **one more**
  reconstruction..." -- "one more" still has its antecedent at `:57-59`, "Reconstructing a program
  from its words has been tried repeatedly", which I did not touch.
* `:78-79` "They are recorded here rather than in a report..." -- "They" now covers the bullet and
  the withdrawal notes around it. Plural over a one-item list; pre-existing phrasing, left alone.
* I grepped the header block for "the two", "both bullet", "zero-pass" and "reconstruction": the
  only surviving hit is the `:70` one above.
* `crates/rexx-exec/src/run.rs:5602` and `crates/rexx-exec/src/ir/drive.rs:478` are the two places
  in the crate that cite this file by name. Both were read; both cite passages I did not touch
  (the plain-`SELECT` boundary, and the "procedure as a loop body's first instruction" row).

## Step 5: gates

Each run unpiped with its own exit status read. From `rust/`.

| Gate | Result |
| --- | --- |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `memcap 8G cargo test --release --workspace --no-fail-fast` | exit 0, 1509 passed, 0 failed |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --release --workspace --no-fail-fast` | exit 0, 1509 passed, 0 failed, `corpus_differential ... ok` |

The suite is real evidence here: the new row is compared by
`both_engines_agree_on_every_case_file`, which asserts the two engines against each other and the
tree-walker against the recorded block. It was green with the bytes I captured from the oracle,
which is the row's second measurement and an independent one.

**A caveat on clippy.** No `.rs` file changed in this task, and the target directory was warm, so
the green is provisional in exactly the way `rust/CLAUDE.md` describes. It is also not evidence of
anything about this change, since the change is a data file.

## Files changed

* `rust/crates/rexx-exec/tests/ir_dual_cases/loop-header-boundaries` -- 31 insertions, 30
  deletions. The only file staged.

`docs/superpowers/specs/2026-08-15-phase-5-inherited-surface.md` is untracked and was untracked at
the start of my session. It is not mine and I did not stage it.

## Self-review findings

Read back with fresh eyes; two things changed as a result.

1. The row comment's original wording ("owed to the first real clause after the loop") was replaced
   by the version naming lines 5 and 3, as described in Step 2.
2. My first draft of the I2 replacement kept the plan-file citation. I removed it once I noticed
   the row's program and Step 1b's program differ by the `H ran` marker.

No dangling "this" or orphaned subject found after the deletions; the checks are listed in Step 4.

## Concerns, and things found and not fixed

1. **"Each was measured on both engines, and the engines agree on each" cannot be verified for the
   one bullet that now remains.** The bullet records no program, and the file says so two lines
   below. The sentence is not *contradicted* -- a measurement can have been taken and its program
   not written down -- but nobody can re-run it, so it is an assertion resting on a record that no
   longer exists. It was in this state before my edit, covering a pair one of which was
   unre-runnable; deletion of the other bullet makes it more visible without changing its truth
   value. I left it, because rewriting it is the shape of change this plan has measured introducing
   a new false claim three times running, and because narrowing it correctly needs a decision about
   whether the bullet survives at all -- which `:68-70` already took and which is not mine.
2. **The plural voice of the block now covers a single item.** "these are places", "Each", "none",
   "They are recorded here". Grammatically a one-member set satisfies all of them, so nothing is
   false. Left alone under the same reasoning as (1).
3. **No divergence found that this plan does not name.** All three programs I ran agree with the
   oracle byte for byte on stdout, stderr and exit status, on both engines. Nothing to record under
   the "found a divergence, move on" rule.
4. **The oracle-crashing shapes were avoided.** None of my programs queues the same `CALL ON`
   condition twice at one boundary; each queues `zx` and then `zy`, two differently named
   conditions, which `rust/CLAUDE.md` records as safe.
