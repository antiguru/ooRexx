STATUS: DONE

# Review of Task 7, commit 3a9d6446 (eval grows terms, arithmetic and concatenation; Raised propagates)

Reviewer: continuing from Task 6's fix round on this crate family. Not the
author. Read-only review, plus independent oracle re-verification and two
scratch mutations.

## Inputs

`task-7-brief.md`, `task-7-report.md`, the design spec's "Expression
evaluation" and "Errors, and the reporting subsystem" sections, `rexx-core`'s
`RootSet` (`roots.rs`), and the full content of the five changed files at HEAD
(`8d9791e9`): `src/error.rs`, `src/eval.rs`, `src/lib.rs`, `src/value.rs`,
`tests/spike.rs`. Confirmed with `git log 3a9d6446..HEAD -- <those 5 paths>`
that nothing has touched them since Task 7 landed, so reading HEAD is reading
Task 7's own diff. `src/plan.rs` was correctly left out of the commit (a
concurrent, unrelated change flagged in the report) and is not this review's.

Independent verification, not just reading:

* Re-ran every oracle transcript the report cites (`1/0`, `1//0`, `'abc'+1`,
  `2**'x'`, `2**2.5`, `'y'**2`, `'y'**'x'`, `\'abc'`, DIGITS-1 `+12345`/
  `-12345`) against `~/dev/repos/ooRexx/build/bin/rexx` 5.3.0 under
  `( ulimit -v 1048576; ... )`. All ten match the report exactly, byte for
  byte including the rc.
* `cargo test -p rexx-exec` (debug): 38 unit + 10 integration + 2 doctests,
  all green — the same 38 the report's post-`sub_code`-swap count states.
* `cargo test -p rexx-exec --release --lib`: 38/38 green.
* `cargo clippy -p rexx-exec --all-targets -- -D warnings`: clean.
* `cargo fmt -p rexx-exec -- --check`: clean.
* `cargo build --workspace --exclude rexx-parse`: clean.
* Two scratch mutations (git-archive copy, not the working tree — see
  "Mutation testing" below).

## Verdict 1: spec compliance — PASS

Read against "Expression evaluation" and "Errors, and the reporting
subsystem" line by line.

* **Term forms.** `Literal`, `Constant` (upcased spelling — `1e5` → `1E5`,
  tested and oracle-matched), `Variable`/`Stem` sharing one slot-read arm
  (D15a), `Compound` via `compound_parts`/`tail_key`/`stem_get`, `DotVariable`
  restricted to the three admissible names with everything else falling
  through to the loud path. Matches the spec's "Variable, Stem and Compound
  go through the plan's slots... DotVariable is .nil, .true and .false only."
* **Arithmetic**, all seven operators, through `rexx-num` under the
  activation's *current* settings (`self.activation().settings.digits()`/
  `.form()` read fresh on every call, never cached) — matches D15's rule that
  the *value* is rendering-fixed at creation but the operation runs under
  whatever DIGITS/FORM are current now.
* **Concatenation**, all three forms sharing one `concat` helper keyed by
  separator — `Abuttal`/`||` join with `b""`, `Blank` with `b" "`. Verified by
  mutation (below) that treating `Abuttal` as `Blank` is caught, which
  matters because that exact substitution is criterion 6's own listed
  mutation for the 4a gate.
* **Prefix**, `+`/`-` routed through real arithmetic (`Number::zero().add`/
  `.sub`, not a sign flip — measured DIGITS-1 case re-verified above), `\` as
  a text-only logical check, never numeric.
* **The `**` asymmetry is deliberately implemented, not accidental.** Base
  always goes through `arith_operand` first, unconditionally, before the
  branch on `op == Operator::Power` — so `'y' ** 'x'` necessarily takes the
  same 41.1 path as `'y' ** 2` by construction, not by a case anyone had to
  get right twice. The exponent's own failure (parses-but-not-whole, or
  doesn't parse at all) is unified under 26.8 via `Raised::power_exponent_not_
  whole`, whose doc comment states the asymmetry is "the fact being
  reproduced, not an implementation shortcut" and cites the same oracle pair
  the report does. Re-verified against the real oracle (see Inputs) and
  against a mutation that swaps the exponent's failure to route through
  `nonnumeric` instead — caught (below).
* **The `Raised`/Task-12 boundary held.** `error.rs`'s `Raised` carries
  exactly `condition`/`number`/`sub`/`additional`. No message catalogue, no
  two-line stderr format, no clause echo. `execute()`'s new `Failure::Raised`
  arm (`lib.rs:1077-1086`) prints `"raised {condition} {number}.{sub} (not
  yet rendered: Task 12), additional {additional:?}"` and returns
  `NOT_IMPLEMENTED_EXIT` (120) — **not** `256 - number`. 120 sits outside the
  157–253 band a real Rexx error can produce, so a differential run cannot
  mistake this for a coincidental pass, which is exactly the property
  "Failing loudly" demands of an out-of-band code. Confirmed this is a
  deliberate reuse of the existing not-implemented code, not a partial
  implementation of Task 12's mapping — the comment says so and the code
  matches.
