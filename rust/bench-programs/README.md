# Phase 0 benchmark programs

One `.rex` file per D9 performance dimension (`docs/superpowers/plans/2026-07-27-rust-rewrite.md`,
Global Constraints and D9), each sized to run roughly 0.5-2s under `build/bin/rexx` except
`startup.rex`, which is deliberately as close to instantaneous as a program can be.

| File | Covers |
|---|---|
| `dispatch.rex` | Tight method-send loop: one object, one instance method, 5,000,000 sends |
| `dispatchclass.rex` | The same dimension reached through a class object instead of an instance: one class-method send per pass, 4,000,000 sends. Runs on this crate, where `dispatch.rex` needs `~new` |
| `varlookup.rex` | Plain simple-variable read/write, no stems, 19,000,000 iterations |
| `compound.rex` | Stem/compound-variable access — the workload the compound-variable memo prototype measured at -24% (`[[compound-variable-memo-prototype]]` in project memory); 500 tails is inside its measured 100-10,000 sweet spot |
| `strings.rex` | `SUBSTR`/`POS`/`CHANGESTR`/concatenation, 3,000,000 iterations |
| `arith.rex` | Decimal arithmetic, alternating `NUMERIC DIGITS 9` and `NUMERIC DIGITS 20` every iteration so both settings are exercised throughout the run rather than only at startup |
| `alloc.rex` | Allocation churn: a fresh `.array` and `.string` every iteration, neither retained past it, sized to force multiple collections |
| `alloc4c.rex` | Allocation churn restricted to the 4c surface (no message sends): a new compound-variable tail and a concatenated string every iteration. Not the same axis as `alloc.rex` -- see its own header for what carries over and what does not |
| `parse.rex` | `PARSE` in the shapes a real program uses: a positional pattern, a **variable** pattern, and `PARSE VAR` with several targets. Added 2026-09-10 to close a coverage hole -- `exec_parse` is 6.8% of `rexxcps`' profile and no program here exercised it at all |
| `textnum.rex` | Values that arrive as **text** and are then used as numbers, with fresh handles each iteration so a handle-keyed cache cannot answer from the previous one. Added 2026-09-10 for the same reason: `rexxcps` performs 5,580,002 of these conversions and no program here performed one |
| `decloop.rex` | A **decimal-controlled** `DO` -- `do j=1.1 to 2.2 by 1.1`. `emptyloop.rex` and every other loop here use integer control, and the two are not the same axis: measured 2026-09-10, integer control is 1.22x the oracle and decimal control 1.63x |
| `decrender.rex` | The same decimal control plus the renderings a real program performs on the control value (`length(j)`, `j='foobar'`), which is `rexxcps`' inner-loop shape. **1.99x**, the worst ratio in the suite, and the construct that carries `rexxcps` above every other primitive it exercises |
| `startup.rex` | `say 1` — cold-start timing (D2's gate), timed separately with `rexx-time`, not through criterion's statistical sampling |
| `heapshape.rex` | Full-GC pause over a ~1M-object graph. It prints its own figures, so the suite reports those rather than timing the process. **The slot strings are wider than seven bytes on purpose**: a shorter one lives in the Rust handle and allocates nothing, which collapses the graph to ~1,001 objects -- see the program's own comment |

## Determinism

Same rule as `corpus/`: byte-identical output on every run of the same interpreter. No `DATE()`,
`TIME()`, process IDs, or unordered iteration. Every program in this directory was run under
`build/bin/rexx` and confirmed to exit 0 with identical output across repeated runs before being
committed.

`heapshape.rex` is the exception and is the reason `NOT_BENCHMARKED` exists: it calls `TIME()` and
prints wall figures, so its output varies between runs by construction. Only its `gc_forced=` line
is byte-stable.

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
