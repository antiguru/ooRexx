**OPEN: found by Phase 9d Task 6.8, not yet ruled.** Two variable-reference shapes differ from the oracle, both before and after Task 6.8.

# Variable references: two gaps outside Task 6.8's clusters

Found 2026-10-19 by Task 6.8 (`task-6.8-report.md`, Concerns).
* **`>a.` on an unset caller stem.** The oracle creates the caller's stem: after
  `use local abc.; use arg >abc.`, `abc.1` reads `A.1` and the caller's `symbol('a.')` is `VAR`.
  The crate gives `ABC.1` and `LIT`
  (`docs/superpowers/records/2026-10-18-phase-9d/t6.8/probes/p5`).
* **A reference passed on from an alias of an object variable.** A method whose `use arg >q` bound
  `q` to the caller's exposed `v` passes `>q` on. The next callee's `use arg >r` reads `R` here and
  the object variable's value on the oracle (`t6.8/probes/p14`, the same at the parent commit).
