# Task 6, fix round 1: re-review

Base `5e4f84964`, head `14e25eca3`, two commits (`211763aaa` the fix, `14e25eca3` the sitting).
Read-only on the checkout; everything built and mutated in a copy of `rust/` outside the repository
with `interpreter/`, `oodocs/` and `ootest/` symlinked in and `target/` deleted first. Oracle probes
run from a fresh empty directory, one program per directory, three descriptors read separately.

**Round verdict: ACCEPT with one Important residual to record and four Minors.** Five of the six
findings are closed. Finding 6 is not: the negative control was re-run but no transcript was pasted,
which is the one thing the finding asked for. I ran the control myself and it does what the report
says it does, so the substance is confirmed by this review rather than by the round. Nothing in the
diff is a regression: `fmt` 0, `clippy` 0, corpus gate `112 of 112`, `coverage` 17/17, `sourceline`
1/1, and twenty-six oracle probes agree byte for byte on both engines.

---

## Verdicts

### 1. `is_stem_receiver` over-fires -- ADDRESSED

`eval.rs:1082` (`is_stem_receiver`) and `eval.rs:1102` (`stem_default_is_string_or_number`), test at
`eval.rs:3591`.

The narrowing is exactly as ruled: unset stem true, `String`/`Number` default true, nested stem
chased, `.nil` false. Measured (oracle vs both engines, byte for byte):

| program | oracle | head |
|---|---|---|
| `s. = .nil` `say s. + 1` | rc 159, 97.1, **no frame** | rc 215, 41.1, **no frame** -- frame gone, the 41-vs-97 residual is the ruled pre-existing one |
| `a. = .nil` `b. = a.` `say b. + 1` | rc 159, 97.1, no frame | rc 215, 41.1, no frame -- the nested `.nil` chases correctly |
| `say b. + 1` | frame, scope `String` | identical |
| `s. = "abc"` `say s. + 1` | frame | identical |
| `a. = "abc"` `b. = a.` `say b. + 1` | frame, value `"abc"` | identical -- the nested-stem chase is right in the positive direction too |
| `b. = a.` `say b. + 1` (nested *unset*) | frame, value `"A."` | identical |
| `s. = "abc"` `drop s.` `say s. + 1` | frame, value `"S."` | identical |
| `say b.1 + 1` (compound) | no frame | identical |
| `a. = a.` / `a. = b.` `b. = a.` | frame | identical (no recursion hazard: the assignment does not build a cycle) |
| `say +b.`, `say b. * 2`, `s. = "abc"` `say s. // 2`, `say s. % 2` | frame, operator's own spelling | identical |

**Attacking from the under-firing side, the predicate itself survives.** The narrowing can only
change the answer for a value that `operator_operand_gap` already passed (it answers `Some` for a
class object and for a `Body::Native`, chasing stem defaults on the way), that `to_number` then
failed, and that is a heap `Body::Stem`. The reachable defaults are then: `.nil` (now false, and the
oracle emits no frame -- correct), an inline or heap `Text` (true, correct), a `Body::Num` or numeric
`Text` (never reaches the failing path), and a nested stem (recursed, correct in both directions,
measured above). I could not construct a stem shape where the oracle emits a frame and the predicate
answers false.

The in-crate test really is the instrument it claims to be: I reverted `is_stem_receiver` to the
un-narrowed `matches!(..., Some(Body::Stem { .. }))` in the copy and ran it --

```
test eval::object_operand_tests::a_nil_defaulted_stem_carries_no_operator_frame ... FAILED
panicked at crates/rexx-exec/src/eval.rs:3591:9:
a `.nil`-defaulted stem's own arithmetic failure must carry no operator-forwarded frame -- no
method ever ran to raise from -- but got "       *-* Compiled method \"+\" with scope \"String\"...
test result: FAILED. 0 passed; 1 failed; 694 filtered out
```

-- and green again on restore. It is a plain unit test, red without any gate variable, which is the
right side of the structural/verdict line. No corpus program pinning the `41` was added, as ruled.

