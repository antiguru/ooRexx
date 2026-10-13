# Task 2 re-review (ed0923fcc..82c8bfabc)

Verdict: needs one more fix (one Important finding, one Minor). Prior findings are otherwise resolved.

## Prior findings
- I1 (false "remaining red groups are the 9b work"): resolved. Sentence deleted; the criterion (ruling 1) is stated in
  phase-9b-gate.md and in plan Task 10 Step 2; SecurityManager's 5 failed tests match suite-9d-set.txt:200-204.
- Minor, header order: resolved ("byte (LC_ALL=C) order"; header is a little long on one line, harmless).
- Minor, revision claim: resolved (sentence dropped).
- Minor, wall time / exit status: resolved for the second run (run-output.txt ends `exit status: 101`, `wall time: 2 s`).
  The "29 s first run" is now cited in gate.md without any committed file holding it (the re-captured run-output.txt
  has no build line; only ed0923fcc's version did). See Minor below.
- Minor, TEST_FILEEXISTS_NIL_INVALID double ownership: resolved per ruling 2 (plan Task 4 and gate.md both say Task 4
  removes line 198 of suite-9d-set.txt, which exists at that line).

## Open findings

### Important
- phase-9b-gate.md owner table: "base/security.manager/SecurityManager (7 blocked tests; ...) | Task 7" is wrong for one
  of the 7. tests.tsv: 6 are blocked on `SYSDROPFUNCS` (Task 7's list) but `TEST_CALL_PASS_THREE_ARGS` is blocked on
  `SYSSEARCHPATH` (SecurityManager.testGroup:143 `return SysSearchPath('foo','bar','c')`), which is Task 4's routine
  (Task 7 Step 2 only says SysDropFuncs unblocks the group, and Task 4's per-task check does not run this group). Fix:
  say 6 blocked tests are Task 7's and `TEST_CALL_PASS_THREE_ARGS` is Task 4's (and, in the gate record or plan, make
  Task 7 Step 4's SecurityManager run the place that confirms it).

### Minor
- gate.md "the first run including the build took 29 s" is not in any committed file at HEAD (only in the history of
  ed0923fcc's run-output.txt). Drop it or say it is the earlier run's figure from that commit.

## Checked and fine
- Every other owner row: SysStem*/SysDumpVariables -> Task 3; SysFileXXX (SysFileCopy/Move/IsFileDirectory/TempFileName),
  SysFileDateTime, SysSearchPath, SysFileSearch, tilde (SysTempFileName), base/class/File (SysGetFileDateTime) -> Task 4;
  SysFileTree (17 failures) -> Task 5; SysGetMessage, SysFormatMessage, ADDRESS (SysGetErrorText) -> Task 6. All match
  the blocked-routine column of tests.tsv and the routine lists under the plan's task headings.
- Figures: the per-group table in gate.md matches tests.tsv (unchanged by the re-run, per the report); 18 groups / 16 red
  is in run-output.txt; exit 101 and 2 s are the last two lines of run-output.txt; suite-9d-set.txt:198 and :200-204 are
  the cited lines.
- suite-9b-groups.txt body unchanged; header edit only. No edits or commits made by this review.
