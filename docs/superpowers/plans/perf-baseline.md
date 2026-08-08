# Phase 0 performance baseline — the C++ oracle

Task 0.7. This is the number every later phase's D9 performance gate (Global Constraints,
"Performance gate") compares against: no phase from 2 onward closes with a Rust subsystem slower
than its C++ counterpart on this suite. Coming out worse than what is recorded here, on the
platforms recorded here, is the definition of a gate failure.

**One later section does not belong to Task 0.7 and is marked as such**: `rexxcps`, near the
bottom, records both interpreters on the Phase 4a design spec's own R1-R4 criteria, measured
2026-08-06. It is here because a performance figure belongs in this file, not because Phase 0
produced it.

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

## What is still missing

- macOS 15 arm64, Windows/MSVC, FreeBSD 14.2, and OpenBSD 7.8 rows. None have been run. CI must
  add a job per platform that builds the C++ oracle, runs this same suite, and either commits
  numbers here or documents why a platform could not produce them (the known OpenBSD SIGSEGV is
  the anticipated case for that one, per Task 0.7's own text).
- A Rust side for the seven `bench-programs/` dimensions. The criterion rows at the top of this
  file are still the C++ half of a comparison with nothing on the other half; the `rexxcps`
  section above is the first Rust figure this file carries, and it covers one program rather than
  the seven.
