# Task 1 report: activation scope

Status: DONE_WITH_CONCERNS.

## Commits

- `66b0f6854` Phase 6.1 Task 1: Activation layout for the scope move
- `16b67c6cc` Phase 6.1 Task 1: activation-scoped clock, seed, SETLOCAL, conditions
- `12239c974` Phase 6.1 Task 1: keep pop_activation inline on the call path (perf round 1)
- `e77576cd0` Phase 6.1 gate record: Task 1 performance

## What was implemented

- **Layout (`66b0f6854`)**: `Activation` gets `cold: Option<Box<ActivationCold>>`, which holds
  `condition` (moved out of the struct), `active_condition: Option<Rc<ActiveCondition>>`,
  `random_seed` and `locals`. Also `cached_clock: i64` with `NO_CLOCK = 0`, `elapsed_anchor: i64`,
  and `current_case_text` stored in the struct itself. Accessors `condition()`,
  `replace_condition()` and `cold_mut()`. `size_of::<Activation>() == 480`, per your ruling on R4;
  the assert's comment states the no-growth purpose. Program outputs were unchanged: the probe set
  printed the same on the base and the layout build, and the lib tests passed 1010/1010.
- **Behaviour (`16b67c6cc`)**:
  - `ActivationFlags` gets the `ELAPSED_RESET` and `DEBUG_PAUSE` bits.
  - `Activity` loses `locals`, `active_condition`, `current_case_text`, `debug_pause`,
    `elapsed_anchor` and `pending_elapsed_reset`, and `random_seed` is renamed to `random_source`.
  - `nested` copies from the caller, through `Inherited`: the anchor, the stale cached clock, the
    reset flag, and both conditions (`ActivationCold::inherited`). The seed and the SETLOCAL list are
    not copied.
  - A `CALL ON` handler inherits the trapped condition as its `active_condition`. It is set on the
    delivering activation for the handler's call and put back afterwards, the same way `condition`
    already was.
  - `Interp::top_level_activation_mut` serves RANDOM (`next_seed`) and SETLOCAL/ENDLOCAL.
  - A top-level activation's first unseeded draw takes a seed from `activation_seed`, which advances
    `random_source` once with a splitmix64 step. This is the in-crate stand-in for the oracle's
    `<<16 ^ rand()`, and it keeps successive activations' streams from being shifted copies of each
    other.
  - When an activation with SETLOCALs outstanding ends, the oldest is restored (`pop_activation`).
    This is skipped when a REPLY moves the activation on.
  - The `SELECT CASE` value and the debug-pause flag now live on the activation.
  - `replace_debug_pause` sets the running activation's bit; push, pop and the REPLY spawn derive the
    trace cache from the resumed activation's bit.
  - P89's three copy lines are deleted.
- **Perf round 1 (`12239c974`)**:
  - The termination restore moves to a `#[cold]` helper that works on the running activation in place.
  - `pop_activation` is `#[inline(always)]`.
  - `debug_pause()` reads the running activation's flag. A first version added a field to
    `TraceCache` instead, and that cost per-call code.

## Witnesses

Corpus programs, in `rust/corpus/phase-6-1.txt` and registered in `corpus.rs`, `coverage.rs`,
`collect_stress.rs`, `ir_recorded.rs` and `trace_oracle.rs`. The list goes *before* `phase-6.txt`,
because `the_differential_reads_every_phase_subset_file` compares against a sorted directory
listing:

| program | scout probe |
|---|---|
| `scope_clock_routine` | `c2` |
| `scope_clock_external` (+ `.d/extr.rex`) | `c5` |
| `scope_clock_internal` | `int` |
| `scope_random_seed` | `rnd` |
| `scope_random_unseeded` | the unseeded-RANDOM predicate |
| `scope_setlocal_method` | `sl1` |
| `scope_setlocal_routine` | `sl6` |
| `scope_propagate_routine` | `cond1` |
| `scope_propagate_after_routine` | `cond2` |
| `scope_case_text_absorbed` | `ct2` |
| `scope_case_text_absorbed_other` | `ct4` |
| `scope_internal_depth` | Review Focus 2 at depth 2000 |
| `scope_debug_pause_routine` (+ `.stdin`) | the `dbgcall` shape with a `::ROUTINE` that runs `trace ?r` itself |

The debug-pause routine runs its own `trace ?r` because a `::ROUTINE` does not inherit the caller's
trace setting: the oracle's routine constructor uses package settings.

