# Phase 3 parser plan — fourth pass (final go/no-go)

Target: `docs/superpowers/plans/2026-07-28-phase-3-parser.md` at `08c8f355`.
Reviewed: 2026-07-28. Oracle: `build/bin/rexx` and `build/bin/rexxc` in this worktree.
Prior rounds: `.superpowers/sdd/phase-3-plan-review.md` (three passes, 1078 lines).

Every finding is tagged **VERIFIED** (I ran it, or read the C++) or **PLAUSIBLE**.
Probe files live under the session scratchpad.

---

## Verdict

**NO-GO** — one Critical, seven Important, eight Minor.

The Critical is not a re-plan. It is a missing interface requirement worth about
five sentences, and I did the investigation for it below, so the fix needs no
new probing. But it must land before Task 3.4 commits, because it changes what
`split_clauses` produces and what `Program` carries, and Task 3.9 is four tasks
downstream with a Files list that forbids it from repairing either.

The round-3 prediction was right about the shape of the plan: task order,
sizing, `ParseCtx`, the error-ground-truth recipe, the keyword analysis and the
D10 spike are all sound, and every one of the seven T-findings from round 3 is
genuinely fixed (checked individually, §6). What the final edit round did not do
is introduce a new contradiction the way the previous three did — the recurrence
this time is narrower: **exit-gate criterion 1 contradicts its own justification
paragraph three lines below it**, which is the fourth revision of that one
criterion.

The other six Importants are pre-existing and were missed by all three earlier
rounds, because all three reviewed the plan against the C++ and the running
interpreter, and never against `build/bin/rexxc` — a parse-only oracle sitting
in the tree that changes the answer to several of the plan's method questions.

---

## 1. Critical

### C1 — Nothing in the plan produces the clause spans gate criterion 6 requires

**VERIFIED**, from the oracle output the criterion is written against.

Gate criterion 6 and Task 3.9's acceptance:

> for every clause in `trace_output.rex`, the text reconstructed from the AST is
> **byte-identical** to the corresponding **`*-*`** line the interpreter prints,
> after stripping the line number, the marker and the leading indentation.

Here is what the interpreter prints for that file (`cat -A`, `$` = end of line):

```
     2 *-* x = 1 + 1$
     3 *-* y = x * 3$
     4 *-* if y > 5 $        <- trailing blank, and the clause STOPS before `then`
     4 *-*   then$           <- `then` is a clause of its own, mid-line
     4 *-*     say "big"$    <- third clause on the same source line
     5 *-* trace off$
```

Three things follow, and the plan provides for none of them.

**(a) `THEN` ends a clause, and Task 3.4 does not know that.** Task 3.4's rule is

> A clause ends at `;`, at end of line unless continued, or at `:` for a label.

`THEN` and `ELSE` are absent. In the C++ this is not clause-splitter business
either — `LanguageParser::nextClause` (`LanguageParser.cpp:1009`) splits only at
`;`/EOL/EOF, and the mid-clause break is done by the *instruction* parser:
`InstructionParser.cpp:133` and `:174` call `trimClause(); reclaimClause();` to
end the clause early and hand the remainder back as the next clause. Task 3.6
gives `parse_instruction` a `ClauseCursor` that "owns the clause list and a
position" — a cursor over a fixed list has no operation that splits the clause
it is sitting on. Verified the same mechanism drives labels: with `trace r`,
`here: nop; say "two"` traces as three clauses, `here:` / `nop;` / `say "two"`.

**(b) A clause span is not a node span; it includes the terminator.** In
`nextClause` the clause end is `location.setEnd(tokenLocation)` where
`tokenLocation` is the *end-of-clause token*. Verified in the trace: `nop;` and
`do i = 1 to 2;` are printed **with their semicolons**, and `if y > 5 ` and
`when 1 ` **with the trailing blank** that precedes `then`. An `Instruction`
node's span would carry neither. So byte-identical reconstruction needs a
clause-level span retained per instruction, not the node's own extent.

