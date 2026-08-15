# Task 3 report: a call's op names an address rather than a slot

Commits: `266acca07` (the widening), plus the plan correction below.

## What changed, and why

**`ir::NodePath`** (`rust/crates/rexx-exec/src/ir/mod.rs`). A `u32`, one bit per
step below a sentinel `1`, most significant first. `ROOT` is the sentinel alone;
`child(right)` shifts and refuses rather than dropping the sentinel; `steps()`
yields the bits below it outermost first. The refusal matters because a dropped
sentinel does not produce an error, it produces a *shorter path*, which resolves
to a different node -- a wrong answer where the refusal is merely an address
nobody can give out.

`child` carries `#[cfg_attr(not(test), expect(dead_code, reason = "no production
caller builds a path with a step in it"))]`, so the first production caller
reddens the line rather than leaving a stale exemption behind it. A *bare*
`#[expect(...)]` does not work here -- under `cfg(test)` the unit tests below do
call `child`, so `dead_code` never fires and `--all-targets` warns
`unfulfilled_lint_expectation`, which `-D warnings` rejects -- and I first
shipped a plain `allow` on that basis. The `cfg_attr` form is the narrower
answer; see the round-1 section.

**Both ops carry a path.** `Op::CallExpr { index: u32, slot: u16, path:
NodePath, site: u16, dst: u16 }` and `Op::TraceFunction { index: u32, slot: u16,
path: NodePath, src: u16 }`.

**`Interp::chunk_node_at(instruction, slot: u16, path: NodePath) -> Option<&Expr>`**
(`run.rs`) replaces `chunk_call_at` and `chunk_expr_at`, both of which are gone.
It resolves the slot and then walks the path. The call-shaped match that
`chunk_call_at` used to do moved into `Op::CallExpr`'s driver arm, which is the
arm that wants a call rather than an expression.

**`Op::TraceFunction`'s arm gained the `tracing_intermediates` guard** in front
of the descent. `Interp::trace_intermediate` (`eval.rs:244`) returns immediately
under the same condition before it reads `expr` at all, so no output moves; what
changes is that an untraced run walks no path. `tracing_intermediates` was
already `pub(crate)` (`trace.rs:683`), so no visibility widened.

**`compile`'s `push_value` passes `NodePath::ROOT`** at both pushes. Nothing new
is promoted.

**Two comments were corrected because this change falsified them**, not for
tidiness:

* `Op::CallExpr`'s "**Only at the root of a slot's expression**, because a slot
  is the finest address an op has" -- the premise is exactly what this task
  removes. Replaced with what the address now is. The *behaviour* it described
  (a root call promotes, a nested one does not) is unchanged and is asserted by
  `golden_tests`'s
  `a_call_at_the_root_of_a_value_takes_its_own_op_and_a_nested_one_does_not`, so
  the prose restating it is gone rather than restated.
* `push_value`'s "a call *inside* a larger expression does not and cannot: a
  slot is the finest address an op has, so there is nothing for an op to name."
  Same false premise, same removal.

## The width

The assertion is now `const _: () = assert!(size_of::<Op>() == 16)` and its doc
comment was rewritten rather than renumbered: it states what entry 24 measured
(a twelve-byte head, a control carrying a dead variant at that same width, and a
sixteen-byte build; nine rounds per axis; every axis's width column negative;
the cost the two-arm reading found belonging to *having an extra variant*, which
widening an existing variant is not) and says the number rests on that sitting
rather than on a cache-line argument.

**The check, and the deliberate failure that proves it is live.** All field
errors were fixed first and the workspace built clean (`cargo build --workspace
--all-targets`, exit 0) *before* the assertion was read, which is what the
brief's trap warns about.

| assertion | result |
|---|---|
| `== 15` | `error[E0080]: evaluation panicked: assertion failed: size_of::<Op>() == 15`, exit 101 |
| `== 20` | `error[E0080]: evaluation panicked: assertion failed: size_of::<Op>() == 20`, exit 101 |
| `== 16` | `Finished dev profile`, exit 0 |

So the assertion is evaluated, and 16 is what holds.

**The doc's claim about widening was measured too**, not inferred. With
`CallExpr`'s `slot` temporarily at `u32` (and the two type errors that follow
fixed, so const evaluation ran): `== 20` builds, exit 0; `== 16` fails with
E0080. Repeated with `site` at `u32` instead: `== 20` builds, exit 0. Both
probes were reverted from a copy taken beforehand, and the tree was rebuilt to
confirm.

