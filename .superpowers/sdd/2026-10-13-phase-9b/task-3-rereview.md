# Phase 9b Task 3 re-review, fix round 1

Range `ef1357a27..0009afbc5`. Probes ran from fresh empty directories (deleted afterwards), oracle and
crate under `memcap 4G timeout 20`. The crate binary (10:13:28) is newer than the last source change
(`stems.rs`, 10:10:26). No crashing shape was run on the oracle.

## Verdicts

- **I1, entry 38: resolved.** `oracle-crashes.txt` entry 38 has the program, rc 139, 3 runs of 3, the
  date, the cause location (`writeVariable`, `RexxUtilCommon.cpp:1449` is the `ObjectToString` call,
  checked) and the crate's answer. The crate-only pin is the unit test
  `a_string_method_raising_inside_the_dump_is_trapped`, with the reason in its doc; I ran it, it passes.
  The shape is in no witness. The pin is a unit test, not a case file; that satisfies the ruling.
- **I2, 999999999 wording: resolved.** Report text and Concern 5 now name the eager
  `new_array(bounds * 3)` (`StemClass.cpp:1654`), the 1 GiB / 4 GiB measurement, and the OOM licence.
  Matches the review's table.
- **Minor 2, line-by-line dump: resolved.** `dump_line` writes each line to the file or to standard
  output as soon as the value converts. Interleave witness (`talker`, a `STRING` method that says
  `STRING ran`): oracle and crate both print `Name=S., Value='S.'`, `STRING ran`,
  `Name=S.1, Value='K'`, rc 0. The full `rexxutil_dump_variables.rex` witness: stdout and rc 216
  identical; stderr differs only in the program path (my two directory names).
- **Minor 3, sort hold: resolved.** `interp.roots.activity_mut().push_temp(value)` follows
  `required_string_value` in the loop (`stems.rs:692`), a real root, not a comment. Note: no
  committed test fails without it, because the reqstr latch still covers every reachable case; the
  report says the probe was temporary. That is what the ruling accepted ("if wrong, a redundant root").
- **Minor 4, entry 37 wording: resolved and accurate.** Untrapped on the oracle, 3 runs each:
  `call SysStemSort '*'` gives 40.919 and `first` 3, `last` 2 gives 40.921, both rc 216 with the
  ordinary report (`Error 40 ... Incorrect call to routine.`), and the crate matches (rc 216, same
  report). rc 216 is an ordinary error exit (256 - 40), not a signal: a signal would be 139, as in
  entries 37 and 38, and a control `exit 7` gives 7. The entry's wording ("reported untrapped as
  usual", rc 216 with the ordinary report, 40.920 the witness's own last clause) matches.
- **Minor 1 and the out-of-scope item:** left alone as ruled (queued to 9b1 and the reqstr gate).
  Not part of this diff.

## Entry 38 and the crash-pin description

Entry 38's description and the crate-only pin's reason are consistent: oracle SIGSEGV even under
`SIGNAL ON SYNTAX`; crate `trapped 93.900`. The test allows the `Name=A` line before the trap but
does not require it, with the reason (variable order) stated.

## New defects in the fix diff

None found. `dump_line` converts before it writes, copies via `to_text` before `write_out`, and the
`file` handle is passed `&mut`; failed writes are still ignored as before. The record files
(`run-output`, corpus, collect-stress) are re-run at `d6c1b2d85`. The `sourceline_oracle` file count
(98) matches the program's new line count in the diff.

## Out-of-scope observations

None.

**Verdict: all sent findings resolved; no open findings.**
