# Task 4 report: the false attribution in the plan, and three comments naming the wrong function

Commit `88b9e4f15`, on `plan/rust-rewrite`. Parent `d0b7504a5`. Comments and one document only,
no expression changed.

## Step 1: what I checked, and what I found

The tree had moved past the controller's snapshot: HEAD at session start was **`d0b7504a5`**
("Pin the zero-pass requeue as a row, now that it agrees"), not `56d9d1c86`. All four sites the
controller named were still at the quoted line numbers.

| claim | where | found |
|---|---|---|
| "is opened by `run_loop`: the header's, once before the body runs," | `rust/crates/rexx-exec/src/run.rs:5231` | present, at that line |
| "condition are the header's and `END`'s -- both of which `run_loop`" | `rust/crates/rexx-exec/src/clause.rs:526` | present, at that line |
| "`run_loop`'s `Simple` arm opens the header's clause" | `docs/superpowers/plans/2026-08-14-pre-phase-5-defects.md:457` | present, at that line |
| the "Repeating loops with a real header were never wrong" bullet | same file, `:464` | present, at that line |
| "`step`'s `InstructionKind::Do` arm calls `run_loop`, which runs the whole loop before returning" | same file, `:372` | **true, and left alone.** `run.rs:2118` matches `InstructionKind::Do(body) \| InstructionKind::Loop(body)` and `:2128` calls `self.run_loop(...)`; `run_loop` returns only once the construct is over. |

Code claims the brief asked me to confirm:

