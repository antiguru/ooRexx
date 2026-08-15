# Phase 2 whole-branch review

Reviewer: independent agent. Scope: all 66 commits of the Phase 2 branch, the
Rust and doc diffs, `phase-2-gate.md`, and `progress.md`.

Every finding below is marked **VERIFIED** (I ran it against
`build/bin/rexx` and/or the Rust build, or read the cited C++) or
**PLAUSIBLE**. Nothing here is asserted from reading alone.

## Verdict

**Not fit to merge as Phase 2 until C1 is fixed.** The arithmetic core is in
excellent shape — I could not break it. What is wrong is the `NUMERIC
DIGITS`/`FUZZ` validation rule, which is not the interpreter's rule at all.

## What I re-verified independently (all clean)

| probe | cases | divergences |
|---|---|---|
| error-boundary sweep: `%`/`//` quotient-width, overflow/underflow edges for `+ - * / **`, literal error-41 edges, pow magnitude edges | 1,639 | **0** |
| `FORMAT`/`TRUNC` boundary sweep: `before`/`after`/`expp`/`expt` around every width, both forms | 23,904 | **0** |
| `NUMERIC FUZZ` differential (a gap in the eleven curated sets — no case file carries a fuzz column) | 135,000 | **0** |
| randomised arithmetic, three unused seeds | 60,000 | **0** |
| randomised `FORMAT`/`TRUNC` fuzz, carry-biased mantissas | 30,000 | **0** |

Error identity, which the harnesses cannot see (they compare only the major
number): I checked **every** `ArithError` and `FormatError` variant's
sub-code, `~message` text and `~additional` array against
`condition('o')` in the interpreter. All eleven match byte for byte,
including `42.001`'s double space, `42.901`'s literal `"9"` at DIGITS 9/15,
`93.941`'s `"1.000000000"` vs `"1"` mantissa distinction, and the empty-array
(not `.nil`) result for no-substitution messages. **No finding in this
category.**

C++ citations spot-checked and **all accurate**: `NumberStringClass.cpp:316`
(`checkOverflow`), `:2192`, `:3194` (`comp`), `Numerics.hpp:113`,
`NumberStringMath.cpp:315` (`mathRound` — the fixed-digit-count carry claim is
correct), `NumberStringMath2.cpp:106/224/331/355/811/1059`,
`StringClass.cpp:753/795/920/2391-2410` (the `notEqual` pointer really is
repeated three times). **No lying citation found.**

