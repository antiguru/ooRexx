# Performance to-do, ordered, 2026-09-20

Everything surfaced by the llvm-lines pass, the DWARF correlation, the `GRANTING`
A/B, the control-flow spike and the `rexxcps` op-mix measurement. Sizes are on
`rexxcps` unless another axis is named.

Every figure carries its evidence class:

* **MEASURED** -- an A/B was run and the number is the difference.
* **DERIVED** -- computed from a measured constant, not itself run.
* **ESTIMATED** -- reasoned from a profile share, no A/B.
* **UNKNOWN** -- named, never sized.

The constant most of the op work is derived from: **one dispatched op that does
no work costs 20.0 instructions** (821,610,422 Ir over 41,001,861 ops, from the
TRACE A/B). Multiply ops removed by 20 to size any fusion.

**Corrected twice on 2026-09-21. The constant is 8, not 20, and it is a floor.**

**The 20.0 is wrong.** It came from a whole-program delta that deleted the three
`TRACE` **instructions** from the source, which removed those clauses' own
execution as well as the trace ops, so the difference was divided by an op count
that did not account for all of it. Measured per address at the dispatch jumps,
the loop overhead of one region op is **8 instructions**: 5 for the dispatch and
3 for the advance.

**And 8 is a floor, not a ceiling.** A fusion that also removes the value handoff
between its two ops is worth more; the one measured came to 54.8 per removed op.
An elision that removes a dispatch doing nothing is worth 8.

Every DERIVED figure below that multiplies an op count by 20 is overstated by
about 2.5x. Item 5's revived floor goes from 0.33% to roughly 0.14%.

The baseline it is all against: `f9ffe9a8b`, 20,000,000 clauses, 127,886,118 ops
dispatched, 21,147,250,696 instructions, **6.39 ops per clause, 165.4 Ir per op,
1,057.4 Ir per clause**.

---

## 1. Do not emit trace ops after a statically known TRACE setting

**MEASURED, 3.89%** as an upper bound: stripping all three `TRACE` instructions
from `rexxcps` takes it from 21,147,250,696 to 20,325,640,274 instructions and
from 6.39 to 4.34 ops per clause.

Moritz, 2026-09-20: *"it would still be a good habit to avoid work, i.e., don't
emit trace instructions after trace off."* **This is the reason it is first, not
the percentage.** A program that turns tracing off should not pay for tracing.

The defect, minimally: `zw = 1` / `zv = zw + 1` / `say zv` compiles to 11 ops with
no trace ops. The same three clauses with `trace off` prepended compile to 18 ops
carrying `TraceLiteral`, `TraceRead` and `TraceOperator`. Emission is per-chunk,
so one `TRACE` anywhere taints all of it, and `TRACE OFF`'s setting is a
compile-time literal.

Scope this item to the **static** case only: a `TRACE` instruction whose setting
is a literal fixes the emission decision for the clauses after it. The dynamic
case is item 2.

**Risk: this is the highest-risk item on the list**, because trace output is
differentially observable on all three descriptors and the corpus has programs
that trace. The gate is the corpus differential and the trace programs, not
reasoning.

**What kills it:** a clause after `trace off` whose emission decision cannot be
fixed at compile time because control can reach it from before the `TRACE`.
Establish that by enumerating the reachable predecessors, not by inspection.

## 2. Recompile on an observed dynamic TRACE setting

**MEASURED as part of the same 3.89%.** `trace value x` cannot be resolved at
compile time, but once the value is seen the chunk can be recompiled for it.

The machinery exists: `drive.rs:568` already carries the staleness check, "whether
the setting in force is still the one this chunk's trace ops were emitted for,
and the whole of what makes a compiled-in emission decision safe". This item is
making that check drive a recompilation rather than only a validation.

**Depends on item 1** for the emission decision it selects. Do not start before
item 1 has landed and its gates are green.

## 3. Fuse `Condition` with `JumpUnless`

**DERIVED, ~0.56%.** 5,900,006 ops removed at 20 Ir each.

Their executed counts are **exactly equal** at 5,900,006, so they always co-occur;
no analysis is needed to know the pair is always fusible in the emitted stream.

Lowest-risk item on the list: both ops are the interpreter's own, the fusion is
local to compile and drive, and nothing about it touches semantics visible to a
program.

## 4. Fuse the comparison into the branch

**DERIVED, up to ~1.1%.** Up to 11,800,012 ops at 20 Ir each, where the `IF` or
`WHEN` expression's root is a comparison operator, turning `Binary` + `Condition`
+ `JumpUnless` into one op. `Binary` executes 7,860,011 times, so at most 5.9M of
those are comparison-rooted and the true figure is lower.

Subsumes item 3 where it applies; do item 3 first anyway, since it is
unconditional and item 4 is not.

**Run before anything else, and it may kill the item outright:** enumerate every
comparison operator's path to `logical()`. `logical()` was verified to produce
only `b'0'` and `b'1'`; the enumeration over operators was never done. If any
comparison can answer something else, `Op::Condition` is not a no-op and the
fusion is unsound.

## 5. Fuse a constant load with the store that consumes it

**DERIVED, ~1.4%.** `LoadConstant` 9,880,016 plus `Const` 5,060,417 is 14,940,433
constant loads; `Clause LoadConstant Store` and `Clause Const Store` are among the
commonest static clause shapes, 13 of 129 in `rexxcps`.

The upper bound assumes every constant load feeds a store, which it does not.
Count the adjacent pairs in the emitted stream before committing to a figure.

## 6. Stop paying a region entry for an empty region

**UNKNOWN size.** 25 of 129 static clauses in `rexxcps` compile to a bare `Clause`
op whose region is empty, and `Clause` is 84.0% of the outer dispatch and 12.8% of
all ops. Those clauses pay an outer dispatch and a region entry for nothing.

Size it first: the static count is 19% of clauses but says nothing about how often
they execute. Get the executed count before designing anything.

## 7. Assert the bound before indexing

**MEASURED per axis, and the largest item outside the driver on the narrow axes.**
`core/src/slice/index.rs` inside the driver: **4.68% of `varlookup`**, **3.29% of
`emptyloop`**, 1.36% of `decrender`, 1.03% of `dispatchclass`, in retired
instructions.

The one candidate the control-flow spike did **not** reject, and it is not a
control-flow change. Project memory already records asserting the bound before
indexing as worth several percent on `Index`'s panic path.

Not yet sized on `rexxcps`, which is the gap to close before scheduling it.

## 8. The work inside the ops, which is where the rest is

**ESTIMATED from profile shares, none of it A/B'd.** After item 1, dispatch is
4.34 ops per clause at 20 Ir, about 87 of 1,016 Ir per clause. **The other ~929
is work inside the ops**, and no item above touches it.

By self cost per dispatched op on `rexxcps`:

* `exec_parse` **11.19 Ir/op, 6.77%** -- the largest single non-driver item, and
  `bench-programs/README.md` independently records it at 6.8%.
* numeric: `numeric_order` 3.80 + `Number::add_signed` 3.66 + `Number::mul` 2.47
  = **9.93 Ir/op**.
* text/number conversion: `to_text` 2.69 + `inline_text_to_number` 2.28 +
  `heap_to_number` 2.12 = **7.09 Ir/op**. `rexxcps` performs 5,580,002 of these.
* allocation and collection: `collect_now` 2.81 + `alloc_with` 2.46 + `free` 1.52
  = **6.79 Ir/op**.
* `apply_binary` 6.84, `assign_expr_target` 4.18, `__memcpy_avx_unaligned_erms`
  4.51, `append_tails` 3.26, `concat_values` 2.99.

Each of these is a separate piece of work and none has been opened. **This is the
half of the problem the whole session has not touched.**

## 9. Per-send method cache

**UNKNOWN, narrowed.** Executes the revisit condition D28 recorded and D29 built
the guard for. Full write-up, hazards and sizing run in
`2026-09-17-crexx-derived-candidates.md`.

Narrowed but not settled: `MethodDict::slot` is 2.85% and `Interp::lookup` 1.44%
of marginal hot-line Ir on `dispatchclass`, which is a third currency and not the
kill threshold's. The clean per-iteration count still has to be taken, and
whichever hash symbol belongs to the method dictionary still has to be
established rather than guessed from a type name.

## 10. Move ops off the AST re-entry path

**UNKNOWN.** `Say`, `Parse`, `Message`, `Call` and `EvalExpr` carry an instruction
index and re-enter `eval.rs`. `Op::Exec` is `INTERPRET` and must stay a re-entry.
Sizing run named in the CREXX item. May be subsumed by item 9.

## 11. Placement: is a deliberate pass available at all?

**UNKNOWN, and the largest unclaimed prize measured.** Dead padding alone swings
`dispatchclass` by 123 points of L1i and 11 of cycles, non-monotonic in pad size,
with `Ir` flat. Every commit currently draws from that at random.

Nobody has tried a linker order file, `-C llvm-args` ordering, PGO or BOLT. The
padding table proves the prize exists; nothing yet says it can be taken
deliberately.

**This is a question to answer before it is a task to schedule**, and answering it
is cheap.

## 12. Broad hot-set footprint reduction

**UNKNOWN, and large.** `rexxcps` is capacity-bound: 32K to 64K halves its
simulated I1 misses, 32K to 128K removes nine tenths, while 8-way to 64-way at
32K buys 5.5%. Its hot set does not fit in 64 KiB.

No single function delivers this -- `run_ops_from` is 13.8% of the hot set on
`dispatchclass` and the driver's entire perfect-packing saving is 1.79 KiB. It is
a programme, not a task, and it should not start before item 11 says whether
placement is steerable, because the two interact.

---

## Housekeeping, not performance

* **Restore the subject of the measurement comment above `fn run_ops`.** It reads
  "a change with that profile is not worth 7 instructions" and names no change.
  From `git log -S`, it belongs to `132c33955` and is about bundling the
  range-invariant arguments into a struct. One line. It nearly caused the
  `GRANTING` A/B to be skipped as already done, and the control-flow spike hit it
  too.
* **`GRANTING` stays as it is.** The const generic earns its instructions: merging
  the instantiations costs +4.00% Ir on `varlookup` and is worse on `rexxcps` on
  all three counters. Its reported -3.83% cycles win on `dispatchclass` was
  withdrawn as unattributable, and that withdrawal is an inference from the
  padding envelope rather than a re-measurement.

---

## Two readings, 2026-09-20, and what they change

### Tratt, "Retrofitting JIT Compilers into C Interpreters" (2026)

Meta-tracing an existing interpreter rather than rewriting it: `yk` traces a C
interpreter through a modified LLVM (`ykllvm`), records basic-block traces,
optimises and emits machine code. Roughly **400 lines added and under 50 changed**
to `yklua`. Measured about **2x geometric mean on Lua**, up to 6x on
interpreter-loop-heavy code, and near nothing on allocation-heavy code. The author
calls the figures "a floor, not a ceiling", and warns that "small benchmarks
flatter the sorts of techniques" used.

**What is directly useful to us without adopting any of it.**

*The annotation that mattered most was about decoding.* `yk_idempotent` on the
opcode-loading function lets the optimiser constant-fold instruction decoding and
remove nearly all guard overhead; **removing that one annotation cost about 4x**.
That is a statement about where an interpreter's cost hides: not in the switch,
but in everything the switch cannot prove constant. It is the same shape as our
own finding that dispatch is 20 Ir of 234 and the other ~214 is work the dispatch
cannot specialise.

*The caveat lands on us squarely.* `rexxcps` is a 1000-clause loop, which is
exactly the kind of benchmark the author says flatters these techniques. Any
figure we take from it should be read the same way.

**What does not transfer.** `yk` is built for **C** interpreters through a
modified LLVM toolchain; whether it reaches a Rust interpreter at all is the first
question and it is unanswered here. Beyond that, a JIT emits and executes memory,
which this workspace confines to two files in one crate by decision, and adds a
toolchain dependency where the budget has been one crate. And correctness here is
defined byte for byte against the oracle on three descriptors, including
traceback, `SIGL` and trace output, which a trace compiler would have to preserve
through deoptimisation.

**So: not a candidate, but it re-ranks the list.** It is evidence from a
neighbouring project that the interpreter's per-operation *work* is where the
multiple is, not its dispatch. That is item 8, and this moves it up rather than
adding anything new.

### Allen and Cocke, "A Catalogue of Optimizing Transformations" (1971)

**The copy at that URL is truncated.** `pdfinfo` reports 6 pages, the file is 6
pages, and page 6 ends mid-sentence in the middle of PROCEDURE INTEGRATION. What
is present is the introduction and the procedure-integration section. The
catalogue proper -- the transformation list the paper is famous for -- is not in
this copy. Anything wanted from the rest has to come from another source.

**From the pages that are there, one section maps onto our measurement exactly.**
Their four subprogram linkages are CLOSED, OPEN, SEMI-OPEN and SEMI-CLOSED, and
they list what a CLOSED linkage costs:

* registers must be saved and restored on entry and exit;
* specific registers must be allocated to specific functions, constraining
  register assignment;
* parameters and globals set or used in the callee are unknown to the caller, so
  memory must hold current values at the call and first uses after it must access
  storage;
* in most cases all information must be passed in storage;
* **"the called program cannot take advantage of particular argument values,
  recurrences or relationships. For example, the called routine cannot take
  advantage of a constant valued argument."**

**That last line is our ~124 instructions per op outside the driver.** Every op
that calls `exec_parse`, `apply_binary`, `to_text` or `Number::add_signed` is a
CLOSED linkage: the callee cannot exploit what the call site already knows, even
where the op carries exactly that knowledge. `Op::Arith` holds a per-site type
hint and still calls a general routine.

Their SEMI-OPEN and SEMI-CLOSED forms are the middle ground we have not
considered: compiling the callee together with its caller so parameter locations
can be recognised as unnecessary, or compiling the callee first and letting the
caller exploit which registers and globals it actually touches.

**A caution the authors put in their own introduction**, and it is the same one
this session has been enforcing: *"it is not generally clear that a particular,
so called, optimizing transformation even results in an improvement to the
program. A more correct term would be 'amelioration'."* They also note that no
general means exists of establishing optimal bounds, and that the correctness of
transformations is under-studied. Fifty-five years on, the padding control that
retired a -3.83% result this week says the same thing.

### What this changes on the list above

Nothing is added and nothing is removed. **Item 8 moves to the top of what
matters**, with a sharper name than "the work inside the ops": it is the cost of
a closed linkage at every op boundary, and both readings point at it from
different directions. Items 1 through 7 remain worth doing, remain small, and
remain the only things on the list that are sized.

### Kildall, "A Unified Approach to Global Program Optimization" (1973)

**This one changes how item 1 should be built.**

The framework: a program graph of nodes and control-flow edges, a set `P` of
"optimizing pools" with a meet operation forming a **finite meet-semilattice**,
and an **optimizing function** `f: N x P -> P` mapping a node and its input pool
to an output pool. The algorithm starts at the entry node with an entry pool,
pushes each node's output to its immediate successors, **meets** the incoming pool
with whatever the successor already has, and reprocesses a successor whenever that
meet reduced its information or it is being seen for the first time. It iterates
to a fixpoint, and **the order in which successors are processed is unimportant.**
Kildall proves it correct and terminating.

His own motivating example is constant propagation: the pool is a set of
`(variable, constant)` pairs, and the answer at a node is the intersection over
every path reaching it, because it is not known at compile time which path
execution takes.

**Item 1 is exactly this problem and should be written as an instance of it.**
"Which TRACE setting is in force at this clause" is a forward dataflow question:

* pool: the trace setting in force, plus a top element for "not yet known" and a
  bottom for "not statically known";
* meet: two settings that agree give that setting, two that disagree give bottom;
* optimizing function: a clause that is not a `TRACE` passes its pool through, a
  `TRACE <literal>` produces that literal, a `TRACE VALUE <expr>` produces bottom;
* emission: a clause whose pool is a single known setting is emitted for it, and a
  clause at bottom is emitted traced.

The lattice is tiny, so the fixpoint is reached almost immediately.

**This retires the "what kills it" line written for item 1 above.** That line said
to enumerate the reachable predecessors of every clause after a `trace off` by
hand. The meet does that by construction and is the reason the algorithm is
correct rather than plausible. **Write the analysis, do not enumerate the cases.**

**One hazard the framework does not cover on its own.** Kildall's algorithm is over
a single program graph. `INTERPRET` compiles its own chunk at run time, and an
interpreted fragment can contain a `TRACE` instruction that the enclosing graph
never sees. An `Op::Exec` must therefore drive the pool to bottom for everything
reachable after it, unless something stronger is proved. Getting that wrong is a
program that silently stops tracing.

**Item 5 is Kildall's own example.** Constant propagation is what his pool of
`(variable, constant)` pairs computes, so the fused store-constant is a
consequence of the analysis rather than a peephole.

**And it reframes `Op::Arith`'s hint.** The per-site hint is a *dynamic*
approximation, learned by falling through once and then demoting permanently. A
constant-propagation or type-propagation pool would answer statically at the sites
where the operand type is provable, removing the guard rather than learning it.
That is a larger piece of work and it is the same lever as item 8.

**The caution, in his own conclusions:** the optimizing functions given "by no
means exhaust those which are useful". Combined with Allen and Cocke on the
correctness of transformations being under-studied, and with this project's own
oracle differential, the rule stays what it already is: the analysis decides what
is legal, the differential decides whether it was.

---

## Item 1 landed 2026-09-21, and its measured value on `rexxcps` is zero

Commits `7a28f68e6` (the analysis, emission unchanged, with the check that
validates it) and `7fbadfecb` (emission wired to it). Both gated green on
`rust/CLAUDE.md`'s set, each status read unpiped from its own file with the sha
recorded either side.

**The mechanism works and reaches the ceiling exactly.** On a program whose
`TRACE OFF` is a top-level literal: **-40,721,661 ops (-31.91%)** and
**-362,582,256 instructions (-1.75%)** against BASE, landing at 86,884,259 ops
where deleting every `TRACE` outright gives 86,884,257 -- the two extra being that
clause's own `Clause` and `Exec`. Writing `TRACE OFF` used to cost 40.7M ops; it
now costs two.

**On the pinned `rexxcps.rex` it removes nothing at all.** 127,886,118 ops before
and after, the rendered stream `diff`-identical, instructions moved -0.0017%
against a measured noise band of 0.0047%. Its three `TRACE` instructions are
`trace value tracevar`, `trace value trace()` and `trace off`, and **all three sit
inside `do i = 1 to averaging`**. Two are bottom by construction because their
setting is an expression; the third is refused because it is inside a loop.

**The loop case is the analysis being right, not a gap.** A `trace off` inside a
loop poisons the loop head, because the head is reached both from before the
`TRACE` and from after it and those pools disagree. The head genuinely runs under
two settings on different iterations, so no per-clause static emission for it can
be correct. Nobody should later read this as a limitation and try to remove it.

### The correction this forces, and it is mine

Item 1 above is labelled **MEASURED, 3.89%**. That figure came from an A/B that
**deleted** the `TRACE` instructions. Deleting the cause measures the cost of the
thing; it does not measure what a fix can recover, because a fix has to keep the
instructions and their semantics. The realisable share on `rexxcps` is **0.00%**.

**Measuring a ceiling by removing the cause is not measuring the fix.** Where the
two differ, the difference is exactly the semantics the fix must preserve. Every
other ceiling on this list derived the same way -- items 3, 4 and 5 are ops-removed
times 20 instructions -- is subject to the same gap, and those at least remove ops
that genuinely exist on the hot path.

### Two operational findings from the run

* **`memcap 8G` kills a cold release compile of this workspace.** The first gate
  attempt died at 137 with `rustc` peaking over the cap. The split that works is
  builds outside the cap, tests under it. This is not written down anywhere and
  every future task will hit it.
* **The op-count method reproduces**: the four dispatch jump addresses read
  19,441,281 / 1,120,000 / 98,924,837 / 8,400,000 at BASE, row for row what
  `2026-09-20-instructions-per-op.md` recorded from a different session. The noise
  band was measured at 0.0047% rather than assumed.

### Process note

`7fbadfecb` is `056680a01` **amended**, which breaks the standing never-amend
rule. No harm landed: the full gate set was re-run at the amended head and the
disclosure that the measurements predate the amend was made unprompted. A
follow-up commit is the answer next time.

### Correction, 2026-09-21: my account of the loop case was wrong twice

I wrote above that a `trace off` inside a loop "poisons the loop head, because the
head is reached both from before the `TRACE` and from after it and those pools
disagree", and called it the analysis being right rather than a gap. **Both halves
are wrong**, and the implementer corrected them with green assertions rather than
argument.

**The meet is never reached.** `analyse` returns all-`Unknown` for the whole body
before it builds a graph at all, from a precondition that every instruction which
can change the setting sits at the body's own top level. The unit test
`a_trace_below_the_top_level_refuses_the_body` asserts exactly that for the loop
shape.

**And the disagreement would not generally occur.** `TraceMode::NORMAL` and
`TraceMode::OFF` are the **same `ChunkTrace`** -- nothing in `all`, `labels`,
`intermediates`, `results`, `commands` or `debug` separates them, only `failures`,
which no emission reads. So `trace off` under a normal entry meets to itself.
`trace_off_from_normal_leaves_the_whole_body_known` is that as a green assertion.

**What actually forces the precondition is forward skip edges over an event**, not
back edges: `if 0 then trace off`, a zero-trip `do i = 1 to 0` skipping its own
body, a `LEAVE` or `ITERATE` jumping past it. The counterexample is `trace i` /
`if 0 then trace off` / `say 'x'`, which is bottom in truth and `Known(OFF)` under
a fallthrough-only edge set. Refusing the body is how that unsoundness is avoided.

**So the loop case is a limitation of the current edge set, and it is legitimate
future work rather than correctness.** With skip edges modelled, a zero-trip loop
makes the instruction **after** the `END` bottom while the clauses inside the loop
after the `TRACE` stay known -- and the inside is the hot half. The implementer
reasons, without measuring and without claiming it, that this would take the
in-loop variant from 768 ops toward the 537 ceiling. **That is the follow-up worth
having, and it is the difference between this item being worth nothing on
`rexxcps` and being worth most of its ceiling.**

The measured half stands unchanged: 768 ops at HEAD against 768 at BASE on that
variant, so as it stands the loop case yields nothing.

This is the same shape as the recorded hazard that my outcome rulings hold and my
mechanism rulings do not. The outcome -- the loop case yields nothing today -- was
right. The mechanism I gave for it was invented.

### Three things the implementer measured rather than reasoned

1. **`INTERPRET` propagates.** `trace off` then `interpret "trace i"`: the oracle
   traces every clause after it in the enclosing body, rc 0. So `Interpret` drives
   the pool to bottom, which is what the brief demanded be measured rather than
   assumed.
2. **An internal routine does not.** `call zsub` where `zsub` does `trace i`: the
   oracle's entire stderr is `9 *-*   return`, rc 0. That is what
   `never_retraces` already assumed, and **nothing had ever checked it**.
3. **A hazard the brief did not name.** An instruction can change the setting
   part-way through its own clause: under `trace off`, `zg = trace('i') 'tail'`
   echoes `>L> "tail"` and `>O> " " => "O tail"` under the `I` the call just
   installed. Emitting that clause for its entry pool loses both, observed against
   the oracle. The analysis answers bottom at any event.

   **And the negative control is the part to keep:** reverting that fix reddens
   the existing oracle-checked `ir_recorded_cases/trace-settings:214` *and* the new
   witness identically, so the new witness added no coverage and was **dropped
   rather than committed**. That is this project's own "can fail is not adds
   coverage" rule applied by the implementer to its own test.

### What item 1 leaves, from the implementer's own concerns

**Item 2 owns the whole 3.89% on `rexxcps`.** The list above reads as though item 1
banks part of it; on that axis it banks none. `rexxcps`'s two `trace value <expr>`
instructions are bottom to any static analysis, and a runtime recompile once the
setting is observed is orthogonal to where they sit. Item 1's dependency for item
2 is built and real, so item 2 is now unblocked and is where the measured value is.

**The top-level precondition is the next thing to relax, and it is sized.** A
literal `TRACE OFF` one level down gives the entire win back: 768 ops against 768.
Relaxing it means the currently-inert edges become real, and every missing edge is
an unsound "known" answer.

**Which skip shape forces the precondition in practice** -- the implementer's
reasoning, explicitly unmeasured and not claimed:

> The **zero-trip `do`** is the one that forces it, and the only one that forces
> it for the shape that matters. Take a `TRACE` sitting directly in a loop body
> with no `IF`/`SELECT` around it and no `LEAVE`/`ITERATE` before it, which is
> `rexxcps` line 86 and the whole hot-half question. `if 0 then trace off` does
> not arise: there is no branch. `LEAVE`/`ITERATE` does not arise: there is no
> jump. The zero-trip `do` does: `do i = 1 to 0` runs the body zero times, so
> control leaves the header for the instruction past the `END` without executing
> the `TRACE`, and a fallthrough-only chain claims it ran.

**So the follow-up is small if scoped to loops:** one edge per loop, header to the
instruction after its own `END`, which `InstructionKind::Do(loop_)`'s `loop_.end`
already gives and `compile.rs` already builds the same map for; **the back edge
deleted in this work comes back and becomes load-bearing**, inert only under the
current precondition; and the precondition narrows from "every event at indent 0"
to "no event inside an `IF`/`SELECT` branch, and no `LEAVE`/`ITERATE` in a loop
that contains one".

**Interactive debug is refused whole, deliberately.** A `TRACE I` typed at a pause
is invisible to any analysis of the body, so nothing changes under `RXTRACE=ON`.

**The orphaned comment could not be left after all**, and the implementer said so
rather than quietly doing it. `plan.rs:178`'s compound sentence sat above
`never_retraces()`, which commit 2 deletes; it would have glued itself onto
`trace_events()`, where it is false. It was moved onto `compound()`, which is what
it describes. I had declined that fix to keep the diff about one thing; the
deletion forced it, and shipping a false comment is the one alternative
`CLAUDE.md` forbids outright.

### The memcap trap, with the command that died

This is not written down anywhere and every future task will hit it. At a cold
tree:

    REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast

    Compiling rexx-core v0.1.0 (.../rust/crates/rexx-core)
    memcap: line 67: 3543867 Killed ( echo "$BASHPID" > "$cg/cgroup.procs"; exec "$@" )
    memcap: OOM-killed at the 8G cap (peak 8.0G)

**rc 137, and the tests never ran** -- the last log line before the kill is a
`Compiling`, so this is `rustc` peaking over the cap, not the test run the cap
exists for. **A status-only reading calls that a red gate.**

The split that works, and the one every gate in this task was run under:

    cargo build --workspace --all-targets --release                              # outside the cap
    REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast # under it
    cargo build --workspace --all-targets                                        # outside
    REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast           # under it

`rust/CLAUDE.md` already records that a cap which turns green tests red is a second
experiment rather than a cap. **This is the same defect one stage earlier -- a cap
that turns a _build_ red -- and it reads identically in a status file.**

### The skip-edge follow-up, scoped by the implementer, and why it does not bank `rexxcps`

With the loop edges alone -- and both halves are a walk over the instruction list,
not an analysis -- an event in a loop body leaves the loop **head** bottom, because
the entry side and the back edge may disagree; everything in the body **after** the
event `Known`; and the instruction after the `END` bottom via the zero-trip edge.
**The hot half is the part that goes `Known`**, which is the point.

Two cautions on the small version, both cheap to get wrong:

* an event at loop depth two needs the zero-trip edge of **every** enclosing loop,
  not only the innermost;
* `DO FOREVER` and `DO UNTIL` cannot trip zero times, and the edge should be added
  to them **anyway**. A spurious edge only lowers an answer, and special-casing
  which loops can be empty is the kind of reasoning that has been wrong here
  before.

It becomes **large** the moment `IF`/`SELECT` skips and `LEAVE`/`ITERATE` are
modelled properly rather than refused, because that is the real CFG --
`if_targets`, `when_targets`, `otherwise_range`, leave/iterate resolution -- and
every one of those edges is **unsound by omission rather than loudly wrong**.

**But it still banks nothing on `rexxcps`, and the line numbers say why.** Its
`TRACE` instructions sit at 38 (`trace value tracevar`), 76 (`trace value
trace()`) and 86 (`trace off`), and the timed 1000-clause body is lines 41 to 83.
So the hot body sits **after** a `trace value` and **contains** another, and both
are bottom to any static analysis whatever the edge set. The `trace off` that the
loop work would recover is at 86, past the body entirely.

That confirms the implementer's concern 1 from a second direction: **on `rexxcps`,
only item 2 can bank anything.**

### Item 2's design, which my one-line entry above did not pin down

"Recompile on an observed dynamic setting" is not specific enough to know whether
it banks anything, and saying it does would repeat the error corrected above. The
design that would:

**Promote a runtime observation into the static pool, behind a guard.** When a
`trace value <expr>` is observed to yield a particular setting, compile the chunk
with the pool after that instruction as `Known(<observed>)` rather than bottom, and
keep it valid with a check that the instruction still yields the same setting. The
staleness machinery at `drive.rs:576` already detects a setting that no longer
matches what a chunk was emitted for; this extends what the chunk is keyed on from
the entry setting to the observed outcomes at its events.

That is quickening with a deoptimisation guard, not a recompile on entry, and it
is a larger piece of work than the entry in the list implies. **Its guard
obligation is the whole of its risk**: a guard that fails to fire is a program that
silently stops tracing, which is the same failure mode item 1 was built to avoid
and which no gate catches by accident.

---

## Item 2 was built and reverted, 2026-09-21, and the entry above is wrong twice

Commits `941b5fb82` (the loop edges item 1 left), `3c2e6825a` (the
speculation and its guard) and `4ef6af5bb` (the revert of the second). All
figures below are `valgrind --tool=callgrind`'s `summary:` line, two rounds
each, with the widest spread inside one build at 0.020%.

### **3.89% is not this item's ceiling, and the same error is in the entry above**

Item 2 reads "MEASURED as part of the same 3.89%". It is not. That figure
comes from replacing `rexxcps`' three `TRACE` instructions with `nop`, and
**reproduces**: 20,326,186,020 against 21,148,574,783, -3.889%. But an
unguarded speculation of *every* clause -- the most any promotion of this
shape can reach, measured on one binary with the speculation switched by an
environment variable -- reaches only 20,788,670,478, **-1.706%**. The 2.18%
between them is the three `TRACE` instructions still *executing*, which no
fix removes: line 76's `trace value trace()` runs 280,000 times, once per
pass of `do loop = 1 to 14`.

**This is the correction under "Item 1 landed" applied to the next item and
missed.** Measuring a ceiling by removing the cause is not measuring the fix,
and item 2's entry inherited the same number from the same A/B.

### **The guard is not free, and that is what killed it**

The emission change is worth **-114,839,985 instructions, -0.543%** on
`rexxcps`, measured with the guard deleted. A guard that keeps it honest
costs **+105,880,903, +0.501%**, measured with the emission left
unspeculated -- so the two nearly cancel, and the committed pair netted
-0.032% against a 0.020% spread.

The cost is paid per clause by every program, whether or not it speculates:
`bench-programs/emptyloop.rex` +1.25% and `varlookup.rex` +1.08%, neither of
which contains a `TRACE`. Four shapes were measured and none is cheaper --
a call to `run_bounded_from_chunk`, the same behind an out-of-line `#[cold]`
helper, settling the flow inside the branch, and splicing the unspeculated
ops into the stream so the guard is one assignment to the program counter.
The restructuring around the guard is not the cost: the same arm with the
branch deleted measures `emptyloop` at 9,998,565,949 against 9,998,565,263.

**What the item needs is a guard that is not per clause.** The setting can
only change at an event, but every clause after one rests on it and has
nowhere else to ask. Until that is answered, item 2 is not worth building:
its whole value on `rexxcps` is 0.54% and its guard costs 0.50%.

### Two things the entry above gets wrong about the mechanism

* **Keying the chunk on an observed outcome banks nothing here.** The chunk
  is chosen when an activation starts, and `rexxcps`' 1000-clause body runs
  inside one activation: measured, `run_activation` is entered 8 times at the
  top level against 280,002 for the `subroutine:` calls, and the timed body
  is in one of the 8. An observation recorded on the first pass of
  `trace value` has no later entry to change. What does work is speculating
  that an event *keeps* the setting it was entered with, which needs no
  observation at all -- that is what `3c2e6825a` built.
* **`drive.rs:576`'s staleness check cannot be the deoptimisation.** It moves
  the clause echo between the stream and a run-time gate, in both directions;
  the value echoes are ops, and an op that is not in the stream cannot be
  gated back on. Anything that removes them needs somewhere else to send
  control, which is the whole of why the guard costs what it does.

### What did land, and what it is worth

`941b5fb82` models a loop's back edge and its zero-trip edge and narrows
item 1's precondition from "every event at the body's top level" to "no event
inside an `IF` or a `SELECT`", with a `LEAVE`/`ITERATE` reaching every
enclosing block. On the in-loop literal shape -- `rexxcps` with its two
`TRACE VALUE` instructions removed -- `rexx-ir` renders **766 ops at
`7fbadfecb` and 536 at `941b5fb82`**. On `rexxcps` itself it renders 770
against 739 and measures 21,148,574,783 against 21,148,181,856, which is
inside the spread: every one of the 31 ops is at a source line before the
timed body.

## Items 4 and 5 are dead, 2026-09-21, measured before either was built

Both were sized from counts that were never checked against the stream as
emitted. A read-only investigation checked them, and the implementer was stopped
before building item 5.

### Item 4 is unsound, not merely small

`eval.rs:1287` checks `operator_message_receiver(left)` and returns
`send_operator(...)` **before any per-operator dispatch**, so a comparison whose
left operand is an object answers whatever a user-defined method answers,
unvalidated. `logical()`'s validation sites are never reached on that path. It
covers every comparison operator at once, because the escape sits ahead of the
operator rather than inside it.

Verified against the oracle, rc 222, stdout `ret: banana`, stderr `Error 34.1:
... must be exactly "0" or "1"; found "banana".`:

    zk = .Kustom~new
    zr = (zk = 1)
    say 'ret:' zr
    if zk = 1 then say 'then'
    ::class Kustom
    ::method '='
      return 'banana'

`WHEN` raises 34.2 by the same route; a stem whose default is an object, a class
object through its metaclass, and a method returning an object all reach it.
Dropping `Op::Condition` swaps a user-visible 34.1/34.2 for
`Loud::register_not_logical`, whose own doc says it is "an internal
inconsistency, never a program error". `Op::Condition` also owes the `>>>` line.

**This candidate has now died twice under two names**, as CREXX candidate C and
as item 4 here, both times reasoned from "a Rexx comparison always answers 0 or
1" with nothing run.

### Item 5 is six ops per program run, not 1.4%

Strictly adjacent constant-load-then-`Store` pairs on `rexxcps`: one
`Const` -> `Store` and five `LoadConstant` -> `Store`. Their source clauses are
`rexxcps=2.2`, `count=200`, `averaging=100`, `tracevar='Off'`, `empty=0` and
`full=0`, at lines 6, 11, 12, 14, 25 and 36. The timed body is lines 41 to 83, so
**every one of them is program setup that executes once**.

The pairs inside the timed body are separated by a `TraceLiteral` that still has
to emit, so reaching them means fusing across a live op. Execution-weighted that
route is about 1,740,000 ops, roughly 34.8 million instructions, **about
0.165%** -- under the 0.3% floor the item's own brief set.

**The error was in the sizing method, not the arithmetic.** The ~1.4% counted
constant loads and store-shaped clause templates without checking whether the two
were adjacent, or whether the adjacent ones were in the loop.

**Pin the six to the `rust/` subtree `63686bcc7247d7cd2baac82969fc43242c63f01a`,
measured 2026-09-21.** That subtree is what `941b5fb82`, `4ef6af5bb` and
`bc26d7936` all carry: `git rev-parse <commit>:rust` answers it for each, and
`git diff --stat 941b5fb82 4ef6af5bb` is empty because the revert restores the
tree the speculation was built on. Naming the subtree rather than a commit is
what removes the ambiguity, since three commit names are equally correct here and
the one in between them, `3c2e6825a`, is not. The measuring binary's sha256 was
`3f91e0ac39eda1b203abec64f766ee50fc32295a44bac03dfe9b29a73855fa67`. The
six adjacent pairs are adjacent *because* `7fbadfecb` made emission per clause:
`rexxcps`'s first event is at line 38, so the six setup clauses before it now
compile bare and their constant load sits directly against its `Store`. Rendered
under setting `i`, where the setting is in force from the first clause, those six
acquire a `TraceClause` and a `TraceLiteral` and the count goes to **zero**.

So this verdict is conditional, and the condition is the part to carry forward:
**item 5 is dead while the timed body still emits value echoes.** Anything that
stops the hot body echoing makes its ten hot pairs adjacent and item 5 worth
roughly its original figure. That is the same thing item 2 tried and failed to
do, so the dependency is real but not close.

The execution weighting was tested at its weakest point rather than asserted. The
two `SELECT` branch counts that decide 87 against 114 per body were run with a
counter on every branch, on both interpreters: line 66 fires once, line 73 never,
line 65 thirteen times, line 68 fourteen. And had both been wrong, 0.165% becomes
0.216%, still under the stop threshold. The model also predicts `Parse` at 112 per
body against a measured 2,240,002, with the residual two named as `parse source`
and `parse version` at startup.

**One thing left open, recorded as a hypothesis and not a finding:** `Condition`
and `Arith` are each exactly 28 per body short of the model and nothing else is.
Line 50, `if 17<length(j)-1`, is the only clause in the `j` loop carrying both at
that rate, so one uncounted clause would close both gaps with a single cause.
Nothing has been run that shows it. An earlier explanation, that the counted
dispatch excludes an `Op::LoopRun` body, was withdrawn: the `Parse` total only
reconciles if the `parse var` inside `do 1; ... end` **is** counted.

### Item 3 survives the same check

`Condition` -> `JumpUnless` is adjacent at every site on `rexxcps`, under
settings `n`, `off`, `a`, `i` and `r`, with `n` confirmed as the stream that
runs. Nothing sits between them. On the same stream every `Condition` is fed
through a trace op rather than from its `Binary`, which is why item 4's
adjacency is zero and item 3's is complete.

## Item 3 landed 2026-09-21 at `725aa8863`, three times its estimate

`Op::Condition` and `Op::JumpUnless` become `Op::ConditionJump`. The fusion is
made **where the ops are emitted**, not by a pass over a finished stream, so
nothing renumbers and no side table keyed by an op index moves. `Op::JumpUnless`
survives for the arrivals that are not a validated condition, an `IF` falling to
`Op::EvalExpr` and an `Op::WhenTest`.

| | BASE `bc26d7936` | HEAD `725aa8863` |
|---|---|---|
| instructions | 21,147,005,593 | 20,792,862,096 |
| within-build spread | 0.0068% | 0.0002% |
| ops dispatched | 127,885,295 | 121,425,289 |
| static stream | 739 | 720 |

**-354,143,497 instructions, -1.6747%**, against a derived estimate of 0.56%.

Reproduced independently: a separate build of the same source, `sha256
b079d104213a2dc188c49a446acf342b00e90c587c8c2850ab0b8e2c6c4b3f11`, measures two
rounds at 20,794,262,141 and 20,794,343,764, mean 20,794,302,952, spread
0.00039%, which is **-1.6679%**. The two builds differ by 0.0068%, inside BASE's
own spread. The static stream reproduces exactly: 720 ops, `Condition` 0,
`JumpUnless` 0, `ConditionJump` 19.

Coverage is a census rather than a sample: across 621 programs, 765 `Condition`
ops and 765 immediately followed by a `JumpUnless`, in both stream shapes,
echoes on and off. Rendering HEAD independently answers 765 `ConditionJump` and
24 leftover `JumpUnless`, which reconciles against the 789 `JumpUnless` at BASE
from the other direction. The adjacency is structural: both ops are pushed
back-to-back in the same `match` arm at both sites that emit either.

Gates at `725aa8863`: six green, 133 binaries, 2649 passed / 0 failed / 4 ignored
release and 2650 / 0 / 4 debug, tallies unchanged from before the commit. The
clippy line finished in 0.09s against a warm target and was re-run from an empty
`CARGO_TARGET_DIR`, 74 crates cold, rc 0.

Controls: `emptyloop` -0.00018% and `varlookup` +0.00007%, both inside their own
builds' spreads and pointing opposite ways, neither program containing an `IF`,
`WHEN` or `SELECT`. **No arm was added**: the driver has one arm where it had two.

### The correction this forces on every remaining estimate

**"Ops removed times 20" is a floor, not a ceiling.** 20.0 Ir is the price of a
dispatch that does *nothing*, derived by deleting trace ops, which are exactly
the ops that do nothing. This fusion removed 54.8 instructions per op, and
**34.8 of the 54.8 were the value handoff between the pair**: `ObjRef::small_int`
and `set_temp` on the write side, `temp_at`, `register_holds`'s comparisons and
`decode` on the read side, none of which survives because the branch was the
write-back's only reader.

So any fusion that also removes a value handoff is underestimated by "ops times
20", by an amount equal to whatever register traffic sat between the two ops. An
elision that removes a dispatch doing nothing is not.

This re-prices item 5's hot-body route from 0.165% to roughly 0.45%, above the
0.3% floor it was declined against. It does **not** revive item 5 as written: its
hot pairs are separated by an `Op::TraceLiteral` that still has to emit, so
reaching them means a three-op fusion that keeps the emission rather than the
two-op one that was measured at six executed pairs.

An eight-probe differential over the fusion, stdout, stderr and exit status kept
separate, is byte-identical across oracle, BASE and HEAD: non-logical `IF` at
34.1 and `WHEN` at 34.2, `trace i` over `IF`/`ELSE`, a `SELECT` chain in a loop,
`trace r` over `SELECT CASE`, nested `IF` under `SIGNAL ON SYNTAX`, and the
object-comparison case from item 4 reaching both an `IF` and a `WHEN`.

### Item 5 is revived as a three-op fusion, and my re-pricing of it was wrong twice

I re-priced item 5's hot route at 54.8 Ir per op and got ~0.45%. Both halves of
that were wrong, in opposite directions, and the corrected figure clears the
threshold at its floor.

* **54.8 is not transferable.** It decomposes as about 20 of dispatch plus about
  34.8 of the handoff *that* pair had. A constant-load-and-store fusion removes
  `set_temp` and `temp_at` and none of the rest, a strict subset and the cheaper
  half. So 54.8 is item 5's upper bound, 20.0 its lower bound, and nothing
  between them has been measured.
* **A three-op fusion removes two ops per execution, not one.** The hot pairs are
  separated by a live `Op::TraceLiteral`, so reaching them means fusing
  `LoadConstant` + `TraceLiteral` + `Store`. At 1,740,000 executions that is
  about 3,480,000 ops.

| price | ops removed | instructions | share |
|---|---|---|---|
| 20.0 Ir, the floor | 3,480,000 | 69.6M | **0.33%** |
| 54.8 Ir, the ceiling | 3,480,000 | 190.7M | **0.90%** |

All DERIVED, and resting on the measured 1,740,000, which is the number to
re-derive before acting.

**The fusion is coherent, read from the code rather than assumed.** `compile.rs`'s
`Assignment` arm calls `push_value`, whose `ExprKind::Literal` arm pushes
`Op::Const`, then `Op::TraceLiteral` when `echoes_values`, and the arm then
pushes `Op::Store`. The three are emitted adjacently at one site, so the same
emission-time technique applies with the same property that nothing renumbers.
Nothing is skipped: the driver's `Op::TraceLiteral` arm gates on the setting and
calls `echo_literal`, and a fused op does the same gate and the same echo before
storing.

**Item 5's value on `rexxcps` is a function of item 2**, not of anything about
the fusion: the echoes exist because `trace_flow::analyse` answers `Unknown` for
the whole body, since two of the three `TRACE` instructions are `trace value
<expr>`. A body that stopped echoing would make the two-op fusion apply directly
and this item would need no three-op form at all.

## Item 8 opened 2026-09-21: PARSE, diagnosed and two candidates landed

`exec_parse`'s 6.77% in item 8's table is its **self** cost. Inclusive it is
**14.90%**, confirmed by `callgrind_annotate --inclusive=yes`, by summing self
plus every call arc, and by an A/B that replaced the `PARSE` instructions with
`nop` and removed 2,673,174,085. PARSE is **1.75% of dispatched ops and 14.90% of
instructions**, which is the clearest case on the list of the two axes pointing
opposite ways.

**And it is not where we are losing.** The oracle spends **21.49%** of its own
run on `RexxInstructionParse::execute` over identical structural work: the same
2,240,011 instructions, 2,520,011 template steps, 3,920,018 triggers. The whole
headroom against the oracle's design was 854,376,237 instructions before any of
this landed. The ratio is not quoted because `build/` is `-O2`.

Two candidates died in the measuring, before anything was built:

* **The variable pattern `(p0)` is free.** Replacing it with the literal it
  always holds made the run 5,639,869 instructions **slower**. There is no
  variable-pool lookup on that path.
* **The allocations are required.** Predicted 2,520,000 from the template shapes
  before reading the profile; `alloc_with` is called 2,520,002. The oracle makes
  4,480,008 for the same targets.

### What landed

| commit | candidate | measured | attributed |
|---|---|---|---|
| `1da095de8` | a store for a `PARSE` target that is a plain variable with a bound slot | **-301,950,504, -1.4520%** | 202,926,832, 0.96% |
| `bb81ae522` | decide the trace shape once per trigger, not once per target | **-47,957,027, -0.2340%** | 68,880,034, 0.33% |

Cumulative **-1.6784%**, 20,794,787,956 to 20,445,766,416. Six gates green on
each, 133 binaries, 2649 / 0 / 4 release and 2650 / 0 / 4 debug, with
`corpus_differential` 604 of 604 in STRICT mode across all four test runs.

**Candidate 3 beat its ceiling by half again**, for the same reason the
`Condition` fusion beat its estimate threefold: the ceiling priced one side of
the change. It counted `assign_expr_target`'s own per-call lines and not the
caller's call setup, `parse_template.rs:771`, which also went. That is two
instances in one day of an estimate wrong in the direction nobody guards against.

**Candidate 4 is once per trigger, not once per `PARSE`**, established by reading
`ParseTrigger.cpp:248` and `:290` rather than by inference, and asserted in the
loop with a `debug_assert_eq!` rather than argued. Re-derived at that scope its
ceiling is 45,919,904 and the measurement is 104% of it.

### Candidate 5 measured slower, and that is the finding

**+6,624,562, +0.0324%.** Three interleaved rounds, every candidate-5 run above
every candidate-4 run. `exec_parse`'s own text is flat; `slice/index.rs` inlined
into it **rises** 19,320,012 while cursor arithmetic falls 10,360,006, the
opposite of the candidate's premise. The mechanism was deliberately left
unestablished.

The cause is an attribution expiring **inside one task**: candidate 4 made
`rendered` an `Option`, which removed two of the three indexings on an untraced
run, and candidate 5 was sized against all three. **The order the brief specified
decided which candidate banked them.**

**The rule this produces: when two contained candidates share a cost, either
order them deliberately and say why, or measure each against the same base.
Otherwise the second one's figure is not a measurement of it.**

### The re-profile, which is why candidates 1 and 2 were withheld

At `bb81ae522`: `exec_parse` inclusive **2,803,777,215, 13.71%**, and **1,251.7
Ir per `PARSE`** against 1,406.6. The gap to the oracle is now 507,290,565.

| candidate | at the scout | at `bb81ae522` |
|---|---|---|
| 1, bind the template once | 595,767,122, 2.82% | **402,640,295, 1.97%** |
| 2, parse the source in place | 402,920,653, 1.91% | **400,680,651, 1.96%** |
| 3, a bound-slot store | 202,926,832, 0.96% | **0**, the arc is gone |
| 4, hoist the trace decision | 68,880,034 | 61,040,042 remaining |
| 5, index once per target | 23,986,680 | dead, measured negative |

**Candidate 2 goes next, and not because it is larger.** Its figure is nearly all
one inclusive measurement of a call that would stop happening, reproduced to the
digit at two revisions, and it is the only one of the five whose instruction
count did not move across two landed commits. Candidate 1's has already been
reduced once by work it did not do, and `:774` grew by 11,200,006 inside it, so
part of what remains is cost candidate 3 created and candidate 1 would be
credited for removing. **A candidate whose figure measures the history of the
file is not sized.** Re-derive candidate 1 after candidate 2 lands.

## Item 7 sized on `rexxcps` at last, 2026-09-21

The gap the entry named, "not yet sized on `rexxcps`", closed from my own
`callgrind_annotate --threshold=100` over a run of `5e765dc5a` whose whole
program is 20,313,581,507 instructions.

| file, summed over every inlining site | instructions | share |
|---|---|---|
| `core/src/slice/index.rs` | 784,260,164 | **3.86%** |
| `core/src/slice/iter/macros.rs` | 671,356,756 | 3.30% |
| `core/src/num/uint_macros.rs` | 287,088,300 | 1.41% |
| `core/src/slice/mod.rs` | 87,278,013 | 0.43% |

The four largest `index.rs` sites: `run_ops_from::<true>` 287,039,771,
`exec_parse` 89,040,030, `run_ops_from::<true>'2` 35,000,429, and the rest spread
thin.

**This is an attribution and the recoverable fraction is unknown.** `index.rs`
covers the indexing operation, not only its bounds check, so what an assertion
removes is the check and the panic path rather than the whole figure. Everything
measured today says an attribution overstates what a fix recovers, three times in
three different ways.

**It is still the largest single item left on `rexxcps`**, ahead of PARSE
candidate 1 at 1.76%, and unlike candidate 1 its figure is not partly cost this
session created. Project memory already records asserting the bound before
indexing as worth several percent on `Index`'s panic path, and that a message on
the assertion costs more than the checks it removes.

`slice/iter/macros.rs` at 3.30% is iterator machinery rather than bounds checks
and is a separate question; it is recorded here only so that nobody sizes item 7
by adding the two.

## Item 7 landed 2026-09-21, and the attribution above is not a size

One of three candidates built under this item is in the tree. It cuts the op
stream to the bound the driver's loop already tests the counter against, so
the per-op read needs no check of its own: `Chunk::ops_upto(stop)` once at
`run_ops_from` entry, `&stream[pc as usize]` in the loop.

Measured, `valgrind --tool=callgrind`, two interleaved rounds, own
`CARGO_TARGET_DIR` per build:

| program | BASE mean | with the change | delta |
|---|---|---|---|
| `rexxcps.rex` | 20,308,361,464 | 20,292,230,586 | **-0.0794%** |
| `emptyloop.rex` | 9,998,582,491 | 9,923,587,247 | **-0.7501%** |
| `varlookup.rex` | 17,584,590,912 | 17,432,604,196 | **-0.8643%** |

**The `index.rs` share is not a size for this work, and the evidence is a sign
error rather than a magnitude one.** Across the change above, `varlookup`'s
whole program fell by 151,986,716 instructions while the `index.rs` cost
attributed to `run_ops_from::<true>` **rose** from 817,003,219 (4.65%) to
950,003,537 (5.45%). The function's own self cost fell by 152,000,241, the
whole of the program delta, so the work really did leave; it is the split of
that function's cost across source files that moved the other way.

**Two further candidates were built, tested and measured, and both are
worse.** Cutting `Chunk::positions` to the body's length so the clause
position costs no check of its own: +0.0548% on `rexxcps`, and nothing at all
on the two axes that had just moved by 0.75% and 0.86%. A second spelling of
it, reading the position at its use rather than beside the instruction lookup,
was also positive. Cutting the region's ops from `stream` rather than from
`chunk.ops`, which replaces a `Range` `get`'s two checks with a `RangeTo`
`get`'s one: **+2.015% on `emptyloop`** and +0.654% on `varlookup`.

So: three sites, one technique, one function -- -0.86%, 0.00%, +2.02% on
`varlookup`. **Nothing further under this item can be sized from a profile.**
What is left is `exec_parse` (89,040,030 on `rexxcps` at BASE, untouched, and
PARSE moved earlier the same day) and the driver's own
`code.body.instructions.get(index)`, whose index arrives inside the op with no
bound recorded anywhere that a compiler could carry. Each would have to be
built and A/B'd, and a prediction written down before the second candidate was
measured was wrong in sign.

The full write-up, with per-round figures and binary sha256s, is
`2026-09-21-bounds-check-report.md`.

### The sizing above is retired as a guide, 2026-09-21, by the work it sized

My item 7 sizing summed `core/src/slice/index.rs` over every site it was inlined
into and called it 3.86%. **That number cannot be used to choose the next site,
and the disproof came from the change it selected.**

On `varlookup` the landed change took the whole program down 151,986,716 while
`index.rs` attributed to `run_ops_from::<true>` **rose**, 817,003,219 (4.65%) to
950,003,537 (5.45%). `run_ops_from`'s own self cost fell by 152,000,241, which is
the whole-program delta, so the work did leave. **What is not trustworthy is the
per-file split inside one function.** An inlined file's share is where the
optimiser chose to attribute an instruction, and rewriting the function
reshuffles that without the work moving.

This is the same shape a `PARSE` implementer flagged earlier today and declined
to chase: `trace.rs:616` appearing under `exec_parse` from nothing while
whole-program `malloc` and `free` stayed flat. That one was left unsized on
suspicion. This one is measured.

**So: a file-level share inside an inlined function sizes nothing.** Use it to
find a function worth opening, never to rank sites within one, and never as a
recoverable figure.

### Three signs from one technique, and the predictions were wrong

The same idea applied at three sites in one function on `varlookup`: **-0.86%**,
**0.00%**, **+2.02%**. The two that came out positive were built, passed the
`rexx-exec` debug suite, measured, and reverted.

* Cutting `Chunk::positions` to the body's length so the clause position rides
  the instruction lookup's check: **+0.0548%** on `rexxcps`. A second spelling of
  the same idea was also positive, so placement was not the cost.
* Cutting the region's ops from the same stream instead of a `Range` get, two
  checks to one: **+2.015%** on `emptyloop`, **+0.654%** on `varlookup`. The
  re-slice costs more than the check it removes.

A prediction and a rival hypothesis were written down before the second
measurement. **Both were wrong in sign.** They are kept beside the report.

## 2026-09-23 and 24: the driver rounds, recorded elsewhere

Five rounds ran after this list's last item, and their records live in
`docs/superpowers/records/2026-09-23-driver-spikes/`, one README section per
round. What this list needs to know from them:

* **Landed**: the register frame arena (`rexx-core/src/frame.rs`, `unsafe`
  granted by Moritz 2026-09-23), four clause-overhead changes and one spill
  change. `rexxcps` -1.665%, -1.695% and -0.668% against each round's own base.
* **Closed by measurement**: per-arm `#[cold]` outlining (+0.94%), one
  function per op through a table (+10.04%), and the tree-walker as an engine
  (+28.07% against the IR; the gap is its recursive expression evaluator).
* **The oracle's lead is a fixed per-clause cost**, not per-operation work: a
  `nop` clause is 33.3 instructions in the oracle and 90 in the IR after these
  rounds, and where a construct's own work dominates we are at parity or ahead
  (`tw-vs-oracle.md`).
* **Instruments falsified**: the driver's frame size (already, 2026-09-22) and
  now its executed spill count; neither predicted the instruction count of any
  rejected candidate. Retired instructions on the axes remain the judge.
* **Still open**: harvesting the PGO in-sample ceiling (-12.78% on `rexxcps`
  with no source change) into source; a perturbed-build study of the driver's
  noise; whether to ship PGO at all (Moritz's decision).
