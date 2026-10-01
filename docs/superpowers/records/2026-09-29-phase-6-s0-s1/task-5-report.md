# Task 5 report: per-activity roots

Status: DONE

## Commits

- `50cce849c` Split the root set into interpreter and per-activity parts
- `5ecf22e37` Record Task 5's instruction counts (`docs/superpowers/plans/phase-6-perf.md` `## Task 5`)

## The split

`rexx-core/src/roots.rs`:

- `RootSet { globals, cells, activity: ActivityRoots }` -- the running activity's part inline (P13),
  reached by `RootSet::activity()` / `activity_mut()` (`#[inline(always)]`, zero loads).
- `pub struct ActivityRoots { temps, frames: Rc<FrameArena>, slots, aliases, alias_count,
  frame_starts, parked, parked_free }`, fields moved with their doc comments verbatim; exported from
  `rexx_core`. Each `ActivityRoots::new` creates its own `FrameArena` (one arena per activity).
- Moved to `ActivityRoots` verbatim (bodies and comments): `park`, `release`, `live_parked`,
  `push_frame`, `pop_frame`, `push_temp`, `frames`, `set_frame_block`, `temps_len`, `push_slots`,
  `live_frames`, `frame_len`, `frame_aliases`, `pop_slots` (LIFO assert), `slot_ref`,
  `take_frame_aliases`, `put_frame_aliases`, `alias_slot`, `resolve`, `resolve_aliased`,
  `grow_slots` (LIFO assert). New `ActivityRoots::iter` (temps, registers, slots, parked) and
  `Default`.
- Stay on `RootSet`, because an alias can reach a cell: `promote`, `slot_value`, `set_slot_value`,
  `frame_slot`, `set_frame_slot`, `clear_frame_slot`, `at`, `write`; bodies changed only in
  `self.x` -> `self.activity.x` for moved fields.
- `RootSet::iter` = globals, cells, then `activity.iter()`.
- Call sites: `roots.X(` -> `roots.activity_mut().X(` / `roots.activity().X(` by sed over `crates/`
  (rexx-exec, rexx-api tests, rexx-core tests), then rustfmt. The compiler found two borrow
  conflicts, both in `rexx-core/tests/roots.rs` (`alias_slot(.., roots.slot_ref(..))`), hoisted into a
  `let`. Six source comments naming `RootSet::{temps_len,grow_slots,park,live_frames}` now name
  `ActivityRoots::`; `SlotFrame`'s doc names `ActivityRoots`. Historical records under `docs/` left.
- `frame.rs` module doc: "Each activity owns one arena (D-U4)", properties hold "for each arena",
  LIFO a logical property per arena (out-of-order or cross-arena release trips the debug assertion,
  at worst mis-scans roots), the `FrameArena` value may move, its blocks never do, no `RegFrame` live
  across a move. Grant comment: "Granted by Moritz, 2026-09-23, re-granted per activity 2026-09-29
  (D-U4)". The arena type is unchanged. `unsafe_sites.rs` unchanged (the grant sites did not change).

## Miri

`R` = `scratchpad/surface-4/rustup-home` (nightly default, `miri 0.1.0 (f7575a9da8 2026-09-24)`).
Each over a `git archive` copy in `$S/<N>-tree`, Stacked Borrows (Miri's default, no `MIRIFLAGS`):

```
cd $S/<N>-tree/rust && RUSTUP_AUTO_INSTALL=0 RUSTUP_HOME=$R CARGO_TARGET_DIR=$S/<N>-target \
  cargo +nightly miri test -p rexx-core --lib --offline
```

| N | commit | exit | result line |
|---|---|---|---|
| miri-base | 0993b9457 | 0 | `test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 40.94s` |
| miri-r1 | 50cce849c | 0 | `test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 40.93s` |

The 17 include the six `frame::tests`.

Integration tests naming `RootSet` or the frame arena (`tests/roots.rs`, `collect.rs`, `heap.rs`,
`uninit.rs`; the others in `rexx-core/tests/` name neither, and `unsafe_sites.rs` is a source
scanner), on a `git archive` copy of 50cce849c, one run per `T`:

```
cd $S/miri-r1-tree/rust && RUSTUP_AUTO_INSTALL=0 RUSTUP_HOME=$R CARGO_TARGET_DIR=$S/miri-r1-target \
  timeout 20m cargo +nightly miri test -p rexx-core --offline --test T
```

| T | exit | result line |
|---|---|---|
| roots | 0 | `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.06s` |
| collect | 0 | `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.51s` |
| heap | 0 | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.83s` |
| uninit | 0 | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.08s` |

Each log's `Running` line names a binary under `miri-r1-target/miri/`, so these ran under Miri; the
pass counts equal the native `cargo test -p rexx-core` counts for the same files. None failed, so the
base was not run.

Incident: a first attempt ran `cargo miri` without `+nightly`. The trees' `rust-toolchain.toml` pins
`stable`, so rustup auto-installed `stable` into `R`, then failed (no miri for stable; both exit 1).
I uninstalled that `stable` toolchain from `R` (`rustup toolchain list` under `R` again shows only
`nightly-x86_64-unknown-linux-gnu (active, default)`) and reran with `+nightly` and
`RUSTUP_AUTO_INSTALL=0`.

## Performance

One round. base 1754a3b5a vs r1 50cce849c, recipe and full table in phase-6-perf.md `## Task 5`.
`Compiling rexx-exec` 1 for each build. Every program within +/-0.0006% except heapshape -0.1058%
(timed program, outside the zero band; a decrease). rexxcps -0.0004%. Largest increase +196 Ir
(sayloop, +0.0002%; startup +0.0003%). Running total inside budget.

## Gates

At 50cce849c, `status.txt` verbatim:

```
50cce849c37b3cd68dc0375fc5528f780e1838e0
started 2026-09-29T23:55:33+02:00
load at start 2.99 2.78 5.84 2/1609 132734
G1 fmt exit 0
G2 clippy(empty target) exit 0
G3 release build (test --no-run) exit 0
load G4 10.52 9.26 7.98 1/1608 137768 2026-09-29T23:58:38+02:00
G4 release test exit 0
G4 Compiling lines: 0
load after G4 12.81 17.01 12.85 7/1622 268037
G5 debug build (test --no-run) exit 0
load G6 10.94 15.99 12.77 5/1625 273485 2026-09-30T00:06:53+02:00
G6 debug test exit 0
G6 Compiling lines: 0
load after G6 1.65 7.73 10.27 1/1590 402483
G7 clippy --features pinning exit 0
G8 pinning self-tests exit 0
50cce849c37b3cd68dc0375fc5528f780e1838e0
finished 2026-09-30T00:13:35+02:00
```

Test totals from the logs: G4 2783 passed 0 failed, G6 2784 passed 0 failed.

## Concerns

1. Miri covers `rexx-core --lib` and the integration tests above; rexx-exec (the call sites) was not run under Miri.
2. Root order handed to the collector changed: cells now come before temps/registers/slots/parked
   (were after slots). Mark order is not observable in a mark-sweep; G4/G6 (collect-stress corpus
   included) green.
3. `ActivityRoots` is not swappable yet (no `mem::swap` entry point, no second instance): no API was
   added for a later stage. `frames` stays `Rc<FrameArena>`, so moving an `ActivityRoots` moves only
   the handle.
