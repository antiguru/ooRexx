# Task 4 report: a call nested inside an expression

**Status: DONE_WITH_CONCERNS.** Commit `b73ef0b6e97001d24aec3a13d5b458913ca6ce40`, one commit.
All three gates green. Two things the brief could not know moved: the design of
`native_shape` (deliberately, and the plan is corrected) and the stack the
corpus sweep runs on.

---

## What changed and why

### `ir/compile.rs`

* `native_shape(expr, path: Option<NodePath>)`. `ExprKind::Call` answers
  `path.is_some()`; every other arm ignores the address and passes
  `descend(path, ..)` down.
* `descend(path, right) -> Option<NodePath>` is the one place a step is taken,
  so the `false`/`true` flags stay in step with `Interp::chunk_node_at`'s arms.
* `push_native` gains a `Call` arm emitting `Op::CallExpr` + `Op::TraceFunction`,
  and takes `calls`, `index`, `slot`, `path`. Both it and `push_value` carry a
  `clippy::too_many_arguments` expectation; `push_value` already had one.
* `push_value`'s root-call special case is **deleted**. `native_shape(expr,
  Some(NodePath::ROOT))` accepts it and `push_native` emits it.
* `u16::try_from(slot)` moved into `push_native`'s `Call` arm rather than being
  done eagerly in `push_value`, so a native expression at a slot past `u16` is
  refused only if it actually holds a call -- which is what the deleted code did.

### `ir/mod.rs`

* `NodePath::child`'s `cfg_attr(not(test), expect(dead_code, ..))` is **gone**.
  It is an `expect`, so a production caller reddens it; the controller's check
  holds.
* `Op::CallExpr`'s doc no longer says the decision is `push_value`'s, and names
  the two new golden tests instead of the deleted one.
* `Calls`' doc said "a slot per `Op::Call`". That was already false before this
  task -- `Op::CallExpr` reserves sites too -- and is corrected.
* `push_value`'s doc said "a call and a `.name` are two such expressions"
  declined by `native_shape`. Corrected to `.name` alone.

### Deviation from the brief: the address is `Option<NodePath>`

The brief's `native_shape` takes `path: NodePath` and charges the depth in
**every** arm, with `ExprKind::Call` answering `true` unconditionally. That
version refuses a whole slot whenever *any* node stands past the width -- so a
**call-free** expression nesting more than 31 operators, which compiles to
native ops entire today, would fall back to one `Op::EvalExpr`.

`rust/corpus/lang/deep_nested_expr.rex` is exactly such a program. Measured, by
instrumenting `check_body` to print an op histogram and running it once against
`HEAD` and once against this change:

```
lang/deep_nested_expr.rex main: 12004 ops
  {"Arith": 2999, "Clause": 2, "Load": 1, "LoadConstant": 3000, "Say": 1,
   "Store": 1, "TraceLiteral": 3000, "TraceOperator": 2999, "TraceRead": 1}
