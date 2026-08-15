# Task B — Comparison operators (Phase 2, Task 2.6)

Status: DONE. All twelve operators implemented in `rust/crates/rexx-num/src/compare.rs`
and wired into the differential harness; 46,270 generated cases against
`build/bin/rexx`, 0 divergences; a hand-run FUZZ probe reproduced and pinned
as unit tests, since NUMERIC FUZZ has no column in the `digits|a|op|b` case
format.

## What the interpreter does

Every value in this crate's harness is a plain parsed string (never a
`NumberString` produced by prior arithmetic), so the operators that actually
fire are `RexxString`'s (`StringClass.cpp`), not `NumberString`'s — the
latter only matters for values that already went through arithmetic, which
is out of scope for a `digits|a|op|b` case file. `RexxString::comp`
(`StringClass.cpp:753`) tries to convert *both* operands to `NumberString`
first; only if both succeed does it hand off to `NumberString::comp`
(`NumberStringClass.cpp:3194`) for a numeric comparison. If either operand
doesn't convert, it falls back to `stringComp` (`:795`) — not an error.

**Numeric path** (`=`, `<`, `>`, `<=`, `>=`, `\=`, `<>`, `><` — the last two
are just alternate spellings of `\=`; the interpreter's operator table
literally repeats the same method pointer for all three tokens). `comp()`
branches on sign:

1. Different non-zero signs: decided from the sign alone, no computation.
2. Both zero: equal (every spelling of zero collapses).
3. Same non-zero sign: either a direct digit-array compare (when both
   operands, aligned to a shared exponent, fit within `DIGITS - FUZZ`
   digits) or, failing that, an actual subtraction at that working
   precision, whose sign is the answer.

Path 3's "direct compare" branch is a pure optimisation, not a separate
rule: when the aligned digit width already fits the working precision, no
rounding could occur either way, so comparing the digits and subtracting
them are provably the same answer. I did not port it separately — `compare.rs`
always takes the subtraction route for same-sign operands, reusing the
already-verified `Number::sub`. Path 1 is *not* a mere optimisation, though,
and had to stay distinct: two enormous, opposite-signed, individually
in-range operands (each within `MAX_EXPONENT`) must never overflow when
compared, even though a same-sign subtraction of their magnitudes could
(`9.999999999e999999999` vs its negation, if actually subtracted, sums the
magnitudes and overflows). The interpreter never attempts that computation
for opposite signs, so `numeric_order` checks sign first and returns before
ever calling `sub`.

`FUZZ` never widens the `DIGITS` used to *truncate* the operands before
computing (that's still plain `DIGITS`, inside `sub`) — it only narrows the
precision at which the final subtraction's sign is trusted. This means two
values can already compare equal at `FUZZ 0` purely because both truncated
to the same digits at the (fuzz-independent) `DIGITS+1` working width; see
`truncation_to_digits_can_make_longer_operands_compare_equal_without_fuzz`
in `tests/compare.rs`, confirmed against the interpreter (`123456789 =
123456780` is true at `DIGITS 5`, false at `DIGITS 9`, entirely without
`FUZZ`).

**Strict path** (`==`, `\==`, `<<`, `>>`, `<<=`, `>>=`): `primitiveIsEqual`
and `primitiveStrictComp` (`StringClass.cpp:674`, `:920`) are both a
shorter-prefix-then-length compare with **no** blank stripping in either
direction. Rust's slice `Ord` implements exactly that already (shared
prefix decides; ties break on length), so `a.as_bytes().cmp(b.as_bytes())`
*is* the port — there was nothing to hand-write.

**Non-numeric fallback** (`stringComp`, `StringClass.cpp:795`): strips
*leading* blanks/tabs from both sides (not trailing), compares byte-for-byte
up to the shorter length, and — if that prefix matches but lengths differ —
treats the longer side's leftover bytes as needing to be all blank/tab to
still be equal (otherwise the first non-blank leftover byte is compared
against a literal space). This is why non-strict `=` treats `"1"` and
`"1  "` as equal but strict `==` does not.

## What I implemented

`rust/crates/rexx-num/src/compare.rs` (new):
- `CompareOp`, one variant per operator except that `\=`/`<>`/`><` share
  `NotEqual` (matching the interpreter's own operator table, which shares
  one method pointer for all three).
- `compare(a, b, digits, fuzz, op) -> Result<bool, ArithError>`: dispatches
  to the strict byte-slice comparison, or to `numeric_order`/`string_order`
  depending on whether both operands parse as `Number`.
- `numeric_order`: the sign-branch-then-subtract algorithm above.
- `string_order`: the `stringComp` port.

`rust/crates/rexx-num/src/lib.rs`: added `mod compare;` and
`pub use compare::{compare, CompareOp};` (module declaration/re-export only,
as scoped).

`rust/crates/rexx-num/src/bin/muldiv.rs`: added a `compare_op(&str) ->
Option<CompareOp>` token mapper and a branch in `main` that calls `compare`
for the twelve comparison tokens (printing `1`/`0`, or `<E{code}>` on the rare
error path) before falling through to the existing arithmetic dispatch.
FUZZ is hardwired to 0 there, since the case format has no column for it.

