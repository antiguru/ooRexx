# Task 6 review findings, the run.rs split, and the pre-Phase-5 whole-branch review

## Context

`docs/superpowers/plans/2026-08-14-pre-phase-5-defects.md` ran as an SDD plan and completed its five
tasks and its whole-branch review. Task 6 was added afterwards and run inline by the controller, so
no task reviewer saw it until a review was dispatched over `1f4176b47..01f8010b7`. That review
returned **request changes**; its full text is
`.superpowers/sdd/2026-08-14-pre-phase-5-defects/task-6-review.md`. None of its findings have been
addressed.

One commit landed after that review, `56d9d1c86` ("Drain a clause boundary instead of answering one
condition and losing the rest"). It closes the first shape the review recorded under N2 and has had
no review of any kind. The final whole-branch review of this plan therefore spans
`e74780054..HEAD`, so that both Task 6's commits and the drain commit are reviewed.

Tasks 1 through 5 here close the review's findings. Task 6 takes the first rank of the `run.rs`
split, whose scouting report is `.superpowers/sdd/2026-08-15-task-6-review-and-run-split/file-size-scout.md`.

**Ordering is load-bearing.** Task 1 changes behaviour, and three of the prose findings record
transcripts that Task 1's fix may move again. Every prose task re-measures against the live oracle
at the tree it finds, not against the transcripts quoted in this plan, which were measured at
`56d9d1c86`.

## Global Constraints

* **Wrap every oracle run** as
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.
* **Read stdout, stderr and exit status as three separate descriptors.** Never `2>&1`.
* **Run every probe from a fresh empty directory you `mkdir` yourself.** The scratchpad root is on
  the oracle's external-routine search path and a probe calling an unresolved name will execute a
  stale `.rex` there. Use absolute paths for every redirect.
* **`rust/CLAUDE.md` binds this work.** Read it before your first probe. It carries the oracle-crashing
  programs, the gate commands, and the reasons behind both.
* **Engine selection is `REXX_ENGINE=tree-walker` or `REXX_ENGINE=ir`.** Every behavioural claim is
  made on both engines or it is not made.
* **Do not fix a divergence this plan does not name.** Record it in the report and in the
  found-and-not-fixed list, with its program and its three descriptors on both engines, and move on.
  The one exception is a site the fix mechanically forces.
* **Every behavioural fix needs a witness that fails without it.** A test that cannot fail is a
  defect. Prove each witness by mutation: revert the fix, run the suite with `--no-fail-fast` under
  `memcap 8G`, and report *every* test that caught it, not the first. `cargo test <name>` exits 0
  when it matches nothing, so assert a non-zero run count.
* **Gates, run unpiped from `rust/`, each exit status read on its own:** `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --release --workspace`, and
  the corpus gate under `REXX_CORPUS_GATE=1`. Without that env `corpus_differential` runs in REPORT
  mode and exits 0 regardless, so a plain `--release --workspace` does not gate the corpus.
* **Prefer deleting a false claim to rewriting it.** Measured on the previous plan: every new false
  statement a fix round introduced landed in a finding fixed by rewriting, none in the four fixed by
  deleting. Rewrite only where deletion would cost a reader something real, and when you rewrite,
  name the subject of every clause explicitly.
* **No em-dashes in code comments.** Comments state the contract at the top and the reasoning at the
  decision point.
* **Line numbers in this plan were taken at `56d9d1c86` and will move.** Quote and grep for the
  sentence, do not trust the number.
* **Commit first, then read the hash back with `git log`**, then quote it. Never `git add -A`.

---

### Task 1: close the `INTERPRET` regression `e74780054` caused

`e74780054` gave a plain `DO` a header clause and an `END` clause. Inside an `INTERPRET` fragment
those are boundaries the oracle does not have, and they take a delivery that the oracle leaves for
the `INTERPRET` clause's own boundary. This is review finding **C1**, and the disposition is
**close it**, decided by Moritz on 2026-08-15.

#### The program and what it does now

```rexx
call on user c1 name h1
call on user c2 name h2
zr = raiser()
interpret 'do; say ''body''; end'
say 'after' zr
exit 0
raiser:
raise user c1 return 5
h1:
say 'h1' sigl
raise user c2 return 1
h2:
say 'h2' sigl
return
```

Measured by the controller at `56d9d1c86`, `rc 0` everywhere, stderr empty:

| | stdout |
|---|---|
| oracle | `h1 3` / `body` / `h2 4` / `after 5` |
| `1f4176b47`, both engines | `h1 3` / `body` / `h2 4` / `after 5` (matched) |
| `56d9d1c86`, both engines | `h1 3` / `h2 4` / `body` / `after 5` (diverges) |

The rule the oracle follows is already written down in `crates/rexx-exec/src/clause.rs`'s module
doc, with the measurement behind it: an `INTERPRET` fragment runs in an activation whose condition
queue is separate, so a condition pending when the `INTERPRET` clause runs is offered no boundary
*inside* the fragment.

#### Steps

1. **Reproduce** the program above on both engines and confirm the divergence still stands at the
   tree you find. If it does not, stop and report that instead of fixing nothing.

2. **Fix it.** CORRECTED 2026-08-15, after Task 1 built the mechanism this step first named and
   measured it wrong. This step read "give the new header and `END` clauses no boundary when a
   fragment's `clause_line_override` is in force". That suppression is too broad: it regresses
   `interpret 'zq = raiser(); do; say ''body''; end'`, where the oracle **does** deliver at the
   `DO` header inside the fragment. Measured by the controller at `11638b91e`, oracle and both
   engines, `rc 0`: `h1 3` / `h2 3` / `body` / `after 5`. The distinction is not whether the
   boundary is inside a fragment, it is **which fragment queued the condition**: a condition
   pending when the `INTERPRET` clause ran waits for that clause's own boundary, and a condition
   queued inside the fragment is delivered at the fragment's own boundaries. The suppression must
   reach both engines: the compiled engine
   never calls `run_loop`, `Op::LoopRun` enters `run_loop_with_header` directly
   (`crates/rexx-exec/src/ir/drive.rs:1337`), which is why `e74780054`'s own step suppression had to
   go in `leave_stepped_clause`.

3. **Measure the suppression against the empty-fragment shape before you keep it.** Construct
   `interpret 'do; end'` under a double requeue, measure it on the oracle and on both engines
   before and after your change, and report all three. A suppression that breaks it is the wrong
   suppression.

   CORRECTED 2026-08-15: this step was written from the reviewer's statement that the shape
   "diverges *identically* before and after `e74780054`". Task 1 measured it and it does not
   diverge at all: the oracle and both engines agree, before and after. The step stands as a
   control that must keep agreeing; its stated premise does not.

4. **Measure the two siblings** the reviewer identified as pre-existing divergences of the same
   family, unchanged by `e74780054`:
   * `interpret 'if 1 = 1 then say ''body'''`
   * `interpret 'do zi = 1 to 1; say ''body''; end'`
   in the same surrounding program. Report, for each, whether your suppression closes it. If it
   does, that is in scope: it is the same mechanism, not another divergence. If it does not, record
   it as found-and-not-fixed with its transcripts and do not chase it.

5. **Witness it.** At least one `ir_dual_cases` stanza that reddens when the suppression is removed.
   Mutation-test with `--no-fail-fast` under `memcap 8G` and report every catcher. Note in the report
   which harness actually gates it: under a plain `cargo test --release --workspace`,
   `corpus_differential` is in REPORT mode.

6. **Sweep.** Raw before/after over every `.rex` under `corpus/` and `bench-programs/`, both engines,
   all three descriptors. Report exactly what moved. Nothing but your new case should.

7. **Correct the sweep sentence in the previous plan.** `docs/superpowers/plans/2026-08-14-pre-phase-5-defects.md`
   carries a Task 6 OUTCOME claim that the before/after sweep covers `corpus/` and `bench-programs/`
   and moved only one program. That sweep cannot see an `INTERPRET` shape, and this divergence is
   one the task *created* rather than encountered. Correct the sentence so it says what the sweep
   covers and what it therefore cannot witness. Do not restate a number you have not measured.

8. **Gates** as in Global Constraints, plus the corpus gate under `REXX_CORPUS_GATE=1`. Run clippy
   from a clean target directory at least once and say so.

---

### Task 2: witness the two `settle_block_indent` arguments

Review finding **I4**. `e74780054` added two `settle_block_indent` calls that are part of the fix,
are observable against the oracle, and are pinned by nothing. At `56d9d1c86` they are
`crates/rexx-exec/src/run.rs:6175` (`it.settle_block_indent(true, do_indent)`) and
`crates/rexx-exec/src/run.rs:6244` (`it.settle_block_indent(false, do_indent)`). Task 1 will have
moved those lines; find them by the surrounding `Simple` arm, not by number.

Both arguments are **correct**. The reviewer verified three `trace r` programs (a plain `DO` with a
body, an empty `DO` under a double requeue, nested plain `DO`s) match the oracle byte for byte on
all three descriptors and both engines, and that the pre-fix binary got the indent wrong on two of
them. What is missing is a test that fails when either argument is flipped.

Measured by the reviewer, each flip builds clean and leaves `corpus`, `trace_oracle`, `trace_indent`
and all of `ir_dual` green:

* first call, `true` becomes `false`: a handler delivered at a plain `DO`'s header clause loses two
  columns in its activation, `15 *-*     h2:` becomes `15 *-*   h2:`.
* second call, `false` becomes `true`: the handler delivered at `END`'s clause gains two columns.

#### Steps

1. **Re-measure both flips at the tree you find** (after Task 1) and report the transcripts. Do not
   assume the two columns above survived Task 1's change.

2. **Add a witness per argument.** Two distinct witnesses, one per call, each of which reddens when
   its own argument is flipped and stays green when the other is. The corpus program
   `corpus/lang/do_clause_boundaries.rex` deliberately carries no `TRACE`, which is right for its
   own subject, so the witness belongs in an `ir_dual_cases` row or a `trace_oracle` case rather than
   in that corpus program.

3. **Prove the selectivity by mutation**, not by assertion. Flip each argument in turn, run with
   `--no-fail-fast` under `memcap 8G`, and report every test that caught each flip. A single run
   that stops at the first failing assert does not measure selectivity.

4. **Gates** as in Global Constraints.

---

### Task 3: the two false passages in `loop-header-boundaries`

Review findings **I1** and **I2**, both in
`rust/crates/rexx-exec/tests/ir_dual_cases/loop-header-boundaries`. Both are transcripts that were
true when written and were falsified by `e74780054`. Re-measure everything below against the live
oracle at the tree you find before you write a word.

#### I1: the zero-pass bullet, and the re-measurement claim under it

At `56d9d1c86` this is around `:44-53` and `:62-67`. The bullet records `do i = 1 to raiser()` with
`raiser` returning 0 and a requeueing handler: "the oracle prints `after` and then `G ran 6` ... and
this crate prints `G ran 3` and then `after`". The reviewer reconstructed that program from the
bullet's own numbers (`do` on line 3, `g:` reachable, `say 'after'` on line 6) and measured:

```
oracle                     after / G ran 6
1f4176b47, both engines    G ran 3 / after
01f8010b7, both engines    after / G ran 6      <- matches the oracle
```

So the divergence the bullet records was closed by `e74780054`. The sentence below it, "Re-measured,
and both engines still print what that bullet records", is false. The section framing above
("LOOP-HEADER BOUNDARIES THAT DIVERGE FROM THE ORACLE ARE NOT ROWS HERE ... Each was measured on both
engines, and the engines agree on each") no longer holds for its first bullet.

**Fix by deleting the bullet and its paragraph and promoting the program to a row**, not by
rewriting the claim in place.

#### I2: the "runs the other way" paragraph

At `56d9d1c86` this is around `:89-101`:

```
#   oracle       after unset / G ran 7
#   this crate   G ran 6 / after set
```

Both lines are wrong. The oracle half was corrected in the previous plan itself ("CORRECTED: this
line read `after unset` when it was written, and the oracle prints `after set`") and the case file,
the other place the same transcript lives, kept the uncorrected wording. The crate half is closed:
the `DO UNTIL` stanza roughly fifty lines below in the same file records `H ran 6` / `after set` /
`G ran 7`, and the reviewer confirmed the corpus program's F block prints the same against the live
oracle. The surrounding sentence, "Every other shape recorded in this file has this crate delivering
*later* than the oracle. This one delivers *earlier*, on both engines", is false about a shape the
file now pins as agreeing two hundred lines away.

#### Steps

1. Re-measure both programs against the live oracle on both engines, all three descriptors, at the
   tree you find. Report the transcripts.
2. Delete I1's bullet and paragraph; promote its program to a row.
3. Resolve I2 by deletion where deletion loses nothing, and by a rewrite that names its subject
   where it does not. Say in the report which you chose for each half and why.
4. Re-read the section framing at the top of that block after your edits and confirm every sentence
   in it is true of what remains. Report the sentences you checked.
5. `cargo test --release --workspace` plus the corpus gate under `REXX_CORPUS_GATE=1`, and fmt and
   clippy. A case-file edit changes what the harness compares, so the suite is evidence here.

---

### Task 4: the false attribution in the plan, and three comments naming the wrong function

Review findings **I3** and **N1**. Both are claims about which code does what. Neither is a
behavioural change. Verify each against the tree before rewriting it.

#### I3: "repeating loops with a real header were never wrong on this route"

`docs/superpowers/plans/2026-08-14-pre-phase-5-defects.md`, in Task 6's record, around `:464`. The
reviewer measured at `1f4176b47` against the oracle, both engines, the same `raise ... return` route
with one requeue in the loop body:

| program | oracle | `1f4176b47` |
|---|---|---|
| `do zi = 1 to 1 / zr = raiser() / end / say 'after' zr` | `h1 5` `h2 6` `after 5` `h3 7` | `h1 5` `h2 6` `h3 6` `after 5` |
| `do 1 / zr = raiser() / end / ...` | same | same divergence |
| `do while zn < 1 / ... / zr = raiser() / end / ...` | `h1 7` `h2 8` `after 5` `h3 9` | `h1 7` `h2 8` `h3 8` `after 5` |
| `do forever / zr = raiser() / leave / end / ...` | `h1 5` `h2 6` `after 5` `h3 8` | `h1 5` `h2 6` `h3 6` `after 5` |

All four agree at `01f8010b7`. So three of the four shapes the bullet names by name
(`do zi = 1 to 2`, `do while`, `do 2`) *were* wrong on this route, and the stated reason, that
`run_repeating` "drains the queue there", is the wrong mechanism: `run_repeating` drains the queue
the body left at the *next* header clause, and the requeue the handler left behind was still being
taken by the step boundary. The third change is what closed them.

The fourth name is wrong differently. `do label zl` with no header is `LoopKind::Simple`
(`crates/rexx-parse/src/instruction.rs:960-975`) and never reaches `run_repeating` at all. Measured:

```
oracle                     h1 3 / h2 4 / body / after 5
1f4176b47, both engines    h1 3 / body / h2 5 / after 5
01f8010b7, both engines    h1 3 / h2 4 / body / after 5
```

Delete the bullet, or replace it with the measured statement: every `LoopKind` was wrong on the
trailing delivery, and the step boundary was the single cause.

#### N1: three comments name `run_loop` as the function that opens the header and `END` clauses

At `56d9d1c86`: `crates/rexx-exec/src/run.rs:5184` ("is opened by `run_loop`: the header's, once
before the body runs"), `crates/rexx-exec/src/clause.rs:520` ("both of which `run_loop` opens as
clauses in their own right"), and the same claim in the previous plan around `:457`.

`run_loop` (`run.rs:5961`) evaluates the header plan and delegates. The `Simple` arm that opens both
clauses is in `run_loop_with_header`; a repeating loop's are in `run_repeating`. The compiled engine
never calls `run_loop` at all: `Op::LoopRun` enters `run_loop_with_header` directly
(`crates/rexx-exec/src/ir/drive.rs:1337`). A reader who follows the pointer lands in a function with
no `in_clause` in it.

While in `clause.rs`: the sentence around `:519-524` generalises to `DO`/`LOOP`, and for a repeating
loop `END` has no clause of its own. The next header re-test carries `END`'s line
(`HeaderClause::End`). The sentence is true of `Simple`, which is what the paragraph is about, so
bound it rather than deleting it.

#### Steps

1. Confirm each claim above against the tree before acting, including that `Op::LoopRun` enters
   `run_loop_with_header` directly and that `run_loop` contains no `in_clause`. Report what you
   checked and what you found, including anything that has moved since these line numbers were taken.
2. Fix I3 by deletion or by the measured replacement. Fix the three N1 sites.
3. `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
   `cargo doc --no-deps` so that a moved intra-doc link is caught. `cargo test`'s doc-test pass
   compiles doc code blocks and does not check links, and `rustdoc::broken_intra_doc_links` is
   warn-by-default and denied nowhere in this workspace.

---

### Task 5: the ANSI spec's premise, the `clause.rs` comment the drain falsified, and the shapes to record

Review finding **N2**. Its first half, two conditions pending at one boundary losing one, was closed
by `56d9d1c86`, which replaced the single `Option` with a `VecDeque` drained at the boundary. What
is left is the items below, none of which changes behaviour.

#### 1. The ANSI spec records the wrong side of the disagreement

`docs/superpowers/specs/2026-08-15-ansi-condition-delivery.md:9-10` lists as a known
ANSI-versus-ooRexx disagreement: "8.2.4 drains a boundary where **ooRexx and this crate** deliver one
condition and do not re-check". Both halves are now false. The reviewer measured that ooRexx drains:

```rexx
call on user c1 name h1
call on user c2 name h2
call on user c3 name h3
do zi = 1 to 2
  zr = raiser()
end
say 'after' zr
```
(with `h1`/`h2` each `raise ... return`, `h3` plain)

```
oracle                     h1 5 / h2 6 / h3 5 / h1 5 / h2 6 / after 5 / h3 7
01f8010b7, both engines    h1 5 / h2 6 /         h1 5 / h2 6 / after 5 / h3 7
```

On the second pass the oracle's clause at line 5 delivers `h3`, left over from the previous pass's
`END`, and then `h1`, queued by that same clause, both reporting `SIGL 5`, before `END`'s boundary
takes the next. Only what a handler queues *during* delivery is deferred. And this crate drains too
since `56d9d1c86`. So this was never a licensed deviation from ANSI, it was a divergence from
ooRexx, and Phase 7 would design against a false premise.

Re-measure the program above against the live oracle and against both engines at the tree you find,
then correct the spec. If the corrected reading means ANSI 8.2.4 and ooRexx now *agree* on this
point, say so plainly and remove it from the disagreement list rather than restating it.

#### 2. `clause.rs`'s "delivers at most one and does not re-check"

Around `crates/rexx-exec/src/clause.rs:455-459`: "That boundary had already taken one -- this
function delivers at most one and does not re-check -- so the wait is the design rather than a
missing call, and the oracle waits too." The clause in the middle is false since `56d9d1c86`. The
surrounding point may still be sound for a trap queued *during* a delivery, which is what it was
measured on. Establish which part survives, correct the false clause, and leave the rest bounded to
what was measured. Check the whole comment block, not only that sentence.

#### 3. The `ITERATE` shape, to record and not to fix

Pre-existing, byte-identical before and after `e74780054`, both engines. With `do while zn < 2` on
line 5, `zn = zn + 1` on 6, `zr = raiser()` on 7, `iterate` on 8, `end` on 9 and `say 'after' zr` on
10: the oracle defers the second requeue past the re-test and delivers it at the *next pass's* first
body clause (`h3 6`), then leaves the last one for the `say` (`h3 10`); this crate delivers both at
the `ITERATE`'s own line (`h3 8`, twice) and prints the last one before `after`.

Reproduce it at the tree you find, on both engines and all three descriptors, and record it where
this project records found-and-not-fixed divergences. **Do not fix it.** If `56d9d1c86`'s drain
changed it, that is the news: report the new transcripts rather than the ones above.

#### 4. The bare `OPTIONS` instruction, to record and not to fix

ADDED 2026-08-15 by the controller, after the three items above were written, and written into the
plan rather than only into the dispatch so that a regenerated brief carries it. **Decided by Moritz
on 2026-08-15 not to be fixed.**

`docs/superpowers/plans/phase-4-exclusions.txt` records the `::OPTIONS` **directive** and its reason,
which is the directive's own. The bare `OPTIONS` **instruction** is a different case that no row
covered, and refusing it rejects a program the oracle runs:

```
program: `options 'nothing'` then `say 'ran'`
oracle          rc 0, stdout "ran", stderr empty
both engines    rc 120, stdout empty, stderr "rexx-exec: OPTIONS is not implemented (Phase 5)"
```

Re-measure at the tree you find and record it beside the `ITERATE` shape. **Do not implement
`OPTIONS`.** If the re-measurement disagrees with the transcript above, report that rather than
reconciling it.

#### Steps

1. Re-measure all four items at the tree you find. Report every transcript.
2. Correct the spec, correct the `clause.rs` comment, record the `ITERATE` shape and the `OPTIONS`
   over-refusal.
3. Gates as in Global Constraints.

---

### Task 6: move `run.rs`'s test module into `run/tests.rs`

`crates/rexx-exec/src/run.rs` is 17,381 lines, 5.0x the next largest file in the tree. Those figures
and the analysis behind them are in
`.superpowers/sdd/2026-08-15-task-6-review-and-run-split/file-size-scout.md`, which you should read
first. **Verify its numbers before relying on them; they are a scout's, not measured by the
controller.** By production code the ratio is 1.8x, so this task takes only the part that is clearly
worth taking: the test module, roughly 7,700 lines and 44% of the file, moved out for one line of
code.

The scout established that no visibility widens: the test module calls zero private `impl Interp`
methods. `crates/rexx-exec/src/ir/drive.rs` already does exactly this move, so copy its shape rather
than inventing one.

**This task changes no test content and no production code.** A diff that alters an assertion, a
test name, or a line of `impl Interp` has exceeded its scope.

#### Steps

1. Read the scout report. Verify independently: the line count of `run.rs`, the extent of the test
   module, and the claim that it calls no private `impl Interp` method. Report each number you
   confirmed and each you found different.
2. Record the test count before the move: `cargo test --release --workspace` and the per-binary
   `N passed` figures, so the after-count can be compared to something real.
3. Move the module, following `ir/drive.rs`'s shape. If any item does need its visibility widened,
   stop and report rather than widening it, since the scout's central claim would then be wrong.
4. Verify: the test count is unchanged binary by binary, not just in total; `cargo fmt --all --check`;
   `cargo clippy --workspace --all-targets -- -D warnings` from a clean target directory;
   `cargo test --release --workspace`; the corpus gate under `REXX_CORPUS_GATE=1`; and
   `cargo doc --no-deps` for intra-doc links broken by the move, which `cargo test`'s doc-test pass
   does not check.
5. Report the resulting line counts of `run.rs` and `run/tests.rs`.

---

## The whole-branch review

Not a task. After Task 6, the final review runs over `e74780054..HEAD` on the most capable model, so
that it covers Task 6 of the previous plan, the unreviewed drain commit `56d9d1c86`, and everything
this plan lands. Moritz asked for this explicitly as the third item of the session.

---

### Task 7: rename `{name}/mod.rs` to `name.rs`, where that is what it means

Requested by Moritz on 2026-08-15. The tree has exactly four `mod.rs` files, and **only two of them
are in scope.** Confirm the set yourself with `find crates -name mod.rs` before starting.

#### In scope

* `crates/rexx-exec/src/builtin/mod.rs` becomes `crates/rexx-exec/src/builtin.rs`
* `crates/rexx-exec/src/ir/mod.rs` becomes `crates/rexx-exec/src/ir.rs`

Both keep their sibling directory. `name.rs` beside `name/` is the 2018 convention and is what
`run.rs` beside `run/tests.rs` will already look like once Task 6 lands.

#### Out of scope, and this is the point of the task rather than an omission

* `crates/rexx-exec/tests/support/mod.rs`
* `crates/rexx-parse/tests/gate_walk/mod.rs`

**These must NOT be renamed.** Measured by the controller: `mod support;` is declared by seven test
binaries (`builtin_status.rs`, `corpus.rs`, `input_oracle.rs`, `parse_version_oracle.rs`,
`state_builtin_oracle.rs`, `trace_indent.rs`, `trace_oracle.rs`) and `mod gate_walk;` by
`rexx-parse/tests/tiling.rs` and `rexx-parse/tests/variants.rs`.

CORRECTED 2026-08-15, and the error was the controller's. This paragraph first named `tiling.rs`
alone and the steps below said "eight declaring sites". There are nine: `variants.rs:28` declares
`mod gate_walk;` too. The controller's probe piped its `grep` through `head -12`, and the cut fell
exactly one line above `variants.rs`. A truncated search reads exactly like a complete one, which is
the third time this plan has met that shape.

Renaming them to `tests/support.rs` and `tests/gate_walk.rs` would
still resolve as modules, **and** cargo would additionally auto-discover each as an integration-test
target, compiling the helpers standalone as a test binary that exists for no reason. The `mod.rs`
form under `tests/` is the idiom that prevents exactly that.

Record that reasoning where a future reader of those two files will find it, so the next person
applying this convention does not undo it.

#### Steps

1. Confirm the four-file set and the declaring sites above. Report anything that has moved, and do
   not trust the list's completeness: run the search yourself, unpiped.
2. Rename the two `src/` files with `git mv`, so the history follows.
3. Build. Nothing else should need editing: `mod builtin;` and `mod ir;` resolve to either spelling.
   If any other file needs a change, stop and report what and why before making it.
4. Leave a note at the two `tests/` files saying why they keep `mod.rs`.
5. Gates: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --release --workspace`, the corpus gate under `REXX_CORPUS_GATE=1`, and
   `cargo doc --no-deps` read for warnings, since a module path changing is exactly what breaks an
   intra-doc link.
6. Report the test count before and after, binary by binary. A rename that silently drops a test
   target is the failure this task must not produce.

---

### Task 8: adopt timely-dataflow's clippy lint set

Requested by Moritz on 2026-08-15. Source: `https://github.com/TimelyDataflow/timely-dataflow`,
`Cargo.toml`, its `[workspace.lints.clippy]` section. **Re-fetch it and use what you find**, rather
than trusting the transcription below, which the controller took on 2026-08-15.

#### What the controller already measured, so you do not rediscover it

CORRECTED 2026-08-15, twice, and both errors were the controller's. This paragraph first said the
pass covered "42 of timely's warn-level lints" and gave its figures as violation counts.

* **The set is 54 active warns, not 42.** The controller's reading came from a summarised fetch and
  was short by twelve. Two independent re-fetches, one by the implementer and one by the reviewer,
  both parsed the upstream table programmatically and agree: 60 entries, 5 `allow` plus 54 active
  `warn` plus `as_conversions` commented out, and zero set difference against what this workspace
  now carries. The twelve the controller never tested contribute no violations.
* **The figures below are WARNING counts, not sites.** Under `--all-targets`, code compiled into
  both a lib and a lib-test target is counted twice. Distinct primary spans are `as_conversions`
  **299**, `shadow_unrelated` **314**, `needless_pass_by_ref_mut` **10**. The controller re-verified
  this by counting distinct `(file, line, column)` primary spans. Moritz took the allow-versus-fix
  decision on the inflated numbers; it stands on the corrected ones, since 299 casts each needing a
  truncation-and-sign judgement and 314 renames across files under differential work are both still
  out of this plan's scope.

One `cargo clippy --workspace --all-targets` pass over this tree with 42 of timely's warn-level
lints enabled, counted from `--message-format=json` by lint code:

| lint | violations |
|---|---|
| `clippy::as_conversions` | 574 |
| `clippy::shadow_unrelated` | 380 |
| `clippy::needless_pass_by_ref_mut` | 20 |
| every other lint tested | 0 |

Re-measure before you act. If your numbers differ from these, that is a finding: report it.

#### Decisions already taken, by Moritz on 2026-08-15

* **`as_conversions` is `allow`, with a recorded reason.** Not because the lint is wrong but because
  574 sites in a numeric interpreter each need a truncation-and-sign judgement, and a wrong one is a
  silent behavioural change against a byte-for-byte oracle bar. The comment must say what would
  close it: a cast helper in the shape of Materialize's `CastFrom`/`CastLossy`, which this tree does
  not have. Do not write the helper.
* **`shadow_unrelated` is `allow`, with a recorded reason.** 380 renames across files under active
  differential work is churn, and the comment should say so.
* **`needless_pass_by_ref_mut`'s 20 violations are fixed.** Each is a `&mut` parameter never used
  mutably. Fix them by narrowing the parameter, not by silencing the lint.

#### Steps

1. Fetch timely's `[workspace.lints.clippy]` section and reproduce its full membership: the
   allow-level entries and the warn-level entries. Record in your report anything that has changed
   since the controller's reading.
2. Add it to this workspace's `Cargo.toml` under `[workspace.lints.clippy]`, beside the existing
   `[workspace.lints.rust]` block. **Do not touch `unsafe_code = "forbid"`** or the comment above it:
   that line is the record of which crates have been granted an unsafe exception and relaxing it has
   already been done once by mistake and reverted.
3. Set `as_conversions` and `shadow_unrelated` to `allow` with the reasons above stated at the site.
4. Fix the `needless_pass_by_ref_mut` violations. Each fix is a signature narrowing; report the
   count you fixed and confirm it matches the count you measured.
5. **Establish that the adopted lints are actually in force**, which a green build does not show. For
   at least three lints that currently report zero violations, demonstrate the lint fires: introduce
   the violation in a scratch copy, confirm `cargo clippy --workspace --all-targets -- -D warnings`
   goes red, and revert. A lint set that is configured but not reaching the code reads exactly like
   a clean tree.
6. Gates, each exit status read on its own: `cargo fmt --all --check`, `cargo clippy --workspace
   --all-targets -- -D warnings` **from a clean target directory**, `cargo test --release
   --workspace`, and the corpus gate under `REXX_CORPUS_GATE=1`.
7. Report the final lint membership, the three lints you proved live and how, and the violation
   counts before and after.