*Nit, not a finding:* the test's `assert_eq!(code, 215)` pins this crate's divergent exit status
(the oracle's is 159), so whichever task lands 97.1's forwarding rule must edit this test. The doc
block names the residual, so this is legible rather than surprising, and it doubles as a tripwire.

### 2. The `RAW_STDERR_COMPARISON` reason -- ADDRESSED

`corpus.rs:308-317`, `corpus/phase-5a.txt:175-179`.

The rewritten reason is true and I verified the measurement two independent ways rather than taking
it from the report.

*By reading:* `PREFIX_OFFSET` is 7 and `PREFIX_LENGTH` 3 (`tests/support/mod.rs:141`), and the frame
line is seven spaces then `*-*`, so it *is* a trace line; `normalize_line` copies `0..10` verbatim
and collapses only the space run after, which is one space. The remaining stderr lines are the
clause echo (one space after its marker) and the two `Error` lines, which are not trace lines at
bytes 7..10 and are returned untouched. The frame line's quote count is even, so the continuation
branch never arms.

*By running:* I deleted all six entries from `RAW_STDERR_COMPARISON` in the copy, rebuilt, and ran
the gate -- `112 of 112 matching`, exit 0. So the six programs pass under the normalised comparison
too, which is the operational content of "the normaliser is currently a no-op here".

(First attempt at this experiment produced a false red because restoring `eval.rs` from a backup
gave it an older mtime than the mutated build and cargo did not rebuild. `touch` then rerun. Noting
it because it is the stale-binary hazard in a new costume.)

### 3. The DO-header site -- ADDRESSED, fix rather than escape hatch

`run.rs:7295-7302` calls `eval.rs:1054`'s shared `blame_stem_forwarded_operator`.

**It is genuinely the same call, not a copy.** The frame-rendering body was extracted out of
`arith_left_operand` into one `pub(crate)` function and both sites call it; there is no duplicated
logic block anywhere in the diff. The success path is byte-for-byte the old code (`?` was identity
there -- both functions return `Result<Number, Failure>`).

The three new corpus programs, measured directly rather than through the harness's bounded excerpt:

```
operator_frame_stem_do_initial: byte-identical on both engines, rc=215, frame:        *-* Compiled method "+" with scope "String".
operator_frame_stem_do_to:      byte-identical on both engines, rc=215, frame:        *-* Compiled method "+" with scope "String".
operator_frame_stem_do_by:      byte-identical on both engines, rc=215, frame:        *-* Compiled method "+" with scope "String".
```

and the other three (`plus`, `power`, `prefix_minus`) likewise, all six with matching exit status.

**The DO forms the three probes do not cover behave as the site's own doc predicts**, all
byte-identical to the oracle on both engines:

| form | oracle | ours |
|---|---|---|
| `do i = 1 to 3 for b.` | 26.3, no frame | identical |
| `do b.` (repeat count) | 26.2, no frame | identical |
| `do i = 1 downto b.` | not a keyword; 41.1 on `"1 DOWNTO B."`, no frame | identical |
| `do while b. + 1 > 0` | frame (reaches `arith_left_operand`) | identical |
| `loop i = b. to 5` | frame | identical |
| `do i = 1 to 2; do j = b. to 3; ...` | frame, no indent at depth 2 | identical |
| `trace r` + `do i = b. to 5` | echo, then frame, then echo again | identical |
| `trace i` + `do i = 1 to b.` | `>L>`/`>V>`/`>K>` then the frame | identical |

`FOR` and the bare repeat count go through `whole_nonneg`, exactly as `header_number`'s pre-existing
doc says, and the `TRACE I` case is the strongest single piece of evidence that the new call is
positioned correctly.

### 4. Constraint violations in the new prose -- PARTIALLY ADDRESSED

Closed: `dispatch.rs:861`'s "`pub(crate)` since Phase 5a Task 6" is now "`pub(crate)`: an arithmetic
operator's left operand, and a controlled `DO` header's numeric position, can each raise..." --
passes the strike test. `eval.rs`'s "one of the two object shapes", "The one predicate" and "and
unchanged here" are all gone.

Not closed, and three more of the same shape added this round -- see Minor 1 below.

### 5. The performance staleness test -- ADDRESSED

Re-run by me at HEAD rather than taken from the report:

```
git merge-base --is-ancestor 15a1ffa98 HEAD          -> ancestor
git log --oneline 15a1ffa98..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml  -> 22 commits
```

21 at the round's base `5e4f84964`, 22 at `211763aaa` and at HEAD (the sitting commit touches only
`bench-baselines/`, so it does not list). I checked **all 22** against `progress.md`, not a sample:
every one is recorded. Pin valid. The wider pathspec closes the `rexx-parse/src` gap the brief
named.

### 6. The negative control's transcript -- NOT ADDRESSED

