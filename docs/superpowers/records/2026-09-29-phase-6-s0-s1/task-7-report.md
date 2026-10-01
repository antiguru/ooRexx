# Task 7 report: activation-relative slot positions, slots per activation segment

Status: DONE_WITH_CONCERNS

Base: 8b253ef97

## Commits

- `31a394482` Give each activation its own slot segment, addressed by identity
- `a0a82f2b7` Keep slot clearing and native frame naming off the call path (perf round 2)
- `863d147d1` Record Task 7's instruction counts and wall clock (`phase-6-perf.md`, "## Task 7")

## Failing test, before and after

`rust/crates/rexx-exec/tests/outer_context.rs` now runs `o9b.rex`, `o9c.rex` (Phase 8 Task 9 probes)
and the new `rust/crates/rexx-exec/tests/outer_context/o9d.rex` against the oracle on all three
descriptors, plainly and under `run_program_collect_every_alloc`. `o9d.rex` sets and reads unbound
simple, stem and compound names (one tail resolved through the outer caller's `X`, one through an
unbound tail symbol), drops an unbound simple name, reads the pool with `GetAllContextVariables`,
then in the outer program reads them back, runs `INTERPRET` over one and passes it to a
`PROCEDURE EXPOSE`; the method above grows its own frame with `INTERPRET` after the outer frame grew.

Before (at 8b253ef97 plus the test edit), `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test outer_context`:

```
assertion `left == right` failed: .../task-9-probes/o9c.rex, collecting at every allocation: false
  left: ("get null\n", "rexx-exec: CallContextInterface.SetContextVariable through a kept outer call context, of a variable its activation has not bound, is not implemented (Phase 6)\n", 120)
 right: ("get null\nset\nret\nafter VAR LIT\n", "", 0)
```

After: `test a_kept_outer_context_reaches_its_callers_variables ... ok` (debug, and in G4/G6).

Oracle for `o9d.rex` (run from an empty dir, `( ulimit -v 1048576; LD_LIBRARY_PATH=<oracle lib>:<forge> rexx o9d.rex )`): rc 0, empty stderr, stdout

```
get null T. null main
set set set dropped set set
get2 made tdef u7 null kx
set set
all 7 again two main
inner ra rb rc ra rb LIT LIT
ret
after again VAR tdef tdef VAR u7 LIT LIT
tails kx K.MAIN kn LIT two
interp again 3
proc proc-set from-proc again
```

The test compares against the live oracle (gate-only), so the expected output is not committed as a file.

## Position representation

- `SlotFrame { depth, serial }`: `serial` is minted by `RootSet::push_slots` from a counter in the
  interpreter part of the `RootSet` (unique across activities); `depth` indexes the frame's record
  in its activity's `segments`. The record (`Segment { start, serial }`) is the only holder of the
  segment's start. Every slow-path resolution goes through `ActivityRoots::segment`, which
  debug-asserts the record's serial equals the handle's (test
  `a_closed_frames_handle_is_refused_by_its_successor`).
- `SlotRef(Target)`, `Target::Slot { frame: SlotFrame, index }` or `Target::Cell(index)`; alias
  entries hold `Target`. The "targets descend" debug assertion is kept as
  `(to.depth, to_index) < (frame.depth, index)`.
- Storage stays one arena `Vec`; each open frame owns the segment from its record's start to the
  next record's. `grow_slots_of(frame, n)` works below the top: it splices `n` slots at the
  segment's end and moves the records above (`above.start += n`). `aliases` is only extended when
  an alias is made.