```

**Identical before and after**, and with no `EvalExpr` and no `Generic` in it.
Under the brief's version that whole assignment becomes one `Op::EvalExpr` --
an optimisation plan making a corpus program slower. So the address is
`Option<NodePath>`, only the call arm reads it, and the plan
(`docs/superpowers/plans/2026-08-12-expression-promotion.md`) and the brief are
both corrected to match, with the measurement written into the plan.

Both designs give the **same** bound for the case the brief's own depth test
covers (a call at 31 operators promotes, at 32 the slot goes general), so the
tests the brief asks for are unaffected.

### The stack: the one thing this change broke and how

`compile`'s expression walk takes a frame per operator. The wider parameter
lists make each frame bigger, and `ir::corpus_shape_tests` compiles
`deep_nested_expr.rex` on a libtest thread's 2 MiB and overflowed, aborting the
whole `rexx-exec` lib test binary.

**Corrected in fix round 1 (review finding 3): the reason first given here was
false.** This report said the sweep is the only caller reaching `compile`
without going through `Interp::on_interpreter_thread`. It is not --
`golden_tests`' `compile_for_test` calls `super::compile` directly on libtest
threads throughout, and `plan.rs`'s unit tests reach it through
`Interp::chunk_for` on libtest threads. The true reason no other harness
overflowed is that **no other direct caller compiles a body this deep**. The
false version implies those callers are protected when they are not. The fix
itself is unaffected; `INTERPRETER_STACK_BYTES` is still the right size,
because that is what `on_interpreter_thread` gives the same walk in production.

Attributed rather than assumed: with the sweep's own `root_of`/`native`
recursion stubbed out it still overflowed, so it is `compile`'s walk and not
the test's restatement.

Measured thresholds, by `RUST_MIN_STACK` bisection on `cargo test -p rexx-exec
--lib ir::corpus_shape`:

| tree | aborts at | passes at |
|---|---|---|
| `HEAD` (before this task) | 1966080 (1920 KiB) | 2097152 (2048 KiB) |
| this task, sweep called inline | 2621440 (2560 KiB) | 2883584 (2816 KiB) |

So **before this task the sweep was passing with under 128 KiB of a 2 MiB stack
to spare.** That was luck, not design.

Production is unaffected and that was checked, not reasoned: `rexx-run` on
`corpus/lang/deep_nested_expr.rex` exits 0 and prints `3000` both before and
after, and generated programs of 6000, 9000, 12000, 18000, 24000, 30000 and
40000 terms all exit 0 -- the interpreter thread has 512 MiB.

The fix is that the sweep spawns a thread of `crate::INTERPRETER_STACK_BYTES`
and resumes a panic on the test thread, which is the stack the interpreter
compiles bodies on anyway. It is **not** a mechanism for the depth divergence
and adds no depth counting.

### Tests

* **Deleted** `an_operand_that_needs_eval_leaves_the_whole_expression_general`.
  Its subject, `zw = length('ab') + 1`, promotes now (it went red, as expected).
  What it stated -- one operand with no op takes the whole expression down -- is
  stated by `the_value_shapes_outside_the_native_set_stay_general`'s
  `zw = .nil || za` row for a term outside the native set, and by the new width
  test for a call with no address.
* **Added** `a_call_promotes_at_the_root_and_below_it` and
  `a_call_nested_past_the_paths_width_leaves_the_slot_general`, replacing
  `a_call_at_the_root_of_a_value_takes_its_own_op_and_a_nested_one_does_not`.
* `corpus_shape_tests`: `root_of`'s `Call` arm keeps `Root::CallExpr` with a
  corrected comment; `native` takes a `depth` and gets a `Call` arm bounded by
  `DEEPEST_ADDRESSED_CALL = 31`, written out here rather than read off
  `NodePath` for the reason the module doc already gives about the operator sets.
* Five new stanzas in `tests/ir_dual_cases/operators`; the `arithmetic` stanza's
  falsified comment corrected, its oracle bytes untouched.

---

## Gate commands, unpiped

```
$ cargo fmt --all --check
GATE_FMT=0

$ memcap 8G cargo clippy --workspace --all-targets -- -D warnings
GATE_CLIPPY=0

$ memcap 8G cargo test --workspace --no-fail-fast
GATE_TEST=0
binaries: 82   passed: 1464   "test result: FAILED": 0   "overflowed": 0
```

`clippy` was **also** run from a cold `CARGO_TARGET_DIR` (in the scratchpad, not
the shared `target/`), per `rust/CLAUDE.md`'s warning that a warm target can
report green without linting: exit 0, and the log shows all eight workspace
crates checked. The `clippy::byte_char_slices` violation that rule cites is no
longer in the tree. The scratch target directory was deleted afterwards.

Two lints were caught and fixed on the way: `clippy::manual_repeat_n` in the new
golden test, and a `cargo fmt` diff in the register assertion.

Targeted runs, with counts read rather than statuses:

* `cargo test -p rexx-exec --lib ir::golden_tests::a_call` before implementing:
  `3 tests ... 1 passed; 2 failed` -- the two new ones failed, the pre-existing
  `a_call_compiles_to_a_clause_region_and_one_call_op` passed.
* `cargo test -p rexx-exec --lib ir::` after: `76 passed; 0 failed`.
* `cargo test -p rexx-exec --test ir_dual`: `9 passed; 0 failed`.

---

## Oracle captures

Probe directory: a fresh `mkdir`'d subdirectory of the session scratchpad,
`.../task4-nested-calls/oracle`, empty before use. Every run:

```
( ulimit -v 1048576; \
  LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx <ABS>/sN.rex </dev/null \
  >/<ABS>/sN.out 2>/<ABS>/sN.err )
