# Phase 5f Task 4b — the numeric group

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 4.
BASE `41a8ba94d`. Landed at `7512e1246`.

`CEILING FLOOR ROUND MODULO`. The refresh moved exactly those four rows from
`loud` to `answers` with zero regressions. String now reads 126 `answers`, 6
`uncomparable` and 3 `loud`: **109 of the phase's 112 bound**, 3 left — `encodeBase64 decodeBase64` and `hashCode`.

## 1. The first task in this phase with no core to bind to

Tasks 2 and 3 lifted cores out of `builtin/` that a BIF already used. There is
no `FLOOR()`, `CEILING()`, `ROUND()` or `MODULO()` builtin, so there was
nothing to lift — and three of the four could not be written in `rexx-exec` at
all, because `Number`'s `negative`, `digits` and `exponent` are `pub(crate)`
and each of these moves the value by one at a chosen digit position.

`crates/rexx-num/src/rounding.rs` is new: `floor`, `ceiling`, `round` and
`is_integer`. `MODULO` stayed in `rexx-exec` — it is `//` plus a sign
correction, and `div`/`add` are already public.

**An i64 route was never available.** `'1E100'~floor` is the full 101-digit
integer under DIGITS 9: the answer's width is bounded by the exponent, not by
the precision.

## 2. Rounded to DIGITS first, then stepped

`NumberString::floor` is `prepareNumber(number_digits(), ROUND)->floorInternal()`
(`classes/NumberStringClass.cpp:1591`), and both halves are visible from a
program:

| send | answer | why |
|---|---|---|
| `'123456789.5'~floor` at DIGITS 9 | `123456790` | ten digits round to nine first, leaving no decimals to floor |
| `'1234'~floor` at DIGITS 3 | `1230` | the rounding is the whole of the answer |
| `'0.999999999'~floor` | `0` | nine digits survive, and they are all decimals |

So `to_integer` rounds, then asks one question of the rounded value, then hands
it to `trunc` — which is why `floor` never produces exponential form.

## 3. ROUND takes a half away from zero

The comment above `NumberString::round` says "this is really defined as
floor(number + .5)". `roundInternal` does not do that, and the two differ on
every negative half: `'-2.5'~round` is `-3` where `floor(-2)` is `-2`.
`'-0.4'~round` is `0` and not `-0`, because the truncation drops the sign along
with the digits.

## 4. MODULO's wholeness gate bounds the exponent, not the digit count

`NumberString::isInteger` (`:3830`) returns true for a zero exponent before it
looks at anything else, and only then compares the adjusted length against
`createdDigits`. The two come apart, which is why the witness carries the pair:

| send at DIGITS 9 | answer |
|---|---|
| `'1234567890'~modulo(7)` | `3` — ten digits, but the exponent is zero |
| `'1E9'~modulo(7)` | 93.940 — one digit, adjusted length ten |
| `'1E8'~modulo(7)` | `2` |
| `'12345678901'~modulo(7)` | 26.12 — whole, but `//`'s quotient needs ten digits |

The last row says the gate and the arithmetic are separate checks: the target
passes and the operation still refuses.

**MODULO is not `//`.** A negative remainder gains the divisor, so
`'-13'~modulo(5)` is `2` where `-13 // 5` is `-3`; a zero remainder does not,
so `'-10'~modulo(5)` is `0`. It keeps `//`'s scale: `'10.0'~modulo(3)` is `1.0`.

## 5. Four refusals in a fixed order

93.940 is new — `&1 method target must be a whole number; found "&2".` — and
`MODULO` is its only raiser.

| send | answer |
|---|---|
| `'abc'~modulo()` | 93.943, the target has no numeric value |
| `'1.5'~modulo()` | 93.940, the target is not whole — before the divisor is read |
| `'55'~modulo()` | 93.903 |
| `'55'~modulo('a')` | 93.907 |

The divisor's three ways of being wrong — not numeric, not whole, not positive
— share 93.907, so `.nil`, `'1E9'`, `0`, `-3` and `2.5` all report it.

## 6. The control

Predictions in the scratchpad before any mutation ran, one per instrument:

* **A** — `cargo test -p rexx-num --test rounding`, the primitive's unit tests.
* **B** — `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`, the
  differential witnesses.

The control was designed to separate them rather than to show that something
goes red, because the two overlap by construction: the witnesses were written
from the same measurements as the unit tests.

| mutation | predicted | measured |
|---|---|---|
| M1 `floor`'s Lower arm ignores its decimals test | A red, B red | **confirmed** — A `a_value_with_only_zero_decimals_is_already_whole`, B `string_rounding.rex` stdout |
| M2 the half boundary `>= 5` becomes `> 5` | A red, B red | **confirmed, and under-predicted** — A reddened `a_step_may_carry_past_every_digit` as well, which I had not named |
| M3 `is_integer`'s precision bound `>` becomes `>=` | A red, B red | **confirmed** — B differs on stdout, stderr *and* exit code, because `'1E8'~modulo(7)` starts raising |
| M4 the divisor's *presence* is read before the target check | A green, B red on rows 7 and 8 | **half falsified** — row 7 moved to 93.903, row 8 did not move at all |
| M5 `modulo_over` drops the negative correction | A green, B red | **confirmed** |

