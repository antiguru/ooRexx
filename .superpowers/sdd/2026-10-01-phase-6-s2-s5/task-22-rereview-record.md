# Task 22 fix round 1: record re-review

Reviewer: sonnet, read-only. Diff `4f811a962..540e6a72e`, files `phase-6-gate.md`, `phase-6-pinning.md`,
`rust/tsan.supp`. Evidence in `s4-close-evidence/`. Item 14 (`LEAD:` lines, criteria 1/2/outer_context
log repointing) not flagged, as instructed.

## Verdicts

* **G1 (blocking-ops block): FIXED.** I reran the recorded command (with the `sed` prefix strip) at
  the head: 154 lines, and the block's 154 lines are identical (`diff` prints nothing). No count in
  prose. Every hit is classified: `input.rs:80` split out; the new `sync.rs:83/:127`, `timer.rs:454`,
  `pool.rs:43/:188`, `signal.rs`, `dispatch/library.rs:1155`, `lent.rs`, `callbacks.rs` rows are
  placed in the right class (`pool.rs:43` and `library.rs:1155` are inside `#[cfg(test)]`, `signal.rs`
  and `sync.rs:234+` inside `mod tests`, checked).
* **G2 (ADDRESS row vs P64): FIXED.** Row says unredirected command (P64), off-baton, and inline
  without a pool reservation, uninterruptible, DEVIATIONS 12. Matches progress.md P64 and
  `phase-4-exclusions.txt` entry 12.
* **G3 (`input.rs` vs P69): FIXED.** Split row `input.rs:80`: "keeps the baton and idles on the inbox
  (P69, DEVIATIONS 13); not a park; a halt ends it (P60)", inline fallback cited to DEVIATIONS 12.
  Checked against `input.rs:372-379` (`reserve()` else inline `read_stdin_chunk`; pool path wraps in
  `signal::unblocked`) and entry 13's text. `dispatch/stream.rs` row no longer carries console input.
* **G5 (queued list): FIXED.** `stdin-chars-after-drain` added, `interpret-translation-error-traceback`
  dropped, `send-site-cache-self-customization` labelled "a design note, not a defect". All six
  files exist in `.superpowers/sdd/queued/`.
* **G6 (figures without a source): PARTIAL.**
  * Overshoot table: all 28 cells equal `overshoot.log` digit for digit; the command beside it is the
    script's usage line; `overshoot.sh`/`.rex` present. Fixed (but see N3, N4).
  * `b8ec39593` row run: replaced by a `c66650b52` run; `s2rows.log` shows `ok`, no rerun;
    `s2-table.txt:138` is the SysSleep `pass same` row quoted; the table differs from G6's only at
    SysSleep (diffed). Fixed.
  * Pinning: `pinning.log:799` `18 passed; 0 failed` quoted; `pinning-diff.log` arrivals equal the S4
    columns of the table (all 16 figures); `pinning-diff.py` and the diff command are committed.
    One clause is unevidenced (N1).
  * TSan: `tsan.sh` is committed; `tsan/tsan-1.out` has `api exit 0`, `lib exit 0`, `int exit 0`,
    `no tsan log`; `test result` lines in `api.txt:108` (70), `lib.txt:1045` (990, 13 filtered),
    `int.txt:39/47/89/96` (32, 2, 36 with 1 filtered, 1) equal the prose. Fixed.
  * Criterion 2 lines: G4 `:1660`, `:2047`, G6 `:1664`, `:2052`, `outer_context` G4 `:3405-3407`,
    G6 `:3254-3256` all open at the quoted tests. Fixed.
  * Not fixed: figures the brief said to drop unless re-derived are still present with no cited
    log (N2).
* **G7 (counts in prose): FIXED.** `123 lines`, "ten more", `18 measured::` (now quoted from
  `pinning.log:799`), `1127 in all` are gone; remaining counts are `test result` lines quoted from
  the cited logs beside the command.
* **G8 (TSan exclusions): FIXED.** Thirteen exclusions listed by exact path, matching `tsan.sh`'s
  `DEPTH` list one for one and `lib.txt:1045`'s `13 filtered out`; plus the `signals` test. One
  reason each (some "the same"). The bounded-path sentence is present and names an existing test
  (`pool.rs:343`) that asserts `exits == 1`, `takes == 1`, 25 levels.