Rounding-point rule: I traced `adjustPrecision(resultPtr, digits)`
(`NumberStringMath.cpp:448`) — round, then strip leading zeros, then
`checkOverflow` — against every Rust call site. `addsub`, `mul`, `div` and
`pow` all do round → `assemble` → `check_range` in that order. The one
structural difference is that `mathRound` runs its own `checkOverflow` before
the strip, on the unstripped digit count. I worked through whether that is
reachable: it needs a raw result whose top digit position is 1000000000 with a
surviving leading zero, and neither addition (which emits a carry digit only
when there is a carry) nor subtraction (whose top position equals the larger
operand's, already ≤ MAX_EXPONENT) can produce one. **PLAUSIBLE non-issue,
recorded so the next auditor does not redo it.**

`unsafe`: `forbid` at `[workspace.lints.rust]`, and — the escape that
matters — **all six** crates carry `[lints] workspace = true`. No `unsafe`
token anywhere in `rust/`. VERIFIED.

---

## C1 — CRITICAL: `NUMERIC DIGITS`/`FUZZ` use a fixed cap; the interpreter uses the *current* DIGITS

`rust/crates/rexx-num/src/settings.rs:140`, `:143`, `:162`

The C++ rule is
`result->requestUnsignedNumber(setting, number_digits())`
(`interpreter/instructions/NumericInstruction.cpp:103` for DIGITS, `:139` for
FUZZ): the new value must be convertible as an unsigned whole number **within
the DIGITS setting currently in force**, and ≤ `Numerics::MAX_WHOLENUMBER`
(`999999999999999999` on 64-bit, `Numerics.hpp:86`).

`set_digits_str` instead applies a fixed `value > MAX_EXPONENT` (999999999)
cap and never consults `self.digits`. `set_fuzz_str` applies only
`u32::try_from`. Six divergences, **VERIFIED** by running the same inputs
through `build/bin/rexx` and through `Settings`:

| starting DIGITS | statement | interpreter | `rexx-num` |
|---|---|---|---|
| 3 | `numeric digits 1000` | **error 26.005** | Ok, digits = 1000 |
| 3 | `numeric digits 12345` | **error 26.005** | Ok, digits = 12345 |
| 1 | `numeric digits 10` | **error 26.005** | Ok, digits = 10 |
| 2 | `numeric digits 100` | **error 26.005** | Ok, digits = 100 |
| 10 | `numeric digits 1000000000` | **Ok, digits() = 1000000000** | error 26.005 |
| 18 | `numeric digits 999999999999999999` | **Ok** | error 26.005 |
| 3 | `numeric fuzz 12345` | **error 26.006** (`FuzzNotWhole`) | error **33.001** (`FuzzNotBelowDigits`) |

The last row is a wrong *error number*, which this project treats as
contract. The FUZZ case fails in the interpreter at the conversion step,
before the fuzz-vs-digits comparison is ever reached; `rexx-num` converts
successfully and then reaches the comparison.

Two further consequences:

- `Settings::digits` is `u32`. `numeric digits 4294967296` from DIGITS 10 is
  legal (VERIFIED: `digits()` returns it), and `u32` cannot hold it. This is
  a type-width problem, not only a check.
- Commit `b9faa2d0` ("Cap NUMERIC DIGITS, and stop two casts wrapping above
  it") introduced this cap *in order to* make two `i32` casts safe. That is a
  correctness cost paid for a soundness shortcut — the brief asked for exactly
  this shape.

Why no test caught it: `tests/settings.rs:25`
(`digits_is_capped_at_max_exponent`) checks `999999999` Ok / `1000000000`,
`2147483647`, `4294967296` rejected — all from a *default* `Settings`, i.e.
DIGITS 9. At DIGITS 9 the fixed cap and the real rule agree on all four
values by coincidence (999999999 is nine digits; the other three are ten).
The corpus program `rust/corpus/num/settings.rex` probes no value above
`1e3`. The eleven curated differential sets contain **no settings cases at
all** — the case-file formats (`digits|a|op|b`, `digits|func|number|args`)
have no column that could express one.

Fix shape: replace the constant with "at most `self.digits` significant
digits, and ≤ 999_999_999_999_999_999", widen the field to `u64`, and apply
the same conversion rule to FUZZ *before* the fuzz-vs-digits comparison.

## I1 — IMPORTANT: a third `u32` → `i32` narrowing of the class already fixed twice

`rust/crates/rexx-num/src/muldiv.rs:177`

```rust
if !int_digits.is_zero() && int_digits.digits.len() as i32 + int_digits.exponent > digits as i32
```

`digits as i32` wraps negative once `digits >= 2^31`, so the not-a-whole-number
test succeeds for every non-zero quotient. **VERIFIED** against the built
crate:

```
digits=2147483647  123456 % 2  -> Ok("61728")
digits=2147483648  123456 % 2  -> Err(IntegerDivideNotWhole)
digits=4294967295  123456 // 2 -> Err(RemainderNotWhole)
```

This is the same defect the branch already fixed at `lib.rs:548`
(`let digits = digits as i64;`) and `pow.rs:44` (`digits as i64`), each with a
comment stating the reason: *"`digits` is a bare `u32` here (not bounded by
`Settings`, which is the caller most external code goes through)"*. `div` is
equally public and equally unbounded; the sweep that produced those two fixes
missed this one. `Settings` currently gates it — but C1 says the gate is
itself wrong, and the correct gate (up to 999999999999999999) does **not**
keep `digits` inside `i32`.

## I2 — IMPORTANT: `set_form_str` uppercases; `NUMERIC FORM VALUE` is case-sensitive. The comment claiming otherwise is false.

`rust/crates/rexx-num/src/settings.rs:175-182`

```rust
// The interpreter accepts any unambiguous case; these are the only
// two spellings it takes.
self.form = match text.to_ascii_uppercase().as_str() {
```

**VERIFIED** against `build/bin/rexx`:

```
numeric form value 'ENGINEERING'  -> ENGINEERING
numeric form value 'engineering'  -> rc=25, 25.011, found "engineering"
numeric form value 'Engineering'  -> rc=25, 25.011, found "Engineering"
numeric form value 'ENG'          -> rc=25   (no abbreviation either)
numeric form value ' ENGINEERING' -> rc=25   (no trimming either)
```

Only the *keyword* form (`numeric form engineering` written in source) is
case-insensitive, and only because the tokenizer has already uppercased the
token — VERIFIED separately. `set_form_str` takes a runtime `&str`, so it
models the VALUE path, where `to_ascii_uppercase()` accepts three spellings
the interpreter rejects. The comment asserts a mechanism the interpreter does
not have, and "unambiguous" further implies abbreviations are taken, which
they are not.

`rust/corpus/num/settings.rex` tests only `'ENGINEERING'` and `'BOGUS'`.

## Minor findings

**M1 — `muldiv.rs:200-201`, `usize` → `u32` truncation.**
`(left.digits.len() + right.digits.len() + int_digits.digits.len() + digits as usize + 10) as u32`
wraps for `digits` near `u32::MAX`, giving the remainder path a working
precision far below what it needs. Same class as I1, same gating, unreached in
practice today. PLAUSIBLE (arithmetic reasoning; I did not run it, because the
`want = digits + 1` long division ahead of it would not terminate at that
size).

**M2 — `lib.rs:478-483`, a doc comment that cites documentation that does not
exist.** `format.rs:447` says `Number::round_to`'s *"`digits == 0` is a
documented no-op sentinel"*. `round_to`'s own doc comment says only "Rounds to
at most `digits` significant digits, half-up" and never mentions the sentinel;
the behaviour lives in the unannotated `if keep == 0` at `lib.rs:485`. VERIFIED
by reading. The sentinel is load-bearing (it is why `round_to_places` exists as
a separate function) and is reachable from a public entry point:
`compare(a, b, d, d, op)` yields `working_digits == 0` at `compare.rs:144` and
silently skips rounding. `Settings` forbids `fuzz == digits`, so this is
gated the same way I1 is.

**M3 — `tests/settings.rs:25`, a test that pins a mechanism the interpreter
does not have.** `digits_is_capped_at_max_exponent` passes only because a
fixed 999999999 cap and the real "fits the current DIGITS" rule agree at
DIGITS 9. Renaming is not enough; it needs cases started from a non-default
DIGITS. VERIFIED (see C1).

**M4 — error-type API is three designs, not one.** `ArithError::code(self)`
and `FormatError::code(self)` take `self` by value; `SettingsError::code(&self)`
by reference. `ArithError::sub_code(&self)` and `SettingsError::sub_code(&self)`
return `(u16, u16)`; `FormatError::sub(&self)` is differently named and returns
a bare `u16`. All three types then implement an identical
`code`/`additional`/`message` protocol with no shared trait, so the Phase 3
dispatch layer must special-case each. VERIFIED by reading; no behavioural
consequence today.

**M5 — `compare` is the crate's only `&str` entry point.** `compare.rs:85`
takes two `&str` and re-parses; every other public operation takes `&Number`.
It models `RexxString::comp` only. The `NumberString::comp` path — where the
left operand is already numeric and the fallback is
`stringValue()->stringComp(...)`, i.e. the *canonical* rendering rather than
the source text — has no entry point. The module doc admits the restriction;
worth naming as a Phase 3 trap because `1+0 = "abc"` and `"1" = "abc"` compare
different left-hand strings. VERIFIED by reading `StringClass.cpp:753-775` and
`NumberStringClass.cpp:3200-3208`.

**M6 — two rendering entry points, one form-blind.** `Number::format` and
`Display` (`lib.rs:534`, `:572`) produce SCIENTIFIC unconditionally;
`format_form` (`format.rs:99`) honours `Form`. `full_precision` (`lib.rs:207`),
used for `ArithError`'s operand echoes, goes through the form-blind one. That
happens to be right (the interpreter substitutes the operand's *source
spelling*, which no form applies to), but nothing in the code says so, and any
future caller reaching for `Display` under `FORM ENGINEERING` gets the wrong
text. VERIFIED by reading.

**M7 — `Number`'s "`digits` never empty" invariant is enforced by convention.**
The field doc (`lib.rs:301`) says "Never empty", but the field is `pub(crate)`
and `addsub.rs`, `muldiv.rs`, `pow.rs` and `format.rs` all build `Number { .. }`
literals directly, bypassing `assemble`. `divide_power`'s
`if left.len() == 1 && left[0] == 0 { break; }` (`pow.rs:228`) can in principle
leave `result` empty. I traced it: reaching that break with an empty `result`
requires a subtraction to have happened, which implies `this_digit > 0`, which
implies the digit was already pushed — so it is unreachable, and even if
reached, `assemble` collapses it to canonical zero. PLAUSIBLE non-issue;
recorded because the invariant is real and unguarded.

**M8 — `phase-2-gate.md:64-68`, an internally inconsistent count.** "2,528 of
them match a plain `<operand> <operator> <operand>` form. Restricted to the
arithmetic groups proper — ADDITION 304, … EXPONENT 123 — that is **2,553
assertions**, which would drop straight into the existing `digits|a|op|b`
harness." A restriction cannot be larger than what it restricts. VERIFIED: the
six per-group counts are exact (304 + 406 + 1050 + 373 + 297 + 123 = 2,553) but
2,553 is the *total* `assertSame` count in those groups, not the plain-form
subset. So the number that "would drop straight into the harness" is smaller
than 2,553 and is not stated.

## Assessment of `phase-2-gate.md`

Accurate on everything I could check, and unusually so:

- All eleven case-set counts regenerated from the committed generator and
  counted: 8712, 8112, 17424, 20184, 2112, 32368, 1800, 6720, 12136, 640,
  15840 = **126,048 exactly**. VERIFIED.
- Twelve `corpus/num/` and thirteen `corpus/lang/` programs. VERIFIED.
- No `unsafe`, `forbid` at the workspace root, all six crates opting in, no
  `cargo` invocation in any of the three `.github/workflows/` files. VERIFIED.
- The `fmt`/`fmt2` reconstruction caveat is stated in the generator docstring
  as claimed. VERIFIED.
- The per-group ooTest counts are exact. VERIFIED (M8 is the phrasing around
  them, not the numbers).

Two things to change before merge:

1. **M8**, above.
2. **A material omission.** The "blind spot worth naming" section names error
   boundaries — correctly, and my 1,639-case boundary sweep found nothing, so
   `fmtcarry` appears to have closed it. What it does not name is that
   `Settings` has **no differential coverage of any kind**: the eleven sets
   carry no fuzz column and no settings column, `tests/settings.rs` is
   hand-written unit tests never compared against the oracle, and
   `settings.rex` is a corpus program that cannot run. C1 and I2 both live in
   exactly that hole, and the 135,000-case FUZZ run I had to write myself to
   cover comparison is the same gap seen from the other side. The gate should
   say so, because "126,048 cases, 0 divergences" reads as covering the phase
   and it does not cover `NUMERIC`.

Everything else in the document — including the "cannot assess" reasoning for
criteria 1–3 and the refusal to soften the performance miss — is honest and I
would merge it as written once those two are fixed.

## Categories with nothing to report

- **Error identity** (numbers, sub-numbers, `~message`, `~additional`): all
  eleven variants verified against the interpreter; nothing wrong.
- **C++ citations in comments**: fourteen checked, all accurate.
- **Rounding-point preservation**: every restructured routine checked against
  `adjustPrecision`'s round → strip → check order; all preserve it.
- **The `substitute` re-scan class**: the single-pass implementation is
  correct, and its greedy digit consumption is safe — the generated table
  contains only `&1`–`&4` (VERIFIED by scanning the generated `errors.rs`).
- **`unsafe` escapes**: none.

---

# Scoped re-review: commit e91a333d (Task H)

Verdict: **the rule is now right.** Two things to fix before merge (H2 is a
one-word change; H3 needs a decision, not necessarily code). H1 is a separate
pre-existing defect this review surfaced, not part of this diff.

## 1. Is the ported conversion faithful?

Read against the C++: `unsignedNumberValue` (`NumberStringClass.cpp:632`),
`checkIntegerDigits` (`:937`), `createUnsignedValue` (`:788`),
`maxValueForDigits` (`Numerics.hpp:160`) and the `validMaxWhole` table
(`Numerics.cpp:59-77` — `validMaxWhole[18]` really is `999999999999999999`,
so `max_value_for_digits`'s `digits >= 18` arm is exact).

Two structural differences, both traced to non-issues:

- **PLAUSIBLE non-issue.** C++ step 5 tests `-numberExp >= digitsCount` — the
  *original* digit count — where `settings.rs:202` tests
  `decimals >= length`, the count *after* the DIGITS truncation. They differ
  only when `length <= decimals < digitsCount`. That state is unreachable:
  with `carry == false`, `checkIntegerDigits` requires all `length` kept
  digits to be zero and the leading digit is never zero (a nonzero `Number`
  has no leading zero), so it has already returned false; with
  `carry == true` it explicitly rejects `decimalPos > length`. The one
  surviving case, `decimals == length && carry`, gives 1 on both sides.
- **PLAUSIBLE non-issue.** `createUnsignedValue`'s early-out
  `exponent + intlength > ARGUMENT_DIGITS` (18) has no Rust counterpart;
  `create_unsigned_value` relies on `checked_mul` and the `max_value` test.
  Equivalent, because the slice passed in never has a leading zero, so
  `exponent + intlength > 18` implies a value `>= 10^18 > max_value`.

**Empirically: 39,916 settings differential cases, one diverging value —
and it is not a settings defect.** Two batches
(`14,592` and `25,324`), comparing outcome, resulting `digits()`/`fuzz()`,
sub-code *and* substitution values, across 24 starting DIGITS/FUZZ pairs
(1–20, 30, and fuzz at digits−1). Coverage aimed at the third dimension the
brief asked for: negative zero in five spellings; every exponent spelling
(`1e2`, `1E2`, `+1e2`, `1e+2`, `0.001e5`, `1000e-2`, `10e-1`, `.5`, `5.`,
`1.e2`); leading/trailing blanks and tabs; junk (`1.2.3`, `1e`, `1e+`,
`0x10`, `+`, `-`, `--1`); all-nines carry shapes at every width 1..20
(`999…9.5`, `.45`, `.55`, `.95`, `0.999…95`, `999…9.4999…`, `999…9.5000…1`);
the 10^18 ceiling from every side; and FUZZ set after DIGITS had already been
changed in the same run. **Zero divergences except the one below.**

### H1 — Important, VERIFIED, PRE-EXISTING (not introduced by this diff)

`rust/crates/rexx-num/src/lib.rs:345-355`

`Number::parse` reads the sign and then expects a digit immediately. The
interpreter skips blanks *and tabs* between the sign and the number —
`NumberStringClass.cpp:1289-1295`, comment and all:

```c
    if (ch == RexxString::ch_MINUS || ch == RexxString::ch_PLUS)
    {
        inPtr++;
        // spaces are allowed after a sign, so skip them too
        while (*inPtr == RexxString::ch_SPACE || *inPtr == RexxString::ch_TAB)
```

VERIFIED against `build/bin/rexx`:

```
["+ 3"] + 0 = 3            Number::parse("+ 3")       -> None
["- 3"] + 0 = -3           Number::parse("- 3")       -> None
["  +   3  "] + 0 = 3      Number::parse("  +   3  ") -> None
["-  0.5"] + 0 = -0.5      Number::parse("-  0.5")    -> None
["+ .5"] + 0 = 0.5         Number::parse("+ .5")      -> None
["+ 3.5e2"] + 0 = 350      Number::parse("+ 3.5e2")   -> None
["-<TAB>3"] + 0 = -3       Number::parse("-\t3")      -> None
```

So the crate raises error 41 where the interpreter computes a value. The rule
is narrow and I pinned its edges: blanks are legal only at the two ends and
directly after a single sign — `"+ 3 e2"`, `"3 e2"`, `"3e 2"`, `"3 4"`,
`"+ + 3"` and `"1 ."` are error 41 on both sides.

This survived all 126,048 differential cases because no generator emits an
embedded blank after a sign: `gen-cases.rs`'s `literal()` writes the sign
straight onto the digits, and `CMP_VALS`'s blank cases (`" 1"`, `"1 "`) are
end-blanks only. `corpus/lang/whitespace_significant.rex` does not cover it
either. It surfaced here only because a settings case file happened to carry
`"+ 3"` as a candidate value.

## 2. Did the `u32` -> `u64` widening get carried everywhere?

Audited every cast site in the crate after the change, not just the ones the
diff touched.

### H2 — Important, VERIFIED, INTRODUCED BY THIS DIFF

`rust/crates/rexx-num/src/muldiv.rs:296`

```rust
            if i > n.len() + want + d.len() {
```

`want` is now `crate::working_length(digits)`, which *saturates* to
`usize::MAX`. The sum then overflows `usize`. Debug build, overflow checks on:

```
Number::div(&n("123456"), &n("2"), u64::MAX, DivOp::IntegerDivide)
  thread 'main' panicked at rust/crates/rexx-num/src/muldiv.rs:296:20:
  attempt to add with overflow
```

`1 / 8` at the same `digits` panics identically. This is precisely the class
the diff's own comments say it is defending against, in five separate places
("must stay huge rather than wrap", "must compare large, not wrap") — the
saturation was added at the producer and this one consumer still adds to it.

In release it wraps rather than panics, and I tried to turn that into a wrong
answer. I could not: the wrapped bound is `n.len() + d.len() - 1`, and the
guard is only evaluated while no quotient digit has been produced, which is at
most iteration `n.len() + d.len() - 1` — it survives by exactly one. So I am
reporting the debug panic as the defect and explicitly *not* claiming a
release miscomputation. `saturating_add` on both terms fixes it.

Not reachable from a `Settings`-legal DIGITS: `working_length(10^18)` is
`10^18 + 1`, and the sum stays well inside `usize`. It needs the bare-`u64`
public entry point.

### H3 — Important, VERIFIED, ENABLED BY THIS DIFF

`rust/crates/rexx-num/src/muldiv.rs:213-300` (`long_divide`), same shape in
`pow.rs:164-245` (`divide_power`)

The old fixed cap held DIGITS at 999999999. It is now 999999999999999999 —
correctly, that is the interpreter's ceiling — and nothing in `long_divide`
bounds its work by anything but `want`. An inexact quotient therefore runs
forever:

```
Number::div(&n("1"), &n("7"), 999_999_999_999_999_999, DivOp::Divide)   -> no return
n("123456").div(&n("7"), 999_999_999_999_999_999, DivOp::Remainder)     -> no return
```

Both killed at every timeout I gave them; `long_divide` is being asked for
10^18 + 1 quotient digits. The interpreter, at the *same legal setting*,
terminates:

```
numeric digits 999999999999999999 ;  r = 1 / 7   ->  rc=5  [The NIL object]
numeric digits 1000000000         ;  r = 1 / 7   ->  rc=42 [Arithmetic underflow;
                                                     exponent ("-1000000000")
                                                     exceeds 9 digits.]
```

The second is the interesting one: at DIGITS 10^9 the quotient's exponent
falls below `MIN_EXPONENT`, so the interpreter reports 42.902. `rexx-num`
reaches the same conclusion — but only after generating a billion digits,
because the range check happens after `long_divide` returns. The interpreter
allocates its accumulator up front and fails fast; this port discovers the
same fact one digit at a time.

This is not a defect the fix introduced so much as one it *legitimised*: at
the old cap the same division cost ~1 GB and minutes, which is bad but
bounded. It is worth a decision rather than necessarily code — either bound
the generated quotient by the point at which `q_exp` must underflow, or record
it as accepted debt with the interpreter's error 5 as the target behaviour.

### The rest of the audit — clean

Verified correct, each by reading and by the extreme-`digits` probe below:
`working_length` (`lib.rs:221`); `addsub.rs`'s `digits_usize` and the two
`saturating_add` fast-path sums, and `max_len` via `i64::try_from`;
`mul`'s `digits_usize`; `div`'s `digits_i64` and the `exact` sum now carried
in `u64`; `pow`'s `work` and `divide_power`'s result cap; `round_to`'s
saturated `keep`; `format`'s saturated `digits` and `low_threshold`.
`settings.rs:130`'s `digits as u32` is guarded by the `digits >= 18` arm
above it, and `:177`'s `digits as usize` by `length as u64 > digits`.
`mul`'s `let keep = digits_usize + 1` cannot overflow because its guard
requires `product.len() > digits_usize`, impossible at saturation.

**The reported new site is real and correctly fixed.** With `digits` large
enough to saturate, `expt` is `i64::MAX` and the old `2 * expt` overflows —
debug panic, and in release a negative threshold that makes
`|exponent| > 2*expt` true for *every* value with a negative adjusted
exponent, i.e. spurious exponential form. All three trigger sites
(`format.rs:186`, `:329`, `:443`) now use `saturating_mul(2)`. Confirmed by
running `format(0.001)`, `format(1e-30)`, `format_form(1e-30, Engineering)`
and `format_with(1e-30,,,,expp=2)` at `digits` = 40, 10^6, 10^18-1,
u64::MAX/2, u64::MAX-1 and u64::MAX under a **debug** build: no panic, and
identical output at every `digits`.

## 3. Are I1 and I2 genuinely resolved?

**I1 — yes, VERIFIED.** `muldiv.rs:189` now compares in `i64` with `digits`
saturated (`i64::try_from(digits).unwrap_or(i64::MAX)`) and the exponent
widened with `i64::from`, not `as`:

```
digits=9                     123456 % 2 -> Ok("61728")
digits=2147483647            123456 % 2 -> Ok("61728")
digits=2147483648            123456 % 2 -> Ok("61728")   (was Err(IntegerDivideNotWhole))
digits=4294967295            123456 % 2 -> Ok("61728")   (was Err)
digits=999999999999999999    123456 % 2 -> Ok("61728")
```

The companion Minor M1 (`(… + digits as usize + 10) as u32` in the remainder
path) is closed too — it is now `digits.saturating_add(exact_extra)` in `u64`.

**I2 — yes, VERIFIED.** `set_form_str` is an exact match on the two uppercase
spellings; `"engineering"`, `"Engineering"`, `"ENG"`, `" ENGINEERING"` and
`"ENGINEERING "` all return error 25. The false comment is gone, and its
replacement states the keyword-versus-VALUE distinction — which I re-checked
against the interpreter: `numeric form engineering` written in source works
(the tokenizer uppercases it), `numeric form value x` with `x = "engineering"`
is 25.011.

**M2 also closed in passing.** `round_to`'s `digits == 0` no-op sentinel is now
documented at `lib.rs:497-503`, naming `compare(a, b, d, d, op)` as the
reachable caller, and `format.rs:460`'s citation now points at a doc comment
that exists.

## 4. What the differential cases cannot see

Everything re-run against the post-widening build:

| probe | cases | divergences |
|---|---|---|
| error-boundary arithmetic sweep | 1,639 | **0** |
| `FORMAT`/`TRUNC` boundary sweep | 23,904 | **0** |
| randomised `FORMAT`/`TRUNC` | 30,000 | **0** |
| randomised arithmetic, two more unused seeds | 30,000 | **0** |
| `NUMERIC FUZZ` comparison, DIGITS to 20 | 95,220 | **0** |
| settings differential (new) | 39,916 | **1 value** (H1) |

H2 and H3 are the two things none of that could reach, and neither was found
by sampling values — both came from attacking the widened parameter itself.
That is the same lesson the gate document already records; it now has a second
instance.

## Categories clean in this diff

- **No rounding point moved.** The conversion is a new code path with no
  rounding of its own beyond the DIGITS-boundary round it is porting, and no
  existing operator's round → `assemble` → `check_range` order changed.
- **No new lying comment.** Every comment added by this diff that asserts a
  C++ mechanism was checked against the cited source: the
  `requestUnsignedNumber` rule, `MAX_WHOLENUMBER` at `Numerics.hpp:86`,
  `maxValueForDigits` at `:160`, the three ported function citations, and the
  keyword-versus-VALUE claim in `set_form_str`. All accurate.
- **No new `unsafe`**, and the widening introduced no new `allow`.
