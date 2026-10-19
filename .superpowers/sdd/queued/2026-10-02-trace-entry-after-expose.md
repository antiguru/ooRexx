**CLOSED 2026-10-19 by Phase 9d Task 6.10.** `queued-probes/texpose` is identical to the oracle on stdout, stderr and rc (`docs/superpowers/records/2026-10-18-phase-9d/t6.10/queued/`).

# `>I>` missing for a method whose `TRACE` follows its `EXPOSE`

Found by Phase 6 S2-S5 Task 9. A method whose first instruction is `EXPOSE` and second `TRACE A`
announces itself on the oracle; this crate does not (`trace_invocation_entry` announces only on the
first instruction). Pre-existing, not concurrency.

```rexx
o = .k~new
o~m
::class k
::method m
  expose v
  trace a
  v = 1
```

Oracle stderr: `>I> Method "M" with scope "K" in package ...`, `7 *-* v = 1`, `<I< Method "M" ...`,
rc 0. This crate: `7 *-* v = 1` only, rc 0.