* **G9 (nits): FIXED.** (a) Wording now says REPLY's cell is `pass` here and in G6 and `differ` in G4;
  checked in `s2-table.txt:115`, `g6-criterion-one-table.txt:115`, `g4-criterion-one-table.txt:115`.
  (b) P67 appends "holds for one interpreter per process (queued `2026-10-05-two-interpreters-signal-race`)".
  (c) `tsan.supp:2-3` cites `"S4 close", "Criterion 3, race checking"`, which matches the heading.

## Rulings

* P64 as stated in the gate: correct (every unredirected command off the baton; inline and
  uninterruptible only without a pool reservation, DEVIATIONS 12), matches progress.md:298.
* P69 as stated: correct (reader keeps the baton, no other activity runs, DEVIATIONS 13 owns the
  starvation), matches progress.md:318 and entry 13. The pinning paragraph "not a park, no kind" is
  consistent with it.

No em-dashes and no forward-looking sentence in the added lines (grepped the diff).

## New findings

* **N1 (Low) `phase-6-pinning.md` S4 close, "with one count in `pinning.log` altered, it prints that
  row".** A claim about a run nobody logged; `pinning-diff.log` shows only the clean run. Either
  commit the altered-log output as evidence or delete the clause.
* **N2 (Medium) Figures with no cited log, carried over from the deleted scratch.** G6 told the fixer
  to drop any figure it could not re-derive. These sit in `phase-6-gate.md` with no evidence file, and
  the only source is `task-22-report.md` (lines 70-73, 87, 172):
  * "65,551 frames, 65,526 of them `Plan::note`" and "141,427 frames of `TestSendMessage0`
    callbacks" (exclusion table rows 1 and 9; `gdb` runs);
  * `SigBlk: fffffffe3ffbea07` against `0000000000004003` (signals row);
  * "19.7 ms late with the socket timeout and 1.5 ms with the `ppoll` timeout", "about 1.5 ms late
    under niceness 5", and "the test allows 26 ms" (the `Wake` paragraph after the overshoot table).
  Re-derive and commit the log, or drop the figures (the reasons hold without them).
* **N3 (Low) "one run each on a quiet machine"** (gate.md, overshoot paragraph): "quiet" is not
  evidenced by any log. Drop the phrase.
* **N4 (Low) overshoot table, `c66650b52` row vs the prose.** The prose says `c66650b52` changed only
  other unix targets' wait, yet its cells (0.000413-0.000456) are 4-6 times `2e6917afe`'s
  (0.000064-0.000097) and match the S3-close row (0.000411-0.000426). Same machine and run, so the
  difference is machine state or the `timerfd`/`poll` path; the record does not remark on it and a
  reader will wonder. One sentence ("within the S3 close's range, under the 26 ms bound") would
  settle it; I did not verify the cause.
* **N5 (Low) loom paragraph.** `loom.log` holds no command line and no commit; "At `c66650b52`, in a
  target dir of its own" is asserted. The test result line (`358.99s`) is quoted correctly. Add the
  command to the log or drop "in a target dir of its own".
* **N6 (Info) gate.md blocking-ops table, tests row.** `scheduler/pool.rs:43` is listed with the
  `(#[cfg(test)] code)` parenthetical attached only to the next entry, `dispatch/library.rs:1155`.
  Both are `cfg(test)`; the placement reads as if only the second is.

## Out of scope observations

* `loom.log` and `s2rows.log` start mid-build (scratch paths under `/tmp/claude-1000/...`), so
  neither carries its own exit status; `s2rows.sh` prints `s2rows exit $?` to the terminal, not the
  log. The prose "exits 0" rests on the `ok` line.
* The criteria bullets still cite bare `g4-test-release.txt:NNNN`, which live under
  `bg/52b038a80/logs/`; item 14 covers the repoint, and the path should be given when it is done.
* The `rexxutil.rs:87` hit is a doc comment containing `std::fs::remove_file`; the row says so
  ("a doc comment"), correct.
