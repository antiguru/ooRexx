# Final whole-branch review: source retention, scanner, tokens, clause splitting

Reviewer slice: `source.rs`, `scanner.rs` (+submodules), token type / `TokenCursor`,
`clause.rs`, and their tests. Branch `plan/rust-rewrite`, 2026-07-29.

Status: IN PROGRESS. Appended after each check; if this file ends abruptly the
review was killed mid-way and everything above the cut is valid.

## Findings so far

(none yet; checks begin below)

## Check log

### Check 1: locateToken / sourceNextToken vs port (in progress)

Read `rust/crates/rexx-parse/src/scanner.rs` (1190 lines) in full and
`interpreter/parser/Scanner.cpp` lines 137-1243 (nextSpecial, scanComment,
locateToken, sourceNextToken).

Verified equivalent, case by case:

- locateToken blank/tab handling, significant-blank early return: match.
- `--` line comment: truncate + CLAUSE_EOL: match. One positional difference,
  see Minor M1 below: C++ `truncateLine` shortens the line so the EOL
  terminator's location is at the `--`; Rust `truncate_line` moves the offset
  to the ORIGINAL line end, so the empty Eoc span sits at the untruncated
  line's end. No token stream difference, only the (empty) Eoc span position.
  Nothing downstream reads an Eoc span for error positions (errors use clause
  start), so cosmetic.
- Continuation `,`/`-`: save position, scan blanks/comments/`--` to line end;
  back up and return NORMAL if anything real found; at line end consume the
  continuation, step line, return SIGNIFICANT_BLANK if blanks significant:
  match, including `--` inside the lookahead (truncate + continue) and
  comments spanning lines.
- Continuation at end of file: C++ leaves `character` = ','/'-' and falls out
  to CLAUSE_EOL; Rust breaks and falls out to ClauseEol. Same token stream.
- Significant-blank confirmation: C++ tests `inch=='"'|'\''|'('|'['` without
  gating on NORMAL_CHAR, but on EOL/EOF `inch` can only be INVALID_CHARACTER
  or ','/'-' (set in the continuation path), none of which are in the set, so
  Rust's `Located::Normal(c) => ... , _ => false` is equivalent. Checked the
  C++ control flow specifically for this.
- Blank token span: both record position AFTER locateToken, 1 char. Match.
- CLAUSE_EOL token: C++ empty location at currentLength; Rust empty span at
  line end, with debug_assert offset==len. Match (modulo M1).
