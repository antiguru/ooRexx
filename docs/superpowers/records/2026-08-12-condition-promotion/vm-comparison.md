# The two-level driver, placed against other VMs

A read-only comparative analysis of `Interp::run_ops`' nested region loop
(`rust/crates/rexx-exec/src/ir/drive.rs`) against how other bytecode and IR
virtual machines run per-statement work, and what the ooRexx C++ oracle itself
does.

Nothing in any repository was modified to produce this.

## How to read the confidence tags

Every load-bearing claim carries one of these:

* **[read]** — read in a local repository during this analysis, with the path
  and line cited. These are the claims to trust.
* **[ran]** — observed by running a program on this machine during this
  analysis, with the command shown. Also trustworthy, within the limits of what
  one program shows.
* **[recall-high]** — recalled from the implementation; I would be surprised to
  be wrong about the shape, though names and version numbers may drift.
* **[recall-med]** — recalled; the shape is probably right, a detail is probably
  wrong. Verify before betting a rewrite on it.
* **[recall-low]** — I half-remember this. Treat as a pointer to look it up, not
  as a fact.
* **[principle]** — reasoning from general interpreter design, not a claim about
  any particular implementation.

Where I do not know something, it says so rather than filling the gap.

---

## 1. What the design actually is

Read before anything was compared, because the brief's summary is close but not
exact in two places that change the answer.

### 1.1 There are three levels, not two

**[read]** `rust/crates/rexx-exec/src/ir/drive.rs`:

| Level | Function | What it loops over | What it owns |
| --- | --- | --- | --- |
| L1 | `Interp::run_chunk_clauses` (drive.rs:190) | activation re-entries | `offer_to_trap`, `apply_flow`, the activation `pc` |
| L2 | `Interp::run_ops::<GRANTING>` (drive.rs:310 onwards) | ops, by a `pc: u32` | construct-level flow, the `SelectFrame` stack, `absorb` |
| L3 | the `'region` labelled block (drive.rs:511–1256) | one clause's ops, as a slice | the clause's own work |

L3 is entered from L2's `Op::Clause` arm and reads
`chunk.ops_in(pc + 1, end)` — a `&[Op]` — then `for region_op in ops`
(drive.rs:522–525). It has no counter of its own.

The brief calls this a two-level loop. The L1/L2 split is a third level and it
matters for the comparison, because **L1/L2 is the ordinary shape and L3 is
the unusual one** (§4).

### 1.2 A jump inside a region ends the clause

**[read]** drive.rs:1125–1136 and 1233–1235:

```rust
Op::JumpUnless { reg, target } => {
    match self.register_holds(registers, *reg) {
        Ok(true) => {}
        Ok(false) => break 'region Ok(RegionEnd::At(*target)),
        Err(failure) => break 'region Err(failure),
    }
}
...
Op::Jump { target } => {
    break 'region Ok(RegionEnd::At(*target));
}
```

`break 'region` leaves the region carrying `RegionEnd::At`, which
drive.rs:1258–1270 hands to `leave_stepped_clause` and then assigns to the L2
`pc`. So the clause's boundary has *run* by the time the jump takes effect.
That is correct for the jumps emitted today — an `IF`'s `JumpUnless` is the
last op of its region (compile.rs:438–444, then `close_region`), and its target
is a different clause — and it is exactly what makes a jump that must stay
inside one clause inexpressible.

### 1.3 The blocked feature is narrower than the brief says

The brief names "`IF a, b THEN` comma list, short-circuit `&`/`|`". The second
half is not a Rexx feature.

**[read]** `/home/moritz/dev/repos/ooRexx/interpreter/expression/ExpressionOperator.cpp:179–190`:

```cpp
RexxObject *RexxBinaryOperator::evaluate(RexxActivation *context, ExpressionStack *stack )
{
    // evaluate both expression terms
    RexxObject *left = left_term->evaluate(context, stack);
    RexxObject *right = right_term->evaluate(context, stack);
```

`&` and `|` are ordinary binary operators and evaluate both operands
unconditionally. There is no subclass overriding this for AND/OR
(`/bin/grep -rn "public RexxBinaryOperator" --include=*.hpp` finds none).

**[read]** `.../interpreter/expression/ExpressionLogical.cpp` — the comma list
is `RexxExpressionLogical`, a *term* in the expression tree whose `evaluate`
loops the sub-expressions and `return TheFalseObject`s on the first false one.
That is the only short-circuiting construct in the language.

**[read]** The Rust side agrees: `ExprKind::Logical(Vec<Expr>)`
(`crates/rexx-parse/src/ast.rs:216`) is built only by the `parse_logical`
translation of `LanguageParser::parseLogical`
(`crates/rexx-parse/src/expr.rs:664–675`), used by `IF`/`WHEN`/`WHILE`/
`UNTIL`/`GUARD`; and `Interp::eval_logical_list`
(`crates/rexx-exec/src/eval.rs:1122–1142`) breaks on the first false element.

So the prize for an intra-clause forward branch, **today**, is exactly one
thing: promoting a comma-list condition. Everything else already works. And it
is not broken today either — `native_shape` (compile.rs:836–859) has no
`ExprKind::Logical` arm, so a comma-list condition falls to the `Op::EvalExpr`
whole-expression path and is evaluated by the tree-walking evaluator inside the
promoted clause. It is unpromoted, not wrong.

That is worth stating plainly before recommending anything: the immediate prize
is small. The argument for change has to be made on the ceiling, not on the
comma list (§6.4).

### 1.4 What the clause boundary owes

**[read]** `crates/rexx-exec/src/run.rs:4732–4855` (`enter_stepped_clause`) and
4856–4900 (`leave_stepped_clause`). Per clause, in order:

1. `self.activation_mut().clock_stale = true` — the per-clause `DATE`/`TIME`
   cache invalidation, explicitly mirroring `settings.timeStamp.valid = false`
   in the C++ loop.
