# Task C — Formatting (Phase 2, Task 2.7)

Status: DONE. `ENGINEERING` form, `FORMAT`, and `TRUNC` implemented in
`rust/crates/rexx-num/src/format.rs`, verified with ~90 probe programs run
under `build/bin/rexx` 5.3.0 and pinned as 31 new unit tests.

## What the interpreter does

### ENGINEERING form

Only the *grouping* of an already-exponential number changes between forms —
whether a number goes exponential at all is decided identically in both
(confirmed by sweeping the plain/exponential boundary literal-for-literal
under both `NUMERIC FORM` settings: same boundary, both directions).

Once exponential, ENGINEERING's exponent is the largest multiple of 3 not
exceeding the number's adjusted exponent — floor division, not truncation:
`12e-25` (adjusted -24) groups to `-24`, but `12e-28` (adjusted -27) groups to
`-27`, and `1e-25` (adjusted -25) groups to `-27`, not `-24`. Found by
sweeping literal `1eK`/`12eK` for `K` in `-30..=30` and reading off exactly
where the mantissa's integer-digit count (1, 2, or 3) changes. When the
stored digit count is shorter than the mantissa needs, the shortfall is
**appended as zero digits**, never shown as a decimal: `1e10` is `10E+9`
(engineering), not `1.0E+9` scientific-style or `1E+10`.

### `FORMAT(number, before, after, expp, expt)`

- Always rounds to the **current `NUMERIC DIGITS`** first, regardless of
  `expt` — a 15-digit literal at `DIGITS 9` behaves as if it had been rounded
  to 9 digits before any of `before`/`after`/`expt` are applied.
- `before` constrains the **integer part's width** (the mantissa's, in
  exponential form) — pads with leading spaces, or errors if too narrow.
  Zero itself needs 1 slot, so `before 0` always errors, even for `format(0,
  0)`. A negative number needs one more slot for the sign; the error message
  substitutes the *requested* `before`, not the space actually available
  (`format(-123.456, 3)` reports "too large for 3 spaces", not 2) — cosmetic
  detail I did not attempt to reproduce since this crate carries error codes,
  not message text.
- `after` rounds (half up, not truncates) or zero-pads the decimal part to
  exactly that many digits. `after 0` means no decimal point at all.
- `expt` (default: current `DIGITS`) replaces `digits` in the *exact same*
  two-sided trigger `Number::format` already uses: exponential once the
  adjusted exponent `>= expt`, or once the adjusted exponent is negative
  *and* the raw exponent's magnitude exceeds `2 * expt`. The `adjusted < 0`
  guard is not redundant here the way it is for `Number::format`: with an
  explicit `expt` smaller than `DIGITS`, a value can have many more
  significant digits than `2*expt+1`, so the raw-exponent test alone would
  misfire (`9.996996` with `expt 1` stays plain, confirmed, even though its
  raw exponent -6 would trip an unguarded low-end check).
  `expt == 0` forces exponential for any nonzero number (the trigger becomes
  `adjusted >= 0`, always true).
- `expp` (default: unconstrained) pads the exponent's digits with **leading
  zeros** to that width, or errors if too narrow. `expp == 0` is not "zero
  padding" — it forces **plain form unconditionally**, skipping the
  exponential decision entirely (`format(1e100, , , 0)` prints all 101
  digits). Confirmed `expp` wins when it conflicts with `expt`
  (`format(123, , , 0, 0)` → `"123"`, not exponential). The exponent-oversize
  check runs *before* the before-oversize check (confirmed with a case where
  only the exponent check could plausibly fail).
- **The interesting part**: rounding a decimal to `after` places can carry,
  and the carry can push the integer-digit count past what the *original*
  exponent choice assumed. `9.996E+20` rounded to 0 decimals is `1E+21`, not
  `10E+20` — SCIENTIFIC tolerates only one mantissa integer digit, so the
  carry must move the exponent. `99.996E+20` the same way is `10E+21`, not
  `100E+20` — the carry stays inside ENGINEERING's 1-3 digit budget so the
  exponent doesn't move. A number that started **plain** can cross into
  exponential the same way: at `DIGITS 9`/`expt 3`, `999.9996` (adjusted
  exponent 2) prints plain, but rounded to 0 decimals it carries to `1000`
  (adjusted exponent 3, clearing the trigger) and must render `1E+3`. All
  three confirmed against the interpreter; see `format.rs`'s doc comment on
  `resolve_exponential_state` for why patching the already-rounded mantissa
  in place gets this wrong and a from-scratch re-derivation is needed.

