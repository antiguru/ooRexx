# Whole-branch review fixes

Commit `b029abe77`, "Say what a boundary now delivers, and gate the asserts that police it".
Base for every measurement below unless stated otherwise: `3492b0b9e`, the branch head when this
started. Nothing in the change touches behaviour; every hunk is a comment, a document, or a
test-data annotation.

## I1: the pre-drain rule, and the search for its paraphrases

### The search

The previous sweep grepped six literal spellings, none of which matched "one delivery per clause
boundary". I ran the search again with terms chosen to catch paraphrases rather than known wordings,
over `rust/`, `docs/` and `.superpowers/`, with `/bin/grep -a` and `./target/` excluded. The term
groups were: delivery-count phrasings (`one delivery per`, `a single delivery`, `delivers one`,
`deliver one`, `one condition per`, `at most one`, `one trap per`, `single pending`, `one pending`);
queue phrasings (`pending condition`, `pending trap`, `queued condition`, `next boundary`, `boundary
delivers`, `boundary owes`, `drain`/`drains`/`drained`, `per boundary`); ordering phrasings (`walking
forward`, `first match`, `the first pending`, `first queued`, `the earliest`, `oldest`, `only the
first`, `only one`); and a file-set sweep, reading every `boundary` hit with context in every file
that mentions condition delivery at all (`clause.rs`, `run.rs`, `run/tests.rs`, `lib.rs`,
`activation.rs`, `error.rs`, `eval.rs`, `ir/drive.rs`, `ir/compile.rs`, every `ir_dual_cases/` file,
`ir_dual.rs`, `trace_indent.rs`, `trace_oracle.rs`, `corpus.rs`, `corpus/*.txt`, `corpus/lang/*.rex`,
`sourceline_oracle/*.txt`).

| Site | What it says | Disposition |
|---|---|---|
| `corpus/lang/do_clause_boundaries.rex:9` | "one delivery per clause boundary, walking forward" | **Rewritten.** The named finding. |
| `crates/rexx-parse/tests/sourceline_oracle/do_clause_boundaries.txt:10` | the same sentence | **Regenerated**, not hand-edited. Method below. |
| `crates/rexx-exec/src/run.rs`, `deliver_pending_traps` doc, lead sentence | "Runs a `CALL ON` trap's handler, at the clause boundary the condition has been waiting for." | **Deleted. This is the seventh site.** `56d9d1c86` added the drain summary *after* the existing doc instead of replacing it, so the function carried two summary paragraphs, the older one the pre-drain rule in paraphrase. `git log -L` on the region confirms the mechanism. The drain summary now leads; the "wait is the measured part" and `Trap::delayed` paragraphs are untouched and still true of this function. |
| `crates/rexx-exec/src/lib.rs:1700-1722` (`pending_traps`) | "A queue, drained at a boundary… The oracle takes everything a clause left pending, not one" | Correct already. No change. |
| `crates/rexx-exec/src/clause.rs:461-470` (`enter_clause` tripwire) | "That boundary drains, but it drains only the entries that were queued when it began" | Correct already. No change. |
| `crates/rexx-exec/src/clause.rs:337` | "the boundary reads the waiting condition off `Interp` too" | Singular, but it is describing why `ClauseEntry` is zero-sized, not stating a delivery count. Left. |
| `crates/rexx-exec/src/run.rs:5492` | "a boundary drains only the entries that were queued when it began" | Correct already. No change. |
| `crates/rexx-exec/src/activation.rs:52` | "`CALL ON` runs the handler as an ordinary internal call at the next clause boundary" | About *when*, not *how many*. Left. |
| `crates/rexx-exec/tests/ir_dual_cases/assignment-and-say:334` | "One delivery per pass, each at the body clause's own boundary" | A statement about that row's own program, which queues one condition per pass. Left. |
| `crates/rexx-exec/tests/ir_dual_cases/assignment-and-say:474` | "One delivery, at the `IF`'s own boundary" | Same: that row queues one. Left. |
| `crates/rexx-exec/tests/ir_dual_cases/loop-header-boundaries:19` | "a clause boundary drains only the entries that were queued when it began" | Correct already. No change. |
| `crates/rexx-exec/src/ir/drive.rs:1584` | "keeps the two engines to one boundary per construct" | About how many *boundaries* a construct owes, not how many conditions one delivers. Left. |
| `corpus/oracle-crashes.txt:121` | "the oracle drains a boundary in queue order" | Correct already. No change. |
| `corpus/phase-4c.txt:212-216` | "The oracle takes everything the clause left pending… A single-slot queue answered the last condition and lost the rest" | Correct already. No change. |
| `corpus/lang/condition_queue_drain.rex:3` | "A boundary takes everything the clause left pending, in the order it was queued -- not one and not the last." | Correct already. No change (its *second* paragraph is I3). |
| `docs/superpowers/specs/2026-08-15-ansi-condition-delivery.md:15, :310` | both are the *correction* of an earlier pre-drain premise, and say so in the same sentence | Left. Correcting them would delete the record of the correction. |
| `docs/superpowers/specs/2026-08-15-ansi-condition-delivery.md:267` | "the crate presumably uses one delivery *mechanism*" | Not a count. Left. |
| `.superpowers/sdd/2026-08-14-pre-phase-5-defects/ansi-do-boundaries.md:257` | the pre-drain premise, unsuperseded in that copy | Left: it is the ledger the spec above was derived from and then corrected, and the spec is the live document. Untracked in git either way. |
| `docs/superpowers/plans/2026-08-14-pre-phase-5-defects.md:370`, `:463`, `docs/superpowers/plans/2026-08-15-task-6-review-and-run-split.md:63` | statements about *where* a boundary is, not how many it delivers | Left. |
| `docs/superpowers/plans/2026-07-27-rust-rewrite.md:2460-2461` | ANSI readings for Phase 7 | Left; framed as the standard's text, not this crate's. |

