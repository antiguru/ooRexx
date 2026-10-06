# Task 22 review: the written record (gate section and pinning)

Reviewer: sonnet, read-only. HEAD `75f8aeda8`. Files: `docs/superpowers/plans/phase-6-gate.md`
(S4, lines 455-817), `docs/superpowers/plans/phase-6-pinning.md` `## S4 close` (771-802),
`rust/tsan.supp`, `docs/superpowers/plans/phase-4-exclusions.txt` DEVIATIONS 9-13.

## Verified, no finding

Opened and compared against source:

* Status lines (gate.md:640-655) equal `bg/52b038a80/status.txt`; G4 `3029 passed, 1 failed` and G6
  `3034 passed, 0 failed` equal the sums of the `test result` lines; `g4-test-release.txt:2052`,
  `:2190` are the P48 rerun line and the failing cell; `g6-criterion-one-table.txt` cell for
  SysSleep TEST_SLEEP_CONCURRENT is `same (P48 rerun)` as quoted.
* Criterion 1: G4 and G6 `criterion-one`, `-s3` and `timer` tables, filtered to non-`pass`/non-`same`
  rows, equal the S3 close's tables (gate.md:350-405). The only differences between G4 and G6 are
  the SysSleep row and REPLY TEST_REPLY_TWICE_REPLYASSERT's normal cell (G4 `differ: rc 0`, G6
  `pass`), exactly as lines 699-703 say. Every Alarm and Ticker row is `pass` in both timer tables.
* `the_framework_ticker_runs_without_dash_u` ok in G4 (`:1660`) and G6 (`:1664`);
  `the_alarm_and_ticker_groups_pass_in_both_modes` ok in both (`:2047`, `:2052`); the three
  `outer_context` tests ok at `g4-test-release.txt:3405-3407` (G6 `:3254-3256`).
* loom: `g9-loom.txt:80` `15 passed ... finished in 371.56s`.
* TSan: ten run directories under `p6-scratch/t22/tsan-run-1..10` each total 1127 passed, 0 failed;
  run 10 gives 986 (16 filtered), 70, 32, 2, 36 (1 filtered), 1; no `tsan.*` file in any
  `tsan-run-N`. Command text equals `tsan-final.sh`. (`tsan-final-2` holds one `tsan.*` log: the
  `_dl_close_worker` report that preceded its suppression, as the report says.)
* Pinning: `pinning.log` and `pinning2.log` both `18 passed`; S3 columns of the arrivals table equal
  pinning.md:703-711.
* Rulings P52-P69 are all listed (gate.md:769-799) and each matches its `progress.md` ruling,
  including P67 as amended and P69. P66 and P68 appear only as "withdrawn by P69" (line 796); no
  withdrawn ruling is described as current anywhere in the S4 text.
* DEVIATIONS 9-13 exist in `phase-4-exclusions.txt:1440-1570`, each `OWNER: none`, with the cited
  rulings; they are the only DEVIATIONS rows added since `7266ae03c` (`git log` on the file,
  `git diff` of `+ N.` rows). Entry 13 is the P69 text. Rows 10 and 12 mention no withdrawn ruling.
* No em-dash and no forward-looking sentence in the S4 gate text or the S4 pinning diff.

## Findings

1. **The blocking-operations command output is stale at HEAD (gate.md:459-609), and the S4 close
   does not say so.** Run at HEAD, the command prints 154 lines, not 123. Line numbers in the
   classification table moved (`command.rs:622` is now `:644`, `:690` is `:712`, `:466` is `:467`,
   `rexx-run.rs`, `rexxutil.rs:597/619/752` shifted) and Task 21 added hits the table does not
   classify: `input.rs` `std::io::stdin().lock().read(&mut chunk)` (the table's `input.rs:75`,
   `:128` no longer exist), `dispatch/library.rs` `std::fs::OpenOptions::new()`,
   `scheduler/pool.rs` `std::thread::sleep(100 ms)`, `signal.rs` (process spawn, two sleeps),
   `sync.rs` and `timer.rs` (`Wake::wait`), and test-file hits in `scheduler/tests/callbacks.rs`,
   `lent.rs`, `pool.rs`. Correction: rerun the command at the head, replace the output block, and
   classify the new sites (the stdin read is off-baton via P60/P64 on a pool thread and also
   keeps the baton per P69, so say which; `Wake::wait` is the timer thread's own wait, same class
   as `sync.rs:42`). If the section is meant as the Task 18 snapshot, retitle the first line of
   459 to say "at Task 18's fix round 1; later changes in `command.rs`, `input.rs`, `sync.rs`,
   `timer.rs`, `signal.rs` are not reclassified here" and drop the table's present-tense line
   numbers.
