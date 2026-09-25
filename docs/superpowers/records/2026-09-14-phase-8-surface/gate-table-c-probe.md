# `gate_table_c` under `REXX_CORPUS_GATE=1`: what it is, since when, and whether it is licensed

Investigated read-only against `/home/moritz/dev/repos/ooRexx-rust-rewrite` (worktree, branch
`plan/rust-rewrite`). No file in the worktree was edited, staged, or committed.

## 1. What `REXX_CORPUS_GATE=1` switches

`rust/crates/rexx-exec/tests/gate_tables/mod.rs` defines `CORPUS_GATE_ENV = "REXX_CORPUS_GATE"`.
`corpus_gate()` reads it; `Report::new` prints "STRICT (the gate)" when it is set and "REPORT
MODE -- NOT THE GATE" otherwise, but **both modes run every probe and compute every verdict
identically** -- the variable does not change what is measured.

What it changes is `verdict_is_gated(phase)`:

```
corpus_gate() && (closing_phase() == Some(phase) || CLOSED_PHASES.contains(&phase))
```

`gate_table_c.rs`'s `concept_and_class_gate_table` test collects every row whose owning phase is
gated and whose verdict is not `Some(Verdict::Agree)` into a `gated` list, and at the end asserts
`gated.is_empty()`. Without the env var, `verdict_is_gated` is always `false`, so `gated` stays
empty and the test can only fail on a **structural** failure (checked unconditionally, in every
mode -- things like a missing probe file, a derived probe not matching its committed file, or an
oracle run that did not finish). With the var set, any row owned by a phase in `CLOSED_PHASES`
(currently `5a`..`5j`, `7`) whose verdict is not `Agree` turns the run red.

