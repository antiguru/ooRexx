# Task 26 report: the Phase 6 performance gate

Status: stopped at Step 3, over budget. Steps 2 and 4 not started; `phase-6-perf.md` and
`phase-6-gate.md` not written (they wait for the lead's ruling). Evidence:
`docs/superpowers/records/2026-10-01-phase-6-s2-s5/perf/`.

## Step 1

HEAD measured: `c484f4516` (no later commit touches `rust/`). `callgrind.sh` and every bench
program hash equal `s5-evidence/perfbase/script-hashes.txt` and `program-hashes.txt`
(`sha256sum -c` clean).

Builds: `perf/build.sh` (the perfbase recipe, controls omitted), scratch `/tmp/claude-1000/p6-t26`.
Each log one `Compiling rexx-exec` line, exit 0 (`perf/progress.txt`).

| name | rev | sha256 of `rexx-run` | `.text` bytes |
|---|---|---|---:|
| base | `1754a3b5a` | `6dcfcfd92ae84897e9c0127b41852bbe1f891c32bc70b82ea9aa945023ed36cd` | 2732523 |
| s1 | `1a81353e3` | `a41ae66fdee043891951ca14af4f5922b996a6b30a5df44cebe33903ba6d8cc1` | 2833355 |
| head | `c484f4516` | `f5ff3be9f249c60a95c21140271e3f6a091887d40467b4a8ff98e3f5cac7ab8b` | 3130478 |

Base and s1 `.text` sizes equal perfbase's.

```
bash rust/bench-programs/callgrind.sh -r 3 -j 16 -o $S/cg base=$S/bin/base/rexx-run s1=$S/bin/s1/rexx-run head=$S/bin/head/rexx-run
```

Exit 0 (`perf/cg-exit.txt`), stderr only the outputs line, no SPREAD flag; every spread 0.0000%
except base and head `rexxcps` 0.0001% (`perf/cg-table.txt`).

### Reproduction check: not exact

Base and s1 do not reproduce the perfbase Ir exactly. Per program, this run minus perfbase is
+913 to +1188 Ir for base and s1 on every program but `rexxcps` (base +1997, s1 -12328); derived by
joining the two `cg-table.txt` files on program. The s1-vs-base percentages equal perfbase's to four
places on every program but `rexxcps` (-0.8622 here, -0.8621 there).

Cause measured as the process environment, not the build (`perf/checks.txt`):
- `cgpath-table.txt`: the base and s1 binaries copied to the perfbase scratch paths give the same Ir
  as this run's base path (`startup` 58074163 both), so the binary path is not it.
- `cgenv-table.txt`: one extra environment variable moves base `startup` from 58074163 to 58074544
  (+381 Ir).

The perfbase controls were not re-run; their band (0 Ir except `rexxcps` 3,388 Ir) is used below.
`rexxcps` is a TIMED program and its s1 count moved 12,328 Ir between the two runs, more than that
band.

### Running total against base, delta against S1

Bar: +1.0% beyond the band; `fibfunc` +2.58%.

| program | base Ir | s1 Ir | head Ir | head vs base % | head vs s1 % | verdict |
|---|---:|---:|---:|---:|---:|---|
| alloc | 25168678552 | 20270018733 | 20306537732 | -19.3182 | +0.1802 | ok |
| alloc4c | 3224395147 | 3176498954 | 3140691245 | -2.5960 | -1.1273 | ok |
| arith | 11520060248 | 11498165931 | 11488858483 | -0.2708 | -0.0809 | ok |
| assign | 19637557608 | 18840415736 | 18983599529 | -3.3301 | +0.7600 | ok |
| compound | 9271611305 | 9187029589 | 8937498289 | -3.6036 | -2.7161 | ok |
| decloop | 2565175813 | 2536049191 | 2469011111 | -3.7489 | -2.6434 | ok |
| decrender | 4348396391 | 4276915138 | 4228327169 | -2.7612 | -1.1361 | ok |
| dispatch | 20483284942 | 20493850247 | 21219503159 | +3.5942 | +3.5408 | OVER |
| dispatchclass | 15850195790 | 15490532866 | 16022935465 | +1.0898 | +3.4370 | OVER |
| emptyloop | 9308098509 | 9259510179 | 7786100225 | -16.3513 | -15.9124 | ok |
| extcall | 8281259626 | 8230425336 | 7552641804 | -8.7984 | -8.2351 | ok |
| fibcall | 8345686523 | 8324736299 | 8476898128 | +1.5722 | +1.8278 | OVER |
| fibfunc | 7988911869 | 8138967467 | 8302021934 | +3.9193 | +2.0034 | OVER (bar 2.58) |
| heapshape | 3278634306 | 2373955779 | 2332058986 | -28.8710 | -1.7649 | ok |
| nop | 9437300276 | 9240156972 | 9283340889 | -1.6314 | +0.4674 | ok |
| parse | 1541986815 | 1534028319 | 1536336992 | -0.3664 | +0.1505 | ok |
| sayloop | 114813474 | 114714872 | 109048569 | -5.0211 | -4.9395 | ok |
| sendloop | 13863190672 | 13703470952 | 14298810887 | +3.1423 | +4.3444 | OVER |
| startup | 58074163 | 58070010 | 58097457 | +0.0401 | +0.0473 | ok |
| strings | 17749287588 | 17503789392 | 17537481670 | -1.1933 | +0.1925 | ok |
| textnum | 1176579585 | 1152443733 | 1155146223 | -1.8217 | +0.2345 | ok |
| varlookup | 14878111883 | 14461721850 | 13437530651 | -9.6826 | -7.0821 | ok |
| rexxcps | 17817352138 | 17663726114 | 17798509267 | -0.1058 | +0.7631 | ok |

Over budget: `dispatch`, `dispatchclass`, `fibcall`, `fibfunc`, `sendloop`. The env offset (about
1 kIr) is below 0.0001% of any of them and does not change a verdict. No round started (brief Step 3).

Named risks: Task 2's pin depth and Task 12's variable barrier are both inside `head` and not
separable without extra builds. The variable-heavy programs (`assign`, `compound`, `varlookup`,
`alloc`) are all below base; the over-budget programs are the call/send/dispatch shapes.

## Commits

See the commit carrying this report and `perf/`.

## Overlap with the background gates (amendment: `-j 4` while gates run)

The amendment arrived after the runs. The gate status file
(`bg/c484f4516/status.txt`) had no `finished` line during any of them; its last line is
`load G4 10.90 9.96 5.38 ... 2026-10-07T02:08:39+02:00`.
- Builds of base, s1, head (three release builds, `memcap 8G`): before about 02:09, overlapping G3/G4.
- Main callgrind run: `-j 16`, first output 02:10:28, `cg-exit.txt` written 02:22:57, overlapping G4.
- `cgpath` and `cgenv` checks: `-j 4`, until 02:26:46.
Callgrind counts do not depend on `-j`. G4's wall-clock tests ran beside a `-j 16` callgrind load and
may need a re-run if any of them failed.

## Ruling P88: rounds

Diagnosis first: `perf/diagnosis.md` (profiles of s1 and head on `sendloop`, `dispatch`,
`fibfunc`, `fibcall`, `dispatchclass`; scripts in `perf/prof-scripts/`).

Round 1, `ba8f7c581`: one shared `Rc<[u8]>` name per invocation (calling convention, method
identity, `record_call_convention`). Checks: fmt; clippy `-D warnings`; `cargo test --workspace
--release --no-run`, then `memcap 8G cargo test --workspace --release` exit 0 (3048 passed, 0
failed, 4 ignored, summed from `test result:` lines); `concurrency_tests` exit 0 (38 passed) and
with `--features pinning` exit 0 (56 passed); clippy `--features pinning` exit 0; loom
(`RUSTFLAGS="--cfg loom"`) exit 0 (15 passed). Measured with `r1/round.sh` (`-j 16`, after the
gates' `finished` line at 03:05:29):

| program | r1 vs base % | r1 vs s1 % | verdict |
|---|---:|---:|---|
| dispatch | +2.40 | +2.35 | over |
| dispatchclass | -0.15 | +2.17 | inside |
| fibcall | +1.27 | +1.52 | over |
| fibfunc | +3.47 | +1.57 | over (bar 2.58) |
| sendloop | +1.38 | +2.56 | over |

Every other program inside (`phase-6-perf.md` `## S2-S5 gate`, full table).

Rounds 2 and 3 not committed. Five further candidates were built in a scratch tree and measured on
the five programs (`perf/experiments/`, table in the record): none removes cost on every program;
the best for `dispatch` (-1.05%) adds about 24 Ir per call on `fibfunc` and `fibcall`. I stopped
rather than commit a candidate that moves programs over budget further over. Wall clock (Step 2)
and Step 4 not done: still over budget.
