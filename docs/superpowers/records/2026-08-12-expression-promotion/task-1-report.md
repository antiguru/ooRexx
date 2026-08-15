# Task 1 report: the non-arithmetic binary operators as one native op

## What changed, and why

### `eval.rs` -- the operand/operator split

`Interp::concat`, `Interp::eval_compare` and `Interp::eval_logical` each ran one
operand prologue (`push_frame`, evaluate left, `push_temp`, evaluate right,
`push_temp`) followed by the operator's own work followed by `pop_frame`. Each
is split at that seam into a method over two already-rooted values:

* `concat_values(&mut self, left_value: ObjRef, right_value: ObjRef, separator: &[u8])`
* `compare_values(&mut self, op: Operator, left_value: ObjRef, right_value: ObjRef)`
* `logical_values(&mut self, op: Operator, left_value: ObjRef, right_value: ObjRef)`

Each keeps its body verbatim between the prologue and the `pop_frame`, and its
doc comment. The rooting contract moved to the caller and is stated on each of
the three, citing `Interp::arith_general`'s existing wording. The measured
behaviour the three collapsed `eval_node` arms documented all survives: `Blank`
inserting exactly one space however much whitespace separated the terms (now on
`concat_values`, where the separator argument is), the comparison
operators (on `compare_values`), and "both operands always evaluated, never
short-circuited" (split between the new `eval_node` arm, which does the
evaluating, and `logical_values`, which does the checking -- each half is
stated where it now lives).

`Interp::apply_binary` is the one dispatch, exactly as the brief specifies it,
with a `Loud::binary_operator(op)` arm rather than `unreachable!`. `eval_node`'s
three arms collapsed into one, placed after the arithmetic arm, guarded
`is_native_binary(*op) && !is_arithmetic(*op)`.

The families are enumerated by free functions: the existing `is_arithmetic`,
plus new `is_concatenation`, `is_comparison` (moved out of `eval_node`'s
comparison arm rather than copied) and `is_logical`. `is_native_binary` is their
positive disjunction.

`logical_values`' old `_ => unreachable!("eval_node only dispatches And/Or/Xor
here")` became `other => return Err(Loud::binary_operator(other).into())`, for
the same reason the brief gives for `apply_binary`'s arm.

### `lib.rs` -- `Loud::binary_operator`

A new loud message in the shape of the internal-inconsistency ones already there
(`missing_body`, `chunk_map_too_short`, `op_not_driven`): a plain message, no
`owned_message` owner tag. It names the variant (`{op:?}`) rather than the
spelling, because `Operator::Abuttal`'s spelling is the empty string and a
message naming it would name nothing; `Operator`'s derived `Debug` is one word
with no tree behind it, unlike `Loud::expression`'s subject. `Operator` was added
to `lib.rs`'s `rexx_parse` import list.

### `ir/mod.rs`, `ir/compile.rs`, `ir/drive.rs`, `ir/golden.rs`

* `Op::Binary { op: Operator, lhs: u16, rhs: u16, dst: u16 }` with a doc comment
  in `Op::Arith`'s style. **`const _: () = assert!(size_of::<Op>() == 12)` still
  holds** -- the build passes with the variant added, and the assertion was not
  touched.
* `Op::TraceOperator`'s doc comment said "a separate op from the `Op::Arith` that
  computed it" and "immediately behind the `Arith` whose register it reads".
  Both are false once a second op is echoed by it, so both are corrected to name
  the operation rather than `Arith`.
* `native_shape`'s `Binary` arm asks `is_native_binary` instead of
  `is_arithmetic`.
* `push_native`'s `Binary` arm keeps its register discipline byte for byte and
  chooses `Op::Arith` when `is_arithmetic(*op)` and `Op::Binary` otherwise.
  **Only the `Arith` branch calls `hints.reserve()`**, with a comment at the
  decision point saying why.
* `assert_operator_echoes_follow_their_op` now accepts either computing op in
  front of the echo (an or-pattern with a shared guard); its doc comment is
  corrected in the same two places.
* `assert_region_ops_name_their_clause`'s exhaustive match gains `Op::Binary` in
  the "names no instruction" group.
* The driver's arm mirrors `Op::Arith`'s: the same two `debug_assert!`s, both
  operands read before the destination is written, `self.apply_binary`,
  `break 'region Err(failure)`, `set_temp`. The matching
  `Loud::op_not_driven("Binary")` arm sits directly after `Arith`'s.
