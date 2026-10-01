# Task 2 report: benchmarks, the measurement script and the Phase 6 base

Status: DONE_WITH_CONCERNS

## Commits

- `55ee6c08b` Add the Phase 6 benchmark programs and callgrind script
- `0696153b4` callgrind.sh: skip the stdout check on timed programs, widen deltas
- `f0e3e727e` callgrind.sh: report the median of three rounds, add a table-only mode
- `cc74f959c` Record the Phase 6 base, its noise band and wall clock
- `1f652a43d` Cut the Phase 6 benchmark prose to commands and measurements (prose rule from Moritz; comments and headers only, the four programs re-checked against the oracle: all three descriptors same)

## Programs and oracle comparison

New in `rust/bench-programs/`: `fibcall.rex` (fib(22) by internal CALL), `fibfunc.rex` (by
function call), `sendloop.rex` (empty `::METHOD` sent in a loop, 5,000,000), `extcall.rex`
(`TestIntArg` from liborxfunction in a loop, 3,000,000, `::requires 'orxfunction' LIBRARY`).

The fib programs repeat fib(22) `n = 30` times so they have the `n = <integer>` loop bound
`rexx-bench`'s harnesses require of a `Role::Loop` axis (and scale linearly when `rexx-arms`
halves it); a single fib(22) is about 30 ms. Sized to about 1 s on the base `rexx-run`.

