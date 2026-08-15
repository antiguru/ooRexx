# Task 1 review: re-grain `owners.rs` to arm granularity and assert `lib.rs` against it

Range `222d438d..f9cfb060`, two commits. Reviewed in a detached worktree at
`f9cfb060`; the primary tree was never touched.

**Spec compliance: PASS.** All six steps done as written, and Step 5's
falsification is stronger than the report claims — see "The measurement the
report did not make" below.

**Quality: PASS with findings.** Three Important, four Minor. None weakens the
new assertion; all three Importants are false or unbacked *statements* about
the harness, two of them in files the re-graining falsified and the sweep
commit missed, one of them a coverage claim I measured false.

---

## Spec compliance, step by step

**Step 1 — the three inherited measurements.** Re-derived at `222d438d`, not
taken from the report.

* `instruction_owner`: two `match` tokens in the body, the outer one and
  `InstructionKind::Call(call) => match &**call {`. Exactly one nested match.
* `expr_owner`: two `match` tokens, the outer one and the word inside a
  comment ("the arm-grained match above says so"). Flat, as claimed.
* `expand_for_witnesses`: one real entry plus an identity fall-through
  (`old-loud.rs:159-169`).

All three held. Nothing was substituted.

**Step 2 — arm-grain only where `lib.rs` is.** The full row-level diff of
`owners.rs` is four lines:

```
-    InstructionKind::Call(_) => ("Call", Owner::Phase("Phase 5")),
+    rexx_parse::Call::Named { .. } => ("Call::Named", Owner::InScope),
+    rexx_parse::Call::Dynamic { .. } => ("Call::Dynamic", Owner::InScope),
+    rexx_parse::Call::Trap(_) => ("Call::Trap", Owner::InScope),
+    rexx_parse::Call::Qualified { .. } => ("Call::Qualified", Owner::Phase("Phase 5")),
```

No other row's owner or tag moved. "Leave every other row alone" is satisfied
literally.

**Row set matches the enums, and every row is reachable.** Both checked with a
program, not an argument:

* Wildcard-freeness of the *inner* match is real, not asserted-by-prose.
  Deleting the `Call::Trap` arm gives
  `error[E0004]: non-exhaustive patterns: &rexx_parse::Call::Trap(_) not covered`.
  A new `rexx_parse::Call` arm is a compile error in `owners.rs`.
* `Call::Named`/`Dynamic`/`Trap` are `Owner::InScope` rows, so
  `every_in_scope_variant_is_witnessed_by_the_phase_subsets` (`coverage.rs:684`)
  fails unless a subset program parses into each; it passes. `Call::Qualified`
  is constructed under `assert_constructs` in `loud.rs`. All four rows are
  reachable; none is decoration.
* Counts re-derived by hand from the table: 43 rows (39 variants + 4 arms,
  `InstructionKind` having 40 variants), InScope 31, `4c` 4, `Phase 5` 7,
  `Phase 7` 1, phase-owned 12, `EXPECTED_OUT_OF_SCOPE` 17 rows. Every tag in
  each table is unique, so `table_owner`'s `.find` cannot pick a wrong row.

**Step 3 — the assertion.** `table_owner` (`loud.rs:136`) reads the owner out
of `owners.rs` by tag; `every_out_of_scope_variant_fails_loudly` requires the
executed program's stderr to end with exactly that owner. No owner string is
written down in `loud.rs` as data — grep for `"4c"`/`"Phase 5"`/`"Phase 7"`/
`"4b"` in that file returns two comment lines and nothing else. The chain is
`owners.rs` → `run_program` → `lib.rs`, with nothing hand-maintained between.

The equality is partial and the code says so: only the 16 phase-owned rows are
compared, because `Loud::instruction` is unreachable for an implemented
variant. That limit is inherent, and disclosing it is right. What is *said*
about the uncovered direction is not — Important 3.

**Step 4 — deletions.** `expand_for_witnesses`, `instruction_arm` and
`loud.rs`'s `SPLIT_TABLE_PHASES` import are gone. Nothing lost coverage:

* `expand_for_witnesses` was a reconciliation; `assert_witness_set_is_complete`
  now compares the same two sets with a plain `.map`.
* `instruction_arm`'s only remaining caller was `assert_constructs`, which now
  calls `owners::instruction_tag` and gets the arm-grained tag directly. Its
  `Signal` arms were never named by any witness row (there is no `Signal`
  witness at either commit).
* The per-witness `SPLIT_TABLE_PHASES` check was strictly redundant, and
  redundant *in the same binary*: `owners::assert_owner_strings_are_split_table_phases`
  appears in the `loud` binary's own test list (8 tests, `cargo test --test loud`),
  and it holds every `Owner::Phase` row of all seven tables to that set —
  including rows `loud.rs` never asked about.

**Step 5 — falsification.** Reproduced independently, and extended. See below.

**Step 6 — the frozen message.** Stronger than the report's before/after
capture: `git diff 222d438d f9cfb060 -- src/lib.rs` contains **no non-comment
line**. Every changed line is a `///` doc line. The emitted message text cannot
have moved, by construction, so `keyword-exempt.txt`'s 790 derived rows are
safe for a structural reason and not only an observational one.

## The measurement the report did not make

The report shows M1 (a coherent `owners.rs`-side edit) goes red. That proves
the assertion *can fail*, which this project does not accept as proof it adds
coverage. So I ran the same mutation on both trees.

Mutation, identical in both: `Address` `Phase("4c")` → `Phase("Phase 5")`, with
`EXPECTED_OUT_OF_SCOPE` and both phase counts updated to match — a
wrong-but-internally-consistent plan amendment.

* At `f9cfb060`: `coverage` 10/10 green, `owners` 5/5 green, `loud` 7 passed /
  1 failed, the failure being
  `Address (Phase 5): stderr does not end with " is not implemented (Phase 5)": "rexx-exec: ADDRESS is not implemented (4c)\n"`.
  Exactly the report's M1, including the isolation.
* At `222d438d`: `cargo test --workspace` → **1020 passed / 0 failed**. The
  whole pre-task suite is green on a table that contradicts `lib.rs`.

So the assertion catches a class the old harness could not see, and the reason
is structural: the old witness carried its own `owner: "4c"` literal, checked
against `SPLIT_TABLE_PHASES` and against `lib.rs`, never against `owners.rs`.
The gate's prediction — that this would move the duplication into the
reconciler — is beaten, and nothing hand-maintained replaced
`expand_for_witnesses`.

---

## Findings

### Important

**I1. `rust/corpus/phase-4b.txt:93-102` — three sentences this task falsified,
in a live file the corpus tests read.**

```
# **That rule is per *arm* where a variant is split, and CALL is the one case
# left.** owners.rs's table is variant-grained, so `InstructionKind::Call`
# reads `Owner::Phase("Phase 5")` -- ...
# ... loud.rs's `expand_for_witnesses` is
# where the arm-level truth is machine-checked; reading the coarse `Call` tag
# alone would say that no program here may contain a `CALL` at all, which is
# not what the table means.
```

All three clauses are now false: the table is arm-grained, there is no
`InstructionKind::Call` row, and `expand_for_witnesses` does not exist. There
is also no "coarse `Call` tag" left for the caveat to be about. This is not a
dated plan — `corpus.rs` and `coverage.rs` both read this file, and its header
is the standing rule for what may be listed in the 4b subset. It is exactly the
class `f9cfb060` set out to sweep ("files this task did not otherwise open"),
and the sweep did not reach it.

**I2. `rust/crates/rexx-exec/tests/owners.rs:214-221` — the same stale claim
inside the file the task rewrote.**

```rust
// In scope since 4b's Task 4: unlike `InstructionKind::Call`, which
// stays split -- `Owner::Phase("4b")` at the time this comment was
// written, ... `Owner::Phase("Phase 5")` since Task 7 moved `Call::Trap`
// in scope, leaving only `Call::Qualified` loud (review round 1's M6
// corrects this comment, ... an edit at line 129 below was not
// propagated here) --
```

`InstructionKind::Call` no longer "reads `Owner::Phase("Phase 5")`" — the row
this sentence describes is the one the commit deleted, and the sentence now
contradicts the new block 30 lines above it (`owners.rs:185-195`). The pointer
"an edit at line 129 below" was exact at `222d438d`, where line 129 was
`InstructionKind::Call(_) => ("Call", Owner::Phase("Phase 5")),`; line 129 is
now the `Select` row. ("below" was already the wrong direction at base — the
comment sits ~60 lines further down.)

**I3. "corpus.rs covers that direction" is false, measured.** Three places:
`src/lib.rs:675-679`, `tests/loud.rs:503-509`, `tests/owners.rs:608-612`. The
shared sentence:

> a phase string written onto [an implemented variant] would be data nothing
> reads; `corpus.rs`'s byte-for-byte run against the oracle is what covers that
> direction, by failing as soon as an implemented construct prints a gap.

Measured: with `instruction_owner` changed to return `Some("Phase 5")` for
`InstructionKind::Say` and nothing else touched, `cargo test --workspace` is
**1020 passed / 0 failed** — corpus.rs included. Nothing covers that direction,
which is what the sentence's own first clause says. `corpus.rs` covers a
*different* risk (an implemented construct starting to emit a gap message,
i.e. a `run.rs` regression), not a wrong owner string on an implemented
variant. The honest form is "nothing covers it, and nothing needs to, because
no path reads it". This matters more than an ordinary prose slip: it is the
paragraph the task added specifically to bound its own claim, and the report
flags overclaiming here as the defect to avoid.

### Minor

**M1. `tests/loud.rs:211-212`** — the new sentence "Nothing in `owners.rs`'s
tables is owned by `"4b"` any more" is true today (grep: no `Owner::Phase("4b")`
row in any of the seven tables), but it is asserted only for
`INSTRUCTION_TAGS` (`owners.rs:493-499`). A future `"4b"` row in `EXPR_TAGS` or
the other five would leave it false with nothing red. "any more" is also the
"used to" shape the `e96f3435` rule bars.

**M2. `tests/owners.rs:473`** — "**`INSTRUCTION_TAGS` is one row per owner, not
one per variant**" is literally false: 43 rows, five distinct owners. The
intended sense is one row per *owned unit* (variant, or arm where a variant is
split). The rest of the sentence is correct.

**M3. `docs/superpowers/plans/phase-4b-gate.md:876-887`** still states in the
present tense that `owners.rs` "is **variant**-grained", that `loud.rs`
"already carries the reconciliation (`instruction_arm` and
`expand_for_witnesses`)", and that a straight equality assertion "is therefore
**not available**". The report discloses this and leaves it deliberately as a
dated record; noting it because a 4c planner who opens the gate directly reads
three false statements, and the disclosure lives in a task report they may not
open.

**M4. `tests/owners.rs:224`** still says `ExprKind::Call`'s two `CallTarget`
forms are "both 4b's" — the twin of the sentence `f9cfb060` deliberately
rewrote in `coverage.rs:76` to "this crate evaluates both". Same claim, two
files, one updated. Not false, but the inconsistency is one more thing to
reconcile later.

### Checked and found correct

* All three of `f9cfb060`'s replacements are true of the tree. "Nothing in
  `owners.rs`'s tables is owned by `4b`" — grep confirms. "`CallTarget` has
  exactly two forms and this crate evaluates both" — `ast.rs:91-98` has two
  variants, `eval_call` (`eval.rs:525-528`) matches both. "a builtin-named call
  still fails loudly, through `Loud::unresolved_call` naming `4c`" — measured,
  `say abs(1)` gives `rexx-exec: routine "ABS" is not implemented (4c)`, rc
  120; `say 'f'(1)` likewise. "The four that remain … are all `Phase 5`'s" —
  true and asserted.
* `Loud::unresolved_call` is not affected by the split: `call ns:sub` gives
  `rexx-exec: CALL is not implemented (Phase 5)`, rc 120.
* No new hand-maintained structure. `EXPECTED_OUT_OF_SCOPE`, the pinned counts
  and the witness rows all predate this task and are all pinned *against* the
  tables rather than reconciling them; M1's isolation result (only the new
  assertion red) is the empirical form of that.
* `rust/scripts/mutate-4b.sh` names neither deleted function, so its 12
  mutations are unaffected by the deletion.
* `rexx-parse/tests/variants.rs` has its own separate `tags!`/`INSTRUCTION_TAGS`
  and is untouched by the re-grain.
* The report's "T0-M2 and T0-M4 are moot" is correct — both subjects deleted.

## Not verified

* `cargo fmt`, `cargo clippy`, the three gates and `mutate-4b.sh` — the lead
  verified these at `f9cfb060` and asked that they not be re-run.
* The report's M2 (six `lib.rs` instruction owners → `"4c"`). The lead verified
  the equivalent single-row mutation on `Call::Qualified`; I verified the
  complementary `owners.rs`-side direction instead, which is the one the gate's
  prediction was about.
* The report's Step 6 method (16 witness programs captured before and after).
  Replaced with a stronger structural check: the `src/lib.rs` diff is
  comment-only, so no message text could have moved.
