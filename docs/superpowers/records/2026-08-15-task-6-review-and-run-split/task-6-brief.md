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

