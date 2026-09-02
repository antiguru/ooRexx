# Gate parallelization: measurement, design, acceptance

**Built 2026-08-31, mid-5b.** Moritz first asked for it before Task 1, then changed the order
("Let's do parallelization after 5b lands, before 5c"), then brought it forward again after Task 5's
fix round: "poll the rayon test parallelization forward". Driven directly by the controller, not via
SDD. The measurements and design below are as they stood before the build; what was actually built,
and the three things this design got wrong, are in "Built" at the end.

Bringing it forward is what avoided the cost this section was written to record: eight 5b tasks
remain, each ending in a gate run and most taking at least one fix round, so the phase would have
paid roughly 20-30 hours of gate wall clock that this change cuts to minutes per run.

## Measured, before any change

From Task 0's own gate logs at `4c553f383` plus one direct run of the prebuilt release binary.

Command that printed the per-binary figures:
`REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (gate 5, debug) and
`REXX_CORPUS_GATE=1 cargo test --release --workspace` (gate 4, release), each redirected to a log.

| binary | debug | release |
|---|---|---|
| ir_dual | 2218.98s | 539.30s |
| bif_assertions | 1084.79s | 249.41s |
| assertions | 446.50s | 102.52s |
| collect_stress | 55.50s | 12.70s |
| gate_table_c | 53.80s | 13.81s |
| corpus | 30.09s | 8.82s |
| gate_table_d | 16.58s | 4.50s |
| builtin_status | 7.68s | 2.48s |

Three binaries are 91% of the debug gate's ~63 min. `nproc` is 32, memory 124 GB, and the gate uses
about one core.

**Cause.** Each of the three is a single `#[test]` wrapping a serial `for` loop over independent
cases (`ir_dual.rs:1586`, `assertions.rs:870`, `bif_assertions.rs:539`). libtest's thread pool
cannot reach inside one test. cargo also runs test binaries one at a time, which is a second-order
serialization that only matters once the first is fixed.

**A hypothesis this killed.** `assertions.rs` has five `#[test]`s that each recompute the same
sweep, which looks like a free 5x. Measured with the prebuilt binary:

```
$ target/release/deps/assertions-b7e90b25bf7b1601 --exact assertions_differential --test-threads=1
test result: ok. 1 passed; ... finished in 102.40s
```

against the whole binary's 102.52s. The five already overlap under libtest's pool, so memoizing the
shared sweep saves CPU and no wall time at all. Recorded because it looked certain before it was run.

## Design

In all three, the expensive call is a **pure** function inside an otherwise cheap accumulation loop:
`compare(case) -> Option<String>`, `evaluate_row(row) -> RowOutcome`, `evaluate(&row.kind) ->
RowOutcome`. So the change per file is to parallel-map that one call into an index-ordered `Vec` and
leave the existing serial fold untouched, consuming precomputed values.

Two consequences worth stating. The emitted report is byte-identical by construction, because the
fold still walks the same values in the same order. And `assertions.rs`'s order-dependent
`occurrence_of` stays inside the serial fold, so it needs no precompute pass.

**rayon, not a hand-rolled pool.** `rayon` 1.12.0 is already in `Cargo.lock` (criterion pulls it) and
its `.crate` is in the offline registry cache, so it resolves and builds offline with no new
transitive dependency and one line of lockfile churn. It goes in as a **dev-dependency** of
`rexx-exec`, so the interpreter's own runtime dependency set is untouched. `par_iter().map().collect()`
gives the index-ordered result directly, over a mature work-stealing scheduler; case costs are very
uneven here, which is exactly what work stealing is for, and writing that by hand is the kind of
reuse-over-writing this project's rules already call for.

Alternatives weighed and rejected, recorded so they are not re-litigated:

* **Proc macros (`test_case`, `rstest`, a custom one) generating one test per case.** Out on two
  independent grounds. The case list is read at runtime from `ootest/`, a git-ignored svn working
  copy, so compile-time enumeration goes stale whenever that checkout moves with nothing to notice;
  and the counts are around 4,259 + 5,185 + 10,600 cases, which is a compile-time problem of its own.
* **`datatest-stable` / `libtest-mimic`.** Registering cases at runtime avoids the staleness problem
  and would give per-case names and nextest integration. But neither is in the offline cache, each
  target needs `harness = false` and a rewrite, and it breaks the consolidated report -- the
  headline counts, per-group tables and attribution-by-construct that the constraints tell tasks to
  read. Whole-set assertions (`the sweep compared nothing at all`, `EXPECTED_SUBSET`'s pinning,
  `the_drop_reasons_account_for_every_call_outside_the_population`) would need a second pass anyway.
