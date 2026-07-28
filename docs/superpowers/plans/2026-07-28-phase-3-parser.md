# Phase 3 — Parser Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `rexx-parse` — turn Rexx source text into an AST that Phase 4 can execute, with error messages and `SOURCELINE` matching the interpreter exactly, and with clause source reconstructible so `TRACE`'s `*-*` lines can be produced. `TRACE`'s value lines are Phase 4's, deliberately — see Task 3.9.

**Architecture:** A hand-written scanner and clause splitter feed a parser that produces plain owned Rust data (D13). The program source is retained as one `String`; every AST node holds a byte range into it, because `SOURCELINE`, error reporting and `TRACE` all expose original text. `TRACE` needs one range the others do not: the **clause** span, which runs to the end of the clause's terminating token and which `THEN`/`ELSE`/`OTHERWISE` and a label's `:` can cut mid-line. Task 3.4 produces it, Task 3.6 splits it, every `Instruction` carries it. Whether the layer above the token stream uses `chumsky` combinators or hand-written recursive descent is decided by the Task 3.1 spike, not assumed here.

**Tech Stack:** Rust 2024, `rexx-num` (already built), optionally `chumsky` 0.13.0 (present in the offline registry cache). No other new dependencies.

## Global Constraints

- Behaviour is defined by what `build/bin/rexx` does, **not** by the ANSI standard or the documentation. Where they disagree, the interpreter wins.
- Zero `unsafe`. `unsafe_code = "forbid"` at `[workspace.lints.rust]`; every crate carries `[lints] workspace = true`.
- Error numbers **and sub-numbers** are contract. Programs trap on them.
- Every `cargo` command takes `--offline`.
- `rexx-parse` depends on `rexx-num` and nothing else in the workspace. It must not depend on an executor.
- The AST is plain owned Rust data **inside one arena object per code body** (D13, closed). Not garbage-collected, not reference-counted between nodes. The arena half matters: it is what lets nodes reference each other by index instead of by pointer.
- The C++ tree is the oracle and is never modified.
- No task may leave the differential sets from Phase 2 regressed; `rexx-num` is a dependency now.

## What is being replaced

Measured, for scale and for locating reference behaviour:

| C++ file | LOC | what it does |
|---|---|---|
| `interpreter/parser/InstructionParser.cpp` | 4,650 | the 35 keyword instructions |
| `interpreter/parser/LanguageParser.cpp` | 4,398 | expression grammar, symbol handling |
| `interpreter/parser/DirectiveParser.cpp` | 2,867 | `::class`, `::method` and the rest |
| `interpreter/parser/Scanner.cpp` | 1,955 | tokens |
| `interpreter/parser/ProgramSource.cpp` | 768 | source retention, `SOURCELINE` |
| **these five** | **14,638** | |
| the whole `interpreter/parser/` directory | 17,483 | including `Token.cpp`, `Clause.cpp`, `KeywordConstants.cpp` and headers |

19 token classes (`Token.hpp`), 35 keyword→instruction mappings
(`KeywordConstants.cpp`), 52 instruction classes, 17 expression classes.

**The keyword tables are bigger than "35".** `InstructionParser.cpp` is 4,650
lines mostly because of the *sub*-keyword tables, which the 35 figure hides:
`subKeywords` 50, `subDirectives` 40, `parseOptions` 10, `conditionKeywords`
12. Size Task 3.6 and Task 3.7 by those, not by the instruction count.

## The number this phase must beat

C++ cold start is **5.1 ms** from a memory-mapped image (`perf-baseline.md`).
Under D2 the Rust build parses `CoreClasses.orx` (4,193 lines) and
`StreamClasses.orx` (1,010 lines) — **5,203 lines** — at every interpreter
start. Total cold start must stay inside **~55 ms**, so parse throughput on
those two files *is* cold-start time. Measure on them specifically, never on
synthetic input.

## File structure

```
rust/crates/rexx-parse/
    src/
        lib.rs          # public API: parse_program -> Program,
                        # parse_interpret -> Fragment, ParseError
        source.rs       # ProgramSource: the retained text, line index, SOURCELINE
        token.rs        # Token, TokenKind, Span, ParseCtx, TokenCursor
        scanner.rs      # source -> tokens; comments, continuations, literals
        clause.rs       # tokens -> clauses; the `;`/EOC and label rules,
                        # clause spans, ClauseCursor
        expr.rs         # expression grammar (construction per D10)
        instruction.rs  # 35 keyword instructions + assignment/command/message/label
        directive.rs    # the 9 directives: annotate attribute class constant
                        # method options requires resource routine
        ast.rs          # the node types Phase 4 consumes
        error.rs        # ParseError -> error number, sub-number, line, substitutions
    tests/
        scanner.rs  clause.rs  expr.rs  instruction.rs  directive.rs
        sourceline.rs  errors.rs
    benches/
        parse.rs        # throughput on CoreClasses.orx + StreamClasses.orx
```

`ast.rs` is the interface Phase 4 consumes; keep it free of parsing concerns
so the D10 choice does not leak past it.

---

## Task 3.1: The D10 spike — decide parser construction

**This task produces a decision and a document, not production code.** Its
output governs every later task's shape, so it goes first and is timeboxed.

**Files:**
- Create: `rust/crates/rexx-parse-spike/` (a scratch crate, deleted at the end)
- Create: `docs/superpowers/plans/d10-decision.md`

**Interfaces:**
- Produces: the decision recorded in `d10-decision.md`, plus a token-stream
  shape that Task 3.3 will implement for real.

Implement **the expression grammar only** — precedence, abuttal
concatenation, message sends (`~`, `~~`), function and array-reference forms,
compound variables — twice over the same hand-written token stream:

1. with `chumsky` 0.13.0 combinators
2. by hand, recursive descent

Do **not** implement all 35 instructions twice. The expression grammar alone
is enough signal; the parent plan says so explicitly and the timebox exists to
stop this becoming Phase 3 itself.

- [ ] **Step 1: Extract the expression corpus**

The L0 corpus at `rust/corpus/lang/` has 14 programs. Pull every distinct
expression shape out of them plus these, which cover the forms that separate
the two approaches:

```rexx
a + b * c                      /* precedence */
a b c                          /* abuttal concatenation */
a || b                         /* explicit concatenation */
obj~method(1, 2)               /* message send */
obj~~method                    /* cascading message send */
arr[i, j]                      /* array reference */
stem.i.j                       /* compound variable */
f(g(h(x)))                     /* nested calls */
-x ** 2                        /* prefix vs power binding */
a = b = c                      /* LEFT-associative. With a=2 b=2 c=1 this is
                               1: (a=b)=c is (2=2)=1 is 1=1 is 1, where
                               a=(b=c) would give 2=(2=1) is 2=0 is 0.
                               All-equal operands cannot tell them apart. */
f(x)                           /* call */
f (x)                          /* NOT a call -- abuttal of f and (x) */
say a""b                       /* NOT abuttal: ""b is an empty BINARY
                               literal, so this prints just a */
```

`f(x)` versus `f (x)` is the one the parent plan singles out as the combinator
hazard, because a blank changes a function call into a concatenation. Verified
they differ. Do not omit it — an earlier draft of this plan did, and it is
precisely the case that separates the two D10 options.

For each, capture the interpreter's answer with a driver that prints the
evaluated result, so both spike implementations are checked against the same
ground truth rather than against each other.

- [ ] **Step 2: Build both implementations**

Same token stream, same AST output type. **Equal effort on the two arms, not
equal wall time** — the implementer has no clock, so a wall-clock timebox is not
a rule it can follow.

An arm stops on the first of two conditions: it reaches a working expression
grammar over the whole Step 1 corpus, or it visibly stalls. Stalled means the
same construct has been attacked three times without the arm getting closer —
the same test still fails, or the fix for one construct broke another that was
passing. Once one arm reaches a working grammar, the other gets the same *number
of attempts* and then stops wherever it is.

Record, in those words, which of the two conditions ended each arm. "The
combinator arm never reached a working expression grammar" is a legitimate and
decisive result and must be recorded as the outcome, not as an incomplete
measurement. Pushing an arm past its stall point destroys the comparison,
because the two attempts then differ in effort rather than in difficulty.

- [ ] **Step 3: Measure the axes**

The parent plan fixes axes 1–3 and no others. Axis 4 is not a measurement of the
two implementations; it is a property of the dependency, already known, and it
belongs in the same document because it decides the same question.

1. **Lines of code** — the whole expression grammar, excluding the shared
   token stream.
2. **Error fidelity** — at each failure site, can the exact interpreter error
   number, sub-number and **line** be produced, along with the substitution
   values the message quotes? Test with deliberately malformed expressions and
   compare against `build/bin/rexxc`, which gives the parse verdict without
   executing the file. This is the axis most likely to decide it,
   because it is what Phase 3's gate checks. There is **no column** anywhere in
   the oracle — do not measure the spike on one, and note that ooRexx locates
   an error by quoting the offending token, so the substitution values are what
   actually pin the position.
3. **Parse throughput on `CoreClasses.orx`** — not on synthetic input. Under
   D2 this number is cold-start time.
4. **Dependency and portability cost.** `chumsky` 0.13.0 pulls **28 transitive
   packages**; the hand-written arm pulls **zero**. `chumsky`'s `default`
   feature set is `["std", "stacker"]`, and `stacker` depends on `psm`, which
   has a `build.rs` with `cc` in its `[build-dependencies]`. So choosing
   `chumsky` puts **a C compiler on the Rust build path**. This branch's CI
   builds five platforms including OpenBSD (`ci/platforms`), so that is a
   portability cost, not a convenience cost. Record it as a fact in
   `d10-decision.md` alongside the three measurements; do not re-derive it.

