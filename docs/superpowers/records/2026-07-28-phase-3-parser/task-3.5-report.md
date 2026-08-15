# Task 3.5 report: the expression grammar

**Status: complete, fix round 1 applied.**
Three commits on `plan/rust-rewrite` from `b2f32e7c`: `911a8ee3`, `aa05315b`, and `1dd397f7` for the review's fix round.
`rexx-parse` holds 143 tests, up from 79.
`cargo clippy --offline --all-targets -- -D warnings` is clean, `cargo fmt --check` is clean, and the whole workspace's tests pass.

**Fix round 1 is recorded in its own section at the end of this document.**
Everything before it describes the state at `aa05315b`, and where the fix round changed a decision the later section says so rather than the earlier text being rewritten, so that the review's findings still have something to point at.

## What was built

`rust/crates/rexx-parse/src/ast.rs` holds `Expr`, `ExprKind` and the helpers, and `src/expr.rs` holds the grammar.
Tests live in `src/ast/tests.rs`, `src/expr/tests.rs` and `src/expr/differential.rs`, and the differential corpus in `rust/corpus/expr/precedence.tsv`.

Four things in this task were not settled by the brief and are recorded here so they can be argued with rather than discovered later.
They are the shape of `Expr`, why `tests/expr.rs` does not exist, the eight `#[allow(dead_code)]` attributes, and a defect in Task 3.3's read-only `SymbolTable`.
Each has its own section below.

## `Expr`'s shape

**A tree of owned children, `Box` and `Vec`, with a `Range<usize>` span on every node.**

The two constraints the parent supplied decide it.
Task 3.1 Step 3b chose a flat chain with nesting by index for *instructions*, and the reason it gave was that `SIGNAL`, `ITERATE`, `LEAVE` and `END` jump into the instruction sequence, so an index is what a jump target is.
Nothing jumps into the middle of an expression, so an index buys nothing there and costs a second arena plus an index no borrow check covers.
D13's arena is still satisfied: the `CodeBody`'s one `Vec<Instruction>` owns every `Expr` inline, so the whole body remains one arena object and only the shape inside an instruction differs.

Gate criterion 1's span property holds by construction rather than by discipline.
`Expr::new(kind, extent)` widens `extent` to cover each child before storing it, so a caller that computes an extent too narrowly still produces a containing span.
`ast/tests.rs` pins that with extents that deliberately cover neither child, and `expr/tests.rs` re-checks the whole tree on every `shape()` call.

One decision inside that is worth flagging, because it went back and forth.
**Parentheses create no node and do not widen the inner node's span.**
`parse_subterm` returns the parenthesised expression itself, which is what the C++ does, so `(a)` and `a` give the same node and its span covers `a` alone.
Widening it would have made spans monotone in source extent, which is superficially nicer, but it would also mean a `Variable` node whose span is `(a)` rather than `a`, and the span is the one place the source spelling of a symbol is recoverable from.
Containment still holds either way, so the gate is satisfied: `(a) + b` has root span `1..7`, which starts after the `(`.

The collapses, each with its reason in a doc comment at the site:

| Collapsed | Into | Why |
|---|---|---|
| `ExpressionDotVariable`, `SpecialDotVariable` | `DotVariable` | Not two syntaxes. The C++ pre-loads `.nil`, `.true` and `.false` into `dotVariables` as `SpecialDotVariable` (`LanguageParser.cpp:782`-`784`) to skip a lookup. |
| `ExpressionMessage`, the `[]` collection message | `Message` with name `[]` | `parseCollectionMessage` builds a `RexxExpressionMessage` named `[]` (`:3317`), and `"abc"[2]` and `"abc"~"[]"(2)` both give `b`. |
| prefix `>` and prefix `<` | `VariableReference` | `parseMessageSubterm` maps both to `parseVariableReferenceTerm` (`:3661`-`3666`). |

`IndirectVariableReference` is not modelled: its only constructor is `InstructionParser.cpp:4521`, in `USE ARG`, so it belongs to Task 3.7 and not to the expression grammar.
`BuiltinFunctions` resolution is out of scope per the brief, so `Call` keeps the name and resolves nothing.

## Why there is no `tests/expr.rs`

The brief names `rust/crates/rexx-parse/tests/expr.rs`, and that is incompatible with the narrowing the parent ordered in the same message.
`parse_expr` takes `&ParseCtx` and `&mut TokenCursor`, and once those are `pub(crate)` no integration test can construct either, so no test under `tests/` can call the grammar at all.
The two instructions cannot both be satisfied, and narrowing is the one that reflects the architecture, because Phase 4 consumes the AST and never the token stream.

