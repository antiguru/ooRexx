# Phase 5f Task 3a — the pad family: CENTER, CENTRE, LEFT, RIGHT, COPIES

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 3.
BASE `370fdbf3c`. Landed at `a9fb5e675`, report corrected at `23d7f85eb`.

Five of Task 3's twenty-five. String now reads 96 `answers`, 6 `uncomparable`,
33 `loud`: **79 of the phase's 112 bound**, 33 left.

## 1. All four computations needed lifting

The plan asks which of the 25 needed a lift. For this family the answer is
**all of them**: not one of `center`, `left`, `right`, `copies` had a `&[u8]`
core, and every one was a `(interp, name, args)` builtin entry with the
computation inlined after its own argument reads. Four cores came out —
`center_bytes`, `left_bytes`, `right_bytes`, `copies_bytes` — and the builtins
now call them, so the builtin and the method cannot come to disagree. That is
the shape `numeric::sign_of` already set for `String~sign`.

Three of the four answer `Option<Vec<u8>>`, where `None` is the oracle's
`return this` arm: `center` when the width equals the receiver's length
(`classes/StringClassSub.cpp:59`), `right` likewise (`:485`), and `copies` for
a count of one (`classes/StringClassMisc.cpp:288`). `left` has no such arm
(`:248`) and answers bytes. **Section 5 shows those `None` arms are currently
unwitnessed**, which is a fact about this tree, not about the cores.

## 2. The argument layer is not the builtin's, and this family shows it twice

The global constraint says the builtin computes the value and none of the error
surface. Measured, that is not a style rule here — the two surfaces disagree in
both the error *number* and the *substitution*:

| send | reports |
|---|---|
| `'abc'~center('7.5')` | 93.923 `Invalid length argument specified; found "7.5".` |
| `center('abc','7.5')` | 40.12 `CENTER argument 2 must be a whole number; found "7.5".` |
| `'ab'~copies('-1.0')` | 93.906 `… must be zero or a positive whole number; found "-1.0".` |
| `copies('ab','-1.0')` | 93.906 `… must be zero or a positive whole number; found "-1".` |

The last pair is the sharp one: same sub-code, same message template, and the
builtin substitutes the **converted** value where the method substitutes the
text it was handed. A shared argument layer would have got that wrong silently,
and it is why the refusals witness ends untrapped on exactly that send.

`error.rs`'s own doc for `argument_not_non_negative` cites the builtin form
(`copies('ab','-1.0')` reports `found "-1"`), which stands — the method's
different substitution is a second fact beside it, not a correction.

## 3. COPIES counts, the other three measure

`CENTER`/`CENTRE`/`LEFT`/`RIGHT` share `pad_arguments`: a required
`lengthArgument` (93.903 omitted, 93.923 otherwise) and an
`optionalPadArgument` (93.922 for anything but exactly one byte, and 88.909 —
an 88, not a 93 — for a value with no string value at all).

`COPIES` shares none of it. Its count is `nonNegativeArgument`, 93.906, a
different sub-code for the same shape of mistake, so it has its own
`copies_argument`. Measured: `'abc'~center(-1)` is 93.923 and
`'abc'~copies(-1)` is 93.906.

`CENTER` and `CENTRE` are one body registered twice, as
`memory/Setup.cpp:582`-`:583` registers them.

## 4. Twenty-two ooTest assertions unblocked

`corpus/bif-exempt.txt` dropped from 76 rows to 54. Every one of the 22 was
exempt as `Phase 5` and every one is unblocked by the same send: `~copies` on a
String. `XRANGE.testGroup:297` is `self~assertSame(xrange()~copies(2), …)` and
`D2C.testGroup:102` is `self~assertSame('7F'x||'FF'x~copies(249), d2c(vlong))`
— read, not inferred from the count.

## 5. Nine rows on other classes moved with them

`method-bodies.txt` changed 21 rows, not five. The other sixteen are the
interesting half, and they are the same effect the ooTest rows above are:

- **`DateTime~maxDate`, `~minDate`, `~offset` and `TimeSpan~"+"`, `~"-"`,
  `~duration`, `~makeString`, `~string`** — eight rows, `loud` -> `answers`.
  These are Rexx-level bodies in `CoreClasses.orx` whose formatting reaches
  `~right`/`~left`, so binding this family is what let them run.
- **`DateTime~elapsed`** — `loud` -> `unstable`. It answers now, and its answer
  is an elapsed time, so two runs of the same probe differ. `unstable` is the
  correct verdict for it and not a defect.
- **Seven `CircularQueue` rows** stay `loud` and change their evidence: what
  they hit first moved from `method "LEFT" of class "String"` to
  `method "MAKEARRAY" of class "Queue"` (and `"SUPPLIER"` for `~supplier`).
  Nothing about them improved except that this family is no longer what blocks
  them.

None of this was predicted; it is what the refresh reported and it was read
row by row rather than summarised from the count.

## 6. The control — three mutations, predicted before running

**G — COPIES reads its count as a length.** Predicted: refusals rows 17 and 18
turn 93.906 into 93.923 and the stderr tail moves; row 16 does *not* move,
because an omitted argument is 93.903 either way; the values file stays green.
Measured exactly that. **Confirmed, every part.**

**H — CENTER's `return this` arm removed.** Predicted: **nothing moves at all.**
With `width == len` the general path takes the truncating branch, which yields
the same bytes, and this tree has no identity accessor to tell the two apart
(`identityHash` and `hashCode` are Task 4). Measured: nothing moved, on either
file. **Confirmed — and the point of running it is that it says out loud that
all three `None` arms are unwitnessed today rather than leaving that assumed.**

**I — CENTER's odd byte taken from the other side.** Predicted: values lines 2,
3, 4 and 5 red, refusals green. Measured: lines 2, 3, 4, 5 **and 10**, and
inside lines 4 and 5 two further sends moved that the prediction did not name
(`'abcd'~center(3)` and `'abcde'~center(4)`). The line-set prediction missed
line 10 because I enumerated the file's sends from memory instead of reading
them; the mutation is caught in five places, so the witness is not the weak
part here.

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

Pre-commit chain: method-bodies refresh rc 0 (21 rows changed -- see section
5), fmt rc 0, clippy rc 0, strict corpus 383 of 383, full workspace test rc 0.
