# Task 3 report: the borrow-shape spike

Status: **DONE.** Three commits:

* `5b5ccaf6` the spike itself,
* `41378f46` `cargo fmt`, separate as the constraint requires,
* `2838974d` two fixes from the team lead's first review pass: the loud-failure
  message was dumping the whole AST, and the `E0502` claim was prose that only
  said it had been compiled,
* `5244d6f4` the formal review's three behaviour findings: `StackSpan`'s two
  ends could come from different call chains, `Loud::instruction` could abort
  instead of failing loudly, and `run_activation`'s activation-stack assumption
  was unrecorded and unchecked,
* `27788d94` the review's comment items, plus the one of them that turned out to
  need code: every `eval` result is now rooted, inside a per-clause temps frame,
* `17089933` a re-measurement forced by `0f33843a` making the parser's tree
  walks iterative, which invalidated this report's phase table and `6f8831af`'s
  correction of it,
* `f4c59faa` the second review round's doc items: what the doctest pair is
  worth and what it cannot see, and what generalises the `form_name` test
  beyond the one form it exercises.

BASE was `87fe09d8`.

Tests: **10 integration tests plus 2 doctests in `rexx-exec`, all passing.**
Workspace 592 passed / 0 failed / 3 ignored with `RUST_MIN_STACK` raised;
581 / 0 / 3 with `--exclude rexx-exec`, so this crate contributes exactly 11 and
the count does not drop. On the *default* test stack the workspace still aborts
in `rexx-parse`'s corpus walk, which is the Phase 3 drop defect the team lead is
scheduling separately and is reproduced with `--exclude rexx-exec`.

The shape holds. It compiles, it survives a fragment created mid-instruction,
and the variable pool is in it. Two things the design said were not true, and
two things nobody had checked at all, are in "Findings" below.

## Pre-flight reading

`task-3-brief.md`; the plan's Task 3 and its global constraints; the spec's
D16, D17, D19, "The borrow shape", "Crate layout", "Output and trace sinks",
"Failing loudly"; `rexx-parse/src/lib.rs` (`Program`, `Fragment`,
`parse_program`, `parse_interpret`), `ast.rs` (`CodeBody`, `Instruction`,
`InstructionKind`, `Expr`, `ExprKind`), `token.rs` (`SymbolId`, `SymbolTable`);
`rexx-core/src/lib.rs`, `body.rs`, `roots.rs`, `heap.rs`; Task 1 and Task 2
reports and `progress.md`.

Five questions went to the team lead before any code was written. Four were
answered by spec revision 7 (`e9485185`) and commit `efe5d2d2` while the work
was in flight; the implementation matches all of them. The fifth is recorded
below with a correction I got wrong twice and have to own.

## Step 1: the borrow discipline

`rust/crates/rexx-exec/src/lib.rs`, `Interp::run_activation`. The whole claim:

```rust
let program = Rc::clone(&self.activation().program);
let plan = Rc::clone(&self.activation().plan);
let code = Code { body: &program.main, symbols: &program.symbols, slots: &plan.by_symbol };

while let Some(instruction) = code.body.instructions.get(self.activation().pc) {
    match self.step(&code, instruction)? { ... }
}
```

`code` borrows two locals, so `self.step`, which takes `&mut self`, has nothing
to collide with.

The version that does not compile is in that function's doc comment, and the
`E0502` beside it is **compiled, not paraphrased**. I wrote the wrong version
into the file, built it, pasted rustc's output, and deleted the function again:

```text
error[E0502]: cannot borrow `*self` as mutable because it is also borrowed as immutable
   --> crates/rexx-exec/src/lib.rs:851:13
    |
849 |         let body = &self.activations.last().expect("a live activation").program.main;
    |                     ---------------- immutable borrow occurs here
850 |         while let Some(instruction) = body.instructions.get(self.activation().pc) {
    |                                       ----------------- immutable borrow later used here
851 |             self.step_wrong(body, instruction)?;
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
```