So all of Task 3.5's tests are in-crate `#[cfg(test)]` modules, and `tests/expr.rs` does not exist.
The same reasoning applies to `parse_expr` itself, which is `pub(crate)`: Task 3.6 calls it and nothing above the crate ever will.

## The eight `#[allow(dead_code)]` attributes

This is the honest cost of narrowing before the caller exists, and it is larger than a single item.
`cargo clippy --all-targets` compiles the library once with `cfg(test)` off, and in that compilation every `pub(crate)` item whose only caller is a test is dead.
Before the attributes were added the library build emitted **20 dead-code warnings**, which `-D warnings` turns into errors.

`#[allow(dead_code)]` on an item does make what that item calls live, so four attributes on the layer entry points cleared 16 of the 20.
The remaining four needed their own.
The full list, each carrying a comment naming the task that deletes it:

| Item | File | First non-test caller |
|---|---|---|
| `parse_expr` | `expr.rs` | Task 3.6 |
| `parse_expression` | `expr.rs` | Task 3.6 |
| `parse_logical` | `expr.rs` | Task 3.6 |
| `split_clauses` | `clause.rs` | Task 3.6 |
| `impl Terminators`, for `CONTROL`/`COND`/`OVER`/`IF`/`PARSE_WITH` | `expr.rs` | Task 3.7 |
| `Clause`, all three fields | `clause.rs` | Task 3.6 |
| `ParseCtx::source` | `token.rs` | Task 3.6 |
| `TokenCursor::new`, `TokenCursor::back` | `token.rs` | Task 3.6, and see below |

There is deliberately **no** crate-wide `#![allow(dead_code)]`: a blanket one would also hide code that is dead by mistake.
The reason is stated once in `lib.rs`, where a reader looking for it will start.

`#[expect(dead_code)]` does not work here, and it is worth recording why, because it looks like the right tool.
The lint fires in the library compilation and does not fire in the library-as-test compilation, so `expect` would be unfulfilled in the second one and `unfulfilled_lint_expectation` is itself a warning.

**Two things this exposes that the parent should decide on.**
First, the pattern will recur: Task 3.6's instruction parsers will also have only test callers until Task 3.11 wires `parse_program`, so 3.6 and 3.7 will each add their own attributes and only 3.11 clears them.
An alternative is to leave each layer `pub` until 3.11 and do one narrowing pass with one test migration; that trades an honest visibility for a clean build.
Second, **`TokenCursor::back` has no caller and may never get one.**
The C++ consumes a token and calls `previousToken`; this grammar peeks and only then consumes, so it never steps back.
Its three tests still pass, and the method is kept, but if Task 3.6 also does not use it then it should go.

## The defect in Task 3.3's read-only `SymbolTable`

`ParseCtx::symbols` is documented as "Read-only by the time parsing starts: `scan` has already interned every symbol in the program."
**That is false for the expression grammar, and being the first caller is how it surfaced.**

A message name from a literal has to be upcased and would have to be interned, and `scan` never saw it as a symbol: in `a~'length'` the token is a literal whose bytes are `length`, and the message name is `LENGTH`.
The bracket form needs `[]`, which is not a symbol spelling at all.

`ExprKind::Message::name` is therefore `Box<[u8]>` holding the upcased name, not a `SymbolId`.
That is defensible on its own terms, and the doc comment says so: a method name is resolved against a behaviour keyed by string, not against a variable slot keyed by index, so the C++'s `commonString` is deduplication and nothing more.
It is still an inconsistency, because every other name in the tree is a `SymbolId`.

The same wall shows up for compound-variable tails, and there it was avoided rather than worked around.
`ExprKind::Compound` carries only the interned full dotted name, and the decomposition is `ast::compound_parts`, a pure function of the spelling with no parse-time decision in it.
The tail *variables* do need `SymbolId`s eventually, for a variable pool keyed by index, so Phase 4 decomposes and interns in the pass where it assigns slots.
Storing the pieces un-interned would have put bare strings where every variable reference carries a `SymbolId`, which is worse than storing nothing.

I added `ParseCtx::symbols`'s limitation to its own doc comment rather than widening the type, because widening it means either `&mut ParseCtx` — which contradicts the brief's `parse_expr(&ParseCtx, ...)` signature — or interior mutability, and both are Task 3.3's call and not mine.

## Mutation testing

Nine mutations, applied one at a time to a pristine copy of `src/`, restored and diffed after each.
Eight were caught, one was not, and the one that was not led to a code change.

