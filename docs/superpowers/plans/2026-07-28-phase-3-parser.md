# Phase 3 — Parser Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `rexx-parse` — turn Rexx source text into an AST that Phase 4 can execute, with error messages and `SOURCELINE` matching the interpreter exactly, and with clause source reconstructible so `TRACE`'s `*-*` lines can be produced. `TRACE`'s value lines are Phase 4's, deliberately — see Task 3.9.

**Architecture:** A hand-written scanner and clause splitter feed a parser that produces plain owned Rust data (D13). The program source is retained as one byte buffer (`Vec<u8>`, because a Rexx literal may hold bytes that are not valid UTF-8); every AST node holds a byte range into it, because `SOURCELINE`, error reporting and `TRACE` all expose original text. `TRACE` needs one range the others do not: the **clause** span, which runs to the end of the clause's terminating token and which `THEN`/`ELSE`/`OTHERWISE` and a label's `:` can cut mid-line. Task 3.4 produces it, Task 3.6 splits it, every `Instruction` carries it. Whether the layer above the token stream uses `chumsky` combinators or hand-written recursive descent is decided by the Task 3.1 spike, not assumed here.

**Tech Stack:** Rust 2024, `rexx-num` (already built), optionally `chumsky` 0.13.0 (present in the offline registry cache). No other new dependencies.

## Global Constraints

- Behaviour is defined by what `build/bin/rexx` does, **not** by the ANSI standard or the documentation. Where they disagree, the interpreter wins.
- Zero `unsafe`. `unsafe_code = "forbid"` at `[workspace.lints.rust]`; every crate carries `[lints] workspace = true`.
- Error numbers **and sub-numbers** are contract, and for **parse-time** errors they are the *only* error property that is. A program can trap a parse error through `INTERPRET` and read the number: measured, `signal on syntax` around `interpret "x = )"` traps with rc 37.
- **Parse errors are deliberately not reproduced 1:1.** Rendered message text and substitution values are *observable* — a trapped syntax error hands the program `ERRORTEXT`, `MESSAGE` and `ADDITIONAL` — so this is a recorded deviation from the oracle, not an unobservable difference. Specifically: error 36's byte-position substitution is not produced at all. Runtime errors are unaffected; `rexx-num`'s numbers, sub-numbers and message text stay byte-exact.
- Every `cargo` command takes `--offline`.
- `rexx-parse` depends on `rexx-num` and nothing else in the workspace. It must not depend on an executor.
- The AST is plain owned Rust data **inside one arena object per code body** (D13, closed). Not garbage-collected, not reference-counted between nodes. The arena half matters: it is what lets nodes reference each other by index instead of by pointer.
- The C++ tree is the oracle and is never modified.
- No task may leave the differential sets from Phase 2 regressed; `rexx-num` is a dependency now.
- **Word-size-dependent limits are hard-coded to the 64-bit value and that is a recorded exposure, not an oversight.** `ARGUMENT_DIGITS` is 18, and the boundary is *observable*: `::options digits 123456789012345678` is rc 0 while nineteen digits is **26.5**, both measured. A differential run against a 32-bit build of the oracle would therefore disagree with this build. Phase 3's gate runs on one platform so nothing here catches it, but `ci/platforms` builds five, so whoever wires the Rust tree into CI needs to know. This is unlike the scanner's `INTEGER_CONSTANT` limit, which Task 3.3 skipped precisely because it is *not* observable.

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
        token.rs        # Token, TokenKind, Span, ParseCtx, TokenCursor,
                        # SymbolId, SymbolTable, Keywords
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
   executing the file.

   **This axis has since been devalued and the D10 decision does not rest on
   it.** Parse errors are no longer reproduced 1:1 — see the Global Constraints —
   so measure the arms on whether each can produce the right *number and
   sub-number* at each failure site, and stop there. Do not measure substitution
   values, and do not measure a column. `d10-decision.md` records that the
   verdict stands on throughput and dependency cost alone.

   Note also, because an earlier draft of this task asserted the opposite: the
   oracle *does* expose a byte position, in errors 36.901 and 36.902. We simply
   do not reproduce it. ooRexx otherwise locates an error by quoting the
   offending token.
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

**One constraint binds the decision, whichever shape wins.** All five of `THEN`,
`ELSE`, `OTHERWISE`, `END` and `WHEN` must remain instructions of their own, each
carrying its own `clause_span`. **None of the five may be absorbed into a parent
node**, however tempting that is under the tree outcome.

The reason is that the oracle traces each as a separate `*-*` clause, so each
needs somewhere to keep its own span, and Task 3.7b keeps no separate clause
list — an absorbed keyword would leave those bytes with nowhere to live.
Measured, all five: `RexxInstructionThen` sets its location to the `THEN` token's
own (`ThenInstruction.cpp:76`), and `trace r` on `if 1 = 1 then say "a"` prints
three `*-*` lines for one source line; `end` prints its own line; `otherwise`
prints alone with no trailing blank; `when 0 = 1 ` prints with its trailing
blanks.

Two of the five are gated directly, and the other three are not, so do not look
for all five in gate criterion 6: `THEN` appears via `trace_output.rex` and `END`
via Task 3.9 Step 1's probe A. `ELSE`, `OTHERWISE` and `WHEN` fall outside
criterion 6's three-file scope but are bound by the same rule, because Task 3.6
Step 4 needs a node named for each of the 35 keywords and the `samples/`
round-trip criterion parses all five in quantity.

This is a constraint on the choice, not the choice itself, and it is stated here
rather than nine tasks downstream because this is where it is cheap to honour.
Task 3.1 is the first task executed, so a shape chosen without it is a shape that
fails Task 3.6 Step 4 five tasks later.

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
- Produces: `ProgramSource::new(text: Vec<u8>, kind: SourceKind) -> ProgramSource`
  and `pub enum SourceKind { Program, Interpret }`,
  `ProgramSource::line(&self, n: usize) -> Option<&[u8]>` (1-based),
  `ProgramSource::line_count(&self) -> usize`,
  `ProgramSource::line_of(&self, byte: usize) -> usize` returning the 1-based
  physical line containing that byte,
  `ProgramSource::line_span(&self, n: usize) -> Option<Range<usize>>`, and
  `ProgramSource::span_bytes(&self, span: Range<usize>) -> Option<&[u8]>`.
  Every later task uses `line_of` for error reporting.

  The last two were added during Tasks 3.3's implementation, because without them
  the crate is unusable and the omission was a defect in this task's original
  interface list. `line_span` is how the scanner works a line at a time and still
  reports absolute offsets: it adds the line's start to every in-line offset, so
  the terminator rules stay here in one place instead of being re-derived by a
  second scanner. `span_bytes` is how a token or clause span becomes bytes again,
  which Task 3.9 needs to reconstruct `TRACE` text.

  **There is deliberately no whole-text accessor.** `span_bytes` returning
  `Option` rather than panicking or clamping is also deliberate: a span from
  anywhere other than the scanner may be out of range or assembled backwards, and
  both must yield no bytes rather than a plausible wrong answer.

Source retention comes first because everything else holds ranges into it.

**The retained source is bytes, not a Rust `String`, and this is not a style
choice.** A Rexx source file may contain arbitrary bytes that are not valid
UTF-8. Measured: a file whose second byte sequence is a raw `FF FE` inside a
literal runs fine, `c2x` gives `FFFE` and `length` gives 2, and invalid bytes in
a comment are ignored as comment text. `String::from_utf8` would reject that
file, so a `String`-typed source rejects legal programs. `Vec<u8>` in, `&[u8]`
out, everywhere.

`SOURCELINE` returns a Rexx string, which is a byte string, so `line` returning
`&[u8]` is the faithful signature rather than a concession.

This costs almost nothing above the scanner, because the one thing that *does*
need `&str` is safe by construction: a symbol cannot contain a non-ASCII byte
(`LanguageParser::characterTable` is zero for every byte 0x80-0xFF, and `bäc = 2`
is error 13.1), so converting a symbol's bytes for interning cannot fail. Do it
with `std::str::from_utf8(...).expect(...)` and say why in the expect message,
because that invariant is the scanner's to maintain. Literal values stay raw
bytes; see Task 3.3.

**Two behaviours found while implementing this task, both in `ProgramSource.cpp`
and both now implemented here.** They are recorded because every later task holds
byte ranges into this text and must agree with it.

* **A Ctrl-Z (`0x1A`) truncates the source**, before any line scanning, even
  mid-line and mid-comment. Everything from that byte onward is discarded.
  Verified two ways: `rexxc` accepts a file whose `x = )` sits after a `0x1A`, so
  truncation happens before parsing; and `sourceline()` reports **2** for a
  three-line file whose second line contains a `0x1A`, with `sourceline(2)`
  returning only the text before it. So the retained text is not the file's bytes,
  and no span may refer past the truncation point.