- Fast path: `frame_slot`/`set_frame_slot`/`clear_frame_slot` read `slots[top_start + index]`
  while `indirect == 0`. `indirect` counts `Some` alias entries plus open `begin_indirect`s
  (`CallerSwap` opens one for its lifetime). The precondition "frame is the top one or an
  indirection is open" is a debug assertion that also checks the cached `top_start` against the
  record (test `a_lower_frame_is_refused_by_the_top_accessors`). `frame_slot_of`/`set_frame_slot_of`/
  `clear_frame_slot_of` resolve any open frame through its record; `Interp::variable_in` (a
  suspended activation's read) uses it.
- Native frame token: `NativeFrame::id`, a per-activity wrapping `u32` (round 1 minted an
  `ActivationId`, which grew `NativeFrame` from 256 to 264 bytes and cost extcall +0.98%), handed to rexx-api as
  `u64` (`Activation::set_frame`, `Surface::in_caller`); `suspended_caller` finds the frame by
  that id instead of indexing `native_handles` by row.
- The per-segment-`Vec` design (each frame its own `Vec`) was built first and measured at
  assign +6.19%, varlookup +4.74%, emptyloop +3.51% (callgrind, one round) and dropped.

## Refusal removal (three descriptors)

- Message: `refuse_unbound_outer`, `unbound_outer`, `governing`, the `outer_caller` flag and the
  outer-only branches of `context_variable`/`compound_parts` are gone
  (`dispatch/library/surface.rs`, `activity.rs`).
- Exclusions row: `docs/superpowers/plans/phase-4-exclusions.txt`, the "A BLOCKING MEMBER ON AN
  OUTER CONTEXT" row's STILL DIFFERS / OWNER: Phase 6 paragraph replaced by a FIXED 2026-09-30
  note; the sentence naming o9c.rex "for the refusal below" corrected.
- `refusal-sites.tsv`: the refusal was a `crate::Loud { message }` literal, not a constructor, so
  it had no row; `refusal_sites.rs` re-derives the table and passed unchanged (debug run and G4/G6).

## Miri

Over `git archive` copies, `RUSTUP_AUTO_INSTALL=0 RUSTUP_HOME=.../surface-4/rustup-home
CARGO_TARGET_DIR=<own> cargo +nightly miri test -p rexx-core --offline <sel>`, each log showing
`Compiling rexx-core` from the archived tree:

| selection | 31a394482 | a0a82f2b7 |
|---|---|---|
| `--lib` | 17 passed, exit 0 | 17 passed, exit 0 |
| `--test roots` | 17 passed, exit 0 | 17 passed, exit 0 |
| `--test collect` | 14 passed, exit 0 | 14 passed, exit 0 |
| `--test heap` | 5 passed, exit 0 | 5 passed, exit 0 |
| `--test uninit` | 7 passed, exit 0 | 7 passed, exit 0 |

The debug-only `should_panic` tests run under Miri (debug build).

Mutation check of the new rexx-core tests (in the archived tree, `cargo test -p rexx-core --test
roots --test collect`): dropping the record shift in `grow_slots_of` reddens
`growing_a_frame_below_the_top_keeps_every_frames_values` and
`a_frame_beneath_another_grows_and_aliases_into_it_hold`; appending instead of inserting reddens
the same two; blinding the serial check reddens `a_closed_frames_handle_is_refused_by_its_successor`.

## Performance

Recipe and full tables: `docs/superpowers/plans/phase-6-perf.md` "## Task 7" (commit 863d147d1).
Budget: running total against 1754a3b5a at most +0.3% beyond each program's noise band.

- Rejected design (per-frame `Vec`, uncommitted): assign +6.19%, varlookup +4.74%, emptyloop +3.51%.
- Round 1, `31a394482`: over budget on extcall +0.9802%, heapshape +0.5677%, parse +0.3525%.
- Round 2, `a0a82f2b7`: every program inside budget. extcall +0.1469% is the only positive
  delta; every other program is negative (startup -0.0029% the smallest).
  assign -1.0090%, nop -2.0996%, fibcall -0.9538%, sendloop -1.0439%, rexxcps -0.2789%.

Wall clock (two runs of `wallclock.sh -r 5`, the second with an identical-binary control `base2`):
run 1 has decloop +8.54% and startup +4.17%; run 2 has decloop +2.41%, startup -7.69%, heapshape
+4.37% (run 1 +3.10%). decloop's +8.54% exceeds its +6.10% band in run 1 only; heapshape's +4.37%
exceeds ±4% in run 2 only, its Ir is -0.4104%, and it is a TIMED program. Everything else is inside
±4% or its band in both runs, or faster.

## Gates (status.txt verbatim)

Round 1 (`31a394482`):

```
31a394482bfd925eaa9fb817a2ca48c133e16067
started 2026-09-30T02:31:10+02:00
load at start 1.02 3.71 3.82 2/1576 1023911
G1 fmt exit 0
G2 clippy(empty target) exit 0
G3 release build (test --no-run) exit 0
load G4 14.07 12.95 7.99 15/1672 1032542 2026-09-30T02:35:55+02:00
G4 release test exit 0
G4 Compiling lines: 0
load after G4 13.75 15.61 11.11 14/1653 1161915
G5 debug build (test --no-run) exit 0
load G6 14.13 15.84 11.81 13/1654 1168638 2026-09-30T02:44:00+02:00
G6 debug test exit 0
G6 Compiling lines: 0
load after G6 2.76 10.79 11.46 1/1609 1298402
G7 clippy --features pinning exit 0
G8 pinning self-tests exit 0
31a394482bfd925eaa9fb817a2ca48c133e16067
finished 2026-09-30T02:50:58+02:00
```

Round 2 (`a0a82f2b7`, the code under this report):

```
a0a82f2b7a5574d6b2077e10b277d9ea744982c0
started 2026-09-30T03:12:48+02:00
load at start 5.17 3.75 5.13 3/1596 1315954
G1 fmt exit 0
G2 clippy(empty target) exit 0
G3 release build (test --no-run) exit 0
load G4 13.93 12.89 8.95 13/1650 1321384 2026-09-30T03:17:31+02:00
G4 release test exit 0
G4 Compiling lines: 0
load after G4 13.75 15.96 11.97 13/1641 1450789
G5 debug build (test --no-run) exit 0
load G6 14.35 16.11 12.53 13/1666 1457173 2026-09-30T03:25:26+02:00
G6 debug test exit 0
G6 Compiling lines: 0
load after G6 11.66 16.36 14.28 8/1633 1587881
G7 clippy --features pinning exit 0
G8 pinning self-tests exit 0
a0a82f2b7a5574d6b2077e10b277d9ea744982c0
finished 2026-09-30T03:32:18+02:00
```

`863d147d1` adds only `phase-6-perf.md` and was not gated.

## Concerns

1. **Fast-path precondition is a debug assertion.** `frame_slot`/`set_frame_slot`/`clear_frame_slot`
   read the top segment without consulting the handle while `indirect == 0`. A caller passing a
   frame that is not the top one, outside a `begin_indirect`, reads the wrong frame in release; in
   debug it panics ("addressed as the top one"). The full debug gate (G6) is green, and
   `variable_in` was the one caller that needed `frame_slot_of`. A future caller addressing a
   parked or lower frame must use the `_of` accessors or open an indirection.
2. **`depth` is part of the handle.** `SlotFrame` carries the record's index for O(1) lookup; the
   serial is checked against it in debug only. A REPLY that moves a segment to another activity's
   records must rewrite the moved activation's `frame` (and any alias target into it; the existing
   `park_reply` assertion says method frames hold none).