### How the capture was regenerated, and how it was verified

`sourceline_oracle/*.txt` is a mechanical capture of the program text. `sourceline_oracle.rs`'s own
module doc carries the driver and the shell loop; I used them unchanged, except that the driver ran
on a **byte-identical copy** in a fresh scratch directory rather than on the repository file, because
`CLAUDE.md` forbids `.Package~new` on a file inside the repository. `cmp` confirmed the copy equal to
the repository file before each run. The capture's own line count is the driver's `a~items`.

Verification, both files:

* `cmp <(tail -n +2 capture.txt) corpus/lang/<name>.rex` -> equal. The capture body is byte-for-byte
  the program, so nothing was hand-edited into it.
* `cargo test --release -p rexx-parse --test sourceline_oracle` -> `1 passed; 0 failed`, exit 0. That
  test reads the header `count N`, asserts the captured line count matches its own body, and compares
  every line against `ProgramSource`'s own answer for the corpus file.

Counts moved with the comment edits: `do_clause_boundaries` 167 -> 170, `condition_queue_drain`
90 -> 93.

## Every transcript measured

All oracle runs: `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx FILE
</dev/null )` from a fresh empty directory made for the purpose, absolute paths throughout, stdout,
stderr and exit status read as three descriptors. Both engines: `REXX_ENGINE=tree-walker` and
`REXX_ENGINE=ir` on `target/release/rexx-run`, at `3492b0b9e` for the two I2 probes and at the
committed tree for the corpus programs.

