# Phase 3 final review — crate public surface, gate harnesses, gate documents

Reviewer slice: `rust/crates/rexx-parse/src/lib.rs` and public surface, `benches/parse.rs`,
the four gate harnesses (`tests/samples.rs`, `tests/tiling.rs`, `tests/variants.rs`,
`tests/sourceline_oracle.rs`, `tests/gate_walk/`), `docs/superpowers/plans/phase-3-gate.md`,
`docs/superpowers/plans/d10-decision.md`.

All work was read-only. Every quantitative claim below that I flag as verified was
independently reproduced by running `cargo test`/`cargo bench --test`/`grep`/`file`/`wc`
against the tree exactly as it stands, not taken on trust from the documents.

## Bottom line

No critical defects. Two Important findings, both about the honesty/precision of
`docs/superpowers/plans/phase-3-gate.md`'s criterion-1 discussion and one small
factual slip in criterion 5's count. The public surface and the four gate harnesses
are sound: every harness can genuinely fail, none has a silent-no-op path, the
"exhaustive match, no wildcard" claim for the nine enums in `tests/variants.rs`
checks out by direct inspection, and Phase 4 has everything it needs (clause spans,
labels, the source, directive bodies, symbol table) through `pub` fields.

---

## Important

### 1. The recovered falsifiable evidence for "expressions nest" covers two of nine recursive `ExprKind` variants, and the gate document's framing sentence reads more broadly than that

`phase-3-gate.md` section 1 says, introducing the substitute for the vacuous
containment check:

> **What was added in its place**: a falsifiable property that catches what nesting
> was reaching for.

That sentence, read on its own, sounds like it recovers real evidence for
"expressions nest" in general. The next two sentences correctly narrow the scope to
`Binary` (operand tightness and ordering) and `Prefix` (operator precedes operand),
and the document is careful nowhere else to overclaim. But `ExprKind` has nine
variants that can hold child expressions: `Prefix`, `Binary`, `Call`, `QualifiedCall`,
`Message`, `List`, `Logical`, `VariableReference` (`ClassResolver` holds only
`SymbolId`s, no `Expr` children). I confirmed by reading `rust/crates/rexx-parse/src/ast.rs`
`ExprKind::for_each_child` and `tests/tiling.rs`'s `binary_tightness_errors`/
`prefix_order_errors` that only `Binary` and `Prefix` get a falsifiable tightness
check. `Call.args`, `QualifiedCall.args`, `Message.target`/`.super_class`/`.args`,
`List`, `Logical`, and `VariableReference.inner` are exercised over the whole corpus
by `containment_errors` alone, which is the *vacuous* check (`Expr::new` widens the
parent span to cover every child, so no input — however mis-parsed — can violate
containment for any of these forms either).

So: a bug that attached the wrong argument to a `Call`, mis-slotted a `Message`'s
`super_class`, or misplaced an item in a `List`/`Logical` would sail through this
gate's corpus-wide checks undetected, the same way a `Binary` mis-nesting would have
sailed through before the substitute was added. This is not a criticism of whether
criterion 1 is "MET" — the criterion's literal wording is only "expressions nest",
which containment satisfies trivially, and the document is not obligated to go
further than it did. But a reader taking the summary sentence at face value could
credit the fix with more generality than it has. I'd tighten the summary sentence to
name the two variants explicitly, matching the precision the rest of the paragraph
already has, or add one sentence noting the other seven multi-child variants remain
on the vacuous check alone.

