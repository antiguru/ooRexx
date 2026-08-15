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

Because those are two independent adjustments, **some bytes end up in no clause
at all**: in `if 1 = 1   then    say "a"` the condition keeps its three trailing
blanks, `then` carries none on either side, and `say "a"` starts at `say`, so the
four blanks after `then` belong to neither neighbour. Task 3.6's `split_before`
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