2. `trace_entry.stepped()` — the `>I>` first-instruction decay.
3. `printed_indent(code, index)` → `clause_state.current_value_indent`.
4. `clause_line(...)` → `enter_clause(line)` — `SIGL`, and the clause boundary
   at which a queued `CALL ON` handler is delivered.
5. the `*-*` echo, if `Echo::Gated` (a compiled clause carries `Op::TraceClause`
   instead).
6. `roots.push_frame()` — the temporaries frame, plus a debug watermark.

and on the way out: watermark assertion, `pop_frame`, `record_failure_site` on
either the clause's own failure or the boundary's, `leave_clause`.

Around that, L2 adds `grant_procedure_permission`, the
`mem::take(&mut self.procedure_permitted)`, and the `stale` trace-setting gate
(drive.rs:392–433).

This list is not negotiable — it is the language, not an implementation
choice — and every design below has to say where it goes.

---

## 2. Per-statement bookkeeping in a flat, `pc`-driven loop

Six implementations, in decreasing order of how much I could verify locally.

### 2.1 Perl — the boundary *is* an instruction, and branches live beside it

This is the closest analogue to what the Rust engine needs, and I was able to
verify the shape on this machine.

**[ran]** `perl -MO=Concise -e 'my $x = 1 + 2; if ($x > 2) { print "y\n" } else { print "n\n" } my $z = $x && 5;'`

```
2     <;> nextstate(main 1 -e:1) v:{ ->3
4     <1> padsv_store[$x:1,9] vKS/LVINTRO ->5
3        <$> const[IV 3] s/FOLD ->4
5     <;> nextstate(main 2 -e:1) v:{ ->6
9        <|> cond_expr(other->a) vK/1 ->j
8           <2> gt sK/2 ->9
...
f           <|> and(other->g) sK/1 ->h
```

Three facts fall out of that dump at once:

* **Statement boundary as an instruction.** `nextstate` is one op per
  statement, carrying package, sequence and file:line. It is dispatched by the
  same flat runloop as everything else.
* **Intra-statement branching in the same flat stream.** `and(other->g)` is a
  LOGOP (`<|>`) — Perl's short-circuit `&&` is a *branch op inside a
  statement*, not a nested evaluator. `cond_expr(other->a)` is the same shape
  for the `if`.
* The two coexist with no nesting whatsoever.

**[recall-high]** What `pp_nextstate` does is, near enough verbatim:

```c
PP(pp_nextstate) {
    PL_curcop = (COP*)PL_op;      /* current line/file: caller, warn, die */
    TAINT_NOT;
    PL_stack_sp = PL_stack_base + CX_CUR()->blk_oldsp;  /* reset the stack */
    FREETMPS;                     /* free this statement's mortals */
    PERL_ASYNC_CHECK();           /* signals */
    return NORMAL;
}
```

Item by item against §1.4: `PL_curcop` is `SIGL` and the failure site;
`FREETMPS` is `roots.pop_frame` — literally the temporaries frame; the
`PERL_ASYNC_CHECK` is the halt check. Perl pays the same per-statement bill
Rexx does, as one op, in one flat loop, and still has intra-statement branches.

**[ran]** And the cost model: `perl -d -MO=Concise -e 'my $x = 1; print $x;'`
shows `dbstate(main 1 -e:1)` where the ordinary compile shows `nextstate`.
Under the debugger a *different boundary opcode* is compiled in its place, so
the un-debugged path pays nothing for debuggability beyond the `nextstate` it
was paying anyway. **[recall-high]** Separately, `PL_runops` is a function
pointer switched between `Perl_runops_standard` and `Perl_runops_debug` — a
second copy of the whole runloop.

Perl therefore uses **three** of the cost models in §3 at once: boundary as an
instruction, second copy of the code (`dbstate`), second copy of the loop
(`PL_runops`).

### 2.2 CPython — side table when off, patched instruction when on

**[ran]** `python3 --version` → 3.14.6. Disassembling a four-statement function
(`dis.dis(f, adaptive=True)`) shows **no line instruction at all**:

```
  4           RESUME                   0
  5           LOAD_FAST_BORROW_LOAD_FAST_BORROW 1 (a, b)
              BINARY_OP                0 (+)
              STORE_FAST               2 (x)
  6           LOAD_FAST_BORROW         2 (x)
```

and `f.__code__.co_lines()` gives the side table the line numbers in that dump
came from:

```
[(0, 2, 4), (2, 18, 5), (18, 32, 6), (32, 36, 7), (36, 40, 10), (40, 44, 9), (44, 48, 10)]
```

— `(start_offset, end_offset, line)` triples decoded from `co_linetable`.

**[ran]** Turn on `sys.monitoring` LINE events for that code object and
disassemble again:

```
  3           INSTRUMENTED_LINE        1
              BINARY_OP                0 (+)
  4           INSTRUMENTED_LINE        2
              LOAD_SMALL_INT           3
```

The **first instruction of each line's range has been overwritten in place**
with `INSTRUMENTED_LINE`, keeping the displaced instruction's oparg. Turning
monitoring off restores the original opcodes exactly. The un-traced path pays
literally zero: no flag, no check, no extra op.

**[recall-high]** The displaced opcode is stashed in the code object's
monitoring data so the instrumented handler can chain to it; the mechanism is
PEP 669 (`sys.monitoring`, 3.12+). **[recall-med]** Before 3.12, `sys.settrace`
worked differently: a per-frame `f_trace_lines` flag plus a check in the eval
loop that compared `f_lasti` against the line table
(`_PyCode_CheckLineNumber`), i.e. cost model (a) rather than (b).
**[recall-high]** `co_lnotab` (pre-3.10) was a per-instruction line *delta*
table; PEP 626's `co_linetable` (3.10+) replaced it with a range table and
guaranteed every executed instruction has a line.

Note also **[ran]** `RESUME` at function entry and `JUMP_BACKWARD` at loop
back-edges: CPython's periodic checks (eval breaker, signals, GC) are at
*specific instructions*, not per instruction. That is the exact shape of the
C++ oracle's `yieldInstructions` counter — a check placed where it must be, not
everywhere.

