# Task 3.5 review: the expression grammar

Reviewed `b2f32e7c..aa05315b` (`911a8ee3`, `aa05315b`) against
`task-3.5-brief.md`, the dispatch's supplied constraints, and
`build/bin/rexx` / `build/bin/rexxc` 5.3.0 as the oracle.

## Verdict 1: spec compliance

**Pass.**
Every level of `RexxToken::precedence` (`Token.cpp:111`) is reproduced exactly, and I read the C++ table out line by line rather than trusting the report's transcription.
All nine levels are left-associative and each one is pinned by a shape assertion whose operands discriminate.
Prefix `+ - \` sit outside the table by construction, in `message_subterm`'s self-recursion, which is why `-2 ** 2` is 4.
The 14 in-scope expression forms from `interpreter/expression/` are all present, with the three collapses argued at the site rather than assumed.
Both supplied constraints on `Expr`'s shape hold: the span containment property is structural, and the tree is one arena per code body.

## Verdict 2: code quality

**Pass with changes.**
One behavioural divergence from the oracle, one test-oracle weakness that undercuts the property the task turns on, and one rule violation on the `allow(dead_code)` owners.
None is structural; all three are small, local edits.
The mutation testing is honest: I re-applied six of the nine mutations to the committed code and every one was caught, with failure counts within one of the reported figures.

---

## The seven adjudications

1. **`tests/expr.rs` cannot exist.** *Accept as-is.* `parse_expr` takes two `pub(crate)` types, so no separate crate can call it; the Files list is wrong, not the resolution. The five `TokenCursor` tests that moved from `tests/tokens.rs` to `src/token/tests.rs` are the honest cost.

2. **Per-task narrowing versus one pass at 3.11.** *Keep the per-task ratchet.* See the recommendation below; the fix is to the owner-naming rule, not to the ratchet.

3. **`ExprKind::Message::name` as `Box<[u8]>`.** *Accept as-is.* A method name resolves against a string-keyed behaviour, not a slot-keyed pool, so a `SymbolId` buys nothing here, and `a~'length'` cannot be interned without `&mut ParseCtx` or interior mutability, both of which are Task 3.3's call. Record that Phase 4 will want its own method-name intern table, which is a different table from `SymbolTable`.

4. **`TokenCursor::back`.** *Needs change: delete it now.* The C++ pattern it mirrors (consume, then `previousToken`) is not the shape this parser has, 3.6 and 3.7 will be written in the same peek-then-consume style, and it already carries an `allow` plus three tests that exercise nothing anything calls. Waiting one task only makes the deletion cost the same and the attribute a day older.

5. **`d10-decision.md`'s `f(,)` claim.** *Accept as-is.* Confirmed against the oracle and against the implementation: `f(,)` is zero arguments (`(call F)`), `f(1,)` one, `f(,1)` two, `f(1,,)` one, and `(1,)`/`(,)` are two-element lists while `(1,,)` is three. `args.truncate(real)` is what does it, and removing it fails three tests.

6. **`parse_logical` returning `Result<Expr>`.** *Accept as-is.* Confirmed in the C++: `parseLogical` (`LanguageParser.cpp:4283`-`4288`) raises `Error_Invalid_expression_logical_list` for every absent element including the first, so it never returns `OREF_NULL` and `requiredLogicalExpression`'s null check (`LanguageParser.hpp:218`-`221`) is unreachable. Dropping the `Option` is the right call and the probe that forced it is sound.

7. **35.1 as the empty-expression placeholder.** *Accept but record, with a better shape available.* 35.1 is a real number the interpreter uses in this file, so a leaked placeholder reads as plausible and would pass unnoticed. The C++ does not have this problem because `requiredExpression(int terminators, RexxErrorCodes error)` takes the number from the caller. Mirroring that signature in `parse_expr` removes the placeholder entirely rather than leaving Task 3.6 owing a sub-number, and it is a two-line change.

---

## Findings

### Critical

None.

### Important

**I1. A stem or compound superclass override is rejected, and the oracle accepts it.**

`super_class_term` (`expr.rs:619`-`640`) admits only `SymbolClass::Variable` and `SymbolClass::DotSymbol`.
The C++ gate is `isVariableOrDot()`, which is `isVariable() || subclass == SYMBOL_DOTSYMBOL`, and `isVariable()` is `SYMBOL_VARIABLE || SYMBOL_STEM || SYMBOL_COMPOUND` (`Token.hpp:572`, `576`).

* Input: `r = a~b:c.`
* Wrong output: parse error 20.917.
* Right output: parses. Measured, `build/bin/rexxc` gives rc=0, and at run time the form reaches error 88.914 (`Argument SCOPE must be an instance of the Class class`), which is a *runtime* error, so the parse must succeed.
* Same for `a~b:c.d` and `a~b:c.d.e`, both rc=0 under `rexxc`.
* The current rejection of `a~b:1` (20.917), `a~b:1e5` (20.917) and `a~b:.` (20.917) is correct and must stay: `SYMBOL_CONSTANT` and `SYMBOL_DUMMY` are not in `isVariableOrDot`.

Fix: extend the match to `Stem` and `Compound`, building `ExprKind::Stem` and `ExprKind::Compound`, which is what `addText` does for those classes.

**I2. `Expr::shape` collides on two pairs, and one of them is the argument-counting property.**

`render_args` (`ast.rs:391`-`402`) prints `_` for an omitted argument, and `_` is a legal symbol character, so a variable named `_` renders identically.
`shape`'s `Message` arm (`ast.rs:369`-`374`) prints the name unquoted, so a name containing a blank renders identically to a name plus one argument.

* Input: `f(_,1)` and `f(,1)`.
* Wrong output: both render `(call F _ 1)`.
* Right output: two different strings. The first passes two real arguments, the second one omitted and one real, and that distinction is the whole of finding 4 in the dispatch.
* Input: `a~"b c"` and `a~b(c)`.
* Wrong output: both render `(msg~ A B C)`.
* Right output: two different strings. Measured, `a~"b c"` sends the message `B C` (error 97.1 quotes it), so the name really does hold a blank.

Neither collision breaks a committed assertion today, because no test uses `_` as a variable or a spaced message name.
It is Important rather than Minor because the crate's own `ast::tests::a_shape_renders_a_literal_and_a_constant_differently` sets exactly this standard, in exactly these words: "A rendering that could not tell `'2'` from `2` would make every shape assertion in `expr/tests.rs` weaker than it looks."

Fix: quote the message name as the literal arm already does, and use an omitted-argument marker no symbol can spell.

**I3. Four of the nine `#[allow(dead_code)]` attributes name no owner in the code.**

