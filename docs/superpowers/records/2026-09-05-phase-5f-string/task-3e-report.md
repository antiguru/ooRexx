# Phase 5f Task 3e — ABS, TRUNC, FORMAT

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 3.
BASE `757bb3dd4`. Landed at `e0eea4821`.

String now reads 114 `answers`, 6 `uncomparable`, 15 `loud`: **97 of the
phase's 112 bound**, 15 left — `max` and `min` in Task 3, and Task 4's 13.

## 1. For these three the target always wins

The method's ordering is the simplest in the phase: **read the receiver, then
the arguments.** Measured, and it is not the builtin's:

| send | reports |
|---|---|
| `'abc'~trunc('x')` | 93.943, the target |
| `'abc'~format(1,-1)` | 93.943, the target |
| `trunc('abc','x')` | 40.12, the argument's type |
| `trunc('abc',-1)` | 93.943, the target |
| `format('abc','x')` | 40.12, the argument's type |

So the builtin is *argument type → target → argument range* and the method is
*target → argument*. `abs_of`, `trunc_of` and `format_over` therefore take a
`Number` and already-range-checked widths, and each caller does its own reads
in its own order. That is the third variation on the same theme in Task 3,
after 3c's two.

## 2. A count is an `i64`, not a `usize`

`count_method_argument` answers `Option<i64>` where
`optional_non_negative_argument` beside it narrows to `usize`. The narrowing is
right for an index into bytes and wrong for a width: `padding_width` reads an
`i64` and decides for itself which widths are absurd — `'1'~format(,,3000000000)`
answers `1` while the same width with an exponent to pad is `System resources
exhausted` — and narrowing first would move that decision.

## 3. A test that had been fragile all along

`dispatch::tests::an_unimplemented_method_is_loud_where_an_unknown_one_is_a_condition`
searched for a loud name by **sending every name String answers** until one
refused. `ABS` sorts early and its body reads `NUMERIC DIGITS`, which wants a
live activation the test does not have, so the search panicked at
`activation.rs:1464` before it reached a loud name.

It was luck that it worked before: `String~sign` has needed an activation since
it landed, and the loop only ever stopped short of it. The search now consults
the registry — a name with no `NATIVE_METHODS` row — and sends only that one,
so the assertion stays behavioural while the search no longer runs bodies.

## 4. The control — three mutations, one of them falsified

**S — TRUNC reads its receiver after its argument.** Predicted: refusals rows 4
and 5 alone, since rows 1-3 have no argument to lose to and rows 6-7 are
FORMAT. Measured: `4,5c4,5`. **Confirmed.**

**U — FORMAT's `before` and `after` swapped.** Predicted: values lines 4 and 5,
the only lines passing either. Measured: `4,5c4,5`. **Confirmed.**

**T — ABS drops its rounding** (`value.abs().into_round(digits)` ->
`value.abs()`). Predicted: values line 8, the only line under `numeric digits
3`. Measured: **nothing moved, on either file. The prediction is falsified.**

`abs_of` hands its result to `interp.number(_, saturate(digits), form)`, which
rounds to the same precision on the way out, and rounding to a fixed precision
is idempotent — so the explicit `into_round` cannot change a rendered answer.
I did not find a program that distinguishes the two, and I am not claiming
there is none; what is established is that **neither witness can see it**, and
the call is left in place because it mirrors the oracle's own
`copyForCurrentSettings` and `sign_of`'s doc contrasts against it.

## 5. Gates

| | |
|---|---|
| G1 `cargo fmt --all --check` | rc 0 |
| G2 `cargo clippy --workspace --all-targets -- -D warnings` | rc 0, zero warnings |
| G3 `cargo test --release --workspace --no-fail-fast` | rc 0 |
| G4 same with `REXX_CORPUS_GATE=1` | rc 0 |
| G5 `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace` | rc 0 |
| G6 `REXX_PHASE_GATE=5c` | rc 0 |
| G7 `REXX_PHASE_GATE=5d` | rc 0 |

Pre-commit chain: method-bodies refresh rc 0 (three rows `loud` -> `answers`,
no row on any other class moved), fmt rc 0, clippy rc 0, strict corpus 395 of
395, full workspace test rc 0.