3. **Native frame id wraps at u32.** A kept context used after 2^32 further native calls in the
   same activity could name a live frame with the same number. The row index it replaces was
   reused immediately.
4. **Unrelated divergence found, not fixed:** `drop w.1` with `W.` never assigned: oracle leaves
   `symbol('W.')` = `VAR` (it creates the stem), this crate answers `LIT`, in plain Rexx and through
   `DropContextVariable`. Measured with a debug build of this task's first (per-frame `Vec`)
   prototype; `stem_drop_tail` is not touched by this task, and I did not re-run it on the base
   binary. Not in the exclusions file; I left it
   out of `o9d.rex`.
5. **Wall clock:** decloop +8.54% (run 1 only) and heapshape +4.37% (run 2 only) past their bars,
   each inside in the other run, with Ir at -0.36% and -0.41%.
6. I ran `git checkout -p -- crates/rexx-exec/src/variables.rs </dev/null` once to revert a
   one-line experiment (forbidden form). With no input it changed nothing; `git diff` afterwards
   showed only the intended round-2 edits, and the experiment line was removed by hand.

## Fix round 1

Status: DONE_WITH_CONCERNS (extcall over budget after round 3 of 3; reported for a ruling).

Commits: `d112d68f9` (I1, I2, M1, M2), `8d26923bb` (M3, `phase-6-perf.md` "### Round 3").

