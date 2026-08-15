STATUS: DONE

# Task 8a report: a byte-slice comparison entry point for rexx-num

Scope: `rexx-num` only, adding entry points that call the existing
`string_order` rather than duplicating it, so `rexx-exec`'s Task 8
(comparison operators) can call a real comparison without going through
`&str` (which cannot carry a non-UTF-8 Rexx string, D14) or re-parsing an
already-decoded `Number` on every call.

Filling this in as I go, per the project's write-report-first discipline.

## What's there already, read before writing anything

`src/compare.rs` (217 lines, pre-existing) already has the whole rule:
`compare(a: &str, b: &str, digits, fuzz, op) -> Result<bool, ArithError>`,
`numeric_order` (the numeric family, ported from `NumberString::comp`), and
`string_order` (the string fallback, ported from `RexxString::stringComp`).
`string_order`'s own body already operates entirely on `&[u8]` internally
(`a.as_bytes()` is the very first thing it does to each operand) --
`&str` was only ever the *public* constraint, not something the algorithm
itself needed. That is what makes this task a signature change plus two
new entry points, not a rewrite.

`compare`'s only current caller outside this crate's own tests is
`rexx-parse/src/expr/differential.rs` (the Task 3 differential harness,
`#[cfg(test)]` only), confirmed by grep across every crate. It must keep
working unchanged, per the brief.

## Design

Three public functions, all funnelling into one `compare_decoded`, so
there is exactly one place `numeric_order`/`string_order` are called from
regardless of entry point:

* `compare(a: &str, b: &str, ...)` -- unchanged signature, now a one-line
  call into `compare_decoded(a.as_bytes(), None, b.as_bytes(), None, ...)`.
  Every existing test in `tests/compare.rs` (10 tests) passes unmodified
  against this, which is what proves the refactor is behaviour-preserving,
  not just plausible.
* `compare_bytes(a: &[u8], b: &[u8], ...)` -- the byte-slice entry point
  the brief asks for. Also a one-line call into `compare_decoded` with
  nothing pre-parsed.
* `compare_decoded(a: &[u8], a_number: Option<&Number>, b: &[u8], b_number: Option<&Number>, ...)`
  -- the "already-decoded operand" entry point, and where the actual rule
  lives now. `None` means "parse `bytes` if a numeric operator needs a
  value", which is what the other two pass for every operand. `bytes` is
  required even when `number` is `Some`, because the strict family and the
  non-numeric string fallback compare an operand's own *text*, not a value
  derived from it -- `"007"` and `"7"` share a `Number` but must not
  strict-compare equal, and a `Number` alone cannot answer that question.

Chose `Option<&Number>` rather than a three-state "unknown / numeric /
confirmed-not-numeric" type: the brief's stated need is "a caller holding
a parsed `Number` need not throw it away", which `Option<&Number>` answers
exactly, and inventing a third state to also skip re-*attempting* a parse
of a confirmed-non-numeric string is a different, smaller optimisation the
brief did not ask for -- re-parsing a string that turns out not to be a
number costs a failed scan, not a wrong answer, and `rexx-core::NotNumeric`
is not reachable from `rexx-num` to represent that state anyway (the
dependency runs the other way: `rexx-core` depends on `rexx-num`, per D15,
not back).

`compare_decoded` avoids cloning a caller-supplied `Number` to satisfy the
borrow checker: rather than materialising `Option<Number>` up front (which
would clone a `Some(&Number)` for no reason -- exactly the case this entry
point exists to make cheap), a local `Option<Number>` per side stays `None`
and is only filled by a fresh parse when the caller's own value was
`None`, with the final `Option<&Number>` borrowing from either the
caller's reference or that local, whichever applies. When the caller
already has a `Number`, nothing is cloned and nothing is parsed.

`string_order` retyped from `&str` to `&[u8]` -- a pure signature change,
confirmed by the diff: every line inside it already worked on byte slices
(`skip_leading_blanks(&[u8]) -> &[u8]`, `tail_order(&[u8], bool)`), so
dropping the two `.as_bytes()` calls at the top is the entire change. No
behaviour differs, and the C++ this is ported from (`StringClass.cpp:795`)
never assumed UTF-8 either -- it compares raw operand bytes, so a `&[u8]`
signature is truer to what `stringComp` actually is than `&str` ever was.

