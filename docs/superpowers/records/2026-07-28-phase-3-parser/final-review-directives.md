# Final whole-branch review, Phase 3: directives, block structure, error gate

Reviewer slice: `directive.rs`/`directive/`, `block.rs`/`block/`, `error.rs`/`error/`,
`tests/errors.rs`, `rust/corpus/errors/parse-errors.tsv`, `rust/corpus/README.md`.
Read-only review; two other reviewers active in the same worktree.

Status: COMPLETE (2026-07-30). Verdict: no Critical findings; one Important
(F1, a false provenance claim in the corpus header, decision unaffected); one
Minor (F2, a wrong-line C++ citation); nits (F3). Everything else checked out
against the live oracle.

## Findings so far

### F1 (Important): the corpus header's install-class provenance is wrong; both codes fire DURING translation

The header of `rust/corpus/errors/parse-errors.tsv` claims, for the nine
install rows: "rexxc rejected it, but after translation finished, while
installing the package" and "Confirmed in the C++ tree at
LibraryDirective::install, reached from PackageClass::processInstall, which
runs from a called stub and therefore after translation is complete."

Both claims are false for these nine rows:

* `LibraryDirective::install` (interpreter/instructions/LibraryDirective.cpp:124)
  is the `::REQUIRES x LIBRARY` install path. None of the nine rows is a
  `::REQUIRES`; they are all `::ROUTINE`/`::METHOD ... EXTERNAL`.
* The actual raise sites are inside the parser, mid-translation:
  * 90.999 `Error_External_name_not_found_routine`: raised by `syntaxError` in
    `routineDirective`, interpreter/parser/DirectiveParser.cpp:2688 (LIBRARY
    form) and :2732 (REGISTERED form), immediately after
    `PackageManager::resolveRoutine` returns NULL.
  * 98.903 `Error_Execution_library`: raised by `reportException` in
    `PackageManager::getLibrary` (interpreter/package/PackageManager.cpp:214),
    reached at parse time from `routineDirective` via
    `resolveRoutine(library, entry)` (the two-arg overload,
    PackageManager.cpp:376) and from `LanguageParser::createNativeMethod`
    (DirectiveParser.cpp:1379-1381) via `PackageManager::resolveMethod` for
    the `::METHOD ... EXTERNAL "LIBRARY ..."` rows.
* Confirmed live: `rexxc` (translate only, no install step) rejects
  `nop\n::routine r external "LIBRARY x"` with `Error 98.903: Unable to load
  library "x"` at line 2, and `::routine r external "registered x"` with
  `Error 90.999: Unable to find external routine "R"` at line 1. If these
  fired only after translation from an install stub, rexxc could not have
  reported them at all.

What survives: the *decision* to accept these nine rows in the Rust parser is
still defensible, on the header's other, correct evidence line: the failure
depends on the machine environment (whether library "x" is loadable), never on
the program text. A clean-room parser with no package manager cannot
reproduce it. But the recorded justification is not the real one, and "after
translation is complete" is exactly the kind of claim Phase 4 would build on
(e.g. modelling when install-time errors interleave with execution).

Run-time consequence: in the C++, 98.903/90.999 abort translation mid-file;
directives after the failing one are never parsed. The Rust parser parses past
that point. For single-error corpus rows this is invisible; for multi-error
inputs it is the recorded eager-scan deviation shape, and those are excluded.
So corpus verdicts are unaffected; the header text is the defect.

### F2 (Minor): one wrong-line C++ citation, same class as the expr.rs four

`rust/crates/rexx-parse/src/directive.rs:1224` cites
"`LanguageParser::scanSymbol(RexxString *)` (`Scanner.cpp:1650`)". The actual
definition is `interpreter/parser/Scanner.cpp:1792`; line 1650 sits inside the
unmatched-quote error handling of a different function (the
Error_Unmatched_quote_single/double raise). The comment's substance
(seven-way classification, the E+digits exponent wrinkle admitting `a.e+5`)
was checked against the real scanSymbol at 1792 and is accurate; only the
line is wrong.

### F3 (Nits): six citations point 1-5 lines off, at the right function's own
doc comment or an adjacent line; none lands in a different function

* block.rs:183 and block/tests.rs:424: `addCompound` cited at
  LanguageParser.cpp:2124; definition 2127 (2124 is its doc comment).
* block.rs:542: `matchLabel` at BaseDoInstruction.cpp:172; definition 174.
* block.rs:657: END arm cited at LanguageParser.cpp:1500; `case KEYWORD_END:`
  is 1505 (1500 is the tail of the previous arm).
