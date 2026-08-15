# Phase 3 gate closure — implementer's report

Task: build the four unowned gate harnesses, confirm the measurement criteria, and write the gate assessment.
Full assessment with per-criterion verdicts: `docs/superpowers/plans/phase-3-gate.md` (committed).
This report is the working record behind it.

## Commits

* `31db9b89` — two corpus programs the criteria needed witnesses for (`gate_variants.rex`, `no_trailing_newline.rex`).
* `f8887d34` — `tests/samples.rs`, `tests/tiling.rs`, `tests/variants.rs`, shared `tests/gate_walk/mod.rs`.
* `c6267de1` — `tests/sourceline_oracle.rs` plus 16 committed expectation files under `tests/sourceline_oracle/`.
* `7ce33726` — `docs/superpowers/plans/phase-3-gate.md`, and the samples.rs comment corrected to the two `iconv`-confirmed non-UTF-8 samples.

All four harnesses are `cargo test` targets per the overriding constraint; the only script-shaped piece is the SOURCELINE oracle capture, which is a driver plus committed expectation files that a Rust test reads, exactly the allowed shape.

## Pre-flight

Six questions were sent before building; all six were answered and all six changed something:

1. Corpus witness for the no-trailing-newline case: authorised, created, and the gate doc says the corpus was shaped to the criterion.
2. Enumeration scope: hard-gate everything whose coverage is complete; it came out complete for all nine enums, so nothing is report-only.
3. Property 1 vacuity: treated as a finding, not a demonstration target; the falsifiable binary-tightness property was added in its place.
4. `;` stays rejected, with the explanatory failure message and the samples-extension warning in doc and module comment.
5. `tests/sourceline_oracle/` location confirmed; regeneration commands live in the test's module comment, not only here.
6. Gate doc assesses all eleven criteria, marking verified-here versus cited-from-task per criterion.

## Measurements (all fresh, this session)

* `samples/`: 301 files, 67,519 lines; `rexxc` 0 failures under the ulimit wrap; Rust parses all 301.
* Variant residue before `gate_variants.rex`: 14 variants across 9 enums (list in the gate doc and the commit message).
* Corpus facts: no null clauses, no `,`/`-` continuations, no `::RESOURCE`, no shebangs, every pre-existing file newline-terminated.
* Differential sets: regenerated and re-run end to end, 12 sets, 128,368 cases, 0 divergences.
* Throughput re-measured: `CoreClasses.orx` 2.68–2.71 ms, `StreamClasses.orx` 675–687 µs, within ~3% of Task 3.10; ~6% of the ~55 ms budget, no fits verdict drawn.
* Workspace: 575 passed, 0 failed, 3 ignored (pre-existing 2–3 GB allocation tests); clippy `-D warnings` clean; zero `unsafe`; dead-code grep prints nothing.
* `git status` checked after the first `.Package~new` driver run: nothing appeared in the tree.

## What went wrong

* **The interstice scanner's first version had a real bug**: a `-` followed by only blanks to the end of the checked range was classed as a line continuation, which swallowed the subtract operator in `call_procedure.rex`'s `recurse(n - 1)`.
  The corpus run caught it immediately, which is the failure direction the harness should have; the fix requires an actual line terminator and carries a comment naming the trap.
  This is probe-discipline shaped: the scanner was written against the clause-interstice picture and first misbehaved in the expression-gap context.
* **Property 1 is unfalsifiable by any input**, because `Expr::new` widens spans over children.
  Caught in pre-flight rather than after building, but the brief asked for a violating *input* that cannot exist; the criterion's intent was recovered with the tightness property and the vacuity is recorded in the gate doc rather than worked around.
* **The shared test module tripped `-D warnings` dead-code per binary**, since each test crate compiles its own copy.
  Fixed with a scoped, commented `#![allow(dead_code)]` in `tests/gate_walk/mod.rs`; it is outside criterion 10's `src/` anchor, and the gate doc says so explicitly so a future grep-tightening finds the reasoning.
* Small friction: clippy's `single_range_in_vec_init` fired on a one-element probe array (hoisted the range, commented), and one walker lifetime needed explicit plumbing before child references could escape the visitor closures.
* Not wrong but worth naming: the enumeration walker reimplements the crate-private `ExprKind::for_each_child` from the public field surface.
  That is deliberate (an integration test cannot reach the private one, and the duplication is exhaustive-match-guarded), but it is a second copy of child-visitation logic that a future variant addition must update in two places; the compiler enforces both.

## Limits worth knowing

* The 128,368-case differential run is still a script-plus-oracle workflow, not a `cargo test`; it needs `build/bin/rexx` and stays outside the workspace suite, as it was in Phase 2.
* The tiling checker's `;` strictness and the `::RESOURCE` interaction mean extending it to `samples/` is not a one-line change; both are documented at the decision points.
* Nothing was edited in `src/`, `benches/parse.rs`, or any existing test file; the permitted-file boundary held without exceptions.