The one new piece of logic: `parse_bytes(bytes: &[u8]) -> Option<Number>`,
`std::str::from_utf8(bytes).ok().and_then(Number::parse)`. A Rexx number's
characters are ASCII by definition (the same fact `rexx-core`'s
`NotNumeric` doc comment states, for the same reason), so invalid UTF-8
can never be a number and is treated as a parse failure exactly like
malformed ASCII text already is -- no third outcome to invent, and no
change to `Number::parse`'s own signature (which stays `&str`; a general
byte-slice parse entry point is out of this task's scope, per the brief,
and is a different, larger surface than one comparison function needs).

## Oracle verification

All nine transcripts from the brief, run together wrapped
(`( ulimit -v 1048576; build/bin/rexx FILE )`):

```
' a' = 'a'       -> 1
'09'x'a' = 'a'   -> 1
'a' = 'a'||'09'x -> 1
'a' = 'a '       -> 1
'a b' = 'a  b'   -> 0
'' = ' '         -> 1
'01' = '1'       -> 1
' 1 ' = 1        -> 1
'a' = 1          -> 0
```

All nine match the brief exactly. Traced *why* the first row is the
discriminating one rather than taking that on faith: the wrong rule the
brief names ("blank-pad the shorter operand on the right") would take
`'a'` (1 byte) and `' a'` (2 bytes), pad `'a'` on the right to `'a '`, and
compare that against `' a'` -- first bytes `'a'` (0x61) and `' '` (0x20)
differ, so the wrong rule answers `0` where the oracle answers `1`. The
real rule (strip *leading* blanks, implemented in `string_order` already)
reduces both operands to `"a"` before comparing, giving `1`. None of the
other eight rows can tell the two rules apart, because none of them has a
leading blank on the *shorter* side specifically -- confirmed by checking
each: rows 2-3 test leading/trailing tabs (not the pad-vs-strip
distinction, since both rules agree when the shorter side has no leading
blank at all), rows 4-6 test trailing padding and interior blanks, rows
7-8 are numeric (converted before any blank rule applies), row 9 falls
back to string comparison but with no blanks on either side to strip or
pad.

Added a genuinely non-UTF-8 case beyond the brief's nine, since that is
specifically the gap `compare`'s `&str` signature cannot express and the
whole reason `compare_bytes` exists: `'C3'x` alone (a UTF-8 continuation
lead byte with nothing after it, invalid on its own). Verified against the
oracle:

```
a = 'C3'x ; b = 'C3'x ; c = 'C4'x
say (a = b)                    -> 1
say (a = c)                    -> 0
d = '20C3'x                    /* a real leading blank (0x20) then 0xC3 */
say (d = a)                    -> 1   (leading blank stripped, as usual)
say (d == a)                   -> 0   (strict never strips)
```

Confirms the interpreter's own rule has no UTF-8 assumption anywhere in
it -- exactly what makes a byte-oriented `string_order` the right port,
not an approximation of a character-oriented one.

## Tests

`rust/crates/rexx-num/tests/compare.rs`, seven new tests alongside the ten
pre-existing ones (all ten still pass unmodified):

* `the_nine_transcripts_that_pin_leading_not_padded_blank_stripping` --
  all nine oracle rows, through `compare_bytes`, with the pad-vs-strip
  reasoning above in the doc comment so a future reader does not have to
  re-derive why the first row is the one that matters.
* `a_non_utf8_operand_compares_correctly_where_str_could_not_express_it`
  -- the `'C3'x` case, both non-strict and strict, plus the leading-blank
  variant.
* `compare_and_compare_bytes_and_compare_decoded_agree_on_every_case_above`
  -- checks computationally, not just by construction, that all three
  entry points answer identically for the same operands: a shared bug in
  `compare_decoded` would still make this pass if the *other* two
  functions had their own copies of the logic, so this is the test that
  would catch a `compare`/`compare_bytes` still routing around
  `compare_decoded` some other way.