* `golden.rs` renders `Binary op=<spelling> lhs= rhs= dst=`, with no `hint`
  field -- and a comment saying that the absence is what a golden reading this
  line beside an `Arith` one says.

### `ir/golden_tests.rs`

`every_binary_operator_but_arithmetic_compiles_to_one_op` is new and holds exact
rendered streams (Step 1's `contains` pair was replaced once `golden.rs` rendered
the op, as the brief directs). It runs one operator per family (`||`, `=`, `&`)
over the same two operands, so the three streams differ in the operator alone,
and then the arithmetic stream beside them -- which is the half that pins the
split, since only it carries `hint=0`.

`only_the_arithmetic_operators_promote` is rewritten rather than deleted, as
`the_value_shapes_outside_the_native_set_stay_general`. **I did not use the
brief's suggested "`ExprKind::Logical` comma list" row**, and the reason is in
the "brief corrections" section below. The rows are `.nil`, `.nil || za` and
`>za`, with `zv || za` as the adjacent success.

### `ir/corpus_shape_tests.rs`

`Root` gains `Binary`; `Root::of` classifies `Op::Binary`; `root_of` and `native`
restate the widened set **independently**, through a new `other_family(op)` that
spells the operators out rather than calling `eval::is_native_binary`
-- which is the property that lets this file fail. `Root::Binary` is added to the
anti-vacuity list, so a corpus that stopped reaching the new op would be red.
The module doc's "an `Op::EvalExpr` where an `Op::Arith` is expected" sentence is
generalised to "a computing op", since the drift it describes is now possible in
two ops.

`other_family`'s doc says why `Operator::Backslash` is in neither list.

### `tests/ir_dual_cases/operators` (new)

Every expected byte is the C++ oracle's -- see the capture section below. The
stanzas cover, in order: the three concatenation spellings
with the blank under one space and under four; both comparison families over the
same operands; a comparison under narrowed `NUMERIC DIGITS` with a strict control
under the same setting; a comparison under non-zero `NUMERIC FUZZ` and the same
site with `FUZZ` back to zero; the three logical operators over every operand
combination; `say (0 & 'x')` untrapped, which is the "both operands checked
though the first decides" witness; `say ('y' & 'x')` untrapped, which reports the
**left** operand's text; a trapped failure reached through each of the three
families; an operand that is itself an operator; a `trace i` stanza carrying `||`,
the abuttal (tag `""`), the blank (tag `" "`), a comparison and a nested logical;
and a `trace r` stanza.

### `tests/ir_dual_cases/arithmetic` (Step 6b)

Two comments claimed the other three families are unpromoted. The header's
"Family" paragraph and the family stanza's own comment are both corrected to say
what those rows are still worth -- the four families together in one clause
stream, agreeing on tag, indent and order -- and to point at `operators` for the
three families' own rows. No rows were deleted.

### Four tests that the promotion silently emptied -- see "concerns"

`src/eval.rs` (`eval_survives_exactly_max_eval_depth_terms_and_prints_the_oracles_own_answer`,
`eval_raises_11_1_exactly_one_term_past_max_eval_depth`), `src/lib.rs`
(`the_stack_span_does_not_depend_on_what_else_the_program_evaluated`) and
`tests/spike.rs` (`records_the_stack_cost_of_one_eval_frame`) all build a deep
chain of `||''` and measure `eval`'s recursion. With `||` promoted, that chain
compiles to native ops and `eval` is never entered, so `outcome.stack.max_depth`
is 0 and all four fail. Each now runs on `Engine::TreeWalker` explicitly, with a
doc comment saying that the recursion being measured is `eval`'s and that the
compiled engine does not enter it. `eval::tests::depth_limited` is the one place
carrying the measurement behind that, cited from the other three.

## Gate commands, unpiped exit statuses, counts

Run from `rust/`. Final tree, after every edit above.

| command | status | counts |
| --- | --- | --- |
| `cargo fmt --all --check` | `FMT_STATUS=0` | -- |
| `cargo clippy --workspace --all-targets -- -D warnings` | `CLIPPY_STATUS=0` | -- |
| `memcap 8G cargo test --workspace --no-fail-fast` | `TEST_STATUS=0` | 1460 passed, 0 failed, 4 ignored |

Each status was read from `$?` on the command itself, not through a pipe; the
test log was written to a file and summed afterwards. The clippy run was over a
warm target directory, which `rust/CLAUDE.md` warns makes a same-session green
provisional -- flagged here rather than claimed as a phase-boundary lint.