2. **gate.md:471 contradicts P64.** The row says an `ADDRESS` command waits off the baton "where
   another activity is alive or a test mode is set". P64 (progress.md, Task 21 fix round 2) makes
   every unredirected command wait off the baton, single-activity too, and falls back inline only
   without a pool reservation (DEVIATIONS 12). Correction: "an `ADDRESS` command with nothing
   redirected: the child starts on the baton, a pool thread posts what it writes ... Where no pool
   thread can be reserved, `command.rs:712` waits on the baton, uninterruptible (DEVIATIONS 12)."
3. **gate.md:476 contradicts P69 and the pinning record.** `input.rs:75`, `:128` are classified
   "spec 2.1 wrapper (`PULL` and console input) ... pinned, counted by the pinning report". The
   pinning record you added says a console read "is not a park" and has no kind (P69, DEVIATIONS
   13; `ParkKind::Input` removed at `4cf934a48`). Correction: split `input.rs` out of that row: "a
   read of the default input stream: keeps the baton and idles on the inbox (P69, DEVIATIONS 13);
   not a park; a halt ends it (P60)".
4. **The `LEAD:` line (gate.md:694-695).** Proposed exact text, to replace both lines. The
   `<...>` slots are filled from `bg/75f8aeda8/status.txt` when it reads `finished`; at review time
   it holds only `G1 fmt exit 0` and `G2 clippy exit 0`, so no result may be written yet:

   ````
   G4 at `52b038a80` is red on the regression above and is not waived. `2e6917afe` fixes it. The
   gates were run again in full by the same script on `75f8aeda8`, the head that carries the fix;
   status file `.superpowers/sdd/2026-10-01-phase-6-s2-s5/bg/75f8aeda8/status.txt`, logs beside
   it in `logs/`. Its result lines, verbatim apart from the repository path:

   ```
   <contents of status.txt>
   ```

   G4 release: <passed> passed, <failed> failed. G6 debug: <passed> passed, <failed> failed (sums of
   `test result` lines).
   ````

   If G4 on `75f8aeda8` is green, add one sentence: "`the_s2_rows_of_the_derived_list_in_both_modes`
   passes in G4 with no P48 rerun on the SysSleep row." If anything else is red, name the test, the
   log and line, and the cause before closing S4; do not substitute the row run above. Two
   consequences to apply in the same edit: (a) Criteria 1, 2 and the `outer_context` bullet
   (lines 699-713) cite `52b038a80` logs and line numbers; repoint them to the `75f8aeda8` logs,
   since only those include `2e6917afe`; (b) G9 loom at `52b038a80` (line 760-763) predates
   `2e6917afe` and `b8ec39593`, so the `--no-run` sentence can go once the `75f8aeda8` G9 line
   exists. Keep the `52b038a80` run as the record of the regression, not as the close.
5. **Queued list (gate.md:812-817) against `.superpowers/sdd/queued/`.** Missing:
   `2026-10-05-stdin-chars-after-drain` (Task 21 fix round 4, concern 5; queued by `75f8aeda8`,
   after the section was written; task-22-report.md concern 6 flagged it). Mis-filed:
   `2026-10-03-interpret-translation-error-traceback` was found by the Task 12 review (S3), says
   "not related to parking", and is not an S4 finding; remove it or move it to the S3 section.
   Doubtful: `2026-10-04-send-site-cache-self-customization` is a design note ("not scheduled"),
   not a defect found in S4; either drop it or say "design note". Correct entries: `aborted-run-leaks-interp`
   (Task 18, P53), `call-on-notready-stdin-eof` (Task 21 rr4 N-3), `stream-fifo-terminal-seek`
   (rr3 F-2), `two-interpreters-signal-race` (rr3 F-3).
