# Task 8 review: `PUSH`, `QUEUE`, and the in-process queue

Reviewing `1663538a..ed7f1e94` (one commit, 9 files) against
`task-8-brief.md`, `task-8-report.md` and `rust/CLAUDE.md`.

Everything below marked **(ran)** was established by executing something.
Everything marked **(read)** was established by reading code, the diff, or the
C++ oracle source. Where a claim in the report is contradicted, the
contradiction was produced by running, not by argument.

## Verdicts

**Spec compliance: MET, with one documented deviation that is justified and
one accuracy defect in a required artefact.** Every file the brief names was
created or modified; the measurement the brief asks for was taken and is
independently reproducible; the unit test exists and is mutation-sensitive;
the KNOWN GAP row exists. The row, however, states as its central claim
something that this same commit made false, and it does not name the
degenerate implementation that actually survives the suite.

**Task quality: GOOD, with three Important findings, all of them about what
the coverage is claimed to be rather than about what the code does.** The
implementation is correct against the oracle, the substitute coverage is real
and I killed it three different ways, the ownership bookkeeping is complete
and touches only the right tables, and the two deviations from the brief are
both correct. What is wrong is the *description* of the safety net: three
separate places (a shipped source comment, the KNOWN GAP row, the report)
attribute the catch to the wrong test and describe a gap that is narrower
than the real one.

## Restored mutations

I mutated `crates/rexx-exec/src/queue.rs` (twice), `crates/rexx-exec/src/run.rs`
(twice) and `corpus/phase-4b.txt` (once) to test whether tests can fail. All
were restored with `git checkout --`; `git status --short` is empty and
`cargo test --workspace` is back at 978 passed / 0 failed / 4 ignored.

## Baseline re-established at `ed7f1e94` (ran)

* `cargo test --workspace`: 978 passed, 0 failed, 4 ignored.
* `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`: `39 of 39
  matching, mode: STRICT`.
* `cargo fmt --all --check`: exit 0. `cargo clippy --workspace --all-targets
  -- -D warnings`: exit 0 (read unpiped).

---

## Spec compliance, requirement by requirement

| Brief requirement | Verdict | Evidence |
|---|---|---|
| Create `rust/crates/rexx-exec/src/queue.rs` | MET | file exists, 160 lines (read) |
| Modify `src/lib.rs`, `src/run.rs` | MET | `mod queue`, `Interp::queue`, ctor init, `instruction_owner` arm, two `step()` arms (read) |
| Modify `tests/owners.rs` | MET | tags, `EXPECTED_OUT_OF_SCOPE`, counts (read + ran) |
| Modify `docs/.../phase-4-exclusions.txt` | MET, defective content | see I2 below |
| Step 1: measure interleaving with a 4c-shaped probe | MET | independently reproduced: oracle prints `C`/`A`/`B` (ran) |
| Step 2: unit test against the queue type | MET | two tests, both mutation-sensitive (ran) |
| Step 3: implement `queue.rs` and the two arms | MET | oracle byte-match under `TRACE R` (ran) |
| Step 4: KNOWN GAP row + single-program rule | MET, defective content | row present at `phase-4-exclusions.txt:762-813`; see I2, M3 |
| Step 5: run the suite, update `owners.rs`, commit | MET | all five gates reproduced (ran) |
| "ships with zero differential coverage" | DEVIATED, justified | a corpus witness was added; see "Deviation A" (ran) |

### The measurement is right (ran)

Fresh `mkdir`, oracle wrapper, three separate descriptors:

* `push "a"; queue "b"; push "c"` then three bare `PULL`s: stdout `C`/`A`/`B`,
  empty stderr, rc 0. So storage order before `PULL`'s upcase is `c`, `a`,
  `b` -- `PUSH` at the head, `QUEUE` at the tail. Exactly what `queue.rs`'s
  unit test pins.
* The corpus program's own shape under `TRACE R`: the oracle traces
  `>>>   "hello"` for `push vv`, `>>>   "hello there"` for `push vv 'there'`,
  and `>>>   ""` for both the bare `queue` and the bare `push`. The claim
  that a bare `PUSH`/`QUEUE` traces a null string rather than being skipped
  is correct.
* Cross-process, reproduced: `rxapi` running (pid 885), writer program rc 0
  with empty stdout and stderr, a *second* `rexx` process printing
  `queued()` as `0`. The row's correction of the scoping document is right.
* `trace r` + `exit 0`: oracle emits `>>>   "0"`, `rexx-run` does not. The
  corpus program's header is right to end on a bare `exit` and right about
  why.

