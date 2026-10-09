# Heapshape running-total bisect: report

Measurement only. Binaries were built from `git archive <sha> rust interpreter` copies under
`/tmp/claude-1000/p61/hsb/src2-<short>` (mtimes touched), one shared `CARGO_TARGET_DIR=/tmp/claude-1000/p61/hsb/target`.
Every build log has a `Compiling rexx-exec` line. The brief's `-p rexx-run` names no package, so the command was
`CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx-run` (script:
`/tmp/claude-1000/p61/hsb/scripts/build.sh`, logs in `.../hsb/logs/`).

base61 `/tmp/claude-1000/p61/t1/bin/base/rexx-run` sha256 `2ea19b3e...c891e` matches the gate record's `## Task 1`
row for `e6af1198b`. T8 binary (`/tmp/claude-1000/p61/t9/perf/bin/base/rexx-run`) was reused as the brief says.
Built binaries' sha256 are in `/tmp/claude-1000/p61/hsb/shas.txt`.

## 1. Checkpoints

```
memcap 8G bash rust/bench-programs/callgrind.sh -r 2 -j 6 -o /tmp/claude-1000/p61/hsb/cg1 -p heapshape \
  base61=/tmp/claude-1000/p61/t1/bin/base/rexx-run t1=.../bin/b4d847074/rexx-run ... t7=.../bin/b71606f16/rexx-run \
  t8=/tmp/claude-1000/p61/t9/perf/bin/base/rexx-run
```

All spreads 0.0000%, exit 0.

| checkpoint | commit | heapshape Ir | step vs previous | vs base61 |
|---|---|---|---|---|
| base61 | e6af1198b | 2334068271 | | |
| T1 | b4d847074 | 2334068201 | -0.0000% | -0.0000% |
| T2 | 7b84c819d | 2334093992 | +0.0011% | +0.0011% |
| T3 | ec97e4190 | 2338147676 | +0.1737% (+4.05M) | +0.1748% |
| T4 | 66d9eac18 | 2338167015 | +0.0008% | +0.1756% |
| T4a | 5e59d66b3 | 2337167304 | -0.0428% | +0.1328% |
| T5 | 37d874a36 | 2337299087 | +0.0056% | +0.1384% |
| T5a | 1fd487d02 | 2362072592 | **+1.0600% (+24.77M)** | +1.1998% |
| T6 | 072e536ac | 2357048341 | -0.2127% | +0.9846% |
| T7 | b71606f16 | 2358026248 | +0.0415% | +1.0264% |
| T8 | 6358ca7a6 | 2358028284 | +0.0001% | +1.0265% |

