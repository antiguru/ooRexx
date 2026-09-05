# Task 4 report — the witnesses, verified in a clean extract

The closing task of `docs/superpowers/plans/2026-09-04-phase-5c-followup.md`. Run by the controller
rather than an implementing agent: it builds nothing and its whole content is verification. `$D` is
`…/scratchpad/task4` and `$B` is `/home/moritz/dev/repos/claude-build-scratch/5c-followup-task4`.
Every claim is **measured** with the command that produced it.

The plan's own instruction is `**Enumerate from the tree, not from this plan**`, so every list below
comes from `ls`, from the committed subset file, or from the test source, and never from a task
report.

## 1. The witnesses are filed, once each

`ls corpus/lang/mutablebuffer_*.rex`, six programs: `mutablebuffer_state`, `_instance`, `_readers`,
`_mutators`, `_caseless`, `_conversion`. For each, **measured**:

* named in `corpus/phase-5c.txt` -- six of six, by `grep -qxF` on the exact line.
* **not** also named in `corpus/unfiled.txt` -- none of the six, so no witness is both filed and
  excused. (`corpus.rs`'s own `both.is_empty()` assertion enforces this too; this is the independent
  read of it.)
* carries its `crates/rexx-parse/tests/sourceline_oracle/<name>.txt` companion -- six of six.

## 2. The subset file and its pin agree line for line

`corpus/phase-5c.txt` with comments and blanks stripped, against the string literals of
`EXPECTED_SUBSET_5C` extracted from `crates/rexx-exec/tests/coverage.rs`: **29 entries each, `diff`
silent, both directions**. Every one of the 29 exists on disk under `corpus/`.

## 3. Verified in a clean `git archive` extract

The plan requires this because 5c's own phase flip did not work and being checked in place is how
that was missed. `git archive 44114f49f rust interpreter` into `$B/extract`, with
`CARGO_TARGET_DIR=$B/target` -- its own, so nothing built in the worktree can satisfy it.

| check | reading |
|---|---|
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus --no-fail-fast` | rc 0, **`363 of 363 matching`** |
| `cargo test --release -p rexx-exec --test coverage --no-fail-fast` | rc 0, 20 passed |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | rc 0, 1 passed |

**That the extract compiled its own tree rather than reusing the worktree's, measured** rather than
assumed, because a fast green run is exactly what a build that did not happen looks like: 28
`Compiling` lines in `$D/ex-corpus.err`, `Compiling rexx-exec v0.1.0
($B/extract/rust/crates/rexx-exec)` naming the extract's path, the test binary reported as
`$B/target/release/deps/corpus-f9f9e8218e9b7b8e`, and `du -sh $B/target` 449M.

## 4. What this task did not change

No `CLOSED_PHASES` change, no new subset file, no `SUBSET_FILES` row -- the plan's head rules those
out and nothing here needed them. The witnesses were already filed by the tasks that wrote them,
which is the amendment Task 2 forced; this task confirms it rather than performing it.

## 5. Open, and not closed by this task

* The receiver sweep over the other 13 collection classes (105 rows, moves 0 today) is Moritz's
  call, recorded since Task 1.
* `MutableBuffer~verify` answers `counted` on every path while the builtin's past-the-end zero is an
  untagged text (`task-3a-report.md` §6.1).
* Task 3e's M3 prediction was partly falsified and is recorded as such in that report.
