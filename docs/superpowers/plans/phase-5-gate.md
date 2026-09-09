# Phase 5 gate — the object model

**Assessed 2026-09-09 at `baa8c0057`**, toolchain `rustc 1.98.1 (48a229cea 2026-09-01)`.

Phase 5 ran without a gate document. This is it, written at the close rather than at the start,
which is itself worth recording: every criterion below was chosen after the work, so none of it
constrained the work while it was being done.

## The criterion

Moritz, 2026-09-09:

> *"We never wrote a phase 5 gate, and the whole phase is 'object model', which isn't specific.
> We've done abstract work to enable the object model and work to build object infra in the same
> phase, and I think the close is no more known gaps in the area."*

and, on how phases actually behave here:

> *"even in phase 5 we noticed that phase 4 left gaps we had to close later, so it's not an exact
> science."*

**"No more known gaps" is a criterion that can be met by not looking.** So it is assessed against
enumerations that are *derived and run*, not against prose. What follows names the instrument, its
reading, and what it cannot see.

## Instrument 1 — gate table C, the concept and class surface

1488 rows: 21 concept, 63 class wiring, 57 hierarchy edge, 1347 method. Row sets derived from the
shipped documentation; verdict from running each probe on both engines against the C++ oracle in a
subprocess, all three descriptors compared.

```
agree: 1378   unanswered: 110   loud: 24

by owning phase -- rows, and rows not yet `agree`:
  5a: 135 rows, 0 not yet `agree`
  5b: 6 rows, 0 not yet `agree`
  5c: 1225 rows, 0 not yet `agree`
  6: 13 rows, 13 not yet `agree`
  7: 94 rows, 82 not yet `agree`
  deferred-rexxcontext-stackframes: 10 rows, 10 not yet `agree`
  never-expected-to-agree: 5 rows, 5 not yet `agree`
```

Every open row is owned by Phase 6, Phase 7, or a named deferral. The 24 loud rows all wait on one
thing, the `stream_uninit` library entry point.

## Instrument 2 — gate table D, the directive and option surface

79 rows, same method.

```
agree: 75   diverge-both: 4   loud: 4

  5a: 36 rows, 0 not yet `agree`     5b: 2 rows, 0 not yet `agree`
  5c: 36 rows, 0 not yet `agree`     5d: 1 rows, 0 not yet `agree`
  7: 2 rows, 2 not yet `agree`
  deferred-parse-error-rendering: 2 rows, 2 not yet `agree`
```

Both tables end with `gated by this run: 0 row(s) whose owning phase is closing or closed and whose
verdict is not agree`.

## Instrument 3 — the method-body table, which is the behaviour one

1347 rows, one per documented (class, method, arm), classified by **sending** the name to a real
receiver rather than by asking whether the name exists.

```
answers: 1192   unanswered: 76   loud: 67   uncomparable: 8   unstable: 4   diverge: 0
```

**The `diverge` verdict is empty.** It held two rows this morning, `DateTime~date` and
`DateTime~timeOfDay`; chasing them found a defect in the builtin argument path — a message send in
a builtin's argument position corrupted the call — which is fixed at `baa8c0057` with its own
witness. That defect was not an object-model gap, and no Phase 5 row was about it.

The 67 loud rows by class: Stream 25, Message 17, RexxQueue 14, EventSemaphore 4, File 3, RexxInfo
2, MutexSemaphore 2 — streams, queues and files to Phase 7, messages and semaphores to Phase 6.

## What these instruments cannot see, stated rather than left to be discovered

**Table C's 1347 method rows assert presence, not behaviour.** A row is `o~hasMethod("isAbstract")`.
So the 1225 rows owned by 5c say the documented names are in the dictionaries, not that the methods
work. Instrument 3 covers the same names by sending them, which is why it is here and why the close
does not rest on table C alone. It was checked by reading a probe, not assumed.

**The attribution is coarser than the phase list.** The owning-phase values are only `5a`, `5b`,
`5c`, `5d`, `6`, `7` and named deferrals. **No row is owned by 5e, 5f, 5g, 5h, 5i or 5j**, so the
work of those six sub-phases sits inside 5c's 1225 rows and cannot be read off per phase. The
tables evidence the surface, not the phase boundaries.

**`CLOSED_PHASES` names `5a`, `5b`, `5c`, `5d` only**, so the enforcement that a closed phase owns
no open row does not currently bind 5e–5j. It binds nothing extra today, because no row is owned by
them — but the guard is narrower than the phase list.

**"Known" is the load-bearing word.** Instrument 3 sends each documented name *with no arguments*,
so for a method needing arguments an `answers` verdict is agreement about an arity error, not about
a result. A method that takes arguments and computes the wrong answer from them is invisible to all
three instruments. That is the shape of gap this gate cannot exclude, and the differential corpus is
the only thing that reaches it.

## Reading

**On the criterion as stated, Phase 5 can close.** Every gate-table row owned by a Phase 5
sub-phase is `agree`, the behaviour table has no divergence, and every remaining gap is owned by
Phase 6, Phase 7, or a named deferral.

**With the caveat that the criterion is weaker than it sounds**, for the reason in the last
paragraph above, and with Moritz's own observation standing: Phase 4 left gaps that Phase 5 closed,
and the defect fixed at `baa8c0057` is another one — found in Phase 5's own closing assessment, and
belonging to Phase 4's surface. The honest expectation is that Phase 5 will be found to have left
gaps too.

## Standing gates at this commit

```sh
cargo fmt --all --check                                   # 0
cargo clippy --workspace --all-targets -- -D warnings     # 0
cargo test --release --workspace --no-fail-fast           # 0, 118 `ok` result lines, 0 FAILED
REXX_CORPUS_GATE=1 cargo test --workspace --no-fail-fast  # 0, 118 `ok` result lines, 0 FAILED
```

The `ok` counts are recorded beside the exit statuses on purpose: a run that never starts reports
zero failures, and that happened once during this phase.
