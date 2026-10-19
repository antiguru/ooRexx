**CLOSED by Phase 9d Task 6.12** (`security-manager-method-event`): `setProtected` marks the method its object stands for, and a copy and `define` carry it. Witness `corpus/lang/security_manager_set_protected.rex`, identical to the oracle.

# Protected built-in methods skip the security manager

Found 2026-10-19 (`task-6.11-rereview-r2.md`, Minor 1). `.Array~method('APPEND')~copy~setProtected`,
`define`d into a subclass and called from a `.Routine` that has `setSecurityManager`, makes the
oracle's manager receive `method APPEND`. The crate never consults the manager. This holds for
every built-in method, `SORTWITH` included, and predates Task 6.11.
