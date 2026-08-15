# Task 2 re-review: fix round 1

Reviewed: `355017720` alone, against `task-2-review.md` and `task-2-report.md`.
The other three commits in the review range (`d07a8988e`, `fdd992cdb`, `3b78deaa7`) are
another task's and are not read here.

* **Findings addressed: F1, F2, F3, F4, F5, F6, F7, F9, F10 all touched. F8 declined by the coordinator's own ruling.**
* **Verdict: ANOTHER ROUND NEEDED.** Two of the replacements are themselves false,
  measured, and both are in code comments -- one of them in the very doc the round
  rewrote to fix a self-contradicting mechanism.

The two independent checks the dispatch asked for came out: the `datadriven`
mechanism is exactly as the implementer describes it and I reproduced two of the
held-out mutations; both register numbers in the landed golden-test doc are
right. What is wrong is the *condition* one of those numbers was measured under,
which the doc does not state, and a universal quantifier that replaced a count.

---

## How this was verified

A detached `git worktree` at `355017720` under the session scratchpad, never the
repository tree. `ootest` symlinked in (untracked here). `compile.rs` and
`run.rs` backed up with `cp` into a private directory, restored from the copy,
and every restore verified with `sha256sum -c` plus `git status --short` (clean
but for the `ootest` symlink). `rexx-run` and the test binaries rebuilt after
every restore, and the crate re-run green at the end: **748 passed, 0 failed**.

**Gates, re-run rather than taken from the report.** The worktree's
`target/` was created by this review, so the clippy run is the clean-directory
variant `rust/CLAUDE.md` asks for.

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0; no line matching `^warning` or `^error` anywhere in the output |
| `memcap 8G cargo test --workspace --no-fail-fast` | exit 0, **1468 passed, 0 failed, 4 ignored** -- the baseline exactly |

**Mutations, each its own build, each restored and re-verified before the next:**

| id | mutation | what it was for |
|---|---|---|
| R-MU2 | `chunk_node_at` loses its `Do`/`Loop` arm, case directory holding only `loop-header-values` | F1's re-measurement |
| R-MU1 | the header arm never takes the native path, same isolation | the "stays green" half of F1 |
| R-MU3 | every header slot resolves to slot `0`, same isolation | the "three mutations" count |
| R-LEAK | `push_native`'s binary arm never releases the operand register | the golden doc's first bullet, with and without the `debug_assert_eq!` |
| R-EARLY | the header's registers released at the region's end instead of past the `END` | the golden doc's second bullet |
| R-SPARE | one leaked register per header slot, `debug_assert_eq!` removed | the `compile.rs` comment's own measurement |
| R-MU6 | `Op::TraceKeyword` emitted in front of the slot's ops, the new check removed | the F5 replacement, and the `DO OVER ... FOR` row alone |

I also captured the new `trace r` row against the C++ oracle myself, from a
fresh `mkdir`ed directory, stdout/stderr/status as three descriptors.

---

## Check 1 -- the `datadriven` mechanism and the held-out re-measurement

**Confirmed, every clause, from `datadriven-0.9.0/src/lib.rs`.**

* `test_files` (`lib.rs:286`) builds its list from `fs::read_dir` into a
  `VecDeque` walk and **never sorts**. `walk_exclusive` (`lib.rs:238`) iterates
  that order.
* `walk_exclusive` runs **every** file, pushing at most one accumulated
  `tf.failure` per file and panicking at the end with all of them. The only
  `break` is in `TestFile::run_normal` (`lib.rs:523`) and it is within one file.
* Neither is what stops this harness: `render_both_engines`'s engine-vs-engine
  `assert_eq!` is inside the callback. My R-MU2 run panicked at
  `crates/rexx-exec/tests/ir_dual.rs:946`, out of `walk` entirely.

**Directory order, both places, reproduced:** `os.listdir` on the repository
tree returns `loop-header-boundaries` before `loop-header-values`; a fresh
worktree of `355017720` returns them the other way round. The first round's
observation was an artifact of one directory's inode order, exactly as the fix
says.

**Held-out mutations reproduced.** With every other case file moved out:

| run | result |
|---|---|
| R-MU2, file alone | **FAILED** -- `stdout differs between engines`, left `"1\n2\n"`, right `""`: the call-bound row |
| no mutation, file alone (control) | **ok** -- the isolation itself does not redden the test |
| R-MU1, file alone | **ok** -- the promotion's contract holds: the bytes do not move |
| R-MU3, file alone | **FAILED** -- left `"1\n3\n"`, right `"1\n"` |