- [ ] **Step 3b: Decide the AST's shape — tree or flat instruction chain**

The spike builds an AST either way, so settle this while it is cheap. The C++
links instructions into a **chain** (each node points to the next) rather than
nesting them in a tree, and D13's text says "plain owned Rust data inside one
arena object per code body" — the arena half of which an earlier draft of this
plan dropped.

The two shapes give Phase 4 different dispatch loops: a chain walks a `next`
pointer, a tree recurses. Getting it wrong is not a parser fix, it is a Phase 4
rewrite. Record the choice and the reason with the D10 decision.

- [ ] **Step 4: Write `d10-decision.md`**

Record the three measurements, the dependency cost from axis 4, which stop
condition ended each arm, the Step 3b shape decision, the overall decision, and
— importantly — what would change it. The parent plan's starting position is
(a): hand-written scanner and clause splitter with `chumsky` above the token
stream. State plainly if the measurements contradict that.

- [ ] **Step 5: Delete the spike crate and commit**

`rust/Cargo.toml` has `members = ["crates/*"]`, so creating and deleting the
spike crate never touches it. The decision document is the only file to stage.

```bash
rm -rf rust/crates/rexx-parse-spike
git add docs/superpowers/plans/d10-decision.md
git commit -m "Decide D10 with measurements"
```

---

## Task 3.2: `ProgramSource` and `SOURCELINE`

**Files:**
- Create: `rust/crates/rexx-parse/src/source.rs`
- Test: `rust/crates/rexx-parse/tests/sourceline.rs`

**Interfaces:**
- Produces: `ProgramSource::new(text: String) -> ProgramSource`,
  `ProgramSource::line(&self, n: usize) -> Option<&str>` (1-based),
  `ProgramSource::line_count(&self) -> usize`,
  `ProgramSource::position(&self, byte: usize) -> (usize, usize)` returning
  1-based (line, column). Every later task uses `position` for error
  reporting.

Source retention comes first because everything else holds ranges into it.

- [ ] **Step 1: Capture the interpreter's behaviour**

```rexx
/* probe.rex */
say sourceline()            /* the count */
say "[" || sourceline(1) || "]"
say "[" || sourceline(2) || "]"
```

Run it under `build/bin/rexx` and record the answers. Check specifically: does
`sourceline(n)` include the trailing newline? What does it return for `n`
past the end, and for `n = 0`? Do not guess these; a wrong answer here is
invisible until `TRACE` is wired up in Task 3.9.

- [ ] **Step 2: Write the failing test**

```rust
#[test]
fn sourceline_returns_lines_without_terminators() {
    let src = ProgramSource::new("say 1\nsay 2\n".to_string());
    assert_eq!(src.line_count(), 2);
    assert_eq!(src.line(1), Some("say 1"));
    assert_eq!(src.line(2), Some("say 2"));
    // Out of range is an ERROR in the interpreter, not an empty answer:
    // sourceline(0) raises 40.14 and sourceline(99) raises 40.34. Verified.
    // `line` returning None is how this crate reports that; Task 3.8 turns
    // it into the right error number. Do not let it render as "".
    assert_eq!(src.line(3), None);
    assert_eq!(src.line(0), None);
}

#[test]
fn position_is_one_based_line_and_column() {
    let src = ProgramSource::new("say 1\nsay 2\n".to_string());
    assert_eq!(src.position(0), (1, 1));
    assert_eq!(src.position(4), (1, 5));
    assert_eq!(src.position(6), (2, 1));
}
```

- [ ] **Step 3: Run it and watch it fail**

`cargo test --offline -p rexx-parse --test sourceline`

- [ ] **Step 4: Implement**

Build a line-start index once at construction. `position` is a binary search
over it — not a scan, because error reporting calls it and Task 3.10 measures
throughput.

- [ ] **Step 5: Handle the line-terminator cases the probe found**

CRLF and a final line without a terminator both need explicit tests, with the
expected values taken from Step 1's probe rather than from intuition.

- [ ] **Step 6: Commit**

---

## Task 3.3: Scanner and tokens

**Files:**
- Create: `rust/crates/rexx-parse/src/token.rs`, `src/scanner.rs`
- Test: `rust/crates/rexx-parse/tests/scanner.rs`

**Interfaces:**
- Consumes: `ProgramSource` from Task 3.2.
- Produces: `Token { kind: TokenKind, span: Range<usize> }`, and
  `scan(&ProgramSource) -> Result<Vec<Token>, ParseError>`. `TokenKind`
  mirrors the C++ 19 classes in `interpreter/parser/Token.hpp`.

**`ParseError` is created here, not in Task 3.8.** Every task from this one on
returns it, so define the minimum now — `{ code: u16, sub: u16, byte: usize,
subs: Vec<String> }` — and let Task 3.8 *complete* it with the line
resolution, the message table and the per-error numbers. A plan that defers
the type to 3.8 leaves 3.3 through 3.7 unable to compile.

**Later tasks need the token vector as well as the clause.** `Clause` holds a
`Range<usize>` into this `Vec<Token>`, so a function given only a `Clause`
cannot reach the tokens. Introduce the context struct here and thread it
through:

```rust
pub(crate) struct ParseCtx<'a> {
    pub source: &'a ProgramSource,
    pub tokens: &'a [Token],
}
```

Every `parse_*` in Tasks 3.5–3.7 takes `&ParseCtx` plus its own cursor. Naming
it now avoids each task inventing a different way to reach the same two things.

**`TokenCursor` is defined here too**, because the token vector lives here and
Task 3.5's `parse_expr(&ParseCtx, &mut TokenCursor)` is the only other place it
is named. It is a position inside one clause's token range, not inside the whole
vector, so an expression parser cannot walk off the end of its clause:

```rust
pub(crate) struct TokenCursor {
    /// Index range into `ParseCtx::tokens` that this cursor may visit.
    range: Range<usize>,
    /// Next index to yield; always inside `range` or equal to `range.end`.
    pos: usize,
}

impl TokenCursor {
    pub fn new(range: Range<usize>) -> Self { Self { pos: range.start, range } }
    /// Index of the next token, or None at the end of the range.
    pub fn peek(&self) -> Option<usize> {
        (self.pos < self.range.end).then_some(self.pos)
    }
    /// Yield the next token index and step past it. Deliberately not called
    /// `next`: `clippy::should_implement_trait` fires on an inherent `next`
    /// with this signature, and gate criterion 8 runs clippy with
    /// `-D warnings`.
    pub fn advance(&mut self) -> Option<usize> {
        let i = self.peek()?;
        self.pos += 1;
        Some(i)
    }
    /// Step back one token. Panics if nothing has been yielded yet, because
    /// that is a parser bug rather than a source error.
    pub fn back(&mut self) {
        assert!(self.pos > self.range.start, "TokenCursor::back before start");
        self.pos -= 1;
    }
    pub fn position(&self) -> usize { self.pos }
}
```

Tasks 3.5–3.7 build one of these over the clause they are parsing, with
`TokenCursor::new(clause.tokens.clone())`.

**The significant-blank rule, stated once.** This is the rule that makes `f (x)`
a concatenation and `f(x)` a call, so it is the deciding case for D10 and the
single most important thing in this task. The C++ emits `TOKEN_BLANK` only when
**both** hold (`Scanner.cpp:726` and `Scanner.cpp:755–771`,
`Token.hpp:595–596`):

1. the **previous** token is a symbol, a literal, `)` or `]` —
   `RexxToken::isBlankSignificant()`; and
2. the next non-blank character starts a symbol, starts a quoted literal, or is
   `(` or `[`.

Otherwise the run of blanks is discarded and scanning continues. Two
consequences the rest of this plan depends on:

- **A `,` or `-` line continuation becomes a significant blank**, not nothing.
  `Scanner.cpp:342–348` returns `SIGNIFICANT_BLANK` from the continuation path
  when the previous token made blanks significant. Verified: `say "a"-` then
  `"b"` prints `a b`, while `say "a"||-` then `"b"` prints `ab` — in the second
  the previous token is the `||` operator, so the continuation's blank is
  dropped. A scanner that merely erases a continuation produces a silently wrong
  program.
- **The two continuation characters are this task's business, not Task 3.4's.**
  Both are handled in `locateToken` (`Scanner.cpp:271`, the `,`/`-` branch at
  `:309–387`), before any clause exists. `split_clauses` never sees an `Eoc` at
  a continued line end, so it has no continuation rule to implement.

**The `Eoc` model, stated once**, because two of the tests below depend on it.
`scan` emits one `Eoc` at each clause terminator: an explicit `;`, or an end of
line that is not continued. It **never emits two `Eoc` in a row** and never
emits a trailing `Eoc` for an empty final clause, which is what makes a blank
line or a stray `;;` produce no clause at all. This mirrors the effect of
`nextClause`'s null-clause skipping (`LanguageParser.cpp:1009`) without
reproducing the C++'s separate `CLAUSEEND_EOL`/`CLAUSEEND_EOF` subclasses, which
nothing in this phase needs to tell apart.

This is below the line where combinators help, so it is hand-written
regardless of the D10 outcome.

- [ ] **Step 1: Read the C++ scanner's hard cases**

`interpreter/parser/Scanner.cpp`, 1,955 lines. The parts that matter and are
easy to get wrong:

- `--` line comments versus the subtraction operator
- `/* */` comments, which **nest** in Rexx
- **both** continuations at end of line, `,` and `-`, neither of which is an
  operator there, and both of which turn into a significant blank rather than
  into nothing — see the rule above
- quoted literals with doubled quotes, and the `'…'x` / `'…'b` suffixes
- blanks as significant tokens (`TOKEN_BLANK`), under the two-sided rule stated
  above — abuttal concatenation needs them, so a significant one cannot be
  silently dropped, and an insignificant one must not be emitted
