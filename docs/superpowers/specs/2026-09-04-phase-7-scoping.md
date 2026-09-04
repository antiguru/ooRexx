# Phase 7 scoping survey — streams, files, and what actually blocks them

**Measured 2026-09-04 at `fd9bfdcbd`.** Not a spec. This is the survey the spec has to be written
from, and its purpose is to replace the row counts Phase 7 was going to be scoped on with what the
tree and the oracle actually say. Every figure below names the command that produced it.

The short version: **the 82 table C rows are not the work, and two of the three boundaries this
survey was asked to settle move work out of Phase 7 rather than into it.**

---

## 1. Nothing stream-shaped exists yet, and `SAY` does not depend on it

One probe per builtin, oracle and crate, three descriptors read separately, from a fresh directory:

```text
                oracle                     crate
.Stream~new     The Stream class           rc 120  the LIBRARY REXX entry point "stream_init" ...
lineout         wrote                      rc 120  routine "LINEOUT" is not implemented (4c)
linein          x                          rc 120  routine "LINEIN" ...
lines           0                          rc 120  routine "LINES" ...
charout         ok                         rc 120  routine "CHAROUT" ...
stream(...)     (empty)                    rc 120  routine "STREAM" ...
qualify         1                          rc 120  routine "QUALIFY" ...
say 'x'         x                          x       rc 0, agrees
```

**`SAY` is not stream-backed here.** On the oracle it goes through `.OUTPUT` and a program can
prove it -- see D-P7-1, measured after this section was written and no longer open. The answer
there is that the two agree today because every route to observing the difference is itself loud,
so `SAY` stays direct and the stream work is additive.

**The refusal message attributes every excluded builtin to `4c`, and that is wrong for all of
them.** `run.rs:5846` answers an excluded builtin with `Loud::unresolved_call`, whose owner is the
hardcoded `"4c"` at `lib.rs:681`. `phase-4-exclusions.txt` assigns eight of the fifteen to Phase 7's
streams, three to the platform layer and four to RXAPI. Nothing asserts the owner in that message,
so nothing caught it. Whatever Phase 7 implements, it should make these say what they mean.

---

## 2. `Stream` is native; `File` is Rexx **and native**, and is blocked on both

`interpreter/RexxClasses/StreamClasses.orx` defines both, and they are not the same kind of work.

**`Stream` (`:121`) is `EXTERNAL 'LIBRARY REXX stream_*'` throughout** — `stream_init`, `stream_chars`,
`stream_lines`, `stream_position`, `stream_state` and the rest. This is genuine Phase 7 native work
and `dispatch/native.rs` already reserves the entry points for it.

**`File` (`:506`) is ordinary Rexx**, and every one of its methods is unreachable here for a reason
that has nothing to do with files:

```text
.File~new('/tmp')  then any of ~name ~exists ~isDirectory ~absolutePath ~parent ~length ...
  oracle  answers each one
  crate   rc 120  method "LENGTH" of class "MutableBuffer" is not implemented (Phase 5)
```

The chain is `File~init` (`:526`) → `normalizePathSyntax` (`:581`) → `.mutableBuffer~new(path)`
(`:588`). On unix the separator is `/`, so the two Windows branches in that method are dead and the
**reachable** requirement is five methods:

| needed by `File~new` | verdict in `corpus/method-bodies.txt` |
|---|---|
| `MutableBuffer~new` | `loud` (class) |
| `MutableBuffer~length` | `loud` (instance) |
| `MutableBuffer~endsWith` | `loud` (instance) |
| `MutableBuffer~string` | `loud` (instance) |
| `MutableBuffer~delstr` (trailing separator only) | `loud` (instance) |

**CORRECTED 2026-09-04, by the plan review this survey led to. The sentence that stood here said
`File`'s rows are gated on five `MutableBuffer` methods "not on file I/O". That is false: they are
gated on both.** `File~init`'s next line after `normalizePathSyntax` is `self~qualifiedPath`
(`:527`), unconditional on the one-argument path, and `qualifiedPath` (`:637`) calls `qualifyImpl` —
which is `EXTERNAL 'LIBRARY REXX file_qualify'` and `deferred("file_qualify", Family::File)` at
`dispatch/native.rs:276`, owner **Phase 7**. Measured, a `File` subclass overriding `qualifyImpl`
prints from it *before* the constructor returns.

So implementing `MutableBuffer` moves `.File~new('/tmp')`'s refusal from `MutableBuffer LENGTH` to
`file_qualify`; it does not make `File` construct. `SysFileSystem::qualifyStreamName` →
`canonicalizeName` (`platform/unix/SysFileSystem.cpp`) is what stands behind that entry point.

**How the error was made, since it is the same shape twice in one document**: having established
that `File` is Rexx rather than native, I stopped reading at the method that confirmed it. The
constructor's next line was on the screen. §3 records the same habit finding `File`'s rows hollow
only because someone opened the probe.

