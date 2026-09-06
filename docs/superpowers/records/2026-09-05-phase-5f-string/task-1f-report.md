# Phase 5f Task 1f — the range writes, and the row that shares no argument layer

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 1's sixth family and the first of
the mutators.

BASE `1c849c62d`. Landed at `6c9dfd28f`.

Five more rows: `insert overlay replaceAt delStr delWord`.

## 1. The second contract

These are the first rows whose `MutableBuffer` twin does something else with the answer. Measured,
oracle: `'abcdef'~insert('XY', 2)` is `abXYcdef` and the receiver is still `abcdef`, where
`.MutableBuffer~new('abcdef')~insert('XY', 2)` returns *the buffer*, now `abXYcdef`. So the core and
the argument layer are shared and the ending is not: `String` builds into the result buffer and
answers `text_built`, the buffer writes itself back and answers the receiver.

`replace_at_plan` and `replace_at_bytes` are the extraction that keeps `replaceAt`'s span arithmetic
in one place -- which of the receiver is overwritten, how long the answer is, and the three-part
assembly -- while each native keeps its own capacity handling and ending.

## 2. `replaceAt` shares no argument layer, and this is where the plan was wrong

The plan said the 41 shared rows share the core *and* the argument layer. **That is false for exactly
one row.** Measured on the oracle, the same send to the two receivers:

| send | `MutableBuffer` | `String` |
|---|---|---|
| `~replaceAt` | 88.901 | 93.903 |
| `~replaceAt('X')` | 88.901 | 93.903 |
| `~replaceAt('X', 0, 1)` | 88.912 | 93.924 |
| `~replaceAt('X', 'x', 1)` | 88.912 | 93.924 |
| `~replaceAt('X', 1, 1, 'ab')` | 88.910 | 93.922 |
| `~replaceAt(.nil, 1, 1)` | 88.909 | 88.909 |

`RexxString::replaceAt` opens `stringArgument(newStrObj, ARG_ONE)`
(`classes/StringClassSub.cpp:380`); `MutableBuffer::replaceAt` opens `stringArgument(str, "new")`
(`classes/MutableBufferClass.cpp:572`) and takes its position and pad by name too. One method name,
two argument conventions, in one interpreter.

**`insert` is not like this** -- both receivers answer 93.906 for a bad second argument -- so it is
`replaceAt` alone and not something about mutators. The plan was corrected at `1c849c62d`, before any
of this code was written.

**A witness that sends only good arguments cannot see any of it**: every divergence is on the refusal
path. So `string_edits_refusals.rex` sends the five diverging shapes to *both* receivers as pairs,
and an `insert` pair beside them as the contrast that shows the divergence is specific rather than
general.

## 3. The negative control, predicted row by row

Written before the run: stub `native_string_replaceat` to raise 93.903 and nothing else; then rows
20, 22, 24 and 26 move, **every `buf` partner stays put**, rows 1 to 15 and the `insert` pair do not
move, stderr moves because the untrapped last send is `replaceAt('X', 0, 1)`, and the count falls
375 -> 373.

Measured: **373 of 375**, and the diff is exactly

```text
20 raised 93.924 -> 93.903      stderr  Error 93.924:  Invalid position argument specified; found "0".
22 raised 93.924 -> 93.903           -> Error 93.903:  Missing argument in method; argument 1 is required.
24 raised 93.922 -> 93.903
26 raised 88.909 -> 93.903
```

**The paired rows are what make that readable.** Each moved `str` row sits beside a `buf` row that did
not, so the control shows the divergence belongs to the receiver rather than to the witness -- a
single-receiver witness would have shown four numbers changing and proved nothing about whose they
were.

## 4. Three more citations written from the shape of the file

`insert` was drafted as `StringClassSub.cpp:294`, `overlay` as `:340` and `delstr` as `:263`. The real
sites are `:170`, `:292` and `:117`. That is six wrong line numbers across two families now, every one
written from memory of the file's layout and every one caught by looking it up before the code was
applied. A C++ line number now gets written only by the command that reads it.

## 5. Gates

| | |
|---|---|
| G1 `cargo fmt --all --check` | rc 0 |
| G2 `cargo clippy --workspace --all-targets -- -D warnings` | rc 0, zero warnings |
| G3 `cargo test --release --workspace --no-fail-fast` | rc 0 |
| G4 same with `REXX_CORPUS_GATE=1` | rc 0, strict corpus `375 of 375 matching` |
| G5 `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace` | rc 0 |
| G6 `REXX_PHASE_GATE=5c` | rc 0 |
| G7 `REXX_PHASE_GATE=5d` | rc 0 |

Run at `6c9dfd28f`, 03:24:39 to 03:39:01.
