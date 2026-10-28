**CLOSED 2026-10-28 by Phase 9d Task 8.6 (7596ced8d).**

**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** probe still differs: directory(d) answers empty for a directory named d'ff'x here and enters it on the oracle.

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

Added by the Phase 9b final fix, from the final review's M2 (.superpowers/sdd/2026-10-13-phase-9b/final-review.md).
Task 6's `cd` resolves by bytes, so `'cd' 'dir'||'ff'x` now enters a non-UTF-8 directory and the
`Interp::cwd_text` sites are reachable. The review's measurements after that `cd`, oracle beside it:
- DIRECTORY(): platform.rs:181 (`interp.cwd_text()`). Crate ends `EFBFBD`, oracle ends `FF`.
- ADDRESS command working directory: command.rs:621 (`builder.current_dir(interp.cwd_text())`).
  `address system 'pwd'` answers rc 127 in the crate, rc 0 on the oracle.
- `SysCls` (process.rs:65) was the third such site, crate -1 against oracle 256. The final fix
  passes it the bytes.
`grep -rn 'cwd_text()' rust/crates --include=*.rs` at the final fix also lists install.rs:1905
(`normalize(&path, &self.cwd_text())`), which the review did not probe.