### 2.3 Ruby YARV — side table for attribution, patched twin instructions for events

**[recall-med-high]** on the shape, **[recall-med]** on names and versions; Ruby
is not installed here and I could not verify any of it.

* `iseq->body->insns_info` is a side table of pc-range → line plus an `events`
  bitmask. Backtraces and `RUBY_EVENT_LINE` placement both read it.
* Historically (1.9 through roughly 2.5) YARV compiled an explicit `trace`
  instruction into the ISeq at each event position. You could see it in
  `RubyVM::InstructionSequence#disasm`. It was removed because it cost dispatch
  on every line whether or not anything was tracing.
* The replacement is `trace_`-prefixed *variants* of the ordinary instructions:
  the VM carries a doubled instruction table, and enabling a hook rewrites the
  instruction at the event position to its `trace_` twin (in the
  direct-threading build, rewrites the stored label pointer). The twin fires the
  event, then does the ordinary work. `rb_iseq_trace_set` is the entry point I
  remember for this.

So Ruby is CPython's model arrived at independently: side table when off,
in-place instruction substitution when on, with the substitution set generated
mechanically rather than hand-written.

### 2.4 JVM / JVMTI — side table with no runtime semantics at all

**[recall-high]** on the class-file side, **[recall-med-high]** on HotSpot's
interpreter.

* `LineNumberTable` is an attribute of the `Code` attribute: `start_pc` → line
  number. It is *debug metadata*. Nothing in the JVM's execution semantics
  depends on it, javac may attribute instructions to lines in any order it
  likes, and the JIT reorders freely.
* JVMTI breakpoints: HotSpot rewrites the bytecode at the bci to `_breakpoint`
  (0xca) and keeps the original opcode in the method's `BreakpointInfo` list —
  the same in-place patching as CPython and Ruby.
* JVMTI single-step and method entry/exit: the template interpreter keeps
  several dispatch tables (`_normal_table`, `_safept_table`, and an `_active_table`
  pointer selecting between them) and switches the active one wholesale when
  notifications are required. Compiled frames are deoptimised, and a method
  carrying a breakpoint is kept out of the JIT.

The contrast with Rexx is the important part, and it is a contrast about
*language semantics*, not about implementation: **the JVM's per-statement work
is optional**, so the JVM is free to make it arbitrarily expensive when on and
free when off. Rexx's per-clause work is mandatory — the temporaries frame and
`SIGL` are owed on every clause of every run — so the "make it free when off"
family of tricks only reaches the *trace* part of §1.4's list, never items 1, 4
or 6.

### 2.5 Lua — side table, derived boundaries, one flag folded into the fetch

**[recall-high]** on the line table, **[recall-med]** on the 5.4 `trap`
mechanism. Lua is not installed here.

* `Proto->lineinfo` is one signed byte per instruction holding the line *delta*
  from the previous instruction, with `abslineinfo[]` anchors every 128
  instructions for deltas that do not fit. `luaG_getfuncline` walks it. Pure
  side table, and a very compact one.
* Line hooks are **derived, not marked**: `luaG_traceexec` compares the line of
  the current pc against the line of the previously executed pc (`oldpc`) and
  fires on a change, with a special case that always fires after a backward
  jump. There is no statement-boundary instruction and there is no statement
  boundary in the semantics — Lua's line hook is a debug facility with
  deliberately loose guarantees.
* Cost model: in 5.4 the dispatch macro `vmfetch()` tests a `trap` flag held in
  the `CallInfo`, set when any hook is active, and only then calls
  `luaG_traceexec`. The flag is reloaded after operations that could change it.
  In 5.3 the loop tested `L->hookmask` more directly. Either way: one
  already-in-a-register test per instruction, which is cost model (a).

Lua is the implementation whose per-statement facility is *cheapest to have*
and *weakest*, and the two are the same fact.

### 2.6 V8 Ignition — side table plus a second copy of the bytecode

**[recall-med]**. Not verifiable here.

* `BytecodeArray` carries a `source_position_table`: bytecode offset → source
  position, with a bit distinguishing *statement* positions from *expression*
  positions. Stack traces and the debugger's "step over statement" both read it.
* Debugging patches break locations to `kDebugBreak*` bytecodes — but into a
  **separate** `BytecodeArray` held by the function's `DebugInfo`, with the
  original array kept intact. Optimised code is deoptimised and the function is
  marked non-optimisable while breakpoints exist.

V8 is therefore the "second copy of the code" model in its purest form.

### 2.7 The answer to the question as asked

> Which of them makes the boundary an *instruction*, and which uses a side
> table plus a flag check?

| VM | Boundary marked how, un-traced | Boundary marked how, traced |
| --- | --- | --- |
| Perl | **instruction** (`nextstate`) — always | **instruction** (`dbstate`, chosen at compile time) |
| ooRexx C++ | **instruction** — every instruction *is* a clause | same instruction, `if (tracingAll())` inside it |
| CPython 3.12+ | side table (`co_linetable`) | **instruction**, patched in place (`INSTRUMENTED_LINE`) |
| CPython ≤3.11 | side table | side table + per-frame flag check in the loop |
| Ruby YARV ≥2.6 | side table (`insns_info`) | **instruction**, patched to a `trace_` twin |
| Ruby YARV ≤2.5 | **instruction** (`trace`) — always | same instruction |
| JVM | side table (`LineNumberTable`) | patched `_breakpoint` opcode + dispatch-table swap |
| Lua | side table (`lineinfo`), boundary *derived* | same, plus a `trap` flag per fetch |
| V8 | side table (source positions) | second BytecodeArray with `kDebugBreak` |
| **this project** | **instruction** (`Op::Clause`) — always | same instruction, plus `Op::TraceClause` inside the region |

