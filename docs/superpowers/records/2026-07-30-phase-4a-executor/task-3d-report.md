# Task 3d report: the nested-call recursion, which reaches the sized path

Status: **DONE.** Commit `ae7e8bce`, "Count the nested-call recursion too, on
one shared depth budget". BASE `2d067c2a`.

Tests: `rexx-parse` **398 passed / 0 failed**; workspace **619 passed / 0 failed
/ 3 ignored**; clippy clean workspace-wide; fmt clean. Every exit status checked
directly, not through a pipe.

## Discipline

Every build unpiped with its exit status read from `${PIPESTATUS[0]}` or `$?`
before any number was taken from the resulting binary. Oracle invocations
wrapped as `( ulimit -v 1048576; build/bin/rexx FILE )`. Staged four
`rexx-parse` paths and nothing else; `rexx-extract`'s uncommitted work left
untouched.

## Step 1: the two cliffs, bisected

Ours, `say f(f(…'a'…))` on the 512 MiB thread D19 gives `rexx-exec`'s public
entry point, debug, bisected to within 500:

```
sized nested_calls: survives 91,948, aborts 92,337
```

The oracle, same program, bisected to within 500 and then checked three times
on each side for the jitter its paren cliff shows:

```
oracle n=34,293:  213 213 213     (parsed, then 43.1 at run time)
oracle n=34,500:  213 213 213
oracle n=34,760:  245 245 245     (Error 11.1)
```

So `[34,500, 34,760]`, and unlike the paren cliff it is sharp on both sides at
this resolution. The oracle's call cliff is **lower** than its paren cliff
(34.6k against 39.9k) while ours is **higher** (92k against 89k), which is worth
noting only because it rules out any tidy "calls cost more" story: the two
implementations order the two constructs oppositely.

## Step 2: the failing tests, and defeating the mechanism

Two tests in `tests/deep.rs`, both written before the fix.

`a_call_nesting_past_the_native_cliff_raises_11_1_instead_of_aborting` parses
100,000 nested calls on a sized thread and asserts `(11, 1)`.

`parens_and_calls_share_one_budget_rather_than_one_each` puts
`MAX_EXPR_DEPTH / 2 + 100` levels of each construct in one expression, so
neither alone would trip a separate counter.

**Verified by mutation rather than by watching them go green.** With the
`arg_list` guard removed in a scratch copy of the tree, run individually:

* the call test **aborts the test binary** with "has overflowed its stack",
  which is the failure it exists to remove,
* the shared-budget test **fails outright** (`expect_err` on a successful
  parse), which is the two-separate-budgets scenario reaching the end.

The four pre-existing tests in the file pass against the mutant, so neither new
test is riding on an existing one.

## Step 3: `arg_list` counted, on one shared budget

`MAX_PAREN_DEPTH` becomes `MAX_EXPR_DEPTH`, `paren_depth` becomes `expr_depth`,
and `arg_list` checks and increments the same field the grouping-paren arm does.

**One budget, and the answer to the question the plan asked.** The two
recursions genuinely differ in code path and in stack cost per level, which is
an argument for separate counters until you notice they differ in the thing
that does not matter and agree in the thing that does: they spend the *same
stack*. Two budgets of 50,000 would let `f((f((…))))` reach 100,000 real levels,
past both native cliffs, and abort exactly as it did before any counter
existed. The limit has to mean "total expression nesting" because that is the
quantity the stack cares about. That is not an argument I wanted to leave in a
report, so it is `MAX_EXPR_DEPTH`'s doc comment and the shared-budget test.

Two implementation details worth stating because both were choices:

* **Counted once per argument list, not once per argument.** `f(a, b, c)` is
  one level of nesting and three siblings; counting per argument would charge a
  wide call as though it were a deep one.
* **Decremented before the caller's `?`**, which needed the body split into
  `arg_list_inner`, because `arg_list` has several early returns and a count
  that unwinds on success but not on error is wrong on exactly the paths nobody
  tests. Same shape the paren arm already used.

Boundary confirmed on the shipped binary, and it is exact for both constructs:
50,000 parses, 50,001 raises 11.1, for parens and for calls alike.

**50,000 stays, now justified for both.** It sits above both oracle cliffs,
which is the direction to err in: a lower limit would raise a condition where
the oracle returns an answer, which is a worse failure than accepting a few
thousand levels the oracle refuses. It stays far below the shallower native
cliff, which `MEASURED_NATIVE_CLIFF` and a `const _: () = assert!(…)` now pin
so it cannot be raised past the point where it never fires.

## Step 4: the corrections, re-measured rather than carried over

Both came from Task 3c's review, and **both were re-measured on the final code
rather than copied from the review**, which mattered: this task's own counter
moved one of them again.

| construct, default 2 MiB thread | before Task 3c | after Task 3c | after Task 3d |
|---|---|---|---|
| grouping parens | 337 / 338 (A/B measured) | 331 / 332 | 331 / 332 |
| nested calls | [350, 360] as Task 3c reported it | 349 / 350 | 341 / 342 |