`rust/crates/rexx-num/tests/compare.rs` (new): the behaviours from the brief
(numeric `=` ignores form, strict `==` doesn't; `" 1" == "1"` false;
`0 = -0`; `1e2 = 100` true / `1e2 == 100` false; `"a" << "b"`), the
non-numeric fallback, the opposite-sign no-overflow case, the
DIGITS-without-FUZZ truncation-equality case, and the FUZZ table below.

## Verification

### Differential harness (`build/bin/rexx` vs `./target/release/muldiv`)

Generated with ad hoc Python scripts under
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/`
(`gen_compare_cases.py`, `gen_compare_extreme.py`, `gen_compare_truncation.py`).
All twelve operators, `DIGITS` in `{1,2,3,5,7,9,12,18,30}`, `FUZZ` fixed at 0
(the format's limit).

| set | cases | divergences |
|---|---|---|
| general (curated equal-forms/trailing-zeros/blanks/exponents/negative-zero/non-numeric + 6,800 random pairs) | 40,132 | **0** |
| exponent-extreme (same-magnitude opposite-sign at the `+/-999999999` edge; near-equal same-sign at the edge; top-vs-bottom-edge mixes) | 1,728 | **0** |
| truncation boundary (operand length vs DIGITS, digit flipped at every position, plain and decimal) | 4,410 | **0** |
| **total** | **46,270** | **0** |

No `<E..>` markers appeared on either side of any of the three sets — I
specifically checked, since the design argument in `numeric_order`'s doc
comment (same-sign subtraction can't overflow; opposite-sign is decided by
sign alone) predicts comparisons never error for in-range operands.

### NUMERIC FUZZ probe

Case format can't carry a FUZZ column, so I ran a small script directly
under `build/bin/rexx`:

```rexx
call test 9, 0
call test 9, 1
call test 9, 2
call test 9, 8
call test 5, 0
call test 5, 4
exit 0
test:
  parse arg dd, ff
  numeric digits dd
  numeric fuzz ff
  call pair "123456789", "123456780"
  call pair "123456789", "123456789"
  call pair "-123456789", "-123456780"
  call pair "123456780", "123456789"
  call pair "1000000000", "1000000009"
  call pair "100", "100.001"
  return
pair:
  parse arg a, b
  eq=(a=b); ne=(a\=b); lt=(a<b); gt=(a>b); le=(a<=b); ge=(a>=b)
  seq=(a==b); slt=(a<<b); sgt=(a>>b)
  say a "vs" b": =" eq "\=" ne "<" lt ">" gt "<=" le ">=" ge "==" seq "<<" slt ">>" sgt
  return
```

Observed (abbreviated to the interesting transitions):

- `DIGITS 9 FUZZ 0`: only the identical pair (`123456789`/`123456789`)
  compares equal.
- `DIGITS 9 FUZZ 1`: `1000000000 = 1000000009` becomes true (their
  difference is in the 10th digit, which `FUZZ 1` drops from the working
  precision); the 9-digit pairs and the decimal pair are still unequal.
- `DIGITS 9 FUZZ 2`: the 9-digit pairs also become equal; the decimal pair
  (`100` vs `100.001`) still isn't.
- `DIGITS 9 FUZZ 8`: everything collapses, including the decimal pair.
- `DIGITS 5 FUZZ 0`: every pair is already equal — `DIGITS` itself
  (independent of `FUZZ`) truncates all these operands to the point of
  exact equality before any fuzz logic runs. `FUZZ 4` at `DIGITS 5` changes
  nothing further.
- `==`, `<<`, `>>` are identical across every `FUZZ` setting tested — FUZZ
  never touches the strict operators.

Reproduced exactly by `numeric_fuzz_relaxes_the_numeric_operators_but_never_the_strict_ones`
in `tests/compare.rs`.

## Test / clippy output

- `cargo test --offline` (whole workspace): all green, no regressions.
  `rexx-num`'s own suite: `addsub` 7, `compare` **10 (new)**, `muldiv` 9,
  `parse` 7, `pow` 7, `settings` 6 — 46 integration tests, plus the
  crate's existing unit tests, all passing.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: clean,
  zero diagnostics.
- No `unsafe` introduced; `#![forbid(unsafe_code)]` untouched.

## Notes / things worth flagging

- I did not port the interpreter's digit-array fast path
  (`NumberStringClass.cpp:3246-3320`) as a separate code branch. I verified
  it is mathematically subsumed by the subtraction branch it's short-circuiting
  (same working precision, and the fast path only applies exactly when no
  rounding could occur), so reusing the already-oracle-verified `Number::sub`
  for all same-sign comparisons reproduces it without duplicating the
  memcmp-with-remainder-scan logic. This is a simplification of the
  *implementation*, not of the *behaviour* — the 46,270-case differential
  run (including a dedicated batch aimed at DIGITS-vs-operand-length
  boundaries) found nothing this treatment gets wrong.
- `.nil` comparisons (`TheNilObject` special-casing in the C++) are
  unreachable from this crate's harness: case-file operands are always
  plain strings assigned to Rexx variables, never the singleton NIL object,
  so I did not implement that branch. Flagging in case a future task feeds
  `compare` from a context where `.nil` is reachable.
- The brief's interface list doesn't mention a `compare` function signature,
  so I designed one; happy to adjust the shape (e.g. taking pre-parsed
  `Number`s instead of `&str`, or returning `Ordering` instead of taking an
  operator) if a consuming task wants something different — the string
  fallback requires the original text either way, so I don't think a
  `Number`-only entry point can replace this one.
