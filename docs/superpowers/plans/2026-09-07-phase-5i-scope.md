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

**Phase 5's own exit gate has never been assessed.** ~~Every closed phase has a
gate document -- `phase-2-gate.md`, `phase-3-gate.md`, `phase-4a` through
`4e`. Phase 5 has only `2026-08-26-phase-5a-gate-close.md`, a sub-phase.~~
**FALSE, corrected 2026-09-08 in the close section below**: that is a universal
over a set nobody enumerated, and `find docs/superpowers -iname '*gate*'` returns
files it does not name, `records/2026-09-07-phase-5h-mapped-collections/gates.md`
among them. **The heading's own claim -- that Phase 5's exit gate has never been
assessed -- survives**, and the enumeration supporting it is in *One correction
to the struck sentence above*.

Phase 5's exit clauses in the parent plan include `CoreClasses.orx` executing, the
64-class wiring set, `RexxInfo` present as an instance rather than a class,
security-manager interception points (D12), cold start measured against C++
(D2), and rung L2. No delivery evidence was found for the last three.

Closing Phase 5 is blocked on that assessment, not on these 133 rows.

---

# What the phase actually cost, written at its close

Added 2026-09-08 by Task 9, against `532cbe2cf`. The sections above are left as
written on 2026-09-07 so that the predictions can be read against the outcome.

## The row count

`corpus/method-bodies.txt`, refreshed at the tip with
`REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies`
and byte-identical to the committed table:

| | at `bc9d991cc` | at `532cbe2cf` |
| --- | --- | --- |
| `loud` | 204 | 67 |
| `answers` | 1058 | 1190 |
| `unanswered` | 71 | 76 |
| `diverge` | 2 | 2 |
| `unstable` | 4 | 4 |
| `uncomparable` | 8 | 8 |

Every row that moved moved one of two ways: 132 `loud` -> `answers`, and five
`loud` -> `unanswered`. **No row went from `answers` to anything**, and the two
`diverge` rows are the same two -- `DateTime~date` and `~timeOfDay`, the
builtin-argument-run defect.

The five are `Pointer`'s instance rows -- `=`, `==`, `\=`, `\==` and `isNull`.
That is the outcome this note predicted when it took Pointer in: `.Pointer~new`
now raises `93.967` on both sides, so the probe's receiver is never built and
the row learns nothing about its own method rather than recording a refusal.

**Every in-scope class ends at zero `loud` except `RexxInfo`, which ends at
two**: `executable` and `libraryPath`, declined because they answer a `.File`
and `File` is Phase 7's. So 137 of the scoped 139 closed and two were declined
with a destination. The 67 that remain are exactly the classes this note put out
of scope -- `Stream`, `RexxQueue`, `Message`, `EventSemaphore`, `MutexSemaphore`,
`File` -- plus those two.

## The predictions

* **"The cheapest first move is `ARG` option `A`: nine rows on one fix" -- HELD.**
  The eight `of` rows closed on it, and `corpus/collection-arity.tsv`'s
  `Properties setLogical` moved from `send-differs` to `agree` in the same
  change. Task 1's own report qualifies it in a way this note did not: the eight
  rows close on the *zero-argument* path, so they are closed rows rather than
  fully exercised bodies.
* **"Pointer's six rows and Buffer's one are the same error site" -- HELD**, and
  the cascade this note predicted is visible in the table above.
* **The row count was right and the instrument was not.** The number 139 was
  taken from an instrument that sends every method with no arguments, and the
  phase then built a second one -- `corpus/introspection-arity.tsv` -- that sends
  an argument list a method could accept. Its verdicts do not reduce to the first
  table's, and neither of them compares the contents of an object-valued answer.
  What that cost is the subject of Task 9's report.

## The blocker this note named is still the blocker

**Phase 5's own exit gate has still never been assessed.** Nothing in Phase 5i
assessed it, and the clauses with no delivery evidence -- security-manager
interception points (D12), cold start measured against C++ (D2), and rung L2 --
are unchanged. Closing **Phase 5** remains blocked on that. Closing **Phase 5i**
does not.

## One correction to the struck sentence above

**"Every closed phase has a gate document" is false**, and so was the first
correction of it, which replaced the universal with a hand-written list that
missed four files. A hand-written list looks like evidence and is only a memory,
so this one is the command and its output, run 2026-09-08 from the repository
root:

```
$ find docs/superpowers -iname '*gate*' | sort
docs/superpowers/plans/2026-08-26-phase-5a-gate-close.md
docs/superpowers/plans/phase-2-gate.md
docs/superpowers/plans/phase-3-gate.md
docs/superpowers/plans/phase-4a-gate.md
docs/superpowers/plans/phase-4b-gate.md
docs/superpowers/plans/phase-4c-gate.md
docs/superpowers/plans/phase-4d-gate.md
docs/superpowers/plans/phase-4e-gate.md
docs/superpowers/plans/ppwizard-gate.md
docs/superpowers/records/2026-07-28-phase-3-parser/gate-report.md
docs/superpowers/records/2026-08-15-phase-5-object-model/review-r2-gate.md
docs/superpowers/records/2026-08-26-phase-5a-gate-close
docs/superpowers/records/2026-08-27-phase-5b/gate-parallelization.md
docs/superpowers/records/2026-09-07-phase-5h-mapped-collections/gates.md
```

**Phase 5h has one** -- `records/2026-09-07-phase-5h-mapped-collections/gates.md`,
opening `# Phase 5h - gate readings`. The other three the hand-written list
missed were opened: the Phase 3 file is an implementer's report pointing at
`plans/phase-3-gate.md`, the 5b one is about parallelizing gate *runs*, and
`records/2026-08-15-phase-5-object-model/review-r2-gate.md` reviews the criteria
the Phase 5 spec proposes rather than assessing whether Phase 5 met them.

**What this note actually rests on survives, narrowed to what the command
supports**: of the files `find` returns, none assesses Phase 5's own exit
clauses. Task 9's report carries the same enumeration and the same narrowing.

## What Phase 5i added to the ledger of things owed

`docs/superpowers/records/2026-09-07-phase-5i-introspection/found-not-fixed-register.md`.
Its D59 row is the one that binds first: D59 was a temporary licence, Task 2
retired its premise by making weak references reachable, and its removal is
owed **after this phase closes and before new work starts**.