So F1's withdrawal is correct and its replacement measurement is real. R-MU3 is
the one new fact: it is a **fourth** mutation the file alone catches, which
matters for the count the file's header now carries (N4 below).

## Check 2 -- both register numbers in the landed golden-test doc

**Both numbers are right. The condition under which one of them was measured is
not stated, and the sentence around it is false as the tree stands (N1).**

* R-EARLY (`registers.release(outer)` at the region's end instead of the
  deferred release past the `END`): the body's assignment lands at
  **register 0** -- `14: LoadConstant dst=0 ... 16: Store index=1 at=2 src=0`.
  Exactly two tests move in the whole crate:
  `a_header_operands_register_goes_back_to_the_body` and
  `a_nested_loops_registers_sit_above_the_enclosing_loops_and_a_later_loops_reuse_them`.
  `ir_dual` stays green, population sweep included. **The doc's second bullet is
  correct in every particular**, and so is its conclusion that the harmful
  direction is invisible to every differential harness in the tree.
* R-LEAK (`push_native`'s binary arm never releasing the operand's register)
  **with the `debug_assert_eq!` also removed**: the assignment lands at
  **register 3** -- `14: LoadConstant dst=3 ... 16: Store index=1 at=2 src=3` --
  and exactly two tests move, this one and
  `a_chain_of_operators_reuses_the_destination_register`. The number and the
  pair are right *under that condition*.
* R-LEAK **as the tree actually stands**, i.e. the single mutation the bullet
  names: **seven tests move**, and `ir_dual` goes to 8 passed / 1 failed. See N1.

---

## The findings, one line each

* **F1 -- addressed, and the replacement is measured and correct.** The plan's
  false paragraph is withdrawn, the mechanism is restated accurately, the case
  file's header says how the measurement has to be taken, and three of the four
  rows carry what they caught. Two overclaims ride along it (N3, N4).
* **F2 -- addressed; the message and the two-direction comment now say what each
  side of the equality means.** The comment's own measurement sentence is loose
  in one clause (N5).
* **F3 -- addressed.** `HeaderRole::keyword()`'s doc now reads "or `None` for
  the roles this table withholds one from", with `Initial` measured to match the
  oracle and `OverFor` measured not to; both variant docs say which is which, so
  the sentence checks out against its own neighbours.
* **F4 -- addressed, and the recording is true where I could measure it.** The
  golden test's doc now says the harmful direction is invisible to the
  differential harnesses; R-EARLY confirms it. The leak half of the same block is
  N1.
* **F5 -- addressed but the replacement is wrong.** "reddens thirteen tests" is
  gone; what replaced it ends in "every pinned `DO` stream", which is false. N2.
* **F6 -- addressed.** The self-contradiction is gone and the two directions are
  named as opposites with a number each; both numbers verified (Check 2). The
  block it sits in is N1's.
* **F7 -- addressed, properly.** The `trace r` row landed. I captured its
  program under the oracle myself: all four diffs empty, and the recorded
  expected block is **byte-identical** to the oracle's own output, tag for tag.
  R-MU6 confirms it catches the echo-position mutation as a file of its own.
* **F8 -- not addressed, by the coordinator's ruling**, which the review itself
  endorsed ("not editing it was right"). Nothing to re-review.
* **F9 -- addressed.** "Three instructions" is gone. Minor wording note at N6.
* **F10 -- addressed.** `Op::TraceKeyword`'s doc is reflowed; the diff is pure
  rewrap, no meaning moved.

### The two deletions no finding named

Both are in plan Step 4, and neither took a load-bearing fact with it:

* "and six were run over the whole workspace (Task 2's report has the table)" --
  the uniqueness fact it carried survives as "None of these is a *unique* catch
  -- each of those mutations reddens pre-existing tests too." What is lost is the
  pointer to where the mutation evidence lives; the report still has the table.
* "Two rows the dispatched list asked for are not here" -> "One row" -- correct
  arithmetic now that `trace r` is back, and the `DO FOREVER` row's own reason
  is unchanged. The clause "and no mutation reddened it" was dropped with it,
  which is right: it rested on the inference F1 withdrew.

