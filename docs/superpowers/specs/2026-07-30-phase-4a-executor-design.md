# Phase 4a executor design

**Goal:** run a classic Rexx program that has no procedures, no `PARSE` and no builtin calls, byte-for-byte as the oracle runs it, and fix the execution model while doing it.

**Status:** design approved 2026-07-30.
This document is the spec for Phase 4a alone.
Phases 4b and 4c get their own spec and plan, and this document defines their boundaries so that nothing falls between them.

## Scope of this document

Phase 4 in the parent plan (`docs/superpowers/plans/2026-07-27-rust-rewrite.md`, section 2) is one row: "Non-OO Rexx runs: assignment, `DO` (all variants), `IF`, `SELECT`, `CALL`, `PARSE`, `SAY`, `SIGNAL`, conditions, and all 81 builtin functions."
That is larger than Phases 0 to 3 combined, so it is split into three sub-phases.
Each produces software that runs, and each closes against its own named corpus.

This document specifies 4a in full, states the 4b and 4c boundaries, and records the four Phase 4 decisions taken now because they constrain the dispatch loop's design.

## What phases 1 to 3 hand over

Exact interfaces, verified against the tree at commit `9f68662a`.

From `rexx-core` (628 lines of `src`):

* `ObjRef`, a tagged 64-bit handle: `Decoded::Heap { slot: u32, generation: u32 }`, `Decoded::SmallInt(i64)` in `SMALL_INT_MIN..=SMALL_INT_MAX` (plus or minus 2^61), `Decoded::Nil`.
* `Heap` with `alloc(Body) -> ObjRef`, `alloc_with(BehaviourId, Body)`, `get(ObjRef) -> Option<&Object>`, `get_mut(ObjRef) -> Option<&mut Object>`, `collect(&RootSet) -> CollectStats`, `live_count()`.
* `Body`, currently `String(String) | Array(Vec<ObjRef>) | Instance(Vec<(String, ObjRef)>) | WeakRef(ObjRef)`, with an exhaustive `trace` and no wildcard arm.
* `RootSet` with `add_global`, `push_frame() -> FrameId`, `pop_frame(FrameId)`, `push_temp(ObjRef)`, `iter()`.
* `BehaviourTable` with `define`, `set_superclass`, `lookup(BehaviourId, &str) -> Option<MethodId>`, and `BehaviourId::{STRING, ARRAY, OBJECT}`.

From `rexx-num`:

* `Number::parse(&str) -> Option<Number>`, `format(u64) -> String`, `round_to`, `whole_value(usize) -> Option<i64>`, `is_zero`, `zero`, `one`.
* `add`, `sub`, `mul`, `div(.., DivOp)`, `pow`, each `(&self, &Number, digits: u64) -> Result<Number, ArithError>`.
* `compare(.., CompareOp)`.
* `Settings` with `digits()`, `fuzz()`, `form()`, `set_digits_str`, `set_fuzz_str`, `set_form_str`.
* `ArithError` and `FormatError`, each with `code()`, `additional()`, `message()`.

From `rexx-parse`:

* `parse_program(Vec<u8>) -> Result<Program, ParseError>` and `parse_interpret(Vec<u8>) -> Result<Fragment, ParseError>`.
* `Program { source: ProgramSource, instructions: Vec<Instruction>, directives: Vec<Directive>, labels: BTreeMap<Box<[u8]>, usize>, symbols: SymbolTable }`.
* `CodeBody { instructions, labels }`, one per directive body.
* Instruction lists are flat and source-ordered, and every jump target is a `usize` index into the same body's `instructions`.
* Every node carries a byte range into the retained source.
* `compound_parts(&str) -> (&str, Vec<Tail>)`, where `Tail::Constant` is a piece that stands for itself and `Tail::Variable` is a piece whose value supplies it.

Three handovers from Phase 3 bind this phase and are addressed below: `resolveCalls` (`LanguageParser.cpp:1690`) is deliberately not ported, `TRACE`'s value lines are Phase 4's, and nothing in Phase 3 observes AST shape.

## The split