Only T5a is over +0.2% (T3's +0.17% is below the threshold and was not bisected). T6 gives back -0.21%, so the
+1.03% at T8 is T5a's +1.06% minus T6's -0.21% plus T3's +0.17% and small residue. T4a (the R9 truth judgment) is
-0.04% here: it is not the carrier.

## 2. Bisect of T5a (`git log --oneline --reverse 37d874a36..1fd487d02 -- rust`)

Five commits, each built and measured with `-p heapshape -r 2` (`/tmp/claude-1000/p61/hsb/cg3`, spreads 0.0000%),
against T5 `37d874a36` = 2337299282 Ir (this run's reading; 195 Ir off the first run's, spread-free in each):

| commit | subject | heapshape Ir | step | vs T5 |
|---|---|---|---|---|
| 8e52e14ca | a collection trigger that counts bytes | 2374153646 | +36.85M (+1.577%) | +1.5768% |
| 5f5b80b3d | perf round 1: charge after the allocation | 2355131919 | -19.02M (-0.801%) | +0.7630% |
| 0d18b0045 | refusal-sites.tsv re-derived | 2355131724 | -195 | +0.7630% |
| d0a3d5db3 | fix round 1: arrays, buffers, ~copy charged | 2362072592 | +6.94M (+0.295%) | +1.0599% |
| 72287a744 | fix round 2: multidimensional extend | 2362072592 | 0 | +1.0599% |

So two commits carry the net: `8e52e14ca` (+36.85M, partly recovered by `5f5b80b3d`) and `d0a3d5db3` (+6.94M).
`1fd487d02` (the T5a close) has the same Ir as `72287a744`.

## 3. Attribution

`python3 -I rust/bench-programs/cgdiff.py A.cg B.cg` on r1 outputs (`/tmp/claude-1000/p61/hsb/diff-*.txt`;
functions over 3000 Ir):

| step | function | self Ir change | calls change |
|---|---|---|---|
| T5 -> 8e52e14ca | `Interp::collect_now` | +17,830,480 | none (5 calls before and after) |
| T5 -> 8e52e14ca | `Interp::concat_values` | +19,019,361 | none (1,001,019 calls) = +19 Ir per call |
| 8e52e14ca -> 5f5b80b3d | `Interp::concat_values` | -19,019,361 (all of it) | none |
| 0d18b0045 -> d0a3d5db3 | `Interp::collect_now` | +7,929,986 | none (5 calls) |
| 0d18b0045 -> d0a3d5db3 | `Interp::begin_invoke` | -1,003,592 | none (1,003,598 calls) |

Call counts of every function over 300 calls are unchanged in all three steps, so this is per-call cost, not extra
calls. The work is added work inside inlined bodies, not a codegen shift:

* `concat_values` +19 Ir per call (`8e52e14ca`): the R10 charge, `Heap::charge_body_bytes` plus the threshold test,
  inlined into `Interp::text_bytes` at each long concat. `5f5b80b3d` moved the charge after the allocation and
  removed the whole +19.0M. Not part of the final +1.06%.
* `collect_now` +17.83M (`8e52e14ca`, persists to HEAD): `Heap::collect` (inlined in `collect_now`) gained
  `live_bytes += object.body.held_bytes();` in both mark loops (`rust/crates/rexx-core/src/heap.rs`, the loop body
  after the `WeakRef` check, and the resurrect loop), plus the per-collection bookkeeping at the end. heapshape holds
  about a million live Text objects in a graph (`rust/bench-programs/heapshape.rex`), so a per-survivor `match` on
  `Body` is visible. This is R10 byte-aware trigger work (Task 5a), not R9.
* `collect_now` +7.93M (`d0a3d5db3`): `Body::held_bytes` gained `Array` and `Instance { native: Buffer }` arms
  (`rust/crates/rexx-core/src/body.rs`), so the same per-survivor match got longer for every survivor. The R10
  ruling extension (arrays, buffers) is the cause.
* I did not measure visits per collection, so a per-survivor Ir figure is not given.

## 4. Eight programs, base61 against T8

`memcap 8G bash rust/bench-programs/callgrind.sh -r 2 -j 6 -o /tmp/claude-1000/p61/hsb/cg4 -p "pingmsg pingguard
pingsem alloc alloc4c heapshape rexxcps emptyloop" base61=... t8=...`, exit 0, spreads at most 0.0004%:

| program | base61 | T8 | delta |
|---|---|---|---|
| pingmsg | 1676421138 | 1668808183 | -0.4541% |
| pingguard | 1159017575 | 1152173108 | -0.5905% |
| pingsem | 1181250583 | 1176104577 | -0.4356% |
| alloc | 20345537204 | 20288479443 | -0.2804% |
| alloc4c | 3140684589 | 3113058259 | -0.8796% |
| heapshape | 2334068271 | 2358028284 | +1.0265% |
| rexxcps | 17788420418 | 17792514518 | +0.0230% |
| emptyloop | 7786099900 | 7761252576 | -0.3191% |

The review's Tasks 1-8 column is confirmed: heapshape +1.0265% is the only program over budget.

## Conclusion

The +1.03% at T8 comes from Task 5a, commits `8e52e14ca` (+36.85M Ir, of which `5f5b80b3d` recovered 19.02M) and
`d0a3d5db3` (+6.94M), net +24.77M or +1.06%, against T3's +4.05M (+0.17%) and T6's -5.02M (-0.21%). It is added
work, not codegen: call counts do not move, and the Ir sits in `collect_now` where `Heap::collect` sums
`Body::held_bytes()` for every survivor in both mark loops (+17.83M, then +7.93M when `held_bytes` gained the
Array and buffer arms); the other +19.0M was the per-concat charge and is already gone. The R9 truth judgment
(Task 4a) is not involved (-0.04%). Candidate change, not tried: stop summing live bytes over survivors. Count
the bytes of freed bodies in the sweep (garbage is small in heapshape) and keep a running
`live_bytes = charged_total - freed`, or accumulate `held_bytes` only for the few variants that can be nonzero
(Text non-inline, Array, Buffer) with a precomputed flag per object. Either needs a measurement with
`callgrind.sh -p heapshape` against `37d874a36`, and an assertion test that the running total equals the survivor
sum, before it is trusted.