* **Every unimplemented `ExprKind` fails loudly.** Checked `Operator`'s full
  31-variant list (`token.rs:197`) against `eval_node`'s match: the 7
  arithmetic + 3 concatenation operators are named explicitly; the other 21
  (all comparison, strict, logical, `Xor`, `Backslash`) fall through the
  outer `other => Err(Loud::expression(other).into())` arm, correctly
  deferring to Task 8. `Call`, `QualifiedCall`, `Message`, `ClassResolver`,
  `List`, `VariableReference`, and any `DotVariable` beyond the three, all
  take the same path. This matches the spec's 9-in-scope/6-loud split for
  4a's full `ExprKind` set (Task 7 implements the arithmetic/concatenation
  slice of the 9; Task 8 the rest).
* **The temps discipline holds on every success path I traced.** In
  `eval_prefix`, `eval_arithmetic` and `concat`, every value that survives
  past a further allocation is `push_temp`'d immediately after the `eval`
  call that produced it, before anything that could allocate runs on it.
  Traced `to_number`/`to_text`/`arith_operand` in `value.rs` and confirmed
  none of them call `self.heap.alloc*` — only `text()`/`number()` do, and
  both are called last, with the fresh `ObjRef` returned to the caller before
  anything else can allocate. `Heap::alloc_with` (`rexx-core/src/heap.rs:192`)
  takes no `&RootSet` and cannot itself trigger a collection, so the
  "collect-on-every-allocation" gate (criterion 4, not yet built) is the only
  thing that will ever exercise this at all, and everything traced here is
  ready for it on the paths that succeed. One qualification below.
* **The `u32` narrowing is saturated with a stated reason, not a silent
  clamp.** `saturate_digits` (`eval.rs:311-323`) has a five-line doc comment
  explaining why: `u32::MAX` significant figures would exhaust memory in
  `rexx-num` long before the clamp could matter, so no corpus or realistic
  program reaches it. The call site (`eval_prefix`, `eval_arithmetic`) names
  the function rather than inlining `.unwrap_or(u32::MAX)`, so the reasoning
  travels with every use.

## Verdict 2: code quality — PASS, with one Minor

**Minor: `pop_frame` is skipped on every error path in `eval_prefix`,
`eval_arithmetic` and `concat`, which is the exact defect class this
codebase already named and fixed once, in a commit that predates Task 7.**

All three functions open a `RootSet` frame and use `?` (or an explicit
`return Err(...)`) freely inside its scope — e.g. `eval_arithmetic`:

```rust
let frame = self.roots.push_frame();
let left_value = self.eval(code, left)?;      // error here skips pop_frame
self.roots.push_temp(left_value);
let right_value = self.eval(code, right)?;     // so does this
...
let left_number = self.arith_operand(left_value)?;  // and this
...
.map_err(Raised::from)?;                       // and this
...
self.roots.pop_frame(frame);
Ok(value)
```

Compare `step_in_temps_frame` (`lib.rs:864-873`), landed at `27788d94`
("Record the assumptions the review round named, and root eval's results"),
a genuine ancestor of Task 7's `3a9d6446`, so this is precedent Task 7 had
available, not a pattern invented after it:

> "The frame is opened and closed **here rather than inside `step`**,
> because `step` returns through a dozen `?` paths and a frame closed on
> only some of them is worse than none: it would leak on exactly the paths
> nobody tests."

`step_in_temps_frame` avoids exactly this by capturing the whole `Result`
into a local (`let flow = self.step(...)`) and popping unconditionally
before ever looking at whether it's `Ok` or `Err`. Task 7's three new
functions don't apply that same shape to their own frames, despite `eval`
itself (`eval.rs:53-70`) demonstrating the identical discipline correctly
for the **depth** counter three lines above `eval_node`'s dispatch — the
depth is decremented unconditionally by capturing `eval_node`'s result into
a local first, never through `?`. So the file shows both the right pattern
and the wrong one, a few dozen lines apart.

