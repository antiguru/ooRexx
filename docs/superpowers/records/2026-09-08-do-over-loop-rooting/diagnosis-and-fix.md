# `DO OVER` bound a dead handle: the diagnosis, and the fix that landed

Closes register row 15 of `docs/superpowers/records/2026-09-07-phase-5i-introspection/`.
Diagnosed and fixed 2026-09-08 from `f8eeff8d1`. Every line marked **Ran** was produced by a
command in this session; **Read** is source that was opened. Nothing here is inferred unless
it says so.

## 1. It was the compiled engine's FLAT loop path

**Ran**, one program, three configurations, under `run_program_collect_every_alloc`:

| configuration | result |
|---|---|
| `Engine::Ir` (the default) | PANIC `dispatch.rs` "a live value" |
| `Engine::TreeWalker` | rc 0 |
| `Engine::Ir` + `REXX_NO_FLAT=1` | rc 0 |

Both paths that resolve the whole loop inside the `DO` clause's own step are safe. Only the
flattened path, where the clause closes and the body's clauses run in the op driver's own
frame, is not.

## 2. The drop site, from a captured backtrace

**Ran** — a backtrace taken inside `RootSet::pop_frame` when it truncated the temporaries stack
from 31 entries to 4:

    RootSet::pop_frame
    Interp::finish_plain_clause      run.rs
    Interp::run_ops_from::<true>     ir/drive.rs
    Interp::run_ops::<true>          ir/drive.rs

`Op::LoopRun` ends its region as soon as `flat_loop_start` answers `FlatStart::Flat`, and the
driver then closes the `DO` clause on either exit. `Interp::over_items` had pushed the loop's
items into that clause's temps frame.

Under it: `Heap::collect` marks from `RootSet::iter` chained with `Heap::immortal` and nothing
else, and `Interp::flat_top`/`flat_loops` are neither. `LoopState::OverItems`' `Vec<ObjRef>`
was invisible to the collector by construction.

**Ran** — liveness at the first pass boundary: `["heap(7,1)=true", "heap(6,8)=false",
"heap(50,0)=false"]`. Two of three index strings already gone, one slot's generation advanced
to 8, so its storage had been reused.

## 3. It reached an ordinary run

The Phase 5i record left this as NOT ATTEMPTED. **Ran**: plain `rexx-run`, no stress mode, no
environment, default engine — panic at rc 101 in 0.032 s. The program is 50 `::METHOD`
directives with 25-byte names, `DO name OVER .methods`, and an inner 1000-pass allocating loop.
The oracle answers `50 / ok`. The threshold is allocation volume, not method count: m=50/k=1000
panics where m=50/k=200 and m=20/k=1000 do not.

## 4. What the fix is

`LoopState::OverItems` now holds one `Body::Array` snapshot plus a cursor, which is the
oracle's own shape — **Read**, `interpreter/instructions/DoBlock.cpp:88` marks `to`, `:144`
reads `((ArrayClass *)to)->get(overIndex)`, and `instructions/DoBlockComponents.cpp:226` carries
the comment `// anchor immediately to protect from GC` before storing the target, replacing it
with the materialised array at `:252`.

Who roots the snapshot differs by engine, and each answer is the one that engine already
relies on:

* tree-walker and the compiled engine's nested path — `over_snapshot`'s own `push_temp`, since
  the whole loop runs inside the `DO` clause's step;
* the flattened path — `flat_loop_start` writes the snapshot into the register the `OVER`
  target was evaluated into. **Read**: `ir/compile.rs` already allocates a loop header's
  registers "in the *enclosing* scope and released past the whole loop, because these registers
  are what roots the header's values while the loop runs", guarded by a `debug_assert_eq!`. The
  register file is a region of the same `temps` the collector walks, so no new root kind and no
  release path was added.

The state also carries the snapshot's slots again as a plain cursor, so a pass costs one vector
index rather than a handle resolution. Every entry of that cursor is a slot of the snapshot, so
the snapshot rooting it is what keeps the cursor sound; a `debug_assert_eq!` at construction
pins exactly that.

## 5. Two corrections this work made to itself

**A prediction that was falsified, and reading the wrong signal.** The control for "is the
register write load-bearing?" was run by disabling it and comparing **exit codes**: everything
still answered rc 0, and the write looked inert. It was not. Comparing **output** instead:
`heavy.rex` printed `109` where it should print `200`. A swept snapshot had made
`array_slots` answer `None`, which the first version of `loop_advance` read as "no more items"
and turned into a quiet early exit — a silent wrong answer where the shape it replaced was a
panic.

The fix separates the three cases so that running out of items is the only quiet exit, and a
snapshot that is not a live array or a slot that is empty are loud. **Ran**, with the register
write disabled after that change: all five witnesses panic at rc 101 with
`a DO OVER's snapshot is a live array for the whole of its loop`, on the default engine, and
answer correctly with it enabled.

**The control also widened the fix's coverage claim.** Before, only the native-table shape died.
Now every `DO OVER` builds a fresh snapshot, so the register root is load-bearing for all five
shapes — array target, `StringTable`, user-class `makeArray`, native table, nested.

## 6. Verification

**Ran** — baseline `f8eeff8d1` built from `git archive HEAD` into its own tree and target
directory, distinct `sha256`, exit codes on an ordinary `rexx-run`:

| program | baseline | fixed |
|---|---|---|
| `heavy` — 200 methods over `.methods` | 101 | 0 (`200 / ok`) |
| `min` — 50 methods, k=1000 | 101 | 0 (`50 / ok`) |
| `nested` — `DO OVER .methods` around `DO OVER .routines` | 101 | 0 (`400 / ok`) |
| `user` — class `makeArray` | 0 | 0 |
| `st` — `StringTable` | 0 | 0 |

`nested` is the witness that a fix must cover `flat_loops` and not only `flat_top`.

**Ran** — all six probe programs answer byte-identically to the oracle on both engines, and the
five witnesses plus the four earlier probe shapes pass under
`run_program_collect_every_alloc` on both engines.

## 7. What it costs

**Ran**, interleaved, three sittings, `/usr/bin/time -f %e`, stdout byte-identical between the
two binaries:

| program | baseline | fixed | oracle |
|---|---|---|---|
| `overentry.rex` — 400,000 `DO OVER` **entries** | 0.12 s | 0.15 s | 0.10 s |
| `overpass.rex` — 20 entries, 2,000,000 **passes** | 0.12 s | 0.13 s | 0.11 s |
| `bench-programs/emptyloop.rex` | 0.53 s | 0.53 s | — |

A loop with no `OVER` is unaffected. The cost is one arena allocation per `DO OVER` entry — the
snapshot — which is what the oracle pays too (`makeArray()` allocates a Rexx array per loop) and
which it absorbs better than we do. **This is an axis where we were already behind the oracle
before the change and are further behind after it**, and the alternative that would not pay it
is recorded in section 8.

## 8. The alternative that was prototyped and not taken

`RootSet::park`/`release` on the item vector: **Ran**, it fixes the same witnesses and the whole
workspace is green against it, at +8% on `overentry` rather than +25%. It was not taken because
it adds a release obligation on five separate paths where a `FlatLoop` leaves service, and a
missed one is over-retention rather than a crash — invisible except to an assertion nobody
writes. The landed fix has no release path at all.

If `DO OVER` entry cost ever shows up in a real workload, a park that *owns* the vector rather
than cloning it would be cheaper than both, and that is the thing to measure first.