| | Mutation | Tests failed |
|---|---|---|
| M1 | `binary_rest` recurses at `prec` instead of `prec + 1`, making everything right-associative | 13, including all six associativity tests and the differential |
| M2 | `precedence(Power)` demoted from 7 to 5 | 3 |
| M3 | prefix operand parsed with `subterm` instead of `message_subterm` | 2 |
| M4 | `args.truncate(real)` removed, keeping trailing omitted arguments | 3 |
| M5 | `Expr::new` stops widening its extent by its children | 16 |
| M6 | the `KEYWORD` gate removed from `is_terminator` | **0** |
| M7 | the blank-next-to-a-terminator check removed | 2 |
| M8 | a literal message name not upcased | 1 |
| M9 | blanks skipped before checking for a call's `(` | 12 |

**M6 survived, and the reason is not a test gap.**
Every terminator set the C++ builds that carries a keyword flag also carries `TERM_KEYWORD` (`Token.hpp:532`-`538`), and a set without one fails the inner sub-keyword match anyway, so the gate is a fast path with no observable effect.
That is now stated at the gate, and `tests::the_keyword_gate_is_what_admits_a_keyword_terminator` constructs a set the C++ never would — `EOC | TO` without `KEYWORD` — so the gate's meaning is pinned and M6 is caught.

## Every oracle probe, with its raw output

All probes ran against `build/bin/rexx` 5.3.0 r0, build date Jul 27 2026, with `LD_LIBRARY_PATH` covering `build/lib` and `build/bin`.

Two harnesses were used.
`ask` wraps each line as `r = <expr>` on line 7 of a file whose first six lines set `a = 2  b = 3  c = 1  d = 4  s = "xy"  t = "9"`, then prints `"["r"]"`.
`ask2` is the same with `a = 2  b = 2  c = 1`, which is the table the associativity probes need.
`parse` wraps each line as `r = <expr>` on line 3 and reports `build/bin/rexxc`'s exit code and error numbers, giving a parse verdict without executing.

### Precedence and associativity, `ask` (a=2 b=3 c=1)

```
\0 ** 0                      => [1]
2 * 3 ** 2                   => [18]
2 + 3 * 4                    => [14]
a b + 1                      => [2 4]
1 = 1 2                      => [0]
2 = 2 & 1                    => [1]
1 | 0 & 0                    => [1]
1 | 1 && 1                   => [0]
2 ** 3 ** 2                  => [64]
10 - 3 - 2                   => [5]
100 / 10 / 2                 => [5]
7 // 4 * 2                   => [6]
a = b = c                    => [0]
-2 ** 2                      => [4]
-2 ** -2                     => [0.25]
- -2                         => [2]
\\0                          => [0]
- "5"~length                 => [-1]
\a = 2                       =>      7 *-* r = \a = 2
                                Error 34 running ... line 7:  Logical value not 0 or 1.
                                Error 34.901:  Logical value must be exactly "0" or "1"; found "2".
```

`a = b = c` giving 0 here is **not** evidence of anything, and this is the error the brief warned about: with `b = 3` neither grouping distinguishes, since `(2=3)=1` is `0=1` is 0 and `2=(3=1)` is `2=0` is 0.
Re-run with the discriminating table:

### Comparison associativity, `ask2` (a=2 b=2 c=1)

```
a = b = c                    => [1]
a == b == c                  => [1]
a > b > c                    => [0]
a \= b \= c                  => [1]
a >< b >< c                  => [1]
```

Left association predicts 1, 1 and 0; right association predicts 0, 0 and 1.
So `=`, `==` and `>` each discriminate.
`\=` and `><` give the same answer under both groupings and prove nothing, which is why the test uses the first three.

### Blanks, abuttal, calls and symbol forms, `ask2`

```
abs ('2.5')                  => [ABS 2.5]
abs('2.5')                   => [2.5]
a''b                         => [2]
a""b                         => [2]
(a)(b)                       => [22]
(a) (b)                      => [2 2]
1e5                          => [1E5]
1E5                          => [1E5]
'abs'(-3)                    =>      4 *-* r = 'abs'(-3)
                                Error 43 running ... line 4:  Routine not found.
                                Error 43.1:  Could not find routine "abs".
'ABS'(-3)                    => [3]
a b if                       => [2 2 IF]
f(,)                         =>      Error 43.1:  Could not find routine "F".
a~1                          =>      Error 97.1:  Object "2" does not understand message "1".
"5"~length                   => [1]
"abc" ~ length               => [3]
a.                           => [A.]
a.b.c                        => [A.2.1]
.true                        => [1]
1 || 2 3                     => [12 3]
1 2 || 3                     => [1 23]
a b c                        => [2 2 1]
```

