# Phase 3 plan — round 5 fixes

Applied `.superpowers/sdd/phase-3-plan-review-round4.md` to
`docs/superpowers/plans/2026-07-28-phase-3-parser.md` and
`docs/superpowers/plans/2026-07-27-rust-rewrite.md`.

Commits: `ca4a7525` (parent plan), `26aefbce` (Phase 3 plan), `a2924ee3`
(naming follow-up: `Instruction::clause_span` named in all three tasks that
touch it).

**16 of 16 findings fixed. None deferred.**

## Critical

**C1 — clause spans.**
Fixed across all four tasks the review named, plus two the review did not.
Task 3.4: `Clause` gains a `span` field documented as running from the first
token to the end of the terminating token, so `;` is inside it and an
end-of-line terminator stops at the last content byte; four numbered rules
replace the old one-sentence rule, citing `LanguageParser.cpp:1009`, `:1072`,
`:1378`/`:1403`/`:1465`/`:1494`, `InstructionParser.cpp:173–174`,
`Clause.cpp:138` and `IfInstruction.cpp:58–66`; two new tests assert the `;` and
the label `:` are inside the span.
Task 3.6: `ClauseCursor` is given a full definition with `split_before(&ParseCtx,
at) -> Clause`, which ends the current clause at the **start byte** of token `at`
and re-presents the remainder, reproducing `trimClause`/`reclaimClause`; the
Files list gains `Modify: src/clause.rs`.
Task 3.7b: `Program` retains the spans because every `Instruction` carries the
span of the clause it came from; stated explicitly, with the note that this holds
under either Step 3b AST shape.
Task 3.9: Files list gains `src/clause.rs` and `src/ast.rs`; acceptance says the
expected text includes the terminating `;` and the trailing blank; Step 3 now
says fix the clause span rather than widen the enclosing node.
Beyond the review: the Architecture paragraph at the top now names the clause
span as a range the other consumers do not need, and Task 3.9's "costs nothing
beyond the spans `SOURCELINE` and error reporting already need" is corrected —
that sentence was the reason Task 3.4 had no rule producing them.

## Important

**I1 — gate criterion 1 contradicted its own justification.**
Restated as two separable properties: expression spans contain their operands'
spans; instruction spans are ordered and non-overlapping with only
whitespace-class bytes between them. The justification paragraph now says
interstices are *permitted* and constrained to whitespace, comments and
continuations, instead of saying coverage must be gap-free. The
expression/instruction split is stated as deliberate so the criterion survives
either Task 3.1 Step 3b outcome. The continuation parenthetical is gone, replaced
by the correct statement that a continuation *is* inside a node's span.

**I2 — the significant-blank rule.**
Stated once in Task 3.3 as a two-sided rule with `Scanner.cpp:726`,
`Scanner.cpp:755–771` and `Token.hpp:595–596`. Both broken assertions fixed:
`a -- b` is `[Symbol, Eoc]` with no `Blank`, and `a - b` is
`[Symbol, Operator, Symbol, Eoc]`. The `Eoc` model is now stated explicitly
(one per terminator, never two in a row, none for an empty final clause) so the
two assertions cannot disagree again. Two tests added: `f (x)` versus `f(x)`, and
the continuation-becomes-a-blank case. Task 3.3 Step 1 lists both continuation
characters and says they are the scanner's business; Task 3.4 Step 1 says it has
no continuation rule of its own.

**I3 — directive arithmetic.**
Task 3.7's prose now says 9 `directives[]` rows and 40 `subDirectives[]` rows,
citing `KeywordConstants.cpp:52–63` and `:363–405`, and states there is no set of
36. Step 1 explains the 45 as an unanchored-grep artefact (11 enum members + 41
`SUBDIRECTIVE_*` names − 7 shared suffixes), gives two anchored `sed`/`grep`
extractions verified to return 9 and 40, and says not to assert 9 and 36. The
five spellings in both tables are named, with `::CLASS … SUBCLASS` and
`::METHOD … CLASS` as the reason it matters.

**I4 — `parse_interpret` dropped its source.**
Return type is now `Fragment { source: ProgramSource, instructions:
Vec<Instruction> }`, with the `trace r` + `interpret` probe reproduced showing
the fragment's own clause text alongside the parent program's `SOURCELINE`.

**I5 — two of the seven error numbers are not parse errors.**
Task 3.6 Step 4 now carries a nine-row measured table separating `rexxc` from
`rexx`: `then`/`else`/`when`/`otherwise`/`end`/`parse` are parse errors;
`procedure` (17.1), `leave` (28.1) and `iterate` (28.2) are runtime, cited to
`RexxActivation.cpp:1250`, `:1214` and `:1161`. The parser must accept all three,
and `KEYWORD_CLAUSES` gains those three rows as bare clauses. Gate criterion 4 is
scoped to parse-time errors with `rexxc` as the definition.