There are **nine**, not the ten in the dispatch nor the eight in the report:
`expr.rs:95`, `159`, `171`, `190`; `clause.rs:44`, `82`; `token.rs:661`, `698`, `726`.

Naming an owner:

* `expr.rs:95` `impl Terminators` — "Tasks 3.6 and 3.7 pick a set per instruction."
* `clause.rs:44` `Clause` — "Task 3.6 is the first non-test reader of all three fields."
* `token.rs:661` `ParseCtx::source` — "Task 3.6 reads this…"
* `token.rs:698` `TokenCursor::new` — "Task 3.6 builds one of these per clause…"
* `token.rs:726` `TokenCursor::back` — "Task 3.6 or 3.7 either uses it or it should go."

Naming none:

* `expr.rs:159` `parse_expr`
* `expr.rs:171` `parse_expression`
* `expr.rs:190` `parse_logical`
* `clause.rs:82` `split_clauses`

The report's table lists Task 3.6 as the first non-test caller for all four, and the report asserts each attribute carries "a comment naming the task that deletes it".
That is true of the table and false of the code.
An `allow` whose owner lives only in a report that nobody greps is an `allow` with no owner.

### Minor

**M1. The attribute count in the report is one low, and `lib.rs` points at an attribute that is not there.**
The report's table merges `TokenCursor::new` and `TokenCursor::back` into one row, so it says eight where the code has nine.
`lib.rs:21` says "The `#[allow(dead_code)]` attributes **below** and in `clause.rs`, `expr.rs` and `token.rs`", and `lib.rs` carries none.

