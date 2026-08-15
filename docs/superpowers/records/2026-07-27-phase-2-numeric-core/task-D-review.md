# Task D review — error messages from the generated table

Reviewer: sdd-errors review agent, 2026-07-28. Scope per lead's brief: (1) the
pre-carry exponent logic in `format.rs`, (2) the String-carrying error enum
design, (3) `substitute`/`error_text` edge cases. Test/clippy/differential
status and the lead's own probe verifications were taken as given.

## Finding 1 — Critical, VERIFIED: the post-carry exponent check was deleted, not moved

The report's premise for the new pre-carry block is that the C++ "checks
`expp` against the exponent it derives from `this` alone, entirely before the
section that later applies `after` and can carry" (doc comment,
`rust/crates/rexx-num/src/format.rs:129`). That is only half the story. The
C++ checks **twice**:

- `interpreter/classes/NumberStringClass.cpp:2057` — the pre-carry check the
  new Rust block correctly mirrors (trigger condition, ENGINEERING grouping
  via truncating division with the `-2` shift ≡ `div_euclid`, and the
  substitution being the pre-carry reframed `this` — all match; I re-derived
  the grouping equivalence for negative exponents).
- `interpreter/classes/NumberStringClass.cpp:2190` — a **second** raise of
  `Error_Incorrect_method_exponent_oversize` inside the post-rounding "redo
  the whole trigger thing" block (the [bugs:#1474] kludge, lines 2128–2195),
  reached when supplied `decimals` forces rounding. At that point
  `numberExponent`/`numberDigits` have been re-adjusted, so both the decision
  and the substitution use the *post-carry* state.

The diff moved the old post-resolve check (previously in the `Some(exp)`
render branch) up to the new pre-carry position and deleted the original.
Result: when the pre-carry exponent fits `expp` but the carry pushes its
digit count past it, the Rust code silently succeeds where the interpreter
raises 93.941. Two verified divergences:

| input (digits, call) | `build/bin/rexx` | Rust (`fmt-check`) |
|---|---|---|
| 9, `format(9.996E99,,0,2)` — pre-carry exp 99 fits 2, carry → 100 | 93.941 `Exponent of "1" is too large for 2 spaces.` | `Ok("1E+100")` |
| 15, `format(9999999999.6,,0,1,10)` — plain pre-carry (adjusted 9 < expt 10), carry → 1E+10 | 93.941 `Exponent of "1.000000000" is too large for 1 spaces.` | `Ok("1E+10")` |

The second case shows the recheck also fires when the *pre-carry state was
not exponential at all* — `initial_eng_exp` is `None` there, so the new block
never runs, and nothing else checks.

The implementer's own probe (`format(9.996e20,,0,1)`) is the both-checks-fail
case, where the first (pre-carry) site fires — so their observation
("substitution is the pre-carry mantissa") is correct *for that case*, and
the pre-carry block should stay. What's missing is restoring a check after
`resolve_exponential_state` against the final exponent.

