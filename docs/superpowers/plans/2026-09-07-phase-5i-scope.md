# Phase 5i -- scope, decided before any work

Written 2026-09-07, at `bc9d991cc`, deliberately as a SCOPE note and not a
plan. Phase 5 never had one, which is why it crept from 5a to 5h; 5i is the
boundary drawn first, so that what is not in it has somewhere else to be.

Row counts are `corpus/method-bodies.txt` `loud` rows, measured at this
commit: 204 remain of 1347.

## In scope

| group | rows | note |
| --- | --- | --- |
| the introspection surface: `Package` 33, `RexxInfo` 28, `Method` 16, `RexxContext` 14, `Class` 11, `StackFrame` 10, `Routine` 8 | **120** | a program asking the interpreter about its own program -- the one direction that matches the phase's title, *Object model* |
| `of` on the eight mapped classes | 8 | **one cause**: `ARG` option `"A"` answers an Array, unimplemented. Also unblocks `Properties setLogical`, the last non-stream `send-differs` row in `collection-arity.tsv` |
| `Object`: `instanceMethod`, `instanceMethods`, `isInstanceOf` | 3 | |
| `Buffer~new`, `WeakReference~value` | 2 | |

**133 rows**, plus the two arity rows that come with the `ARG` fix.

The cheapest first move is `ARG` option `"A"`: nine rows on one fix, and it is
the only one of these with a measured single cause.

## Out of scope, with a destination

* **`EventSemaphore` 4 and `MutexSemaphore` 2 -> Phase 6.** Decided by Moritz
  2026-09-07: defer rather than pull in. The crate's own refusals tag them
  Phase 5, but Phase 6 is *Concurrency* and its exit gate names activities,
  the kernel lock, guard locks, `REPLY` and `GUARD`. Whether `post`,
  `isPosted` and `reset` could work without any of that is UNMEASURED and
  deliberately so -- the answer would not change where they belong.
* **`Stream` 25, `File` 3 -> Phase 7** (*Streams & platform*). Measured: the
  refusal is the `stream_uninit` LIBRARY entry point, which names Phase 7
  itself.
* **`RexxQueue` 14 -> Phase 7.** Measured: `rexx_query_queue`, and spec D93
  already assigns it. Also gated on **D7, which is reopened** -- see
  `2026-07-27-rust-rewrite.md`'s D7 block. Three items share that one
  dependency: `RXQUEUE` (a wholly excluded builtin), cross-process `QUEUED`
  (a partial exclusion), and this class.

## Two open questions, to settle before the plan and not during it

* **`Pointer` 6.** Moritz said "maybe". The crate tags it Phase 5; Phase 8 is
  *Native API* and `Pointer` is one of its value types. Five of its six rows
  cascade from `Pointer~new`, so it is one decision and not six.
* **`Message` 17.** Introspection-shaped, and it sits in the same cluster --
  but Phase 6's exit gate explicitly lists "message objects". Counted OUT of
  the 133 above until this is settled.

Both are instances of the same thing: **the phase tag in the crate's refusal
and the phase title in the plan disagree for about 30 of the 204 rows.** That
disagreement is a symptom of Phase 5 having had no scope, so anything
object-shaped was tagged 5. Settling these two settles the pattern.

## Prerequisite, learned the hard way

**Extend `corpus/collection-arity.tsv`'s instrument to the introspection
classes before sizing this.** It covers the collections only, so the
introspection cluster currently has no arity row anywhere -- and gate table
C's method rows are `hasMethod` readbacks, which is how a phase once got
planned around rows that were already answering. `method-bodies.txt` is the
instrument that runs them, and it is the one the counts above come from.

## Not this phase, and not any sub-phase

**Phase 5's own exit gate has never been assessed.** Every closed phase has a
gate document -- `phase-2-gate.md`, `phase-3-gate.md`, `phase-4a` through
`4e`. Phase 5 has only `2026-08-26-phase-5a-gate-close.md`, a sub-phase.
Its exit clauses in the parent plan include `CoreClasses.orx` executing, the
64-class wiring set, `RexxInfo` present as an instance rather than a class,
security-manager interception points (D12), cold start measured against C++
(D2), and rung L2. No delivery evidence was found for the last three.

Closing Phase 5 is blocked on that assessment, not on these 133 rows.