**I2, stanza 2 of `ir_dual_cases/condition-queue-drain`** (the file's own program, extracted verbatim):

```
oracle        rc 0  stderr empty
tree-walker   rc 0  stderr empty
ir            rc 0  stderr empty

h ran 5
g ran 6
k ran 5
h ran 5
g ran 6
after 1
k ran 7
```

`cmp` equal on all three. Line numbering of that program: `do zi = 1 to 2` is line 4, `zn = ra()` is
line 5, `end` is line 6, `say 'after' zn` is line 7. So the body clause is **line 5**, and the
comment's three "line 4"s were wrong. The stanza's recorded expectations were already right; only the
prose was.

**I2, stanza 1 of the same file:**

```
oracle / tree-walker / ir, all rc 0, stderr empty, cmp equal:
h ran 3
g ran 3
after 3
```

**`corpus/lang/do_clause_boundaries.rex`**, at the committed tree:

```
oracle / tree-walker / ir: rc 0, stderr empty (0 bytes), stdout cmp-equal on all three
A trap2 42 / A body / A after set 5
B trap2 54 / B after set 5
C trap2 65 / C after set 5
D trap2 78 / D trap3 79 / D after set 5
E trap2 90 / E after set 5 / E trap3 91
F after set / F trap2 102
G trap2 110 / G body 1 / G body 2 / G after set 5
H trap2 121 / H after set 5
I trap2 131 / I after set 5
J trap2 139 / J after set 5
K trap2 149 / K body / K after set 5
```

**`corpus/lang/condition_queue_drain.rex`**, at the committed tree:

```
oracle / tree-walker / ir: rc 0, stderr empty (0 bytes), stdout cmp-equal on all three
A h1 36 / A after 1
B h1 43 / B h2 43 / B after 3
C h1 46 / C h2 46 / C h3 46 / C after 7
D h1 54 / D after 1 / D h2 55
E h1 67 / E h2 68 / E h3 67 / E h1 67 / E h2 68 / E after 1 / E h3 69
```

Block C's transcript is the evidence for I3 below: all three of its handlers report the assignment's
own line, and none of them requeues.

## Finding by finding

### I1 -- rewrite (program) and regenerate (capture)

`do_clause_boundaries.rex`'s rule sentence is load-bearing: the file's whole subject is which clauses
a plain `DO` ends, and the rule is what the blocks are read against. So this is an anchor rather than
a deletion. It now states the half the blocks actually exercise -- what a handler queues while it runs
is owed to the next boundary -- and hands the other half to `condition_queue_drain.rex` by name
instead of restating it. I checked that every block does exercise the deferral half: `h1` requeues `c2`
unconditionally in that program, so it is not a claim about the blocks that set `zq`.

Capture: regenerated, never hand-edited, as above.

`deliver_pending_traps`'s stale lead sentence: **deleted**, the drain summary promoted to first
paragraph. Nothing else in that doc moved.

### I2 -- anchor

The number is what invited the error, so the number is gone from the sentence. The comment now names
the clause (`zn = ra()`) and reads the transcript's `5`/`6` back off it, which is a thing the reader
can point at rather than count.

### I3 -- rewrite, with the reason stated

Read from the program: `zq = 0` is set before block A and is not changed again until block D sets it
to 1; block E sets it to 2; every handler's requeue is behind `if zq > 0` or `if zq > 1`. So blocks A,
B and C requeue nothing at all, and the transcript above confirms it -- block C's three handlers all
report line 46 and there is no fourth line. The separating blocks are D and E, and the comment now
says so and says why, following block E's own comment as the model.

One correction to the finding's own wording, which I did not copy: it calls the engine that block C
fails to separate a "deliver-one-and-stop" engine. That is not right -- a deliver-one-and-stop engine
would run block C's `h2` and `h3` at *later* clauses, which is visible. The engine block C cannot
separate is the **drain-until-empty** one, which is the half the surrounding paragraph is about. I
wrote "deliver-one-and-stop" first, caught it in self-review, and the committed text says
drain-until-empty.

### I4 -- rewrite, and the inconsistent rulings

The five `run.rs` sites, `plan.rs` and both `lib.rs` sites now name `run/tests.rs`. Two more of the
same defect turned up in the same sweep and are fixed with them:

* `tests/trace_oracle.rs:112` named `run.rs`'s `task_9s_two_new_indents_are_the_oracles_own_and_
  normalisation_cannot_see_them`; it is at `run/tests.rs:3236`.
* `tests/corpus.rs:146` named `rexx-exec/src/run.rs` unit tests as DEVIATION 0's pinned witnesses; all
  three are in `run/tests.rs`.

And one that is a different defect at the same site: `run.rs`'s `invoke_call` comment named
`current_clause_line_is_restored_after_a_nested_call_or_signal`, which **does not exist**. The test is
`current_clause_line_is_restored_after_a_nested_expression_call` (`run/tests.rs:3423`), whose own doc
comment describes exactly the mechanism the `run.rs` comment cites. Corrected with the path.

The three the review called imprecise: **I touched none of them**, and the reason is the same for all
three. Each says "`run.rs`'s own test module" / "`run.rs`'s own copy" / "`run.rs`'s own tests" -- a
statement about which *module* owns the thing, and `run.rs` still declares `mod tests;`, so `run`'s
test module is exactly what `run/tests.rs` is. None of them asserts that a named item sits in
`run.rs`'s bytes, which is what made the eight false. Rewriting a true sentence is the operation this
branch has measured as its most reliable source of new false ones, so they stay:
`eval.rs:1499`, `queue.rs:244`, `error.rs:2071`.

**The two rulings were inconsistent, and the later one held.** `4e71fe0d3` met these comments, judged
them false, and recorded them as found-and-not-fixed. `b0ea317b6` met the identical situation with six
comments naming a deleted `builtin/mod.rs` and fixed them, on the grounds that `CLAUDE.md` requires a
comment stating something false to be corrected or removed rather than hedged. Both cannot be right.
The rule is written without an exemption for "found while doing something else", so `b0ea317b6`'s
reading is the one that holds, and this change applies it to the set `4e71fe0d3` left.

### I5 -- add a gate, with its negative control run first

**What I removed:** `deliver_one_pending_trap`'s marking loop in `run.rs`,

```rust
for pending in self.pending_traps.iter_mut().skip(queued_before) {
    pending.queued_during_delivery = true;
}
```

replaced by `let _ = queued_before;` so the unused binding did not change what was being measured into
a compile error. Applied in place at `3492b0b9e`, with `run.rs` copied to the scratchpad first and
restored from that copy afterwards -- not from git, per `CLAUDE.md`'s own rule about a harness that
restores with `git checkout --`. `git status --short` and `git diff --stat` were both empty after the
restore, and the marking loop re-read at the site.

**What each profile said:**

| Run | Result |
|---|---|
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --release --workspace --no-fail-fast` | **exit 0**, 1509 passed, 0 failed |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (debug) | **exit 101**, 1503 passed, **6 failed** |
| `memcap 8G cargo test --workspace --no-fail-fast` (debug, ungated) | **exit 101**, 1503 passed, **6 failed** |

Every one of the six failures is the same panic:

```
thread 'rexx-interp' panicked at crates/rexx-exec/src/clause.rs:475:9:
a clause at line 39 began while a condition queued by this activation's clause at line 38 was
still waiting: some construct ran an instruction inside its own step without ending its header
clause first
```

The six: `corpus_differential` (`tests/corpus.rs`), `both_engines_agree_across_every_population`,
`both_engines_agree_on_every_branch_shape`, `both_engines_agree_on_every_case_file`,
`the_known_engine_divergences_still_diverge_exactly_as_recorded` (all `tests/ir_dual.rs`), and
`the_l0_subset_passes_again_under_collect_on_every_allocation` (`tests/collect_stress.rs`).

**Two corrections to the review's own figures.** It reported *five* reddened tests; I measured six,
identically with and without `REXX_CORPUS_GATE=1`, so the gate is not the difference. Its 1509-passed
release figure reproduced exactly. I did not carry either count into `CLAUDE.md`, since a comment
there may not name the size of a set; the counts live here, where they are measurements.

The gate text names `Interp::enter_clause`'s tripwire and the temps-frame watermark `SteppedClause`
carries as things that live only in a debug build, without claiming they are the whole set. The
watermark's own comment at the site says "Cheap enough to leave on in debug and absent in release",
which is the check on that half. `[profile.release]` is untouched.

The quoted command's expected answer against the tree as committed is stated in the gate entry itself
-- exit 0, no failures -- and it is checked below.

### M1 -- delete the universal, point at the assertion

`git show de05ea581` alone exits 0; **path-qualified it exits 128**: `git show
de05ea581:rust/crates/rexx-exec/tests/ir_dual_cases/loop-header-boundaries` -> 128, and the file was
added later, by `b6d548562`. Sixteen commits have touched it since. So "every row was measured to
produce the identical bytes on both engines at `de05ea58`" is a universal over a set that did not
exist when it is claimed to have been measured.

Deleted rather than re-bounded, and replaced with a pointer at the thing that actually checks it:
`both_engines_agree_on_every_case_file` (`tests/ir_dual.rs:907`) walks `tests/ir_dual_cases` with
`datadriven` and calls `render_both_engines` on every stanza, so the property is asserted on every
run. Read directly at the site, including the `cases > 0` guard that stops an emptied directory from
passing vacuously.

### M2 -- one clause, as asked

`leave_clause_without_boundary`'s doc now says that taking `&self` and never reading it is the
contract rather than a stub: spending the `ClauseEntry` is the whole of the work, and that is what
leaves the interpreter untouched.

### Not in the list, found on the way

`f86cb0d58` stacked `end_promoted_branch`'s doc comment onto the end of `leave_select`'s, so the whole
block sat in front of `end_promoted_branch` while `leave_select` had no doc at all -- which made a
correct description of `leave_select` a false description of `end_promoted_branch`. The `leave_select`
half is moved back onto `leave_select`, text unchanged. Same shape as the `deliver_pending_traps`
defect above, from a different commit.

## Gates

Each exit status read on its own, at the committed tree (`b029abe77` content; the runs were the
staged working tree, which is byte-identical to it).

| Gate | Command | Result |
|---|---|---|
| format | `cargo fmt --all --check` | exit 0 |
| lint | `CARGO_TARGET_DIR=<fresh> cargo clippy --workspace --all-targets -- -D warnings` | exit 0, zero lines matching `warning`. Fresh directory removed and recreated; log shows all seven `rexx-*` crates Checked and `rexx-inventory` Compiled, so the linter did re-examine the code |
| tests | `memcap 8G cargo test --release --workspace` | exit 0, 1509 passed, 0 failed |
| corpus gate, unfiltered | `REXX_CORPUS_GATE=1 memcap 8G cargo test --release --workspace` | exit 0, 1509 passed, 0 failed |
| **new debug run** | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | exit 0, 1509 passed, 0 failed |

The debug run's expected answer as written into `CLAUDE.md` -- exit 0, no failures -- is the row
above, so the quoted command carries a stated expectation and that expectation was checked.

`sourceline_oracle` was additionally run on its own after each capture regeneration: exit 0,
`1 passed`.

## Files changed

`rust/CLAUDE.md`, `rust/corpus/lang/condition_queue_drain.rex`,
`rust/corpus/lang/do_clause_boundaries.rex`, `rust/crates/rexx-exec/src/clause.rs`,
`rust/crates/rexx-exec/src/lib.rs`, `rust/crates/rexx-exec/src/plan.rs`,
`rust/crates/rexx-exec/src/run.rs`, `rust/crates/rexx-exec/tests/corpus.rs`,
`rust/crates/rexx-exec/tests/ir_dual_cases/condition-queue-drain`,
`rust/crates/rexx-exec/tests/ir_dual_cases/loop-header-boundaries`,
`rust/crates/rexx-exec/tests/trace_oracle.rs`,
`rust/crates/rexx-parse/tests/sourceline_oracle/condition_queue_drain.txt`,
`rust/crates/rexx-parse/tests/sourceline_oracle/do_clause_boundaries.txt`.

13 files, 91 insertions, 71 deletions. No `.rs` behaviour hunk: the only lines inside function bodies
that moved are comments, and `git diff` carries no change to any expression or statement.

## Self-review

Method: I re-read the whole diff hunk by hunk after the edits were in, checking each new sentence
against the thing it describes rather than against the finding that prompted it -- which is where the
first two below came from -- then re-derived the claims that had a program behind them from the
transcripts above, then re-read the surrounding paragraphs of each edited comment rather than only the
edited lines.

Four findings, all mine, all fixed before the commit:

1. **A false statement in an I3 fix.** "a deliver-one-and-stop engine produces block C's output
   exactly" -- copied from the finding's own wording. It is the drain-until-empty engine that block C
   cannot separate; a deliver-one-and-stop engine gives block C a visibly different transcript.
   Corrected in place. This is the "correction rounds add false statements" shape, arriving inside the
   correction round it warns about.
2. **A weak inference in the I1 fix.** The first draft said "no block here queues two conditions at
   one clause, so no block here reads that half back". The premise is true but the inference is not
   airtight -- a boundary can owe two entries without a clause queuing two, which is exactly what
   `condition_queue_drain.rex`'s block E does. Replaced with a claim about which file owns which
   subject, which needs no inference.
3. **An exhaustive reading in the `CLAUDE.md` entry.** "The tripwires this crate leans on live only in
   that build -- X and Y" reads as an enumeration of a set that has many more members. Reworded to "X
   among them, and Y".
4. **A wrong harness attribution in the same entry.** "the two-engine differential harnesses and
   `collect_stress`" omits `corpus_differential`, which is the *oracle* differential and was one of the
   six. Now names `ir_dual.rs`, `corpus.rs` and `collect_stress.rs`.

Checks behind the count of four: every added or changed sentence in the diff was traced to either a
transcript in this report, a line of source read at its own site (`ir_dual.rs:907` for M1,
`run.rs:5187` for the watermark, `run/tests.rs` for each named test, `Cargo.toml` for
`[profile.release]`), or a `git` command whose output is quoted here. Two claims I could not settle
from a file were settled by running: I2's line numbering, and I3's block C, both above. The count is
of findings I made and fixed, not of hunks; three of the four are in text I wrote in this change and
one is in a claim I copied from the review.

## Concerns left open

* The review's "five tests redden" is six on both debug spellings I ran. Not reconciled; possibly
  measured at a different commit.
* `.superpowers/sdd/2026-08-14-pre-phase-5-defects/ansi-do-boundaries.md:257` still carries the
  pre-drain premise that `docs/superpowers/specs/2026-08-15-ansi-condition-delivery.md` was derived
  from and corrected. Left deliberately, but a reader who finds the ledger before the spec finds the
  uncorrected version.
* `docs/superpowers/specs/2026-08-15-phase-5-inherited-surface.md` is untracked and predates this
  change; not mine and not staged.