```

Three separate descriptors, absolute redirects, no `2>&1`, no `REWRITE`, none
of the oracle-crashing forms. Each probe file's bytes are character-for-character
the stanza's `program` block, so the `line N` numbering in a report matches.

| stanza | probe | what the bytes are |
|---|---|---|
| call as an operand of each family | `s1.rex` | rc 0, seven stdout lines |
| call under a prefix operator | `s2.rex` | rc 0, five stdout lines |
| two calls, call inside a call's argument | `s3.rex` | rc 0, six stdout lines |
| raise from a nested call, trapped | `s4.rex` | rc 0, three stdout lines: `in-arg 42 at 2`, `operator 41 at 6`, `callee 40 at 10` |
| `trace i` | `s5.rex` | rc 0, three stdout lines and 34 stderr lines |

Checked for trailing whitespace with `cat -A`: none of the trace lines has any.

The case files record the **tree-walker's** rendering and separately assert the
two engines agree, so `both_engines_agree_on_every_case_file` passing is this
crate matching the oracle byte for byte on all five stanzas.

The `operators` header claims every row produces identical bytes on both engines
**before** these operators were promoted as well as after. To keep that true of
the new rows, `compile.rs` and `ir/mod.rs` were restored to `HEAD` with the new
stanzas in place and `both_engines_agree_on_every_case_file` run: `1 passed`.

---

## Every test-adjacent comment, the implementation it excludes, and the evidence

Each mutation below was applied to `compile.rs`, run, and reverted from a
pristine copy.

| # | claim | mutation | result |
|---|---|---|---|
| M1 | golden: "the right one separates the address from the descent ... a compiler stepping into the left child whatever the operand would render `root.L` under both"; `operators`: "an op whose route always stepped into the left operand would land on `1` in `say 1 + length(za)`" | `descend(path, true)` -> `descend(path, false)` in both `native_shape`'s and `push_native`'s right-operand paths | `a_call_promotes_at_the_root_and_below_it` **FAILED**; `both_engines_agree_on_every_case_file` **FAILED** |
| M2 | `operators`: "a route that spelled a prefix's only operand the way it spells a right operand finds nothing there" | prefix arm `descend(path, false)` -> `descend(path, true)` in both functions | `both_engines_agree_on_every_case_file` **FAILED** |
| M3 | golden: "a call that allocated a register for itself would render the same ops and reserve more" | `registers.alloc()` in `push_native`'s `Call` arm | **FAILED** with `reserved 3 registers`, the predicted number |
| M4 | `operators`: "handing every call op in a chunk the same resolution site reddens this stanza" | `site: calls.reserve()?` -> reserve and then write `0` | `both_engines_agree_on_every_case_file` **FAILED** |
| M5 | `operators`: "an echo emitted in front of its call op would print `>F>` ahead of `>A>`" | `Op::TraceFunction` pushed before `Op::CallExpr` | `both_engines_agree_on_every_case_file` **FAILED** |
| M6 | `corpus_shape_tests` module doc: "refusing every node past depth 8 reddens both" | depth-8 guard at the top of `native_shape` | whole workspace, `--no-fail-fast`: exactly **two** failures, `ir::corpus_shape_tests::every_corpus_body_...` and `ir::golden_tests::a_call_nested_past_the_paths_width_...`; every other binary green |
| M7 | `corpus_shape_tests` module doc: "refusing only an address past depth 8 reddens the golden test alone and leaves this file green" | `descend` refuses past 8 steps | whole workspace, `--no-fail-fast`: exactly **one** failure, `a_call_nested_past_the_paths_width_leaves_the_slot_general` |

**One comment was changed because its claim was wrong.** The `operators`
two-calls stanza first said an op handed the other's resolution "answers `44`
for `say length(za) || pos('c', za)` where `43` is right, and `33` for the row
after it". Under M4 every op shares site 0, whose first resolution is `LENGTH`,
so `pos('c', za)` invokes `LENGTH` with two arguments and the row raises rather
than printing `44`. The comment now names the mutation and its measured effect,
and adds the discrimination that is actually load-bearing: a stanza whose calls
were all `length` would pass under M4, which is why the two calls are of
different routines.

**One comment was corrected because this task falsified its measurement.**
`corpus_shape_tests`' module doc said a depth-8 bound "reddens **this and
nothing else in the workspace**". M6 shows it now reddens the new golden test
too. It was rewritten around M6 and M7, which split the bound into its two
halves and say why both witnesses exist.

### "Can fail" is not "adds coverage"

* `a_call_nested_past_the_paths_width_leaves_the_slot_general` **is** a sole
  catcher: M7 reddens it and nothing else in the workspace.
* `a_call_promotes_at_the_root_and_below_it` is **not** a sole catcher for M1 --
  the `operators` stanza catches the same mutation. It is a more direct signal
  (it names the address in the failure) rather than a new one, and its register
  case (M3) is not covered by any output-level row.
* The five new `operators` stanzas are not claimed to be sole catchers. M2, M4
  and M5 were not separately re-run with the file held out; what the file is
  for, per its own header, is pinning oracle bytes, which an engine-against-engine
  comparison structurally cannot do.

---

## What `deep_nested_expr.rex` compiles to

**Unchanged.** 12004 ops in its main body: 3000 `LoadConstant` + 3000
`TraceLiteral`, 2999 `Arith` + 2999 `TraceOperator`, 2 `Clause`, 1 `Load`, 1
`TraceRead`, 1 `Store`, 1 `Say`. No `EvalExpr` and no `Generic`. Byte-identical
histogram at `HEAD` and after this change, measured by the same instrumented
sweep run twice.

`corpus_shape_tests`' independent restatement agrees: `root_of` calls for
`Root::Arith` and the compiled clause ends in one. It agrees **because** the
depth bound is charged only at a call -- under the brief's version this program's
expectation and its stream would both have moved to `Root::EvalExpr`, and the
sweep would have gone on passing while the program got slower.

---

## What in the brief was wrong or stale, and how it was resolved

1. **`native_shape` charging the depth at every node.** Corrected in the plan
   and in the brief, with the `deep_nested_expr.rex` measurement written into
   the plan. See above.
2. **"`push_native` needs `index` and `slot` for the address, so thread them
   through."** It also needs `calls`, for `calls.reserve()?`. Corrected in the
   plan and the brief, together with the note that both functions then need a
   `clippy::too_many_arguments` expectation.
3. **The brief's file list omits `rust/crates/rexx-exec/src/ir/mod.rs`**, which
   has to change: `NodePath::child`'s `expect(dead_code)` must come off or the
   build fails under `-D warnings`, and `Op::CallExpr`'s doc names the deleted
   golden test. Not corrected in the plan -- the file list is generated from the
   plan's own Files section, and `mod.rs` is listed there for Task 3.
4. **The brief does not mention
   `an_operand_that_needs_eval_leaves_the_whole_expression_general`**, which
   this task falsifies as squarely as the test it does name. Deleted, with the
   reasoning in the commit message and above.
5. **The plan's global constraint says `size_of::<Op>() == 12`.** The tree says
   16 (`ir/mod.rs:79`), as the controller stated. Not touched by this task; not
   corrected, because it is a Task 3 fact and correcting it here would collide
   with whatever recorded the widening.
6. **The stack.** Nothing in the brief could have predicted it. Recorded in the
   plan beside the threading instruction, since that is what a future task
   widening these signatures needs to read.

---

## Things I was unsure of

* **Whether to shrink `compile`'s frames instead of enlarging the sweep's
  stack.** Bundling the six emission sinks into one `&mut` struct would save
  roughly what the four new parameters cost -- but the measurement says the
  sweep was already within 128 KiB of the limit *before* this task, so a
  successful bundle would restore exactly the hair's-breadth margin that just
  failed. I judged an explicit stack size better than a fragile pass, and did
  not do the refactor, which the plan did not ask for. If a reviewer disagrees,
  the numbers to argue from are in the table above.
* **`Root::CallExpr` is not in the sweep's anti-vacuity list** while every other
  `Root` variant is. I did not add it: I did not verify that a corpus clause
  actually has a call at the root of its value, and a row asserted without that
  check is the vacuity it is meant to prevent. Worth someone's attention, but
  it predates this task.
* **`DEEPEST_ADDRESSED_CALL = 31` is inert against the current corpus**, because
  no corpus program nests a call that deep. It is a correct restatement and it
  would matter if one ever did; the number itself is pinned by the golden test
  and by `ir::tests::a_node_path_carries_thirty_one_steps_and_refuses_the_thirty_second`.
* **No timing was run**, per the controller's instruction. The claim that this
  promotes `strings.rex`'s and `alloc4c.rex`'s lines is not measured here.
* **The depth divergence** is recorded in the plan, unchanged, with no mechanism
  added. I did not re-run the 100,001-term probe; nothing in this task's work
  bears on it.

---

# Fix round 1

Commit `8a48bbb1df02bb47c3b921f9a1285aee0f990798`. Findings 1-6 addressed;
finding 7 is the controller's and the plan file was not touched.

Gates, unpiped: `cargo fmt --all --check` **0**; `memcap 8G cargo clippy
--workspace --all-targets -- -D warnings` **0**; `memcap 8G cargo test
--workspace --no-fail-fast` **0** -- 82 binaries, 1464 passed, 0
`test result: FAILED`, 0 `overflowed`. The count is unchanged because the round
adds a **row** to an existing test and an assertion to `compile`, not a new
`#[test]`.

