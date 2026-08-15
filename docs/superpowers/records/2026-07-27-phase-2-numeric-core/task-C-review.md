# Task C review — FORMAT / TRUNC / ENGINEERING (re-review after fix round 1)

Reviewed: `review-f360bb35..e79f6fb7.diff` (commits e9fb7c1d, e79f6fb7) against
`task-C-brief.md` and `task-C-report.md`. C++ oracle read at
`interpreter/classes/NumberStringClass.cpp`; edge behavior settled by running
`build/bin/rexx` directly (probes below).

## Verdict A: SPEC COMPLIANCE — ✅ (one boundary exception, filed under B)

| Requirement (brief) | Status | Evidence |
|---|---|---|
| ENGINEERING form, exponent forced to multiple of 3 | Met | `group` uses `div_euclid(3)*3`; C++ `NumberStringClass.cpp:2034-2043` uses `-2` bias + truncating division, which is algebraically the same floor for all signs (checked −27..0 by hand). Interpreter probe `format(0.0001,,,,1)` under ENGINEERING → `100E-6`, matches Rust path. |
| Form-aware entry point, `Number::format` unchanged | Met | `format_form` added; `lib.rs` diff is only `mod format;` + re-export. |
| `FORMAT(number, before, after, expp, expt)`, exact interpreter reproduction | Met* | Trigger `adjusted >= expt \|\| (adjusted < 0 && \|raw\| > 2*expt)` matches `NumberStringClass.cpp:2029` and re-trigger `:2158` including the #1474 once-exponential-stays-exponential kludge (`showExponentWasTrue`), mirrored by `eng_exp.is_some() \|\|`. Displayed-exponent-0 suppression matches `:2356-2371`: blanks only when `mathexp != -1`, width `leadingExpZeros + exponentSize + 2 = expp + 2` — exactly the Rust `" ".repeat(width+2)`. *Exception: u32 args ≥ 2^31, see findings 1–2. |
| Error numbers part of contract | Met | Both oversize paths are 93 (`Error_Incorrect_method_exponent_oversize` `:2059/:2192`, `before_oversize` `:2300`). Precedence verified in C++: exponent check (`:2057`, `:2190`) runs before integers check (`:2298`); Rust checks `expp` before `render_integer_padded`. Sign consumes one `before` slot (`:2268-2271`) and error reports the requested value (`:2300`) — both mirrored. |
| `TRUNC(number, n)`, truncate not round, default 0 | Met | `truncate_to_places` has no carry path; `truncInternal` `:1413` confirms zero-padding/sign-drop structure; differential 20k cases at 0. |
| Interpreter is the oracle, not docs/ANSI | Met | Report's method + my 5 independent probes (below) all agree. |
| Brief's minimum probe coverage pinned as tests | Met | `tests/format.rs` covers 0–5 args, negatives, carries, both errors + ordering, expp/expt zeros, TRUNC negatives/underflow. |

### Independent interpreter probes (all match the Rust code path, traced by hand)

| Probe | Interpreter | Rust (traced/ran) |
|---|---|---|
| `format(0.25,,0,,0)` — underflow-carry inside exponential frame, expt 0 | `3E-1` | `3E-1` ✓ |
| `format(0.25,,0,2,0)` | `3E-01` | `3E-01` ✓ |
| `format(0.5,,0,,0)` — mantissa has no decimals, `after` is a no-op | `5E-1` | `5E-1` ✓ |
| `format(0.6,,0,,1)` — full-underflow round-up stays plain | `1` | `1` ✓ |
| ENG `format(0.0001,,,,1)` — fractional grouping with the −2 bias | `100E-6` | `100E-6` ✓ |

