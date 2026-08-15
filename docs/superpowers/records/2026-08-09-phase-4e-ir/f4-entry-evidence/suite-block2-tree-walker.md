### Provenance

| | |
|---|---|
| measured | 2026-08-11T12:47:38+02:00 |
| repo commit | `131f1b4061545fcd180346c1a470a1fd9da8bcc4` |
| oracle `bin/rexx` | `/home/moritz/dev/repos/ooRexx/build/bin/rexx` -- size=62600 bytes, mtime=2026-08-05 16:02:20.172072564 +0200, sha256=bb5bb8ccbb96c376e329b91aafdad891f975ba06c941dbceba82c3848fa13019 |
| oracle `lib/librexx.so.4` | size=17853856 bytes, mtime=2026-08-05 16:02:20.064306594 +0200, sha256=42136c4038004fe2d5104181e06873301f032ced97c54a0fe84042e006d9b6fb |
| oracle `lib/librexxapi.so.4` | size=667792 bytes, mtime=2026-07-30 23:06:46.077533141 +0200, sha256=3536b76379c23fc7e3c4bce97b57d3c4812291ce5d68e71adc507ee690dc7d66 |
| this crate `rexx-run` | `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run` -- size=13819064 bytes, mtime=2026-08-11 12:37:37.314532195 +0200, sha256=f166bb30215747c9753379d2009011f8956c6695bd5efce130711a1999c98255 |
| this crate's engine | `REXX_ENGINE=tree-walker`, set by this harness on every run |
| address-space cap | `ulimit -v 8388608` KiB, **both sides, every axis** |
| pairs per axis | 9 sampled, 1 warm-up pair(s) discarded, oracle and this crate alternating |
| pairs for the offset line | 51 sampled, 5 warm-up |
| statistic | median; interval is the distribution-free sign-test interval for the median at a 95% target |
| working directory of every child | a fresh empty temporary directory |

### Fixed per-process offset (`startup.rex`)

**Not comparable, and not a pass.** This crate has no `CoreClasses.orx` bootstrap yet (Phase 5), so it starts fast by not doing the work the oracle does at startup. The two numbers below are each side's own fixed cost, reported so every axis above can be read net of it -- not as a result about which interpreter starts faster.

| side | median | min | max | 95.1% interval | spread |
|---|---:|---:|---:|---|---:|
| oracle | 6.175 ms | 4.186 ms | 7.152 ms | 5.837 - 6.437 ms | 48.0 % |
| this crate | 1.624 ms | 1.208 ms | 2.382 ms | 1.537 - 1.743 ms | 72.2 % |

Both offsets include one `/bin/sh` `exec` from the `ulimit` wrapper, on both sides equally.

### Axes

`iters/s` is the program's own loop bound divided by the median wall time. `iters/s net` divides by the median wall time less that side's per-process offset above.

| axis | iterations | side | median | min | max | interval | spread | iters/s | iters/s net |
|---|---:|---|---:|---:|---:|---|---:|---:|---:|
| `alloc4c` | 1000000 | oracle | 1.1508 s | 1.1417 s | 1.1656 s | 1.1467 - 1.1557 s | 2.08 % | 868966 | 873654 |
| `alloc4c` | 1000000 | this crate | 2.2306 s | 2.2171 s | 2.2566 s | 2.2209 - 2.2331 s | 1.77 % | 448306 | 448632 |
| `arith` | 500000 | oracle | 1.1566 s | 1.1545 s | 1.1616 s | 1.1551 - 1.1585 s | 0.61 % | 432289 | 434609 |
| `arith` | 500000 | this crate | 3.0974 s | 3.0739 s | 3.1294 s | 3.0756 - 3.1257 s | 1.79 % | 161426 | 161511 |
| `compound` | 5000000 | oracle | 1.1435 s | 1.1292 s | 1.1570 s | 1.1310 - 1.1523 s | 2.43 % | 4372406 | 4396144 |
| `compound` | 5000000 | this crate | 6.5360 s | 6.5004 s | 6.5591 s | 6.5083 - 6.5585 s | 0.90 % | 764992 | 765182 |
| `emptyloop` | 25000000 | oracle | 0.8935 s | 0.8837 s | 0.9162 s | 0.8924 - 0.9059 s | 3.64 % | 27978921 | 28173616 |
| `emptyloop` | 25000000 | this crate | 2.7920 s | 2.7772 s | 2.8049 s | 2.7810 - 2.8012 s | 0.99 % | 8954040 | 8959251 |
| `strings` | 3000000 | oracle | 0.8571 s | 0.8301 s | 0.8848 s | 0.8415 - 0.8622 s | 6.38 % | 3500298 | 3525699 |
| `strings` | 3000000 | this crate | 9.0338 s | 8.9979 s | 9.1682 s | 9.0090 - 9.1115 s | 1.88 % | 332084 | 332144 |
| `varlookup` | 19000000 | oracle | 1.2008 s | 1.1948 s | 1.2141 s | 1.1959 - 1.2101 s | 1.60 % | 15822618 | 15904401 |
| `varlookup` | 19000000 | this crate | 5.0424 s | 5.0261 s | 5.0668 s | 5.0321 - 5.0531 s | 0.81 % | 3768022 | 3769236 |