* block.rs:284: InstructionParser.cpp:4505; the `(var)` indirect branch is
  4506-4507.
* block.rs:180: `variables` at LanguageParser.hpp:502; actual 501.
* block.rs:649: DoInstruction.hpp:105 for LOOP_BLOCK; actual 104.

### Citation audit summary (priority 4)

Verified every C++ citation in directive.rs and block.rs against the tree
(34 named-function citations mechanically, ~20 line-only content citations by
reading the cited lines). All fifteen directive-parser function citations
(nextDirective 64, checkDirective 154, hasBody 189, parseClassReference 287,
classDirective 334, methodDirective 629, optionsDirective 948,
decodeExternalMethod 1403, attributeDirective 1457, constantDirective 1854,
annotateDirective 1940, processAnnotation 2209, resourceDirective 2266,
routineDirective 2565, requiresDirective 2779) are line-exact. The
LanguageParser citations (translateBlock 1176, topBlockInstruction 1772,
flushControl 1919, isExposed 1991, addCompound content, expose 2218,
autoExpose 2232, localVariable 2250, getRetriever 2507, addClause 2544,
translateConstantExpression 1725, blockError 4180, misplaced-label 1224-1244,
Incomplete_do_then 1370, ELSE-label 1430, pushDo 1188) are exact.
Content-claims checked and true: Utilities.hpp:52 toUpper is
`isLower(c) ? c & ~0x20 : c`; DirectiveParser.cpp:831/834 getRetriever-before-
checkDirective order for DELEGATE and the 892 after-order for the accessor
pair; 1656's own C++ comment really says "so errors get diagnosed on the
correct line"; 1091 falls SUBDIRECTIVE_ERROR through to SYNTAX for NOVALUE;
2606 really passes Error_Invalid_subkeyword_class (25.901) for a second
EXTERNAL on ::ROUTINE; 1886-1907/2230-2251 are the shared signed-number form
with the two different error codes; StringClass.hpp skipBlanks/skipNonBlanks
test exactly space and tab; the header cites (LanguageParser.hpp:306-312,
:504, RexxInstruction.hpp:137, OtherwiseInstruction.hpp:55,
IfInstruction.hpp:64, DoInstruction.hpp:82) are exact. Exceptions are F2 and
the F3 nits only.

### Quoted-oracle audit (priority 3): every quoted line reproduced

Beyond the tests/errors.rs quotes already verified above, all remaining
quoted oracle output in the slice reproduces verbatim against build/bin/rexxc:

* error.rs:31-32 (7/7.1 select example) and error/tests.rs all five quoted
  blocks: 8.2 "ELSE has no corresponding THEN clause." at line 2; 10.1 "END
  has no corresponding DO, LOOP, or SELECT." at line 3; 25.901 major "Invalid
  subkeyword found." + sub naming "JUNK"; 7.1 "SELECT on line 3 requires
  WHEN."; 36.901 "Left parenthesis \"(\" in position 5 on line 1 requires a
  corresponding right parenthesis \")\"." All byte-identical.
* directive/tests.rs:29-31 and 567/580: `::method m class` with no ::CLASS is
  99.905 at rc 157; `::routine r external "LIBRARY x"` is 98.903 at rc 158;
  `::resource d end stop` body closed by STOP is rc 0. All confirmed,
  including the exact return codes.
* directive/tests.rs:1436-1437: `::method m abstract` + 3 blank lines +
  `  return 1` prints "5 *-* return 1" / 99.933 line 5, confirming the
  offending-clause (not directive-clause) line convention.
* directive.rs:161: a ::RESOURCE body closed by `::end` is 99.943 ("Missing
  ::RESOURCE end marker \"::END\"...") since the default marker comparison is
  case-sensitive. Confirmed.

No fabricated oracle lines found anywhere in the slice.

### Option-grammar differential probes (priority 5): 35 probes, zero divergences