A note on how I ran the probes: my first attempt at the `site` probe left a
call site unedited, so two `E0308`s stood, and I discarded that run's `== 20`
result and redid the probe cleanly rather than reading a number off a build
that had errors in it. **What I originally wrote here -- that `E0063`/`E0027`
suppress const evaluation while `E0308` does not -- is withdrawn and was false.**
It was one anecdote per error code, from probes differing in more than the code,
and the reviewer reproduced the configuration it claimed: field errors and the
`E0080` print together, so const evaluation runs. No mechanism replaces it: what
I can say is that the assertion is live (the table above), that a build with
errors standing is not a build whose assertion you have read, and that the fix
is to make the build clean before reading it.

**`PlanSlot`'s doc claim was falsified by this change and is corrected.** It
said an `Option<u32>` there "is the difference between an `Op` that stays the
width every other variant already fits in and one that grows". Measured at the
new width: with `PlanSlot(Option<u32>)` (and `UNRESOLVED`, `of` and `resolved`
adjusted), `size_of::<Op>() == 16` still passes, exit 0. The claim was true at
twelve and is false at sixteen, so the sentence now says the cost is paid by
every op in every chunk and leaves *whether it breaks the width* to the
assertion, citing that measurement. Its "for a field two of them carry" went at
the same time -- a count of a set -- and names `Op::Load` and `Op::Store`
instead.

## Tests

**Step 1/2, red then green.** The two `NodePath` unit tests were run against a
deliberately wrong implementation first -- `child` always answering `Some` (the
sentinel dropped) and `steps()` without its `.rev()`:

```
test ir::tests::a_node_path_carries_thirty_one_steps_and_refuses_the_thirty_second ... FAILED
    assertion failed: path.child(false).is_none()
test ir::tests::a_node_paths_steps_come_back_outermost_first ... FAILED
    left: [false, true]  right: [true, false]
test result: FAILED. 0 passed; 2 failed
```

Each test reddens on its own mutation: the count assertion is order-independent,
so the first test's failure is `child`'s alone and the second's is `steps()`'s
alone. With the real implementation both pass.

The second test is written against the real API rather than the brief's sketch,
as instructed, and pins the asymmetric case: a one-step path, or two equal
steps, reads the same in either direction and would not distinguish the
reversed iterator.

**A third test I added, `run::tests::a_paths_steps_land_on_the_node_it_names`.**
Nothing else in the workspace enters `chunk_node_at`'s descent loop -- every op
this crate emits carries `ROOT` -- so its `Binary`/`Prefix` arms had no coverage
at all. It parses `zz = -za + zb * f(1)` and checks each single-step and two-step
address by node kind, the two-step variables by name, the `true` step off a
prefix, the step off a leaf, and a slot the addressing does not name. (Round 1:
the first version of this test used `zz = -za + f(1)` and claimed an order
sensitivity it did not have -- see the round-1 section at the end.)

**"Can fail" is not "adds coverage", so I ran the extra step.** Mutation:
`chunk_node_at`'s two `Binary` arms swapped, so `false` takes the right operand
and `true` the left.

* whole workspace with the new test: `test result: FAILED. 598 passed; 1
  failed`, the one failure being `a_paths_steps_land_on_the_node_it_names`;
* whole workspace with the new test *deleted* and the mutation still in place:
  exit 0, nothing caught it.

So the test catches something the suite would otherwise miss. Both runs were
`memcap 8G cargo test --workspace --no-fail-fast`.

## Gates, unpiped

From `rust/`:

| command | exit | result |
|---|---|---|
| `cargo fmt --all --check` | 0 | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | clean |
| `CARGO_TARGET_DIR=<scratch> cargo clippy --workspace --all-targets -- -D warnings` | 0 | clean from an empty target directory; `rexx-exec` fingerprints confirm it was checked rather than skipped |
| `memcap 8G cargo test --workspace --no-fail-fast` | 0 | **1464 passed, 0 failed**, no `FAILED` line anywhere in the output |

The clean-target lint used a scratch `CARGO_TARGET_DIR` rather than `cargo
clean`, deliberately: other agents are building in the shared `target/` and
wiping it would have broken their runs.

1464 includes the three tests this task adds. The `rexx-exec` lib target went
596 -> 598 -> 599 across the session as they landed, which is the same +3.

## Expectations edited

Exactly one, `golden_tests.rs`:

```
-         1: CallExpr index=0 slot=0 site=0 dst=0
-         2: TraceFunction index=0 slot=0 src=0
+         1: CallExpr index=0 slot=0 path=root site=0 dst=0
+         2: TraceFunction index=0 slot=0 path=root src=0
```