`unanswered` (`gate_tables::UNANSWERED`) is a row's verdict when **the oracle's own probe never
reached the question the row is about**, so no byte comparison of the two sides means anything --
concretely here, when a method probe's receiver (`o = .ClassName~new`) itself raises on both
sides, so the `say o~hasMethod(...)` lines below it never run on either interpreter. `unanswered`
is explicitly **not** `agree` (`gate_table_c.rs:1712`-`:1719`, "that is the safe direction: a row
nothing could answer must never make the gated count smaller"), so once a phase enters
`CLOSED_PHASES`, every one of its `unanswered` rows reddens the corpus gate exactly like a real
divergence would.

## 2. Since when

**First failing commit: `98a0db498edb50b55ad55c5de729c4df4b92bae5`, "Repair the eleven test
binaries Phase 7 left red, and the cadence that hid them", 2026-09-12.**

`class-set.txt` and `class-methods.txt` (the row sets `gate_table_c.rs` reads) have not changed
since `9738e9858` (2026-09-03, "Phase 5d Task 1"), well before Phase 7 started -- confirmed with
`git log --oneline --follow` on both files. The 82-row set (File instance 50, Stream instance 24,
StreamSupplier instance 8, all owned by phase `"7"`, all `not-covered` in `class-set.txt`) has
existed unchanged since then. What changed on 2026-09-12 is `gate_tables/mod.rs`'s
`CLOSED_PHASES`, which gained `"7"` in that commit (confirmed with
`git log --follow -- rust/crates/rexx-exec/tests/gate_tables/mod.rs` and reading `CLOSED_PHASES`
at every commit in the file's history).

Built both sides from pristine `git archive` extractions, each in its own `CARGO_TARGET_DIR`,
running `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test gate_table_c --no-fail-fast`:

* At the parent, `5bcb28edb9bd079cd6f932a827cbcec0ffd91846` (2026-09-11): exit 0, `22 passed; 0
  failed`. The report already shows `7: 94 rows, 82 not yet agree`, but `gated by this run: 0
  row(s)` -- phase 7 is not yet in `CLOSED_PHASES`, so nothing about it is gated.
* At `98a0db498edb50b55ad55c5de729c4df4b92bae5`: exit 101, `21 passed; 1 failed`. Same `7: 94
  rows, 82 not yet agree`, but now `gated by this run: 82 row(s)`, and
  `concept_and_class_gate_table` panics: "82 row(s) of gate table C owned by a closing or closed
  phase do not `agree` with the oracle".

This matches the two measurements already on file (82 rows at `2d155158c` and at `e0b30d156`,
with the same File/Stream/StreamSupplier breakdown) and identifies the exact commit where the
count went from harmless to gated: **the row set did not move; the gate around it did.** Phase 7's
own formal close, `73aed8f25` (2026-09-13), inherits the same failure and does not change the
`CLOSED_PHASES` comment's substance beyond marking it permanent.

## 3. Is it licensed

**Not by any file `phase-4-exclusions.txt` or the phase docs use for that purpose, but it is a
known, recorded state -- one that contradicts the project's own standing gate rule.**

`phase-4-exclusions.txt`'s header states Phase 7 "HAS CLOSED AND OWES THIS FILE NOTHING,
2026-09-13" and lists what it delivered; nothing in it names File, Stream, StreamSupplier, or
gate table C. `licensed_divergences.rs` and the divergence-licensing mechanism it backs are about
byte-level oracle disagreements (e.g. the `MAX_EVAL_DEPTH` license), not about an `unanswered`
verdict, so they do not apply here either.

What **does** cover it is `docs/superpowers/plans/phase-7-gate.md` section 8, "The gate readings",
recorded from the run at `73aed8f25`:

| gate | command | exit | figures |
|---|---|---|---|
| G3 | `cargo test -j 4 --release --workspace --no-fail-fast` | 101 | 2348 passed / 6 failed |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --workspace --no-fail-fast` | 101 | 2347 / 8 |
| G5 | `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 | 516 of 516 |

and: "**G4's** is that set [G3's six] plus the two gate-only tables, `concept_and_class_gate_table`
and `directive_option_gate_table`." So Phase 7's own close-out **ran** the whole-workspace corpus
gate, **saw** `concept_and_class_gate_table` (gate table C) in the failing set, and **recorded it
in the phase's own gate document** rather than treating it as a silent regression. The document's
framing is that the phase's *exit criterion* -- the one-sentence row quoted at the top of that
file -- "is the sentence, and it is met", and that sentence does not mention G4. G5, the narrower
`-p rexx-exec --test corpus` run, is 516 of 516 and is a **different test** (`corpus.rs`, not
`gate_table_c.rs`); its green result does not cover gate table C at all.

This is a real gap against `rust/CLAUDE.md`'s own gate list, which states for exactly this command
("`REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast`... a debug run"): "Expected
against the tree as committed: exit 0, no failures." Phase 7 closed with that gate at exit 101 and
said so in its own gate document, but `CLOSED_PHASES`'s comment in `mod.rs` ("a verdict of its own
moving is a regression") does not acknowledge the 82 pre-existing rows, and no exclusion-file row
was written for them. So: **known and on the record, not silent -- but not licensed by the
mechanism (`phase-4-exclusions.txt` / `licensed_divergences.rs`) the project uses to make a gap
like this stop being red.**

## 4. Is the failure the rows being wrong, or the table's expectation being wrong

**The table's expectation (its row set) is incomplete, not the interpreter's behaviour. On the
two rows checked, the crate agrees with the oracle byte for byte.**

Both `file__instance.rex` and `streamsupplier__instance.rex` open with `o = .File~new` /
`o = .StreamSupplier~new` (a bare, zero-argument constructor call) because `class-set.txt` commits
no construction expression for either class (columns 7 and 8 of both rows are `-`, i.e.
`NO_PROGRAM`). Ran both, from fresh empty directories, oracle and crate compared on three separate
descriptors, no `2>&1`, `oracle-crashes.txt` checked first for either bare-`~new` shape (not
present -- entries there are all about `.Stream~new('file', ...)` forms taking an argument, or
`~copy`/`~uninit` sequences, none matching a bare zero-argument `~new`):

* `.File~new`: oracle and crate both exit 163, both stdout empty, both stderr
  `Error 93.901: Not enough arguments for method; 1 expected.` -- identical.
* `.StreamSupplier~new`: oracle and crate both exit 159, both stdout empty, both stderr
  `Error 97.1: Object "STREAM" does not understand message "!QUERY_STREAMTYPE".` -- identical.

So neither row is a divergence at all; `unanswered` is exactly the correct label for "both sides
raised before the question was asked", per the table's own definition.

**The deeper finding**: this is fixable, and the fix already exists in a sibling gate.
`rust/crates/rexx-exec/tests/method_bodies.rs` reads the same `class-set.txt` file and the same
column, and would face the identical bare-`~new` problem -- except it carries its own
`RECEIVER_OVERRIDES` table with working construction expressions for exactly these three classes:
`("File", ".File~new('/')")`, `("Stream", ".Stream~new('/dev/null')")`,
`("StreamSupplier", ".Stream~new('/etc/hostname')~supplier")`. Its own committed
`corpus/method-bodies.txt` shows the crate answering the great majority of File's, Stream's and
StreamSupplier's instance methods correctly against the oracle (`answers rc 0`) once a real
receiver is used. `method_bodies.rs`'s own `check_receivers_match_table_c` -- which asserts that
its receiver matches gate table C's committed one -- explicitly **skips** any class present in
`RECEIVER_OVERRIDES` (`gate_table_c.rs:624`: `if overridden.contains(...) { continue; }`), so
nothing anywhere asserts that the two tables agree for File, Stream, or StreamSupplier, and gate
table C was never given the matching commitment. Gate table C's `class-set.txt` row was not
touched by the Phase 7 work that built this override table (last touched 2026-09-03; the override
table's own comments are dated Phase 7 Task 13 and Task 22, both after that).

One more thing found along the way, reported because it bears directly on this: `method_bodies.rs`
lines 381-382 state "`class-set.txt` gives `File` the construction `.File~new('.')`, which is the
documented example and the right thing in that column" -- checked against the file as it stands
today (`awk -F'\t'` over the `File` row), and that is false: columns 7 and 8 are both `-`. Whether
that comment was ever true or was written aspirationally was not established; either way it is a
comment describing a mutable repo aggregate that no longer matches the file it cites.

**Verdict:** the 82 rows are not evidence of a live crate defect. They are evidence that gate table
C's own row set was never updated to give File, Stream, and StreamSupplier a working construction
expression, something a neighbouring table in the same tree already does for the identical
classes. Closing this needs a `class-set.txt` change (a plan amendment, per that file's own rule
that a row is added or moved only as one) plus re-deriving the three probe files it drives --
not a crate fix.

## Disk state left behind

Everything was built under this session's scratchpad
(`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/gatec-bisect-t8surface/`),
never in the repository. Two `git archive` extractions (`before-5bcb28e/`, `cand-98a0db4/`), each
with its own `CARGO_TARGET_DIR`, plus four small ad hoc probe directories under the scratchpad
root (`oracle-probe-file/`, `oracle-probe-streamsupplier/`, `crate-probe-file/`,
`crate-probe-streamsupplier/`) were created and have all been deleted by explicit path. What
remains is three small log files (`before-5bcb28e.log` ~49K, `cand-98a0db4.log` ~50K,
`build-rexx-run.log` ~1.6K) inside `gatec-bisect-t8surface/`. `df -h /tmp` before this work: 25G
used / 39G available; after cleanup: 26G used / 37G available.
