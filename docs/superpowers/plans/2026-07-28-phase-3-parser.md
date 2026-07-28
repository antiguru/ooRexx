# Phase 3 — Parser Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `rexx-parse` — turn Rexx source text into an AST that Phase 4 can execute, with error messages, `SOURCELINE` and `TRACE` output matching the interpreter exactly.

**Architecture:** A hand-written scanner and clause splitter feed a parser that produces plain owned Rust data (D13). The program source is retained as one `String`; every AST node holds a byte range into it, because `SOURCELINE`, error reporting and `TRACE` all expose original text. Whether the layer above the token stream uses `chumsky` combinators or hand-written recursive descent is decided by the Task 3.1 spike, not assumed here.

**Tech Stack:** Rust 2024, `rexx-num` (already built), optionally `chumsky` 0.13.0 (present in the offline registry cache). No other new dependencies.

## Global Constraints

- Behaviour is defined by what `build/bin/rexx` does, **not** by the ANSI standard or the documentation. Where they disagree, the interpreter wins.
- Zero `unsafe`. `unsafe_code = "forbid"` at `[workspace.lints.rust]`; every crate carries `[lints] workspace = true`.
- Error numbers **and sub-numbers** are contract. Programs trap on them.
- Every `cargo` command takes `--offline`.
- `rexx-parse` depends on `rexx-num` and nothing else in the workspace. It must not depend on an executor.
- The AST is plain owned Rust data (D13, closed). Not garbage-collected, not reference-counted between nodes.
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
| **total** | **17,483** | |

19 token classes (`Token.hpp`), 35 keyword→instruction mappings
(`KeywordConstants.cpp`), 52 instruction classes, 17 expression classes.

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
        lib.rs          # public API: parse_program, ParseError
        source.rs       # ProgramSource: the retained text, line index, SOURCELINE
        token.rs        # Token, TokenKind, Span
        scanner.rs      # source -> tokens; comments, continuations, literals
        clause.rs       # tokens -> clauses; the `;`/EOC and label rules
        expr.rs         # expression grammar (construction per D10)
        instruction.rs  # the 35 keyword instructions
        directive.rs    # ::class, ::method, ::routine, ::requires, ::attribute
        ast.rs          # the node types Phase 4 consumes
        error.rs        # ParseError -> interpreter error number, sub-number, line, column
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
a = b = c                      /* comparison chaining */
```

For each, capture the interpreter's answer with a driver that prints the
evaluated result, so both spike implementations are checked against the same
ground truth rather than against each other.

- [ ] **Step 2: Build both implementations**

Same token stream, same AST output type. Timebox each; if one runs long, that
is itself a result and should be recorded rather than pushed through.

- [ ] **Step 3: Measure the three axes**

The parent plan fixes these and no others:

1. **Lines of code** — the whole expression grammar, excluding the shared
   token stream.
2. **Error fidelity** — at each failure site, can the exact interpreter error
   number *and* position be produced? Test with deliberately malformed
   expressions and compare against `build/bin/rexx`. This is the axis most
   likely to decide it, because Phase 3's gate is error messages with line and
   column.
3. **Parse throughput on `CoreClasses.orx`** — not on synthetic input. Under
   D2 this number is cold-start time.

- [ ] **Step 4: Write `d10-decision.md`**

Record the three measurements, the decision, and — importantly — what would
change it. The parent plan's starting position is (a): hand-written scanner
and clause splitter with `chumsky` above the token stream. State plainly if
the measurements contradict that.

- [ ] **Step 5: Delete the spike crate and commit**

```bash
rm -rf rust/crates/rexx-parse-spike
git add docs/superpowers/plans/d10-decision.md rust/Cargo.toml
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

This is below the line where combinators help, so it is hand-written
regardless of the D10 outcome.

- [ ] **Step 1: Read the C++ scanner's hard cases**

`interpreter/parser/Scanner.cpp`, 1,955 lines. The parts that matter and are
easy to get wrong:

- `--` line comments versus the subtraction operator
- `/* */` comments, which **nest** in Rexx
- the continuation comma at end of line, which is not an operator there
- quoted literals with doubled quotes, and the `'…'x` / `'…'b` suffixes
- blanks as significant tokens (`TOKEN_BLANK`) — abuttal concatenation needs
  them, so they cannot be silently dropped

- [ ] **Step 2: Write failing tests for each**

```rust
#[test]
fn block_comments_nest() {
    let toks = scan_ok("/* a /* b */ c */ say 1");
    assert_eq!(kinds(&toks), [TokenKind::Symbol, TokenKind::Blank, TokenKind::Symbol, TokenKind::Eoc]);
}

#[test]
fn double_dash_starts_a_line_comment_but_minus_does_not() {
    assert_eq!(kinds(&scan_ok("a -- b")), [TokenKind::Symbol, TokenKind::Blank, TokenKind::Eoc]);
    assert_eq!(kinds(&scan_ok("a - b")).len(), 5);
}

#[test]
fn doubled_quotes_are_one_quote() {
    assert_eq!(literal_text(&scan_ok("'it''s'")), "it's");
}
```