Two clean groups. The VMs whose per-statement work is *optional debug
metadata* (JVM, Lua, V8, modern CPython, modern Ruby) all use a side table and
buy the boundary only when someone asks for it. The VMs whose per-statement
work is *language semantics* (Perl's `FREETMPS`/`PL_curcop`, ooRexx's clause
boundary) all make the boundary an instruction that always runs.

**This project is in the second group, and that is correct.** `Op::Clause` is
the right call and the comparison supports it. Nothing below argues otherwise —
the argument is only about what happens *inside* the region the op opens.

---

## 3. Cost models, and which are available in safe Rust with `match` dispatch

The six models seen above, and their standing here.

### (a) Side table + a per-instruction or per-statement flag test

Lua's `trap`, pre-3.12 CPython, and the ooRexx oracle's own
`if (tracingAll())` / `if (settings.intermediateTrace)` / `if (clauseBoundary)`.

**Available and already used.** `Echo::Gated` (run.rs:4826) and the `stale`
read (drive.rs:392–406) are exactly this. A predictable, correctly-predicted
branch on a hot flag costs a few instructions and near-zero cycles.

The project's own measurements say the cost is real but small and that the
gates are worth keeping cheap: **[read]** run.rs:4905–4912 records that leaving
`echo_stepped_clause` un-inlined cost `bench-programs/emptyloop.rex` 525 million
user instructions out of ~38 billion, and drive.rs:407–431 records 4
instructions per pass for reading `stale` at the region rather than hoisting it.

### (b) In-place instruction patching

CPython's `INSTRUMENTED_LINE`, Ruby's `trace_` twins, HotSpot's `_breakpoint`.

**Technically available, and I would not do it.** The chunk is already shared
behind a cache and already carries interior mutability without `unsafe`:
**[read]** `Chunk::remember_call(&self, ...)` (mod.rs:1329) mutates through a
shared reference because `CallSite` is a `Cell<Option<Resolved>>` (mod.rs:1124),
and the workspace forbids `unsafe` (rexx-run.rs:52).

But patching the *op stream* means `ops: Vec<Cell<Op>>` rather than `Vec<Op>`,
which loses `&[Op]` slicing, loses `match region_op` borrowing of payloads (a
`Cell` only gives you `get()` by value, and `Op` is 16 bytes — **[read]**
task-1-brief.md pins `size_of::<Op>() == 16`), and makes every op read a copy.
Worse, a chunk can be running on more than one activation at once (a recursive
routine re-enters `run_chunk` with the same chunk), and patching state that is
per-*execution* into storage that is per-*chunk* is a defect the type system
will not catch. `Cell` also rules out sharing a chunk across threads, which
`Cell<CallSite>` already does — so this is not new — but widening that
commitment for a trace gate is a bad trade.

### (c) A second copy of the code

V8's debug BytecodeArray, Perl's `dbstate`.

**Available, and already the design.** **[read]** drive.rs:392–406: the chunk
cache key is widened by the trace setting, so a body entered under a second
setting compiles a second chunk. The fallback for a `TRACE` executed *while a
chunk is running* is a run-time gate (`stale`) rather than a recompile.

This is exactly Perl's `nextstate`/`dbstate` split and V8's two BytecodeArrays,
arrived at independently, and it is the strongest thing in the current design.
It is worth saying so explicitly, because it means the interesting question was
already answered well.

### (d) A second copy of the loop

Perl's `PL_runops` pointer, HotSpot's dispatch-table swap, `--enable-debug`
interpreter builds.

**Available in the cheapest possible form, and already used once.**
**[read]** `run_ops::<const GRANTING: bool>` (drive.rs:310) is a const-generic
monomorphisation whose doc explains the point: "It is a `const` parameter rather
than a value because each caller knows its own answer at the call site, which
turns a per-clause branch into no code at all."

A `run_ops::<GRANTING, TRACING>` would be the same trick for the trace gates —
two copies of the loop, chosen at the call site, with every trace branch
const-folded out of the un-traced one. Costs: code size and I-cache pressure
(the loop is already large: drive.rs:310–1400), a doubling of the
monomorphisation count each time a const parameter is added, and the discipline
that every gate must be a `const`-readable expression rather than a field read.
It is realistic. It is not obviously worth it, because the chunk-per-setting
key (c) already removes most of the same branches at compile time.

### (e) Boundary as an always-run instruction

Perl's `nextstate`, this project's `Op::Clause`. Already the design; see §2.7.

### (f) Deoptimisation

Not applicable: there is no JIT and no speculative compilation to deoptimise
from. Worth noting only because it is what lets the JVM and V8 be so cavalier
about debug support, and it is a tool this project does not have.

### What is *not* available: computed goto and threaded dispatch

**[recall-high]** Stable safe Rust has no computed goto. `match` on an enum
compiles to a single bounds-checked jump table with one indirect branch site,
which is precisely the branch-prediction problem that Ertl & Gregg's work on
interpreter dispatch (replication, superinstructions, threaded code) addresses
by giving each opcode its own indirect branch.

Three escapes exist, none recommended here:

* An array of `fn` pointers ("closure/threaded code"). Safe and stable, gives
  one indirect call per op, but pays a real call frame per op and defeats the
  register allocation of the loop's locals.
* **[recall-med]** Explicit tail calls (`become`) — as far as I know still
  unstable. **Do not take my word for this**; check `rustc` rather than this
  document, because a recorded answer about the compiler goes stale.
* `#[inline(always)]` fan-out and manual replication of the `match` at several
  sites, which is the poor man's replication and costs code size.

**[read]** The project already measured that layout effects at this scale are
larger than the effects being chased: drive.rs:298–309 records a
range-invariant-argument struct that removed 6 instructions per range entry and
moved cycles from 8.63 to 9.12 billion, "no mechanism below 'the layout
changed' was found". That is the correct prior for any dispatch-shape change
here: **the measurement will be dominated by layout noise unless it is
interleaved and repeated** (and there is a memory note to that effect —
`benchmark-comparisons-must-interleave`).

---

## 4. Two-level loops in the wild

### 4.1 The L1/L2 split is completely ordinary

