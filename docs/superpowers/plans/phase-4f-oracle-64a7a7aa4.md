### Provenance

| | |
|---|---|
| measured | 2026-08-12T12:03:44+02:00 |
| repo commit | `64a7a7aa4762e6f1c1ac6b667df976bf17aeced3` |
| oracle `bin/rexx` | `/home/moritz/dev/repos/ooRexx/build/bin/rexx` -- size=62600 bytes, mtime=2026-08-05 16:02:20.172072564 +0200, sha256=bb5bb8ccbb96c376e329b91aafdad891f975ba06c941dbceba82c3848fa13019 |
| oracle `lib/librexx.so.4` | size=17853856 bytes, mtime=2026-08-05 16:02:20.064306594 +0200, sha256=42136c4038004fe2d5104181e06873301f032ced97c54a0fe84042e006d9b6fb |
| oracle `lib/librexxapi.so.4` | size=667792 bytes, mtime=2026-07-30 23:06:46.077533141 +0200, sha256=3536b76379c23fc7e3c4bce97b57d3c4812291ce5d68e71adc507ee690dc7d66 |
| this crate `rexx-run` | `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run` -- size=13848192 bytes, mtime=2026-08-12 11:47:20.051835056 +0200, sha256=4178780bd19451051875c6b6fb5467062807a8edf0400e7d0c29dadd21b14c6c |
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
| oracle | 5.194 ms | 3.887 ms | 6.929 ms | 4.954 - 5.406 ms | 58.6 % |
| this crate | 1.576 ms | 1.136 ms | 2.285 ms | 1.464 - 1.671 ms | 72.9 % |

Both offsets include one `/bin/sh` `exec` from the `ulimit` wrapper, on both sides equally.

### Axes

`iters/s` is the program's own loop bound divided by the median wall time. `iters/s net` divides by the median wall time less that side's per-process offset above.

| axis | iterations | side | median | min | max | interval | spread | iters/s | iters/s net |
|---|---:|---|---:|---:|---:|---|---:|---:|---:|
| `alloc4c` | 1000000 | oracle | 1.1540 s | 1.1270 s | 1.1791 s | 1.1355 - 1.1597 s | 4.52 % | 866518 | 870435 |
| `alloc4c` | 1000000 | this crate | 1.3461 s | 1.3274 s | 1.3743 s | 1.3309 - 1.3693 s | 3.49 % | 742882 | 743752 |
| `arith` | 500000 | oracle | 1.1602 s | 1.1499 s | 1.1691 s | 1.1541 - 1.1682 s | 1.65 % | 430945 | 432883 |
| `arith` | 500000 | this crate | 2.2343 s | 2.2227 s | 2.2462 s | 2.2248 - 2.2371 s | 1.05 % | 223782 | 223940 |
| `compound` | 5000000 | oracle | 1.1440 s | 1.1283 s | 1.1564 s | 1.1318 - 1.1485 s | 2.45 % | 4370714 | 4390649 |
| `compound` | 5000000 | this crate | 2.7075 s | 2.6922 s | 2.8312 s | 2.6957 - 2.7235 s | 5.14 % | 1846701 | 1847776 |
| `emptyloop` | 25000000 | oracle | 0.8998 s | 0.8857 s | 0.9702 s | 0.8871 - 0.9577 s | 9.39 % | 27782894 | 27944192 |
| `emptyloop` | 25000000 | this crate | 1.5167 s | 1.5118 s | 1.5271 s | 1.5131 - 1.5260 s | 1.01 % | 16482726 | 16499865 |
| `strings` | 3000000 | oracle | 0.8660 s | 0.8485 s | 0.9027 s | 0.8501 - 0.8838 s | 6.26 % | 3464271 | 3485174 |
| `strings` | 3000000 | this crate | 4.3154 s | 4.2779 s | 4.4018 s | 4.2996 - 4.3489 s | 2.87 % | 695188 | 695441 |
| `varlookup` | 19000000 | oracle | 1.2102 s | 1.1963 s | 1.2303 s | 1.2017 - 1.2238 s | 2.81 % | 15699469 | 15767137 |
| `varlookup` | 19000000 | this crate | 2.5537 s | 2.5464 s | 2.5655 s | 2.5480 - 2.5606 s | 0.75 % | 7440085 | 7444678 |

#### Ratios and the gate call

The throughput ratio (oracle iters/s over this crate's) and the wall ratio (this crate's median over the oracle's) are the same number, because both sides run the same iteration count.

**The ratio interval is indicative, and the verdict is not taken from it.** It divides one side's interval by the other's, so its joint coverage is at least 92.2% by Bonferroni -- one minus the two sides' miss probabilities added -- not the 96.1% either side carries alone. The verdict applies Global Constraints' rule directly: this crate's point estimate against the oracle's interval, slow side.

| axis | oracle median | this crate median | ratio | ratio interval | verdict |
|---|---:|---:|---:|---|---|
| `alloc4c` | 1.1540 s | 1.3461 s | 1.17x | 1.15x - 1.21x | SLOWER |
| `arith` | 1.1602 s | 2.2343 s | 1.93x | 1.90x - 1.94x | SLOWER |
| `compound` | 1.1440 s | 2.7075 s | 2.37x | 2.35x - 2.41x | SLOWER |
| `emptyloop` | 0.8998 s | 1.5167 s | 1.69x | 1.58x - 1.72x | SLOWER |
| `strings` | 0.8660 s | 4.3154 s | 4.98x | 4.86x - 5.12x | SLOWER |
| `varlookup` | 1.2102 s | 2.5537 s | 2.11x | 2.08x - 2.13x | SLOWER |

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
| oracle | 1.8073 s | 1.7963 - 1.8195 s | `Averaged: 200 x 100 iterations of 1000 clauses (over 1.2s)` |
| this crate | 3.3751 s | 3.3455 - 3.3807 s | `Averaged: 100 x 100 iterations of 1000 clauses (over 3.6s)` |

| side | median cps | min | max | 96.1% interval | spread |
|---|---:|---:|---:|---|---:|
| oracle | 16675477 | 16520051 | 16923581 | 16620752 - 16847397 | 2.42 % |
| this crate | 2968598 | 2767269 | 2999292 | 2963031 - 2994691 | 7.82 % |

**Internal cps ratio: 5.62x** (oracle median over this crate's median), interval 5.55x - 5.69x.

### Axes this crate cannot run

Measured here rather than left out of the table, with the status and message each one actually produced. These belong to later tasks in this phase; what belongs to this one is that they are visible.

| axis | exit status | message |
|---|---:|---|
| `alloc` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
| `dispatch` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
| `heapshape` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |

