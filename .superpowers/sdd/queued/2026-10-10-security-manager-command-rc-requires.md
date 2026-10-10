# The security manager is consulted where the oracle does not consult it

The Task 11a implementer (Phase 6.1, fix round 1) found this on 2026-10-10. It predates the task, and the control probes differ the same way.

With `setSecurityManager` installed, the oracle does not consult the manager for a host command's RC or for the `::REQUIRES` of an external call. The crate consults it for both. See the classification table in `.superpowers/sdd/2026-10-07-phase-6-1/task-11a-report.md`, where both sites are marked n/a. Before fixing, cite the oracle's source for each of the two checks.