**(c) `Program` has nowhere to keep them, and `parse_interpret` throws the text
away.** `Program { source, instructions, directives, labels }` (Task 3.7b) has
no clause list, and interpreted code is traced too — verified:

```
     2 *-* interpret "say 1+1; say sourceline(1)"
     2 *-* say 1+1;          <- the fragment's own clause text, semicolon included
```

Task 3.9's stated escape hatch cannot reach any of this. Its Files list is
`Modify: src/source.rs` only, and its remedy is *"widen that node's span until
it can"* — widening the `If` node to cover `then say "big"` produces one span
where the oracle prints three lines. The remedy is orthogonal to the defect.

**Cost of leaving it.** Tasks 3.4, 3.6 and 3.7b ship and commit; Task 3.9 then
discovers that `Clause`, `ast.rs` and `Program` all need changing — which is the
"rework of every node type" the same task's own preamble exists to prevent, and
the same shape as the `::RESOURCE` scanner-mode retrofit the plan works hard to
front-load.

**Fix (no further investigation needed):**

- Task 3.4: state that a `Clause`'s byte span runs from its first token to the
  end of its terminating token, so the `;` is inside it; and that `THEN`/`ELSE`
  terminate a clause mid-line, as does the `:` of a label when tokens follow.
  Cite `LanguageParser.cpp:1009` and `InstructionParser.cpp:133`.
- Task 3.6: `ClauseCursor` needs the `trimClause`/`reclaimClause` equivalent —
  end the current clause at the cursor and re-present the remainder.
