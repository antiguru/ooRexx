# Task 6 report: `ObjRef` `!Send`/`!Sync`, bounded countdown reload, S0 gate

Base 97f564413. Status: DONE_WITH_CONCERNS

## Commits

- `6956e90f2` Make ObjRef !Send and !Sync
- `98d9fae33` Bound the clause countdown's reload for a run with no deadline
- `b9f85325e` Record Task 6's instruction counts and wall clock (S0 gate)

## Step 1: `ObjRef` `!Send`/`!Sync`

`rust/crates/rexx-core/src/handle.rs`: a `PhantomData<*const ()>` second field on the `ObjRef`
tuple struct. `cargo check --workspace --all-targets` unchanged (0 warnings, 0 errors, same as
before the change).

Doctests (`cargo test -p rexx-core --doc`):

```
test crates/rexx-core/src/handle.rs - handle::ObjRef (line 52) - compile fail ... ok
test crates/rexx-core/src/handle.rs - handle::ObjRef (line 57) - compile fail ... ok
test crates/rexx-core/src/handle.rs - handle::ObjRef (line 65) ... ok
```

The `compile_fail` doctests fail for the stated reason and not an unrelated one -- extracted and
compiled directly against the built rlib (`rustc --edition 2021 --crate-type bin -L target/debug/deps
--extern rexx_core=<rlib>`):

```
error[E0277]: `*const ()` cannot be sent between threads safely
  --> probe_send.rs:3:18
   |
 3 |     require_send(rexx_core::ObjRef::NIL);
   |     ------------ ^^^^^^^^^^^^^^^^^^^^^^ `*const ()` cannot be sent between threads safely
note: required because it appears within the type `PhantomData<*const ()>`
note: required because it appears within the type `ObjRef`
  --> crates/rexx-core/src/handle.rs:72:12
   |
72 | pub struct ObjRef(u64, PhantomData<*const ()>);
```

and the `Sync` probe is the same shape with "cannot be shared between threads safely". Both point
at line 72 (the `PhantomData` field), which is the marker's doing. The positive-control doctest
(`require_send(0u64); require_sync(0u64);`) compiles, so the two failures are not an unrelated
mismatch (e.g. a missing import or a typo `rustc` would reject for a different reason).

No pattern match or direct tuple construction of `ObjRef` exists outside `handle.rs` (checked by
grep); every other crate goes through the accessor methods, so the second field needed no other
call-site change.

## Step 2: bounded countdown reload

`rust/crates/rexx-exec/src/clause.rs`: `Deadline::NO_DEADLINE_SPACING` (`u32::MAX`) removed;
`countdown_reached`'s no-deadline arm now reloads `Deadline::CLAUSES_PER_CHECK` (1024), same as the
deadline-set arm. `Interp::new`'s initial `clause_countdown` (`lib.rs`) moves to the same constant,
so the cold path runs on the 1024-clause cadence from the very first clause, deadline or not.
`execute()`'s `if interp.deadline.is_some() { interp.clause_countdown = CLAUSES_PER_CHECK; }` is
dropped: `Interp::new` already leaves the field at that value and no clause has been counted yet at
that point in `execute`, so the line was setting a value already in place.

Behaviour: `cargo test -p rexx-exec --test deadline` -- 10/10 pass, all engine/deadline shapes
unchanged. `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus --release`: 655 of 655
matching. `rexx-run` (the binary every benchmark runs) never sets a deadline
(`Invocation::none()`/`join_command_line`, no `.with_deadline`), so every benchmark program is
exactly the population this step's cold path now visits that it did not before.

## S0 gate: performance

Full recipe, builds, oracle comparison and tables in `docs/superpowers/plans/phase-6-perf.md`
`## Task 6 (S0 gate)`. Base `1754a3b5a`; `stepA` = `6956e90f2` (Step 1, running total through
Tasks 1-5); `stepB` = `98d9fae33` (Step 1+2, the full S0 running total). `.text`: base 2,732,523,
stepA 2,729,083, stepB 2,729,035 bytes.

