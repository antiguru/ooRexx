**OPEN: queued by Moritz's ruling 2026-10-19 (Deviation 50).** Close Deviation 50 at no per-clause cost.

# Swap to a chunk compiled for the new trace setting

Deviation 50 lets a clause whose trace line cannot be delivered run its own work before its 97.1.
Phase 9d Task 6.10 fix round 1 measured every in-driver fix as a cost on untraced clauses
(`docs/superpowers/records/2026-10-18-phase-9d/t6.10/r1/important-1/options/`; the cheapest, a
`TraceStale` op on `wip/t610-r1-fallible-trace` at `3e5c927b3`, costs assign +0.54% and rexxcps
+1.73%). The unbuilt alternative: at the sites that change the trace setting (the TRACE
instruction, the `TRACE()` builtin, TRACE in INTERPRET), switch the running body to a chunk
compiled for the new setting, so no chunk ever runs stale. It needs a pc map between the two chunks
and loop frames that survive the swap. Probes: `t6.10/r1/important-1/probes/`. When it lands,
withdraw Deviation 50.