## Finding 1 -- the false glosses and the coverage gap

The claim checks out. `zz = .nil || length('a')` puts `ExprKind::DotVariable`
on the left of a native operator, `native_shape`'s `_ => false` arm declines it,
and the whole slot becomes one `Op::EvalExpr` -- with the call's address
perfectly reachable. Confirmed by the new row passing.

Both glosses corrected:

* `ir/mod.rs`, `Op::CallExpr`: the "every call an address reaches" clause is
  replaced by **"An address reaching a call is necessary and not sufficient"**,
  stating that `native_shape` decides for a whole slot at once and that one
  sibling term with no op leaves the call with nothing. It names the three
  tests and says which half each states, rather than saying "three halves" --
  that phrasing was written and then removed in this round for the count rule.
* `golden_tests.rs`: the opening is now "wherever an address reaches it **inside
  a slot that compiles natively**", with a paragraph saying explicitly that this
  test cannot show the other half and naming where that row is.

**The new row and the implementation it excludes.** Row added to
`the_value_shapes_outside_the_native_set_stay_general`:

```rust
// A call beside a term with no op: the address reaches the call and
// the slot still goes general, because the choice is the slot's.
&b"zw = .nil || length('a')\n"[..],
```

It excludes a `push_value` that treats a reachable address as sufficient --
emitting the call's op and then covering the slot with `Op::EvalExpr`, which
runs the call twice and prints its `>F>` line twice. Written as a mutation:

