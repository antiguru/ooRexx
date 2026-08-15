# Task 3.3 review: Scanner and tokens

Reviewed at `410b2783` against `.../task-3.3-brief.md`, `.../task-3.3-report.md` and
`review-fb2924db..410b2783.diff`.

## Verdicts

**Spec compliance: PASS.**
Every deliverable the brief names exists, with the shape the brief specifies: the 19
token classes in `TokenClass` order, the 32 operator subclasses in `TokenSubclass`
order, `Tag` with one variant per `TokenKind` variant, `SymbolId`/`SymbolTable` with
the `Cow` interning path verbatim, all six keyword tables in `KeywordConstants.cpp`
order with a linear unsorted `index_of`, `ParseCtx`, `TokenCursor` with `advance`
rather than `next`, `ParseError { code, sub, byte, subs }`, and
`scan(&ProgramSource) -> Result<Scanned, ParseError>`.
The two-part significant-blank rule is both parts, the `Eoc` model is the one the
brief states rather than a near-miss, literals carry a decoded value, and the brief's
five Step-2 tests are present with their bodies intact and extended.
Two spec deviations exist, both raised by the implementer and both adjudicated below;
neither changes a mandated type.

**Code quality: PASS.**
The port is structural rather than rule-by-rule and that choice is what makes it hold
up: I could not find a behavioural divergence from the oracle in the scanner's own
territory. Independent differential runs of my own, not re-runs of the report's:

| corpus | files | oracle scanner-class errors | mismatches |
|---|---|---|---|
| `ootest/`, `samples/`, `support/` | 783 | 2 (13.1 line 2) | 0 |
| `rust/corpus-l1/*.rex` | 12,059 | 1 (6.1 line 7) | 0 |
| rest of the repository | 141 | 0 (22 are 98.903 `::requires`) | 0 |
| random ASCII programs, 1-14 tokens | 4,000 | 1,303 | 0 |
| random byte programs incl. 0x80-0xFF and 251-char names | 1,500 | 1,123 (653 x 13.1, 132 x 30.1) | 0 |
| `::RESOURCE` shape/marker/body matrix | 136 | 44 x 99.943 | 0 |

Across all of that, **the scanner never once invented an error the interpreter does
not raise** (`oracle ok` and `rust error` never co-occurred), and every 6.x / 13.1 /
15.x / 30.1 / 99.943 it did raise matched the oracle's number, sub-number and reported
line exactly. Of the 136-file resource matrix, every one of the 44 files `rexxc`
accepts has a per-resource line count equal to the interpreter's own `~resources~items`.
The measurement table in the report reproduces exactly (8,118/526 and 2,121/273).

Findings: **0 Critical, 2 Important, 8 Minor.** None blocks the task.

---

## The five concerns, adjudicated

### 1. `Scanned` gained `resources: Vec<ResourceBody>` — **accept as-is**

The shape is right. `directive` is the index of the `::` token, and because a clause
can never begin with a `Blank` (a clause start passes no previous token, so
`locate_token` is called with blanks insignificant), `real[0]` is always
`clause_first`. So Task 3.7's lookup is one integer comparison against
`clause.tokens.start` — no re-derivation of the marker rule, and no name-upcasing
duplicated into the scanner. `Vec<Range<usize>>` per line is exactly what
`getStringLine()` returns (`new_string(current, currentLength)`, terminator excluded).

I stress-tested the consumption contract rather than assuming it: 17 directive
spellings x 8 bodies. It gets right, against the oracle, every case I could think to
break it on — a literal name, `::resource<TAB>data`, `::  resource   data`, a trailing
`--` comment on the directive line, `;` mid-line (`conditionalNextLine`), a symbol
marker upcased versus a literal marker case-preserved *in both directions*
(`end zz` + body `zz` is 99.943, `end 'zz'` + body `zz` is 0 lines), the prefix match
(`::ENDING more` terminates a default-marker resource), and `end ''` where the empty
marker prefix-matches every line and correctly yields a zero-line resource.