- Operator dispatch table `) ] ( [ , ; : ~ + - % / * & | = < > \ 0xAA 0xAC`:
  compared case by case against Scanner.cpp:799-1161 including nesting order
  of nextSpecial calls (e.g. `<` tries `<` then `=` then `>`; `\` tries `=`
  then `>` then `<`, with strict variants inside). All match. Rust merges the
  `\` and 0xAA/0xAC cases; C++ has them as two identical switch arms. Same
  operators produced.
- CHECK_ASSIGNMENT: C++ macro checks a following `=` via nextSpecial for
  + - % // / ** * && & || | ; Rust `check_assignment` called at exactly those
  spots. `=` itself correctly NOT an assignment shortcut in both.
- Invalid character: C++ builds UTF-8 substitution text for error 13.1; Rust
  raises 13.1 with no substitution, per project rule (number gated only).
- nextSpecial: blanks insignificant, comments skipped, so `1 = = 1.0`,
  `1 =/*c*/= 1.0`, `=` + continuation scan as `==`: structure matches C++
  (locateToken(false) then match target).
- scanComment: nesting level, step-2 open, `*/` and `/*` recognition via
  get-then-check: match. C++ records opening line only for the 6.1
  substitution, which the port omits by design; 6.1 raised at end of file:
  match.

Still to check: scanSymbol (Scanner.cpp:1239) and scanLiteral (Scanner.cpp:1611)
against the port; hex/binary packers (Scanner.cpp:420/574).

CORRECTION to the `--` bullet above: C++ `truncateLine()` is
`lineOffset = currentLength` (LanguageParser.hpp:162), i.e. it moves the
offset to the line end exactly as the Rust `truncate_line` does; the line is
not shortened and `currentLength` is unchanged. The Eoc span positions match
exactly. M1 withdrawn; there is no finding here.

### Check 1 continued: scanSymbol

Compared Scanner.cpp:1239-1602 against `Scanner::scan_symbol`.

- State machine (Start/Digit/SPoint/Point/E/ESign/EDigit/Excluded)
  transition-by-transition: match, including E staying in E on non-digit
  symbol chars (sign handled at loop bottom in both).
- dot counting at loop top: match.
- Exponent sign path: C++ guards with `haveNextChar()`, which
  LanguageParser.hpp:159-160 defines IDENTICALLY to `moreChars()`
  (`lineOffset < currentLength`), so the guard is always true there; the Rust
  comment stating exactly this is correct, and the Rust fallthrough (step,
  find no symbol char, back up to the sign) lands on the same final position
  as the C++ early break. `1e+5` symbol vs `1e+x` -> `1E` + operator: same in
  both.
- eoffset back-up after Excluded: C++ `eoffset != 0`, Rust `Option`: same.
- Length check: MAX_SYMBOL_LENGTH = 250 confirmed at LanguageParser.hpp:458;
  error 30.1 in both. Rust checks after scanning, C++ after building the
  value; same observable.
- Classification: Dummy (lone dot), Constant (leading digit), leading-dot
  Constant-vs-DotSymbol via state==Excluded, Variable/Stem/Compound via
  dotCount and trailing dot: all match. C++ tests the trailing dot on the
  upcased value, Rust on source bytes; '.' is unchanged by upcasing, so same.
- C++ additionally computes INTEGER_CONSTANT numeric subtype
  (all-digit, <= REXXINTEGER_DIGITS, no leading zero); the Rust token has no
  numeric subtype. This is a representation choice for later phases, not a
  scan-behaviour difference; noted as an observation only.
- Symbol chars: is_symbol_char matches characterTable semantics (a-zA-Z0-9
  .!?_, nothing >= 0x80); ASCII-only claim verified against
  Scanner.cpp:60-134 table (all 0 above 0x7F).

### Check 1 continued: scanLiteral

Compared Scanner.cpp:1611-1780 against `Scanner::scan_literal`.

- Delimiter loop with doubled-quote counting: structurally identical,
  including the break conditions (line end right after delimiter, or next
  char not the delimiter).
- Unterminated literal: 6.2 for ', 6.3 for " (error numbers to be
  cross-checked against the message table, below).
- Hex/bin marker: C++ `!isSymbolCharacter(getNextChar())` relies on reading
  the line terminator byte one past the content; Rust `following_char()`
  returns None there, and None -> not a symbol char -> marker accepted:
  equivalent, and the following_char doc comment describes exactly this.
- Value construction incl. doubled-quote collapse: match.
- Span ends after the x/b marker when present, in both.

### Check 1 continued: hex/binary packers and error numbers

packHexLiteral (Scanner.cpp:420) / packBinaryLiteral (Scanner.cpp:574) vs the
Rust free functions: right-to-left grouping validation (blank at either end,
group multiple of 2 resp. 4), character count arithmetic, and the packing
loops all match. The asymmetry is faithful: hex skips whitespace only at the
top of each output byte (valid because hex groups are even, so whitespace
never splits a byte), binary skips inside the bit loop (4-bit groups can
split a byte). Both C++ and Rust have it that way round.

Error numbers hard-coded in the port, all confirmed against
interpreter/messages/RexxErrorCodes.h:
6.1 comment, 6.2 single quote, 6.3 double quote, 13.1 invalid char,
15.1 hexblank, 15.2 binblank, 15.3 invhex, 15.4 invbin, 15.5 invhex_group,
15.6 invbin_group, 30.1 name too long, 99.943 missing resource end. All
correct.

VERDICT Check 1: locateToken/sourceNextToken/scanSymbol/scanLiteral/
scanComment/nextSpecial/packers are a faithful structural port. No behavioral
divergence found. The two documented departures (eager scan, collapsed Eoc)
are as described.

### Check 1 continued: ::RESOURCE body handling

Compared `scan_resource_if_directive`/`check_marker` against
DirectiveParser.cpp:2266 (resourceDirective) and LanguageParser.cpp:939/959
(conditionalNextLine / checkMarker).

- checkMarker is a prefix memcmp against the line start, length-guarded: Rust
  identical.
- conditionalNextLine: advance only if lineOffset != 0: Rust identical.
- Missing end marker: 99.943: confirmed Error_Translation_missing_resource_end
  = 99943.
- Shape matching: C++ takes name as symbol-or-literal, END subkeyword must be
  a symbol, marker value symbol-or-literal; Rust matches (token_value None for
  non symbol/literal rejects, END required to be Symbol). C++ raises errors
  for malformed directives (19.921 etc.); Rust deliberately leaves malformed
  clauses alone for the directive parser, per its doc comment. Consistent
  with the design split.
- C++ takes marker via token->value(): upcased for a symbol, verbatim for a
  literal; Rust token_value does the same (symbol name comes back from the
  interned upcased table).

### Check 2: citation audit (scanner.rs) - running list

Verified against the oracle:
- DirectiveParser.cpp:2266 resourceDirective: EXACT (function at 2266).
- LanguageParser.hpp:415 isSymbolCharacter: EXACT.
- Scanner.cpp:60 characterTable: EXACT.
- Scanner.cpp:1220 "numeric-symbol state machine": line 1220 is the
  SymbolScanState enum typedef, which is precisely the state machine: good.
- LanguageParser.hpp:152-162 scanning primitives: EXACT range.
- LanguageParser.hpp:159-160 haveNextChar==moreChars: EXACT, and the claim
  itself verified true (both are `lineOffset < currentLength`).
- MAX_SYMBOL_LENGTH = 250: confirmed LanguageParser.hpp:458.
Still to verify: LanguageParser.cpp:1009 (nextClause null-clause skip),
LanguageParser.cpp:764 (translate/firstLine), ProgramSource.cpp:448 and :594,
Token.cpp:95 (checkAssignment), GlobalNames::DEFAULT_RESOURCE_END == "::END".

Check 2 results (scanner.rs citations), completed:
- LanguageParser.cpp:1009 nextClause: EXACT (definition at 1009, the
  null-clause skip is inside it at ~1021).
- ProgramSource.cpp:448: EXACT; line 448 is the shebang check inside
  BufferProgramSource::buildDescriptors (fn at 348) that sets firstLine = 2.
- ProgramSource.cpp:594: EXACT; the `interpretAdjust == 0` guard in
  ArrayProgramSource::setup.
- GlobalNames DEFAULT_RESOURCE_END = "::END": confirmed
  (interpreter/memory/GlobalNames.h:257).
- Token.cpp:95 checkAssignment: function starts at Token.cpp:93; line 95 is
  its opening comment line inside the function. Points into the right
  function: fine.

FINDING (Minor) F1, scanner.rs:144-145: the `first_line` doc comment says
"`LanguageParser::translate` positions there (`LanguageParser.cpp:764`)".
Line 764 (`lineNumber = source->getFirstLine();`) is inside
`LanguageParser::initializeForParsing()` (defined at LanguageParser.cpp:751,
called from compileSource at :739), not inside `translate` (defined at
:1093). Same defect class the sibling found in expr.rs: right behaviour,
right line, wrong function name. The behavioural claim itself is true.
Fix: name `initializeForParsing` instead of `translate`.

### Check 3: token.rs (token type, keyword tables, TokenCursor)

Keyword tables RE-VERIFIED (the prior run's unconfirmed claim): all six
tables in token.rs compared entry-for-entry and in order against
interpreter/parser/KeywordConstants.cpp:
- directives (9, cpp lines 54-62): exact.
- instructions (35, cpp 68-102): exact.
- subKeywords (50, cpp 108-157): exact.
- conditionKeywords (12, cpp 333-344): exact.
- parseOptions (10, cpp 350-359): exact.
- subDirectives (40, cpp 365-404): exact.
The C++ file also holds two builtin-function tables not ported; the Rust doc
comment does not claim them, so no issue.

token.rs citations verified:
- Token.hpp:77 TokenClass, 19 members: exact, and TokenKind/Tag mirror the
  order exactly.
- Token.hpp:110-141 operator subtypes: order matches Operator enum exactly
  (Plus..Backslash including Abuttal/Concatenate/Blank in place).
- Token.hpp:595 isBlankSignificant: exactly SYMBOL|LITERAL|RIGHT|SQRIGHT,
  matching makes_blank_significant.
- Scanner.cpp:1527-1593 classification, Scanner.cpp:1546 INTEGER_CONSTANT:
  both point at exactly those lines (verified during Check 1 read).
- LanguageParser.cpp:2371 integer object construction: the
  isIntegerConstant/requestInteger block sits at 2368-2373: good.
- Numerics REXXINTEGER_DIGITS "9 on 32-bit, 18 on 64-bit": confirmed by
  Numerics.hpp:162 comment.
- Token.cpp:95 checkAssignment: fn at 93, line points inside it: fine.
- "TOKEN_PREFIX/POINT/CONTINUE/NULL appear only in the enum declaration":
  re-ran the grep over interpreter/ *.cpp *.hpp excluding Token.hpp: zero
  hits. Claim true.
- LanguageParser.cpp:450 ArrayProgramSource one-element wrap (cited in
  source.rs): exact.

Unverified measured claim noted (not checked yet, needs running the scanner):
SymbolTable doc's "CoreClasses.orx holds 8,118 symbol occurrences over 526
distinct upcased symbols, and StreamClasses.orx 2,121 over 273".

TokenCursor: peek/advance/peek_real/advance_real/start/end reviewed; range
confinement is upheld (peek bounds at range.end, peek_real re-bounds after
skipping blanks). Forward-only design documented and consistent.

### Check 4: clause.rs (split_clauses, Clause, ClauseCursor, split_before)

Logic review:
- split_clauses respects the Scanned::tokens invariants and degrades sanely
  on an unterminated slice. Label peeling (`a: b: nop` -> three clauses)
  matches the C++'s trim/reclaim loop behaviour, which re-enters
  nextInstruction and re-runs the label check.
- The "colon at start+1 exactly" claim is sound: a Blank token is only
  emitted when the next real character is a symbol char, quote, `(` or `[`,
  and `:` is none, so no Blank can separate label and colon; C++ uses
  nextToken (not nextReal) and would also not see a label if one could.
- `here: ; nop` shape: label clause pushed, `start` reaches `limit`, loop
  exits without an empty clause. Correct.
- split_before non-partition semantics and assertions look right.

clause.rs citations, all verified:
- LanguageParser.cpp:1009 nextClause: exact (checked earlier).
- InstructionParser.cpp:150-176 label arm of nextInstruction: exact range,
  including nextToken-not-nextReal and the interpret 47.1 error.
- InstructionParser.cpp:2809 labelNew "sets the end unconditionally": fn at
  2792, line 2809 is precisely the colonToken location/setEnd code. Good.
- IfInstruction.cpp:58-66 RexxInstructionIf setEnd from THEN token start:
  exact.
- ThenInstruction.cpp:76 setLocation(token->getLocation()): exact.
- Clause.cpp:138 RexxClause::trim moves only the start: exact, confirmed
  setStart-only.
- LanguageParser.cpp:1329-1360: inside translateBlock (fn at 1176), and is
  exactly the THEN-consumption code raising Error_Then_expected_if/when.
- Error numbers: 18.1 = Error_Then_expected_if, 18.2 = Error_Then_expected_when,
  8.1 = Error_Unexpected_then_then (raised InstructionParser.cpp:448 in the
  nextInstruction keyword dispatch, as the PendingThen comment claims),
  47.1 = Error_Unexpected_label_interpret. All confirmed in RexxErrorCodes.h.

### Check 5: line-boundary knowledge leak (priority 3) - starting

Check 5 result: CLEAN. Line terminator bytes (\r, \n, 0x1a) appear outside
source.rs only in test inputs and in a differential.rs join("\n") for a
failure message. No whole-text accessor exists; non-test consumers reach the
source only via line()/line_span()/line_of()/span_bytes()/join_span()
(scanner via line/line_span, error.rs via line_of). Nobody re-derives line
boundaries.

### Check 6: byte-vs-str discipline (priority 5)

In this slice: retained text is Vec<u8>, lines are &[u8], literal values are
Box<[u8]>. The only source-bytes-to-str conversion is scan_symbol's
from_utf8, guarded by the characterTable argument (symbols are ASCII by
construction; exponent signs are ASCII too) - sound. SymbolTable::intern
takes &str but its non-keyword caller is scan_symbol only. token_value
round-trips symbol names via as_bytes. tests/scanner.rs:30 uses
String::from_utf8 with an expect on known-ASCII test input: acceptable.
No violation found in the slice.

### Check 7: oracle spot-checks of "measured" comments (priority 4) - starting

### Check 7 results: oracle spot-checks of "measured" comments

All run as `( ulimit -v 1048576; build/bin/rexx ... )` on probe files in the
scratchpad. CONFIRMED, exactly as the comments state:
- scanner.rs continuation blank: `say "a"-`/`"b"` prints `a b`;
  `say "a"||-`/`"b"` prints `ab`.
- scanner.rs two-sided blank: `abs (2.5)` -> `ABS 2.5`, `abs(2.5)` -> `2.5`.
- scanner.rs nextSpecial: `1 = = 1.0`, `1 =/*c*/= 1.0`, `=-` continuation all
  print 0; `1 = 1.0` prints 1.
- scanner.rs scanSymbol: `say 1e+5` -> `1E+5`; `1e+y` -> 41.1
  `Nonnumeric value ("1E")`.
- scanner.rs MAX_SYMBOL_LENGTH: 250-char name works, 251 -> 30.1.
- scanner.rs module doc eager-scan departure: line 1 `say )` + line 3
  `x = 'unclosed` -> 37.2 on line 1 (unclosed literal never reached).