**The second underline is the interesting one and it is not what the brief's
sketch predicted.** The brief's expected diagnostic blamed the argument
(`immutable borrow later used by call`). The real one blames the **loop
condition**. That difference matters for the next person: it means passing the
body to `step` some other way is not the fix. I checked that rather than
asserting it, by compiling a second wrong version whose `step_wrong_noargs()`
takes no arguments at all. Identical `E0502`, same second underline. What has
to change is where `body` is rooted, not who it is passed to, and `Rc::clone`
is the only thing that changes it. Both facts are in the doc comment.

A quieter second demonstration is in `step`'s `Assignment` arm: `let name =
code.symbols.name(*id).as_bytes()` survives the `self.eval(...)` call that
follows it, because it borrows the local `Rc` and not `self`. The same slice
read out of `self` would not.

## Step 2: a fragment created mid-instruction

`Interp::run_fragment`. `parse_interpret` at run time, the resulting `Rc<Fragment>`
a local that outlives the nested loop, the body executed inside the current
activation with **no frame pushed**.

The nested loop's program counter is a local `usize` rather than the
activation's, because the activation's is sitting on the `INTERPRET` instruction
and has to still be there afterwards. That is only sound because a fragment can
never jump, which is Task 1's measured 47.1: a label inside `INTERPRET` text is
an error, so `Fragment::body.labels` is always empty. The `Flow::Goto` arm of
the nested loop is an `unreachable!` carrying that reason.

`INTERPRET` the keyword still fails loudly through `run_program`, with exit 120
and the fragment not run, and there is a test for exactly that. The fragment
path is reached only through `run_program_interpret_spike`, whose doc comment
says 4b deletes it. The alternative I rejected, and recorded in that doc
comment: hooking the fragment onto an innocent instruction such as `NOP` proves
the same lifetime while lying about which node owns it and leaves nothing for 4b
to delete.

Oracle transcripts captured before the tests were written, all under
`( ulimit -v 1048576; build/bin/rexx FILE )`:

```rexx
zzz = 'from the enclosing frame'
interpret "say zzz"
interpret "zork = 42"
interpret "say zork"
zzz = zzz || '!'
interpret "say zzz"
```
```text
from the enclosing frame
42
from the enclosing frame!
rc=0
```

```rexx
say 'before'
interpret "say 'inside'"
interpret "exit"
say 'after'
```
```text
before
inside
rc=0
```

The second one is worth keeping: `EXIT` inside a fragment ends the **program**,
not the fragment, so control leaves the nested loop and the enclosing one
together and both `Rc` locals drop in order. `rexx-exec` reproduces both byte
for byte.

### What the fragment case did not prove

Worth being explicit, because "the spike runs a fragment" reads as covering more
than it does.

* **Nesting.** No fragment here runs an `INTERPRET` of its own, so `run_fragment`
  has never actually been reentrant. The shape is built for it -- each level
  would clone its own `Rc` into its own local, which is the same argument one
  level down -- but the argument is by construction, not by execution. 4b runs
  it for real.
* **A fragment under a live loop.** The enclosing body here is straight-line.
  A fragment created inside a `DO` body, with block state on the activation's
  stack, is Task 11's shape and is not exercised.
* **`LEAVE`, `ITERATE` or `SIGNAL` crossing the boundary.** The nested loop only
  ever sees `Flow::Next` and `Flow::Exit`. `Flow::Goto` is `unreachable!` there
  on the strength of Task 1's 47.1 measurement, which covers labels *inside* the
  fragment and says nothing about a fragment jumping to an enclosing label. 4b
  owns `SIGNAL` and has to answer that separately.
* **A fragment that fails.** `parse_interpret` returning `Err` takes the loud
  path, which is right for 4a and is not what the oracle does; Task 12 owns the
  real reporting, including the fact that a `ParseError`'s byte offset is a
  position inside the one-line fragment and not inside the program.
* **Collection during a fragment.** `Heap::alloc` does not collect on its own
  yet, so the fragment's values have never survived a real collection. The
  activation's slots are in `RootSet` and therefore in `iter()`, so the
  reachability is right by construction, but nothing has tested it under a
  collector that runs.

## Step 3: the sized thread, and the numbers Task 11 needs

The thread is in `rexx_exec::run_program`, the **public entry point**, not in
`src/bin/rexx-run.rs`. The binary reads bytes, calls the entry point, writes
what comes back. Everything from `parse_program` onward runs on the sized
thread, which is not a stylistic choice: `Rc<Program>` is `!Send`, so a program
parsed on the caller's thread cannot be handed across.

**Stack size: 512 MiB** (`INTERPRETER_STACK_BYTES`). Reserved address space, not
resident memory; Linux commits stack pages on first touch.

**Per-frame cost: 784 bytes per `eval` level in a debug build, 192 in release.**
`x86_64-unknown-linux-gnu`, rustc 1.96.1, measured over a 100,000-term left-deep
expression. Debug is the number to size against, because that is what `cargo
test` runs and therefore what the in-process harnesses sit on.

**Whole-pipeline budget: roughly 685,000 levels in debug, about 783 bytes per
level**, bisected independently of the probe and agreeing with it to 0.2 per
cent. This figure moved twice; see "`eval` is not the recursion that binds" and
its correction below, and re-run the bisection rather than trusting any table
in this report, including the current one.

**Recommended depth limit for Task 11: exactly 100,000, raising 11.1 above it.**
The reasoning, since inheriting it beats re-deriving it:

* It is the largest depth the oracle is *measured* to survive (D19: 100,000
  terms prints `100000` and exits 0; 150,000 and 200,000 both exit 139). Any
  limit above 100,000 buys the reproduction of nothing, because there is no
  measured oracle behaviour up there to reproduce, and it widens the window
  where we succeed and the oracle SIGSEGVs, which criterion 1 reports as a
  failure.
* It sits about six and a half times below the ~685,000 the stack survives in
  debug, which leaves the real `eval` room to grow several times past the
  spike's before the margin is gone.
* Off by one matters here: a 100,000-term expression *reaches* depth 100,000, so
  the check has to raise above 100,000 and not at it, or the one depth the
  oracle is known to handle becomes the first one we refuse.

The measurement is a test, `records_the_stack_cost_of_one_eval_frame`, not a
figure I wrote down once. It runs `say 'a'` followed by 99,999 repetitions of
`||''` (the concatenation analogue of D19's `1 + 1 + … + 1`, chosen because it
needs no arithmetic and so no `rexx-num` dependency yet), asserts the oracle's
answer `a\n` and rc 0, asserts the depth reached is exactly 100,000, and prints
both numbers under `--nocapture`. It measures `eval` itself rather than a
replica, by taking the address of a local at the first level and at the deepest,
both inside the same function so the frames above `eval` cancel. Taking a raw
pointer and casting it to `usize` is safe code; there is no `unsafe` anywhere in
this crate and the workspace forbids it.

**Three recursions ride on this stack, not one,** and the third is easy to miss:

* `eval`, once per term,
* `Plan::note`, walking the same expression to assign slots,
* and **`Drop` for the AST**, which recurses once per `Box<Expr>` level when the
  `Rc<Program>` goes. Nothing in the source says "recursive"; the derive does.

The 784 figure covers only the first. The test is deliberately a whole run so
that passing at this size is what says the other two fit alongside it.

For Task 11: the depth limit has to be at least 100,000 (D19) and below what
this stack survives (~684,000 debug). Both bounds are checked by assertions in
that test against `INTERPRETER_STACK_BYTES` itself rather than against the
comment describing it, so raising the limit without raising the stack fails the
test.

## The review round

Three things, all from the team lead reading the committed spike. Commit
`2838974d`.

### The loud-failure message dumped the whole AST

Mine, and a real defect rather than a tidiness one. `Loud::expression` wrote
`{kind:?}`, and `ExprKind`'s derived `Debug` walks the tree, so one clause of
`corpus/lang/deep_nested_expr.rex` produced **373,332 bytes** of stderr
beginning `Binary { op: Plus, left: Expr { kind: Binary { op: Plus, left: ...`.
Now 128 bytes: `rexx-exec: the operator `+` is not implemented: ...`.

The fix is a `form_name(&ExprKind) -> String` whose every arm returns a
`&'static str` or one `Operator::spelling`, so no input can make the answer
long, and whose match is exhaustive with no `_` arm so a new variant is a
compile error rather than a silent "unknown". The two operator forms name the
operator, because "a dyadic operator is not implemented" does not tell a reader
which one to go and implement.

Why it mattered past size: failing loudly is a gate criterion, criterion 1
compares stderr byte for byte, and every later task inherits this path. The
instruction-side message was already bounded (`InstructionKind::keyword()` is
`&'static str`) and so is the parse-error one, so this was the only site.

The test asserts the property rather than the byte count: the same unimplemented
form gives the **same message** for a 1-term and a 3,000-term expression, plus a
loose 300-byte bound. Asserting a byte count would have to be updated every time
the wording changes, and would then be enforcing the wording rather than the
bound.

### "Compiled here rather than paraphrased" was prose, and now is not

The team lead's question was the right one: a doc comment cannot fail to
compile, so either the claim is backed by a `compile_fail` doctest or it is
prose that only says it was compiled. It was the latter. I did compile it, by
hand, and pasted rustc's real output, but **nothing in the tree would have
noticed the claim going stale.**

There are now two doctests on `run_activation`, and `cargo test` runs both
(private-item doctests are collected, checked rather than assumed). A miniature
of the same borrow using only `std`: one version that must compile, and the same
miniature **one line different** that must not.

What that is worth, stated precisely because the attribute invites overclaiming:

* `compile_fail` proves only "this does not compile", not "this fails with
  `E0502`". The `compile_fail,E0502` spelling looks like it pins the code and
  does not. **Measured on rustc 1.96.1**: a doctest annotated
  `compile_fail,E0502` whose body is `let x: u32 = "not a u32";` passes, and
  that is `E0308`. So a `compile_fail` snippet with a typo passes for the wrong
  reason.
* What narrows it is the twin that must compile. The two share everything but
  one line, so breakage in the shared part fails the passing twin instead of
  silently satisfying the failing one. **Checked by mutation, not assumed:**
  rewriting the passing snippet's `Rc::clone` line into the failing snippet's
  shape makes it fail, with `E0502`.
* What the pair still cannot catch is a typo confined to the failing snippet's
  own one different line. Nothing available here closes that.

The verbatim rustc output stays in the doc comment, now labelled as captured by
hand rather than implied to be enforced.

### `eval` is not the recursion that binds the stack budget -- and then it was again

**Read the correction at the end of this section before using any number in
it.** The measurement below was right when taken and describes a tree that no
longer exists.

The team lead asked whether the measurement separates `eval` from the AST's
`Drop`. It did not: the probe is inside `eval`. So I separated them, by choosing
programs that reach different phases. `exit` as the first instruction leaves a
deep expression parsed and dropped but never evaluated; a bare command clause is
skipped by `Plan::build` as well, leaving parse and drop alone. Bisected on
`rexx-run`, debug, 512 MiB:

| what runs | deepest surviving | implied bytes per level |
|---|---|---|
| parse and drop only | 600,000 to 700,000 | about 820 |
| parse, plan and drop | 600,000 to 640,000 | about 860 |
| all four, including `eval` | 600,000 to 640,000 | about 860 |
| `eval` alone, probed directly | n/a | 784 |

**Adding `eval` does not move the cliff.** The phases are sequential and each
unwinds before the next, so what binds is the compiler-generated `Drop` for the
`Box<Expr>` chain at about 820 bytes per level, slightly *more* than `eval`'s
784. Sizing against 784 alone is optimistic by roughly ten per cent; the honest
budget is about 860.

**And a depth limit on `eval` does not close the abort path.** `exit` followed
by a 700,000-term expression aborts in the drop with nothing evaluated, and no
counter in `rexx-exec` is in a position to see it. That is the same Phase 3
defect the team lead is scheduling, seen from the interpreter's side: it is not
only that `rexx-parse`'s own tests overflow on a 2 MiB thread, it is that
`rexx-exec` cannot guard against it either.

### The correction: after `0f33843a`, `eval` binds again

`0f33843a` made `Expr`'s `Drop`, `block.rs`'s `visit_expr` and the gate walk
iterative. That removed the recursions the table above was measuring, so every
figure in it is obsolete, and so is `6f8831af`'s 850-bytes-per-level correction
of it. Re-bisected on the current debug binary, to within 2,000 levels
(commit `17089933`):

| what runs | deepest surviving | bytes per level |
|---|---|---|
| parse and drop only | over 4,000,000, no cliff found | under 134 |
| parse, plan and drop | 3,354,442, fails at 3,356,347 | about 160 |
| all four, including `eval` | 684,618, fails at 686,523 | about 783 |

So **`eval` binds, at the 784 the probe inside it always reported.** The
bisected 783 and the probed 784.0 are two different methods agreeing to 0.2 per
cent, which is a better position than either alone. About **685,000 survivable
levels**, so a limit at 100,000 keeps roughly six and a half times the headroom.
**The recommendation of 100,000 is unchanged; only the reason is.**

Two things worth carrying forward.

`Plan::note` is now this crate's own remaining recursion, at about 160 bytes per
level. Nowhere near binding, and a candidate for the same explicit-worklist
treatment `rexx-parse`'s walks got.

And the earlier table was wrong in a way that was visible **without**
re-measuring, which is the part I should have caught. Dividing a 100,000-wide
bracket gave parse-and-drop a *smaller* per-level cost than
parse-plan-and-drop -- adding a phase appeared to make each level cheaper,
which no model of sequential phases produces. The incoherence was the tell, and
I published the table without noticing it. Quote the bisected cliff, not a
per-level number divided out of a coarse bracket.

## Findings

### 1. The design's fragment plan, prove-or-disprove (the team lead's item 3)

Answered before implementing and confirmed in code. Spec revision 7 (`e9485185`)
adopted both halves.

**(a) "Built against the enclosing plan's name map" is sufficient for reads and
insufficient for writes.** A fragment that introduces a name the enclosing body
never mentions has to bind that name to a slot, and the binding must outlive the
fragment. Measured:

```rexx
/* ZORK appears in no instruction of the enclosing body */
interpret "zork = 42"
interpret "say zork"        /* prints 42 */
```

The enclosing plan is an `Rc` the activation holds a clone of, so it is not
uniquely owned and cannot be extended, and `RootSet::grow_slots` hands out the
*slot* while recording no *name* for it. `Activation::extra:
HashMap<Box<[u8]>, usize>` is where the name goes; resolution is
`plan.names.get(name).or_else(|| extra.get(name))`, and allocation writes `extra`
and calls `grow_slots`. `DROP (v)` has the identical hole, so this constrains
Tasks 6 and 9 and not only fragments.

**(b) The `(enclosing body, fragment id)` cache key is sound and useless.** A
fragment is re-parsed on every execution and its text can differ per iteration,
so a fragment id can only be a per-parse counter: every lookup misses, every
entry is retained, and `do 1000000; interpret s; end` accumulates a million
plans read zero times each. Since (a) moves the durable state to the activation,
a fragment's plan is genuinely ephemeral. `BodyKey` in the code has **no
fragment arm**, and its doc comment says why, which carries the finding better
than a variant nothing constructs would.

**(c) `grow_slots`'s top-frame assertion holds for the fragment case** exactly as
D16 predicted, since a fragment runs in the creating activation and pushes no
frame. No change needed there.

### 2. Nobody guards the parser's recursion, and it aborts where the oracle reports

This is the one I would most want read. D19 puts a depth counter on `eval` and
says the failure mode it excludes is "a native stack overflow, which aborts with
no message and no exit code". **Nested parentheses recurse in `rexx-parse`, and
nothing counts that recursion.** D19 flags `rexx-parse`'s exposure as
"unverified" and leaves it as a check in 4a's plan; here is the measurement.

`say ((((…'a'…))))`, both sides, `( ulimit -v 1048576; … )` for the oracle and
the debug `rexx-run` on its 512 MiB thread:

| nested parens | oracle rc | rexx-run rc (debug) |
|---|---|---|
| 20,000 | 0 | 0 |
| 35,000 | 0 | 0 |
| 40,000 | **245** (Error 11.1) | 0 |
| 80,000 | 245 | 0 |
| 85,000 | 245 | 0 |
| 90,000 | 245 | **134** (SIGABRT) |

Three things follow.

* **`rexx-parse` survives 20,000 nested parens.** D19's named check, done. It
  survives 85,000 on the sized thread in debug, and 300,000 in release.
* **From 40,000 up we succeed where the oracle raises 11.1 at rc 245.** That is
  an exit-code divergence criterion 1 reports as a failure, and it is precisely
  D19's "a generously sized stack is a *divergence*" hazard, arriving through
  parentheses rather than through terms. D19 states the corpus rule against the
  *term* cliff (between 100,000 and 150,000) and there is no rule at all for the
  paren cliff, which sits between 35,000 and 40,000 and is an order of magnitude
  lower.
* **At 90,000 we abort with SIGABRT and "fatal runtime error: stack overflow",**
  where the oracle prints a two-line 11.1 and exits 245. `eval`'s depth counter
  cannot prevent this: the overflow happens inside `parse_program`, before the
  interpreter exists.

The good news is that the fix is not a new deviation. The oracle answers deep
parentheses with the **same 11.1** the plan already assigns to evaluation depth,
so a depth counter in `rexx-parse`'s subexpression recursion raising 11.1 makes
both cliffs report the same condition. I have not written it: it is `rexx-parse`
surface and belongs to Task 11 or its own task, and it needs the oracle's exact
paren cliff pinned down first.

### 3. `SymbolId` cannot index an array, so the hot path is a hash

D16 wants slots reachable by the id the AST already carries. `rexx_parse::SymbolId`
is a newtype over a **private** `u32` with no accessor, so nothing outside
`rexx-parse` can turn one into a `Vec` index. `Plan::by_symbol` is therefore a
`HashMap<SymbolId, usize>`, and variable lookup is 8.1% of runtime on the mixed
benchmark and 32.2% on stem-heavy code. Task 6 either adds `SymbolId::index()`
to `rexx-parse` and gets an array index, or keeps the hash deliberately. Recorded
so the choice is made rather than inherited.

### 4. My Q5 was right, then I "corrected" it wrongly. Both messages were sent.

I reported that `rexx-core/src/lib.rs` exported neither `NotNumeric` nor
`SlotFrame`, then re-read the file, found both present, and sent a correction
saying I had read a stale copy. **The correction is the wrong one.** At my BASE:

```text
$ git show 87fe09d8:rust/crates/rexx-core/src/lib.rs | grep 'pub use body\|pub use roots'
21:pub use body::{BehaviourId, Body, Object};
24:pub use roots::{FrameId, RootSet, ...}   <- SlotFrame absent
```

The lead's commit `efe5d2d2` "Export NotNumeric and SlotFrame from rexx-core"
landed between my two reads, in response to the original question. So the gap
was real, the fix was needed, and `rexx-exec` would not compile without it: it
names `SlotFrame` in `Activation`. Recording it here because the correction
message may otherwise read as "that work was unnecessary", and it was not.

### 5. Two workspace failures that are not this task's

`cargo test --workspace` fails at HEAD, and it fails identically with
`--exclude rexx-exec`. Both come from `7f8f6922` "Add Phase 4a control-flow
corpus and the phase-4a.txt subset list", which landed while this task was in
flight. `rexx-parse` has no dependency on `rexx-exec`, so neither can be mine.

* **`rexx-parse` aborts on `corpus/lang/deep_nested_expr.rex`.**
  `every_corpus_lang_program_parses` and
  `the_corpus_exercises_at_least_one_directive_with_a_body` both parse it, and a
  3,000-term AST does not fit a `cargo test` thread's default 2 MiB stack. It is
  a pure stack-depth failure: `RUST_MIN_STACK=67108864 cargo test -p rexx-parse
  --test program` passes. The file's max paren nesting is 1, so the recursion
  consuming the stack is the left-deep `Box<Expr>` chain's `Drop`, not the
  parser. **This is finding 3's third recursion, biting a real test on day one,**
  and it is also D19's own argument for the sized thread arriving from an
  unexpected direction: `rexx-parse`'s tests parse deep programs and have no
  sized thread to do it on.
* **`sourceline_matches_the_interpreter_for_every_corpus_program` failed**: no
  `crates/rexx-parse/tests/sourceline_oracle/exit_with_value.txt` for the new
  `corpus/lang/exit_with_value.rex`. **Fixed by someone else since**; the
  workspace is green on a raised stack as of the final verification below. The
  stack-overflow one is not fixed and is now its own scheduled task.

## What did not work

The parts that never reach a commit, in the order I hit them.

**A `Drop` guard for the depth counter.** The obvious way to keep `eval`'s depth
correct across the `?` early returns is a guard whose `Drop` decrements. It
cannot be written here: the guard would have to hold `&mut self` for the
duration, which is exactly the borrow the recursive call needs. Hence the
`eval`/`eval_node` split, where the outer function owns the bookkeeping and the
inner one does the work. That split is also why the measured 784 bytes is a
*pair* of frames rather than one, which is the conservative direction.

**`.unwrap_or_else(std::panic::resume_unwind)` on the thread join.** Reads
perfectly and does not compile: `E0271`, "expected `resume_unwind` to return
`Outcome`, but it returns `!`". A function *item* passed by name does not get
the `!`-to-`T` coercion that the same call written inline would. A `match` on
`join()` was the fix.

**`kind.keyword().unwrap_or(match kind { … _ => unreachable!() })`** in the loud
message. Caught by reading rather than by the compiler, which would have taken
it: `unwrap_or`'s argument is evaluated eagerly, so this panics on every
instruction that *does* have a keyword, which is most of them. Restructured as a
`match` on the `Option`.

**Iterating the fragment's `HashMap` to allocate its enclosing slots.** Correct
and non-deterministic: two names introduced by one fragment would get their
enclosing slots in whatever order the map iterated, differing run to run.
Nothing observable depends on that today, which is the reason to fix it before
something does. Walk order is now recovered from the plan's own local slot
numbering, which is assigned in walk order.

**My first answer to the fragment question was that a fragment needs no plan at
all.** If names resolve through the activation by text, the fragment's plan
collapses into `slot_of` and disappears. That is wrong for the hot path: D16
wants evaluation to reach a slot through the `SymbolId` the AST already carries,
and a fragment's ids are its own table's, so it genuinely needs its own
`SymbolId`-to-slot map even though it needs no *name* map. The finding that
survived is narrower than the one I started with, and better for it.

**`Vec<Option<usize>>` indexed by `SymbolId`,** which is what D16's "integer
slots" wants. Blocked: `SymbolId` wraps a private `u32` with no accessor. Became
finding 3.

**Establishing that the workspace failures were not mine.** The obvious moves
were both wrong. `git stash -u` would have stashed another agent's uncommitted
corpus work, and moving their untracked files aside risked colliding with a live
agent mid-write. What worked was `cargo test --workspace --exclude rexx-exec`,
which reproduces both failures with my crate not built at all, plus the
structural argument that `rexx-parse` does not depend on `rexx-exec`. Cheaper
than the `git archive` copy I had started planning.

## Verification

All from `rust/`, at `17089933`:

* `cargo test -p rexx-exec` -> **10 integration tests plus 2 doctests, 0
  failed.**
* `cargo clippy -p rexx-exec --all-targets -- -D warnings` -> clean.
* `cargo clippy --workspace --all-targets -- -D warnings` -> clean.
* `cargo fmt -p rexx-exec -- --check` -> clean.
* `cargo test --workspace` -> **595 passed, 0 failed, 3 ignored**, and now on
  the **default** test stack as well as with `RUST_MIN_STACK` raised. The abort
  in `rexx-parse`'s corpus walk that this report recorded earlier is gone,
  fixed by `0f33843a`.
* `rexx-run` exercised directly on `hello.rex` (prints `hello`, rc 0),
  `frag.rex` (loud failure, rc 120, "INTERPRET is not implemented"), the
  100,000-term `deep.rex` (prints `a`, rc 0, matching the oracle), and
  `corpus/lang/deep_nested_expr.rex` (128 bytes of stderr, down from 373,332).

One process note, because it cost a bad commit: `cargo clippy … | tail -3` in an
`&&` chain reports `tail`'s exit status, not clippy's, so a clippy failure sails
through the chain. I committed code that failed `-D warnings` that way and had
to amend. Gate on the command directly, or on `${PIPESTATUS[0]}`.

One clippy fix was needed during the work: `collapsible_match` on the plan
pass's `Say { expression }` arm, rewritten as `Say { expression: Some(expression) }`.

## Deviations from the brief, and choices it left open

* **`rust/Cargo.toml` was not modified.** The brief lists it under "Files", but
  `members = ["crates/*"]` already globs, so creating the directory is the whole
  change. `rust/Cargo.lock` did change (eight lines, the new package and its two
  path deps) and is staged with the crate, following Task 2's precedent.
* **`NOT_IMPLEMENTED_EXIT = 120`**, chosen because Task 12 has not chosen one yet
  and Step 1 needs it on day one. Outside 157..=253 where `256 - major` lives,
  below 126 so it cannot be read as a shell's `128 + signal`, and not 0, 1, 2,
  126 or 127. A test asserts the band. Its doc comment names Task 12 as the
  owner of the final value.
* **`Outcome` carries the sinks** (`stdout`/`stderr` as `Vec<u8>`) because
  `run_program(text) -> Outcome` has no sink parameter and because the value has
  to be `Send` to cross the `join()`. D17's "their relative interleaving is not
  observable" is the licence. The cost, recorded in the type's doc comment: a
  long-running program buffers rather than streams. Nothing in 4a's corpus does
  that; Task 14 should know it anyway.
* **The trace sink is a field nothing writes.** `Interp::trace` becomes
  `Outcome::stderr` and only the loud-failure path appends to it; the `*-*` and
  `>>>` lines are Task 13's. It exists now because the design puts both sinks
  on `Interp` and D17 keeps them separate for a measured reason, and because
  the loud path then already writes to the right buffer rather than being
  rerouted later, which is when a stray ordering difference would appear.
* **Only two dependencies**, `rexx-core` and `rexx-parse`. The design names
  `rexx-num` and `rexx-inventory` too; they arrive with Tasks 4, 8 and 12, and
  adding them unused now would be noise.
* **Everything is in `src/lib.rs`.** The brief's file list names it, and the
  design's per-concept layout (`value.rs`, `plan.rs`, `activation.rs`, `eval.rs`,
  `run.rs`, `trace.rs`, `error.rs`) is what Tasks 4 to 13 create. The module doc
  says so.
* **`EXIT` with no expression is implemented**, four lines, because it is what
  constructs `Flow::Exit` and it is what makes the measured `interpret "exit"`
  transcript testable. `EXIT` *with* a result is Task 9's, together with the
  mapping from that result to a process exit code, and takes the loud path.
* **`Novalue`** is in the read path from the start rather than retrofitted,
  because D16 says explicitly not to retrofit it. The spike reads the flag and
  does nothing with it, which is the correct amount of nothing until 4b's
  `SIGNAL ON NOVALUE`.
* **The temps discipline is in `eval`'s binary arm** (`push_frame`/`push_temp`/
  `pop_frame`) even though `Heap::alloc` does not currently collect on its own,
  with a comment saying exactly that. Establishing it now is cheaper than
  finding the day allocation starts collecting.
