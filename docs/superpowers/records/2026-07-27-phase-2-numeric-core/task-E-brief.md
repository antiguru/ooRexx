# Task E — Arithmetic benchmark at parity (Phase 2, Task 2.9)

Phase 2's exit gate requires the Rust arithmetic to reach **parity** with the
C++ build. Phase 1's looser 1.5x threshold was a viability check only and no
longer applies.

## The baseline

`docs/superpowers/plans/perf-baseline.md` records the C++ numbers measured on
this machine. The relevant row is `arith` at **1.157 s** (criterion mean).
The benchmark program is `rust/bench-programs/arith.rex`.

## What parity means here

The C++ figure is an interpreted Rexx program: it pays for parsing, dispatch
and variable lookup that a Rust microbenchmark of `Number` does not. A direct
comparison would flatter the Rust side and prove nothing.

So build the comparison honestly:

1. Read `rust/bench-programs/arith.rex` and determine exactly which
   arithmetic operations it performs and how many times.
2. Write a criterion benchmark in `rexx-num` that performs the same
   operations the same number of times through `Number`.
3. Report both figures and state plainly what the comparison does and does
   not establish — specifically that the C++ side includes interpreter
   overhead the Rust side has not built yet.

The honest claim available at this phase is about the arithmetic itself, not
about end-to-end interpreter speed. Say so rather than implying more.

## Criterion settings

The existing suite uses `sample_size(10)`, 500 ms warmup and a 30 s
measurement ceiling, because the default 100 samples over 5 s costs minutes
per benchmark at this scale. Match those settings or the numbers are not
comparable with the recorded baseline.

## Before you start

`rexx-num` has no criterion dev-dependency yet; `rexx-core` and `rexx-bench`
already use `criterion = "0.8.2"`, so it is in the offline registry cache.
Add the same version and a `[[bench]]` section with `harness = false`. Every
cargo command in this project must be run with `--offline`.

For reference, `arith.rex` runs 500,000 iterations, each doing `/` and `*`
and `-` at DIGITS 9, then `/` and `**` and `//` at DIGITS 20, then two `+`
into an accumulator. The DIGITS switch happens inside the loop deliberately.

## Constraints

- Add benchmarks; do not change arithmetic behaviour. Any change to a
  `Number` method must keep all differential sets at zero divergences.
- Report numbers, do not tune the implementation for them. If Rust is
  slower, say so — that is a finding, not a failure to hide.