**M2. `expr.rs`'s module doc names six functions that do not exist.**
It claims "One function per `LanguageParser` method, keeping the same names" and then lists `parse_full_subexpression`, `parse_subexpression`, `parse_message_subterm`, `parse_subterm`, `parse_message`, `parse_arg_list`.
The methods are `full_subexpression`, `subexpression`, `message_subterm`, `subterm`, `message`, `arg_list`.
Only `parse_expression` and `parse_logical` keep the C++ spelling.

**M3. `binary_rest`'s message-operator arm is documented unreachable and is reachable.**
`expr.rs:439`-`442` says "Not reachable from here, because `message_subterm` drains the whole cascade before returning".
It does not drain it on the variable-reference path: `>a~b` returns from `message_subterm` with the `~` unconsumed, and `binary_rest` then takes that arm.
Verified: `>a~b` gives `(msg~ (vref A) B)` and `<a.~b` gives `(msg~ (vref stem:A.) B)`, both rc=0 under `rexxc`.
The C++ reaches its own `TOKEN_TILDE` case the same way, because `parseVariableReferenceTerm` (`LanguageParser.cpp:3719`) returns straight out of `parseMessageSubterm`.
The code is right; only the comment is wrong.

**M4. `Expr::shape` and `ExprKind::for_each_child` are `pub` on a re-exported type solely to serve in-crate tests.**
Now that every Task 3.5 test is in-crate, `pub(crate)` would compile.
Leaving a test-only rendering in the crate's public surface is in mild tension with the narrowing this task performed elsewhere.

**M5. `(a) + b`'s root span slices to `a) + b`.**
Verified: root `1..7`, left `1..2`, right `6..7`, and `Expr::new` does widen (removing the widening fails 16 tests).
Containment holds, so gate criterion 1 property 1 is satisfied, and the decision not to widen for parentheses is argued at the site.
Record the consequence: any later consumer that slices source by `Expr::span` — a `TRACE` renderer, error underlining — gets unbalanced text for a parenthesised operand.
`ParseError::byte` uses the clause start, so nothing depends on it today.

---

## Verified in detail, no finding