Error numbers: both the before-oversize and exponent-oversize conditions are
RC **93** (`Incorrect call to method`), condition codes 93.942 and 93.941
respectively (found via `condition('O')~code` after `signal on syntax`).
Malformed-argument errors (non-whole-number, negative, wrong arg count,
non-numeric target) are RC 93/40 but out of this crate's scope — see Notes.

### `TRUNC(number, places)`

Rounds to current `DIGITS` first (same as `FORMAT`), then truncates —
drops, never carries — to exactly `places` decimal digits, zero-padding if
`places` exceeds the existing decimal count. `places` defaults to 0. Unlike
`FORMAT`, **never** produces exponential form (`TRUNC(1e10)` is the
eleven-digit integer; `TRUNC(1e-10)` is `0`). A result that truncates to
exactly zero magnitude drops its sign (`TRUNC(-0.5, 0)` is `"0"`, not
`"-0"`).

## What I implemented

`rust/crates/rexx-num/src/format.rs` (new):
- `FormatError` — `BeforeOversize` / `ExponentOversize`, both `code() == 93`,
  matching `ArithError`'s established pattern of exposing the bare RC and
  letting the variant itself carry the finer distinction.
- `Number::format_form(digits, form)` — the form-aware sibling to
  `Number::format`, implemented as `format_with` with every optional argument
  `None` (proved equivalent by construction and cross-checked against
  `Number::format` itself for SCIENTIFIC across a dozen magnitudes/digits
  settings).
- `Number::format_with(digits, form, before, after, expp, expt) ->
  Result<String, FormatError>` — the `FORMAT` builtin.
- `Number::trunc(digits, places) -> String` — the `TRUNC` builtin.
- Private helpers: `group` (the floor-to-multiple-of-3 rule), `reframe`
  (rescales a number to a chosen exponent), `resolve_exponential_state` (the
  carry-re-derivation loop described above), `round_to_places`/
  `truncate_to_places` (decimal-place cuts — deliberately not built on
  `Number::round_to`, whose `digits == 0` is a documented no-op sentinel,
  whereas `places == 0` is an ordinary case here that must still round `0.6`
  up to `1`), `render_integer_padded` (the shared sign/pad/oversize-check
  renderer used for both plain numbers and exponential mantissas).

`rust/crates/rexx-num/src/lib.rs`: added `mod format;` and
`pub use format::FormatError;` only, as scoped. I did **not** need to change
`adjusted_exponent`'s visibility or touch `Number`'s existing methods —
`digits`/`exponent`/`negative` are already `pub(crate)`, so `format.rs`
recomputes the one-line adjusted-exponent formula itself rather than reaching
into `lib.rs`.

`rust/crates/rexx-num/tests/format.rs` (new): 31 tests, organized around the
findings above (defaults, before/after independently and combined, the two
oversize errors and their ordering, `expt`'s two-sided trigger, `expp`'s
padding and its `0` special case and its priority over `expt == 0`, the three
carry-re-derivation scenarios, and TRUNC's truncate-vs-round/no-exponential/
sign-dropping/pre-rounds-to-DIGITS behaviours).

## Verification

All of the above was established with ~15 ad hoc Rexx probe scripts under
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/`
(`format1.rex` through `format10.rex`, `eng_sweep.rex`, `eng_lit.rex`,
`carry1.rex`-`carry3.rex`, `order1.rex`-`order2.rex`, `both_zero.rex`, plus
error-probing scripts using `signal on syntax` + `condition('O')~code`),
covering:
- `FORMAT` with 0-5 arguments, all combinations of omitted vs. supplied.
- Negative numbers in every position (`before`'s sign slot, mantissa sign).
- Rounding requiring carry, including carries that cross a `before` boundary
  or move the exponent itself.
- `before` too narrow (error 93.942) at every combination with sign/zero/
  exponential form; the exact boundary (`before == neededIntegers` succeeds,
  `- 1` fails).
- `expp`/`expt` independently and combined, including their `0` special
  cases and which one wins when both are `0`.
- `TRUNC` with/without `places`, on negatives, on values needing DIGITS
  rounding first, and on values that vanish to zero.
- `ENGINEERING` across `-30..=30` orders of magnitude via literal sweeps
  (avoiding arithmetic, whose own rounding artifacts otherwise obscure which
  digit-count effects belong to grouping vs. to the operand).

Two of my initial test expectations were themselves wrong (copy-paste from
the wrong probe transcript, not implementation bugs) and were caught by the
test run, then corrected against a fresh interpreter check: one engineering
case I never actually verified (assumed exponential where the interpreter
stays plain) and one exponent value confused between a 12-digit and
15-digit example.

## Test / clippy output

- `cargo test --offline --workspace`: all green. `rexx-num`: `addsub` 7,
  `compare` 10, `format` **31 (new)**, `muldiv` 9, `parse` 7, `pow` 7,
  `settings` 6 — 77 integration tests, no regressions to the pre-existing 46.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: clean,
  zero diagnostics.
- No `unsafe`; `#![forbid(unsafe_code)]` untouched. Only files touched:
  `format.rs` (new), `tests/format.rs` (new), `lib.rs` (two lines: `mod
  format;` + re-export).

