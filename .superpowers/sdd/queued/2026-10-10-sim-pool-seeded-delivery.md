# P52 lending under simulation: a pool with seeded delivery

Out of 6.1 scope: spec section 4 runs the native-call pool at bound 0 in sim, and criterion 6 says the seeded gate is no evidence for P52 lending. Queued by Phase 6.1 Task 12 (2026-10-10).

Scout C (`scout-c-report.md`, open decision 2) recommended bound 0 first and, second, real pool threads whose completions are awaited at once and delivered at a seeded later clause, since P52 lending is where the Phase 6 races were and sim does not reach it at bound 0. No probe: a design item.

Suspected site: the pool under sim (`Pool::new` in `lib.rs`, `scheduler/pool.rs` `reserve`), and completion delivery through the inbox (`scheduler.rs` `file_completions`).
