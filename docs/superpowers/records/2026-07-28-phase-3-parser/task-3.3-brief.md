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
  `Scanned { tokens: Vec<Token>, symbols: SymbolTable, keywords: Keywords }`.
  `scan` returns the table because it owns interning; it cannot borrow one it is
  still filling. `TokenKind` mirrors the C++ 19 classes in
  `interpreter/parser/Token.hpp`.

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
    /// Read-only by the time parsing starts: `scan` has already interned every
    /// symbol in the program. Tasks 3.6 and 3.7 need it to compare a clause's
    /// first symbol against the pre-interned keyword ids, and Task 3.6 needs it
    /// to recover a label's spelling when it builds `Program::labels`.
    ///
    /// Not for error substitutions: this phase does not reproduce them.
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

