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
| `Pointer`, added by the decision below | 6 | one error site, `93.967`, shared with `Buffer~new` |

**139 rows** -- the 133 this note was written with, plus Pointer's 6 once the
question below was settled -- and with them the two arity rows that come with
the `ARG` fix.

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

## The two open questions, settled 2026-09-07

Both are now decided by Moritz, and the sections below record what they were
decided on rather than only which way they went.

* **`Pointer` 6 -> IN.** Measured on the oracle the day of the decision:
  `.Pointer~new` raises `93.967  NEW method is not supported for the Pointer
  class`, and `.Buffer~new` raises the same error with `Buffer` in it. So
  Pointer's six rows and Buffer's one are **the same error site**, and Buffer
  was already in scope. Five of the six cascade from `new` because
  `class-set.txt` gives Pointer no construction expression, so the instance
  arm's receiver is what `new` refuses to build. Phase 8 owns making a
  Pointer that holds something; 5i owns only the documented refusal, and the
  phase's handover says so.
* **`Message` 17 -> OUT, to Phase 6.** The parent plan's Phase 6 exit gate
  names "message objects" verbatim (`2026-07-27-rust-rewrite.md`'s phase
  table, row 6). Eight of the seventeen -- `start`, `startWith`, `reply`,
  `replyWith`, `wait`, `notify`, `halt`, `messageComplete` -- are concurrency
  whatever else is true, so taking the class here would split it across two
  phases, which is the creep this note exists to stop.

**Phase 5i is therefore 139 rows**, the 133 above plus Pointer's 6.

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
