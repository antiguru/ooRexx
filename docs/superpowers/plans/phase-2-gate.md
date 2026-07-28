# Phase 2 exit gate — assessment

Phase 2 built `rexx-num`: Rexx decimal arithmetic, the `NUMERIC` settings,
comparison, `FORMAT`/`TRUNC`, and the error messages for all of it.

The gate has five criteria. **One is met, one failed and is recorded as debt,
and three cannot be assessed at this phase.** The three are not skipped work —
they were written assuming an interpreter that the phase ordering does not
deliver until Phase 4, and no amount of Phase 2 effort could satisfy them.

That is a defect in the plan, not in the phase, and it should be fixed in the
plan text before Phase 3 starts so the same trap is not set again.

---

## 1. Corpus programs — CANNOT ASSESS

> All eleven `rust/corpus/num/` programs run under the Rust build with zero
> divergences from the oracle.

`rexx-diff` runs a Rexx *program* under an interpreter and compares output.
There is no Rust interpreter at Phase 2. The parser is Phase 3 (D10 opens it)
and dispatch is Phase 4. Phase 2 delivers a library.

Also: there are now **twelve** programs, not eleven. `form_notation.rex` and
`format_trunc.rex` arrived with Tasks 2.6 and 2.7.

**What was done instead.** Every behaviour those programs pin was verified
directly against `build/bin/rexx` through the differential harness, at a
volume no corpus program reaches — see criterion 4's table. The corpus
programs remain valuable as an end-to-end check and should run at Phase 4.

## 2. ooTest arithmetic assertions — CANNOT ASSESS

> Every arithmetic assertion extractable from ooTest passes.

Same blocker: those are Rexx programs.

**A route that would not need an interpreter**, not taken here and offered as
an option: `rexx-extract` already pulls programs out of ooTest. Assertions
shaped like `assertEquals(<literal>, <literal><op><literal>)` could be
rewritten mechanically into differential cases and run through the existing
harness. That would cover a real slice of this criterion at Phase 2 rather
than deferring all of it. It is genuine scope, so it is written down rather
than done.

## 3. ANSI X3.274 vectors — CANNOT ASSESS

> ANSI X3.274 arithmetic test vectors pass, or the deviations are documented
> and justified against the oracle's behaviour where the standard and ooRexx
> disagree.

No vector files exist anywhere in the tree, and the session was offline.

The criterion partly answers itself. The project's own rule is that **where
the standard and ooRexx disagree, the interpreter wins** — so the vectors are
a secondary check by construction, while the differential corpus tests
against the authoritative oracle directly. What the vectors would add is an
*enumeration of the deviations*, which is exactly what this criterion asks to
document and exactly what cannot be produced without them.

## 4. Performance parity — FAILED, recorded as debt

> `arith` benchmark at parity with `perf-baseline.md`.

| | mean | gap |
|---|---|---|
| C++ `interpreter/arith`, re-measured | 1.1546–1.1595 s | — |
| Rust, as first written | 1.9666 s | 1.70× |
| Rust, after the division rewrite | **1.4067 s** | **1.22×** |

Recorded in `d1-decision.md` with the caveat that matters: **1.22× is a lower
bound, not a near miss.** It times Rust arithmetic alone against a C++ figure
that includes parsing, dispatch and variable lookup, because no Rust
interpreter exists to measure. The gap widens once Phases 3–4 add those costs.
Re-measure at Phase 4, which `d1-decision.md` already schedules.

The benchmark asserts its own result equals `4629643519330627.7808`, the
string the interpreter prints for the same program, so the two sides are known
to compute the same thing rather than merely to take similar time.

## 5. Lints and `unsafe` — MET

> `cargo clippy --all-targets -- -D warnings` clean; zero `unsafe`; crate root
> still `forbid`.

All three verified: `unsafe_code = "forbid"` at `[workspace.lints.rust]`, no
`unsafe` in any crate, no `allow(unsafe…)` escape anywhere, clippy clean
across the workspace with `-D warnings`.

**One honest qualification.** No CI workflow builds or tests the Rust tree.
`.github/workflows/` covers the C++ build on five platforms and never invokes
cargo. Every "clippy clean, N tests pass" in this phase is a local claim with
nothing enforcing it between sessions. Wiring the Rust crates into CI is worth
doing before Phase 3 adds substantially more code.

---

## What the phase actually verified

Differential testing against `build/bin/rexx`, which is the method the phase
rests on:

| set | cases | covers |
|---|---|---|
| `addsub` | 8,712 | `+` `-` |
| `addsub2` | 8,112 | `+` `-`, second value list |
| `muldiv` | 17,424 | `*` `/` `%` `//` |
| `md2` | 20,184 | the same, second value list |
| `pow` | 2,112 | `**` |
| `cmp` | 32,368 | all comparison operators, strict and numeric |
| `fmt` | 1,800 | `FORMAT` `TRUNC` |
| `fmt2` | 6,720 | the same, wider arguments |
| `fmt3` | 12,136 | ENGINEERING form; before/after × expp/expt |
| `fmtedge` | 640 | exponent extremes, non-numbers |
| `fmtcarry` | 15,840 | the exponent check after a rounding carry |
| **total** | **126,048** | |

Every count above was regenerated and counted for this document rather than
copied from a report. Plus randomised sets on unused seeds, and 180 targeted
power-range cases at the exponent limits.

All eleven are reproducible from
`rust/crates/rexx-num/tests/gen-curated-sets.py`, which matters because the
oracle captures themselves live in a session scratchpad and do not survive.
Two of the eleven, `fmt` and `fmt2`, are reconstructions rather than byte-exact
copies of the sets originally used — that is recorded in the generator's own
docstring, and both were re-run against the oracle at 0 before the claim was
made.

**Four defects were found *after* their tasks were first reported done**, all
by review or by corpus extension rather than by the tests written alongside
the code:

- `FORMAT`/`TRUNC` overflowing on width arguments at or above 2³¹
- the missing `NUMERIC DIGITS` upper bound, and two `i32` casts it left
  reachable — one of which wrapped silently in release rather than panicking
- the exponent-width check being *moved* rather than duplicated, so a rounding
  carry went unreported
- `substitute` re-scanning text it had just injected

## The blind spot worth naming

The corpus samples values and arguments. Until `fmtcarry`, **nothing targeted
the boundary where a valid result becomes an error.** The carry defect
survived 21,296 FORMAT cases precisely because the successful-render path
agrees with the interpreter at exactly that boundary — only the error path
diverged. Phase 3 should build error-boundary sets deliberately rather than
hope value sampling reaches them.