| | Deliverable | Closes against |
|---|---|---|
| 4a | Value model, variables, expression evaluation, control flow. `Assignment`, `Say`, `Nop`, `Do`/`Loop` in every variant, `If`/`Then`/`Else`, `Select` including `SELECT CASE`, `When`, `WhenCase`, `Otherwise`, `Leave`, `Iterate`, `End`, `Drop`, `Numeric`, `Trace`, `Exit`. | A named L0 subset plus the `base/expressions` L1 groups |
| 4b | `Call`, `Return`, `Procedure`, `Use`, `Signal` in all three forms, condition traps, `Raise`, `Interpret`, `Push`, `Queue`, and the in-process queue. Routine resolution, which is handover 1. | The `base/keyword` L1 groups |
| 4c | `Parse`, `Arg`, `Pull`, and the 66 in-scope builtins, ticked off one at a time. | The `base/bif` L1 groups, 66 rows |

The parent plan's Phase 4 row closes when 4c closes.
`Message`, `Command`, `Address`, `Guard`, `Reply`, `Forward`, `Options` and every directive are outside all three: see "Out of scope" below.

## Decisions

These are new decision blocks for section 1 of the parent plan, numbered after D14.

### D15 value representation

**Decided: mirror the oracle's dual representation.**

Rexx makes the difference between a value that came from text and a value that came from arithmetic observable.
Measured with `build/bin/rexx`:

```
x = '007'   ;  say x      -> 007
            ;  say x + 0  ->   7
```

A single canonical representation cannot produce both without carrying the original spelling anyway, so the representation carries it:

* `Body::Text { bytes: Vec<u8>, num: Option<Box<Number>> }` for a value whose identity is its bytes.
  The `num` cache fills on first arithmetic use, through `Heap::get_mut`.
* `Body::Num { value: Number, text: Option<Vec<u8>> }` for a value whose identity is its number.
  The `text` cache fills on first string use, formatted under the `NUMERIC` settings in force *at formatting time*, which is why it is a cache of a rendering and not a second source of truth.
* `ObjRef::SmallInt` for a whole result that fits, using the tagging Phase 1 already built.
  Formatting still goes through `Number`'s rules, because `numeric digits 3` makes `1234 + 0` print `1.23E+3`.

`Body::String(String)` is deleted.
It holds a UTF-8 Rust `String`, and D14 closed on byte strings: `reverse('ää')` yields invalid UTF-8 in the oracle, so a value that cannot hold arbitrary bytes cannot hold a Rexx string.
Phase 1 built it before D14 closed.

**Cost of being wrong:** contained.
The variants are private to `rexx-exec`'s value module behind constructor and accessor functions; a later phase can add a third representation without touching the instruction loop.

### D16 variable pool and slot assignment

**Decided: a per-`CodeBody` resolution pass, computed on first execution and cached on the body.**

Variable lookup is 8.1% of runtime on the realistic mixed benchmark and 32.2% on stem-heavy code (perf profile, 2026-07-25).
The oracle's answer is integer slots assigned at parse time (`RexxLocalVariables`, 602 lines).
Phase 3's AST carries `SymbolId` and no slots, deliberately, so the assignment moves to first execution:

* Walk the body once, collect every referenced `SymbolId`, assign dense slot indices.
* Decompose every `ExprKind::Compound` through `compound_parts` and record a slot index for each `Tail::Variable` piece.
  This is not only speed: `compound_parts` returns text, and the tail pieces were never interned, so *something* has to map them to variables and the plan is the only place that sees the body as a whole.
* Record the label table (Phase 3 already built it) and, in 4b, the resolved call targets, which is where handover 1 is discharged.

An activation is then `Vec<Option<ObjRef>>` of exactly the plan's length.

The rejected alternatives: a `HashMap<SymbolId, ObjRef>` per activation hashes on the measured hot path and allocates a map per call; a `Vec` indexed by `SymbolId` directly makes a small routine in a large file pay for every symbol in the file.

### D17 trace granularity

**Decided: the dispatch loop emits a trace event per evaluation step from the start, and 4a formats the value lines.**

`RexxActivation.hpp:90`-`110` enumerates 19 prefixes, `TRACE_PREFIX_CLAUSE` at `:92` through `TRACE_PREFIX_INVOCATION_EXIT` at `:110`.
Everything except the clause prefix carries an evaluated value.
Phase 3 ships the clause prefix, the `*-*` source line, and nothing else.

Emitting an event per evaluation step forbids constant folding and expression fusion.
That is accepted, on two grounds: the profile puts dispatch at 38.9% of realistic runtime and allocation at 26%, so folding was never where the time is, and the alternative is designing the dispatch loop twice.
`TRACE.testGroup` holds 239 expected trace-output lines, which is what the decision buys.