One real casualty, worth restoring in the next round rather than a defect in
itself: the deleted sentence recorded that the call row and the `DO OVER ... FOR`
row "record shapes nothing else holds". The reviewer had verified the second half
independently (no other test or case file in the crate runs a `DO OVER ... FOR`
under trace). That verified fact is now nowhere in the plan; the case file's own
row comment says only that the row pins our answer while the gap is open.

---

## New defects this round introduced

### N1 -- the golden test's leak bullet is measured under a condition it does not state, and the sentence above it is false as the tree stands

`golden_tests.rs`, `a_header_operands_register_goes_back_to_the_body`:

> **The stream pins both ways of getting that wrong, and they are opposite
> failures.** Measured, each as its own mutation, and in both directions the
> whole `ir_dual` suite stays green -- the population sweep included:
>
> * `push_native` never releasing the operand's register puts the assignment at
>   register 3. [...] This test and `a_chain_of_operators_reuses_the_destination_register`
>   move, and nothing else in the crate.

Measured, that mutation alone, on the committed tree:

```
ir::golden_tests::a_chain_of_operators_reuses_the_destination_register
ir::golden_tests::a_header_operands_register_goes_back_to_the_body
ir::corpus_shape_tests::every_corpus_body_compiles_the_minimum_promotion_set_to_its_own_ops
corpus_differential
keyword_assertions_differential
the_exempt_set_matches_the_current_failures
both_engines_agree_across_every_population        <- ir_dual, 8 passed / 1 FAILED
```

`both_engines_agree_across_every_population` fails with the new assertion's own
message, from `crates/rexx-exec/src/ir/compile.rs:418`, `left: 4  right: 3`. So
in the leak direction the `ir_dual` suite does **not** stay green, and the test
the sentence singles out as staying green -- the population sweep -- is the one
that goes red. `a_header_operands_register_goes_back_to_the_body` itself does not
show register 3 either; it panics on the `debug_assert` before `render` is
reached.

Both halves become true only if the `debug_assert_eq!` is removed as well, which
I confirmed: seven failures collapse to the two the doc names, and the stream
then shows `dst=3`. The report's own F2 section states that condition for its own
mutation ("with the assertion removed"); F4 and this doc drop it. The first
round's `MU5` row already recorded the seven-way spread and that "the new
`debug_assert_eq!` fired seven times" -- so the report now contains both
statements, and they cannot both be true.

This is the same defect class the round existed to remove: a measurement of one
configuration written as a claim about another. It is also the one place where it
matters most, because the sentence is what tells the next reader that this golden
test is the tree's only witness for the leak.

**Fix:** say the condition. "With the `debug_assert_eq!` above removed, so the
mutation reaches the stream at all: ... two tests move." And drop or qualify "in
both directions the whole `ir_dual` suite stays green" -- it is true of the
harmful direction and false of the leak direction as the tree stands, which is
worth saying, because it is the assertion doing the work.

The same false claim is in `task-2-report.md`'s F4 section ("Two mutations, each
at the final state, and in both the whole `ir_dual` suite stays green including
the population sweep").

### N2 -- "every pinned `DO` stream" is false, and it is a universal quantifier over an in-repo enumeration

`compile.rs`, `assert_keyword_echoes_precede_their_value`'s doc, the sentence
that replaced "reddens thirteen tests":

> Emitting the echo in front of the slot's own ops instead of behind them, with
> this check removed, moves the population, loop-shape and case-file sweeps,
> `trace_oracle`'s control-variable transcripts and **every pinned `DO` stream**

Measured (R-MU6, the check removed, whole crate, `--no-fail-fast`): thirteen
tests move -- the three sweeps, the three `trace_oracle` transcripts, and **seven**
golden streams. The eighth, `two_constructs_ending_at_one_instruction_release_to_the_lower_mark`,
**stays green**. It pins a `DO` stream -- two of them, and it is one of the five
pinned streams this task's own promotion moved -- but it pins a substring plus
`chunk.registers`, and this mutation reorders ops inside a slot group without
changing any op index, so the substring still matches. It does move under
R-SPARE, which is what proves it is a `DO` stream pin and not something else.

A count that rots was replaced by a universal quantifier over an enumeration
that lives in this repository -- the shape `rust/CLAUDE.md` singles out as the
one that goes stale *and* which is wrong today.

