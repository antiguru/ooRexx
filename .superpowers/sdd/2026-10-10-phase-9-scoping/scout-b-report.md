# Scout B report: where performance parity stands at `a98238be5`

Read-only survey, 2026-10-10. HEAD confirmed `a98238be5` (`git rev-parse --short HEAD`). Machine quiet:
`uptime` before the suite `22:12:31 load average: 0.16, 0.58, 0.81`, after `22:18:21 load average: 1.06, 0.95, 0.91`
(the one-minute figure at the end is the suite itself). One idle `rexx-run p.rex` from 21:50 belonging to another
session was present, at 0% CPU. Scratch: `/tmp/claude-1000/p9scope/b/` (`suite.md` full suite output, `s/three.tsv`
raw three-way counts, `s/run3.sh` and `s/med.py` the scripts).

## 1. What "every benchmark" can mean, and what parity is defined as

**Programs.** `rust/bench-programs/*.rex` holds 23 timed programs plus `pingpong/` (3, criterion 7 of Phase 6, not
suite axes) and `heapshape`. `rexx-bench-suite`'s `AXES` runs 23 entries: 21 `Loop` axes (`alloc alloc4c arith assign
compound decloop decrender dirread dispatch dispatchclass emptyloop extcall fibcall fibfunc nop parse sayloop sendloop
strings textnum varlookup`), `startup` as `Role::Offset`, `heapshape` as `Role::SelfTimed`, plus
`rust/bench-rexxcps/rexxcps.rex` as its own section. Command: `grep -n 'name:' rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs`;
the suite's own table lists them all. **Not in the suite:** `layout-pad.py` and the three `pingpong/` programs (hand-run by
`callgrind.sh`). `samples/` whole-program benchmarks: D9 says "Whole-program benchmarks come from `samples/`", and the only one
the tree measures is `samples/rexxcps.rex`, replaced for timing by the fixed-count `rust/bench-rexxcps/rexxcps.rex`
(README there: the stock file scales its own count to ~1 s per trial). No other `samples/` program is benchmarked anywhere
(inferred from `grep -rn 'samples/' docs/superpowers/plans/perf-baseline.md`, read, not re-derived).

**Definition of parity** (`docs/superpowers/plans/2026-07-27-rust-rewrite.md:45-46`, Global Constraints "Shipping gate (parity)"):
no phase from 2 on closes with a Rust subsystem slower than its C++ counterpart on the Phase 0 suite; "slower" means the
point estimate falls outside the C++ baseline's confidence interval on the slow side. Row 4 (`:655`) and the Phase 4
"debt route" paragraph (`:726`) say the debt route is **closed** for `arith`, `compound`, `strings`, `varlookup`,
`alloc4c` and `rexxcps`, and **survives only for `dispatch`, `alloc.rex` and `startup`** (scoped to Phase 5 work, not a
conceded bar). Every axis added since (`assign decloop decrender dirread extcall fibcall fibfunc nop parse sayloop sendloop
textnum`) has no stated debt status either way. Phase 6's S2-S5 and 6.1 explicitly **accepted** `dispatch`, `sendloop`,
`fibcall`, `fibfunc` over their Ir budget ("accept and record", Moritz 2026-10-07, `phase-6-gate.md:~1285`) so those four
are recorded debt by ruling, not by the row-4 paragraph. Row 9 text still reads "every benchmark at parity"; struck by
Moritz 2026-10-10 per the brief.

## 2. The oracle build

Two oracles exist on this machine. Neither is recorded in any perf record as a timing baseline except the first.

| | `ooRexx/build` | `ooRexx-rust-rewrite/build` |
|---|---|---|
| build type | `RelWithDebInfo`, `-O2 -g -DNDEBUG` (`grep CMAKE_BUILD_TYPE build/CMakeCache.txt`) | **`Release`, `-O3 -DNDEBUG`** (same command) |
| version | `parse version`: 5.3.0, 30 Jul 2026 | 5.3.0, **27 Jul 2026** |
| `bin/rexx` sha256 | `bb5bb8cc...3019` (matches every 2026-08-20 section of `perf-baseline.md`) | `b35fcb2e749d611bb9610733c03bd07f0f103b0d9ca9c99adbee63fed3668bb6` |
| `librexx.so.4` sha256 | `42136c40...b6` (17,853,856 B) | `863e39e048adbefb4177f1dea9aaa3a33aac22a085ee37b281bf1e19fcbd1b2a` (3,274,008 B, same hash as `rev2-task5-834282/build`) |
| in the repo's records | yes, the suite's hard-coded `ORACLE_ROOT` | **no**: `grep -rn 'ooRexx-rust-rewrite/build' docs` finds nothing about it as a baseline |

So `rust/CLAUDE.md`'s "no `-O3` oracle on this machine" is out of date: `ooRexx-rust-rewrite/build` is a gitignored
`Release` build of this checkout's `interpreter/`, built 2026-07-27 17:14, which runs (`rexx` answered `parse version`, rc 0).
It predates the 2026-07-30 interpreter state of `ooRexx/build` by a few days of upstream commits, so it is not
version-identical, only the same 5.3.0 line. The suite cannot use it (`ORACLE_ROOT` is a `const`, `child.rs:27`), so I
measured it with a separate three-way script (section 3b). Nothing was built.

## 3a. The suite at HEAD (against `ooRexx/build`, the suite's oracle)

Command: `memcap 4G ./target/release/rexx-bench-suite > suite.md` (exit 0, `suite.status`), the 2026-08-20 section's
method unchanged: 9 sampled pairs, 1 warm-up pair, oracle and crate alternating, `perf stat -e cycles:u,instructions:u`
on the same runs, `ulimit -v 8388608` both sides, fresh empty workdir. Crate `rexx-run` sha256 `17288b82f12f...306e`,
built at HEAD (`cargo build --release` printed `Compiling rexx-bench`, `rexx-run` was current; mtime 21:50 is after
the last code commit, HEAD's recent commits are docs). Every axis printed identical stdout on both sides and was stable
within a side ("Same work" table, no `no` cell: `grep -c '| no |' suite.md` = 0).

Ratios are crate over oracle, below 1.00 is the crate ahead. "09-10" is `paths-to-parity.md`'s head table (wall, 2026-09-10
`8565dcfa8`), or where that table lacks the axis, the nearest record named in the last column.

| axis | oracle wall | crate wall | **wall ratio** | **cycles ratio** | **instr ratio** | verdict | at `paths-to-parity` date |
|---|---:|---:|---:|---:|---:|---|---|
| `alloc` | 1.131 s | 1.582 s | 1.40x | 1.40x | 1.71x | SLOWER | 1.81x wall; 1.78x cycles |
| `alloc4c` | 0.956 s | 0.566 s | 0.59x | 0.58x | 0.79x | faster | 0.62x |
| `arith` | 1.154 s | 1.290 s | 1.12x | 1.12x | 1.09x | SLOWER | 1.17x |
| `assign` | 0.763 s | 1.026 s | 1.34x | 1.35x | 1.77x | SLOWER | not measured then |
| `compound` | 1.137 s | 0.640 s | 0.56x | 0.55x | 0.89x | faster | 0.63x |
| `decloop` | 0.158 s | 0.231 s | 1.46x | 1.52x | 1.44x | SLOWER | 1.63x (bench README) |
| `decrender` | 0.240 s | 0.424 s | 1.76x | 1.81x | 1.78x | SLOWER | 1.99x (bench README) |
| `dirread` | 0.060 s | 0.292 s | **4.84x** | **6.46x** | 7.17x | SLOWER | not measured then (Phase 6.1) |
| `dispatch` | 1.254 s | 1.956 s | 1.56x | 1.57x | 1.65x | SLOWER | 1.78x wall; **1.23x cycles** at `86ca524e6` |
| `dispatchclass` | 0.905 s | 1.474 s | 1.63x | 1.64x | 1.70x | SLOWER | 1.81x wall; **1.23x cycles** at `86ca524e6` |
| `emptyloop` | 0.890 s | 0.458 s | 0.51x | 0.50x | 0.62x | faster | 0.61x |
| `extcall` | 0.546 s | 1.102 s | **2.02x** | 2.05x | 1.50x | SLOWER | 1.70x (`phase-6-perf.md:201`) |
| `fibcall` | 0.567 s | 0.779 s | 1.37x | 1.39x | 1.66x | SLOWER | not measured then |
| `fibfunc` | 0.541 s | 0.781 s | 1.44x | 1.46x | 1.78x | SLOWER | not measured then |
| `nop` | 0.589 s | 0.621 s | 1.05x | 1.05x | **2.43x** | SLOWER | not measured then |
| `parse` | 0.120 s | 0.139 s | 1.16x | 1.20x | 1.31x | SLOWER | 1.48x instr (`bbe86427d`) |
| `sayloop` | 0.192 s | 0.040 s | 0.21x | 0.17x | 0.22x | faster | not measured then |
| `sendloop` | 1.040 s | 1.215 s | 1.17x | 1.17x | 1.38x | SLOWER | not measured then |
| `strings` | 0.861 s | 1.146 s | 1.33x | 1.34x | 1.68x | SLOWER | 1.80x wall; 1.79x cycles |
| `textnum` | 0.122 s | 0.103 s | 0.84x | 0.81x | 0.99x | faster | 1.12x instr |
| `varlookup` | 1.197 s | 0.691 s | 0.58x | 0.57x | 0.78x | faster | 0.76x |
| `rexxcps` (`bench-rexxcps`) | 1.172 s | 1.951 s | 1.66x (internal cps 1.66x, 1.63-1.69) | 1.68x | 1.81x | SLOWER | 1.97x |
| `startup` (offset, 51 pairs) | 18.771 ms | 33.856 ms | 1.80x | n/a | n/a | not a pass either way | 1.73x wall; 6.03x cycles, 13.34x instr |
| `heapshape` (self-timed) | | | gc pause 0.364x, build 0.697x | | | faster | 3.5x/1.45x debt earlier, now ahead |

Tally, read from the table: 21 loop axes, **15 SLOWER and 6 faster** (`alloc4c compound emptyloop sayloop textnum
varlookup`); plus `rexxcps` slower, `startup` offset 1.80x, `heapshape` ahead. Over 1.5x wall: `dirread`, `extcall`, `decrender`,
`dispatchclass`, `dispatch` (1.56x), `rexxcps`. The "09-10" column is wall; the cycles figures in the last column come
from `paths-to-parity.md:421-436`. Wall and cycles ratios agree within 0.06 on every axis except `dirread`, `sayloop`
(`perf`'s per-process cost matters on 0.04-0.06 s programs).

Movement since 2026-09-10 (same-column, read off the table above): `alloc` 1.81 to 1.40, `strings` 1.80 to 1.33,
`rexxcps` 1.97 to 1.66, `arith` 1.17 to 1.12, `decrender` 1.99 to 1.76, `decloop` 1.63 to 1.46, `parse` 1.48 (instr) to
1.31. **Regressed in cycles:** `dispatch` and `dispatchclass` from 1.23x cycles (`86ca524e6`) to 1.57x and 1.64x. The
regression lines up with Phase 6's accepted Ir overruns (`phase-6-perf.md:1313-1400`, S2-S5 gate: `dispatch` +2.4% to
+3.6%, `sendloop`, `fibfunc`), but those are +1 to +4%, not +30%. A different 09-10-to-now cause is therefore not
excluded; I did not bisect it (inferred, not run). The 09-10 `86ca524e6` figure is itself a different instrument
(single-event runs, 3 reps), so the 0.34 jump is not a clean A/B.

## 3b. The same crate against a Release (`-O3`) oracle

Script `s/run3.sh`: for 21 programs (all loop axes except `extcall`, plus `startup`), 1 discarded + 4 sampled rounds,
oracle Release, oracle `build/`, crate interleaved per round, one `perf stat -e cycles:u,instructions:u` per run, medians,
`ulimit -v 1048576` on every run, `memcap 4G` around the script, fresh empty workdir, all 252 runs rc 0
(`awk -F'\t' '$4!=0' three.tsv` printed nothing). Fewer rounds than the suite's 9; read ratios to about 0.05.

Columns: crate/Release cycles, crate/Release instructions, crate/`build/` cycles, and Release/`build/` cycles
(`s/med.txt`, `python3 -I med.py`).

| axis | cyc vs Release | instr vs Release | cyc vs `build/` | Release / `build/` cyc |
|---|---:|---:|---:|---:|
| `alloc` | 1.44 | 1.70 | 1.40 | 0.97 |
| `alloc4c` | 0.55 | 0.79 | 0.57 | 1.03 |
| `arith` | 1.10 | 1.09 | 1.11 | 1.01 |
| `assign` | 1.36 | 1.77 | 1.35 | 0.99 |
| `compound` | 0.55 | 0.89 | 0.55 | 1.01 |
| `decloop` | 1.47 | 1.43 | 1.52 | 1.03 |
| `decrender` | 1.89 | 1.77 | 1.82 | 0.97 |
| `dirread` | 6.81 | 7.14 | 6.27 | 0.92 |
| `dispatch` | 1.67 | 1.65 | 1.59 | 0.95 |
| `dispatchclass` | 1.73 | 1.70 | 1.67 | 0.97 |
| `emptyloop` | 0.51 | 0.61 | 0.51 | 1.00 |
| `fibcall` | 1.42 | 1.66 | 1.39 | 0.98 |
| `fibfunc` | 1.53 | 1.78 | 1.45 | 0.95 |
| `nop` | 1.01 | 2.43 | 1.01 | 0.99 |
| `parse` | 1.19 | 1.31 | 1.19 | 1.00 |
| `sayloop` | 0.18 | 0.22 | 0.18 | 1.01 |
| `sendloop` | 1.22 | 1.37 | 1.19 | 0.97 |
| `strings` | 1.34 | 1.68 | 1.34 | 1.00 |
| `textnum` | 0.80 | 0.98 | 0.83 | 1.03 |
| `varlookup` | 0.58 | 0.77 | 0.57 | 0.99 |
| `startup` | 6.04 | 12.88 | 5.76 | 0.95 |

**The `-O3` Release oracle is within 8% of `build/` on every axis (Release / `build/` 0.92 to 1.03) and the Release
oracle is the faster of the two on none by more than 8%.** The gap figures above therefore do not move with the oracle build; the
`-O3` caveat in `rust/CLAUDE.md` costs at most 0.08 on any axis here. Caveat: the two oracles differ in source by a few
days of commits as well as in flags, so this is not a clean flags-only A/B, and `rexxcps` and `extcall` were not run
three-way (the former lives in `bench-rexxcps/`, the latter needs `LD_LIBRARY_PATH` handling I did not add).

## 4. Shape: what the records attribute each over-parity axis to

I did not profile or optimise anything. Records are cited by path; "no record" means I found none.

* **`rexxcps` (1.66x), `assign` (1.34x wall, 1.77x instr), `nop` (1.05x wall, 2.43x instr), `emptyloop`-class clause
  cost.** Fixed per-clause machinery: 1,014.6 Ir per clause against the oracle's 534.4, of which interpretation
  machinery (driver, loop, operator dispatch) is +305.67/cl, 63.7% of the gap
  (`docs/superpowers/records/2026-09-20-performance-items/2026-09-22-category-rollup.md`). The driver's register
  allocation is the cost, not the dispatch table (`2026-09-22-driver-gap-report.md`, 37.6% spill/reload). Code generation of
  the hot path is worth about 13% of `rexxcps` (PGO in-sample, `2026-09-23-driver-spikes/README.md`). Our IPC is above the
  oracle's, so the gap is retired instructions (`paths-to-parity.md:421-445`). `nop` is the oracle at 33.3 Ir per clause
  against ours 90 (`2026-09-20-performance-todo.md:~1109`) while wall is near parity, the IPC gap hides it.
* **`alloc` (1.40x), `strings` (1.33x).** Value materialisation and string building: text/number conversion and string
  building +61.38/cl, the C allocator +45.02/cl, built-ins +23.29/cl (category rollup above); `paths-to-parity.md:674-700`
  ("allocation 2.06x, string materialisation 1.81x" on instructions, 2026-09-10). No record attributes `alloc.rex`'s
  remaining 1.40x since the hashing round of 2026-09-10 (`paths-to-parity.md` mentions the round at the table head).
* **`dispatch`, `dispatchclass`, `sendloop`, `fibcall`, `fibfunc` (1.17x to 1.63x).** Send/call path. Phase 6's gate records
  inlining lost on the send path, allocation through generic `RawVecInner::try_allocate_in`, and a larger entry block
  in `ops_loop_steady` (+44 Ir per `fibfunc` call): `phase-6-perf.md:1313-1400`, `perf/diagnosis.md` in
  `docs/superpowers/records/2026-10-01-phase-6-s2-s5/`. Over-budget accepted by Moritz 2026-10-07. D28 keeps message
  resolution dynamic with no per-call-site cache (roadmap `:735`).
* **`decloop` (1.46x), `decrender` (1.76x).** Decimal loop control spelling: 11.32% of `decrender` is cache-probe cost
  and `Number::format_with` plus malloc/free is 5.3% (`paths-to-parity.md:723-830`, "it is the *spelling*, not the
  fractional step"). Open at 2026-09-10; the 1.99x there is now 1.76x.
* **`parse` (1.16x wall, 1.31x instr).** PARSE +41.65/cl (category rollup); `parse in place` landed -0.68%
  (`2026-09-21-parse-in-place-report.txt`); `parse-cost-scout-report.md` in the same directory is the cost breakdown.
* **`arith` (1.12x).** Arithmetic and comparison +30.58/cl (category rollup), the shared decimal helpers
  (`tree-walker.md` row: "arithmetic helpers are shared"). Gap 1.09x instructions and 1.12x cycles, the smallest in the
  slower set. No record sets a bar for it.
* **`dirread` (4.84x wall, 6.46x cycles), the largest multiple among the loop axes.** `bench-programs/README.md`:
  "Added in Phase 6.1 from Task 7's measurement of that path; sized against this crate (0.3 s), not the oracle (0.05 s)",
  and the program's own header. **No record attributes the gap**: it was added as a regression guard for a Phase 6.1 path
  and was never compared against the oracle in a perf record before today (`grep -rln dirread docs .superpowers`
  finds only `phase-6-1-gate.md` and the README). It is a 0.06 s oracle run, so process-spawn noise is a larger share
  of the oracle denominator than on other axes, but 7.17x instructions is not noise.
* **`extcall` (2.02x wall, 1.50x instr, 2.05x cycles; IPC 2.65 against 3.62).** Native extension call
  (`liborxfunction`'s `TestIntArg`). `phase-6-gate.md:1299` records `extcall` +15.82% wall over s1 "not attributed"; the
  1.70x of `phase-6-perf.md:201` has no attribution either. This is the one axis where we retire fewer relative
  instructions than cycles: a stall shape, not an instruction-count shape (inferred from the IPC column).
* **`startup` (1.80x wall, 6.0x cycles, 12.9x instructions user-side).** This crate parses and installs the `.orx`
  sources at every start; the oracle restores a saved image. `paths-to-parity.md:103` "Path D -- startup, via the image";
  D2's cold-start gate. Instruction ratio 12.88 here against 13.34 at `86ca524e6`: unmoved. The crate's offset is
  33.9 ms now against 10.4 ms on 2026-08-20 (`perf-baseline.md:1388`), because Phase 5 loads the class library (inferred, not run), which the
  2026-08-20 note said it did not yet.

## What this changes for the Phase 9 spec

1. **Parity as a work item is 15 loop axes plus `rexxcps` and `startup`, not a handful.** The 2026-08-20 "four of six faster" standing
   is stale: of 21 loop axes, 6 are ahead. Row 4's closed-debt list (`arith compound strings varlookup alloc4c rexxcps`) still
   has `arith`, `strings` and `rexxcps` over, so the parity clause is unmet on three axes it says have no debt route.
2. **Moritz's "2x is achievable" is already true on every axis but three**: `dirread` (4.84x wall), `extcall` (2.02x), and
   `startup` (1.80x wall but 6x cycles, 12.9x instructions). `decrender` (1.76x) and `dispatchclass` (1.63x) are close.
3. **A non-regression bar for Phase 9 should be per axis against these figures**, and `dispatch`/`dispatchclass` are the axes
   with a recorded regression since 2026-09-10 (1.23x to 1.57-1.64x cycles) whose cause no record names.
4. **A Release oracle exists** (`ooRexx-rust-rewrite/build`, sha256 above), gitignored, unrecorded and 3 days older source than
   `ooRexx/build`. It shifts no ratio by more than 8%, so the `-O3` rule is satisfiable by recording this build's hashes in
   `perf-baseline.md` rather than building another. The suite cannot select it (`ORACLE_ROOT` is a const).
5. Machine: Linux only measured; the parity gate also names macOS, which no record here measured.
