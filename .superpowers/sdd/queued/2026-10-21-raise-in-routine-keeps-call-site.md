**CLOSED 2026-10-22 by Phase 9d Task 8.3 (68e7f15d5).**

**OPEN, queued 2026-10-21 by the controller; proposed 9d.**

# A raise inside a Rexx ::routine leaves its call site's resolution kept

Found by Phase 9d Task 7.1 fix round 2 (task-7-report.md, "Fix round 2", Concern 2). A call op that
raises now forgets its site, so the next call looks the name up again as the oracle does, but only
for raises that come back through the call op (library and native routines, builtins). A raise inside
a Rexx `::routine`'s own activation leaves the site's resolution kept, where the oracle looks the
name up again. Probe it against the oracle before changing anything.