- ResourceBody: unmatched quote + unclosed comment inside a resource body:
  rexxc rc 0. `::resource data; say 'x'`: rc 0. End marker prefix match:
  body line `STOPPING? no, prefix match` ends a `'STOP'` resource (rc 0;
  first attempt failed only because my probe put code after the directive,
  which is 99.916 and unrelated). Missing marker: 99.943 reported against
  the directive line.
- first_line: `#!` file runs; `interpret "#! nothing here"` -> 13.1 on
  '23'X.
- source.rs Interpret rules: `interpret` with embedded '0a'x -> 13.1 on
  '0A'X; `interpret ""` accepted and runs on; c2x of '1a'x inside a literal
  prints 1A.
- clause.rs Clause::span: `nop;` traces with its semicolon, `say 1 ;` keeps
  the blank before `;`, `say 1;   ` drops trailing blanks, `say 1 -- c`
  keeps the whole comment, `here: ; nop` traces `here:` then `nop`.
- split_before: `if 1 = 1   then    say "a"` traces condition with all three
  trailing blanks, `then` bare, `say "a"` unindented-at-say; `if 1 = 1;` +
  `then` next line traces the condition without a semicolon.
- PendingThen: nop/if/blank/blank/nop reports line 5 with "IF instruction on
  line 2" substitution; nop/nop/if reports line 3 in both fields. 18.1
  confirmed; (18.2 for WHEN not probed, number confirmed statically.)
