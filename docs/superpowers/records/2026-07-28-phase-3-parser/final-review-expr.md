# Final whole-branch review: expression grammar and AST types

Reviewer slice: `rust/crates/rexx-parse/src/expr.rs` (+ `expr/tests.rs`,
`expr/differential.rs`), `rust/crates/rexx-parse/src/ast.rs` (+ `ast/tests.rs`),
`rust/crates/rexx-parse/tests/tiling.rs`. All files read in full. Every C++
citation checked was checked against the in-repo oracle tree, and every
quantitative comment claim listed below was re-probed against `build/bin/rexx` /
`build/bin/rexxc` under the `ulimit -v` wrapper. Probe files are in the session
scratchpad (`values.rex`, `expr-blank-probe.rex`, `errprobe*.sh`, `e*.rex`,
`runtime914.rex`).

## Critical

None found.

## Important

### I1. Three C++ citations point at the wrong lines of `LanguageParser.cpp`

No runtime effect, but this is the branch's tracked defect class (stated
measured facts that do not check out), and a reader following these citations
lands inside a *different* function, which is exactly what makes a stale
citation worse than none:

* `expr.rs:274` (doc of `parse_constant_expression`): cites
  `LanguageParser::parseConstantExpression` (`LanguageParser.cpp:3400`).
  The function is at **line 2632**. Line 3400 falls inside `parseMessage`.
* `expr.rs:255` (doc of `parse_paren_expression`): cites
  `LanguageParser::parenExpression` (`LanguageParser.cpp:3465`).
  The function is at **line 2695**. Line 3465 falls inside
  `parseVariableOrMessageTerm`'s region.
* `expr.rs:381` (doc of `need_variable`): cites
  `LanguageParser::needVariable` (`LanguageParser.cpp:3555`).
  The function is at **line 885**. Line 3555 falls inside `parseMessageTerm`.

Borderline in the same family: `expr.rs:1105` (doc of `qualified_symbol`)
cites `parseQualifiedSymbol` (`LanguageParser.cpp:3245`); the doc comment
starts at 3250 and the function at 3261, so 3245 lands in the tail of
`parseFunction`. Off by a whole function boundary, unlike the usual
comment-block slack.

Every one of these four functions' *described behaviour* is correct against the
real C++ (I read all four bodies); only the line numbers are wrong. The other
~30 citations I checked across expr.rs, ast.rs and the test files are accurate:
`Token.cpp:111` (precedence), `Token.cpp:189/194-201/252` (isTerminator),
`Token.hpp:521-538` (TERM_* bit values match Rust's `Terminators` constants bit
for bit, including all five composite sets), `Token.hpp:576` (isVariableOrDot),
`Utilities.hpp:52` (ASCII-only toUpper), `LanguageParser.cpp` 2725 / 2753 /
2812 / 2878 / 2924 / 3083 / 3168 / 3309 / 3369 / 3455 / 3500 / 3617 / 3701
(TERM_EOC in the cascade, exact) / 3719 / 3757 / 4264 / 2333 / 2352 / 2153 /
2184 / 782-784 / 2544 / 1319 / 1530, `LanguageParser.hpp:220/228`,
`RexxInstruction.hpp:107`, `DoInstruction.hpp:105`, `SelectInstruction.hpp:76`,
`SelectInstruction.cpp:222`, `IfInstruction.cpp:147`, `EndIf.cpp:154`,
`ElseInstruction.cpp:113/145`, `ThenInstruction.cpp:76`,
`KeywordConstants.cpp:52-63` (nine directive rows), and every error constant
(`RexxErrorCodes.h`: 19909, 20917, 20923, 20930, 31002, 31003, 35001, 35901,
35934, 36901, 36902, 37001, 37002, 37901 all match the Rust claims).

## Minor

* **M1.** `expr/differential.rs:208`: `SYNTAX_ERRORS` lists eight major numbers
  (`6, 13, 19, 20, 25, 35, 36, 37`) but the corpus's ERR rows only ever carry
  26, 34, 35, 41, 42 (the four rejected rows are all 35), and the TSV header
  names only `35, 36, 37, 19, 20` as syntax errors. The `6`/`13`/`25` entries
  are unexercised and disagree with the header's own statement. Harmless today;
  a regenerated corpus could silently reclassify a row through the wider list.
* **M2.** `expr/tests.rs:741` (`every_node_in_a_dense_expression_contains_its_operands`):
  the containment half of `check_spans` cannot fail on parser output, because
  `Expr::new` widening makes containment structural. `tiling.rs` says this
  openly for its property 1 and keeps the check as a pin; this test's comment
  ("the cases that stress the widening") implies a falsifiability it does not
  have. Suggest one sentence borrowing tiling.rs's framing.