**Why this is Minor and not Major: it does not bite today.** Every real
call path into `eval` goes through `step` → `step_in_temps_frame`, whose own
frame is popped unconditionally regardless of what happened inside. Because
`RootSet::pop_frame` truncates to an absolute saved length
(`self.temps.truncate(frame.0)`) rather than decrementing by a count, any
inner frame `eval_arithmetic` etc. left unpopped on an error path sits
*inside* the range `step_in_temps_frame`'s own frame will truncate away, so
it gets swept up transitively the moment the outer frame closes. And since
4a has no trapping (`SIGNAL ON` is 4b's), a `Raised` that reaches `execute()`
ends the whole run anyway. The one place it's currently visible at all is
`eval.rs`'s own test helpers (`eval_source`/`eval_in_place`), which call
`interp.eval(...)` directly with no enclosing frame — there, a failed
`eval_source` call genuinely leaves an orphaned temp in `self.roots.temps`
for the rest of that `Interp`'s life, harmless to any current assertion
(over-rooting can't cause a spurious collection; it can only retain garbage
a beat too long) but a real, demonstrable instance of the gap, not a
theoretical one.

Worth fixing before more code calls `eval` from a context that isn't always
wrapped by `step_in_temps_frame` (a bare `eval` in a future task's control-
expression evaluator, for instance) — at that point the leak stops being
transitively swept and starts being a genuine unbounded-growth path in a
long-running program. Not blocking Task 7's own gate: nothing in criterion 1
through 7 can currently observe it, and the fix is mechanical (the same
capture-then-pop shape `step_in_temps_frame` already uses).

Everything else in the two files reads clean: helper extraction
(`arith_operand`, `concat`) removes real duplication rather than adding
indirection: `eval_prefix`'s `+`/`-` arm and `eval_arithmetic`'s six non-power
operators both funnel every operand through the identical nonnumeric-or-41.1
conversion. Doc comments carry oracle transcripts inline rather than
asserting behavior from memory, matching this project's established style.
The `Failure`/`Loud`/`Raised` three-type shape is exactly as thin as the
brief's boundary asked for.

## Mutation testing (scratch, `git archive HEAD` copy, not the working tree)

Two mutations, both direct hits on claims above:

1. **Concatenation separator, `Abuttal` → always `b" "`** (criterion 6's own
   listed mutation, "Abuttal treated as Blank"). `eval::tests::
   the_three_concatenation_forms` failed immediately: `x'b'` produced
   `a b` instead of `ab`.
2. **Power exponent failure routed through `Raised::nonnumeric` instead of
   `Raised::power_exponent_not_whole`.** `eval::tests::
   a_non_numeric_power_exponent_raises_26_8_not_41_1` failed immediately:
   `(41, 1)` instead of the expected `(26, 8)`.

Both mutations caught by the task's own tests; scratch copy discarded
afterward, working tree untouched (`git status` clean throughout).

## Working-tree hygiene: Task 8 (or later) work-in-progress is in this shared tree

Partway through this review, `git status` showed `eval.rs`, `error.rs`,
`rust/crates/rexx-exec/Cargo.toml` and `rust/Cargo.lock` modified, unstaged
(`error.rs` partly staged, `MM`) — a substantial, apparently in-progress
`eval_compare`/`eval_logical`/`eval_logical_list` implementation plus a new
`rexx-inventory` dependency, reading as Task 8's comparison/logical work
(possibly reaching into Task 12's message catalogue, given the
`rexx-inventory` addition and its own comment about "the text an error
prints"). This is not committed anywhere I can see, so it is not part of
`3a9d6446`/HEAD and not what I reviewed.

Not this review's to evaluate, same as `plan.rs` was not Task 7's own report's
to evaluate. What I did about it: nothing to the files themselves. All the
verification above — the 38/38 test counts, clippy, fmt, the two mutations —
was re-run in a `git archive HEAD` copy specifically *because* the live tree
had already gone dirty by the time I noticed, so none of it depends on
whatever state the shared tree happens to be in right now. `rust/crates/
rexx-parse/tests/sourceline_oracle.rs` is also modified, unstaged, and
equally not evaluated here. Flagged to the team lead so it isn't mistaken for
something this review passed judgment on.

## What I did not re-derive

The exact bisected/probed stack-frame byte counts (784→1600 debug,
192→624 release) — the reasoning in the report and in `lib.rs`'s
`INTERPRETER_STACK_BYTES` doc comment is coherent, correctly labeled as "a
probe reading, not a re-bisection," correctly defers final confirmation to
Task 11, and `records_the_stack_cost_of_one_eval_frame` passes in both debug
and release here. Re-bisecting to the byte is Task 11's stated job, not a
gap in this one.

## Summary

Spec compliance: **PASS**. Code quality: **PASS**, one Minor (the temps-frame
leak-on-error pattern, currently swept transitively by every real caller,
worth fixing before that stops being true). All ten oracle transcripts
independently re-verified byte-for-byte; two scratch mutations confirm the
power asymmetry and the concatenation-form distinction are load-bearing in
the test suite, not merely asserted. Nothing here blocks the commit or asks
for rework before moving on.