Intermediate runs the brief calls for:

* **Step 2** (`cargo test -p rexx-exec every_binary_operator_but_arithmetic`):
  status 101, `0 passed; 1 failed; ... 594 filtered out`. The run count is 1, so
  the filter matched.
* **Step 4** (workspace, after the `eval.rs` split alone, before any `ir/`
  change): status 101, and the *only* failures were the two golden tests this
  task was in the middle of writing (`593 passed; 2 failed` in the `rexx-exec`
  lib binary; every other binary green). That is what says the split is
  behaviour-preserving on the tree-walker.
* **Step 5** (workspace, after `Op::Binary` landed): status 101, four failures,
  all four the `||''`-chain tests described above.

## The oracle capture: what each stanza's bytes came from

Every program was written to a **fresh empty directory I created**
(`<scratchpad>/task1-binary/probes`), and run with absolute paths through:

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx /abs/path/PROG.rex </dev/null ) >PROG.out 2>PROG.err
```

stdout, stderr and the exit status were captured as three separate descriptors;
nothing was captured as `2>&1`. The tagged block was then assembled as
`rc> $?`, then `out> ` per stdout line, then `err> ` per stderr line, with the
absolute program path rewritten to `/nonexistent/ir-dual-case.rex` (the
`INLINE_PATH` the harness reports a program with no file behind it under). The
case file was assembled by a script that read the `.rex` and the captured block
from disk, so no expected byte was retyped.

Each program was then run on **both** engines of this crate through
`REXX_ENGINE=tree-walker|ir target/debug/rexx-run`, again with three separate
descriptors, and each block was compared to both. All eleven matched the oracle
byte for byte on both engines before the file was written.

The stanza-to-probe map: `p1` concatenation spellings; `p2` comparison families;
`p3` `NUMERIC DIGITS`; `p4` `NUMERIC FUZZ`; `p5` logical truth table; `p7` both
operands checked; `p6` left operand reported; `p12` a failure per family,
trapped; `p11` nesting; `t1` `trace i`; `p14` `trace r`.

**The "before as well as after" measurement the brief requires.** With
`native_shape`'s one line reverted to `is_arithmetic` (the promotion off, HEAD's
behaviour) and everything else in place, `both_engines_agree_on_every_case_file`
passes: status 0, 1 passed. With the promotion on, it passes again. So every row
produces identical bytes on both engines in both configurations, which is what
says a row could ever have told the two apart. `compile.rs` was restored from a
copy taken before the revert, not from git.

**Two oracle-crashing forms were kept out**: no `select; when 1 = 0 then; when
2 = 2 then nop; end`, no `say date('M','0','D')`, no `NUMERIC DIGITS` above 1000
(the highest here is 9).

## Mutation measurements

Three mutations of the promoted path, each re-run with `tests/ir_dual_cases/operators`
moved out of the directory (`memcap 8G cargo test --workspace --no-fail-fast`):

| mutation | caught without the new file? | by |
| --- | --- | --- |
| no `Op::TraceOperator` emitted behind `Op::Binary` | yes | `both_engines_agree_on_every_case_file`; isolated to `tests/ir_dual_cases/calls`, whose `say 'outer got' result` row holds a blank operator |
| `logical_values` checks the right operand before the left | yes | `eval::tests::logical_operators_check_the_left_operand_first` |
| `compare_values` uses `fuzz = 0` instead of the setting | yes | `run::tests::numeric_fuzz_changes_comparison_and_resets_to_0_with_no_expression` |

So the new file is **not** the sole catcher for any of the three. That result is
written into its own header, with the argument for keeping the rows anyway: they
pin oracle bytes, which an engine-against-engine comparison structurally cannot
do and a unit test does not do either (a unit test says what this crate should
print; these say what the C++ interpreter does). That is the same conclusion the
`arithmetic` file's header records about most of its own rows.

Every mutated file was restored from a copy taken before the mutation, and the
restoration was diffed against that copy.

## What I found wrong or ambiguous in the brief, and how I resolved it

1. **`result?` as the collapsed arm's body is a type error.** `eval_node`'s arms
   evaluate to `Result<ObjRef, Failure>` (`ExprKind::Literal(bytes) =>
   Ok(self.literal(bytes))`), and `result?` is an `ObjRef`. The arm ends in
   `result` instead. Same semantics: the frame is popped before the value or the
   failure leaves the arm.

2. **`!is_arithmetic(*op)` in that guard is redundant given the arm order.** The
   brief asks for both the ordering ("place it after the arithmetic arm") and the
   guard. I kept the guard as written: it makes the arm correct independently of
   arm order, and the cost is one comparison on a path that is about to allocate.

3. **The rewrite of `only_the_arithmetic_operators_promote` cannot use an
   `ExprKind::Logical` comma list.** The brief suggests that row. `push_value` --
   the function that asks `native_shape` at all -- is reached from an
   assignment's value and a `SAY`'s expression, and a comma list cannot appear in
   either; it appears only in a condition, and a condition compiles to
   `Op::EvalExpr` whatever its shape. A row for it would therefore pass under
   every implementation of the function it is testing, which is the
   "test that cannot fail" defect. I used `DotVariable` (which the brief also
   names) and `VariableReference` instead, and the test's doc comment states why
   the comma list is not a row.

4. **Two doc comments said "the twelve comparison operators" and listed
   eighteen.** `eval_node`'s comparison arm and `eval_compare`'s doc both carried
   it, while `compare_op`'s own `unreachable!` message already said eighteen. The
   brief says eighteen throughout. `is_comparison` enumerates eighteen. Both
   comments now say eighteen -- a false comment must be corrected rather than
   carried across a move.

5. **`compare_op`'s `unreachable!` message named the wrong caller** after the
   split ("eval_node only dispatches the eighteen comparison operators here").
   Corrected to `apply_binary`. I did not convert it to a `Loud`: it is not this
   task's subject, and it is guarded by `is_comparison` at its one caller.

6. **The brief's file list is incomplete.** `src/lib.rs` (for
   `Loud::binary_operator` and for one of the four emptied tests) and
   `tests/spike.rs` are changed and are not on it. The four emptied tests are the
   substantive omission and are the first concern below.

## Concerns

1. **The promotion moves a deep chain's recursion from `eval` to `compile`, and
   `compile` has no depth limit.** Measured, on this tree:

   | program | tree-walker | compiled engine |
   | --- | --- | --- |
   | `say 'a'` + 100,000 &times; `\|\|''`, promotion **off** | rc 245 (Error 11.1) | rc 245 |
   | `say 'a'` + 100,000 &times; `\|\|''`, promotion **on** | rc 245 | **rc 0, prints `a`** |
   | `say 1` + 100,000 &times; `+0`, promotion **off** | rc 245 | **rc 0, prints `1`** |

   The third row is the important one: **this divergence is pre-existing**, and
   belongs to the arithmetic promotion this task did not touch. What this task
   does is widen it from one operator family to four. D19's evaluation-depth
   limit is `eval`'s own counter, and a promoted expression does not enter
   `eval`; `native_shape` and `push_native` recurse once per operator with no
   equivalent bound. Nothing in this workspace states what the compiled engine
   does at depth, and `tests/ir_dual.rs`'s populations contain nothing deep
   enough to notice.

   I did **not** add a bound to `native_shape`. It is a design decision the plan
   has not made (a bound would also have to be restated independently in
   `corpus_shape_tests::native`, and `corpus/lang/deep_nested_expr.rex` nests
   three thousand terms on purpose), and the corpus-shape module doc cites
   "bounding `native_shape`'s recursion at depth 8" as a *mutation* it catches.
   **This is a decision for the plan, not for this task.**

2. **Four tests changed engine, and that narrows what they say.** The D19 pair
   now states the limit for the tree-walker only. Before this task they stated it
   for the default engine and were, for `||`, telling the truth about it; they
   are pinned now because the same program on the default engine no longer
   recurses at all. This is the visible face of concern 1 and I have written it
   into `depth_limited`'s doc comment rather than only here.

3. **Concatenation and comparison have no failure of their own that I could
   find.** Concatenation renders both operands and cannot raise; a comparison
   drops to a string compare rather than failing when an operand will not
   convert, and `numeric_order`'s only fallible step is a subtraction its own
   comment argues cannot overflow. I probed `1E999999999 > 1`,
   `1E-999999999 < 1` and `9E999999999 = 8E999999999` on the oracle: all three
   answer normally. So the "failing case per family" stanza reaches the two
   through a **failing operand** and says so in its comment. If a failing
   comparison exists, I did not find it.

4. **The clippy green is same-session and over a warm target directory**, which
   `rust/CLAUDE.md` says makes it provisional. A phase-boundary run from a clean
   target directory is still owed by whoever closes the phase.

5. **`Loud::binary_operator` has no test.** Nothing constructs an
   `ExprKind::Binary` around `Operator::Backslash`, so the arm is not reachable
   through a program, and I did not write a test that reaches it by building an
   `Expr` by hand -- that would assert the message rather than the property. It
   is loud rather than `unreachable!`, which is what the brief asks for.

6. **The corpus-shape sweep's `Root::Binary` anti-vacuity row passes**, so the
   corpus does reach a promoted non-arithmetic operator at a value root. I did
   not check *which* program supplies it.

## Commits

One commit, read back with `git log --format='%H %s' -1` after committing:

`5a801e1e98e8b931346b6c3c87960647cddbabcb` -- "Give the other twenty-four binary
operators one op, and one dispatch". 11 files changed, 826 insertions, 162
deletions.

`.superpowers/` is in `.gitignore`, so this report is not part of it.

---

# Fix round 1

Against `task-1-review.md`'s eight findings, on top of `5a801e1e9`. Nothing in
the mechanism changed; every edit is prose except the case-file rows added for
findings 4 and 8.

## A rule that landed between the commit and this round

`rust/CLAUDE.md` gained a rule at `d2efbaf5b` (Moritz, 2026-08-12): **a comment
may not name the size of a set** -- "name the set, never its cardinality" -- and
it cites two phrases this task wrote (`the eighteen comparison operators`, `all
three concatenation spellings`) as examples of what has to go. It supersedes the
reading that a count is safe when its referent is external and immutable, so the
right fix for finding 2 is **not** "twelve" -> "eighteen" but dropping the
number. Measurements keep their numbers; enumerations in code are untouched.

That changed the shape of findings 1 and 2 and widened the round to the prose
this task wrote. What I de-counted: `eval.rs`'s module doc, the four family
predicates and `apply_binary`/`compare_values`/`compare_op`'s docs, `compare_op`'s
`unreachable!` message, `corpus_shape_tests::arithmetic`'s doc, and the
`operators` header and stanza comments. What I left: counts that are
measurements (`Three mutations ... were each re-run`, `Seven of the eighteen
mapping rows survived a mutation run`, `one space and ... four` describing a
program's own text), and pre-existing counts in files this round did not
otherwise touch -- `golden_tests.rs` in particular describes committed streams
row by row ("three ops are the `IF`'s own clause") and is a sweep of its own.

## Per finding

**1 and 7, together.** `eval.rs`'s module doc no longer counts or enumerates the
frame-opening sites; it states the mechanism, which is what was load-bearing. A
new paragraph beside it states finding 7's behaviour change: `eval_node`'s binary
arm binds `apply_binary`'s result and pops before returning it, so an
**operator's** own raise now discards its operands where HEAD propagated through
`?` first, while the `?` on an **operand's** evaluation still skips past. The arm
itself carries a short note pointing at that paragraph. `run.rs`'s I16 test doc
and `rexx-core/src/roots.rs`'s `pop_frame` doc lost the same count; `roots.rs`
keeps the whole argument for why `pop_frame` truncates and why no balance
assertion may be added.

*The argument I satisfied myself with for finding 7*, in the order I checked it:

1. `pop_frame` truncates to the watermark **this arm took itself**
   (`roots.rs`'s own doc), so it can discard only what the arm rooted -- the two
   operands and whatever their evaluation pushed. Nothing below the watermark is
   reachable from it.
2. Pops are idempotent by that same doc, so popping here and being truncated
   again by `step_in_temps_frame` is not a double-free of anything.
3. The one thing the failure path needs beyond that is that the `Failure`
   leaving `apply_binary` carries no root. I read `error.rs`: `Failure` is
   `Loud`, `Raised` and `Exited`, and `/bin/grep -rn "ObjRef" src/error.rs`
   returns three lines -- the import, a doc mention, and `Exited(Option<ObjRef>)`.
   `Raised`'s fields are a `Cow<'static, str>`, two `u16`s, `Vec<Substitution>`
   (bytes), `Option<Vec<u8>>` twice and a `Delivery`. So only `Exited` carries a
   value, and the functions `apply_binary` dispatches to build only `Raised` and
   `Loud` -- readable in those functions, and that is how the doc states it
   rather than as a reachability claim.
