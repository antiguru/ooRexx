# Suspected upstream defect: `NumberString::numberValue` drops the sign when the whole mantissa rounds away

**Status: write-up only. NOT filed, and not a reproducible bug report.**
Filing against ooRexx is Moritz's call. I could not construct a case that observes
this from a Rexx program, and the section "Why I could not observe it" says why
that is a property of the code rather than a gap in my effort.

**Found:** while porting `requestNumber` for Phase 3 Task 3.7 of the Rust rewrite.
**Build:** Open Object Rexx 5.3.0 r0, build date Jul 27 2026, 64-bit, from this
repository at `f98c2e29` (2026-06-19).
**Severity:** low. Latent, not live.
**Confidence:** high that the asymmetry exists as described; the reasoning is from
the source and the four call sites are enumerated below. Zero direct observation of
a wrong value, by construction.

## What the code does

`interpreter/classes/NumberStringClass.cpp` has four conversions from a
`NumberString` to a machine integer. Each one, after `checkIntegerDigits` has
rounded the value to `numDigits` significant digits, has a branch for the case
where the decimal point ends up left of every remaining digit. At that point the
value is either 0 or 1 depending only on whether the rounding carried, so each
function returns exactly that:

| line | function | signed? | rejects negatives? | applies `numberSign` here? |
| --- | --- | --- | --- | --- |
| 595 | `numberValue(wholenumber_t&, wholenumber_t)` | yes | no | **no** |
| 679 | `unsignedNumberValue(size_t&, wholenumber_t)` | no | yes, line 650 | no, correctly |
| 1083 | `int64Value(int64_t*, wholenumber_t)` | yes | no | **no** |
| 1181 | `unsignedInt64Value(uint64_t*, wholenumber_t)` | no | yes, line 1150 | no, correctly |

All four read:

```cpp
    // if because of this adjustment, the decimal point lies to the left
    // of our first digit, then this value truncates to 0 (or 1, if a carry condition
    // resulted).
    if (-numberExp >= length)
    {
        // since we know a) this number is all decimals, and b) the
        // remaining decimals are either all 0 or all 9s with a carry,
        // this result is either 0 or 1.
        result = carry ? 1 : 0;
        return true;
    }
```

## Why I read this as a defect rather than as intent

Three things, in the order they persuade me:

1. **Every other return path in the same two signed functions applies the sign.**
   In `numberValue` the fast path at line 575 and the final return at line 618 are
   both `result = ((wholenumber_t)intnum) * numberSign;`. In `int64Value` the fast
   path and the final return both multiply by `numberSign` as well. Only the
   carry-only branch does not.
2. **The two unsigned siblings omit the sign correctly, and for a stated reason.**
   `unsignedNumberValue` and `unsignedInt64Value` both reject a negative outright
   before reaching the branch, with the comment *"we can't convert negative values
   into an unsigned one"*. So the same expression is right in two functions and
   wrong in two, which is the signature of a line copied between them rather than
   of a decision taken four times.
3. **No comment marks it deliberate.** The comment above the branch explains why
   the result is 0 or 1. It says nothing about the sign, where the file is
   otherwise careful to explain each numeric edge case it handles on purpose --
   the `INT64_MAX + 1` edge case twenty lines below carries three lines of
   explanation.

So the reading is: `-0.9999999999` converted under `NUMERIC DIGITS 9` should be
`-1` and is `+1`, and the same for any negative value whose entire mantissa rounds
away to a carry.

## Why it is latent rather than live

The branch needs all three of:

* the value's magnitude to be below 1, so that the decimal point lands left of
  every digit after rounding;
* more significant digits than the precision, so that rounding happens at all;
* the first dropped digit to be 5 or more, so that the rounding carries, and every
  kept digit to be a 9, so that the carry propagates all the way out.

That is a narrow shape. `-0.9999999999` under `NUMERIC DIGITS 9` is the smallest
example. Reaching it also requires a caller that asks for a signed machine integer
from an arbitrary user-supplied number, which is a much smaller set than the
callers of the unsigned conversions.

## Why I could not observe it

**This is the part that stops it being a bug report.**

`requestNumber` forwards to `numberValue`, and the caller Phase 3 traced into this
branch is the `TRACE` instruction's skip count (`traceNew`,
`interpreter/parser/InstructionParser.cpp`). Parse time converts the value happily
-- `trace "-0.9999999999"` is rc 0 from `rexxc` -- but run time then rejects every
numeric `TRACE` outside interactive debugging:

```
$ build/bin/rexx p.rex     # p.rex is:  trace "-0.9999999999"  /  nop
Error 24 running .../p.rex line 1:  Invalid TRACE request.
Error 24.901:  Numeric TRACE requests are valid only from interactive debugging.
```

`trace -1` and `trace 1` reach the same 24.901, so the rejection is of numeric
`TRACE` as a category and not of this value. Whatever integer the parse produced is
discarded before anything can print it or branch on it. So the sign cannot surface
through this path at all, for any input.

I did not audit the other callers of `requestNumber` and `int64Value` for one that
both reaches the branch and exposes its result. That is the work someone filing
this would need to do, and it is the difference between this document and a
reproducible report.

## What the Rust rewrite did with it

Reproduced as written, not corrected, because the interpreter defines the
behaviour for this project and a silent divergence would be worse than a faithful
oddity. `rexx-num`'s `Number::whole_value` carries the branch as
`return Some(i64::from(carry));` with a comment naming this document's reasoning,
and `tests/whole.rs::the_carry_only_path_drops_the_sign_as_the_cpp_does` pins both
signs so the asymmetry cannot be tidied away by accident:

```rust
assert_eq!(whole("0.9999999999", TRACE_DIGITS), Some(1));
assert_eq!(whole("-0.9999999999", TRACE_DIGITS), Some(1));
```

If upstream fixes this, that test is the one to change, and it says so.

## If it is filed

The minimal patch is two lines, `* numberSign` added at
`NumberStringClass.cpp:595` and `:1083`. The two unsigned sites must be left
alone. A fix would want a test through a caller that exposes the value, which is
the same audit the previous section leaves undone.