```rust
} else {
    // MUTATION: the address reaches the call, so use it even though the
    // slot as a whole is general.
    if let ExprKind::Binary { right, .. } = &expr.kind
        && matches!(right.kind, ExprKind::Call { .. })
    { /* Op::CallExpr + Op::TraceFunction at root.R */ }
    ops.push(Op::EvalExpr { index, slot, dst });
```

* **With the row**: `the_value_shapes_outside_the_native_set_stay_general`
  FAILED, message `... promoted a value shape outside the native set`.
* **With the row held out and nothing else changed**: `memcap 8G cargo test
  --workspace --no-fail-fast` exited **0**. No failures anywhere.

So the row is a **sole catcher** for that class -- exactly the gap the review
named. That measurement is recorded in the test's own doc comment, not only
here.

## Finding 2

`corpus_shape_tests.rs`'s "Two mutations were applied" is now "Each mutation
below was applied to `compile` and the whole workspace run under it with
`--no-fail-fast`." The numeral is gone rather than renumbered, and the bullets
below are the set.

## Finding 3

Fixed in `corpus_shape_tests.rs` and, above, in this report. The comment now
says calling `compile` from a libtest thread is ordinary and names
`golden_tests` as doing it throughout, and that what is unusual is the **depth
of the body**, which is why this sweep is the one that overflowed.
`INTERPRETER_STACK_BYTES` is justified as the size `on_interpreter_thread`
gives the same walk in production, which is a separate claim and still true.