- ParseError::byte: `say 1,` + `'unclosed` -> 6.2 reported on line 1.

FINDING (Minor) F2, scanner.rs:147-149 (first_line doc): "Found by
differential testing: 494 of 790 files under `ootest/` and `samples/` open
with `#!/usr/bin/env rexx`". The population 790 reproduces exactly
(350 .rex + 409 .testGroup + 7 .testUnit + 20 .cls + 4 .frm), but TODAY the
count of files whose first line starts with `#!` is 492, and only 446 of
those open with the quoted spelling `#!/usr/bin/env rexx` (others:
`#!@OOREXX_SHEBANG_PROGRAM@` 36, `#!/usr/bin/rexx` 10). So the number is
off by 2 under the generous reading and by 48 under the literal reading of
"open with `#!/usr/bin/env rexx`". The load-bearing claim (hundreds of
files, all would be 13.1 without the skip) stands. This is the branch's
known defect class: a quoted measurement that is not quite true as written.

- SymbolTable doc counts VERIFIED by running the scanner via a scratchpad
  harness (path-dependency build, no repo changes): CoreClasses.orx
  8,118 symbol occurrences / 526 distinct, StreamClasses.orx 2,121 / 273.
  Exactly as the comment states.

### Check 8: tests that cannot fail (priority 6) - starting

