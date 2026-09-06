# Phase 5f Task 3f — MAX and MIN, and a literal pool that changes an answer

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 3.
BASE `fb029e695`. Landed at `PENDING`.

**Task 3 is complete**: all twenty-five rows with a builtin behind them are
bound. String reads 116 `answers`, 6 `uncomparable`, 13 `loud`: **99 of the
phase's 112 bound**, and the 13 left are exactly Task 4's.

## 1. The method never takes the integer fast path

`max_min_over` is the general path lifted out; the fast path stays in the
builtin wrapper. That is not an optimisation choice — `RexxInteger::Max` is a
different class's method, and `RexxString::Max` reaches `NumberString::maxMin`
unconditionally, so a `String` receiver always takes the general one.

The two paths disagree about an omitted argument, and a program can see it:
`max(1,,3)` is 93.903 at rc 163 and `max(1.0,,3)` is 40.5 at rc 216.

`A_COUNT` means nothing is refused ahead of the body, so the method's
omitted-argument error is the **builtin's** 40.5 at rc 216 rather than a
93.9xx — measured, `'55'~max(,77)` reports `Missing argument in invocation of
MAX; argument 1 is required.` A non-numeric argument is 93.904 at rc 163, and
it renders the argument as an *object*: `'55'~max(.array)` reports
`found "The Array class"`.

## 2. An integer literal changes what a string literal *is*

The first cut of the refusals witness disagreed with the oracle on three rows,
and the cause was in the witness rather than the crate. Measured on the oracle,
each pair differing only in the line before:

```text
             say '5'~max(,7)      40.5    rc 216
zz=5       ; say '5'~max(,7)      93.903  rc 163
n=6        ; say '5'~max(,7)      40.5    rc 216
n=4        ; say '4'~max(,7)      93.903  rc 163
say 1 '5'~max(,7)                 40.5    rc 216
```

**An integer literal anywhere in the program makes a string literal of the same
spelling take the integer path.** The variable's name does not matter and the
value must match exactly; a literal used in an expression rather than assigned
does not do it. `~class` cannot see it — `5~class` is `The String class` too —
so the behaviour is the only instrument.

The first witness had `when n = 5` beside `'5'` receivers, so its own clause
numbering decided its answers. It now spells its receiver `55` and its operands
`77` and `88`, and says why in its header. This crate does not reproduce the
effect, for the reason `integer_object`'s doc already gives: nothing in the
value model separates `1` from `'1'`. **That is a divergence worth Moritz's
attention rather than mine, and it is not new — this is a fourth spelling of
the one `integer_object` already records.**

## 3. The control — one confirmed, one falsified, one that could not have run

**V — the omitted-argument raise becomes 93.903.** Predicted: refusals rows 5,
6, 7 and nothing else. Measured: `5,7c5,7`. **Confirmed.**

**W — ties take over.** The first attempt mutated `Extreme::wins` and moved
nothing: `wins` is the *integer* path's comparison and the method never runs
it. **That mutation could not have failed**, and running it is what said so.
Re-aimed at `Extreme::op`, the `CompareOp` the general path uses: predicted
values lines 3, 4 and 5; measured lines **4 and 5**. Line 3 was wrong —
`'0'~max('-0')` cannot show a tie, because every spelling of zero is unsigned
by the time the answer is rendered, which `sign_of`'s own doc states.

**X — the 93.904 renders the failed conversion instead of the object.** Nothing
moved. For `.array` the two are the same bytes, so this witness cannot separate
them; I did not find a value where they differ and am not claiming none exists.

## 4. Gates

| | |
|---|---|
| G1 `cargo fmt --all --check` | PENDING |
| G2 `cargo clippy --workspace --all-targets -- -D warnings` | PENDING |
| G3 `cargo test --release --workspace --no-fail-fast` | PENDING |
| G4 same with `REXX_CORPUS_GATE=1` | PENDING |
| G5 `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace` | PENDING |
| G6 `REXX_PHASE_GATE=5c` | PENDING |
| G7 `REXX_PHASE_GATE=5d` | PENDING |

Pre-commit chain: method-bodies refresh rc 0 (two rows `loud` -> `answers`, no
row on any other class moved), fmt rc 0, clippy rc 0, strict corpus 397 of 397,
full workspace test rc 0.
