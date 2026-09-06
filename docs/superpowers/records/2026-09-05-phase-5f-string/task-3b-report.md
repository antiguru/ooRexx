# Phase 5f Task 3b — STRIP, ABBREV, COMPARE

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 3.
BASE `709f64468`. Landed at `PENDING`.

Three more of Task 3's twenty-five. String now reads 99 `answers`, 6
`uncomparable`, 30 `loud`: **82 of the phase's 112 bound**, 30 left.

## 1. All three needed lifting, and all three came out pure

Same answer as 3a — none of `strip`, `abbrev`, `compare` had a core — with a
difference worth recording for the rest of Task 3: **these three cores need no
`Interp` at all.** `strip_bytes` is a subslice of its input, `abbrev_holds`
answers a `bool` and `compare_at` a `usize`. 3a's four all took `&Interp`
because they allocate through `buffer` and can raise 5.1.

So "needed a lift" is not one question. Of the seven bound so far, four lift to
an allocating core and three to a pure function. The remaining eighteen split
the same way and the numeric group will not fit either shape, since its
computation reads `NUMERIC DIGITS`.

## 2. STRIP's second argument is a set, and omitted is not the same as null

Documented, in rexxref's own words (`funct.xml`, `bifStrip`):

> The third argument, chars, specifies the set of characters to be removed, and
> the default is to remove all whitespace characters (spaces and horizontal
> tabs). If chars is a null string, then no characters are removed.

with `STRIP("12.0000", "T", '.0') --> "12"` as its own two-character example.
The crate's `STRIP` builtin already implemented and documented this, so the
method shares it rather than deciding anything.

The consequence for the argument reader is that **omitted and `''` are
different arguments**: omitted takes the whitespace default, `''` is an empty
set that strips nothing. `strip_arguments` therefore uses
`optional_string_or_none_argument` and answers `Option<Vec<u8>>`;
`optional_string_method_argument` beside it would have collapsed the two.

The option does not work that way: `''` is as invalid as `'Z'`, both 93.915
naming the accepted set. Measured, `'  ab  '~strip('')` is 93.915 while
`'abc'~strip('B','')` is `abc`.

The default set is a space **and** a horizontal tab, which the witness shows
without naming either: a `'09'x`-wrapped receiver strips clean.

## 3. ABBREV answers text, not a boolean

`'Print'~abbrev('Pri') + 1` is 2 and `datatype('Print'~abbrev('Pri'))` is `NUM`.
The answer is the one-byte text `1` or `0`, so it goes on to arithmetic.

Its length argument is a **minimum**, so a shorter candidate is not an
abbreviation — and section 5's mutation K is what says the witness sees that.

## 4. `string_compare.rex` allocates nothing

`collect_stress.rs`'s `NO_ALLOCATION_PROGRAMS` gained a row. ABBREV answers an
interned `1`/`0` and COMPARE a counted integer, so a whole program of them
never reaches the heap. Its refusals sibling is *not* on the list, because a
traceback is a wide string. The both-directions assertion at the use site is
what turned this up rather than it passing unnoticed.

## 5. The control — three mutations, predicted before running

**Enumerated by reading each `say` line of the two values witnesses.** 3a's
prediction missed a line because I listed the sends from memory instead; doing
it from the file is the whole difference here.

**J — STRIP's omitted set becomes the empty set.** Predicted: `string_strip.rex`
lines 1, 2, 3, 4 and 8 red — every line whose sends omit the set, plus line 8
where one of three sends does; lines 5, 6, 7 unchanged because they pass one.
Refusals and both compare files green. Measured: `1,4c1,4` and `8c8`, nothing
else. **Confirmed, every part.**

**K — ABBREV's minimum stops being a minimum** (the `info.len() < minimum` arm
dropped). Predicted: `string_compare.rex` lines 2 and 3 only —
`abbrev('', 1)` and `abbrev('Pri', 4)` are the two sends whose minimum is not
already their candidate's own length. Measured: `2,3c2,3`. **Confirmed.**

**L — COMPARE takes its tail from the shorter side.** Predicted: lines 5, 6, 7,
8. Also predicted, and worth writing down: the pad sends on lines 7 and 8 that
do *not* move, because an all-pad tail answers 0 from either side. Measured:
`5,8c5,8`. **Confirmed.**

## 6. Gates

| | |
|---|---|
| G1 `cargo fmt --all --check` | PENDING |
| G2 `cargo clippy --workspace --all-targets -- -D warnings` | PENDING |
| G3 `cargo test --release --workspace --no-fail-fast` | PENDING |
| G4 same with `REXX_CORPUS_GATE=1` | PENDING |
| G5 `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace` | PENDING |
| G6 `REXX_PHASE_GATE=5c` | PENDING |
| G7 `REXX_PHASE_GATE=5d` | PENDING |

Pre-commit chain: method-bodies refresh rc 0 (three rows, `abbrev`, `compare`
and `strip`, `loud` -> `answers`, and no row on any other class moved — read
from the diff), fmt rc 0, clippy rc 0, strict corpus 387 of 387, full workspace
test rc 0.
