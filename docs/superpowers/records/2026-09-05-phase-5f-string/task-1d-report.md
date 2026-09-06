# Phase 5f Task 1d — the extraction readers

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 1's fourth family.
BASE `62c6bae69`. Landed at **`GATECOMMIT`**.

Five more rows: `substr [] subChar subWord subWords`.

## 1. The extraction, and what the C++ says about it in its own words

`substr_arguments` is the shared layer -- a 0-based start, an optional length, and a pad defaulting
to a blank, with a pad wider than one character refused at 93.922 here rather than in the core.

**The C++ states this design at the site.** `RexxString::substr`
(`classes/StringClassSub.cpp:598`) is one line under the comment *"use the common code shared with
MutableBuffer"*, and that common code is `StringUtil::substr`'s padding overload
(`classes/support/StringUtil.cpp:66`) -- there are two overloads and the other, at `:128`, takes no
pad. `MutableBuffer::substr` calls the same one.

`[]` is `substr`'s two-argument form with different defaults, and the difference is not in the
argument layer: the length defaults to one byte, is capped at the end of the receiver, and never
pads. That cap needs the bytes, so it stays in each receiver's own body.

## 2. `subWords` answers an Array, and the witness has to ask

`RexxString::subWords` returns `ArrayClass *` (`classes/StringClassWord.cpp:200`) where `subWord`'s
return type is `RexxString *`. **Rendering cannot tell them apart** -- an `Array` of three words and
a string holding two newlines print identically -- so the witness asks for `~items` and `~class~id`
rather than reading the answer off the page.

## 3. The witnesses

`corpus/lang/string_extract.rex` (rc 0) and `corpus/lang/string_extract_refusals.rex` (rc 163),
byte-identical to the oracle on all three descriptors on both engines.

## 4. The negative control, and how it differs from 1c's

Written before the run: stub `native_string_substr` to raise 93.903 and nothing else; then
`string_extract.rex` goes red on all three descriptors, `string_extract_refusals.rex` on rows 2, 3, 4
and 5, with row 1 unchanged (the stub raises the same error), row 6 unchanged (the arity check is
outside the body), row 13 unchanged (a `MutableBuffer` receiver), the untrapped last send unchanged,
and the strict count falling 371 -> 369.

Measured: **369 of 371, exactly those two programs**, and the refusals diff is exactly rows 2 to 5 --
`93.924 93.924 93.923 93.922` against four `93.903` -- with stderr byte-identical.

**Four rows carry this family's surface where 1c's was carried by one.** That is a property of
`substr`'s arguments, which raise four distinguishable errors, and not of how either witness was
written. A family whose arguments are all one shape needs a row that reaches past them -- 1c's `.nil`
row -- or nothing witnesses the body at all.

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
