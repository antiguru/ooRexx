**OPEN: found by the Phase 9d Task 6.11 round-2 re-review, not yet ruled.** A security manager is not consulted for a protected built-in method.

# Protected built-in methods skip the security manager

Found 2026-10-19 (`task-6.11-rereview-r2.md`, Minor 1). `.Array~method('APPEND')~copy~setProtected`,
`define`d into a subclass and called from a `.Routine` that has `setSecurityManager`, makes the
oracle's manager receive `method APPEND`. The crate never consults the manager. This holds for
every built-in method, `SORTWITH` included, and predates Task 6.11.