### Check 8 results: tests

- src/token/tests.rs: FINDING (Minor) F3, see below.
- src/clause/tests.rs: substantive throughout; the label-with-gap cases
  (`here : nop`, `here /*c*/: nop`) genuinely pin the span-reaches-the-colon
  behaviour; the unterminated-slice test constructs the impossible input
  honestly. Token.hpp:580 isSymbolOrLiteral citation: EXACT. No cannot-fail
  assertions.
- tests/tokens.rs: substantive; the keyword-order test pins first/interior/
  last positions plus the shared-spelling (VALUE at 46/7) case. Good.
- tests/scanner.rs (1015 lines, read in full): high quality. The 18-case
  error table asserts number, sub-number AND clause-start line; the sweeps
  at the bottom assert their own case counts so they cannot silently shrink.
  No tautologies found.
- tests/sourceline.rs (structure + assertions skimmed): substantive,
  no tautologies found.

Resolved during this check: the post-loop `emit_eoc` in `Scanner::run` IS
reachable (not dead): `say 1,` at end of file takes SignificantBlank ->
ClauseEof, so `source_next_token` returns None with a non-Eoc last token and
only the final emit terminates the clause; the existing test
`a_continuation_with_no_next_line_is_simply_consumed` exercises exactly
this path. Comment on it is accurate.