- Task 3.7b: `Program` (and `parse_interpret`'s return; see I4) must retain the
  clause spans, or each instruction must carry its own.
- Task 3.9: add `Modify: src/ast.rs, src/clause.rs` to the Files list, and say
  the expected text includes the terminating `;` and any trailing blank.

---

## 2. Important

### I1 — Exit-gate criterion 1 contradicts its own justification, three lines down

**VERIFIED** by reading it. The criterion:

> each one **tiles**: every node's span contains its children's spans, and the
> top-level nodes' spans cover the source in order, without gaps or overlaps,
> from the first non-comment byte to the last.

The paragraph immediately below it:

> Leading indentation, blank lines and the `,`/`-` continuations belong to no
> node at all.

Both cannot hold. Any corpus program with an indented line or a blank line has
gaps between consecutive top-level spans, so "without gaps … from the first
non-comment byte to the last" fails on essentially every file. This is the
fourth revision of this one criterion (round 2's R8, round 3's T6, now this),
and it is the named failure mode again — the correction landed in the prose and
the assertion above it still says the opposite.

**Should say:** every node's span contains its children's; top-level spans are
in source order and do not overlap; and the only bytes between consecutive
top-level spans are whitespace, comments and continuations. That is checkable,
still catches a dropped clause (which leaves a *non-whitespace* gap), and does
not require covering bytes the plan itself says belong to no node.

Second, smaller point in the same criterion: "every node's span contains its
children's spans" presumes a tree. Under the flat instruction chain that Task
3.1 Step 3b may choose, instructions are siblings, not children. Say which
half of the property applies to expressions (nesting) and which to instructions
(ordering), so the criterion survives either Step 3b outcome.

### I2 — The significant-blank rule is never stated, and four statements that depend on it are wrong

**VERIFIED** against `Scanner.cpp:753–771` and `Token.hpp:595–596`.

The C++ emits `TOKEN_BLANK` only when **both** hold: the previous token is a
symbol, a literal, `)` or `]` (`isBlankSignificant`), **and** the next non-blank
character begins a symbol, a quoted literal, `(` or `[`. Otherwise the blank is
discarded. That single rule is what makes `f (x)` a concatenation and `f(x)` a
call — the hazard the parent plan calls D10's deciding case, and which Task 3.1
is built around. It appears nowhere in the plan, and the four places that lean
on it are wrong:

- **Task 3.3 Step 2**, first assertion of `double_dash_starts_a_line_comment…`:
  ```rust
  assert_eq!(kinds(&scan_ok("a -- b")), [TokenKind::Symbol, TokenKind::Blank, TokenKind::Eoc]);
  ```
  There is no `Blank`. The look-ahead after `a `'s blank finds `-`, which is not
  a symbol start, a quote, `(` or `[`, so the blank is dropped; then
  `locateToken` sees `--`, calls `truncateLine()` and returns `CLAUSE_EOL`
  (`Scanner.cpp:313–317`). The kinds are `[Symbol, Eoc]`.
- **Task 3.3 Step 2**, second assertion: `assert_eq!(kinds(&scan_ok("a - b")).len(), 5)`.
  Under the same rule it is 4 — `[Symbol, Operator, Symbol, Eoc]`. It can only
  be 5 if `scan` emits an EOL `Eoc` *and* a separate EOF `Eoc`, and the
  `block_comments_nest` test three lines above assumes it does not (it expects
  exactly one `Eoc`). The two assertions cannot both be right under any single
  blank/Eoc model.
- **Exit gate 1**: "the `,`/`-` continuations belong to no node at all" is
  false. A continuation *becomes* a significant blank (`Scanner.cpp:342–348`),
  and the blank is the abuttal operator, so it sits inside a concatenation
  node's span. **VERIFIED**: `say "a"-` / `"b"` prints `a b`, not `ab`, with no
  spaces anywhere near the continuation. A scanner that merely erases the
  continuation produces a silently-wrong program — precisely the class of bug
  this plan is organised around.
- **Task 3.3 Step 1 vs Task 3.4 Step 1**: Step 1 lists only the comma
  continuation; Task 3.4 claims both continuations as clause-splitting business.
  Both are handled in `locateToken`, i.e. in Task 3.3's scanner, before any
  clause exists.

State the rule once in Task 3.3 with the citation, fix the two assertions, and
delete the continuation parenthetical from the gate.

### I3 — Task 3.7 Step 1's directive arithmetic does not add up, and it is told to assert it

**VERIFIED.** The step:

> 45 constants come back. Nine are the top-level directives listed above; the
> other 36 are option sub-keywords. Split them by reading which table each
> appears in (`subDirectives` has 40 entries), and assert both counts in a test
> so a mis-extraction fails loudly.

The 45 is an artefact of an unanchored regex: `DIRECTIVE_[A-Z_]+` also matches
inside `SUBDIRECTIVE_*`. Measured:

| set | count |
|---|---|
| distinct strings the plan's grep returns | 45 |
| `DirectiveKeyword` enum members (`Token.hpp:333–346`) | 11 — the 9 table entries plus `DIRECTIVE_NONE` and `DIRECTIVE_LIBRARY` |
| distinct `SUBDIRECTIVE_*` names | 41 (40 in the table, plus `SUBDIRECTIVE_NONE`) |
| names appearing in *both* tables | 7 — `ATTRIBUTE CLASS CONSTANT METHOD ROUTINE LIBRARY NONE` |

11 + 41 − 7 = 45. So "the other 36 are option sub-keywords" is wrong: the
sub-keyword table has 40 entries, `DIRECTIVE_LIBRARY` is an enum member with no
table row at all, and `NONE` is a sentinel. Worse, the prescribed method —
"split them by reading which table each appears in" — cannot produce a
partition, because five real option names (`ATTRIBUTE`, `CLASS`, `CONSTANT`,
`METHOD`, `ROUTINE`) are in both tables, which is itself worth telling an
implementer, since `::CLASS … SUBCLASS` and `::METHOD` share spellings across
the two levels. A test asserting 9 and 36 enshrines a false split and yields a
sub-keyword table four entries short.

The same conflation is in the task's prose ("The other 36 `DIRECTIVE_*`
constants are their option sub-keywords … 40 of them per the `subDirectives`
table"). Round 2 flagged that sentence; it was never fixed.

**Should say:** 9 top-level directives (`directives[]`), 40 option sub-keywords
(`subDirectives[]`), 7 spellings in both, and assert 9 and 40 with an anchored
extraction rather than 9 and 36 from a substring match.

### I4 — `parse_interpret` drops the source its own spans index

**VERIFIED** as an interface defect, and as a behaviour requirement.

Task 3.7b: `parse_interpret(text: String) -> Result<Vec<Instruction>, ParseError>`.

The architecture is "the program source is retained as one `String`; every AST
node holds a byte range into it". `parse_interpret` takes ownership of the text
and returns only instructions, so every span in the result indexes a `String`
that no longer exists. `parse_program` gets this right by returning
`Program { source, … }`; `parse_interpret` needs the same.

It is not hypothetical: interpreted clauses are traced with the *fragment's*
text, while `SOURCELINE` inside the fragment still resolves against the parent
program. **VERIFIED**:

```
     2 *-* interpret "say 1+1; say sourceline(1)"
     2 *-* say 1+1;
     2 *-* say sourceline(1)
       >>>   "trace r"        <- parent program's line 1
```

So Phase 4 needs both: the fragment's retained text for `*-*`, and the enclosing
program's for `SOURCELINE`. Return type should carry the fragment source.

### I5 — Two of the seven error numbers in Task 3.6 Step 4 are not parse errors

**VERIFIED**, and this is a distinction the plan makes nowhere.

> All seven measured against `build/bin/rexx`: `then` 8, `else` 8, `when` 9,
> `otherwise` 9, `end` 10, `procedure` 17, `parse` 20. A loop that parses each
> keyword by itself therefore cannot pass.

The numbers are all correct (re-measured, including the sub-numbers: 8.1, 8.2,
9.1, 9.2, 10.1, 17.1, 20.903). But **17 is a runtime error, not a parse error**:

```
$ cat pre_procedure.rex        $ build/bin/rexxc pre_procedure.rex
say "BEFORE-RAN"               rc=0, no error   <- it PARSES
procedure
$ build/bin/rexx pre_procedure.rex
BEFORE-RAN                     <- executed first
Error 17 … Unexpected PROCEDURE.
```

`Error_Unexpected_procedure_call` is raised in
`execution/RexxActivation.cpp:1250`, not in the parser. Same for `LEAVE`/
`ITERATE` (error 28, `RexxActivation.cpp:1161`/`:1214`) — bare `leave` also
passes `rexxc` cleanly. So a Phase 3 parser must **accept** bare `procedure`,
`leave` and `iterate`; a parser that rejects them at parse time diverges from
the oracle on every program with an unreachable `leave`.

Consequences: cite 17 as evidence about parsing and an implementer may build the
check in the wrong phase; and gate criterion 4's "for every syntax error the
parser raises … match `build/bin/rexx`" needs scoping to parse-time errors,
otherwise the comparison set is ill-defined. `rexxc` is exactly the instrument
that draws that line (see I6).

### I6 — `build/bin/rexxc` is a parse-only oracle, and the plan never mentions it

**VERIFIED.** `rexxc inputfile` with no output file "only performs a syntax
check" — it parses without executing. Behaviour, measured:

| input | `rexxc` | `rexx` |
|---|---|---|
| `x = )` | rc 219, `Error 37 … line 2`, `Error 37.2` | same, after aborting the whole file |
| bare `then` | rc 248, `Error 8`/`8.1` | same |
| bare `procedure`, bare `leave` | **rc 0** | error 17 / 28 at runtime |
| `x = 1/0` | **rc 0** | error 42 at runtime |
| `address system` + `"echo hi"` | rc 0, **nothing executed** | runs `echo` |

This bears on four places the plan reasons about method:

- **Task 3.3 Step 5** currently says the differential method is "a program that
  scans correctly runs to completion under both". The claim it rests on — that
  no introspection exposes a token stream — is true, but *running* is a weaker
  and side-effecting proxy for *parsing*, and `rexxc` gives the parse verdict
  directly on inputs that cannot safely be run.
- **Task 3.8 Step 1**: route (a) "run the bad file, capture stderr" works
  because ooRexx parses the whole file first — but only for files that fail.
  `rexxc` additionally gives the negative direction (this file *does* parse)
  without executing it.
- **Gate criterion 4** gets a clean definition of "syntax error" (I5).
- **Gate coverage**: it makes the parent plan's `samples/` criterion trivial —
  see I7.

Not a defect in what the plan asserts; a defect in what it leaves on the table,
in the one phase whose entire method is differential comparison.

### I7 — The exit gate silently drops two of the parent plan's Phase 3 criteria

**VERIFIED.** `docs/superpowers/plans/2026-07-27-rust-rewrite.md:402`, the row
that *is* the Phase 3 gate:

> Round-trips every `.rex` in `samples/` to an AST; `SOURCELINE`, error
> line/column reporting, and `TRACE` output formatting match the oracle
> byte-for-byte; parse throughput on `CoreClasses.orx` recorded

Against that, the Phase 3 plan's gate:

- **`samples/` is gone.** The plan gates on the 14 corpus programs plus the two
  bootstrap files — 5,203 lines. `samples/` is **301 files, 67,519 lines**, and
  **all 301 pass `build/bin/rexxc`** (measured), so the oracle side costs one
  shell loop. It is the broadest coverage available and by far the best
  instrument for gate criterion 2 (every `Instruction`/`Expr` variant
  constructed), which 14 hand-written programs will struggle to satisfy
  honestly. Dropping it may be the right call, but a phase plan that narrows its
  parent's gate must say so.
- **The parent still says "line/column".** The Phase 3 plan correctly removed
  column everywhere (C2, three rounds ago) but the governing document was never
  corrected — here and at line 2360 ("For Phase 3 that is error messages with
  line and column"). `phase-2-gate.md` says the fix for an unsatisfiable
  criterion "should be fixed in the plan text before Phase 3 starts so the same
  trap is not set again". The trap is still set, in the document a gate assessor
  will read.

Fix: correct the parent plan's row 402 and line 2360, and either adopt the
`samples/` criterion (recommended — it is cheap and strong) or record the
deviation in the Phase 3 plan explicitly.

---

## 3. Minor

Each is a line or two, and an implementer can absorb them in passing.

**M1 — Task 3.3 Step 4 still asserts a column exists.** *"Rexx source is
byte-oriented and the column numbers in error messages count bytes."* There are
no column numbers in error messages; the plan says so at length in Task 3.1,
Task 3.8 Step 4 and gate criterion 4. The byte-orientation advice is right for
other reasons (`'…'x` literals, DBCS); the justification is the one claim three
rounds removed. Reword rather than delete.

**M2 — `TokenCursor` is used and never defined.** `parse_expr(&ParseCtx, &mut
TokenCursor)` (Task 3.5) is the only mention in the plan. Nothing says which
task produces it or how Task 3.6's `ClauseCursor` yields one over a clause's
token range. Name it in Task 3.3 beside `ParseCtx`, where the token vector lives.

**M3 — Task 3.9's Files list contradicts its own steps.** `Modify: src/source.rs`
only, while Step 3 says "adjusting node spans" and Step 2 reconstructs from the
AST. Supports C1; fix with it.

**M4 — Gate criterion 5 has no oracle-side method.** `SOURCELINE(n)` for every
line of every corpus program requires the interpreter's answer for files you may
not edit. `.Package~new("f.rex")~source` returns the line array — **VERIFIED**,
27 items for `do_variants.rex` (= `wc -l`), no terminators — though it executes
the prolog. Name the route; otherwise the obvious approach (append a driver to
each corpus program) changes the files under test.

**M5 — Task 3.9's "239 expected trace-output lines" does not reproduce.**
`ootest/ooRexx/base/keyword/TRACE.testGroup` (1,338 lines) has **135** lines
containing `*-*` and **243** containing a value marker. 239 matches neither.
It is a motivating figure, not a test input, so the argument survives — but the
plan's other counts are all exact and this one should be too.

**M6 — the value-marker enumeration is incomplete.** Task 3.9 and gate criterion
6 exclude "`>L>`, `>O>`, `>V>`, `>>>`, `>=>`". The interpreter also emits
`>A>`, `>I>`, `>C>`, `>M>`, `>P>` and `>K>` — `>K>` 33 times in that testGroup
and once in my own two-line probe (`>K> "TO" => "2"`). All carry values, so all
are equally out of scope. Phrase the exclusion as "every marker except `*-*`",
which cannot go stale.

**M7 — "Phase 3 owns the *formatting* of traced source" no longer fits.** After
`08c8f355` the Interfaces line says "the source-text slice per clause … Nothing
else" and the acceptance strips the marker and the indentation. Formatting is
now Phase 4's. Delete the sentence.

**M8 — criterion 2 makes criterion 1's "All 14" stale.** Criterion 2 says to add
programs where a variant is unreachable; criterion 1 counts 14. Say "all corpus
programs".

**M9 — Task 3.1 Step 5 stages a file it cannot have changed.** `git add
docs/…/d10-decision.md rust/Cargo.toml` — `rust/Cargo.toml` has
`members = ["crates/*"]`, so creating and deleting the spike crate never
touches it.

---

## 4. Exit gate, criterion by criterion

"Satisfiable by this phase" = reachable with a parser plus the oracle binaries,
no executor. "As written" = reachable by an implementer following the plan's own
task interfaces.

| # | criterion | by this phase | as written | why |
|---|---|---|---|---|
| 1 | 14 corpus programs parse **and** tile | **Yes** | **No** | Parsing: all 14 pass `rexxc` (verified). Tiling as worded demands top-level spans cover the source "without gaps … from the first non-comment byte to the last", which its own next paragraph says is impossible (indentation, blank lines, continuations belong to no node). Restate as containment + ordering + whitespace-only interstices — I1. |
| 2 | every `Instruction`/`Expr` variant constructed, by enumeration | **Yes** | **Yes** | Parser-only, and the criterion permits adding programs. Would be far stronger over `samples/` (I7). |
| 3 | `CoreClasses.orx` + `StreamClasses.orx` parse end to end | **Yes** | **Yes** | 5,203 lines; oracle parses both at every start, and `rexxc` confirms either alone. |
| 4 | number, sub-number, line, substitution values match, by running a bad file | **Yes** | **Yes, with a scoping gap** | All four observable: stderr gives `Error N … line L` + `Error N.S`, and `condition('o')~additional` gives the substitutions via the INTERPRET route. Gap: "every syntax error the parser raises" is undefined while errors 17 and 28 are runtime, not parse (I5); `rexxc` draws the line (I6). |
| 5 | `SOURCELINE(n)` for every line of every corpus program, incl. last line and no trailing newline | **Yes** | **Yes, method unstated** | Verified: no trailing newline still counts its last line; CRLF strips the CR; `sourceline(0)`→40.14, past-end→40.34. Oracle side via `.Package~new(f)~source` — M4. |
| 6 | `TRACE`'s `*-*` lines reconstructible for `trace_output.rex` | **Yes** | **No** | A parser can do it — the data is all static. But it needs clause spans including the terminating `;` and the trailing blank, and a separate span for the mid-line `then` clause. Task 3.4 produces none of that, `Program` cannot hold it, and Task 3.9 may only modify `source.rs` — C1. |
| 7 | throughput on 5,203 lines recorded against ~55 ms, with a plain fit statement | **Yes** | **Yes** | Asks for a number and an honest statement. Bench settings match `perf-baseline.md` exactly (verified: `sample_size(10)`, 500 ms warmup, 30 s). |
| 8 | clippy clean, zero `unsafe` | **Yes** | **Yes** | `unsafe_code = "forbid"` at `[workspace.lints.rust]` (verified). Note `phase-2-gate.md`'s standing qualification: no CI runs the Rust tree, so this stays a local claim. |
| 9 | Phase 2's twelve differential sets still at 0 — 128,368 cases | **Yes** | **Yes** | 12 entries in `gen-curated-sets.py`'s `SETS` (verified). |

**Tally: 9 of 9 satisfiable by this phase; 7 of 9 as written.** Both failures are
wording-and-interface, not reachability — which is a real improvement on Phase
2, where three of five needed an interpreter that did not exist. Criterion 6 is
the one that costs something if dispatched unfixed.

Separately: the parent plan's Phase 3 gate row contains two criteria this gate
does not (I7).

---

## 5. Placeholders, hand-waving, cross-task naming

- **No "TBD", no "add error handling", no "similar to Task N"** anywhere. Checked.
- **Every step carries real code or a real command** except Task 3.4 Step 3
  ("Run, fail, implement, pass") and Task 3.7 Step 4 ("Implement the remaining
  eight, committing per directive"). Both are acceptable: 3.4's rules and tests
  are concrete above it, and 3.7's eight directives each have a named table and
  a per-directive commit.
- **Signatures are consistent across tasks** — `ParseCtx` now threads through
  all three `parse_*` (T4 fixed), `ParseError` has one field list in both places
  that state it (T5 fixed), `ProgramSource::position` is produced in 3.2 and
  consumed in 3.8, `Clause` in 3.4 is consumed in 3.5–3.7. The two exceptions
  are `TokenCursor` (M2) and `parse_interpret`'s return type (I4).
- **Every task has an Interfaces block.** Task 3.10's names only what it
  consumes, which is right for a benchmark.
- **Task 3.1's timebox is a number** (four hours each, hard) — the round-1
  placeholder finding is fixed.

---

## 6. Round-3 findings: verified fixed

Checked individually against the current text rather than against the commit
messages, because each of the previous three rounds found the fix round had
broken something else.

- **T1** (`>>>`/`>V>` in Task 3.9's acceptance) — now `*-*`, with the value
  markers shown as Phase 4's in a worked example, and the gate criterion
  matching. Fixed, and `08c8f355` then went further and scoped the whole task
  to `*-*`, which is the right call: I re-measured the indentation claim it
  rests on, and it holds — `call sub` traces at depth 0 while the routine's own
  clauses trace one level deeper, and a `do` body one level deeper than its
  header, so static nesting plus call depth accounts for it. No per-node depth
  needed.
- **T2** (bare-keyword numbers) — all seven re-measured with sub-numbers, all
  correct: `then` 8.1, `else` 8.2, `when` 9.1, `otherwise` 9.2, `end` 10.1,
  `procedure` 17.1, `parse` 20.903. See I5 for the category error that survives.
- **T3** ("almost entirely directives" in Step 6) — gone; Step 6 now states
  8.3% and gives the real reason. Scoped scan of Task 3.7: no surviving copy.
- **T4** (`ParseCtx` in every `parse_*`) — all three signatures updated.
- **T5** (`ParseError` field list) — one shape, stated twice identically, with
  "no field added or removed" as the instruction.
- **T6** (round-trip → tiling) — restated as tiling, but the restatement is
  self-contradictory. I1.
- **T7** (duplicate Step 5 in Task 3.7) — renumbered 1–7. Verified by scoping
  the scan to the task, per the note in `9fd4334f`.

## 7. Verified clean

Claims I tried to break and could not. All measured in this session.

**Counts, all exact:** `directives[]` 9, `keywordInstructions[]` 35,
`subKeywords[]` 50, `conditionKeywords[]` 12, `parseOptions[]` 10,
`subDirectives[]` 40, 19 token classes (`Token.hpp:79–97`), 52 instruction
classes, 17 expression classes, LOC 4650/4398/2867/1955/768 = **14,638** with
the 17,483 directory total on its own labelled row, CoreClasses 4,193 +
StreamClasses 1,010 = **5,203**, 347 `::` lines = **8.3%**, C++ cold start
median **5.119 ms** (`perf-baseline.md:101`), 12 differential sets, chumsky
0.13.0 and 0.9.3 both in the offline registry cache, criterion 0.8.2 with
matching bench settings.

**The keyword families are exactly the table.** The plan's 11/8/11/4/1 partition
diffed as a set against `grep -oE '"[A-Z]+", *KEYWORD_[A-Z_]+'` — identical, no
invention, no omission. `ASSIGN` does not appear; `LOOP` does.

**The sorted-table claim is right, in both directions.** `keywordInstructions[]`
is alphabetical with explicit codes and `resolveKeyword` binary-searches it —
at **`KeywordConstants.cpp:417`** exactly as cited. The file's own header says
"It is critical for all the following tables to be in ASCII alphabetic order".
And the contrast the plan draws holds: Phase 0's table comes from
`LanguageParser::builtinTable[]` in `BuiltinFunctions.cpp`, a table of function
pointers whose *index* is the builtin code — `rexx-inventory/build.rs` extracts
it with the comment "Order is table order … do not sort it". Two different
tables with two different disciplines, and the plan's "check which kind you are
looking at" is exactly the right instruction. (`builtinFunctions[]`, the
*name* table, is alphabetical and binary-searched like the others.)

**Keywords are not reserved**, and `rust/corpus/lang/keyword_as_variable.rex`
passes `rexxc` and runs rc 0.

**`-2 ** 2` is 4**, and it is precedence rather than literal scanning:
`x = 2; say -x ** 2` → 4, `say -(2**2)` → -4, `say (-2)**2` → 4. Bonus, not in
the plan and worth knowing at Task 3.5: `say 2 ** -1 ** 2` → **0.25**, so `**`
is left-associative.

**`say a b + 1` → `1 3`**, matching the asserted `Concat(a, Add(b, 1))`.

**Comparison is left-associative:** `a=2; b=2; c=1; say a = b = c` → 1
(right-assoc gives 0). At `a=b=c=1` both give 1, as the plan's comment says.

**Comments separate without producing a blank:** `say a/*c*/b` → `12`,
`say a b` → `1 2`. Also `say a/*c*/ b` → `1 2` and `say a /*c*/b` → `1 2`,
consistent with the significant-blank rule.

**`say a""b` prints `x`**, not `xy` — empty binary literal, correctly labelled.

**`f(x)` vs `f (x)`:** `ROUTINE-CALLED-WITH-1` vs `VAR 1`. The hazard is real
and `whitespace_significant.rex` pins it.

**Error ground truth:** `signal on syntax` cannot catch its own file — rc 219,
handler never runs. `interpret "x = )"` inside an installed trap fires and gives
`code` 37.2 with `position` = the INTERPRET line (3). Both plan claims exact.

**An instruction after a directive joins that directive's body:** prints only
`main`, rc 0. Not a syntax error.

**`Command` is the never-fail fallback:** `address system` then
`"echo hello-from-command"` runs it and sets `rc` 0. A clause *starting* with
`~` is error 35.1, and the plan's table correctly says only the standalone
`q~append(1)` form is a message instruction.

**`::RESOURCE`'s terminator is `::END`, case-sensitively** — lowercase `::end`
gives error 99.943, and a `::END` with leading text on the line does not
terminate it. Task 3.7 Step 3 tells the implementer to verify exactly this
before implementing, which is the right instruction.

**All 14 corpus programs and all 301 `samples/*.rex` pass `rexxc`.**

---

## 8. Fix order

1. **C1** — clause spans, the mid-clause `THEN`/`ELSE`/label break, `Program`'s
   field, Task 3.9's Files list. This is the one that must precede Task 3.4.
2. **I1** — restate gate criterion 1's tiling in one sentence.
3. **I2** — state the significant-blank rule with its citation; fix the two
   scanner assertions and the continuation parenthetical.
4. **I4**, **I3**, **I5** — `parse_interpret`'s return type; the directive
   arithmetic and the count assertions; the parse-time/runtime split.
5. **I6**, **I7** — adopt `rexxc` as the parse oracle in Tasks 3.3/3.8 and gate
   criterion 4; then decide `samples/` and correct the parent plan's row 402 and
   line 2360.
6. **M1–M9** as edits in place.

Nothing above needs new investigation. Every number, marker and signature the
fixes need is in this file.
