# Task E — Arithmetic benchmark at parity (Phase 2, Task 2.9)

## What was built

- `rust/crates/rexx-num/benches/arith.rs` — a criterion benchmark, `harness = false`,
  replaying `rust/bench-programs/arith.rex` directly against `Number`.
- `rust/crates/rexx-num/Cargo.toml` — added `criterion = "0.8.2"` as a dev-dependency
  and a `[[bench]] name = "arith" harness = false` section. Nothing else in the file
  changed.

No file under `crates/rexx-num/src/` was touched.

## Operation mapping

`arith.rex`, per iteration (500,000 iterations total):

```
numeric digits 9
a = i / 3
b = a * a - 1
numeric digits 20
c = i / 7
d = c ** 2 // 5
total = total + b + d
```

This is 8 `Number` operations per iteration (2 `div`, 1 `mul`, 1 `sub`, 1 `pow`,
1 remainder-`div`, 2 `add`), 4,000,000 total across the run, reproduced in the
benchmark with the same `DIGITS` value at each call site:

| Source line | `Number` call |
|---|---|
| `a = i / 3` | `i_num.div(&three, 9, DivOp::Divide)` |
| `b = a * a - 1` | `a.mul(&a, 9)` then `.sub(&one, 9)` |
| `c = i / 7` | `i_num.div(&seven, 20, DivOp::Divide)` |
| `d = c ** 2 // 5` | `c.pow(&two, 20)` then `.div(&five, 20, DivOp::Remainder)` |
| `total = total + b + d` | `total.add(&b, 20)` then `.add(&d, 20)` — current `DIGITS` at this point in the loop is 20, set two lines earlier; that is what the interpreter would use too |

`i` is re-parsed from its decimal string every iteration (`Number::parse(&i.to_string())`)
because there is no integer-valued `Number` constructor to reuse instead, and `i`'s
value is different every iteration anyway. The five literal operands (`3`, `7`, `2`,
`5`, `1`) are parsed once outside the loop since they never change — matching what a
real interpreter would do with cached literal values, and avoiding overstating the
Rust side's parsing cost on values that were never going to be reparsed.

Criterion settings match `rexx-bench`'s `interpreter` benchmark and the recorded
baseline exactly: `sample_size(10)`, 500 ms warm-up, 30 s measurement-time ceiling.
Criterion extended past the ceiling to 39.76 s to collect its 10 samples (20 measured
iterations), the same adaptive behavior noted for `interpreter/strings` in
`perf-baseline.md` — not a configuration problem.

## Results

```
cargo bench --offline -p rexx-num --bench arith
```

| | Lower bound | Point estimate | Upper bound |
|---|---:|---:|---:|
| `arith/500k_mixed_digits` | 1.9634 s | **1.9666 s** | 1.9695 s |

C++ baseline (`docs/superpowers/plans/perf-baseline.md`, `interpreter/arith`):
**1.1570 s** (1.1546 s – 1.1595 s, 3/10 samples flagged as outliers).

## What this does and does not establish

**Does establish:** for this exact operation mix (500,000 iterations of the divide/
multiply/subtract/power/remainder-divide/add sequence above, at `DIGITS` 9 and 20),
the Rust `Number` implementation is slower than the equivalent C++ arithmetic —
and it is slower even though the Rust side pays none of the costs the C++ figure
includes. The C++ `interpreter/arith` baseline is a full interpreted run: lexing,
parsing, instruction dispatch, and variable lookup by name are all in that 1.157 s,
on top of whatever the arithmetic itself costs. This benchmark has none of that —
no lexer, no parser, no bytecode dispatch, no name-based variable lookup — only
direct Rust calls into `Number::{div,mul,sub,pow,add}` and `Number::parse`.

**Does not establish:** anything about the eventual Rust interpreter's end-to-end
speed on this program. That number does not exist yet — Phase 2 is `rexx-num` and
its neighbours; dispatch and variable lookup are later phases. A future interpreter
built on this `Number` will pay *additional* cost on top of the 1.9666 s measured
here, not less, so this result is if anything an early warning rather than a
one-off. It also does not isolate *which* part of `Number` is slow (parsing,
allocation per digit-vector operation, the long-division loop, etc.) — that would
need per-operation benchmarks, which are out of this task's scope.

## The honest finding

**Rust is slower, not faster: 1.9666 s vs. 1.1570 s, roughly 1.70x the C++ figure —
despite the Rust side excluding all interpreter overhead that the C++ figure
includes.** Phase 2's exit gate (D9, Global Constraints) requires parity with the
C++ build; on this benchmark the Rust arithmetic does not meet it. This is reported
as measured; no arithmetic code was changed to influence this number, and none
should be changed for that purpose in response to it (per the task's own
constraint) — any future fix belongs to a task that owns performance work in
`rexx-num`'s implementation, with its own before/after differential-testing
discipline, not to this benchmarking task.

A plausible source (not investigated further, since that is out of scope here):
`Number` represents digits as `Vec<u8>` with one allocation per intermediate result,
and `Number::parse` runs a byte-by-byte scanner over a freshly formatted string for
`i` every iteration — decNumber-style C++ arithmetic typically works over fixed-size
inline buffers with far fewer heap allocations per operation. Confirming that would
need allocation profiling or per-operation micro-benchmarks, which this task was not
asked to produce.

## Verification

- `cargo clippy --offline --workspace --all-targets -- -D warnings` — clean, no
  warnings.
- `cargo test --offline --workspace` — 127 passed, 0 failed, 0 ignored (same count
  as the brief's stated baseline; no test regressions from this change).
- `cargo build --offline -p rexx-num --benches` — builds cleanly.

## Constraints honored

- No file under `crates/rexx-num/src/` was modified.
- `crates/rexx-num/Cargo.toml` changed only by adding the dev-dependency and the
  `[[bench]]` section.
- No arithmetic behaviour was changed; no attempt was made to tune the
  implementation to improve the measured number.
- No git command was run; the working tree is left uncommitted for centralized
  staging.
