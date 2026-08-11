# Phase 0 performance baseline — the C++ oracle

Task 0.7. This is the number every later phase's D9 performance gate (Global Constraints,
"Performance gate") compares against: no phase from 2 onward closes with a Rust subsystem slower
than its C++ counterpart on this suite. Coming out worse than what is recorded here, on the
platforms recorded here, is the definition of a gate failure.

**Every "this crate" figure in this file is a tree-walker figure, and re-running the harness no longer produces one** (2026-08-11, `phase-4e-gate.md`). `rexx-run` defaulted to the tree-walker when these were measured and `rexx_bench::child::Side::rust` inherited that default; Phase 4e made the compiled stream the default and made the harness name the arm on every child and print it in its provenance block. So `rexx-bench-suite` with no arguments now measures the IR arm, and `--engine tree-walker` is what reproduces the arm these rows were taken on. Nothing here has been re-measured.

**Two later sections do not belong to Task 0.7 and are marked as such.** The `rexxcps` section
records both interpreters on the Phase 4a design spec's own R1-R4 criteria, measured 2026-08-06;
the Phase 4d-1 section below it is the interleaved two-interpreter baseline, measured 2026-08-08.
Both are here because a performance figure belongs in this file, not because Phase 0 produced
them. Each is a dated measurement in its own right and none of them reuses another's numbers.

**Only the Linux row exists.** macOS, Windows, FreeBSD and OpenBSD baselines still need to be
produced by CI runs; this file has one platform's numbers, not five. Do not treat an absent
platform as "presumed fine" — Task 0.7's own text anticipates the OpenBSD leg may not even
produce a baseline given the pre-existing SIGSEGV there, and that has not been checked from this
machine at all.

## Linux (this machine)

### Environment

| | |
|---|---|
| OS | Debian GNU/Linux forky/sid, kernel `7.1.3+deb14-amd64` (`uname -a`: `Linux wasabi 7.1.3+deb14-amd64 #1 SMP PREEMPT_DYNAMIC Debian 7.1.3-1 (2026-07-04) x86_64 GNU/Linux`) |
| CPU | AMD RYZEN AI MAX+ 395 w/ Radeon 8060S (`/proc/cpuinfo`, `model name`) |
| Cores | 32 (`nproc`) |
| Memory | 124 GiB |
| rustc | `rustc 1.96.1 (31fca3adb 2026-06-26)` |
| cargo | `cargo 1.96.1 (356927216 2026-06-26)` |
| cc | `cc (Debian 15.3.0-1) 15.3.0` |
| criterion | 0.8.2 |
| C++ oracle | `build/bin/rexx`, `Open Object Rexx Version 5.3.0 r0`, build date Jul 27 2026, CMake `Release` |
| Repo commit | `4009933716d42fe266672e5014584bb880c171dc` (branch `plan/rust-rewrite`) |

The machine was otherwise idle during measurement except for this benchmark run itself; no other
CPU-bound job ran concurrently with `cargo bench`.

### Benchmark programs

`rust/bench-programs/*.rex` — one per D9 dimension, see `rust/bench-programs/README.md` for what
each covers and how the loop counts were chosen. All seven were confirmed to run under
`build/bin/rexx` with exit code 0 and byte-identical output across repeated runs before being
timed. Direct single-run wall-clock times (`time build/bin/rexx <file>`, unloaded machine),
confirming the 0.5-2s sizing target:

| Program | Wall time | Exit code |
|---|---:|---:|
| `dispatch.rex` | 1.212 s | 0 |
| `varlookup.rex` | 1.219 s | 0 |
| `compound.rex` | 1.133 s | 0 |
| `strings.rex` | 0.858 s | 0 |
| `arith.rex` | 1.175 s | 0 |
| `alloc.rex` | 1.186 s | 0 |
| `startup.rex` | 0.007 s | 0 |

`startup.rex` (`say 1`) is deliberately not sized into the 0.5-2s window — it exists to measure
cold start, and cold start is the whole point of it being small.

### Criterion results

```sh
cd rust
REXX_BENCH_BINARY="$(pwd)/../build/bin/rexx" cargo bench --offline -p rexx-bench --bench interpreter -- --save-baseline cpp-linux
```

Raw criterion output (mean, `[95% CI lower-bound, point estimate, 95% CI upper-bound]`), saved
under the `cpp-linux` baseline in `rust/target/criterion/` (not committed — criterion's data
directory is local measurement output, reproducible from this file plus the command above):

| Benchmark | Lower bound | Point estimate | Upper bound | Notes |
|---|---:|---:|---:|---|
| `interpreter/startup` | 4.7294 ms | 4.7799 ms | 4.8796 ms | see "Cold start" below — this criterion number is a secondary consistency check, not the D2 figure |
| `interpreter/dispatch` | 1.2037 s | 1.2141 s | 1.2252 s | |
| `interpreter/varlookup` | 1.1958 s | 1.1991 s | 1.2023 s | |
| `interpreter/compound` | 1.1369 s | 1.1402 s | 1.1435 s | |
| `interpreter/strings` | 854.96 ms | 858.14 ms | 863.74 ms | 1/10 samples flagged high-mild outlier |
| `interpreter/arith` | 1.1546 s | 1.1570 s | 1.1595 s | 3/10 samples flagged outliers (2 low-mild, 1 high-severe) |
| `interpreter/alloc` | 1.1558 s | 1.1621 s | 1.1699 s | 1/10 samples flagged high-mild outlier |