`MutableBuffer`'s `method-owner` in `class-set.txt` is `5c` — a phase in `CLOSED_PHASES` — and that
part stands.

---

## 3. `File`'s 50 "answering" rows are agreement on a constructor raise

`corpus/method-bodies.txt` reads `File 57 answers / 3 loud`, which contradicts §2 until you open the
probe. `corpus/gate-tables/methods/file__instance.rex` says it in its own header:

> a bare `~new` raises 93.901 on the oracle. So `~new` raises and no line below it is reached; the
> row's evidence is that raise, which is what the row set says there is to have.

`.File~new` with **no argument** raises on both sides, identically, and the fifty `hasMethod` lines
below it never run. The rows are honest about what they measure and they are not evidence that any
`File` method works. `StreamSupplier`'s nine `answers` are the same shape.

**This is the Phase 5c lesson recurring one layer down** — table C's rows were `hasMethod`
readbacks, and a phase was nearly planned on implementing them. Here the *body* table inherits the
same blindness for any class whose zero-argument constructor raises. Whoever writes the Phase 7
plan must not count these rows as progress, in either direction.

---

## 4. 623 loud method rows are owned by a closed phase, and that is by design

Grouping every `loud` row by its class's `method-owner`:

```text
owner 5c                                623 loud rows
owner 7                                  28 loud rows
owner deferred-rexxcontext-stackframes   10 loud rows
```

**This is not a hole in the phase gates, and I checked before writing it down.** The 5d plan states
the rule: *"The gate is one rule: no row may move `loud` → `diverge`. `loud` → `answers` is progress
and is not gated. A count of implemented bodies is **not** a criterion."* The same plan recorded
`MutableBuffer` at `51 loud / 0 reached` before 5d began. The method-body gate is a **drift** gate —
it catches regressions and table drift, and never requires a row to become `answers`.

So phase closure means "nothing regressed and every gated row agrees", not "the documented methods
work". That is a defensible design and it is written down. **It is also why Phase 7 has a
prerequisite that no phase owns.**

The largest holders, for whoever scopes the follow-on: `String` 112, `MutableBuffer` 52, `Queue` 34,
`Package` 33, `CircularQueue` 31, `List` 29, `RexxInfo` 28, `Array` 27, `Stem` 20. Some are recorded
decisions — Task 3 deliberately left `String`'s operator-message rows, Task 5 left `Package`'s — and
some, `MutableBuffer` among them, are simply unbuilt.

---

## 5. The three boundary questions, answered

**`RXFUNCADD` / `RXFUNCDROP` / `RXFUNCQUERY` are not Phase 7's.** D7 in the parent plan is closed:
*"RXAPI daemon | Phase 10 | closed — bridge to the C++ rxapi"*. Three of the fifteen excluded
builtins leave Phase 7 on this answer.

**`Queue`'s ten native entries are not file I/O either.** They are `rexx_create_queue`,
`rexx_open_queue`, `rexx_push_queue`, `rexx_pull_queue` and the rest — the *external* queue API,
which the parent plan's `rexxapi/` row lists under "the separate RXAPI daemon process — macrospace,
external queues, subcom registry", and which `RexxQueue` (`StreamClasses.orx:439`) wraps.
`Family::owner()` in `dispatch/native.rs:139` returns `"Phase 7"` for `Family::Queue`. **That
attribution should be re-examined rather than inherited**; on this reading the entries are gated on
the RXAPI bridge.

**`::REQUIRES LIBRARY` and `::ROUTINE EXTERNAL` are the native library loader**, and they are the
one boundary that stays. Both refuse ahead of any search today and both are table D rows owned by
`7`.

---

## 6. What Phase 7 is actually for, which the row counts hide

D11 is settled as *"RexxUtil / `Sys*` | Phase 7, and L2 | subset in Phase 7, rest in Phase 10"*, and
the parent plan is blunt about why:

> the suite cannot start — not "runs with some failures", cannot start — without `SysFileExists` and
> `.File`, and the framework's own runner additionally needs `SysFileTree`. **`Sys*` blocks L2.** A
> plan that schedules it as a Phase 10 nicety cannot reach its own Phase 5 gate.