**M4 is the finding.** Reading whether the divisor is *present* is not reading
whether it is *valid*, and only the first of those two checks moved: row 8
(`'1.5'~modulo('a')`) still reports 93.940 under M4, because the divisor is
there and the target check still runs before anything looks at it. So M4 left
row 8 unwitnessed, and a second mutation was needed for it.

M4 and M5 are also the two that answer whether the corpus witnesses earn their
place over the unit tests. Both are green on A: the ordering of the refusals
and the sign correction are properties of the *binding*, and `rexx_num` never
sees either. The unit tests localise a failure to the primitive; only the
corpus sees the layer above it.

Two more, predicted after M4's result and before either ran:

| mutation | predicted | measured |
|---|---|---|
| M4b the divisor's *validity* is read before the target check as well | A green, B red with row 7 at 93.903 and row 8 at 93.907 | **confirmed exactly** |
| M6 `native_string_floor` replaced by a bare 93.903 raise — the plan's own gate control | A green, B red on `string_rounding.rex` | **confirmed, and under-predicted** — the refusals program reddens too, its row 1 being `'abc'~floor` |

So every refusal row in the ordering claim now has a mutation that moves it,
and the plan's "replace one landed body with a stub" control is measured rather
than assumed.

**Nothing pre-existing could have caught any of these**, and that was checked
rather than assumed: `corpus-l1/` carries `modulo_test_*.rex` and
`round_test_bug1265.rex`, but nothing runs them — the only mention of
`corpus-l1` in the crate tree is a parse-depth comment in
`crates/rexx-parse/src/expr.rs`. Every catcher above is something this task
added.

## 7. Bookkeeping

`corpus/refusal-sites.tsv` gained `method_target_not_whole` and **19 of its
rows moved a line number**: the new constructor sits above them in `error.rs`,
and column 4 is a definition site. They were re-derived from the source rather
than shifted by arithmetic, and `refusal_sites.rs` agrees.

`collect_stress.rs`'s `NO_ALLOCATION_PROGRAMS` is untouched — both witnesses
allocate, `string_rounding.rex` conspicuously so at 101 digits.

## 8. What is left

`encodeBase64 decodeBase64` and `hashCode`. Recon, read-only, done while this
task's gates were queued:

* **Base64** is the standard alphabet with `+`, `/` and `=` padding
  (`classes/StringClassConversion.cpp:90` and `:153`). Refusals are one
  message, 93.962 `Invalid Base64 encoded string.`, for a length that is not a
  multiple of four, a character outside the alphabet, and an `=` anywhere but
  the last one or two positions. A null string round-trips to itself.
  `'ff'x~encodeBase64` is `/w==`, which is the non-ASCII case the plan asks for.
* **`hashCode` answers eight bytes, little-endian**, and for a String it is the
  Java string hash: `'abc'~hashCode` is `x2c('6278010000000000')`, 96354.
  `''~hashCode` is eight zero bytes and `.nil~hashCode` is `0xDEADBEEF`.
  **`.array~new~hashCode` is an address** and does not reproduce between runs,
  so the four rows that move are not all alike — that is Task 4d's problem, and
  it is the plan's own open question 2.

## 9. Gates

Seven gates, run over the committed tree.

| gate | command | status |
|---|---|---|
| G1 | `cargo fmt --all --check` | rc 0 |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | rc 0 |
| G3 | `cargo test --release --workspace --no-fail-fast` | rc 0, 0 failed suites |
| G4 | G3 with `REXX_CORPUS_GATE=1` | rc 0, 0 failed suites |
| G5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | rc 0, 0 failed suites |
| G6 | `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1` | rc 0, 0 failed suites |
| G7 | `REXX_PHASE_GATE=5d REXX_CORPUS_GATE=1` | rc 0, 0 failed suites |

Run by `scratchpad/gates-4b.sh` over the committed tree, its status file opening
with `sha 7512e12461df4a082cf013dea099c038b20ea5db`.

Pre-commit chain: method-bodies refresh rc 0 (the four rows above `loud` ->
`answers`, no row on any other class moved), `cargo fmt --all --check` rc 0,
clippy rc 0, strict corpus 401 of 401, `rexx-num` rc 0, `refusal_sites`,
`coverage`, `collect_stress` and `sourceline_oracle` each rc 0.

## 10. `Object~hashCode` is deviation 4 at a second message

Raised as a fork for Task 4d and settled without one, because the tree had
already met it. `RexxObject::hashCode` is `getHashValue()` rendered as eight
raw bytes, and `getHashValue()` is virtual with
`identityHash() { return ((uintptr_t)this) ^ UINTPTR_MAX; }`
(`classes/ObjectClass.hpp:340`) as its base. That is the same value
`~identityHash` returns through a different door, and `native_identity_hash`
(`crates/rexx-exec/src/dispatch.rs:4566`) already answers the handle under
deviation 4's licence, with `Object identityHash`'s row reading `unstable`.

So 4d implements `getHashValue` per class rather than "hashCode". Measured
against the oracle, twice per receiver in separate runs: `'abc'` is 96354,
DateTime and TimeSpan reproduce their own values, and `.object~new` does not —
two objects in one run differ from each other and from the next run's. The
value arms reach `answers`; the base arm lands `unstable`, assigned by the
harness rather than chosen, since it runs the oracle twice and sees it move.
