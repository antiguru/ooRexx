# Task H — Fix the NUMERIC validation rule and three narrowing casts

Findings from the Phase 2 whole-branch review. Full text in
`branch-review.md`. C1 is a defect I introduced through a wrong brief, so
treat the *rule* below as the specification and re-verify it yourself rather
than trusting either that brief or this one.

## C1 (Critical) — the DIGITS/FUZZ rule is not a fixed cap

`crates/rexx-num/src/settings.rs`.

The C++ is `result->requestUnsignedNumber(setting, number_digits())` —
`interpreter/instructions/NumericInstruction.cpp:103` for DIGITS, `:139` for
FUZZ. The new value must be convertible as an unsigned whole number **within
the DIGITS setting currently in force**, and must not exceed
`Numerics::MAX_WHOLENUMBER`, which is `999999999999999999` on 64-bit
(`Numerics.hpp:86`).

`set_digits_str` instead applies a fixed `value > MAX_EXPONENT` (999999999)
test and never looks at `self.digits`. `set_fuzz_str` applies only
`u32::try_from`.

Verified against `build/bin/rexx`:

| starting DIGITS | statement | interpreter | current code |
|---|---|---|---|
| 3 | `numeric digits 1000` | error 26.005 | Ok |
| 3 | `numeric digits 12345` | error 26.005 | Ok |
| 1 | `numeric digits 10` | error 26.005 | Ok |
| 9 | `numeric digits 1000000000` | error 26.005 | error 26.005 |
| 10 | `numeric digits 1000000000` | **Ok**, `digits()` = 1000000000 | error |
| 18 | `numeric digits 999999999999999999` | **Ok** | error |
| 3 | `numeric fuzz 12345` | error **26.006** | error **33.001** |

The FUZZ row is a wrong error *number*, which this project treats as
contract. In the interpreter the conversion fails before the fuzz-vs-digits
comparison is ever reached. The current code converts successfully and then
reaches the comparison, so it reports the wrong thing for the wrong reason.

**Why the existing test did not catch it**: `tests/settings.rs:25` starts from
a default `Settings`, i.e. DIGITS 9. At DIGITS 9 a fixed 999999999 cap and the
real rule agree on every value it tries, because 999999999 is nine digits and
its three rejected values are ten. The agreement is a coincidence of the
starting setting. My original probe made exactly this mistake.

**Type width.** `Settings::digits` is `u32`. `numeric digits 4294967296` from
DIGITS 10 is legal and `u32` cannot hold it, so this is a width problem as
well as a check problem. Widen the stored settings to `u64`.

That widening ripples: every operation currently takes `digits: u32`. Carry it
through rather than truncating at a boundary — a truncation would reintroduce
exactly the class of defect I1 and the branch's two earlier fixes are about.
The differential sets are your safety net for the ripple; they must all stay
at 0.

## I1 (Important) — a third narrowing cast of a class already fixed twice

`crates/rexx-num/src/muldiv.rs:177`:

```rust
if !int_digits.is_zero() && int_digits.digits.len() as i32 + int_digits.exponent > digits as i32
```

`digits as i32` wraps negative at or above 2^31, so the
not-a-whole-number test succeeds for every non-zero quotient. Verified:

```
digits=2147483647  123456 % 2   -> Ok("61728")
digits=2147483648  123456 % 2   -> Err(IntegerDivideNotWhole)
digits=4294967295  123456 // 2  -> Err(RemainderNotWhole)
```

The branch already fixed this exact shape at `lib.rs` (`Number::format`) and
`pow.rs` (`as_whole`), each with a comment explaining that `digits` is a bare
unbounded `u32`. `div` is equally public and was missed. Note that after C1 the
legal range reaches 999999999999999999, so `Settings` no longer keeps `digits`
inside `i32` even in principle.

Also **M1**, same class, same file: `muldiv.rs:200-201` truncates a computed
working precision with `as u32`.

**Sweep the whole crate for this class again and report what you find, including
"nothing else".** Two previous sweeps each missed a site; do not truncate your
search output and assume it is complete — that is how the earlier sweep missed
this one.

## I2 (Important) — FORM VALUE is case-sensitive; the comment says otherwise

`crates/rexx-num/src/settings.rs:175-182` uppercases the input and comments
that "the interpreter accepts any unambiguous case". Verified against
`build/bin/rexx`:

```
numeric form value 'ENGINEERING'   -> ENGINEERING
numeric form value 'engineering'   -> rc=25, found "engineering"
numeric form value 'Engineering'   -> rc=25, found "Engineering"
numeric form value 'ENG'           -> rc=25   (no abbreviation)
numeric form value ' ENGINEERING'  -> rc=25   (no trimming)
```

Only the *keyword* form written in source is case-insensitive, and only
because the tokenizer uppercases the token before the instruction sees it.
`set_form_str` takes a runtime `&str`, so it models the VALUE path. Accept
only the two exact uppercase spellings, and rewrite the comment — it asserts a
mechanism the interpreter does not have, and "unambiguous" additionally
implies abbreviations work, which they do not.

## M2 (Minor) — a doc comment citing documentation that does not exist

`format.rs:447` describes `round_to`'s `digits == 0` behaviour as a
"documented no-op sentinel". `round_to`'s own doc comment never mentions it;
the behaviour is an unannotated `if keep == 0` at `lib.rs:485`. The sentinel is
load-bearing and reachable from a public entry point — `compare(a, b, d, d,
op)` produces `working_digits == 0`. Document it where it lives.

## M3 (Minor) — a test pinning a mechanism that does not exist

`tests/settings.rs:25` (`digits_is_capped_at_max_exponent`). Renaming is not
enough; it needs cases that start from a non-default DIGITS, which is the only
thing that distinguishes the two rules.

## Verification

- Re-verify C1's rule yourself against `build/bin/rexx` before writing code,
  from at least three different starting DIGITS. Do not take the table above
  as sufficient; it is the same kind of artifact that caused the defect.
- Add settings cases that start from a non-default DIGITS.
- All eleven differential sets stay at 0. Regenerate with
  `python3 crates/rexx-num/tests/gen-curated-sets.py <name>`; the names are
  addsub addsub2 muldiv md2 pow cmp fmt fmt2 fmt3 fmtedge fmtcarry, totalling
  126,048 cases. Oracle drivers are `data-addsub-oracle.rex` for the first six
  and `data-format-oracle.rex` for the rest.
- `cargo test --offline --workspace` (166 pass, 3 ignored) and
  `cargo clippy --offline --workspace --all-targets -- -D warnings` clean.

## Constraints

- Error numbers and sub-numbers are contract. 26.005, 26.006, 33.001 and 25.011
  are all verified; do not change them, and make sure the FUZZ path reports
  26.006 rather than 33.001 for a non-convertible value.
- Do not run git.
- `--offline` on every cargo command. Zero `unsafe`.
- Leave the three `#[ignore]` markers in `tests/format.rs` alone.
- Not in scope, deliberately: the error-type API inconsistency (M4) and
  `compare`'s `&str`-only entry point (M5). Both are recorded for Phase 3.