`abs ('2.5')` is `ABS 2.5` with a blank, confirming `d10-decision.md` against the Task 3.1 brief's `ABS2.5`.
`1e5` printing `1E5` is why `ExprKind::Constant` holds a `SymbolId`: the value *is* the upcased spelling.
`'abs'(-3)` failing where `'ABS'(-3)` gives 3 is why `CallTarget` keeps a literal name apart from a symbol one.

### Message names, brackets and prefix forms, `ask2`

```
"abc"~'length'               => [3]
"abc"~'LENGTH'               => [3]
"abc"~"lEnGtH"               => [3]
1.50                         => [1.50]
0.0                          => [0.0]
"abc"[2]                     => [b]
"abc"~"[]"(2)                => [b]
.array~of(1,2)~[1]           =>      Error 19.909:  String or symbol expected after tilde (~).
.array~of(1,2)[2]            => [2]
.array~of(1,2)~~append(9)~items => [3]
"abc"~length~"+"(1)          => [4]
- 2 ** 2                     => [4]
+ - 2                        => [-2]
\ 0                          => [1]
>a                           => [2]
<a                           => [2]
```

### More values, `ask2`

```
1 to 3                       => [1 TO 3]
a.1.b                        => [A.1.2]
a..b                         => [A..2]
'a' || 'b'                   => [ab]
"abc"~"[]"(2)                => [b]
f(g(h(2)))                   =>      Error 43.1:  Could not find routine "H".
```

`a.1.b` giving `A.1.2` and `a..b` giving `A..2` is the tail classification: `1` and the empty piece stand for themselves, `B` is looked up.

### Unary operators and arithmetic normalisation, `ask2`

```
-2.50                        => [-2.50]
+2.50                        => [2.50]
+' 2.50 '                    => [2.50]
-'007'                       => [-7]
2.50 + 0                     => [2.50]
'007' + 0                    => [7]
+'1e2'                       => [100]
-0                           => [0]
0 - 2.50                     => [-2.50]
0 + 2.50                     => [2.50]
'abc' || 'def'               => [abcdef]
'abc' 'def'                  => [abc def]
1 2                          => [1 2]
0.5 * 2                      => [1.0]
1 / 3                        => [0.333333333]
2 ** 0.5                     =>      Error 26.8:  Operand to the right of the power operator (**) must be a whole number; found "0.5".
100 % 7                      => [14]
100 // 7                     => [2]
' 1 ' = 1                    => [1]
' 1 ' == 1                   => [0]
1 && 1                       => [0]
0 && 1                       => [1]
1 & 1                        => [1]
```

Prefix `-x` matches `0 - x` and prefix `+x` matches `x + 0` on all of these, which is how the throwaway evaluator implements them.

### Logical-value strictness, `ask2`

```
' 1 ' & 1                    => Error 34.901:  Logical value must be exactly "0" or "1"; found " 1 ".
'1.0' & 1                    => Error 34.901:  ... found "1.0".
1.0 & 1                      => Error 34.901:  ... found "1.0".
'01' & 1                     => Error 34.901:  ... found "01".
\' 1 '                       => Error 34.901:  ... found " 1 ".
\'1.0'                       => Error 34.901:  ... found "1.0".
'xy' = 'xy '                 => [1]
'xy' == 'xy '                => [0]
2 = ' 2 '                    => [1]
'9' + 1                      => [10]
'xy' || 3                    => [xy3]
```

A logical operand is a byte comparison against `0` and `1`, not a numeric one.

### Negated comparison equivalences, `ask2`

```
2 \> 2                       => [1]      2 <= 2   => [1]
3 \> 2                       => [0]      3 <= 2   => [0]
2 \< 2                       => [1]      2 >= 2   => [1]
2 <> 2                       => [0]      2 >< 2   => [0]
2 \= 2                       => [0]
'2 ' \== '2'                 => [1]
'2 ' \>> '2'                 => [0]      '2 ' <<= '2'  => [0]
'2 ' \<< '2'                 => [1]      '2 ' >>= '2'  => [1]
```

This is what lets the evaluator map `\>` onto `LessEqual`, `\<` onto `GreaterEqual`, `\>>` onto `StrictLessEqual` and `\<<` onto `StrictGreaterEqual`.

### Parse verdicts, `parse` (rexxc)