4. It is also the better behaviour: the operands are dead at that point, and
   freeing them at the raise rather than at the end of the instruction is
   strictly less to hold.

**2.** `eval.rs:18` and `run.rs`'s `numeric_less` doc lost their "twelve". The
second also named `eval_compare`, corrected to `compare_values`. `eval.rs`'s
`CompareOp has only twelve variants` sentence -- which the round's instructions
said not to sweep up, because it is correct about `rexx-num`'s enum -- I did
de-count, and **this is the one place I went against the instruction**. The
reason it was excluded was that it is true; the new rule says correctness is not
the bar. Its argument is that several operators share one `CompareOp`, and it now
says exactly that, with the shared members still named. Flagged so it can be put
back if the exclusion was about something else.

**3.** All six dangling references corrected: `eval_logical_list`'s back link,
`compare_op`'s doc, the `num`-cache test's comment twice, and `run.rs`'s
`test_case_when` doc twice. `/bin/grep -rn "eval_compare\|eval_logical\b"` over
`src/` and `tests/` now returns only `eval_logical_list`.

`test_case_when`'s doc needed more than a rename. Its reason for not routing
through the operator was "needs a second `eval.rs` visibility bump", and that is
no longer true: `apply_binary` is `pub(crate)` for the driver's sake, so `run.rs`
can call it. The reason that does survive is that only one side of that
comparison is a value -- routing it would mean allocating one to hold
`case_text` so `compare_values` could render it straight back to the bytes
already in hand. That is what the doc says now.