* **`run_loop` contains no `in_clause`.** Confirmed. `run_loop` is `run.rs:6008-6029`: it calls
  `loop_header_plan`, then `eval_loop_header` or `LoopHeaderValues::default()`, then
  `run_loop_with_header`. Nothing else. The only two `in_clause` calls on the `DO`/`LOOP` path in
  the `Simple` case are at `run.rs:6218` (the header's, before `run_bounded` at `:6228`) and
  `run.rs:6282` (`END`'s, in the `FellThrough | Iterated` arm), and both sit inside
  `run_loop_with_header`'s `match &body.kind { LoopKind::Simple => ... }`.
* **`Op::LoopRun` enters `run_loop_with_header` directly.** Confirmed at
  `rust/crates/rexx-exec/src/ir/drive.rs:1337`: `self.run_loop_with_header(code, index, clause,
  body, source, BodyEngine::Chunk { chunk, registers }, values)`. The compiled engine never names
  `run_loop`.
* **A repeating loop's clauses are opened in `run_repeating`.** Confirmed: `run_repeating` is
  `run.rs:6438-6764` (next `fn` is `do_body_outcome` at `:6765`), and it holds `in_clause` at
  `:6531` (the per-pass header clause, at `header_line`) and `:6696` (the `UNTIL` test).
* **`END` has no clause of its own in a repeating loop.** Confirmed. `header_line` is
  `match &header_clause { Do => do_line, End => end_line, Iterate { line, .. } => *line }`
  (`run.rs:6527-6530`), and `header_clause = HeaderClause::End` is set on fall-through at `:6630`.
  So `END`'s line is carried by the *following* pass's header re-test.
* **`do label zl` with no header is `LoopKind::Simple`.** Confirmed in `create_loop`
  (`rust/crates/rexx-parse/src/instruction.rs:908`): after the label is parsed, the `if self.at_end()`
  branch returns `Loop { kind: LoopKind::Simple, .. }` for a `DO` (a `LOOP` takes `Forever`).
  The brief's `:960-975` still covers that return.

## Step 1, historical code read (for the I3 replacement)

* At `1f4176b47`, `leave_stepped_clause` called `self.leave_clause(entry.entry, code, ran)`
  unconditionally, with no `DO`/`LOOP` arm. So the step's own boundary delivered for every construct.
* At `1f4176b47`, `run_repeating`'s per-pass clause was already present in the shape it has now:
  `let header_line = match &header_clause { ... HeaderClause::End => end_line ... }` followed by
  `let header = self.in_clause(code, header_line, ...)`.
* At `1f4176b47`, `run_loop_with_header`'s `LoopKind::Simple` arm contained **no `in_clause` at all**
  -- it went straight to `run_bounded`, and its `FellThrough` arm did only the `END` trace echo.

## Re-measurement

Built `1f4176b47` in a throwaway worktree with its own `CARGO_TARGET_DIR` (removed afterwards;
`git worktree list` is back to the four pre-existing entries). Probes run from a fresh empty
directory, oracle wrapped as `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx
FILE )`, stdout / stderr / exit status read as three descriptors. Programs are at
`<scratchpad>/probe-t4/p1..p5.rex`; each ends `raise user c1 return 5` in `raiser`, `h1` raises `c2`,
`h2` raises `c3` (except `p5`, where `h2` returns), and every handler prints its name and `SIGL`.

**Every row of both of the brief's tables reproduces exactly.** Oracle rc 0 and empty stderr
throughout; both Rust engines rc 0 and empty stderr throughout.

| program | oracle | `d0b7504a5`, `tree-walker` and `ir` | `1f4176b47`, `tree-walker` and `ir` |
|---|---|---|---|
| `do zi = 1 to 1 / zr = raiser() / end / say 'after' zr` | `h1 5` `h2 6` `after 5` `h3 7` | agrees | `h1 5` `h2 6` `h3 6` `after 5` |
| `do 1 / zr = raiser() / end / say 'after' zr` | `h1 5` `h2 6` `after 5` `h3 7` | agrees | `h1 5` `h2 6` `h3 6` `after 5` |
| `do while zn < 1 / zn = zn + 1 / zr = raiser() / end / say 'after' zr` | `h1 7` `h2 8` `after 5` `h3 9` | agrees | `h1 7` `h2 8` `h3 8` `after 5` |
| `do forever / zr = raiser() / leave / end / say 'after' zr` | `h1 5` `h2 6` `after 5` `h3 8` | agrees | `h1 5` `h2 6` `h3 6` `after 5` |
| `zr = raiser() / do label zl / say 'body' / end / say 'after' zr` | `h1 3` `h2 4` `body` `after 5` | agrees | `h1 3` `body` `h2 5` `after 5` |

Nothing in the brief's tables failed to reproduce. The one thing the brief's prose got slightly
loose is the phrase "one requeue in the loop body": the handler chain in rows 1-4 leaves **two**
requeues (`h1` raises `c2`, `h2` raises `c3`), which is what makes the third delivery the one that
moves. I avoided the count in the text I wrote.

## Choices: delete, anchor or rewrite

* **`run.rs:5231` -- anchor.** The claim is load-bearing (it is the reason the step may skip its
  boundary without losing the construct's deliveries), so deletion would leave the reader with no
  answer to "then who opens them?". I replaced the single wrong name with the origin both engines
  actually share, `run_loop_with_header`, and split the two cases so the sentence is no longer a
  `Simple`-only rule wearing a `DO`/`LOOP` label.
* **`clause.rs:526` -- anchor, with the scope the brief asked for.** Same reasoning, plus the
  brief's own instruction to bound the sentence rather than delete it. The `Simple` half is stated
  as `Simple`, and the repeating half now says `END` has no clause of its own and names
  `HeaderClause::End` as where its line goes.
* **Plan `:457` -- anchor, one identifier.** `run_loop` -> `run_loop_with_header`. The rest of that
  sentence I verified line by line and left untouched.
* **Plan `:464` -- replace with the measured statement, not delete.** Two reasons deletion was
  wrong here. First, the next bullet opens "`if ... then do` diverged **too**", which loses its
  antecedent if the only bullet asserting a divergence goes. Second, the false bullet is not merely
  uninformative, it points the reader at the *opposite* conclusion about the fix's scope, and a
  reader arriving at this record would carry that away. So the bullet is replaced by the two
  re-measured transcript tables plus the mechanism I could verify in code, and it now separates the
  four repeating shapes (step boundary took the requeue) from `do label zl` (no clause at the `DO`'s
  line at all, so the delivery fell into the body) rather than attributing one cause to both.

The plan's own headline claim is bounded to what I ran -- "Every loop shape **measured** on this
route", with the table naming them -- rather than "every `LoopKind`", because `LoopKind::Over` was
not measured and `LoopKind::With` is the loud path.

## Gates, on the committed tree

* `cargo fmt --all --check` -> exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` -> exit 0, zero warning or error lines.
  The run re-checked `rexx-exec`, the crate I edited (`Checking rexx-exec v0.1.0` in its output).
* `cargo doc --no-deps` -> **exit 0, with 18 pre-existing warning lines** (13 diagnostics plus the
  5 per-crate roll-ups). None of them names `run.rs` or `clause.rs`, and `rexx-exec` contributes
  exactly two, both `redundant explicit link target` at `crates/rexx-exec/src/lib.rs:378` and `:379`,
  which I did not touch. The others are `rexx-extract` (6: `private_intra_doc_links` in `bif.rs` and
  `keyword.rs`), `rexx-bench` lib (3: `Figure::fmt` and `Sitting::per_pass` unresolved,
  `Workload::rendered` private), `rexx-bench` bin (1:
  `the_caveat_matches_the_committed_baseline` unresolved), `rexx-num` (1: `FormatError::sub` private).
  My edits introduce no `[` or `]` anywhere, so no intra-doc link was created, moved or broken.

## Files changed

* `rust/crates/rexx-exec/src/run.rs` -- one block comment in `leave_stepped_clause`.
* `rust/crates/rexx-exec/src/clause.rs` -- one doc comment on `leave_clause_without_boundary`.
* `docs/superpowers/plans/2026-08-14-pre-phase-5-defects.md` -- `:457` one identifier, `:464` bullet
  replaced.

`git diff -U0` over `rust/` with every line starting `//` or `///` filtered out returns nothing, so
the Rust half is comments only.

## Self-review findings, fixed before reporting

1. My first draft of the plan bullet ended "it moved for the same reason the unlabelled plain `DO`
   did" -- an attribution I had not measured (I never ran the unlabelled variant). Removed.
2. The same draft folded all five rows under one mechanism ("the step's own boundary took the
   requeue"). That is false of row 5: `p5`'s chain stops at `h2`, and its divergence is the absence
   of any clause at the `DO`'s own line, which I then confirmed by reading the `Simple` arm at
   `1f4176b47`. Split into two paragraphs with the right cause on each.
3. A parenthetical said the per-pass header clause is "where `h2` runs". True of rows 1-3, **false
   of row 4**: in `do forever / zr = raiser() / leave / end`, `h2` runs at the `leave` clause's
   boundary (`SIGL` 6 is the `leave` line), not at a header re-test. Rewritten to a statement that
   does not depend on which boundary the handler happened to land on. This one was caught after the
   first commit and the commit was amended, so `1a56cdd0e` is superseded by `88b9e4f15`.
4. Dropped a self-referential clause ("the other half of what the bullet this replaces got wrong"),
   which a future reader cannot check against a bullet that is gone.

## Found and NOT fixed

**The same false attribution lives at other sites the brief does not name.** I verified each rather
than guessing, and left them alone because the brief scopes N1 to three sites and a rewrite round
over eight more is where this project reliably manufactures new false statements. All are in
`rust/crates/rexx-exec/src/run.rs`:

* `:614` -- "`run_loop`'s own `Simple` arm never builds one of these at all". The `Simple` arm is in
  `run_loop_with_header`.
* `:628` -- "a stem target takes the loud path in `run_loop`". The loud path is
  `run_loop_with_header`'s `if loop_header_plan(body).is_none()` at `:6177`.
* `:6415` -- "`Simple`, which never repeats and runs through `run_loop`'s own arm directly".
* `:6474` -- "before `run_loop` ever called into this function", of `run_repeating`.
  `run_loop_with_header` calls `run_repeating` (`:6342`).
* `:6730` -- "`run_repeating`/`run_loop`'s own `Simple` arm".
* `:6748` -- "`LoopKind::Simple` (`run_loop`'s own `Simple` arm passes it)".
* `:11089` -- "the new `trace_keyword` call in `run_loop`'s `LoopKind::Count` arm". `run_loop` has
  no `LoopKind` arms; that `trace_keyword` is at `:6091`, in `echo_header_value`.
* `:11606` -- "`(a.)` is already `ExprKind::Stem` by the time `run_loop`'s own `matches!` check sees
  it". That `matches!(target.kind, ExprKind::Stem(_))` is at `:850`, in `loop_header_plan`.

Two more mentions I checked and judged **not** false, so they need nothing:
`run.rs:4933` and `:5139` ("`run_loop`/`run_repeating` resolve an entire multi-pass loop inside this
one call") are true, since `run_loop` does resolve the whole construct through its delegate; and
`clause.rs:447` ("a `DO`/`LOOP`'s control setup (`run_loop`)") is right for the tree-walker, whose
header evaluation is `run_loop`'s `eval_loop_header` call.

**No divergence found.** Every probe agreed with the oracle at `d0b7504a5` on both engines, stdout,
stderr and exit status.

## Concerns

* The plan bullet now carries transcripts measured at a commit (`1f4176b47`) that no longer exists
  in the working tree. They are reproducible from git and the report says how, but a future reader
  cannot re-run them without rebuilding that revision. That is inherent to recording a before/after
  in a plan document; the alternative was deleting the record of scope entirely.
* `cargo doc`'s 18 warning lines are pre-existing and unrelated, but they mean this gate cannot fail
  loudly on a *new* warning without someone diffing the list. I diffed it by inspection (rexx-exec's
  two are both in `lib.rs`); a `-D warnings` on rustdoc would need those 13 diagnostics fixed first.
</content>
</invoke>

---

# Fix round 1: I1, the eight-site sweep, and the minors

Commit `3bea85d86`, on top of `88b9e4f15`. Comments and documents only; `git diff -U0` over `rust/`
with `//`, `///` and `//!` lines filtered out returns nothing.

## I1: `clause.rs`, the `END` re-test

Accepted, and the reviewer is right. The clause carrying `END`'s line is opened by the iteration
attempt that *terminates* the loop, and "on the following pass" tells a reader there is no such
clause on the last iteration -- which mispredicts rows 1 and 2 of the table I had just added.
Verified at the tree: `run_repeating` sets `header_clause = HeaderClause::End` on fall-through, and
the next trip round its `loop` opens `in_clause` at `header_line = end_line` *before* the control
test decides whether to continue. Reworded to "the header re-test that follows the body carries
`END`'s line (`HeaderClause::End`), the re-test that ends the loop included."

## The sweep, per site

`run.rs` unless stated. The reviewer's corrected line numbers were right; I confirmed each by its
quoted text rather than by line.

**Pure renames to `run_loop_with_header` (bound 1 -- only the name changed):**

| site | what it said |
|---|---|
| `:614` | "`run_loop`'s own `Simple` arm never builds one of these at all" |
| `:628` | "a stem target takes the loud path in `run_loop`" |
| `:6417` | "runs through `run_loop`'s own arm directly" |
| `:6476` | "before `run_loop` ever called into this function" |
| `:6732` | "`run_repeating`/`run_loop`'s own `Simple` arm" |
| `:6750` | "`LoopKind::Simple` (`run_loop`'s own `Simple` arm passes it)" |

`:628`'s target verified: the refusal is `run_loop_with_header`'s `if loop_header_plan(body).is_none()`.
`:6476`'s verified: `run_loop_with_header` is what calls `run_repeating`.

**Not renames (bound 2), plus one more of the same shape:**

* **`:731`** -- "(`run_loop`'s own measurement: the oracle traces a bare repeat count under `FOR`...)".
  I read `run_loop`'s doc comment in full: it has the loud path, the one-implementation paragraph and
  the compiled-stream paragraph, and **no** bare-count measurement. The measurement is stated inline
  right there and needs no pointer, so the smallest edit that removes the falsehood is to drop the
  attribution and keep the measurement: "(measured: the oracle traces a bare repeat count under
  `FOR`, ...)".
* **`:11091`** -- "deleting the new `trace_keyword` call in `run_loop`'s `LoopKind::Count` arm makes
  `interp.trace` empty ... with every other assertion in this file untouched". `run_loop` has no
  `LoopKind` arms, and the only `trace_keyword` on the header path is `echo_header_value`'s at
  `:6093`. I did **not** substitute a function name here, because the blast-radius half of the
  sentence rides on the location: deleting `echo_header_value`'s `trace_keyword` would kill every
  `>K>` line in the file, so "every other assertion untouched" would then be false, and I have not
  run that mutation. Replaced the mutation description with the route I could verify by reading:
  `HeaderRole::Count` answers `FOR` in `HeaderRole::keyword` (`:753`), which is what
  `echo_header_value` hands `trace_keyword`. The load-bearing claim -- that this is the test which
  would have caught the omission -- is kept verbatim.
* **`:11608`** -- "`run_loop`'s own `matches!` check". That `matches!(target.kind, ExprKind::Stem(_))`
  is at `:850`, in `loop_header_plan`. Renamed to `loop_header_plan`.
* **`crates/rexx-exec/src/ir/drive/tests.rs:107`, which neither of us listed** -- "The construct is
  resolved by the same `run_loop` either way". Found by grepping the whole crate rather than
  re-reading my own list. It is in the IR test module, describing the promoted path, which never
  enters `run_loop`. Pure rename; the sentence's point ("the same code either way, so both engines
  print the same bytes") survives it exactly.

## Checked and left alone, because they are true

* **`:7762`** -- "a loop written *inside* the fragment ... its own `run_loop` consumes the `Flow`".
  **Confirmed at the tree, not from the doc comment.** `run_fragment` calls `run_bounded` with a
  literal `BodyEngine::TreeWalker`, with no branch above it.
  (**Corrected in round 2: all three line numbers this paragraph originally gave were wrong** --
  `:7794` for the `fn`, `:7858` for the literal. At the committed tree the `fn` is at `run.rs:7796`,
  its single `run_bounded` call at `:7854` and the `BodyEngine::TreeWalker` literal at `:7861`;
  `:7858` is `Some(&fragment.source),`. I read a `sed` range and wrote line numbers off it instead
  of grepping for each one. The judgement was right and the tree is unaffected, but a paragraph
  whose whole job was to justify leaving a site alone pointed at three things that are not there.)
  So a fragment's clauses are always tree-walked, and `step`'s `InstructionKind::Do` arm calls
  `run_loop`. True; untouched.
* **`:2114`** -- "the loud path, `run_loop`'s own doc comment". `run_loop`'s doc does carry the loud
  path (`COUNTER`, `DO WITH` and a stem `OVER`, deciding it in `loop_header_plan`). Agreed with the
  reviewer; untouched.
* **`:4934`, `:5140`** -- "`run_loop`/`run_repeating` resolve an entire multi-pass loop inside this
  one call". True: `run_loop` does resolve the whole construct, through its delegate.
* **`clause.rs:29`** -- "`run_loop` was a third". An account of what a past review round said about
  a set of call sites, not a claim about where clauses are opened today.
* **`clause.rs:447`** -- "a `DO`/`LOOP`'s control setup (`run_loop`)". Right for the tree-walker,
  whose header evaluation is `run_loop`'s `eval_loop_header` call, and that is the engine whose
  clause-line exemption the passage is about.

## `docs/superpowers/plans/phase-4e-anchor.md:378`

**Treated as a dated record of a past state, with a marked correction rather than a rewrite**, which
is what that document already does to itself twice (its header block on which arm produced the
figures, and the "Superseded as Phase 4f's input state" paragraph).

**It was true when written, and I could date that precisely.** `git log -S` on the sentence gives
`b6531a680`, 2026-08-09. `git log -S "fn run_loop_with_header"` gives `08137f3ae`, 2026-08-10
("Flatten a loop header into the compiled stream"). So the shared entry really was
`Interp::run_loop` on the day Task 4a was recorded, and it stopped being so the next day. The
correction says that, names both commits, and says the figures are untouched and not re-measured.

I checked the two endpoints rather than asserting continuity: at `08137f3ae`, `Op::LoopRun` already
called `run_loop_with_header` (`git show 08137f3ae:.../ir/drive.rs`), and it still does. The
correction is worded as those two points, not as "ever since".

## Minors

* **M2.** Citation changed from `instruction.rs:908` (which is `fn create_loop`) to `:960-975`.
* **M3.** `run.rs:5230`'s "Every boundary the construct does owe is opened under..." is now "The
  boundaries the construct does owe are opened elsewhere: a plain `DO`'s header and `END` clauses in
  `run_loop_with_header`'s own `LoopKind::Simple` arm, and a repeating loop's clauses in
  `run_repeating`." No universal, both functions named.
* **M4.** The aside "which is where both engines resolve the construct" is gone with it, so there is
  nothing left to be in tension with `run_loop_with_header`'s own doc at `:6150`. I also dropped
  `run_loop` from that doc's own summary line, which called the function "`run_loop` past its
  header" -- engine-specific, and its next paragraph already says the same thing correctly.

## Self-review: false or unintended statements I caught in my own drafts this round

**Two**, both fixed before the commit.

1. **An over-broad claim in the `phase-4e-anchor` correction.** My first draft said the compiled
   stream "has entered `run_loop_with_header` ... and never calls `Interp::run_loop` at all" *since*
   `08137f3ae` -- a continuity claim over every commit in between, which I had not checked. Narrowed
   to the two endpoints I did verify.
2. **An unintended word drop, not a falsehood.** Rewrapping `:628`'s paragraph turned "`remaining`
   is `FOR`'s own budget" into "`FOR`'s budget", editing a sentence I had no business changing.
   Restored.

Both came out of reading the rendered passage rather than the diff hunk, which is the check that
found the row-4 error last round too.

## Gates, on the committed tree

* `cargo fmt --all --check` -> exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` -> exit 0, zero warning or error lines,
  and the run re-checked `rexx-exec`.
* `cargo doc --no-deps` -> exit 0, **18 warning lines, byte-identical as a set to the pre-sweep run**
  (`diff` of the sorted warning lines is empty; only their interleaving moved, which is parallel
  documentation jobs). None names `run.rs`, `clause.rs` or `ir/drive/tests.rs`.

## Still not fixed, and named rather than swept

`run.rs:6750` carries "every `LoopState` variant `run_repeating` drives is a real, repetitive loop",
a universal over an in-repo enum. It is pre-existing, it is not part of the falsehood I was sent to
fix, and bound 3 governs what I write rather than what I find. Flagging it rather than rewriting a
sentence nobody has measured.

---

# Fix round 2: the complete enumeration, and F2

Commit `6c24c45ab`, on top of `3bea85d86`. Comments only; `git diff -U0` over `rust/` with `//`,
`///` and `//!` lines filtered returns nothing.

## The grep, and every hit verdicted

Command, run from `rust/`: `/bin/grep -rn "run_loop" crates/ --include=*.rs` -> **32 hits**. All 32
are below. Line numbers are at the committed tree `6c24c45ab`.

**I also ran it without `--include=*.rs`, which returns 34.** The two extra are
`crates/rexx-exec/tests/ir_dual_cases/loop-refusals:13` and `.../do-block-handler-indent:20`, both
already saying `run_loop_with_header` and both correct. They are reported because the failure mode
of the last three passes was a search narrower than the family, and `--include=*.rs` is exactly that
kind of narrowing -- it happened to cost nothing here, but only by luck.

### Hits naming `run_loop_with_header` (correct by construction, no action)

`run.rs` `:614`, `:628`, `:2127`, `:5232`, `:6001`, `:6019`, `:6029`, `:6160`, `:6417`, `:6476`,
`:6732`, `:6752`; `ir/drive.rs` `:1323`, `:1337`; `ir/drive/tests.rs` `:108`, `:152`, `:613`;
`ir/mod.rs` `:138`, `:161`; `clause.rs` `:527`. I read each rather than counting them: none makes a
claim about `run_loop`, and none attributes to `run_loop_with_header` something that lives elsewhere.

### Hits naming bare `run_loop`

| # | site | text | verdict |
|---|---|---|---|
| 1 | `lib.rs:185` | "`run_loop`/`run_repeating` each drive their own `run_bounded` calls" | **True, no action by instruction.** Holds through the delegate, same standard as 4 and 5. |
| 2 | `run.rs:2114` | "the loud path, `run_loop`'s own doc comment" | **True.** `run_loop`'s doc does carry the loud-path paragraph (`COUNTER`, `DO WITH`, stem `OVER`, decided in `loop_header_plan`). |
| 3 | `run.rs:2128` | `self.run_loop(` | **Code, not a claim.** The tree-walker's call site. |
| 4 | `run.rs:4933` | "`run_loop`/`run_repeating` resolve an entire multi-pass loop inside this one call" | **True.** `run_loop` does resolve the whole construct, through its delegate. |
| 5 | `run.rs:5139` | same shape | **True.** |
| 6 | `run.rs:6009` | `fn run_loop(` | **Code, not a claim.** The definition. |
| 7 | `run.rs:7764` | "its own `run_loop` consumes the `Flow`" | **True.** `run_fragment` passes a literal `BodyEngine::TreeWalker` (`run.rs:7861`), so a loop in a fragment always tree-walks. |
| 8 | `clause.rs:29` | "round 2 ... said it had 'exactly two callers' -- `run_loop` was a third" | **Out of family.** A numbered account of what four past review rounds each got wrong, past tense throughout; not a claim about today's call graph. Flagged, not edited. |
| 9 | `clause.rs:447` | "a `DO`/`LOOP`'s control setup (`run_loop`)" | **FIXED -- I reversed my round-1 verdict.** See below. |
| 10 | `lib.rs:970` | "`run_loop`'s `DO`/`LOOP` COUNTER/`DO WITH` check, and its stem-target `DO OVER` deviation" | **FIXED.** That check is `run_loop_with_header`'s `if loop_header_plan(body).is_none()` at `run.rs:6178`, raising `Loud::instruction` at `:6179`. Both edge cases go through that one guard. Identical falsehood to `:628`. |
| 11 | `lib.rs:1095` | "because `run_loop` reaches this function for them" | **FIXED.** True of the tree-walker only; `Op::LoopRun` reaches the same raise without entering `run_loop`. |
| 12 | `tests/loud.rs:485` | "`Do`/`Loop`, which `run_loop` does reach here" | **FIXED.** Same shape as 11. |

### On hit 9, where I changed my mind

In round 1 I verdicted `clause.rs:447` true "for the tree-walker, whose header evaluation is
`run_loop`'s `eval_loop_header` call", and the reviewer accepted it. **I now think that was wrong,
and the coordinator's ruling on hits 11 and 12 is what shows it.** The passage documents an
exemption in `enter_clause`'s tripwire. That tripwire is shared code reached from `run_repeating`'s
`in_clause` on *both* engines, so the exemption has to hold on both -- and the compiled engine opens
the same two clauses at the same line, differing only in what drives the header. Naming `run_loop`
describes one engine's driver as if it were the rule's subject, which is precisely the defect
"true transitively on the tree-walker, false of the compiled engine" names. Changed to "the header
evaluation", which is engine-neutral and adds no new proposition. **Flagging it explicitly because
the reviewer passed it once**, so the controller can overrule me.

### Scope: `docs/` and `.superpowers/`

`/bin/grep -rn "run_loop" docs/` excluding `run_loop_with_header` returns hits in four documents;
`.superpowers/` returns more, including two literal `.diff` files.

**I scoped the family to live claims in code comments about which function does what at this tree,
and excluded dated records.** The reasoning, and what it excludes, so this is a decision rather than
an omission:

* `docs/.../2026-08-15-task-6-review-and-run-split.md:290-296` -- my own task's plan text, which
  *quotes* the false comments as the finding. Correct as written.
* `docs/.../2026-08-09-phase-4e-ir.md:696,702,716` -- the Phase 4e plan, written before the split
  and planning the extraction ("the shared step is extracted from `run_loop`"). Closed phase.
* `docs/.../2026-08-08-phase-4e-ir-design.md:121,476` -- the design spec. `:476`'s "`run_loop`
  dispatches five `LoopKind` variants" was true when written and is now false; **it is the one
  exclusion I am least sure of**, since it is the same shape as `phase-4e-anchor.md:378`, which I
  did correct.
* `.superpowers/sdd/2026-07-30-phase-4a-executor/*` -- session records, task reports and two review
  `.diff` files. Frozen artifacts; correcting a recorded diff would falsify it.

**Why `phase-4e-anchor.md:378` was treated differently in round 1**: it was named by the
coordinator, and that document is still cited as a live anchor for figures, so a reader arrives at
it expecting current-tree statements. If the controller wants the design spec corrected too, it is
one marked-correction block of the same shape and I will take it.

## F2: `run.rs:11094`'s exclusivity claim

Took the bounding option, as advised. "This is the one test that would have caught the omission,
since the report's own verification claimed..." becomes "It is asserted here because the report's
own verification claimed...". The exclusivity claim is gone; what remains is the reason the
assertion exists, which is what the `since` clause actually supported. I did **not** run the
mutation: witnessing "the one test" needs the whole file's suite run with the assertion removed,
which is the expensive half, and the cheap half would not have supported the claim anyway.

## Gates, on the committed tree

* `cargo fmt --all --check` -> exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` -> exit 0, zero warning or error lines,
  `rexx-exec` re-checked.
* `cargo doc --no-deps` -> exit 0, 18 warning lines, `diff` of the sorted warning set against the
  pre-sweep baseline is empty. None names any file this round touched.

## Self-review: what I caught in my own work this round

**Two, and the first is the more serious.**

1. **A gate check of mine was vacuous and I nearly reported it as passing.** I ran the
   comments-only check as `cd rust/ && git diff -U0 -- rust/`, which matches `rust/rust/` and so
   returns nothing -- indistinguishable from "no non-comment lines changed". I caught it only
   because a later `git diff -- rust/` from the same directory printed an empty diff for a tree I
   knew had changes. Re-run from the repo root, where it genuinely reports comments-only. This is
   the same failure as a misspelled flag: a command that does not run reads exactly like one that
   passed.
2. **A reversed verdict, not a false statement**: hit 9 above, which I had called true in round 1.

**Not self-caught, and worth recording as such**: the three wrong line numbers in my round-1
justification for leaving `:7762` alone. The reviewer found those. I have corrected that paragraph
in place above, with a note saying what it said and why it was wrong -- I wrote line numbers off a
`sed` range rather than grepping for each. That is the third time on this branch that my own probe
bookkeeping, not the subject matter, produced the wrong number.

## Still not fixed, and named rather than swept

* `run.rs:6753`'s "every `LoopState` variant `run_repeating` drives" -- left alone by instruction.
* `clause.rs:29`'s historical list, hit 8 above.
* `docs/.../2026-08-08-phase-4e-ir-design.md:476`, the exclusion I am least sure of.

---

# Fix round 3: the 4e design spec's loop-promotion risk

Commit `9c465989a`, on top of `6c24c45ab`. One file, two lines added, no code touched.

## It is the same falsehood, and the correction is warranted

I read it first to check whether it might be true in a sense the anchor doc's was not. It is not.
`docs/superpowers/specs/2026-08-08-phase-4e-ir-design.md:476` says "`run_loop` dispatches five
`LoopKind` variants -- it refuses `COUNTER` and `LoopKind::With` before its match". At this tree
`run_loop` has no `match &body.kind` and no refusal at all: it evaluates the header plan and calls
`run_loop_with_header`, which holds both.

**One way it is weaker than the anchor doc's claim, and it does not rescue it.** The anchor said
`run_loop` was the shared entry *under both engines*; this bullet never mentions engines, so it is
not making the engine-sharing claim. But it still attributes to a named function two things that
function does not do, which is the whole family, and the coordinator's reason for correcting it
holds independently: a Phase 5 reader looking up where loop dispatch lives lands in a function with
no dispatch in it.

## Archaeology, and it was true when written

Every state below was read at that commit with `git show`, and every commit was dated with
`git log -S`. I assert no continuity across the gaps.

| commit | date | where the refusal is | where the `LoopKind` match is |
|---|---|---|---|
| `47bb1b832` (added this bullet) | 2026-08-08 | `run_loop`, as `body.counter.is_some() \|\| matches!(body.kind, LoopKind::With { .. })`, ahead of the match | `run_loop` |
| `08137f3ae` ("Flatten a loop header into the compiled stream") | 2026-08-10 | still `run_loop`, now as `loop_header_plan(body)` answering `None` | `run_loop_with_header` |
| `b6d548562` ("Make a refused loop refuse on the compiled stream too") | 2026-08-10 | `run_loop_with_header` | `run_loop_with_header` |
| this tree | 2026-08-15 | `run_loop_with_header` (`run.rs:6178`) | `run_loop_with_header` (`run.rs:6187`) |

`git log -S "Loop promotion is the largest task and the highest risk"` and
`git log -S 'dispatches five \`LoopKind\` variants'` both return exactly `47bb1b832`, so the bullet
and the `run_loop` clause landed together. At that commit `run_loop_with_header` did not exist --
`git show 47bb1b832:.../run.rs | /bin/grep -n "fn run_loop"` returns `run_loop` and `run_repeating`
and nothing else. **So the bullet was accurate on the day it was written**, and what moved the two
things out of `run_loop` was the loop promotion the bullet is itself assessing as a risk.

**The refusal and the dispatch moved separately, and I would have got this wrong by assuming.** My
first instinct was that `08137f3ae` moved both, since it is the commit that introduced
`run_loop_with_header`. It did not: at `08137f3ae` the match had moved and the refusal had not.
That is why the correction names two commits rather than one, and why I checked the state at each
rather than inferring from the commit that introduced the function.

## The correction itself

Same shape as `phase-4e-anchor.md`: the original bullet is left byte-for-byte intact and the
correction is a marked blockquote nested under it. It says that both attributions now live in
`run_loop_with_header`, that the bullet was accurate when written, that the promotion it assesses is
what moved them, gives the four dated states, names `run_loop_with_header` as the function both
engines enter with each engine's route, and says the risk assessment itself is untouched by the
split. It repeats no count -- the original's "five" stays in the original and is not restated.

## Also in this file, checked and left alone

`:121`, the architecture mermaid node `tw[Tree-walker: step_in_temps_frame, run_bounded, run_loop]`.
**True and out of family.** It enumerates tree-walker components, and `run_loop` is still exactly
that -- `step`'s `InstructionKind::Do` arm calls it and only the tree-walker reaches it. The node
makes no claim about dispatch or about engine-sharing. No action, and no correction block, because
an unnecessary correction is its own defect.

## Gates, on the committed tree

* `cargo fmt --all --check` -> exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` -> exit 0.
* `cargo doc --no-deps` -> exit 0, 18 warning lines, sorted `diff` against the pre-sweep baseline
  empty.

All three are unchanged by construction this round -- `git diff --stat` shows one markdown file --
and are run because the gate is the gate, not because a docs edit could move them.

## Self-review count this round

**One.** My first draft of the correction said the split commit moved both the refusal and the
dispatch. Checking the state at `08137f3ae` rather than trusting its subject line showed the refusal
was still in `run_loop` there, and dating the refusal's move needed a second `git log -S`. Caught
before the edit was written, so it never reached the file -- but it is the same shape as the errors
the earlier rounds shipped: an inference from a true premise, one premise short.

## Standing, unchanged from round 2

* `run.rs:6753`'s "every `LoopState` variant `run_repeating` drives" -- left alone by instruction.
* `clause.rs:29`'s historical account of four past review rounds.
* `docs/.../2026-08-09-phase-4e-ir.md` and `docs/.../2026-08-15-task-6-review-and-run-split.md`,
  excluded on the reasoning in round 2 and confirmed as standing.
