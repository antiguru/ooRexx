# Phase 4e spike -- findings

Branch `spike/ir-mechanics`, off `plan/rust-rewrite` at `d1bf9dad`.
Answers the mechanics the design spec asserted without evidence, each by running it rather than arguing it.
All five units ran and are committed on that branch.
S3's price is settled by reading rather than by measurement, and S4 records why the measurement it was meant to give is not available yet.
The branch is not proposed for merge: Task 0 of the Phase 4e plan re-lands the parts that belong on the main branch.

## S2 -- where the register file lives (`4b285394`)

**Both designs the spec proposed are wrong.**

A register `SlotFrame` of its own makes `grow_slots` on the variable frame beneath it panic, and that assertion is deliberate.
A program reaches the growth on `INTERPRET` introducing a name, on `DROP (v)`, and on the first `CALL` in a program that never writes `RESULT`.

Extending the activation's own `push_slots` request reaches none of the cases that matter, because they push no frame at all.
`CALL label` reuses the caller's frame outright, `INTERPRET` pushes nothing and runs inside the enclosing activation, and a `CALL ON` handler enters through the same label path.
`PROCEDURE` then swaps the frame mid-body, so even where there is a push, its size is not known when the chunk starts.

**Registers go in the temporaries stack**, as an indexable region: `reserve_temps`, `temp_at`, `set_temp`.
Temps are independent of slot frames, are already roots the collector walks, and nest -- which is what a fragment chunk compiled and run inside a running chunk needs.
The stack also carries no balance assertion by decision, so watermark-truncate survives a condition unwinding through a chunk.

**Consequence for the spec.** The `spike/bytecode-vm` measurements used a separate register `SlotFrame`, so that design cannot run a `CALL`, an `INTERPRET` or a `DROP`.
Its headline figures were taken on a shape that does not survive contact with those, and what the working shape costs is unmeasured.

## S1 -- extracting the driver's per-clause obligations (`614388fc`)

`run_activation`'s loop is semantics, not plumbing, so a second driver written beside it is a second copy of that semantics.
Two obligations extract cleanly: `grant_procedure_permission`, which takes the `Instruction` because the rule is per clause and a `Label` neither grants nor spends it, and `apply_flow`, which returns `Ok(None)` to continue or `Ok(Some(Ended))` to finish and is where the 28.1-28.4 family now lives once.

The trap offer stays at the call site deliberately.
Its position **is** the semantics -- one offer per activation, made by the activation that is unwinding -- and moving it inside would give a nested `run_bounded` a second offer.

**`run_fragment`'s `Leave`/`Iterate` arms are not a call site for `apply_flow`.**
A review called them a byte-for-byte duplicate; that is imprecise.
They are the same event at a different boundary: the same four constructors and the same `record_leave_failure` call, but they resolve the name against the *fragment's* own symbol table, which is the last point at which the interned id means anything, and they `seal_site_level` first.
So "extract the loop into a form both engines use" is achievable at the activation boundary and is **not** one universal contract.

**The `Generic` payload is just an instruction index.**
Reconstructing `Code` while holding `&mut self` is already solved: `run_activation` clones the `Rc<Program>` and `Rc<Plan>` into locals and builds `Code` from those, so it outlives every `&mut self` call in the loop.
An IR driver does the same once per chunk-run.

Behaviour preservation: the workspace suite passes and the corpus differential reports **51 of 51 matching** under `REXX_CORPUS_GATE=1`.
A plain `cargo test` does not establish this -- that runner defaults to REPORT mode and exits 0 whatever it finds.

## S3 -- the promoted clause-unit contract

The spec defines a `Clause` op as carrying "the source line and the static indent".
Reading `step_in_temps_frame` against that shows two things it cannot do.

### A `Clause` op must carry the instruction index, not only line and indent

The clause unit has eight obligations, and three of them need the **instruction node itself**, not a precomputed line and indent:

* `clause_site(source, instruction)` produces the clause *text* for the `*-*` echo.
* `record_failure_site(code, index, source, instruction)` resolves the failing clause.
* `clause_line(source, instruction)` derives the line, with `clause_state.line()` as fallback.

The other five are stores and scoping: `clock_stale`, the `>I>` `TraceEntry` decay, `current_value_indent` from `printed_indent`, the temps watermark and frame, and the debug tripwire.

So a promoted clause still reaches its AST node.
`Clause { index, line, indent }` is the shape that works, with line and indent as a memo of what `clause_line` and `printed_indent` would recompute.

**Full independence from the AST is not available while trace echo and failure sites are byte-compared against the oracle.**
The `spike/bytecode-vm` prototype hid this by refusing to run whenever tracing was on, which is why its `Clause` op appeared to need only two fields.

### The driver is a two-level loop, not one flat loop