## Finding 4 -- the missing adjacency assertion

`assert_call_echoes_follow_their_op` added in the shape of its four siblings and
called beside them in `compile`. It checks the position, the register **and**
the whole address (`index`, `slot`, `path`), because `>F>` carries no operator
and what it carries instead is the node `trace_intermediate` reads to pick its
tag.

Verified it can fail, both ways:

| mutation | result |
|---|---|
| `Op::TraceFunction` pushed in front of `Op::CallExpr` | panics: `the function echo at 1 does not follow the call whose address and register it names ...` |
| echo pushed with `path: NodePath::ROOT` instead of the call's own | panics: `the function echo at 2 does not follow ...` |

The second is the one worth noting: a root-addressed echo is **correct** for a
call at the root and wrong only for a nested one, so it is precisely the failure
this task's widening made possible, and no ordering check would see it.
`Op::TraceFunction`'s doc now cites the assertion the way `Op::TraceOperator`
and `Op::TracePrefix` cite theirs.

## Finding 5 -- the two things that can drift

* `DEEPEST_ADDRESSED_CALL` now carries the cost of its own independence in its
  doc: only a corpus program nesting a call deeper than it can falsify the
  number, so a `NodePath` whose width moved would redden
  `ir::tests::a_node_path_carries_thirty_one_steps_and_refuses_the_thirty_second`
  and the width golden test while this constant went on calling for
  `Root::EvalExpr` at the depths the wider address had just reached.
* `Root::CallExpr` **is now in the anti-vacuity list**, and this is a small
  deviation from the instruction, taken only after removing the reason for it.
  The instruction was not to add an *unverified* row; I verified it first, by
  adding the row and running the sweep alone -- `1 passed`. So the corpus does
  reach a call at the root of an assignment's or a `SAY`'s value, and
  `root_of`'s `Call` arm does fire. Adding it also repairs the block's own
  claim, which says "every expression root a promoted clause can end in is
  reached by this population at least once" and was false for this one variant.
  Revert is one line if the controller prefers the documented-gap form.

## Finding 6

Three repairs in `ir_dual_cases/operators`:

* the 125-character line rewrapped, and the paragraph around it rewrapped with
  it so the break falls at a sentence;
* "The `trace i` row carries the abuttal" -> "The `trace i` row **for the binary
  operators** carries the abuttal";
* "The rows separate them by condition number **under one trap**" -> "**each
  under a `SIGNAL ON SYNTAX` of its own**", which is what the stanza has.

## What else I re-read, since a correction round is where false statements are born

Every doc comment I touched was re-read whole, with the paragraph either side.
Four things were found and fixed that no finding named:

* "are what state the three halves of that" -- a count, in a sentence I wrote
  this round. Replaced by naming what each test states.
* "**What varies across the cases below is the address and nothing else**" --
  false of the register case in the same test. Replaced.
* "**The last row holds a call**" -- positional, and it rots the moment a row is
  appended. Replaced by "One row holds a call", with the row's own inline
  comment identifying it.
* "the two pushes below the call arm" in the new assertion's doc -- the pushes
  are *in* the arm, not below it. Replaced by naming `push_native`'s call arm.

One more, in `corpus_shape_tests.rs`: "Nothing this sweep runs can falsify the
number" is a claim about the current corpus. Reworded to the mechanism -- "only
a corpus program nesting a call deeper than this can falsify the number" --
which cannot rot.

## Nothing I disagree with

All six findings are correct as stated. Finding 1 is a real defect of mine and
its coverage half was the one I did not see: I checked that each *new* comment's
discrimination held, and did not check whether an *existing* sentence I was
extending had grown wider than the code. The row that closes it is a sole
catcher, so the gap was real and not theoretical.