**Instruction counts (the binding gate).** `callgrind.sh -r 3 -j 16`, exit 0, no SPREAD, every
program's `rc` 0. Every one of the 23 programs is inside budget: largest `stepB` delta is `nop`
at +0.0303% against a +0.3%-beyond-band budget; `heapshape` is negative; `emptyloop`'s +0.0152%
matches the spec's own N=50-to-N=1024 scaling from the spike's +0.27% (1024/50 = 20.5x fewer
visits; 0.27/20.5 = 0.0132%, close to measured). Round 1 closes S0's instruction-count budget.

**Wall clock.** `wallclock.sh -r 5`, base vs stepB. Every program inside +/-4% (or its own wider
band per Ruling P10 -- `startup` -7.69%) except `dispatch` at +10.44%.

**`dispatch`'s excursion: investigated, not resolved by a round.** Three follow-up measurements
(all in phase-6-perf.md):
1. An independent re-run of base vs stepB reproduces it: +11.62%.
2. `stepA` alone (Step 1 only, an instruction count *decrease* of -5 Ir on `dispatch`, i.e.
   provably no added work) against base: **+14.91%** -- a bigger wall-clock swing than stepB's,
   from a change that executes fewer instructions.
3. An identical-binary control (base copied to a second path, same sha256) against base: -0.79%,
   inside band, confirming the instrument itself is not this noisy when nothing about the code
   differs.

So `dispatch`'s wall clock swings double digits from a flat-or-negative instruction-count change,
and does not when the binary is literally unchanged: this is layout noise (icache/branch-predictor
placement), not added cost, consistent with Task 2's own three padding controls already putting
`dispatch` at -3.58% (`pad2`) from dead code alone -- a real (non-padding) code change apparently
perturbs it more than that padding did. `sendloop` shows the same shape more intermittently
(beyond +/-4% in two of four runs, flat Ir every time).

I did not spend rounds chasing this: the instruction count is flat, so no candidate design change
is indicated, and hand-tuning code placement to satisfy one benchmark's linker-address luck is not
a principled S0 change. Reporting for a ruling instead, per the design's stopping rule and the
precedent of Ruling P10 (which already widened `decloop`/`startup`'s bands for the same reason).
**Recommendation**: extend Ruling P10's per-program band to `dispatch` (and possibly `sendloop`)
given evidence its true noise floor exceeds what the padding controls measured; instruction counts
already gate the S0 budget and are unaffected by this.

## Gates

`S=<p6-t6 scratch>` `bash .../p6-gates/gates.sh`, started at `b9f85325e` 2026-09-30T01:16:38+02:00.
`status.txt` verbatim:

```
b9f85325e5e10b249c3790dd8dcd90dc68d44625
started 2026-09-30T01:16:38+02:00
load at start 0.48 0.95 2.99 1/1585 573289
G1 fmt exit 0
G2 clippy(empty target) exit 0
G3 release build (test --no-run) exit 0
load G4 10.80 8.07 5.53 1/1610 577555 2026-09-30T01:19:37+02:00
G4 release test exit 0
G4 Compiling lines: 0
load after G4 1.69 5.12 5.17 2/1604 706238
G5 debug build (test --no-run) exit 0
load G6 1.69 5.12 5.17 1/1602 707124 2026-09-30T01:25:31+02:00
G6 debug test exit 0
G6 Compiling lines: 0
load after G6 1.91 4.60 5.15 1/1602 835899
G7 clippy --features pinning exit 0
G8 pinning self-tests exit 0
b9f85325e5e10b249c3790dd8dcd90dc68d44625
finished 2026-09-30T01:32:13+02:00
```

HEAD unchanged before/after (`b9f85325e`), `git status --short` clean. Test totals: G4 release
2786 passed 0 failed, G6 debug 2787 passed 0 failed, G8 4 passed.

## Concerns

1. **`dispatch`'s wall clock, +10.44% to +14.91% across four runs, needs a ruling** (detailed
   above). Instruction counts are flat/negative throughout; three controls point at layout noise.
2. **`sendloop`'s wall clock is intermittently over +/-4%** (two of four runs), also with a flat
   instruction-count delta; likely the same class of noise as `dispatch`, less consistently
   triggered.
3. Steps 1 and 2 landed as separate commits (not called for explicitly by the brief) so that
   Step 2's own cost could be measured apart from Step 1's; Step 1 measured as free at the
   instruction level (`stepA - base` is at most a few hundred Ir on every program, several
   programs negative), consistent with a zero-sized marker adding no code.