**So Phase 7's value is unblocking L2** — running the real ooRexx test suite against this crate —
and the critical path to that is `.File`, which is blocked on five `MutableBuffer` methods **and on
`file_qualify`, a Phase 7 native entry point** (see §2's correction), plus `SysFileExists` and
`SysFileTree`. Stream's twenty-four native entry points are a larger body of work and are still not
on that path, but `File`'s own native entries are.

---

## Open decisions for the spec

* **D-P7-1. Does `SAY` have to go through `.OUTPUT`?** The oracle's does; this crate's does not, and
  they agree today. Whether any observable behaviour separates them — a redirected `.OUTPUT`, a
  `~say` override, `SAY` after a stream error — is **not measured** and is the first thing the spec
  should settle, because the answer decides whether streams are additive or a re-plumbing.
* **D-P7-2. Does Phase 7 take the five `MutableBuffer` methods, or does a 5c follow-up?** They are
  owned by a closed phase, they block the phase's critical path, and nothing currently owns
  implementing them.
* **D-P7-3. Does `Family::Queue` stay Phase 7's?** §5 says the entries are RXAPI-backed. If it
  moves, `dispatch/native.rs:139` and the ten `deferred(...)` rows move with it.
* **D-P7-4. What is the phase's gate?** Table C's `File`/`Stream`/`StreamSupplier` rows cannot be
  it — §3 shows they pass on a constructor raise. A candidate: the method-body table's `File` and
  `Stream` rows moving `loud` → `answers`, plus a corpus program per stream operation compared
  byte for byte, plus `SysFileExists`/`SysFileTree` if L2 is the goal.
* **D-P7-5. Is the excluded-builtin owner message worth fixing here?** §1 — all fifteen say `4c`.
  Cheap, unpoliced, and it misleads exactly the reader trying to scope this phase.

## What this survey did not do

* **No stream semantics were read.** `streamclasses.xml` (51 sections) and the `~open` option
  grammar are unsurveyed; this is a scoping pass, not the spec's content.
* **`SysFileExists` and `SysFileTree` were not probed**, only cited from the parent plan.
* **The `MutableBuffer` five were not costed.** Whether they are an afternoon or a week is unknown;
  `String`'s 112 loud rows suggest the surrounding machinery may already exist, and that was not
  checked.
* **No benchmark**, no gate run — nothing in this document changes code.

---

## Decisions taken, 2026-09-04

Moritz decided D-P7-2 and D-P7-4. The other three were my recommendation, put to him with the
survey and not separately confirmed; they are marked as such so a reader knows which is which.

**D-P7-2 — `MutableBuffer` gets a 5c follow-up of its own, before Phase 7 starts.** Not folded into
Phase 7. Cleaner ownership, and it fixes the class rather than the five methods `File` happens to
need. It costs a plan and gate cycle before any stream work begins, which was the trade taken.

**D-P7-4 — Phase 7's gate is the method-body rows plus corpus programs.** `File` and `Stream` rows
move `loud` → `answers` in `corpus/method-bodies.txt`, and a corpus program per stream operation is
compared byte for byte against the oracle. Table C's rows cannot be the gate for the reason §3
gives. L2 starting was offered and not taken, so `SysFileExists`/`SysFileTree` are not gate
criteria — they stay the phase's purpose without being its exit test.

**D-P7-1 — defer the `SAY`-through-`.OUTPUT` indirection (my recommendation).** Measured, and the
indirection is real: `.output~destination(.stderr)` moves `SAY`'s output to stderr on the oracle,
so `SAY` genuinely goes through the Monitor. But `.OUTPUT` is a `Monitor` and not a `Stream`
(`.output~isA(.Stream)` is `0`; `.STDOUT` is the Stream behind it), and `.OUTPUT`, `.STDOUT`,
`.STDERR` and `Monitor~destination` are all loud here — so **no program can currently observe the
difference**, and a loud refusal is a safe answer rather than a wrong one. Implement `.STDOUT` and
`.STDERR` as Streams, leave `.OUTPUT` and `Monitor` loud, and keep `SAY` direct. Routing `SAY`
through a Monitor send is a hot-path change and parity is a standing goal; it belongs with `.OUTPUT`
whenever that is built, as its own scoped change with its own measurement.

**D-P7-3 — `Family::Queue` moves off Phase 7 (my recommendation).** `dispatch/native.rs:139` returns
`"Phase 7"` for it and the ten entries are the external-queue RXAPI surface.

**D-P7-5 — the excluded-builtin owner message is fixed in Phase 7's first task (my
recommendation)**, with an assertion so it cannot drift back.

---

## What the `MutableBuffer` follow-up is actually for

Measured after the decision, because it changes the shape of that work:

```text
.MutableBuffer~new            oracle: The MutableBuffer class    crate: same, rc 0
say .MutableBuffer~new('abc') oracle: abc                        crate: rc 120 MAKESTRING loud
~length ~string ~endsWith ~delstr   oracle answers               crate: each loud
```

**The constructor is built and the 51 instance methods are hollow**, which is 5c's design rather
than an oversight. `dispatch.rs`'s own doc says why:

> The buffer's contents are not kept, and nothing that would read them answers -- which is what
> keeps the rendering honest, since the oracle renders a buffer as its contents rather than as a
> default name

So the constructor deliberately carries no state, and every reader refuses, so that hollowness is a
declared gap and never a wrong answer. **The follow-up therefore has to give the constructor real
storage first**; it is not a matter of filling in method bodies against state that already exists.
`a_constructor_taking_arguments_answers_an_instance_and_refuses_its_state` asserts the current pair
and will have to change with it.