* **`\r` and `\n` are independently valid terminators.** Measured with
  `sourceline()`: a bare `\r` ends a line (count 2), `\r\n` collapses to **one**
  terminator (count 2), and `\n\r` is **two** terminators producing an empty line
  (count **3**). A rule that only special-cases CRLF is a near-miss that gets
  `\n\r` wrong.

Neither is a line-content question alone. Task 3.3's scanner and Task 3.4's rule 1
both say "end of line", and that phrase means CR, LF, or CRLF-as-one here; "end of
file" means the end of the *truncated* text.

Rexx has **no Unicode string semantics** to reproduce here. `length('ää')` is 4,
`substr(s,1,1)` yields the single byte `C3`, and `reverse` reverses bytes into
invalid UTF-8 — all measured. The interpreter vendors `utf8proc` for exactly one
purpose, decoding the offending sequence so error 13.1 can print a whole
character, and this phase does not reproduce parse-error text at all, so we need
no equivalent.

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
    let src = ProgramSource::new(b"say 1\nsay 2\n".to_vec(), SourceKind::Program);
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
fn line_of_is_one_based() {
    let src = ProgramSource::new(b"say 1\nsay 2\n".to_vec(), SourceKind::Program);
    assert_eq!(src.line_of(0), 1);
    assert_eq!(src.line_of(4), 1);
    assert_eq!(src.line_of(6), 2);
}

#[test]
fn source_may_hold_bytes_that_are_not_utf8() {
    // A Rexx literal may contain arbitrary bytes. Verified against the oracle:
    // a file holding a raw FF FE inside a literal runs, and c2x reports FFFE.
    // A String-typed source would refuse to construct here.
    let src = ProgramSource::new(b"s = '\xff\xfe'\n".to_vec(), SourceKind::Program);
    assert_eq!(src.line(1), Some(&b"s = '\xff\xfe'"[..]));
    assert_eq!(src.line_count(), 1);
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
- Produces: `Token { kind: TokenKind, span: Range<usize> }`, where
  `TokenKind::Symbol` carries a `SymbolId` rather than text; `SymbolId`,
  `SymbolTable` and `Keywords`; `ParseCtx` and `TokenCursor`; and
  `scan(&ProgramSource) -> Result<Scanned, ParseError>` where
  `Scanned { tokens: Vec<Token>, symbols: SymbolTable, keywords: Keywords,
  resources: Vec<ResourceBody> }`.
  `scan` returns the table because it owns interning; it cannot borrow one it is
  still filling. `TokenKind` mirrors the C++ 19 classes in
  `interpreter/parser/Token.hpp`.

  **Two of those differ from this task's first draft, both for measured reasons.**

  **The program-versus-interpret distinction lives on `ProgramSource`, not on the
  scan.** `ProgramSource::new(text: Vec<u8>, kind: SourceKind)` takes it and
  `scan` reads it back off the source, which is why `scan` needs no mode
  parameter and why it is impossible to build a source one way and scan it the
  other.

  It belongs there because **all three** of `new`'s behaviours are program-only,
  and an earlier draft of this task got that wrong by fixing only the first. An
  `INTERPRET` argument is exactly **one physical line** — the oracle builds it as
  `ArrayProgramSource` over a one-element array — so under `SourceKind::Interpret`
  `new` does essentially nothing, and the offending bytes stay on that single line
  for `characterTable` to reject like any other invalid character. No scanner
  special-casing is needed. Measured, every case:

  | under `INTERPRET` | result |
  |---|---|
  | `#! nothing here` | 13.1 on `#` (`'23'X`); as a *file's* line 1 it is accepted |
  | a raw `0a`, `0d`, or `0d0a` | 13.1, the CRLF case naming `'0D'X` |
  | a trailing `0a` | 13.1, so it is not a terminator either |
  | a `1a` anywhere, including last | 13.1, so **Ctrl-Z is not truncation here** |
  | `1a` inside a *literal* | survives as data: `interpret "say c2x('"||'1a'x||"')"` prints `1A` |
  | `;` between clauses | still separates, so the fix must not be broader than this |
  | `interpret ""` | accepted, as **one empty line**, where an empty *program* has none |

  The last two rows are the guard against over-correcting: a fix that rejects
  `;` or refuses an empty fragment has gone too far.

  `resources` exists because a `::RESOURCE` body is **raw text and must never be
  tokenised**. Verified: a body containing `'unmatched and /* unclosed` gives
  `rexxc` rc 0 and `package~resources` returns it verbatim, so tokenising it
  invents errors 6.2 and 6.1. Each entry is keyed by its `::` token index, which
  equals the clause's first token index because a clause can never begin with a
  `Blank`. Task 3.7 consumes it with one integer comparison.

**A named deviation: eager scanning can report the wrong error NUMBER.** `scan`
walks the whole program before parsing begins, while the interpreter interleaves
scanning and parsing. So a scan error later in the file can mask a parse error
earlier in it. Measured: `say )` on line 1 with `'unclosed` on line 3 gives the
oracle 37.2 on line 1 and gives us 6.2 on line 3.

This is **outside** the project's parse-error relaxation, which keeps the number
and sub-number exact and relaxes only message text, substitutions and the
position. It is accepted anyway, by the repo owner's decision, on the measured
grounds that it does not occur on real input: **zero** mismatches across 2,470
oracle scanner-class errors over `ootest/`, `samples/` and `corpus-l1`, because a
program with two independent syntax errors is already broken.

It does occur on **generated** input, at any clause boundary including a mid-line
`;`, hitting **144 of 4,000** adversarial programs. **Task 3.8 must therefore
exclude multi-error inputs from its error corpus**, or it will record our answer
as the expected one and enshrine the deviation as a test.

**`ParseError.byte` is the clause start, not the offending character.** Measured
across 14 crafted cases and 2,426 fuzz hits: 6.1, 6.2, 13.1 and 15.3 all report
the clause's line. That is faithful, and it matches error 36, whose main message
carries the clause's line while its substitution carries the token's. Two
consequences worth stating: the field name reads as "offending byte" and is not
one, and if Task 3.8 ever fills `subs` it needs the offending position as a
**second** field, because 13.1 quotes `"ä" ('C3A4'X)` and 15.3 quotes `found "g"`.

**A plan-wide contradiction, stated once here rather than rediscovered per task.**
This plan declares `ParseCtx`, `TokenCursor`, `Clause` and `ClauseCursor` as
`pub(crate)` while also naming `tests/tokens.rs`, `tests/clause.rs` and
`tests/scanner.rs` as their test files. An integration test under `tests/` is a
separate crate and **cannot see a `pub(crate)` item**, so as written those two
instructions cannot both be satisfied. Two tasks have now hit this independently
and resolved it the same way.

The resolution: `pub(crate)` is the intended **end state**, and each item ships
`pub` until an in-crate caller exists, because with no caller `pub(crate)` trips
`dead_code` and gate criterion 8 runs clippy with `-D warnings`. Do not treat the
`pub(crate)` in the code blocks below as a defect while the tests are where they
are.

**Done in Task 3.5, and here is what it actually cost.** Task 3.5 narrowed all
four items and moved their tests into `#[cfg(test)]` modules beside the code. Two
consequences the plan should carry rather than have each later task rediscover:

* **A task's own `tests/<name>.rs` file may not be reachable.** `tests/expr.rs`
  cannot exist, because `parse_expr` takes `&ParseCtx` and `&mut TokenCursor` and
  an integration test cannot construct either once they are `pub(crate)`. Where a
  Files list names `tests/<name>.rs` for something that touches a `pub(crate)`
  type, the tests go in-crate and the Files list is what is wrong.
* **Narrowing costs `#[allow(dead_code)]`**, because the library target compiles
  with `cfg(test)` off and the items are then unused until a real caller lands.
  `#[expect]` is unusable here: the lint fires in one of the two compilations and
  not the other, so `expect` itself becomes an unfulfilled-expectation warning.
  **Every such `allow` must name the task that deletes it**, and a task that
  becomes a real caller must delete the ones it satisfies. An `allow` with no
  named owner is permanent, which is the failure mode to avoid.

**Settled after Task 3.5: keep the per-task narrowing.** Deferring to one pass at
the end does not avoid the test migration, it enlarges it — 3.6's and 3.7's
instruction tests would all have to move too — and lands it inside the phase's
biggest wiring change. It would also leave `ParseCtx` and `TokenCursor` publicly
reachable for four more tasks with nothing to stop a dependency forming on them.

**The owner rule is mechanical, not prose.** Put the owner in a trailing comment on
the attribute line itself:

```rust
#[allow(dead_code)] // deleted by Task 3.6
```

Task 3.5 proved why this matters: its report tabulated an owner for every
attribute, the code named one for only five of nine, and the table is not the code.
A paragraph above the item does not survive. **Gate criterion 8 therefore also
asserts that every `allow(dead_code)` line in `rexx-parse` matches `Task 3\.\d`**,
which is greppable and auditable.

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
    /// Tasks 3.6 and 3.7 need it to compare a clause's first symbol against the
    /// pre-interned keyword ids, and Task 3.6 needs it to recover a label's
    /// spelling when it builds `Program::labels`.
    ///
    /// Not for error substitutions: this phase does not reproduce them.
    ///
    /// **Not fully read-only, which an earlier draft of this task claimed.**
    /// Task 3.5 found two names the scanner never interned because they are not
    /// symbol tokens: a message name written as a literal, `a~'length'`, which
    /// resolves case-insensitively exactly as `a~length` does (measured, both
    /// give 3), and the bracket form's implicit `[]`. Task 3.5 therefore stores
    /// `ExprKind::Message::name` as `Box<[u8]>`, the one name in the tree that is
    /// not a `SymbolId`, rather than widening this to `&mut` or reaching for
    /// interior mutability. Interning at parse time remains possible, but it is a
    /// change to this borrow and so a decision for whoever needs it.
    pub symbols: &'a SymbolTable,
    /// Every reserved *spelling* this parser recognises, pre-interned by `scan`
    /// before it reads any source, so their ids are fixed and every keyword
    /// test is an integer comparison. Keywords are NOT reserved words, so this
    /// is only ever consulted positionally — see Task 3.6.
    pub keywords: &'a Keywords,
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

**Symbols are interned here, and this is the one decision in this task that is
not a port.** A `Symbol` token carries a `SymbolId`, not its text.

```rust
/// A symbol's identity: the upcased spelling, interned. Two symbols with the
/// same `SymbolId` name the same variable, method or label.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SymbolId(u32);

/// Interns upcased symbol spellings. Owned by `ProgramSource`'s parse, handed
/// to `Program` so Phase 4 can resolve a `SymbolId` back to text for error
/// messages and `SIGNAL`'s label lookup.
#[derive(Default, Debug)]
pub struct SymbolTable {
    by_name: std::collections::HashMap<Box<str>, SymbolId>,
    names: Vec<Box<str>>,
}

impl SymbolTable {
    /// Intern `text`, upcasing it. Returns the same id for every spelling that
    /// differs only in case.
    ///
    /// `to_ascii_uppercase` is byte-identical to the interpreter's
    /// `translateChar` over everything this can receive, and the reason is
    /// `LanguageParser::characterTable` (`Scanner.cpp:60`): it maps only `!`,
    /// `.`, `0`-`9`, `?`, `A`-`Z`, `_` and `a`-`z`, and is **zero for every byte
    /// from 0x80 to 0xFF**. A non-ASCII byte therefore cannot be part of a
    /// symbol at all -- `bäc = 2` is a parse-time error 13.1, `Incorrect
    /// character in program "ä" ('C3A4'X)`. This matters because Step 4 says a
    /// UTF-8 byte sequence must survive a round trip through the scanner, which
    /// is true of literals and comments and must not be read as licence to admit
    /// non-ASCII into a symbol, where it would silently under-upcase.
    pub fn intern(&mut self, text: &str) -> SymbolId {
        // Cow, not Box<str>, because `Box<str>: From<&str>` copies: building
        // the key eagerly would allocate on the lookup path even when the
        // symbol is already interned, which is the common case by an order of
        // magnitude. Borrow when the text is already upper, allocate only to
        // upcase, and allocate the owned key only on a genuine miss.
        let key: std::borrow::Cow<'_, str> = if text.bytes().any(|b| b.is_ascii_lowercase()) {
            std::borrow::Cow::Owned(text.to_ascii_uppercase())
        } else {
            std::borrow::Cow::Borrowed(text)
        };
        if let Some(&id) = self.by_name.get(key.as_ref()) {
            return id;
        }
        let id = SymbolId(u32::try_from(self.names.len()).expect("symbols fit u32"));
        let owned: Box<str> = key.into_owned().into();
        self.names.push(owned.clone());
        self.by_name.insert(owned, id);
        id
    }

    /// The upcased spelling. Panics on an id from a different table, which is
    /// a parser bug rather than a source error.
    pub fn name(&self, id: SymbolId) -> &str {
        &self.names[id.0 as usize]
    }

}
```

**`Keywords` covers all six tables, not just the 35.** It gets a definition here
because an earlier draft named it in three places and specified it nowhere,
leaving an implementer to invent the type.

```rust
/// The pre-interned spelling tables. Built by `scan` before it reads any
/// source, so a keyword test never hashes a string.
///
/// One table per C++ table, with the counts the plan's inventory gives:
/// 35 keyword instructions, 50 `subKeywords`, 12 `conditionKeywords`,
/// 10 `parseOptions`, 9 `directives`, 40 `subDirectives`. They are separate
/// because the same spelling means different things in different positions:
/// `VALUE` is a `parseOptions` entry and a sub-keyword of several
/// instructions, and nothing may conflate them.
pub struct Keywords {
    pub instructions: KeywordSet,
    pub sub_keywords: KeywordSet,
    pub conditions: KeywordSet,
    pub parse_options: KeywordSet,
    pub directives: KeywordSet,
    pub sub_directives: KeywordSet,
}

