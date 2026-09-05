# Phase 5f Task 1b — the counting and word-search family

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 1's second family.
BASE `a246b692d`. Landed at **`GATECOMMIT`**.

Six more rows: `countStr caselessCountStr wordPos caselessWordPos containsWord caselessContainsWord`.

## 1. The extraction

`wordpos_arguments` joins the four search helpers Task 1a lifted out. `buffer_caseless_wordpos` is
gone the way `buffer_caseless_pos` went: it differed from `buffer_wordpos` only in which scan it
called, so the scan is a parameter and the four `MutableBuffer` natives that used the pair now name
theirs at the call.

`countStr` needed no new argument helper -- its whole argument layer is one
`string_method_argument`, which both receivers already shared -- so what is shared there is the
counter and its limit. **The limit is unbounded on purpose**: `countStrRexx` passes
`Numerics::MAX_WHOLENUMBER` (`classes/StringClassMisc.cpp:423`), which no count over a string can
reach.

The C++ shares these the same way it shares `POS`: `RexxString::wordPos`
(`classes/StringClassWord.cpp:256`) forwards to
`StringUtil::wordPos(getStringData(), getLength(), ...)` (`classes/support/StringUtil.cpp:1565`),
which is what `MutableBuffer::wordPos` calls too.

## 2. The witnesses

`corpus/lang/string_counts.rex` and `corpus/lang/string_counts_refusals.rex`, byte-identical to the
oracle on all three descriptors on both engines, rc 0 and rc 163. The refusals program sends two
shapes to a `MutableBuffer` receiver for the same reason Task 1a's did.

**Both `countStr` and `containsWord` answer a `String`**, asked directly with `~class~id` rather than
inferred from the rendering -- a count and a truth value render the same as an `Integer` would.

## 3. Six citations I had guessed, and all six were wrong

The first draft of these bodies carried `StringClassMisc.cpp:646`, `:661`,
`StringClassWord.cpp:280`, `:296`, `:312` and `:328`. The real sites are `:419`, `:434`, `:256`,
`:284`, `:270` and `:298`. Nothing about the code was wrong; the line numbers were written from the
shape of the file rather than from it, and every one was off. They were caught by looking them up
before the code was applied, which is the only reason this is a note rather than six wrong citations
in the tree.

## 4. The negative control

Predicted before running: stub `native_string_countstr` so it raises 93.903 and nothing else; then
`string_counts.rex` goes red on all three descriptors, `string_counts_refusals.rex` goes red on the
rows that answer or raise something else, the other programs do not move, and the strict count falls
from 367 to 365.

Measured: **365 of 367, and exactly those two programs.** `string_counts.rex` differed on stdout,
stderr and exit code; `string_counts_refusals.rex` on stdout alone.

The strict count moved 365 -> 367 when the two witnesses were filed, which is how this report knows
they run rather than only parse.

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
