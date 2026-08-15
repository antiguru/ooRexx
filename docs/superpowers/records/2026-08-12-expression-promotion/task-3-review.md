# Task 3 review: a call's op names an address rather than a slot

Range reviewed: `faf6360ce..b930290ec` (`266acca07` the widening, `b930290ec` the plan correction).

**Spec compliance: pass for the code, incomplete for the plan correction.**
The widening itself meets the brief exactly: nothing new is promoted, one expectation
moved and only the two rendered field names in it, the assertion is at sixteen with a
doc that cites entry 24, `NodePath` has the two methods and both unit tests, the descent
is one function taking `slot` and `path`, both arms changed, the guard is in front of the
descent, and `tracing_intermediates` needed no visibility change. The plan correction
(commit 2) leaves the plan still briefing the withdrawn table -- finding 1.

**Code quality: changes requested.**
No behavioural defect found, and behaviour preservation holds under checking. Four
statements in comments, the report or the plan are false or unsupported (findings 1-4),
one of which the brief specifically asked to be corrected rather than hedged.

---

## What I verified independently, and how

A sandbox at `b930290ec` (`git archive`, own `CARGO_TARGET_DIR`, `interpreter/` symlinked
so `rexx-inventory`'s build script finds `rexxmsg.xml`) so nothing touched the working
tree or the shared `target/`.

| check | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no diagnostics |
| `cargo test -p rexx-exec --lib` | **599 passed, 0 failed** (matches the report's 596 -> 599) |
| `cargo test --workspace --no-fail-fast` | 1432 passed, 32 failed -- **1464 tests**, the report's number; the 32 are environment (untracked `corpus-l1`, oracle binary, checkout-shaped assertions), see the control below |
| assertion live: `== 15` | `error[E0080]: evaluation panicked` |
| `CallExpr.slot` widened to `u32`, `== 16` | E0080; at `== 20` builds -- the `Op::CallExpr` doc's "20 bytes" claim is true |
| `PlanSlot(Option<u32>)`, `== 16` | builds, exit 0 -- the report's measurement reproduces |

**Behaviour preservation.** `chunk_node_at` at `NodePath::ROOT` is `chunk_expr_at`
verbatim: same two arms in the same order, `_ => None`, and the old `u32::from(*slot)`
against a `u16` slot cannot change which values match `0`. The call-shaped match moved
intact into `Op::CallExpr`'s arm, and both `None` routes still reach
`Loud::call_op_off_its_node`, so the failure set is unchanged.

**The `tracing_intermediates` guard is output-neutral**, and provably so on two counts:
`Interp::trace_intermediate` (`eval.rs:244`) returns under the same condition before it
reads `expr`, and `Op::TraceFunction` is emitted at exactly one place
(`compile.rs:775`), immediately behind the `Op::CallExpr` carrying the same `index`,
`slot` and `path` -- whose own descent would already have broken the region on any
address the trace op could fail on. The skipped `roots.temp_at` is a pure index read.
The `continue` is inside `for region_op in ops`, which carries no counter of its own
(`drive.rs:1223` is the same idiom), so it cannot skip a `pc` advance.

**`NodePath`.** `child` tests the top bit before shifting, so it refuses at the width
rather than dropping the sentinel; `steps()` computes `depth` from `leading_zeros` and
walks `(0..depth).rev()`, which is outermost first. Both unit tests redden for the right
reason and neither passes on the other's mutation.

**Mutation control for the new test.** With `chunk_node_at`'s two `Binary` arms swapped
I ran the whole workspace and diffed the failure set against a matched baseline in the
same sandbox: **exactly one added failure**, `run::tests::a_paths_steps_land_on_the_node_it_names`.
The report's claim that deleting the test leaves the mutation uncaught is confirmed
(the environment-blocked `ir_dual` sweep could not have caught it anyway -- `compile`
emits only `NodePath::ROOT`, so no compiled program enters the descent loop).

---

## Findings, most severe first

### 1. The plan still briefs the withdrawn table, in the paragraph an implementer reads first

Commit `b930290ec` exists to stop the plan building the table its own withdrawal
paragraph deletes. It missed two places:

* **`docs/superpowers/plans/2026-08-12-expression-promotion.md:11`**, the plan's
  **Architecture** paragraph: "A call's op stops naming an expression *slot* and starts
  naming **an entry in a new per-chunk address table**, which carries the slot and a
  bit-encoded path down to the node". That is the withdrawn design, stated as the plan's
  architecture. This is the same defect the commit fixed at line 401 and in Task 4's
  `Consumes:` line, left standing where it is most likely to be read.
* **line 366**: "So `NodePath`, `NodeAddr` and `Chunk::nodes` are all withdrawn" --
  contradicted seven lines later at 373 by "`NodePath` stays". `NodePath` landed. The
  sentence pre-dates this commit, but it sits inside the passage the commit was
  reconciling and is the same class of error.

The report's own enumeration ("`NodeAddr` and `Chunk::nodes` still appear at lines 366,
373 and 401 -- those are the withdrawal paragraph itself and my note") is a grep for two
type names, and line 11 describes the table without naming either.

### 2. The `E0063`/`E0027` refinement is false, and the plan's trap paragraph with it

The report states, as guidance for anyone repeating the width check, that "it is
`E0063`/`E0027` field errors specifically that suppress const evaluation, not type errors
generally". I reproduced the exact configuration in this crate: added a `probe: u32`
field to `Op::CallExpr`, left the construction and match sites unedited, and set the
assertion to a value that cannot hold. `cargo build -p rexx-exec --lib` printed

```
error[E0063]: missing field `probe` in initializer of `Op`
error[E0027]: pattern does not mention field `probe`
error[E0080]: evaluation panicked: assertion failed: size_of::<Op>() == 999
```

Const evaluation ran. The same holds for standalone `rustc` probes of `E0063`, `E0027`
and `E0308` in isolation (rustc 1.97.1, the toolchain in use): all three leave the
`E0080` in place. So field errors do not suppress const evaluation, and the refinement
rests on one anecdote per error code with the two probes differing in more than the code.

This also falsifies the plan at line 377 -- "const evaluation does not run while they
do" -- which the correction commit left standing and which the next person to widen an
op will read. Whatever the first attempt actually hit, it was not this. The width
*result* is unaffected: I confirmed the assertion is live, that 15 fails, and that a
`u32` `slot` gives 20.

### 3. `a_paths_steps_land_on_the_node_it_names`'s doc claims an order sensitivity the test does not have

The test's doc comment says `zz = -za + f(1)` is asymmetric "so a descent taking the
steps **in the wrong order**, or the wrong branch, arrives at a node of a different kind"
-- and the report repeats it. I removed `.rev()` from `steps()` and ran the suite:

```
test ir::tests::a_node_paths_steps_come_back_outermost_first ... FAILED
test run::tests::a_paths_steps_land_on_the_node_it_names ... ok
```

It passes. Its two-step assertions are `[false,false]` (reads the same reversed) and
`[false,true]`/`[true,false]`, both of which are `None` in either direction -- reversal
turns each into the *other* one, which the test also expects to be `None`. The property
is genuinely pinned, but by `a_node_paths_steps_come_back_outermost_first`, not here.
The branch half of the claim is true and I confirmed it by mutation.

### 4. `PlanSlot`'s corrected sentence hedges a falsified premise instead of correcting it

The landed text (`ir/mod.rs`): "**A `u32` with one reserved value rather than an
`Option<u32>`, and it is the op array that weighs it.** An `Option<u32>` is eight bytes
where this is four, and it is a field `Op::Load` and `Op::Store` carry, so **the cost is
paid by every op in every chunk** rather than by those two. Whether the wider one would
*break* the width is the assertion above `Op`'s to say ...: measured 2026-08-12, with
this type holding an `Option<u32>` that assertion still passes."

I reproduced that measurement -- `PlanSlot(Option<u32>)` builds with `size_of::<Op>() == 16`.
So at the landed width the wider field costs the op array nothing, and the sentence
asserts a cost ("is paid by every op in every chunk") that its own next clause measures
at zero. What the change actually falsified is the *decision's* justification: the width
no longer forces `u32`-with-sentinel. Saying so is the correction; deferring "whether it
breaks the width" to an assertion the same sentence reports as passing is the hedge the
rule forbids. (The naming fix in the same edit -- "for a field two of them carry" ->
"`Op::Load` and `Op::Store`" -- is right, and is the numeral-as-quantifier trap handled
correctly.)

### 5. The assertion's doc gained the measurement but kept the argument it was to replace

The brief asked for the comment to be **rewritten** rather than renumbered, because its
first paragraph argues the budget from "every op in every chunk pays for the widest
variant" while entry 24 measured that width free on those axes. The landed comment keeps
that paragraph verbatim (only "today" dropped) and appends the measurement.

The appended paragraph itself is accurate against entry 24 -- I read it: three arms (A
head at 12, C head plus a dead variant at 12, B at 16), nine rounds per axis, every
axis's width column negative, `emptyloop` +3.00% at 9/9 charged to the variant and
-0.74% to the width, and "an argument from cache lines" correctly disclaimed. It omits
one caveat the entry states in its own voice -- "Nothing about a *widened variant*, which
is what would actually land ... the exact change was not built" -- though the entry also
endorses the inference ("the width column is the right prediction for it"), so this is an
elision rather than a false claim. The retained first paragraph is about bytes and stays
true; the tension with the measurement two lines below is the reason the brief asked for
a rewrite.

### 6. `render_path`'s `L`/`R` mapping is unexercised

Every op `compile` emits carries `NodePath::ROOT`, so `golden.rs`'s loop body never runs
in any golden expectation: swapping `'R'` and `'L'` reddens nothing today. Not required
by this task's brief, and Task 4's first golden test covers it -- noted so it is not
assumed covered now.

### 7. `#[allow(dead_code)]` on `NodePath::child` -- accurate, and one narrower option exists

The reason is accurate (no production caller builds a stepped path), the scope is the
item, and the idiom matches `ChunkTooLarge::what` (`ir/mod.rs:882`). I confirmed the
report's claim about `#[expect]`: under `--all-targets` it fails with
`unfulfilled_lint_expectation` because the unit tests call `child`. The report's "that
does not work here" is true of a bare `#[expect]` only --
`#[cfg_attr(not(test), expect(dead_code, reason = "..."))]` passes
`cargo clippy --all-targets -- -D warnings` cleanly (verified, exit 0) and keeps the
property the author wanted, that Task 4 must remove it. A suggestion, not a defect.

### 8. Nit: `NodePath(0)` would underflow `steps()`

`u32::BITS - 1 - self.0.leading_zeros()` is `31 - 32` for a zero path: a debug panic and
a wrong answer in release. Unreachable as written -- the field is private and both
constructors (`ROOT`, `child`) keep a set sentinel -- so this is only a note about what
a future in-module constructor would have to preserve.

---

## Things the report flagged that I agree with, and did not re-litigate

`Loud::call_op_off_its_node`'s doc (`lib.rs:868`) describing only `Op::Call` and carrying
"the three forms" is pre-existing and out of this task's files. `Op::Arith`'s doc is
still true today and is Task 4's to falsify. The `TraceFunction` guard making the loud
check trace-conditional costs nothing observable, for the pairing reason above.
