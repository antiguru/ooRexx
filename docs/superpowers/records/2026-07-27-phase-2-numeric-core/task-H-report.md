# Task H report — NUMERIC validation rule and narrowing casts

Status: complete. All work uncommitted, confined to `rust/crates/rexx-num/`
(note: the brief's paths omit the `rust/` prefix; the crate lives at
`rust/crates/rexx-num/`).

## C1 — the rule, re-derived from `build/bin/rexx`

I re-probed before writing code, from starting DIGITS 1, 2, 3, 4, 5, 9, 10,
18, 19, 20 and 30 (`signal on syntax` + `interpret`, per the suggested
harness). The brief's table is correct as far as it goes, but the real rule
has a rounding dimension the table does not show:

**The candidate is converted by `requestUnsignedNumber(setting,
number_digits())`: it is rounded at the DIGITS currently in force (half-up on
the first dropped digit), every decimal digit that remains must reduce to
zero — literal zeros, or all nines consumed by the carry — and the result
must not exceed `Numerics::maxValueForDigits(D)` = 10^min(D,18) − 1. DIGITS
additionally requires ≥ 1. Conversion failure is 26.005 (DIGITS) / 26.006
(FUZZ); only a successful conversion reaches the fuzz-below-digits check
(33.001).**

Probes that establish it (all against `build/bin/rexx`):

| start D | statement | result |
|---|---|---|
| 3 | `numeric digits 1000` | E26.5 |
| 9 | `numeric digits 1000000000` | E26.5 |
| 10 | `numeric digits 1000000000` | OK, digits 1000000000 |
| 18 | `numeric digits 999999999999999999` | OK |
| 30 | `numeric digits 1000000000000000000` | E26.5 (MAX_WHOLENUMBER cap) |
| 1 | `numeric digits 10` | E26.5 (positions, not significant digits) |
| 2 | `numeric digits 1e2` | E26.5 (same: 1-digit mantissa, 3 positions) |
| 4 | `numeric digits 999.9999` | **OK, digits 1000** (rounding carry) |
| 3 | `numeric digits 999.6` | E26.5 (carry lands one position too wide) |
| 3 | `numeric digits 999.4` | OK, digits 999 (rounds down) |
| 9 | `numeric digits 0.99999999995` | **OK, digits 1** (carry from below the point) |
| 9 | `numeric digits 100.0000001` | OK, digits 100 (rounds clean at D=9) |
| 19 | `numeric digits 1.9999999999999999995e17` | OK, digits 200000000000000000 |
| 20 | same value | E26.5 (no rounding at D=20; fraction remains) |
| 3 | `numeric fuzz 12345` | E**26.6** |
| 3 | `numeric fuzz 12` | E33.1 [3, 12] |
| 20→18-digit D | `numeric fuzz 999999999999999998` | OK |

So the rounding-carry rows (999.9999 → 1000, 0.99999999995 → 1) are accepted
values the brief's "must fit" phrasing alone would not predict. This also
ruled out the obvious implementation: the rounded *rendering* of
0.99999999995 at D=9 is `1.00000000` — it contains a decimal point in the
very case the interpreter accepts, so no check on `format()` output can
express the rule. I ported `NumberString::unsignedNumberValue` +
`checkIntegerDigits` + `createUnsignedValue` (`NumberStringClass.cpp:632`,
`:937`, `:788`) digit-for-digit instead, after reading all three.

Verification of the port: a dedicated differential harness (scratchpad only,
not committed) ran 1,056 `(kind, starting-DIGITS, candidate)` cases — 2 kinds
× 11 starting DIGITS × 48 values including every boundary above — comparing
OK/error, the error sub-code, *and* the substitution values against the
interpreter: **0 divergences**.

## The u64 widening

`Settings::digits`/`fuzz` are `u64` (legal range reaches 999999999999999999);
`FuzzNotBelowDigits` carries `u64` fields; every operation signature widened
and carried through: `add`/`sub`/`mul`/`div`/`pow`/`compare`/`round_to`/
`format`/`format_form`/`format_with`/`trunc`, plus `DEFAULT_DIGITS` and the
three differential bins' parsers. No truncation at any boundary: a new
`working_length(digits)` helper saturates the `digits + 1` operand-truncation
width, and every comparison site saturates `digits` into `i64`
(`i64::try_from(..).unwrap_or(i64::MAX)`) rather than casting — exact because
the other side of each comparison is bounded by ±MAX_EXPONENT plus a digit
count.