`task-6-report.md:398-402`. The round re-ran the control over six programs and reports it in one
sentence -- "every one reddens by exactly the frame line, nothing else; restored, rebuilt, gate green
at 112 of 112". That is the summary the finding was raised against, now covering six programs
instead of three. No transcript was pasted. The finding said "Re-run it and paste all three in full",
and the brief's verification list repeats it.

**I ran it myself.** Mutation: an early `return` guarded by `std::hint::black_box(true)` at the top of
`blame_stem_forwarded_operator` -- which disables the mechanism at its single definition, and so at
both call sites, rather than something adjacent to it. Result, `REXX_CORPUS_GATE=1`, exit 101:

```
106 of 112 matching
mismatches (6):
  [UNCLASSIFIED] lang/operator_frame_stem_plus.rex: stderr differ
      rust:   stderr="     8 *-* say b. + 1\nError 41 running .../operator_frame_stem_plus.rex li..."
      oracle: stderr="       *-* Compiled method \"+\" with scope \"String\".\n     8 *-* say b. + 1\n..."
  [UNCLASSIFIED] lang/operator_frame_stem_power.rex: stderr differ        ("**")
  [UNCLASSIFIED] lang/operator_frame_stem_prefix_minus.rex: stderr differ ("-")
  [UNCLASSIFIED] lang/operator_frame_stem_do_initial.rex: stderr differ   ("+", `do i = b. to 5`)
  [UNCLASSIFIED] lang/operator_frame_stem_do_to.rex: stderr differ        ("+", `do i = 1 to b.`)
  [UNCLASSIFIED] lang/operator_frame_stem_do_by.rex: stderr differ        ("+", `do i = 1 by b. to 3`)
by owner:
  UNCLASSIFIED: 6
panicked at crates/rexx-exec/tests/corpus.rs:703:5:
STRICT (REXX_CORPUS_GATE) mode: 6 of 112 corpus programs disagree with the oracle
test corpus_differential ... FAILED
```

Restored, rebuilt, `112 of 112 matching`, exit 0. The claim is true; it is this review that
demonstrates it, not the round.

---

## Important: the mechanism covers only the conversion-failure half, and the residual is unrecorded

The report says "Nothing in this task's own scope was left undone" and, this round, "What I could not
close: Nothing new." Both are false. The task's goal sentence is an error **raised inside a native
method that an operator invoked**; the frame is emitted only when the operand fails to *convert*. An
error raised by the forwarded method after conversion succeeds gets no frame. Measured, oracle vs
both engines, all differing in exactly the frame line and nothing else:

| program | oracle | ours |
|---|---|---|
| `s. = 1` `say s. / 0` | `*-* Compiled method "/" with scope "String".` + 42.3 | 42.3, **no frame** |
| `s. = 1` `say s. ** 999999999999` | `*-* Compiled method "**" ...` + 26.8 | 26.8, **no frame** |
| `s. = 'abc'` `say s. & 1` | `*-* Compiled method "&" ...` + 34.901 | 34.901, **no frame** |
| `s. = 'abc'` `say \s.` | `*-* Compiled method "\" ...` + 34.901 | 34.901, **no frame** |
| `numeric digits 1` `s. = '9.9E999999999'` `do i = s. to 5` | `*-* Compiled method "+" ...` + 42.901 | 42.901, **no frame** |

**None of these is a regression** -- I checked `bench-baselines/pinned/rexx-run-15a1ffa98` on the
divide-by-zero case and it emits no frame either, and the mechanism did not exist at the pin. But
the last row is *inside this round's own diff*: `run.rs:7295` blames only in the `Err` arm of the new
`match`, while the `Ok` arm's `round_via_unary_plus(...).map_err(Raised::from)?` raises from the same
forwarded unary `+` on the same stem receiver, and the oracle emits the frame there. The round chose
that branch structure and said nothing about the other one.