* **`datadriven`.** Already a dependency at 0.9.0 and already used, at `ir_dual.rs:915` and
  `ir_dual_oracle.rs:199`, for the case-file tests. It organises case data and does not parallelize
  execution, and the hot sweeps are not case-file tests.
* **`cargo-nextest`.** In the cache, and it would fix cargo's one-binary-at-a-time serialization.
  Complementary rather than a substitute: it cannot split a single 37-minute test. It also needs
  checking against the `emit_uncaptured` trick these harnesses use to escape libtest's thread-local
  capture, which may behave differently under process-per-test. A separate step, after this one.

## Deferred, and Moritz's call rather than mine

The gate executes overlapping program sets roughly three times: `assertions` runs the expressions
rows, `bif_assertions` the bif rows, and `ir_dual` re-runs both extractions plus the corpus on two
engines, from the same extractors. That is the largest remaining win after parallelization, and it
changes what each binary asserts and how failures are attributed, so it is not being done here.

## What has to be checked, because a faster gate that stopped comparing is the worst outcome here

1. **The emitted report is byte for byte what the serial one emitted** on the same tree. Capture
   before and after and diff. This is the property, and it is checkable rather than argued.
2. **A negative control**: perturb one case so it diverges, and confirm the parallel sweep still
   catches it and names the right case. A sweep that silently compared nothing would satisfy (1).
3. **The invocation count survives.** `support/oracle.rs` counts its own runs on purpose -- its doc
   says the one thing a faking classifier cannot do is start a subprocess -- so if any parallelized
   sweep runs the oracle, that counter becomes atomic rather than being dropped.
4. **Memory under the gate's `memcap 8G`** with N concurrent interpreters. Measure before deciding
   whether the cap or the thread count moves; do not raise a documented gate command on a guess.
5. **Thread-local instrumentation.** `ir/drive.rs` carries `thread_local!` counters. A test that
   asserts on one must run its program and read the counter on the same thread; the parallel map
   keeps both inside `f`, which preserves that, but any such assertion must be checked rather than
   assumed.

## Ordering

Task 0 touches `corpus.rs`, `coverage.rs`, `collect_stress.rs`, `ir_dual.rs` and `trace_oracle.rs` --
four of which this work also touches. Nothing here starts until Task 0 has committed.

## Built

Five files, +34 -16: `rayon` 1.12.0 as a dev-dependency of `rexx-exec`, and one
`par_iter().map().collect()` per sweep precomputing the pure call into an index-ordered `Vec`, with
every serial fold left to consume it unchanged. Gates 1-5 all pass on the live tree.

### Three things the design above got wrong

**Five sweep sites, not three.** "The change per file is to parallel-map that one call" is wrong on
wall clock. `assertions.rs` and `bif_assertions.rs` each have *two* full-sweep tests, not one:
`assertions_differential` plus `the_exempt_set_matches_the_current_blocked_rows`, and
`bif_assertions_differential` plus `the_exempt_set_matches_the_current_failures`. libtest overlaps
the tests within a binary -- the same fact the "hypothesis this killed" section above rests on -- so
a binary's wall time is its slowest test. Parallelizing only the `*_differential` half of each would
have left the other sweep serial as the new bottleneck and bought **no wall clock at all** in either
binary. The remaining `evaluate` calls in both files are single-row or early-exit `find_map`, so
five is the whole set.

**Check 3 needed no work, and is moot besides.** `support/oracle.rs:143` is *already* an
`AtomicUsize` incremented with `fetch_add`, not the plain counter the check assumed would have to be
converted. Moot regardless: none of the three binaries constructs an `Oracle`. Every mention of one
across all three files is prose.

**Check 5 needed no work.** No test file reads any of the ten `thread_local!` counters in
`ir/drive.rs` and `ir/compile.rs`. The single hit across `tests/` is a *comment*, in
`ir_dual_cases/message-sends`, naming `TRACE_OP_ECHOES` to explain why those rows do not rest on it.

### Why N interpreters in one process is safe