Oracle comparison, each from a fresh empty directory, oracle as
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`,
Rust as `LD_LIBRARY_PATH=<same> $S/bin/base/rexx-run FILE` (base sha256 `3a8255...85b0`),
stdout/stderr/exit compared separately (`$S/oracmp.sh`):

```
fibcall stdout=same stderr=same rc=0/0 out=[fib(22) = 17711|] errbytes=0
fibfunc stdout=same stderr=same rc=0/0 out=[fib(22) = 17711|] errbytes=0
sendloop stdout=same stderr=same rc=0/0 out=[done|] errbytes=0
extcall stdout=same stderr=same rc=0/0 out=[3000000|] errbytes=0
```

Nothing touches rxapi state; no NUMERIC DIGITS; none of oracle-crashes.txt's shapes.

**liborxfunction loading.** `rexx-run` takes `LD_LIBRARY_PATH` from its process environment
(`Interp`'s `library_search`, read once at start, `rexx-exec/src/lib.rs` `library_search_of`) and
looks a bare library name up there before `dlopen`. `callgrind.sh` puts `REXX_LIB_DIR` (default
the oracle's `build/lib`, whose `liborxfunction.so` differs from the worktree's own `build/lib`
copy) on `LD_LIBRARY_PATH` for every run. Registering the programs in `rexx-bench` needed two
harness changes so `extcall` runs there too: `Side::rust` now carries
`LD_LIBRARY_PATH=<ORACLE_ROOT>/lib` (its test now asserts the env holds nothing else), and
`interpreter_under_test` appends the oracle's lib dir to the criterion harness's search path.
`PROGRAMS` and `AXES` list the new programs (all `Role::Loop`).

## Script location

`rust/bench-programs/` (`callgrind.sh`, `cgsum.py`, `layout-pad.py`). Reason: the script's subject
is that directory -- it enumerates it and refuses to run when its `PROGRAMS` line disagrees with
`*.rex` there -- and it is shell plus Python, not Rust, so it has no place in the `rexx-bench`
crate's build. The existing `rexx-bench` tests enumerate only `*.rex`, so the non-`.rex` files do
not disturb them.

The program list is committed in the script (`PROGRAMS=` line, all of `bench-programs/*.rex` plus
`rexxcps`, which the 2026-09-23 meas.sh measured from `bench-rexxcps/`; meas.sh's other axes all
live in `bench-programs/`).

## Binaries

| name | source | sha256 | `.text` |
|---|---|---|---:|
| base | `9d863ccc5` | `3a82550968f46b4d0a82d38151bda74469d1a89b3980c42d2cdd35a640fa85b0` | 2,737,579 |
| pad1 | base + 8 pads | `1070e97a9bd5624fd00eae2fad777cf836bce2542bbbbadaa1a9edc02b1ca9be` | 2,738,747 |
| pad2 | base + 48 pads | `a8bef57c1fa6dc8d10ae7216e266567d69b32865558bba1a626acf3ede6f27af` | 2,745,131 |
| pad3 | base + 192 pads | `4dc74e8c313914226bda2d05f0c1153b39eb9d449c515f5ac7ffd053af93cf40` | 2,768,171 |
| t1pa | `0d911ab7c` | `39b588c01897934d77b38d2243ffc315504e48dddd6086c05262dfe38f1f4d1b` | 2,737,787 |

Each from `git archive`, every file touched, own `CARGO_TARGET_DIR`; each build log has exactly
one `Compiling rexx-exec` line. Pads: `#[inline(never)]` functions kept by a `#[used]` static,
inserted before the first `impl Interp` of `ir/drive.rs`. `nm -S`: the multiset of symbol sizes,
pads excluded, is identical to base in all three controls (3869 sized symbols); the two
`run_ops_from` instances keep sizes `0x37ed`/`0x38e7` and only move.

## Base table and noise band

Full table in `docs/superpowers/plans/phase-6-perf.md`. Summary: noise bands are at most
0.0006% on every program except `sayloop` (0.0030%) and `startup` (0.0045%); most are under
0.0001%. This is far below the 0.3% budget granularity, unlike the 2026-09-19 pad sweep's 0.2%.

## Task 1 parent (0d911ab7c) against the base

Task 1 moved `rexxcps` -0.0221% (base executes 3,961,138 fewer instructions in sitting 2; same in both
sittings), outside its band. Nothing else moved beyond noise: every other program is within its
band or under 0.0001% beyond it, except `startup` and `sayloop`, whose deltas sit inside their
own round-to-round spread (detail in the record).

## Wall clock

Five interleaved rounds, base vs pad2 on all programs, oracle on extcall; binaries staged at one
fixed path. pad2 vs base: -3.58% (dispatch) to +3.38% (decrender), inside ±4%. `extcall` base
0.992 s median vs oracle 0.563 s (1.76x). Load average ~6 from other sessions during the run.

## Gates

At `1f652a43d` (same script; the run at `cc74f959c` was also all exit 0, kept at `p6-t2/gates-cc74f959c/`). status.txt verbatim:

```
1f652a43de1d4d109de80d324c114671a2ca23b4
started 2026-09-29T16:31:59+02:00
load at start 2.69 5.34 9.31 6/2514 2367198
G1 fmt exit 0
G2 clippy(empty target) exit 0
G3 release build (test --no-run) exit 0
load G4 4.53 5.62 9.31 6/2524 2369397 2026-09-29T16:32:20+02:00
G4 release test exit 0
G4 Compiling lines: 0
load after G4 5.89 6.97 8.97 3/2516 2499036
G5 debug build (test --no-run) exit 0
load G6 5.89 6.97 8.97 5/2519 2499364 2026-09-29T16:38:29+02:00
G6 debug test exit 0
G6 Compiling lines: 0
load after G6 4.43 7.05 8.60 2/2540 2627975
1f652a43de1d4d109de80d324c114671a2ca23b4
finished 2026-09-29T16:44:46+02:00
```

Summed `test result` lines: G4 release 2779 passed 0 failed; G6 debug 2780 passed 0 failed.

## Concerns

1. **Run-to-run excursions in instruction counts.** Two single runs in 345 (sitting 2) landed far
   above their binary's other runs: extcall +0.42% (42,000,000 of 42,000,341 in `rexx_api::handles::Table::register`,
   which inserts into a seeded `std` HashMap) and rexxcps +0.042% (`resolve_fixed_call` plus
   280,003 extra `memcmp` calls; its label lookup is a BTreeMap). Cause of either not established. An excursion on extcall exceeds the +0.3% S0/S1 budget by
   itself. I switched `callgrind.sh` to the median of three rounds (default now 3), which absorbs
   one excursion per cell; two in one cell would not be absorbed. Later stages should read the
   spread columns and re-run a cell whose spread is large.
2. The noise band as the spec defines it (largest control delta) is 0.0000-0.0045%, so for
   instruction counts the band is effectively zero and the budget is the whole margin.
3. `rexx-bench` behaviour changed slightly: the suite's Rust side now runs with
   `LD_LIBRARY_PATH` set (all axes), which alters its environment size relative to earlier suite
   sittings (the argv/env wall-clock artifact recorded in project memory).
4. Sitting 1 exited 1 only because the first script version compared stdout on heapshape and
   rexxcps, which print timings; every run exited 0. Fixed in `0696153b4`.
5. The wall-clock harness is a scratch script (quoted verbatim in the record), not committed.

## Fix round 1

### Commits

- `1754a3b5a` Hash the interpreter's maps with FxHasher instead of RandomState (new Phase 6 base)
- `44c465281` Flag high-spread callgrind cells, commit the wall-clock script (callgrind.sh SPREAD flag, one-or-more binaries, `-p`; wallclock.sh; rexx-bench `Side::for_axis`/`axis_library_dir`)
- `e4928c929` wallclock.sh: read bash's carried time digit, add -T
- `833847a9b` Re-measure the Phase 6 base at 1754a3b5a (phase-6-perf.md rewritten; cgdiff.py)
- `d1086bb6e` Re-derive refusal-sites.tsv after the lib.rs import line (test data only; gates at `833847a9b` failed `the_table_holds_every_constructor_the_source_defines` in G4 and G6 on shifted lib.rs line numbers; re-derived with `REXX_REFUSAL_SITES_REFRESH=1`, only lib.rs line numbers change)

### Ruling 1: default-hasher maps

Command, from `rust/`:

```
git grep -nE '(^|[^A-Za-z_])Hash(Map|Set)\b' <REV> -- 'crates/rexx-*/src/*.rs' ':!*tests.rs' ':!*/tests/*' | grep -vE '^[^:]*:[^:]*:[0-9]+:\s*//'
```

At `1f652a43d`: 141 lines (piped to `wc -l`), in handles.rs, lib.rs, environment.rs,
directives.rs, body.rs, activation.rs, dispatch/library.rs, install.rs, ir/compile.rs,
libraries.rs, behaviour.rs, builtin.rs, selector.rs, dispatch/package.rs, token/symbols.rs.
At `1754a3b5a` (working tree, without `<REV>`):

```
crates/rexx-core/src/lib.rs:48:pub type NameMap<K, V> = std::collections::HashMap<K, V, NameHasher>;
crates/rexx-exec/src/dispatch/package.rs:1028:fn cloned<K: Clone, V: Copy, S>(held: Option<&std::collections::HashMap<K, V, S>>) -> Vec<(K, V)> {
```

Both name their hasher. `git grep -nE 'RandomState' -- 'crates/rexx-*/src' | grep -v '//'`: no
output. Converted mechanically (`HashMap` -> `FxHashMap`, `HashSet` -> `FxHashSet`, `::new()` ->
`::default()`), plus two `kept:` initialisers in `rexx-exec/src/tests.rs`. `rustc-hash = "2.1.2"`
added to rexx-api, rexx-exec, rexx-parse; `git diff 1f652a43d 1754a3b5a -- rust/Cargo.lock`: three
`+ "rustc-hash",` dependency lines, no `[[package]]` added.

Converted maps that are iterated (from
`git grep -nE '\.(<field names>)\b[^;]{0,40}\.(iter|iter_mut|keys|values|values_mut|drain|into_iter|retain)\(' ...`):

| site | reaches output |
|---|---|
| `rexx-api/src/handles.rs:72` `entries.values()` | no (GC roots) |
| `rexx-core/src/body.rs:813` `NativeObject::keys` | yes, unsorted, via `Interp::native_keys` (`environment.rs:1191`) and `dispatch/package.rs:641`; `run/loops.rs:656` sorts |
| `rexx-core/src/body.rs:919` `entries.values()` | no (GC roots) |
| `rexx-exec/src/dispatch/library.rs:500` `kept.drain()`, `:514` `kept_strings.retain`, `:521` `.keys().count()` | no |
| `rexx-exec/src/lib.rs:2894` `stem_exposers.values()`, `:2916` `security_managers.values()`, `:2991` `kept_strings.retain` | no (GC roots, pruning) |
| `rexx-exec/src/stem.rs:555` `stem_exposers.retain` | no |
| `rexx-exec/src/environment.rs:436` `known.into_iter()` | no (sorted on the next line) |

### Ruling 2: repeat runs of the new binary

```
rust/bench-programs/callgrind.sh -r 32 -j 16 -p "extcall rexxcps" -o $S/r2/rep32 base=$S/r2/bin/base/rexx-run
```
```
extcall 32 runs exlibc min 8281259754 max 8281259754
rexxcps 32 runs exlibc min 17817340311 max 17817354515
```

rexxcps's residual 14,204: between its min and max runs, +33 calls each to chrono
`Source::new`, `getenv`, `statx` (the program calls `TIME()`); `cgdiff.py` output quoted in
phase-6-perf.md. In the 3-round sitting every program but `rexxcps` and `heapshape` (also
`TIME()`) is identical to the instruction across all 12 runs.

### Ruling 3: SPREAD flag

`callgrind.sh -T` at `44c465281` over the old sitting 2 (`$S/cg2`) prints `SPREAD extcall pad2
0.4238%`, four `startup` lines (0.0114-0.0169%) and `SPREAD rexxcps pad2 0.0422%`, exit 1. The new
sitting: exit 0, no SPREAD line.

### Ruling 4: new base

Base `1754a3b5a`, sha256 `e500b628...de39b`, controls pad1/pad2/pad3 rebuilt from it (1
`Compiling rexx-exec` line each). Noise band 0 instructions on every program except `heapshape` (100)
and `rexxcps` (11,030, 0.00006%). Task 1 comparison deleted (the base moved).

Old base medians (`9d863ccc5`, sitting 2) to new: extcall -16.44%, startup -4.97%, sayloop -2.58%,
rexxcps -0.58%, alloc +0.27%, every other program within ±0.26%.

### Ruling 5 / I2 / M2

phase-6-perf.md now carries the oracle comparison command and its four same/same/0/0 lines, the
build, sha256, `.text`, `nm -S` and repeat-run commands with outputs, and the wall-clock command
(`wallclock.sh`, committed).

### Ruling 6 / M3

`Side::rust` env is empty again; `Side::for_axis(name)` adds `LD_LIBRARY_PATH` from
`child::axis_library_dir`, which answers only for `extcall`. Used by the suite's loop axes,
`rexx-arms` and the criterion harness. Test `only_extcall_gets_a_library_path` asserts it on
extcall and absence on every program in `PROGRAMS` and `NOT_BENCHMARKED`.
`cargo test -p rexx-bench`: 30 + 10 + 10 passed.

### Wall clock

pad2 against base, `wallclock.sh -r 5`: -7.69% (`startup`, 0.026 s) to +6.10% (`decloop`, 0.246 s).
`decloop` and `startup` are outside the spec's ±4%. `extcall` base 0.930 s, oracle 0.546 s (1.70x).

### Gates

At `d1086bb6e`, status.txt verbatim:

```
d1086bb6e3f29fc2af8f18678d45d6d656b1157e
started 2026-09-29T17:39:28+02:00
load at start 0.97 4.02 7.50 3/1906 2928299
G1 fmt exit 0
G2 clippy(empty target) exit 0
G3 release build (test --no-run) exit 0
load G4 3.79 4.54 7.63 1/1912 2930291 2026-09-29T17:39:39+02:00
G4 release test exit 0
G4 Compiling lines: 0
load after G4 5.83 6.32 7.64 8/1958 3058879
G5 debug build (test --no-run) exit 0
load G6 5.83 6.32 7.64 13/1967 3058892 2026-09-29T17:45:41+02:00
G6 debug test exit 0
G6 Compiling lines: 0
load after G6 4.67 6.43 7.43 6/1943 3187517
d1086bb6e3f29fc2af8f18678d45d6d656b1157e
finished 2026-09-29T17:51:58+02:00
```

Summed `test result` lines: G4 release 2780 passed 0 failed; G6 debug 2781 passed 0 failed.

### Concerns

1. Wall-clock layout noise on this sitting exceeds ±4% on `decloop` (+6.10%) and `startup`
   (-7.69%), both under 0.3 s.
2. `NativeObject::keys` order reaches program output unsorted; with FxHasher it is now the same on
   every run, where before it varied.
3. `alloc` +0.27% and `extcall` -16.44% from the hasher change are inside the new base, not measured
   as a separate Phase 6 change.

## Fix round 2

Commit `1cd896327`.

- N1: `rexx-bench-band.rs` builds the rust side per axis with `rust.for_axis(&axis)`.
  `./target/release/rexx-bench-band --axes extcall --pairs 1 --warmup 0` (from `rust/`): exit 0,
  rows `oracle 0.558552 ... 3000000` and `rust-ir 0.933181 ... 3000000`. Control: `rexx-run
  extcall.rex` with `LD_LIBRARY_PATH` unset exits 158, `Error 98 ... line 9`.
- N2: `wallclock.sh` writes each run's exit status to `<program>.<arm>.r<round>.rc`, compares
  stdout with the first binary's round 1 except on `heapshape` and `rexxcps`, and exits 1 on
  either. `wallclock.sh -r 1 -o $S/wf base=$S/r2/bin/base/rexx-run false=/bin/false`: exit 1, 23
  "exited 1" lines, 21 "stdout differs" lines. `wallclock.sh -r 1 -x extcall -o $S/wt
  base=$S/r2/bin/base/rexx-run pad2=$S/r2/bin/pad2/rexx-run`: exit 0.
- `cargo fmt --all --check` exit 0; `cargo clippy -p rexx-bench --all-targets -- -D warnings`
  clean; `cargo test -p rexx-bench` 30 + 10 + 10 passed, 0 failed. No gate run.
