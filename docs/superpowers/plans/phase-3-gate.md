# Phase 3 exit gate — assessment

Phase 3 built `rexx-parse`: source retention, the scanner, clause splitting, the expression and instruction grammars, directives, block assembly, and parse errors, all measured against `build/bin/rexx` and `build/bin/rexxc`.

The gate has eleven criteria.
**Ten are met, one is met with a recorded vacuity**, and two of the ten needed the corpus reshaped before they could be tested at all.
Unlike Phase 2, no criterion was unassessable: the gate was written against a parser and the two oracle binaries, and that is what exists.

Every claim below says whether it was **verified for this document** (the command was run and its output read) or **cited from a task's report** (the task's tests were re-run, but the oracle captures behind them were not re-taken).
Those are different strengths and are not blurred.

Four criteria had no harness at all when the gate closed and no task ever owned them: tiling (1), variant enumeration (2), the `samples/` round-trip (3), and `SOURCELINE` (6).
Their harnesses were built at the gate, as `cargo test` targets rather than scripts, so they run every time the workspace tests run.

---

## 1. Corpus programs tile — MET, with one property vacuous by construction

> Every program in `rust/corpus/lang/` parses without error, and each one tiles: expressions nest, and instructions are ordered with only whitespace-class interstices.

Verified for this document: `cargo test --offline -p rexx-parse --test tiling`, 11 tests, all pass over the 16 corpus programs.

**Property 1, expression nesting, is satisfied by construction and no input can falsify it.**
`Expr::new` (`rust/crates/rexx-parse/src/ast.rs`) widens the extent it is given to cover every child before storing it, so a parser path that computes a span wrongly produces a too-wide span, never a containment violation.
Checking it over the corpus therefore says nothing about the parser.
That is the vacuity failure mode this project has now hit three times, and it is recorded here rather than ticked quietly.
The check is kept anyway, as a pin against a future change to `Expr::new`, and its can-fail probe exercises the checker on a hand-built node, which is evidence about the checker and not about the parser.

**What was added in its place**: a falsifiable property that catches what nesting was reaching for.
For every `Binary` node, the left operand's span ends at or before the right operand's starts, and the bytes between them, after removing whitespace-class bytes, are operator bytes (plus the parentheses that belong to no node) and nothing else; for the two synthesised operators, abuttal and blank, the gap holds no operator byte at all; for every real operator it holds at least one.
A mis-nesting that attaches an operand to the wrong operator preserves containment under widening and breaks this.
The AST does not retain the operator token's own position (`ExprKind::Binary` carries only the `Operator` value), so the check is over the gap's byte class rather than an exact position; that absence is itself worth knowing before Phase 4 wires up error reporting on operators.
A companion check pins that a `Prefix` node's span starts before its operand's.

**Property 2, instruction ordering, is falsifiable by parser bugs** and no construction guarantees it: clause spans come from the splitter per clause and the assembler appends in clause order, so a dropped clause leaves its bytes in an interstice and a span miscomputation can overlap.
The full-file clause sequence is the main body's instructions, then each directive's own clause followed by its body's instructions.

Recorded decisions, each pinned by a probe test:

* A `;` is **not** whitespace-class, although a null clause (`;;`, or `;` alone on a line) would legitimately put one into an interstice.
  The corpus has no null clause today (measured), and permitting `;` would weaken the dropped-clause net.
  If a corpus program ever gains one, or if this checker is ever extended to `samples/`, revisit that first: 301 real programs are far likelier to hold a null clause than 16 hand-written ones.
* The corpus has **zero** `,`/`-` continuations today (measured), so the permitted continuation case is exercised only by the checker's own probes.
* A `::RESOURCE` directive's raw body lines belong to the `Resource` node but sit between clause spans, so a corpus program that gains one will fail tiling for a reason that has nothing to do with a dropped clause.
  No corpus program has one today.

Eight probe tests prove each checker rejects hand-built violations: escaped child spans, mis-nested operands, overlapping operands, a missing operator, overlapping clause spans, a dropped clause, an interstitial `;`, and bytes after the last clause.
No parseable input can produce any of these, so hand-built nodes through the public field surface are the only demonstration possible.

One defect in the harness itself was found and fixed during construction: the first interstice scanner classified a `-` followed by only blanks up to the end of the checked range as a line continuation, which swallowed the subtract operator in `recurse(n - 1)` as a phantom continuation.
A continuation now requires an actual line terminator.
The corpus caught this on the first run, which is the direction of failure a checker should have.

## 2. Variant enumeration — MET, gated wider than worded

> Every `Instruction` and `Expr` variant is constructed at least once, asserted by a test that enumerates the variants rather than by inspection, over `corpus/lang` and `samples/` together.

Verified for this document: `cargo test --offline -p rexx-parse --test variants`.

The literal wording covers `InstructionKind` (40 variants) and `ExprKind` (15).
Both are gated, and so are `DirectiveKind` (9), `LoopKind` (6), `Call` (4), `Signal` (3), `Use` (2), `Trace` (4) and `ParseSource` (7), because the C++'s 52 instruction classes collapse into those sub-enums and Phase 4 dispatches on them; coverage came out complete, so nothing fell back to report-only.

The enumeration cannot go stale: each tag function is a `match` with no wildcard arm, generated by a macro that emits the checked tag list from the same invocation, so a new variant is a compile error in the test rather than a silently shrinking check.

**Fourteen variants were unreachable from the existing corpus plus all 301 samples** and needed a program written for them, which is `rust/corpus/lang/gate_variants.rex`:

* `InstructionKind::Queue`, `InstructionKind::Options`
* `ExprKind::QualifiedCall`, `ExprKind::ClassResolver`, `ExprKind::VariableReference`
* `DirectiveKind::Annotate`
* `LoopKind::With`
* `Call::Qualified`, `Call::Trap`
* `Signal::Value`
* `Use::Local`
* `Trace::Default`, `Trace::Skip`
* `ParseSource::LineIn`

That fourteen real programs' worth of samples never spell `queue`, `options`, a namespace-qualified call, `::annotate`, `do with`, `use local` or `parse linein` is a fair sketch of how far the sample set's idiom lags the language's surface.
`gate_variants.rex` passes `build/bin/rexxc`, runs silently under `build/bin/rexx` (both verified), and keeps every runtime effect behind `if 0 then` or inside a routine or method that is never called, because the `SOURCELINE` driver below loads it as a package and that runs its prolog.

## 3. `samples/` round-trips — MET

> Every `.rex` file under `samples/` round-trips to an AST.

Verified for this document, both directions:

* Oracle: all files pass `build/bin/rexxc` with the `ulimit -v 1048576` wrap, 0 failures (re-measured for this document, not carried forward from the plan).
* Rust: `cargo test --offline -p rexx-parse --test samples` parses every file.

Measured counts, printed by the test rather than asserted exactly: **301 files, 67,519 physical lines**, the same as when the plan was written.
The test recurses (`samples/*.rex` is only the 36 top-level files), asserts floors of 250 files and 60,000 lines so a walk that silently finds nothing cannot pass, and reads every file as bytes.

Bytes matter here as more than style: two shipped samples, `samples/windows/rexutils/drives.rex` and `samples/windows/oodialog/controls/ToolTip/comboBoxToolTip.rex`, are ISO-8859 and not valid UTF-8 (measured with `file` and confirmed with `iconv`), so a `read_to_string` harness would have failed on real files.
That is the first hard evidence for D14's bytes-not-`str` rule from files in the tree rather than from constructed probes.

## 4. `CoreClasses.orx` and `StreamClasses.orx` parse end to end — MET

Verified for this document: `cargo bench --offline -p rexx-parse -- --test` runs both parses and their node-count assertions (41/347/2,390 for `CoreClasses.orx`, 7/153 and its nested count for `StreamClasses.orx`), both `Success`.
The counts themselves are Task 3.10's change-detector baseline, cited from its report; what this document verified is that today's parser still produces them.

## 5. Parse errors, both directions — MET, cited with the harness re-run

> Soundness over the crate's error corpus with matching number, sub-number and plausible line; completeness over that corpus plus `samples/` and the two `.orx` files.

The harness is `rust/crates/rexx-parse/tests/errors.rs` (32 tests) over `rust/corpus/errors/parse-errors.tsv`, whose rows carry the oracle's answers and, as their own field, which direction each row is.
**Measured for this document: 1,021 corpus rows**, against 385 when the criterion was drafted and 492 at Task 3.8, so the count is reported per the criterion's own instruction rather than asserted.
The recorded exceptions (the 18.1/18.2 versus 35.1 label structural difference, the eager-scan masking deviation, the nine non-translation rejections in two classes) are cited from Tasks 3.6 and 3.8.

Verified for this document: the 32 tests pass in the workspace run.
Cited, not re-verified: the oracle answers baked into the TSV were not re-captured against `rexxc`; the file's own header records how to regenerate them.
The completeness direction over `samples/` is independently covered by criterion 3's fresh `rexxc` sweep above.

## 6. `SOURCELINE(n)` matches the interpreter — MET, with a created witness

> For every line of every corpus program, including the last line and a file without a trailing newline.

Verified for this document: `cargo test --offline -p rexx-parse --test sourceline_oracle`.

The oracle side was captured fresh for this gate: a driver runs `.Package~new(file)~source` for each of the 16 corpus programs, prefixes its own output because constructing the package runs the file's prolog and the two interleave, and the capture keeps only prefixed lines.
Every invocation was wrapped in `ulimit -v 1048576`.
The expectations are committed at `rust/crates/rexx-parse/tests/sourceline_oracle/`, one file per program, and the exact regeneration commands live in the reading test's module comment, because this document's scratch driver does not survive and the expectations do.
`git status` was checked after the first driver run: `.Package~new` wrote nothing into the tree this time.

The test compares `ProgramSource::line_count` and every `line(n)` byte-for-byte, and asserts `line(count + 1)` stays `None`, because the interpreter raises 40.34 past the end rather than answering an empty string.

**The corpus had to be reshaped before this criterion was testable**: every existing corpus program ends with a newline (measured), so the criterion's named edge case had no witness.
`rust/corpus/lang/no_trailing_newline.rex` was created for it, and the oracle counts its unterminated last line as a full line (7 lines where `wc -l` says 6), which is exactly the behaviour the criterion wanted pinned.
The reading test fails if the corpus ever stops containing such a file, so the witness cannot silently rot away.
A later reader should know the corpus was shaped to the criterion here, not that the criterion was satisfied by luck.

## 7. `TRACE`'s `*-*` lines reconstructible — MET, cited with the tests re-run

Scoped, per the gate's own recorded narrowing, to `trace_output.rex` and Task 3.9's two probes.
The tests are in `rust/crates/rexx-parse/tests/sourceline.rs` (`trace_output_rex_reconstructs_every_traced_line` and the probe tests), which reconstruct each `*-*` text from `clause_span` through `join_span` and compare against oracle transcripts captured by Task 3.9.
Verified for this document: the tests pass in the workspace run.
Cited: the transcripts themselves were not re-captured.

## 8. Parse throughput as a share of the cold-start budget — MET as re-scoped

> Recorded against the ~55 ms cold-start budget, with a plain statement of what share parsing accounts for and what is still unmeasured, and explicitly not as a fits-or-not verdict.

Re-measured for this document (`cargo bench --offline -p rexx-parse`, release profile):

| file | Task 3.10 | this document |
|---|---|---|
| `CoreClasses.orx` (4,193 lines) | 2.64 ms | 2.68–2.71 ms |
| `StreamClasses.orx` (1,010 lines) | 655 µs | 675–687 µs |

Parsing the two shipped bootstrap packages costs **about 3.4 ms, roughly 6% of the ~55 ms budget**.
What is unmeasured is everything else in that budget: bootstrap execution, heap setup and class construction, none of which exists yet.
No fits-or-not conclusion is available at the end of Phase 3, and none is drawn; asking for one was the overreach this criterion was rewritten to remove.

## 9. Clippy clean, zero `unsafe` — MET

Verified for this document:

* `cargo clippy --offline --workspace --all-targets -- -D warnings` finishes clean, with the four new gate test files included in `--all-targets`.
* `unsafe_code = "forbid"` stands at `[workspace.lints.rust]`, and a grep over every crate's `src/`, `tests/` and `benches/` finds no `unsafe` token anywhere.

## 10. Every `allow(dead_code)` names its deleting task — MET

Verified for this document: the gate's own anchored grep prints nothing.

```bash
grep -rnE '^\s*#\[allow\(dead_code\)\]' rust/crates/rexx-parse/src/ | grep -v 'Task 3\.[0-9]'
```

In fact no `allow(dead_code)` remains under `src/` at all; the last three went when Task 3.7b became their items' caller.
One new `#![allow(dead_code)]` exists in `tests/gate_walk/mod.rs`, outside the criterion's scope on purpose: it is per-binary surplus in a module shared by two test crates, not a not-yet-called library item, and its comment says so.

## 11. Phase 2's differential sets still at 0 — MET

Verified for this document, from generation onward rather than from a cached result: all twelve sets were regenerated from `rust/crates/rexx-num/tests/gen-curated-sets.py`, the oracle answers re-captured through `data-addsub-oracle.rex` / `data-format-oracle.rex` under the `ulimit` wrap, and the Rust side re-run through the `addsub`, `muldiv` and `fmt-check` binaries.

| set | cases | divergences |
|---|---|---|
| `addsub` | 8,712 | 0 |
| `addsub2` | 8,112 | 0 |
| `muldiv` | 17,424 | 0 |
| `md2` | 20,184 | 0 |
| `pow` | 2,112 | 0 |
| `cmp` | 32,368 | 0 |
| `signblank` | 2,320 | 0 |
| `fmt` | 1,800 | 0 |
| `fmt2` | 6,720 | 0 |
| `fmt3` | 12,136 | 0 |
| `fmtedge` | 640 | 0 |
| `fmtcarry` | 15,840 | 0 |
| **total** | **128,368** | **0** |

This remains a script-and-oracle workflow rather than a `cargo test`, because it needs the built C++ interpreter; the workspace's own `rexx-num` tests are curated pins, not these sets.
That standing gap is Phase 2's, recorded in its gate document, and is unchanged here.

---

## The workspace, in one run

`cargo test --offline --workspace --no-fail-fast`: **575 passed, 0 failed, 3 ignored**, the three being `rexx-num` format tests that allocate 2–3 GB and say so in their `#[ignore]` reasons.
`rexx-parse` alone is 391 tests.
`cargo fmt` clean before every commit.
The Phase 2 gate's qualification still stands: no CI builds or tests the Rust tree, so every figure in this document is a local claim.

## What was created to make criteria testable

Two corpus programs exist because the gate needed them, and the corpus was shaped to the criteria rather than the criteria being satisfied by what happened to be there:

* `no_trailing_newline.rex`, the witness for criterion 6's edge case.
* `gate_variants.rex`, reaching criterion 2's fourteen otherwise-unconstructed variants.

Both were validated against `rexxc` and run under `rexx` before entering the corpus, and both now flow through every corpus-wide harness: tiling, variants, `SOURCELINE`, and criterion 5's corpus walks.

## What went wrong, so the next gate expects it

* The interstice scanner's first version swallowed a subtract operator as a phantom line continuation (criterion 1 above).
  The corpus caught it on the first run; the fix carries a comment naming the trap, and the probes pin the corrected behaviour.
* Property 1 as worded is unfalsifiable, and demonstrating it with a hand-built lying span would have read as parser evidence while being checker evidence.
  The criterion's intent was recovered by adding the tightness property instead of by working around the wording; this is recorded per the standing rule that a criterion satisfiable without meaning anything must be said plainly.
* The shared test walk module produced per-binary dead-code errors under `-D warnings`, because each test crate compiles its own copy and neither uses every item.
  The scoped `#![allow(dead_code)]` that fixed it sits one grep-anchor away from criterion 10's pattern; it does not match because that grep is anchored to `src/`, and this paragraph exists so a future tightening of that grep to `tests/` finds the reasoning already written.
* Clippy's `single_range_in_vec_init` rejected a legitimate one-element probe array; the workaround (hoisting the range) is commented at the site.
  Trivial, but it is the kind of friction that tempts a blanket allow, which was not added.