**[recall-high]** CRuby has exactly it. `vm_exec` is an outer function that sets
up an `EC_EXEC_TAG()` (setjmp) and calls `vm_exec_core`, the real dispatch loop;
when a Ruby-level exception unwinds to the tag, `vm_exec` finds the rescue entry
and **re-enters `vm_exec_core`** at the handler. The outer level exists to own
exception dispatch; the inner one owns instructions.

That is the same split as `run_chunk_clauses` / `run_ops`, and the drive.rs doc
gives the same reason **[read]** (drive.rs:182–189): the outer level owns
`offer_to_trap` because a trap offer is one per activation, and the inner level
must not make one because it is also entered for a construct's body.

**[recall-high]** CPython does it differently — `_PyEval_EvalFrameDefault` is
one loop with a `goto exception_unwind` label inside it, and inlined
Python-to-Python calls `goto start_frame` rather than recursing — but that is a
different way to spend the same structure, not an argument against it.

So: **nothing to answer for at L1/L2.** If the question is "is a two-level loop
unusual", the honest answer for this level is no, and CRuby is the citation.

### 4.2 The L3 branch-free region is the part I cannot find a precedent for

What L3 is, structurally, is a **superinstruction with a nested dispatch loop**.

**[recall-high]** Superinstructions are standard: a fused opcode covering a
common sequence, so that one dispatch does several ops' work. CPython ships them
— **[ran]** the 3.14 disassembly above contains
`LOAD_FAST_BORROW_LOAD_FAST_BORROW 1 (a, b)`, one instruction doing two loads.
Forth systems, Ertl & Gregg's work, and JVM interpreters all use the technique.

But a superinstruction **inlines** the fused sequence into one handler. The
whole point is to remove the dispatch. L3 keeps the dispatch — every region op
still goes through the same `match` — and gives up branching in exchange. That
is the opposite trade from the one superinstructions make, and I cannot name a
VM that makes it.

Candidates I considered and rejected as analogues:

* **Forth's inner/outer interpreter.** The "outer" interpreter is the
  text/compile loop, not an execution level. Not analogous; the name is a trap.
* **SQLite's VDBE.** One flat loop. A whole SQL statement is one program, so
  the statement level sits *above* the VM entirely.
* **Zend/PHP.** `execute_ex` with a handler chain; flat.
* **Emacs Lisp bytecode, Squeak, LuaJIT's interpreter.** All flat.

Two things that *are* genuinely two-level, and what they gained:

* **PL/pgSQL** **[recall-med-high]**. `exec_stmts` walks a `List` of statement
  nodes and `exec_stmt` switches on `cmd_type`, recursing for nested blocks.
  Per-statement work is `estate->err_stmt = stmt` (read by the error-context
  callback — the exact analogue of `record_failure_site` and `SIGL`) plus
  `CHECK_FOR_INTERRUPTS()`. Expressions are not in this engine at all: they are
  handed to the SQL executor. So the second level is a *different engine*, and
  nobody flattens it because the statement rate is negligible against the cost
  of running a SQL expression. **What it gained: nothing it had to defend.**
  The ratio of per-statement overhead to per-statement work is a thousand times
  more forgiving than a Rexx interpreter's.
* **Truffle/Graal** **[recall-med-high]**. The classic Truffle shape is an AST
  interpreter — every node has `execute()`, and the *host* call stack is the
  interpreter, so there is no dispatch loop at all until partial evaluation
  compiles the whole tree away. That is the extreme of "nesting is free, because
  the compiler removes it". Notably, Oracle later shipped a **Bytecode DSL**
  that generates a *flat* bytecode interpreter from the same node
  specifications, motivated by AST footprint and interpreter-mode startup — i.e.
  the direction of travel in that ecosystem is tree → flat, not the reverse.
  **[recall-med]** on the DSL's details; the existence and the motivation I am
  confident about.

* **Shells** **[recall-med-high]**. bash's `execute_command_internal` is a
  recursive tree-walker over the command tree; `line_number`, `$LINENO` and the
  `DEBUG` trap are per simple command. No bytecode, no loop, no problem — same
  reason as PL/pgSQL: the per-statement overhead is invisible next to a fork.

* **Other Rexxes.** **[recall-med]** Regina is an AST/list interpreter
  (`interpret()` in `interprt.c` walking node structs), so clause-level tracing
  is done where a clause node is entered, and expressions recurse — the same
  shape as the ooRexx oracle. **[recall-med]** NetRexx compiles to Java
  source/bytecode, so `trace` becomes emitted runtime calls at each statement:
  the boundary becomes an instruction at compile time and there is no
  interpreter loop to design. **I do not know Executor's internals** and will
  not guess; if it matters to the decision it should be read rather than
  recalled.

* **PL/SQL** **[recall-low]**. Compiled to DIANA/MCODE and interpreted; the loop
  shape is not public and I do not know it. Listed only so its absence is
  explicit.

### 4.3 The direct existence proof

The Rust design treats two things as in tension: *mandatory per-statement
bookkeeping* and *intra-statement branching in a flat op stream*.

**[ran]** Perl has both, in one flat threaded loop, and its per-statement
bookkeeping is item-for-item the same list as Rexx's — `PL_curcop` for the
line, `FREETMPS` for the temporaries frame, `PERL_ASYNC_CHECK` for the halt
check. `and(other->g)` and `cond_expr(other->a)` are branch ops that sit between
two `nextstate`s and do not cross either.

They are not in tension. The tension in this codebase comes from L3 being a
`for` over a slice instead of an indexed loop — an implementation detail, not a
consequence of Rexx's semantics.

---

## 5. What ooRexx itself does

This is the part I could verify, so it is cited line by line. Caveat first:
`/home/moritz/dev/repos/ooRexx` is a fork with local commits — `git log` shows
`0004ef91b Ask an activity to yield with a flag instead of writing into its
frame` touching `RexxActivation.cpp`, and `NumberStringClass.cpp` is
uncommitted-modified. The loop's *shape* below is upstream; the yield-flag
comment quoted at §5.1 is local work.

