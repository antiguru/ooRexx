# Task 2 report: the prefix operators as a native op

## What changed, and why

### `eval.rs` -- the operand/operator split

`Interp::eval_prefix` ran one operand prologue (`push_frame`, evaluate the
operand, `push_temp`), then the operator's own `match op`, then `pop_frame`. It
is split at that seam:

* `eval_prefix` keeps the prologue and calls the new dispatch.
* `pub(crate) fn apply_prefix(&mut self, op: PrefixOp, value: ObjRef) -> Result<ObjRef, Failure>`
  keeps the `match op` body verbatim.

Both measured facts in the old doc comment moved to `apply_prefix`, where the
code they are about now lives: `numeric digits 1 ; say -12345` giving `-1E+4`
(so `+`/`-` are `0 + x`/`0 - x` and not a sign flip), and `say \'abc'` being
34.901 rather than 41.1 (so `\` is a text check and never a numeric one).

The rooting contract is stated on `apply_prefix` in the wording Task 1 gave
`concat_values`: **the operand must already be rooted by the caller**, because
both arms allocate the value they answer with. The driver's arm cites it and
says what supplies it (the register).

### The error-path shape of `eval_prefix`, and why

`eval_prefix` now binds `apply_prefix`'s `Result` and pops unconditionally:

```rust
let result = self.apply_prefix(op, value);
self.roots.pop_frame(frame);
result
```

**This is a change of behaviour on the failure path and it was chosen rather
than inherited.** Before the split, every failure inside the operator's own work
left through `?` or an early `return Err(...)`, so the `pop_frame` was skipped;
now an operator's own raise discards its operand. That is exactly the shape
Task 1 gave `eval_node`'s collapsed binary arm, and the reasoning is the one
`eval.rs`'s module doc already states for that arm:

* `pop_frame` truncates to a watermark this function took itself, so it can only
  discard what this function rooted.
* Nothing the failure carries away is a root -- `Failure::Exited` is the variant
  carrying an `ObjRef`, and `apply_prefix`'s arms build only `Raised`
  (`Raised::nonnumeric` through `arith_operand`, `Raised::from` on an
  `ArithError`, `Raised::not_logical`).
* The `?` on the *operand's* evaluation still skips past the pop, and that is
  equally safe, for the reason the module doc gives: `step_in_temps_frame` pops
  unconditionally with an outer watermark.

The alternative -- keeping the skip by propagating with `?` out of
`apply_prefix` into `eval_prefix` -- would have made the two operator families
differ on the failure path for no reason anyone could state at either site. So
the two now read the same, and the module doc's paragraph about the failure path
was widened from naming `eval_node`'s binary arm alone to naming `eval_prefix`
beside it, and from "the functions `apply_binary` dispatches to" to "neither the
functions `apply_binary` dispatches to nor `apply_prefix`'s own arms".

### `trace.rs` -- `echo_prefix_op`

`trace_intermediate`'s `ExprKind::Prefix` arm open-coded the spelling and called
`trace_prefix_op`. It now calls a new `Interp::echo_prefix_op(op, value)`, which
is the shape `echo_operator` has and exists for the same reason: the compiled
stream's `Op::TracePrefix` emits the same line from a register, and two
emissions that could disagree about the tag or the indent would be a divergence
only an exact stderr comparison could see.

### `rexx-parse` -- `PrefixOp::spelling()`

**This file is not in the brief's list, and the deviation is deliberate.**
`golden.rs` renders an operator by its spelling rather than its `Debug` name --
its own comment on `Op::Arith` says why: "a golden that read `Plus` could not
say whether the tag would" -- and `echo_prefix_op` needs the same three bytes.
Rather than a second copy of the match (there was already one in
`trace_intermediate` and one in `ast.rs`'s test-only `shape()`, which spells
them `u+`/`u-`/`u\` to disambiguate from the binary operators and so cannot
serve), `PrefixOp` gains `spelling()` next to its own type in `ast.rs`, which
is where `Operator::spelling()` sits relative to `Operator` (`token.rs`) -- the
symmetry is method-beside-type, not the two methods beside each other. The echo
and the renderer both read it.

Its doc cites `scanner.rs:672` (`b'\\' | 0xAA | 0xAC` all scan to one
`Operator::Backslash`) and `expr.rs:801` (`message_subterm` builds
`PrefixOp::Not` from that token), which is what makes "canonical" a real claim
for `\` and not decoration.

### `ir/mod.rs` -- the two ops

* `Op::Prefix { op: PrefixOp, src: u16, dst: u16 }`
* `Op::TracePrefix { op: PrefixOp, src: u16 }`

`const _: () = assert!(size_of::<Op>() == 12)` still holds unchanged -- the
build compiles, which is what that assertion is. Nothing was relaxed.

`Op::TracePrefix` is a separate variant from `Op::TraceOperator` rather than a
reuse, and the doc says why at the site: a prefix operator's line is `>P>`
(`trace_prefix_op`) and a binary operator's is `>O>` (`trace_operator`), which
are different `TracePrefix` constants into the same C++ renderer -- so an echo
that reused the binary op would put the right value on the wrong line. That is
also why it carries a `PrefixOp` where the other carries an `Operator`.

### `ir/compile.rs`

* `native_shape` gains `ExprKind::Prefix { operand, .. } => native_shape(operand)`.
  The arm is shorter than the `Binary` one because `PrefixOp` has no member
  outside the promotable set, and the comment says that rather than leaving it
  looking like an omission.
* `push_native` gains a `Prefix` arm: the operand into `dst`, then
  `Op::Prefix { op, src: dst, dst }`, then `Op::TracePrefix { op, src: dst }`.
  No register is allocated -- the operand is read before `dst` is written.
* `assert_prefix_echoes_follow_their_op` is new, in
  `assert_operator_echoes_follow_their_op`'s shape, checking position, register
  **and** operator; called beside the others in `compile`.
* `assert_region_ops_name_their_clause`'s exhaustive `match` gained both new ops
  in its `None` arm (neither carries an instruction index).
* `push_native`'s own doc comment gained the prefix case.

### `ir/drive.rs`

`Op::Prefix`'s arm mirrors `Op::Binary`'s: the two `debug_assert!`s on the
registers, the source read before the destination is written,
`self.apply_prefix(*op, value)`, `break 'region Err(failure)` on failure,
`self.roots.set_temp(registers, *dst as usize, value)`. `Op::TracePrefix`'s arm
mirrors `Op::TraceOperator`'s and calls `echo_prefix_op`. Both got their
`Loud::op_not_driven` arms in the outer `match`.

### `ir/golden.rs`, `ir/golden_tests.rs`, `ir/corpus_shape_tests.rs`

`render` gained both ops, the operator by spelling. `Root` gained `Prefix`, and
`Root::of`, `root_of` and `native` each restate the widened set independently
(`root_of`/`native` from the parse tree, never by asking `compile`).

`Root::Prefix` is in the anti-vacuity control list, and that was **measured, not
assumed**: the row was added and the sweep run, and
`every_corpus_body_compiles_the_minimum_promotion_set_to_its_own_ops` passed, so
the corpus does contain a promoted clause whose value is a prefix. (`Root::CallExpr`
is absent from that list, which is the precedent for what I would have done had
it failed.)

### `tests/ir_dual_cases/operators`

Seven stanzas appended (`git diff | grep -c '^+program$'` -> 7), and the header
widened -- see the oracle section below.

## The depth divergence

Nothing to report. No test broke, and no test was re-pinned to one engine.
Promoting the prefix operators widens the set of expressions that reach the
divergence exactly as the plan says, but the workspace was green on the first
run after the promotion landed, so there was no test to pin.

## Commit

`1ccb81199084372bd6d9e4dd2559dcccc227d3a0` -- "Give the prefix operators one op,
and a line of their own". Ten paths staged by name, no `git add -A`; the tree is
clean afterwards.

## Gate commands, unpiped, on the committed tree

Run from `rust/`, on the exact bytes of that commit.

| command | exit | result |
| --- | --- | --- |
| `cargo fmt --all --check` | 0 | -- |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | -- |
| `memcap 8G cargo test --workspace --no-fail-fast` | 0 | 82 binaries, 1461 passed, 0 failed, 4 ignored |

The TDD steps, with their run counts read rather than their statuses:

* Step 2, before the ops existed: `cargo test -p rexx-exec a_prefix_operator_compiles`
  exited 101 and reported `0 passed; 1 failed; 0 ignored; 595 filtered out` --
  so the test ran rather than being filtered away. It failed on the first row
  (`za = +zb`), rendering `EvalExpr index=0 slot=0 dst=0` where the expectation
  is `Prefix op=+ src=0 dst=0` behind a `Load`.
* After Step 4: the same command, `--lib`, `1 passed; 0 failed`.

**A note on the filter:** `cargo test -p rexx-exec a_prefix_operator_compiles`
without `--lib` runs every test binary in the crate, and the tail of its output
is a run of `0 passed; 0 filtered out`-style lines from the integration
binaries; the `1 passed` line is in the middle. `--lib` was used afterwards for
that reason, not because the filter was failing to match.

## Mutations run, and what they say

Each was applied to a copy-backed file and restored from that copy (never from
git), and both files were `diff`ed against their backups afterwards -- identical.

1. **The prefix echo emitted in front of its op** (`push_native` order swapped).
   `assert_prefix_echoes_follow_their_op` fires with its own message: "the prefix
   echo at 3 does not follow the operation whose operator and register it names".
   So the new assertion is reached and can fail on position.
2. **The prefix echo carrying the wrong operator** (`Op::TracePrefix { op:
   PrefixOp::Plus, .. }` regardless of the node). The same assertion fires, at
   index 4. So it can fail on the tag as well, which is the half a pure ordering
   check would miss.
3. **`echo_prefix_op` emitting `>O>` instead of `>P>`** (`trace_operator` in
   place of `trace_prefix_op`). Whole workspace, `--no-fail-fast`, exit 101.
   Caught by three tests: `both_engines_agree_on_every_case_file` (this task's
   `trace i` stanza), `prefix_operators_covers_plus_and_backslash` and
   `controlled_loop_covers_the_control_variables_own_value_lines`.
   **So the new stanza is not a sole catcher for that mutation.** The two
   `trace_oracle` witnesses run `Invocation::none()`, whose engine is
   `Engine::DEFAULT` -- `Engine::Ir` (`invocation.rs:160`) -- so they pin those
   bytes on the **compiled** engine, and what the new stanza adds is the
   tree-walker's own bytes and the two engines' agreement on them. An earlier
   version of this paragraph and of the case file's header said the reverse; see
   the fix-round section at the end.

## Oracle captures

Every expected byte in the seven new stanzas came from
`/home/moritz/dev/repos/ooRexx/build/bin/rexx`, run under the wrapper, from a
fresh empty `mktemp -d` directory holding nothing but the program, with stdout,
stderr and the exit status taken as three separate descriptors and never
`2>&1`. The capture script is
`<scratchpad>/task2-prefix/capture.sh`; the shape of each run is

```
run="$(mktemp -d ...)"; cp "$prog" "$run/case.rex"
( cd "$run" && ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
    /home/moritz/dev/repos/ooRexx/build/bin/rexx "$run/case.rex" </dev/null \
    >"$run/o" 2>"$run/e" )
```

with the run directory's own path rewritten to `/nonexistent/ir-dual-case.rex`,
which is `ir_dual.rs`'s `INLINE_PATH` and what a raised condition's middle line
prints. `REWRITE` was never set; the harness refuses to run under it.

The stanzas, and what each one's bytes are:

1. **Each prefix operator over a read and over a literal.** `+za`/`-za`/`\zb`,
   `+5`/`-5`/`\1`, and `-0`, `+'7'`, `-'7.50'`. rc 0.
   The comment's claim that `-` is the subtraction rather than a sign flip is a
   measurement and not an inference: a separate control probe
   (`<scratchpad>/task2-prefix/progs/ctl.rex`) ran `say 0 - 0`, `say 0 - 7.50`,
   `say 0 + 7` and, at `numeric digits 1`, `say 0 - 12345`, and the oracle gave
   `0`, `-7.50`, `7`, `-1E+4` -- the same four answers the prefix rows get. The
   control probe is not itself a stanza.
2. **A prefix over something other than a plain term.** `- -za`, `-(-za)`,
   `\\zb`, `-(za + 1)`, `-(za || za)`, `\(za = 3)`, `\(za > 9)`, `-2 ** 2`,
   `-(2 ** 2)`, `-za + 1`, `\zb & zb`. rc 0.
   `-2 ** 2` answers 4 and `-(2 ** 2)` answers -4 in the same stanza, which is
   the pair that pins "a prefix takes a whole message subterm before any dyadic
   operator is considered" to the parenthesis rather than to a coincidence;
   `-za + 1` (-2) and `-(za + 1)` (-4) are the second such pair.
3. **`NUMERIC DIGITS`.** `-12345` and `+12345` at DIGITS 1 (`-1E+4`, `1E+4`),
   `\0` at the same setting as the control that says the text check reads no
   setting, then `-12345` at DIGITS 9. rc 0.
4. **`\'abc'`**: rc 222, 34.901 with `"abc"` as the substitution.
5. **`-'abc'`**: rc 215, 41.1 with `("abc")`. Written beside row 4 deliberately
   -- the same operand under the two operators, since either row alone is
   satisfied by an implementation that gave one number to both.
6. **`trace i`**, the `>P>` positions. `zb = -za`, `say \(za = 3)`,
   `say -(za + 1)`, `say - -za`. The last is the one that pins post-order: two
   `>P>   "-"` lines, inner first (`"-2"` then `"2"`). rc 0.
7. **`trace r`**, the other direction: no `>P>` line at all. `echo_prefix_op`
   has its own `tracing_intermediates()` early return, which is a different call
   site from the one the existing `>O>` `trace r` row tests.

The header was widened to say the file's subject now includes the prefix
operators: the Shapes, Settings, Failure and Trace paragraphs each gained their
prefix sentence. Two existing sentences were **corrected rather than left**,
because this task falsified them:

* "The raise an operator here makes of its own is the logical family's 34.901"
  -- no longer the only one, now that `\`'s 34.901 and `-`'s 41.1 are rows. It
  is scoped to the binary operators and the prefix pair is stated beside it.
* "Four mutations of the promoted path were each re-run..." -- that measurement
  was taken on the binary path, and the sentence now says so, so that it is not
  read as a claim about the rows this task added.

## What I found wrong or ambiguous in the brief, and how I resolved it

* **The brief's `eval_prefix` skeleton ends `Ok(result)`.** With the work moved
  into `apply_prefix`, which returns a `Result`, that would not compile; the
  line is `result`. Taken as the shape being indicated rather than the literal
  text, which is also what makes the error-path decision above the one the brief
  wanted -- it is Task 1's shape.
* **The brief does not list `rexx-parse` among the files.** Adding
  `PrefixOp::spelling()` there is a deviation; the reasoning is in the
  `rexx-parse` section above. Nothing else in that crate changed.
* **The brief's golden test body is `assert_eq!` against the full rendered
  stream of `za = -zb` alone**, while its own doc comment says "the echo carries
  the operator: `\` and `-` trace different lines from one value". A single row
  cannot say that -- a renderer with `-` hardcoded passes it -- so the test is
  three rows differing in the operator alone, `+`, `-` and `\`, over the same
  full stream. The doc comment is the brief's, with a paragraph added saying why
  the rows are together.
* **`assert_region_ops_name_their_clause` is a third exhaustive `match` over
  `Op` that the brief does not mention.** The compiler named it; both new ops
  went into its `None` arm.

## Things I was unsure of

* **Whether `PrefixOp::spelling()` belongs in `rexx-parse` or as a free function
  in `rexx-exec`.** I chose the former for symmetry with `Operator::spelling()`
  and because it leaves one home for the three bytes; the cost is a public API
  addition to another crate, which a reviewer may prefer reversed. Reversing it
  is local: `golden.rs` and `echo_prefix_op` are the only two readers.
* **Whether the `trace r` stanza earns its place.** It is not required by the
  brief. I kept it because `echo_prefix_op`'s gate is a separate call site from
  `echo_operator`'s and the file already argues for having both directions.
  Checked in fix round 1 rather than left argued: `ChunkTrace` carries only
  `clauses` and `labels` (`trace.rs:252`-`256`), and `compile` reads it at one
  site, the clause echo (`compile.rs:1011`). So `Op::TracePrefix` is compiled
  into the stream whatever the setting, and `echo_prefix_op`'s own early return
  is the only thing suppressing the line -- on both engines. The row exercises a
  live gate.
* **`push_native`'s prefix arm takes no register and so never calls
  `registers.mark()`/`release()`.** That is right for a single operand landing
  in `dst`, and the golden test's stream for `za = -zb` uses register 0 alone --
  but I have not written a witness for a *deep* prefix chain's register count
  specifically, beyond what the corpus sweep sees.

---

## Fix round 1

Commit `87f0e0d323fb694de76403555f67ac77aad47e96` -- "Name the engine the prefix
trace witness actually runs on". Three paths: `rexx-exec/src/lib.rs`,
`rexx-exec/tests/ir_dual_cases/operators`, `rexx-parse/src/ast.rs`.

Review: `.superpowers/sdd/2026-08-12-expression-promotion/task-2-review.md`.
Spec compliance passed; every finding was in prose except finding 4.

Gates, from `rust/`, each exit status read unpiped on the committed bytes:
`cargo fmt --all --check` 0; `cargo clippy --workspace --all-targets -- -D
warnings` 0; `memcap 8G cargo test --workspace --no-fail-fast` 0, 82 binaries,
1461 passed, 0 failed, 4 ignored -- unchanged from the first commit, as a round
that changed no behaviour should be.

### Finding 1 -- the engine claim, which I had backwards

**The review is right and I verified it before acting.** `Invocation::none()`
sets `engine: Engine::DEFAULT` (`invocation.rs:204`) and
`pub const DEFAULT: Engine = Engine::Ir` (`invocation.rs:160`, checked here
rather than copied from the review, which cited 203 for the field). So
`trace_oracle`'s `check_witness` runs the **compiled** engine, and the case
file's header had the credit exactly reversed.

**How I got it wrong is worth recording, because it is a shape this project has
measured before.** I grepped for `engine:` across `lib.rs`/`run.rs`/`plan.rs`,
found `engine: Engine::TreeWalker` in `lib.rs`, and concluded that was the
default a test gets. That line is inside `Interp::new()`, a constructor for
this crate's own unit tests, and its doc comment says so explicitly:
"**engine**, which is [`Engine::DEFAULT`] and reaches an `Interp` through
`execute`". The premise was true, the file I needed (`invocation.rs`) was never
opened, and the wrong conclusion then travelled into a committed comment.

Corrected in the case file's header, which now names `Invocation::none()`,
`Engine::DEFAULT` and `Engine::Ir` so the claim is checkable at the sentence,
and says what the row actually adds: the tree-walker's own bytes and the two
engines' agreement, which `render_both_engines` asserts before the expected
block is compared. The report's mutation-3 paragraph is corrected the same way.

**I checked whether the belief reached anywhere else.**
`git show --format="" -U0 1ccb81199 | grep -i "engine\|tree-walker\|witness"`
over the added lines gives four hits: the two "both engines enter" sentences on
`apply_prefix` and `echo_prefix_op`, which are true and unaffected, and the two
header lines corrected here. Nothing else in the commit rests on it.

### Finding 2 -- a comment citing rows that cannot discriminate

Correct: `-'7.50'` keeps the trailing zero under a sign flip too, so that row
shows the two candidate implementations agreeing rather than separating. The
row that separates them is `-12345` at `numeric digits 1`.

Stanza 1's comment now says what its rows do -- the prefix answers and the
`0 + x`/`0 - x` answers agree -- and says plainly that **these rows cannot tell
a sign flip from the subtraction**, pointing at the `NUMERIC DIGITS` stanza that
can. That stanza's comment now carries the discrimination, beside the rows that
make it.

### Finding 3 -- a control row that cannot fail

Correct, and it cannot be repaired in place: `\`'s operand set is exactly `"0"`
and `"1"`, and no precision changes either one's value or rendering, so no `\`
row under any `NUMERIC DIGITS` can answer differently. **The `say \0` row and
both sentences claiming it as a control are deleted**, in the header and in the
stanza. What shows `\` is a text check is the `\'abc'` -> 34.901 stanza beside
`-'abc'` -> 41.1, which the file already has and the Failure paragraph already
cites.

### Finding 4 -- the deviation's payoff collected

`lib.rs`'s `form_name` now reads `op.spelling()` in its `ExprKind::Prefix` arm,
the way the `ExprKind::Binary` arm below it already did, and `PrefixOp` comes
out of `lib.rs`'s module-level import (its only other uses are in that file's
own `#[cfg(test)]` module, which imports it separately). The comment above the
pair said "Both spellings are `&'static`", which was about the `match` that is
now gone; it says instead that each arm asks its own operator type.

**This is the enumeration-from-a-survey defect.** I listed the copies of the
spelling match from what I had already read rather than from a search of the
tree, and missed the one that mattered most -- the one whose neighbour was
already calling the method.

### Finding 5 -- a discrimination no row exercises

Correct. **Dropped rather than added, and the reason is in the file**: what
would exercise it is a `\` written `0xAA` or `0xAC`, and I re-measured the
oracle myself rather than restating the review's number -- `trace i` with all
three spellings over the same operand prints `>P>   "\"` three times. But
neither byte is valid UTF-8; both the program *and* the `*-*` echo in its
expected block would carry one; and `datadriven` reads case files through
`fs::read_to_string` (`datadriven-0.9.0/src/lib.rs:483`). So no such row can
live in this file, and the header now says that instead of implying a
discrimination the rows do not make. The abuttal/blank half of the sentence is
kept and is true for the reason the review gives.

### Finding 6 -- controls promoted from probes to rows

`say 0 - 0`, `say 0 + 7`, `say 0 - 7.50` are now rows of stanza 1, and
`say 0 - 12345` is a row of the `NUMERIC DIGITS` stanza at both settings. Both
stanzas were re-captured from the oracle in full rather than edited by hand --
fresh `mktemp -d`, absolute paths, three descriptors, no `REWRITE`. The header
no longer cites a probe that is not in the file.

### Findings 7 and 8 -- nits

The report's "beside `Operator::spelling()`" is corrected: the symmetry is
method-beside-type, and `Operator::spelling` is in `token.rs` while the new one
is in `ast.rs` next to `PrefixOp`. `PrefixOp::spelling`'s doc now reads "in the
case that has it".

### Nothing else changed

No behaviour changed in this round except `form_name`'s message construction,
which produces the identical bytes. The two re-captured stanzas' expected blocks
are new oracle output for programs with rows added; every previously recorded
line in them is unchanged.