#### Ratios and the gate call

The throughput ratio (oracle iters/s over this crate's) and the wall ratio (this crate's median over the oracle's) are the same number, because both sides run the same iteration count.

**The ratio interval is indicative, and the verdict is not taken from it.** It divides one side's interval by the other's, so its joint coverage is at least 92.2% by Bonferroni -- one minus the two sides' miss probabilities added -- not the 96.1% either side carries alone. The verdict applies Global Constraints' rule directly: this crate's point estimate against the oracle's interval, slow side.

| axis | oracle median | this crate median | ratio | ratio interval | verdict |
|---|---:|---:|---:|---|---|
| `alloc4c` | 1.1508 s | 2.2306 s | 1.94x | 1.92x - 1.95x | SLOWER |
| `arith` | 1.1566 s | 3.0974 s | 2.68x | 2.65x - 2.71x | SLOWER |
| `compound` | 1.1435 s | 6.5360 s | 5.72x | 5.65x - 5.80x | SLOWER |
| `emptyloop` | 0.8935 s | 2.7920 s | 3.12x | 3.07x - 3.14x | SLOWER |
| `strings` | 0.8571 s | 9.0338 s | 10.54x | 10.45x - 10.83x | SLOWER |
| `varlookup` | 1.2008 s | 5.0424 s | 4.20x | 4.16x - 4.23x | SLOWER |

#### Same work on both sides

A wall time is only about the workload if the workload ran. Every sampled run on each side printed the same bytes, and the two sides printed the same bytes as each other.

| axis | stable within a side | identical across sides | stdout |
|---|---|---|---|
| `alloc4c` | yes | yes | `12888896` |
| `arith` | yes | yes | `4629643519330627.7808` |
| `compound` | yes | yes | `5000000` |
| `emptyloop` | yes | yes | `done` |
| `strings` | yes | yes | `138000000` |
| `varlookup` | yes | yes | `19000000` |

### `samples/rexxcps.rex`

The oracle's own clauses-per-second benchmark, run from the read-only C++ tree. It self-calibrates: a trial that comes in at or under a second is run again at twice the count, so the two sides do **different amounts of work** and their wall times are not directly comparable. The clauses-per-second figure each side prints is per clause and is the comparable one. Each side's `Averaged:` line is quoted so the asymmetry is visible rather than inferred.

| side | wall median | wall interval | `Averaged:` |
|---|---:|---|---|
| oracle | 1.8072 s | 1.7902 - 1.8230 s | `Averaged: 200 x 100 iterations of 1000 clauses (over 1.2s)` |
| this crate | 4.6103 s | 4.5937 - 4.6236 s | `Averaged: 100 x 100 iterations of 1000 clauses (over 4.3s)` |

| side | median cps | min | max | 96.1% interval | spread |
|---|---:|---:|---:|---|---:|
| oracle | 16685605 | 16437352 | 16951078 | 16552715 - 16901228 | 3.08 % |
| this crate | 2347179 | 2338984 | 2358464 | 2340816 - 2355426 | 0.83 % |

**Internal cps ratio: 7.11x** (oracle median over this crate's median), interval 7.03x - 7.22x.

### Axes this crate cannot run

Measured here rather than left out of the table, with the status and message each one actually produced. These belong to later tasks in this phase; what belongs to this one is that they are visible.

| axis | exit status | message |
|---|---:|---|
| `alloc` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
| `dispatch` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
| `heapshape` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |

