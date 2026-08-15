# The two-level driver: checking five claims about flattening `run_ops`

Read-only structural analysis, 2026-08-12. Every file quoted was read at `HEAD`
(`git show HEAD:<path>`) because `ir/*.rs` and `run.rs` were being edited live in
the same tree. Where the working tree differs it is said so explicitly.

Every "this compiles" / "this does not compile" statement below was established
by writing the program, running `rustc --edition 2024` against it, and reading
the exit status. The probes are in
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/drivestudy/`
(`p1.rs` .. `p5.rs`). Toolchain: `rustc 1.97.1 (8bab26f4f 2026-07-14)`,
edition 2024, workspace `rust-version = "1.96.1"`.

---

## Summary of verdicts

| # | claim | verdict |
|---|---|---|
| 1 | per-clause work survives any loop shape | **confirmed in substance, two items misfiled** |
| 2 | `break 'region Err(_)` is a *compiler-enforced* single exit | **refuted** — nothing enforces it today; and a flat loop could carry a *stronger* guarantee |
| 3 | `header` becomes stale-readable loop state | **confirmed, and understated** — it carries an `ObjRef`, so it is a GC hazard, not a wrong number |
| 4 | the hoisted `clause` licenses skipping a per-op body lookup | **confirmed, with the invariant stated precisely** |
| 5a | forward skip on the slice iterator | **confirmed it compiles** — needs `while let`, `nth`, a relative count; `advance_by` is unstable |
| 5b | a `pc` over the region slice only | **confirmed, and it is the smallest change that gets the whole benefit** |
| 6 | any construct needing a *backward* in-region jump? | **no** |
| 7 | forced vs incidental | separated below |

And the finding that was not asked for and that changes the decision:

* **`&` and `|` do not short-circuit in this language.** Half of the stated
  motivation for flattening does not exist.
* **The comma list can be compiled with region-ending jumps only**, i.e. inside
  today's two-level shape, with no driver change at all.
* **Neither the corpus nor any bench program exercises the construct** in a way
  that could move a measured axis.

---

## 1. Per-clause work

### What `enter_stepped_clause` actually does

`run.rs:4732`. In order:

1. `self.activation_mut().clock_stale = true;`
   `DATE`/`TIME`'s per-clause cache invalidation. Mirrors
   `RexxActivation::run`'s `settings.timeStamp.valid = false` right after
   `nextInst->execute()` returns (`RexxActivation.cpp:647`, cited in the code).
   **Genuinely per-clause and forced**: two `DATE()` calls in one clause must
   agree; in two clauses they need not. Per-op would break the first half;
   per-construct would break the second.
2. `let stepped = self.activation().trace_entry.stepped(); … = stepped;`
   The `>I>` "am I still on the first instruction" decay. The C++ clears its
   flag at the bottom of its instruction loop (`RexxActivation.cpp:657-659`),
   where an `IF` then-clause and a `DO` body clause are each a separate
   iteration. **Per-clause, and this function is the only site that counts
   clauses at every nesting depth** — the code records a measurement for the
   version that lived in `run_activation`'s loop instead
   (`if 1=1 then trace l` as a routine's first clause announced the pair here
   and nothing on the oracle).
3. `let indent = self.printed_indent(code, index); self.clause_state.current_value_indent = indent;`
   `printed_indent` is `code.indents.indent_of(instructions, target)` — a
   precomputed table lookup — plus `self.activation_indent + self.indent_offset`.
   **Per-clause**; the static half is already compile-time, the two offsets are
   activation state. Could be moved into the `Op::Clause` payload as a
   compile-time constant only for the static half, which is already a table hit.
4. `let line = self.clause_line(…); let entry = self.enter_clause(line);`
   Sets `SIGL` **and** opens the clause boundary, as one operation, because
   `clause.rs` exists precisely to make them inseparable. Per-clause and
   forced: the oracle's boundary is "after every instruction of the
   activation's own flat list" (`RexxActivation.cpp:642-654`).
5. the `Echo::Gated` echo — already elided under `Echo::Compiled`.
6. `let temps_at_entry = self.roots.temps_len(); let frame = self.roots.push_frame();`
   The GC temps frame plus the debug watermark. **Finer than per-clause is
   allowed and already happens** — `condition_value` pushes a per-pass frame
   inside a `DO`'s single clause frame precisely so a `WHILE` re-test does not
   accumulate one `ObjRef` per pass. **Coarser is a leak**, so per-clause is
   the coarsest correct granularity, not a semantic requirement.

### What `leave_stepped_clause` does

`run.rs:4856`: the watermark `debug_assert`, `pop_frame`, `record_failure_site`
if the clause failed, `leave_clause`, and `record_failure_site` again if the
*boundary* failed. `leave_clause` (`clause.rs`) is cheap on the common path: it
destructures the token, returns early if `ran` is `Err`, returns early if
`self.pending_trap.is_none()`, and only otherwise roots the value and delivers.

So the per-clause cost that a flat loop would have to re-express is: two
activation field writes, one indent table lookup plus two adds, one line
computation, one temps `push_frame`/`pop_frame` pair, and two branch-not-taken
checks. None of it is elidable by changing the loop's shape.

### Where the claim is wrong

The claim's list includes **`Interp::settle` and the activation-depth
`debug_assert`** as per-clause work. They are not. Reading `run_ops`'s arms:

* `Op::Clause` → `RegionEnd::At(next)` does `pc = next; continue;` — it reaches
  **neither** the `debug_assert_eq!(self.activations.len(), depth, …)` nor
  `settle`.
* `Op::Jump`, `Op::SelectCaseText`, `Op::EnterWhen`, `Op::EnterOtherwise` also
  `continue` past both.
* `settle` and the depth assert are reached by: `Op::Generic`, `Op::EndBranch`,
  the branch-end `frames.pop_if` arm, and the two `Op::Clause` outcomes that
  produce a `Flow` (`Op::Call`, `Op::LoopRun`, or a handler that ended the
  program).

The common promoted clause — an assignment, a `SAY`, an `IF` condition — reaches
neither. So `settle` is per-*flow-producing* clause, and folding it into a claim
about per-clause overhead overstates the fixed cost the flat loop would have to
carry.

### The claim's conclusion, checked

*"A flat loop re-expresses the boundary as ops rather than removing it."*
**Confirmed, and the code already contains the measurement that says so.** The
`Op::Clause` arm carries a note recording that hoisting *one* piece of per-clause
state (`stale`) into a `run_ops`-entry local was measured against
`bench-programs/emptyloop.rex` with `perf stat -e instructions:u`: 40.3009
billion before, **40.7259 billion (17 per pass)** as a hoisted local, **40.4009
billion (4 per pass)** read per `Op::Clause`. That is a direct, in-repo
measurement of the general shape "move per-clause state to loop lifetime", and
it came out worse.

Concretely, a flat loop needs either an `Op::EndClause` (one more op per
promoted clause, executed) or a per-op `pc ∈ [region_start, region_end)` test
(two comparisons per op), *plus* every jump op has to learn whether a clause is
open — which is exactly what `RegionEnd::At` encodes today for free.

---

## 2. The single exit through `leave_stepped_clause`

### The claim is refuted as stated

**Probe `p2.rs`** reproduces the exact shape — `#[must_use] SteppedClause`
wrapping a `#[must_use] ClauseEntry(())` with a private field, a `'region:`
labelled block, `leave_stepped_clause` consuming the token — and smuggles one
`?` into the region:

```rust
let entry = self.enter_stepped_clause();
let ran: Result<RegionEnd, Failure> = 'region: {
    let v = self.fallible()?;          // skips the leave entirely
    break 'region Ok(RegionEnd::At(v));
};
match self.leave_stepped_clause(entry, ran)? { … }
```

It compiles under `-W unused` with **zero warnings** and runs, leaving `n = 1`
where a completed clause would leave `n = 101`. The `?` returns from the
*function*, not from the labelled block; `SteppedClause` has no `Drop`; and
`#[must_use]` does not fire on a value that is bound and used on the other path.

What *is* compiler-enforced is only the other direction: `leave_stepped_clause`
takes a `SteppedClause`, and nothing outside `clause.rs`/`run.rs` can build the
`ClauseEntry` inside it, so **a leave with no enter does not compile**. That is
exactly what `clause.rs`'s module doc claims and no more. The module doc is
honest about this ("There the two halves are two statements and *can* come
apart"); the claim under review upgraded it.

### The convention is nonetheless perfectly held today

Measured, and stated as a method rather than a grep of code: I extracted the
region block from `HEAD`'s `drive.rs` by locating its opening line
(`let ran: Result<RegionEnd, Failure> = 'region: {`) and its closing
`Ok(RegionEnd::At(end))`, lines 510–1213, and searched that range for the
literal character `?`. **Zero occurrences** — not zero `?` operators, zero `?`
characters, including in comments. So the discipline is total today, and it is
discipline.

### What would restore a real guarantee — and it works flat

* **A `Drop` guard holding `&mut Interp` alongside a body that also uses
  `self`**: does not compile (`p3.rs`, E0499, "cannot borrow `*it` as mutable
  more than once at a time"). That much of the claim is right.
* **A `Drop` guard the body reaches the interp *through*** (`g.it.work()?`):
  **does compile and does run the teardown on the `?` path** (`p4.rs`, prints
  `n=101`). So "cannot be a `Drop` guard" is false as a borrow-checker
  statement. The real blocker is different and stronger: `leave_stepped_clause`
  **returns `Result<ClauseOutcome<T>, Failure>`** — the boundary itself can fail
  (a delivered `CALL ON` handler that raised) and can report
  `ClauseOutcome::Ended`. `Drop::drop` returns `()`. The four extra arguments
  (`code`, `index`, `clause`, `source`) are all shared references or `Copy` and
  would sit in guard fields without trouble; the *outcome* is what cannot come
  back out.
* **A closure / helper returning a `Result` the caller funnels**: that is
  `in_stepped_clause_with`, and it is what the tree-walker uses. In a flat loop
  the clause spans loop iterations, so there is no closure to wrap it in without
  reintroducing the inner loop — i.e. the alternative *is* the current design.
* **A sealed error type, which does work, and works flat.** Make `run_ops`
  return `Result<_, Sealed>` where `Sealed` has no `From<Failure>` and is only
  constructible by the funnel that runs the leave. Then `?` on any
  `Result<_, Failure>` inside the region is a **compile error**, while
  `break 'region Err(f)` still compiles. `p5.rs` demonstrates exactly this:

  ```
  error[E0277]: `?` couldn't convert the error to `seal::Sealed`
     |         let v = it.fallible()?;
     |                    ----------^ the trait `From<seal::Failure>` is not implemented
  ```

  The cost is that the `?` sites in `run_ops` that are legitimately *outside* a
  clause (`self.settle(…)?`, `self.leave_branch(…)?`, `self.end_promoted_branch(…)?`,
  `self.when_frame(…)?`) must each be routed through an explicit conversion. In
  a flat loop the natural funnel is a helper
  `fn seal(&mut self, open: &mut Option<SteppedClause>, f: Failure) -> Sealed`
  that runs the leave when a clause is open — a single call site the type forces
  every failure through.

**Net.** The claim has the polarity backwards. Today's guarantee is a
convention with a perfect record and no compiler behind it; a flat loop is not
*obliged* to weaken it and could be built with the first compiler-checked
version this driver has ever had. That is an argument *for* the flat loop being
feasible, not against it — though it is also work nobody has asked for, and the
convention's record is 703 lines long and unblemished.

---

## 3. `header: Option<LoopHeaderValues>`

### What it is and who touches it

Declared inside the `'region` block:

```rust
let mut header: Option<LoopHeaderValues> = None;
```

* **Written** by `Op::LoopHeaderValue { role, src }`:
  `header.get_or_insert_with(LoopHeaderValues::default)` then
  `self.accept_header_value(*role, value, values)`.
* **Read and consumed** by `Op::LoopRun { index }`: `header.take().unwrap_or_default()`.

Those are the only two ops that touch it. Both are region-only: reaching either
from the outer loop is `Loud::op_not_driven(…)`.

`LoopHeaderValues` (`run.rs`) is:

```rust
initial: Option<Number>, to: Option<Number>, by: Option<Number>,
for_remaining: Option<u64>, over: Option<ObjRef>, count: Option<u64>,
```

### Why the claim is understated

`drive.rs`'s own comment says these values "are not `ObjRef`s and so have no
register to live in: a bound is a `Number` and a budget is a count." **`over` is
an `ObjRef`.** The comment is wrong on that field.

It is safe today because `compile`'s `Do`/`Loop` arm allocates the header's
registers in the *enclosing* scope and releases them past the `END` — its own
comment says so: "these registers are what roots the header's values while the
loop runs: a `DO OVER`'s target lives in `LoopState` for the construct's
lifetime, and a register released at a body clause's own boundary would be
handed out again and overwritten while it is still in use."

So the Rust local's `ObjRef` is rooted **by the register that produced it**, and
that root has a compile-time lifetime. A `header` that outlived its region — the
exact hazard a flat loop introduces — could therefore hold an `ObjRef` whose
register has since been reallocated and overwritten. That is a use-after-free
under collect-on-every-allocation, not a wrong number. `tests/collect_stress.rs`
is the instrument that would catch it, and it is the same instrument that caught
the `ClauseValue`-returns-`()` mutation `clause.rs` records.

### Does a flat design have a natural home?

Yes, and it is one line: **reset `header = None` in the flat loop's `Op::Clause`
arm.** That runs at exactly the frequency the `let mut header = None` runs at
today (once per promoted clause), so it costs the same and closes staleness by
the same rule. The residual loss is type-level, not behavioural: today no op
outside the region can name `header` at all; flat, every arm can.

The bigger structural loss is adjacent and larger than `header`: **flattening
deletes the `op_not_driven` discriminator for the whole region-op family.**
Today, reaching `Op::EvalExpr`/`Op::Load`/`Op::Arith`/… from the outer loop is
*provably* a jump into the middle of a region, and the driver says so loudly. In
a flat loop there is no "outside a region", so that tripwire degrades from a
structural impossibility to an `Option<SteppedClause>` flag check — and for ops
that do not read the clause it disappears entirely.

---

## 4. The hoisted `clause` reference

### What the invariant actually is

Two halves of one statement:

* **`compile::assert_region_ops_name_their_clause`** (`compile.rs:1506`), an
  unconditional `assert!` run once per compile: for every `Op::Clause { index,
  end }`, every op in `ops[at+1 .. end]` that carries an instruction index —
  `TraceClause`, `EvalExpr`, `Store`, `Say`, `WhenTest`, `Call`, `CallExpr`,
  `TraceFunction`, `LoopRun` — carries **that same `index`**. Ops with no index
  (`Const`, `Load`, `Arith`, `Binary`, `Prefix`, the trace echoes, `Jump`,
  `JumpUnless`, `TraceKeyword`, `LoopHeaderValue`, …) are exhaustively listed as
  `None` rather than defaulted, so a new op variant is a compile error there.
* **`drive::debug_assert_names_the_clause`** (`drive.rs`), the run-time half in
  debug: it compares `std::ptr::from_ref` of `code.body.instructions[index]`
  against `std::ptr::from_ref(clause)` — **pointer identity**, not index
  equality — for a stream that reached the driver some other way.

So the invariant is "inside a region, every index-bearing op names the region's
own clause", checked on the emitted stream rather than asserted about the
emitting code. That is exactly what licenses the single
`code.body.instructions.get(index)` in the `Op::Clause` arm.

Note in passing: `assert_clause_regions_hold_no_generic_op`'s doc says "an
`IF`'s region spans its branches, and each branch's clauses are regions of their
own", offered as the reason nested `Op::Clause` inside a region is legitimate.
**No `compile` arm produces that today.** The `If` arm calls `close_region`
immediately after `Op::JumpUnless`; the branch's instructions are emitted at
their own later `op_of` positions, outside the region. Likewise `Do`/`Loop`
closes right after `Op::LoopRun`, and the body is reached through
`run_bounded_from_chunk`. The check is correct and the justification is stale.

### The costed alternative

`Chunk::ops_in`'s doc carries the measurement for the slice walk itself:

> **What a clause region is walked as, rather than one `Chunk::op_at_index` per
> op** … the range is settled once on the way in and the bounds check per op
> goes with it. Worth 2 instructions per promoted clause on
> `bench-programs/varlookup.rex`, whose regions hold three ops.

That is the bounds-check half. The instruction-lookup half is separate: hoisting
`clause` removes one `slice::get` (bounds check plus index) per index-bearing op,
which in `varlookup`'s three-op regions is roughly one lookup per region.

A flat loop's two options:

* **Re-look-up per op.** A bounds-checked `code.body.instructions.get(index)` on
  the hot path — reinstates what the hoist removed, and the check is not
  elidable because `index` is an untrusted `u32` payload.
* **A `let mut clause: Option<&Instruction> = None;` loop local.** This
  compiles: `clause` borrows `code`, which borrows locals of `run_activation`
  rather than `self`, so it survives `&mut self` calls across the loop — that is
  precisely the borrow discipline `run_activation`'s doc comment records, and
  the existing `Op::Clause` arm already holds a `&Instruction` across
  `self.enter_stepped_clause(…)`. The cost is one `Option` discriminant check
  per index-bearing op instead of a bounds check — roughly a wash — plus the
  loss of the structural guarantee from §3.

**Verdict: confirmed.** The flat loop does have to pay one of these two, and
neither is free.

---

## 5. The cheaper alternatives

### 5a. Forward skip on the existing slice iterator

**It compiles, with three specific constraints.** (`p1.rs`)

1. **`for region_op in ops` must become `while let Some(region_op) = ops.next()`.**
   `for` desugars through `IntoIterator::into_iter(ops)` and the iterator is no
   longer nameable inside the body. `for region_op in ops.by_ref()` does not
   help: the `&mut` borrow is held for the whole loop.
2. **The borrow checker permits advancing the iterator from inside a `match` arm
   on the item it just yielded.** `slice::Iter<'a, Op>` has
   `Item = &'a Op`, and `'a` is the *slice's* lifetime, independent of the
   `&mut self` on `next()`. So `region_op: &Op` and `ops.nth(k)` coexist.
   Verified: `p1.rs` compiles and runs, and the skipped op's side effect is
   absent from the result.
3. **`Iterator::advance_by` is unstable on this toolchain.** `rustc 1.97.1`
   rejects it with `error[E0658]: use of unstable library feature
   'iter_advance_by'` (tracking issue #77404). Use `nth(k - 1)` for a skip of
   `k`, or `for _ in 0..k { ops.next(); }`.

**Clippy is clean.** `clippy::while_let_on_iterator` does *not* fire, because
the lint suppresses itself when the iterator is used inside the loop body.
Checked by running `cargo clippy` on the probe as a library crate: the only
warning emitted was `clippy::result_unit_err`, an artifact of the probe's
`Result<_, ()>`.

**The op carries a relative skip count, not an absolute target.** An absolute op
index is meaningless to an iterator that has no position. The count must be
region-relative and forward-only, and it needs a compile-time assertion of its
own (`assert_skips_stay_in_their_region`, in the family of
`assert_region_ops_name_their_clause`) — because an overrun is *silent*: `nth`
past the end exhausts the iterator, the `for`/`while let` ends, and the region
falls through to `Ok(RegionEnd::At(end))` as if the clause had completed. That is
the same class of defect the outer loop's `Op::Jump` out-of-range check was
added to make loud, and the slice walk has no equivalent.

### 5b. A `pc` over the region slice only

`let mut i = 0; while let Some(op) = ops.get(i) { … }`, with jumps setting `i` to
a region-relative index.

This is the smallest change that gets the *entire* stated benefit:

* forward **and** backward in-region jumps, so nothing about the construct set
  is constrained;
* absolute (region-relative) targets rather than skip counts, so an out-of-range
  target is one comparison away from being loud rather than silent;
* **no** change to the outer loop, the clause unit, `RegionEnd`, `SelectFrame`,
  `header`'s scope, the `op_not_driven` family, or the hoisted `clause`;
* it costs back exactly the **2 instructions per promoted clause on `varlookup`**
  that `ops_in`'s doc records, and nothing else.

If the driver must gain in-clause control flow, this is the change to make. The
flat loop is a strictly larger edit for a strictly smaller superset of one
construct's needs.

---

## 6. Is a backward in-region jump ever needed?

**No.** Every repetition in this interpreter is resolved at a level above the op
stream:

* `Op::LoopRun` calls `Interp::run_loop_with_header` with
  `BodyEngine::Chunk { chunk, registers }`.
* `run_loop_with_header` → `run_repeating`, whose per-pass body dispatch is
  `self.run_bounded(code, body_start, end_index, source, engine)`.
* `run_bounded`'s chunk arm is `run_bounded_from_chunk`, which does one
  `chunk.op_at(start)` and **re-enters `run_ops`** for the body's range.

So the loop's "backward jump" is a Rust `loop { … }` in `run_repeating`, one
level up. Each pass is a fresh `run_ops` call; each body clause is its own
`Op::Clause` region; the `DO` header's own region is entered exactly once, and
`run_repeating` re-echoes the `DO` clause per pass through a separate
`trace_clause` call rather than by re-running the region.

`SIGNAL`, `ITERATE` and `LEAVE` are `Flow` values absorbed by `absorb` in
instruction space and applied by `apply_flow`, which writes the activation `pc`
that `run_chunk_clauses`'s outer loop reads back. `SELECT` re-entry into
`OTHERWISE` is `Flow::Goto(target)` from `leave_branch`. None of these is an op
index.

The op stream's only emitted jumps are: `Op::JumpUnless` (`compile.rs:408` for
`IF`, `:560` for a listed `WHEN`), both inside regions and both region-ending;
and `Op::Jump` (`:526` for a `SELECT` whose first `WHEN` is not the next
instruction, and `:1149` in `emit_before`), both emitted **outside** any region.
So `run_ops`'s in-region `Op::Jump` arm is presently unreachable from any
compiled stream — it exists for the type.

`run_ops` even documents that it does not check a backward jump out of its
range, "and it is not emitted: it would re-run ops inside the range, which is a
wrong answer rather than a silent one."

---

## 7. Forced by ooRexx semantics vs incidental

### Forced — each with an observable that pins it

| thing | why it is forced |
|---|---|
| clause boundary after every instruction | `RexxActivation::run` calls `processClauseBoundary()` after each `nextInst->execute()` (`RexxActivation.cpp:642-654`). Observable: a `CALL ON` condition queued by `zres = one(1)` has its handler run *after* the assignment completes. |
| `SIGL` = the current clause's line | one control transfer from being read. `clause.rs` records three fix rounds where a missing boundary produced a wrong `SIGL` **and** a mis-timed handler from one omission. |
| `DATE`/`TIME` clock cache invalidated per clause | `settings.timeStamp.valid = false` after each execute (`RexxActivation.cpp:647`). Observable in both directions: agreement within a clause, freedom across clauses; and a callee's clauses must invalidate only the callee's cache. |
| `>I>` trace-entry decay per clause | `RexxActivation.cpp:657-659`. Measured divergence: `if 1=1 then trace l` as a routine's first clause. |
| `*-*` clause echo per clause, markers included | the oracle's `RexxInstruction::traceInstruction`, called from every instruction's own `execute`, `Then`/`Else`/`Otherwise`/`When`/`Label` included. |
| the clause echo's indent | trace output is compared byte for byte. |
| `PROCEDURE` permitted only as an activation's first instruction | error 17.1. Both halves of the driver's handling carry their own measured witness: dropping the take makes `loop-header-boundaries`' "procedure as a loop body's first instruction" diverge; dropping the grant makes `assignment-and-say`'s "procedure after an assignment in a called label" diverge. |
| a `TRACE` run mid-body affects later clauses | the `stale` fallback, in both directions (a chunk compiled to echo under a setting that no longer does, and vice versa). |
| the temps frame no coarser than per clause | not directly observable, but coarser is unbounded growth; `condition_value`'s per-pass frame shows finer is permitted. |

### Incidental — the implementers' choices

* **The program counter being an op index rather than an instruction index.**
  Deliberate and documented at the top of `drive.rs`; it is what makes a
  promoted construct possible at all, but it is this crate's design, not Rexx's.
* **The two-level loop, and the `'region:` labelled block as its inner half.**
  Chosen so the region's ops sit in `run_ops`'s own frame with no callee between
  the clause's two ends — the alternative (`in_stepped_clause_with`) exists and
  is what the tree-walker uses.
* **`RegionEnd` as a type implementing `ClauseValue`** rather than reusing
  `Flow`. Chosen so a counter position roots nothing while a `Flow` roots its
  value.
* **`SelectFrame` kept in a `run_ops`-local `Vec` rather than on `Interp`**, so
  a raise simply drops it — explicitly argued in that type's doc.
* **`header` region-scoped**; **`clause` hoisted per region**; **`ops_in`'s slice
  walk** — all three are optimisations with recorded measurements, not
  requirements.
* **`Echo::Compiled` vs `Echo::Gated`** — *whether* to echo is semantics; moving
  the decision into the op stream is an optimisation.
* **`GRANTING` as a const generic** — the *split* (grant at the activation's own
  level, not in a construct's body) is semantics; making it a const parameter so
  it compiles to no code is an optimisation.
* **The `#[inline(always)]` annotations** on `settle`, `in_clause`,
  `in_stepped_clause*` and `echo_stepped_clause` — each carries its own
  `perf stat -e instructions:u` figure.
* **`op_not_driven` being loud rather than a panic** — this crate's standing
  rule for a state the type system admits and the compiler does not produce.
* **`debug_assert_names_the_clause` / `assert_region_ops_name_their_clause`** —
  tripwires for an invariant the op-index design created; they have no analogue
  in the language.

---

## What I found that was not asked, and that changes the decision

### A. `&` and `|` do not short-circuit, so half the motivation is void

`eval_logical_list`'s own doc comment, in the repository, measured:

> **Short-circuits on the first element that checks out false**, unlike `&`
> (`logical_values`' own doc comment). Measured with `if 0, (1/0) then nop`
> followed by `say 'reached'`, which prints `reached` and exits 0 … The probe
> is `(1/0)` rather than the `'x'` an earlier version of this comment cited,
> because `'x'` cannot tell the two candidate rules apart.

And `apply_binary` routes `Operator::And | Operator::Or | Operator::Xor` to
`logical_values(op, left, right)` — a plain two-operand function, reached from
`eval_node`'s binary arm after **both** operands have been evaluated and rooted.

`&` and `|` therefore already compile to native ops today: `native_shape`
accepts `ExprKind::Binary` whenever `is_native_binary(op)`, and `push_native`
emits `Op::Binary` plus its `>O>` line. There is nothing to promote.

**The only short-circuit construct in the language is the comma list**
(`ExprKind::Logical`), which `native_shape` declines by falling to its `_ =>
false` arm, and which the live Task 1 brief for `Op::Condition` explicitly
relies on declining ("a native condition is never a comma list — `native_shape`
has no `ExprKind::Logical` arm — which is what licenses passing `checked:
false`").

### B. The comma list can be compiled inside today's shape, with no driver change

The short-circuit exit of a comma-list condition **is** the branch. For
`IF a, b THEN`, `WHEN a, b`, and every other keyword that builds one, a false
element means the whole list is false, and the list's only consumer is the
branch. So each element's false exit can be folded into an op that **ends the
region**, which is precisely what `Op::JumpUnless` already does:

```
  <ops for element 1>
  Op::ListElement { index, reg, false_target }   // >>> element; validate 34.6;
                                                 // if false: >>> "0", end region at false_target
  <ops for element 2>
  Op::ListElement { … }
  …
  <ops for element n>
  Op::ListTail { index, reg }                    // >>> element n; validate; >>> "1"
  Op::JumpUnless { reg, target: false_target }
```

The false target is one value for the whole list — `if_targets(…).false_target`
for `IF`, `WhenInfo::false_target` for a listed `WHEN` — and `PatchKind::Enter`
already resolves it to `first_op_of[false_target]`, which correctly skips the
`Before::ThenEnd` boundary a path that never entered the branch does not owe.
The elements can share one register, since element *k*'s value is dead once
validated.

The trace shape this must reproduce is fixed and small, and the repository has
it measured: `trace r` over `if 1, 1 then say 'x'` gives **three** `>>>` lines —
one per element and one for the list's overall result — and the failing element
traces *before* it raises (`if 1, 'x' then nop` shows `>>> "1"`, `>>> "x"`, then
34.6). Both are satisfiable by the layout above.

If that is right, the two-level loop does not block the construct at all, and
neither §5a nor §5b nor the flat loop is needed to reach it.

### C. There is no workload behind it

Measured, method stated, because a count over code is this project's most
frequently wrong number.

* **Corpus.** I scanned all `*.rex` under `rust/corpus/` with a script that
  finds clauses whose first keyword is `IF`/`WHEN`/`WHILE`/`UNTIL`/`GUARD` and
  that contain a comma at **paren depth zero**, skipping quoted strings — a
  depth-tracking scan rather than a regex over commas, because commas in call
  arguments would otherwise dominate. 70 files scanned, 3 raw hits, of which
  **two are prose inside multi-line comments** (`deep_nesting_indent_cap.rex`,
  `mutation_form_at_render.rex` — my same-line `/* */` stripping does not span
  lines). **One genuine occurrence, in one file**:
  `lang/prefix_dotvar_logical_over_label.rex`, `if 1 = 1, 2 = 2 then say 'both'`.
* **Bench programs.** All ten `rust/bench-programs/*.rex` — the programs behind
  every figure in `docs/superpowers/plans/phase-4f-record.md` — contain **zero**
  such clauses. Counted with `/bin/grep -acE` per file (the standing rule about
  `grep` silently skipping binary files applies; `-a` was used), and every file
  reported `0`, so there is no under-count hiding in a single aggregate number.

So promoting the comma list **cannot move any axis in the phase-4f ledger**:
`alloc4c`, `arith`, `compound`, `emptyloop`, `strings`, `varlookup`, `rexxcps`.
Whatever it is worth, it is worth it on programs nobody is measuring, and the
ledger's own discipline — "predicted movement … stated before the measurement",
"a change that reached its axis's target by a route other than its stated
hypothesis has not confirmed the hypothesis" — has no axis to state a prediction
against.

---

## Bottom line

The conclusion the owner found surprising is right, but three of the five stated
reasons are not the load-bearing ones.

* Claim 1 is right in substance and the repository already contains a
  measurement of the general mechanism (`stale` as a hoisted local: +0.425
  billion instructions on `emptyloop`, 17 per pass against 4). Two items in its
  list — `settle` and the depth assert — are not per-clause and should come out.
* Claim 2 is backwards: today's guarantee is a convention, and a flat loop could
  be given the compiler-checked version this driver has never had. Drop it, or
  restate it as "the convention's blast radius grows from one labelled block to
  the whole loop body".
* Claim 3 is right and stronger than stated: `LoopHeaderValues::over` is an
  `ObjRef` rooted by a register with a compile-time lifetime, so a stale
  `header` is a collector hazard. But its natural flat home is one line
  (`header = None` in the `Op::Clause` arm). The larger loss in the
  neighbourhood is the `op_not_driven` family, which flattening deletes.
* Claim 4 is right, and `ops_in`'s "2 instructions per promoted clause on
  `varlookup`" is the number to cite.
* Claim 5 is right, and 5b is the one to pick.

The decisive arguments are the ones nobody made: **`&`/`|` are not
short-circuit**, so the only affected construct is the comma list; **the comma
list is compilable with region-ending jumps alone**, so today's shape does not
block it; and **the construct appears once in the whole corpus and never in a
bench program**, so no measurement this project takes could see the difference.