`crates/rexx-parse/tests/sourceline_oracle/scope_*.txt` was generated with the sanctioned driver,
run on copies of the files in scratch.

Crate tests in `scheduler/tests.rs`, all REPLY shapes. Every one ran 5 times on the oracle, with the
same output all 5 times; the doc comments give the counts.

| test | probe |
|---|---|
| `a_reply_continuation_keeps_its_methods_seed_and_main_keeps_its_own` | RF1 (b), replaces the P89 witness |
| `a_reply_continuation_keeps_its_methods_elapsed_clock` | RF1 (a) |
| `a_method_starts_its_own_elapsed_clock_across_a_reply` | `reply`, as predicates |
| `a_reply_continuation_keeps_its_methods_setlocal_list` | `sl2`, RF1 (c) |
| `a_reply_continuation_propagates_its_methods_condition` | RF1 (d), SIGNAL ON variant |
| `a_reply_in_a_call_on_handler_does_not_reply_for_the_method` | RF1 (d) as the brief words it; the oracle answers 91.999, rc 165 |
| `a_reply_continuation_keeps_its_select_case_value` | RF1 (e) |

The leak test `builtin/datetime/tests.rs::a_callees_own_time_r_leaks_into_the_caller_after_it_returns`
pinned the old divergence. It is rewritten as `a_callees_own_time_r_stays_in_the_callee`, to the
oracle's rule: oracle `0.001349 0.000003 0.001359`, 3 runs, the same shape each time.

`oracle-crashes.txt` entry 10b gains `sl3`'s route. The oracle aborted with
`free(): invalid pointer`, rc 134, 5 of 5 runs; this crate answers `after internal internal 1`.

## TDD evidence

- RED, `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus`, run on the tree
  holding only the witnesses: `790 of 801 matching`. The mismatches were exactly the scope witnesses
  scout B recorded as failing: clock routine/external/internal, random seed, setlocal
  method/routine, both propagates, both case texts, debug pause. `scope_internal_depth` and
  `scope_random_unseeded` already agreed.
- RED for the REPLY shapes, from the base binary (`git archive e6af1198b`) through
  `/tmp/claude-1000/p61/t1/orc.sh`:
  - `reply`: `m before zero 0`
  - `sl2`: `cont endlocal 0` / `main endlocal 1`
  - RF1 (b): `main 100` where the oracle prints `807`
  - RF1 (d) SIGNAL ON variant: 98.918 where the oracle reports 42.3
  - RF1 (e): no `cont other`
  - RF1 (a) and the `CALL ON` shape already agreed.
- GREEN:
  - `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus --test
    ir_recorded_oracle`: `801 of 801 matching`, exit 0, at `16b67c6cc`'s tree and again at the round-1
    tree.
  - `memcap 8G cargo test -j 4 -p rexx-exec --lib scheduler::tests::a_`: 42 passed.

## Per-task check

Run before each code commit:
- `cargo fmt --all`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `memcap 8G cargo test -j 4 --workspace --no-fail-fast`: exit 101 on the first run, from two
  binaries:
  - `collect_stress`: three new programs run zero collections and needed adding to the committed
    list.
  - `refusal_sites`: line shifts in `trace/format.rs`.
  Both were fixed (the list was updated, and the table re-derived with
  `REXX_REFUSAL_SITES_REFRESH=1`), and both binaries were re-run green: 36 passed and 5 passed. On
  the round-1 tree the only failure was `refusal_sites` again (line shift), re-derived and green
  (5 passed); every other test binary reported `ok`.
- The `REXX_CORPUS_GATE=1` pair: exit 0 both times.

## Performance

Recorded in `docs/superpowers/plans/phase-6-1-gate.md` `## Task 1`, against the base `e6af1198b`.
The noise band, from the layout controls pad1/pad2/pad3, is 0: the largest control delta is 136
instructions.

| program | before round 1 (t1) | after round 1 (r1) | wall r1 |
|---|---:|---:|---:|
| fibcall | +0.61% | +0.20% | -0.13% |
| fibfunc | +0.56% | +0.27% | +0.13% |
| dispatch | +0.79% | +0.10% | -3.16% |
| dispatchclass | +0.84% | +0.10% | -2.99% |
| sendloop | +1.18% | +0.14% | -2.40% |

Instructions are callgrind with libc excluded. All five programs are inside +0.5% after one round,
and wall clock is inside ±4%.