Read all of directive.rs (1272 lines) and probed rexxc with combos chosen to
be absent from obvious test rows: repeated ::OPTIONS options (digits twice,
form twice, numeric twice, all rc 0, matching Rust's push-without-dedup),
LIBRARY/NAMESPACE mutual exclusion both ways (25.904), duplicate/conflicting
class options (abstract abstract, public private, both 25.901), method option
guards (class class, abstract+external both orders, external twice, all
25.902 with the ::ROUTINE-only 25.901 quirk correctly NOT applying to
::METHOD), attribute guards (get set / set get / external twice /
delegate+external, all 25.925), REGISTERED admitted only on ::ROUTINE
(99.917 on ::METHOD/::ATTRIBUTE, mixed-case "Registered" still passing on
::ROUTINE to install-time 90.999), external word-count and whitespace-only
specs (99.917), `::resource d end` with no marker (19.921), namespace REXX in
mixed case (99.944), empty ::ANNOTATE and empty ::OPTIONS (rc 0),
`::constant c + 5` accepted as "+5", `-1e` failing the number test (19.916),
paren value on ::ANNOTATE rejected 19.923 where ::CONSTANT allows it,
`::class c inherit ,` (19.908), literal class ref with embedded colon
(rc 0, one name), the `a.e+5` exponent wrinkle (rc 0) vs `1e+5` (99.925),
and `::requires "x" library` followed by a plain clause (99.916 from the
next-directive gate, which is the caller's job in Rust too). Every oracle
answer matches what the Rust code paths I read produce. No silently-ignored
or silently-accepted option found.

### ::RESOURCE (priority 6): terminator rule and body bytes both faithful

The C++ rule (LanguageParser::checkMarker, LanguageParser.cpp:959) is a
case-sensitive PREFIX memcmp against the raw line, and the Rust check_marker
(scanner.rs:1065-1068) is the same test. Probed rexxc on ten edge shapes and
the Rust scanner (rust/target/release/scan-check) on the six not already in
the corpus; all agree:

* marker STOP closed by line "STOPPING? prefix" (prefix match fires; program
  then fails 99.916 on the trailing nop in both);
* lowercase body "stop" does NOT close marker "stop"-from-symbol because the
  symbol upcases to STOP (99.943 both), while a literal marker keeps case;
* leading space defeats the prefix (" ::END" leaves the resource open, 99.943
  reported against the directive's own line in both);
* "::END trailing junk" closes (prefix);
* `::resource d; say 'x'` skips the rest of the directive's line (rc 0 both,
  corpus row 78 pins the same);
* clause continuation `::resource d ,` / `end stop` works in both;
* an empty literal marker `end ""` closes on the first body line in both;
* an empty literal name is accepted in both; non-UTF-8 body bytes are rc 0.
The scanner-side shape gate (3 or 5 real tokens, resource/end symbols,
name and marker Symbol-or-Literal) was compared clause-condition by
clause-condition with the directive parser's own checks; they enforce the
same four conditions, so the body-lookup `expect` in directive.rs:1159-1164
is not reachable without a prior error. Corpus rows 68, 78, 145, 223, 305,
310, 377, 432, 488, 595, 598, 640, 657, 661, 867, 902, 1025, 1062, 1071
cover the rest of the space.

### Block structure and the twenty errors (priority 7): faithful; no divergence found

* The twenty errors are 14.1/.2/.3/.4/.5/.901, 47.2/.3/.4, 10.1/.2/.3/.4/.7,
  7.1, 7.2, 8.2, 9.2, 18.1, 18.2. Every one has 1-12 corpus rows (counted per
  pair), and the corpus gate checks number, sub AND line for each, so all
  twenty are reachable and line-pinned; my 36-row oracle sample confirms
  corpus accuracy.
* Thirteen structural probes of shapes plausibly not in the corpus, all
  matching the Rust logic as read: END with a name closes THROUGH an
  OTHERWISE to the labelled SELECT (rc 0 / 10.4 on mismatch, reported against
  the END's line); WHEN after OTHERWISE is 9.1 (OtherwiseInstruction
  overrides only isBlock, so topBlockInstruction stops there and
  finds a non-SELECT; Rust Control::Otherwise is in is_block() and
  enclosing_select answers None, instruction.rs:2600 raises 9.1); a second
  OTHERWISE and OTHERWISE inside DO are 9.2; END directly closing a bare
  THEN/ELSE is 10.1 (the C++'s Error_Unexpected_end_then/else are
  structurally unreachable, matching block.rs:661-674's argument and the
  oracle probes); label directly before END in a DO is 47.2 at the label's
  line; label after a finished THEN is rc 0 unless an ELSE follows (47.3);
  `when 1 = 1` / `then nop` as separate clauses inside SELECT is rc 0.
* The misplaced-label mapping was compared arm by arm with
  LanguageParser.cpp:1224-1244: withinIf(IFTHEN, ELSE)=47.3,
  withinSelect(SELECT, SELECT_CASE, WHENTHEN, ENDWHEN, OTHERWISE)=47.4,
  isControl blocks (DO/LOOP)=47.2, ENDTHEN deliberately excluded on both
  sides (handled at the ELSE). Exact match.

### Test-quality pass (priority 8): no cannot-fail test found

Read all of tests/errors.rs, error/tests.rs, block/tests.rs and
directive/tests.rs. Every multi-assertion test asserts on distinct inputs or
distinct fields; the one historical shared-#[test] hazard is explicitly
split in error/tests.rs (its comment records why). Discriminating design is
present where it matters: floors on corpus counts so an emptied file cannot
pass vacuously, the exact install==9 count, the negative direction for every
absorbing rule (label_colon_deviation near-misses, the 40-subdirective
refusal table, `a_byte_on_a_terminator_belongs_to_the_line...` which is the
one line assertion an off-by-one cannot also satisfy), and pinned counts for
CoreClasses (347 = 32/303/12) and StreamClasses (7/139/5/2). Twelve more
surprising "measured" claims probed against rexxc (19-digit DIGITS is 26.5,
`::method 3` and `::method .a` rc 0, `::annotate public` 25.928,
`digits " 9 "` and `digits 1e2` rc 0, `-"5"` 19.916, `"aB"` rc 0 vs `"a b"`
99.925, the 250/251-byte name boundary, `when 2, 3` case list rc 0); all
match.

### Coverage boundary

Checked: everything in the eight priorities. Rust-side behavior for
non-corpus probe inputs was established by reading the specific code path
(directive.rs and block.rs read in full) and, for ::RESOURCE shapes, by
executing the shipped scan-check binary; there is no full-parse CLI harness,
so the novel probe inputs were not run through the Rust parser end to end.
Not checked: clause.rs/instruction.rs internals beyond instruction.rs:2600
(other reviewers' slices); the seventeen-shape compound-cache claims beyond
the eleven probed; line numbers for the directive-option probes (pair-level
only; corpus rows carry the line gate). git status clean throughout; no
files appeared under support/ or anywhere else.

## Check log

* Corpus sample vs oracle (priority 1): 36 rows, two seeded random batches of
  18 (10 translation + 3 install + 5 accepted each; the 3 install overlap
  across batches since there are only 9). Each program unescaped, run through
  `( ulimit -v 1048576; build/bin/rexxc FILE )`, and major.sub + line
  extracted from the real output. Result: 36/36 match the recorded class,
  expect, and line. Zero mismatches. Rows covered include both label-colon
  rows' neighbours (18.1@5, 18.2@3), 99.916, 20.923 (the mutation-added row),
  and 6 of the 9 install rows.
* F1 addendum: the false "after translating it, while installing the package"
  claim is repeated in rust/crates/rexx-parse/tests/errors.rs (Class::Install
  doc, lines 64-65, and the comment at lines 433-435 "the directive parsed,
  and then the interpreter could not bind it"). The GATE ITSELF is correct:
  it reads the per-row class field, never a code prefix
  (a_rejection_that_is_not_a_translation_error_is_accepted, and the exact
  install==9 count in the_corpus_holds_at_least_the_rows_it_was_measured_with
  stops silent growth). Property honoured; provenance text wrong.
* Quoted-oracle check, tests/errors.rs: ALL VERIFIED against the live oracle.
  (a) label-colon: `if 1 = 1` / `then: nop` reproduces "Error 35 ... line 2:
  Invalid expression." / "Error 35.1: Incorrect expression detected at ':'".
  (b) eager-scan: `say )\n\n'unclosed\n` reproduces "Error 37 ... line 1:
  Unexpected \",\", \")\", or \"]\"." / "Error 37.2: Unmatched \")\" in
  expression.". (c) all seven INTERPRET-only codes measured via
  condition('o'): 99.908/912/915/923/924/914 and 47.1 all match, messages
  byte-identical ("INTERPRET data must not contain EXPOSE.", errortext
  "Unexpected label.", message with found "X", additional=X), and position is
  the interpret instruction's own line as claimed. No fabricated oracle lines
  in tests/errors.rs.
* Located 98.903/90.999 definitions (Error_Execution_library = 98903,
  Error_External_name_not_found_routine = 90999) in
  interpreter/messages/RexxErrorCodes.h; traced raise sites; probed rexxc
  live on two install rows. Result: F1 above.

