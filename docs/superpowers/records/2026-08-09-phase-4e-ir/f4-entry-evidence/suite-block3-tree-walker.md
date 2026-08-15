### Provenance

| | |
|---|---|
| measured | 2026-08-11T12:54:34+02:00 |
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
| oracle | 5.741 ms | 4.156 ms | 7.534 ms | 5.289 - 6.046 ms | 58.8 % |
| this crate | 1.508 ms | 1.137 ms | 2.525 ms | 1.430 - 1.676 ms | 92.1 % |

Both offsets include one `/bin/sh` `exec` from the `ulimit` wrapper, on both sides equally.

### Axes

`iters/s` is the program's own loop bound divided by the median wall time. `iters/s net` divides by the median wall time less that side's per-process offset above.

| axis | iterations | side | median | min | max | interval | spread | iters/s | iters/s net |
|---|---:|---|---:|---:|---:|---|---:|---:|---:|
| `alloc4c` | 1000000 | oracle | 1.1601 s | 1.1457 s | 1.1730 s | 1.1531 - 1.1635 s | 2.35 % | 861964 | 866251 |
| `alloc4c` | 1000000 | this crate | 2.2351 s | 2.2169 s | 2.2449 s | 2.2185 - 2.2438 s | 1.25 % | 447414 | 447716 |
| `arith` | 500000 | oracle | 1.1644 s | 1.1547 s | 1.1736 s | 1.1567 - 1.1729 s | 1.62 % | 429423 | 431551 |
| `arith` | 500000 | this crate | 3.0943 s | 3.0732 s | 3.1170 s | 3.0807 - 3.0998 s | 1.41 % | 161585 | 161664 |
| `compound` | 5000000 | oracle | 1.1456 s | 1.1356 s | 1.1682 s | 1.1405 - 1.1540 s | 2.85 % | 4364624 | 4386607 |
| `compound` | 5000000 | this crate | 6.5551 s | 6.5149 s | 6.6838 s | 6.5220 - 6.6769 s | 2.58 % | 762763 | 762939 |
| `emptyloop` | 25000000 | oracle | 0.8939 s | 0.8737 s | 0.9041 s | 0.8846 - 0.8952 s | 3.40 % | 27968663 | 28149451 |
| `emptyloop` | 25000000 | this crate | 2.7781 s | 2.7728 s | 2.7828 s | 2.7742 - 2.7815 s | 0.36 % | 8999083 | 9003969 |
| `strings` | 3000000 | oracle | 0.8579 s | 0.8440 s | 0.8820 s | 0.8466 - 0.8814 s | 4.43 % | 3497115 | 3520675 |
| `strings` | 3000000 | this crate | 8.9837 s | 8.9248 s | 9.0738 s | 8.9636 - 9.0463 s | 1.66 % | 333937 | 333993 |
| `varlookup` | 19000000 | oracle | 1.2060 s | 1.1932 s | 1.2552 s | 1.1972 - 1.2460 s | 5.14 % | 15755181 | 15830539 |
| `varlookup` | 19000000 | this crate | 5.0348 s | 5.0236 s | 5.0516 s | 5.0281 - 5.0447 s | 0.55 % | 3773756 | 3774887 |

#### Ratios and the gate call

The throughput ratio (oracle iters/s over this crate's) and the wall ratio (this crate's median over the oracle's) are the same number, because both sides run the same iteration count.

**The ratio interval is indicative, and the verdict is not taken from it.** It divides one side's interval by the other's, so its joint coverage is at least 92.2% by Bonferroni -- one minus the two sides' miss probabilities added -- not the 96.1% either side carries alone. The verdict applies Global Constraints' rule directly: this crate's point estimate against the oracle's interval, slow side.

| axis | oracle median | this crate median | ratio | ratio interval | verdict |
|---|---:|---:|---:|---|---|
| `alloc4c` | 1.1601 s | 2.2351 s | 1.93x | 1.91x - 1.95x | SLOWER |
| `arith` | 1.1644 s | 3.0943 s | 2.66x | 2.63x - 2.68x | SLOWER |
| `compound` | 1.1456 s | 6.5551 s | 5.72x | 5.65x - 5.85x | SLOWER |
| `emptyloop` | 0.8939 s | 2.7781 s | 3.11x | 3.10x - 3.14x | SLOWER |
| `strings` | 0.8579 s | 8.9837 s | 10.47x | 10.17x - 10.68x | SLOWER |
| `varlookup` | 1.2060 s | 5.0348 s | 4.17x | 4.04x - 4.21x | SLOWER |

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
| oracle | 1.8047 s | 1.7980 - 1.8121 s | `Averaged: 200 x 100 iterations of 1000 clauses (over 1.2s)` |
| this crate | 4.6061 s | 4.5891 - 4.6230 s | `Averaged: 100 x 100 iterations of 1000 clauses (over 4.3s)` |

| side | median cps | min | max | 96.1% interval | spread |
|---|---:|---:|---:|---|---:|
| oracle | 16757296 | 16702926 | 16917068 | 16739946 - 16841042 | 1.28 % |
| this crate | 2352335 | 2334689 | 2359565 | 2343825 - 2357417 | 1.06 % |

**Internal cps ratio: 7.12x** (oracle median over this crate's median), interval 7.10x - 7.19x.

### Axes this crate cannot run

Measured here rather than left out of the table, with the status and message each one actually produced. These belong to later tasks in this phase; what belongs to this one is that they are visible.

| axis | exit status | message |
|---|---:|---|
| `alloc` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
| `dispatch` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
| `heapshape` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |

