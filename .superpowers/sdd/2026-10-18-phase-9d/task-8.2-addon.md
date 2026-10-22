# Task 8.2 closing add-on (witnesses only)

Re-review r1 (task-8.2-rereview-r1.md) APPROVED. Before the controller writes Deviations 64-68
(draft: deviations-64-68-draft.txt), add three witnesses. No behaviour changes.

1. **Re-review Minor 1:** add the review's `u4.rex` (task-8.2-rereview-r1.md, lines 58-77; complete
   the elided lines yourself) as a corpus program, oracle-identical at head. Show it red at
   9ce1d8384 (scratch copy).
2. **Deviation 67 (Message~start report line):** a crate-only witness that the started activity's
   report names line 6 and `result`'s re-raise line 3 (probe t8.2/probes/message-start/msgstart.rex).
   The oracle's answer is a race (unpinned mostly line 3 twice; under `taskset -c 0` line 6 then
   line 3, the crate's bytes), so this is a unit test or crate-only expectation, the way Task 7.1
   witnessed the scope-override COPY. If the corpus harness can compare against a crate expectation
   file reliably, that is fine too.
3. **Deviation 68 (Stem subclass UNKNOWN runs):** record the probe (`x = .ss~new('S.'); say x~nosuch`
   with `::method unknown` on `ss`; report section "Other departures") under t8.2/addon/ and add a
   crate-only witness asserting the override's answer.

Each witness: name it, show it goes red when its behaviour is broken (a one-line mutation in a
scratch copy is enough), give the test name and file in a short "Closing add-on" section appended to
task-8.2-report.md. Same rules as before (fmt, clippy, the touched tests; no gates, no callgrind).
Commit; send status, commits, test names.
