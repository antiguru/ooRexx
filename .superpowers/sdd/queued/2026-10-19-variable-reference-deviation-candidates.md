**OPEN: Deviation candidates for Moritz, found by Phase 9d Task 6.8.** Neither is copied, neither is witnessed, and no suite test depends on either.

# Variable references: two oracle behaviours that look like artefacts

Found 2026-10-19 by Task 6.8 (`task-6.8-report.md`, Concerns).
* **A failing `PROCEDURE EXPOSE` list under an inherited `SIGNAL ON` trap.** With `sel = '9bad'`,
  `pg: procedure expose c d (sel)` under `SIGNAL ON SYNTAX` raises 31.2 on both sides. The oracle's
  handler reads `c=C d=D`: it runs in an isolated frame where `c` and `d` are not exposed, yet the
  selector is, so a handler assignment to it reaches the caller. The crate's handler reads
  `c=3 d=4`, in the caller's frame, because the crate isolates only after every name resolves.
  NOVALUE reaches the same path since Task 6.8 (`procedure expose c (nolist)` under
  `SIGNAL ON NOVALUE`). Probes: `docs/superpowers/records/2026-10-18-phase-9d/t6.8/probes/p9`,
  `p10`, `p11`.
* **An empty stem's name compared by identity.** The oracle's `isAliasable` compares
  `stem->getValue() == variableName` as pointers. `use arg abc., >abc.` with `.stem~new('ABC.')`
  binds, because the literal is the interned symbol string, and with `.stem~new('AB' || 'C.')`
  raises 98.995. The crate compares the names' bytes and binds both (`t6.8/probes/p16`). Copying
  the identity check would make string representation observable, which Deviation 4 leaves
  unspecified.
