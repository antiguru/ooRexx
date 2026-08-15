# Task 3.7b review: the public entry point

## Verdicts

**Spec compliance: PASS.**
`parse_program` and `parse_interpret` match the brief's interfaces exactly, including the four sign-offs.
`Program::labels` is `BTreeMap<Box<[u8]>, usize>`, built with `entry().or_insert()`.
The 99.914 check is one call, after the main body, reported against the first would-be-directive clause.
Directive bodies are parsed and discarded rather than skipped.
Labels are built from the main body only.
The single-`ClauseCursor` redesign the coordinator asked for is implemented cleanly, with the claimed one non-definition call site.

**Code quality: PASS.**
The composition is small, the borrow-order invariant is real (enforced by the type system, not by convention), and every claim in the report that could be independently checked, checked out.
One cosmetic gap noted below (Minor).

## Findings

### Critical

None.

### Important

None.

### Minor

1. **`Program` and `Fragment` derive nothing, not even `Debug`.** Every other public AST type in this crate (`Instruction`, `Directive`, `Expr`, `SymbolTable`, `SourceKind`, ...) derives at least `Debug`. `Program`/`Fragment` don't, and the reason is structural, not an oversight in this diff: `ProgramSource` itself (from an earlier task) derives nothing, so `Program`/`Fragment` can't derive `Debug` without one first landing on `ProgramSource`. This doesn't bite the task's own test suite, which only ever calls `.unwrap()` on the `Result` (that needs `ParseError: Debug`, which holds) and never `.unwrap_err()` on it (which would need `Program`/`Fragment: Debug`). It does bite anyone probing this crate ad hoc from outside: I hit it immediately writing a throwaway verification test that called `.unwrap_err()`. Pre-existing constraint, not a regression, and not worth blocking on — flagging so a later task doesn't rediscover it the hard way.

## Verification performed

**Borrow order (priority 1).** Read `ast.rs`: `Expr::span`, `Instruction::clause_span`, `Directive::clause_span` are all `Range<usize>` documented as byte ranges, never token indices. `Resource::lines` (the one AST field that touches `ParseCtx::resources`, the field the brief didn't count among the "four" but that `ParseCtx` also borrows) is likewise `Vec<Range<usize>>` — byte ranges copied out at parse time, not a reference into `scanned.resources`. `lib.rs`'s `Parsed` struct holds no token vector and no `Clause`/`ClauseCursor`; `ctx` and `cursor` are local to `parse()` and dropped at its end. `SymbolTable` is moved once, from `scanned.symbols` into `Parsed.symbols` into `Program`/`Fragment.symbols` — no clone. This all compiles for the reason the doc comment claims: nothing in the surviving AST holds a token index.

**Labels, both directions.** Read `instruction.rs`'s `label()` (pre-existing, Task 3.6): upcases a symbol label via `ctx.symbols.name(*id)`, takes a literal label's bytes verbatim — matches the brief's sign-off. Confirmed `build_labels` in `lib.rs` uses `labels.entry(name.clone()).or_insert(index)`, never a plain `insert`. Re-ran mutation M5 (plain `insert` instead of `entry().or_insert()`) myself: CAUGHT, 22 passed / 1 failed, `a_duplicate_label_keeps_the_first_occurrence_not_the_last` failing with `left: Some(4), right: Some(2)` — exactly the report's claimed count.

**99.914 placement (priority 3).** Wrote two throwaway probes (deleted after) calling `parse_interpret` directly and inspecting `ParseError::byte`:
* `"say 1; ::routine r"` → `(99, 914)`, `byte = 7`, exactly where `::routine` starts in the fragment text.
* `"::routine r"` alone → `byte = 0`.
* `"say 1; ::routine r; ::routine s"` (two directive-shaped clauses after the check fires) → still `byte = 7`, the *first* one, confirming the check fires once rather than per-directive.

Re-ran mutation M4 (wrong error number, 915 instead of 914) myself: CAUGHT, 22 passed / 1 failed, matching the report.

**Bodiless-directive fallthrough (priority 4).** Independently re-ran four of the five oracle cases in fresh `mktemp -d`s against `build/bin/rexxc` (not reusing the implementer's files): `::CLASS`, `::OPTIONS`, `::RESOURCE` (with a real `::END`-terminated body) each gave `Error 99.916: Unrecognized directive instruction.` on the trailing `say`. `::CONSTANT`'s counter-case gave `Error 99.938: Constant methods cannot have a method body.`, confirming the directive's own specific check still wins over the generic fallthrough. All four match the report's measurements verbatim.

**Test honesty / mutation re-application (priority 5).** Re-applied three mutations to a live edit of `lib.rs`, ran `cargo test -p rexx-parse --offline --test program` after each, reverted, confirmed `git status --short` was clean and the baseline (23 passed / 0 failed) was restored before moving to the next:

| Mutation | Result | Failing test | Count |
|---|---|---|---|
| M10: `DirectiveKind::Method` arm forced to `false` | CAUGHT | `a_method_directives_body_is_recognised_and_consumed` | 22 passed / 1 failed |
| M5: `labels.insert()` instead of `entry().or_insert()` | CAUGHT | `a_duplicate_label_keeps_the_first_occurrence_not_the_last` | 22 passed / 1 failed |
| M4: 99.914 → 99.915 | CAUGHT | `interpret_rejects_a_directive_with_99_914` | 22 passed / 1 failed |

All three match the report's own table exactly, including M10 — the mutation the report's *first* pass missed before adding coverage. The current suite catches it.

**`directive_has_body`'s exhaustiveness.** Cross-checked `ast.rs`: only `MethodDirective`, `AttributeDirective`, `RoutineDirective` declare a `body: bool` field; `ClassDirective`, `ConstantDirective`, `Annotate`, `Requires`, `Resource`, and `OptionsForm` (a `Vec`, not a struct) do not. `directive_has_body`'s match arms line up 1:1 with this, and the match is exhaustive over `DirectiveKind`'s 9 variants with no wildcard — a tenth directive kind would fail to compile here rather than silently defaulting to `false`.

**Cursor-sharing redesign.** `grep -rn "parse_instructions("` across the crate: exactly one non-definition call site outside `lib.rs`, in `instruction/tests.rs`'s `parse_kind` helper, which now builds its own `ClauseCursor` via `split_clauses` to preserve the old per-test behaviour. Matches the report's claimed blast radius.

**`parse_interpret`'s `Fragment`.** `ProgramSource::new(text, SourceKind::Interpret)` is used, not `SourceKind::Program` — checked directly in `lib.rs`. `SourceKind::Interpret`'s single-line/no-Ctrl-Z/raw-LF-is-13.1 rules are implemented in `source.rs` from an earlier task and already covered by `tests/scanner.rs` and `tests/sourceline.rs`; this task's job was only to wire `SourceKind::Interpret` into the entry point, which it does. Each `parse_interpret` call builds a fresh `SymbolTable` (via a fresh `scan`), so no cross-call id aliasing is possible by construction.

**Full-suite sanity after mutation testing.** `cargo test -p rexx-parse --offline`: 0 failed. `cargo clippy -p rexx-parse --offline --all-targets -- -D warnings`: clean. `cargo fmt -p rexx-parse --check`: clean. `git status --short`: clean (no leftover edits from mutation testing or probes).

## Unverifiable from this review

* Whether `Program`/`Fragment`'s omission of `directives`-emptiness as a *typed* guarantee (rather than a `debug_assert!` plus the 99.914 early return) will cause friction for Phase 4 — that's a forward-looking design question outside this task's brief, not a defect in it.