These sit exactly in the gaps the three case sets could not reach (expt-0 +
after combined with underflow; C++ takes the `adjustedDecimals >= digitsCount`
branch `:2100-2118` which has *no* re-trigger — verified it can never newly
cross the trigger there, so Rust's uniform loop is equivalent).

The restructure-moves-rounding check: C++ rounds the mantissa once in the old
frame and re-groups in place (`:2126-2174`); Rust re-derives from the
untouched `n1` at the new frame. The frame only ever moves after a full carry,
whose mantissa is 1 followed by zeros — re-rounding that at the shifted frame
reproduces the same digits, so the rounding point is preserved. The Rust
comment claiming plain numbers can only newly trigger via the upper arm is
correct: rounding decimals moves the raw exponent toward zero, so the
low-end arm (`\|raw\| > 2*expt`) can never newly fire — C++ `:2158` rechecks
both arms but the low arm is vacuous there.

## Verdict B: findings

### Finding 1 — Important, VERIFIED: overflow panic on `after`/`places >= 2^31`

`format.rs:334` (and the same expression in `round_to_places`):
`let target_exponent = -(places as i32);`

`places = 2_147_483_648` → `as i32` gives `i32::MIN`, negation panics
("attempt to negate with overflow") in debug; in release it wraps to
`i32::MIN`, then `pad_to_exponent`'s `(n.exponent - target_exponent) as usize`
overflows again → sign-extended ~1.8e19 allocation → abort.

Reproduced both sides:
- `build/bin/rexx`: `trunc(1, 2147483648)` **succeeds** and prints a
  2,147,483,650-char string (6.8 GB probe transcript) — `optionalNonNegative`
  (`MethodArguments.hpp:601`) is `size_t`-ranged, so the interpreter does
  *not* clamp at 999,999,999.
- `fmt-check` on `9|TRUNC|1|2147483648|||`: panic at `format.rs:334`.

The report's out-of-scope declaration covers *negative / non-whole* arguments;
2147483648 is a positive whole number inside the declared `u32` domain, so the
carve-out does not cover it. This project has already shipped one i32 overflow
found only by randomized input; this is the same shape.

Suggested fix: widen the decimal-place arithmetic to i64 (exponents are
already bounded by `MAX_EXPONENT`, so one comparison against that bound before
narrowing is enough), or clamp `places`/`after` to `<= MAX_EXPONENT` at entry.

### Finding 2 — Important, VERIFIED: `before >= 2^31` wraps to a spurious error 93

`format.rs` `render_integer_padded`: `let available = before as i32 - i32::from(n.negative);`
`before = 3_000_000_000` wraps negative → returns `BeforeOversize`.
Reproduced: `fmt-check` on `9|FORMAT|1|3000000000|||` → `<E93>`, where the
interpreter (same size_t argument path as Finding 1) pads with 3e9 spaces and
succeeds. Silent wrong answer, no panic. Same fix family as Finding 1.

Not defects of the same kind: `expp` is compared and padded in unsigned/usize
space throughout, so it survives the full u32 range (huge allocations match
the interpreter's own behavior, as Finding 1's probe demonstrates).

### Finding 3 — Minor: harness `arg()` swallows malformed fields

`fmt-check.rs:14` `s.parse().ok()` maps an unparseable argument field (e.g.
`4294967296`, or a stray `-1` from a future generator bug) to "omitted"
silently, so a corrupted case file tests a different case than the oracle ran.
Divergence would usually surface as a mismatch, but the failure would be
mystifying. `unwrap` (or an explicit `<badarg>` marker) would fail loudly.

### Notes, no action needed

- Comments were checked against both the code and the C++; none contradict.
  The long doc comments on `format_with`/`resolve_exponential_state` are
  accurate, including the claims I was suspicious of (the `expp+2` blank
  width, error precedence, floor-vs-truncate grouping).
- The 8-pass cap on the convergence loop: C++ re-derives exactly once
  (`:2138-2195`), and I could not construct a case where Rust's loop changes
  state on a third pass (frame moves require carry-to-power-of-10, which is
  stable on the next pass). The cap is unreachable defensive code, as the
  report says.
- `#[allow(clippy::too_many_arguments)]` on a 7-arg test helper: justified.
- Zero `unsafe` in the diff. Leaf-crate constraint holds (`format.rs` imports
  only `crate::{Form, Number}`).
- `FormatError::code(self)` ignores `self`; consistent with `ArithError`'s
  convention per the report, fine.

## Bottom line

Spec compliance ✅ for everything the brief names; the mechanism claims in the
report are real (verified in C++ and by live probe), not curve-fit. Two
Important integer-width defects at the `u32 >= 2^31` boundary (one panic, one
wrong error), both verified with concrete repros and both *outside* the
report's declared out-of-scope area since the inputs are positive whole
numbers. Fix is small and local. Everything else approved.

---

# Round 2 re-review — commit e4fb4ed1 (overflow fix)

Scope: the fix diff only (`review-round2.diff`). Differential/test evidence
taken from the team lead's verification; this section is the argument from
the code.

## Q1 — equivalence of the data-flow change: VERIFIED equivalent, rounding point unmoved

The cut/carry code in `round_to_places`/`truncate_to_places` is byte-identical
to round 1; only the value-preserving zero-*padding* moved from Number-space
(`pad_to_exponent`) to string-space (`render_integer_padded`). Padding never
rounds, so no rounding point moved. The residual risk was a consumer seeing
the now-unpadded Number between rounding and rendering. There is exactly one:
`adjusted(&rounded)` in `resolve_exponential_state`'s trigger re-check
(`format.rs:237` area). Padding is adjusted-invariant — appending k zero
digits while lowering the exponent by k leaves `exponent + len - 1` unchanged
(including for zero: `[0]@0` and `[0,0..]@-k` both have adjusted 0... the
padded zero gives `-k + (k+1) - 1 = 0`). So every trigger decision, carry
re-derivation, and `before`-oversize check computes the same values as before.
Render-side: the rounded Number reaching `render_integer_padded` provably has
at most `after` decimals (early return requires `exponent >= -places`; the cut
paths produce exactly `-places`), so the `saturating_sub` extension is only
ever additive, and `after == Some(0) => None` is consistent because a number
rounded to 0 places always has `exponent >= 0`, hence `natural_dec == None`
anyway. Sign survives the new `n.clone()` early return just as it survived
`pad_to_exponent`.

## Q2 — carry path staying i32: VERIFIED sound

The deep path requires `i64::from(n.exponent) < target_exponent_wide`, and
`target_exponent_wide <= 0` always. So `target_exponent_wide` lies strictly
between `n.exponent` — itself an `i32` by the crate invariant the fix cites —
and 0 inclusive. Any value strictly between an `i32` and 0 fits `i32`; the
narrowing cast at `format.rs:295` cannot truncate. A `places` large enough to
threaten the cast forces `-places < n.exponent` to reach the branch, i.e.
`n.exponent < -2^31`, unrepresentable. The subsequent
`(target_exponent - n.exponent) as usize` is positive and bounded by
`-n.exponent <= MAX_EXPONENT + digits` (`n` is post-`round_to(digits)`, so
`len <= digits` and adjusted is within `MAX_EXPONENT`), ~1e9, well inside
i32 — pre-existing arithmetic, unaffected by the fix.

## Q3 — pad_to_exponent removal: VERIFIED clean

Its only two callers ever were `round_to_places` and `truncate_to_places`
(round-1 diff); grep of the current `format.rs` shows zero remaining
references. Its behaviour is fully subsumed: the resolve loop needs no
padding (adjusted-invariance above), and rendering reproduces it in string
space.

## Q4 — the new `after` parameter for None callers: no change

`after == None` reproduces the old `dec_part = natural_dec` path exactly, and
in the old code `after == None` meant no padding ever happened either.
Callers: `format_with` threads its own `after` through both branches
(`format.rs:113`, `:124`), so rounding and rendering always see the same
value; `format_form` passes None throughout. `trunc` now passes
`Some(places)` where it passed nothing — traced `trunc("1e-10", 9, 20)`
end-to-end: early-return unpadded, render extends 10 natural decimals with
10 zeros → identical output to the old exponent-folded path.

## Findings

None Critical or Important. Two notes, no action required:

- Minor: `saturating_sub` and `Some(0) => None` are defensive against an
  invariant violation ("input already cut to ≤ after decimals") that would
  otherwise surface as silently-extra or silently-dropped decimals rather
  than a loud failure. The invariant is real and documented; a
  `debug_assert!` would make a future violation visible.
- Note: the three new boundary tests materialise 2–3 GB strings each; fine
  in release (2.93s measured), noticeably costly in a debug `cargo test`
  run. Not a defect.

## Bottom line

Both round-1 findings genuinely resolved, by argument from the code and not
just by the (fix-iterated) differential sets: Finding 1's panic site is gone
(branch decided in i64, deep path provably i32-safe), Finding 2's wrap is
gone (`i64` widening at `format.rs:~445`). No new defects introduced.
