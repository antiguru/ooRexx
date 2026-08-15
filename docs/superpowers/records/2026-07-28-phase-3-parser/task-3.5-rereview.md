# Task 3.5 re-review: the fix round

Re-reviewed `1dd397f7`, which answers `task-3.5-review.md`'s three Important
and five Minor findings plus two adjudications requiring change, against
`build/bin/rexx` / `build/bin/rexxc` 5.3.0.

## Verdict: all findings addressed, task complete.

Every Important finding, both adjudications, and all five Minors are fixed as
claimed. No new defect found in the changed code. One prose-based gate rule is
sound as a stopgap but more brittle than it needs to be; recorded below rather
than filed as a defect, since nothing depends on it failing yet.

## Findings and adjudications, one verdict each

| # | Item | Verdict |
|---|---|---|
| I1 | Superclass gate widened to `Variable \| Stem \| Compound \| DotSymbol` | Fixed (confirmed by caller before this review; re-derived independently from `Token.hpp:572,576`) |
| I2 | `Expr::shape` collisions (`_` omission marker, unquoted message name) | Fixed. No collision found in any constructed pair, including the three the dispatch named specifically. |
| I3 | Four ownerless `#[allow(dead_code)]` attributes | Fixed (confirmed by caller; gate grep prints nothing) |
| Adj. `TokenCursor::back` | Deleted with its three tests | Fixed |
| Adj. `parse_expr`'s terminator-set + sub-number parameters | Fixed, and correctly justified | 
| M1 | Report's allowance count one low; `lib.rs` pointed at a nonexistent attribute | Fixed |
| M2 | `expr.rs` module doc named six nonexistent functions | Fixed |
| M3 | `binary_rest`'s message-operator arm comment called it unreachable | Fixed (confirmed by caller) |
| M4 | `Expr::shape` / `for_each_child` needlessly `pub` | Fixed |
| M5 | `(a) + b` root-span slice comment | Fixed |

## I2 in detail

Verified the exact `quoted()` function (`format!("{:?}", String::from_utf8_lossy(bytes))`)
against every pair the dispatch asked for, by reproducing the function
verbatim outside the crate and by reading `ast.rs:369`-`402` directly:

* **A literal whose text is literally `<omitted>`.** `f('<omitted>', 1)` is
  valid input (confirmed `rc=0` under `rexxc`). It renders `"<omitted>"`
  (quoted, with the surrounding `"` marks Rust's `Debug` adds), which cannot
  equal the omitted-argument marker `<omitted>` (no surrounding quotes). No
  collision.
* **A name containing a double quote.** `quoted(b"B\"C")` → `"\"B\\\"C\""`.
  Compared against a name that is instead `B`, backslash, `"`, `C` (one byte
  longer): `quoted(b"B\\\"C")` → a different, longer escaped string. Rust's
  `Debug` escaping for `str` is injective on valid UTF-8 — backslash and quote
  are always escaped, so two different valid-UTF-8 byte strings never produce
  the same escaped form. No collision.
* **A name containing a backslash.** Covered by the same argument; a lone
  backslash escapes to two characters (`\\`), and no other input produces that
  same two-character run without also containing a backslash.

The mechanism the review flagged (`'a''b'` decoding to `a'b`, which a naive
single-quote wrap could not render unambiguously) is exactly what `{:?}`
avoids: I reproduced both the naive wrap (`'a'b'`) and the `Debug` form
(`"a'b"`) side by side and only the former is ambiguous.

**One residual gap, not a new defect and not one of the pairs asked for:**
`String::from_utf8_lossy` maps invalid UTF-8 to the replacement character
U+FFFD, and two different invalid byte sequences can collapse to the same
replacement text — verified `quoted(&[0xFF]) == quoted(&[0xFE])`. This
predates the fix round (the old `Literal` arm already used
`from_utf8_lossy` unescaped) and is unreachable from any symbol-derived name
(the scanner's symbol alphabet is always valid UTF-8); it would only bite a
message name or literal built from a raw binary literal, which the crate does
not yet exercise. Worth a one-line note if this renderer is still in use when
binary literals get a test, not worth reopening now.

## The terminator-set parameter, in detail

Verified against `interpreter/parser/LanguageParser.cpp` and
`InstructionParser.cpp` directly, not against the report's transcription:

* **18 call sites of `requiredExpression`**, confirmed by grep across both
  files (1 in `LanguageParser.cpp:1742`, 17 in `InstructionParser.cpp`).
* **5 distinct terminator sets**, with the exact counts the report gives:
  `TERM_CONTROL`×6, `TERM_EOC`×8, `TERM_EOC|TERM_WITH|TERM_KEYWORD`×1,
  `TERM_OVER`×2, `TERM_RIGHT`×1. Sums to 18.
* **13 distinct error codes**, all in the 35.9xx block: cross-checked against
  `interpreter/messages/RexxErrorCodes.h:324`-`351` (`address`=35914,
  `assign`=35918, `by`=35905, `control`=35904, `for`=35907,
  `form`=35917, `interpret`=35912, `options`=35913, `over`=35911,
  `select_case`=35933, `signal`=35915, `to`=35906, `trace`=35916). All 13
  confirmed.

The adjudication's own text (`"Mirroring that signature in parse_expr..."`)
names `requiredExpression(int terminators, RexxErrorCodes error)` — the C++
signature already carries both the terminator set and the error code as
separate parameters. Taking both into `parse_expr` is exactly what the
adjudication pointed at, not a step past it; the dispatch's framing that this
"goes one step past what was asked" doesn't hold once the adjudication's own
quoted signature is read literally.

**Signature ergonomics.** `parse_expr(ctx, cursor, term, missing: u16)` adds
two parameters over the pre-fix version, both required for every call. That
mirrors `requiredExpression` exactly (the C++ version has no default either)
and matches the file's existing `error(code: u16, sub: u16)` convention, so it
is not awkward relative to the rest of the file. Every one of Task 3.6's six
`DO`-loop required expressions will pass `Terminators::CONTROL` and a distinct
sub-number; nothing forces a caller to reconstruct the terminator check by
hand, which was the point of the change.

## `TokenCursor::back`

Confirmed: no caller anywhere in the crate (`grep -rn '\.back(' rust/crates/rexx-parse/src/` is empty), the three tests that exercised it are gone, and `TokenCursor`'s own doc comment now states why the method
existed and was removed — "Forward only. ... A `back` method existed and was
removed once the expression grammar showed it had no caller" — which is
exactly the note that stops a future contributor from re-adding it
speculatively.

## The five Minors

1. **Fixed.** `lib.rs` now says "Eight dead-code allowances ... and none in
   this file," which is true: `grep -rn 'allow(dead_code)' src/lib.rs` is
   empty, and the crate has exactly eight attributes (confirmed by count).
2. **Fixed.** The module doc's eleven names (`expression`, `full_subexpression`,
   `subexpression`, `message_subterm`, `subterm`, `message`,
   `collection_message`, `arg_list`, `logical`, `qualified_symbol`,
   `variable_reference_term`) all exist as `Parser` methods, confirmed by
   grep against `expr.rs`, and the eleven C++ names they're mapped to all
   exist in `LanguageParser.hpp`, confirmed by grep there too.
3. **Fixed** (confirmed by caller). The corrected comment's causal claim —
   that `variable_reference_term` returns straight to its caller rather than
   through `message_subterm`'s cascade loop — matches the code at
   `expr.rs:672`-`706` and the arm's own reachability is independently
   re-confirmed.
4. **Fixed.** `for_each_child` is `pub(crate)` and has a live production
   caller (`Expr::new`'s span-widening loop, `ast.rs:270`-`276`), so it needed
   no `#[cfg(test)]`. `shape`, `render_args` and `quoted` are all
   `#[cfg(test)] pub(crate)`, and their only callers (`expr/differential.rs`,
   `expr/tests.rs`, `ast/tests.rs`) are themselves behind `#[cfg(test)] mod`,
   so nothing is broken by the narrower visibility.
5. **Fixed.** The comment now states the general property (a span runs from
   one token's start to another's end, not a self-contained substring) rather
   than special-casing the parenthesis example, and ties it to the
   containment property the gate actually checks.

## Check 5: the "don't spell the attribute out in prose" rule

**My opinion: sound as an immediate fix, but more brittle than it needs to
be, and I would not leave it as the only safeguard.**

The problem it solves is real: `grep -rn 'allow(dead_code)' src/ | grep -v
'Task 3\.[0-9]'` cannot distinguish a real attribute line from a comment that
merely mentions the attribute's name, because grep has no syntax awareness.
Rewording the one offending comment and stating a rule against future
prose mentions removes today's false positive without making the grep itself
smarter.

The brittleness: the rule is enforced by nobody. It lives in a paragraph in
`lib.rs`, and the only thing that would catch a violation is a human noticing
during a future gate run, at which point the exact failure this round just
fixed reappears — a stray line the grep flags as ownerless, requiring a fix
round to explain it's not a real attribute. A textual convention that a tool
cannot check is exactly the kind of rule that survives until someone forgets
it.

A more robust fix was available and wasn't taken: anchor the grep to
line-start attribute syntax, e.g. `grep -n '^\s*#\[allow(dead_code)\]'`. Real
attributes always begin a line (ignoring indentation); a `//`-comment
mentioning the attribute never does, because it starts with `//`. That would
make the gate correct regardless of what any comment says, and the "don't
spell it out" rule would become unnecessary rather than load-bearing. I'd
raise this as a follow-up for whoever owns the phase gate script next, not as
something this task needs to redo — the current rule works for the eight
attributes that exist today, and the grep gate call site (whatever runs it at
the phase boundary) is out of this task's scope.

## Minor observation, not filed

`impl Terminators`'s attribute now reads `// deleted by Task 3.7`, but the
block's own doc comment above it still attributes the block's eventual use to
"Tasks 3.6 and 3.7" jointly (`DO`'s `TO`/`BY`/`FOR`/`WHILE` constants are as
likely to get their first caller from Task 3.6 as `WITH`/`THEN` are from Task
3.7). This item was already in the review's "naming an owner" column before
the fix — it wasn't one of the four I3 required to change — so the round's
decision to give it the same uniform tag as the other seven is a consistency
choice, not a requirement. The single-task tag is a simplification of the
doc's own two-task attribution immediately above it. It doesn't break the
gate grep and causes no functional problem; noting it only because the round
touched a line that didn't strictly need to change.

## Conclusion

Nothing in this fix round introduces a regression. The three Important
findings, both adjudications, and all five Minors check out against the code
as committed, re-derived independently rather than taken on the report's word
where the dispatch asked for that (I2's collision-freedom, the 18/5/13 counts
behind the terminator-set adjudication). Task 3.5's fix round is complete.
