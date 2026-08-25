# Task 16 report: fix round 2

Scoped against `.superpowers/sdd/2026-08-17-phase-5a/task-16-fixround-2-brief.md` and the
re-review it points at, `task-16-rereview.md`. Defects B through J, all in comments or in
`task-16-report.md`, plus one corpus addition (D). No behaviour changed except the corpus
program D adds; nothing in `src/` runs differently, so no sitting was run.

| # | What it was | What was done |
| --- | --- | --- |
| B | `Activation::reply`'s doc claimed its named readers are a self-checking enumeration -- a name going stale is not a compile error, and nothing here says it is | Dropped the enforcement clause; kept "named rather than counted, so a number here is a fact that goes stale with nothing to notice" |
| C | `Interp::deferred`'s doc and the report both attributed a two-order/five-order gap to "sittings" rather than to the two shapes being different programs | Doc rests the claim on the review's own five-order shape, which reproduced at five across two independent sittings, three of the same rows. The report gained the reconstructed `two.rex` inline (the original scratch file is gone) with its own fresh 30-run measurement, five orders on this sitting. **Superseded by round 3, below**: this row originally also said the causal wording moved from "sittings" to "the shape is written", which round 3 found unsupported and retracted -- see the round 3 section |
| D | The guard test's doc credited 34.902 with a corpus backstop that did not exist | Added `corpus/lang/method_guard_when_not_logical.rex`, measured against the oracle and both engines from a fresh directory before adding: rc 222, `Error 34.902`, all three descriptors byte-identical on oracle, `ir` and `tree-walker`. Registered in `corpus/phase-5a.txt` and `coverage.rs`'s `EXPECTED_SUBSET_5A`. Not in `collect_stress.rs`'s `NO_ALLOCATION_PROGRAMS`: measured, it does allocate (the stress test's own both-directions assertion catches this, so it was checked rather than assumed) |
| E | `exec_guard`'s doc carried an unmarked second copy of entry 7's program, with its own (wrong) 6-second deadline and no do-not-run warning | Replaced the inline program and deadline with a pointer to `corpus/oracle-crashes.txt` entry 7, which is not run again -- the file's own rule is that an entry is never re-run to confirm it |
| F | The marker-count table quoted `grep -c` giving `crates/rexx-exec/src/activation.rs:3`, where the command actually gives 4 (`Activation::reply`'s marker wraps two lines) | Deleted the counts; the report now presents the site list alone, located by `grep -n`, since the list is the claim and a line count cannot see a marker that wraps |
| G | The contribution table tagged all four `dispatchclass` cells `min-derived` and said "recomputed from `min`, the uncontaminated statistic on both arms" -- true of the tw rows, but the ir rows are `changed`'s median over `base`'s min | Table tags now read `min/min` (tw) and `median/min` (ir); the prose says which side used which statistic and why (`changed/ir`'s own median is not contaminated the way `base/ir`'s is). The figures are unchanged -- this is a label fix, and the reconciliation still closes at four decimal places |
| H | "19 unresolved-link warnings" understated what the number covers | Now: 19 warnings total, 4 unresolved links and 15 links to private items, none of the 4 in `rexx-core` or `rexx-exec` -- re-counted from a fresh `cargo doc --workspace --no-deps` this round, matching the review's own reading |
| I | The fix-round-1 table's row 6 said the old test name "carried a count" and that "five more" set-size phrases were reworded | The old name carried a false universal, not a count. The genuinely reworded phrases are named instead of counted: "the pair of probes", "The last two rows", "the four legality refusals, one fatal each", and the test's own name |
| J | `raised_guard_not_logical`'s LEGALITY paragraph was inserted between the summary line and the catalogue-text sentence, breaking the local convention its sibling `raised_if_not_logical` keeps | Reordered: summary line and catalogue text stay one paragraph, LEGALITY follows as its own paragraph, matching the sibling six lines above |

## D, measured

From a fresh empty directory, absolute paths, three descriptors read separately, both engines:

```
say .K~m
::class K
::method m class
  expose v
  v = 'x'
  guard on when v
  return 'ran'
```

```
oracle        rc=222  Error 34.902:  Value of expression following GUARD keyword must be
                       exactly "0" or "1"; found "x".
crate ir      rc=222  byte-identical stdout, stderr and exit status
crate tw      rc=222  byte-identical stdout, stderr and exit status
```

Corpus moves 185 -> 186. `corpus_differential` under `REXX_CORPUS_GATE=1`: **186 of 186
matching**. `collect_stress.rs`'s `the_l0_subset_passes_again_under_collect_on_every_allocation`
was run with the new program tentatively added to `NO_ALLOCATION_PROGRAMS`; it failed, showing
the program does allocate, so it was left out of that list rather than forced in -- the
both-directions assertion did its job.

## Gates

```
cargo fmt --all --check                                              -> exit 0, no output
cargo clippy --workspace --all-targets -- -D warnings                 -> exit 0, no warnings
REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast    -> exit 0
  corpus_differential: 186 of 186 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast    -> exit 0
  corpus_differential: 186 of 186 matching
```

## Files changed

* `rust/corpus/lang/method_guard_when_not_logical.rex` -- new corpus program (D).
* `rust/crates/rexx-parse/tests/sourceline_oracle/method_guard_when_not_logical.txt` -- new,
  generated by the sanctioned `.Package~new` driver in `sourceline_oracle.rs`'s own module
  comment, verified byte-identical to the program (15 lines).
* `rust/corpus/phase-5a.txt`, `rust/crates/rexx-exec/tests/coverage.rs` -- the new program
  registered in `EXPECTED_SUBSET_5A`.
* `rust/crates/rexx-exec/src/activation.rs` -- B.
* `rust/crates/rexx-exec/src/lib.rs` -- C (`Interp::deferred`'s doc).
* `rust/crates/rexx-exec/src/run.rs` -- E (`exec_guard`'s doc), J (`raised_guard_not_logical`'s
  doc order).
* `.superpowers/sdd/2026-08-17-phase-5a/task-16-report.md` -- C (the reconstructed `two.rex`
  and its measurement), F, G, H, I.

## What this round could not check

Same as fix round 1: entry 7's hang is not re-run, by the crashes file's own rule, so its
recorded deadline and mode are taken as given rather than re-verified. The reconstructed
`two.rex` in the report is not the original file -- there is no way to confirm it is the same
program byte for byte, only that it is the same shape as described. Its own five-order result
does not confirm or refute the original 18/12 reading; it stands beside it as a second,
independently-measured data point for the same conclusion.

## Fix round 3

The team lead's own concern 2 from round 2's report -- flagged there, not resolved -- turned
out to falsify a sentence round 2 shipped. `Interp::deferred`'s doc said "the distribution
depends on how the shape is written, not on which sitting ran it". Round 2's own
reconstruction of the 18/12 shape is a counterexample to that clause: it claims to be the same
program and measured five orders instead of two, on one sitting. Either the reconstruction is
not that program, or the original 30-run reading was undersampled, and with the original file
gone nobody can tell which -- so the causal clause was asserted in the one place a reader would
go to check it, and unsupported there.

**Ruling: drop the causal clause.** Rest the "no single answer" conclusion on the shape that
does reproduce (five orders across two independent sittings, three of the same rows), state
the 18/12 reading as a sitting whose program was never preserved, and say plainly that nothing
here explains why the two readings differ. Fixed in `Interp::deferred`'s doc
(`crates/rexx-exec/src/lib.rs`) and in `task-16-report.md`'s Determinism section, which had the
same claim in two places (the round-2 paragraph about the reconstruction, and the round-1
paragraph about "which shape decides how many there are, not which sitting runs it"). Added, in
its place, the one claim that does survive without asserting a mechanism: the review's own
statistic that `A-after` follows `B-replied` in 8 of 30 runs of the five-order shape and in 0 of
30 of the 18/12 reading, about 1e-4 if both were the same distribution -- a fact about the two
recorded samples, stated as exactly that and nothing more.

Row C above is updated to point here rather than restate the retracted wording.

## Commits

* `51103413a` Give the guard value check a corpus witness (D)
* `5bc6aca0a` Fix round 2: correct six sentences the re-review found false (B, C, E, J --
  the commit message's own count is wrong, it addresses four; not amended, see below)
* `49e4c9e27` Name what the 98.936/98.937 mutation reddens, not a corpus total -- a defect
  the team lead found while this round was starting, not in the brief. Re-measured on the
  186-entry tree by mutating and restoring `Interp::returned_value`'s raise myself: 184 of
  186 matching, mismatches `lang/method_reply.rex` and `lang/method_reply_exit_status.rex`,
  and `a_value_returned_after_a_reply_reports_the_oracles_own_98_936` reddens beside
  `corpus_differential` -- confirming the team lead's own reading and also fixing a second,
  adjacent false claim in the same sentence ("is the only red thing in the workspace", which
  this test's own mutation-sensitivity already contradicted). Checked the neighbourhood
  (`activation.rs`, `dispatch.rs`, `lib.rs`, `run.rs`, `run/tests.rs`, `coverage.rs`,
  `collect_stress.rs`, `corpus.rs`, `owners.rs`, `loud.rs`, `roots.rs`) for any other comment
  quoting a corpus total the same way: none found.
* `52682fe94` Fix round 3: drop the unsupported causal clause from `Interp::deferred`'s doc.
  Its own message states in the body that round 2's `5bc6aca0a` is titled "correct six
  sentences" and addresses four -- recorded in git per the team lead's instruction, since the
  report is git-ignored.

F, G, H and I are report-only and are not in any commit: `.superpowers/` is git-ignored.

## Concerns

None beyond what fix round 1 already recorded. Every defect here is prose-only or adds a single
corpus program; the corpus gate, fmt and clippy all pass at the new count. `5bc6aca0a`'s subject
says "six sentences" and the commit addresses four (B, C, E, J); caught after committing and left
uncorrected rather than amended.
