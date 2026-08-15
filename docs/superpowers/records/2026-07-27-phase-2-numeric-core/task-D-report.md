# Task D — Error messages from the generated table (Phase 2, Task 2.8)

Status: DONE, with one deliberate, documented gap (see "What has no clean
counterpart" below). `rexx-num` now depends on `rexx-inventory` and every
error type it exposes (`ArithError`, `SettingsError`, `FormatError`) has a
`message()` method that renders the interpreter's exact text, substitutions
filled. Verified with ~30 probe programs run under `build/bin/rexx`, pinned
as 13 new tests, and the four required FORMAT differential sets stay at 0
diffs (21,296 cases total).

## How substitutions are filled

`rexx-inventory`'s table keeps `&1`, `&2`, … literal in `text` (confirmed by
reading its `tests/errors.rs`, which pins e.g. `"Failure during
initialization: File \"&1\" is unreadable."`). Two small helpers added to
`lib.rs` do the filling:

```rust
pub(crate) fn substitute(text: &str, subs: &[&str]) -> String   // &1, &2, ... -> subs[0], subs[1], ...
pub(crate) fn error_text(major: u16, sub: u16, subs: &[&str]) -> String  // lookup + substitute
```

## ArithError (`lib.rs`) — one clean, two collapsed

- **`DivideByZero` → 42.003**, "Arithmetic overflow; divisor must not be
  zero.", no substitution. Single C++ site
  (`NumberStringMath2.cpp:355`, `Error_Overflow_zero`) shared by `/`, `%`,
  and `//` alike — confirmed all three give identical text. Probe: `1 / 0`.