- [ ] **Step 3: Run them and watch them fail**

- [ ] **Step 4: Implement the scanner**

Work over bytes, not chars: Rexx source is byte-oriented and the column
numbers in error messages count bytes. Emit spans, never copied strings —
Task 3.2 retains the text and the AST holds ranges into it.

- [ ] **Step 5: Differential-test against the interpreter**

Write a `.rex` driver that reads a program and reports what the interpreter
makes of its structure, and compare on the 14 L0 corpus programs. Where no
such introspection exists, fall back to: a program that scans correctly runs,
and one that does not raises a syntax error with a specific number and line.

- [ ] **Step 6: Commit**

---

## Task 3.4: Clause splitting

**Files:**
- Create: `rust/crates/rexx-parse/src/clause.rs`
- Test: `rust/crates/rexx-parse/tests/clause.rs`

**Interfaces:**
- Consumes: `Vec<Token>` from Task 3.3.
- Produces: `split_clauses(&[Token]) -> Result<Vec<Clause>, ParseError>` where
  `Clause { tokens: Range<usize>, label: Option<Range<usize>> }`.

- [ ] **Step 1: Establish the rules from the C++**

`interpreter/parser/Clause.cpp`. A clause ends at `;`, at end of line unless
continued, or at `:` for a label. The continuation comma and the `-` line
continuation both suppress the end-of-line break.

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
```

- [ ] **Step 3: Run, fail, implement, pass**

- [ ] **Step 4: Commit**

---

## Task 3.5: Expression grammar

**Files:**
- Create: `rust/crates/rexx-parse/src/expr.rs`, `src/ast.rs`
- Test: `rust/crates/rexx-parse/tests/expr.rs`

**Interfaces:**
- Consumes: clauses from Task 3.4.
- Produces: `Expr` in `ast.rs`, and `parse_expr(&[Token]) -> Result<Expr, ParseError>`.

Built the way Task 3.1 decided. The spike's implementation is a reference, not
a starting point — it was timeboxed and is not production code.

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
`build/bin/rexx`, and evaluate the parsed AST with a throwaway evaluator that
uses `rexx-num` for arithmetic. Any divergence is a precedence or associativity
error. This is the Phase 2 method applied to structure.

- [ ] **Step 5: Commit**

---

## Task 3.6: The 35 keyword instructions

**Files:**
- Create: `rust/crates/rexx-parse/src/instruction.rs`
- Test: `rust/crates/rexx-parse/tests/instruction.rs`

**Interfaces:**
- Consumes: `Expr` from Task 3.5, clauses from Task 3.4.
- Produces: `Instruction` in `ast.rs`, `parse_instruction(&Clause) -> Result<Instruction, ParseError>`.

`KeywordConstants.cpp` has 36 keyword constants and 35 keyword→instruction
mappings; `interpreter/instructions/` has 52 classes. **The table is not
alphabetical and the C++ indexes it by position** — a lesson already paid for
in Phase 0 with the builtin table. Do not sort it.

- [ ] **Step 1: Extract the keyword list, in source order**

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
and every instruction parser, and the corpus will not catch the mistake —
`rust/corpus/lang/` contains no program that uses a keyword as a variable.
Add one.

Write one test per family before implementing any of them, then work through
the families in that order. Step 4 below is what proves nothing was skipped;
if your extraction in Step 1 yields a keyword absent from these five lists,
trust the extraction and fix the list.

- [ ] **Step 3: Implement family by family, committing per family**

`DO` is the largest single instruction in the C++ and deserves its own commit.

- [ ] **Step 4: Assert every keyword is reachable**

```rust
#[test]
fn every_keyword_parses_to_an_instruction() {
    for kw in ALL_KEYWORDS {
        assert!(parses_as_instruction(kw), "{kw} unhandled");
    }
}
```

- [ ] **Step 5: Commit**

---

## Task 3.7: Directives

**Files:**
- Create: `rust/crates/rexx-parse/src/directive.rs`
- Test: `rust/crates/rexx-parse/tests/directive.rs`

**Interfaces:**
- Produces: `Directive` in `ast.rs`, `parse_directive(&Clause) -> Result<Directive, ParseError>`.

`DirectiveParser.cpp` is 2,867 lines. `::class`, `::method`, `::routine`,
`::requires`, `::attribute`, `::constant`, `::options`, with their option
keywords.

This task matters more than its size suggests: `CoreClasses.orx` is almost
entirely directives, so Task 3.10's throughput number depends on it.

- [ ] **Step 1: Enumerate the directives and their options from the C++**
- [ ] **Step 2: Write a failing test per directive**
- [ ] **Step 3: Implement, commit per directive**
- [ ] **Step 4: Parse `CoreClasses.orx` end to end without error**

That file is the real acceptance test for this task.

- [ ] **Step 5: Commit**

---

## Task 3.7b: The public entry point

**Files:**
- Create: `rust/crates/rexx-parse/src/lib.rs`
- Test: `rust/crates/rexx-parse/tests/program.rs`

**Interfaces:**
- Consumes: everything from Tasks 3.2–3.7.
- Produces: `parse_program(text: String) -> Result<Program, ParseError>` and
  `Program { source: ProgramSource, instructions: Vec<Instruction>, directives: Vec<Directive> }`.
  This is the only entry point Phase 4 uses; everything else stays
  `pub(crate)` so the D10 choice cannot leak into the executor.

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

The first `::` directive ends the main instruction stream. Confirm that against
`build/bin/rexx` rather than assuming it — a trailing instruction after a
directive is a syntax error with a specific number, and that number belongs in
Task 3.8's table.

- [ ] **Step 4: Parse all 14 L0 corpus programs through this entry point**
- [ ] **Step 5: Commit**

---

## Task 3.8: Errors with line and column

**Files:**
- Create: `rust/crates/rexx-parse/src/error.rs`
- Test: `rust/crates/rexx-parse/tests/errors.rs`

**Interfaces:**
- Consumes: `ProgramSource::position` from Task 3.2, `rexx-inventory`'s message table.
- Produces: `ParseError { code: u16, sub: u16, line: usize, column: usize, subs: Vec<String> }`
  with `message(&self) -> String` rendered from the generated table.

**This is the phase's gate, not a finishing touch.** Model it on
`rexx-num`'s error work, which is already done and reviewed: carry the
substitution *values*, render on demand. A rendered `String` cannot be
un-spliced, and `condition('o')~additional` exposes the values separately.

- [ ] **Step 1: Collect ground truth**

For each syntax error the parser can raise, write a `.rex` that provokes it and
record what `build/bin/rexx` prints — number, sub-number, line, column and text:

```rexx
signal on syntax name oops
/* the malformed construct */
exit 0
oops: say "rc=" rc; say condition('o')~message
```

- [ ] **Step 2: Write the failing tests from those recordings**
- [ ] **Step 3: Implement**
- [ ] **Step 4: Verify position, not just number**

Column numbers are the easiest thing to get subtly wrong and the least likely
to be noticed. Test a construct whose error is mid-line, not at its start.

- [ ] **Step 5: Commit**

---

## Task 3.9: `TRACE` output formatting

**Files:**
- Modify: `rust/crates/rexx-parse/src/source.rs`
- Test: `rust/crates/rexx-parse/tests/sourceline.rs`

**Interfaces:**
- Produces: the source-text slices `TRACE` needs per clause.

Phase 3 owns the *formatting* of traced source, not the execution that
triggers it. The AST must retain whatever `TRACE` displays; discovering later
that it does not is a rework of every node type.

- [ ] **Step 1: Capture `TRACE` output from the interpreter**

Run `rust/corpus/lang/trace_output.rex` under `build/bin/rexx` and record it
exactly — the clause text, the leading marker, the indentation.

- [ ] **Step 2: Write a failing test that reconstructs that text from the AST**
- [ ] **Step 3: Implement, adjusting node spans if reconstruction is impossible**
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

- [ ] All 14 `rust/corpus/lang/` programs parse without error, and every
      construct in them appears in the AST — not merely "no error raised".
- [ ] `CoreClasses.orx` and `StreamClasses.orx` parse end to end.
- [ ] For every syntax error the parser raises: number, sub-number, line and
      column match `build/bin/rexx`, verified by provoking each one.
- [ ] `SOURCELINE(n)` matches the interpreter for every line of every corpus
      program, including the last line and a file without a trailing newline.
- [ ] `TRACE` source text is reconstructible from the AST for
      `trace_output.rex`.
- [ ] Parse throughput on the 5,203 bootstrap lines is recorded against the
      ~55 ms cold-start budget, with a plain statement of whether it fits.
- [ ] `cargo clippy --offline --workspace --all-targets -- -D warnings` clean;
      zero `unsafe`.
- [ ] Phase 2's twelve differential sets still at 0 — 128,368 cases.

## Notes carried in

- **Write the gate criteria so this phase can satisfy them.** Phase 2's gate
  asked for things needing an interpreter that did not exist until Phase 4;
  three of its five criteria were unassessable. Every criterion above is
  reachable with a parser and the oracle binary, and nothing else.
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
- **Do not sort the keyword table.** The C++ indexes it by position and it is
  not alphabetical. This already cost time once, on the builtin table in
  Phase 0.
- **Keywords are not reserved.** `if = 2; say if` prints 2, and
  `if if = 2 then say if` parses with the same spelling as both keyword and
  variable. A symbol is a keyword only by position — first token of a clause
  that is not an assignment — so keyword recognition cannot live in the
  scanner. Getting this wrong is not a bug to fix later; it is a rewrite of
  the scanner and every instruction parser. The L0 corpus contains no program
  that exercises it, which is exactly why it would go unnoticed.
- **The AST must not discard source.** `SOURCELINE`, error reporting and
  `TRACE` all expose original text. Nodes hold byte ranges into one retained
  `String`.