Each benchmark ran with `sample_size = 10` (criterion's floor), a 500 ms warm-up, and a 30 s
measurement-time ceiling — see "Deviations from the plan's literal spec" below for why the
defaults do not work here. `strings` needed criterion to extend past the 30 s ceiling to
`45.9 s` (55 iterations) to reach 10 samples; this is criterion's own adaptive behavior, not a
configuration error.

The outlier counts above are noted for completeness; none moved the point estimate outside a
narrow band relative to the direct single-run timings in the table above (a good sign the harness
and process-launch overhead are not doing anything strange), so no benchmark was re-run to chase
them.

### Cold start (D2 gate)

`hyperfine` is not installed in this environment and cannot be installed (no network — see
"Deviations" below). `rust/crates/rexx-bench/src/bin/rexx-time.rs` is the substitute: it runs a
command a fixed number of times after a discarded warm-up and reports min/median/mean/max of
wall-clock time via `Instant`, with stdout/stderr sent to `/dev/null` (no capture overhead).

```sh
rust/target/release/rexx-time --warmup 10 --runs 50 -- build/bin/rexx rust/bench-programs/startup.rex
```

| | |
|---|---:|
| min | 3.299 ms |
| median | 5.119 ms |
| mean | 5.099 ms |
| max | 7.735 ms |

**This is the number D2's gate compares against** — not the criterion `interpreter/startup` row
above. The two differ (criterion's point estimate, 4.78 ms, sits inside this run's range but the
two were sampled independently and via different code paths): criterion's number comes from
`rexx_oracle::Interpreter::run`, which captures stdout/stderr through a pipe (`Command::output()`)
on every iteration, while `rexx-time` redirects both to `/dev/null`. The `rexx-time` number is
closer to what a user actually experiences at a shell prompt and is the one to use for D2's
"~50 ms of wall clock over the C++ startup" absolute-delta threshold. Both are reported so a
future re-run has two independent methodologies to check against, not because they are expected
to disagree by much.

## Deviations from the plan's literal spec, and why

Task 0.7 supplies a runnable command for each step, but three details in it do not work as
written in this environment, in order of how much they cost to discover:

1. **`REXX_BENCH_BINARY=../build/bin/rexx cargo bench -p rexx-bench` (as literally written, run
   from `rust/`) does not resolve.** Cargo runs test/bench binaries with the crate's own manifest
   directory as the child process's cwd, not the directory `cargo` itself was invoked from and not
   the workspace root. `../build/bin/rexx`, evaluated inside the `rexx-bench` binary at runtime,
   resolves against `rust/crates/rexx-bench/../build/bin/rexx` -- i.e. `rust/crates/build/bin/rexx`
   -- which does not exist. `REXX_BENCH_BINARY` must be an absolute path (as used above), or a path
   relative to `rust/crates/rexx-bench/`. This is not specific to this machine; it is how cargo
   runs any test/bench harness, so the plan's example command as written will fail identically on
   every platform.

2. **`cargo bench -p rexx-bench -- --save-baseline cpp-linux` (no `--bench` flag) fails** with
   `error: Unrecognized option: 'save-baseline'`. Without a `--bench` filter, cargo passes
   `--save-baseline cpp-linux` to *every* test and bench binary the package produces, including
   `src/lib.rs`'s and `src/bin/rexx-time.rs`'s plain `#[test]` harnesses, neither of which
   understands criterion's CLI flags. The working invocation names the bench target explicitly:
   `cargo bench -p rexx-bench --bench interpreter -- --save-baseline cpp-linux`.

3. **Criterion's defaults (`sample_size = 100`, `measurement_time = 5s`) are wrong for programs
   sized at 0.5-2s.** At the sizing target the plan itself asks for, filling the default
   measurement window would need on the order of 100 samples x ~1s each -- multiple minutes per
   benchmark, ~15-20 minutes for the whole suite, before any adaptive behavior kicks in. This
   harness (`rust/crates/rexx-bench/benches/interpreter.rs`) sets `sample_size(10)` (criterion's
   floor), a 500 ms warm-up, and a 30 s measurement-time ceiling per benchmark, which is enough for
   a point estimate and a 95% CI without turning a baseline run into a multi-minute-per-benchmark
   affair. `strings` still needed criterion to extend past the 30 s ceiling once (to 45.9 s) to
   reach the 10-sample floor; that is expected and not a sign anything is wrong. Later phases
   adding a Rust interpreter's numbers to this same file should keep these group settings rather
   than reintroducing the defaults.

4. **`hyperfine` is not installed and cannot be installed (no network in this environment).**
   `rust/crates/rexx-bench/src/bin/rexx-time.rs` is a small stand-in, per the task brief. Arguably
   this is the better long-term choice regardless of hyperfine's availability: the D9 gate has to
   run on five CI platforms (Windows included), and depending on a binary that needs to be
   separately installed and kept in sync on every one of them is a real ongoing cost that a ~100
   line Rust binary in the workspace does not have. Recommend keeping `rexx-time` as the permanent
   cold-start tool rather than reintroducing a `hyperfine` dependency later just because network
   access happens to be available on CI runners.

5. **`criterion = "0.8.2"` with default features could not resolve fully offline as first
   locked.** Cargo's initial resolution picked `zerocopy v0.8.55` (a transitive dependency of
   `half`, itself pulled in by `criterion`'s default `plotters`/`ciborium` feature set), but only
   `zerocopy` itself -- not its companion proc-macro crate `zerocopy-derive` -- was present at
   0.8.55 in the local registry cache; the highest version with both crates cached was 0.8.52.
   Resolved with `cargo update --offline -p zerocopy --precise 0.8.52` and the same for
   `zerocopy-derive`; the pin is now baked into the committed `rust/Cargo.lock`, so this should not
   need repeating on another platform's cache unless that cache differs from this one.

None of these are corrections to the plan's *intent* -- the benchmark suite, the harness, and the
baseline it produces are what Task 0.7 asks for. They are corrections to the literal commands and
defaults, recorded here so the next person (or the next phase's gate) does not rediscover them by
trial and error.

## Reproducing this baseline

```sh
cd rust
REXX_BENCH_BINARY="$(pwd)/../build/bin/rexx" \
    cargo bench --offline -p rexx-bench --bench interpreter -- --save-baseline cpp-linux
cargo build --offline --release -p rexx-bench --bin rexx-time
LD_LIBRARY_PATH=../build/lib ./target/release/rexx-time \
    --warmup 10 --runs 50 -- ../build/bin/rexx ../rust/bench-programs/startup.rex
```

## `rexxcps`, the Phase 4a R1/R2 criteria -- measured 2026-08-06

**These criteria were written in Phase 4a's design spec
(`docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md:508-509`), which calls
`samples/rexxcps.rex` "the end-of-4c gate", and they were never run until now.**
Phase 4c reached its final whole-branch review with R1 and R2 unmeasured.
The numbers below are the first application of them, and R2 fails.

`rexxcps.rex` is pure Phase 1-4c surface: `SIGNAL ON NOVALUE`, `PARSE SOURCE`/`VERSION`/`VALUE`/
`VAR`, `TIME('R')`, `FORMAT`, `SUBSTR`, `WORD`, `LENGTH`, numeric and non-integer `DO`, compound
variables with variable tails, `SELECT`/`WHEN`, `TRACE VALUE`/`ADDRESS VALUE`, `CALL` to an
internal label, `LEAVE`, `ITERATE`, `EXIT`.
No classes, methods, `::REQUIRES` or message sends, so none of Phase 5's known gaps are in its
path.
Both sides ran the same file from a fresh directory, five runs each, `--release` on the Rust side,
stdout and stderr captured to separate files and exit status read unpiped.

### R1 -- correctness before speed: **PASS**

Stdout is byte-identical between the two interpreters on all five paired runs after masking only
the two timing-derived fields (the `Averaged:` line's counts and seconds, and the `Performance:`
line's cps).
The `REXX version is:` line was **not** masked and matched verbatim, so the crate reproduces the
oracle's `PARSE VERSION` string exactly.
Zero `Failed` lines on either side, reported as corroboration rather than as the criterion, per
R1's own note that "prints no `Failed` line" is satisfied by printing nothing.
Stderr empty and exit 0 on all ten runs.

### R2 -- the cps ratio: **FAIL**

`ratio = oracle_cps / rust_cps`, where above 1.5 fails, 1.0 to 1.5 is recorded as debt, and at or
below 1.0 passes.

| | oracle | rust |
|---|---:|---:|
| mean cps | 16,821,745 | 1,678,469 |
| spread (max-min)/mean | 0.92 % | 0.91 % |

**ratio = 10.02.**
Taking the least- and most-favourable combinations across the five-run spreads gives 9.93 to
10.11, a 1.8 % band, which is nowhere near wide enough to make the fail-vs-debt-vs-pass call
ambiguous.
Independently re-measured in a second session: 16,982,729 against 1,682,913, ratio 10.09.

The two sides did different amounts of work and the record must say so.
The benchmark self-calibrates, running a second trial only when the first comes in at or under one
second: the oracle bumped `count` from 100 to 200 and ran both trials on every run, while the Rust
side's first trial already took about 5.9 s and no second trial ran.
That is a consequence of the ratio rather than a confound in it, since the cps figure each side
prints is per clause.

### R3 -- external cross-check: agrees to within 9 %

Wall clock taken outside the benchmark (`date +%s.%N` around each invocation, not the program's
own `TIME('R')`): oracle 1.8054 s mean over 30 M clauses, rust 6.5714 s over 10 M.
Normalising for the trial-count asymmetry gives external cps of 16,617,116 and 1,521,752, an
**external ratio of 10.92** against the internal 10.02 -- an 8.96 % relative difference, inside
R3's own 10 % bar.
So the failing R2 number is not an artefact of this crate's own `TIME('R')`, which is what R3
exists to rule out.
The 0.6-0.7 s of the Rust side's wall time that its internal timer does not account for is process
startup and parse, and it is why the external ratio runs slightly *higher* than the internal one
rather than lower.

### R4, and what this section is not

R4 requires the baseline to be measured at gate time rather than reused, and it was: both sides
ran in the same session on the same machine, and the oracle figure here is **not** taken from the
Phase 0 criterion rows above.
This section records that measurement; it is not a reusable baseline, and a later gate re-measures
both sides again.

One asymmetry is worth carrying forward.
Under this project's standard `ulimit -v 1048576` this crate reserves 512 MiB of address space
(D19's `INTERPRETER_STACK_BYTES`) before running anything and the oracle reserves nothing
comparable, so under the shared cap this crate has roughly 500 MB of room and the oracle roughly
1000, and the two do not have equal headroom.
It does not touch these figures -- the benchmark's inner loop comes nowhere near either ceiling and
all ten runs completed without an allocation failure or a signal -- but it would matter if this
benchmark were run under a tighter budget.

No optimisation was attempted and none is proposed here.
Recording the number is the whole of it.

## Phase 4d-1 -- the interleaved two-interpreter baseline, measured 2026-08-08 at `107febcd` (superseded)

**Superseded by the re-measurement below.**
This section was measured at commit `107febcd`.
Five speedups landed after that measurement and before this document's next update --
`3799692d`, `b6b1d8a9`, `e1d50dda`, `c428ec8a`, `04ab4af6` -- and each moved every ratio in the
table below, so this section no longer describes the interpreter's current performance.
It is kept rather than replaced: a reader must be able to see that the numbers moved and why,
not find one set silently swapped for another.
The current numbers are in "Phase 4d-1 -- re-measured baseline after five speedups" below.

**A new section, not an edit to the ones above.**
The Phase 0 criterion rows were taken against a different oracle build and this phase's rule is
that a baseline is measured at gate time, so none of those numbers are reused here.
Both sides below were measured in one run, on the same machine, alternating.

Produced by `rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs`.
**Every heading from "Provenance" to "Axes this crate cannot run", and every table and paragraph
under them, is that program's output byte for byte.**
The prose before "Provenance" and after "Axes this crate cannot run" is written here and is not
its output.
That boundary is worth stating exactly, because 4d-2's re-measurement diffs a fresh run against
this block to detect oracle drift, and a delta that turns out to be an edited sentence costs that
task either a false alarm or the budget to reconcile it.

### Method, and the three choices that are not the obvious ones

**The two interpreters alternate within each axis** -- oracle, this crate, oracle, this crate, for
nine pairs -- rather than one side's whole set then the other's.
Frequency drift across minutes on this 32-core part exceeds several of the effects being measured,
and a block-per-side layout would charge that drift to whichever side ran during it.
`the_two_sides_alternate_and_run_in_the_working_directory` observes the order the children actually
ran in rather than asserting about the loop, because nothing in the tables below could distinguish
an alternating run from a side-at-a-time one.

**The address-space cap is 8 GiB, not this project's usual 1 GiB, and it is applied to both sides
on every axis.**
The standard cap could not be used: under `ulimit -v 1048576` this crate aborts with SIGABRT and
`memory allocation of N bytes failed` on `varlookup`, `compound`, `strings`, `arith` and
`rexxcps`, completing only `startup`.
See "Memory" below for the measurement that forced this and for why it is a finding rather than a
configuration detail.

**The statistic is the median, and the interval is the distribution-free sign-test interval for
it.**
The gate's definition of "slower" (Global Constraints, "Performance gate") is the point estimate
falling outside the C++ baseline's confidence interval on the slow side, so an interval is
required and a median alone would not do.
Not a mean and standard error, because run times here are bounded below by the work and have a
long right tail; not a bootstrap, because the reported interval would then depend on a seed.
At nine samples the interval is the second to the eighth order statistic and its achieved coverage
is 96.1%; at the fifty-one samples the offset line uses it is 95.1%.

Every child ran in a fresh empty temporary directory, with `/dev/null` on standard input, and
stdout, stderr and exit status were read as three separate descriptors.

### Provenance

| | |
|---|---|
| measured | 2026-08-08T10:35:41+02:00 |
| repo commit | `7735aac4b393852ac9bd9088697d027b6b9c1f63` |
| oracle `bin/rexx` | `/home/moritz/dev/repos/ooRexx/build/bin/rexx` -- size=62600 bytes, mtime=2026-08-05 16:02:20.172072564 +0200, sha256=bb5bb8ccbb96c376e329b91aafdad891f975ba06c941dbceba82c3848fa13019 |
| oracle `lib/librexx.so.4` | size=17853856 bytes, mtime=2026-08-05 16:02:20.064306594 +0200, sha256=42136c4038004fe2d5104181e06873301f032ced97c54a0fe84042e006d9b6fb |
| oracle `lib/librexxapi.so.4` | size=667792 bytes, mtime=2026-07-30 23:06:46.077533141 +0200, sha256=3536b76379c23fc7e3c4bce97b57d3c4812291ce5d68e71adc507ee690dc7d66 |
| this crate `rexx-run` | `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run` -- size=12927176 bytes, mtime=2026-08-08 09:34:35.308221101 +0200, sha256=77f6680275f1ed4bdd68fab93bccb2568d3c2a440e0741ca53f2f97e361ae6ac |
| address-space cap | `ulimit -v 8388608` KiB, **both sides, every axis** |
| pairs per axis | 9 sampled, 1 warm-up pair(s) discarded, oracle and this crate alternating |
| pairs for the offset line | 51 sampled, 5 warm-up |
| statistic | median; interval is the distribution-free sign-test interval for the median at a 95% target |
| working directory of every child | a fresh empty temporary directory |

### Fixed per-process offset (`startup.rex`)

**Not comparable, and not a pass.** This crate has no `CoreClasses.orx` bootstrap yet (Phase 5), so it starts fast by not doing the work the oracle does at startup. The two numbers below are each side's own fixed cost, reported so every axis above can be read net of it -- not as a result about which interpreter starts faster.

| side | median | min | max | 95.1% interval | spread |
|---|---:|---:|---:|---|---:|
| oracle | 7.226 ms | 5.435 ms | 8.997 ms | 7.010 - 7.581 ms | 49.3 % |
| this crate | 1.660 ms | 1.093 ms | 2.555 ms | 1.559 - 1.952 ms | 88.1 % |

Both offsets include one `/bin/sh` `exec` from the `ulimit` wrapper, on both sides equally.

### Axes

`iters/s` is the program's own loop bound divided by the median wall time. `iters/s net` divides by the median wall time less that side's per-process offset above.

| axis | iterations | side | median | min | max | interval | spread | iters/s | iters/s net |
|---|---:|---|---:|---:|---:|---|---:|---:|---:|
| `arith` | 500000 | oracle | 1.1545 s | 1.1456 s | 1.1764 s | 1.1503 - 1.1637 s | 2.66 % | 433083 | 435811 |
| `arith` | 500000 | this crate | 4.0694 s | 4.0446 s | 4.0974 s | 4.0611 - 4.0936 s | 1.30 % | 122868 | 122918 |
| `compound` | 5000000 | oracle | 1.1440 s | 1.1342 s | 1.1605 s | 1.1349 - 1.1596 s | 2.29 % | 4370766 | 4398548 |
| `compound` | 5000000 | this crate | 13.8938 s | 13.8525 s | 13.9972 s | 13.8582 - 13.9884 s | 1.04 % | 359873 | 359916 |
| `strings` | 3000000 | oracle | 0.8648 s | 0.8472 s | 0.8759 s | 0.8543 - 0.8724 s | 3.31 % | 3469053 | 3498282 |
| `strings` | 3000000 | this crate | 11.8497 s | 11.7281 s | 12.0120 s | 11.8111 - 11.8892 s | 2.40 % | 253172 | 253207 |
| `varlookup` | 19000000 | oracle | 1.2140 s | 1.1891 s | 1.2754 s | 1.1988 - 1.2230 s | 7.11 % | 15651144 | 15744857 |
| `varlookup` | 19000000 | this crate | 28.0086 s | 27.8945 s | 28.1793 s | 27.9243 - 28.1755 s | 1.02 % | 678362 | 678403 |

#### Ratios and the gate call

The throughput ratio (oracle iters/s over this crate's) and the wall ratio (this crate's median over the oracle's) are the same number, because both sides run the same iteration count.

**The ratio interval is indicative, and the verdict is not taken from it.** It divides one side's interval by the other's, so its joint coverage is at least 92.2% by Bonferroni -- one minus the two sides' miss probabilities added -- not the 96.1% either side carries alone. The verdict applies Global Constraints' rule directly: this crate's point estimate against the oracle's interval, slow side.

| axis | oracle median | this crate median | ratio | ratio interval | verdict |
|---|---:|---:|---:|---|---|
| `arith` | 1.1545 s | 4.0694 s | 3.52x | 3.49x - 3.56x | SLOWER |
| `compound` | 1.1440 s | 13.8938 s | 12.15x | 11.95x - 12.33x | SLOWER |
| `strings` | 0.8648 s | 11.8497 s | 13.70x | 13.54x - 13.92x | SLOWER |
| `varlookup` | 1.2140 s | 28.0086 s | 23.07x | 22.83x - 23.50x | SLOWER |

#### Same work on both sides

A wall time is only about the workload if the workload ran. Every sampled run on each side printed the same bytes, and the two sides printed the same bytes as each other.

| axis | stable within a side | identical across sides | stdout |
|---|---|---|---|
| `arith` | yes | yes | `4629643519330627.7808` |
| `compound` | yes | yes | `5000000` |
| `strings` | yes | yes | `138000000` |
| `varlookup` | yes | yes | `19000000` |

### `samples/rexxcps.rex`

The oracle's own clauses-per-second benchmark, run from the read-only C++ tree. It self-calibrates: a trial that comes in at or under a second is run again at twice the count, so the two sides do **different amounts of work** and their wall times are not directly comparable. The clauses-per-second figure each side prints is per clause and is the comparable one. Each side's `Averaged:` line is quoted so the asymmetry is visible rather than inferred.

| side | wall median | wall interval | `Averaged:` |
|---|---:|---|---|
| oracle | 1.7973 s | 1.7630 - 1.8261 s | `Averaged: 200 x 100 iterations of 1000 clauses (over 1.2s)` |
| this crate | 6.0882 s | 6.0737 - 6.1017 s | `Averaged: 100 x 100 iterations of 1000 clauses (over 5.5s)` |

| side | median cps | min | max | 96.1% interval | spread |
|---|---:|---:|---:|---|---:|
| oracle | 16822174 | 16510464 | 17197674 | 16546553 - 17189869 | 4.09 % |
| this crate | 1817623 | 1778937 | 1829822 | 1809548 - 1822448 | 2.80 % |

**Internal cps ratio: 9.26x** (oracle median over this crate's median), interval 9.08x - 9.50x.

### Axes this crate cannot run

Measured here rather than left out of the table, with the status and message each one actually produced. These belong to later tasks in this phase; what belongs to this one is that they are visible.

| axis | exit status | message |
|---|---:|---|
| `alloc` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
| `dispatch` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
| `heapshape` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |


### What the baseline says

**This section and everything after it is written here, not emitted by the harness.**

**The oracle build is identified by three hashes, not one.**
`bin/rexx` is 60 KB of `main`; the interpreter lives in the shared objects it loads, and `ldd`
resolves two of them under the build root, carrying different dates.
The suite fingerprints whatever `ldd` resolves rather than a list someone maintains, and refuses to
print a table at all if it resolves nothing.
This phase measures that build here and again at the end of 4d-2, and `phase-4-exclusions.txt`
records a sweep that moved from 18 mismatches to 12 with no code change and no harness noticing.
The C++ tree also carries an uncommitted local patch, so these hashes are the only identity that
build has.

The `rexx-run` hash was checked rather than assumed: forcing a rebuild of `rexx-exec` and its
dependents under the pinned `[profile.release]` reproduced byte-identical output,
`77f6680275f1ed4bdd68fab93bccb2568d3c2a440e0741ca53f2f97e361ae6ac`, so the binary measured is the
one the recorded commit and profile produce.

**Neither per-process offset is large enough to matter to any axis.**
The largest share it takes is 0.84%, the oracle's on `strings`, which is the oracle's shortest
axis; on this crate's side the largest is 0.041%, on `arith`.
That is what reporting the offset separately was meant to establish rather than assume.

**The spread of ratios across axes is real, not measurement noise.**
This settles the phase's open hypothesis that the wide per-axis ratio spread might be a property of
the oracle's variation rather than this crate's.
It is neither: the four ratio intervals -- 3.49-3.56, 11.95-12.33, 13.54-13.92, 22.83-23.50 -- do
not come close to overlapping each other, while the widest single-side run-to-run spread in the
axes table is the oracle's 7.11% on `varlookup`.
The per-workload differences are a factor of 6.6 from end to end and the measurement variation is a
few percent, so attribution per axis is well founded, which is what Tasks 3 and 4 depend on.

The hypothesis did guess the right side, for what that is worth: the oracle's run-to-run spread
exceeds this crate's on all four axes (2.66 against 1.30, 2.29 against 1.04, 3.31 against 2.40,
7.11 against 1.02).
It is simply two orders of magnitude too small to explain anything.

Absolute throughput is what makes that readable, and it is why the ratio alone would not have been
enough: this crate does 678,362 variable-lookup iterations a second against the oracle's
15,651,144, and 122,868 arithmetic iterations against 433,083.
`arith` is the axis where this crate is closest to the oracle in ratio terms *and* the axis where
both sides are slowest in absolute terms.

**`arith` is 3.52x, against the 1.22x recorded as Phase 2's parity debt.**
`d1-decision.md`'s "Phase 2 addendum -- arithmetic at 1.22x, recorded as debt (2026-07-28)"
recorded 1.22x and said in the same entry that it was a **lower bound**, because it timed Rust
arithmetic alone against a C++ figure that already included parsing, dispatch and variable lookup,
and that it would get worse once the Rust side started paying those costs.
It did, by a factor of 2.9.
This is that debt's scheduled Phase 4 re-measurement, and the entry's own prediction is confirmed
rather than contradicted.

**`rexxcps` is 9.26x, against 10.02x and 10.09x measured 2026-08-06.**
The oracle barely moved (16,822,174 cps here against 16,821,745 and 16,982,729 then, though those
were means of five runs and this is a median of nine); this crate moved from about 1.68 M cps to
1.82 M, roughly 8%.
Recorded as an observation, not attributed: nothing in this task looked for what changed between
those commits.

**Memory: this crate cannot run four of the five axes under the project's standard cap.**
Measured on this tree, both sides, `/usr/bin/time -v` peak resident set:

| program | this crate | oracle |
|---|---:|---:|
| `varlookup.rex` | 4,009,200 KB | 20,152 KB |
| `strings.rex` | 4,337,108 KB | 19,892 KB |
| `arith.rex` | 1,072,888 KB | 20,868 KB |
| `compound.rex` | 1,056,848 KB | 20,180 KB |
| `rexxcps.rex` | 2,686,360 KB | 20,736 KB |
| `startup.rex` | 2,864 KB | 8,380 KB |

The oracle sits at about 20 MB on every one of them and this crate is between 51x and 218x that.
This is resident memory, not reserved address space, so it is not the `INTERPRETER_STACK_BYTES`
asymmetry the `rexxcps` section above records -- 512 MiB of reservation costs nothing in RSS, and
the four loop axes reach two to eight times that reservation in RSS alone.
The figures are far larger than the live data: `varlookup.rex` is
`do i = 1 to 19000000; x = x + 1; y = x; end`, whose live set is two integers.
Whether they scale with the loop count was not measured here; that question is Task 3's or Task 4's.
Under `ulimit -v 1048576` the resident set at the abort was 492,860 KB for `varlookup`, 492,300 KB
for `compound`, 338,912 KB for `strings` and 327,112 KB for `arith`, and the allocation that failed
on `rexxcps` doubled 384 MiB -> 768 MiB -> 1.5 GiB as the cap was raised from 1 GiB to 2 GiB
without ever succeeding.

No optimisation was attempted and none is proposed here.
Landing a speedup before the gate exists destroys the property this unit is built to provide.

### Reproducing

```sh
cd rust
cargo build --offline --release -p rexx-exec --bin rexx-run -p rexx-bench
./target/release/rexx-bench-suite > baseline.md      # about 12 minutes
./target/release/rexx-bench-suite --self-check       # one pair per axis, plumbing only
```

`--self-check` runs a single pair per axis and labels its own output as not a baseline; it exists
so the harness can be exercised without spending twelve minutes and without producing a table that
could be mistaken for one.

The suite exits non-zero, and says so at the top of the section it would otherwise have produced,
when an axis fails to complete or when an axis declared `Role::Blocked` stops failing. The second
of those is the case that matters after Phase 5: three dimensions would otherwise keep appearing
under "Axes this crate cannot run" with status 0 and an empty message, timed by nothing.

## Phase 4d-1 -- re-measured baseline after five speedups, measured 2026-08-08

**Why this section exists.**
The section above was measured at `107febcd`.
Five speedups landed after it -- `3799692d`, `b6b1d8a9`, `e1d50dda`, `c428ec8a`, `04ab4af6` -- and
every task below Task 2b in this phase reads a baseline, so the stale numbers had to be replaced
with current ones rather than annotated in place.
That contradicts this phase's own no-optimisation rule (Global Constraints); the contradiction is
recorded rather than argued away, in `task-2b-brief.md` and again here: this unit can produce a
*current* baseline and attribution, and cannot produce a pre-optimisation one.

**Same harness, same method.**
This is another run of the same `rexx-bench-suite`, described in "Method, and the three choices
that are not the obvious ones" above; that section is not repeated here.
As before, every heading from "Provenance" to "Axes this crate cannot run" below, and every table
and paragraph under them, is that program's output byte for byte; the prose before and after that
range is written here.

**This run also adds `alloc4c`, measured for the first time.**
Task 4 of this phase gave `alloc.rex` a 4c-surface analogue after finding the object model entirely
absent on this crate: `.array~of`, `.string~new`, `~size` and `~length` each fail on their own and
combined, all four with the same `rexx-exec: a message send is not implemented (Phase 5)`, so no
single construct is the blocker -- the whole surface is. `alloc4c.rex`'s own header states plainly
what it does and does not preserve from `alloc.rex`; in short, it substitutes a new
compound-variable tail and a `||` concatenation for the array and string, which measures allocation
throughput rather than the collection pressure `alloc.rex` was sized to force (see "`alloc4c`: the
closest axis, and what that means" below). `rexx-run`'s sha256 is unchanged from the run this one
replaces -- no interpreter code moved, only the benchmark corpus and the harness's axis list did.

### Provenance

| | |
|---|---|
| measured | 2026-08-08T22:49:05+02:00 |
| repo commit | `d233d1e90313b1d5f7f283ea09de749958861f61` |
| oracle `bin/rexx` | `/home/moritz/dev/repos/ooRexx/build/bin/rexx` -- size=62600 bytes, mtime=2026-08-05 16:02:20.172072564 +0200, sha256=bb5bb8ccbb96c376e329b91aafdad891f975ba06c941dbceba82c3848fa13019 |
| oracle `lib/librexx.so.4` | size=17853856 bytes, mtime=2026-08-05 16:02:20.064306594 +0200, sha256=42136c4038004fe2d5104181e06873301f032ced97c54a0fe84042e006d9b6fb |
| oracle `lib/librexxapi.so.4` | size=667792 bytes, mtime=2026-07-30 23:06:46.077533141 +0200, sha256=3536b76379c23fc7e3c4bce97b57d3c4812291ce5d68e71adc507ee690dc7d66 |
| this crate `rexx-run` | `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run` -- size=12996808 bytes, mtime=2026-08-08 21:50:02.437486282 +0200, sha256=c3b2516069a1b5f5d504613986f00b69e0c0fc892c041958212d45a699c0e967 |
| address-space cap | `ulimit -v 8388608` KiB, **both sides, every axis** |
| pairs per axis | 9 sampled, 1 warm-up pair(s) discarded, oracle and this crate alternating |
| pairs for the offset line | 51 sampled, 5 warm-up |
| statistic | median; interval is the distribution-free sign-test interval for the median at a 95% target |
| working directory of every child | a fresh empty temporary directory |

### Fixed per-process offset (`startup.rex`)

**Not comparable, and not a pass.** This crate has no `CoreClasses.orx` bootstrap yet (Phase 5), so it starts fast by not doing the work the oracle does at startup. The two numbers below are each side's own fixed cost, reported so every axis above can be read net of it -- not as a result about which interpreter starts faster.

| side | median | min | max | 95.1% interval | spread |
|---|---:|---:|---:|---|---:|
| oracle | 7.645 ms | 5.307 ms | 8.620 ms | 7.181 - 7.905 ms | 43.3 % |
| this crate | 1.797 ms | 1.199 ms | 2.602 ms | 1.695 - 1.929 ms | 78.0 % |

Both offsets include one `/bin/sh` `exec` from the `ulimit` wrapper, on both sides equally.

### Axes

`iters/s` is the program's own loop bound divided by the median wall time. `iters/s net` divides by the median wall time less that side's per-process offset above.

| axis | iterations | side | median | min | max | interval | spread | iters/s | iters/s net |
|---|---:|---|---:|---:|---:|---|---:|---:|---:|
| `alloc4c` | 1000000 | oracle | 1.1279 s | 1.1144 s | 1.1477 s | 1.1205 - 1.1408 s | 2.95 % | 886579 | 892630 |
| `alloc4c` | 1000000 | this crate | 2.3422 s | 2.3099 s | 2.3615 s | 2.3134 - 2.3591 s | 2.21 % | 426957 | 427285 |
| `arith` | 500000 | oracle | 1.1556 s | 1.1470 s | 1.1572 s | 1.1509 - 1.1568 s | 0.88 % | 432666 | 435548 |
| `arith` | 500000 | this crate | 3.1207 s | 3.1001 s | 3.1378 s | 3.1039 - 3.1296 s | 1.21 % | 160219 | 160312 |
| `compound` | 5000000 | oracle | 1.1500 s | 1.1279 s | 1.1686 s | 1.1414 - 1.1593 s | 3.55 % | 4348006 | 4377107 |
| `compound` | 5000000 | this crate | 6.9965 s | 6.9733 s | 7.0259 s | 6.9847 - 7.0071 s | 0.75 % | 714640 | 714823 |
| `strings` | 3000000 | oracle | 0.8633 s | 0.8485 s | 0.8734 s | 0.8505 - 0.8718 s | 2.89 % | 3475037 | 3506087 |
| `strings` | 3000000 | this crate | 9.1611 s | 8.9866 s | 9.4877 s | 9.0661 - 9.2260 s | 5.47 % | 327470 | 327534 |
| `varlookup` | 19000000 | oracle | 1.2133 s | 1.1977 s | 1.2408 s | 1.1999 - 1.2191 s | 3.55 % | 15659908 | 15759214 |
| `varlookup` | 19000000 | this crate | 5.2800 s | 5.2632 s | 5.3408 s | 5.2670 - 5.2929 s | 1.47 % | 3598469 | 3599695 |

#### Ratios and the gate call

The throughput ratio (oracle iters/s over this crate's) and the wall ratio (this crate's median over the oracle's) are the same number, because both sides run the same iteration count.

**The ratio interval is indicative, and the verdict is not taken from it.** It divides one side's interval by the other's, so its joint coverage is at least 92.2% by Bonferroni -- one minus the two sides' miss probabilities added -- not the 96.1% either side carries alone. The verdict applies Global Constraints' rule directly: this crate's point estimate against the oracle's interval, slow side.

| axis | oracle median | this crate median | ratio | ratio interval | verdict |
|---|---:|---:|---:|---|---|
| `alloc4c` | 1.1279 s | 2.3422 s | 2.08x | 2.03x - 2.11x | SLOWER |
| `arith` | 1.1556 s | 3.1207 s | 2.70x | 2.68x - 2.72x | SLOWER |
| `compound` | 1.1500 s | 6.9965 s | 6.08x | 6.02x - 6.14x | SLOWER |
| `strings` | 0.8633 s | 9.1611 s | 10.61x | 10.40x - 10.85x | SLOWER |
| `varlookup` | 1.2133 s | 5.2800 s | 4.35x | 4.32x - 4.41x | SLOWER |

#### Same work on both sides

A wall time is only about the workload if the workload ran. Every sampled run on each side printed the same bytes, and the two sides printed the same bytes as each other.

| axis | stable within a side | identical across sides | stdout |
|---|---|---|---|
| `alloc4c` | yes | yes | `12888896` |
| `arith` | yes | yes | `4629643519330627.7808` |
| `compound` | yes | yes | `5000000` |
| `strings` | yes | yes | `138000000` |
| `varlookup` | yes | yes | `19000000` |

### `samples/rexxcps.rex`

The oracle's own clauses-per-second benchmark, run from the read-only C++ tree. It self-calibrates: a trial that comes in at or under a second is run again at twice the count, so the two sides do **different amounts of work** and their wall times are not directly comparable. The clauses-per-second figure each side prints is per clause and is the comparable one. Each side's `Averaged:` line is quoted so the asymmetry is visible rather than inferred.

| side | wall median | wall interval | `Averaged:` |
|---|---:|---|---|
| oracle | 1.7832 s | 1.7657 - 1.8173 s | `Averaged: 200 x 100 iterations of 1000 clauses (over 1.2s)` |
| this crate | 4.6818 s | 4.6567 - 4.7063 s | `Averaged: 100 x 100 iterations of 1000 clauses (over 4.3s)` |

| side | median cps | min | max | 96.1% interval | spread |
|---|---:|---:|---:|---|---:|
| oracle | 16936694 | 16561597 | 17197186 | 16604359 - 17149364 | 3.75 % |
| this crate | 2311938 | 2293261 | 2322914 | 2299072 - 2321553 | 1.28 % |

**Internal cps ratio: 7.33x** (oracle median over this crate's median), interval 7.15x - 7.46x.

### Axes this crate cannot run

Measured here rather than left out of the table, with the status and message each one actually produced. These belong to later tasks in this phase; what belongs to this one is that they are visible.

| axis | exit status | message |
|---|---:|---|
| `alloc` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
| `dispatch` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
| `heapshape` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |

### `alloc4c`: the closest axis, and what that means

`alloc4c` measured 2.08x, interval 2.03x-2.11x -- the tightest of the five axes, ahead of `arith` at
2.70x, the next closest.
That invites the question its own header's preserve/not-preserve split exists to answer: is 2.08x
this axis's true position on the allocation dimension, or is `alloc4c` simply easier than
`alloc.rex` would have been?

**The honest answer leans toward the latter, for two structural reasons the header already names.**
`alloc4c` substitutes a new compound-variable tail and a `||` concatenation for `alloc.rex`'s
array-of-3 and `.string~new` -- two heap-allocating operations either way, but not the same two.
First, a compound-variable creation is a single directory insert; it does not, unlike `.array~of`,
need a class-method dispatch, an array's own allocation, and three element stores, so whatever
per-operation overhead separates this crate's (entirely unimplemented) message-send path from the
oracle's is exactly what `alloc4c` cannot exercise.
Second, `alloc.rex`'s array and string are ephemeral -- rebound every pass, collectible under any
interpreter that actually collects -- while `alloc4c`'s compound-variable tails accumulate as a
genuinely live, growing table on both sides; a collector-driven interpreter pays a sweep cost on
`alloc.rex` that it has no reason to pay on `alloc4c`, and the oracle is plausibly such an
interpreter given `alloc.rex`'s own header says it is "sized to force multiple collections".
Neither the message-send overhead nor the oracle's collection behaviour was measured directly here,
so both are read as plausible directions rather than confirmed causes -- but both point the same way,
toward `alloc4c` narrowing the ratio relative to what `alloc.rex` would show, for reasons that have
nothing to do with this crate's own allocator being fast.

**Set against that, two heap allocations a pass is a real and nontrivial amount of work, and 2.08x is
not close to 1.00x.**
If two heap-allocating operations per iteration really are the dominant cost on this axis, 2.08x is
simply the finding: this crate's per-allocation overhead, relative to the oracle's, is smaller here
than its per-operation overhead is on decimal arithmetic, compound-variable *access*, or string
search-and-replace.
Nothing measured in this task -- no collection-pause timing, no allocator call count, no per-side
breakdown of where either interpreter's time on `alloc4c` actually goes -- distinguishes between that
explanation and the structural one above; both are consistent with 2.08x, and this task took no
measurement that would separate them.

**Flag for Task 6's attribution work.** Allocation throughput came out the closest of the five axes
to the oracle, not the furthest.
If Task 6 is building a case where the other four axes' slowness is substantially explained by
allocation cost, this result argues against that case, or at minimum is a complication it needs to
address rather than pass over: the axis built to isolate allocation is the one axis where this crate
is *least* slow relative to the oracle.
Whether that is because allocation genuinely is not this crate's bottleneck, or because `alloc4c`
undermeasures `alloc.rex`'s own allocation load for the two structural reasons above, is Task 6's
question to answer; this task's contribution is making the number and the caveat both available to
it, not resolving which explanation is right.

### Spread, and why this run is accepted despite exceeding the old band on one row

**The bar is the committed baseline's band, taken over the four axis rows `alloc.rex`'s predecessor
section measured, and not `rexxcps`.**
The `107febcd` section's this-crate spreads over `arith`, `compound`, `strings`, `varlookup` ran
1.02% to 2.40%; its oracle spreads over those same four ran 2.29% to 7.11%.
`alloc4c` has no entry in that band -- it did not exist at `107febcd` -- so its own usability is
checked separately, below, against the run it is added to rather than against `107febcd`.

**On the oracle side, every row in this run lands inside that band or below it.**
This run's oracle spreads are `arith` 0.88%, `compound` 3.55%, `strings` 2.89%, `varlookup` 3.55%.
`arith` sits below the old band's lower edge (2.29%) -- tighter, not wider -- and the other three sit
inside it, `compound` and `varlookup` closest to its upper edge (7.11%) at roughly half of it.
No oracle row exceeds the old band.

**On the this-crate side, one of four rows exceeds it, by more than either of the previous run's two
over-band rows did.**
This run's this-crate spreads are `arith` 1.21%, `compound` 0.75%, `strings` 5.47%, `varlookup`
1.47%.
`arith` and `varlookup` sit inside the old band and `compound` sits below its lower edge (1.02%),
tighter again; `strings` (5.47%) sits above its 2.40% upper edge by 3.07 points -- a wider excess
than either of the previous run's two over-band rows (0.39 and 0.46 points), though concentrated in
one row here rather than spread across two.
That is not stated away: this run is noisier than `107febcd`'s on `strings`, and only on `strings`,
out of the eight historical cells.

**`alloc4c`'s own spreads sit inside the band the run it replaces held.**
2.95% oracle and 2.21% this crate are both inside 0.94%-5.42%, the full range that run's own four
axes spanned -- the same check applied to a new axis rather than to a historical one, because no
prior run measured `alloc4c` to compare against directly.

**It is accepted anyway, on the ratio intervals rather than the raw spreads.**
The gate criterion this task exists to support is per-axis attribution, and that is what the ratio
interval carries: 2.03x-2.11x, 2.68x-2.72x, 6.02x-6.14x, 10.40x-10.85x, 4.32x-4.41x.
None of the five comes near overlapping another, so all five axes remain distinguishable from each
other despite the one wider row -- the same conclusion "The spread of ratios across axes is real,
not measurement noise" reached below for `107febcd`, reached again here on `strings`' noisier input.
What the wider row costs is precision on `strings`' own ratio, not the ability to rank the five axes
against each other, and ranking them is what Tasks 3 and 4 need.

**This is a different bar from the one the indicative run failed.**
The controller's own contended run, taken with other work on the machine and recorded in
`task-2b-brief.md` as direction-only, hit 82.77% spread on `strings` and 32.84% on `arith` -- one to
two orders of magnitude past the committed band, wide enough that its ratio intervals would have
overlapped each other and said nothing.
This run's worst row, at 5.47% (this crate `strings`), is roughly a sixth of the indicative run's
*best* row (32.84%).
Exceeding the old band on one row, when the attribution the band exists to protect still holds, is
not the same failure as a spread one to two orders of magnitude past it, and rejecting this run on
that basis would be discarding a usable measurement over noise smaller than what the band is meant to
catch.

**As a coarse cross-check**, `arith`, `compound` and `strings` land close to the contended run's own
ratios: `arith` 2.70x against 2.60x, `compound` 6.08x against 6.32x, `strings` 10.61x against 10.68x.
`varlookup` is the exception: 4.35x against 4.28x is also close in absolute terms, but the contended
run's own spread on that axis was not reported cleanly enough to know what the closeness means.
This is one sentence of agreement between a clean run and a noisy one, not a second baseline; the
noisy run's own spreads make it unusable as a check on anything finer than "same order of magnitude."
`alloc4c` has no figure in the contended run to check against, because it did not exist yet.

### The internal cps ratio is unstable across runs, more than the wall-clock ratios are

Four measurements of the same tree gave four different internal cps ratios: 7.41x (`--self-check`,
one pair, explicitly not a baseline), 6.21x (the controller's contended run), 7.31x (the nine-pair
run this section replaces), and 7.33x (this run, nine pairs, the currently accepted one).
The 7.41x figure's provenance: the coordinator ran `--self-check` on this tree before dispatching
the task that produced the run this section replaces, and passed the resulting number in a message
rather than a committed file, so no output file backs it; it is recorded here as a single-pair,
explicitly-not-a-baseline figure on that basis, not as a measurement any task reproduced.
That four-measurement spread -- 6.21x to 7.41x, about 1.2x peak to peak -- is wider than the spread
across the four wall-clock ratios' own repeat measurements taken within the task that produced the
run this section replaces, none of which moved by more than a few hundredths of a unit between those
repeats.

**That earlier claim is about repeats within one task's own verification, and it does not extend to
the comparison between that task's run and this one -- stretching it to cover this run as well is
the specific mistake this paragraph now corrects.**
Between the nine-pair run this section replaces and this run, the cps ratio moved from 7.31x to
7.33x: 0.3%, the tightest agreement in the whole four-measurement spread.
The four wall-clock ratios moved far less uniformly over that same gap: `arith` held flat at 2.70x
and `varlookup` moved 4.34x to 4.35x, but `compound` fell 6.35x to 6.08x (-0.27, about 4%) and
`strings` fell 10.77x to 10.61x (-0.16, about 1.5%) -- an order of magnitude more than the cps
ratio's own movement over the identical gap.
So cps agreeing tightly between these two runs is not evidence that the wall-clock ratios generally
reproduce closely between them; on this evidence it is closer to the opposite pattern, and asserting
the wider claim would repeat the error this paragraph exists to fix.

**`compound`'s two across-run intervals do not overlap, and that carries further than a wording
fix.**
The run this section replaces reported `compound` at 6.35x, interval 6.32x-6.39x; this run reports
6.08x, interval 6.02x-6.14x -- disjoint.
Both are nine-pair sign-test intervals nominally targeting 96.1% coverage, both taken on a machine
this task judged quiet by every check available to it (the address-space cap held, spreads sat
inside or near the historical band, and the two runs' own `startup` offsets were of the same order),
and they still share no point.
That is direct evidence that between-run variance on `compound` exceeds what either run's own
within-run interval reports: a sign-test interval describes how much one run's median would move if
that exact run were repeated, not how far a second, independently-taken run's median can land.

**This is an open question for Task 8, not one this task can close.**
Global Constraints' performance gate (`:39`) defines its verdict on interval overlap -- the
criterion's point estimate falling outside the C++ baseline's confidence interval, on the slow side.
If between-run variance on an axis exceeds what that axis's single-run interval reports, as
`compound`'s disjoint pair demonstrates it can, a gate decided from one run's interval can be decided
by which run happened to be taken rather than by a real difference in speed.
Two runs cannot establish a variance model and this task does not attempt one; the `compound` pair is
recorded here as evidence that the question is live, for Task 8 to take up with however many runs a
variance estimate actually needs.

The 7.15x-7.46x interval reported above is this run's own sign-test interval and does not capture the
between-run movement described here; it describes how much the median would move if this exact run
were repeated, not how much it moved when the run itself, the time between runs, or both changed.
Read 7.33x as the accepted figure for this measurement and read 6.21x-7.41x as the honest range
across what has actually been observed, not as a tighter interval around 7.33x.

Recorded as an observation about measurement stability, not attributed: nothing in this task looked
for why the cps ratio and the wall-clock ratios move by such different amounts between these two
particular runs.
`compound`'s disjoint intervals are themselves evidence against treating nine-pair runs as reliably
stable run-to-run -- the opposite of what looking at the cps figures alone would have suggested, and
a reminder that a pattern seen in one measurement (cps) does not transfer to another (wall clock)
without checking.

### What moved between the two baselines, and why

**Every ratio fell, and none fell by the same factor.**

| axis | `107febcd` ratio | this run's ratio | change |
|---|---:|---:|---:|
| `arith` | 3.52x | 2.70x | -23% |
| `compound` | 12.15x | 6.08x | -50% |
| `strings` | 13.70x | 10.61x | -23% |
| `varlookup` | 23.07x | 4.35x | -81% |
| `rexxcps` (internal) | 9.26x | 7.33x | -21% |

`alloc4c` is not in this table: it did not exist at `107febcd`, so it has no prior figure to compare
against, and it is not one of the four axes the five commits below were attributed against.

The five commits named at the top of this section are what moved these: `3799692d` (inline a
literal that already spells a small integer), `b6b1d8a9` (small-integer arithmetic without building
a decimal), `e1d50dda` (step a counted loop without building a decimal per iteration), `c428ec8a`
(compute a clause's indent once per position instead of once per step), `04ab4af6` (fill the indent
table in one walk instead of one per position).
`varlookup`'s far larger drop is consistent with `e1d50dda` landing directly in that axis's
loop-stepping path, while `arith`, `strings` and the internal `rexxcps` ratio -- which do not depend
on counted-loop stepping the same way -- moved by a narrower and more similar 21-23%.
This task did not re-attribute which commit caused which axis's share of its drop; that division of
labour is unattempted here and is Task 3's or Task 4's if it is wanted.

No optimisation was attempted in this task and none is proposed here.
The five commits that moved these numbers predate it.

## What is still missing

- macOS 15 arm64, Windows/MSVC, FreeBSD 14.2, and OpenBSD 7.8 rows. None have been run. CI must
  add a job per platform that builds the C++ oracle, runs this same suite, and either commits
  numbers here or documents why a platform could not produce them (the known OpenBSD SIGSEGV is
  the anticipated case for that one, per Task 0.7's own text). `rexx-bench-suite` is Linux-only as
  written: the address-space cap is a `/bin/sh` builtin, and the fingerprints come from `ldd`,
  `stat` and `sha256sum`.
- A Rust side for the axes under "Axes this crate cannot run" above. That table is the live list,
  emitted by the suite from the roles in its own source; each entry exits on a message send and is
  Phase 5's. The suite reports them rather than omitting them, and turns red if one stops being
  blocked while still declared so.
