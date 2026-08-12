# What is left to promote, and what cannot be

Written 2026-08-12, after the expression-promotion plan's Tasks 1-3 landed and Task 4 was in flight.
Two parts: a plan for the next increment, and a survey of every construct still delegating, with the blocker named where there is one.

## The rule that decides whether promoting something is worth anything

`Op::Generic` hands a whole clause to the tree-walker's clause unit, and that unit is not the slow part.
**Promotion pays where an *expression* is involved** -- a native op replaces an `eval` call, its wrapper, its depth bookkeeping and its post-order trace hook -- **or where control flow is involved**, because a jump inside the stream replaces a `Flow` travelling back out through `run_bounded`.
Where an instruction's whole body is one call into an `exec_*` function, promoting it saves one `match` dispatch and buys nothing measurable.

That rule is what sorts the survey below, and it is why the plan is short.

---

# Part 1: the next increment

Two families, in this order.
Neither needs a new addressing scheme, a width change or a new table.

## Task A: the conditional comma list

`ExprKind::Logical(Vec<Expr>)` is `IF a, b THEN` and the `WHEN`/`WHILE`/`UNTIL`/`GUARD` versions of the same syntax.
It is the shape every multi-condition branch takes, and today one anywhere in an expression sends the whole slot to `Op::EvalExpr`.

**What it does** (`Interp::eval_logical_list`): pushes a roots frame; for each item evaluates it, pushes it as a temp, renders its text, emits a `>>>` line at the clause's value indent, and checks `logical_value`, raising **34.6** on an element that is not `0` or `1`; stops at the first false; answers `1` when every element held.
Two measured facts the promotion must keep: **the `>>>` per element fires under `TRACE R` alone**, through the `results`-gated `trace_result` rather than `eval`'s intermediates hook; and **a failing element traces before it raises**.

**The shape it compiles to.** A list is a sequence with an early exit, which is what a stream is good at:

```
   <ops for item 0 -> dst>          // native ops, or Op::EvalExpr for the item
   Op::LogicalItem { src: dst, dst } // >>>, logical_value, 34.6, leaves 0/1
   Op::JumpUnless { reg: dst, target: end }
   <ops for item 1 -> dst>
   Op::LogicalItem { src: dst, dst }
   Op::JumpUnless { reg: dst, target: end }
   ...
end:
```

`dst` carries the answer at `end` on both routes: a false element leaves `0` there and jumps, and a run that never jumps leaves the last element's `1`.
So nothing about `IF`'s existing `Op::JumpUnless` on the condition register changes, and `SELECT CASE`'s own comma list is untouched -- it is `WhenCase`'s `values`, a different node.

`Interp::logical_item(&mut self, value: ObjRef) -> Result<ObjRef, Failure>` is the shared half, split out of `eval_logical_list` at the same seam this plan split `concat`, `eval_compare`, `eval_logical` and `eval_prefix`: the trace, the `logical_value` check and the 34.6 in one function both engines enter, with the *sequencing* -- which is what a stream buys -- in the ops.

**Why the items are not a native `&&` chain**: the list short-circuits and `&` does not, which is measured behaviour these two arms already implement as opposites.
An implementation that compiled a list to `Op::Binary { op: And }` would evaluate an element the oracle never evaluates.

## Task B: the value-returning instructions

`RETURN`, `EXIT`, `PUSH` and `QUEUE` are one shape: evaluate an optional expression, root it, trace it, and produce a `Flow`.
They are `SAY`'s arm with a different tail, and `SAY` is already promoted.

* `RETURN` ends every routine body, so it is the last clause of every call the interpreter makes.
* `EXIT` ends every program.
* `PUSH`/`QUEUE` share `SAY`'s own `evaluateStringExpression` in the C++ and share its trace line here.

Each becomes `<ops for the expression -> dst>` followed by `Op::Return { index, src: Option<u16> }` (and its three siblings), whose driver arm calls the same `Interp` half `step`'s arm calls and then breaks the region with the `Flow` that arm returns.
`Op::Say`'s arm is the template, including `src: Option<u16>` for the bare form -- which for `RETURN` is a real distinction (`RETURN` with no expression leaves `RESULT` unset, `RETURN ''` sets it).

`NUMERIC DIGITS expr` has the same shape with a different tail and can ride along if the register discipline stays identical; `TRACE`, having no expression, should not -- see the survey.

---

# Part 2: the survey

## Already promoted

`DO`, `LOOP`, `IF`, `SELECT`, a listed `WHEN`/`WHEN CASE`, assignment, `SAY`, `CALL name`.
Expressions: literals, constant symbols, the three bare-symbol reads, every prefix operator, every binary operator, and -- once Task 4 lands -- a call at any depth the path carries.

## Promotable, and the sketch is the same each time

**One expression then a `Flow`**: `RETURN`, `EXIT`, `PUSH`, `QUEUE`, `NUMERIC` with a computed setting, `INTERPRET`'s own expression (see below for the fragment), `SIGNAL VALUE`'s.
Part 1's Task B is the pattern; each is an op carrying the instruction index and an optional source register.