/// One table: the interned spellings, in the order the C++ table lists them,
/// so a hit yields that table's own index and the caller maps the index to its
/// own enum.
pub struct KeywordSet {
    ids: Vec<SymbolId>,
}

impl KeywordSet {
    /// The table index of `id`, or `None` if `id` is not in this set.
    ///
    /// Linear over at most 50 `SymbolId`s, which is a handful of `u32`
    /// comparisons in cache and needs no ordering. Do NOT sort this and do not
    /// binary-search it: an entry's position IS its meaning to the caller.
    pub fn index_of(&self, id: SymbolId) -> Option<usize> {
        self.ids.iter().position(|&k| k == id)
    }
}
```

Callers: Task 3.6 uses `instructions` for the positional first-token test and
`sub_keywords`, `conditions` and `parse_options` inside individual instructions;
Task 3.7 uses `directives` for the token after `::` and `sub_directives` for the
rest of the directive. Task 3.7's resolution goes through this type, not through
a string table.

**Labels are NOT keyed by `SymbolId`, and this is the one place interning must
not be used.** An earlier draft of this plan keyed `Program::labels` by
`SymbolId` and was wrong in both directions.

A label may be written as a symbol **or as a literal string**, and the C++ keys
the table by the token's *value*: upcased for a symbol, verbatim for a literal
(`InstructionParser.cpp:153` accepts `isSymbolOrLiteral()`, `labelNew` keys on
`nameToken->value()` at `:2795-2799`). Both `SIGNAL` and `SIGNAL VALUE` then
match that key by exact string equality. Measured, all six cases:

| program | result |
|---|---|
| `'MiXeD': nop` under `rexxc` | rc 0, a literal label is legal |
| label `'MiXeD':`, `signal value 'MiXeD'` | reaches it |
| label `'MiXeD':`, `signal value 'MIXED'` | error 16.1, `Label "MIXED" not found` |
| label `'MiXeD':`, `signal MiXeD` | error 16.1, `Label "MIXED" not found` |
| label `mIxEd:`, `signal value 'MIXED'` | reaches it |
| label `mIxEd:`, `signal value 'mIxEd'` | error 16.1 |

So `Program::labels` is a `BTreeMap<Box<str>, usize>` keyed by the token value,
and Task 3.6 Step 3 builds it by upcasing a symbol label and keeping a literal
label's case exactly.

**That makes `Literal` the asymmetric token kind, and it needs a decoded value
rather than a span.** A literal's value is *not* a slice of its source bytes:
`'it''s'` has the value `it's`, and the `'…'x` and `'…'b` suffixes convert to
raw bytes. Step 4's "emit spans, never copied strings" is the right default and
does not apply here. So `TokenKind::Literal` carries its decoded value, the span
stays alongside for `TRACE` and `SOURCELINE` as with every other token, and the
label key for a literal label is that decoded value. Interning the literal's
value would be wrong for the same reason as above, since interning upcases. Interning the key would make `signal value 'MIXED'`
succeed where the oracle raises 16.1, and `signal value 'MiXeD'` fail where the
oracle succeeds.

Nothing in this phase's gate would catch that: criterion 5 is parse-time and
16.1 is raised at run time, so it would land straight in the interface Phase 4
consumes. It is written down here because this is where the temptation lives.

There is deliberately no `SymbolTable` lookup-by-name method. The earlier draft
had one solely for this label lookup, which no longer goes through `SymbolId`,
and adding an accessor with no caller would be speculative. Phase 4 may need
name-to-id resolution for dynamically computed variable references; that phase
can add it, together with the upcasing rules those forms actually follow.

