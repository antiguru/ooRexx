**OPEN: m5 belongs to the targeted performance round after 9d.** m1 is resolved by Deviation 71 and witness `2589e88b3`, and m4 is fixed by `68e7f15d5`.

**Phase 9d Task 8.3 (2026-10-22):** m4 fixed in 68e7f15d5; m5 measured on bench-programs/define.rex (8d409845b), task-8.3-report.md; m1 is Task 8.5's.
**Phase 9d Task 8.5 (2026-10-27):** m1 witnessed in 2589e88b3 by tests::the_context_variables_list_an_alias_under_its_own_name (rust/crates/rexx-exec/src/tests.rs); listing the alias as Z is Moritz's ruling ("List Z., Deviation"), its Deviation candidate in task-8.5-report.md.

# 9b1 Task 8b review minors (.superpowers/sdd/2026-10-14-phase-9b1/task-8b-review.md)

- m1 (pre-existing): `.context~variables` lists a `USE ARG >z` alias; the oracle omits it.
- m4 (pre-existing, silent wrong answer): `old~instanceMethod('FOO')` on an instance created before
  `define` answers the class's current method; the oracle answers the old one (source and identity).
- m5: `free_unheld_behaviours` builds a std HashSet over every behaviour per freeing collection; no
  bench program runs `define`, so the cost is unguarded. Add a define bench program, then make the
  pass cheaper if it shows.

**Task 8.8 inventory reconciliation:** The remaining m5 cost is owned by [the later performance round](2026-10-19-perf-round-and-whole-review.md). The historical 7.38% profile predates the `n = 3000` benchmark revision in `a2a4f78b7`, so it is not a measurement of the current source. The benchmark addition in `8d409845b` is complete, while dictionary traversal remains open. [The residual audit](../2026-10-18-phase-9d/task-8.8-residual-inventory-audit.md) records the separate evidence for m1, m4 and m5.
