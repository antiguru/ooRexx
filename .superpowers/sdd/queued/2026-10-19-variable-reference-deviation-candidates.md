**RULED: 9d, ruled by Moritz 2026-10-19 ("proposal ok").** derive the semantics; follow the model; Deviation where the oracle departs.

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

The Task 6.8 review (`task-6.8-review.md`, Minor 3) traces the first item to a parser ordering.
The oracle adds plain names to the front of a DROP, EXPOSE or PROCEDURE EXPOSE list and `(name)`
selectors to the back (`InstructionParser.cpp:4495` and `:4523`), so every selector runs first, in
reverse order. `drop list (list)` drops `X` on the oracle and `LIST` here. The crate runs the list
in source order, as the documentation describes. The `PROCEDURE EXPOSE` handler split above
follows from this ordering, so both can take one ruling.

The Task 6.8 round-2 re-review (`task-6.8-rereview-r2.md`, Minor 1) adds a third. After
`say q.` and `call r >q.`, with `r: procedure; use arg >z.`, the oracle's `.context~variables`
lists the alias under the owner's name (`hasIndex('Z.')` 0, `hasIndex('Q.')` 1), although the
alias is reachable only as `Z.`. The crate lists it as `Z.`. The simple-variable form behaves the
same way. This predates Task 6.8.
