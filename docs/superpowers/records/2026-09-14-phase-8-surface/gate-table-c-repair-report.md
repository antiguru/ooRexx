# Gate table C repair: report

Two commits on `plan/rust-rewrite`: `a4b6a37ce7261c8dddd656cde434a4e467c54011` (the repair) and
`5757857a45b2c9f1935317e5c59a73c123e4fee6` (the exclusions entry and the phase-7-gate.md
correction, HEAD). Both gate reads below are from a clean tree at HEAD.

## Status

**Green.** `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --workspace --no-fail-fast --
--test-threads=8` exits 0 at `5757857a4`, 133 of 133 test binaries `ok`, 0 failing, 2636 passed /
0 failed summed across the run. All five gates:

| gate | command | exit | figures |
|---|---|---|---|
| G1 | `cargo fmt --all --check` | 0 | |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 | |
| G3 | `cargo test -j 4 --release --workspace --no-fail-fast -- --test-threads=8` | 0 | 133 binaries ok, 2635 passed / 0 failed |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --workspace --no-fail-fast -- --test-threads=8` | 0 | 133 binaries ok, 2636 passed / 0 failed |
| G5 | `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 | 29 passed / 0 failed / 1 ignored, corpus differential 604 of 604 |

Full status log: `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/gate-table-c-repair-gates.status`,
started `2026-09-16T04:58:36Z`, finished `2026-09-16T05:15:47Z`, first line is the gated commit's
sha (confirmed to be `5757857a4`).

## Whole-table verdict counts, gate table C only

| | before (`5bcb28edb`, and unchanged through `2d155158c`) | after (`5757857a4`) |
|---|---|---|
| agree | 1378 | 1460 |
| unanswered | 110 | 15 |
| loud | 0 | 13 |
| diverge-both | 0 | 13 |
| gated (closed-phase rows not `agree`) | 82 (only under `REXX_CORPUS_GATE=1`, since gating for `"7"` was not yet live at `5bcb28edb`) | 0 |

`loud` and `diverge-both` are the same 13 rows counted two ways (a row can be both), not two
separate populations on top of the other three. The 15 remaining `unanswered` rows are unrelated to
this repair: `Pointer` instance (5, owner `never-expected-to-agree` -- `class-methods.txt` marks
the class `unreachable`, so no run can ever move it) and `StackFrame` instance (10, owner
`deferred-rexxcontext-stackframes`) -- both untouched by this commit, both already `unanswered`
before it, and both correctly outside the gate for reasons unrelated to phase 6 or 7.

## Every row that changed verdict

**82 rows, `unanswered` to `agree`** (all phase 7, all now closing the gate): File instance (50),
Stream instance (24), StreamSupplier instance (8). Checked from a fresh directory against the
oracle before committing, all three constructions verified byte-identical on stdout, stderr and
exit status.

**13 rows, `unanswered` to `diverge-both`/`loud`** (all phase 6, not gated because `"6"` is not in
`CLOSED_PHASES`): Alarm instance (7), Ticker instance (6). The oracle now constructs and answers
every `hasMethod` line; this crate refuses loudly at construction --
`rexx-exec: the LIBRARY REXX entry point "alarm_startTimer" is not implemented (Phase 6)` (plus a
`GUARD`-wait refusal ahead of it for Alarm) and `"ticker_createTimer" is not implemented (Phase 6)`
for Ticker. rc 120 against the oracle's rc 0 on both, stdout empty against seven and six
`instance 1` lines. This is the row I refused to license: it is a real, unimplemented native
feature, not a design choice, so it went into `phase-4-exclusions.txt` as an EXCLUSION owed to
Phase 6 (with its transcript) rather than being written off as agreeing or excused as a divergence.

**No row went the other way.** Nothing that was `agree` moved.

## What adding `"6"` to `CLOSED_PHASES` would turn red