**Why upcased, and why the span still matters.** Rexx folds symbol case:
verified, `abc = 1` then `say ABC` and `say aBc` both print 1, and
`Mixed.Case = 5` then `say MIXED.CASE` prints 5. But the *source* spelling is
observable and must survive: `sourceline(1)` returns `abc = 1`, and `trace r` on
`aBc = 2` prints `aBc = 2`, not the upcased form. The C++ makes exactly this
split — `Scanner.cpp:1492-1511` copies the symbol upcased into the token's value
and calls `setUpperOnly()`, while `tokenLocation` keeps the source position.
So interning the upcased spelling is faithful to the oracle, not a deviation
from it.

**This means `Token` keeps its `span` regardless.** The `SymbolId` is the
*identity*; the `span` is the *occurrence*. `TRACE` and `SOURCELINE` read the
span, name resolution reads the id, and neither substitutes for the other.

**What this buys.** Symbol occurrences in the two bootstrap files outnumber
distinct upcased symbols by roughly an order of magnitude, so this replaces
about ten thousand short-string allocations with that many hash probes plus a
few hundred `Box<str>`. It also turns keyword recognition and variable lookup
into integer comparisons: pre-intern the 35 keyword spellings once and the
positional check in Task 3.6 becomes a `SymbolId` equality test rather than a
case-insensitive string compare.

**What it costs, netted off rather than left out.** Pre-interning the six tables
happens per `SymbolTable`, and `parse_interpret` builds a fresh one per call, so
an `INTERPRET` in a loop pays the whole keyword set every iteration and
`Program::symbols` always carries names that never appear in the source.
Negligible against criterion 8, which parses two files once, and stated because a
"what this buys" paragraph with no cost line is not a measurement.

Deliberately not measured more precisely than "roughly an order of magnitude".
Four crude counts over those files gave ratios from 10× to 16×, and they
disagree because stripping `/* */` comments, `--` line comments and quoted
literals correctly requires the very scanner this task builds. The first attempt
reported `THE` as the most frequent symbol, which is the tell that it was
counting English prose out of the licence header. Record the real ratio in the
Step 5 report once the scanner exists, and treat any number quoted before then
as an estimate.

**Interning symbols is not hash-consing the AST, and the plan does not do the
latter.** Two structurally identical subtrees at different source positions have
different spans, so they are not equal terms and cannot share a node without
moving spans into a side table keyed by the node identity that sharing destroys.
Beyond that, a content hash per node would sit on the parse hot path, which is
cold-start time under D2. A term graph is the right shape for an optimiser IR
built *from* this AST, and that belongs to Phase 4 alongside the value-trace
decision, which constrains folding and fusion anyway. Not this phase.

Add to Step 2's tests: `abc`, `ABC` and `aBc` intern to one `SymbolId` while
keeping three distinct spans; a symbol containing a compound tail
(`stem.i.j`) interns as one symbol, matching the C++, which scans the whole
dotted name as a single token and resolves the tail at run time; and
`SymbolTable::name` round-trips to the upcased spelling, not the source
spelling.

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
`scan` emits one `Eoc` at each clause terminator: an explicit `;`, an end of
line that is not continued, or **end of file**. The third matters and is easy to
miss: every Step 2 test below passes a string with no trailing newline and
expects a final `Eoc`, and Task 3.4's rule 1 lists all three terminators. A model
with only the first two contradicts the tests directly beneath it.
It **never emits two `Eoc` in a row** and never
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

**`TokenKind` needs a payload-free tag, because `Symbol` now carries a
`SymbolId`.** Without it the tests below do not compile: an array literal
`[TokenKind::Symbol, ...]` is an E0308, since `TokenKind::Symbol` is a
constructor rather than a value once it has a field.

```rust
/// `TokenKind` without its payloads, for asserting token *shape*.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Tag {
    Symbol, Literal, Operator, Blank, LeftParen, RightParen,
    Comma, Colon, Eoc,
    // ... one per TokenKind variant, mirroring the C++ 19 classes
}

impl TokenKind {
    pub fn tag(&self) -> Tag { /* one arm per variant */ }
}
```

The test helper is `fn kinds(toks: &[Token]) -> Vec<Tag>`, mapping
`t.kind.tag()`. Assert shape with `Tag` and identity with the `SymbolId`
separately, because a test that asserts both at once cannot say which failed.

- [ ] **Step 2: Write failing tests for each**