* `compare_decoded_uses_the_supplied_number_not_a_fresh_parse_of_the_bytes`
  -- defeat-the-mechanism style: supplies a `Number` that *disagrees* with
  what the accompanying bytes would parse to (`b"999"` paired with a
  supplied `1`) and confirms the answer follows the supplied value, which
  could only happen if the parameter is genuinely consulted rather than
  silently ignored in favour of a fresh parse. Checked in both directions
  (left operand overridden, then right) and with a strict-order comparison
  (`Less`, not just `Equal`) so the override's *direction* is pinned, not
  only its presence.
* `a_supplied_number_does_not_affect_strict_comparison` -- the converse
  check: `"007"`/`"7"` share a `Number` (7) but must not strict-compare
  equal, with the supplied `Number` present on one side, and the result
  cross-checked against `compare_bytes` on the same two byte strings to
  confirm the override changes nothing about the strict answer.

Ran `cargo test -p rexx-num --test compare`: **15/15 pass** (10
pre-existing + 5 new `#[test]` functions; all nine oracle transcripts live
in one of the five, as one test asserting all nine rows rather than nine
separate tests). Full crate: `cargo test -p rexx-num --no-fail-fast`,
every binary green (unit tests, `addsub.rs`,
`compare.rs`, `format.rs`, `parse.rs` [49 passed, 3 ignored --
pre-existing, unrelated to this task], `pow.rs`, `settings.rs`,
`whole.rs`). Also ran `cargo test -p rexx-num -p rexx-parse` together to
confirm the differential harness (`rexx-parse`'s only external caller of
`compare`) still passes unchanged: 255 unit tests, same count as before
this task, 0 failed.

## Verification, exit statuses read directly

* `cargo build -p rexx-num`: exit 0.
* `cargo test -p rexx-num --no-fail-fast`: exit 0, every `test result:` line
  `0 failed`.
* `cargo test -p rexx-num -p rexx-parse --no-fail-fast`: exit 0, `rexx-parse`
  unchanged at 255 passing unit tests.
* `cargo clippy -p rexx-num --all-targets -- -D warnings`: exit 0.
* `cargo fmt -p rexx-num -- --check`: exit 0 (after one `cargo fmt` pass
  wrapped a handful of long test lines).
* `cargo build --workspace`: exit 0, including `rexx-exec`, confirming
  nothing here broke the crate this whole task exists to unblock.
* One rustc lint fired during development and was fixed rather than
  suppressed: `invalid_from_utf8` on a redundant `assert!(std::str::
  from_utf8(&c3).is_err(), ...)` self-check -- the compiler can already
  prove a byte-array literal of this exact shape is invalid UTF-8 at
  compile time, so the runtime assertion was pointless as well as noisy.
  Removed it and left a comment stating the same fact instead of an
  `#[allow(...)]`, since there was nothing here actually worth suppressing.

## What ships

* `rust/crates/rexx-num/src/compare.rs`: `compare_bytes`, `compare_decoded`,
  `parse_bytes` added; `compare` refactored to delegate; `string_order`
  retyped `&str` -> `&[u8]`; module doc comment extended (not rewritten) to
  describe the three entry points and why they share one implementation.
* `rust/crates/rexx-num/src/lib.rs`: `pub use compare::{CompareOp, compare,
  compare_bytes, compare_decoded};`.
* `rust/crates/rexx-num/tests/compare.rs`: five new `#[test]` functions
  (covering the nine transcripts, the non-UTF-8 case, cross-entry-point
  agreement, and the two override-is-genuinely-used/scoped checks), a `cb`
  helper alongside the existing `cmp`/`c`, and `Number`/`compare_bytes`/
  `compare_decoded` added to the file's `use`.

Not touched: `rexx-core`, `rexx-exec`, `rexx-parse` (its differential
harness needed no change, only re-verified), `rust/corpus`.

## Commit