Exactly those same 13 rows -- measured by running `REXX_CORPUS_GATE=1 REXX_PHASE_GATE=6 cargo test
-p rexx-exec --test gate_table_c` (which gates the named phase without editing `CLOSED_PHASES`):
`13 row(s) of gate table C owned by a closing or closed phase do not agree with the oracle`, all
`gate-tables/methods/alarm__instance.rex` and `gate-tables/methods/ticker__instance.rex`. Phase 6
is not closed today: `docs/superpowers/plans/2026-07-27-rust-rewrite.md`'s phase table carries no
`CLOSED` marker on row 6 (unlike rows 5, 7 and 8, which do), and its exit gate -- kernel lock, guard
locks, `REPLY`, `GUARD`, message objects, the ooTest concurrency groups, a clean TSan run -- is
plainly unmet, so `alarm_startTimer`/`ticker_createTimer` being unbuilt is expected rather than a
surprise. `CLOSED_PHASES`'s absence of `"6"` is therefore correct as the tree stands, not a second
instance of the gap this task closed, and nothing in these two commits adds it.

## The two changes the diff raises

**`class-methods.txt` (214 lines) and `class-set.txt` (10 lines): both regenerated, not
hand-edited, by the command each file's own header names**: `cargo run -p rexx-extract --bin
rexx-extract-docs -- --oodocs ../oodocs --interpreter ../interpreter --out corpus/docs`, run from
`rust/`. `tests/extract_docs.rs`'s `every_row_set_is_exactly_what_its_extractor_derives_today`
would fail on a hand edit, so this is the only route the tree accepts, and it is the committed one.

`class-set.txt`'s 10 lines are 5 changed rows (File, Stream, StreamSupplier, Alarm, Ticker), one
line each, each counted twice by `git diff` (removed and added). `class-methods.txt`'s 214 lines
are 107 changed rows, same count doubled: `status` and `reason` are stamped from the **class's**
coverage onto **every** row of that class in `class-methods.txt`, both arms, not only the rows
whose construction I added -- confirmed by counting: File 10 class-arm + 50 instance-arm, Stream 1
+ 24, StreamSupplier 1 + 8, Alarm 0 + 7 (it has no class-arm rows at all), Ticker 0 + 6, summing to
107. The class-arm rows' own probes and verdicts are unaffected -- they already sent to `.ClassName`
directly and stay `agree` -- only their `status`/`reason` text moved, because that text is a fact
about the class, not the arm, and `assert_eq!(row.status, class.status, ...)` in `gate_table_c.rs`
is what requires the two files to agree on it.

**`rexx-extract/src/docs/classes.rs` (+14, -0): this is Task 1 itself, not a side effect of it.**
Gate table C's row set has exactly one place a construction expression can be committed from --
`CONSTRUCTION_PROGRAMS`, the table `method_bodies.rs`'s own `RECEIVER_OVERRIDES` is modelled on and
that the brief named directly. The 14 lines are five new entries in that table (`ALARM`, `FILE`,
`STREAM`, `STREAMSUPPLIER`, `TICKER`), each mapping the class to the Rexx expression I chose and
checked against the oracle from a fresh directory before it went in. Nothing else in the file moved:
`CONSTRUCTION` (the bare-`~new` measurement table), `METHOD_OWNER`, and every other table are
untouched.

## Also touched, and why

`method_bodies.rs`'s own `instance_receiver` falls back to `class-set.txt`'s construction when a
class carries no `RECEIVER_OVERRIDES` entry -- Alarm and Ticker carry none -- so giving them a
construction moved their rows there too, `unanswered` to `loud`, with the same evidence. Its
regression guard (`no_row_started_diverging_or_stopped_answering`) caught the drift and named the
refresh command; `method-bodies.txt` is refreshed to match, 26 lines, the same 13 rows. File,
Stream and StreamSupplier's `RECEIVER_OVERRIDES` entries stay: that table sends the documented
methods and needs a receiver they can act on (a real path, a real readable file), where gate table
C only sends `hasMethod` and needs one that merely constructs, so the two tables' receivers are
deliberately different and `check_receivers_match_table_c`'s skip for these three classes is
permanent -- stated at both sites now rather than left implicit. One stale comment is corrected
along the way: `method_bodies.rs` claimed `class-set.txt` already committed `.File~new('.')` for
File, which was false before this change and remains false after it (the committed expression is
`.File~new('/nonexistent-gate-table-c-file')`).

`docs/superpowers/plans/phase-4-exclusions.txt` gains the Alarm/Ticker entry described above.
`docs/superpowers/plans/phase-7-gate.md` gains a dated `## 9. Correction, 2026-09-16` section: it
does not rewrite section 8, it says that `concept_and_class_gate_table`'s presence in G4's failing
set was not pre-existing as section 8's prose implies, names `98a0db498` (2026-09-12) as where it
started, `a4b6a37ce` as the repair, and that the gate was red for four days.

