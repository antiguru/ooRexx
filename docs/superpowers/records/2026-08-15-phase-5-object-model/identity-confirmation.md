# Object identity: adversarial confirmation of deviation 4

Reviewer note. Every oracle row below was measured in this session with the wrapper
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`,
stdout/stderr/status read as three descriptors, each program copied into a freshly created
empty directory under the session scratchpad and run there by absolute path. Every program
exited rc 0 with empty stderr. Nothing in either repository was modified.

**Caveat on the oracle, found not made.** `/home/moritz/dev/repos/ooRexx` carried an
uncommitted change to `interpreter/classes/NumberStringClass.cpp` (`NumberString::copyIfNecessary`)
before this session began; `git status --porcelain` in that tree names it and no other path. Its
mtime equals `build/bin/rexx`'s, so whether the binary contains it is not established here. It
touches when a `NumberString` is cloned, which is upstream of nothing any row below keys an
`.IdentityTable` on, but it is named rather than assumed away.

**Verdict.** The claim's direction is right and its *extent* is wrong in both directions.
Divergence (a) is real but its stated boundary (seven bytes) is not this crate's boundary.
Divergence (b) is real but does not hold for every literal, and its own text says so in one
sentence and contradicts it in the next. The claim that these are the only two is false: this
report names divergences (c) and (d), plus a package-scope constraint on the (b) fix that the
spec does not state and that a natural implementation would get wrong.

---

## Part 1: the oracle's rule, re-derived

**Instrument.** `.IdentityTable~items` after two or more `[]=` puts, with `.Table` as the
equality-keyed control and `numeric digits 18` set in every program. Liveness of the instrument
was established before any finding was taken (`p1.rex`): two `.Object~new` -> 2; one object put
twice -> 1; three puts `o1,o2,o1` -> 2; `.Table` with `"qqqA"` and `"qq"||"qA"` -> 1 while
`.IdentityTable` with the same shape -> 2. So the table separates objects, merges repeats, and
the `.Table` control demonstrates that the separation is by identity and not by inequality.

**The rule, in one sentence.** Object identity in the oracle has four independent sources, and
"is it a literal" is only one of them: *within one compiled package a literal's TEXT selects one
object shared by every occurrence of either spelling, with the first occurrence in source order
deciding whether that object is a `RexxString` or a `RexxInteger`; independently of that, integers
in −10..100 and the null string are process-wide singletons; and a computed value is a fresh
object unless the operation that produced it had nothing to do, in which case it returns its own
argument.*

Each clause was measured, and the C++ that produces it was then read to check that the measurement
was not a coincidence of my sample:

* **Per package, keyed by text.** `p3.rex` requiring `p3b.rex`: `"ZZZZZZZZZZZZZZZZ"` in the main
  file against the same literal returned from a `::ROUTINE` in the *other* file -> 2, and against
  the same literal returned from a `::ROUTINE` in the *same* file -> 1. Two bytes behaves the same
  way (`"Yy"` -> 2 across packages). `p8.rex` calling `p8b.rex` as an external program -> 2. The
  mechanism is `LanguageParser::commonString` (`interpreter/parser/LanguageParser.cpp:2269`),
  which looks up in `strings`, a table owned by one `LanguageParser`, and `LanguageParser::addText`
  (`:2333`), whose `literals` table is consulted by **both** the `TOKEN_LITERAL` arm and the
  `SYMBOL_CONSTANT` arm. **CONFIRMED.**
* **Both spellings share one entry, first occurrence wins.** `24` then `"24"` then `20+4` -> 1
  item; `"25"` then `25` then `20+5` -> 2 items; `"26"` then `20+6` -> 2. The asymmetry is exactly
  `addText`'s: `SYMBOL_CONSTANT` first stores `name->requestInteger(...)` under the text, so a
  later quoted literal of that text retrieves the *integer*; a quoted literal first stores the
  `RexxString`, and the later numeric constant retrieves that string, which is then not the cached
  integer any arithmetic produces. **CONFIRMED** (measured, and the code says the same thing).
* **The integer cache is −10..100.** Bisected on the oracle with computed-vs-computed pairs
  (`(v-1)+1` against `(v-2)+2`): first non-shared positive value **101**, first non-shared negative
  value **−11**. `interpreter/classes/IntegerClass.hpp:185-186` declares
  `IntegerCacheLow = -10`, `IntegerCacheHigh = 100`. **CONFIRMED**, measurement and source agree
  and were obtained independently.
* **The null string is a process singleton, and it is not the package's `""` literal.**
  `substr("ab",1,0)`, `strip("   ")`, `copies("q",0)` and `left("ab",0)` in one table -> 1 item;
  any of them against a `""` literal -> 2; `""` here against `""` from another package -> 2. The
  singleton is `GlobalNames::NULLSTRING`. **CONFIRMED.**
* **A computed value is fresh, EXCEPT where the operation is a no-op.** This is the clause the
  claim under test does not have. Measured on one 16-byte value `lit`:
  identity **preserved** by `strip(lit)`, `copies(lit,1)`, `changestr("Q",lit,"R")` (no match),
  `right(lit,16)`, `delstr(lit,17)`, `lit||""`, `""||lit`, `lit~string`, `lit~request("STRING")`,
  `lit~send("STRING")`, `parse value lit with pv`, `parse var src pvar`, and by every pure
  transport (assignment, alias, argument passing, `arg(1)`, `value("VV")`, `st.1`, an `.Array`
  element, a `.Directory` index, a stem index). Identity **not** preserved by `substr(lit,1,16)`,
  `left(lit,16)`, `translate(lit)`, `lit~upper`, `word(lit,1)`, `subword(lit,1)`, `space(lit)`,
  `.String~new(lit)`, `~copy` at any length, and `d2c`/`x2c` even when the result is one or three
  bytes. `left` and `right` fall on opposite sides of this line. **CONFIRMED.**

The claim's own summary -- "a **computed** string is always a fresh object" -- is therefore
**false as stated**. It is true for a computed string that was actually computed.

Two further oracle facts that the spec's model does not contain:

* **`.true` and `.false` are their own objects.** `.true~class~id` is `String`, `.true = 1` and
  `.true == 1` both answer 1, and yet `.true` is distinct from the literal `1`, from the literal
  `"1"`, and from `0+1` (each -> 2 items). `.false` likewise against `"0"` and `0`. They are
  installed as `dotVariables` retrievers before parsing, per the comment at
  `LanguageParser.cpp:2436`. **CONFIRMED.**
* **`INTERPRET` gets a fresh package every execution.** A literal in the main program against the
  same literal built by `INTERPRET` -> 2; and `INTERPRET` of the *same* source string executed
  twice -> 2. **CONFIRMED.** This is the same per-package rule, and it is the one a naive fix to
  (b) will violate.

---

## Part 2: this crate's rule, from the code

This crate has no `.IdentityTable`, no `~identityHash` and no identity operator, so nothing below
is measured *as identity*. What is measured is the handle encoding those answers would be read
off. I say for each statement which it is.

**The rule.** Identity here is `ObjRef` equality, and `ObjRef` is a `#[derive(PartialEq)] struct
ObjRef(u64)` (`rust/crates/rexx-core/src/handle.rs:75-76`). So two values are one object exactly
when their handles are the same 64 bits. Four sources:

* **R1, inline text.** `Interp::text_bytes` (`rust/crates/rexx-exec/src/value.rs:186`) returns
  `ObjRef::inline_text(&bytes)` whenever that succeeds, and `ObjRef::inline_text`
  (`handle.rs:131`) is a pure, total, injective function of byte slices of length `0..=INLINE_TEXT`,
  where `INLINE_TEXT` is 7. Every text-creating path in the crate funnels through `text_bytes`
  (`text`, `text_owned`, `text_built` all call it), so **any two equal byte strings of seven bytes
  or fewer are the same object, whatever produced them, including the empty string.**
  *Evidence, not reading:* `cargo test -p rexx-core --offline --test handle` and
  `cargo test -p rexx-exec --offline --lib value::tests` were run in this session and passed,
  which covers `a_short_string_round_trips_through_the_handle_at_every_length`,
  `strings_differing_only_in_length_are_different_handles` (injectivity),
  `a_string_one_byte_too_long_is_refused_rather_than_truncated` (the boundary), and
  `value::tests::a_value_short_enough_is_the_handle_and_has_no_slot`, which asserts
  `heap.live_count()` does not move across construction for every length `0..=INLINE_TEXT` on both
  entry points and that `INLINE_TEXT + 1` is a heap handle again.
* **R2, tagged integers.** `ObjRef::small_int` (`handle.rs:120`) is a pure injective function of
  `i64` over `SMALL_INT_MIN..=SMALL_INT_MAX`, i.e. `±2^61`. `Interp::number`
  (`value.rs:202`) returns it whenever `small_int_for` admits the value under the *producing
  operation's* `DIGITS`. So **any two equal integers inside ±2^61 that are exactly representable
  under the digits in force are the same object.** Code-reading, resting on `handle.rs`'s
  `small_integers_are_encoded_inline_without_allocating`, which passed this session.
* **R3, literals.** `Interp::literal` (`value.rs:72`) returns `ObjRef::small_int` when
  `canonical_small_int` (`value.rs:588`) accepts the bytes -- optional `-`, then digits, no leading
  zero, in range -- and otherwise falls through to `text`. So a literal is R2 when its text is a
  canonical decimal integer and R1/R4 otherwise. Code-reading.
* **R4, everything else is fresh.** `text_bytes`'s fallthrough is `alloc_with`, an unconditional
  new slot. There is no cache, no pool and no interning of values anywhere: `Interp::literal` has
  exactly four callers -- `eval.rs:476` (`ExprKind::Literal`), `eval.rs:479` (`ExprKind::Constant`),
  `ir/drive.rs:685` (`Op::Const`), `ir/drive.rs:702` (`Op::LoadConstant`) -- and none of them
  memoises the returned handle. `Chunk::consts` (`ir.rs:1350`) is `Vec<Box<[u8]>>`: it pools
  *bytes*, and its own doc says `Op::Const` "builds a fresh value from them ... every time it
  runs". **What I searched for and did not find:** `HashMap<Vec<u8>, ObjRef>`,
  `HashMap<Box<[u8]>, ObjRef>` and the word `intern` across `rexx-exec/src` and `rexx-core/src`;
  the only `intern` hits are prose about *symbol spellings* and about `Chunk::consts`. A pool
  keyed some other way (a `Vec` scanned linearly, a field on `Activation`) would escape those terms.
* **R5, no identity-preserving operation exists here.** Every builtin in
  `rust/crates/rexx-exec/src/builtin/string.rs` returns through `interp.` -- I grepped for a
  `Ok(...)` return in that file whose expression does not mention `interp` and found none. Named
  checks: `strip` ends `Ok(interp.text(kept))` (`string.rs:688`); `changestr`'s no-match early
  return is `Ok(interp.text_built(unchanged))` (`string.rs:855`); `Interp::concat_values`
  (`eval.rs:904`) always builds into a buffer and finishes with `text_built`, with no null-operand
  short circuit; `parse_template.rs:727` builds every parsed piece with `self.text(&...[piece])`.
  The one place identity *is* preserved is pure transport: `value()`'s `SymbolKind::Name` arm
  returns `interp.read_by_name(&upper)` unchanged (`builtin/datatype.rs:527-533`), and assignment,
  argument passing and stem storage move the handle.

**Consequence the claim misses.** R2 and R1 are *different tags*, so in this crate a numeric
literal and an equal computed string are **not** the same object: `ObjRef::small_int(9)` and
`ObjRef::inline_text(b"9")` differ in their low two bits, which
`an_inline_string_is_not_a_slot_an_integer_or_nil` asserts directly and which passed this session.
That is why several rows below come out in agreement with the oracle for a reason that has nothing
to do with the oracle's reason.

---

## Part 3: every case tried

`crate` columns are code-reading per Part 2 unless marked. `n/a` marks a case whose crate side
needs a Phase 5 mechanism that does not exist yet (`~copy`, `.String~new`, `.Directory`, `~allIndexes`,
message send); those rows are stated as constraints, not as divergences.

Classification: **=** agreement, **(a)** short computed value shared here and not there,
**(b)** literal fresh here and pooled there, **(c)** integer sharing wider here than there,
**(d)** oracle preserves identity through a no-op and this crate does not, **new** something none
of those covers.

### Literal pooling (`p2.rex`, `p3.rex`, `p9.rex`)

| # | case | oracle | crate | class |
|---|---|---|---|---|
| A | two 16-byte literal tokens, same text | 1 | 2 | (b) |
| B | two 3-byte literal tokens, same text | 1 | 1 | = |
| C | one 16-byte literal evaluated five times in a loop | 1 | 5 | (b) |
| D | 16-byte literal, `"..."` vs `'...'` | 1 | 2 | (b) |
| E | alias `e2 = e1` | 1 | 1 | = |
| F | 16-byte literal vs equal computed | 2 | 2 | = |
| G | two equal 16-byte computed | 2 | 2 | = |
| H | two equal 3-byte computed | 2 | 1 | (a) |
| I | bare symbol's own value vs equal 16-byte literal | 1 | 2 | (b) |
| J | numeric literal `1234567890123456` vs `"1234567890123456"` | 1 | 1 | = |
| K | numeric literal `9234567890123456` twice | 1 | 1 | = |
| L | literal vs the same literal built by `INTERPRET` | 2 | 2 | = |
| M | `INTERPRET` of one source string, twice | 2 | 2 | = |
| N | main-file literal vs same literal in a `::ROUTINE` of that file | 1 | 2 | (b) |
| O | main-file literal vs same literal in a `::METHOD` of that file | 1 | 2 | (b) |
| P | a `::ROUTINE`'s literal returned on two calls | 1 | 2 | (b) |
| Q | 16-byte literal here vs the same literal in a `::REQUIRES`d file | 2 | 2 | = today |
| R | 2-byte literal here vs the same literal in a `::REQUIRES`d file | 2 | 1 | (a) |
| S | the other package's literal, returned twice | 1 | 2 | (b) |
| T | same-package `::ROUTINE` literal (control for Q) | 1 | 2 | (b) |
| 79 | main literal vs an external program `CALL`ed by name | 2 | 2 | = today |
| 85 | two `"12345678"` literals (8 bytes, canonical integer text) | 1 | 1 | = |
| 87 | two `"ABCDEFGH"` literals (8 bytes, not integer text) | 1 | 2 | (b) |
| 88 | two `"ABCDEFG"` literals (7 bytes) | 1 | 1 | = |
| 89 | two `"1234567890123456"` literals (16 bytes, integer text) | 1 | 1 | = |
| 90 | that 16-byte integer-text literal vs an equal computed string | 2 | 2 | = |
| 92 | `"05"` vs `05` | 1 | 1 | = |

### The inline boundary and the empty string (`p4.rex`, `p5.rex`)

| # | case | oracle | crate | class |
|---|---|---|---|---|
| 1 | `""` literal twice | 1 | 1 | = |
| 2 | `""` literal vs `substr("ab",1,0)` | 2 | 1 | **(a) at length zero** |
| 3 | two computed empties from different builtins | 1 | 1 | = |
| 28 | four computed empties (`substr`/`strip`/`copies`/`left`) | 1 | 1 | = |
| 29 | `copies("q",0)` vs `""` literal | 2 | 1 | (a) |
| 30 | `""` here vs `""` from another package | 2 | 1 | (a) |
| 4 | `"7777777"` literal vs computed `"777777"||"7"` | 2 | 2 | **=**, see below |
| 5 | two equal computed 7-byte values | 2 | 1 | (a) |
| 6 | `"88888888"` literal vs computed (8 bytes) | 2 | 2 | = |
| 7 | two equal computed 8-byte values | 2 | 2 | = |
| 8 | `"9"` literal vs `substr("99",1,1)` | 2 | 2 | **=**, see below |
| 37 | `"A"` literal vs `d2c(65)` | 2 | 1 | (a) |
| 38 | `"ABC"` literal vs `x2c("414243")` | 2 | 1 | (a) |
| 39 | `d2c(65)` twice | 2 | 1 | (a) |
| 63 | `"ABC"`, `strip("ABC")`, `copies("ABC",1)` | 1 | 1 | = |
| 20 | 2-byte string vs its own `~copy` | 2 | n/a | **(a), unfixable** |
| 19 | 16-byte string vs its own `~copy` | 2 | n/a | constraint |

Rows 4 and 8 are the ones that matter: they are the deviation's own headline shape -- a computed
short string equal to an existing value -- and they come out in **agreement**, because the literal
side is a `SmallInt` and the computed side is an `InlineText`. Deviation 4's boundary sentence
("below or at the handle's inline capacity: value identity") is therefore not this crate's rule.
Row 20 is the sharpest true statement of (a): a value of two bytes *is* its handle, so no
implementation of `~copy` can make a copy that differs, and the oracle answers 2.

### Integers (`p4.rex`, `p5.rex`, `p7.rex`, `p8.rex`, `p9.rex`, `p10.rex`)

| # | case | oracle | crate | class |
|---|---|---|---|---|
| 9 | literal `41` vs `40+1` | 1 | 1 | = |
| 12 | `40+2` vs `41+1` | 1 | 1 | = |
| 21 | `23` vs `"23"` (numeric spelling first) | 1 | 1 | = |
| 22 | `41`, `"41"`, `40+1` | 1 | 1 | = |
| 50 | `24`, `"24"`, `20+4` (numeric first) | 1 | 1 | = |
| 51 | `"25"`, `25`, `20+5` (quoted first) | 2 | 1 | **new** |
| 52 | `"26"`, `20+6` (no numeric spelling present) | 2 | 1 | **new** |
| 14 | `20+3` vs literal `"23"` | 2 | 1 | **new** |
| 91 | `"-5"` vs `0-5` | 2 | 1 | **new** |
| 75 | literal `100` vs `99+1` | 1 | 1 | = |
| 74 | literal `101` vs `100+1` | 2 | 1 | (c) |
| 76 | literal `-10` vs `-9-1` | 1 | 1 | = |
| 77 | literal `-11` vs `-10-1` | 2 | 1 | (c) |
| 23 | computed-vs-computed sharing, bisected upward | shares 0..100, not 101 | shares to `2^61-1` | (c) |
| 24 | same, downward | shares to −10, not −11 | shares to `-2^61` | (c) |
| 73 | two computed `100000007` | 2 | 1 | **(c), 9 bytes of text** |
| 83 | `"100000007"` literal vs `100000000+7` | 2 | 1 | **(c), 9 bytes of text** |
| 84 | `100000008`, `"100000008"`, `100000000+8` | 2 | 1 | (c) |
| 93 | literal `2305843009213693951` vs computed, `digits 25` | 2 | 1 | (c) |
| 95 | that value computed twice, `digits 25` | 2 | 1 | (c) |
| 94 | literal `2305843009213693952` vs computed (past the tag) | 2 | 2 | = |
| 71 | `do i = 1 to 3` values plus literals `1`,`2`,`3` | 3 | 3 | = |
| 72 | `do i = 200 to 202` values plus literals `200`,`201`,`202` | 5 | 3 | (c) |

Row 72 is worth reading twice. Five, not six and not three: the loop's *first* value is the literal
object `200` itself (no copy on assignment), and `201`/`202` are computed afresh and so differ from
the `201`/`202` literals. It confirms transport-preserves-identity and the cache bound in one figure.

### Identity-preserving operations (`p6.rex`, `p7.rex`, `p8.rex`)

`lit` is `"ABCDEFGHIJKLMNOP"`, 16 bytes.

| # | case | oracle | crate | class |
|---|---|---|---|---|
| 32 | `strip(lit)` (nothing to strip) | 1 | 2 | **(d)** |
| 33 | `copies(lit,1)` | 1 | 2 | **(d)** |
| 55 | `changestr("Q",lit,"R")` (no match) | 1 | 2 | **(d)** |
| 57 | `right(lit,16)` | 1 | 2 | **(d)** |
| 60 | `delstr(lit,17)` | 1 | 2 | **(d)** |
| 53 | `lit || ""` | 1 | 2 | **(d)** |
| 54 | `"" || lit` | 1 | 2 | **(d)** |
| 40 | `parse value lit with pv` | 1 | 2 | **(d)** |
| 41 | `parse var src pvar` (whole string) | 1 | 2 | **(d)** |
| 31 | `substr(lit,1,16)` | 2 | 2 | = |
| 56 | `left(lit,16)` | 2 | 2 | = |
| 34 | `translate(lit)` (no change) | 2 | 2 | = |
| 36 | `word(lit,1)` | 2 | 2 | = |
| 58 | `subword(lit,1)` | 2 | 2 | = |
| 59 | `space(lit)` | 2 | 2 | = |
| 81 | two `copies("ABCDEFGH",2)` | 2 | 2 | = |
| 42 | a parsed word vs an equal literal | 2 | 2 | = |
| 35 | `lit~upper` (already upper) | 2 | n/a | constraint |
| 61 | `lit~string` | 1 | n/a | **(d) constraint** |
| 62 | `lit~request("STRING")` | 1 | n/a | **(d) constraint** |
| 69 | `lit~send("STRING")` | 1 | n/a | **(d) constraint** |
| 48 | `.String~new(lit)` | 2 | n/a | constraint |
| 49 | `.String~new("ABC")` | 2 | n/a | constraint |

### Transport (`p6.rex`, `p8.rex`)

| # | case | oracle | crate | class |
|---|---|---|---|---|
| 46 | `lit` vs `echo(lit)` returning `arg(1)` | 1 | 1 | = |
| 47 | `lit` vs `value("VV")` | 1 | 1 | = |
| 43 | `lit` vs `st.1` holding it | 1 | 1 | = |
| 82 | `lit` vs an `.Array` element holding it | 1 | n/a (projected 1) | = |
| 44 | a stem's `~allIndexes[1]` vs the tail value that made it | 1 | n/a | constraint |
| 80 | a stem's index for a constant tail vs the equal literal | 1 | n/a | constraint |
| 45 | a `.Directory`'s index vs the literal put there | 1 | n/a | constraint |

### Singletons (`p4.rex`, `p5.rex`, `p7.rex`, `p8.rex`)

| # | case | oracle | crate | class |
|---|---|---|---|---|
| 15 | `.nil` twice | 1 | 1 (`ObjRef::NIL`) | = |
| 66 | `.true` twice | 1 | 1 (projected) | = |
| 16 | `.true`, `.true`, `1` | 2 | 2 (projected) | = |
| 64 | `.true` vs literal `"1"` | 2 | 2 (projected) | = |
| 65 | `.true` vs numeric literal `1` | 2 | 2 (projected) | = |
| 26 | `.true` vs `0+1` | 2 | 2 (projected) | = |
| 27 | `.false` vs `0+0` | 2 | 2 (projected) | = |
| 78 | `.false`, `"0"`, `0` | 2 | 2 (projected) | = |
| 17 | `.environment` twice | 1 | n/a | constraint |
| 18 | `.String` twice | 1 | n/a | constraint |
| 67 | a `::CONSTANT`'s value vs an equal literal in that package | 1 | n/a | **constraint** |
| 68 | a `::CONSTANT` read twice | 1 | n/a | constraint |
| 25 | `.true~class~id` / `.true = 1` / `.true == 1` | `String` / 1 / 1 | -- | context |
| 70 | `"NAMECLASH"` literal vs `value("NAMECLASH",,"ENVIRONMENT")` | 2 | -- | uninformative |

The `.true` rows are projections, marked so: `.true` is not implemented here, and D15 plus
`value::tests::nil_has_a_string_value_and_the_booleans_are_plain_strings` (which builds them with
`interp.text(b"1")`/`interp.text(b"0")`) is the whole basis. If Phase 5 instead builds `.true`
through `Interp::literal`, it becomes `SmallInt(1)` and rows 64, 65, 26 all flip to divergence.
**That is a design constraint this report is placing, not a finding about the tree.**

---

## Findings

**F1. Divergence (a)'s stated boundary is not this crate's boundary. CONFIRMED.**
"A value small enough to live in its handle has value identity" is the rule for text, and text is
only one of two things that live in a handle. A literal whose spelling is a canonical decimal
integer is a `SmallInt` regardless of length, and a `SmallInt` is never equal to an `InlineText`.
So (a) does not fire at rows 4 and 8, where the deviation predicts it, and it *does* fire at row 2
(the empty string) and row 20 (`~copy` of a two-byte string), neither of which the deviation
mentions. The correct boundary statement is two statements: equal text of seven bytes or fewer is
one object; equal integers inside `±2^61` are one object; and the two families do not meet.

**F2. Divergence (b) does not hold for every literal, and the deviation contradicts itself about
this. CONFIRMED.** `phase-4-exclusions.txt` first says "a LITERAL is a fresh object on every
evaluation here" and then, one sentence later, "two identical **long** literals are two objects
here". The spec's bullet carries only the first form. Measured against the code: a literal of
seven bytes or fewer is the *same* handle at every evaluation (row 88, `"ABCDEFG"`), and a literal
of any length whose text is a canonical integer is the same handle at every evaluation (rows 85
and 89, at eight and sixteen bytes). (b) is real only for literals of eight bytes or more that are
not canonical decimal integers -- rows A, C, D, I, N, O, P, S, T, 87. Pooling literal *values*
will therefore change fewer programs than the bullet implies, and a gate that samples literals
without controlling for those two shapes will report the fix working when it has not run.

**F3. A third divergence, (c): integer sharing is far wider here than on the oracle. CONFIRMED on
the oracle side, code-reading on ours.** The oracle shares exactly −10..100, measured by bisection
and confirmed at `IntegerClass.hpp:185-186`; everything else is a fresh `RexxInteger` per
arithmetic operation. This crate shares every integer in `±2^61`. The gap is observable at three
bytes of text (rows 74, 77) and at nine (rows 73, 83), so it is **outside** deviation (a)'s
"seven bytes or fewer" wording and is not entailed by entry 59 -- `SmallInt` predates it. Deviation
4's closing line, "this crate's `SmallInt` immediates match the integer row by luck of the same
design", is true only for the arguments in −10..100 that the row happened to use.

**F4. A fourth divergence, (d), and it runs the other way: the oracle preserves identity through
an operation that has nothing to do, and this crate cannot. CONFIRMED on the oracle side,
code-reading on ours.** Named set: `strip` with nothing to strip, `copies(s,1)`, `changestr` with
no match, `right(s,length(s))`, `delstr(s,n)` past the end, concatenation with the null string on
either side, `~string` and `~request("STRING")` on a String, and a `PARSE VALUE`/`PARSE VAR`
template that assigns the whole string to one variable. Every one of these gives this crate a
fresh heap object at 16 bytes, because `strip` ends `Ok(interp.text(kept))`, `changestr`'s no-match
path ends `Ok(interp.text_built(unchanged))`, `concat_values` has no null-operand short circuit,
and `parse_template.rs:727` builds every piece with `self.text`. This is the *opposite* direction
from (a): the oracle shares and we do not. It is invisible today and becomes visible the moment
`.IdentityTable` exists. It is also **not** simply fixable by "return the argument", because `left`
and `right`, and `substr(s,1,length(s))` and `delstr(s,length(s)+1)`, land on opposite sides of the
oracle's line -- the set has to be copied from the C++ per builtin, not derived from a principle.

**F5. `.true`/`.false` are distinct objects from every spelling of `1`/`0`. CONFIRMED (oracle).**
Rows 16, 64, 65, 26, 27, 78. This crate gets the right answer only if they are built through
`Interp::text` and not `Interp::literal`. Stated as a constraint above; if the spec's object model
routes environment symbols through the literal path it introduces a divergence at no benefit.

**F6. The oracle's literal-pool entry is order-dependent between the two spellings of a number.
CONFIRMED.** Rows 50 vs 51 differ only in whether `"25"` or `25` is written first, and the item
count differs. Any implementation that pools literals by text must decide the same way -- numeric
spelling first yields the shared integer, quoted first yields a string that is then *not* the
shared integer. This crate currently answers 1 for both orders (rows 51, 52, 14, 91), which is a
divergence today and would remain one after a naive text-keyed pool.

**F7. What I searched for and could not reach.** I looked for a value pool in this crate by the
terms `intern`, `HashMap<Vec<u8>, ObjRef>` and `HashMap<Box<[u8]>, ObjRef>` over
`rexx-exec/src` and `rexx-core/src`, and for identity-preserving builtins by grepping
`builtin/string.rs` for a `Ok(...)` return not mentioning `interp`. A pool held in a `Vec` and
scanned linearly, or a builtin in another `builtin/*.rs` file that returns an argument handle
through a helper, would escape both. On the oracle side I did not probe: the native API, `.Method`
and `.Routine` objects, `~run`/`~setMethod`, `.MutableBuffer`, streams, or any collection other
than `.Array`, `.Directory` and a stem. I also did not establish whether a package's literal pool
survives the package being loaded twice by two different requirers.

---

## The forward-looking question: would per-package literal pooling introduce a divergence?

**No -- per-package pooling is required, and *global* pooling would introduce one.** Measured:
two identical 16-byte literals in two packages are **two** objects on the oracle (row Q), and so
are two identical 2-byte ones (row R), and so is `""` (row 30), and so is a literal reached through
an external `CALL` rather than `::REQUIRES` (row 79). The C++ reason is that
`LanguageParser::commonString`'s `strings` table and `addText`'s `literals` table both belong to
one `LanguageParser` instance, i.e. one compilation.

So the fix for (b) carries three constraints, and the spec states none of them:

1. **The pool is per compiled unit, not per interpreter.** A global pool turns rows Q, R, S and 79
   from agreement into divergence, which would be trading one defect for a wider one. Note that
   this crate reaches the right answer for Q today *by having no pool at all*; a fix that improves
   the same-package rows and regresses the cross-package rows can look like progress in a gate that
   only measures the former.
2. **`INTERPRET` gets a fresh pool per execution.** Rows L and M: the oracle answers 2 both for
   "main literal vs `INTERPRET`ed literal" and for "the same `INTERPRET` source run twice". If the
   pool is attached to a `Code`/`Chunk` that `INTERPRET` compiles once and caches, row M becomes 1.
3. **A `::CONSTANT`'s value comes out of the same pool as its package's literals** (row 67), and
   the two spellings of a number share one entry with source order deciding the object (F6).

Both constraints 1 and 2 are properties of *where the pool lives*, which is exactly the decision a
task implementing "pool the value" will make in passing and never write down. They belong in the
task's text, not in this report.

## Status of the claim under test

**Needs amendment**, on four counts: (a) is stated at the wrong boundary (F1); (b) is stated too
broadly and the source document contradicts itself about it (F2); the "exactly two divergences"
closure is false, with (c) and (d) both real and (d) pointing the other way (F3, F4); and the fix
for (b) has package-scope and `INTERPRET` constraints that are not recorded anywhere (forward
question). The *licensing* decision is untouched by any of this: (a) still follows from entry 59,
and row 20 shows it cannot be removed without reverting it.