### The C++ citations are all accurate (read)

`QueueInstruction.cpp:69` is exactly `evaluateStringExpression`;
`SayInstruction.cpp:74` calls the same function, so "shares SAY's
`evaluateStringExpression`" is literally true; the order selector is
`(instructionType == KEYWORD_PUSH) ? Activity::QUEUE_LIFO :
Activity::QUEUE_FIFO`; `RexxInstruction.cpp:249`'s else arm is
`traceResult(GlobalNames::NULLSTRING)` and its then arm is `requestString()`
-- both cited correctly. `ast.rs:691-695` does say `PUSH`/`QUEUE` share
`RexxInstructionQueue`.

### The sourceline fixture is oracle-measured, not copied (ran)

I regenerated `crates/rexx-parse/tests/sourceline_oracle/push_queue.txt` from
a scratch directory using the driver `sourceline_oracle.rs`'s module doc
documents, against a *copy* of the corpus file (not the repository file).
Output is byte-identical to the committed fixture, `count 31`.

### Cannot verify from diff

* Whether the unit test was written before the implementation (Step 2 before
  Step 3). The diff is one commit. Both tests are mutation-sensitive, which
  is the property that matters, so this is not a concern -- only unverifiable.
* Whether the oracle probe was run before the implementation existed. Its
  values are independently correct either way (ran).
* Whether `assertions` is still 4224/4259. Taken from the controller's
  baseline; I did not re-run it.

---

## Task quality

### The substitute coverage is real. I killed it three ways.

**Mutation 1 -- swap `push` and `queue` (`push_front`/`push_back` exchanged).**
Both unit tests go red (ran):
```
test queue::tests::queue_alone_is_plain_fifo ... FAILED
test queue::tests::interleaved_push_and_queue_match_the_oracle_order ... FAILED
```
So the tests pin the interleaving, not what the implementation happens to do.

**Mutation 2 -- collapse `push`'s `push_front` to a second `push_back` only.**
`interleaved_...` FAILED, `queue_alone_is_plain_fifo` ok (ran). This is
exactly the split the report claims, and it is the point of having the second
test: the adjacent success is what shows the FIFO half is right for the
ordinary reason.

**Mutation 3 -- `Push | Queue => Ok(Flow::Next)`, the brief's own pure no-op.**
STRICT corpus drops to **38 of 39** (ran). So `lang/push_queue.rex` does
distinguish a correct implementation from a no-op. The implementer's claim
about the witness is true.

**Mutation 4 -- keep everything, delete only `self.queue.push(line)` /
`self.queue.queue(line)`** (evaluate, trace, discard). `cargo test
--workspace` **978 passed / 0 failed**; STRICT corpus **39 of 39** (ran).
Nothing in the tree catches this. This is the real gap, and it is not the gap
any document describes.

**Mutation 5 -- delete `lang/push_queue.rex` from `phase-4b.txt`** (leaving
the `owners.rs` change in place). `coverage.rs:639` fails with
`InstructionKind: 2 in-scope variant(s) unwitnessed by the phase subsets:
Push, Queue` (ran). Deviation A is fully justified.

### Deviation A -- the added corpus witness. Both halves verified.

The implementer argues (a) criterion 1 in `tests/coverage.rs` is a different
requirement from the brief's "zero differential coverage", and (b) without
the witness `cargo test --workspace` would regress.

(b) is verified by Mutation 5 (ran). (a) is verified by Mutations 3 and 4
together (ran): the witness closes the *evaluate/render/trace* half and
provably does not close the *storage* half. Criterion 1 asks for a program
that **constructs** the variant; the brief's claim is about what a run can
**observe**. Distinct requirements, correctly distinguished.

The corpus file itself is well built: `TRACE R` throughout, a bare `EXIT`
with a stated reason I confirmed is still true (ran), and a header that says
plainly what it cannot pin. Its own header is honest where the KNOWN GAP row
is not.

### Deviation B -- refusing to edit `lib.rs`'s name match. Correct.

`lib.rs:409-447` is `let name = match kind { ... }` with no `_` arm, ending
in `kind.keyword().unwrap_or("an instruction")` (read). It lists `Do`, `If`,
`Interpret`, `Procedure`, `Return`, `Say`, `Select`, `Signal`, `Use` -- all
implemented. It is a total display-name function over `InstructionKind`, not
a scope table. Removing `Push`/`Queue` would be a non-exhaustive-match error
with no arm to receive them. The refusal is right and the reasoning in the
report is right.

