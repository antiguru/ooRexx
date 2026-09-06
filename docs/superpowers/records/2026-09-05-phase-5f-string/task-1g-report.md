# Phase 5f Task 1g — the whole-string rewrites, closing the shared set

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 1's last family.
BASE `863c0cb67`. Landed at **`GATECOMMIT`**.

Six more rows: `changeStr caselessChangeStr translate lower space append`. **That closes all 41 rows
`MutableBuffer` already implements.** `String`'s instance arm now reads 47 `answers` against 71
`loud`.

## 1. `append` is the second row that shares nothing

`String~append` is `RexxString::concatRexx` registered under a second name with arity one --
`AddMethod("Append", RexxString::concatRexx, 1)` (`memory/Setup.cpp:578`). `MutableBuffer~append` is
`MutableBuffer::appendRexx` at `A_COUNT` (`:1422`). So one is concatenation answering a new string
and the other is a variadic in-place write, and they share neither an argument layer nor a body.
Measured, oracle: `.MutableBuffer~new('abc')~append('X', 'Y')` is `abcXY` where
`'abcABCabc'~append('X', 'Y')` is 93.902.

That makes two of the 41 — `replaceAt` and `append` — where the plan's "share the core and the
argument layer" does not hold.

## 2. The witness caught a real defect, which is what it is for

`append` was first written to read its argument through `string_method_argument`, which refuses
`.nil` at 88.909. **The oracle renders it**: `'abcABCabc'~append(.nil)` is
`abcABCabcThe NIL object`, because `concatRexx` converts through `requestString`. Both witnesses went
red on that row before anything was committed.

The fix routes `append` through `Interp::apply_binary(Operator::Concatenate, ...)` rather than
reading a string argument -- which is what the C++ does, the method being `||`'s implementation under
another name -- and both witnesses went byte-identical.

**A values-only witness would have caught this one**, unlike every divergence in 1c through 1f. It
showed because the witness sends `.nil` rather than only well-formed arguments.

## 3. The negative control

Written before the run: stub `native_string_append` to raise 93.903 and nothing else; then row 23
moves and nothing else does, stderr stays put because the untrapped last send is `changeStr`, and the
count falls 377 -> 375.

Measured: **375 of 377**, and the diff is the single line
`23 str abcABCabcThe NIL object` against `23 raised 93.903`, stderr identical.

**Row 23 is the row that caught the defect above, and it is the only row here a stub can move.**
Without it, `append`'s rows would witness the arity check and nothing else.

## 4. What the table learned, and what it did not

`String`'s 47 `answers` rows break down as **32 `rc 163`, 11 `rc 0`, 4 `rc 168`** -- the same shape
`MutableBuffer`'s 51 have. Two thirds of them record agreement about a missing argument rather than
about a result. Forty-one methods with bodies, oracle-checked against fourteen corpus programs, and
what `method-bodies.txt` learned about most of them is that their arity check is right. That is D84
arriving on the second class to be measured against it.

## 5. Gates

| | |
|---|---|
| G1 `cargo fmt --all --check` | rc 0 |
| G2 `cargo clippy --workspace --all-targets -- -D warnings` | rc 0, zero warnings |
| G3 `cargo test --release --workspace --no-fail-fast` | **G3** |
| G4 | **G4** |
| G5 | **G5** |
| G6 | **G6** |
| G7 | **G7** |