`in_clause(code, line, body)` is a **scoped closure**: it sets the clause line, runs the whole clause, and then -- only on the success path -- delivers any queued `CALL ON` trap, returning `ClauseOutcome::Ended` when the handler exits.
A flat op stream has no scope to hang that on, and a clause spans a run of ops.

So the driver's outer loop iterates **clauses**, and `in_clause`'s closure runs that clause's ops until the next `Clause` op.
One level of nesting, driven by op indices rather than recursion, against the tree-walker's five layers.

This corrects the spec's "one flat dispatch loop" framing, and it is where the trap-delivery obligation lands -- the spec's residue list said trap delivery "may remain delegated", which is incoherent for the *check*, because a promoted region has no `Generic` op to carry it.

**The price is unmeasured.** A closure call and a `pending_trap` branch per clause is what the two-level shape costs over the prototype's inline stores, and the prototype paid neither.
S4 owns that number, and the spec's stopping rule cannot be anchored until it exists.

### Trace as instructions in the stream

Decided 2026-08-08. Adopt it, with a stated limit on what it reaches.

**What it buys.**

* **It is the concrete realisation of D23.** The spec already decided the trace setting is an input to compilation, "one engine, two chunks". Explicit trace ops make that a visible difference in the op stream rather than a mode flag threaded through every arm.
* **It kills the double-pay rule.** The spec had to say "a `Clause` op must not precede a `Generic` op", because `step_in_temps_frame` already echoes and the echo is not idempotent. If the echo is an op, who pays is visible in the stream instead of being a convention nobody can assert.
* **It makes trace testable where nothing currently tests it.** No corpus instrument in this repository can see a trace indent: `tests/support/mod.rs` normalises at `PREFIX_OFFSET` 7..10 and collapses the leading space run after the marker, which is exactly where nesting indent is rendered, while listing everything it leaves untouched. Golden op-stream tests would cover it, and those are needed anyway because `Generic` masks compiler defects from the differential.
* **The untraced chunk pays literally nothing**, not even a flag test per clause, which is aimed straight at the fixed per-clause overhead this phase exists to remove.
* Explicit ops are schedulable and elidable by a Cranelift or wasm backend; side effects buried in a runtime helper are opaque to one.

**What it cannot reach.**

* **Some of that bookkeeping is not trace.** The clause *line* feeds `SIGL`, condition objects and syntax error messages, and failure-site resolution has to work with trace off. So the clause op is not purely elidable: it splits into an unconditional semantic half and a conditional echo half. Explicit trace ops shrink the clause-unit contract; they do not remove the need for one.
* **`TRACE` is dynamic** -- `TRACE VALUE expr`, a mid-program `trace`, inheritance across activations -- so emission cannot be fully decided at compile time. This relocates the deopt problem rather than solving it, though the spec already committed to recompiling a body when the setting rises.
* **Interactive trace is not an echo at all.** `?` reads standard input and can execute typed clauses and re-run the current one. That is control flow, not an emit.
* **Intermediate-level trace lives inside expression evaluation**, and under `Generic` the expression still runs through `eval.rs` and traces as a side effect. The two regimes therefore coexist until expressions are promoted, which is a drift surface -- the same one that makes `Assignment`'s absence from the minimum promotion set matter.

**Net.** Worth adopting, and it makes the trace section stronger than "compile unoptimised".
The honest framing is that it splits the clause boundary into **unconditional semantics and conditional emission**, and only the second becomes ops.
That distinction is exactly what the design is missing: the `Clause` op is currently defined as line-plus-indent and nothing else, with no contract for the rest.

## S4 -- promoting `Assignment` end to end

A minimal stream in `rexx-exec/src/ir.rs`: `Generic` delegates a whole clause to `step_in_temps_frame`, `Clause` opens a natively-compiled one, and an assignment whose value is a literal compiles to `Const` plus `Store`.
`Store` goes through `assign_expr_target`, which is what `step`'s own arm calls, so stems, compound tails and the `>=>` line are shared code rather than a second implementation.
The engine switch is a mode on `Interp` consulted at the top of `run_activation`, which covers callees for free.

### The boundary holds, and one divergence was real

Eleven tests compare the tree-walker against **both** IR arms on all three descriptors.
The all-`Generic` arm is the control: a disagreement there is the driver's, and one only in the promoted arm is the promotion's.
They cover the delegation boundary (a promoted clause inside a delegated `DO` and inside both branches of a delegated `IF`), `PROCEDURE` with its label-transparency rule, `INTERPRET`, the clause echo, intermediate trace, a failure site, a `CALL ON` handler delivered at a promoted clause's boundary, and `SIGL`.

