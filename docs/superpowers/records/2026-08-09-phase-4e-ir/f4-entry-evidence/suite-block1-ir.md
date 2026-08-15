### Provenance

| | |
|---|---|
| measured | 2026-08-11T12:40:40+02:00 |
| repo commit | `131f1b4061545fcd180346c1a470a1fd9da8bcc4` |
| oracle `bin/rexx` | `/home/moritz/dev/repos/ooRexx/build/bin/rexx` -- size=62600 bytes, mtime=2026-08-05 16:02:20.172072564 +0200, sha256=bb5bb8ccbb96c376e329b91aafdad891f975ba06c941dbceba82c3848fa13019 |
| oracle `lib/librexx.so.4` | size=17853856 bytes, mtime=2026-08-05 16:02:20.064306594 +0200, sha256=42136c4038004fe2d5104181e06873301f032ced97c54a0fe84042e006d9b6fb |
| oracle `lib/librexxapi.so.4` | size=667792 bytes, mtime=2026-07-30 23:06:46.077533141 +0200, sha256=3536b76379c23fc7e3c4bce97b57d3c4812291ce5d68e71adc507ee690dc7d66 |
| this crate `rexx-run` | `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run` -- size=13819064 bytes, mtime=2026-08-11 12:37:37.314532195 +0200, sha256=f166bb30215747c9753379d2009011f8956c6695bd5efce130711a1999c98255 |
| this crate's engine | `REXX_ENGINE=ir`, set by this harness on every run |
| address-space cap | `ulimit -v 8388608` KiB, **both sides, every axis** |
| pairs per axis | 9 sampled, 1 warm-up pair(s) discarded, oracle and this crate alternating |
| pairs for the offset line | 51 sampled, 5 warm-up |
| statistic | median; interval is the distribution-free sign-test interval for the median at a 95% target |
| working directory of every child | a fresh empty temporary directory |

### Fixed per-process offset (`startup.rex`)

**Not comparable, and not a pass.** This crate has no `CoreClasses.orx` bootstrap yet (Phase 5), so it starts fast by not doing the work the oracle does at startup. The two numbers below are each side's own fixed cost, reported so every axis above can be read net of it -- not as a result about which interpreter starts faster.

| side | median | min | max | 95.1% interval | spread |
|---|---:|---:|---:|---|---:|
| oracle | 5.867 ms | 4.169 ms | 7.862 ms | 5.516 - 6.286 ms | 62.9 % |
| this crate | 1.546 ms | 1.124 ms | 2.616 ms | 1.468 - 1.827 ms | 96.5 % |

Both offsets include one `/bin/sh` `exec` from the `ulimit` wrapper, on both sides equally.

### Axes

`iters/s` is the program's own loop bound divided by the median wall time. `iters/s net` divides by the median wall time less that side's per-process offset above.

| axis | iterations | side | median | min | max | interval | spread | iters/s | iters/s net |
|---|---:|---|---:|---:|---:|---|---:|---:|---:|
| `alloc4c` | 1000000 | oracle | 1.1451 s | 1.1401 s | 1.1691 s | 1.1410 - 1.1624 s | 2.54 % | 873279 | 877776 |
| `alloc4c` | 1000000 | this crate | 2.2789 s | 2.2702 s | 2.2842 s | 2.2770 - 2.2841 s | 0.61 % | 438815 | 439113 |
| `arith` | 500000 | oracle | 1.1588 s | 1.1503 s | 1.1757 s | 1.1545 - 1.1684 s | 2.19 % | 431489 | 433684 |
| `arith` | 500000 | this crate | 3.0864 s | 3.0544 s | 3.1082 s | 3.0606 - 3.0998 s | 1.74 % | 162004 | 162085 |
| `compound` | 5000000 | oracle | 1.1415 s | 1.1295 s | 1.1621 s | 1.1296 - 1.1589 s | 2.86 % | 4380185 | 4402814 |
| `compound` | 5000000 | this crate | 6.7416 s | 6.6282 s | 6.7851 s | 6.7316 - 6.7548 s | 2.33 % | 741667 | 741837 |
| `emptyloop` | 25000000 | oracle | 0.8977 s | 0.8822 s | 0.9071 s | 0.8894 - 0.9043 s | 2.78 % | 27848535 | 28031731 |
| `emptyloop` | 25000000 | this crate | 2.8380 s | 2.8211 s | 2.9339 s | 2.8306 - 2.8862 s | 3.97 % | 8809028 | 8813830 |
| `strings` | 3000000 | oracle | 0.8635 s | 0.8449 s | 0.8942 s | 0.8544 - 0.8915 s | 5.71 % | 3474377 | 3498146 |
| `strings` | 3000000 | this crate | 9.1414 s | 9.1368 s | 9.1938 s | 9.1385 - 9.1772 s | 0.62 % | 328177 | 328232 |
| `varlookup` | 19000000 | oracle | 1.2143 s | 1.2007 s | 1.2692 s | 1.2054 - 1.2275 s | 5.65 % | 15647028 | 15722994 |
| `varlookup` | 19000000 | this crate | 4.5522 s | 4.5370 s | 4.5745 s | 4.5476 - 4.5612 s | 0.82 % | 4173805 | 4175223 |

#### Ratios and the gate call

The throughput ratio (oracle iters/s over this crate's) and the wall ratio (this crate's median over the oracle's) are the same number, because both sides run the same iteration count.

**The ratio interval is indicative, and the verdict is not taken from it.** It divides one side's interval by the other's, so its joint coverage is at least 92.2% by Bonferroni -- one minus the two sides' miss probabilities added -- not the 96.1% either side carries alone. The verdict applies Global Constraints' rule directly: this crate's point estimate against the oracle's interval, slow side.

| axis | oracle median | this crate median | ratio | ratio interval | verdict |
|---|---:|---:|---:|---|---|
| `alloc4c` | 1.1451 s | 2.2789 s | 1.99x | 1.96x - 2.00x | SLOWER |
| `arith` | 1.1588 s | 3.0864 s | 2.66x | 2.62x - 2.68x | SLOWER |
| `compound` | 1.1415 s | 6.7416 s | 5.91x | 5.81x - 5.98x | SLOWER |
| `emptyloop` | 0.8977 s | 2.8380 s | 3.16x | 3.13x - 3.24x | SLOWER |
| `strings` | 0.8635 s | 9.1414 s | 10.59x | 10.25x - 10.74x | SLOWER |
| `varlookup` | 1.2143 s | 4.5522 s | 3.75x | 3.70x - 3.78x | SLOWER |

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
| oracle | 1.8177 s | 1.8040 - 1.8408 s | `Averaged: 200 x 100 iterations of 1000 clauses (over 1.2s)` |
| this crate | 4.7507 s | 4.7325 - 4.7780 s | `Averaged: 100 x 100 iterations of 1000 clauses (over 4.4s)` |

| side | median cps | min | max | 96.1% interval | spread |
|---|---:|---:|---:|---|---:|
| oracle | 16682668 | 16396561 | 16867246 | 16442825 - 16758040 | 2.82 % |
| this crate | 2274309 | 2253314 | 2284238 | 2261865 - 2279910 | 1.36 % |

**Internal cps ratio: 7.34x** (oracle median over this crate's median), interval 7.21x - 7.41x.

### Axes this crate cannot run

Measured here rather than left out of the table, with the status and message each one actually produced. These belong to later tasks in this phase; what belongs to this one is that they are visible.

| axis | exit status | message |
|---|---:|---|
| `alloc` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
| `dispatch` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
| `heapshape` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |

