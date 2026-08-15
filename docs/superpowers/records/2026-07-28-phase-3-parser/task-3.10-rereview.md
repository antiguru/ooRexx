# Task 3.10 re-review: the fix round plus two documentation sweeps

Reviewed: `review-9cd1752f..455d6b65.diff` (commits `c1f84b0a`, `bd233ed1`, `c0e1321a`,
`455d6b65`), against `task-3.10-review.md`'s findings and the fix-round section of
`task-3.10-report.md`.

## Verdict

**APPROVED**, with one Important finding about the current tree that is already fixed
outside the reviewed diff, and one Important finding about an argument's rigor that does
not overturn the underlying verdict it supports.

The Critical from the original review (false provenance for `main_instructions` and
`nested_instructions`) is fixed at all four named sites, verified against
`src/directive/tests.rs`'s actual content, not just against the new prose. I1's structural
claim (target indices, not nested vectors) is independently verified against `src/ast.rs`.
Both Minors are fixed. The clone framing is softened as instructed. Build, clippy, fmt and
the full workspace test suite are clean. The panic message was provoked and reads
correctly.

The two documentation sweeps (`bd233ed1`, `455d6b65`) are substantively sound: the marker
arithmetic is correct against the C++ header and source, the fourth handover paragraph's
AST claims are correct against `src/ast.rs`, and no plan document in scope still asserts
the false claim in so many words. But the sweep's own commit message overstated its
completeness ("six places"), and the axis-3 paragraph in `d10-decision.md` that survived
the sweep keeps asserting "decisive by itself" on a weaker footing than the wording admits.

## 1. What the fix round was dispatched to do

### 1a. The false provenance claim, four sites

Fixed and verified true at all four sites named in the review:

* `benches/parse.rs` module doc, "The assertion" / "What the triple does not observe"
  sections (current file, lines 34–75).
* The `Case` struct doc comment (lines 92–97).
* The `assert_eq!` panic message (lines 159–165).
* `d10-decision.md`, "The assertion" and "What the triple does not observe" paragraphs
  (lines 230–258).

Checked against `src/directive/tests.rs` directly, not against the new prose's own
description of itself:

* `core_classes_parses` (line 1468) pins `directives.len() == 347` as a literal
  `assert_eq!`. Confirmed.
* `the_other_shipped_packages_parse` (lines 1511–1516) pins `StreamClasses.orx`'s
  per-kind counts as literals: `classes: 7, methods: 139, attributes: 5, constants: 2`.
  `7 + 139 + 5 + 2 = 153`, matching what the new text claims. Confirmed.
* Grepped `tests.rs` for `41`, `2390`, `610`, and `153` as literals: none appear anywhere
  in the file. `main_instructions` and `nested_instructions` have no acceptance test
  anywhere in the tree, exactly as the new text now says.

The new text states this distinction correctly and does not overclaim in the other
direction either (it does not, for instance, understate that 347 really is pinned).

### 1b. The three unobserved regression shapes, recorded as prose without widening the assertion

Present, word-for-word matching the review's three items, in both `benches/parse.rs`
("What the triple does not observe") and `d10-decision.md` (same heading): control-flow
targets wired to the wrong index with counts unchanged, a clause moved across a
directive boundary while the cross-directive sum holds, and anything inside an `Expr`.
The assertion itself is unchanged (still the same three-count tuple) — confirmed by
diffing `CASES` and the `assert_eq!` shape against the pre-fix-round version, only the
panic message text and doc comments changed there.

### 1c. The two Minors

* "only `parse_program` is inside the timed region" → now "the timed region holds
  `parse_program` plus the cheap node-count check" (module doc line 26,
  `d10-decision.md` line 222–223). Fixed correctly: the nested-sum loop and the triple
  `assert_eq!` do run inside `bencher.iter_batched`'s routine closure, so the new wording
  is accurate.
* "comparable with `perf-baseline.md`" → now explicitly states `perf-baseline.md` "has no
  row for `rexx-parse` or either `.orx` file" and that "comparable" means the criterion
  *settings* agree, not that a value is checked against one recorded there (module doc
  lines 131–134, `d10-decision.md` lines 204–207). Confirmed against `perf-baseline.md`
  directly: grepped for `rexx-parse`, `CoreClasses`, `StreamClasses`, `parse_program` —
  zero hits.

### 1d. The clone framing

Softened as instructed, in both files: "Measured separately, the clone costs about 1 us
on the 141,049-byte file, under 0.1% of the parse... It is excluded anyway, on principle
rather than because it would distort the result." This matches the reviewer's 1.01
µs / 0.04% measurement (stated more loosely as "about 1 us... under 0.1%", which is a
correct, if imprecise, restatement — not a fabrication, and the report's "What went
wrong" section is honest that this number was cited from the reviewer rather than
re-derived). No re-measurement was performed here per the brief's instruction not to
re-run the benchmark.

## 2. The two documentation sweeps

### 2a. No plan document still asserts the false claim — with one exception, already fixed outside this diff

Searched all of `docs/superpowers/plans/` for `cold.start`/`cold-start` case-insensitively.
Every hit in `2026-07-27-rust-rewrite.md`, `2026-07-28-phase-3-parser.md`, and
`d10-decision.md` either correctly states parsing as *a component* of cold start, or
quotes the old wrong claim inside an explicit correction. `perf-baseline.md`'s two hits
are about the unrelated, legitimate `startup.rex` / D2 direct-measurement axis and were
never part of the wrong claim.

**However**, while re-reviewing I found a seventh, unconverted copy of the same defect
that the "six places" sweep did not touch: the Phase 3 **exit gate** criterion in
`2026-07-28-phase-3-parser.md` (originally at the line immediately after the `TRACE`
marker criterion) read:

> Parse throughput on the 5,203 bootstrap lines is recorded against the ~55 ms
> cold-start budget, with a plain statement of whether it fits.

This is the same overreach Step 4 of Task 3.10, four sections above it in the same
document, was explicitly corrected to stop asking for ("say plainly whether it fits...
asks for exactly the conclusion the data cannot support"). The exit gate criterion asked
for the opposite of what the task body a few hundred lines earlier now demands, in the
same file, and `455d6b65`'s "six places" sweep missed it because it states the claim as
an instruction to draw a conclusion rather than in the literal words "cold-start time" —
the sweep's grep-shaped search would not have matched it.

**This is already fixed on the branch**, but by a commit outside the diff I was asked to
review: `git log` shows current `HEAD` is `441551b2` ("Stop the exit gate asking for a
verdict the phase cannot reach"), one commit past `455d6b65`, landed while this re-review
was in progress. Its own commit message states plainly that this is "the seventh copy of
the claim corrected yesterday and it survived the sweep that fixed the other six." I found
this defect independently, by grep, before discovering the fix commit already existed —
so this is confirmed as a real finding against the reviewed diff, now moot for the current
tree only because of a commit that is not part of this diff's scope.

**Report note:** since `441551b2` is not part of `9cd1752f..455d6b65`, I have not scored
it as part of this diff's verdict either way, but I record it because it directly confirms
the brief's warning that "two of three fix rounds on this branch have introduced a defect
while repairing one" generalizes to sweeps as well as fix rounds — a keyword sweep is only
as complete as its search terms, and this one's search terms did not cover a paraphrase of
the same claim four sections away in the same file.

### 2b. Do the six rewrites contradict each other or `d10-decision.md`'s later section, and does D10's verdict still follow?

The six rewrites (in `2026-07-27-rust-rewrite.md`'s D10 options paragraph and Phase-3 note,
`2026-07-28-phase-3-parser.md`'s "number this phase must beat" section and Step 3, and
`d10-decision.md`'s two sections) are mutually consistent: all six now say parsing is *a
component* of the ~55 ms cold-start budget, not the whole of it, and none contradicts
another on this point. The `d10-decision.md` axis-3 paragraph explicitly cross-references
its own later section ("Task 3.10 measured the shipped parser at about 3.29 ms for both
files"), and that number (3.27–3.30 ms combined) matches the later section's own table.
No numeric or factual contradiction found between the rewrites and the fuller Task 3.10
section that follows them.

**But the argument's rigor is weaker than its confident phrasing admits, which is worth
flagging as Important even though it does not overturn the verdict.** Before the fix, the
axis-3 paragraph's reasoning was a clean (if false) syllogism: parse time *is* cold-start
time, chumsky measured a median 8.3× slower, therefore 8.3× is decisive against the ~55 ms
budget by itself. After the fix, the paragraph reads:

> It is a component of that budget, not the whole of it... and Task 3.10 measured the
> shipped parser at about 3.29 ms for both files. An 8.3× multiplier on a component that
> size is what carries the axis.

This sentence asserts decisiveness but does not show why. Two things it glosses over:

1. The 8.3× ratio was measured only on the D10 spike's **expression grammar layer**, over
   1,912 of `CoreClasses.orx`'s 4,193 lines (the subset that parses as bare expressions),
   explicitly excluding the shared scanner and everything at the instruction/directive
   level. Task 3.10's 3.29 ms figure times the **whole shipped parser**: scanning, clause
   splitting, 347+153 directives, and 2,390+610 nested instructions, not just expressions.
   Applying the spike's grammar-layer ratio to the whole-file number as if they measured
   the same scope is an extrapolation the text does not spell out or defend, and chumsky's
   cost for the instruction/directive layer specifically was never measured at all (the
   spike could not parse instructions).
2. Even granting the extrapolation, the resulting comparison — roughly 3.29 ms against
   roughly 27 ms (3.29 × 8.3), an increase of about 24 ms — is being called "decisive"
   against a ~55 ms budget whose other components (bootstrap execution, heap setup, class
   construction) are, by the document's own repeated statement two paragraphs later,
   completely unmeasured. Whether an unverified ~24 ms increase is "decisive" depends on
   how much of the remaining ~52 ms those other components already consume, which nobody
   knows yet.

None of this overturns D10's verdict: the paragraph's own next sentence ("The dependency
cost is independent of it and points the same way") correctly identifies axis 4 — 0 vs 12
transitive packages, a C compiler forced onto 5-platform CI including OpenBSD — as a fully
independent, non-inferential argument that stands on its own regardless of the throughput
axis's rigor. But the paragraph's literal claim, "Either would carry the decision," credits
the throughput axis with the same self-sufficiency as the dependency-cost axis, and on the
throughput axis specifically that credit rests on an unstated extrapolation rather than a
measurement. This is a documentation-rigor gap inherited from removing the false premise,
not a new fabrication — before the fix the logic was airtight-looking (if wrong); after the
fix the confident tone survived the load-bearing premise's removal.

### 2c. The marker arithmetic

Verified directly against `interpreter/execution/RexxActivation.hpp:92–110` and the string
table in `interpreter/execution/RexxActivation.cpp:3567–3588`. The enum holds exactly
nineteen `TRACE_PREFIX_*` values (`CLAUSE` through `INVOCATION_EXIT`, 0–18), and
`trace_prefix_table[]` confirms `TRACE_PREFIX_CLAUSE` is `"*-*"` (index 0) and the other
eighteen are evaluated-value markers. Also spot-checked the two examples the corrected text
cites: `"+++"` is `TRACE_PREFIX_ERROR` (not `CLAUSE`) and `"<I<"` is
`TRACE_PREFIX_INVOCATION_EXIT` — both real entries, confirming the claim that a naive
`>X>`-shape regex would miss them. "Eighteen value markers" (the corrected text) is right;
neither "fifteen" nor "nineteen value markers" would be.

### 2d. The fourth-handover paragraph's AST claims

Verified directly against `rust/crates/rexx-parse/src/lib.rs` and `src/ast.rs`, not just
against the review's own prior confirmation:

* `Program::instructions: Vec<Instruction>` (`lib.rs:101`), explicitly documented as
  "in source order, which is also the execution chain."
* `CodeBody::instructions: Vec<Instruction>` (`ast.rs:488`), same documented property.
* `InstructionKind::If` carries `false_target: Option<usize>` (`ast.rs:635`), not a nested
  body vector.
* `InstructionKind::Select` carries `whens: Vec<usize>`, `otherwise: Option<usize>`,
  `end: Option<usize>` (`ast.rs:663–669`) — indices into the flat vector, not owned
  children.
* `InstructionKind::When`/`WhenCase` carry `false_target`/`exit: Option<usize>`
  (`ast.rs:675–698`).
* `Loop` (used by `Do`/`Loop`, `ast.rs:850–866`) carries `end: Option<usize>`, the same
  shape.

Both structural claims in the handover paragraph are true: the two instruction lists are
flat and source-ordered, and `If`/`Do`/`Select` (and `When`/`WhenCase`/`Loop`) hold target
indices rather than owning nested vectors. The paragraph's conclusion — that a dropped
clause anywhere changes a count, while wiring corruption, cross-directive misattribution,
and `Expr`-internal corruption do not — follows from this structure and was independently
re-derived, not just re-read.

## 3. Build, test, and the panic message

* `cargo build --offline --benches -p rexx-parse`: clean.
* `cargo clippy --offline --all-targets -- -D warnings`: clean.
* `cargo fmt --all -- --check`: clean.
* `cargo test --offline --workspace --no-fail-fast`: every `test result: ok` line workspace-wide,
  0 failed, 3 pre-existing ignored (matching the report). `rexx-parse`'s own suites:
  `directive/tests.rs` 253, `program.rs` 32, `errors.rs` 23, `scanner.rs` 35,
  `sourceline.rs` 25, `tokens.rs` 9 — matches the report's per-file counts exactly, this
  time individually re-derived rather than taken on trust.

**Provoked the panic message.** Edited `main_instructions: 41` to `999` for `CoreClasses.orx`
in the working copy, ran `cargo bench --offline -p rexx-parse --bench parse -- --test`
(runs each benchmark function once without the full statistical pass), and got:

```
thread 'main' (3692) panicked at crates/rexx-parse/benches/parse.rs:152:21:
assertion `left == right` failed: CoreClasses.orx parsed to a (main, directives, nested)
count different from this benchmark's pinned baseline; directives.len() is cross-checked
against src/directive/tests.rs, but main_instructions and nested_instructions are this
benchmark's own first measurement, so check which of the three moved before assuming
which side is wrong
  left: (41, 347, 2390)
 right: (999, 347, 2390)
```

The message reads correctly: it does not claim a divergence from `tests.rs` for counts
`tests.rs` never pinned, and it correctly directs the reader to the `left`/`right` tuple
(which does show plainly that only the first element moved) rather than asserting a
specific count by name. Reverted with `git checkout -- rust/crates/rexx-parse/benches/parse.rs`
immediately after; confirmed via `git diff` (0 lines) and `git status --short` (clean) that
the file matches `HEAD` exactly.

## 4. Everything else stated as a measured fact

No new fabricated-oracle-shaped claim found. The two numeric restatements I checked against
their sources (the marker count against the C++ header, the directive counts against
`tests.rs`) both check out exactly. The 1 µs / 0.1% clone figures are correctly attributed
as the reviewer's own measurement, not re-derived or re-claimed as newly measured here, per
the brief's instruction not to re-run the benchmark.

## Tree state

`git status --short` clean at the end of this review. The one working-copy edit (the
provocation to `benches/parse.rs`) was reverted with a path-scoped `git checkout --` and
confirmed via `git diff`/`git status`. No commits made, no files created outside this
report and the scratchpad. `interpreter/` was not touched.

**Note on branch state:** `HEAD` moved from `455d6b65` (the diff under review) to `441551b2`
during this review, via a commit made outside this task's scope (see §2a). This re-review's
verdict is scored against the specified 4-commit diff; the additional commit is reported
for awareness only.
