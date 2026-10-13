# Remaining lossy UTF-8 conversions on paths

Source: Phase 9b Task 4 review Q5 (.superpowers/sdd/2026-10-13-phase-9b/task-4-review.md), lines at
1cbce6cde. Rexx strings are bytes. A name that is not valid UTF-8 is altered at these sites and names
the wrong file:
- DIRECTORY: platform.rs:181/189/196
- SysFileTree (9b Task 5 owns): rexxutil.rs:797/798/841
- .File: dispatch/files.rs:36/411/556
- .Stream opens: dispatch/stream.rs:61/555/776
- CD command: command.rs:309/567
- external file read: install.rs:1859/1866
Fix: pass OsStr from bytes, with an 'ff'x-name witness and a lossy decoy file per site.