## Fix round 2: StreamSupplier's construction was not portable, and two guards against it

Review found one real defect: StreamSupplier's committed construction embedded this worktree's own
absolute path to its fixture, so the row answered correctly only because this checkout happens to
sit at that path -- a pristine extraction elsewhere, which this project routinely builds for gate
verification, would not find the file.

**The fix**, `5b0b69170`: `.context~package~name` answers the currently running probe's own
absolute path identically on both engines (verified from a directory unrelated to the repository
before committing), and `filespec('path', ...)` on it gives the probe's own directory, so
`.Stream~new(filespec('path', .context~package~name) || '../fixtures/streamsupplier_seed.txt')
~supplier` locates the fixture relative to wherever the probe itself was checked out. Verified
green in a pristine `git archive` extraction to a different absolute path (deleted after).
`class-set.txt` re-derived by the committed extractor command, confirmed byte-identical on a
second re-derivation; `class-methods.txt` does not move. README.md and `method_bodies.rs`'s prose,
flagged as literally true of the fixture file but misleading about the reference to it, corrected
to describe the runtime resolution.

**Two guards, both in `42530000e` and `e6d9d1231`.** First, an in-place invariant: `run_probe`
(`gate_table_c.rs`) now checks, where the probe's path is chosen, that its canonicalized path
stays under the canonicalized corpus root, and records a Structural failure -- red in every mode --
if not, because `method_bodies.rs` already stages probes into a temp directory for a different
table and nothing asserted gate table C never would. Proved both ways with a temporary edit that
staged the resolved path into `std::env::temp_dir()` before the check runs: every probe's row (237
files) turned into the same "outside corpus" failure and the run stopped before any oracle
comparison; reverted, diffed clean, 237 stray files removed from `/tmp`.

Second, an exists-check in `extract_docs.rs`, added after a ruling reversal: the checkout-root
check alone only catches a path rooted in this repository, not one a contributor's own machine
happened to have when they verified the construction locally before committing (this table's own
convention). `every_absolute_path_literal_in_a_construction_program_does_not_exist` extracts every
single-quoted literal beginning `/` from each `CONSTRUCTION_PROGRAMS` entry and asserts none
exist. Zero false positives against the table as committed. Proved both ways with a temporary edit
giving `FILE`'s entry `/etc/hostname` in place of its placeholder: failed naming the path and the
machine; reverted, diffed clean. Deliberately not extended to `method_bodies.rs`'s
`RECEIVER_OVERRIDES`, whose real OS paths are load-bearing there.

**Closing five-gate run**, revision `e6d9d12317f92da6b784cae63fa1c51799795075` (HEAD), from a
clean tree (`git status --porcelain` empty before and after), started `2026-09-16T11:51:51Z`,
finished `2026-09-16T12:08:21Z`:

| gate | command | exit |
|---|---|---|
| G1 | `cargo fmt --all --check` | 0 |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| G3 | `cargo test -j 4 --release --workspace --no-fail-fast -- --test-threads=8` | 0 |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --workspace --no-fail-fast -- --test-threads=8` | 0 |
| G5 | `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 |

267 `test result: ok` blocks, 0 `test result: FAILED`, across the whole run. Two earlier background
five-gate runs (started at the intermediate commits `5b0b69170` and `42530000e`) were killed rather
than let finish or reported: both were superseded mid-run by a further commit before they reached
`finished`, and killing a `cargo test` run can orphan its spawned test binaries. `pgrep -af
"ooRexx-rust-rewrite/rust/target"` was checked right after the kill, before this run started, and
none survived; the reviewer independently re-checked the same way after this run finished, also
clean.