**One failed, and it is the drift surface the design predicted but had never run.**
Under `trace i` the promoted clause dropped the literal's `>L>` intermediate line, because `eval.rs` emits it as a side effect of evaluating and a native `Const` op does not.
Every following line still matched, so nothing but an exact stderr comparison sees it -- and no corpus instrument in this repository does.
**Promoting an expression silently drops intermediate trace unless each op re-emits it**, which is what makes trace-as-instructions a correctness requirement rather than a performance idea.

### The `Clause` op needs none of the fields the design gave it

It was built carrying an instruction index, a line and an indent.
All three went unread and are gone.
The activation's `pc` already addresses instructions, so a clause reaches its own AST node through the map it was found by; and a precomputed line and indent cannot replace `clause_line` and `printed_indent`, whose fallbacks -- `clause_state.line()`, and a fragment's absent indent table -- are exactly the cases a memo gets wrong.
This refines S3: a promoted clause needs **access** to its instruction, not a copy of its coordinates.

### The price is still unmeasured, and the instrument is the finding

Four successive artifacts each produced a different, confident, wrong number:

* one `Vec<u8>` allocated per literal *occurrence* at compile time, so one heap allocation per clause charged to the promoted arm alone;
* `to_text(value).to_vec()` computed before calling `trace_literal`, which gates internally -- a full render and allocation per clause on an untraced run, worth about 39 ns/clause on its own;
* interning added to the promoted arm only, so it paid one hash lookup per assignment that the control never paid;
* and, still unequalised, three ops emitted per promoted assignment against the control's one.

With the first three removed the delta over eight runs is **+7 to +19.6 ns/clause, median about 11**, on a total of roughly 790 ns/clause.
That total is dominated by parsing a 1.8 MB program, and **each clause executes exactly once**, so compile cost amortises over a single execution and the remaining asymmetry cannot be removed within this harness.

**So the clause-unit price is not established, and this workload cannot establish it.**
It needs a chunk cached by `BodyKey` and a workload that executes the same clauses repeatedly -- which means a loop, which means loop promotion.
That is the same ordering the design already puts first, now for a second reason: without it there is nothing to measure with.

The spec's stopping rule therefore still has no anchor, and inventing a threshold before this exists would anchor it to a number no one can reproduce.

## S5 -- splitting resolution from invocation

D24 says method dispatch is implemented once as a shared `send(receiver, selector, args)` and "the IR gains a `Send` op that wraps that function with a patch-table inline cache".
Two independent reviews said a cache cannot wrap an opaque function, because a cache stores what a *lookup* returned and a fused function offers nothing to memoise.
The classic-call path is the proxy: same four-step resolution, same fusion.

### The seam already exists, and opening it is mechanical

Resolution was **already** a distinct phase producing a `Resolved` enum -- `Label(usize)`, `Builtin`, `Routine(InstalledRoutine)` -- computed upstream of the argument loop, with a comment recording that the shape is "load-bearing rather than tidy" because the builtin path needs its arguments already evaluated.
Extracting `resolve_call(name, search_labels) -> Result<Resolved, Failure>` was a splice.
Workspace suite green, corpus 51 of 51.

### A call-site cache is expressible, and it hits

Keyed on the body being run, the target name, and whether labels were searched, which is everything the resolution depends on.
Nothing invalidates it at this phase: `Resolved::Label` indexes the running activation's own body, the builtin table is static, and `Interp::routines` is written only by `install_directives`, at load.

Suite and corpus stay green with it on.
**Mutating the hit path to return the wrong `Resolved` kills nine test binaries**, so the green run is load-bearing rather than a cache that never fires.

### What this does not establish, which is the part that matters

**The cache built here is correct precisely because it needs no guard, and that is exactly the property a send cache lacks.**

* A classic call's resolution is **stable** for a call site. A send's is keyed on the *receiver's behaviour* and varies execution to execution, so it needs a guard -- check the receiver's behaviour against the cached one -- that this proxy never exercises.
* **D22's safety schema does not generalise to it.** "A hint that never removes a precondition check" works for quickened arithmetic because the precondition is O(1) and locally checkable. A send's precondition is "the lookup would still return this method", and checking that *is* the lookup. Real inline caches substitute behaviour identity plus **invalidation on behaviour mutation**, which is a global protocol.
* The forward-constraints list in the spec names selector interning, a `SmallInt` behaviour arm, a receiver in the calling convention and a wider patch entry. **It omits invalidation entirely**, and that is the one with a design cost rather than a mechanical one.

So D24 survives in its conclusion and not in its wording.
The amendment it needs: the shared surface is `resolve` **and** `invoke`, not one `send`; the IR's `Send` op caches the first and calls the second; and behaviour-mutation invalidation joins the forward constraints.

### Not proposed for the main branch

The resolution cache is a spike artifact, kept because it is what proved the seam.
It is a real speedup for repeated calls and it is out of scope for a phase whose gate is a bar written before any optimisation lands.

## Still open

* **The clause-unit price**, behind chunk caching and loop promotion.