**4.** The logical stanza had no `1 | 1` row, so `|` and `&&` answered
identically on every row it had. The stanza now runs each of the three operators
over all four operand combinations -- twelve rows, oracle-captured -- and the
comment names `say 1 | 1` and `say 1 && 1` as the separating pair rather than
pointing at row numbers.

**5.** `1000000` and `1000100` differ in the fifth significant digit. The comment
now says fifth, and names both numbers rather than counting positions.

**6.** The `arithmetic` header's "different route through `eval.rs`" clause is
gone; what differs per family is where the value is computed, and the sentence
now says that. The family stanza's comment no longer reads as though no family
prints from an op: it says that on the compiled engine every line there comes
from an op behind the operation, and that the harness requires the engines to
agree byte for byte, which is what makes a wrong tag or a wrong position visible.

**8.** I added the missing strict row at the second setting in **both** stanzas
rather than weakening the comments, because the comment claimed a control and the
cheaper fix would have been to stop claiming it. `say 1.0001 == 1` under DIGITS 9
answers `0` as it did under DIGITS 4, and `say 1000000 == 1000001` under FUZZ 0
answers `0` as it did under FUZZ 3 -- so the strict answer is now pinned at both
settings and the control is real.

## Two defects I found in my own prose on re-reading, which the review did not name

