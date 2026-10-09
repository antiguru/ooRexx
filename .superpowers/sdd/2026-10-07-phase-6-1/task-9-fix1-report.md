# Task 9 fix round 1 report

Base `d23711935`. Code commit `474e4bdb9`; this report and the gate record in the commit after it.
Paths are under `rust/crates/rexx-exec/`, line numbers at `474e4bdb9`. Scratch:
`/tmp/claude-1000/p61/t9f1/`.

Test commands below run from `rust/` with `CARGO_TARGET_DIR=/tmp/claude-1000/p61/t9f1/target
CARGO_INCREMENTAL=0 memcap 8G`.

## I1: replay divergence is refused

`Replaying` (`src/sim.rs:666`) takes the file's decisions in file order through one cursor, with
the positions of the preemptions and collections indexed for the position-keyed checks:

* `point` (`:715`), for a preemption at a contended step or a collection at an allocation: a
  recorded point below the run's position is passed; a point equal to it that is not the file's
  next decision is a kind mismatch; a floor preemption the file does not hold diverges.
* `pick` (`:760`): the file's next decision must be a pick in range, else it diverges (another
  kind, out of range, or the file exhausted).
* `finish` (`:777`, from `sim_finish`): decisions left at the run's end diverge.
* `diverge` (`:784`) builds the refusal once, `Loud::sim_replay_diverged` (`:1400`), raised at the
  next boundary by `sim_breached` or at the run's end by `sim_finish` (`:1311`); `sim_is_stuck`
  (`:1169`) ends the program's wait for its activities once a replay diverged. After a divergence
  the run takes no decision from the file. Line format:
  `rexx-exec: the replay of `FILE` diverged at decision N: the file holds `pick 999`, the run picks
  from 0 to 2` (N counts from 1; `holds no more` past the end; the run `ends`, `preempts at
  contended step S`, `collects at allocation A`, or `reaches contended step S without preempting at
  P`). Exit 120.

Header hash: the trace's first line is `CONFIG hash=H` (`sim_finish`), and `read_replay` (`:361`)
refuses a header without it and a file whose decisions do not hash to `H`, before running. M3:
`trace_hash` (`:474`) is now private and used by `read_replay`; its `pub use` in `lib.rs` is gone.

Tests:

* `sim::tests::a_replay_that_diverges_is_refused` (`src/sim/tests.rs:536`): DECIDING's
  `sim:3,uniform:0.3` trace replayed on THREE_ACTIVITIES (another program: decision 4, `preempt 16`,
  the run ends); `pick 999` in place of the first pick, rehashed (`picks from 0 to 2`); truncated
  before the first pick, rehashed (`holds no more`); an extra trailing `preempt 999999`, rehashed
  (`the run ends`); `pick 0` inserted first (kind: `preempts at contended step 3`); the first two
  preemptions swapped (passed: `reaches contended step 8 without preempting at 3`). Each asserts
  rc 120 and the whole stderr. The unedited file replays on its own program with rc 0.
* `sim::tests::a_damaged_trace_is_refused_before_the_run` (`:628`): a changed header hash and a
  header without one, each the exact `SimConfig::parse` error.
* `tests/sim_processes.rs` `a_damaged_trace_is_refused_before_the_run` (`:104`): through
  `rexx-run`, status 2, empty stdout, stderr `rexx-run: REXX_SWITCH_MODE: `FILE`: the decisions
  hash to H, the header holds 0123456789abcdef`; the undamaged file replays to the recording's
  stdout.
* Same-program replay stays green: `a_recorded_trace_replays_to_the_same_output_and_hash` (now also
  asserting the header line with its hash). The reviewer's 48 cross-process recordings (four
  programs, six policies, seeds 3 and 11) re-run on the release binary of `474e4bdb9`
  (`/tmp/claude-1000/p61/t9f1/rep.sh`): 48 of 48 match rc 0, stdout and trace hash.

`cargo test -j 4 -p rexx-exec --lib sim::` 22 passed; `--test sim_processes` 3 passed.

## I2: invariant check in one pass

`check_invariants` (`src/scheduler.rs:1036`) builds `parked` (sleepers, message waiters, guard
waits), `ready` (ready queue and set-aside) and `free` vectors indexed by handle, then checks each
handle in O(1); `has_wake_source` and `holds_park_reason` are folded into it, their comments kept
inline. The order of checks and every message is unchanged. `GuardTable::inconsistency`
(`src/guards.rs:184`) was quadratic in a lock's waiters and is now linear (one set of queued
`(waiter, key)` pairs); `GuardTable::waiters` (`:177`) feeds the guard waits to the pass.

The reviewer's nine injected corruptions are permanent `Corruption` variants (`src/scheduler.rs:2532`,
guard helpers `src/guards.rs:211-230`, `BatonReleased` re-takes the baton in `sim_uncorrupt`
`:2668`) and rows of `each_invariant_is_refused_where_a_switch_finds_it_broken`
(`src/sim/tests.rs:719`) beside the committed six: all fifteen fire with their own message, one
line, rc 120; the unbroken run of each program ends normally.

Scaling probe (`/tmp/claude-1000/p61/t9r/p1/many2.rex` with n set, release binaries, wall seconds,
one run each, load average about 4 to 6; `/tmp/claude-1000/p61/t9f1/scale.sh`):