```
foo:bar                            rc=0
foo:bar(1)                         rc=0
foo:1                              rc=0
1:foo                              rc=0
a~super:method                     rc=0
a~method:x                         rc=0
a~b:c(1)                           rc=0
a~b:.nil                           rc=0
a[]                                rc=0
a[1,2]                             rc=0
a ~b                               rc=0
a~ b                               rc=0
a ~ b                              rc=0
1 -                                rc=0
a - - b                            rc=0
a ** -2                            rc=0
a || -b                            rc=0
.array~of(1,2)~at:.array(1)        rc=0
f(a,)                              rc=0
f(,a)                              rc=0
f()                                rc=0
(,)                                rc=0
(a,b)                              rc=0
(1,2)~items                        rc=0
a~~b~c                             rc=0
a~m (1)                            rc=0
>a.                                rc=0
(foo:bar)                          rc=0
(foo:bar(1))                       rc=0
a.                                 rc=0
f(,)                               rc=0
a~                                 rc=237 Error 19.909
a~~                                rc=237 Error 19.909
a~b~                               rc=237 Error 19.909
)                                  rc=219 Error 37.2
]                                  rc=219 Error 37.901
a[1]]                              rc=219 Error 37.901
1 2 3)                             rc=219 Error 37.2
a b )                              rc=219 Error 37.2
(a))                               rc=219 Error 37.2
a[1)                               rc=219 Error 37.2
a(1]                               rc=219 Error 37.901
a~b(1]                             rc=219 Error 37.901
a +                                rc=221 Error 35.1
a ||                               rc=221 Error 35.1
a + * b                            rc=221 Error 35.1
**2                                rc=221 Error 35.1
~a                                 rc=221 Error 35.1
a %% b                             rc=221 Error 35.1
a \(1 = 2)                         rc=221 Error 35.1
()                                 rc=221 Error 35.1
a [1]                              rc=221 Error 35.1
a[1] [2]                           rc=221 Error 35.1
\                                  rc=221 Error 35.901
(a                                 rc=220 Error 36.901
((a)                               rc=220 Error 36.901
f(a b                              rc=220 Error 36.901
a[                                 rc=220 Error 36.902
a[1                                rc=220 Error 36.902
(a[1                               rc=220 Error 36.902
'unterminated                      rc=250 Error 6.2
a~"b                               rc=250 Error 6.3
-                                  rc=221 Error 35.918
a~b:1                              rc=236 Error 20.917
>a.b                               rc=236 Error 20.930
>1                                 rc=236 Error 20.930
>"x"                               rc=236 Error 20.930
```

`a[1)` reporting 37.2 rather than 36.902 is the one that shaped `arg_list`: a closer of the wrong kind is a stray token, not an unmatched opener.

### Argument counts and list sizes

`p3.rex`, with `::routine t; return arg() "/" arg(1,'e') arg(2,'e')`:

```
f()       0 / 0 0
f(,)      0 / 0 0
f(,,)     0 / 0 0
f(1,)     1 / 1 0
f(,1)     2 / 0 1
f(1,,)    1 / 1 0
f(1,2)    2 / 1 1
(1)      ~items => Error 97.1: Object "1" does not understand message "ITEMS".
```

`p4.rex`:

```
(1,)   size 2 items 1
(,1)   size 2 items 1
(,)    size 2 items 0
(1,2)  size 2
(1,,)  size 3
(1)    class The String class
(1,2)  class The Array class
```

**This corrects `d10-decision.md`**, which records "`x = f(,)` is a valid call with two omitted arguments".
It is valid with **zero**: `parseArgList` returns `realcount`, the index of the last real argument, and pops the rest (`LanguageParser.cpp:3145`).
A parenthesised list keeps them, because `parseFullSubExpression` returns `total`, so `f(1,)` passes one argument while `(1,)` builds a two-element array.
The first probe of this used `~items`, which counts non-nil elements and cannot distinguish a two-element array with a hole from a one-element array; `~size` was needed.

### Where a syntax error in a continued clause is reported

```
$ cat p1.rex
/* l1 */
r = 1 +,
    * 2
say r
$ build/bin/rexxc p1.rex
     2 *-* r = 1 +,    * 2
Error 35 running /tmp/p1.rex line 2:  Invalid expression.
Error 35.1:  Incorrect expression detected at "*".

$ cat p2.rex
/* l1 */
r = (1,
    + 2
say r
$ build/bin/rexxc p2.rex
     2 *-* r = (1,    + 2
Error 36 running /tmp/p2.rex line 2:  Unmatched "(" or "[" in expression.
Error 36.901:  Left parenthesis "(" in position 5 on line 2 requires a corresponding right parenthesis ")".
```

