# Phase 5j gate — class lifetime

**Spec:** `docs/superpowers/specs/2026-09-09-phase-5j-class-lifetime.md`, criteria in its §7.
**Assessed 2026-09-09.** Toolchain `rustc 1.98.1 (48a229cea 2026-09-01)`, recorded because the
machine moved from 1.98.0 to 1.98.1 during the phase and one gate run that spanned the change was
discarded rather than read.

Every reading below names the command that produced it. A criterion with no command under it is not
assessed, and none here is in that state.

## 1. The four standing gates

```sh
cargo fmt --all --check                                   # 0
cargo clippy --workspace --all-targets -- -D warnings     # 0
cargo test --release --workspace --no-fail-fast           # 0, zero `test result: FAILED` lines
REXX_CORPUS_GATE=1 cargo test --workspace --no-fail-fast  # 0, zero `test result: FAILED` lines
```

## 2. The witnesses agree with the oracle on three descriptors, both engines

Oracle under the standard wrapper from a fresh empty directory; crate via `rexx-run` with
`REXX_ENGINE` set both ways; stdout, stderr and exit status compared separately.

| program | answer, identical on both sides |
|---|---|
| `corpus/lang/class_collected.rex` | `created: 1` / `after drop+gc: 0` |
| `corpus/lang/class_pinned_by_instance.rex` | `1` / `TEMPC` / `0` |
| `corpus/lang/class_uninit_gc.rex` | `before` / `class uninit` / `after` |
| `corpus/lang/weakref_class.rex` | `live class: TEMPC` / `live object: an Object` / `declared: DECL` |

All four are committed programs asserted by `tests/class_lifetime.rs` and
`tests/class_weak_reference.rs`, named in `corpus/unfiled.txt` until `corpus/phase-5j.txt` can
exist — which it cannot before this phase closes, because `gate_table_d.rs` makes a committed
`phase-<id>.txt` oblige `CLOSED_PHASES` to name that phase.

## 3. A negative control for each mechanism, prediction written first

Recorded in this phase's records directory: `task-1-control.md`, `task-3-control.md`,
`task-4-controls.md`. Six mutations in all. Every prediction is marked confirmed, falsified, or
unobservable, and the one falsified prediction — "no other test reddens", where a second test was
already red for an unrelated reason of my own making — says so.

## 4. A derived enumeration of what holds a class

`interp-objref-holders.tsv`, produced by `derive-holders.py`, which reads the struct's own field
declarations and `object_roots`' own destructure so it cannot drift from either. Twenty `Interp`
fields hold an `ObjRef`; three are handed to the collector, and `object_roots`' comment beside each
of the other seventeen says by what route its objects are reachable.

## 5. `run_program_collect_every_alloc` passes the L0 subset

Green, in G3 and G4 above. Thirty-six programs left its zero-collection list during this phase and
none joined it, because declaring a class is now an allocation; all thirty-six were checked to
contain a class-defining construct by reading them.

## 6. Resident set under class accumulation

`task-7-growth.md`. 200,000 `~subclass` calls: 196,186 collected with no forcing, and with
collection driven the resident set is 82,748 kB against the oracle's 116,812 kB, nothing retained.
A control loop allocating the same strings and creating no class is flat, so the growth is
class-attributable.

**Open, and stated rather than hidden:** left to its own schedule the same program peaks at
1,396,296 kB. That is the collection trigger, not a leak — `collect_at` counts arena objects, and a
class is one small object owning two dictionaries that live off the arena where the heuristic
cannot see them. The fix belongs with the collection trigger and is recorded as a follow-on.

## 7. D59a's four consequences

| consequence | status |
|---|---|
| a class's `UNINIT` running later than the oracle's | **fixed**; the licence is withdrawn, not edited |
| `~subclasses` counting a dropped class | **fixed**; `class_collected.rex` |
| a `WeakReference` to a dropped class still answering it | **never reproduced.** The real defect was its inverse — a weak reference to a *live* class read `.NIL` — found while measuring, fixed twice over |
| OOM under class accumulation | **no leak**; see §6 for the residual, which is scheduling |

## 8. What this phase did not settle

* **Package collection.** `Interp::programs` is push-only and the package class tables have no
  removal site, so a `::CLASS` class cannot become unreachable. The oracle pins declared classes
  too, so it is not a parity gap — but it is why the collectable surface is `~subclass` and
  `~mixinClass` only.
* **The collection trigger's blindness to off-arena weight**, §6.
* **A class collected while a `WeakReference` names it** has no witness: the weak reference is not a
  root and clears in the same collection, so the case cannot be separated from the live-class one.
* **Eager collection.** In one shape — no clause between the creation and the `drop` — this crate
  collects a class the oracle does not, visible through `~subclasses` and through a
  `WeakReference`. Same cause as the DEVIATION row that already licenses it, whose scope now says
  so.