Two things to record on the field, not to change: duplicate names remain Task 3.7's
(verified: two `::resource data` directives give 99.942 from the directive parser
while the scanner records both bodies), as do 25.926 / 21.914 / 19.921 for malformed
directives.

### 2. The `#!` skip lives in the scanner — **accept the placement; needs change for INTERPRET**

Line-1-only is exactly right, and I checked it three ways rather than one:
`say 1` then `#!/usr/bin/env rexx` is 13.1 line 2 in both; a blank before the `#!` on
line 1 is 13.1 line 1 in both; a `#` on line 2 after a real `#!` line 1 is 13.1
line 2 in both. `first_line` reads `line(1).starts_with(b"#!")`, which is the same
predicate as the C++'s `bufferArea[0]=='#' && bufferArea[1]=='!'` for every input with
at least two bytes and safer for shorter ones.

The gap is not the placement, it is the conditionality — see Important 1.

### 3. Eager scan reports a later scan error where the oracle reports an earlier parse error — **accept but record as a named deviation**

The relaxation does not cover this. It relaxes the *line* to "plausible" and drops
message text; number and sub-number stay gated (`progress.md`: "KEEP correct error
number AND sub-number"). A 6.2 where the oracle says 37.2 is a different number, so it
is a deviation and must be written into the deviation register rather than absorbed.
It genuinely cannot be fixed under the mandated `Result<Scanned, ParseError>`.

But the report's frequency claim undersells it, and that matters for Task 3.8. It is
not confined to errors on different lines: it fires whenever *any* clause boundary,
including a mid-line `;`, separates a clause that fails to parse from a later clause
that fails to scan. My 4,000-file adversarial fuzz hit it 144 times (3.6%), e.g.
`[;/*[''` — oracle 35.1 line 1, ours 6.1 line 1, same line, adjacent clauses. On real
code it is as rare as the report says (0 of 783, 0 of 141, 1 of 12,059:
`rust/corpus-l1/LINES_test_stdin_normal.rex`). Record both numbers, and warn Task 3.8
that a differential capture over *generated* invalid input will trip this constantly.

### 4. `ParseCtx` and `TokenCursor` are `pub` — **accept but record; narrowing is owed to Task 3.5, not merely available to it**

`pub` is the right call now; `#[allow(dead_code)]` would be worse because it also
silences a genuinely unused field later. But the report's "Tasks 3.5 to 3.7 can narrow
it" is weaker than what is needed. The owed work is specific and will not happen
unless written down: when Task 3.5 gains the first in-crate caller, narrow both to
`pub(crate)` **and** move the four `TokenCursor` tests out of `tests/tokens.rs` into a
`#[cfg(test)] mod tests` inside `token.rs`, because an integration test cannot see a
`pub(crate)` type and the narrowing will otherwise fail to compile and be reverted.

### 5. `ParseError.byte` is the clause start — **accept but record**

Faithful, and I re-measured it independently rather than taking the report's word:
across 14 crafted cases plus 2,426 fuzz hits, every reported line is the clause's, for
6.1, 6.2, 6.3, 13.1, 15.1 through 15.6, 30.1 and 99.943. `say 1,` continued onto
`b<C3A4>c` is line 1 in both; `say 1,` continued onto `/* unclosed` is line 1 in both
while 6.1's substitution names line 2. The doc comment states this contract clearly.

Two things to record. The field name reads as "the offending byte" and is not one; a
consumer that slices at `byte` expecting the bad character gets the clause's first
character instead. And Task 3.8, if it ever fills `subs`, needs the offending position
as well (13.1's `"ä" ('C3A4'X)`, 15.3's `found "g"`, 6.1's `on line N`) — this struct
cannot supply it, so 3.8 must add a second field rather than assume `byte` will do.
Harmless today because substitutions are ungated.

---

## Findings

### Important

**IMP-1. The `#!` skip is unconditional, and the interpreter suppresses it for `INTERPRET`.**

`BufferProgramSource` sets `firstLine = 2` for a shebang, but `ArrayProgramSource`
guards the same rule with `interpretAdjust == 0` (`ProgramSource.cpp:594`): an
`INTERPRET` string that opens with `#!` is *not* skipped. Verified against the oracle.

* Input: `interpret "#! nothing here"` inside a program.
* Oracle: `Error 13.1: Incorrect character in program "#" ('23'X)`, line 1
  (and with `signal on syntax`, the handler runs).
* Ours: `scan` calls `first_line`, which sees `#!` on line 1 of the interpret string
  and starts on line 2 — an empty program, no error, silently accepted.

Nothing is wrong today because `parse_interpret` does not exist yet. But `scan` is the
single mandated entry point and Task 3.9 must reuse it, so the suppression has to be
reachable from the signature. Cheapest fix that keeps the brief's `scan` shape: move
`first_line` onto `ProgramSource` as a constructor-time property (which is where the
C++ puts it), with a second constructor for the interpret case; the scanner then reads
it instead of computing it. Whatever the fix, it belongs to this task's interface, not
to 3.9 discovering it.

**IMP-2. A token span cannot be turned back into bytes from outside the crate.**

`ProgramSource` exposes `line_count`, `line`, `line_span` and `line_of`; `text` is
private and there is no `text()` or `slice(Range)`. Task 3.3's whole output is
"spans, never copied strings", so every later consumer that needs a token's or a
clause's source text — Task 3.4's clause spans for gate criterion 6, `TRACE`, error
display — has nothing to slice.

The tree already shows the pressure. `tests/scanner.rs:436-437` does:

```rust
let source = ProgramSource::new(b"aBc = 1\nsay ABC\nsay aBc".to_vec());
assert_eq!(&source.line(1).unwrap()[spans[0].clone()], b"aBc");
```

That indexes a *line-relative* slice with an *absolute* span. It passes only because
`spans[0]` is `0..3` and line 1 starts at offset 0. Substitute `spans[1]` (`12..15`,
verified) and it panics with an out-of-range index on a 7-byte line. The comment above
it claims "The span, not the id, is what recovers the source spelling", which the
assertion does not actually establish in general. The other two tests that need bytes
sidestep `ProgramSource` entirely and slice the original `&str` they still hold.

* Input: any consumer holding `Token { span: 12..15 }` for line 2.
* Wrong: `source.line(2).unwrap()[12..15]` panics; `source.line(1).unwrap()[12..15]`
  panics; nothing else is available.
* Right: `&source.text()[12..15] == b"ABC"`.

Add `ProgramSource::text(&self) -> &[u8]` (or a bounds-checked `slice`) and fix that
test line to use it. I note the counter-argument — the brief deliberately refuses
speculative accessors (no `SymbolTable` lookup-by-name) — but this one already has a
caller inside the test suite, badly served.

### Minor

**MIN-1. `scan_resource_if_directive` allocates a `Vec<usize>` for every clause in the
program.** No wrong output. It is called after every emitted `Eoc` and builds `real`
before any cheap rejection:

```rust
let real: Vec<usize> = (self.clause_first..eoc).filter(...).collect();
if real.len() != 3 && real.len() != 5 { return Ok(()); }
```

`CoreClasses.orx` has 2,402 clauses, so 2,402 heap allocations, against the
~7,592 short-string allocations interning saves on that file — about a third of the
task's own stated win handed back. Guard with
`if self.tokens[self.clause_first].kind.tag() != Tag::DColon { return Ok(()) }` first.

**MIN-2. `Operator`'s doc comment contradicts itself.** "Three of these are never
scanned" is followed by a list whose third entry says "`Concatenate` is scanned (from
`||`) but also synthesised". Only two are never scanned. Say two.

**MIN-3. The panic-sweep test's name asserts the opposite of its property.**
`no_input_makes_the_scanner_panic` reads as "some input makes the scanner panic". The
doc comment gets it right ("Not panicking is the property"). Rename to something like
`the_scanner_never_panics`.

**MIN-4. No scanner-level test pins a token span across a CRLF or bare-CR line.**
`line_span` itself is pinned for both in `tests/sourceline.rs`, and I confirmed
empirically that the scanner's addition is right — `ab=1\r\ncd=2\r\nef=3` gives line-2
tokens at `6..8`, `8..9`, `9..10`, and the bare-CR variant gives `5..7`, `7..8`,
`8..9` — but the `line_start + line_offset` addition is this task's and only LF lines
are pinned at this level.

**MIN-5. `Scanned::tokens` does not state its invariant on the field.** The module doc
explains the collapsing, but Task 3.4 reads the field and depends on "empty, or ends
with exactly one `Eoc`, and never two adjacent". Put it where 3.4 will look.

**MIN-6. `ResourceBody` does not say what it deliberately leaves undone.** Duplicate
names (99.942), name validity, and the `END` sub-keyword check are all still Task
3.7's, and the doc reads as though a `ResourceBody` is a validated resource.

**MIN-7. `Scanner::new` re-binds its parameter.** `fn new(source, symbols: SymbolTable)`
followed by `let mut symbols = symbols;`. Write `mut symbols: SymbolTable` in the
signature.

**MIN-8. `translate_char` returns a value no caller reads.** Only `is_symbol_char`
calls it, and only for `!= 0`. The doc justifies keeping the C++ shape, which is a
reasonable call, but a reader will hunt for the upcasing consumer and find none —
interning upcases independently via `to_ascii_uppercase`. One sentence saying the
return value exists to document `characterTable` and is deliberately unused would
close it.

---

## Verified in detail, no finding

Recorded so a later reader does not redo it.

* **19 token classes** match `Token.hpp:77-98` in name and order. `Null`, `Prefix`,
  `Point`, `Continue` are correctly present and documented as never produced; a grep
  confirms the C++ never constructs them either — all four appear only in the enum
  declaration.
* **32 operator subclasses** match `Token.hpp:110-141` in order.
* **All six keyword tables** match `KeywordConstants.cpp` byte-for-byte in content and
  order (checked programmatically: 35/50/12/10/9/40, zero differences). The seventh
  C++ table, `builtinFunctions` (162 entries), is correctly out of scope.
* **The significant-blank rule** is `Symbol | Literal | RightParen | RightBracket` on
  the left (`Token.hpp:595-596`) and symbol-start / quote / `(` / `[` on the right
  (`Scanner.cpp:754-771`). The Rust guards the whole right-hand test with
  `Located::Normal`, where the C++ leaves the four character comparisons unguarded;
  these are equivalent because `INVALID_CHARACTER` is `0x100` and the only other
  value `character` can hold on a non-`NORMAL_CHAR` return is `,` or `-`.
  `abs ('2.5')` gives `ABS 2.5` **with** a blank, and the code comment says so.
* **Blank significance through comments**, the case the brief did not ask for and the
  one most likely to be wrong: `abs/*c*/(2.5)` is a call, and `abs /*c*/ (2.5)`,
  `abs/*c*/ (2.5)` and `abs /*c*/(2.5)` are all concatenations printing `ABS 2.5`.
  All four match, including the blank's span landing after the comment in case 3.
* **`]` on the left of the rule**: `say a[1] b` prints `10 Q`, `say a[1]b` prints
  `10Q`; the token streams differ by exactly one `Blank`.
* **Continuations**: `,` and `-` both, including a continuation that crosses a `--`
  comment line, a continuation onto an empty line ending the clause, a continuation
  with no next line being consumed (this is the one path where `run`'s final
  end-of-file `Eoc` is actually needed), `x =,\ny` meaning `x = y`, `1 =,\n= 1.0`
  being a strict comparison, and `5 -\n- 2` being 3.
* **Span arithmetic** across LF, CRLF, bare CR, an empty line, a `;` mid-line, a token
  adjacent to a terminator, and a last line with no trailing terminator. All correct.
  Cross-line spans do occur (a `=` continued onto the next line spans `25..29`), as
  does the one overlapping span the report documents.
* **`Eoc` model**: one per `;` / uncontinued line end / end of file, collapsed, no
  trailing empty clause. `;;;`, `""`, `"\n\n"` and `/* just a comment */` all produce
  zero tokens; `say 1;;say 2;` produces two clauses.
* **Literal decoding**: `'it''s'`, `"a""""b"` -> `a""b`, `''`, non-UTF-8 bytes, odd
  leading hex group (`'a'x` -> `0A`), odd leading binary group (`'101 0101'b` -> `0x55`),
  tab as a group separator, multiple blanks between groups, the marker suppressed by a
  following symbol character (`'a'xy`, `'41'X5`), abutted hex literals, and a doubled
  quote inside a hex literal correctly reaching 15.3 because the packer sees the raw
  slice with its length *not* reduced by the doubling — matching the C++. Spans cover
  the marker.
* **Both literal packers** match the C++ including its asymmetry: hex skips whitespace
  once before each nibble pair, binary skips it inside the bit loop.
* **`scanSymbol`'s exponent backup** in its exotic forms: `1e+5e+5` scans as
  `1E`, `+`, `5E+5` and the oracle agrees (`Nonnumeric value ("1E")`); `1e+5x` scans as
  `1E`, `+`, `5X`; `1e+` at end of line backs up.
* **Symbol classification** and the 250/251-character boundary
  (`MAX_SYMBOL_LENGTH = 250`, `LanguageParser.hpp:458`).
* **`characterTable`** (`Scanner.cpp:97-116`, non-EBCDIC) is exactly
  `! . ? _ 0-9 A-Z a-z`, matching `translate_char`, so the infallible
  `str::from_utf8` on a symbol's bytes cannot panic.
* **`INTEGER_CONSTANT`'s unobservability**, which the report asserts: its only consumer
  is `LanguageParser.cpp:2371`, and `say 1~class` gives `The String class` for both a
  small and a 22-digit integer. Correctly not reproduced.
* **`LITERAL_HEX`/`LITERAL_BIN`** appear only in `Token.hpp`'s enum. Correctly not
  reproduced.
* **`Ctrl-Z`, alternative logical-not bytes `0xAA`/`0xAC`, tabs as blanks, form feed
  and NUL as 13.1.** All match.
* **`checkMarker` is a prefix match** including the degenerate empty marker.
* **Interning**: `Cow` path exactly as the brief specifies, no allocation on a hit
  when the text is already upper; `intern` is idempotent so the scanner's `RESOURCE`
  and `END` reuse the keyword ids; `index_of` is linear and unsorted with position as
  meaning, pinned by a test that asserts `VALUE` is index 46 in `sub_keywords` and 7 in
  `parse_options`.
* **Test honesty**: I re-ran all 14 reproducible cases of
  `every_error_the_scanner_raises_matches_the_interpreters_number_and_line` against
  `rexxc` myself; all 14 are exactly what the oracle prints. The tests would not pass
  with the logic subtly wrong — `block_comments_nest` fails if nesting is dropped,
  the blank tests pin both halves of the rule (the left half in
  `say "a"||-` and in `blank_significance_depends_on_the_class_of_the_token_before_it`,
  the right half in `a_blank_before_an_operator_is_discarded`), and the operator span
  `2..5` in `1 = = 1.0` pins that `next_special` extends the span rather than merely
  succeeding. The panic sweep asserts only non-panicking and says so, and its count
  assertions prove it ran.

---

## Cannot verify from the diff

* Test count (56 in `rexx-parse`, 232 workspace-wide), clippy `-D warnings` cleanliness
  and the absence of `unsafe`. Pre-verified by the caller; not re-run.
* The process claim that `tests/scanner.rs` was written first and observed failing with
  `E0432: unresolved import rexx_parse::scan`. Not observable from the tree.
* Whether the per-clause allocation in MIN-1 is measurable in wall-clock parse time.
  I counted the allocations (2,402 for `CoreClasses.orx`) but did not benchmark; Task
  3.10 owns throughput.
* The report's claim that the eager-scan deviation is "exactly one file" in
  `rust/corpus-l1`. I reproduced exactly that one file, so the claim holds for that
  corpus; I did not attempt to bound it over any corpus I did not run.