- **`Overflow` and `NotWholeNumber` do not have a single clean counterpart.**
  Grepping the raise sites in `muldiv.rs`/`pow.rs` (outside this task's file
  scope) against the C++ source shows each variant is genuinely raised from
  more than one call site that the interpreter reports *differently*:
  - `NotWholeNumber`: 26.011 for `%` ("Result of % operation did not result
    in a whole number.", no sub), 26.012 for `//` (same shape), or 26.008
    for `**` ("...found \"2.5\".", substituting the exponent). Probes:
    `123456 % 2` at DIGITS 3, `123456 // 2` at DIGITS 3, `2 ** 2.5`.
  - `Overflow`: 42.901/42.902 for the general range check `mul`/`div`/`pow`
    share (substituting the adjusted exponent and DIGITS — probe: `9e999999999
    * 9e999999999` → 42.901 `exponent ("1999999999") exceeds 9 digits`; `1e-999999990
    / 1e20` → 42.902), 42.903 for zero to a negative power (no sub — probe:
    `0 ** -1`), or 42.001 for `**`'s own upfront magnitude precheck
    (substituting the base, `"**"`, and the exponent — probe: `100 **
    999999999` → `Arithmetic overflow detected at:  "100**999999999".`).

  This crate's `Result<_, ArithError>` boundary does not preserve which call
  site raised the error, and I could not touch `muldiv.rs`/`pow.rs` to add
  that context. Recovering it from outside would mean re-deriving those
  files' internal magnitude/carry checks in the caller — exactly the kind of
  arithmetic their own comments flag as easy to get subtly wrong — so I did
  not attempt it. Both variants' `message()` instead report the bare
  major-code text (42.000 "Arithmetic overflow/underflow.", 26.000 "Invalid
  whole number."), the only text that is not wrong for any of the paths that
  share the variant. I grepped the whole C++ tree for a `reportException`
  call using bare `Error_Overflow` or `Error_Invalid_whole_number` and found
  none — the interpreter never actually emits either bare form, so unlike
  every other message in this task, these two cannot be confirmed against a
  live probe. Flagging this plainly rather than picking one of the specific
  sub-messages and being wrong 2/3 of the time.

## SettingsError (`settings.rs`) — fully resolved, variants now carry text

Unlike `ArithError`, every raise site is in this file, so I changed
`NotWholeNumber`/`FuzzNotBelowDigits`/`InvalidForm` from unit variants to
`(String)` tuple variants holding the fully-rendered message, computed right
where the crate already has everything needed:

- **`InvalidForm` → 25.011**, substituting the raw text as given (confirmed
  not uppercased, unlike a bare source-code keyword: `x = "bogus form";
  numeric form (x)` reports `found "bogus form"`, lowercase preserved).
- **`NotWholeNumber` → 26.005 (DIGITS) or 26.006 (FUZZ)** — the same code
  number, different sub-message depending which setter is raising it, which
  the shared variant name alone can't tell apart. `set_digits_str` and
  `set_fuzz_str` each render their own. Probes: `numeric digits abc` (and
  `-1`, `1.5`, `1000000000`, `2147483647`, `4294967296` — all six funnel to
  26.005, confirmed against the C++'s single `requestUnsignedNumber` check
  for DIGITS), `numeric fuzz -1` (26.006).
- **`FuzzNotBelowDigits` → 33.001**, substituting the DIGITS/FUZZ pair the
  setting *would* end up with — the just-attempted candidate for whichever
  one is being set, the unchanged stored value for the other, not
  necessarily either's current `self` field. Confirmed both directions:
  `numeric digits 5; numeric fuzz 10` → `("5") ... ("10")` (fuzz shows the
  rejected candidate); `numeric fuzz 5; numeric digits 3` → `("3") ...
  ("5")` (digits shows the rejected candidate).

`code()` stays `fn code(&self) -> u16` (kept for callers' convenience);
`message(&self) -> &str` returns the stored text.

## FormatError (`format.rs`) — fully resolved, one real correction found

Both raise sites are in this file too, so `BeforeOversize`/`ExponentOversize`
also became `(String)` variants. `code(self) -> u16` stays *by value*
(unchanged) because `src/bin/fmt-check.rs` — outside my scope — calls it as
`FormatError::code(e)`.

- **`BeforeOversize` → 93.942**, "Integer part of "&1" is too large for &2
  spaces." I initially assumed &1 was the padded/reframed mantissa
  `render_integer_padded` works with; it is not. It is `n1` (`self` rounded
  to the call's `digits`, *before* any exponential reframing), rendered in
  its own default SCIENTIFIC form — unaffected by which `form` the call
  itself uses. Confirmed three ways: an ENGINEERING call whose padded
  mantissa is `"123.456789"` reports the un-reframed `"123456.789"`; lowering
  DIGITS until rounding actually changes the value shows *that* rounded
  value, in SCIENTIFIC form, even though the call forces plain output via
  `expt=20` (`"1.2346E+5"`, not `"123460"`); `format(-123.456, 3)` matches
  the doc comment already in the file. &2 is `before` itself, exactly as the
  pre-existing doc comment said (verified, not new).

- **`ExponentOversize` → 93.941**, "Exponent of "&1" is too large for &2
  spaces." **This is the one place I had it wrong on the first pass and the
  probes caught it.** I first assumed &1 was the exponent's own digit count
  (matching &2's role in `needed > width`). Two single-digit-mantissa probes
  (`1e10`, `1e100`, both expp=1) both returned `"1"`, which is consistent
  with *either* hypothesis (mantissa "1" happens to look identical to a
  truncated exponent digit) and would have shipped a wrong text with a
  passing-looking check. A probe with a distinguishable multi-digit mantissa
  (`format(123456789012.345, , , 1, 0)`) broke the tie: `Exponent of
  "1.23456789" is too large for 1 spaces.` — &1 is the **mantissa**, at full
  natural precision, confirmed unaffected by `before`/`after` (both apply
  only after this check passes). Traced to
  `NumberStringClass.cpp:2059`: `reportException(Error_Incorrect_method_exponent_oversize,
  this, mathexp)` — `this` is the NumberString mid-computation, already
  reframed to the mantissa/exponent split, and `mathexp` is `expp` itself
  (not a computed digit count).

  A further wrinkle, also caught by a probe rather than assumed: the same
  C++ code runs its `mathexp` check *before* the section that applies
  `after` and can carry (`9.996E+20` rounded to 0 decimals carries its
  exponent from 20 to 21 — see the existing, untouched
  `after_rounding_carry_can_bump_the_exponent_itself` test). Forcing a carry
  and a too-narrow `expp` simultaneously (`format(9.996e20, , 0, 1)`) shows
  `Exponent of "9.996" is too large for 1 spaces.` — reframed at the
  *pre-carry* exponent (20), not the post-carry one (21, which would read
  `"0.9996"`). `format_with` now computes this pre-carry trigger/grouping
  separately (small, deliberately duplicated 4 lines mirroring
  `resolve_exponential_state`'s first guess) rather than reusing its final,
  carry-resolved result for the oversize check. This also matches the width
  *decision* itself, not just the text — I could not fully rule out from
  probes alone that the decision (oversize or not) also needs the pre-carry
  exponent rather than just the message, so I moved both to use it,
  consistent with the C++ structure, and let the differential suite be the
  check: still 0/21,296 after the change.

## Test/clippy summary

`cargo test --offline --workspace`: 147 passed, 0 failed (134 baseline + 13
new: 4 in new `tests/errors.rs`, 4 in `tests/settings.rs`, 5 in
`tests/format.rs`). `cargo clippy --offline --workspace --all-targets -- -D
warnings`: clean. FORMAT differential sets (fmt/fmt2/fmt3/fmtedge,
1800/6720/12136/640 = 21,296 cases): all 0 diffs against `build/bin/rexx`,
oracle vs. `target/release/fmt-check`.

## Concerns for the team lead

1. `ArithError::Overflow`/`NotWholeNumber` render text the interpreter never
   literally emits (see above) — anyone consuming `.message()` on these two
   should know it's the best available generic text, not a verified exact
   match, unlike every other message this crate produces.
2. `SettingsError`/`FormatError` changed from unit-ish `Copy` enums to
   `String`-carrying `Clone` enums (codes unchanged). No other crate/file
   references either type (`grep`-confirmed), and all touched test files are
   in my permitted scope, but flagging the shape change explicitly since it's
   more than "add a method."
3. The pre-carry-vs-final-exponent distinction for `ExponentOversize` was
   discovered, not anticipated by the brief — worth a second pair of eyes on
   `format_with`'s new `initial_eng_exp` block given how easy this exact
   class of bug (using the converged value where the interpreter uses the
   first guess) is to get wrong, per `resolve_exponential_state`'s own doc
   comment.

## Files touched

- `rust/crates/rexx-num/Cargo.toml` — added `rexx-inventory` path dependency.
- `rust/crates/rexx-num/src/lib.rs` — `ArithError::message`, `substitute`,
  `error_text`.
- `rust/crates/rexx-num/src/settings.rs` — `SettingsError` now carries text;
  `message()`; each setter renders its own sub-message.
- `rust/crates/rexx-num/src/format.rs` — `FormatError` now carries text;
  `message()`; `render_integer_padded` takes the before-oversize
  substitution text; new pre-carry exponent-oversize check.
- `rust/crates/rexx-num/tests/errors.rs` (new) — `ArithError::message` tests.
- `rust/crates/rexx-num/tests/settings.rs`, `tests/format.rs` — updated for
  the new variant shapes plus new message-text tests.

## Follow-up: arithmetic sub-messages

Once the division rewrite landed, `muldiv.rs`/`pow.rs`/`addsub.rs` came back
into scope, so `ArithError::Overflow` and `ArithError::NotWholeNumber` — the
two variants left generic above — were split into one variant per raise
site, each carrying its own confirmed text. `code()` still returns 42/26 for
every one of them; no error number changed. `format.rs`/`settings.rs` were
not touched.

### The map

All text below is verbatim from `build/bin/rexx`, and every case is now a
test in `tests/errors.rs`.

| Variant | Code | Raised from | Text | Substitution |
|---|---|---|---|---|
| `Overflow(String)` | 42.901 | `check_range` (over), shared by mul/div/add/sub/pow | `Arithmetic overflow; exponent ("&1") exceeds &2 digits.` | &1 = adjusted exponent, &2 = literal `9` |
| `Overflow(String)` | 42.902 | `check_range` (under), same sites | `Arithmetic underflow; exponent ("&1") exceeds &2 digits.` | &1 = **raw** exponent (not adjusted), &2 = literal `9` |
| `ZeroToNegativePower` | 42.903 | `pow.rs`, base zero + negative exponent | `Arithmetic underflow; zero raised to a negative power.` | none |
| `PowerOverflow(String)` | 42.001 | `pow.rs`, magnitude precheck | `Arithmetic overflow detected at:  "&1&2&3".` | &1 = base, &2 = `**`, &3 = exponent |
| `DivideByZero` | 42.003 | unchanged from the first pass | `Arithmetic overflow; divisor must not be zero.` | none |
| `IntegerDivideNotWhole` | 26.011 | `muldiv.rs`'s `div`, `%` | `Result of % operation did not result in a whole number.` | none |
| `RemainderNotWhole` | 26.012 | `muldiv.rs`'s `div`, `//` | `Result of // operation did not result in a whole number.` | none |
| `PowerExponentNotWhole(String)` | 26.008 | `pow.rs`, exponent not whole after rounding to DIGITS | `Operand to the right of the power operator (**) must be a whole number; found "&1".` | &1 = exponent |

Every case above has a distinct sub-message; nothing fell back to the bare
major-code text this time, **except** three `checked_add`/`checked_sub`
guards in `muldiv.rs` against i32 exponent overflow. Both operands' exponents
are already bounded to ±999,999,999 by `Number::parse`/`check_range`, so
their sum stays inside i32 — I believe these are unreachable in practice.
If one ever does fire there is no valid exponent left to substitute (that
would be the failure itself), so those three still report the bare 42.000
text, now for a much narrower and better-understood reason than before.

### One genuinely surprising find, twice

`&2` in 42.901/42.902 is **always the literal `9`** — `Numerics::
DEFAULT_DIGITS`, a compile-time C++ constant — never the active `NUMERIC
DIGITS`. Confirmed by provoking the same overflow at DIGITS 9 and DIGITS 15
and getting back identical text ("...exceeds 9 digits.") both times; I would
have shipped the current digits setting without the second probe.

`&1`/`&3` in 42.001, and `&1` in 26.008, substitute the base/exponent **as
originally written in the Rexx source**, not any canonical re-rendering:
`1e10 ** 200000000000` reports the base as `"1E10"` (no `+`), and
`123.456789012345678 ** 999999999` at DIGITS 15 reports the base at its full
18-digit precision, un-rounded. `Number::parse` discards that original
spelling immediately (Rexx itself only does this for literals that have
never been through arithmetic — `say 1e10` prints `1E10`, `say (1e10 + 0)`
prints `1E+10` — confirmed with a probe pair), and nothing in this task's
scope threads the source text through `pow()`. I could not make this
byte-exact. Best effort: render the operand at its own full stored precision
(`digits.len()` significant digits, via a small `full_precision` helper in
`pow.rs`) rather than this crate's usual 9-digit default — strictly closer,
still provably wrong for a leading zero, a bare `E` with no `+`, or a
different exponent-marker case. Documented on `ArithError::message` and
pinned as two tests that assert the current (imperfect) output, so a future
fix has something to diff against.

### An unrelated discovery, not acted on

Verifying this required, per your instructions, the four format sets plus
muldiv (17,424)/addsub (8,712)/pow (2,112) — all 0 diffs. I also noticed
`gen-curated-sets.py` now has a `fmtcarry` set (not on your required list)
that isn't in `tests/format.rs`'s currently-`#[ignore]`d state either. Out of
curiosity, since it clearly exists to test something specific, I ran it
read-only: 148/15,840 cases diverge (correcting an earlier miscount here of
460 — I had counted every line `diff` printed per case, not just the
oracle's `<` line; `diff oracle.txt mine.txt | grep -c '^<'` is the right
count and gives 148, matching the team lead's own measurement). The pattern
(`9.996E9` at DIGITS 9,
`after=0`, `expp=1` → interpreter `<E93>`, this crate `1E+10`) says the
`expp`-oversize *decision* needs the post-carry exponent, not the pre-carry
one my Task D fix used for it (only the *substitution text* wants pre-carry,
which is what I verified at the time — I did not have a case that separated
the two). I have not touched `format.rs` — you asked me not to while it's
under review, and this follow-up's scope was arithmetic only — but flagging
it now rather than leaving it a surprise.

### Test/clippy summary

`cargo test --offline --workspace`: all green, 0 failed (13 tests in
`tests/errors.rs`, up from 4). `cargo clippy --offline --workspace
--all-targets -- -D warnings`: clean. Differential sets: muldiv 17,424,
addsub 8,712, pow 2,112, and the four format sets — all 0 diffs.

## Fix round 1

Two items from review, both landed.

### Critical: the missing post-carry `expp` check

The unrelated discovery above was the bug, confirmed. The interpreter checks
`expp` **twice** (`NumberStringClass.cpp:2057` and again at `:2190`, the
second one a kludge the code there attributes to `[bugs:#1474]`), and my
earlier fix only kept the first. `format_with` now runs a new
`post_carry_exponent_error` right after the pre-carry check, gated the same
way (`if let Some(width) = expp`), so it fires whether or not the pre-carry
check ever triggered.

I did not take the substitution text on faith. Two things needed separate,
empirical confirmation, both against `build/bin/rexx`, not derived from the
C++ by inspection alone (a hand-trace of the C++ got the digit count wrong
on the first pass — see below):

- **The decision needs the post-carry exponent, not resolve's refinement of
  it.** `9.996E+99` at DIGITS 9, `after` 0, `expp` 2: pre-carry exponent 99
  (two digits) fits; rounding carries it to 100 (three digits), which
  doesn't. `9999999999.6` at DIGITS 15, `expt` 10, `after` 0, `expp` 1 is
  the sharper case the reviewer flagged: the pre-carry state isn't even
  exponential (adjusted exponent 9 < `expt` 10), so my old code's guard
  (`if let (Some(exp0), Some(width))`) skipped the check entirely; only the
  carry (adjusted exponent → 10) triggers it, and only the second check
  catches that.
- **The substitution keeps mid-computation trailing zeros.** For the second
  case, the message is `Exponent of "1.000000000" is too large for 1
  spaces.` -- 9 zeros, not resolve's trimmed `"1"`. Traced to
  `NumberStringMath.cpp:315`'s `mathRound`: on a full carry it sets the
  leading digit to 1 and does `numberExponent += 1`, holding the digit
  *count* fixed -- exactly `Number::round_to`'s carry, not
  `round_to_places`'s (which grows the digit vector instead). Both land on
  the same *value*, which is why the successful-render path was never
  affected and 21,296 pre-existing cases never caught this; they disagree
  on digit count, which only this substitution text exposes. The new
  `post_carry_exponent_error` reuses `Number::round_to` (not
  `round_to_places`) for exactly this reason -- see its doc comment for the
  full derivation, including the "back to true scale" step needed because
  `round_to`'s carry lands in mantissa-scale, not the number's own.

Rewrote the doc comment at the old `format.rs:129` (now above the
restructured check) -- it asserted the C++ has no post-carry notion at all,
which was the bug, not a fact.

Regression corpus: `fmtcarry` (15,840 cases) — 148 divergences before,
**0 after**. Did not touch the generator. Also re-ran everything the fix
could plausibly have affected: fmt 1,800, fmt2 6,720, fmt3 12,136, fmtedge
640, muldiv 17,424, addsub 8,712, pow 2,112 — all still 0. Added two new
tests to `tests/format.rs` pinning both confirmed scenarios (`tests/format.
rs`'s existing pre-carry test, `exponent_oversize_message_uses_the_pre_
carry_exponent_not_the_final_one`, still passes unmodified -- that one is
caught by the *first* check alone and never reaches the second).

### Minor: `substitute`'s sequential replace

`error_text`'s `substitute` called `str::replace` once per placeholder,
which re-scans the whole string each time -- a substitution value
containing literal `&2` text would get rewritten again by the next
replacement. Rewrote as a single left-to-right pass over `text` (`lib.rs`),
copying each `subs[n-1]` in without revisiting it. Added
`#[cfg(test)] mod substitute_tests` in `lib.rs` (it's `pub(crate)`, so an
integration test can't reach it) with the exact collision case from review
(`substitute("… &1 … &2", &["&2", "X"])` → `"… &2 … X"`) plus a
malformed/unsupplied-placeholder passthrough case.

Did not act on the review's "carry values, render on demand" suggestion for
`FormatError`/`SettingsError` -- you said that one's still open.

### Test/clippy summary

`cargo test --offline --workspace`: all green, 0 failed (3 new tests in
`format.rs`, 3 new unit tests in `lib.rs`). `cargo clippy --offline
--workspace --all-targets -- -D warnings`: clean. `fmtcarry`: 0/15,840.
All other required sets unchanged at 0.

## Error shape reshape

Every error variant now carries its substitution *values*, typed naturally,
instead of pre-rendered `String` text. `message()` renders from the
generated table on demand; a new `additional()` returns the same values in
the interpreter's own order -- what `condition('o')~additional` would
return. Both are `&self`, non-consuming (`code()` stays by-value on
`ArithError`/`FormatError` specifically because off-limits files call it as
`Type::code(e)` -- `bin/muldiv.rs`, `bin/addsub.rs`, `bin/fmt-check.rs`).

### Shape chosen: typed fields, split further where the code number alone
### didn't distinguish the sub-message

Went with typed fields (`i32`, `u32`, `Number`, `String` only where the
value is genuinely textual) over a uniform `Vec<String>`, and split three
variants further than Task D left them, because a single field per variant
is what makes `additional()`'s order self-evident from the type instead of
needing a separate ordering convention to get right and keep right:

- `ArithError::Overflow(String)` → `Overflow { adjusted_exponent: i32 }`
  (42.901) and `Underflow { exponent: i32 }` (42.902) -- two sub-messages
  sharing the old variant, now two variants; `check_range` (`lib.rs`) picks
  between them at the one place that still has the out-of-range `Number` to
  ask, same as before, just handing over a field instead of rendered text.
  `PowerOverflow(String)` → `{ base: Number, exponent: Number }`.
  `PowerExponentNotWhole(String)` → `{ exponent: Number }`.
- `SettingsError::NotWholeNumber(String)` → `DigitsNotWhole { found:
  String }` (26.005) / `FuzzNotWhole { found: String }` (26.006), for the
  same reason -- one old variant, two sub-messages, now two variants.
  `FuzzNotBelowDigits(String)` → `{ digits: u32, fuzz: u32 }`.
  `InvalidForm(String)` → `{ found: String }`.
- `FormatError::BeforeOversize(String)` → `{ value: Number, digits: u32,
  before: u32 }` (renders `value.format(digits)` on demand -- exactly what
  was pre-rendered before). `ExponentOversize(String)` → `{ mantissa:
  Number, width: u32 }` (renders via the same plain, no-`before`/`after`
  `render_integer_padded` call both raise sites already used).

The three no-substitution variants that only ever needed a code number
(`DivideByZero`, `IntegerDivideNotWhole`, `RemainderNotWhole`,
`ZeroToNegativePower`) stayed unit variants -- `additional()` for those is
`vec![]`, matching the empty-but-present array `condition('o')~additional`
returns for a message with no placeholders. The old generic
`ArithError::Overflow` fallback for the three believed-unreachable
`checked_add`/`checked_sub` guards in `muldiv.rs` became its own unit
variant, `ExponentComputationOverflow` (still bare 42.000, still
undocumented as ever having fired) -- it could not stay named `Overflow`
once that name meant "carries an `adjusted_exponent: i32`" specifically.

None of this is `Copy` (`Number` holds a `Vec<u8>`), but every variant's own
field is the plainest type that value has -- `i32`/`u32` where the
interpreter's substitution is a bare number, `Number` where it is an
operand, `String` only for `SettingsError`'s `found` (genuinely arbitrary
caller-supplied text, not further parseable) and `FormatError`'s rendered
mantissa string is *not* stored -- the `Number` is, and `additional()`
renders it.

### `additional()`'s order

Declaration order in the source XML / the `<Sub position="N"/>` positions
`rexx-inventory` already parses: index 0 is `&1`, index 1 is `&2`, and so
on -- the same order `message()`'s `error_text` substitutes them in, since
both come from the same `additional()` call now (`message` is `error_text
(major, sub, &additional().iter().map(String::as_str).collect::<Vec<_>>())`
in each of the three types). A test in each of `tests/errors.rs`,
`tests/settings.rs`, `tests/format.rs`
(`additional_and_message_agree_on_every_placeholder`) asserts every value
`additional()` returns actually appears in `message()`'s rendered text, so
the two cannot silently drift apart.

### Verification

Re-provoked 17 cases fresh against `build/bin/rexx` after the reshape (not
reused from earlier sessions) and diffed by hand against a throwaway binary
printing this branch's live `additional()`/`message()` output side by side
-- all 17 matched byte-for-byte on the first correctly-written probe (one
probe of mine had an ordering bug -- referenced an uninitialized Rexx
variable -- caught and fixed before drawing any conclusion from it, noted
here rather than silently corrected). All 8 required sets at 0: fmt 1,800,
fmt2 6,720, fmt3 12,136, fmtedge 640, fmtcarry 15,840, muldiv 17,424, addsub
8,712, pow 2,112.

### Test/clippy summary

`cargo test --offline --workspace`: all green, 0 failed, 166 passed (up
from 153 -- new `additional()` coverage in all three test files, plus
`code_still_matches_every_variant_after_the_split`-style regression tests).
`cargo clippy --offline --workspace --all-targets -- -D warnings`: clean.

### Concerns

- `PowerOverflow`/`PowerExponentNotWhole`'s `additional()` still can't be
  exact (the operand-spelling gap, unchanged, still documented on
  `ArithError::message`) -- worth being explicit that this is now a gap in
  the *value* a Rexx program would read via `~additional`, not only in
  displayed text.
- I did not touch `addsub.rs`: it never constructs `ArithError` directly
  (only calls `.check_range()`), so nothing there needed reshaping.
