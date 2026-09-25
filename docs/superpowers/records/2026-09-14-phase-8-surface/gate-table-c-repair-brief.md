# Task: make the whole-workspace corpus gate green, or licensed

You investigated this failure read-only and wrote
`.superpowers/sdd/2026-09-14-phase-8-surface/gate-table-c-probe.md`. You are now
the implementer for the repair. You may write to the tree, and you commit your
own work.

## Where this sits

Phase 8's Surface Task 3 closed at `a442e0b5b` with four of five gates green.
The fifth, the whole-workspace corpus-gated run, has been red since `98a0db498`
(2026-09-12), which is what you established. It is red again at `a442e0b5b`,
measured by the controller from a clean tree with the variable proved to reach a
child process:

    g4: REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --workspace --no-fail-fast -- --test-threads=8
    exit 101, 132 `test result: ok`, exactly one binary failing:
    concept_and_class_gate_table (crates/rexx-exec/tests/gate_table_c.rs:1795),
    21 passed 1 failed.

That is the entire red set today. `phase-7-gate.md` section 8 recorded eight
failures under this gate at Phase 7's close; seven have since been fixed.

This task runs before the queued file-split plan and before Phase 8's remaining
surface tasks, because a gate that has been red for four days hides every
regression it would otherwise catch.

## What the failing test already tells you

From the run's own report, so you do not need to rediscover it:

- verdicts over the whole table: `agree` 1378, `unanswered` 110, `loud` 0.
- by owning phase, rows and rows not yet `agree`: 5a 135/0, 5b 6/0, 5c 1225/0,
  **6: 13 rows, 13 not agree**, **7: 94 rows, 82 not agree**,
  `deferred-rexxcontext-stackframes` 10/10, `never-expected-to-agree` 5/5.
- gated by the run: the 82 whose owning phase is closed. Phase 6's 13 are red by
  what looks like the same cause and are not gated, because `"6"` is absent from
  `CLOSED_PHASES` in `gate_tables/mod.rs` while `"7"` is present.
- the rows carry their own cause: both sides answer rc 163 with a
  `Method INIT with scope "Stream"` or `"TICKER"` traceback. Oracle and crate
  agree that the probe raised during construction before any documented name was
  asked. `unanswered` is a property of the row set, not a divergence, which you
  confirmed for two rows by hand.

## The work

1. **Give the row set a construction expression.** `method_bodies.rs` already
   has `RECEIVER_OVERRIDES` (`.File~new('/')` and so on) and gets real,
   oracle-agreeing answers from it. Gate table C has no equivalent for File,
   Stream and StreamSupplier. Add one. Do not copy the sibling's table blindly:
   for each class you add, check that the oracle constructs the same thing, and
   prefer a receiver whose construction cannot depend on the filesystem outside
   a directory you create. Cover Alarm and Ticker too, whose phase 6 rows fail
   the same way, unless you find they fail differently.
2. **Re-run and adjudicate what remains.** After the receivers exist, some rows
   will answer and agree, some will answer and diverge, and some may still raise.
   Each row that does not end at `agree` gets decided on its own merits against
   the oracle, on three separate descriptors from a fresh empty directory, and
   either fixed or recorded in `docs/superpowers/plans/phase-4-exclusions.txt`
   with its transcript. Do not record a row as licensed because it is
   inconvenient: a real divergence in a closed phase is a defect, and if you find
   one, say so rather than writing it into the exclusions.
3. **The sibling's skip.** `method_bodies.rs`'s cross-check explicitly skips
   reconciling with gate table C for exactly these three classes. Once C has
   receivers, either remove the skip or state in the code why it must stay.
4. **Phase 6's absence from `CLOSED_PHASES`.** Determine from
   `docs/superpowers/plans/2026-07-27-rust-rewrite.md` whether phase 6 is closed.
   If it is, its absence is a hole of the same kind as the one that let this
   failure exist: say so in your report, and say what adding `"6"` would turn
   red. Do not add it in the same commit as the repair.
5. **The record that let it stand.** `docs/superpowers/plans/phase-7-gate.md`
   section 8 documents this gate exiting 101 at Phase 7's close and then declares
   the phase's exit criterion met by a sentence that does not cover that gate,
   which contradicts the gate rule in `rust/CLAUDE.md`. Do not rewrite the
   original sentence: a gate document records what was believed at the time. Add
   a dated correction beside it saying what was actually red, for how long, and
   under which commit it was repaired.

## Definition of done

`REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --workspace --no-fail-fast --
--test-threads=8` exits 0 from a clean tree, and every row that changed verdict
is either agreeing with the oracle or carries an exclusions entry with its
transcript. If you reach a point where green requires licensing something you
believe is a real defect, stop and report instead.

## Constraints

- The C++ tree at the repo root and `samples/`, `build/`, `ootest/`, `oodocs/`,
  `testbinaries/` are read-only. `api/` headers are frozen. No extension under
  `build/` is ever rebuilt.
- `unsafe` only in `rexx-api/src/ffi.rs` and `src/load.rs`; none in `tests/`.
- No process-global state: no `std::env::set_var`/`remove_var`, no
  `set_current_dir`, no writes to fd 0/1/2 from interpreter code.
- Oracle runs from a **fresh empty directory**:
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.
  Three separate descriptors, never `2>&1`. Read `rust/corpus/oracle-crashes.txt`
  first and never run its entries.
- Never register anything with the rxapi daemon. Never load a prebuilt extension
  needing `librexx.so` or `librexxapi.so`.
- Comments minimal: one-sentence overview plus parameters, returns, panics and
  non-obvious properties. No em-dashes. **A comment may never state the size of
  a set**, and a claim of the form "every X" or "the only X" must be derivable by
  running something.
- Git: stage explicit paths, never `git add -A`; never amend, force-push, reset
  --hard, `git checkout --` an edited file, or use bare `git stash`. Commit
  messages go in a file passed with `-F`, ending with the two attribution lines
  used elsewhere on this branch (copy them from a recent commit with
  `git log -1 --format=%B`).
- Commit before any long gate run, and leave the tree clean while one is running.
- Scratch only in this session's scratchpad. Check `df -h /tmp` first, use your
  own `CARGO_TARGET_DIR` for pristine builds, delete them by explicit path, and
  never use `rm` with a star glob.

## Report

Write the full report to
`.superpowers/sdd/2026-09-14-phase-8-surface/gate-table-c-repair-report.md` and
return only: status, the commits, the gate's exit code and failing-binary count
at your last commit, what phase 6 would turn red, and any row you refused to
license with the reason.