* **M3.** `expr.rs:683`: where the C++ tolerates a null right operand after a
  *blank* operator (`LanguageParser.cpp:2961`, "in theory, we've already
  handled that possibility") the Rust raises 35.1 for every operator. Both
  paths are unreachable, because the blank-before-terminator check fires first
  in both implementations, but the deviation from the mirror is not noted at
  the site the way other deliberate collapses are.

## Verified sound (the load-bearing checks, with probe evidence)

* **Precedence and associativity.** The Rust `precedence()` table matches
  `RexxToken::precedence` level for level (read side by side). The C++ pops
  while `token->precedence() <= second->precedence()`, i.e. every dyadic level
  left-associative; `binary_rest`'s recursion at `prec + 1` is the same
  machine. Probed: `-2 ** 2`=4, `2 ** 3 ** 2`=64, `\0 ** 0`=1, `2 * 3 ** 2`=18,
  `a b + 1`=`2 4`, `1 = 1 2`=0, `1 | 0 & 0`=1, `1 | 1 && 1`=0, `10-3-2`=5,
  `7 // 4 * 2`=6, `a = b = c` (2,2,1)=1, `- "5"~length`=-1 -- all exactly as
  the comments claim. The 4,240-row differential corpus is intact
  (2686 OK + 1554 ERR, counts asserted in the test).
* **Blank abuttal.** The blank is an operator at level 4, synthesised abuttal
  when a term abuts a term; blank-next-to-terminator is consumed and ends the
  expression, matching the C++'s `nextReal`/`previousToken` dance. Probed the
  boundary cases the brief asked for: a comment is transparent
  (`a/*x*/b` = `23`, abuttal; `a/*x*/(b)` is a *call* -- oracle raises 43.1
  "Could not find routine A"), a continuation contributes a blank
  (`a,`\n`b` = `2 3`), `(a)(b)`=`23` vs `(a) (b)`=`2 2`, and
  `abs ('2.5')`=`ABS 2.5` vs `abs('2.5')`=2.5. The scanner's token stream
  (tests/scanner.rs:193, :262) encodes exactly these facts, and the parser
  judges call-vs-abuttal purely on token adjacency, so the two layers agree.
* **`Expr::new` widening.** Confirmed it widens extent over children
  (ast.rs:272, pinned by ast/tests.rs). It cannot claim a neighbouring
  construct's bytes: every constructor passes the extent of the contiguous
  token run `[from, cursor)` it consumed, and every child was parsed from
  tokens inside that run, so widening only ever moves the bounds to child
  extremes already inside the run. The only bytes a span covers that belong to
  no node are parentheses (deliberate, tested at expr/tests.rs:700) and a
  trailing blank token swallowed by the terminator check, which containment
  and tightness both tolerate.
* **Call vs array vs message.** `f(,)` passes 0 arguments, `f(1,)` 1,
  `f(,1)` 2 (probed via `arg()`); `(1,)~size`=2, `(1,,)~size`=3, `(,)~size`=2
  (probed) -- the `realcount` vs `total` split between `arg_list` and
  `full_subexpression` is real and correctly placed. `'ABS'(-3)`=3 with
  `'abs'` failing 43.1 justifies `CallTarget::Literal` verbatim bytes.
  `"abc"[2]` = `"abc"~"[]"(2)` = `b` justifies the collapsed Message variant.
  `>a~b`, `>a[1]`, `>a~b~c`, `>a.~b` all translate (probed rc 0), which is the
  one route into `binary_rest`'s tilde arm, and the cascade attaches to the
  innermost term there exactly as the C++'s term stack does.
* **Message names as `Box<[u8]>`.** Applied consistently: symbol names via the
  interned upcased spelling, literal names upcased per byte (matches ASCII-only
  `Utilities::toUpper`), `[]` for brackets, and the same choice in
  `InstructionKind::Label`, `Call::Named`, `Signal::Label`, `ConditionTrap`,
  and `CodeBody::labels`. Nothing re-interns a literal-sourced name.
* **Terminators.** Bit values and composite sets match Token.hpp exactly;
  keyword gating on simple variables only (`1 to. 3` does not stop, probed
  shape test); WHILE picks up UNTIL; OVER never terminates. Sub-numbers
  35.918 (`r =`), 35.912 (bare `interpret`), 35.929 vs 35.934
  (plain `select` vs `select case` empty WHEN element) all reproduce.
* **Errors.** All 23 probed error cases match major numbers, and all 17 probed
  sub-numbers match (20.917, 20.930, 19.909, 35.1, 35.901, 36.901, 36.902,
  37.2, 37.901, 35.929, 35.934, 35.918, 35.912, 88.914 at run time for
  `a~b:c.`, 36.901 clause-line reporting for `r = (1,` continued).
* **Byte-vs-str.** No violations in the slice. `SymbolTable` names being
  `&str` is safe: the scanner's character table is zero for 0x80-0xFF
  (scanner.rs:135, enforced by the expect at scanner.rs:848), so symbol
  spellings are ASCII by construction; everything literal-sourced stays bytes.
* **`parse_message_term` discard contract** is honoured by all three callers
  (instruction.rs:581, :1896, :2399 -- all parse on a trial cursor).
* **AST claims spot-checked**: `LABELED_SELECT_BLOCK` really is declared and
  never set; `addClause` really only appends; the directives table really has
  nine rows matching the nine `DirectiveKind` variants.