### Ownership bookkeeping: complete, and only the right places (read + ran)

* `lib.rs:750` `instruction_owner` -- `Some("4b")` to `None`. Edited.
* `tests/owners.rs:158-159` per-variant tags to `Owner::InScope`; the two
  `EXPECTED_OUT_OF_SCOPE` rows deleted (`owners.rs:334-337`);
  `variant_counts_match_the_audited_split` counts updated to 28 in scope and
  0 in `Owner::Phase("4b")`; `INSTRUCTION_TAGS.len() == 40` correctly left
  alone. All four sites edited.
* `tests/loud.rs` -- both witness rows removed, `assert_eq!(expected_
  instructions.len(), 12)` and `in_scope_counts_match_the_audited_split`'s 28
  both updated. Both the witness rows **and** the pinned counts.
* `src/plan.rs:156-157` and `tests/coverage.rs:231-232` correctly untouched:
  both are `|`-grouped expression walkers alongside `Command`/`Say`/`Return`,
  where no edit is correct.
* No residual `owner: "4b"` witness row and no residual `Owner::Phase("4b")`
  tag anywhere (the one grep hit in `loud.rs` is inside the comment text that
  says so). `SPLIT_TABLE_PHASES` keeping `"4b"` is correctly left alone --
  `assert_owner_strings_are_split_table_phases` only requires owner strings
  to be a subset, so an unused phase name is not stale.

### The `loud.rs` doc-comment correction: in scope, but heavy

