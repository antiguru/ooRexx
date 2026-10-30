# Task 8.7c re-review r3 (fix round 4): APPROVED

Critical 0, Important 0, Minor 0.

Builds: `git archive` of 7ee35b287 and cd070fd13 (rust and interpreter trees), CARGO_INCREMENTAL=0, own target dirs.
Probe copies and per-binary outputs (o oracle, a head 7ee35b287, b cd070fd13): `docs/superpowers/records/2026-10-18-phase-9d/t8.7c/review-probes-rr3/` (`out/K.{o,a,b}.{out,err,rc}`).

## (a) R2-1 addressed
| witness | head vs oracle | cd070fd13 vs oracle |
|---|---|---|
| c2/once.rex | equal (out, err, rc) | out differs |
| c2/oncesig.rex | equal | out differs |
| n2c/u.rex | equal | out differs |
| corpus/lang/reference_conversion_raises_in_clause.rex | equal | out differs |

Red at cd070fd13, oracle-equal at head.

## (b) Round 3 not regressed
rr1 i1d/k, i1f/n, i1g/p, i1c/l: oracle-equal at head and at cd070fd13, head stdout identical to cd070fd13. Round 3 corpus witnesses (reference_rendering_sends, raise_untraced_renders_nothing, lostdigits_names_the_string_value): equal at head, byte-identical to cd070fd13. i1b/g: differs from the oracle at both, stdout of head and cd070fd13 byte-identical (`cmp`); the queued `>=>` item.

## (c) value.rs
`git diff d6720f350 7ee35b287 -- rust/crates/rexx-exec/src/value.rs` is only the `Body::VarRef(_)` arm in `object_string_value_text` (+4 lines). `reference_text` is gone. The rest of the 7ee35b287 rust diff is the corpus witness, its expected file and phase-9.txt.

## (d) Perf
All 99 `.vg` Command lines in fix4-final name rexx-base-97eee8a9a (99) or rexx-head-d8614a7c9 (99) under /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7c/. The pinned head is d8614a7c9; 7ee35b287 differs from it in rust/ only by corpus files, so no code differs.
Re-measured with callgrind.sh -B 0.5 -r 3 (base, pinned head, and my 7ee35b287 build), status 0, no SPREAD:
arith head -libc +0.0464% (my build +0.0464%), parse +0.0396% (+0.0396%). Equal to the report; under the 0.05% budget.

## Findings
None.