* **Precedence, level by level, against `Token.cpp:111` read directly.** 8 `\`, 7 `**`, 6 `* / % //`, 5 binary `+ -`, 4 abuttal/`||`/blank, 3 all 18 comparisons, 2 `&`, 1 `| &&`. `the_precedence_table_matches_the_cpp_level_for_level` enumerates all 32 operators, and `every_comparison_operator_parses_at_the_comparison_level` wires each of the 18 into a `|| … &` sandwich rather than sampling.
* **Associativity at every level, with discriminating operands.** `2 ** 3 ** 2` → 64; `10 - 3 - 2` → 5; `100 / 10 / 2` → 5; `7 // 4 * 2` → 6; `a = b = c` with `a=2 b=2 c=1` → 1, plus `==` and `>` which also discriminate, and the test comment says outright that `\=` and `><` do not; `1 | 1 && 1` → 0. Level 4's mutual associativity is asserted (`a b c`, `'a' || 'b' || 'c'`) with the comment stating that no value can discriminate it, which is correct and honest.
* **`\` against `**`.** `\0 ** 0` → 1, and the test comment explains why the exponent must be 0: for x in {0,1}, `(\x) ** 2` and `\(x ** 2)` are both `\x`.
* **Span containment for every node kind.** `check_spans` recurses on every `shape()` call and on a dense list covering prefix, binary, call, message, cascade, bracket message, list, omitted argument, variable reference and compound. Removing the widening in `Expr::new` fails 16 tests. Prefix nodes are built with the operator token's span alone, so the widening is load-bearing on the ordinary path, not only on a miscomputed one.
* **`f(x)` versus `f (x)`.** `abs('2.5')` → `(call ABS '2.5')`, `abs ('2.5')` → `(blank ABS '2.5')`, and the same rule at a message name: `a~m(1)` → `(msg~ A M 1)`, `a~m (1)` → `(blank (msg~ A M) 1)`. Inserting a `skip_blanks` before the call check fails 13 tests.
* **Argument counting.** `f()`/`f(,)`/`f(,,)` → 0; `f(1,)`/`f(1,,)`/`f(a,,)` → 1; `f(,1)` → 2; `(1,)`/`(,)`/`(,1)` → 2-element lists; `(1,,)` → 3; `(1)` → no list node at all.
* **Error numbers on 45 inputs**, including 24 I chose myself rather than reusing the report's: 20.923 for `(foo:)`, `(foo:+1)`, `(foo:'x')`; 20.917 for `(a~b:)`; 20.930 for `>`, `<`, `(>)`; 37.2 for `f(1))`; 37.901 for `a[1,2]]`; 35.1 for `(a:b:c)`, `(a:b(1):c)`, `1 || || 2`, `(1 + )`. All match `rexxc`. The 20.923 sub-number checked against `rexxmsg.xml:1250`-`1255`.
* **Terminator sets.** `TERM_EOC/RIGHT/SQRIGHT/TO/BY/FOR/WHILE/WITH/THEN/KEYWORD` and the four composites match `Token.hpp:521`-`538` bit for bit; `PARSE_WITH` matches `InstructionParser.cpp:3223`. `OVER` correctly does not terminate on `OVER`. The `isSimpleVariable` gate is reproduced, so `to.` does not terminate a `CONTROL` expression.
* **Mutation testing, re-run by me on the committed code and reverted.** M1 `prec` for `prec + 1` → 14 failures (report: 13). M3 `subterm` for `message_subterm` → 2 (2). M4 drop `args.truncate` → 3 (3). M5 drop the widening → 16 (16). M7 drop the blank-next-to-terminator check → 3 (2). M6 remove the `KEYWORD` gate → 1, and it is `the_keyword_gate_is_what_admits_a_keyword_terminator`, exactly the test the report says was added to catch it. Tree restored to `5da394de`, clippy `-D warnings` clean, `cargo fmt --check` clean, 143 tests green.
* **Corpus integrity.** 4,240 rows, all 4,240 expression texts distinct, 2,686 `OK`, 1,554 `ERR` split 1,047×34 / 333×42 / 164×41 / 6×26 / 4×35, matching the report exactly. The four 35s are the four blank-then-prefix-`\` cases. The differential's three count assertions (2,686 / 1,550 / 4) would fail on a silently shrunk corpus.
* **The bounded evaluator's negative controls.** One test pins a value per operator family so the corpus check cannot pass on error rows alone, and one asserts `Unsupported` for calls, messages, compound variables, lists, variable references and dot variables. The short-circuit defect the report describes is real: Rust's `&&` would diverge on 34 rows, and the fix validates both operands.

---

## Not verifiable from the diff

* That the 4,240 baked answers came from `build/bin/rexx`. I confirmed the row count, the distinctness, the verdict split and the error-number distribution, and I re-derived a sample of values against the binary, but the only complete check is regenerating the corpus. The header forbids hand edits, which is the right guard.
* Where the cursor is left after a keyword terminator stops an expression. The code consumes the blanks and leaves the terminator, matching `nextReal`/`previousToken`, but no test observes the cursor's position, so Task 3.6 is the first thing that will notice if it is wrong.
* M2 (`precedence(Power)` demoted) and M8 (a literal message name not upcased). I re-ran six of the nine; these two I did not.
* Whether Task 3.6 and 3.7 will in fact use `Terminators::CONTROL`, `COND`, `OVER`, `IF` and `PARSE_WITH` as written, which is what clears `expr.rs:95`.