## Notes / things worth flagging

- **Argument validation is out of scope by design, and I want that
  confirmed.** `FORMAT`/`TRUNC` called with a negative/non-whole `before`,
  `after`, `expp`, `expt`, or `places`, a non-numeric target, or the wrong
  argument count all raise errors in the real interpreter (RC 93 "must be
  zero or a positive whole number", RC 40 "must be a whole number" / "Not
  enough arguments", RC 93 "target must be a number") — I probed these to
  map the boundary but did not implement them. This crate's functions take
  `Option<u32>`, which cannot represent "the caller passed something
  invalid" at all; validating raw Rexx argument text is the interpreter
  layer's job, consistent with how `mul`/`div`/`pow` take plain `u32 digits`
  and leave `NUMERIC DIGITS`'s own validation to `settings.rs`. Flagging in
  case whichever task wires this up expects those errors to originate here.
- `resolve_exponential_state`'s convergence loop is capped at 8 iterations
  as a defensive bound, not a proven tight one. I verified by construction
  that growth is monotonic (rounding never shrinks a magnitude) and, for
  every carry-chain scenario I could produce against the real interpreter,
  it converges in 1-2 passes. I did not find or construct a case needing a
  third pass, and DIGITS/expt being bounded by `MAX_EXPONENT` makes a
  genuinely unbounded cascade implausible, but I flag the cap as an
  engineering choice rather than something I proved necessary.
- The error message text (e.g. `Integer part of "100.0" is too large for 2
  spaces` — note only 1 decimal shown though `after` was 2) appears to use a
  different, secondary rounding just for display; I did not chase this down
  since this crate doesn't carry message text at all, only `FormatError`'s
  bare code, matching `ArithError`/`SettingsError`'s existing convention.

## Fix round 1

Team lead ran an independently-generated differential set (1,116 and 7,080
cases, built and oracle-captured before this task started) against
`fmt-check` and found 88/1,116 and 343/7,080 divergences across three
findings. All three are now fixed; **both sets are at 0 divergences**, and
all named regression sets (`addsub`, `muldiv`, `pow`, `rand1`, `p21`, `p22`
via `muldiv` against their oracles) are still at 0.

**Finding 1 and 2 turned out to be the same missing mechanism, not two.**
Before touching code I re-verified both against the real interpreter rather
than trust either the original report or my own prior conclusion (per
`receiving-code-review`) — my original doc comment said `expt == 0` "forces
exponential form for any nonzero number." That was an overgeneralization
from cases (`123`, `123456`) whose adjusted exponent already happened to be
positive; a fresh probe (`format(3.14159,,,,0)` at DIGITS 5, adjusted
exponent exactly 0) showed plain `3.1416`, contradicting it. Rather than
curve-fit a new formula to more black-box probes — the method note that
caused all three findings — I had an agent pull the exact C++ from
`NumberString::formatInternal` (`NumberStringClass.cpp:2018-2029,2158,
2324-2371`), confirmed live with gdb. The trigger comparison itself
(`adjustedLength >= exptrigger`) is exactly what I already had, uniformly,
no special case. What's special is downstream: a **displayed exponent of
exactly 0 is never written as `E+0`** — the exponential path still runs
(`before`/`after` still apply to the mantissa), but the suffix is silently
dropped, or, if `expp` was given explicitly, replaced with `expp + 2` blank
spaces reserving the field it would have taken. `expt == 0` makes this easy
to hit (`adjusted == 0`), but ENGINEERING's floor-to-multiple-of-3 grouping
can *also* collapse a nonzero adjusted exponent (1 or 2) down to a displayed
0, and the same suppression applies there too — confirmed independently
(`format(99,,,,1)` under ENGINEERING at DIGITS 5 prints `99`, not `9.9E+1`,
where SCIENTIFIC with the same arguments does show `9.9E+1`). Implemented as
an `exp == 0` branch in `format_with` right where the `E{sign}{digits}`
suffix used to be unconditional.

One case this missed on the first pass, caught only by re-running the
harness rather than by hand: **zero itself** still needs to flow through
this same machinery. `format(0,,,,0)` still fires the trigger (zero's
adjusted exponent is always exactly 0, and `0 >= 0`), so with `expp` given
it still needs the blank-padding (`format(0,,,,2,0)` is `"0    "`, not
`"0"`) even though zero, by construction, never shows non-suppressed
exponential digits. My original code special-cased zero to skip the whole
exponential/`expt`/`expp` decision — right for the *digits*, wrong for the
padding. Removed the special case entirely; zero now goes through the exact
same trigger → `resolve_exponential_state` → `exp == 0` path as everything
else, which resolves to `Some(0)` or `None` correctly on its own.

**Finding 3** was a real, distinct bug: `round_to_places`/`truncate_to_places`
routed an underflowed result (a nonzero number small enough that every
stored digit gets rounded or dropped past the requested `places`) through
`Number::assemble`/`Number::zero()`, both of which collapse an all-zero
digit vector to the *canonical* zero (`exponent == 0`), discarding the
`target_exponent` that was supposed to become the decimal-place count.
`TRUNC(0.000012345, 1)` produced `"0"` instead of `"0.0"`. Two separate call
sites needed the fix, not one: the early `drop > len` return in
`round_to_places` (which I *had* already special-cased in the original
implementation, but only that one path), and — found only by re-running the
harness after the first fix, not by reasoning about it — the general
`drop <= len` path's *final* `Number::assemble(n.negative, kept, ...)` call,
which hits the exact same collapse whenever `keep == 0` and no rounding
carry occurs to make `kept` non-empty (`0.001` rounded to 2 places: `keep`
is exactly 0, the dropped digit is `1` (no carry), `kept` stays `[]`, and
`assemble` throws away `target_exponent`). Fixed both by constructing
`Number { negative: false, digits: vec![0], exponent: target_exponent }`
directly instead of going through `assemble` whenever the kept digits end
up empty; `truncate_to_places`, which has no carry logic, only needed the
one change (`drop >= len`, not `drop > len`, since there's no carry path to
preserve). Sign is dropped in both, consistent with the already-verified
`TRUNC(-0.5, 0) == "0"` behaviour.

Added 6 new tests pinning all of the above
(`expt_zero_plugs_into_the_ordinary_trigger_like_any_other_value`,
`displayed_exponent_of_zero_is_never_written_as_e_plus_zero`,
`engineering_grouping_can_also_produce_a_displayed_exponent_of_zero`,
`zero_still_triggers_the_displayed_exponent_zero_padding_when_expt_is_zero`,
`after_rounding_underflow_still_shows_the_requested_decimal_places` (extended),
`trunc_underflow_still_shows_the_requested_decimal_places`), for 36 total in
`tests/format.rs`, all passing. `cargo test --offline --workspace` and
`cargo clippy --offline --workspace --all-targets -- -D warnings` both clean.
No files touched outside `format.rs`/`tests/format.rs` (`lib.rs` untouched
this round); did not touch `src/bin/fmt-check.rs`.

## Fix round 2

Review passed spec compliance (the ENGINEERING/`expt`/`expp` mechanism
claims from round 1 were checked against the C++, not just against the case
sets) but found two Important defects, both `i32`-overflow bugs at the
`u32::MAX` end of `places`/`before`, neither covered by the "argument
validation is out of scope" note in round 1 — both inputs are positive whole
numbers, which is exactly the range this crate does own. Both fixed. All
four differential sets (`fmt_check` 1116, `fmt_check2` 7080, `fmt3` 12136,
`edge5` 640) at 0, freshly rebuilt and re-run (confirmed the binary actually
recompiled by `touch`-ing the source first, after round 1's back-and-forth
about stale binaries). All six regression sets (`addsub`, `muldiv`, `pow`,
`rand1`, `p21`, `p22`) at 0. `cargo test --offline --workspace` and `cargo
clippy --offline --workspace --all-targets -- -D warnings` both clean.

**Finding 1** (`format.rs`, `round_to_places`/`truncate_to_places`):
`-(places as i32)` overflows once `places >= 2^31` — the interpreter accepts
`TRUNC(1, 2147483648)` and returns the ~2.1-billion-character result
(`length(...) == 2147483650`, verified against `build/bin/rexx` before
touching anything), but the cast panics in debug ("attempt to negate with
overflow") and wraps to a nonsense negative exponent in release, which then
died as a `capacity overflow` trying to allocate a `Vec` sized from that
garbage value.

**Finding 2** (`render_integer_padded`'s `before` handling): `before as i32`
wraps negative once `before >= 2^31`, making `available` spuriously
negative and raising `BeforeOversize` for an input the interpreter accepts
(`length(format(1, 3000000000)) == 3000000000`, also verified directly).

The reviewer flagged, correctly, that clamping at `MAX_EXPONENT` would be
the wrong fix — both accepted values are already past `999999999`, so
clamping replaces a panic with a silently wrong answer. The right fix
turned out to need more than widening the arithmetic type, because of a
constraint the clamp-vs-widen framing didn't surface: `Number::exponent` is
`i32` (an invariant of the `Number` type in `lib.rs`, out of this task's
scope to change, and correctly so — every other module assumes it), and
`places`/`after` up to `u32::MAX` genuinely cannot always be encoded as an
exponent on that type, widened or not. `round_to_places`/`truncate_to_places`
used to fold the requested decimal-place count directly into the returned
`Number`'s exponent (via `pad_to_exponent`) whenever the number needed
*more* trailing zero decimals than it already had; for a `places` near
`u32::MAX` that would require an exponent like `-3_000_000_000`, which does
not fit in `i32` at all, regardless of the type used to compute it.

Fixed by moving that responsibility: `round_to_places`/`truncate_to_places`
now return the number **unchanged** whenever it already has at most
`places` decimal digits (deciding this with `i64` arithmetic up front, so
the decision itself can't overflow), and the actual zero-padding is done
later, in `render_integer_padded`, directly against the original `u32`
`places`/`after` value in `u64`/`usize` arithmetic — never through
`Number::exponent` at all. `render_integer_padded` gained an `after:
Option<u32>` parameter for this (previously the padding arrived pre-applied
inside the `Number` it was handed); its `before` handling was separately
widened from `i32` to `i64` for finding 2, which needed no architectural
change since `before` was never folded into a `Number` in the first place —
only the space-padding count, which was always computed independently of
`Number::exponent`.

The rounding/truncating logic itself (the part that decides whether to
carry, examining `n`'s own bounded digit vector) is unchanged and still
uses `i32` — proven safe rather than assumed: that branch is only reachable
once `target_exponent > n.exponent`, and since `n.exponent` is always
within `+/-MAX_EXPONENT` (comfortably inside `i32`) while `target_exponent`
is never positive (`places` is never negative), a `target_exponent` large
enough to threaten `i32` can never actually reach that branch — it always
takes the new unchanged-return path instead. This is argued in a comment at
each of the two call sites rather than merely asserted, since a future
change to either bound is exactly the kind of thing that could quietly
invalidate the argument. `pad_to_exponent` is now unused and removed.

**`expp` needed no change.** Checked as asked rather than assumed: grepping
every `as` cast touching `expp`/`width` in `format.rs` shows only `as
usize`/`as u32`, never `as i32` — it was already correct at the full `u32`
range. Added `expp_already_used_no_i32_cast_to_begin_with` as a cheap,
non-gigabyte confirmation (a moderately wide `expp` still pads correctly);
did not attempt to exercise `expp` near `u32::MAX`, since that would demand
allocating a padding field that wide for no additional bug-catching value —
there being no cast to overflow in the first place is the actual evidence.

Per the reviewer's instruction, the three new overflow tests
(`trunc_accepts_places_at_and_past_the_i32_negation_boundary`,
`format_before_survives_the_full_u32_range`,
`format_after_survives_the_full_u32_range`) live in `tests/format.rs` as
unit tests asserting `.len()` only (never comparing or printing the full
multi-gigabyte string, and never writing one to disk); these magnitudes
were not added to any differential case-file set. Verified standalone
before adding them to the suite, in both debug and release profiles, via a
throwaway `/tmp` binary depending on the crate by path (not committed,
already removed) — confirms no panic in either profile and the exact
lengths the interpreter reports. 40 tests total in `tests/format.rs`, all
passing (includes the 3 new plus `expp_already_used_no_i32_cast_to_begin_with`).

No files touched outside `format.rs`/`tests/format.rs`; `lib.rs` untouched
this round; did not touch `src/bin/fmt-check.rs`.