## Files changed

- `rust/crates/rexx-exec/src/`: `activation.rs`, `activity.rs`, `builtin/datetime.rs`,
  `builtin/datetime/tests.rs`, `builtin/numeric.rs`, `builtin/state.rs`, `dispatch.rs`,
  `dispatch/context.rs`, `ir/drive.rs`, `lib.rs`, `run.rs`, `run/call.rs`, `run/condition.rs`,
  `run/interpret.rs`, `run/select.rs`, `run/settings.rs`, `scheduler/tests.rs`, `trace.rs`,
  `trace/format.rs` (round 1 leaves its content as at base).
- `rust/crates/rexx-exec/tests/`: `corpus.rs`, `coverage.rs`, `collect_stress.rs`, `ir_recorded.rs`,
  `trace_oracle.rs`.
- `rust/corpus/`: `phase-6-1.txt`, `lang/scope_*`, `oracle-crashes.txt`, `refusal-sites.tsv`.
- `rust/crates/rexx-parse/tests/sourceline_oracle/scope_*.txt`.
- `docs/superpowers/plans/phase-6-1-gate.md`.

## Self-review findings

- I deleted the stale prose that described the old activity-wide fields:
  - the `Activity` docs;
  - the datetime test docs that named `Activity::elapsed_anchor`;
  - the `Err` arm comment in `deliver_one_pending_trap`, which argued from the old single slot.
- `ActivationCold::inherited` allocates a box per internal call only when the caller has a condition
  or an active condition, which happens inside handlers. A plain call path never allocates.
- `pop_activation` checks `cold.is_some()`. That is one load and branch per activation end, and the
  restore itself is out of line.

## Concerns

1. **The traceback of a propagated condition in a REPLY continuation is missing one line.** In the
   RF1 (d) SIGNAL ON variant, the oracle's report carries `2 *-* say 'got' o~m` (the sender's frame
   at the raise) and ours omits it. Code, message and rc match, and the crate test asserts only
   those. This is a traceback defect of the same family as scout B's `cond3` (D9), not scope. I
   suggest T9 queues it.
2. **The `SELECT CASE` value stays on the activation after its construct ends.** It is set when each
   construct opens and is never cleared. I found no program that can read a stale value, because an
   absorbed WHEN follows its own construct's opening with nothing in between that runs in the same
   activation. But "for the construct's duration" is not enforced.
3. **RF1 (d) as worded does not test RAISE PROPAGATE.** A REPLY inside a `CALL ON` handler is 91.999
   on the oracle, which never reaches a continuation. So RAISE PROPAGATE across a REPLY is witnessed
   by the SIGNAL ON variant you accepted.

## Fix round 1

Findings from `task-1-review.md`.

1. **A SELECT CASE typed at a debug pause overwrote the interrupted construct's value.**
   `run_debug_fragment` (`run/interpret.rs`) now saves `current_case_text` before the typed line runs
   and restores it afterwards. This also makes spec section 2's table wrong for this field: under a
   typed debug line, `current_case_text` is not "shared" with the line, it is kept apart from it.
   The new witness is `corpus/lang/scope_case_text_debug_line.rex` with a `.stdin` file. It is the
   reviewer's probe, added to `phase-6-1.txt`, to `collect_stress.rs`'s zero-collection list, and
   with its `sourceline_oracle` expectation generated by the sanctioned driver from a scratch copy.
   - RED, on the `12239c974` release build: stdout has no `other`, and the trace shows `>>> "1"`
     where the oracle shows `>>> "0"`.
   - GREEN, on the debug build of the fix: all three descriptors identical to the oracle's (`other`,
     `done`, rc 0).
2. **The doc on `Activation::cold` was false.** It now reads "The conditions, `RANDOM` seed and
   `SETLOCAL` list, allocated on the first write or inherited from the caller
   (`ActivationCold::inherited`)."

Covering tests, each run once:

| command | result |
|---|---|
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus` | `802 of 802 matching`, 29 passed |
| `memcap 8G cargo test -j 4 -p rexx-exec --lib` | 1016 passed |
| `memcap 8G cargo test -j 4 -p rexx-parse --test sourceline_oracle` | 1 passed |
| `memcap 8G cargo test -j 4 -p rexx-exec --test collect_stress --test refusal_sites` | 36 and 5 passed |

The first `collect_stress` run failed because the new program had not yet been added to its list.
After adding it, the run above passed.
