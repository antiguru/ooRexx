# Corrected f comparison

The accepted corrected f pin completed all 198 profiles against its e2 BASE, covering every benchmark program and three rotating rounds. Its SHA-256 is `b695b78fe24903b4be5a2874c8cd128193bbe165f92b50a4d6a8a9956d03d3be`, built from source `3e1aa9c879a90214bb5c680357975f58a0081abc`. [Complete verification](../f-complete-verification.json) checked all 792 profiles through this cohort, including actual profiled arguments, successful statuses, raw summaries, and independent output hashes. Stdout agreed outside the original TIMED cases, stderr was empty, and no SPREAD flag occurred.

| Program over budget | Own delta against e2 |
| --- | --- |
| dirread | +0.36465390% |
| parse | +0.22055592% |
| pingmsg | +0.07291944% |

[All program deltas](8.8f-own-delta.tsv) retain the complete medians, own-cost gates, and spreads, while [stream agreement](output-agreement.tsv) keeps outputs and statuses separate. The original reporting command exited one on whole-process OVER flags, which are retained separately from the own-cost assessment. The final unlaunched `rexxcps.f.r3` profile was recovered once after an optional-analysis approval stalled status delivery, with the preceding 197 profiles preserved. [Incident and recovery](../approval-stall-recovery.json) records the distinction between that agent stall and the profiling driver's completed 361-second bounded batch.