Fix caveat, also verified: the post-carry raise substitutes the C++'s
mid-computation mantissa, which still carries the trailing zeros the display
path later trims — `"1.000000000"`, not the `"1"` that `resolve`'s `rounded`
would render, and not the `"1E+10"` display form. A naive
`render_integer_padded(&rounded, None, None, "")` in the recheck will get the
digits wrong; the substitution needs the carry-resolved value reframed with
its digit count as of the rounding step (n1's length minus dropped decimals),
not the display-trimmed one. Probe before trusting any fix.

Blast radius is confined to the error decision/message: the *successful*
render at the same boundary matches (`format(9999999999.6,,0,3,10)` →
`1E+010`, `format(9.996E99,,0,3)` → `1E+100`, both sides). That is why the
21,296-case differential suite stayed at 0 — it evidently has no
carry-plus-narrow-expp boundary case. Worth adding these two to a corpus.

The doc comment at `format.rs:129-140` ("it has no notion yet of the value
carry will eventually produce") misdescribes the C++ and should be rewritten
alongside the fix.

## Finding 2 — Minor, VERIFIED (logic), currently latent: `substitute` re-scans injected text

`rust/crates/rexx-num/src/lib.rs:372` does sequential `String::replace`
passes, so a substitution value injected on pass *i* is re-scanned by pass
*i+1*: `substitute("... &1 ... &2", &["&2", "X"])` yields `"... X ... X"`,
where the interpreter's single-pass splicer would leave the literal `&2` from
the value intact. Not triggerable today — every current ≥2-substitution call
site (33.001, 93.941, 93.942) passes rendered numbers as `&1` — but the
dispatched 42.001/26.008 disambiguation work will substitute user-controlled
operand text into multi-sub messages, making it live. Recommend a single
left-to-right scan now, while the helper has three call sites.

Non-issues checked: the table's max position is `&4` and no message repeats a
position (grepped `rexxmsg.xml`: 392×`&1`, 127×`&2`, 36×`&3`, 5×`&4`), so the
`&10`-clobbering hazard and repeated-`&N` semantics are unreachable; fewer
subs than placeholders leaves the marker literal, which no verified call site
can hit.

## Point 2 — API shape: recommend carrying substitution values, not rendered Strings

Facts that bear on it:

- These errors map to Rexx syntax conditions (25/26/33/93): raising is
  always cold (the program terminates or a trap runs), so neither the
  allocation nor the lost `Copy` matters in practice. The report's shape is
  not *expensive*; that is not the issue.
- The issue is information loss. The interpreter's condition object exposes
  the raw substitution values separately from the rendered text — verified:
  `numeric digits 5; numeric fuzz 10` gives `condition('o')~additional` =
  `["5", "10"]` alongside `~message`. A future interpreter layer building
  real condition objects needs the values, and a rendered `String` cannot be
  un-spliced.
- The subtle raise-site knowledge this task verified (which of the 33.001
  pair is the rejected candidate; DIGITS vs FUZZ sub-message) lives at the
  raise sites either way — carrying values loses none of it.

Recommendation: keep the enum non-`Copy` and per-site rendering knowledge,
but store the substitution values (typed fields or a small `Vec<String>`)
plus the `(major, sub)` pair, and render in `message()` on demand. Same cost
today, and it avoids a second breaking reshape of the same public enums when
condition objects arrive. Not blocking — callers don't exist yet and the
refactor is mechanical — but doing it while fixing Finding 1 is cheap.

## Everything else

- Spec conformance aside from Finding 1: dependency added as a path dep;
  `rexx-inventory` untouched by the diff; error numbers unchanged; all text
  comes from `error_text` (test files' literal strings are assertions, which
  is the correct place for hand-written text).
- The `ArithError` 42.000/26.000 fallback: the fallback itself is sound given
  the fenced-off raise sites, and the doc comment at `lib.rs:326-357`
  describes it accurately, including that the bare forms are unprobeable. No
  finding.
- Commit 2654fbe7 (`#[ignore]` on the three multi-GB tests): justified and
  well documented; note they now run nowhere automatically — a periodic
  `--ignored` CI job would close that gap, but that is outside this task.

Probes used (scratchpad): `format(9.996E99,,0,2)` d9; `format(9999999999.6,,0,1,10)`
d15; same two with wide `expp` for the display path; `numeric fuzz 10` for
`~additional`. Rust side via `cargo run -p rexx-num --bin fmt-check` on
equivalent case lines.

---

# Re-review of fix round 1 (commit 881a01cb), 2026-07-28

Scope: the round-2 diff only. Both round-1 findings are **resolved**; no new
findings. Approved.

## 1. Gating of `post_carry_exponent_error` vs. C++ :2192 reachability — VERIFIED

Traced condition-by-condition against `NumberStringClass.cpp:2082-2196`:

| C++ gate | Rust equivalent | note |
|---|---|---|
| `decimals != -1` | `let after = after?` | |
| post-reframe `numberExponent < 0` (:2085) | `pre_round.exponent >= 0 → None` | reframe by `eng_exp0.unwrap_or(0)` matches C++, which un-adjusts only when `displayedExponent != 0` — a 0 adjustment is a no-op both sides |
| `adjustedDecimals > decimals` (:2091) | `adjusted_decimals <= after → None` | |
| `adjustedDecimals >= digitsCount` → single-digit branch, **no** recheck (:2100-2118) | `excess >= len → None` | |
| kludge: `showExponentWasTrue \| (mathexp != 0 && trigger)` (:2158) | `eng_exp0.is_some() \|\| matches!(expt, Some(..) if ..)` | `mathexp==0` ⇒ `expt` already `None` ⇒ both arms false, matches |
| eng grouping with `-2` shift (:2163-2170) | `group()` `div_euclid` | equivalence re-derived in round 1 |
| `exponentSize > mathexp` (:2190) | `needed > width` | |

Probes beyond the two committed test cases, all interpreter-vs-Rust matched:

- **ENGINEERING group-crossing carry** (neither new test covers eng):
  `numeric form engineering; format(9.99996E101,,0,2)` at DIGITS 9 —
  pre-carry group 99 (2 digits) fits, carry rolls 999.996→1000 into group
  102 (3 digits). Both sides: 93.941 `Exponent of "1.00" is too large for 2
  spaces.` — the 3-char `"1.00"` also independently re-confirms the
  fixed-digit-count carry on a multi-digit mantissa.
- **Carry that stays in group**: `format(99.996E20,,0,2)` eng → `10E+21`
  both sides, no error.
- **Fractional pre-carry oversize**: DIGITS 10,
  `format(1.234567891E-10,,8,1,9)` — both sides 93.941 `Exponent of
  "1.234567891" is too large for 1 spaces.` (first check fires; see below).

**The `eng_exp0.is_some()` kludge arm can never be the sole cause of an
error**, so its lack of dedicated test coverage does not matter: if the
recomputed trigger is false but the pre-state was triggered, the rounding
carry moved a *negative* adjusted exponent toward zero (or left it alone) —
`|exp2| <= |exp0|`, so `exp2`'s digit count cannot exceed `exp0`'s, and
`exp0` already passed the pre-check. A positive-side carry that grows the
digit count keeps the recomputed trigger true (adjusted grew), bypassing the
arm. Keeping the arm mirrors the C++ (which needs it for the display path)
and is harmless. My attempt to construct an arm-only case (the fractional
probe above) confirmed the screen: any such input already fails the
pre-check.

## 2. The mathRound account — VERIFIED

Read `NumberStringBase::mathRound` (`NumberStringMath.cpp:315-357`): rounds
half-up on the single first-dropped digit; on a full carry sets the leading
digit to 1 and does `numberExponent += 1`, holding `digitsCount` fixed.
`Number::round_to` (`lib.rs:408-438`) does exactly the same
(`kept.insert(0,1); kept.pop(); exponent += 1`), and also rounds on the
first dropped digit only. `round_to_places` grows the vector instead. So the
report's story — same value, different digit count, only the substitution
text exposes it — is the correct reason, not a lucky one. The eng probe's
`"1.00"` (3 digits kept) confirms it on a wider mantissa than the committed
`"1.000000000"` case.

Cutting `excess` trailing mantissa digits ≡ `round_to(len - excess)`
because the cut digits are exactly the trailing ones; `len - excess >= 1`
is guaranteed by the `excess >= len` early return, so `round_to`'s
`digits == 0` no-op sentinel is unreachable. Checked.

## 3. Do the tests pin both checks? — VERIFIED by mutation

- Pre-carry check disabled (`if false &&`): **4 failures** —
  `expp_too_narrow_for_the_exponent_is_error_93`,
  `exponent_oversize_is_reported_before_before_oversize`,
  `..._substitutes_the_mantissa_not_the_exponent_digits`,
  `..._uses_the_pre_carry_exponent_not_the_final_one`. (The post-check
  cannot substitute: no-`after` cases return `None` immediately, and the
  both-fail case would report the wrong mantissa.) The two new tests pass
  under this mutant, as expected — they pin the other check.
- Post-carry call removed: **exactly the two new tests fail**, everything
  else passes.

So each check is independently load-bearing and independently pinned;
neither single-check configuration survives the suite. Tree restored clean
after each mutant (`git status` empty).

## 4. `substitute` single-pass rewrite — VERIFIED, no findings

Single left-to-right scan, never re-scans copied-in text; the round-1
collision case is pinned as a unit test. Edges checked by reading and
against the pinned tests: trailing `&`, unsupplied `&9`, `&0` (fails the
`n >= 1` guard → literal passthrough), digit runs that overflow `usize`
(parse fails → passthrough), `&x` (empty digit range → `&` emitted, `x`
continues normally), multi-byte chars (char_indices/len_utf8 correct).
One behavior change from the old `replace` version: digit consumption is
greedy, so a hypothetical `&1` followed by a literal digit would now parse
as `&1N` and pass through rather than substituting `&1`. Unreachable — the
lead re-confirmed the table holds only `&1`..`&4` with no digit following —
and arguably the more defensible reading; noting for the record only.

## Verdict

Round-1 Critical: resolved, VERIFIED (committed tests + eng/fractional
probes + mutation). Round-1 Minor: resolved, VERIFIED. Nothing introduced
that I could find. Approved.

---

# Review of the error-shape reshape (commit 5ab9b1c0), 2026-07-28

Scope: the reshape diff only. **Approved — zero findings.** The reshape is
the design recommended in round 1, executed faithfully, and the one property
the corpus cannot check (`additional()` content against the live
interpreter) now has full coverage between the lead's four probes and this
review's nine.

## 1. Split-instead-of-tag — holds up, VERIFIED

Every variant now maps 1:1 to a distinct `(major, sub)` table row, which is
what makes `sub_code()` a total function with no runtime data dependency.
Judged pair by pair:

- `Overflow`/`Underflow` (42.901/42.902): correct split. Not just different
  sub-messages — different *field semantics* (adjusted vs raw exponent),
  which a shared variant would have had to encode as a comment-enforced
  convention on one `i32`. The C++ counterpart is
  `NumberStringClass.cpp:321/327` (`checkOverflow`), which passes
  `Numerics::DEFAULT_DIGITS` as a genuine second `reportException` argument
  — so the two-item `additional()` with the literal `"9"` is the
  interpreter's own arity, confirmed live (below). A third raise site
  sharing these symbols exists at `NumberStringClass.cpp:410`, passing
  `adjustedSize` for *both* directions — but it is the number-display path
  gated on `|adjustedSize| > MAX_EXPONENT`, unreachable for any value
  `check_range` has already admitted (the ENGINEERING `-2`/grouping shift
  cannot push an in-range adjusted exponent past the bound; checked the
  arithmetic at both ends). Not a modelling gap.
- `DigitsNotWhole`/`FuzzNotWhole` (26.005/26.006): correct for the same
  reason; both reachable (pinned tests).
- `ExponentComputationOverflow`: the only believed-unreachable variant,
  deliberately so (accepted). Splitting it out was forced the moment
  `Overflow` grew a typed field; keeping the guards' fallback distinct from
  a real range overflow is strictly more honest than before.

No variant is collapsible without reintroducing a tag plus an ordering
convention — the thing the single-field-per-position shape exists to avoid.

## 2. `additional()` ordering — the agreement test is NOT the thing pinning it

The lead's suspicion is right. `additional_and_message_agree_on_every_placeholder`
asserts `message().contains(sub)` — and since `message()` is *derived from*
`additional()` (same call, spliced through `error_text`), a wrong order
corrupts both consistently and `contains` still passes. That test's real
job is narrower: it catches a value present in `additional()` but absent
from the rendered text (an arity drift between the two APIs).

Order is actually pinned twice over, elsewhere:

- Directly: the per-variant `assert_eq!(err.additional(), vec![...])` tests
  compare ordered vectors with distinguishable values — `["1999999999",
  "9"]`, `["100", "**", "999999999"]`, `["5", "10"]`, `["-123.456", "3"]`,
  `["1.000000000", "1"]`.
- Transitively: the exact-string message tests, wherever the two values
  are distinguishable (the one indistinguishable case, 93.941's `["1",
  "1"]`, is covered by the multi-digit `"1.23456789"`/`"1"` sibling).

And the pinned order now matches the interpreter, verified live — which was
the genuinely open property, since the XML position order was previously an
assumption about `~additional`:

| case | `condition('o')~additional` | Rust `additional()` |
|---|---|---|
| `9e999999999 * 9e999999999` | `[1999999999, 9]` | same |
| `1e-999999990 / 1e20` | `[-1000000010, 9]` | same |
| `100 ** 999999999` | `[100, **, 999999999]` | same — the operator IS a separate item (XML `<Sub position="2" name="operator"/>`) |
| `format(123456789012.345,,,1,0)` | `[1.23456789, 1]` | same |
| `numeric digits abc` | `[ABC]` (tokenizer-uppercased) | same rule: raw caller text |
| `numeric fuzz -1` | `[-1]` | same |
| `numeric form value 'bogus form'` | `[bogus form]` | same |

With the lead's four (33.001, 93.942, 26.008, 42.003-empty), all ten
substitution-carrying variants plus the empty-array behaviour are
interpreter-verified.

## 3. `Number` substitution rendering — VERIFIED

- `full_precision` renders at `digits.len()` — no re-round by construction
  (rounding to the exact stored length is a no-op), pinned by the 18-digit
  base test at DIGITS 15.
- Trailing zeros survive: `2 ** 2.50` → interpreter `additional` `[2.50]`,
  message `found "2.50"`; Rust byte-identical (probed both sides this
  review — `Number::parse` keeps the trailing zero and `full_precision`
  does not strip it).
- Sign survives: `2 ** -2.5` → `[-2.5]` both sides, probed.
- `BeforeOversize` renders `value.format(call_digits)` — the round-1
  verified rule (n1's own SCIENTIFIC rendering at the call's DIGITS), now
  stored as `(Number, u32)` instead of pre-rendered; the exact-message
  tests still pass unchanged, so the rendering is bit-identical to what it
  replaced.
- `ExponentOversize` renders the mantissa naturally (no before/after),
  preserving the mid-computation trailing zeros — `["1.000000000", "1"]`
  pinned, matching round 2's interpreter probe.
- The 9-digit default appears nowhere on these paths. The one place a
  caller-visible rendering uses call DIGITS (`BeforeOversize`) is the
  verified interpreter behaviour, not a default leaking through.

The operand-spelling gap (`"1E10"` vs `"1E+10"`) is accepted per the lead
and correctly re-documented as a *value* gap, not just a text gap.

## 4. What the corpus cannot see — checked, nothing found

The corpus compares rendered results and error *numbers*; it has never seen
`additional()` at all, and message text only via the unit tests. That is
exactly where this review spent its probes (tables above). Also checked:

- `FormatError::code(self)` stays by-value for the off-limits bin callers —
  compiles against them (lead's test run) and semantics unchanged.
- `render_integer_padded`'s widened placeholder parameters
  (`&Number, u32`): TRUNC passes `(&truncated, 0)` with `before` always
  `None`, so the placeholder is never read — same dead-argument shape as
  before, just typed.
- The `sub_code()`/`code()` mapping duplication inside `ArithError` is
  internally consistent (major of every `sub_code()` equals `code()`);
  pinned by the per-variant tests. A derived consistency assert would be
  nice-to-have, not a finding.

## Verdict

Approved, zero findings. Every substitution-carrying variant's
`additional()` is now interpreter-verified in content, arity, and order;
the reshape loses nothing that was previously verified (all round-1/round-2
exact-message pins pass unchanged) and gains the un-spliceable values the
condition-object layer will need.
