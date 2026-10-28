**CLOSED 2026-10-28 by Phase 9d Task 8.7b (probe agrees: docs/superpowers/records/2026-10-18-phase-9d/t8.7b/probes/compiled-method-package/t4c.rex).**
**OPEN: ruled 9d, cluster dynamic-package-name (Phase 9d inventory).** probe still differs: the package is "M" on the oracle and the caller's file here, the cluster's mechanism.

# A method compiled from source text names the caller's file as its package in `>I>` lines

Found by Phase 6.1 Task 4 (report concern 3). Queued by Phase 6.1 Task 12 (2026-10-10). Same rc and stdout; the trace lines differ.

Probe `t4c.rex`, run from a fresh empty directory:

    o = .k~new
    o~go
    ::class k
    ::method go
      self~setMethod('m', 'trace a; nop')
      self~m

Oracle, rc 0:

    [stdout]
    (empty)
    [stderr]
           >I> Method "M" with scope ".NIL" in package "M".
         1 *-* nop
           <I< Method "M" with scope ".NIL" in package "M".

This crate (`rexx-run` at `6a87cd616`), rc 0:

    [stdout]
    (empty)
    [stderr]
           >I> Method "M" with scope ".NIL" in package "t4c/t4c.rex".
         1 *-* nop
           <I< Method "M" with scope ".NIL" in package "t4c/t4c.rex".

Suspected site: the package a source-compiled method records (`install.rs`, `Method` from source): the oracle's package is named after the method.