Both are the class the round's instructions warn about, and both are in text this
task wrote.

* **A second off-by-one of finding 5's exact shape, one row further out.** The
  concatenation stanza's comment said "the fourth and fifth rows are the same
  expression under one space and under four". They are the third and fourth
  `say` clauses. Rather than renumber -- which rots the next time a row is
  inserted -- the comment now quotes the clause: "`say za zb` is written twice,
  once with one space and once with four". The nesting stanza had the same shape
  ("the third and fourth rows agree") and is now written as the two clauses
  themselves.
* **A wrong term.** My first draft of `compare_op`'s doc said the mapping is
  "onto and not into", which is not what many-to-one means and claims a
  surjectivity I did not check. It says "several operators share one `CompareOp`",
  which is the mechanism the rest of the paragraph explains.

## Oracle capture for the new rows

A fresh empty directory (`<scratchpad>/task1-binary/probes-round1`, `mkdir`ed for
this round), absolute paths, three separate descriptors, no `2>&1`, no
`REWRITE=1`:

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx /abs/path/PROG.rex </dev/null ) >PROG.out 2>PROG.err
```

Three programs -- the revised DIGITS, FUZZ and logical stanzas -- each captured
and then run on both crate engines through `REXX_ENGINE=tree-walker|ir
rexx-run`. All three compared equal across oracle, tree-walker and compiled
engine before being spliced in, and the splice read the `.rex` and the captured
block from disk rather than retyping either.

## Gates, unpiped

| command | status | counts |
| --- | --- | --- |
| `cargo fmt --all --check` | `FMT_STATUS=0` | -- |
| `cargo clippy --workspace --all-targets -- -D warnings` | `CLIPPY_STATUS=0` | -- |
| `memcap 8G cargo test --workspace --no-fail-fast` | `TEST_STATUS=0` | 1460 passed, 0 failed, 4 ignored |

Same warm-target caveat on clippy as before.

## Concern 1 of the original report is closed by the plan, not by me

`aaadb5004` and `3f23ef276` landed while this round was in flight: the depth
divergence is measured at `d0101a39a` (before any of this plan), checked to
700,000 terms, and accepted as a narrow, deliberate licence. Nothing in this
round touches it.

## Commit

`1fdc2a9b107194da8294d6ef45577b63b0450ccf` -- "Stop counting the sets this task
renamed, and pin the rows it claimed". 6 files changed, 118 insertions, 81
deletions. Read back with `git log --format='%H %s' -1`; working tree clean
afterwards.

## Follow-up sweep after the rule was restated

The rule arrived twice -- once in `rust/CLAUDE.md` at `d2efbaf5b`, which is where
I found it mid-round, and once restated by the controller with an explicit list.
The five sites the restatement names (`eval.rs:18`, `run.rs`'s `numeric_less`
doc, the two sentences an earlier round had "corrected" from twelve to eighteen,
and `eval.rs`'s `CompareOp has only twelve variants`) were already de-counted in
`1fdc2a9b1`, which is also where the `eval.rs:1218` exclusion was reversed.

What the restatement's own grep found that the first pass missed, over exactly
the lines these two commits added:

* `apply_binary`'s doc, "the operand handling the three families below do".
* `operators`' comparison stanza, "the two comparison families over the same
  operands" -> "both", and the DIGITS stanza's "the two families differ where it
  sat".
* This report's "all twenty-four operators", "Eleven stanzas", "Four free
  functions now enumerate the families", and one "the eighteen comparison
  operators" describing a doc comment that no longer says it.

Everything else the grep returned is ordinary prose rather than a set's size --
"one space", "the two engines", "two operands", "one implementation", "one row",
"the one dispatch both engines enter" -- and stands.

**Counts in `eval.rs` I did not touch, and why.** The sweep's stated scope is
this task's own prose, and `eval.rs` is full of pre-existing counts outside it --
of the arithmetic set, of the bare-symbol reads, of `Ended`'s arms, of the
keywords a comma list serves. Several are inconsistent with their de-counted
neighbours now; `arith_general`'s `unreachable!` message counting a set a few
lines from `is_arithmetic`'s de-counted doc is the sharpest. **The sweep is
bigger than this round** -- `golden_tests.rs` alone describes committed streams
row by row -- and I have not started it. I am deliberately not listing the sites
here: a list transcribed from a survey without opening every item has been wrong
every time this project has tried one, and the conclusion is what carries the
information anyway.

## Commits for the round

* `1fdc2a9b107194da8294d6ef45577b63b0450ccf` -- the eight findings. 6 files,
  +118/-81.
* `a8d36deb8b1cc3532257a6ced35f1e39d77d6367` -- the follow-up sweep above.
  2 files, +3/-3.

Both read back with `git log --format='%H %s' -1` after committing; working tree
clean after each.

---

# Fix round 2

Against `task-1-rereview.md`'s four new false statements, plus one scoped
inconsistency. Nothing in the mechanism changed.

## NF-1: the deleted numeral was the quantifier's bound

`compare_op`'s doc. Round 1 turned "the **four** backslash-negated forms invert
their positive counterpart's sense rather than getting a `CompareOp` of their
own" into "the backslash-negated forms invert ...", which widened the subject
from the four the colon then names to every backslash-negated operator -- and
that is false for `\==` (which has `StrictNotEqual` to itself) and contradicts
the same sentence's first clause for `\=` (already described there as sharing
`NotEqual` with `<>` and `><`).

The repair names the set rather than counting it: "`\>`/`\<` with their strict
siblings `\>>`/`\<<` invert ...", followed by the two exclusions stated
explicitly, because they are what a reader would otherwise assume in.

**I swept my other de-counts for the same shape** -- a numeral doing quantifier
duty in front of a plural, where removing it widens a claim instead of
shortening a sentence. Every other one turned out to sit in front of a noun
phrase whose members are named in the same breath (`is_concatenation`'s "`||`,
the abuttal ..., and the blank ...", `compare_values`' D15 quotation of both
families, the case-file headers' "every concatenation spelling"), or in front of
a definition bounded by the function it names (`is_arithmetic`, `is_logical`).
`compare_op` was the only one where the numeral was load-bearing, and it was
load-bearing because the sentence had **two** subjects and the number was the
only thing separating them.

## NF-2: measured, and the inference was wrong in the useful direction

I mutated `logical_values` so `Operator::Or` computes `left_bool !=
right_bool`, and ran `memcap 8G cargo test --workspace --no-fail-fast` twice.

| run | status | failures |
| --- | --- | --- |
| mutation, `operators` in place | 101 | `both_engines_agree_on_every_case_file`, `the_exempt_set_matches_the_current_blocked_rows` |
| mutation, `operators` held out | 101 | `the_exempt_set_matches_the_current_blocked_rows` |

**So the file is not the sole catcher.** `assertions.rs`'s
`the_exempt_set_matches_the_current_blocked_rows` reddens without it: it runs
every assertion row extracted from `ootest/` and requires the not-passing set to
equal the committed exempt list exactly, and under the mutation the not-passing
count rises from 35 to 42 -- seven extracted ooTest rows stop passing.

That instrument is outside the four the re-review enumerated (unit tests, Rust
`|` literals, corpus programs, engine-against-engine sweeps), which is why the
inference missed it: it is neither a hand-written witness nor an engine
comparison, it is the C++ suite's own rows run against this crate.

The header now records four mutations rather than three, names this one and what
caught it, and keeps its headline claim -- which the measurement supports rather
than undermines. It also says plainly that the row added to close the stanza's
blind spot did not make the file a sole catcher even for the mutation it closes.

`eval.rs` was restored from a copy taken before the mutation and diffed against
it; `operators` likewise; both rebuilt afterwards.

## NF-3: the list is deleted, the conclusion kept

The report's "Three counts in `eval.rs` I did not touch" was an under-count -- at
least `eval_arithmetic`'s own doc and a second sentence in `small_int_arith`'s
were missing from it, both of the same arithmetic set. That paragraph now states
the conclusion (the file is full of pre-existing counts, the sweep is bigger than
this round, I have not started it) and deliberately lists nothing, because a list
transcribed from a survey without opening every item has been wrong every time
this project has tried one.

## NF-4: the counterfactual was false in one direction

The logical stanza's comment said "without both, an implementation computing one
as the other answers every row identically". Its own history disproves that:
before this round the stanza had `say 1 && 1` and no `say 1 | 1`, and `Xor`
computed as `Or` answered `1` there and failed. The comment now says which row
catches which direction and that it takes both rows to cover both -- which is
what is actually true, and it is also why the file needed the addition.

## The scoped inconsistency

`tests/ir_dual_cases/arithmetic`'s header and stanza comments are de-counted, so
the two case files are written to the same rule: "in four kinds" -> "by kind",
"all seven operators" (twice) -> "every operator", and the two dividing-operator
comments now name `/`, `%` and `//` rather than counting them. Its mutation
measurement keeps its numbers. Nothing outside that file.