The value lines need each subexpression's *source text*, which is affordable only because Phase 3 retained spans on every node.
The C++ sites are `traceValue` (`RexxActivation.cpp:3728`) and `traceOperatorValue` (`:3852`).

### D18 command dispatch is not Phase 4's

**Decided: the `ADDRESS` instruction tracks the environment name so that the `ADDRESS()` builtin reports it, and actual command dispatch, `RC` setting, and the `ERROR` and `FAILURE` conditions land in Phase 7 with the platform layer.**

A command clause raises the not-implemented failure described under "Failing loudly" until then.
Phase 7 owns it because dispatch needs the platform layer, and `ADDRESS ... WITH` redirection needs the stream model that Phase 7 builds.

## Architecture

### The borrow shape, and the spike that proves it

The parent plan (line 2472) records this as the phase's one unsolved question: who owns `Heap` versus `RootSet` during evaluation.

The answer is that `Interp` owns `Heap`, `RootSet`, the activation stack, the settings, the trace sink and the output sink, and **does not own the AST**.
Programs are held as `Rc<Program>`; an activation holds its own `Rc` clone.
Evaluation is therefore `fn eval(&mut self, body: &CodeBody, expr: &Expr) -> Result<ObjRef, Raised>`, where the `&Expr` borrow derives from an `Rc` the caller cloned out first, so it does not conflict with `&mut self`.

If the AST lived inside `Interp`, every evaluation step would need `&self.program` and `&mut self` at once and nothing would compile.

**Task 1 of the plan is a spike that proves this shape end to end**, including the case that motivates the `Rc`: an `INTERPRET` fragment is parsed at run time, its instructions execute inside the activation that made it, and its `Rc<Fragment>` must outlive the instruction.
The spike is written and kept, not written and thrown away: it is the first thing a later phase reads when it wants to know why the shape is what it is.

### Crate layout

One new crate, `rexx-exec`, depending on `rexx-core`, `rexx-num`, `rexx-parse`.

```
rexx-exec/
  src/value.rs        the value model, conversions, string and number identity
  src/stem.rs         stems and compound tail resolution
  src/plan.rs         the per-CodeBody resolution pass (D16)
  src/activation.rs   one frame: slots, block stack, pc, settings
  src/eval.rs         expression evaluation and the operators
  src/run.rs          the instruction loop, control flow, DO block state
  src/trace.rs        trace events and prefix formatting (D17)
  src/error.rs        Raised and its condition payload
  src/lib.rs          Interp, and the public entry point
  src/bin/rexx-run.rs the runner the differential tests drive
```

One file per concept, as elsewhere in the workspace.
`run.rs` is the one at risk of growing past what fits in context; if it passes roughly 800 lines, the loop splits from the per-instruction handlers rather than growing further.

## The value model

### Text and number identity

Conversions are total functions with explicit failure:

* Text to number: `std::str::from_utf8` then `Number::parse`.
  A byte string that is not valid UTF-8 cannot be a Rexx number, since a number's characters are ASCII by definition, so the two failures collapse into one and neither is a panic.
* Number to text: `Number::format(digits)` under the settings in force at that moment.
* `SmallInt` to text: through `Number`, not through `i64::to_string`, because `NUMERIC DIGITS` can force exponential form.

Every value is an `ObjRef`.
The `RootSet` temps stack is pushed before any allocation that could collect while an intermediate is live, which is the discipline `RootSet` was built for.

### Stems and compound variables

`Body::Stem { default: Option<ObjRef>, tails: HashMap<Vec<u8>, ObjRef> }`, with a new `BehaviourId::STEM`.

A hash map, where the oracle has a balanced BST whose `memcmp` alone is 21.6% of stem-heavy runtime and is called only from `CompoundVariableTable::findEntry` (545 lines).
This is the one place where the rewrite is expected to beat the oracle rather than match it.

The semantics are measured, not assumed.
Probed with `build/bin/rexx` on 2026-07-30:

```
a. = 'd'  ;  a.1 = 'one'  ;  say a.1 a.2   -> one d
a. = 'reset'             ;  say a.1 a.2   -> reset reset
drop a.                  ;  say a.1       -> A.1
b.3 = 'x'  ;  drop b.    ;  say b.3       -> B.3
c.1 = 'keep'; drop c.1   ;  say c.1       -> C.1
say novar                                 -> NOVAR
```