Both are rendered fields of the two ops this task changes, and nothing else in
the expectation moved -- the `Clause`, `Store` and op indices are untouched, and
the nested half of that same test is byte-identical. No other test's expectation
was edited, and the whole suite is green, so the refactor was behaviour-preserving.

`golden.rs` renders the path as `root`, `root.L`, `root.R.L` and so on rather
than as the encoding's integer, because a golden expectation is read by a
person.

## The brief was stale, and I corrected the plan

The controller's dispatch was right: **the brief still carried the withdrawn
table**, and so did the plan, in the same file as the paragraph withdrawing it.
Per this crate's rule that a wrong plan is corrected rather than a wrong message,
I corrected the plan (`docs/superpowers/plans/2026-08-12-expression-promotion.md`)
rather than only reporting it:

* the Files section's `ir/mod.rs` line named `NodeAddr` and `Chunk::nodes`;
* a passage after `NodePath` defined `NodeAddr`, gave `Chunk` a `nodes:
  Vec<NodeAddr>` with a reserve call and a `ChunkTooLarge { what: "expression
  addresses past u16" }`. Replaced with a parenthesis saying what stood there,
  that the withdrawal paragraph above is what removes it, and why -- a task
  briefed from this passage would have built the table the paragraph deletes;
* the descent's code block took `addr: NodeAddr`. Now `slot: u16, path:
  NodePath`, which is what landed;
* **Task 4's own `Consumes:` line said `Task 3's NodePath/NodeAddr/Chunk::nodes`**,
  which would have sent Task 4 looking for two types that do not exist. It now
  names `NodePath` and the ops' `path` field.

`NodeAddr` and `Chunk::nodes` still appear at lines 366, 373 and 401 -- those are
the withdrawal paragraph itself and my note, both of which are about their
absence. **That enumeration was a grep for two type names and it was not a check
that the plan was clean**: the review found the plan's Architecture paragraph
describing the same table without naming either type, and line 366 listing
`NodePath` among the withdrawn things while line 373 says it stays. Both are the
team lead's to fix and are fixed at `464c5b31b`.

**Collision risk, flagged rather than assumed away:** the plan is the
controller's file and briefs regenerate from it. My edits are surgical and in one
commit, so a concurrent edit conflicts visibly rather than silently.

## Things I was unsure of, or left alone

* **`Loud::call_op_off_its_node`'s doc (`lib.rs:868`) describes only
  `Op::Call`** -- "reaching it means the op names an instruction whose `CALL` is
  one of the three forms that stay `Op::Generic`" -- while three arms in
  `drive.rs` raise it, including both of mine. That mismatch predates this task
  and `lib.rs` is not in its files, so I left it. It also carries "the three
  forms", a count of a set. Worth a task of its own.
* **`Op::Arith`'s doc says "a call has no register to arrive in, and `EvalExpr`
  names an expression *slot* of an instruction, which a subexpression is not."**
  That is still a true description of what `native_shape`/`push_native` do
  today, so it is not false yet -- but it is the sentence Task 4 falsifies, and
  it is not in Task 4's stated files. Flagging it so Task 4 is not surprised.
* **`compile`'s `ChunkTooLarge { what: "expression slots past u16" }` still
  guards the slot** and is unchanged. There is no `ChunkTooLarge` for a path,
  and there should not be: `NodePath::child` answering `None` is the refusal,
  and it belongs to whichever task first descends.
* **Whether the `TraceFunction` guard should also skip the address check.** As
  written, an untraced run never resolves the address and so never reports
  `call_op_off_its_node` for a bad one. That is what the plan asks for and it
  costs nothing observable today, but it does mean the loud check is
  trace-conditional. I did not change it.

---

# Fix round 1

Review: `.superpowers/sdd/2026-08-12-expression-promotion/task-3-review.md`.
Findings 1 and 2 are the team lead's and are fixed at `464c5b31b`; the plan file
is untouched here. Findings 3, 4 and 5 are mine, and I took 7 as well because it
corrects a claim I made. Sections above have been edited in place where they
carried a statement this round falsified, and each such edit says so.

## Finding 2 fallout: what I withdrew

The report's `E0063`/`E0027` paragraph is gone, replaced by what was actually
measured. I will not defend it: it was a refinement of the team lead's
truncation artifact, built the same way -- one anecdote per error code, probes
differing in more than the code -- and it is the shape my own notes call out,
inventing a mechanism to explain a number. **The width result is untouched**;
only my explanation of how a build can lie about it was wrong, and it is
replaced with no mechanism rather than a different one.

## Finding 3: an assertion, not a trimmed comment

I added an assertion that reddens rather than narrowing the comment, because
narrowing would have left the descent's *own* step order unpinned by anything.
`a_node_paths_steps_come_back_outermost_first` pins `NodePath::steps`; nothing
pinned `chunk_node_at`'s consumption of it, and those are two places a reversal
can live.

The program is now `zz = -za + zb * f(1)`: `+` at the root, a prefix on its
left, a multiplication on its right, and the call as that multiplication's right
operand. `ZB`'s address is `[right, left]`, which read innermost first is
`[left, right]` -- the `right` step off a prefix, which has nowhere to go. So a
backwards descent finds `None` where the test finds `ZB`. The two-step variables
are asserted **by name** through `program.symbols.name`, so a wrong branch lands
on the other name rather than on another node of the same kind.

Mutations, `cargo test -p rexx-exec --lib`, and then the whole workspace:

| mutation | `a_node_paths_steps_come_back_outermost_first` | `a_paths_steps_land_on_the_node_it_names` |
|---|---|---|
| `.rev()` removed from `steps()` | FAILED | **FAILED** (it passed before this round -- the review's finding) |
| descent consumes the steps backwards, `steps()` untouched | ok | **FAILED** |
| the two `Binary` arms swapped | ok | **FAILED** |

For the two descent mutations I ran `memcap 8G cargo test --workspace
--no-fail-fast` and listed the failing test names: **exactly one for each, and
it is this test.** So the second row is coverage the suite did not have -- a
reversal inside `chunk_node_at` is invisible to the `NodePath` test by
construction -- and the third confirms the branch property survived the rewrite.

## Finding 4: `PlanSlot`, corrected rather than hedged

The sentence now reads that the op array **does not** force the reserved value,
gives the measurement, and says plainly that the reserved value buys no width
here and that anything resting on it has to rest on something else. The hedge
the review named -- asserting a cost while deferring "whether it breaks the
width" to an assertion the same sentence reports as passing -- is gone, and so
is the "paid by every op in every chunk" clause that the neighbouring
measurement contradicted.

## Finding 5: the assertion's doc says one thing

The first paragraph no longer argues the budget. It now says only what the
assertion is *for*: the width is a whole-array fact, so a variant that outgrows
the number grows every op, and this line catches that when it happens. The
"which is the argument `PlanSlot` rests on" clause went with it -- after finding
4 it was false.

The second paragraph carries entry 24's caveat in the entry's own voice, which
the first version elided: **both its wide arms widen the enum by adding a dead
variant, where widening a variant already present adds none**, and the entry
calls the width column the right prediction for that case. So the comment now
says sixteen rests on the sitting *plus that prediction*, rather than on the
sitting alone.

## Finding 7 taken: `cfg_attr(not(test), expect(...))`

Verified rather than trusted. `cargo clippy --workspace --all-targets -- -D
warnings` exits 0 with it, and it is live: adding a production call to `child`
(`compile.rs`, temporarily) makes the same command fail with `error: this lint
expectation is unfulfilled`, exit 101. Restored, exit 0. So Task 4 is told by
the compiler to remove the exemption instead of by a comment, which is the only
form of that instruction with a record of holding here. My earlier "`#[expect]`
does not work here" was true of the bare form only, and the report now says so.

## Findings 6 and 8: left alone, deliberately

* **6, `render_path`'s `L`/`R` mapping is unexercised.** True -- every emitted op
  carries `ROOT`, so `golden.rs`'s loop body never runs. I did not add a golden
  for it: this task promotes nothing, so any program that would exercise it is
  Task 4's, and the review says Task 4's first golden covers it. Flagged rather
  than assumed covered.
* **8, `NodePath(0)` would underflow `steps()`.** Unreachable as written -- the
  field is private and `ROOT` and `child` both keep a set sentinel. I did not add
  a guard, because a guard on an unreachable path is a branch no test can redden;
  the invariant is stated at the type ("behind a sentinel `1`") and at `steps()`
  ("zero for `ROOT`, whose value is the sentinel alone"). A future in-module
  constructor has to preserve it, and that is what those two sentences are for.

## Gates, unpiped, from `rust/`

| command | exit | result |
|---|---|---|
| `cargo fmt --all --check` | 0 | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | clean |
| `memcap 8G cargo test --workspace --no-fail-fast` | 0 | **1464 passed, 0 failed** |

Same 1464 as before: this round changed one test's body and program, added no
test and removed none.