While rewording one of those, I removed a uniqueness claim rather than sharpening
it: the old comment called 42.3 "the only one of the three that comes from
`rexx-num` rather than from converting an operand", and my first replacement
tightened that to "the raise ... that comes from `rexx-num`". Both are wrong --
`Number::pow` raises 26.8 from `rexx-num` too, for an exponent that parses but is
not whole (`eval_arithmetic`'s own doc). The comment now just says what the row
does: a zero divisor raises 42.3 from `rexx-num` rather than from converting an
operand.

## Two more found on the round's own re-read

* `compare_op`'s repair reintroduced a count in the very sentence removing one
  ("`\=` is one of the three that share `NotEqual` above"), fixed to "one of the
  operators sharing `NotEqual` above" before the gates.
* The `operators` header briefly had two consecutive paragraphs opening "So",
  the second of them pre-existing; the new one is reworded.

## Gates, unpiped

| command | status | counts |
| --- | --- | --- |
| `cargo fmt --all --check` | `FMT_STATUS=0` | -- |
| `cargo clippy --workspace --all-targets -- -D warnings` | `CLIPPY_STATUS=0` | -- |
| `memcap 8G cargo test --workspace --no-fail-fast` | `TEST_STATUS=0` | 1460 passed, 0 failed, 4 ignored |

## Commit

`8d7f086ecd275c5a1621128a3626ed8dcd6b8f02` -- "Put back the bound a deleted
numeral was carrying, and measure the fourth mutation". 3 files changed, 34
insertions, 24 deletions. Read back with `git log --format='%H %s' -1`; working
tree clean afterwards.