The design assumed this rather than stating it. Nothing in any crate writes process-global state: no
`static mut`, no global singleton (the four `OnceLock`s are read-caches), and no `set_var`,
`remove_var` or `set_current_dir` anywhere under `crates/`. The env half holds *because of the
no-unsafe rule* -- `bin/rexx-run.rs:52` already records that `std::env::remove_var` is `unsafe` and
this workspace forbids it. A rule adopted for unrelated reasons is what makes these sweeps
parallelizable. `assertions.rs:1028` carried the other half in a committed comment written long
before anyone was thinking about threads: "`Interp` is fresh per `run_program` call, so there is no
shared mutable state a stray perturbation could leak through".

### Measured

Interleaved, two rounds, alternating arms, both arms `git archive` extracts of `9c0007723` built and
run with their own `CARGO_TARGET_DIR`. Reproducible to ~1s across rounds. Release:

| binary | serial | rayon | speedup |
|---|---|---|---|
| ir_dual | 489.8s | 24.7s | 19.8x |
| bif_assertions | 234.5s | 23.1s | 10.1x |
| assertions | 99.1s | 9.8s | 10.1x |
| **total** | **823.4s** | **57.7s** | **14.3x** |

**Gate 4 as a whole goes 933.54s to 187.67s, 5.0x**, and that is the figure to quote for the gate:
the binaries this does not touch now dominate it, and `ir_dual` runs at 49.93s rather than 24.7s
under workspace contention. Gate 5 (debug, under `memcap 8G`) completes in 10:24.94 including the
debug build, against the ~63 min this document measured before the change.

### The five checks

1. **Report byte-identity: passed.** With cargo's own lines and libtest's timing chatter stripped,
   `bif_assertions`' 117-line report and `assertions`' 31-line report are byte-identical before and
   after. Stated honestly: `ir_dual` emits no report on a green run, so its "identical" is
   empty-against-empty and proves nothing. What covers `ir_dual` is check 2.
2. **Negative control: passed, and proven able to fail.** Stronger than the "perturb one case" this
   document asked for: `compare` was made to return every case's own name, so each reported line must
   read `[pop] NAME: NAME`. All **10621 of 10621** lines named the case they belonged to. Inverting
   it -- one `.skip(1)` on the zip -- turns that into **0 of 10620** aligned. So the control detects
   misattribution, and the shipped code has none.
3. **Invocation count: no work needed.** See above.
4. **Memory: measured, and neither the cap nor the thread count moves.** Gate 5's real command on the
   parallel arm peaks at **894,176 KB (873 MB) against the 8,192 MB cap**, with rayon's default 32
   threads. `memcap` is a cgroup-v2 `memory.max` that OOM-kills on breach, so this is a pass, not an
   estimate.
5. **Thread-local instrumentation: no work needed.** See above.

### One trap, for whoever runs these arms again

`crates/rexx-bench/src/bin/rexx-bench-suite.rs:742` resolves `rexx-run` as
`CARGO_MANIFEST_DIR/../../target`, which ignores `CARGO_TARGET_DIR`. Gate 5 in a sandbox with its own
target dir therefore fails `every_blocked_axis_still_fails_on_this_crate` -- the test refuses to skip
when the binary is missing, by design. That failure is an artifact of the sandbox, not of the tree
under test; run gate 5 from the real checkout, or build the bin into `<source>/rust/target` first.

### The lockfile went wrong, and how

`58b3043a1` shipped with a false sentence in its own message: "the lockfile churn is one line".
Adding the dev-dependency made cargo re-resolve the whole graph against the local registry cache and
take newer versions of about thirty unrelated crates with it -- `bumpalo` 3.17.0 to 3.20.3, `either`
1.16.0 to 1.18.0, `crossbeam-utils` 0.8.21 to 0.8.22 and the rest -- 138 changed lines. Corrected in
`2d5614e63`, which restores the lock to `9c0007723` plus the single predicted `+ "rayon",` line and
re-runs all five gates against it, since the first green covered the bumped dependency set and did
not transfer. Not amended; the false sentence stands in history.

**The mechanism, because it will recur.** `git diff --stat` read *before* the gates showed the
correct `Cargo.lock | 1 +`. The bumps appeared *during* the gate runs, when cargo rewrote the lock,
and the file was then staged on the strength of that earlier reading. A build tool rewriting a
tracked file between review and commit is not the same hazard as a concurrent agent editing one, and
nothing in the workflow was watching for it. Re-read `git diff --stat` immediately before `git add`,
and treat any generated-and-tracked file (`Cargo.lock` above all) as unread until re-read.