Both report line 2, the clause's line, although the offending token is on line 3.
That confirms `ParseError::byte`'s documented contract, so every error the grammar raises carries the clause's start byte.

### Empty expressions and logical lists

```
$ printf '/* */\nr =\n'                | build/bin/rexxc     Error 35.918:  Missing expression following assignment instruction.
$ printf '/* */\nif then nop\n'        | build/bin/rexxc     Error 35.929:  Missing expression in logical expression list.
$ printf '/* */\nif , 1 = 1 then nop\n'| build/bin/rexxc     Error 35.929
$ printf '/* */\nif 1 = 1, then nop\n' | build/bin/rexxc     Error 35.929
$ printf '/* */\nif 1 = 1, 2 = 2 then nop\n' | build/bin/rexxc   rc=0
```

**`if , 1 = 1 then nop` being 35.929 changed the implementation.**
`parseLogical` raises 35.929 for *any* absent element including the first, so it never returns a null and the C++'s own `requiredLogicalExpression` null check (`LanguageParser.hpp:220`) is dead code.
`parse_logical` returns `Result<Expr, ParseError>` rather than `Result<Option<Expr>, ParseError>` because of this probe; the first draft had the `Option`.

## The differential test

The generator walks every ordered pair of 16 dyadic operators (`+ - * / % // ** ||` blank `= == > & | && \=`) over 7 operand triples, then all 3 prefix operators against all 16 dyadic ones on both sides, then one four-operand chain per operator, then the same pairs with the grouping forced by parentheses.
De-duplicated, that is **4,240 expressions**.
The triples are `(2,3,4) (2,2,1) (1,0,0) (10,3,2) (a,b,c) ('xy','9',2) (1,1,0)`, chosen so left and right association differ.

Each was evaluated by a Rexx driver that reads the expressions from a file and calls a `::routine` with `signal on syntax` so one failure does not stop the run, printing the expression, `OK` or `ERR`, and either the value or the error number `rc` carried.
The whole run takes 65 ms, so this is not a per-process harness.
The result is `rust/corpus/expr/precedence.tsv`, whose header records the generation.