Each cell I measured is a bisected pair. The one that is not is marked as
what it is: Task 3c reported the pre-3c call cliff as a 10-wide bracket and I
never rebuilt that configuration to narrow it, so quoting it as a pair would
be inventing precision. The pre-3c paren pair *is* measured, by rebuilding
with `expr.rs` from `6285d98f^`.

**M1.** The default-thread paren cliff is **331/332**, not the 337/338 that four
places stated. The general lesson is now in the tree beside the number rather
than only in this report: a counter costs stack, so adding one makes the
unprotected case slightly shallower than the measurement that justified it.
Task 3d's guard did it again, to the call figure.

**M2.** "Nested calls are shallower than plain grouping parens" is **backwards**,
in the report and in the probe's doc. Measured like-for-like on one binary,
341 against 331. It was backwards on the original numbers too, without any
re-measurement. The priority it argued for still holds, since calls were the
shallowest *unguarded* recursion, prefix chains being three times deeper; only
the comparison was wrong.

Three in-tree places are corrected: `tests/deep.rs`'s module doc, the
`a_shallow_paren_nesting…` test doc, and `examples/depth_probe.rs`'s module doc.
The fourth place is Task 3c's own report, which I have **not** edited: its
numbers were correct for the code it measured, and rewriting another task's
report to match later code would destroy the record of when the figure was
true. This report is the correction.

The `a_shallow_paren_nesting…` test still parses 300, now with a stated margin
of 31 levels rather than the 37 the old figure implied.

## Step 5: the two minors

**`parse_constant_expression`'s uncounted `(`.** A clause in `MAX_EXPR_DEPTH`'s
doc, as the plan asked, rather than a code change: the outermost parenthesis of
a `RAISE`, `FORWARD`, `USE ARG` default or `ADDRESS … WITH` operand is not
counted, so those four constructs get an effective limit of `MAX_EXPR_DEPTH + 1`
and everything nested inside is counted normally. Counting it is a four-line
change if a later task would rather have one number; I followed the plan.

**The const comparison.** `MEASURED_NATIVE_CLIFF = 88_800` plus
`const _: () = assert!(MAX_EXPR_DEPTH < MEASURED_NATIVE_CLIFF, …)`. A limit at
or above the native cliff would fire only for depths the process no longer
survives reaching, which is the silent abort the counter was added to remove,
arrived at by a plausible-looking edit. Now a compile error.

## The two facts preserved in the tree

Per the request that these live where the reader who needs them is standing,
not only in a review file:

* **The corpus's deepest actual parenthesis nesting is 5**, across 12,103 `.rex`
  files in `rust/corpus/` and `rust/corpus-l1/`. That is in `MAX_EXPR_DEPTH`'s
  doc comment, because it is what makes a 50,000 limit obviously safe and it is
  the sentence that stops a later reader worrying the limit is too low.
* **`select` followed by `otherwise` raises 7.1 before any nesting is built**, so
  a nesting probe on that shape reports a clean parse error at every depth and
  looks like a pass while measuring nothing. That is in
  `examples/depth_probe.rs`, on the new `select_nesting` mode, which is where
  the next person writing a nesting probe will be standing. I hit this myself
  while reviewing Task 3c. `when 1=1 then` is the clause that actually nests,
  and with it `SELECT` parses cleanly to 100,000 on a default thread, which also
  closes Task 3c's read-but-not-measured caveat.

## What is still open, and why

**Prefix-operator chains**, `- - - -1`, recursing in `message_subterm`, which
calls itself for its operand and passes through neither counted site. Aborts
between 1,150 and 1,200 on a default 2 MiB thread. Deliberately not fixed here,
and now named as the single open gap in `expr_depth`'s own doc comment rather
than as one of a pair:

* it does not reach the sized path at any depth anyone has produced, which is
  the property that made the call recursion a task rather than a note,
* closing it needs a second check in a third function and its own oracle cliff,
  neither of which this task had,
* and the plan scoped Task 3d to the recursion that reaches the sized path.

**Not attempted, per the plan:** a stack-aware counter. The costing is in the
plan and in Task 3c's review.

## Verification

All from `rust/`, at `ae7e8bce`, each exit status read directly:

* `cargo test -p rexx-parse --no-fail-fast` -> **398 passed, 0 failed**, exit 0.
* `cargo test --workspace --no-fail-fast` -> **619 passed, 0 failed, 3
  ignored**, exit 0.
* `cargo clippy -p rexx-parse --all-targets -- -D warnings` -> exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` -> exit 0.
* `cargo fmt -p rexx-parse -- --check` -> exit 0.
* Shipped-binary spot checks: `nested_calls_sized` at 49,999 and 50,000 parses,
  at 50,001, 100,000 and 200,000 raises 11.1; `select_nesting` at 100,000
  parses; `paren_default` 331/332 and `nested_calls` 341/342.