| n | head2 `every` | head2 `sim:1,uniform:0.5` | fix1 `every` | fix1 `sim:1,uniform:0.5` |
|---:|---:|---:|---:|---:|
| 250 | 0.05 | 0.10 | 0.05 | 0.04 |
| 500 | 0.08 | 0.48 | 0.08 | 0.09 |
| 1000 | 0.10 | 3.07 | 0.09 | 0.13 |
| 2000 | 0.11 | 20.66 | 0.12 | 0.35 |

Per switch the check is linear in the handles, so the whole run is still O(n) per switch times the
switches; 1000 to 2000 is 2.7x against head2's 6.7x.

## M1: one refusal per violation

`Sim::invariant_broken` (`src/sim.rs:879`): `sim_check_switch` (`:1344`) checks nothing after the
first violation. The invariant test asserts `(120, "rexx-exec: the scheduler found X\n")` exactly.
Mutation: deleting the early return reddens it on `ReadyTwice` with the line twice (run, then
restored from a copy).

## M2: trace file edges

`nonempty_path` (`src/sim.rs:328`) refuses `trace=` and `replay=` with an empty path; a second
`trace=` or `replay=` is refused; an item after a path item that is no item there is refused by
`comma_in_path` (`:337`) naming the whole path, `` `replay=/tmp/c,d.txt`: a path may not contain a
comma ``. The replay form parses its items before reading the file, so the refusal names the right
file. A trace that cannot be written is `Loud::sim_trace_unwritten` (`:1405`) from `sim_finish`:
rc 120 and `rexx-exec: the trace `PATH` could not be written: ERROR`.

Tests: `a_trace_path_is_refused_where_it_cannot_be_the_file_meant` (`src/sim/tests.rs:154`, nine
exact parse errors) and `a_trace_that_cannot_be_written_is_refused` (`:662`, rc, stdout and stderr).

The two new `Loud` constructors have `LIMIT` rows in `corpus/refusal-dispositions.tsv`;
`refusal-sites.tsv` re-derived (`REXX_REFUSAL_SITES_REFRESH=1 cargo test -p rexx-exec --test
refusal_sites`: the two rows and line shifts), then `refusal_sites` 5 passed and
`refusal_dispositions` 3 passed.

## M4

`pre_1_reaches_every_interleaving_over_a_seed_range` (`src/sim/tests.rs:397`): seeds 1 to 40 under
`pre:1,k=3`, the set of outputs equals the three interleavings exactly.

## I3: perf record

Gate record `docs/superpowers/plans/phase-6-1-gate.md`, `## Task 9`, `### Fix round 1`. base61's
sha256 matches `## Task 1`. Command:

```
memcap 8G bash rust/bench-programs/callgrind.sh -r 2 -j 6 -o /tmp/claude-1000/p61/t9f1/cg1 -p "pingmsg pingguard pingsem alloc alloc4c heapshape rexxcps emptyloop" base=/tmp/claude-1000/p61/t9/perf/bin/base/rexx-run head2=/tmp/claude-1000/p61/t9/perf/bin/head2/rexx-run fix1=/tmp/claude-1000/p61/t9f1/bin/head/rexx-run base61=/tmp/claude-1000/p61/t1/bin/base/rexx-run
```

Exit 0, spreads at most 0.0017%. Against base61:

| program | base % | head2 % | fix1 (running total) % | fix1 vs head2 % |
|---|---:|---:|---:|---:|
| pingmsg | -0.4541 | -0.2545 | -0.2545 | +0.0000 |
| pingguard | -0.5914 | -0.3665 | -0.3661 | +0.0004 |
| pingsem | -0.4358 | -0.2494 | -0.2485 | +0.0009 |
| alloc | -0.2804 | +0.0738 | +0.0738 | +0.0000 |
| alloc4c | -0.8796 | -0.8139 | -0.8139 | -0.0000 |
| heapshape | +1.0265 | +1.1558 | **+1.1558** | -0.0000 |
| rexxcps | +0.0230 | +0.0924 | +0.0924 | -0.0000 |
| emptyloop | -0.3191 | -0.3191 | -0.3191 | -0.0000 |

heapshape is over the +0.5% budget, as reviewed; not addressed here (bisect and ruling elsewhere).

## Per-task check at `474e4bdb9`

* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -j 4 --workspace --all-targets -- -D warnings`: exit 0.
* `memcap 8G cargo test -j 4 --workspace --no-fail-fast`: exit 0, 3110 passed, 0 failed, 4 ignored
  (`/tmp/claude-1000/p61/t9f1/ws.txt`).
* `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus --test
  ir_recorded_oracle`: exit 0, corpus 29 passed 1 ignored, ir_recorded_oracle 21 passed.

## Concerns

1. A trace truncated at a line boundary and given a matching hash, where the dropped decisions are
   all position-keyed (preemptions, collections) and no pick follows, replays without refusal: the
   file does not record how many contended steps or allocations the run had. Run: DECIDING's
   `sim:3,uniform:0.3` trace (51 decisions) cut to its first 10, the last pick plus one preemption,
   rehashed: the replay exits 0 with the recording's stdout and prints a different `trace=` hash
   (`/tmp/claude-1000/p61/t9f1/trunc/`). Any truncation without a rehash is refused by the header
   hash.
2. heapshape stays +1.16% over the 6.1 base (I3), pending the bisect and Moritz's ruling.
