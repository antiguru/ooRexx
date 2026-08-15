### Provenance

| | |
|---|---|
| measured | 2026-08-11T13:01:30+02:00 |
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
| oracle | 6.175 ms | 4.593 ms | 8.067 ms | 5.578 - 6.400 ms | 56.2 % |
| this crate | 1.555 ms | 1.066 ms | 2.385 ms | 1.493 - 1.728 ms | 84.9 % |

Both offsets include one `/bin/sh` `exec` from the `ulimit` wrapper, on both sides equally.

### Axes

`iters/s` is the program's own loop bound divided by the median wall time. `iters/s net` divides by the median wall time less that side's per-process offset above.

| axis | iterations | side | median | min | max | interval | spread | iters/s | iters/s net |
|---|---:|---|---:|---:|---:|---|---:|---:|---:|
| `alloc4c` | 1000000 | oracle | 1.1579 s | 1.1437 s | 1.1656 s | 1.1438 - 1.1641 s | 1.89 % | 863603 | 868233 |
| `alloc4c` | 1000000 | this crate | 2.2793 s | 2.2667 s | 2.2875 s | 2.2674 - 2.2867 s | 0.91 % | 438726 | 439026 |
| `arith` | 500000 | oracle | 1.1566 s | 1.1491 s | 1.1761 s | 1.1517 - 1.1685 s | 2.33 % | 432300 | 434620 |
| `arith` | 500000 | this crate | 3.0859 s | 3.0640 s | 3.0985 s | 3.0720 - 3.0975 s | 1.12 % | 162029 | 162111 |
| `compound` | 5000000 | oracle | 1.1448 s | 1.1290 s | 1.1766 s | 1.1388 - 1.1476 s | 4.16 % | 4367706 | 4391394 |
| `compound` | 5000000 | this crate | 6.7435 s | 6.6775 s | 6.7800 s | 6.7078 - 6.7695 s | 1.52 % | 741457 | 741628 |
| `emptyloop` | 25000000 | oracle | 0.8825 s | 0.8798 s | 0.9445 s | 0.8799 - 0.9012 s | 7.33 % | 28327645 | 28527248 |
| `emptyloop` | 25000000 | this crate | 2.8255 s | 2.8166 s | 2.9176 s | 2.8172 - 2.9050 s | 3.57 % | 8847910 | 8852781 |
| `strings` | 3000000 | oracle | 0.8658 s | 0.8556 s | 0.8873 s | 0.8622 - 0.8717 s | 3.66 % | 3465060 | 3489951 |
| `strings` | 3000000 | this crate | 9.1493 s | 9.0536 s | 9.1841 s | 9.0881 - 9.1658 s | 1.43 % | 327894 | 327950 |
| `varlookup` | 19000000 | oracle | 1.2193 s | 1.2083 s | 1.2544 s | 1.2101 - 1.2359 s | 3.77 % | 15582682 | 15662000 |
| `varlookup` | 19000000 | this crate | 4.5516 s | 4.5333 s | 4.6019 s | 4.5414 - 4.5825 s | 1.51 % | 4174396 | 4175823 |

#### Ratios and the gate call

The throughput ratio (oracle iters/s over this crate's) and the wall ratio (this crate's median over the oracle's) are the same number, because both sides run the same iteration count.

**The ratio interval is indicative, and the verdict is not taken from it.** It divides one side's interval by the other's, so its joint coverage is at least 92.2% by Bonferroni -- one minus the two sides' miss probabilities added -- not the 96.1% either side carries alone. The verdict applies Global Constraints' rule directly: this crate's point estimate against the oracle's interval, slow side.

| axis | oracle median | this crate median | ratio | ratio interval | verdict |
|---|---:|---:|---:|---|---|
| `alloc4c` | 1.1579 s | 2.2793 s | 1.97x | 1.95x - 2.00x | SLOWER |
| `arith` | 1.1566 s | 3.0859 s | 2.67x | 2.63x - 2.69x | SLOWER |
| `compound` | 1.1448 s | 6.7435 s | 5.89x | 5.85x - 5.94x | SLOWER |
| `emptyloop` | 0.8825 s | 2.8255 s | 3.20x | 3.13x - 3.30x | SLOWER |
| `strings` | 0.8658 s | 9.1493 s | 10.57x | 10.43x - 10.63x | SLOWER |
| `varlookup` | 1.2193 s | 4.5516 s | 3.73x | 3.67x - 3.79x | SLOWER |

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
| oracle | 1.8111 s | 1.8041 - 1.8355 s | `Averaged: 200 x 100 iterations of 1000 clauses (over 1.2s)` |
| this crate | 4.7609 s | 4.7463 - 4.7636 s | `Averaged: 100 x 100 iterations of 1000 clauses (over 4.4s)` |

| side | median cps | min | max | 96.1% interval | spread |
|---|---:|---:|---:|---|---:|
| oracle | 16693627 | 16378313 | 16884776 | 16502331 - 16860165 | 3.03 % |
| this crate | 2268800 | 2260173 | 2276934 | 2265101 - 2273929 | 0.74 % |

**Internal cps ratio: 7.36x** (oracle median over this crate's median), interval 7.26x - 7.44x.

### Axes this crate cannot run

Measured here rather than left out of the table, with the status and message each one actually produced. These belong to later tasks in this phase; what belongs to this one is that they are visible.

| axis | exit status | message |
|---|---:|---|
| `alloc` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
| `dispatch` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
| `heapshape` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |

