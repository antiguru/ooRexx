# Task 4 review: a call nested inside an expression

Commit `b73ef0b6e`. Reviewed against `task-4-brief.md` and `task-4-report.md`
(status DONE_WITH_CONCERNS), diff `review-4cc6f8943..b73ef0b6e.diff`, and the
tree at `95b01a671` (the one commit since touches only a new plan document, so
every reviewed file is at `b73ef0b6e`).

**Spec compliance: PASS.** Every step of the brief is done, and the one
deviation is deliberate, correct, argued from a measurement, and carried back
into the plan.

**Code quality: PASS WITH FINDINGS.** No behavioural defect found. Three comment
statements are false or stale, one of them an instance of the defect class this
plan has now produced in all four tasks, and one structural assertion the file's
own precedent calls for is missing.

---

## What I ran

| check | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 (warm target; the cold-target run is the report's, not re-done) |
| `cargo test --workspace --no-fail-fast` | 0 `test result: FAILED`, 0 `overflowed its stack` |
| `cargo test -p rexx-exec --lib ir::` | 76 passed, 0 failed |
| `cargo test -p rexx-exec --test ir_dual` | 9 passed, 0 failed |
| `RUST_MIN_STACK=262144 cargo test -p rexx-exec --lib ir::corpus_shape` | 1 passed |
| oracle re-capture of stanzas `s4` and `s5`, fresh empty directory | byte-identical to the recorded rows |

---

## The six checks the brief asked for

### 1. The deviation (`Option<NodePath>`), and its measurement — **upheld**

The argument is right and the claim holds.

The brief's `native_shape(expr, path: NodePath)` refuses a whole slot whenever
*any* node stands past the width, because each operator arm answers
`path.child(..).is_some_and(..)`. `NodePath::ROOT` is `NodePath(1)` and
`child` refuses once the sentinel reaches bit 31
(`rust/crates/rexx-exec/src/ir/mod.rs:800`), so the width is 31 steps.
`rust/corpus/lang/deep_nested_expr.rex` is one assignment of 3000 `1 +` terms —
2999 operators on its left spine, two orders of magnitude past the width — and
holds no call. Under the brief's version that whole assignment becomes one
`Op::EvalExpr`. That is not a judgement call: it follows from the arm's shape.

I confirmed the *after* state through the same path `corpus_shape_tests` uses
rather than taking the report's histogram. `deep_nested_expr.rex` is in
`rust/corpus/phase-4a.txt:47`, so the sweep covers it; `root_of` returns
`Root::Arith` for that assignment and `check_body` asserts the compiled clause's
last value-producing op equals it
(`rust/crates/rexx-exec/src/ir/corpus_shape_tests.rs:392-402`). The sweep passes,
so the assignment ends in `Op::Arith` and not `Op::EvalExpr`. The report's
histogram is also internally consistent with the op naming the golden tests pin
(a numeric `1` is `ExprKind::Constant` → `Op::LoadConstant`): 1 `Clause` + 3000
`LoadConstant` + 3000 `TraceLiteral` + 2999 `Arith` + 2999 `TraceOperator` + 1
`Store`, plus the `SAY` body's 4 = 12004.

The deviation is also test-covered, which is worth stating because the depth
test the brief asked for cannot see it: both designs give the same answer for a
call at 31 and at 32. What separates them is `corpus_shape_tests` + a call-free
deep program, which is exactly the M6/M7 split the module doc now records, and
that split is analytically sound (a node bound reddens the sweep and the golden
test; an address bound reddens the golden test alone).

### 2. The stack fix — **sound; "production is unaffected" is established, its stated reason is not**

The fix does not swallow a panic and does not change what the sweep asserts.
`std::panic::resume_unwind(panic)` re-raises the original payload on the test
thread, which is the same pattern `on_interpreter_thread`
(`rust/crates/rexx-exec/src/lib.rs:2601`) already uses. The body moved verbatim:
the only lines the diff removes from this file besides doc comments are the
`fn` signature and the `native`/`root_of` arms, so no assertion changed.

Independently verified that the fix does what it claims: the sweep passes under
`RUST_MIN_STACK=262144`, a stack a sixteenth the size of the one it used to
depend on. That is the property that matters, and it does not rest on the
report's `RUST_MIN_STACK` bisection (which I did not re-run — it needs the
pre-task tree).

"Production is unaffected" is established rather than assumed: `rexx-run` over
the program and over generated programs to 40000 terms is a measurement of the
production path, and that path spawns `INTERPRETER_STACK_BYTES` at
`lib.rs:2601`. See F3 for the reason given for it, which is wrong.

### 3. The deleted test — **its property is genuinely re-covered**

`an_operand_that_needs_eval_leaves_the_whole_expression_general` stated: an
operand with no op takes the whole expression down, plus an adjacent success
showing the case is about the term and not the shape.
`the_value_shapes_outside_the_native_set_stay_general`
(`rust/crates/rexx-exec/src/ir/golden_tests.rs:556`) states both halves: the row
`zw = .nil || za` is a non-native **operand** taking the whole slot to
`EvalExpr`, and `zw = zv || za` is the adjacent success in the same test. The
new width test covers the call-with-no-address half. No silent loss.

### 4. The depth-bound test — **both halves present, bound pinned to the width**

`a_call_nested_past_the_paths_width_leaves_the_slot_general`
(`golden_tests.rs:145`) computes `width` by walking `NodePath::child(true)` to
exhaustion, then asserts the *success* at `nested(width)` — the call's op present
with `path=root` + 31 `.R` steps spelled out, and no `EvalExpr` anywhere — and the
*refusal* at `nested(width + 1)` as an exact three-op render. The success half is
what pins the bound to `NodePath`'s width rather than to a nearby number: a
compiler giving up at any shallower depth reddens it. Taking the width off
`NodePath` rather than writing `31` is the right call here and the doc says so,
naming `ir::tests::a_node_path_carries_thirty_one_steps_and_refuses_the_thirty_second`
as where the number itself lives.

### 5. `corpus_shape_tests`' independence — **still independent; the constant can drift silently**

The file still restates the promoted set over the parse tree alone: `native` and
`root_of` call nothing in `compile`, and every operator is spelled out. The new
`Call` arm and `DEEPEST_ADDRESSED_CALL = 31` keep that discipline, and 31 is the
correct number for the current `NodePath`.

It can drift. See F5.

### 6. The five new stanzas and the corrected `arithmetic` comment — **consistent, well-formed, oracle-true where I checked**

Tags are the file's four (`program` / `rc>` / `out>` / `err>`);
`render_both_engines` builds the whole expected block from the tree-walker's
output, so a stanza with no `err>` rows does assert empty stderr, and
`chunks_refused == 0` is asserted, so the compiled arm really compiled these
programs rather than falling back. No oracle-crashing form (no `INTERPRET` of a
crashing shape, no unbounded recursion, no non-UTF-8 byte). Arithmetic checks
out by hand in every stanza (`5, 7, 15, 7` for the corrected `arithmetic`
stanza; `3, 3, n=2, 1, 1, "2 2", 323` for the first new one; `8, 43, 34, 3, 5, 4`
for the two-calls one).

I re-captured two of the five against the C++ oracle from a fresh empty
directory — the trapped stanza and the `trace i` stanza, the two carrying the
most claim — and both are byte-identical to what the file records, including
`in-arg 42 at 2` / `operator 41 at 6` / `callee 40 at 10` and all 34 stderr
lines.

The corrected `arithmetic` comment is now true of its rows, and its added
sentence about the last row ("the register it lands in is the one the enclosing
operator reads as its right") is true of `zt + 1 + length('x')`.

### 7. `NodePath::child`'s `expect(dead_code)` — **removal was required**

`rust/crates/rexx-exec/src/ir/compile.rs:839` (`descend`) is the only
non-`cfg(test)` caller of `child` in the workspace; every other hit is in
`ir/mod.rs`'s test module, `run.rs`'s test module, or `golden_tests.rs`. Because
the attribute was an `expect`, a production caller makes it unfulfilled and
`-D warnings` fails the build. Required, not convenient — which is what the
`expect`-rather-than-`allow` comment it replaced was for.

### Constraints

* **No second implementation.** Nothing was added to `drive.rs`; the promoted
  call still enters `Interp::eval_call_resolved` at `drive.rs:615` with the
  resolution from `chunk.resolved_call(*site)`. The new `push_native` arm only
  emits ops.
* **Computing op emits nothing, echo immediately behind.** `Op::CallExpr` is
  followed by `Op::TraceFunction` at the single emission site. But see F4.
* **`Engine::TreeWalker`.** The diff names it nowhere.
* **Comment-vs-test discrimination.** I checked all seven new claims by reading
  the mechanism rather than trusting the table. Six hold — including the two the
  report leans on hardest: a route that always stepped left renders `root.L`
  where the golden test's `right` case asserts `root.R`, and a prefix operand
  spelled as a right step reaches `chunk_node_at`'s `_ => return None`
  (`run.rs:6851-6857`) and becomes `Loud::call_op_off_its_node`. The
  shared-site claim is right too, and its correction — that the two calls must
  be of *different* routines, because an all-`length` stanza would pass under
  that mutation — is a genuine improvement over what it replaced. F1 is the
  seventh.

---

## Findings, most severe first

1. **`ir/mod.rs:233-234` and `golden_tests.rs:47` claim a promotion rule wider than the code's, and nothing tests the gap.** `Op::CallExpr`'s doc says the promoted set is "every call an address reaches"; the golden test's doc opens "A call compiles to `Op::CallExpr` **wherever it sits in a value**". Both are false: `push_value` promotes only when `native_shape` accepts the **whole slot**, so `zz = .nil || length('a')` — address reaches the call, one sibling node outside the native set — compiles to one `Op::EvalExpr` and the call gets no op. This is the plan's recurring defect in its fourth task, and here it is also a coverage gap: `the_value_shapes_outside_the_native_set_stay_general`'s rows hold no call, and the width test's refusal is about the address, so no test excludes an implementation that promotes the call and leaves the rest general. Fix the two glosses; a row `zz = .nil || length('a')` in the existing general-shapes test would close the gap in one line.
2. **`corpus_shape_tests.rs:50-51` says "Two mutations were applied to `compile`" and then describes three.** This task rewrote the second bullet into two mutations (refuse every node past 8; refuse only an address past 8) without touching the count above it. A comment naming the size of a set, now wrong by one — name the set or drop the numeral.
3. **`corpus_shape_tests.rs:501` states a falsehood: "this sweep is the one caller that reaches `compile` directly".** `golden_tests.rs:40`, `:380` and `:1346` call `super::compile` directly on libtest threads, and `plan.rs`'s own unit tests reach it through `Interp::chunk_for` on libtest threads. The true reason no other harness overflowed is that no other direct caller compiles a body this deep — which is the sentence a future reader needs, because the false one implies those callers are protected. The same wrong reason appears in the report and in the plan ("every other one reaches `compile` through `Interp`, which runs on a thread with `INTERPRETER_STACK_BYTES`"). The fix itself is unaffected.
4. **`Op::TraceFunction` has no adjacency assertion, where the four sibling echo ops do.** `compile.rs:708-712` runs `assert_trace_ops_open_a_clause_region`, `assert_literal_echoes_follow_their_load`, `assert_read_echoes_follow_their_load`, `assert_operator_echoes_follow_their_op` and `assert_prefix_echoes_follow_their_op` — the last added by Task 2 at this same seam — and none for `Op::TraceFunction` behind `Op::CallExpr` with the same `src`/`dst`. The property holds by construction today (one emission site), but so does every other one these guard, and this task is what turned a root-only emission into one reachable at any depth. The gap predates the task; this was the moment to close it.
5. **`DEEPEST_ADDRESSED_CALL = 31` and `root_of`'s `Call` arm can both be wrong without any test reddening.** No corpus program nests a call past depth 31, so the constant is inert: widen `NodePath` to a `u64` and `a_node_path_carries_thirty_one_steps_and_refuses_the_thirty_second` reddens in another file while this constant silently keeps expecting `EvalExpr` for calls at 32-63. Separately, `Root::CallExpr` is the one `Root` variant absent from the anti-vacuity list (`corpus_shape_tests.rs:578-586`), so `root_of`'s `Call` arm may never fire in the sweep at all. Both are honestly flagged in the report; both are worth a line in the file itself, since the independence that makes drift possible is deliberate and the reader should be told the cost.
6. **`ir_dual_cases/operators` prose nits from this edit.** Line 52 is 125 characters where the file wraps at ~78 — an insertion left unwrapped mid-sentence. "The `trace i` row carries the abuttal" (line 52) reads as definite where the file now has two `trace i` rows. And the new stanza's "The rows separate them by condition number under one trap" describes three `signal on syntax name` statements with three labels; the sibling stanza at line 277 has the same shape and does not say "one trap".
7. **The plan's stack paragraph is stale in its own tense.** "`ir::corpus_shape_tests` compiles `deep_nested_expr.rex` directly on a libtest thread" is present tense and was made false two sentences later by the fix the same paragraph describes. Otherwise the corrected plan text is true: the `native_shape`/`descend` block matches the code exactly, the `push_native` arm matches, the threading note and the `too_many_arguments` note match, and the measurement it records is the one the sweep now confirms.

---

## Judgement on the deviation and the plan correction

The implementer was right to deviate and right to correct the plan rather than
implement the brief and note the problem. The brief's version would have made a
corpus program compile to a single `Op::EvalExpr` in a plan whose purpose is to
promote expressions — an optimisation task shipping a de-optimisation — and the
argument for `Option<NodePath>` is the correct one: only the call arm is
addressed, so only the call arm should pay for an address. The corrected plan
text is true as written, with the one tense slip at F7.

The report's own two reservations are both fair. Not doing the frame-shrinking
refactor is the right call for the reason given (a successful bundle restores
exactly the margin that just failed), and the numbers to argue from are in the
report. Leaving `Root::CallExpr` out of the anti-vacuity list rather than adding
an unverified row is the right call for the reason given, and is F5's second half.