Distribution: 2,686 `OK`, and 1,554 `ERR` of which 1,047 are error 34, 333 are 42, 164 are 41, 6 are 26 and 4 are 35.
The four 35s are `2 \3`, `2 \2`, `1 \0` and `10 \3`, all the blank operator followed by a prefix `\`: no blank token is emitted before a `\`, because blank significance needs the next real character to start a symbol, a literal, `(` or `[`, so the `\` lands in a dyadic position and that is 35.1.
The test therefore requires a *parse* failure with the same major number for those four, an accepted parse and a failed evaluation for the other 1,550, and an equal value for the 2,686.

**Two defects the differential found, both in the throwaway evaluator rather than the grammar.**
The first run diverged on 34 of 4,240 cases, all of them logical operators: `logical(left)? && logical(right)?` short-circuits in Rust where the interpreter validates both operands, so `2 = 3 & 4` gave `0` where the interpreter raises 34.901 on the `4`.
The second was the four syntax-error cases above, which the first harness reported as divergences because it assumed every generated expression was well formed.

The evaluator is bounded to numeric and string literals, simple variables from the fixed table, the arithmetic and comparison operators through `rexx-num`, concatenation explicit and abuttal, the logical operators, prefix `+ - \`, and parentheses.
Anything else returns `Unsupported`, and the corpus check treats `Unsupported` as a failure so the bound cannot quietly widen.
Two tests guard the evaluator itself: one pins a value per operator family so the corpus check cannot pass on error cases alone, and one asserts that calls, messages, compound variables, lists, variable references and dot variables are all out of scope.

## Concerns

1. **`tests/expr.rs` does not exist**, because the narrowing makes an integration test structurally impossible for this layer. See the section above.
2. **Eight `#[allow(dead_code)]` attributes**, and the pattern will recur in 3.6 and 3.7 and only clear at 3.11. An alternative is one narrowing pass at 3.11.
3. **`ParseCtx::symbols` being read-only is not quite true**, and `ExprKind::Message::name` carries bytes where every other name carries a `SymbolId` as a result.
4. **`TokenCursor::back` has no caller** and this grammar will never give it one.
5. **`d10-decision.md` says `f(,)` passes two omitted arguments; it passes zero.** The document is otherwise accurate on everything re-checked here. It has not been edited, because `docs/` is out of scope for this task.
6. `parse_expr` reports 35.1 for an empty expression, which is a number the interpreter does not use there. The instruction's own sub-number is not knowable from inside the expression grammar, so Task 3.6 must call `parse_expression` and raise its own.

---

# Fix round 1

Review verdict: spec PASS, quality PASS WITH CHANGES, 0 Critical, 3 Important, 5 Minor.
Commit `1dd397f7`, which sits on top of the coordinator's `5da394de` and `d37c5b6a`; those two touch only `docs/`, which this task does not.
All three Important items, both adjudications and all five Minors are addressed.
Nothing was disputed.

## I1 — the superclass override gate was too narrow

`super_class_term` admitted only `Variable` and `DotSymbol`, where the C++ gate is `isVariableOrDot()` (`Token.hpp:576`), which reads `isVariable() || subclass == SYMBOL_DOTSYMBOL` and so is `VARIABLE | STEM | COMPOUND | DOTSYMBOL`.
I re-verified the whole gate against `rexxc` myself rather than taking the reviewer's table:

```
a~b:c.           rc=0
a~b:c.d          rc=0
a~b:c.d.e        rc=0
a~b:.nil         rc=0
a~b:c            rc=0
a~b:1            rc=236  Error 20.917
a~b:.            rc=236  Error 20.917
a~b:1e5          rc=236  Error 20.917
```

`Stem` and `Compound` are now admitted and the other three still rejected.
A literal cannot pass either, and that falls out of `isVariableOrDot` reading only the subclass, which a literal token has none of; the code gates on `TokenKind::Symbol` for the same effect and says so.
`a_colon_after_a_message_name_is_a_superclass_override` now covers all eight cases above plus `a~b:'c'`.

The probe run that produced this also cost a self-inflicted detour worth recording, because it is the same class of error the brief warns about.
My first attempt reused a scratchpad `parse.sh` guarded by `ls … || cat > …`, which left a *different* script from an earlier point in the session in place, one taking its expression as `$1` rather than on stdin.
It printed one line for an empty expression and `rc=221 Error 35.918`, which is a plausible-looking result for a probe harness and means nothing at all.
Reading the output rather than skimming it for the expected shape caught it; a fresh script under a new name fixed it.

## I2 — `Expr::shape` collided on the property this task turns on

Two collisions, both fixed, and both now have a test that fails if the rendering regresses.

An omitted argument rendered `_`, and `_` is a legal symbol character, so `f(_,1)` and `f(,1)` rendered alike.
That is exactly the argument counting `f(,)` against `f(1,)` exists to pin.
It now renders `<omitted>`, and neither `<` nor `>` can start a rendered leaf, so the string has no other source.

A message name rendered unquoted, so `a~"b c"` and `a~b(c)` both gave `(msg~ A B C)`.
Message names and literals now render through `{:?}`, quoted and escaped.
Quoting alone would not have been enough: `'a''b'` decodes to `a'b`, which an unescaped `'…'` cannot render unambiguously either.
`rexxc` translates all four of `f(_,1)`, `f(,1)`, `a~"b c"` and `a~b(c)`, so these are real pairs rather than contrived ones.

## I3 — every dead-code allowance now names its owner on its own line

Eight attributes, each `#[allow(dead_code)] // deleted by Task 3.N`.
Simulating the phase gate finds nothing:

The gate command the plan now specifies finds nothing, which is the check that matters:

```
$ grep -rn 'allow(dead_code)' rust/crates/rexx-parse/src/ | grep -v 'Task 3\.[0-9]'
$
```

One thing that grep exposed and that the prose fix had to account for: `lib.rs`'s explanatory comment originally spelled the attribute out, so a naive gate would have found a ninth line with no owner.
It now says "dead-code allowances" instead and states the rule that the attribute must not be spelled out anywhere but on an item, because that is what makes the grep exact.

**The count is eight, not nine.** The reviewer counted nine at `aa05315b` and my report said eight, so the report was one low as the Minor states. Deleting `TokenCursor::back` removed one, which brings the current count back to eight for a different reason.

## Adjudication: `TokenCursor::back` deleted

Removed, along with the three tests that exercised it.
`TokenCursor` is now documented as forward only, with the reason at the type: the C++ consumes a token and calls `previousToken`, this grammar peeks and only then consumes, and 3.6 and 3.7 are the same style.
The doc comment records that the method existed and was removed once the expression grammar showed it had no caller, so a later reader does not re-add it speculatively.

## Adjudication: `parse_expr` takes the missing-expression sub-number

The 35.1 placeholder is gone.
The signature is now `parse_expr(ctx, cursor, term, missing: u16)`, mirroring `requiredExpression(terminators, error)` (`LanguageParser.hpp:228`).

**The terminator set is a parameter too, which goes one step past the instruction, and here is the evidence for it.**
`requiredExpression` has 18 call sites, and they pass five distinct terminator sets:

```
      6 requiredExpression(TERM_CONTROL,
      8 requiredExpression(TERM_EOC,
      1 requiredExpression(TERM_EOC | TERM_WITH | TERM_KEYWORD,
      2 requiredExpression(TERM_OVER,
      1 requiredExpression(TERM_RIGHT,
```

So a `parse_expr` fixed at end-of-clause would have left `DO`'s six `TERM_CONTROL` required expressions to reimplement the required check, which is the thing this change exists to avoid.

The sub-number is a bare `u16` with an implicit major of 35, and that is checked rather than assumed: all 13 error codes those call sites pass are in the 35.9xx block (`RexxErrorCodes.h:322`-`352`), as are 35.929 for a logical list and 35.934 for a `SELECT CASE` list.

`an_empty_required_expression_raises_the_sub_number_the_caller_supplied` passes 918 and 912 and asserts both come back, so the parameter cannot be ignored.

## The five Minors

1. The report's allowance count was one low, and `lib.rs` implied an attribute in `lib.rs` that did not exist. Both fixed; see I3.
2. `expr.rs`'s module doc named six functions that do not exist. It now maps each `Parser` method to the `LanguageParser` method it answers to, using the real names.
3. **`binary_rest`'s message-operator arm is reachable, and the comment claiming otherwise was wrong.** The code was right. The route is a prefix `>` or `<`: `variable_reference_term` returns straight to its caller rather than through `message_subterm`'s cascade loop, so the `~` in `>a~b` arrives at the loop. Verified with `rexxc`: `>a~b`, `<a~b`, `>a.~b`, `>a[1]` and `>a~b~c` are all rc=0. `a_cascade_can_follow_a_variable_reference` now covers four of them, so the arm has tests rather than a wrong comment.
4. `Expr::shape` and `ExprKind::for_each_child` were needlessly `pub`. `for_each_child` is now `pub(crate)`, which keeps it live because `Expr::new` calls it. `shape` is now `#[cfg(test)] pub(crate)`, along with `render_args`, `quoted` and the `SymbolTable` import it needs, because it renders trees for test assertions and nothing else.
5. `(a) + b`'s root span slicing to `a) + b` is documented behaviour and containment holds, so only the comment changed. It now says that a span is the extent from one token's start to another's end rather than a self-contained substring, notes that a clause span crossing a comma continuation has the same property, and states which containment the gate actually checks.

## Mutation testing the fix round

Five mutations against the fixed code, each applied alone, restored and diffed after.
All five are caught.

| | Mutation | Tests failed |
|---|---|---|
| F1 | the I1 regression: superclass gate back to `Variable \| DotSymbol` | 1 |
| F2 | the opposite error: gate widened to admit any symbol class | 1 |
| F3 | an omitted argument renders `_` again | 2 |
| F4 | a message name renders unquoted again | 13 |
| F5 | `parse_expr` ignores the caller's sub-number and raises 35.1 | 1 |

F1 and F2 both fail the same single test, which is the point: the gate has to be exactly `isVariableOrDot`, and a test that only checked the accepted cases or only the rejected ones would catch one mutation and not the other.

## Test count

143 in `rexx-parse`, unchanged in total from `aa05315b` but not in composition: three tests went with `TokenCursor::back`, and three were added (`a_shape_renders_an_omitted_argument_unlike_any_variable`, `a_shape_quotes_a_message_name_so_a_blank_in_one_cannot_hide`, `a_cascade_can_follow_a_variable_reference`).
The differential corpus is untouched and still passes: it holds no `~`, no `[`, no call and no colon, so I1 cannot have changed what it parses, which I confirmed by re-running it rather than reasoning about it.

## Concerns after fix round 1

The six concerns in the earlier section stand except for these three, which the round settled:

* `TokenCursor::back` is deleted rather than open.
* `parse_expr`'s 35.1 placeholder is gone, so Task 3.6 owes nothing here.
* The eight allowances now carry machine-checkable owners.

Two remain unchanged and still want a decision at some point: `tests/expr.rs` cannot exist while the narrowing holds, and `ExprKind::Message::name` carries bytes where every other name carries a `SymbolId` because `ParseCtx::symbols` is read-only.
The coordinator has accepted both, the second with a note that Phase 4 will want its own method-name intern table.