This is squarely the question the task asked me to judge ("whether the document is
honest about the distinction rather than blurring them"). My answer: the document
does not blur *vacuous vs. falsifiable* — that distinction is stated plainly and
correctly. It does slightly oversell the *scope* of the falsifiable substitute in
its one-sentence summary, correctable in one place.

### 2. Section 5's "1,021 corpus rows" is off by one; the actual count is 1,020

`phase-3-gate.md` section 5: "**Measured for this document: 1,021 corpus rows**,
against 385 when the criterion was drafted and 492 at Task 3.8."

I counted independently two ways and both give 1,020, not 1,021:

* `rust/corpus/errors/parse-errors.tsv` has 1,082 total lines, 61 of which are
  `#`-comment lines and exactly one of which is the `class\tprogram\terror\tline`
  header row that `tests/errors.rs`'s `cases()` explicitly skips
  (`line.starts_with('#') || line.starts_with("class\t")`). `1082 - 61 - 1 = 1020`,
  confirmed by `grep -vc '^#\|^class\t\|^$'` against the file directly.
* `tests/errors.rs`'s own floor assertion,
  `the_corpus_holds_at_least_the_rows_it_was_measured_with`, asserts
  `cases.len() >= 1020` with the comment "Floors rather than exact counts... measured
  at" the value the floor equals (matching the project's own stated convention
  elsewhere, e.g. `samples.rs`'s comment "Measured at 301 files... asserts floors
  of 250"). Running it (`cargo test -p rexx-parse --test errors
  the_corpus_holds_at_least_the_rows_it_was_measured_with`) passes against the
  current tree.

Both independent measurements land on 1,020. This looks like the count was computed
by subtracting only the 61 comment lines from the raw `wc -l` total and forgetting
the header row is also not a data row. Low stakes — it doesn't change any verdict,
and 1,020 is still comfortably above the 492 cited from Task 3.8 — but it's a
concrete, checkable factual error in a document whose stated discipline is exactly
"count precisely and report per the criterion's own instruction rather than
asserted" (that section's own words). Worth a one-line correction.

## Minor

* `phase-3-gate.md` section 1 says "Eight probe tests prove each checker rejects
  hand-built violations" and lists eight violation kinds. `tests/tiling.rs` actually
  has nine "rejects" probe tests (plus one positive-control "permits" test, for
  eleven total, which matches the doc's separately-stated "11 tests" figure). The
  omitted ninth is `the_prefix_checker_rejects_an_operator_with_nothing_in_front`,
  which the same section does mention in passing two paragraphs earlier ("A
  companion check pins that a `Prefix` node's span starts before its operand's")
  but never folds into the "eight probe tests" count or list. Undercounts the
  document's own evidence rather than overclaiming, so low risk, but worth fixing
  to "nine" with the prefix case added to the list.
* `phase-3-gate.md` numbers "Property 1" (expression nesting, vacuous) and
  "Property 2" (instruction ordering, falsifiable) straight from the criterion's own
  two clauses. `tests/tiling.rs`'s module doc numbers three properties instead:
  1 = nesting (vacuous), 2 = binary tightness (the substitute), 3 = instruction
  ordering. So "Property 2" names two different things depending on which of the two
  files a reader is looking at. Each document is internally consistent on its own;
  the collision only bites a reader cross-referencing both by that label. Trivial
  fix: don't reuse "Property N" as a name in one of the two places, or align the
  numbering.
* `d10-decision.md`'s "Axis 3 in detail" section computes "roughly 24 ms of extra
  parse time" by multiplying the spike's 8.3× grammar-layer ratio onto Task 3.10's
  whole-parser number, in the same paragraph that says doing exactly that "would
  conflate them." The hedging is present and correct (the paragraph immediately
  disclaims decisiveness and says nobody knows what the rest of the budget costs),
  but a reader skimming for a number could walk away with "chumsky costs ~24ms out
  of a ~55ms budget" as if it were a supported figure rather than an explicitly
  flagged non-computation. Consider dropping the number entirely rather than stating
  and then disclaiming it.

## What I checked and found clean (stated explicitly per instructions)

**Public surface.** `pub use` in `lib.rs` re-exports every `pub` item defined in
`ast.rs`; nothing is stranded. The `pub(crate)` boundary (`ParseCtx`, `TokenCursor`,
`Clause`/`ClauseCursor`, `Terminators`, `ExprKind::for_each_child`, etc.) is entirely
parse-time plumbing with no plausible Phase-4 use, and every non-obvious case
carries a doc comment saying why it stops there (e.g. token.rs: "Crate-internal:
nothing above the parser names it. Phase 4 consumes the AST, not the token stream it
was built from."). The wider scanner-level exports (`scan`, `Scanned`, `Token`,
`TokenKind`, `Keywords`, `Operator`, `SymbolClass`, `Tag`) are `pub` because the
crate's own integration tests (`tests/scanner.rs`, `tests/tokens.rs`) and
`src/bin/scan-check.rs` are separate compilation units that need them — not surplus
exposed for no reason. Phase 4's four named needs are all reachable as plain `pub`
fields: `Instruction::clause_span` / `Directive::clause_span`, `Program::labels`,
`Program::source` (`ProgramSource::line`, `line_count`, `span_bytes`, `join_span`,
`line_of` all `pub`), and every directive's `body: Option<CodeBody>`. `Program::symbols`
plus `SymbolTable::name` close the loop for resolving a `SymbolId` back to text.
Zero `#[allow(dead_code)]`/`#![allow(dead_code)]` remain under `src/`; the sole
remaining one is in `tests/gate_walk/mod.rs`, exactly as criterion 10 claims, and I
reproduced its own anchored grep command directly (empty output, confirming the
claim).

**Gate harnesses can all genuinely fail.** `tests/samples.rs` and `tests/variants.rs`
both assert floors (`files.len() >= 250`, `lines >= 60_000`; `corpus_count >= 14`,
`files.len() >= 250`) specifically so a directory walk that silently found nothing
cannot pass — I confirmed `rex_files_under`/`std::fs::read_dir` panics loudly rather
than degrading silently if the directory is missing, and neither test's main loop
skips a parse failure without panicking or accumulating it into a hard assertion.
`tests/tiling.rs`'s three properties are honestly labeled: property 1 (containment)
is vacuous by construction and the document says so; the binary-tightness and
prefix-order substitutes are genuinely falsifiable (see Important #1 for scope) and
each is demonstrated failing on a hand-built AST via its own unit test, which I ran.
`tests/sourceline_oracle.rs` records its regeneration driver and shell loop directly
in the test file's module doc (not only in a gitignored report), and asserts
`saw_unterminated_final_line` so the no-trailing-newline edge case cannot silently
go unexercised — I confirmed `no_trailing_newline.rex` is 6 lines by `wc -l` against
an oracle expectation of `count 7`, matching the "7 lines where `wc -l` says 6"
claim exactly.

**`tests/variants.rs`'s "no wildcard" claim, verified by direct inspection and by
grep.** I read all nine `tags!` macro invocations (`InstructionKind` 40 arms,
`ExprKind` 15, `DirectiveKind` 9, `LoopKind` 6, `Call` 4, `Signal` 3, `Use` 2,
`Trace` 4, `ParseSource` 7 — every count in the gate document's criterion 2 matches
what I counted in the macro invocations exactly) and confirmed none contains a `_`
wildcard arm. `grep -n '_ =>' tests/variants.rs` finds exactly one hit, at line 241,
inside the test body's own secondary dispatch match (routing an already-tagged
`InstructionKind` to its nested sub-enum tracker) — not inside any of the nine
`tags!` blocks, so it does not weaken the "adding a variant is a compile error"
property. `tests/gate_walk/mod.rs` has zero wildcard arms anywhere, matching its own
module doc's claim.

**Reproduced numbers.** `cargo test -p rexx-parse --test tiling --test variants
--test samples --test sourceline_oracle`: 11/1/1/1 tests, all pass, matching the
document exactly. `cargo bench -p rexx-parse -- --test`: both `CoreClasses.orx` and
`StreamClasses.orx` cases print `Success`, confirming the 41/347/2390 and 7/153/610
node-count assertions in `benches/parse.rs` hold (these are a deliberate
fixed-file "change detector," not a corpus-size count that could rot — the document
is explicit about that distinction and I agree with it). `cargo test -p rexx-parse`
totals exactly 391 tests, matching "rexx-parse alone is 391 tests." `file` and a
UTF-8 decode attempt confirm both named samples
(`samples/windows/rexutils/drives.rex`, `.../comboBoxToolTip.rex`) are genuinely
non-UTF-8 ISO-8859 text. Criterion 8's throughput share arithmetic checks out
(2.68–2.71 ms + 675–687 µs ≈ 3.4 ms, 3.4/55 ≈ 6%) and the paragraph explicitly
disclaims any fits-or-not verdict — I found no sentence in criterion 8 that a reader
could mistake for one.

## Not independently re-verified (outside my slice / time budget)

Criterion 11's differential-set regeneration (128,368 cases, `rexx-num` territory)
and criterion 7's `TRACE` reconstruction (`tests/sourceline.rs`) I read but did not
re-run the oracle capture behind them; the gate document itself labels both
appropriately as verified-tests-only vs. cited-oracle-capture, which is the honest
framing regardless of what another reviewer's slice turns up.