- a **raw-text mode** for `::RESOURCE`, whose body is copied verbatim up to a
  terminating `::END` rather than tokenised. Task 3.7 needs it, but it is a
  scanner capability and must be designed in here — retrofitting a mode switch
  into a finished scanner four tasks later is the expensive order.
- a comment **separates** tokens but produces **no blank**. Verified with
  `a = 1; b = 2`: `say a/*c*/b` prints `12` while `say a b` prints `1 2`. So a
  comment is not whitespace and not nothing — dropping it entirely glues the
  tokens into one symbol, and emitting a blank for it inserts a space the
  interpreter does not.

- [ ] **Step 2: Write failing tests for each**

```rust
#[test]
fn block_comments_nest() {
    // `1` is a symbol in Rexx, and the blank between `say` and `1` is
    // significant: previous token is a symbol, next character starts a symbol.
    let toks = scan_ok("/* a /* b */ c */ say 1");
    assert_eq!(kinds(&toks), [TokenKind::Symbol, TokenKind::Blank, TokenKind::Symbol, TokenKind::Eoc]);
}

#[test]
fn double_dash_starts_a_line_comment_but_minus_does_not() {
    // build/bin/rexx: say 1 -- 2  =>  1     (the `-- 2` is a comment)
    //                say 1 - 2    =>  -1
    // No `Blank` in either. In "a -- b" the look-ahead past `a `'s blank finds
    // `-`, which starts neither a symbol, a literal, `(` nor `[`, so the blank
    // is discarded; then `--` truncates the line and yields the clause end.
    assert_eq!(kinds(&scan_ok("a -- b")), [TokenKind::Symbol, TokenKind::Eoc]);
    // In "a - b" the same look-ahead discards the first blank, and the blank
    // before `b` is insignificant because the previous token is an operator.
    assert_eq!(
        kinds(&scan_ok("a - b")),
        [TokenKind::Symbol, TokenKind::Operator, TokenKind::Symbol, TokenKind::Eoc]
    );
}

#[test]
fn a_significant_blank_needs_both_sides() {
    // Left side must be a symbol, a literal, `)` or `]`; right side must start
    // a symbol or a literal, or be `(` or `[`.
    assert_eq!(
        kinds(&scan_ok("f (x)")),
        [TokenKind::Symbol, TokenKind::Blank, TokenKind::LeftParen,
         TokenKind::Symbol, TokenKind::RightParen, TokenKind::Eoc]
    );
    assert_eq!(
        kinds(&scan_ok("f(x)")),
        [TokenKind::Symbol, TokenKind::LeftParen,
         TokenKind::Symbol, TokenKind::RightParen, TokenKind::Eoc]
    );
}

#[test]
fn a_continuation_becomes_a_significant_blank() {
    // build/bin/rexx: say "a"-  /  "b"   =>  a b     (blank, so a concatenation)
    //                 say "a"||-  /  "b" =>  ab      (previous token is `||`)
    assert_eq!(
        kinds(&scan_ok("say \"a\"-\n\"b\"")),
        [TokenKind::Symbol, TokenKind::Blank, TokenKind::Literal,
         TokenKind::Blank, TokenKind::Literal, TokenKind::Eoc]
    );
    assert_eq!(
        kinds(&scan_ok("say \"a\"||-\n\"b\"")),
        [TokenKind::Symbol, TokenKind::Blank, TokenKind::Literal,
         TokenKind::Operator, TokenKind::Literal, TokenKind::Eoc]
    );
}

#[test]
fn doubled_quotes_are_one_quote() {
    assert_eq!(literal_text(&scan_ok("'it''s'")), "it's");
}
```

- [ ] **Step 3: Run them and watch them fail**

- [ ] **Step 4: Implement the scanner**

Work over bytes, not chars. Rexx source is byte-oriented: `'…'x` and `'…'b`
literals are defined over bytes, and the interpreter never re-encodes source
text, so a DBCS or UTF-8 byte sequence must survive a round trip through the
scanner unchanged. Decoding to `char` would also make every span a character
index, and `SOURCELINE` and `TRACE` slice the retained `String` by bytes. There
are **no column numbers in error messages** — see Task 3.8 Step 4 and gate
criterion 4 — so nothing about byte orientation follows from column counting.

Emit spans, never copied strings — Task 3.2 retains the text and the AST holds
ranges into it.

- [ ] **Step 5: Differential-test against the interpreter**

There is **no** introspection that exposes a token stream. D13's research
settled this: nothing in the language or the C API exposes an object below
`Method`/`Routine`/`Package`, and source comes back as text.

But there is a **parse-only oracle**: `build/bin/rexxc FILE` with no output file
syntax-checks without executing. Measured: a file whose body is `address system`
then `"echo hi"` gives rc 0 and runs nothing under `rexxc`, while `rexx` runs
the command. Errors go to **stderr** and the version banner to stdout, so
`build/bin/rexxc FILE 2>&1 1>/dev/null` isolates the parse verdict:

```bash
build/bin/rexxc FILE >/dev/null 2>&1; echo "parses=$?"   # 0 = parses
build/bin/rexxc FILE 2>&1 1>/dev/null                    # the error text alone
```

So the method has three parts, and the first two use `rexxc` rather than
running the program:

1. A program that scans correctly gets rc 0 from `rexxc`, and the Rust scanner
   raises no error on it either. This is the **negative** direction, *this file
   parses*, which running cannot give: a file can fail at runtime for reasons the
   scanner never touched, so a non-zero rc from `rexx` proves nothing about
   scanning.
2. A program that does not scan gets the same error number, sub-number and line
   from `rexxc` as from the Rust scanner.
3. Where scanning changes *meaning* rather than validity, `rexxc` cannot help,
   because both spellings parse. Compare **output** under `build/bin/rexx`
   instead. `say a/*c*/b` versus `say a b` is the model: both are valid, and
   only the printed result distinguishes a scanner that emits a blank for a
   comment from one that does not. `f (x)` versus `f(x)` is the same shape.

Build cases of the third kind deliberately — they are the only ones that catch
a scanner which is wrong but not broken. Use `rexx` only for those, so that a
program with a side effect is never run for a verdict `rexxc` could have given.

- [ ] **Step 6: Commit**

---

## Task 3.4: Clause splitting

**Files:**
- Create: `rust/crates/rexx-parse/src/clause.rs`
- Test: `rust/crates/rexx-parse/tests/clause.rs`

**Interfaces:**
- Consumes: `Vec<Token>` from Task 3.3.
- Produces: `split_clauses(&[Token]) -> Result<Vec<Clause>, ParseError>` where

  ```rust
  #[derive(Clone, Debug)]
  pub(crate) struct Clause {
      /// Index range into the `ParseCtx::tokens` slice, terminating token
      /// excluded. That terminator is an `Eoc` under rule 1 below and a
      /// `Colon` for a label clause.
      pub tokens: Range<usize>,
      /// Byte range in the retained source: from the start of the first token
      /// to the END of the terminating `Eoc` token. For an explicit `;` that
      /// puts the semicolon inside the span; for an end of line it stops at
      /// the last byte of the line's content, excluding the line terminator.
      pub span: Range<usize>,
      /// The label's own token range, when the clause is `name:`.
      pub label: Option<Range<usize>>,
  }
  ```

**The `span` field is what `TRACE` prints, and it is not the same as any AST
node's extent.** Task 3.9 reconstructs `*-*` lines from it, and Task 3.6 narrows
it when an instruction ends mid-clause. Producing it here rather than deriving
it later is the whole reason this field exists.

- [ ] **Step 1: Establish the rules from the C++**

`Clause.cpp` is only 211 lines and holds the clause *data structure*; the
splitting logic lives in `LanguageParser.cpp` (`nextClause`, `:1009`) and
`Scanner.cpp`. Read those. Four rules, and the last two are the ones an earlier
draft of this plan missed entirely:

**(1) A clause ends at `;`, at an uncontinued end of line, or at end of file.**
That is all `nextClause` splits on. The two continuations are already resolved by
Task 3.3's scanner, so no `Eoc` reaches `split_clauses` at a continued line end
and this task has no continuation rule of its own.

**(2) The clause span includes its terminator.** `nextClause` ends the clause
with `location.setEnd(tokenLocation)` where `tokenLocation` is the location of
the *end-of-clause token* (`LanguageParser.cpp:1072`). Verified against
`build/bin/rexx` with `trace r`: `nop;` and `do i = 1 to 2;` are traced **with
their semicolons**, and `here:` **with its colon**. An AST node's own extent
carries neither, which is why `Clause::span` is a separate field.

**(3) A label's `:` terminates the clause when tokens follow it.** `here: nop`
is two clauses, `here:` and `nop`. In the C++ this is
`trimClause(); reclaimClause();` at `InstructionParser.cpp:173–174`, driven from
the instruction parser rather than from `nextClause`. Verified with `trace r`:
`here: nop; say "two"` traces as three clauses, `here:` / `nop;` / `say "two"`.

**(4) `THEN`, `ELSE` and `OTHERWISE` end a clause mid-line.** Also driven from
the instruction parser: `trimClause()` at `LanguageParser.cpp:1378`, `:1403`,
`:1465` and `:1494`. `RexxClause::trim` (`Clause.cpp:138`) moves the clause's
*start* forward to the current token and leaves the end alone, and the
instruction that just ended narrows its own end separately — `RexxInstructionIf`
sets its end to the **start offset** of the `THEN` token
(`IfInstruction.cpp:58–66`), which is why the traced text is `if y > 5 ` with the
trailing blank and stops before `then`.

**This task implements rules 1, 2 and 3. Rule 4 is Task 3.6's**, and the split of
work is not the C++'s: see the note after the tests for why rule 3 moves down a
layer and rule 4 cannot. So `split_clauses` must produce clauses that Task 3.6's
cursor can cut further — `tokens` is a range, `span` is derivable from any
sub-range of it, and nothing in `Clause` is shared or interned.

- [ ] **Step 2: Write the failing tests**

```rust
#[test]
fn a_trailing_comma_continues_the_clause() {
    assert_eq!(clause_count("say 1,\n  + 2"), 1);
    assert_eq!(clause_count("say 1\nsay 2"), 2);
}

#[test]
fn a_colon_makes_a_label_clause() {
    let cs = clauses("here: say 1");
    assert_eq!(cs.len(), 2);
    assert!(cs[0].label.is_some());
}

#[test]
fn a_clause_span_includes_its_terminating_semicolon() {
    // build/bin/rexx, trace r:  `nop;` is traced with the semicolon.
    let src = "nop;\nsay 1\n";
    let cs = clauses(src);
    assert_eq!(&src[cs[0].span.clone()], "nop;");
    // An uncontinued end of line is a terminator too, but contributes no bytes.
    assert_eq!(&src[cs[1].span.clone()], "say 1");
}

#[test]
fn a_label_span_includes_its_colon() {
    // build/bin/rexx, trace r:  `here: nop` traces as `here:` then `nop`.
    let src = "here: nop\n";
    let cs = clauses(src);
    assert_eq!(&src[cs[0].span.clone()], "here:");
}
```

The label test is the one that fails first, because rule 3 makes `here:` a clause
that rule 1 alone does not produce.

**Rule 3 is implemented here even though the C++ implements it one layer up.**
That is a deliberate deviation: a symbol-or-literal followed by `:` at the start
of a clause is recognisable from the token stream alone, so `split_clauses` can
do it and `Clause::label` already exists to hold the result. Rule 4 cannot be
moved the same way and must stay in Task 3.6, because only the instruction parser
knows whether a `THEN` token is the `THEN` of an open `IF` or a variable named
`then` — keywords are not reserved, and Task 3.6 states why at length.

- [ ] **Step 3: Run, fail, implement, pass**

- [ ] **Step 4: Commit**

---

## Task 3.5: Expression grammar

**Files:**
- Create: `rust/crates/rexx-parse/src/expr.rs`, `src/ast.rs`
- Test: `rust/crates/rexx-parse/tests/expr.rs`

**Interfaces:**
- Consumes: `Clause` from Task 3.4; `ParseCtx` and `TokenCursor` from Task 3.3.
  Build the cursor with `TokenCursor::new(clause.tokens.clone())`, so an
  expression can never read past the end of its own clause.
- Produces: `Expr` in `ast.rs`, and `parse_expr(&ParseCtx, &mut TokenCursor) -> Result<Expr, ParseError>`.

Built the way Task 3.1 decided. The spike's implementation is a reference, not
a starting point — it was stopped at its first stall or first success, whichever
came sooner, and is not production code.

- [ ] **Step 1: Port the precedence table from the C++**

`LanguageParser.cpp`. Rexx's precedence is not C's: prefix `-` binds **tighter**
than `**`, so `-2 ** 2` is **4**, where C and Python give −4. Verified against
`build/bin/rexx`, and worth stating because the first draft of this plan
asserted the opposite from memory.

Verify every level the same way — against the binary, not the documentation —
and write the probe result into the test file as a comment beside each
assertion, so the next reader can see where the expectation came from.

- [ ] **Step 2: Write the failing tests, one per precedence level**

```rust
#[test]
fn prefix_minus_binds_tighter_than_power() {
    // build/bin/rexx: say -2 ** 2   =>   4      (C and Python give -4)
    assert_eq!(eval_shape("-2 ** 2"), "Power(Prefix(Minus, 2), 2)");
}

#[test]
fn abuttal_concatenation_binds_tighter_than_plus() {
    // build/bin/rexx: a = 1; b = 2; say a b + 1   =>   1 3
    assert_eq!(eval_shape("a b + 1"), "Concat(a, Add(b, 1))");
}
```

- [ ] **Step 3: Run, fail, implement, pass**

- [ ] **Step 4: Differential-test evaluation shape against the interpreter**

A parse tree cannot be compared directly to the interpreter, so compare
*results*: generate expressions over a small operand set, evaluate under
`build/bin/rexx`, and evaluate the parsed AST with a throwaway evaluator. Any
divergence is a precedence or associativity error. This is the Phase 2 method
applied to structure.

**Bound the evaluator, or it becomes Phase 4.** It needs exactly: numeric
literals, string literals, simple variables from a fixed table, the arithmetic
and comparison operators via `rexx-num`, concatenation both explicit and
abuttal, and parentheses. That is enough to catch every precedence and
associativity error. It does **not** need message sends, function calls,
compound variables or control flow — parse those, and assert on their tree
shape directly instead of evaluating them.

- [ ] **Step 5: Commit**

---

## Task 3.6: The 35 keyword instructions

**Files:**
- Create: `rust/crates/rexx-parse/src/instruction.rs`
- Modify: `rust/crates/rexx-parse/src/clause.rs` — this task adds `ClauseCursor`
  there, beside `Clause`, because `split_before` below is a clause operation
- Test: `rust/crates/rexx-parse/tests/instruction.rs`

**Interfaces:**
- Consumes: `Expr` from Task 3.5, `Vec<Clause>` from Task 3.4, `ParseCtx` from
  Task 3.3.
- Produces: `Instruction` in `ast.rs`, `ClauseCursor` in `clause.rs`, and
  `parse_instruction(&ParseCtx, &mut ClauseCursor) -> Result<Instruction, ParseError>`.

**It takes a cursor, not a single clause.** `DO`/`END`, `IF`/`THEN`/`ELSE`,
`SELECT`/`WHEN`/`OTHERWISE` and every `::method` body span many clauses, so a
function handed one clause cannot parse any of them. `ClauseCursor` owns the
clause list and a position; `parse_instruction` advances it.

**The cursor must also be able to split the clause it is sitting on.** Task 3.4's
rule 4: `THEN`, `ELSE` and `OTHERWISE` end a clause mid-line, so a cursor that
can only step forward over a fixed list cannot express `if y > 5 then say "big"`
— which the interpreter traces as **three** clauses. This is the C++'s
`trimClause`/`reclaimClause` pair (`LanguageParser.cpp:1378`, `:1403`, `:1465`,
`:1494`; `Clause.cpp:138`), and it is why the cursor owns a mutable pending
clause rather than borrowing a slice:

```rust
pub(crate) struct ClauseCursor {
    clauses: Vec<Clause>,
    /// Next index in `clauses`, used only when `pending` is None.
    pos: usize,
    /// The remainder of a clause that `split_before` cut in two. Yielded ahead
    /// of `clauses[pos]`.
    pending: Option<Clause>,
}

impl ClauseCursor {
    pub fn new(clauses: Vec<Clause>) -> Self {
        Self { clauses, pos: 0, pending: None }
    }

    /// The clause being parsed, without consuming it.
    pub fn peek(&self) -> Option<&Clause> {
        self.pending.as_ref().or_else(|| self.clauses.get(self.pos))
    }

    /// Consume and return the clause being parsed.
    pub fn next_clause(&mut self) -> Option<Clause> {
        if let Some(c) = self.pending.take() {
            return Some(c);
        }
        let c = self.clauses.get(self.pos)?.clone();
        self.pos += 1;
        Some(c)
    }

    /// End the current clause immediately before token index `at`, and
    /// re-present tokens `at..` as the next clause.
    ///
    /// Returns the clause the caller just finished, whose `span` ends at the
    /// START byte of token `at` — so `if y > 5 then` yields `if y > 5 ` with
    /// the trailing blank, matching `RexxInstructionIf`
    /// (`IfInstruction.cpp:58-66`). The remainder keeps the original
    /// terminating byte, so its span still includes any `;`.
    ///
    /// Panics if `at` is outside the current clause's token range, which is a
    /// parser bug rather than a source error.
    pub fn split_before(&mut self, ctx: &ParseCtx, at: usize) -> Clause {
        let cur = self.next_clause().expect("split_before with no current clause");
        assert!(cur.tokens.contains(&at), "split_before outside the clause");
        let cut = ctx.tokens[at].span.start;
        self.pending = Some(Clause {
            tokens: at..cur.tokens.end,
            span: cut..cur.span.end,
            label: None,
        });
        Clause { tokens: cur.tokens.start..at, span: cur.span.start..cut, label: cur.label }
    }
}
```

**Every `Instruction` carries a `clause_span: Range<usize>`**, copied from the
`Clause` that `next_clause` or `split_before` returned. Use that name, because
Task 3.7b retains it and Task 3.9 reconstructs `*-*` lines from it, and neither
of those implementers sees this task's brief. It is not the node's own extent: a
`THEN` is its own `Instruction` whose `clause_span` covers just the `then` token,
exactly as `RexxInstructionThen` sets its location to the `THEN` token's
(`ThenInstruction.cpp:58-77`).

**Five clause types are not keyword-driven at all,** and one of them is the
default. None of these appears in the 35:

| clause shape | node | C++ class |
|---|---|---|
| second token is `=` | `Assignment` | `AssignmentInstruction` |
| `Clause::label` is set — Task 3.4 already split it | `Label` | `LabelInstruction` |
| a standalone message send, e.g. `q~append(1)` | `Message` | `MessageInstruction` |
| a keyword from the 35 | the 35 nodes | `interpreter/instructions/` |
| **anything else** | `Command` | `CommandInstruction` |

`Command` is the fallback, and it is not exotic: a bare `"echo hi"` clause is
a command dispatched through `ADDRESS`. Verified — `address system` then
`"echo hello-from-command"` runs it and sets `rc`. Dispatch must try the
others and fall through to `Command`, never fail.

`KeywordConstants.cpp` has 36 keyword constants and 35 keyword→instruction
mappings; `interpreter/instructions/` has 52 classes. **`keywordInstructions[]`
is alphabetical and `resolveKeyword` (`KeywordConstants.cpp:417`) binary-searches
it**; each entry stores its instruction code explicitly, so nothing depends on
position. Sorting it is not merely safe, it is required.

This is the opposite of Phase 0's *builtin-function* table, which is positional
and must not be reordered. Two drafts of this plan carried that warning across
to this table, where it is wrong. Check which kind you are looking at.

- [ ] **Step 1: Extract the keyword list**

```bash
grep -oE '"[A-Z]+", *KEYWORD_[A-Z_]+' interpreter/parser/KeywordConstants.cpp
```

Assert the count is 35 in a test, so a mis-extraction fails loudly.

- [ ] **Step 2: Write a failing test per instruction family**

Five families. This is the extracted list, not a remembered one — the first
draft of this plan invented an `ASSIGN` keyword that does not exist and missed
`LOOP`, which does. All 35, each in exactly one family:

1. control flow (11) — `DO`, `LOOP`, `IF`, `THEN`, `ELSE`, `SELECT`, `WHEN`,
   `OTHERWISE`, `LEAVE`, `ITERATE`, `END`
2. data (8) — `DROP`, `EXPOSE`, `PARSE`, `PULL`, `PUSH`, `QUEUE`, `SAY`, `ARG`
3. procedure (11) — `CALL`, `RETURN`, `PROCEDURE`, `SIGNAL`, `EXIT`,
   `INTERPRET`, `GUARD`, `REPLY`, `FORWARD`, `RAISE`, `USE`
4. settings (4) — `NUMERIC`, `ADDRESS`, `TRACE`, `OPTIONS`
5. `NOP` (1)

`LOOP` is ooRexx's own extension and shares most of `DO`'s body.

**Keywords are not reserved words, and this is the single most important
structural fact in this task.** Every one of the 35 is a legal variable name.
Verified against `build/bin/rexx`:

```rexx
if = 2;  say if          /* prints 2 */
do = 3;  say do          /* prints 3 */
say = 4; say say         /* prints 4 */
end = 5; say end         /* prints 5 */
if if = 2 then say if    /* prints 2 -- keyword and variable in one clause */
do i = 1 to 2; say do; end   /* DO still loops while `do` holds 3 */
end. = 0; end.1 = 7      /* a stem named end. is fine too */
```

So a symbol is a keyword **only** by position: first token of a clause, and
only when the clause is not an assignment. That is why no `ASSIGN` keyword
exists — a clause whose second token is `=` is an assignment regardless of
what its first token spells. Recognition therefore belongs in clause dispatch
and must never live in the scanner, which cannot know a token's position in
its clause.

Design the dispatch this way from the start. Retrofitting it after building a
scanner that classifies keywords lexically means rewriting both the scanner
and every instruction parser.

`rust/corpus/lang/keyword_as_variable.rex` exercises it: all 35 as variables,
keyword and variable spelled the same in one clause, `DO` and `SELECT` still
working while their names hold values, a stem named `end.`, a compound tail
spelled `if`, and `PARSE` while `parse` is a variable. It must pass.

Write one test per family before implementing any of them, then work through
the families in that order. Step 4 below is what proves nothing was skipped;
if your extraction in Step 1 yields a keyword absent from these five lists,
trust the extraction and fix the list.

- [ ] **Step 3: Implement family by family, committing per family**

`DO` is the largest single instruction in the C++ and deserves its own commit.

Three sub-grammars inside this task are each bigger than a typical keyword and
must not be folded in silently:

- **`PARSE` templates.** `parse value X with a b +3 c` — positional patterns,
  literal patterns, absolute and relative column offsets, `.` placeholders.
  The `parseOptions` table has 10 entries (`ARG`, `LINEIN`, `PULL`, `SOURCE`,
  `VALUE`, `VAR`, `VERSION`, …); the template grammar is separate from them and
  is shared with `ARG` and `PULL`. Give it its own commit.
- **`ADDRESS`**, including the `WITH` input/output redirection forms —
  `CommandIOConfiguration.cpp` exists for exactly this and is not small.
- **`SIGNAL` and labels.** `SIGNAL` names a label, so the parser must build a
  label table for the code body; `LabelInstruction` is a real node type. A
  label may also spell a keyword.

The `subKeywords` table has 50 entries and `conditionKeywords` 12; between them
and `parseOptions` they are most of `InstructionParser.cpp`'s 4,650 lines.

- [ ] **Step 4: Assert every keyword is reachable — with a valid clause each**

A bare keyword is mostly **not** a valid clause, so a loop that parses each
keyword by itself cannot pass. But **only some of the failures are parse
errors**, and that distinction decides what this parser must accept. Measured
against both oracles, with `rexxc` giving the parse verdict and `rexx` the
runtime one:

| bare clause | `rexxc` | `rexx` | is it a parse error? |
|---|---|---|---|
| `then` | rc 248, 8 / 8.1 | same | **yes** |
| `else` | rc 248, 8 / 8.2 | same | **yes** |
| `when` | rc 247, 9 / 9.1 | same | **yes** |
| `otherwise` | rc 247, 9 / 9.2 | same | **yes** |
| `end` | rc 246, 10 / 10.1 | same | **yes** |
| `parse` | rc 236, 20 / 20.903 | same | **yes** |
| `procedure` | **rc 0** | rc 239, 17 / 17.1 | **no** |
| `leave` | **rc 0** | rc 228, 28 / 28.1 | **no** |
| `iterate` | **rc 0** | rc 228, 28 / 28.2 | **no** |

`Error_Unexpected_procedure_call` is raised in
`execution/RexxActivation.cpp:1250`, `LEAVE`'s error 28 at `:1214` and
`ITERATE`'s at `:1161` — all three in the executor, none in the parser. **So this
parser must accept bare `procedure`, `leave` and `iterate` and produce their
nodes.** A parser that rejects them at parse time diverges from the oracle on
every program containing an unreachable `leave`, and moves a Phase 4 check into
Phase 3 where it cannot be checked against anything.

Take these from a run, not from this table: two earlier drafts got the numbers
wrong in two different ways, and a third got the parse/runtime split wrong. Pair
each keyword with a minimal clause that parses, and check the resulting node
type:

```rust
const KEYWORD_CLAUSES: &[(&str, &str)] = &[
    ("SAY",  "say 1"),
    ("DO",   "do 1\nend"),
    ("IF",   "if 1 then nop"),
    ("NOP",  "nop"),
    // These three parse bare, per the table above. Do not wrap them in a loop
    // or a routine to "make them legal" -- that would hide the fact that the
    // parser accepts them standing alone, which is the behaviour under test.
    ("PROCEDURE", "procedure"),
    ("LEAVE",     "leave"),
    ("ITERATE",   "iterate"),
    // ... one legal clause per keyword, all 35
];

#[test]
fn every_keyword_reaches_its_instruction_node() {
    assert_eq!(KEYWORD_CLAUSES.len(), 35, "a keyword lost its clause");
    for (kw, src) in KEYWORD_CLAUSES {
        assert!(parses_to_node_named(src, kw), "{kw} unhandled");
    }
}
```

The length assertion is the load-bearing half: it fails when a keyword is
added to the extraction but nobody wrote a clause for it.

**Five of the 35 rows cannot be written until Task 3.1 Step 3b lands.**
`THEN`, `ELSE`, `END`, `WHEN` and `OTHERWISE` only exist as nodes of their own
under the flat instruction chain; under a tree they are absorbed into their
parent and have no node to name. Write the other 30 first and fill these in
once the AST shape is decided.

- [ ] **Step 5: Commit**

---

## Task 3.7: Directives

**Files:**
- Create: `rust/crates/rexx-parse/src/directive.rs`
- Test: `rust/crates/rexx-parse/tests/directive.rs`

**Interfaces:**
- Consumes: `ParseCtx` from Task 3.3, `ClauseCursor` from Task 3.6.
- Produces: `Directive` in `ast.rs`, and
  `parse_directive(&ParseCtx, &mut ClauseCursor) -> Result<Directive, ParseError>`.

Same reason as Task 3.6: a `::method` body spans many clauses, so a function
handed one clause cannot parse it. An earlier draft of this plan fixed the
signature in 3.6 and left 3.7 with the one that cannot work.

`DirectiveParser.cpp` is 2,867 lines. There are **nine** top-level directives,
not the seven an earlier draft listed: `::ANNOTATE`, `::ATTRIBUTE`, `::CLASS`,
`::CONSTANT`, `::METHOD`, `::OPTIONS`, `::REQUIRES`, `::RESOURCE`, `::ROUTINE`.
Those are `RexxToken::directives[]` (`KeywordConstants.cpp:52–63`), exactly nine
rows. The option sub-keywords are a **separate** table,
`RexxToken::subDirectives[]` (`KeywordConstants.cpp:363–405`), with **40** rows
(`PUBLIC`, `GUARDED`, `ABSTRACT`, `INHERIT` and so on), and that is where the
file's bulk is. Nine plus forty, two tables; there is no set of 36 anywhere.

**`::RESOURCE` needs a scanner mode this plan otherwise has nowhere.** Its body
is *raw text* up to a terminating `::END` — not tokenised, not clause-split.
Task 3.3's scanner must be able to switch into a copy-until-delimiter mode and
back. Discovering that here rather than in Task 3.3 is the point of listing it.

This task matters more than its size suggests, though not for the reason an
earlier draft gave: only **347 of `CoreClasses.orx`'s 4,193 lines** start with
`::`, 8.3%. The directives are a small fraction of the text but they frame all
of it -- every method body is inside one -- so Task 3.10 cannot parse that file
at all until this task works.

- [ ] **Step 1: Extract the directive and option tables**

**Extract from the tables, anchored, and never from the enum.** An earlier draft
used `grep -oE 'DIRECTIVE_[A-Z_]+' interpreter/parser/*.hpp | sort -u` and got 45,
then split it 9 / 36. That grep is unanchored, so it matches `DIRECTIVE_ABSTRACT`
*inside* `SUBDIRECTIVE_ABSTRACT`. Measured, the 45 decomposes as: 11
`DirectiveKeyword` enum members (`Token.hpp:333–346`) plus 41 distinct
`SUBDIRECTIVE_*` names, minus **7** suffixes that occur in both sets
(`NONE`, `ATTRIBUTE`, `CLASS`, `CONSTANT`, `LIBRARY`, `METHOD`, `ROUTINE`). So 45
is an artefact and 36 is not a real set.

Two further traps in the enum, which is why the tables are the source of truth:
`DIRECTIVE_LIBRARY` is an enum member with **no row in `directives[]`**, and
`DIRECTIVE_NONE` / `SUBDIRECTIVE_NONE` are sentinels.

Extract the two tables separately:

```bash
sed -n '/directives\[\] *=/,/^};/p'    interpreter/parser/KeywordConstants.cpp \
  | grep -oE 'KeywordEntry\("[A-Z]+", *DIRECTIVE_[A-Z_]+\)'          # expect 9
sed -n '/subDirectives\[\] *=/,/^};/p' interpreter/parser/KeywordConstants.cpp \
  | grep -oE 'KeywordEntry\("[A-Z]+", *SUBDIRECTIVE_[A-Z_]+\)'       # expect 40
```

Assert **9** and **40** in a test, so a mis-extraction fails loudly rather than
silently narrowing the task. Do not assert 9 and 36; that enshrines a false split
and yields a sub-keyword table four entries short.

**The two tables are not disjoint, and that matters for parsing.** Five
spellings appear as rows in both: `ATTRIBUTE`, `CLASS`, `CONSTANT`, `METHOD`,
`ROUTINE`. So `::CLASS c SUBCLASS d` uses `CLASS` at the top level and
`SUBCLASS` as an option, while `::METHOD m CLASS` uses `CLASS` as an option of
`::METHOD`. Resolution is by position — the token after `::` looks up in
`directives[]`, everything after it looks up in `subDirectives[]` — the same
positional rule as Task 3.6's keywords, and for the same reason.

- [ ] **Step 2: Write a failing test per top-level directive**

Nine tests, each parsing a minimal legal instance:

```rust
#[test] fn routine_directive_parses() {
    assert!(matches!(directive("::routine r\n  return 1\n"), Directive::Routine { .. }));
}
```

- [ ] **Step 3: Implement `::RESOURCE` first**

It is the one that changes the scanner, so it must not be last. Its body is raw
text up to `::END`; verify the exact terminator and whether it is
case-sensitive against `build/bin/rexxc` before implementing, then add the
scanner mode Task 3.3 left room for. A missing or mis-cased terminator is a
**parse** error, so `rexxc` sees it: measured, a lowercase `::end` gives
`Error 99` / `Error 99.943 Missing ::RESOURCE end marker "::END"` at rc 157.

- [ ] **Step 4: Implement the remaining eight, committing per directive**

- [ ] **Step 5: Assert every option sub-keyword is reachable**

Same shape as Task 3.6's Step 4: a table pairing each option with a legal
directive that carries it, plus a length assertion against the extracted count.

- [ ] **Step 6: Parse `CoreClasses.orx` end to end without error**

That file is the real acceptance test for this task. Not because it is mostly
directives — it is 8.3% — but because every method body in it sits inside one,
so nothing in the file parses until the directives do, and Task 3.10's
throughput number depends on the whole file parsing.

- [ ] **Step 7: Commit**

---

## Task 3.7b: The public entry point

**Files:**
- Create: `rust/crates/rexx-parse/src/lib.rs`
- Test: `rust/crates/rexx-parse/tests/program.rs`

**Interfaces:**
- Consumes: everything from Tasks 3.2–3.7.
- Produces:

  ```rust
  pub fn parse_program(text: String) -> Result<Program, ParseError>;
  pub fn parse_interpret(text: String) -> Result<Fragment, ParseError>;

  pub struct Program {
      pub source: ProgramSource,
      pub instructions: Vec<Instruction>,
      pub directives: Vec<Directive>,
      pub labels: BTreeMap<String, usize>,
  }

  /// What `INTERPRET` produces. Carries its own source for the same reason
  /// `Program` does: the instruction spans index it and nothing else.
  pub struct Fragment {
      pub source: ProgramSource,
      pub instructions: Vec<Instruction>,
  }
  ```

  These two are the only entry points Phase 4 uses; everything else stays
  `pub(crate)` so the D10 choice cannot leak into the executor.

**Both return types retain their own source, and that is not optional.** The
architecture is "the program source is retained as one `String`; every AST node
holds a byte range into it". An earlier draft had `parse_interpret` take ownership
of `text` and return only `Vec<Instruction>`, which leaves every span in the
result indexing a `String` that no longer exists.

It is not a theoretical defect. Interpreted clauses are traced with the
*fragment's* own text while `SOURCELINE` inside the fragment still resolves
against the enclosing program, so Phase 4 needs both strings at once. Verified
under `trace r`:

```
     2 *-* interpret "say 1+1; say sourceline(1)"
       >>>   "say 1+1; say sourceline(1)"
     2 *-* say 1+1;                <- the fragment's clause text, semicolon included
       >>>   "2"
2
     2 *-* say sourceline(1)
       >>>   "trace r"             <- the PARENT program's line 1
trace r
```

**Every `Instruction` retains the span of the clause it came from**, as Task 3.6
established, and those spans are what the `*-*` lines above are sliced from. So
`Program` needs no separate clause list: the spans travel with the instructions,
including the mid-line `then` that Task 3.6's `split_before` produces. Whichever
of the two Task 3.1 Step 3b shapes wins, the invariant is the same — an
instruction's `clause_span` is a range into the `ProgramSource` sitting in the
same struct.

`INTERPRET` parses a string at *runtime*, so the parser is not a build-time
tool that runs once — Phase 4 calls back into it during execution. That second
entry point differs from the first in three ways worth getting right now:
directives are not permitted, labels are not permitted, and errors report
against the `INTERPRET` instruction's own line rather than a position inside
the fragment. The third is verifiable today: `interpret "x = )"` inside an
installed trap gives `condition('o')~position` = the INTERPRET line. Note that
the third does **not** make `Fragment::source` redundant: the error line comes
from the caller, while the traced clause text comes from the fragment.

Until this task, the pieces are wired only by tests. It exists because a
reviewer must be able to reject the composition independently of the parts.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn a_program_with_directives_separates_them_from_instructions() {
    let p = parse_program("say 1\n::routine r\n  return 2\n".to_string()).unwrap();
    assert_eq!(p.instructions.len(), 1);
    assert_eq!(p.directives.len(), 1);
    assert_eq!(p.source.line(1), Some("say 1"));
}
```

- [ ] **Step 2: Run it and watch it fail**
- [ ] **Step 3: Implement the composition**

The first `::` directive ends the main instruction stream. An instruction
appearing *after* a directive is **not** an error — it joins that directive's
body. Verified: a file of `say "main"` / `::routine r` / `return 2` /
`say "after directive"` runs rc 0 and prints only `main`, because the trailing
instruction became part of routine `r`. An earlier draft of this plan asserted
it was a syntax error and told Task 3.8 to record a number that does not exist.

**The token vector and the `ParseCtx` do not outlive this function, and must not
try to.** `ParseCtx` borrows the `ProgramSource` and the `Vec<Token>`, so the
order is: build `ProgramSource`, `scan` it, build `ParseCtx` borrowing both,
`split_clauses`, `ClauseCursor::new`, parse everything, drop the context, then
move the `ProgramSource` into `Program`. That works precisely because every span
that survives is a **byte** range into the source rather than a token index — so
no `Instruction` or `Expr` may hold a token index. If one does, this composition
does not compile, which is the correct outcome.

- [ ] **Step 4: Parse every `rust/corpus/lang/` program through this entry point**

Fourteen today. Gate criterion 2 permits adding more, so count the directory
rather than hard-coding a number.
- [ ] **Step 5: Commit**

---

## Task 3.8: Errors with number, sub-number, line and substitutions

**Files:**
- Create: `rust/crates/rexx-parse/src/error.rs`
- Test: `rust/crates/rexx-parse/tests/errors.rs`

**Interfaces:**
- Consumes: `ProgramSource::position` from Task 3.2, `rexx-inventory`'s message table.
- Produces: the completed `ParseError { code: u16, sub: u16, byte: usize, subs: Vec<String> }` — the same shape Task 3.3 defined, no field added or removed —
  with `message(&self) -> String` rendered from the generated table.

**This is the phase's gate, not a finishing touch.** Model it on
`rexx-num`'s error work, which is already done and reviewed: carry the
substitution *values*, render on demand. A rendered `String` cannot be
un-spliced, and `condition('o')~additional` exposes the values separately.

- [ ] **Step 1: Collect ground truth — and note the obvious recipe does not work**

`signal on syntax` **cannot** catch a syntax error in its own file. ooRexx
parses the whole file before executing anything, so the trap is never
installed. Verified: a file containing `signal on syntax name oops` and then
`x = )` prints the error to **stderr** and exits rc 219; the handler never
runs. An earlier draft of this plan specified exactly that broken recipe.

Three routes that do work, with different trade-offs:

```bash
# (a) syntax-check the bad file with rexxc and capture stderr. rexxc with no
#     output file parses WITHOUT EXECUTING, so nothing in the file runs. It
#     reports the file's own line number. The version banner goes to stdout and
#     the error to stderr, so redirect to separate the two.
build/bin/rexxc bad.rex 2>&1 1>/dev/null; echo "rc=$?"
#   ->    1 *-* x = )
#         Error 37 running .../bad.rex line 1:  Unexpected ",", ")", or "]".
#         Error 37.2:  Unmatched ")" in expression.
#         rc=219
```

```bash
# (b) run the bad file. Same error text and same rc as (a) for a PARSE error,
#     because ooRexx parses the whole file before executing anything -- but it
#     runs the file when the file is valid, and it also reports RUNTIME errors,
#     which this phase must not gate on. Use only to confirm (a).
build/bin/rexx bad.rex 2>&1; echo "rc=$?"
```

```rexx
/* (c) INTERPRET a fragment inside an installed trap. The trap fires and
   condition('o')~code is available -- but POSITION is the line of the
   INTERPRET instruction, not a position inside the fragment. */
signal on syntax name oops
interpret "x = )"
exit 0
oops: say condition('o')~code; say condition('o')~position
```

Use **(a)** as the default, including for anything positional. Use (c) only when
you want the condition object's fields, which stderr does not expose separately.

**(a) also gives the negative direction, which the other two cannot.** `rexxc`
answering rc 0 means *this file parses*, so a case can be recorded as "must not
raise a parse error" rather than only as "must raise error N". That is what
separates a parse error from a runtime one: bare `procedure`, bare `leave` and
`x = 1/0` all give `rexxc` rc 0 and fail only under `rexx` (17, 28, 42), so none
of them belongs in this task's expectations. Task 3.6 Step 4 has the measured
table.

- [ ] **Step 2: Write the failing tests from those recordings**
- [ ] **Step 3: Implement**
- [ ] **Step 4: Verify what the oracle actually exposes — there is no column**

The condition object carries exactly these, and none of them is a column:
`PROPAGATED ERRORTEXT MESSAGE STACKFRAMES POSITION INSTRUCTION CODE RC
CONDITION PACKAGE TRACEBACK PROGRAM ADDITIONAL DESCRIPTION`. `POSITION` is the
**line**. Nothing on stderr carries a column either — ooRexx locates an error
by *quoting the offending token* in the message text, not by offset.

So verify **number, sub-number, line, and the substitution values**. The
substitutions are where the token gets quoted, so they are what actually pins
the location, and `ADDITIONAL` exposes them separately from the rendered text
exactly as `rexx-num` already models.

Keeping a column internally is still worth it for future tooling, but nothing
in this phase can check it against the oracle, so nothing in this phase may
gate on it.

- [ ] **Step 5: Commit**

---

## Task 3.9: `TRACE` source lines (`*-*` only)

**Files:**
- Modify: `rust/crates/rexx-parse/src/source.rs`, `src/clause.rs`, `src/ast.rs`
- Test: `rust/crates/rexx-parse/tests/sourceline.rs`

**Interfaces:**
- Consumes: `Instruction::clause_span`, produced by Task 3.4 and split by
  Task 3.6, and the `ProgramSource` inside `Program`/`Fragment` from Task 3.7b.
- Produces: the source-text slice per clause that `TRACE`'s `*-*` line needs. Nothing else — no depth field, no value-trace hooks.

`src/clause.rs` and `src/ast.rs` are in the Files list because Step 3 adjusts
spans, and a span this task finds wrong is a span Task 3.4 or Task 3.6 produced.
The AST must retain whatever `TRACE` displays; discovering later that it does not
is a rework of every node type.

`TRACE` indents by nesting depth, but **do not store a depth on each node.**
Measured: indentation is static nesting within a code body plus one level per
call frame, so the static half is derivable from the AST's own structure and
the dynamic half is the executor's stack depth. Storing it would be an AST tax
that buys nothing and has to be kept correct forever.

Scope this task to `*-*` and stop. That marker is the *source* clause, so it
needs no evaluated value and no executor. It is **not** free, though: it needs
the clause spans that Task 3.4 produces and Task 3.6 splits, which are a
different thing from what `SOURCELINE` and error reporting need. `SOURCELINE`
slices whole lines out of the retained source and error reporting resolves a
single byte offset to a line, and neither can express `if y > 5 then say "big"`
as three separate texts. An earlier draft of this plan claimed the clause spans
came free with those two, and Task 3.4 consequently had no rule producing them.

**The value traces are deliberately out of scope for Phase 3, and for this
plan.** **Every marker except `*-*`** carries an evaluated value, so all of them
can only be produced by an executor. Do not enumerate them: the obvious list
`>L> >O> >V> >>> >=>` is incomplete, and measured against
`ootest/ooRexx/base/keyword/TRACE.testGroup` the file also contains `>K>`, `>I>`,
`>A>`, `>M>`, `>F>`, `>E>`, `>R>`, `>N>`, `>C>` and `>P>` — fifteen distinct
markers in all. `>K>` alone appears 33 times, and once in a two-line probe
(`>K> "TO" => "2"` from `do i = 1 to 2`). "Everything except `*-*`" cannot go
stale; a list can.

They are a real conformance item — that testGroup is 1,338 lines with **135**
lines carrying `*-*` and **243** carrying a value marker — but committing to them
shapes the executor, because emitting an event per evaluation step is what forbids
constant folding, operation fusion and skipping intermediate materialisation.

That tension belongs in a Phase 4 decision block, made deliberately, not
inherited from a Phase 3 plan that had no business deciding it. The one thing
Phase 3 owes Phase 4 is that the AST can *reconstruct* clause source; whether
the executor emits per-operation values is Phase 4's call, and the natural
answer is a separate traced path since `TRACE` is off by default.

- [ ] **Step 1: Capture `TRACE` output from the interpreter**

Run `rust/corpus/lang/trace_output.rex` under `build/bin/rexx` and record it
through `cat -A`, so trailing blanks are visible. They matter: the file's own
output is

```
     2 *-* x = 1 + 1$
     3 *-* y = x * 3$
     4 *-* if y > 5 $        <- trailing blank; the clause STOPS before `then`
     4 *-*   then$           <- `then` is a clause of its own, mid-line
     4 *-*     say "big"$    <- third clause on the same source line
     5 *-* trace off$
```

Six `*-*` lines from a six-line file, three of them on source line 4. That is
Task 3.4's rule 4 and Task 3.6's `split_before` observed from the outside.

`trace_output.rex` contains **no** `;` and no label, so it does not exercise Task
3.4's rule 2 or rule 3. Record two scratch probes as well — put them in the
session scratchpad, do not edit the corpus, which Phase 4 also depends on:

```rexx
/* probe A: terminators are inside the clause span */
trace r
nop;
do i = 1 to 2; say i; end
trace off
```

```rexx
/* probe B: a label is its own clause, colon included */
trace r
here: nop; say "two"
trace off
```

Measured, probe A traces `nop;`, `do i = 1 to 2;` and `say i;` **with their
semicolons** (the loop body indented one level, which acceptance strips), and
probe B traces `here:` / `nop;` / `say "two"` as three clauses.

- [ ] **Step 2: Write a failing test that reconstructs that text from the AST**
- [ ] **Step 3: Implement, adjusting *clause* spans if reconstruction is impossible**

Acceptance: for every clause in `trace_output.rex` and in both Step 1 probes, the
text reconstructed from the AST is **byte-identical** to the corresponding
**`*-*`** line the interpreter prints, after stripping the line number, the marker
and the leading indentation — and **nothing else**. In particular a terminating
`;` and a trailing blank before a `then` are part of the expected text, not
whitespace to be trimmed. Reconstruct from `Instruction::clause_span`, not from
the node's own extent; the two differ exactly where this task is hardest.

`*-*` is the *source* marker and is the only one Phase 3 can produce. Under
`trace i` the interpreter also emits value markers, and every one of those carries
an evaluated **value**, which requires execution:

```
     2 *-* x = 1 + 1      <- source. This is Phase 3's business.
       >L>   "1"          <- value. Phase 4's.
       >O>   "+" => "2"   <- value.
       >>>   "2"          <- value.
       >=>   X <= "2"     <- value.
```

An earlier draft named `>>>`/`>V>` in this task's acceptance, which would have
made it and gate criterion 6 need an interpreter -- reintroducing the exact
Phase 2 failure into the one criterion that was already clean.

If a clause cannot be reconstructed, fix the span that produced it, in
`src/clause.rs` or wherever Task 3.6 set it, and record which construct needed it.
Do **not** widen the enclosing node's span to cover the shortfall: widening the
`If` node to cover `then say "big"` produces one span where the oracle prints
three lines, and a node whose span does not match its own source text is a defect
that surfaces again in error reporting.

- [ ] **Step 4: Commit**

---

## Task 3.10: Parse throughput on the bootstrap files

**Files:**
- Create: `rust/crates/rexx-parse/benches/parse.rs`
- Modify: `rust/crates/rexx-parse/Cargo.toml`

**Interfaces:**
- Consumes: the whole parser.

- [ ] **Step 1: Add criterion**

`criterion = "0.8.2"`, `[[bench]] name = "parse" harness = false`. Match the
settings the existing benchmarks use — `sample_size(10)`, 500 ms warmup, 30 s
ceiling — or the numbers are not comparable with `perf-baseline.md`.

- [ ] **Step 2: Benchmark the real files, not synthetic input**

```rust
// interpreter/RexxClasses/CoreClasses.orx   4,193 lines
// interpreter/RexxClasses/StreamClasses.orx 1,010 lines
```

- [ ] **Step 3: Assert the parse succeeded inside the benchmark**

Phase 2 learned this the hard way: a timing comparison means nothing unless
both sides do the same work. Assert a node count, so a parser that silently
stops early cannot post a good number.

- [ ] **Step 4: Record the number against the cold-start budget**

C++ cold start is 5.1 ms from a memory-mapped image; the budget is ~55 ms
total. Write the measurement into `d10-decision.md` and say plainly whether it
fits.

- [ ] **Step 5: Commit**

---

## Exit gate

- [ ] Every program in `rust/corpus/lang/` parses without error, **and** each one
      **tiles**, which for this AST means two separable properties:

      1. **Expressions nest.** Every `Expr` node's span contains the spans of its
         operands.
      2. **Instructions are ordered.** Consecutive instruction spans are in source
         order and do not overlap, and the only bytes between one instruction's
         span and the next are whitespace, comments and `,`/`-` continuations.

      Property 1 is stated for expressions and property 2 for instructions on
      purpose, because Task 3.1 Step 3b may make instructions a flat chain rather
      than a tree, in which case they are siblings and containment does not apply
      to them. The criterion holds either way.

      Tiling rather than concatenation, because concatenation cannot work.
      Expression spans nest, so summing all of them reproduces the text several
      times over; summing only leaf spans loses the text that belongs to interior
      nodes — the blank in `a b` *is* the abuttal operator, so leaf spans give
      `ab` where the source has `a b`.

      **Interstices are permitted, and only whitespace-class bytes may sit in
      them.** Leading indentation, blank lines and comments belong to no node, so
      demanding coverage "without gaps from the first byte to the last" would fail
      on every file that indents anything. A `,`/`-` continuation is a special
      case worth naming: the scanner turns it into a significant blank, which is
      an abuttal operator and therefore *is* inside a node's span, so a
      continuation may legitimately be covered rather than in an interstice.
      Permitting it either way keeps the check independent of that detail.

      This still catches what the criterion is for: a dropped clause leaves a gap
      containing **non-whitespace** bytes, and a mis-nested expression breaks
      containment. "No error raised" catches neither.
- [ ] Every `Instruction` and `Expr` variant is constructed at least once,
      asserted by a test that enumerates the variants rather than by inspection.
      Where a variant is still unreachable, add a program that reaches it — an
      unconstructed variant is untested code that Phase 4 will nonetheless dispatch
      on.

      **Run the enumeration over the corpus and `samples/` together, and
      `samples/` is the primary instrument.** The corpus is 14 hand-written
      programs today and cannot cover 52 instruction and 17 expression classes
      honestly; `samples/` is 301 real programs that the criterion below already
      parses, so collecting variant tags while parsing them costs nothing extra.
      Hand-written additions then fill the residue, which is the small set worth
      writing by hand.
- [ ] Every `.rex` file under `samples/` round-trips to an AST — **301 files,
      67,519 lines**. This is the parent plan's own Phase 3 criterion and it is
      cheap on the oracle side: all 301 pass `build/bin/rexxc` today (measured), so
      the expected answer for every one of them is "parses", and the whole oracle
      half is one shell loop:

      ```bash
      # NOT samples/*.rex -- that glob is only the 36 top-level files. Recurse.
      find samples -name '*.rex' | while IFS= read -r f; do
        build/bin/rexxc "$f" >/dev/null 2>&1 || echo "FAIL $f"
      done
      ```

      A Rust-side failure is therefore unambiguous, with no per-file expectation
      to curate.
- [ ] `CoreClasses.orx` and `StreamClasses.orx` parse end to end.
- [ ] For every **parse-time** error the parser raises: number, sub-number, line
      and **substitution values** match the oracle. Number, sub-number and line
      come from `build/bin/rexxc bad.rex 2>&1 1>/dev/null`, which prints
      `Error N running … line L` and `Error N.S` and executes nothing. The
      substitution values are spliced into that same text; where the error is also
      reachable through `interpret`, cross-check them against
      `condition('o')~additional`, which exposes them unspliced. Not column — the
      oracle exposes none, and gating on something unobservable is the mistake
      Phase 2 made three times over.

      "Parse-time" is defined by `rexxc`, not by judgement: an input `rexxc`
      rejects is a parse error and belongs here; an input `rexxc` accepts is not,
      even if `rexx` fails on it. Bare `procedure` (17.1), bare `leave` (28.1) and
      `x = 1/0` (42.3) all get rc 0 from `rexxc` and are therefore Phase 4's, and
      **this parser must accept them**. Without that line the comparison set is
      undefined and an implementer can build a runtime check in the wrong phase.
- [ ] `SOURCELINE(n)` matches the interpreter for every line of every corpus
      program, including the last line and a file without a trailing newline.
      Oracle side via a **separate driver** so the files under test are not
      edited: `.Package~new(f)~source` returns the line array with no terminators.
      Verified — 27 items for `do_variants.rex`, matching `wc -l`. Constructing the
      package **runs the file's prolog**, so its output interleaves with the
      driver's; give the driver's lines a unique prefix and filter on it.

      ```rexx
      /* srclines.rex -- run as: rexx srclines.rex corpus/lang/do_variants.rex */
      parse arg f
      a = .Package~new(f)~source
      say "SRCLINES:" a~items
      do i = 1 to a~items
        say "SRC" i "[" || a[i] || "]"
      end
      ```
- [ ] `TRACE`'s `*-*` source lines are reconstructible from the AST for
      `trace_output.rex` and for Task 3.9 Step 1's two probes, including a
      terminating `;` and the trailing blank before a mid-line `then`. **Every
      marker except `*-*`** is explicitly **not** gated here — all of them carry
      an evaluated value, so they need an executor, and committing to them shapes
      Phase 4's optimisation choices. The exclusion is phrased as "everything
      except `*-*`" rather than as a list because the list is longer than it
      looks: `TRACE.testGroup` alone contains **fifteen** distinct value markers,
      and the obvious five are not even the five most frequent.
- [ ] Parse throughput on the 5,203 bootstrap lines is recorded against the
      ~55 ms cold-start budget, with a plain statement of whether it fits.
- [ ] `cargo clippy --offline --workspace --all-targets -- -D warnings` clean;
      zero `unsafe`.
- [ ] Phase 2's twelve differential sets still at 0 — 128,368 cases.

## Notes carried in

- **Write the gate criteria so this phase can satisfy them.** Phase 2's gate
  asked for things needing an interpreter that did not exist until Phase 4;
  three of its five criteria were unassessable. Every criterion above is
  reachable with a parser and the two oracle binaries, and nothing else.
- **`build/bin/rexxc` is the parse-only oracle, and it is the default one.**
  `rexxc FILE` with no output file syntax-checks without executing: measured, a
  file that runs `address system` then `"echo hi"` gives rc 0 and produces no
  output under `rexxc`, while `rexx` runs the command. The banner goes to stdout
  and errors to stderr, so `rexxc FILE >/dev/null 2>&1` gives a clean parse
  verdict and `rexxc FILE 2>&1 1>/dev/null` gives the error text alone. Two
  things follow. It gives the **negative** direction — *this file parses* — which
  running cannot, and it **draws the parse-time/runtime line** that gate criterion
  4 needs: bare `procedure`, bare `leave` and `x = 1/0` all get rc 0 from `rexxc`
  and fail only under `rexx`. Three earlier review rounds checked this plan against
  the C++ and the running interpreter and never against `rexxc`, which is how six
  method questions stayed open.
- **Target error boundaries deliberately.** Phase 2's corpus sampled values and
  missed a defect that lived exactly where a valid result becomes an error.
  For a parser the equivalent is the boundary between a program that parses and
  one that does not — build cases that straddle it rather than hoping value
  sampling reaches them.
- **A probe drawn from one dimension cannot reveal a second.** Phase 2's
  `NUMERIC DIGITS` rule took three attempts because each probe set was drawn
  from one dimension: integers only, then integers from varied starting states,
  before anyone tried a fraction. When probing the interpreter here, vary a
  dimension you have no reason to think matters.
- **The keyword table is sorted, and that is deliberate.** `keywordInstructions[]`
  is alphabetical and `resolveKeyword` (`KeywordConstants.cpp:417`) binary-searches
  it; the instruction codes are stored explicitly in each entry, never implied by
  position. This is the *opposite* of the builtin-function table in Phase 0,
  which is positional and must not be sorted — an earlier draft of this plan
  carried the Phase 0 warning across, and it does not transfer. Check which kind
  of table you are looking at before assuming either.
- **Keywords are not reserved.** `if = 2; say if` prints 2, and
  `if if = 2 then say if` parses with the same spelling as both keyword and
  variable. A symbol is a keyword only by position — first token of a clause
  that is not an assignment — so keyword recognition cannot live in the
  scanner. Getting this wrong is not a bug to fix later; it is a rewrite of
  the scanner and every instruction parser. `corpus/lang/keyword_as_variable.rex`
  exists to catch it; before it was written, nothing in the corpus did.
- **The AST must not discard source.** `SOURCELINE`, error reporting and
  `TRACE` all expose original text. Nodes hold byte ranges into one retained
  `String`, and the retained `String` travels in the same struct as the nodes —
  `Program` and `Fragment` both, which is why `parse_interpret` cannot return a
  bare `Vec<Instruction>`.
- **A clause span is not a node span.** `Instruction::clause_span` runs to the end
  of the clause's terminating token, so `nop;` includes its `;` and `here:`
  includes its `:`, while `if y > 5 then say "big"` is three clauses whose first
  ends at the `THEN` token's start byte and therefore carries a trailing blank.
  `TRACE`'s `*-*` line prints `clause_span`; error reporting and `SOURCELINE` use
  the retained source directly. Task 3.4 produces these spans, Task 3.6 splits them, Task 3.7b retains
  them and Task 3.9 checks them — a defect here surfaces four tasks downstream as
  a rework of every node type, which is why it is front-loaded.
