# Phase 5f Task 3d — the bit operations and DATATYPE

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 3.
BASE `afa3cabbd`. Landed at `PENDING`.

`BITAND BITOR BITXOR DATATYPE`. String now reads 111 `answers`, 6
`uncomparable`, 18 `loud`: **94 of the phase's 112 bound**, 18 left — 5 in
Task 3 (`abs max min trunc format`) and 13 in Task 4.

## 1. Each bit operation's default pad is its own identity

`bit_operation_over` takes the pad **already defaulted**, because the default
is not a property of the operation's shape but of the operation itself:
`'ff'x` for BITAND, `'00'x` for BITOR and BITXOR. Naming them
(`convert::BITAND_PAD`, `convert::BITOR_PAD`) is what lets the method rows pick
the right one without repeating the byte.

The consequence a witness has to pin is that the shorter operand is *extended*
with the pad rather than the longer one truncated, so the identity default
makes the tail pass through unchanged. `'abc'~bitAnd` — no operand at all —
answers `abc`, and `'ffff'x~bitAnd('00'x)` is `00FF` where
`'ffff'x~bitAnd('00'x,'00'x)` is `0000`. Both are on the last line of the
values witness, and section 4's mutation P is what says they are load-bearing.

## 2. DATATYPE names the letter where every other option names the string

`error.rs`'s `invalid_option` doc says `found` is the whole option string, and
for `STRIP` and `VERIFY` that is right. `DATATYPE` is the exception, in both
forms. Measured:

| send | reports |
|---|---|
| `'5'~dataType('Zonk')` | `found "Z"` |
| `datatype('5','Zonk')` | `found "Z"` |
| `'  ab  '~strip('Zonk')` | `found "Zonk"` |
| `strip('  ab  ','Zonk')` | `found "Zonk"` |
| `'abcabc'~verify('ab','Zonk')` | `found "Zonk"` |

So `datatype_option_argument` is its own reader rather than
`option_method_argument`, which substitutes the whole string. An empty option
has no first letter at all and reports the NUL byte, which the report renders
`?`.

**This also found a gap in Task 3b.** `string_strip_refusals.rex` tested only
single-letter and empty options, where "the whole string" and "its first
letter" are the same bytes — so it could not have seen the difference. Its
untrapped tail is now `strip('Zonk')`, and the two witnesses together are what
separate the two rules. Mutation Q confirms neither file alone does it.

## 3. The lift

`builtin::datatype` becomes crate-visible, the fourth module to do so.
`datatype_matches` answers `Option<bool>` — `None` for a letter outside the
thirteen — so **the refusal stays with each caller** while the letter test is
shared. The builtin raises `invalid_option` and the method
`method_option_not_recognised`; measured, the two constructors build the same
93.915, and the difference is only which module owns the site.

`datatype_kind` is the no-option answer, `NUM` or `CHAR`.

## 4. The control — three mutations, predicted before running

**P — BITAND's default pad becomes `'00'x`.** Predicted: `string_bits.rex`
lines 2, 3, 5, 6 and 7 — every line with a send that reaches the pad — and
lines 1 and 4 unchanged, one because both operands are a byte long and the
other because it passes a pad of its own. Measured: `2,3c2,3` and `5,7c5,7`.
**Confirmed, every part.**

**Q — DATATYPE's 93.915 substitutes the whole option.** Predicted: the datatype
refusals file on stderr alone, no stdout row (the trap prints the number and
all three are 93.915), and `string_strip_refusals` green. Measured exactly
that. **Confirmed** — and it is the pair of files that catches it, which is the
point of section 2.

**R — DATATYPE's `W` uses the fixed default digits.** Predicted:
`string_datatype.rex` line 10 alone, the only line under `numeric digits 2`,
with the `I` and `9` sends on that same line not moving because their precision
is fixed already. Measured: `10c10`. **Confirmed.**

## 5. Gates

| | |
|---|---|
| G1 `cargo fmt --all --check` | PENDING |
| G2 `cargo clippy --workspace --all-targets -- -D warnings` | PENDING |
| G3 `cargo test --release --workspace --no-fail-fast` | PENDING |
| G4 same with `REXX_CORPUS_GATE=1` | PENDING |
| G5 `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace` | PENDING |
| G6 `REXX_PHASE_GATE=5c` | PENDING |
| G7 `REXX_PHASE_GATE=5d` | PENDING |

Pre-commit chain: method-bodies refresh rc 0 (four rows `loud` -> `answers`,
and no row on any other class moved), fmt rc 0, clippy rc 0, strict corpus 393
of 393, full workspace test rc 0.