**Fix:** name the failure without quantifying over the set: "...and the pinned
streams whose `DO` header carries a keyword echo", or simply stop at the sweeps
and the transcripts, which are the categories that make the point.

### N3 -- "Each row is measured to catch something" is not measured for the `DO OVER ... FOR` row

Plan Step 4's new lead sentence claims a per-row property; the sentences under it
name three rows, and the file has four. The harness cannot produce a per-row
answer for the fourth from a whole-file run, because it panics at the first
disagreeing stanza -- under R-MU6 that is the `to`/`by`/`for` row, which is first
in the file, so nothing later in the file is exercised at all. The report's table
records no run for that row.

I measured it: with the `DO OVER ... FOR` stanza as the only file in the
directory, R-MU6 **FAILS** on stderr, `>K>   "OVER" => "abc"` moving in front of
its own `>V>` line. So the claim is *true* and was *unmeasured* -- the word
"measured" is the false part, and it is one run away from being earned.

The case file's header has the matching overclaim: "each row's own comment names
the one it caught". The `DO OVER ... FOR` row's comment names nothing it caught,
and the call row's comment names two rather than "the one".

### N4 -- "under three mutations" is a cardinality over an open set, and a fourth is in the task's own table

The case file's header: "Alone, this file reddens
`both_engines_agree_on_every_case_file` under three mutations". R-MU3 -- the
report's own MU3, every header slot resolving to slot `0` -- reddens it as well,
file alone, `"1\n3\n"` against `"1\n"`. The honest form names the mutations
rather than counting them, which the row comments already do.

### N5 -- the `debug_assert`'s comment loses the qualifier the report kept

`compile.rs`: "measured, leaking one per slot with this line removed leaves the
whole `ir_dual` suite green and moves the pinned streams that hold a `DO`, and
nothing else."

Measured (R-SPARE): `ir_dual` green, and six golden streams move --
`a_counted_loop...`, `a_traced_counted_loop...`, `a_block_has_an_empty_header...`,
`a_header_operands_register...`, `a_nested_loops...`,
`two_constructs_ending_at_one_instruction...`. Two pinned `DO` streams do **not**
move (`a_header_bound_that_is_a_symbol_and_one_that_is_a_call_take_their_own_ops`
and `a_header_slot_outside_the_native_set_leaves_the_other_slots_native`): their
bodies are `nop`, so no op's register index changes. On the reading "all pinned
`DO` streams move" the sentence is false; on the reading "what moves is pinned
`DO` streams and nothing else" it is true. The report states the same measurement
unambiguously -- "moves six golden streams, every one of them a pinned `DO`" --
and the comment lost that shape. Same species as N2, milder because the wording
is ambiguous rather than plainly universal.

### N6 (nit) -- "body" now means two things in one doc

F9's replacement reads "The body is `DO`, `nop`, `END`", meaning the program
body, in the doc of a test named
`a_counted_loop_compiles_its_header_to_a_clause_region_and_its_body_to_generic`,
where "its body" is the loop's -- the `nop`. "The program is `DO`, `nop`, `END`"
costs nothing and removes the collision.

---

## What is right, and worth saying so

* The `datadriven` correction is exact -- I checked all three clauses against the
  crate source and reproduced both directory orders. Nothing in the new mechanism
  paragraph overstates what I found.
* The new `trace r` row is a genuine oracle capture: byte-identical to the C++
  interpreter on both channels, and identical to what both engines print.
* The register discipline's two directions are now separated correctly, and the
  numbers -- 3 and 0 -- are both reproduced.
* R-MU1 confirms the contract the file's header claims: a header that stops
  compiling natively moves no bytes.
* The `HeaderRole::keyword()` correction closes the last of the three comments
  that had asserted the `OverFor` gap away, and the three now agree with each
  other and with the oracle.
* No behaviour changed: the only non-comment edits are the assertion's message
  string and the new stanza, and the workspace is green at the baseline numbers.

---

## Process note

The session scratchpad's `backup/` directory is shared between teammates, and
this review wrote `compile.rs`, `run.rs` and `sums.txt` into it before noticing
-- overwriting whatever was under those names. My copies were moved to
`rereview-t2/backup/` and a note left in `backup/NOTE-task2-rereview.txt`. If
another agent's restore-from-backup misbehaves in this session, that is where to
look.
