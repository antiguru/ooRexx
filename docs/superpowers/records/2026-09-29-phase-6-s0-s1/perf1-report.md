# S1 front-end perf round: report

Status: DONE_WITH_CONCERNS. One committed round; sendloop, fibfunc and dispatchclass wall
clock remain over the +-4% bar, dispatch moved from +18.85% to +5.87%. Started at 2050300f4.

## Commits

- `49a55bea9` Resume a call in the op loop that runs its callee (S1 front-end round 1)
- `92ac5c054` Record S1 front-end round 1 figures (`docs/superpowers/plans/phase-6-perf.md`,
  `## S1 front-end round`)

This report is not committed: `.superpowers` is ignored and nothing under it is tracked.

## Rounds

| | base | head | r1 |
|---|---:|---:|---:|
| sendloop Ir d% | 0 | +1.23 | -1.08 |
| sendloop L1i / op-cache | 1.0M / 268M | 41.2M / 472M | 69.9M / 735M |
| sendloop wall d% | 0 | +16.96 | +17.06 |
| dispatch Ir / wall d% | 0 | -1.39 / +18.85 | +0.12 / +5.87 |
| dispatch L1i / op-cache | 145M / 998M | 253M / 1561M | 164M / 1160M |
| dispatchclass Ir / wall d% | 0 | -3.20 / +5.44 | -2.24 / +7.03 |
| fibfunc Ir / wall d% | 0 | +3.87 / +9.11 | +4.30 / +8.86 |
| fibfunc L1i / op-cache | 44.9M / 437M | 89.2M / 558M | 93.8M / 607M |
| fibcall Ir / wall d% | 0 | +2.64 / +2.52 | +2.12 / +6.05 |
| rexxcps Ir / wall d% | 0 | +0.50 / +0.65 | +0.09 / +5.02 |
| alloc Ir / wall d% | 0 | -19.89 / -18.94 | -19.55 / -22.88 |
| nop Ir / wall d% | 0 | -4.21 / -14.46 | -2.09 / -16.72 |

Exploration below; commands and full tables in the perf doc.

## Gates

On `92ac5c054`: G1 fmt 0, G2 clippy 0, G3 release build 0, G4 release test 0 (139 `test result:
ok`, 0 Compiling lines), G5 debug build 0, G6 debug test 0 (139 ok, 0 Compiling), G7 clippy
pinning 0, G8 pinning self-tests 0; finished 21:38:18, tree clean.

## Concerns

- The wall-clock target is not met, and more rounds of the same kind would be choosing a layout
  (see the section at the end). rexxcps wall +5.02% and fibcall +6.05% are new overruns on r1
  whose Ir improved; the same layout effect, not verified further.
- fibfunc Ir +4.30% (head +3.87%) and fibcall +2.12% remain over the +0.3% budget.
- Semantics moved: a body's first-instruction permission is now placed by `grant_for` (first
  non-label clause after the entry) and reaching it sets `procedure_permitted`, taken by every
  driven clause; the old granting instance runs only when the entry is not a run of clauses.
  Covered by the gates, no new witness.
- `Op::CallingClause` is a new op: golden renders of regions holding a call or send changed
  (9 lines in `golden_tests.rs`).

### Exploration log

Batch 1 (sendloop, fibfunc, nop control; perf stat one run each; callgrind -r 1). Variants:
`va` resume folded into `ops_loop_steady` (resume_region only delivers), `vac` + begin_invoke's
non-Rexx arms out of line, `vabc` + granting instance out of `drive`, `vbp` one instance for
granting and steady (runtime flag).

| variant | sendloop L1i | sendloop opc | fibfunc L1i | nop Ir d% vs base |
|---|---:|---:|---:|---:|
| base | 1.88M | 244M | 45.1M | 0 |
| head | 40.1M | 474M | 86.2M | -4.21 |
| va | 21.9M | 310M | 74.7M | +35.22 |
| vac | 48.6M | 417M | 62.7M | +35.22 |
| vabc | 19.8M | 390M | 77.7M | +35.22 |
| vbp | 1.79M | 230M | 66.6M | +46.85 |