- **I1.** The fast accessors take the fast path only when `frame.serial == fast_serial`, where
  `fast_serial` is the top record's serial while `alias_count` is zero and `u64::MAX` otherwise
  (kept by push, pop and `set_alias`). A mismatch falls through to the `_of` path, so any open
  frame is served correctly with no precondition. `begin_indirect`/`end_indirect` are gone (the
  counter is `alias_count` again) and `CallerSwap` no longer opens one. `segment()`'s serial check
  is `assert_eq!`. Tests: `the_top_accessors_reach_a_lower_frame` (replaces the debug-only
  should_panic), `a_closed_frames_handle_is_refused_by_its_successor` (now in release too, through
  `frame_slot`).
  The ruled `alias_count == 0 && frame.serial == top_serial` form was measured first: small assign
  +4.7% vs t7b (sete/test/je per access); folding the alias check into `fast_serial` brought it to
  one extra load per access.
- **Assert cost** (ruling I1): measured on the `&&` variant, `assert_eq!` against
  `debug_assert_eq!` in `segment()`, otherwise identical, `$SC/small/run.sh` (callgrind, one run,
  reduced-iteration copies of the bench programs): assign +11,682 Ir (+0.002%), varlookup +391,959
  (+0.09%), emptyloop +492,208 (+0.16%), fibcall +450,245 (+0.10%), sendloop +1,591,713 (+0.38%),
  compound -7,898. Kept.
- **I2.** `Interp::native_token` = `row << 32 | id`; `suspended_caller` indexes
  `native_handles[row]` and checks the id (O(1)). Test `a_stale_native_frame_token_is_refused`
  (`run/tests.rs`): a live token resolves to the suspended caller; the previous frame's token at the
  same row and the live id at a row no frame holds are both refused.
- **M1** `rust/scripts/mutate-4b.sh`: the `grow_slots` panic text removed. **M2**
  `an_alias_that_does_not_descend_is_refused` (debug-only) fires the descent assertion through
  `alias_slot` + `frame_slot`. **M3** verdict line and round-3 tables in `phase-6-perf.md`.

Miri at `d112d68f9` (same recipe): `--lib` 17, `--test roots` 17, `--test collect` 14,
`--test heap` 5, `--test uninit` 7, all passed, exit 0.

Callgrind (3 rounds, vs 1754a3b5a): every program inside +0.3% except **extcall +0.3642%**
(t7b was +0.1469%); assign +0.0145%, varlookup +0.0108%, compound +0.2202%, sendloop +0.1102%,
rexxcps -0.2362%. extcall's rise over t7b is 18.0M Ir, 6 per call: the token's shift/or and row
load, and the serial load on the loop's slot accesses.

Wall clock (`wallclock.sh -r 5`, base/t7c/base2): inside every bar except assign +4.21% (Ir +0.0145%,
base2 +0.19%).

Gates at `d112d68f9`:

```
d112d68f9c91d17bfbd07d00c62e16e37becf0b2
started 2026-09-30T04:17:38+02:00
load at start 2.24 3.15 4.05 3/1592 1618482
G1 fmt exit 0
G2 clippy(empty target) exit 0
G3 release build (test --no-run) exit 0
load G4 13.97 10.79 7.08 13/1668 1624485 2026-09-30T04:21:11+02:00
G4 release test exit 0
G4 Compiling lines: 0
load after G4 14.14 15.24 10.60 13/1646 1754002
G5 debug build (test --no-run) exit 0
load G6 14.91 16.00 11.49 13/1665 1760332 2026-09-30T04:29:10+02:00
G6 debug test exit 0
G6 Compiling lines: 0
load after G6 5.69 13.72 12.62 3/1622 1889589
G7 clippy --features pinning exit 0
G8 pinning self-tests exit 0
d112d68f9c91d17bfbd07d00c62e16e37becf0b2
finished 2026-09-30T04:36:09+02:00
```

Concerns: extcall +0.3642% needs a ruling (round 3 of 3 spent); the per-activity u32 id still
wraps, but a live frame's token is now unique by its row.