The pre-existing comment above `INSTRUCTION_WITNESSES` said "17 entries -- 16
coarse ... `Call` becomes two rows and `Signal` becomes one" while the array
held 14 entries and `assert_witness_set_is_complete` asserted 14 a few dozen
lines below (verifiable from the diff's own context, read). It was false, it
sat directly above the array being edited, and CLAUDE.md requires correcting
rather than hedging a false comment. Correcting it was in scope.

The correction is now a five-sentence changelog of three previous staleness
events. The counts it narrates are all machine-checked by the two assertions
immediately below, so the prose history carries no verification weight and is
the fourth generation of a comment whose entire failure mode is going stale.
Not a defect; noted because "the comment that keeps rotting got longer again"
is the shape worth watching.

---

## Findings

### Critical

None.

### Important

**I1. `rust/crates/rexx-exec/src/queue.rs:119-121` -- false statement in a
shipped comment. (ran)** The test's doc says `step`'s `Push`/`Queue` arms
"are exercised separately by the loud-witness removal in `tests/loud.rs`".
They are not. Removing the witness rows removed the only thing that ran a
`push`/`queue` program from that file; `loud.rs` runs programs for
*out-of-scope* variants only, and its module doc says so explicitly. Measured:
with both arms replaced by `Ok(Flow::Next)`, `cargo test -p rexx-exec --test
loud` is 8 passed / 0 failed, fully green. The gate that actually catches it
is `corpus.rs` via `lang/push_queue.rex` (38 of 39). CLAUDE.md requires this
be corrected, not hedged. The same false claim is in `task-8-report.md:138-142`
("is caught instead by `tests/loud.rs`'s removal of the `Push`/`Queue`
witness rows -- the loud gate would fail loudly on any real program touching
either keyword"), which is the report's entire "Test that could fail" answer.

**I2. `docs/superpowers/plans/phase-4-exclusions.txt:762-767` -- the KNOWN GAP
row's central claim is false as of the commit that added it. (ran)** It says
"no corpus program can distinguish a correct PUSH/QUEUE from `Push | Queue =>
Ok(Flow::Next)`, a pure no-op". `lang/push_queue.rex`, added by this same
commit, distinguishes exactly that: STRICT corpus goes to 38 of 39 under that
mutation. The row's own later paragraph ("WHAT COVERAGE EXISTS INSTEAD")
contradicts the first. The supporting measurement quoted (`push "X"` / `queue
"Y"` gives empty/empty/rc 0) is true but is about a different program than
the one this task shipped.

**I3. The degenerate implementation that actually survives every gate is named
nowhere. (ran)** Deleting only the two `self.queue.push(line)` /
`self.queue.queue(line)` calls -- evaluate the expression, trace it correctly,
then discard the line -- leaves `cargo test --workspace` at 978/0 and the
STRICT corpus at 39/39. `queue.rs`'s unit tests construct `Queue` directly and
so cannot see the wiring; the corpus witness sees only the trace. So nothing
in the tree asserts that `run.rs` writes to `Interp::queue` at all. Both the
KNOWN GAP row (I2) and `queue.rs:122-124` describe the strictly weaker
`Ok(Flow::Next)` gap, which is closed. The accurate statement of the gap is
"no test distinguishes a correct `PUSH`/`QUEUE` from one that evaluates and
traces its expression and then discards the line", and a reader of the row
today would come away believing less is covered than is, and more is covered
than is, at the same time.

### Minor

**M1. `queue.rs:81-82` -- "oldest `PULL` target at the front" is misleading.
(read)** With `PUSH` inserting at the front, the front holds the *most
recently written* line. The module doc's own phrasing at `queue.rs:32-34`
("the head is the next line `PULL` will remove") is the accurate one; the
struct doc should use it.

**M2. `queue.rs:122-124` -- "the whole of Task 8's coverage for the property
that would make a degenerate `Push | Queue => Ok(Flow::Next)` wrong" is
overstated, for the same reason as I2. (ran)** `lang/push_queue.rex` is also
coverage for that, and is the only coverage for the half `queue.rs` cannot
reach.

**M3. `phase-4-exclusions.txt:327-329` -- the section header is now false and
was hedged around rather than corrected. (read)** The header says "Each is a
real divergence with no owner assigned". The new row opens "UNLIKE THE REST OF
THIS SECTION, this is not a measured divergence from the oracle -- nothing is
known to be wrong". The brief mandated the row's location, so the row belongs
where it is; it is the header that now states something false about its own
contents. On the framing question the controller raised: the row's "nothing
is known to be wrong" is **honest** -- I found nothing wrong with the
implementation, and the row flags its own exceptional status rather than
hiding it. What is not honest is I2, in the same row.

**M4. `run.rs:1834-1861` -- two arms identical but for the last call, and the
comment survives in only one. (read)** The `Push` arm's `None =>` carries "No
expression queues a null string, traced as one -- the same `else` arm `SAY`'s
own blank line takes"; the `Queue` arm's bare `None => Vec::new()` has
nothing. A single `|`-pattern arm selecting the sink at the end, or a shared
helper, removes both the duplication and the asymmetry.

**M5. The unit tests pin a convention 4c is free to break, and nothing guards
it. (read)** They assert the internal `VecDeque` order is `[c, a, b]`. That
follows from the oracle's `C`/`A`/`B` only under the added premise "`PULL`
removes from the front". `Queue` has no removal method, so a 4c `PULL`
implemented as `pop_back` would leave both unit tests green and print
`B`/`A`/`C`. The premise is stated in the module doc but is not enforced by
any type or test. Worth a line in the handoff to 4c, or a `pop_front`-shaped
method whose doc is the guard.

**M6 (pre-existing, not this task's). `tests/owners.rs:185-186` still says
`InstructionKind::Call` "keeps its `Owner::Phase("4b")` because
`Call::Trap`/`Call::Qualified` are still loud". (read)** False since Task 7:
`Call` is `Owner::Phase("Phase 5")` and `Call::Trap` is in scope -- the same
file says so at line 120 and in `EXPECTED_OUT_OF_SCOPE`. Task 8 did not touch
these lines and is not responsible, but it is the second stale comment of the
same vintage in a file this task edited, and the implementer corrected the
one in `loud.rs` on exactly this reasoning.

**M7. `lang/push_queue.rex` exercises string values only. (read)** "Render it
to string form" is pinned for literals and concatenation; no number goes
through `to_text` here, so a rendering defect specific to numeric form would
not show. Low risk -- `to_text` is shared with `SAY`, which is heavily
covered -- but the header's claim to pin "render it to string form" is
broader than what the program does.

## What is not wrong

Stated because I looked and found nothing:

* No `unsafe`; no new allocation site (lines are `Vec<u8>`, no `ObjRef`, so
  `Interp::alloc_with` does not apply and there is nothing for the collector
  to trace -- `queue.rs:81-88` says this and is right).
* `roots.push_temp(value)` before `to_text` mirrors `SAY`'s arm exactly
  (`run.rs:884-902`).
* Nothing fails as a plausible Rexx condition where it should fail loudly:
  both arms always succeed, matching the oracle.
* No scope creep beyond the two deviations and the `loud.rs` comment, all
  three defensible.
* No dead code. `Queue::push`/`Queue::queue` are both called from `run.rs`;
  clippy is clean at `-D warnings`.
* `--` used throughout, no em-dashes introduced.