**A list of expressions**: `DROP`, `PROCEDURE EXPOSE`, `USE ARG`, `RAISE`'s several expression positions, `PARSE`'s source expression where it has one.
Each is Task A's shape -- ops per element, then an op that consumes them -- but the payoff is much smaller, because these run once per activation rather than once per iteration.
**`USE ARG` is the interesting member**, because a compiled `USE ARG >name` could resolve the caller's slot at compile time rather than at run time; that belongs with the argument work below rather than here.

**Control flow that leaves the clause**: `LEAVE`, `ITERATE`, `SIGNAL label`.
The first two already answer a `Flow` the driver propagates; promoting them is an op that produces the same `Flow` without entering the clause unit, worth doing only when they sit in a hot loop.
**`SIGNAL label` is the one with a real design question**: it is a non-local goto that unwinds every enclosing construct, and the stream's `op_of` gives the target op, so the mechanism exists -- what needs care is that a jump out of a `Clause` region must run the same teardown `Flow::Goto` currently causes on the way out. Do not promote it before the loop and select frames' teardown is expressed as ops.

**Structure with nothing to promote**: `THEN`, `ELSE`, `OTHERWISE`, `END`, `LABEL`, `NOP`.
Each is `Ok(Flow::Next)` or, for `END`, a match on the style that raises for a `SELECT` with no `WHEN`.
Their constructs already absorb them; an op would save one dispatch.

**One call into an `exec_*` function**: `TRACE`, `ADDRESS`, `PROCEDURE`, `PARSE`/`ARG`/`PULL`, `RAISE`.
Promoting these is mechanical and the rule at the top says it buys nothing.
They are listed so that "why is `PARSE` still `Generic`" has an answer written down, not because they are queued.

## Blocked, with the blocker

* **`INTERPRET`'s body.** The instruction's own expression is promotable; the *fragment* it builds is not. Interpreted text is a different `CodeBody` from the one a chunk's `op_of` indexes, so `run_fragment` passes `TreeWalker` unconditionally. Removing the blocker means compiling a fragment to its own chunk, cached under its own key -- which is a real design, not a task: the text is computed at run time, so the cache key is the text, and an `INTERPRET` in a loop would compile once per distinct string.
* **`CALL (expr)`** learns its name at run time, so the resolution site a compiled `CALL` keeps has nothing to key on. It could still take an op that skips the clause unit while resolving afresh; the site cache is what it cannot have.
* **`CALL ON`/`OFF`** resolves no name at all and is condition-handler bookkeeping.
* **`MESSAGE`, `COMMAND`, `EXPOSE` (standalone), `GUARD`, `REPLY`, `FORWARD`, `OPTIONS`** are not implemented in **either** engine -- they reach `step`'s loud fallback, and `run.rs` names none of them. They cannot be adopted into the IR because there is nothing to adopt; they are Phase 5's object model and the concurrency instructions.
* **A body that overruns a machine width** is refused whole and runs on the tree-walker, bumping `chunks_refused`: registers past `u16`, the op stream past `u32`, constants past `u32`, expression slots past `u16`, call sites past `u16`, arithmetic sites past `u32`. Not a language blocker; the refusal is deliberate and is the compiler's one error.

## The expression shapes still declining, and what each needs

* **`ExprKind::Logical`** -- Task A above.
* **`DotVariable`** (`.nil`/`.true`/`.false`): trivially promotable as a native op, since the value is a constant per name. Worth nothing on any measured axis; do it when something else touches that arm.
* **`VariableReference`** (`>name`/`<name`): promotable, and it traces as an operator rather than a read. It matters only as a *call argument*, where it is not an expression at all but an `Argument::Reference` -- so it belongs to the argument work, not here.
* **`QualifiedCall`, `ClassResolver`, `Message`, `List`**: the object model. Not implemented in either engine.

## The change that would make `NodePath` unnecessary

Compiling a call's **arguments** into a contiguous region, so a call op carries `(argc, base)` and never needs to find its node.
The registers already live on the temporaries stack (`run_chunk` reserves them with `reserve_temps`), so the region needs no new machinery.

**What makes it a plan rather than a task** is that an argument is not a value:
`Argument` is `Value(ObjRef)` or `Reference { target: SlotRef, value: ObjRef, name: Box<[u8]> }`, and a position can be absent -- `call sub 1,,3` is `[Some, None, Some]`, and the omission holds its place rather than closing up.
So the region's element is `Option<Argument>`, and the ops that build it are `ArgValue`, `ArgOmitted` and `ArgRef` rather than a single push.

**It needs no second implementation**, which an earlier note in this project claimed: `invoke_call`'s argument loop is a prologue that builds `Vec<Option<Argument>>`, and everything after it -- the builtin outcome, `SIGL`, the depth guard, the activation push, the `Ended` arms -- consumes that vector. Split it at that line into a gather half and an invoke half, exactly as `concat`/`eval_compare`/`eval_logical`/`eval_prefix` were split, and both engines enter the second.

**The two things that need care.** The `>A>` indent is re-read from `current_value_indent` at every position on purpose, because an argument's own callee overwrites it -- so the echo ops must read it live rather than carry it. And a compiled `ArgRef` would resolve the caller's slot at compile time, which is a *better* answer than the run-time one and therefore needs its own differential evidence rather than being assumed equivalent.
