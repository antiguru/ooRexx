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