### 5.1 The instruction loop is flat, and its `pc` is a pointer

**[read]** `interpreter/execution/RexxActivation.cpp:468` is
`RexxActivation::run`, and its main loop is at 596–665:

```cpp
while (true) {
    try {
        RexxInstruction *nextInst = next;
        while (nextInst != OREF_NULL) {
            if (++instructionCount > yieldInstructions) { ...relinquish... }
            current = nextInst;
            next    = nextInst->nextInstruction;   // prefetch
            nextInst->execute(this, &stack);       // ONE clause

            stack.clear();                         // per-clause
            settings.timeStamp.valid = false;      // per-clause
            if (clauseBoundary) processClauseBoundary();   // per-clause, gated
            if (current->getType() != KEYWORD_EXPOSE) traceEntryAllowed = false;

            nextInst = next;
        }
        ...
```

Points that matter:

* **One level.** There is no nested dispatch loop anywhere in the interpreter.
  The outer `while (true)` is the exception/reply restart level — the same role
  CRuby's `vm_exec` plays — not a second dispatch level.
* **The `pc` is `next`, a `RexxInstruction*`** threaded by `nextInstruction`.
  Control instructions write it through `setNext`.
* **An instruction *is* a clause.** There is no sub-clause instruction
  granularity at all, which is why per-clause work can live in the loop body.
* **The periodic yield check is counted, not per-instruction** — the same design
  as CPython's `RESUME`/`JUMP_BACKWARD` eval-breaker points. The local comment
  says reading the flag in the loop condition "costs about 2% of dispatch",
  which is a useful calibration for how much a per-clause check is worth here.

### 5.2 Per-clause work is split between the loop and each `execute`

* **In the loop** (above): expression-stack reset, timestamp invalidation, the
  gated clause-boundary work, the `>I>` trace-entry decay.
* **At the top and bottom of each instruction's `execute`.** **[read]**
  `interpreter/instructions/IfInstruction.cpp:134–164`:

  ```cpp
  void RexxInstructionIf::execute(RexxActivation *context, ExpressionStack *stack)
  {
      context->traceInstruction(this);
      RexxObject *result = condition->evaluate(context, stack);
      context->traceResult(result);
      ...
      context->pauseInstruction();
  }
  ```

  **[read]** `RexxActivation.hpp:375` and `:380`:

  ```cpp
  inline void traceInstruction(RexxInstruction *v) { if (tracingAll()) traceClause(v, TRACE_PREFIX_CLAUSE); }
  inline void pauseInstruction()                   { if (pausingInstructions()) doDebugPause(); }
  ```

  and `:339`:

  ```cpp
  inline void traceIntermediate(RexxObject *v, TracePrefix p) { if (settings.intermediateTrace) traceValue(v, p); }
  ```

  So the oracle's trace cost model is **(a) throughout**: an inlined flag test
  at each trace point, duplicated into every instruction handler by the C++
  inliner. No patching, no second table, no second loop, no chunk-per-setting.
  This project's chunk-per-setting (§3(c)) is *better* than the oracle's, not a
  copy of it.

* **The expensive boundary work is behind one bool.** **[read]**
  `RexxActivation.cpp:4059–4110` (`processClauseBoundary`): queued `CALL ON`
  traps, halt-test exit, trace-test exit, halt condition, external trace
  on/off. **[read]** `:4112` recomputes the gate:
  `clauseBoundary = settings.haveClauseExits() || !(conditionQueue == OREF_NULL || conditionQueue->isEmpty());`
  A program with no exits and no pending condition pays one predictable branch
  per clause and nothing else.

### 5.3 Constructs are `pc` writes plus a side stack

* **`IF`** **[read]** IfInstruction.cpp:144–159: evaluates the condition, and on
  false does `context->setNext(else_location->nextInstruction)`. That is it —
  one `pc` write in a flat stream.
