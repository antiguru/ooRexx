# Phase 5f Task 1c — the region tests

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 1's third family.
BASE `8330c3890`. Landed at **`GATECOMMIT`**.

Eight more rows: `startsWith caselessStartsWith endsWith caselessEndsWith match caselessMatch
matchChar caselessMatchChar`.

## 1. Where this family's argument layer stops looking like the others

**A missing `match` argument is `88.901` at rc 168, not `93.903` at rc 163.** The C++ takes it as a
*named* argument, so the name reaches the message: `Missing argument; argument match is required.`
`.nil` in the same position is `88.909` rather than a conversion error. Measured, oracle:

```text
'abcABCabc'~startsWith          88.901  rc 168
'abcABCabc'~startsWith(.nil)    88.909
'abcABCabc'~match               93.903  rc 163
```

That is what `MutableBuffer`'s six `answers rc 168` rows have been recording since the 5c follow-up,
and it is why this family's refusals witness ends at rc 168 where the other two end at 163.

## 2. The extraction, and one duplication removed on the way

`starts_with` and `ends_with` are now byte predicates neither receiver owns, parameterised by the
compare -- **not `slice::starts_with`**, because an empty `match` answers `0` on both spellings and
the standard method answers true.

`match_region` split in two: `match_region_arguments`, which reads the offset and length and touches
no receiver, and `match_region_over`, which compares once the bytes are in hand. That is the split the
borrow forces -- `to_text` is `&mut Interp` -- and it is also where the two short-circuits live, so
they are now written once for both receivers.

**`native_mutable_buffer_startswith` and `native_mutable_buffer_endswith` each carried a hand-rolled
copy of `named_string_argument`'s body** while their two caseless twins called the helper. Folding all
four onto the helper is a pure simplification this family happened to be standing on.

## 3. The witnesses

`corpus/lang/string_match.rex` (rc 0) and `corpus/lang/string_match_refusals.rex` (rc 168),
byte-identical to the oracle on all three descriptors on both engines. The refusals program sends two
shapes to a `MutableBuffer` receiver, and its untrapped last send is a bare `startsWith`, so the
88.901 text and the frame line are compared as bytes rather than as a code.

## 4. What clippy caught that the `MutableBuffer` twin gets away with

`caselessMatchChar`'s fold was written as `set_member.to_ascii_uppercase() == byte.to_ascii_uppercase()`
and is `manual_ignore_case_cmp`, so `-D warnings` refused it; it is
`u8::eq_ignore_ascii_case` now.

**`native_mutable_buffer_caselessmatchchar` carries the same expression and is not flagged**, because
there it is spelt across two statements inside a closure over an iterator. So the lint is about the
shape of the expression rather than about the comparison, and the two receivers are not inconsistent
in behaviour -- only in what the linter can see. The MutableBuffer spelling is left alone: changing it
would be an unmeasured edit to a landed family in a commit about a different one.

## 5. The negative control, predicted row by row

Written before the run: stub `native_string_startswith` to raise 88.901 and nothing else; then

```text
string_match.rex           RED, all three descriptors
string_match_refusals.rex  RED on row 20 only
  row 1  bare startsWith     88.901 unchanged -- the stub raises the same error
  row 2  startsWith('a','b') 93.902 unchanged -- the arity check is outside the body
  row 20 startsWith(.nil)    88.909 -> 88.901, the stub never reaching the conversion
  row 21 MutableBuffer bare  88.901 unchanged -- a different receiver's body
  untrapped last send        unchanged, so stderr does not move
strict count 369 -> 367
```

Measured: **367 of 369, exactly those two programs**, and the refusals diff is one line --
`20 raised 88.909` against `20 raised 88.901` -- with stderr byte-identical and rc 168 on both sides.

**So this family's whole refusal surface is witnessed by one row.** Everything else in that program
is raised by the arity check, by a different receiver, or by the same error the stub raises. The
untrapped last send pins the frame line and the message of one error; it is not a check on the body,
and without the `.nil` row nothing here would have gone red on stdout at all.

## 6. Gates

| | |
|---|---|
| G1 `cargo fmt --all --check` | rc 0 |
| G2 `cargo clippy --workspace --all-targets -- -D warnings` | rc 0, zero warnings |
| G3 `cargo test --release --workspace --no-fail-fast` | **G3** |
| G4 | **G4** |
| G5 | **G5** |
| G6 | **G6** |
| G7 | **G7** |