The two logical/`\` rows are a second call site entirely (`logical_values`, `eval_prefix`), which the
task never touched.

I am not asking for the fix here -- the ruling that governs finding 3 also permits a named residual,
and this is wider than the DO header. What is missing is the record. The task should carry, in the
report and the ledger: *the operator-forwarded frame is emitted for a 41.1 conversion failure only;
an error the forwarded method raises after converting (42.x, 26.8, 34.901) carries the oracle's frame
and does not carry ours, at both the operator site and the DO-header site, measured.*

---

## Minors

**Minor 1 -- the set-size minor from finding 4 is not closed, and three more were added.**
`corpus/phase-5a.txt:171` went from "The three vary the operator" to "Varies the operator across the
**three** programs" -- the subject noun-phrase changed, the cardinality did not. New this round:
`corpus/phase-5a.txt:178` "a no-op on all **three** programs' stderr"; `corpus.rs:314` "a no-op on
these **three** programs' stderr", which is a Rust comment naming the size of `RAW_STDERR_COMPARISON`'s
own membership -- a set this very commit grew from three to six; `run.rs:7292` "Measured, all
**three** positions". In fairness, `phase-5a.txt` is full of pre-existing "the three X" (lines 3, 31,
49, 67, 142, 149), so the ambient standard in that file is looser than the constraint text; the
`corpus.rs` and `run.rs` ones are Rust comments and have no such cover.

**Minor 2 -- the sitting's universal is false for two of its own rows.** The report says "every
`pinned>head across_builds instructions:u` ratio -- `arith` included -- is now exactly `1.000000`, min
equal to max". In `bench-baselines/phase-5a-arms.tsv`, `alloc4c/ir/small` reads median 1.000000, min
1.000000, **max 1.000001**, and `strings/tw/large` reads **min 0.998014**. The medians are all
1.000000 and the conclusion (layout, not work) stands; the commit message's own wording ("exactly 1.0
this round") is true where the report's added "min equal to max" is not. This is the third round in
which a correction round's prose has restated a measurement slightly wider than the measurement.

**Minor 3 -- a premise in the new `stem_default_is_string_or_number` doc is false of the oracle.**
`eval.rs:1097-1101`: "`.nil`, a class object and one of this crate's own native objects answer none of
the arithmetic operators, so a forward landing on one of them never reaches a method to raise from."
Measured: `s. = .environment` then `say s. + 1` is **rc 0** on the oracle printing `The NIL object` --
the Directory's own `UNKNOWN` answers the `+` send with `.nil`. It does not *raise*, so the
conclusion (no frame) survives, but the stated premise does not. (Ours refuses loudly at rc 120,
"the operator `+` applied to one of the interpreter's own objects is not implemented (Phase 5)" -- a
pre-existing Phase 5 refusal, not this task's.)

**Minor 4 -- `run.rs:7288` "Each header position is rounded through a real unary `+`".** `FOR` and a
bare repeat count are header positions and are not; the function's own doc block eight lines above
says so, and the comment does point at it ("this function's own doc"). Measured behaviour matches
that doc, so this is wording only.

---

## Checks run

* `cargo fmt --all --check` -- exit 0, unpiped.
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0, unpiped.
* `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus corpus_differential` -- exit 0,
  `112 of 112 matching`, before and after every mutation.
* `cargo test --release -p rexx-exec --test coverage` -- exit 0, 17 passed.
* `cargo test --release -p rexx-parse --test sourceline_oracle` -- exit 0, 1 passed.
* Corpus arithmetic: `phase-4a` 31 + `phase-4b` 12 + `phase-4c` 12 + `phase-5a` 57 = **112**, and
  `SUBSET_FILES` (`corpus.rs:603`) names exactly those four. 109 -> 112 is the three additions and
  nothing silently left.
* The three new `sourceline_oracle` fixtures: `count 9 / 8 / 8` matching the `.rex` line counts, and
  each fixture body byte-identical to its corpus program. The walk that requires them
  (`sourceline_matches_the_interpreter_for_every_corpus_program`) is ungated, so a missing fixture is
  structural-red.
* ASCII: zero non-ASCII bytes in every file the diff touches except `run.rs`, whose four `…`
  ellipses are at lines 1146-1599, far outside the diff and pre-existing.
* Twenty-six oracle probes, three descriptors, both engines.

## Out-of-scope observations

* `run.rs:1146`, `:1187`, `:1237`, `:1599` carry `…` in doc comments -- pre-existing, outside the
  diff, but they are the only non-ASCII in any file this task touches.
* `s. = .environment` then `say s. + 1` is rc 0 on the oracle (`The NIL object`) against our rc 120
  loud refusal -- a Phase 5 refusal boundary, untouched by this task, recorded here only because
  Minor 3 rests on the measurement.
* The three new corpus programs' header comments open "Fix round 1, finding 3:", which references a
  review artifact a reader of the corpus cannot see. `environment_object_in_a_loop_header.rex`
  already opens "Phase 5a Task 6, fix round 2", so the shape is house style; the *finding number* is
  new. Not worth a change on its own.
