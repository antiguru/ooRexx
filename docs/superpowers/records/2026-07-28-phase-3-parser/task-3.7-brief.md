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
string lookups. `RexxToken::directives[]` and `subDirectives[]` are named in this
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