So: assigning to the stem replaces the whole collection and sets the default for every tail.
`DROP` of the stem clears the map and the default together, returning tails to uninitialised.
An uninitialised compound yields its derived name, which is the upcased stem plus the resolved tail values.
An uninitialised simple variable yields its own upcased spelling.

Tail resolution follows Phase 3's measurement: with `b = 2` and `c = 1`, `a.b.c` names `A.2.1`, so a `Tail::Variable` piece contributes its variable's *value* and the stem contributes its own name.

## Expression evaluation

Recursive over `Expr`.
The tree already carries precedence and associativity, so evaluation never reconsiders either.

* `Binary`: arithmetic through `rexx-num` under the current settings; comparison in two families, the numeric or string comparison operators and the strict byte comparisons `==` and `\==`; `Abuttal`, `Blank` and `||` concatenate bytes.
* `Prefix`: `+`, `-`, `\`.
* `Logical`: an AND of its parts, and a part that is not `0` or `1` raises 34.x at run time, which is where a plain `WHEN a, b` differs from `SELECT CASE`.
* `Literal`, `Constant`: the bytes the parser decoded, which for a `Constant` is the upcased spelling, so `say 1e5` prints `1E5`.
* `Variable`, `Stem`, `Compound`: through the plan's slots.
* `DotVariable`: `.nil`, `.true` and `.false` only in 4a; every other environment symbol is Phase 5's and fails loudly.
* `Call`, `QualifiedCall`, `Message`, `ClassResolver`, `List`, `VariableReference`: not 4a's, and each fails loudly.

## Control flow

```rust
enum Flow { Next, Goto(usize), Exit(Option<ObjRef>) }
```

A program counter walks the body's `instructions`; each step answers `Flow` or raises.
Loop state is a per-activation `Vec<Block>` holding the control variable's slot, the `to`, `by` and `for` values, the iteration counter, the block's label and its `end` index.
`LEAVE` and `ITERATE` unwind that stack to the matching label and jump.
4b adds `Return` and `Signal` variants.

Evaluation order inside a controlled loop is the order the keywords were written in, which Phase 3 recorded in `Controlled::order` precisely because an expression can have side effects.

**This is where handover 4 lands.**
Nothing in Phase 3 observes AST shape, so a control-flow target wired to the wrong index passes every parser test.
4a's corpus therefore carries the shapes that expose it: nested `DO` with `LEAVE` naming an outer label, `ITERATE` from inside a `SELECT` within a loop, `IF`/`ELSE` chains where the false target and the then-exit differ, and a `SELECT` whose `WHEN` branches all fall to the same exit.

## Conditions and errors in 4a

`Result<T, Raised>`, where `Raised` carries the condition name, the error number and sub-number, and the substitution values.

4a's raisers are arithmetic (`ArithError` already supplies number and additional values), a `SELECT` that reaches its `END` with no `WHEN` taken (7.3), the logical-value check (34.x), and the `DO` control conversions.
No trapping: `SIGNAL ON` is 4b's, so in 4a a raise terminates the program with the oracle's message on stderr and the oracle's exit code.

### Failing loudly

Every feature that 4a does not implement fails in a way that cannot be mistaken for parity: a distinct process exit code and a message naming the construct and the sub-phase that owns it, never a plausible Rexx condition.

The reason is specific.
If an unimplemented builtin raised 43.1, "could not find routine", a differential run against the oracle would show a divergence that reads like a resolution bug, and a program that happened to expect 43.1 would *pass*.
An implementation gap must never be able to produce a passing test.

## Trace

`Interp` holds the current setting and a sink.
`eval` calls the sink per evaluation step; the sink drops the event unless the setting selects it.
4a formats the prefixes its own expressions and instructions produce; the invocation prefixes belong to 4b, which is what introduces invocations.

## Output

`SAY` writes to a sink on `Interp`, defaulting to stdout, with no line-length handling beyond the oracle's.
This is not the Phase 7 stream model and does not pretend to be: `.output` as an object, redirection, and the stream classes are Phase 7's.
The sink exists so that a test can capture output without a subprocess.

## Testing

### L0 differential corpus

`rust/corpus/` programs run under `rexx-run` and under `build/bin/rexx`, compared through `rexx-oracle`'s `normalize` and `diff`.

4a implements no builtins, so it cannot run every existing corpus program.
Its gate therefore quantifies over a **named subset, listed in `rust/corpus/phase-4a.txt`**, containing the programs that use only 4a features, plus new programs written for this phase.
4b and 4c each add their own list file rather than editing this one, so that what a sub-phase unblocked stays visible.
The count is *reported* by the harness, not asserted in the criterion, because counts rot: Phase 3 had one criterion whose hard-coded number moved twice.

Two rules carry over from `corpus/README.md` and are not relaxed: a corpus program must be deterministic, so no `TIME()` and no `DATE()`, and when the self-test with the same binary on both sides reports a divergence the corpus is at fault, never `normalize`.

The comparison runs as a `cargo test`, with the oracle's expected output committed the way `tests/sourceline_oracle/` does it, so that `cargo test` alone is the gate and a script regenerates the expectations.
A criterion enforced only by a script nobody runs is not enforced.

### L1 extracted assertions

`rexx-extract` turns `.testGroup` methods into standalone programs; `ootest/` holds 409 groups and 12,176 extractable assertions.
4a's target is `ootest/ooRexx/base/expressions` (11 groups).
The extracted programs that need 4b or 4c features are listed, with the sub-phase that unblocks each, rather than silently skipped.

### What the tests cannot see

Stated so that the gate is not read as stronger than it is:

* An `Expr` evaluated in the wrong order with the same result is invisible unless a side effect exposes it, and 4a has no side effects inside an expression except `TRACE` output.
  Trace-output tests are therefore the only observation of intra-expression evaluation order, which is a second reason D17 lands in 4a.
* The value model's cache behaviour is invisible to the corpus: a stale `num` cache and a correct one differ only in speed, unless the cache is stale *across* a `NUMERIC` change, which is exactly the case the unit tests must construct deliberately.
* Nothing here measures GC correctness under pressure.
  A missing `push_temp` shows up as a collected live value only when a collection happens at the right moment, so 4a runs its corpus a second time with a stress mode that collects on every allocation.

## 4a exit gate

Each criterion names the set it quantifies over, and each can fail.

1. The named L0 subset runs with zero divergences, and the harness reports the program count.
   The negative control holds: substituting any other binary for `rexx-run` reports divergences on every program, so a zero is meaningful.
2. Every extracted `base/expressions` assertion that needs only 4a features passes, and the ones that do not are listed with the sub-phase that unblocks them.
3. `trace r` and `trace i` output for a committed set of programs matches the oracle byte for byte, including the value lines.
4. The corpus passes again under collect-on-every-allocation.
5. Zero `unsafe`, `clippy -D warnings` clean, `cargo fmt` clean.
6. The borrow-shape spike is committed with its findings written down.

Criterion 3 is the one that would be easiest to write vacuously.
"Trace output matches" over a program set with no value lines in it would pass while observing nothing, so the committed set must contain at least one program per prefix that 4a can emit, and the criterion says so by naming the prefixes rather than counting the lines.

## Phase 4 gate items decided now

### The exclusions file

`docs/superpowers/plans/phase-4-exclusions.txt`, one row per excluded builtin: the name, what is excluded, the phase that delivers it, and the failure it produces meanwhile.
Phase 7 and Phase 10 delete their own rows.

Fifteen of the 81 entries in `builtinTable[]` (`BuiltinFunctions.cpp:3042`) are excluded, leaving 66 in scope for 4c:

* Phase 7, streams and platform: `CHARIN`, `CHAROUT`, `CHARS`, `LINEIN`, `LINEOUT`, `LINES`, `STREAM`, `QUALIFY`, `USERID`, `SETLOCAL`, `ENDLOCAL`.
* Phase 10, RXAPI: `RXQUEUE`, `RXFUNCADD`, `RXFUNCDROP`, `RXFUNCQUERY`.

Two partial rows, because a whole-builtin exclusion would overstate the gap:

* `VALUE`'s external-selector form, which reads a pool such as `ENVIRONMENT`, is Phase 7's; the variable-access form is 4c's.
* `QUEUED` is in scope, because 4b builds the in-process queue; only the external named queues that `RXQUEUE` reaches are Phase 10's.

Phase 4's gate then reads "66 of 81, and the excluded set is exactly this file", which is falsifiable in both directions.
Without the file, "all 81" is a criterion the phase ordering cannot satisfy, which is how Phase 2 came to fail three of five.

### The rexxcps gate

`samples/rexxcps.rex` is the end-of-4c gate.
It measures clauses per second over a mix reconstructed from an analysis of 2.5 million lines of trace output, and it deliberately issues no commands, using an `RC=expression` and `PARSE` sequence instead, so it does not need the dispatch that D18 excludes.
Its dependencies are `parse var`, `parse version`, `parse value`, `parse upper`, `parse source`, `trace value`, `trace off`, `time('R')`, one internal `call subroutine`, and `signal on novalue`: all inside Phase 4, nothing from Phase 5 or later.

Measured under the oracle on this machine on 2026-07-30: 16,608,454 clauses per second, 1.83 s wall, exit 0.
"Clauses" is the program's own nominal count rather than a measured tally, so the figure is meaningful only as a ratio between two interpreters running the identical program.

Four criteria:

1. **Correctness before speed.** The 1000-clause mix carries `say 'Failed<n>'` guards throughout. The run completes, exits 0, and prints no `Failed` line.
2. **The cps ratio**, both interpreters, same machine, same session. Above 1.5x fails the gate; between parity and 1.5x is recorded as debt, which is the shape Phase 1 used, because the alternative is a sound design stalling a phase over 10%.
3. **An external cross-check.** rexxcps times itself with `TIME('R')`, our own builtin, so a defect there flatters the number and the benchmark cannot detect it. Wall-clock both runs externally and require the two ratios to agree within 10%.
4. **The baseline is measured at gate time**, not reused. `perf-baseline.md` has no rexxcps row and its figures come from a different day; cps is machine and load specific.

rexxcps is not a corpus program.
Its output carries timings, and the corpus rule is determinism, so it gets its own comparison rather than a loosened `normalize`.
It adds to the seven `bench-programs/` dimensions and replaces none: one mixed number can be met while `compound.rex` regresses.

### The two carried debts

The parent plan requires both to be re-measured at Phase 4, on equal footing, which neither earlier phase could do:

* D1's GC pause, 1.45x (26.5 ms against 18.2 ms), inside Phase 1's viability threshold but outside the parity gate that applies from Phase 2 on.
* Phase 2's arithmetic, 1.22x, and that figure is a lower bound: it timed Rust arithmetic alone against a C++ number that included parse and dispatch.

Both re-measurements belong to 4c, since both need a whole program to run.

## Out of scope for 4a

Named with the owner, so that nothing is merely absent:

* `Call`, `Return`, `Procedure`, `Use`, `Signal`, `Raise`, `Interpret`, `Push`, `Queue`, condition trapping: 4b.
* `Parse`, `Arg`, `Pull`, the 66 builtins: 4c.
* `Message` sends, `Guard`, `Reply`, `Forward`, every directive, environment symbols beyond `.nil`, `.true` and `.false`, and the 32 classes: Phase 5.
* `Command` dispatch, `Address` beyond tracking the environment name, the stream model: Phase 7 (D18).
* Concurrency, `REPLY`'s threading semantics: Phase 6.

## Risks

* **The `Rc<Program>` shape may not survive 4b.** A routine call from body A into body B, with both alive, is the case 4a does not exercise. The spike covers `INTERPRET` because that is the awkward case available now; if 4b finds the shape insufficient, the fix is an arena of packages rather than per-program `Rc`, and the change is confined to how bodies are reached.
* **The value model's two caches can desync from the settings.** A `text` cache formatted under `DIGITS 9` is wrong after `NUMERIC DIGITS 3`. The rule is that `Body::Num`'s text cache is invalidated on any settings change, and the test for it must construct that sequence deliberately, since no corpus program will stumble into it.
* **Trace output is the widest surface in 4a and the least specified.** 239 expected lines exist for the whole of `TRACE`, but the oracle's exact spacing and the interaction between value lines and the clause line are measured, not documented. Budget for the oracle being the only specification.
* **4a cannot run most of the existing corpus**, because it has no builtins. That is why its gate names a subset, and the risk is that the subset is chosen to be easy. The mitigation is that the subset is committed as a file and 4b and 4c grow it, so a program left out has to be left out in writing.
