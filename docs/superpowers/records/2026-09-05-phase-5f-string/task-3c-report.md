# Phase 5f Task 3c — the eight base conversions

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 3.
BASE `d6aaf93b6`. Landed at `edc6c6d77`.

`B2X C2X X2B X2C` at no arguments and `C2D D2C D2X X2D` at one. String now
reads 107 `answers`, 6 `uncomparable`, 22 `loud`: **90 of the phase's 112
bound**, 22 left — 9 in Task 3 and 13 in Task 4.

## 1. The lift split three ways, and the third way is new

`builtin::convert` becomes crate-visible, the third module to do so after
`numeric` and `string`.

- **Four trivial**: `b2x_bytes`, `c2x_bytes`, `x2b_bytes` take `&Interp` for
  `buffer`; `x2c_bytes` needs nothing at all, since `pack_hex` sizes its result
  from its input.
- **One with the arguments in front**: `x2d_c2d_over` takes the string and an
  already-read length, because there the length's range check runs *before*
  anything looks at the string. Measured, `'ZZ'~x2d(-1)` is the length's
  93.923 where `'ZZ'~x2d(4)` is the invalid-character 93.933.
- **One with a length-reader closure**: `d2x_d2c_over` reads its length
  *through the caller*, because there the ordering is the other way round.

The closure is the new shape. It exists because the two forms do not merely
raise different numbers for the same mistake — they check in a different
order, and no fixed argument list can express both.

## 2. The ordering, measured three ways

| send | reports |
|---|---|
| `d2x('x','abc')` — builtin | 40.12, the length's **type** |
| `d2x('x',-1)` — builtin | 93.928, the **value** |
| `'x'~d2x('abc')` — method | 93.928, the **value** |
| `'x'~d2x(-1)` — method | 93.928, the **value** |
| `'1.5'~d2x('abc')` — method | 93.923, the length |

So the builtin is *length type → value → length range*, and the method is
*value → length*. `'1.5'` is what separates the method's two halves: it passes
`scan_decimal`, so the length gets its turn, and only afterwards would the
significant-decimals check have refused it.

## 3. The unit test caught a regression I wrote

The first cut moved the builtin's whole length read into the closure, which
collapsed its *type* check onto its *range* check and put both after the value.
`builtin::convert::tests::the_call_layer_is_checked_before_the_operation_layer`
went red on `d2x('x','abc')`, exactly the row it exists for. The fix keeps
`whole_number` in the builtin and passes only `length_of` through the closure,
which reproduces all five rows of the table above.

Worth recording plainly: the ordering was already pinned by a test written for
the builtin, and that test is what stopped a method-side refactor from
loosening it.

## 4. What the witnesses pin

`corpus/lang/string_convert.rex`, rc 0, 30 lines — the residue grouping rule
(`'101'~b2x` is 5, `'101 0000'~b2x` is 50), the signedness a length introduces
(`'ff'~x2d` is 255, `'ff'~x2d(2)` is -1), and the zero-length short-circuit
that answers before validating anything (`'zz'~x2d(0)` is 0).

`corpus/lang/string_convert_refusals.rex`, rc 163, 71 lines — the four refusal
families, and the ordering rows above. `'x'~d2c` is **93.929** where `'x'~d2x`
is 93.928: the sub-code names the method, which is one distinction more than
the shape suggests.

Both byte-identical to the oracle on all three descriptors and both engines.

## 5. One more ooTest assertion row

`assertions.rs`'s `EXEMPT` drops from 13 rows to 12.
`Literals::test_hexadecimal` occurrence 15 is `.String~xdigit~x2c`, which this
task's `~x2c` unblocks. 4247 of 4259 rows now pass. The row was found by
reading the exempt list for an expression naming a method this task binds, and
confirmed by the suite going green once it was removed — not by counting.

## 6. The control — three mutations, predicted before running

**M — the zero-length short-circuit moved below the string's validation.**
Predicted: the values program dies at its last line, rc 0 -> 163, stdout loses
line 10 alone, stderr gains a traceback; refusals green. Measured: `10d9`,
stderr DIFF, rc 163, refusals green. **Confirmed.**

**N — the length read moved above the value check.** Predicted: refusals rows
13 and 14 only — rows 8-12 have no length to read, row 15 already reports the
length, row 18's value is good. Measured: `13,14c13,14`. **Confirmed, every
part.**

**O — D2C stops doubling its length into hexadecimal digits.** Predicted: values
lines 6 and 8, the only two sends carrying a D2C length. Measured: `6c6 8c8`.
**Confirmed.**

## 7. Gates

| | |
|---|---|
| G1 `cargo fmt --all --check` | rc 0 |
| G2 `cargo clippy --workspace --all-targets -- -D warnings` | rc 0, zero warnings |
| G3 `cargo test --release --workspace --no-fail-fast` | rc 0 |
| G4 same with `REXX_CORPUS_GATE=1` | rc 0 |
| G5 `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace` | rc 0 |
| G6 `REXX_PHASE_GATE=5c` | rc 0 |
| G7 `REXX_PHASE_GATE=5d` | rc 0 |

Pre-commit chain: method-bodies refresh rc 0 (eight rows `loud` -> `answers`,
and no row on any other class moved), fmt rc 0, clippy rc 0, strict corpus 389
of 389, full workspace test rc 0.