`5bf9b03d`, "Add byte-slice and pre-parsed comparison entry points to
rexx-num". Staged and committed exactly the three files listed above.
`git status` after the commit shows only another agent's untracked
`rust/crates/rexx-exec/src/error.rs`, not touched by this task.

## Addendum: `ArithError::sub_code` made public, and the pow asymmetry pinned

Small follow-on, same crate, dispatched after the above landed. Two asks:
make the sub-number accessor public (Task 7 is blocked on it), and check
whether `rexx-num` already produces the six-row power-operator asymmetry
table, pinning it if so or reporting a finding if not.

### What was already in the tree, read before changing anything

`rust/crates/rexx-exec/src/error.rs` exists (another agent's in-progress
Task 7 work, not `mod`-wired into `lib.rs` yet, so it does not currently
compile as part of the crate). It already:

* Explicitly requests `pub fn sub_code(&self) -> (u16, u16)`, "the same
  shape as `SymbolId::index()`", and calls its own hand-copied `match`
  "a known, flagged gap, not an oversight... a stopgap pending it, and
  should be deleted the moment it lands." So the shape was not really an
  open question -- Task 7 already named the exact signature it needs.
* Already documents the power-operator asymmetry precisely, independent
  of today's dispatch: `power_exponent_not_whole`'s doc comment states
  `2 ** 'x'` and `2 ** 2.5` are both 26.8, `'y' ** 2` and `'y' ** 'x'` are
  both 41.1 with the base checked first, and says this is "deliberately
  not routed through `nonnumeric`: the oracle's own asymmetry between the
  two operands is the fact being reproduced, not an implementation
  shortcut."

Also noticed, worth flagging even though it is not my file to fix: `impl
From<ArithError> for Raised` currently reads `let number = error.code();`
(consuming `error` by value, since `code(self)` takes ownership) and then
uses `error.additional()` and `match error { ... }` afterward -- once
`mod error;` is wired in, that would be "use of moved value". Rewriting to
call `sub_code()` first (which borrows) resolves this as a side effect,
since nothing in the rewritten shape needs to consume `error` at all
(`sub_code()` and `additional()` both take `&self`). Not fixed here --
`error.rs` is another agent's file -- but worth knowing before wiring
`mod error;` in surfaces it as a fresh compile error instead.

### `sub_code`, made public as it stood

Kept the existing signature and body unchanged, `pub fn sub_code(&self) ->
(u16, u16)`, rather than splitting into a `sub()` beside `code()`. Reasons,
recorded in both functions' doc comments so a reader does not have to
reconstruct them:

* It is exactly the shape Task 7 already asked for and modelled on an
  existing precedent (`SymbolId::index()`), not a shape I invented.
* `message()` already calls `self.sub_code()` internally to get both
  numbers at once -- making this public gives external callers the same
  natural entry point rather than a different one.
* `code()` predates `sub_code` and several callers only ever want the
  major (`bin/muldiv.rs`/`bin/addsub.rs` render it as `<E{major}>` and
  never touch the sub-number); forcing them to destructure a pair for a
  value they discard would not read better.

Investigated whether `code()`'s own signature should change too (`self` by
value, inconsistent with `sub_code(&self)`'s borrow) since a caller
wanting both would naturally want to call them in either order without a
clone. Found this is **not free**: `code`'s existing external callers
include `ArithError::code(e)` (associated-function syntax, in
`bin/muldiv.rs`/`bin/addsub.rs`) and `.map_err(ArithError::code)` (passing
the function itself as a value, in `tests/pow.rs`/`tests/muldiv.rs`).
Method-call syntax (`e.code()`) auto-refs a `&self` method transparently,
but these two call shapes do not -- changing `code` to `&self` would break
all of them, rippling across four files for a change nobody asked for.
Left `code(self)` exactly as it is; documented the coexistence instead of
forcing a signature change to "fix" it.

### The pow asymmetry: two of six rows are rexx-num's, four structurally cannot be

Re-verified all six transcripts against the oracle (wrapped), matching
the dispatch and `error.rs`'s own doc comment exactly:

```
2**'x'   -> Error 26.8, found "x"
2**2.5   -> Error 26.8, found "2.5"
'y'**2   -> Error 41.1, Nonnumeric value ("y")
'y'**'x' -> Error 41.1, Nonnumeric value ("y")   (base checked first)
2**-1    -> 0.5
0**0     -> 1
```

Then checked `rexx-num::Number::pow`'s actual signature:
`pub fn pow(&self, exponent: &Number, digits: u64) -> Result<Number,
ArithError>`. **Both operands are already-parsed `Number`s.** This is the
finding, not a gap to paper over: `pow` cannot receive a string that
failed to parse, because there is no way to construct the call at all
without a `Number` in hand for each side -- a non-numeric operand never
reaches this function, full stop, regardless of anything inside it. So of
the six rows:

* `2**2.5` and `2**-1`/`0**0` are genuinely `rexx-num`'s own behaviour
  (both operands parse; `pow` itself decides the outcome) -- **pinned**,
  with the full `(26, 8)` pair for the first (via the now-public
  `sub_code`) rather than only the major, since the major alone was
  already covered elsewhere in this file.
* `2**'x'`, `'y'**2`, `'y'**'x'` (and the fact the base is checked before
  the exponent) are decided **before** `pow` is ever called, by whichever
  caller runs `Number::parse` on each operand and picks which error to
  raise depending on which one (if either) failed -- `rexx-exec`'s job.
  There is no call this crate could make, and therefore no test this crate
  could write, that exercises that routing, because `pow`'s own type
  signature makes the non-numeric case unrepresentable as an argument.

Recorded this directly in the new test's doc comment
(`the_two_pow_asymmetry_rows_that_are_actually_this_crates_to_own`,
`tests/pow.rs`) rather than only in this report, specifically naming why a
later "simplification" cannot unify the two paths inside `pow` without
changing its signature to accept unparsed text -- a much larger change
than a simplification, and a decision for whoever owns that boundary, not
an invitation.

### Tests added

* `tests/pow.rs`:
  `the_two_pow_asymmetry_rows_that_are_actually_this_crates_to_own` -- the
  two rexx-num-owned rows, `2**-1 -> "0.5"` and `2**2.5`'s full `(26, 8)`
  pair, with the four-row structural exclusion recorded in the doc
  comment.
* `tests/errors.rs`:
  `sub_codes_major_half_agrees_with_code_on_every_variant` -- a
  consistency check (`code()` and `sub_code().0` must never disagree)
  over the same eight provoked errors
  `code_still_matches_every_variant_after_the_split` already uses, so a
  future variant added to one `match` and not the other is caught.
  `sub_code_is_42_3_for_divide_by_zero_and_26_8_for_power_exponent_not_
  whole` -- re-pins, through the real public accessor, the exact two
  numbers `error.rs`'s stopgap had independently verified against the
  oracle before this task, so a regression shows up in `rexx-num` itself.

### Verification

* `cargo build -p rexx-num`: exit 0.
* `cargo test -p rexx-num --no-fail-fast`: exit 0, every binary green
  (added tests: 17 in `tests/errors.rs`, up from 15; 9 in `tests/pow.rs`,
  up from 8).
* `cargo clippy -p rexx-num --all-targets -- -D warnings`: exit 0.
* `cargo fmt -p rexx-num -- --check`: exit 0 (after one `cargo fmt` pass).
* `cargo build --workspace`: exit 0, including `rexx-exec`, which does not
  yet reference `sub_code` (its own file that would is not `mod`-wired in)
  but confirms nothing here broke the rest of the tree.

### Commit

`4a320f1c`, "Make ArithError::sub_code public, and pin the pow asymmetry
that is rexx-num's to own". Staged and committed exactly
`rust/crates/rexx-num/src/lib.rs`, `tests/errors.rs`, `tests/pow.rs`.
`git status` after the commit shows only other agents' in-flight
`rust/crates/rexx-exec/` files (`lib.rs` modified, `error.rs`/`eval.rs`
untracked), none touched by this task.