* **Loops** **[read]** `BaseDoInstruction.cpp:270–299`: `execute` builds a
  `DoBlock`, calls `context->newBlockInstruction(doblock)` to push it on the
  activation's own block stack, runs the first iteration test, and returns.
  **[read]** `EndInstruction.cpp:113–150`: `END`'s `execute` fetches
  `context->topBlockInstruction()` and calls the loop's `reExecute`, which
  **[read]** (BaseDoInstruction.cpp:311–330) does
  `context->setNext(nextInstruction)` — a **backward `pc` write** — re-traces the
  `DO` clause and re-tests. `SIGNAL` disabling active blocks is handled by
  `hasActiveBlockInstructions()`.

  That is exactly the `SelectFrame` design in drive.rs, and drive.rs already
  says so **[read]** (drive.rs:61–77: "This is the tree-walker's own Rust call
  frame, flattened").

### 5.4 Intra-clause control flow comes from the host language

The oracle never needs an intra-clause jump because it never linearises an
expression. **[read]** `ExpressionLogical.cpp` — the comma list is a term whose
`evaluate` is a C++ `for` loop with a `return` in it; **[read]**
`ExpressionOperator.cpp:179` — a binary operator is a term whose `evaluate`
calls its two children. The C++ call stack and the C++ `return` *are* the
intra-clause control flow.

### 5.5 Is the oracle's structure the reason the Rust IR came out this way?

**Half of it, and not the half that causes the problem.**

What was inherited, correctly and deliberately: the clause unit. **[read]**
drive.rs:22–34 says the compiled driver "discharges exactly the same per-clause
obligations, through the same functions, because those were extracted from that
loop rather than copied out of it", and §1.4's list maps one-to-one onto §5.1's
loop body. That inheritance is sound and the differential test suite depends on
it.

What was *not* inherited, and could not be: the oracle's intra-clause control
flow, which lives in the C++ call stack. The Rust **tree-walker** does inherit
it — `eval_logical_list` (eval.rs:1122) is the same `for`-with-a-`break` in
Rust, and it short-circuits fine. The Rust **IR engine** flattened the
expression evaluator into a linear op vector and thereby deleted the host-level
control flow, without putting a counter in its place.

So the two-level shape is not a copy of the oracle. It is an artifact of
half-flattening: the clause level was flattened into ops with a `pc` (L2, and
this is faithful to §5.3), and the expression level was flattened into ops
*without* a `pc` (L3, and this is where the host stack's branching was lost).

That framing is the finding, and it is grounded in source on both sides.

---

## 6. Verdict

### 6.1 Is the two-level loop unusual?

**The L1/L2 split: no.** CRuby's `vm_exec`/`vm_exec_core` is the same split for
the same reason (the outer level owns condition/exception dispatch). Keep it.

**The L3 branch-free region: yes, and I could not find a precedent.** Every VM
surveyed either keeps a `pc` over all of its ops (CPython, Ruby, Lua, V8, Perl,
JVM, the ooRexx oracle's own instruction pointer) or does not linearise the
sub-statement level at all (Truffle, PL/pgSQL, bash, Regina, the ooRexx oracle's
expressions). A linearised op sequence in which forward branches are structurally
inexpressible is, as far as I know, unique to this driver.

**Are the *reasons* unusual? No — the reasons are excellent and they are not
what causes the problem.** The per-clause bill (§1.4) is real, mandatory, and
correctly discharged; Perl pays the same bill for the same reasons. Making the
boundary an instruction is the right answer and puts this project in the same
group as Perl and the oracle. Compiling a chunk per trace setting is *better*
than the oracle's per-point flag tests and matches V8's and Perl's models. None
of that is under criticism.

### 6.2 Recommendation

**Do not flatten. Give the region a counter, in the smallest form that admits
forward jumps only.**

The per-clause work does not move. `enter_stepped_clause` … region …
`leave_stepped_clause` stays exactly as it is, and so does `RegionEnd::At` for
jumps that leave the clause. The only change is that the region can now skip
forward inside itself.

**Variant A — forward-only, minimal (recommended).** Drive the existing slice
iterator by hand instead of with `for`, and add one op:

```rust
let Some(ops) = chunk.ops_in(pc + 1, end) else { break 'region Err(...) };
let mut ops = ops.iter();
while let Some(region_op) = ops.next() {
    match region_op {
        // ... every existing arm, unchanged ...

        /// Skips `by` ops forward, staying inside this region.
        Op::SkipUnless { reg, by } => {
            match self.register_holds(registers, *reg) {
                Ok(true) => {}
                // `nth` on a slice iterator advances the pointer; it is O(1).
                Ok(false) if *by > 0 => { ops.nth(*by as usize - 1); }
                Ok(false) => {}
                Err(failure) => break 'region Err(failure),
            }
        }
    }
}
```

Why this shape and not an index:

* **`slice::Iter::nth` is O(1)** — it advances the start pointer rather than
  stepping. **[recall-high]**; verify with a disassembly rather than trusting
  this line. `Iterator::advance_by` is the clearer spelling **[recall-med]**
  that it is stable at the workspace's `rust-version = "1.96.1"` floor — check
  the compiler, do not trust this document. `nth` has been stable since 1.0 and
  is the safe choice.
* **The common path is byte-for-byte what it is today.** No bounds check is
  added to any op that is not a skip; `ops.next()` is the same pointer bump the
  `for` desugars to. That matters given the project's own measurement history
  (§3) where 4 instructions per pass was a reportable number.
* **Backward jumps are unrepresentable**, so the region cannot loop, so the
  clause cannot fail to terminate, so `leave_stepped_clause` still runs exactly
  once per `enter_stepped_clause`. That invariant is currently held by the
  `for`; Variant A holds it by the op's type instead of by the loop's shape,
  which is the stronger place to hold it (there is a memory note that only
  type-level fixes held for a defect that recurred three times).
* **Short-circuiting is always a forward branch.** Forward-only is not a
  compromise for the actual requirement.

Cost, concretely: one new `Op` variant, and one arm in each of the exhaustive
matches the task-1 brief already enumerates for adding an op — `ir/mod.rs`
(the variant and its doc), `ir/compile.rs` (`Root::of` and the emission),
`ir/drive.rs` (the region arm, plus an `op_not_driven` arm in L2's list),
`ir/golden.rs` (the render arm), and the pinned streams in `golden_tests.rs`.
**[read]** task-1-brief.md documents exactly this blast radius for `Op::Condition`,
so the cost is known rather than estimated. `size_of::<Op>()` stays 16 with a
`{ reg: u16, by: u16 }` payload.

Then `native_shape` gains an `ExprKind::Logical` arm that emits, per element:
the element's native ops, an `Op::Condition`-style validation (the comma list's
own raiser, 34.6 — **[read]** the brief's Step 1 is explicit that the comma
list's elements are already validated and re-checking reports 34.6 as 34.1), and
a `SkipUnless` to a shared "result is 0" tail. Careful with trace order: the
comma list's per-element `>>>` echo at
`clause_state.current_value_indent` (eval.rs:1130) has to be reproduced op for
op, which is the same discipline every other promotion in this phase has
followed.

**Variant B — a full region `pc`, if and when a guard needs one.** Replace the
iterator with `let mut i = 0; while let Some(op) = ops.get(i) { i += 1; ... }`
and a region-relative `Op::Branch { target: u16 }`. This admits backward and
shared targets (a slow-path block reached from several guards). It costs one
bounds check per region op that Variant A does not — **which must be measured,
interleaved, on a benchmark that actually has promoted clauses in its hot loop.**
**[read]** `bench-programs/emptyloop.rex` is `do i = 1 to n / nop / end`, and
drive.rs:407–431 records that it "has nothing for the answer to decide" because
its body holds no promoted clause — so it is the *wrong* program for this
measurement, and using it would produce a confident zero.

### 6.3 What a flat design would look like, and what it costs

Asked precisely, so answered precisely, even though I am recommending against it.

**The shape.** Delete `Op::Clause`'s region. Emit `Op::ClauseStart { index }`
and `Op::ClauseEnd { index }` as ordinary ops in the single stream, driven by
L2's existing `pc`. `ClauseStart` runs `grant_procedure_permission`, the `stale`
read, the `mem::take` and `enter_stepped_clause`, pushing the returned
`SteppedClause` token onto a driver-local stack. `ClauseEnd` pops it and runs
`leave_stepped_clause`. Jumps become ordinary `Op::Jump` with no special
meaning, and short-circuiting is free.

**What it costs, in descending order of seriousness:**

1. **The failure path breaks, and this is the real reason the current shape
   exists.** Today the region's `Result` reaches `leave_stepped_clause` **as a
   value** (drive.rs:1258 — `self.leave_stepped_clause(entry, code, index, clause, source, ran)?`),
   which is why every op's error path is `break 'region Err(...)` rather than
   `?`. **[read]** run.rs:4664–4672 records what that buys: both the clause's own
   failure site *and the boundary's* are recorded there, and a measured
   divergence is cited — "a failing handler queued by an `IF`'s own condition
   echoes `2 *-* if raiser() = 'V'`, and with the record left to the caller a
   promoted `IF` echoed nothing there while an unpromoted one echoed correctly".
   With `ClauseEnd` as an op, a `?` out of any op jumps clean over it: the temps
   frame is never popped, the failure site is never recorded, and a queued
   `CALL ON` handler is never delivered at the boundary. Fixing that means the
   driver's error path must unwind an explicit stack of open clauses — which is
   a `catch`-region per clause, which is the two-level structure again, only
   spelled with a `Vec` and a loop instead of a labelled block.
2. **`SteppedClause` moves from a register to a heap-backed stack.** Today
   `let entry = self.enter_stepped_clause(...)` is a local. Flat, it must live
   somewhere a `pc`-driven loop can find it: a `Vec<SteppedClause>` push and pop
   per clause, on the hottest path in the interpreter.
3. **The `GRANTING` const-generic split dies.** **[read]** drive.rs:310's doc:
   the const parameter "turns a per-clause branch into no code at all" and
   avoids "a `mem::take` per body clause that answers `false` every time". One
   flat stream is entered by both callers, so that becomes a runtime bit again.
4. **The one-fetch-per-region optimisation dies.** **[read]** drive.rs:377–391:
   the clause instruction is "fetched once and read by every index-bearing op
   inside the region rather than each resolving its own `index` against the
   body, which is a bounds-checked lookup of the same instruction per op". Flat,
   every op re-resolves.
5. **Two static checks lose their subject.** `compile::assert_region_ops_name_their_clause`
   and the per-op `debug_assert_names_the_clause` are region-shaped; the
   `Generic`-not-inside-a-region assertion (compile.rs:1243 onwards) becomes a
   different, weaker check about op ordering.

Item 1 alone disqualifies the flat design as an incremental change. It is not
unsolvable — see §6.4 — but it is a redesign of the failure path, not a
refactor of a loop.

### 6.4 The single strongest argument against my own recommendation

**Variant A preserves a structure whose real cost is a ceiling on future
optimisation, and the migration only gets more expensive from here.**

The comma list is a narrow prize (§1.3) and I said so. The thing that is not
narrow is *guards*. Every specialisation this engine might want next needs a
branch inside a clause with a shared fallback:

* an inline cache on a compound-variable read that bails to the general path;
* an integer fast path for `Op::Arith` that falls back when the operands are not
  small ints, or when `NUMERIC DIGITS` is not the value the op was compiled for;
* a `chunk.trace()` staleness check that jumps to a slow tail rather than
  running `stale` per clause;
* anything at all resembling deoptimisation.

Variant A's forward-only relative skip handles the trivial diamond and nothing
else. The moment two guards want to share one slow-path block, or a guard wants
to jump *past* the clause end into the ordinary stream, Variant A is not enough;
Variant B is, until a target needs to be computed or a slow path needs to be
out-of-line. The predictable trajectory is: forward skip, then a region `pc`,
then region-relative *and* absolute targets, then an escape hatch to reach ops
outside the region — at which point a flat stream has been re-derived with a
worse encoding, and the migration has been paid for two or three times instead
of once.

And the strongest part of the counter-argument is that §6.3's blocking
objection is *solvable*, and solvable exactly once: making the driver's error
path unwind an explicit clause stack is a bounded, well-understood piece of
work that every flat VM with per-statement cleanup has already done (Perl's
`FREETMPS`-on-unwind through `LEAVE`/`cxstack`; CPython's `exception_unwind`
label popping the block stack). The current design's `Result`-as-a-value trick
is elegant *because* the region is a lexical block — it is a benefit of the
shape, not an independent reason for it, and quoting it as the reason is
circular.

If the roadmap includes guarded specialisation, flattening now is cheaper than
flattening later, and this recommendation is the wrong call. The counter to the
counter is only that the roadmap has not committed to that, and a structure
should not be rewritten for optimisations that have not been scheduled — but
that is a judgement about the plan, not about the VM design, and it should be
made by whoever owns the plan rather than inferred from this document.

---

## 7. Gaps in this analysis, stated rather than papered over

* Ruby, Lua, V8 and HotSpot claims are all recall. None of those sources is on
  this machine and there is no network access. The *shapes* I would defend; the
  function names, version numbers and file paths I would not.
* I do not know Executor's or PL/SQL's interpreter structure and did not guess.
* Regina and NetRexx are recall-medium and were not read.
* The claim that `slice::Iter::nth` is O(1), and the stability of
  `Iterator::advance_by` on the 1.96.1 floor, are both recall. Both are one
  command away from being facts; ask the compiler, not this file.
* No measurement was taken. Every performance statement here is either quoted
  from a measurement already recorded in the repository (with its line cited) or
  is a prediction labelled as such. In particular, the claim that Variant A adds
  nothing to the common path is a claim about codegen and has not been checked
  against a disassembly.