nop +35%: all of it in `ops_loop_steady` (cgdiff). Batch 2 gated the resume on a bool (`vbp2`,
`vabc2`): nop +33.98% / +23.38%; a small-nop callgrind with `--dump-instr` showed 61M Ir of
`memcpy` per 2M clauses, the merged region state built as one tuple and copied. Batch 3 assigned
each state variable in place (`vabc3`): nop +13.86% vs base; no memcpy left, the rest is spills
and a per-iteration resume test (per-instruction diff of the hot path: 104 instructions per
clause against head's 87). L1i and op-cache counts swing by 2x to 20x between layouts of the same
design (`vbp` 1.79M L1i on sendloop, `vbp2` 19.7M), so they rank variants only within a batch.

Budget arithmetic: head's nop is -4.21% against base, the S1 budget is +0.3%, so a merged resume
may cost at most about 4 Ir per clause.

Batch 4 onward: the merged resume state costs the plain clause however it is arranged (sentinel
`pc` instead of a per-iteration test, the resumed header kept out of the merge, `cold_path`: small
nop still +10 Ir per clause). What removed it: a compile-time split. `close_region` marks a region
holding `CallExpr`, `CallArgs` or `Send` as `Op::CallingClause`; the loop has one region expansion
for `Op::Clause` (no merge, head's code) and one for `Op::CallingClause` and resumed regions. Small
nop back to head (300.82M vs 300.80M), sendloop L1i 1.18M, dispatch 141M (head 251M).

Granting: the out-of-line granting instance cost fibcall ~90 Ir per call (+152M); inlined in
`drive` it keeps a third region expansion on every call path. Kept only as a cold fallback; the
body entry instead finds the op of the first non-label clause (`grant_for`) and cuts the loop's
stream there, so the bound check every op already makes reaches it (no per-clause test). Two ways
to hand the permission over were measured on small programs (Ir, against head):

| variant | nop | sendloop | dispatch | fibfunc | fibcall |
|---|---:|---:|---:|---:|---:|
| `vs3` grant clause opened in the calling expansion | -1.33% | -1.63% | +1.22% | +0.34% | -0.43% |
| `vs4` grant via `procedure_permitted`, taken by every driven clause | +1.33% | -1.80% | +1.18% | +0.22% | -0.36% |

`vs3` cycles against head (perf stat, two runs each): sendloop -9.5%, dispatch -10%,
dispatchclass -3%, fibfunc +0.5%, fibcall +5%.

## Round 1: committed `49a55bea9`

Figures and commands: `docs/superpowers/plans/phase-6-perf.md`, `## S1 front-end round`. Against
base, head -> r1: sendloop Ir +1.23% -> -1.08%, wall +16.96% -> +17.06%; dispatch Ir -1.39% ->
+0.12%, wall +18.85% -> +5.87%; dispatchclass wall +5.44% -> +7.03%; fibfunc Ir +3.87% -> +4.30%,
wall +9.11% -> +8.86%; fibcall Ir +2.64% -> +2.12%, wall +2.52% -> +6.05%; rexxcps Ir +0.50% ->
+0.09%, wall +0.65% -> +5.02%; nop Ir -4.21% -> -2.09%. sendloop L1i 41.2M -> 69.9M (base 1.0M),
op-cache 472M -> 735M: this layout lost what `vs1` (1.18M) had, so round 1 did not fix sendloop.

## Why round 1 did not move sendloop's wall clock

A deterministic view of the front end: callgrind `--dump-instr=yes` on a 200,000-send sendloop
(and dispatch; fibfunc at n=2), counting the 64-byte lines whose instructions run at least once per
send, and how many of them share an L1i set (64 sets, 8 ways; set = address bits 6-11, which
ASLR leaves alone). Script: `$S/hotlines.py FILE N`.

| program | binary | hot lines (bytes) | sets over 8 ways | fullest set |
|---|---|---:|---:|---:|
| sendloop | base | 318 (20,352) | 3 | 10 |
| sendloop | head | 357 (22,848) | 5 | 10 |
| sendloop | r1 | 347 (22,208) | 8 | 12 |
| dispatch | base | 444 (28,416) | 10 | 13 |
| dispatch | head | 490 (31,360) | 24 | 13 |
| dispatch | r1 | 469 (30,016) | 19 | 14 |
| fibfunc | base | 503 (32,192) | 22 | 11 |
| fibfunc | head | 520 (33,280) | 24 | 16 |
| fibfunc | r1 | 518 (33,152) | 30 | 15 |

Round 1 removed the granting instance from the per-send path, but
the per-send hot footprint is still 5-10% over base, and which sets it crowds is decided by
layout: the same design measured 1.18M (`vs1`) and 69.9M (`r1`) sendloop L1i misses. sendloop's
hot lines by function on r1: `begin_invoke` 46, `ops_loop_steady` 52, `run_activation` (drive)
46, `finish_send` 18, `begin_send_op` 13, `resume_region` 9; on base `invoke` 49,
`run_activation` 27, `run_ops_from` 27, `message_term` 19, `exec_message` 13. Per send r1 also
runs about 150 Ir of `memcpy` (76 of it from `push_activation`) and two `malloc`s, as base does.

Not attempted: further rounds that pick a layout by measuring it (the next commit reshuffles it).
What would move this reliably is either a smaller per-send footprint (drive's level switch and
the method-activation setup are the two largest shares) or control over code placement (function
order, PGO), which is a build-configuration decision above this round.
