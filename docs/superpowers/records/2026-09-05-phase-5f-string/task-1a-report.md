# Phase 5f Task 1a — the search family

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 1's first family.
BASE `5fc3ded9f`. Landed at **`GATECOMMIT`**.

Six rows move from `loud` to a body: `pos caselessPos contains caselessContains lastPos
caselessLastPos`.

## 1. The extraction, and why the oracle settles its shape

The argument layer and the search are now four functions in `dispatch.rs` that neither receiver owns:
`forward_search_arguments`, `forward_search`, `backward_search_arguments`, `backward_search`. Each
class's native supplies only its own bytes.

**That is what the C++ does, which is the argument for doing it this way rather than a judgement about
tidiness.** `RexxString::posRexx` (`classes/StringClassMisc.cpp:581`) and `MutableBuffer::posRexx`
(`classes/MutableBufferClass.cpp:803`) both forward to
`StringUtil::posRexx(getStringData(), getLength(), ...)` (`classes/support/StringUtil.cpp:184`). The
oracle shares the body and differs in the byte source; so does this now.

`buffer_caseless_pos` is gone: it differed from `buffer_pos` only in which scan it called, so the scan
is a parameter and the four `MutableBuffer` natives that used the two helpers now name their scan in
the row. **The caseless scan is still not the plain one with a folded compare** -- that stays as
`caseless_find_forward`'s own doc records it.

**The borrow shape decided the rest.** `Interp::to_text` is `&mut self` where `buffer_state` is
`&self`, and `take_result_buffer` is `&self` with interior mutability, so the String natives scope the
byte borrow around the search alone. Resolving the receiver into an owned `Vec` would have been
simpler and would have put an allocation back on every send, which is what entry 75 has just finished
taking out of this path.

## 2. The witnesses

`corpus/lang/string_search.rex` -- values, rc 0. `corpus/lang/string_search_refusals.rex` -- the
argument layer's own errors, rc 163, in `array_make_string_refusals.rex`'s counter-and-trap shape,
with three rows sent to a `MutableBuffer` receiver so a change that moved one receiver and not the
other would show. Both byte-identical to the oracle on all three descriptors, `REXX_ENGINE=ir` and
`REXX_ENGINE=tree-walker`.

Filed in `corpus/phase-5c.txt` and `EXPECTED_SUBSET_5C` together, which that assertion requires.
**The strict corpus count moved 363 → 365**, which is how this report knows the two programs are run
rather than only parsed.

## 3. The negative control

Predicted before running: stub `native_string_pos` so it raises 93.903 and nothing else; then
`string_search.rex` goes red on all three descriptors, `string_search_refusals.rex` goes red on rows
2, 3, 4, 5 and 6, the six `mutablebuffer_*.rex` do not move, and the count falls to 363.

Measured: **363 of 365, and exactly those two programs.** `string_search.rex` differed on stdout,
stderr and exit code; `string_search_refusals.rex` on stdout, printing `93.903` at rows 2, 3, 4 and 6
where the oracle gives `93.924`, `93.923`, `93.924` and `answered 0`.

**Row 5 was predicted to move and did not, and that is worth keeping.** `s~pos('a', 1, 2, 3)` is
93.902 under the stub as well, because `Arity::Fixed(3)` refuses a fourth argument *before* the body
is entered. The count check is outside every body here, so no stub can ever fake it wrong -- and
equally, no witness of mine tests it.

**The refusals witness's untrapped last send did not move either.** It is `s~pos` with no argument,
which the stub also raises as 93.903, so its stderr is identical under the stub. That device pins the
frame line and the message text of *one* error; it is not a check on the body, and this control is
what shows the difference.

## 4. Gates

| | |
|---|---|
| G1 `cargo fmt --all --check` | rc 0 |
| G2 `cargo clippy --workspace --all-targets -- -D warnings` | rc 0, zero warnings |
| G3 | **G3** |
| G4 | **G4** |
| G5 | **G5** |
| G6 | **G6** |
| G7 | **G7** |