FINDING (Minor) F3, rust/crates/rexx-parse/src/token/tests.rs
(`a_cursor_visits_only_its_own_range`): the final assertion
`assert_eq!(tokens[2].kind.tag(), Tag::RightParen)` can never fail: it
asserts a literal the test itself constructed four lines earlier, and the
`tokens` array is otherwise unused because `TokenCursor` never consults the
token slice (it is range-only). The assertion exists to use the variable and
implies a cursor-token relationship that does not exist. Suggest dropping
the array to ranges-only or asserting something the cursor actually did.

## Findings summary

Critical: none found.
Important: none found.
Minor:
- F1 scanner.rs:144-145: citation names `LanguageParser::translate` for
  LanguageParser.cpp:764, but that line is in
  `LanguageParser::initializeForParsing` (:751). Same class as the expr.rs
  citation defects. No behavioural consequence.
- F2 scanner.rs:147-149 and tests/scanner.rs:553-554: "494 of 790 files ...
  open with `#!/usr/bin/env rexx`" - population 790 reproduces exactly, but
  the current count is 492 with any `#!` first line and only 446 with the
  quoted spelling (36 are `#!@OOREXX_SHEBANG_PROGRAM@`, 10 `#!/usr/bin/rexx`).
  Off by 2 (generous reading) or 48 (literal reading).
- F3 token/tests.rs: one tautological assertion (above).

Observation (not a defect): INTEGER_CONSTANT numeric subtype deliberately
not reproduced, documented with a correct justification in token.rs.

## Coverage

Done: full read of source.rs, scanner.rs, token.rs, clause.rs and all five
test files in the slice; line-by-line comparison against Scanner.cpp
(locateToken, sourceNextToken, scanSymbol, scanLiteral, scanComment,
nextSpecial, both packers, characterTable), LanguageParser.cpp/hpp scanning
primitives, DirectiveParser.cpp resourceDirective, checkMarker,
conditionalNextLine; every C++ citation in the slice checked against the
oracle; all hard-coded error numbers checked against RexxErrorCodes.h;
six keyword tables re-verified entry-for-entry; ~30 "measured" claims
re-measured against build/bin/rexx / rexxc; SymbolTable's .orx counts
re-measured by running the scanner from a scratchpad harness.

Not done: the 18 rexxc error-table cases in tests/scanner.rs were verified
statically (numbers) and by 8 same-class oracle probes, not each
individually; tests/sourceline_oracle.rs, tests/samples.rs, tests/program.rs,
tests/tiling.rs were not reviewed (outside the slice's listed test files);
the 18.2 WHEN-shaped probe was not run (number verified statically);
expr/differential.rs byte handling was left to the expr reviewer.

Status: COMPLETE.