6. **Figures with no command or source beside them.**
   * The overshoot table (gate.md:671-677) names the probe in prose but not the script that ran it
     on each tree; its numbers exist only in the bisect note and in
     `/tmp/claude-1000/.../p6-scratch/t22/` (ephemeral). Add the probe's path under
     `.superpowers/sdd/2026-10-01-phase-6-s2-s5/` (copy `probe-at.sh` and the `.rex` there) and the
     log the table was read from.
   * The `b8ec39593` row run (lines 682-692): command quoted, result figures not tied to a log
     (`p6-scratch/t22/s2rows.log`, `s2-table.txt`). Copy both next to `bg/` and cite them.
   * pinning.md:775-801: command quoted, but "18 `measured::` tests", the arrivals and the `diff`
     claim ("checked by `diff` against this file's S3 tables") name no log and no diff command.
     Cite the log (`pinning2.log`) and write the diff command.
   * TSan "10 times" (gate.md:744-747): cite `tsan-final.sh` and the ten `tsan-run-N` directories,
     committed or copied out of `/tmp`.
   * Criterion 2 and the Alarm/Ticker sentence (lines 703-704) give no log lines; G4 `:1660`,
     `:2047`; G6 `:1664`, `:2052`. The `outer_context` bullet cites G4 only; add G6 `:3254-3256`.
7. **Set sizes stated in prose.** gate.md:459-461 "prints 123 lines (113 at `bddcd480e`; the ten
   more are ...)" and pinning.md:775 "18 `measured::` tests" and gate.md:747 "1127 in all" are
   counts. The first is already wrong at HEAD (finding 1). Replace with the command and name the
   artifact, or derive the count from the logged output beside it (the `test result` sums are
   fine as figures quoted from a log; the 123-line count and the "ten more" clause are not).
8. **The TSan exclusions are not enumerated (gate.md:725-727, 755).** `DEPTH` is eight substring
   filters; the 16 filtered lib tests are never named, and `notifier_failures_too_many_to_nest`
   and `the_translator_on_a_pool_thread` (the report calls it the thread-count test) are not
   explained by "tests that measure recursion against the native stack". Add the derived list
   (`cargo test -p rexx-exec --lib -- --list` minus the run's list; `all.txt` and `filtered.txt` in
   scratch hold it) and one reason per name. Add the report's concern 3 as a sentence: the pool's
   deep-recursion callback path is not race-checked at depth.
9. **Wording nits.** (a) gate.md:690-692, "Every other row's cells are G6's, but REPLY ... normal
   cell `differ: rc 0`": that cell is G4's, not G6's (G6 has `pass`); write "Every other row's
   cells are G6's; REPLY TEST_REPLY_TWICE_REPLYASSERT's normal cell is `differ: ...` as in G4".
   (b) P67 (line 797) drops the amendment's qualifier "holds for one interpreter per process",
   which `two-interpreters-signal-race` exists to record; append it. (c) `rust/tsan.supp:2` cites
   `phase-6-gate.md, S4, "Criterion 3"`; the heading is `#### Criterion 3, race checking` under
   `### S4 close`.

## Verdict

CHANGES REQUESTED. Nine findings (3 correct-the-record: 1, 2, 3; 1 placeholder: 4; 1 omission: 5;
4 hygiene: 6-9). Every measured figure that was checkable matches its log; no withdrawn ruling is
described as current; DEVIATIONS 9-13 are complete and accurate. The `LEAD:` line cannot be
closed until `bg/75f8aeda8/status.txt` reads `finished` (when last read: G1 and G2 exit 0, nothing further).
