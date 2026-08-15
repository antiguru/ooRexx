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
    /// with zero leading blanks. Three spans, two gaps.
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

