# Phase 5f Task 1e — the word readers and `verify`

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 1's fifth family, and the last of
the readers.

BASE `1bfa9b7c9`. Landed at **`GATECOMMIT`**.

Five more rows: `word words wordIndex wordLength verify`. **That closes the 30 reader rows** of the
41 `MutableBuffer` already implements; the 11 with the other return contract are what remain.

## 1. `verify`'s option is the argument with an error of its own

`verify_arguments` is the shared layer: the reference set, the `M`/`N` option defaulting to `N`, a
0-based start, and an optional range. **An unrecognised option is 93.915**, which names the accepted
set, and it is raised before the start is looked at. Measured, oracle:
`'abcabc'~verify('ab', 'Z')` is 93.915 at rc 163, and so is the same send to a `MutableBuffer`.

The word readers need no shared layer beyond the position argument both receivers already took
through `required_position_argument`; `words` takes no argument at all.

## 2. A guard that now has a second caller

The 5c follow-up's Task 3a recorded that `MutableBuffer~verify` was the only caller reaching
`verify_bytes`'s start-past-the-end guard, and left open whether the builtin follows
(`task-3a-report.md` §6.1). **`String~verify` is that second caller now**, and the witness covers the
case rather than leaving it to inference: `'abcabc'~verify('abc', 'N', 9)` and
`'abcabc'~verify('', 'N', 9)` are both 0 on the oracle and on both engines. Whether `VERIFY` the
builtin agrees is still open and still not this phase's.

## 3. The witnesses

`corpus/lang/string_words.rex` (rc 0) and `corpus/lang/string_words_refusals.rex` (rc 163),
byte-identical to the oracle on all three descriptors on both engines.

## 4. The negative control, and the first one whose stderr half earns its place

Written before the run: stub `native_string_verify` to raise 93.903 and nothing else; then
`string_words.rex` goes red on all three descriptors, `string_words_refusals.rex` on rows 8, 9 and 10,
with row 7 unchanged (the stub raises the same error), row 11 unchanged (a `MutableBuffer` receiver),
rows 1 to 6 unchanged (other bodies), **and stderr moving** -- because this family's untrapped last
send is `verify('ab', 'Z')` rather than a bare-argument case.

Measured: **371 of 373, exactly those two programs**, and both halves as predicted:

```text
stdout   8 raised 93.915 -> 93.903      stderr  Error 93.915:  Method option must be one of "MN"; found "Z".
         9 raised 93.924 -> 93.903           -> Error 93.903:  Missing argument in method; argument 1 is required.
        10 raised 93.924 -> 93.903
```

**Across three families the untrapped last send was decorative twice and load-bearing once, and the
difference is which send it is.** 1c's and 1d's were bare-argument cases, so a stub raised the same
error and their stderr never moved; 1e's carries an error the stub cannot reach. A refusals witness
whose final send takes no arguments pins a frame line and nothing else.

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
