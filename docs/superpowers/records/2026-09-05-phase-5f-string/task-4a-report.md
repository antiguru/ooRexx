# Phase 5f Task 4a — the comparison family

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 4.
BASE `e171f6e2f`. Landed at `PENDING`.

`EQUALS CASELESSEQUALS COMPARETO CASELESSCOMPARETO CASELESSABBREV
CASELESSCOMPARE`. String now reads 122 `answers`, 6 `uncomparable`, 7 `loud`:
**105 of the phase's 112 bound**, 7 left — `ceiling floor round modulo`,
`encodeBase64 decodeBase64`, and `hashCode`.

## 1. EQUALS has no argument type to refuse

Every other method in this family reads its operand with `stringArgument` and
refuses a value with no string value at 88.909. `EQUALS` does not:

| send | answers |
|---|---|
| `'abc'~equals(.nil)` | `0` |
| `'abc'~equals(.array)` | `0` |
| `'abc'~equals(1)` | `0` |
| `'abc'~compareTo(.nil)` | 88.909 |
| `'abc'~caselessCompare(.nil)` | 88.909 |

Every object renders, and a rendering that is not the receiver is simply not
equal. So `equals_argument` reads the operand's **string value** where
`compare_to_arguments` beside it uses the strict reader; an omitted operand is
still 93.903 for both. The refusals witness ends untrapped on
`'abc'~compareTo(.nil)`, and rows 7 and 8 beside it are the `equals` half of
the same pair — mutation Y is what says both halves are needed.

## 2. COMPARETO's default length is the one thing the arguments cannot supply

`RexxString::compareToRexx` (`classes/StringClassMisc.cpp:1075`) defaults the
length to `max(mine, theirs) - start + 1`, which needs the *receiver*. So
`compare_to_arguments` answers `Option<usize>` and the body fills it in.

`primitiveCompareTo` (`:1097`) then has three arms a witness has to separate: a
start past both strings is `0`, a start past only the receiver is `-1`, and a
start past only the other is `1`. Output line 8 of the values witness is those
three and nothing else.

Where the compared regions are equal but of different lengths, the longer is
the greater — which is what mutation Z moves, and only on the one line whose
operands differ in length.

## 3. The caseless twins share their exact twins' argument layers

`caselessAbbrev` and `caselessCompare` reuse 3b's `abbrev_arguments` and
`compare_arguments` unchanged; only the byte cores gained a caseless arm.
**The pad is folded too**: measured, `'abc'~caselessCompare('ABCXX','x')` is 0,
so `caseless_compare_at` is not `compare_at` over two upcased copies — the
pad has to be compared caselessly as well, which mutation AA pins.

## 4. The control — three mutations, all confirmed

**Y — EQUALS reads its operand strictly.** Predicted: the values program dies at
output line 3, rc 0 -> 168, stdout keeping lines 1-2 and losing 3-11, stderr
gaining a traceback; and refusals rows 7 and 8 turning `answered 0` into
`raised 88.909` while its own tail does not move. Measured: `3,11d2` with
stderr DIFF and rc 168, and `7,8c7,8`. **Confirmed, every part.**

**Z — COMPARETO's default length uses `min`.** Predicted: values line 6 alone —
lines 4 and 5 have equal-length operands, line 7 passes a length, and line 8's
starts are all past a string. Measured: `6c6`. **Confirmed.**

**AA — the pad stops folding case.** Predicted: values line 11 alone. Measured:
`11c11`. **Confirmed.**

## 5. `git checkout --` cost me the two helpers

Between mutations I ran `git checkout -- crates/rexx-exec/src/dispatch.rs` to
undo a helper added only for mutation Y. It took the file back to `HEAD`, which
discarded `equals_argument` and `compare_to_arguments` — both uncommitted. They
were reconstructed from the script that had written them and the tree verified
byte-identical against the two witnesses before the run continued, so nothing
is lost; recording it because the rule against that command exists precisely
for this and I used it anyway. The remaining two mutations were run by copying
the file to the scratchpad and back.

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

`collect_stress.rs`'s `NO_ALLOCATION_PROGRAMS` gains `lang/string_caseless.rex`:
every answer in it is `1`, `0`, `-1` or a counted integer, all of which fit in a
handle.

Pre-commit chain: method-bodies refresh rc 0 (six rows `loud` -> `answers`, no
row on any other class moved), fmt rc 0, clippy rc 0, strict corpus 399 of 399,
full workspace test rc 0.