**I6 — `rexxc` as a parse-only oracle.**
Adopted in five places. Task 3.1 axis 2 compares against `rexxc`. Task 3.3
Step 5's three-part method now uses `rexxc` for the first two parts and `rexx`
only for the meaning-changing cases, with the stream-routing recipes
(`>/dev/null 2>&1` for the verdict, `2>&1 1>/dev/null` for the text). Task 3.7
Step 3 verifies the `::END` terminator with `rexxc` (99.943, rc 157, measured).
Task 3.8 Step 1 has `rexxc` as route (a) and explains the negative direction.
Gate criterion 4 and a new "Notes carried in" entry.

**I7 — dropped parent criteria.**
`samples/` adopted as a gate criterion: every `.rex` under `samples/`, 301 files,
67,519 lines, with the oracle loop written out. The loop uses `find`, not
`samples/*.rex` — that glob matches only the 36 top-level files, which I caught
while verifying. Criterion 2 now says to run the variant enumeration over the
corpus and `samples/` together and names `samples/` as the primary instrument.
Parent plan: line 402's "line/column" → "line", and the row records 301 files;
line 2360 likewise, with an explicit "do not gate any phase on a column";
line 376 (D10's own decision block) had the same false claim and is fixed too;
line 2402's rejected-alternative now distinguishes executing the samples from
parsing them.

## Minor

**M1** — Task 3.3 Step 4's byte-orientation justification rewritten around
`'…'x`/`'…'b` literals, source round-tripping and byte-indexed spans, with an
explicit "there are no column numbers in error messages".

**M2** — `TokenCursor` defined in Task 3.3 beside `ParseCtx`, with a full
implementation scoped to one clause's token range. Task 3.5's Interfaces block
names it and says how to build it. `advance` rather than `next`, because
`clippy::should_implement_trait` would fail gate criterion 8.

**M3** — Task 3.9's Files list gains `src/clause.rs` and `src/ast.rs`, and its
Interfaces block names what it consumes.

**M4** — Gate criterion 5 names the oracle route: a separate driver using
`.Package~new(f)~source`, with the prolog-executes caveat and a prefix filter.
Driver code included and verified.

**M5** — 239 replaced with the measured figures: `TRACE.testGroup` is 1,338 lines
with 135 carrying `*-*` and 243 carrying a value marker.

**M6** — Phrased as "every marker except `*-*`" in Task 3.9 and gate
criterion 6, with the fifteen distinct value markers enumerated once as evidence
that a list goes stale. Ten of the fifteen were missing from the old list; the
review named six of those ten, and `>F>`, `>E>`, `>R>` and `>N>` I found while
verifying.

**M7** — "Phase 3 owns the *formatting* of traced source" deleted.

**M8** — Criterion 1 now says "Every program in `rust/corpus/lang/`". Task 3.7b
Step 4 likewise, with a note to count the directory.

**M9** — Task 3.1 Step 5 stages only `d10-decision.md`, with the reason
(`members = ["crates/*"]`) stated.

## Rulings applied

**Task 3.1's timebox** rewritten with no wall clock: equal effort, an arm stops
on first success or first visible stall (same construct attacked three times
without progress), the second arm gets the same number of attempts, and which
condition ended each arm is recorded. "The combinator arm never reached a working
expression grammar" is preserved as a legitimate and decisive result.

**Task 3.1 axis 4** added: chumsky 0.13.0 costs 28 transitive packages against
zero, and `default = ["std", "stacker"]` → `stacker` → `psm`, whose
`[build-dependencies]` include `cc`, so choosing chumsky puts a C compiler on the
build path across five CI platforms including OpenBSD. Verified the feature and
dependency chain in the offline registry.

## Deviations — both raised and both approved by the repo owner

Do not re-open either of these in a later review round. Both were flagged when the
fixes landed and both were accepted.

**Markdown style.** The standing rules ask for one sentence per line, `*` bullets
and no em-dashes. This 1,000-line document uses hard-wrapped prose, `-` bullets
throughout (mandatory for the `- [ ]` step syntax the SDD tooling reads) and
em-dashes pervasively. The file's own conventions were matched rather than putting
a second style inside one document. **Approved: no conversion wanted.**

**Rule 3 moved down a layer.** Label splitting is done by `split_clauses` in Task
3.4, not by the instruction parser as in the C++, because a symbol-or-literal
followed by `:` at the start of a clause is recognisable from the token stream
alone and `Clause::label` already existed. Rule 4 (`THEN`/`ELSE`/`OTHERWISE`)
cannot move the same way, because keywords are not reserved and only the
instruction parser can tell a `THEN` from a variable named `then`. Flagged in the
plan itself as a deliberate deviation. **Approved.**