```rust
#[test]
fn block_comments_nest() {
    // `1` is a symbol in Rexx, and the blank between `say` and `1` is
    // significant: previous token is a symbol, next character starts a symbol.
    let toks = scan_ok("/* a /* b */ c */ say 1");
    assert_eq!(kinds(&toks), [Tag::Symbol, Tag::Blank, Tag::Symbol, Tag::Eoc]);
}

#[test]
fn double_dash_starts_a_line_comment_but_minus_does_not() {
    // build/bin/rexx: say 1 -- 2  =>  1     (the `-- 2` is a comment)
    //                say 1 - 2    =>  -1
    // No `Blank` in either. In "a -- b" the look-ahead past `a `'s blank finds
    // `-`, which starts neither a symbol, a literal, `(` nor `[`, so the blank
    // is discarded; then `--` truncates the line and yields the clause end.
    assert_eq!(kinds(&scan_ok("a -- b")), [Tag::Symbol, Tag::Eoc]);
    // In "a - b" the same look-ahead discards the first blank, and the blank
    // before `b` is insignificant because the previous token is an operator.
    assert_eq!(
        kinds(&scan_ok("a - b")),
        [Tag::Symbol, Tag::Operator, Tag::Symbol, Tag::Eoc]
    );
}

#[test]
fn a_significant_blank_needs_both_sides() {
    // Left side must be a symbol, a literal, `)` or `]`; right side must start
    // a symbol or a literal, or be `(` or `[`.
    assert_eq!(
        kinds(&scan_ok("f (x)")),
        [Tag::Symbol, Tag::Blank, Tag::LeftParen,
         Tag::Symbol, Tag::RightParen, Tag::Eoc]
    );
    assert_eq!(
        kinds(&scan_ok("f(x)")),
        [Tag::Symbol, Tag::LeftParen,
         Tag::Symbol, Tag::RightParen, Tag::Eoc]
    );
}

#[test]
fn a_continuation_becomes_a_significant_blank() {
    // build/bin/rexx: say "a"-  /  "b"   =>  a b     (blank, so a concatenation)
    //                 say "a"||-  /  "b" =>  ab      (previous token is `||`)
    assert_eq!(
        kinds(&scan_ok("say \"a\"-\n\"b\"")),
        [Tag::Symbol, Tag::Blank, Tag::Literal,
         Tag::Blank, Tag::Literal, Tag::Eoc]
    );
    assert_eq!(
        kinds(&scan_ok("say \"a\"||-\n\"b\"")),
        [Tag::Symbol, Tag::Blank, Tag::Literal,
         Tag::Operator, Tag::Literal, Tag::Eoc]
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
index, and `SOURCELINE` and `TRACE` slice the retained byte buffer directly.

Byte orientation is decided by those three things and not by error messages, so
do not reason about it from columns either way. For the record, since an earlier
draft of this task got it backwards: errors 36.901 and 36.902 *do* carry a
position and it is a **byte** offset, which agrees with byte orientation rather
than arguing against it — but this phase does not reproduce that substitution at
all. See Task 3.8 Step 4 and the Global Constraints.

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

"End of line" here means what Task 3.2 measured, not what it looks like: a bare
`\r`, a bare `\n`, or `\r\n` as a single terminator, while `\n\r` is **two**
terminators and yields an empty line between them. "End of file" means the end of
the text Task 3.2 retained, which a Ctrl-Z (`0x1A`) may have truncated before this
task ever sees it. Neither is this task's job to detect, and both are the reason
it must not re-derive line boundaries from the source bytes itself.

**(2) The clause span includes its terminator.** `nextClause` ends the clause
with `location.setEnd(tokenLocation)` where `tokenLocation` is the location of
the *end-of-clause token* (`LanguageParser.cpp:1072`). Verified against
`build/bin/rexx` with `trace r`: `nop;` and `do i = 1 to 2;` are traced **with
their semicolons**, and `here:` **with its colon**. An AST node's own extent
carries neither, which is why `Clause::span` is a separate field.

**(3) A label's `:` terminates the clause, unconditionally.** `here: nop`
is two clauses, `here:` and `nop`. In the C++ this is
`trimClause(); reclaimClause();` at `InstructionParser.cpp:173–174`, driven from
the instruction parser rather than from `nextClause`. Verified with `trace r`:
`here: nop; say "two"` traces as three clauses, `here:` / `nop;` / `say "two"`.

**Not "when tokens follow it" — that was this rule's earlier wording and it is
wrong.** `labelNew` (`InstructionParser.cpp:2809`) sets the label's end to the
colon whatever comes next, so the label clause's span stops at the colon *even
when a `;` is the real terminator*. Measured: `here: ; nop` traces as `here:`
then `nop`, not `here: ;`.

That makes the `;` in `here: ; nop` belong to **no clause at all**, which is a
second instance of the interstitial-byte pattern rule 4 produces, reached by a
different route. It is also the case that proves rules 2 and 3 cannot share one
mechanism: rule 2 puts a `;` inside the span when it is that clause's own
terminator (`nop; say "x"` traces `nop;`), and rule 3 excludes it when a colon
got there first. A single "include the terminator" rule gets one of the two
wrong.

**(4) `THEN`, `ELSE` and `OTHERWISE` end a clause mid-line.** Also driven from
the instruction parser: `trimClause()` at `LanguageParser.cpp:1378`, `:1403`,
`:1465` and `:1494`. `RexxClause::trim` (`Clause.cpp:138`) moves the clause's
*start* forward to the current token and leaves the end alone, and the
instruction that just ended narrows its own end separately — `RexxInstructionIf`
sets its end to the **start offset** of the `THEN` token
(`IfInstruction.cpp:58–66`), which is why the traced text is `if y > 5 ` with the
trailing blank and stops before `then`.

**Owed to Task 3.7b, and this list is the whole of `translateBlock`'s job.** Task
3.6 owns per-clause parsing and the rule-4 splits, with `ClauseCursor` carrying one
bit of block state. Everything needing the full control stack is 3.7b's, named here
so it cannot fall through the crack between the two tasks:

* **The misplaced-block errors**, all raised in `translateBlock`
  (`LanguageParser.cpp:1176`) and none in `nextInstruction`: **7.1** for a `SELECT`
  with no `WHEN` at all (measured: `select`/`end` is 7.1, and so is
  `select case 1`/`otherwise nop`/`end`), 7.2 for a non-`WHEN` in a `SELECT`, 8.2 for an `ELSE` with no `THEN`, 9.2 for an `OTHERWISE` with no
  `SELECT`, 10.1 for an `END` with no `DO`, 14.3, and the stack-dependent parts of
  18.1/18.2. Task 3.6 raises 8.1 for a bare `then` and the one-bit 18.1/18.2 only.
* **The misplaced-label errors**, and `EXPOSE`/`USE LOCAL` must-be-first
  (99.907/99.910), which read `lastInstruction`.
* **99.913, and an earlier draft of this list described it wrongly twice.** It is
  **not** method-specific and **not** a `translateBlock` error. Measured:
  `guard on when 1` is 99.913 in the **main program**, with no method and no
  `EXPOSE` anywhere; `expose a` with `guard on when b` is **also** 99.913; and
  `expose a` with `guard on when a` is **rc 0**. It is raised in `guardNew`, a
  `nextInstruction` constructor, and the rule is that the `GUARD` expression must
  reference at least one variable that is exposed at that point. So the owner is
  whoever builds the **per-body exposed-variable table**, not whoever ports
  `translateBlock`. Deferring it out of Task 3.6 is still right, because 3.6 has no
  such table.
* **The chain indices.** `Instruction` deliberately has no `next` and no jump
  targets, because in a `Vec` the chain is index order and no jump target is
  computable without the stack. 3.7b adds them when it can populate them.
* **`SELECT CASE`'s `WHEN` needs two changes, not one.** The sub-number is already
  threaded, so 35.934 is one argument at one call site. But `parseCaseWhenList`
  builds a **list of case values** where `parseLogical` builds an **AND**, so
  `when a, b then` inside `select case` currently gets the wrong node shape as
  well. Single-expression `WHEN`s are identical, which is why this is easy to miss.
* **`end 1` and `end loop` PARSE**, and are **10.3** — a *matching* error, not a
  parse error, because `isSymbol()` is class-agnostic and a number is a legal block
  name. Whoever writes the matching code needs this; a first test here asserted
  20.909 and was wrong.

**A named deviation, of the same class as the accepted eager-scan one.**
`if 1 = 1` followed by `then: nop` is **35.1** in the oracle and **18.1** here.
Both reject, and only an already-invalid program is affected. The cause is
structural: the C++ still holds the label and its colon in one clause at that
point, so a label spelled `THEN` becomes the `THEN` and the leftover `:` fails,
whereas Task 3.4 has already split the colon off and left nothing that can fail.
Repairing it means undoing that split, which is verified across a million clauses,
or hard-coding 35.1 without the path that produces it. Neither is worth it. The
bare case, `then: nop` alone, is **rc 0** in both.

**The empty-expression sub-number is a parameter, not a placeholder.** It depends
on the instruction the expression sits in rather than on the expression: an empty
expression is **35.918** in an assignment and **35.929** in an `IF`, measured. The
expression grammar therefore mirrors the C++'s `requiredExpression(terminators,
error)` and takes the sub-number from its caller, so Task 3.6 supplies the right
one and the grammar never invents a number the interpreter does not use.

An earlier draft had the grammar emit a placeholder 35.1 and left Task 3.6 owing a
replacement. Passing it in is strictly better: it removes the wrong number instead
of scheduling its removal.

Related, and settled by measurement rather than by reading the C++: `parse_logical`
returns `Result<Expr, _>` and not `Result<Option<Expr>, _>`, because
`if , 1 = 1 then nop` is **35.929** — the oracle raises on an absent *first*
element too. That makes `requiredLogicalExpression`'s own null check dead code in
the C++, so do not port it.

**Task 3.6 also owes error 47.1, `INTERPRET data must not contain labels`.**
Measured: `interpret "here: nop"` gives rc 47 and the message
`INTERPRET data must not contain labels; found "HERE".` Task 3.4's
`split_clauses(&[Token])` cannot raise it, having no source kind, and it should
not be given one: the C++ raises it at the point of label *recognition* from
parser state (`isInterpret()`), which this task reproduces through
`ParseCtx::source` and `SourceKind`, with no signature change anywhere.

Because those are two independent adjustments, **some bytes end up in no clause
at all**: in `if 1 = 1   then    say "a"` the condition keeps its three trailing
blanks, `then` carries none on either side, and `say "a"` starts at `say`, so the
four blanks after `then` belong to neither neighbour. Three spans and **one** gap,
bytes 15..19: the condition's end and `then`'s start coincide, so only the second
boundary leaves bytes behind. Task 3.6's `split_before`
therefore takes an end byte and a restart token separately rather than one cut
point. Gate criterion 1's property 2 already permits whitespace between one
clause span and the next, which is exactly this.

**This task implements rules 1, 2 and 3. Rule 4 is Task 3.6's**, and the split of
work is not the C++'s: see the note after the tests for why rule 3 moves down a
layer and rule 4 cannot. So `split_clauses` must produce clauses that Task 3.6's
cursor can cut further — `tokens` is a range and nothing in `Clause` is shared or
interned, so a sub-range is always constructible.

**`span` is NOT derivable from a token sub-range**, and this is the one place it
would be tempting to assume otherwise. The two move independently, per rule 4. On
the worked example in Task 3.6, the `THEN` clause's tokens are `6..8` while its
span stops at token 6's *end*: deriving the span from the token range would give
`then ` with a trailing blank the oracle does not print. Carry `span` explicitly
and let the caller set its end.

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
    /// The remainder of a clause that `split_before` ended early. Yielded ahead
    /// of `clauses[pos]`. Not necessarily contiguous with the clause it was
    /// split from: see `split_before`.
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

    /// End the current clause at byte `end_at`, and re-present tokens `at..`
    /// as the next clause starting at token `at`'s own start byte.
    ///
    /// **This is not a partition, and that is the whole point.** The oracle
    /// makes two independent adjustments with a gap between them, so bytes
    /// between `end_at` and the next clause's start belong to NO clause. Two
    /// positions are required; a single cut point cannot reproduce the
    /// interpreter, and one that tried would be wrong on one side or the other.
    ///
    /// Callers pass `end_at` as follows:
    ///
    /// * `IF`/`WHEN` pass the `THEN` token's **start** byte, so the condition
    ///   clause keeps its trailing blanks. `RexxInstructionIf` does
    ///   `setEnd(...)` from the `THEN` token's start
    ///   (`IfInstruction.cpp:58-66`).
    /// * `THEN`/`ELSE`/`OTHERWISE` pass their own keyword token's **end** byte,
    ///   so the keyword clause carries no blank on either side.
    ///   `RexxInstructionThen` takes the token's whole location
    ///   (`ThenInstruction.cpp:76`). `RexxClause::trim` (`Clause.cpp:138`)
    ///   moves only the start, which is why the two ends move separately.
    ///
    /// Measured, for `if 1 = 1   then    say "a"` under `trace r`: the
    /// condition clause keeps all THREE trailing blanks, `then` carries none on
    /// either side despite four following it, and `say "a"` starts at `say`
    /// with zero leading blanks. Three spans and one gap, bytes 15..19.
    ///
    /// Panics if `at` is outside the current clause's token range, or if
    /// `end_at` is outside the current clause's byte span. Both are parser
    /// bugs rather than source errors.
    pub fn split_before(&mut self, ctx: &ParseCtx, at: usize, end_at: usize) -> Clause {
        let cur = self.next_clause().expect("split_before with no current clause");
        assert!(cur.tokens.contains(&at), "split_before outside the clause");
        assert!(
            cur.span.contains(&end_at) || end_at == cur.span.end,
            "split_before end byte outside the clause"
        );
        self.pending = Some(Clause {
            tokens: at..cur.tokens.end,
            span: ctx.tokens[at].span.start..cur.span.end,
            label: None,
        });
        Clause { tokens: cur.tokens.start..at, span: cur.span.start..end_at, label: cur.label }
    }
}
```

The two call shapes, so no implementer has to derive them. For
`if 6 > 5 then say "big"` the tokens are `[0]if [1]Blank [2]6 [3]> [4]5
[5]Blank [6]then [7]Blank [8]say [9]Blank [10]"big" [11]Eoc`:

```rust
// Parsing the IF: end the condition clause at the THEN token's start.
let cond = cursor.split_before(ctx, 6, ctx.tokens[6].span.start);
// Parsing the THEN: end it at the THEN token's own end.
let then = cursor.split_before(ctx, 8, ctx.tokens[6].span.end);
```

The blank at token 7 lands in neither clause.
That is legal: gate criterion 1's property 2 permits whitespace between one
clause span and the next, and this is exactly that case.

**Every `Instruction` carries a `clause_span: Range<usize>`**, copied from the
`Clause` that `next_clause` or `split_before` returned. Use that name, because
Task 3.7b retains it and Task 3.9 reconstructs `*-*` lines from it, and neither
of those implementers sees this task's brief. It is not the node's own extent: a
`THEN` is its own `Instruction` whose `clause_span` covers just the `then` token,
exactly as `RexxInstructionThen` sets its location to the `THEN` token's
(`ThenInstruction.cpp:76`). That is consistent with "copied from what
`split_before` returned" only because `split_before` takes an explicit end byte
rather than deriving one, which is why it has two positions and not one.

**Four clause types are not keyword-driven at all,** and one of them is the
default. None of these appears in the 35, and the table's fourth row is the
keyword-driven case itself rather than a fifth keyword-less one:

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

**Those two paragraphs describe the C++ and matter only for reading it and for
the Step 1 extraction. Do not build a sorted string table in Rust.** Task 3.3
pre-interns the keyword spellings before scanning, so `ParseCtx::keywords` holds
their `SymbolId`s and recognition is an integer comparison against the clause's
first token. There is nothing to sort and nothing to binary-search at parse time.

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

**All 35 rows are writable, including the five keyword clauses.** `THEN`,
`ELSE`, `END`, `WHEN` and `OTHERWISE` each get a node of their own regardless of
which shape Task 3.1 Step 3b chose, because Step 3b is explicitly constrained to
keep **all five** as separate instructions carrying their own `clause_span` — see
that step, which names the same five and gives the measurement for each. Task 3.1
is the first task in this phase, so the shape is already known by the time you
write this table; there is nothing to defer.

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
`::METHOD`. Resolution is by position — the token after `::` resolves against
`ctx.keywords.directives`, everything after it against
`ctx.keywords.sub_directives` — the same positional rule as Task 3.6's keywords,
and for the same reason.

Those are `SymbolId` comparisons through `KeywordSet::index_of` (Task 3.3), not
string lookups.

**The positional rule has two real exceptions, and they are not cosmetic.**
`::OPTIONS FORM` and `::OPTIONS NUMERIC` resolve their *argument* against
`subKeywords[]`, not `subDirectives[]` — that is what `token->subKeyword()` does at
`DirectiveParser.cpp:1007` and `:1339`. `NOINHERIT`, `SCIENTIFIC` and `ENGINEERING`
are rows of `subKeywords[]` alone, and `SYNTAX` is the reverse, so resolving those
two against `subDirectives[]` would **reject legal input and accept illegal input**.
Measured: `::options numeric noinherit` is **rc 0**, and `::options numeric syntax`
is **25.935**. `RexxToken::directives[]` and `subDirectives[]` are named in this
task only to say which C++ rows the two sets are built from; do not build a
string table in Rust, for the same reason Task 3.6 gives.

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
  pub fn parse_program(text: Vec<u8>) -> Result<Program, ParseError>;
  pub fn parse_interpret(text: Vec<u8>) -> Result<Fragment, ParseError>;

  pub struct Program {
      pub source: ProgramSource,
      pub instructions: Vec<Instruction>,
      pub directives: Vec<Directive>,
      /// Keyed by the label token's VALUE, not by `SymbolId`: upcased for a
      /// symbol label, verbatim for a literal one. See Task 3.3 for the six
      /// measurements that force this and for why interning the key is wrong.
      pub labels: BTreeMap<Box<str>, usize>,
      /// Retained because a `SymbolId` is meaningless without it: Phase 4
      /// resolves names back to text to report them.
      pub symbols: SymbolTable,
  }

  /// What `INTERPRET` produces. Carries its own source for the same reason
  /// `Program` does: the instruction spans index it and nothing else. It carries
  /// its own `SymbolTable` for the same reason, and the ids in it are NOT
  /// comparable with the enclosing `Program`'s — see Task 3.7b.
  pub struct Fragment {
      pub source: ProgramSource,
      pub instructions: Vec<Instruction>,
      pub symbols: SymbolTable,
  }
  ```

  These two are the only entry points Phase 4 uses; everything else stays
  `pub(crate)` so the D10 choice cannot leak into the executor.

**Both return types retain their own source, and that is not optional.** The
architecture is "the program source is retained as one byte buffer; every AST node
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
    let p = parse_program(b"say 1\n::routine r\n  return 2\n".to_vec()).unwrap();
    assert_eq!(p.instructions.len(), 1);
    assert_eq!(p.directives.len(), 1);
    assert_eq!(p.source.line(1), Some(&b"say 1"[..]));
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
try to.** `ParseCtx` borrows the `ProgramSource`, the `Vec<Token>`, the
`SymbolTable` and the `Keywords`, so the order is: build `ProgramSource` (with
`SourceKind::Program` here and `SourceKind::Interpret` from `parse_interpret` —
that choice is made once, at construction, and `scan` reads it back), `scan`
it into a `Scanned`, build `ParseCtx` borrowing all four, `split_clauses`,
`ClauseCursor::new`, parse everything, drop the context, then move the
`ProgramSource` **and the `SymbolTable`** into `Program`. That works precisely
because every span that survives is a **byte** range into the source rather than
a token index — so no `Instruction` or `Expr` may hold a token index. If one
does, this composition does not compile, which is the correct outcome. A
`SymbolId` is not a token index and may be retained.

**A `Fragment`'s `SymbolId`s are not comparable with the enclosing `Program`'s.**
`parse_interpret` builds its own `SymbolTable`, so id 7 in a fragment and id 7 in
the program that ran the `INTERPRET` are unrelated. Phase 4 must resolve a
fragment symbol through the fragment's own table, and if it ever needs to match a
fragment name against a program variable it must go through the **text**:
`fragment.symbols.name(id)` gives the upcased spelling, and Phase 4 compares that
against whatever it is matching. There is deliberately no name-to-id lookup on
`SymbolTable` for this — see Task 3.3 — so Phase 4 adds one if it needs it, with
the semantics its own forms require. Sharing one table across `INTERPRET` calls
would need `&mut` at execution time and is deliberately not done. Task 3.9 does not care, because `TRACE` reads spans and each carries its
own source.

- [ ] **Step 4: Parse every `rust/corpus/lang/` program through this entry point**

Fourteen today. Gate criterion 2 permits adding more, so count the directory
rather than hard-coding a number.
- [ ] **Step 5: Commit**

---

## Task 3.7c: Block structure and the control stack

**Added after Task 3.6, because 3.7b was quietly absorbing a second task.** 3.7b is
139 lines of composition; `translateBlock` (`LanguageParser.cpp:1176`) is **509
lines raising 13 distinct errors**, measured. Folding one into the other would have
made a small, independently reviewable task into the largest in the phase and hidden
the block work inside a step called "implement the composition".

**Files:**
- Create: `rust/crates/rexx-parse/src/block.rs`
- Modify: `rust/crates/rexx-parse/src/ast.rs`, `src/lib.rs`

**Interfaces:**
- Consumes: the `Vec<Instruction>` and `Vec<Directive>` that Task 3.7b assembles,
  plus `ParseCtx` for `SourceKind` and spans.
- Produces: the same instructions with their chain and jump indices filled in, and
  the block-structure errors. `Instruction` gains its `next` and jump-target
  fields **here**, because this is the first task that can populate them — Task 3.6
  deliberately omitted them rather than ship fields nothing sets.

**Ordering, and one gate consequence.** 3.7b accepts every valid program without
this task, because a flat `Vec` in index order is already the chain. What it cannot
do is *reject* an invalid block structure, so **gate criterion 4 cannot be fully met
until this task lands**. Criterion 2's `samples/` round-trip only needs valid files
and is unaffected.

- [ ] **Step 1: Port the control stack**

`pushDo`/`popDo`/`topDo`/`topDoType`/`topBlockInstruction`
(`LanguageParser.hpp:306-312`). A stack of instruction indices, not of nodes, since
the instructions live in a `Vec`.

- [ ] **Step 2: Record the oracle's answer for all 13 errors before implementing**

The thirteen, from `translateBlock` itself: `Error_Incomplete_do_else`,
`Error_Incomplete_do_then`, `Error_Then_expected_if`, `Error_Then_expected_when`,
`Error_Unexpected_end_else`, `Error_Unexpected_end_nodo`,
`Error_Unexpected_end_then`, `Error_Unexpected_label_do`,
`Error_Unexpected_label_if`, `Error_Unexpected_label_select`,
`Error_Unexpected_then_else`, `Error_Unexpected_when_otherwise`,
`Error_When_expected_whenotherwise`. **10.7** belongs on this list too, being the
mismatch number when an `END` fails to close a `SELECT` rather than a `DO`.

Capture number and sub-number for each from `build/bin/rexxc` with a minimal
program, and put the raw output in the report. Do not infer a sub-number from the
symbolic name.

- [ ] **Step 3: `END` matching**

`END` takes an optional block name, and **the name may be any symbol**:
`isSymbol()` is class-agnostic, so a number is legal. Measured: `end 1`,
`end loop`, `end a.` and `end a.1` all **parse**, while `end a b` is 21.909 and is
raised in Task 3.6 already.

**The mismatch number depends on what the `END` failed to close**, so do not assume
one: `do` / `nop` / `end 1` is **10.3**, and `select` / `when 1=1 then nop` /
`end 1` is **10.7**. Both measured. An earlier draft of this step said 10.3 flatly,
and a first test in Task 3.6 asserted 20.909 for `end 1` and was wrong, so capture
each from the oracle rather than deriving it from the shape of the rule.

- [ ] **Step 4: The must-be-first checks, and the per-body exposed-variable table**

`EXPOSE` and `USE LOCAL` must be the first instruction (99.907/99.910), read from
`lastInstruction`. Those two are `translateBlock`'s.

**99.913 is not, and an earlier draft of this step said otherwise.** It is raised in
`guardNew`, a `nextInstruction` constructor, so it is not a block error and not
method-specific. Measured, all three:

| program | result |
|---|---|
| `guard on when 1` in the **main program**, no method anywhere | **99.913** |
| `::method m` / `expose a` / `guard on when b` | **99.913** |
| `::method m` / `expose a` / `guard on when a` | **rc 0** |

So the rule is that a `GUARD` expression must reference **at least one variable that
is exposed at that point**, and what this step actually owes is the **per-body
exposed-variable table** that the check consults. Task 3.6 deferred it for exactly
that reason: it has no such table.

That also means the check logically belongs at instruction-construction time rather
than at block-assembly time. Either revisit the `Guard` instructions once the table
exists, or state plainly in the report why doing it at assembly time is equivalent.
Do not describe it as a block-structure error, which is how this went wrong the
first time.

- [ ] **Step 5: Finish `SELECT CASE`'s `WHEN` — two changes, not one**

Task 3.6 threaded the sub-number, so **35.934** is one argument at one call site.
The second change is the node: `parseCaseWhenList` builds a **list of case values**
where `parseLogical` builds an **AND**, so `when a, b then` inside `select case`
currently has the wrong shape. Single-expression `WHEN`s are identical either way,
which is why this is easy to miss and why it needs its own test.

- [ ] **Step 6: Commit**

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

**Exclude every input with more than one syntax error, and do it deliberately
rather than by luck.** Task 3.3 scans eagerly while the interpreter interleaves
scanning and parsing, so on a program containing both a scan error and an earlier
parse error the two disagree on the error *number*: `say )` on line 1 with
`'unclosed` on line 3 gives the oracle 37.2 line 1 and gives us 6.2 line 3. That
is a recorded, accepted deviation (see Task 3.3), and it does not arise on real
input — zero mismatches across 2,470 oracle scanner-class errors in `ootest/`,
`samples/` and `corpus-l1`. It arises on *generated* input, at any clause boundary
including a mid-line `;`, in 144 of 4,000 adversarial programs. If this step
records our answer for such a file, it enshrines the deviation as an expected
value and the gate stops being able to see it. One error per input.

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
- [ ] **Step 4: Verify what the oracle exposes, and what we deliberately do not reproduce**

The condition object carries exactly these:
`PROPAGATED ERRORTEXT MESSAGE STACKFRAMES POSITION INSTRUCTION CODE RC
CONDITION PACKAGE TRACEBACK PROGRAM ADDITIONAL DESCRIPTION`. `POSITION` is a
**line number**, and there is no column *field* among them. ooRexx normally
locates an error by *quoting the offending token* in the message text rather
than by offset.

**But "there is no column anywhere in the oracle" is false, and earlier drafts
of this plan asserted it.** Errors 36.901 and 36.902 substitute a position:
`Left parenthesis "(" in position 5 on line 3`. It is a 1-based **byte** offset
within the offending token's own physical line — `x = "ää" || (a` reports
position 15 where the `(` is the 13th character — and its line can differ from
the main message's, which reports the clause's start line. Three review rounds
acted on the false version of this claim, so it is spelled out rather than
quietly corrected.

**What this phase gates: number and sub-number, on a plausible line. Nothing
else.** Reproducing message text and substitution values 1:1 was dropped as a
scope decision, and error 36's position is not produced at all. This is an
*observable* deviation, not an unobservable one: a trapped syntax error hands
the program `ERRORTEXT`, `MESSAGE` and `ADDITIONAL`, so a program reading those
would see a difference. That is accepted.

So this step's job is no longer a differential capture of substitutions. It is
narrower: for each error the parser can raise, confirm the number and sub-number
against `rexxc`, and confirm the generated message table has that row so the text
comes out of the table rather than being hand-written. Do not build machinery to
match spliced text, and do not track per-line byte offsets.

Runtime errors are untouched by any of this. `rexx-num`'s numbers, sub-numbers,
message text and `ADDITIONAL` values stay byte-exact, and this relaxation must
not be read as licence to loosen them.

- [ ] **Step 5: Commit**

---

## Task 3.9: `TRACE` source lines (`*-*` only)

**Files:**
- Modify: `rust/crates/rexx-parse/src/source.rs`, `src/clause.rs`, `src/ast.rs`,
  `src/instruction.rs`
- Test: `rust/crates/rexx-parse/tests/sourceline.rs`

**Interfaces:**
- Consumes: `Instruction::clause_span`, introduced in Task 3.6 and set from the
  `Clause::span` that Task 3.4 produces, and the `ProgramSource` inside
  `Program`/`Fragment` from Task 3.7b. Do not go looking for `clause_span` in
  Task 3.4; what Task 3.4 produces is `Clause::span`.
- Produces: the source-text slice per clause that `TRACE`'s `*-*` line needs. Nothing else — no depth field, no value-trace hooks.

`src/clause.rs`, `src/ast.rs` and `src/instruction.rs` are in the Files list
because Step 3 adjusts spans, and a span this task finds wrong is a span Task 3.4
or Task 3.6 produced. `src/instruction.rs` specifically, because the end byte a
`THEN`/`ELSE`/`OTHERWISE` clause stops at is chosen by the instruction parser
when it calls `split_before`, so a wrong end byte is repaired there and nowhere
else.
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
can only be produced by an executor. Do not enumerate them, and this is the third
attempt at that enumeration to be wrong. The obvious list `>L> >O> >V> >>> >=>`
is incomplete; so is any list built by matching `>X>`, because two of the
eighteen non-`*-*` prefixes are `<I<` and `+++`, and `>.>` defeats a
`[A-Za-z=]` class just as `>>>` does. The authority is the interpreter's own
table: **nineteen prefixes**, `*-*` plus eighteen
(`RexxActivation.hpp:92-110`), and `TRACE.testGroup` exercises all nineteen.
`>L>` leads at 58 occurrences and `>>>` follows at 49; `>K>` appears 33 times,
once in a two-line probe (`>K> "TO" => "2"` from `do i = 1 to 2`).
"Everything except `*-*`" cannot go stale; a list can, and did.

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

Measured, probe A traces `nop;`, `do i = 1 to 2;`, `say i;` **with their
semicolons** (the loop body indented one level, which acceptance strips) **and
`end`**, which is a clause of its own and easy to leave off the list. Those four
clauses produce more than four `*-*` lines, per the per-line acceptance above.
Probe B traces `here:` / `nop;` / `say "two"` as three clauses.

- [ ] **Step 2: Write a failing test that reconstructs that text from the AST**
- [ ] **Step 3: Implement, adjusting *clause* spans if reconstruction is impossible**

Acceptance: for every `*-*` line the interpreter prints for `trace_output.rex`
and for both Step 1 probes, the text reconstructed from the clause that line came
from is **byte-identical** to it, after stripping the line number, the marker and
the leading indentation — and **nothing else**. In particular a terminating `;`
and a trailing blank before a `then` are part of the expected text, not
whitespace to be trimmed. Reconstruct from `Instruction::clause_span`, not from
the node's own extent; the two differ exactly where this task is hardest.

**This is a per-line comparison, not a comparison of two sequences**, and the
distinction is not pedantic. `trace r` re-traces a loop body once per iteration,
so the `*-*` lines outnumber the clauses. Measured: a `do i = 1 to 2` loop over
one `say` prints **seven** `*-*` lines for **three** clauses, because the header,
body and `end` repeat per iteration and the header prints once more for the
exit test. Asserting equal counts would fail on any program containing a loop.

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
      2. **Instructions are ordered.** Consecutive `Instruction::clause_span`s are
         in source order and do not overlap, and the only bytes between one
         clause span and the next are whitespace, comments and `,`/`-`
         continuations.

      Property 1 is stated for expressions and property 2 for instructions on
      purpose, because Task 3.1 Step 3b may make instructions a flat chain rather
      than a tree, in which case they are siblings and containment does not apply
      to them.

      Property 2 is stated over `clause_span` and **not** over node extents, and
      that is what makes it shape-independent. A node extent under the tree
      outcome contains its children: a `Do`'s extent covers its whole body, so
      consecutive node extents overlap on every loop in the corpus and the
      property would fail there. A `clause_span` is per-clause under both
      outcomes, so the criterion holds either way only in this form.

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
- [ ] For every **parse-time** error the parser raises: the **number and
      sub-number** match the oracle, on a **plausible line**. Both come from
      `build/bin/rexxc bad.rex 2>&1 1>/dev/null`, which prints
      `Error N running … line L` and `Error N.S` and executes nothing.

      **Message text and substitution values are deliberately NOT gated**, per
      the Global Constraints. Reproducing them 1:1 was dropped as a scope
      decision: the generated message table makes the number nearly free, while
      matching spliced text and every substitution is a large differential
      exercise for a property no program branches on. Error 36's byte-position
      substitution is not produced at all.

      "Plausible line" rather than "the oracle's line" because the two can
      legitimately differ and one line is not always the answer: error 36 puts
      the clause's start line in the main message and the offending token's own
      line in its substitution. Assert the line the oracle's main message gives
      where the parser reports one line; do not build machinery to reproduce
      both.

      **This criterion cannot be met before Task 3.7c.** The 13 block-structure
      errors are rejections that only the control stack can perform, so until 3.7c
      lands the parser accepts programs the oracle rejects. Task 3.7b accepts every
      *valid* program without it, so criterion 2 is unaffected; this one is not.

      Note for anyone re-reading this criterion later: earlier drafts justified
      dropping a column with "the oracle exposes none". That is **false** —
      errors 36.901 and 36.902 substitute a 1-based byte offset within the
      offending token's physical line. The reason we do not produce it is that we
      chose not to, not that it does not exist.

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
      looks. The interpreter's prefix table has **nineteen** entries
      (`RexxActivation.hpp:92-110`, strings in `RexxActivation.cpp`): `*-*` plus
      eighteen others, and `TRACE.testGroup` exercises all nineteen. The obvious
      five are not even the five most frequent.

      **Count these from the table, never by matching a shape.** Three separate
      attempts to count them by regex were wrong, each time because the pattern
      excluded a marker it was looking for: `[A-Za-z=]` between two `>` misses
      both `>>>` and `>.>`, and any `>…>` shape misses `<I<` and `+++` entirely.
      Sixteen of the eighteen have the `>X>` shape, which is exactly why matching
      on it looks like it works.

      **This narrows the parent plan's criterion, deliberately.** The parent asks
      for `TRACE`'s `*-*` source lines to match the oracle byte-for-byte,
      unscoped; this criterion scopes that to `trace_output.rex` plus Task 3.9
      Step 1's two probes. Recorded here rather than left implicit, because a
      phase plan that narrows its parent's gate must say so — the principle round
      4 established when it caught the `samples/` criterion going missing. The
      narrowing is sound: three files chosen to cover every clause-splitting rule
      buy more than reconstructing every clause of all 301 samples, and the
      `samples/` round-trip criterion already covers breadth.
- [ ] Parse throughput on the 5,203 bootstrap lines is recorded against the
      ~55 ms cold-start budget, with a plain statement of whether it fits.
- [ ] `cargo clippy --offline --workspace --all-targets -- -D warnings` clean;
      zero `unsafe`.
- [ ] **Every `allow(dead_code)` in `rexx-parse` names the task that deletes it**,
      as a trailing comment on the attribute line:

      ```bash
      # must print nothing
      grep -rnE '^\s*#\[allow\(dead_code\)\]' rust/crates/rexx-parse/src/ \
        | grep -v 'Task 3\.[0-9]'
      ```

      **The pattern is anchored to attribute syntax on purpose.** An unanchored
      `allow(dead_code)` also matches the phrase inside a doc comment, and the
      first version of this criterion did: it flagged an explanatory paragraph in
      `lib.rs` as an ownerless attribute. The first fix for that was a rule
      forbidding prose from spelling the attribute out, which is brittle — it is
      enforced by nobody and any future comment silently reintroduces the false
      positive. Anchoring makes the check correct whatever the prose says, so the
      rule is unnecessary and is not imposed.

      These attributes exist because narrowing to `pub(crate)` leaves an item
      unused in the library target until a real caller lands, and `#[expect]`
      cannot be used since the lint fires in one of the two compilations and not
      the other. They are meant to be temporary, so each must say who removes it.
      An `allow` with no named owner is permanent, and Task 3.5 showed prose above
      the item does not hold: its report tabulated an owner for all nine while the
      code named one for five.
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
  byte buffer, and that buffer travels in the same struct as the nodes —
  `Program` and `Fragment` both, which is why `parse_interpret` cannot return a
  bare `Vec<Instruction>`.
- **A clause span is not a node span.** `Instruction::clause_span` runs to the end
  of the clause's terminating token, so `nop;` includes its `;` and `here:`
  includes its `:`, while `if y > 5 then say "big"` is three clauses whose first
  ends at the `THEN` token's start byte and therefore carries a trailing blank.
  `TRACE`'s `*-*` line prints `clause_span` **for an unbroken clause**; error
  reporting and `SOURCELINE` use the retained source directly. Task 3.4 produces
  these spans, Task 3.6 splits them, Task 3.7b retains
  them and Task 3.9 checks them — a defect here surfaces four tasks downstream as
  a rework of every node type, which is why it is front-loaded.
- **A continued clause's traced text is not a contiguous byte range**, so
  `clause_span` alone cannot reproduce it. Measured: `say "x",` / newline /
  `    "y"` traces as `say "x","y"` — the comma is kept, the newline is removed,
  and the continuation line's four leading blanks are kept. So it is neither a
  slice nor a simple trim-and-join.

  Task 3.4 measured the same thing from the span side and it is worth stating
  precisely, because it tells Task 3.9 what to build: the clause's span
  **contains** the line terminator (`say 1,` / `  + 2` gives span `0..12`), while
  `trace r` drops it when joining the fragments. So Task 3.9 needs a
  terminator-stripping join over the span, not `span_bytes` on its own.
  `span_bytes` is still the right primitive; it is simply not the whole answer. This is **out of scope for this phase's gate**,
  verified: `trace_output.rex` and both Task 3.9 probes contain no continuations,
  so criterion 6 is unaffected. It is recorded because the note above would
  otherwise read as a general claim, and because whatever gates continued clauses
  later needs a representation richer than one range.
- **Clause spans do not tile the source.** The mid-line split makes two
  independent adjustments, so interstitial blanks belong to no clause: in
  `if 1 = 1   then    say "a"` the condition keeps three trailing blanks, `then`
  carries none, and the four blanks after `then` are in neither. Anything that
  assumes clause spans are a partition is wrong, which is why `split_before`
  takes an end byte and a restart token rather than one cut point.
- **`THEN`, `ELSE`, `OTHERWISE`, `END` and `WHEN` stay separate instructions**
  whichever AST shape Task 3.1 Step 3b picks, because each is its own traced
  clause and there is no other place to keep its span. That constraint is stated
  at Step 3b, not discovered at Task 3.9. Note that only the first three end a
  clause *mid-line*, which is Task 3.4's rule 4 and a different property.
