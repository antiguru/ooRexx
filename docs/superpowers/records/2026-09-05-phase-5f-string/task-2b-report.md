# Phase 5f Task 2b — String's thirty-two operator rows

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 2, D83, D85.
BASE `8051f89d5`. Landed at `644d6d847`.

Thirty-two of the thirty-three operator rows. `?` is Task 2c: it is the one
operator that is not a `rexx_parse::Operator` at all, and it needs a logical
reader this tree has no helper for.

`method-bodies.txt` moves 32 String rows, **26 to `answers` and 6 to
`uncomparable`** — the six strict-ordering sends Task 2a's list exempts, which
until now had never appeared in the table because a `loud` row never reaches
the oracle. This commit is the first real use of `Body::Uncomparable`.

String is now 135 rows: 90 `answers`, 6 `uncomparable`, 39 `loud`. Against the
phase's 112, that is 73 bound and 39 left — 1 (`?`), 25 (Task 3), 13 (Task 4).

## 1. Three things the plan had wrong, each corrected by running

**Arithmetic does not go through `apply_binary`.** The first build raised
`binary operator Plus has no implementation`. That function's own doc says so:
`**`'s exponent is not converted the way its base is, so arithmetic does not
share the operand handling the other families do. The seven arithmetic
operators enter `arith_small_int` then `arith_general`, which is the pair the
expression form and `ir::Op::Arith` also enter — three callers, one path, so
they cannot come to disagree.

**A bare `+` or `-` is the prefix form, not a refusal.** Measured:
`'12'~'+'()` is `12`, `'12'~'-'()` is `-12`, `'12'~'*'()` is 93.903. The unary
reading belongs to exactly the two operators the language has a prefix form
for.

**The receiver converts before the operand is missed.** This one the refresh
caught, not I. With the bodies bound, `REXX_METHOD_BODIES_REFRESH=1` refused to
write and named five regressions:

```text
5 row(s) of the method-body table regressed [...]
  String * (instance arm): loud -> diverge [diverge-both]
  String ** (instance arm): loud -> diverge [diverge-both]
  String / (instance arm): loud -> diverge [diverge-both]
  String // (instance arm): loud -> diverge [diverge-both]
  String % (instance arm): loud -> diverge [diverge-both]
```

The table's receiver is `.String~new('abc')`, and on that receiver the oracle
raises 41.1, where this crate raised 93.903. The rule is not about the
operator: the receiver is converted first, and the missing argument only
surfaces once it converts. Both halves, measured on the oracle:

| send | oracle |
|---|---|
| `.String~new('abc')~'*'()` | `Error 41.1: Nonnumeric value ("abc") used in arithmetic operation.` |
| `'12'~'*'()` | `Error 93.903: Missing argument in method; argument 1 is required.` |
| `.String~new('')~'*'()` | 41.1, naming `""` |

All seven arithmetic operators produce that 41.1 byte for byte, including the
two that answer on a numeric receiver. Four edges were measured before wiring
it: `numeric digits 1` with a numeric receiver is still 93.903 (no rounding
trap), and `'1E999999999'` bare is still 93.903 under `*` and answers
`1E+999999999` under `+` (no overflow trap). So the check is exactly
"is the receiver a number", which is `Interp::arith_operand` — 41.1 with the
operand as it renders — and not a borrowed prefix-plus.

**My own note recording `'12'~'*'()` as 93.903 was right; what I had never
measured was the same send on a non-numeric receiver, which is the receiver
the table uses.** The measurement that existed did not cover the case the
harness runs.

## 2. D83 — the traceback frame — answered by measurement

An operator sent as a *message* carries a `Compiled method "+" with scope
"String".` line that the same operator as an *expression* does not. The
question was whether routing a message through the shared operator dispatch
loses that frame.

It does not, and the witness proves it as bytes rather than by argument:
`string_operators_refusals.rex` ends on an untrapped `s~'+'(1)`, so the frame
is on stderr and compared. The frame comes from `Interp::invoke`, which pushes
it for the send, not from the operator code the send reaches.

## 3. The witnesses

`corpus/lang/string_operators.rex`, rc 0, 24 lines. Every operator is sent
twice — as a message and as a parenthesised expression — so the two dispatches
are compared against each other as well as against the oracle.

`corpus/lang/string_operators_refusals.rex`, rc 215, 80 lines. Missing operands,
41.1 on non-numeric arithmetic operands, 34.901 on logical operands that are
not `0` or `1`, 42.3 on `'1'~'/'('0')`, and the bare-arithmetic pair above.
The six strict-ordering operators are sent **with** an operand; sent bare they
segfault the oracle, which is `corpus/oracle-crashes.txt`'s entry.

Both are byte-identical to the oracle on stdout, stderr and exit status, on
`REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`.

## 4. The control — three mutations, predicted row by row before running

Predictions written to the scratchpad before any mutation was applied.

**A — the receiver conversion deleted** (`_ => {}`). Predicted: rows 25, 27, 28,
36 turn 41.1 into 93.903; rows 1 and 26 do not move because they take the
`Plus`/`Subtract` arms; rows 29–33 do not move because they are already 93.903.
Measured: exactly rows 25, 27, 28, 36. **Confirmed, every part.**

**B — the prefix arms deleted.** Predicted: rows 34 and 35 only. Rows 1 and 26
were predicted *not* to move, because on a non-numeric receiver
`arith_operand` raises the same 41.1 the prefix path does — which is precisely
why rows 34 and 35 had to be added. Measured: rows 34 and 35 only.
**Confirmed, every part.**

**C — one enum mapping changed** (`native_string_op_less_than` forwards
`Operator::GreaterThan`). Predicted: the values file only, the refusals file
green, and both unit tests blind because they check the name column rather
than the enum a body forwards. Measured: the values file only, refusals green,
`operator_spellings_match_the_parser` and `every_listed_operator_has_a_row`
both still pass. **The count was wrong** — I predicted one red line and two
moved, lines 11 and 23, because two program lines send `<`. Both are
message-form sends; neither expression-form line moved, which is the half the
prediction was actually about.

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

Pre-commit chain, all green at the committed tree: method-bodies refresh rc 0,
fmt rc 0, clippy rc 0, strict corpus 379 of 379, full workspace test rc 0.
