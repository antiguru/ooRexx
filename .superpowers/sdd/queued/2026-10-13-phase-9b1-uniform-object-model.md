**CLOSED: Phase 9b1.** phase-9b1-gate.md: every leak fixed or licensed, SysStemSort as Deviation 37, the addRoutine copy as L10.

# Phase 9b1: a uniform object model (representation never observable)

Moritz 2026-10-13: queue as a new Phase 9b1, after 9b and before 9c, "because we don't want to do two
tasks at once". Justified as performance work, approached in a principled way. The principle itself
is in rust/CLAUDE.md ("Specialization is internal").

## Known leaks (a scout completes the list)

* Strings and small integers have no identity: Deviation 31 (OBJECTNAME= refused). Hardest case: a
  value held inline in a variable slot cannot be materialized from inside a method call on it.
  Needs reference semantics for strings, or a materialized-on-demand value with a tag.
* Package state lives in per-ProgramId side maps on Interp: fragile `Package~copy`, the REXX package
  uncopyable, copies never collected (`2026-10-12-package-as-real-object.md`).
* Activity-scoped state the oracle keeps per activation: clock, RANDOM seed, SETLOCAL (memory
  "Activity vs activation scope").
* Refusals that are representation leaks: "a package object this crate did not build", the native
  routine frame's source-less Routine, the REXX package copy.
* Class weight invisible to the collector (`2026-10-12-gc-trigger-blind-to-class-weight.md`).

## Approach

1. Re-judge every Deviation against the principle (Deviation 31 first).
2. Read-only inventory scout. Leaks hide in: refusals saying "this crate does not keep/build X",
   Deviations, side maps keyed by an id instead of an ObjRef, fast paths that assume a plain kind.
   Per leak: the observable, the programs and ooTest tests that reach it, and what the specialized
   form buys today (measured Ir).
3. Design spec: a uniform object model with materialization as the one escape hatch from every
   specialized form. Open decision: string identity follows the oracle (per-package literal
   interning) or our own rule (every value materializable, identity stable once materialized).
4. Implement as the 9b1 sub-phase, every step measured against the pin with callgrind
   whole-process Ir; nothing lands over budget. Package-as-object and the side-table sweep land here.

Must land before 9c: the rexx/rexxc front ends and Phase 10's embedding handles need uniform
identity.

## Added 2026-10-13 (9b Task 3 review)

SysStemSort writes results back by tail; the oracle writes into the stem elements it captured. A
STRING method that empties the stem mid-sort makes the oracle print [S.1] [S.2] and the crate [K]
[heap...]. Fix needs stem element identity. See .superpowers/sdd/2026-10-13-phase-9b/task-3-review.md.

## Added 2026-10-13 (9b Task 9 fix round)

~addRoutine of a COPY of an internal-row Routine object answers 88.914 where the oracle adds it: a copy
carries no directive/row record. Exclusions row "addRoutine REFUSES A LIBRARY Routine OBJECT" names
it; probe add2.rex in docs/superpowers/records/2026-10-13-phase-9b/task-9/.