## I1 / M1

- `muldiv.rs` `div`: the `digits as i32` whole-quotient check is now the
  saturated-i64 comparison, same shape and comment trail as the `format`/
  `as_whole` fixes.
- `muldiv.rs` remainder tail: the working precision is computed in u64 with
  `saturating_add`; the old `as u32` truncation of the sum is gone.

## Cast sweep (complete, untruncated)

I dumped **every** `as {i8,u8,i16,u16,i32,u32,i64,u64,usize,isize}` and
`try_into` in the crate (src + bins) and audited each line — 100 hits, no
`head`. Findings:

1. **One real site, introduced by this task's own widening**: the `2 * expt`
   low-end exponential trigger in `format.rs` (three copies: `format_with`'s
   first check, `resolve_exponential_state`, `post_carry_exponent_error`).
   `expt` defaults to `digits` saturated to i64::MAX, so the doubling
   overflowed — panic in debug, wrapped comparison in release. Fixed with
   `saturating_mul(2)` at all three, pinned by
   `format_survives_a_bare_digits_at_the_top_of_u64` (fails in debug without
   the fix).
2. **Nothing else of the digits-parameter narrowing class.** The remaining
   casts fall into three benign groups: digit-byte casts (values 0–9);
   provably-bounded conversions (branch-guarded, or non-negative i32→usize);
   and a pre-existing class where a *multi-gigabyte literal's own digit
   count* could overflow i32 (`truncated_to`'s exponent bump,
   `adjusted_exponent`, `mul`'s `extra as i32`) — input-size pathologies
   unreachable through any `digits` parameter, unchanged by this task, and
   flagged here rather than silently "fixed".

## I2, M2, M3

- `set_form_str`: exact `"SCIENTIFIC"`/`"ENGINEERING"` only. Re-verified
  myself: `engineering`, `Engineering`, `ENG`, `' ENGINEERING'`,
  `scientific` are all 25.011 via `numeric form value`. Comment rewritten
  (tokenizer-uppercasing explanation); test now asserts all five rejections
  and that a rejection leaves the setting unmoved.
- `round_to`'s `digits == 0` no-op sentinel documented on `round_to` itself
  (`lib.rs`), including the `compare(a, b, d, d, op)` reachability;
  `format.rs`'s comment now points there instead of claiming external
  documentation.
- The fixed-cap test is gone, replaced by four tests that all start from
  non-default DIGITS (including the 4294967296-from-10 row that forces u64,
  and the rounding-carry rows). Two of my first drafts failed because a
  successful `set_digits_str` moves the boundary for the *next* call — both
  behaviours re-probed against the interpreter (`digits 1` then `1e3` fails;
  `digits 999999999` then `1000000000` succeeds) before fixing the tests.

## Verification

- All eleven curated sets regenerated and run twice (once mid-task, once
  after the final `saturating_mul` fix): addsub 8712, addsub2 8112, muldiv
  17424, md2 20184, pow 2112, cmp 32368, fmt 1800, fmt2 6720, fmt3 12136,
  fmtedge 640, fmtcarry 15840 = **126,048 cases, 0 divergences** on every
  set.
- `cargo test --offline --workspace`: **170 passed, 0 failed, 3 ignored**
  (brief expected 166; the +4 are new: 3 net in `tests/settings.rs`, 1 in
  `tests/format.rs`). The three `#[ignore]` markers in `tests/format.rs` are
  untouched.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: clean.
- Zero `unsafe`; `gen-curated-sets.py` untouched; no other crate touched.
- One transparency note: I ran a single read-only `git status` to enumerate
  touched files for this report — no staging, no state change.

## Files changed

All under `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-num/`:
`src/settings.rs`, `src/lib.rs`, `src/addsub.rs`, `src/muldiv.rs`,
`src/pow.rs`, `src/compare.rs`, `src/format.rs`, `src/bin/{addsub,muldiv,fmt-check}.rs`,
`tests/{settings,addsub,muldiv,pow,compare,parse,format}.rs`.

## Not in scope, unchanged

M4 (error-type API inconsistency) and M5 (`compare`'s `&str`-only entry),
per the brief.

## Fix round 1

### H2 — overflowing give-up bound in `long_divide`

`n.len() + want + d.len()` at `muldiv.rs` is now two `saturating_add`s, with
a comment noting that producer-side saturation (`working_length`) demands
matching consumer-side arithmetic. The exact repro — `div(123456, 2,
u64::MAX, IntegerDivide)` — is pinned in `tests/muldiv.rs`; it now returns
error 5 (see H3: the reservation rejects u64::MAX before `long_divide`
runs), and the test running in debug is the no-panic proof.

### H3 — unbounded division at huge legal DIGITS

Where the real bound is: `NumberString::Division` allocates
`3 * ((digits + 1) * 2 + 1)` bytes **up front**, before the first quotient
digit (`NumberStringMath2.cpp:401-417`, `new_buffer(totalDigits * 3)` once
`totalDigits > FAST_BUFFER` = 48), and a failed allocation raises error 5
from the memory manager (`RexxMemory.cpp:1266`). There is no fixed
threshold — the boundary is what malloc grants. Probes: `1/7` computes at
DIGITS 1e10 but is 5.0 at 1e11 on this machine; and crucially `4.0 / 2` and
`123456.0 % 2` are **also** 5.0 at DIGITS 999999999999999999, because the
buffer is sized before the operands are looked at. (The bare-integer forms
`4 / 2`, `123456 % 2` succeed at the same DIGITS — that is the RexxInteger
fast path, a layer above this crate, bypassing `NumberString::Division`
entirely.)

Port: a fallible reservation (`Vec::try_reserve_exact`) of the same request
size, at the same point in `div` — after the `%`/`//` no-integer-part early
returns (the C++ allocates after its equivalents; `0.001 // 7` still
succeeds at DIGITS 10^18-1, test-pinned), gated behind the same
`FAST_BUFFER` cutoff so ordinary divisions never pay for a probe. Failure
maps to the new `ArithError::SystemResources` — code 5, sub (5, 0), empty
`additional()`, table text "System resources exhausted." (per instruction,
trusting rc + table over the interpreter's broken rendering of this one).
~6e18 bytes fails deterministically on any 64-bit machine, so the
`max_legal`-DIGITS tests are stable; the machine-dependent middle (1e10 vs
1e11) is documented, not asserted.

### H1 — blanks after a sign (pre-existing)

`Number::parse` now mirrors `numberStringScan`
(`NumberStringClass.cpp:1264-1296`): blanks are skipped at either end and,
when a sign is present, between the sign and the first digit — and a blank
is exactly space or tab. My probes extended the finding: the old `str::trim`
also accepted LF/VT/FF/CR at the ends, all of which the interpreter rejects
(confirmed `'0a'x||'3' + 0` etc. are error 41), so the ends now use the
same two-byte class instead of Unicode whitespace. New unit tests cover
both directions plus the control-character class.

New curated set: **`signblank`**, 2,320 cases (29 values x 10 operators x 4
right operands x DIGITS 3 and 9) through the muldiv harness and
`data-addsub-oracle.rex` — valid sign-blank spellings, invalid ones
(`'+ 3 e2'`, `'3 4'`, `'++ 3'`, bare signs), and tab variants; 480 of its
oracle rows are error 41, so both directions are pinned. `**` is deliberately
excluded (non-numeric power operands error through a path this harness does
not model). `gen-curated-sets.py` gained only this set, per the reversed
instruction; existing sets untouched.

### Verification (fix round 1)

- All twelve sets at 0: the original eleven unchanged (8712, 8112, 17424,
  20184, 2112, 32368, 1800, 6720, 12136, 640, 15840 = 126,048) plus
  signblank 2,320 = 128,368.
- `cargo test --offline --workspace`: 176 passed, 0 failed, 3 ignored
  (+6: 3 parse, 2 muldiv, 1 errors). `#[ignore]` markers untouched.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: clean.
- Zero `unsafe`. One read-only `git log`/`git diff --stat` pair was run when
  an edit reported the file changed on disk (it was the central commit
  e91a333d); nothing staged, and no further git.
