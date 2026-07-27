# Phase 0 benchmark programs

One `.rex` file per D9 performance dimension (`docs/superpowers/plans/2026-07-27-rust-rewrite.md`,
Global Constraints and D9), each sized to run roughly 0.5-2s under `build/bin/rexx` except
`startup.rex`, which is deliberately as close to instantaneous as a program can be.

| File | Covers |
|---|---|
| `dispatch.rex` | Tight method-send loop: one object, one instance method, 5,000,000 sends |
| `varlookup.rex` | Plain simple-variable read/write, no stems, 19,000,000 iterations |
| `compound.rex` | Stem/compound-variable access — the workload the compound-variable memo prototype measured at -24% (`[[compound-variable-memo-prototype]]` in project memory); 500 tails is inside its measured 100-10,000 sweet spot |
| `strings.rex` | `SUBSTR`/`POS`/`CHANGESTR`/concatenation, 3,000,000 iterations |
| `arith.rex` | Decimal arithmetic, alternating `NUMERIC DIGITS 9` and `NUMERIC DIGITS 20` every iteration so both settings are exercised throughout the run rather than only at startup |
| `alloc.rex` | Allocation churn: a fresh `.array` and `.string` every iteration, neither retained past it, sized to force multiple collections |
| `startup.rex` | `say 1` — cold-start timing (D2's gate), timed separately with `rexx-time`, not through criterion's statistical sampling |

## Determinism

Same rule as `corpus/`: byte-identical output on every run of the same interpreter. No `DATE()`,
`TIME()`, process IDs, or unordered iteration. All seven were run under `build/bin/rexx` and
confirmed to exit 0 with identical output across repeated runs before being committed.

## Sizing

Loop counts were tuned by timing each program directly under `build/bin/rexx` (not through the
criterion harness) and adjusting until each landed between 0.5s and 2s. See
`docs/superpowers/plans/perf-baseline.md` for the measured numbers this produced.

## Two things this corpus (see `../corpus/README.md`) learned the hard way, reconfirmed here

`say a"|"b` does not concatenate three values — `"|"b` is read as a **binary string literal** (the
`b` suffix binds to the preceding quote) and the program dies with error 15.4. None of these
programs use that form; string concatenation here uses explicit `||`.

`.integer` and `.rexxinfo` are not usable as environment classes (`.integer` is internal and
unexposed; `.rexxinfo` is an instance, not a class). None of these programs reference either.
