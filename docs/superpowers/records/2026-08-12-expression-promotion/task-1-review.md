# Task 1 review: the non-arithmetic binary operators as one native op

Reviewed: `ec649c0bc..5a801e1e9` (one commit, 11 files, +826/-162), against
`task-1-brief.md` and `task-1-report.md`.

## Verdicts

**Spec compliance: pass on the code, incomplete on Step 6b.**
Every interface the brief names exists with the signature it names; the split,
the dispatch, the arm order, the hint discipline, the driver arm, the golden
test, the corpus-shape restatement and the eleven-stanza case file are all
present and do what the brief asked. Three deviations from the brief's literal
text (`result` for `result?`, the dropped `ExprKind::Logical` row, two files off
the file list) are each necessary and each disclosed in the report. What is not
complete is Step 6b: the brief named one falsified sentence, and the diff
corrected that one. At least eight further sentences were falsified by this task
or left false by a partial correction, one of them in the module doc of the file
this task rewrote.

**Code quality: good, with required corrections.**
The mechanism is right: one dispatch, one enumeration per family, provably one
hint-reserving path, an adjacency assertion widened without weakening, and tests
that can still redden. Every defect below is in prose, except one test-coverage
hole and one undocumented error-path change.

## What I verified rather than took on trust

Read-only commands, run from `rust/`:

* `cargo fmt --all --check` -- exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0 (warm target,
  as the report itself flags).
* `cargo test -p rexx-exec --lib` -- 595 passed, 0 failed.
* `cargo test -p rexx-exec --test ir_dual` -- 9 passed, 0 failed, including
  `both_engines_agree_on_every_case_file`.
* `git status --short -- crates/rexx-exec crates/rexx-core` -- clean, so what I
  read is what was committed.

Constraint-by-constraint:

* **No second implementation.** The driver's `Op::Binary` arm
  (`ir/drive.rs:830`) and `eval_node`'s collapsed arm (`eval.rs:507`) both end in
  `Interp::apply_binary`. There is no second route to an answer.
* **Only `Op::Arith` reserves a hint slot.** `/bin/grep -rn "hints.reserve"
  crates/rexx-exec/src/ir/` returns exactly one call site, `compile.rs:880`,
  inside the `is_arithmetic(*op)` branch of the `if`/`else` that chooses the op.
  This is true on every path, not only the commented one.
* **`size_of::<Op>() == 12`.** `ir/mod.rs:65` is untouched and the crate builds,
  so the new variant fits.
* **The adjacency assertion.** `assert_operator_echoes_follow_their_op` is still
  called unconditionally at `compile.rs:711`, and the widened `matches!` is an
  or-pattern with a *shared* guard `applied == echoed && dst == src` -- register
  and operator are still both checked, not only position.
* **The operand/operator split preserves order.** `compare_values`' body is
  byte-identical to `eval_compare`'s from the first line after the prologue: the
  two parses and the two settings still all precede the two `render`/`text`
  shared borrows.
* **The rooting contract is stated at each of the three** functions and honoured
  by both callers (`push_temp` in `eval_node`'s arm, the register file in the
  driver, whose arm says so in its own comment).
* **Arm order.** The collapsed arm sits after the arithmetic arm *and* carries
  `!is_arithmetic(*op)`, so `**`'s asymmetric exponent handling is safe against a
  later reordering too.
* **`corpus_shape_tests` restates the set independently.** `other_family` spells
  out all twenty-four operators and calls nothing in `eval`; `root_of`/`native`
  ask `arithmetic(*op) || other_family(*op)`. The file can still fail, and
  `Root::Binary` is in the anti-vacuity list.
* **`Loud::binary_operator`'s premise.** `rexx_parse::Operator` has exactly 32
  variants (7 + 3 + 18 + 3 named by the four predicates, plus `Backslash`), and
  `expr.rs:752` returns error 35.1 for a `Backslash` in dyadic position, so the
  parser really cannot build the node. It is loud anyway, which is what the brief
  asked for.
* **The case file has no oracle-crashing form**: no `SELECT`, no `DATE(`, and the
  highest `NUMERIC DIGITS` is 9.
* **The case file's arithmetic.** I recomputed every `out>` block by hand:
  concatenation precedence and left-associativity (`pq r`, `prq`, `pqp q`,
  `a bc d`), both comparison families over `'1'`/`'1.0'`/`' 1'`/`'ABC'`/`'abd'`,
  the DIGITS-4 vs DIGITS-9 rounding, the FUZZ-3 vs FUZZ-0 pair, the eight-row
  truth table, the trapped stanza's `SIGL` values 2/6/10 and `RC` 42/42/34, the
  nesting rows, and both trace stanzas. Every recorded byte agrees with what the
  program must print. `rc>`/`out>`/`err>` tags are well formed -- the harness
  parses and matches them.

## Findings, most severe first

### 1. `eval.rs`'s own module doc now names three functions that no longer exist, and its count is wrong

`crates/rexx-exec/src/eval.rs:32`:

> **Six functions here open a temps frame and then use `?`, so a raised
> condition leaves their own `pop_frame` unreached. That is deliberate, and
> Tasks 10 and 11 should copy it rather than repair it.** `eval_prefix`,
> `eval_arithmetic`, `concat`, `eval_compare`, `eval_logical` and
> `eval_logical_list` are the six.

`concat`, `eval_compare` and `eval_logical` were deleted by this diff. I read
each remaining site: the sites are now **four** -- the new `eval_node` binary arm
(`eval.rs:516`), `eval_prefix` (`661`), `eval_arithmetic` (`715`) and
`eval_logical_list` (`1082`).

This is the exact class the constraints forbid ("may not say ... how many call
sites there are"), it is in the file this task rewrote, and this task is what
falsified it. The same claim, also falsified, sits in two files the diff did not
touch:

* `crates/rexx-exec/src/run.rs:14461` -- "the six `?`-skipped `pop_frame` sites
  in `eval.rs`".
* `crates/rexx-core/src/roots.rs:123` -- "`rexx-exec` has six functions that open
  a frame and then use `?`".

The `roots.rs` one is load-bearing: it is the argument for why `pop_frame` must
truncate and must not assert. The argument survives the recount, but the number
in it does not, and per the rule the sentence must be corrected or the count
removed -- not hedged. Removing the count and the enumeration (they are what
rots) leaves the mechanism stated and unfalsifiable.

### 2. The twelve-to-eighteen correction is partial; two more "twelve" sentences survive, one naming a deleted function

The report's correction 4 says two doc comments said "twelve" and both now say
eighteen. Two more were left:

* `crates/rexx-exec/src/eval.rs:18` -- "and (Task 8) with the twelve comparison
  operators", in the module doc of the file being rewritten.
* `crates/rexx-exec/src/run.rs:8925` -- "(`eval_compare`'s own doc comment states
  it for the twelve expression operators ...)". Both halves are now wrong: there
  is no `eval_compare`, and `compare_values`' doc says eighteen.

Note also `eval.rs:1218`, which says "`CompareOp` has only twelve variants" --
that one is about `rexx-num`'s enum and is correct; the fix must not sweep it up.

### 3. Six dangling references to `eval_compare` / `eval_logical`

Reachable with `/bin/grep -rn "eval_compare\|eval_logical\b"`:

* `eval.rs:1022` -- `eval_logical_list`'s doc: "unlike `&` (`eval_logical`'s own
  doc comment)". The forward link from `logical_values` was updated; this back
  link was not.
* `eval.rs:1217` -- "for the eighteen `Operator` variants `eval_node` dispatches
  to `eval_compare`". Both halves wrong now: `apply_binary` dispatches, to
  `compare_values`. The `unreachable!` message fifteen lines below it *was*
  corrected to say `apply_binary`, so the two now contradict each other inside
  one function.
* `eval.rs:2032` and `2035` -- `a_comparison_reuses_an_already_parsed_num_cache`'s
  own comment, twice.
* `run.rs:7006` and `7016` -- "Reasoned rather than routed through
  `eval_compare`'s own `Operator::StrictEqual`" and "Calling `eval_compare` would
  work too".

### 4. A stanza comment claims a separation its rows do not show, and the rows cannot tell `|` from `&&`

`tests/ir_dual_cases/operators:164`:

> `&&` is exclusive-or, which is what separates it from `|` in the fourth and
> seventh rows.

The fourth row is `say 1 | 0` and the seventh is `say 1 && 0`; both print `1`.
They agree, so they separate nothing. The stanza has no `1 | 1` row at all -- its
only `|` rows are `1 | 0` and `0 | 0`, and exclusive-or answers identically on
both -- so an implementation computing `Or` as `Xor`, or `Xor` as `Or`, passes
this stanza unchanged. The row that would separate them, `1 && 1` -> `0`, has no
`|` counterpart to be compared against.

Adding `say 1 | 1` (which the oracle answers `1`) closes the comment and the hole
together. Until then this is a comment claiming coverage the stanza does not
have, which is the "test that cannot fail" shape applied to a comment.

### 5. Off-by-one in the FUZZ stanza's comment

`tests/ir_dual_cases/operators:146`:

> at FUZZ 3 two numbers differing in the seventh digit compare equal and two
> differing in the fourth do not

The row is `say 1000000 = 1000100`. `1000000` and `1000100` differ in the
**fifth** significant digit, not the fourth. The recorded byte (`0`) is right;
the sentence describing it is not.

### 6. The corrected `arithmetic` header replaces a false sentence with an inaccurate one

`tests/ir_dual_cases/arithmetic:26`, new text:

> and each family reaches that line by a different route through `eval.rs`, so
> these rows are what says the four agree on the tag, the indent and the order
> whichever route they took

After this task no family reaches the `>O>` line through `eval.rs` on the default
engine -- all four reach it through `Op::TraceOperator`. On the tree-walker all
four reach it through the *same* path, `trace_intermediate`'s `ExprKind::Binary`
arm. What differs per family is where the *value* is computed (`eval_arithmetic`
against `apply_binary`'s three arms), not the route to the line. What these rows
are actually worth -- the four families in one clause stream, agreeing on tag,
indent and order -- is the rest of the same sentence and stands on its own; the
"different route through `eval.rs`" clause should go.

Related, same file, the family stanza's own comment (line 240): "A promotion that
moved one family's node and not another's would print these from an op and could
get the tag or the indent wrong for the family it moved." The conditional is
still true, but all four families now *do* print from an op, and the sentence
reads as though none of them does.

### 7. An error-path behaviour change that "keeps its body verbatim" does not cover

Old `eval_compare` and `eval_logical` returned through `?` (`map_err(...)?` and
`ok_or_else(...)?`) *before* their `pop_frame`, leaving the frame for
`step_in_temps_frame`'s outer truncation to heal -- which is precisely what the
module doc in finding 1 is about. The collapsed arm binds the result and pops
unconditionally:

```rust
let result = self.apply_binary(*op, left_value, right_value);
self.roots.pop_frame(frame);
result
```

So a failing comparison or logical now pops its own frame where HEAD did not.
This is safe -- `pop_frame` truncates to a watermark, pops are idempotent by its
own doc, and a `Failure` carries bytes rather than an `ObjRef` -- and it is the
better behaviour. But it is a real difference from HEAD on a path the report
calls behaviour-preserving, it is not mentioned anywhere, and it changes the
subject of the prose in finding 1. Say it, in the arm or in the report.

### 8. The DIGITS and FUZZ stanzas are one row short of the control they claim

"The strict row under the same setting is the control -- `==` compares bytes and
the setting cannot move it" (line 128), and the same shape in the FUZZ stanza.
Each stanza has exactly one strict row, at one setting. That shows the two
families differ under one setting; it does not show the setting cannot move the
strict answer, which needs the strict row at both settings. Either add
`numeric digits 9` / `say 1.0001 == 1` (and the FUZZ analogue), or say what the
single row actually shows.

### 9. Disclosed deviations from the brief -- no action needed, recorded for the ledger

All three are in the report and all three are defensible:

* `result` rather than the brief's `result?`. The brief's snippet does not type
  check; `eval_node`'s arms are `Result<ObjRef, Failure>`.
* The `ExprKind::Logical` comma-list row the brief asked for was replaced with
  `>za`. The reason -- a comma list only ever appears in a condition, and a
  condition compiles to `Op::EvalExpr` whatever its shape, so the row would pass
  under every implementation of its own subject -- is correct and is written into
  the test's doc comment, not only into the report.
* Two files off the brief's list (`src/lib.rs` for `Loud::binary_operator`,
  `tests/spike.rs` for one of the four re-pinned tests).

## The four re-pinned tests (brief item 4)

Judged individually, against what each name claims:

* `eval_survives_exactly_max_eval_depth_terms_and_prints_the_oracles_own_answer`
  and `eval_raises_11_1_exactly_one_term_past_max_eval_depth` -- both names are
  about **`eval`**, and `MAX_EVAL_DEPTH` is `eval`'s own counter. Pinning them to
  the tree-walker makes the test agree with its own name rather than narrowing
  it. They still fail if the `>` becomes `>=`.
* `records_the_stack_cost_of_one_eval_frame` -- likewise named for `eval`'s frame.
  Unchanged in what it measures.
* `the_stack_span_does_not_depend_on_what_else_the_program_evaluated` -- this one
  **is** narrowed. Its name is about the program, not about `eval`, and before
  this task it made its statement about the default engine. It now makes it about
  the tree-walker only. The doc comment added says so explicitly ("both spans
  would be the depth of nothing -- equal, and equal for the wrong reason"), which
  is the right way to narrow a test, and the alternative (leaving it on the
  default engine) would have been a test that passes vacuously. Accept, but note
  that nothing now measures per-level stack cost on the compiled engine, whose
  `native_shape`/`push_native` recursion is unbounded -- which is the report's own
  concern 1 and belongs to the plan.

The doc comments added to all four say the true thing, and `depth_limited`'s
comment carries the measurement rather than only asserting the claim.

## Case file, remaining checks (brief item 5)

* Eleven stanzas, one `program` directive each, tags well formed, no
  `REWRITE=1` path.
* No oracle-crashing form.
* Coverage against the brief's list: all three concatenation spellings including
  the one-space-vs-four pair, both comparison families, DIGITS, FUZZ, all three
  logical operators, `say (0 & 'x')` and `say ('y' & 'x')`, a nested operand, a
  failure per family, a `trace i` and a `trace r` stanza. All present.
* The header states honestly that all three mutations were caught without the
  file and gives the reason to keep the rows anyway (they pin oracle bytes, which
  an engine-against-engine comparison structurally cannot). That conclusion is
  written into the file itself, not only the report -- which is what was asked.
* The "identical bytes on both engines before the promotion as well as after"
  measurement cannot be re-run from a review; the report describes the method
  (revert `native_shape`'s one line, restore from a pre-revert copy) and it is the
  right method. Taken on the report's word, flagged as such.
* Two comment defects inside the file are findings 4, 5 and 8 above.
