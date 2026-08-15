# Task 7: promote `Assignment` and `Say` -- report

BASE `caf16c90`.
Commits, in order: `f330a96a` (the cases, before anything was promoted), `439e5d6c` (the promotion),
`3bcbcc53` (the register-release pin and two comment corrections the mutations forced), `f55ea409`
(the `inline` the measurement earned).

Suite **1382 -> 1385**, 0 failed, 4 ignored, green in dev, dev+STRICT, release and release+STRICT,
each exit status read unpiped.
`cargo fmt --all --check` 0.
`cargo clippy --workspace --all-targets -- -D warnings` 0, re-run from an empty target directory.

## The op set

Four ops, and the split between them is the whole point of the task.

| op | what it does | why it is its own op |
|---|---|---|
| `Const { dst, konst }` | loads constant `konst` of `Chunk::consts` into register `dst`, through `Interp::literal` | the phase's first op that produces a value without entering `eval.rs` |
| `TraceLiteral { src }` | echoes the `>L>` line of the literal in register `src` | `eval.rs` emits that line as a **side effect** of evaluating a literal; a load emits nothing |
| `Store { index, src }` | writes register `src` through the `Assignment` at `index` | enters `Interp::assign_evaluated`, which is what `step`'s own arm now calls |
| `Say { index, src }` | prints register `src`, or a blank line when `src` is `None` | enters `Interp::say_evaluated`, same reason |

A promoted `Assignment` is `Clause`, then optionally `TraceClause`, then either `Const` +
`TraceLiteral` or `EvalExpr`, then `Store`. A promoted `SAY` is the same with `Say` in place of
`Store`, and with no value ops at all for the bare form.

**`Const` covers `ExprKind::Literal` and nothing else, and the reason is where the bytes live rather
than what the expression is.** A `Constant` -- a symbol whose value is its own upcased spelling,
which is what `3` and `1e5` parse to -- keeps its bytes in the symbol table, and `compile` takes only
the body, the plan and the trace setting. Widening it to take the symbol table would widen what
`Interp::chunk_for`'s key has to name, which that function's own doc comment turns into the
completeness argument for the cache. A `Constant` therefore reaches `Op::EvalExpr`, which traces it
identically because it is the same `eval` call.

**`TraceLiteral` is emitted unconditionally with a run-time gate, not compiled in the way
`Op::TraceClause` is.** `ChunkTrace` carries `clauses` and `labels`; the gate this line answers to is
`intermediates`. Widening `ChunkTrace` would put a second emission decision under a staleness rule of
its own, which is exactly the reasoning `Op::TraceKeyword` already carries for the `>K>` line. What
the op form buys here is that the line exists at all, not its elision.

**`plan` is still not read, and the promise that this task would read it was in `compile`'s own doc
comment -- not in the plan.** (An earlier version of this report attributed it to the plan; the plan
says only that `Store` reaches `assign_expr_target`.) The one thing a compiled assignment could want
from `plan` is the slot its target resolves to, and `Op::Store` deliberately does not resolve targets.
Resolving the name in the compiler would be the second implementation the phase forbids. That doc
comment is corrected to say so rather than to promise a reader it will happen.

## Interning, and how it is proved

`Chunk::consts` is `Vec<Box<[u8]>>`, one entry per **distinct** literal. The compile-time side is
`Constants`, a `Vec` plus a `HashMap<&'a [u8], u32>` keyed on the body's own literal bytes -- borrowed
rather than copied, so a repeated literal costs a hash and a comparison and no allocation, where a
`HashMap<Box<[u8]>, u32>` would allocate once per distinct literal for the key as well as the entry.

Two tests, and each answers a degenerate table the other does not:

* `one_literal_written_twice_is_one_interned_constant` compiles `say 'dup'` / `say 'dup'` / `say 'x'`
  and asserts both the rendered stream and the table. The stream's two `konst=0` fields answer a table
  that dedupes entries and hands the second occurrence an index nothing filled; the table's contents
  answer one that never looks a literal up; and `'x'` taking entry 1 stops both being satisfied by a
  one-entry table.
* `every_instruction_of_an_all_generic_body_compiles_to_one_generic_op` asserts the table is **empty**
  for a body with no promoted instruction in it, which is what says an entry comes from an emitted
  `Const` rather than from every literal a body happens to contain.

Mutations M7 and M8 below are the two directions of getting it wrong.

**Interning is invisible to a running program**, and that is asserted as output, not argued:
`Op::Const` builds a fresh value from the bytes on every pass, so two occurrences share an entry and
share no value. The case row "one literal written twice traces twice" requires two `>L>` lines, one
per occurrence.

## Every test, with its mutation evidence

The mutation table was taken **once, after the last test landed**, with
`cargo test --workspace --no-fail-fast`; each row restores from a `cp` backup, verifies the restore by
sha256 and rebuilds. `run-with-the-case-held-out` means the new case file was moved **out of
`tests/ir_dual_cases/`** -- renaming it in place is not a hold-out at all, because `datadriven::walk`
walks every file in the directory regardless of name, and the first version of the harness did exactly
that and produced four invalid hold-out rows before the error was caught.

### New tests

**`tests/ir_dual_cases/assignment-and-say`, 27 rows, all oracle-measured.** Landed in `f330a96a`,
before the compiler emitted anything, and measured green on both engines there -- which is what says a
row can tell whether it would ever have failed. Shapes: assignment to a simple symbol, a whole stem, a
computed compound tail; `SAY` of a literal, of nothing, of an expression; an unset read. Trace: the
`>L>` line under `trace i` for a `SAY` and for an assignment, the `>C>`/`>=>` pair for stem and
compound targets, a nested arithmetic value, a compound read, a `SAY` inside a matched `WHEN`'s branch
at its own indent, one literal written twice, and **the negative direction** -- the same two
instructions under `trace r`, where `>L>` must not appear. Boundaries: a handler queued by an
assignment's value and delivered at that assignment's own boundary; a handler that **queues again**
there, whose second delivery lands at the next clause's boundary with `SIGL` 4 to say so; a handler
that fails at a `SAY`'s own boundary; one delivered per pass inside a loop; one queued inside a called
label; `PROCEDURE` behind an assignment; `SIGL` at a call inside a value expression; both instructions
failing; a `NOVALUE` trap; both inside an `INTERPRET` fragment; an assignment on an `IF`'s false path.

The file was generated from the oracle rather than transcribed, because two trailing-space cases (a
clause echo that ends at a keyword, and a bare `SAY`'s blank output line) are compared verbatim and
were lost when written by hand.

**Adds coverage: yes, for one mutation and only one.** M15 -- dropping
`grant_procedure_permission` in front of a promoted clause region -- is caught by this file **alone**:
with it held out the whole workspace is green at 1385/0, and with it in place `both_engines_agree_on_
every_case_file` fails on the "procedure after an assignment in a called label" row, the IR arm
printing nothing where the tree-walker raises 17.1.

**Adds coverage: no, for M1, M2, M5 and M6.** Each is caught with the file held out, by
`both_engines_agree_across_every_population` and by rows of `trace-settings` /
`loop-header-boundaries`, with identical pass counts either way. **The brief's premise is wrong on one
point and it is worth correcting where the next reader will see it:** a dropped `>L>` line is *not*
invisible to every corpus instrument here. `tests/support/mod.rs`'s normalisation applies to the
oracle differential; `ir_dual.rs`'s population sweep compares the two engines' stderr byte for byte
and catches M1 outright. What the new file adds over that is the oracle-measured expected bytes -- the
sweep says the two engines agree and says nothing about whether either is right -- and the M15 row.

**`ir::golden_tests::an_assignment_of_a_literal_compiles_to_a_constant_load_and_a_store`.** Written
before the compiler emitted anything and run failing (Step 2): `left: "0: Generic index=0\n"`. Caught
by M3 and M8.

**`ir::golden_tests::one_literal_written_twice_is_one_interned_constant`.** Caught by M7 as its **only**
catcher, and by M3 and M8.

**`ir::golden_tests::two_assignments_and_two_says_in_one_body_reuse_one_register`.** Added *because* of
the table: M9 was green across the whole workspace before it. It is the only catcher for M9.

### Changed tests, and why they changed subject rather than value

`every_instruction_of_an_all_generic_body_compiles_to_one_generic_op` was written out of `say 1` /
`say 2` / `n1 = 3`, all three of which this task promotes -- a body of promoted instructions says
nothing about `Op::Generic`. It is now three unpromoted instructions, and its doc says that promoting
one of *those* should redden it and that the answer is then a different instruction rather than a new
expectation.

`a_traced_select_echoes_its_header_and_each_listed_when` claimed "a `THEN` marker, a branch body and
the `END` get no echo op at all. They are `Generic`". A promoted branch body is a clause of its own and
does carry one. Corrected, and turned into the pair that says the echo op follows the *region* rather
than the construct.

The remaining seven golden streams grew the promoted regions and their shifted indices. Every jump
target, `op_of` entry and register count in them was re-derived by hand from the emitted stream and the
doc comments naming specific op indices were rewritten with them.

### The mutation table

Rows are `cargo test --workspace --no-fail-fast` at head, 1385 tests.

| mutation | result | catchers |
|---|---|---|
| M1 the `>L>` line is never emitted (the spike's own defect) | 3 red | population sweep, `BRANCH_CASES`, case files |
| M2 the `>L>` line is emitted whatever the setting says | 4 red | population sweep, `BRANCH_CASES`, `LOOP_CASES`, case files |
| M3 the `>L>` op is emitted in front of the load, not behind | 14 red | 3 harness tests + 11 golden streams |
| M5 `Store` writes through `assign_expr_target` directly, without tracing | 3 red | population sweep, `LOOP_CASES`, case files |
| M6 `Say` ignores its register and always prints a blank line | 6 red | 5 harness tests + `KNOWN_DIVERGENCES` |
| M7 no interning: one entry per occurrence | 1 red | `one_literal_written_twice_is_one_interned_constant` **alone** |
| M8 interning collapses every literal onto entry 0 | 14 red | 4 harness tests + 10 golden streams |
| M9 a promoted assignment never releases its register | 1 red | `two_assignments_and_two_says_in_one_body_reuse_one_register` **alone** |
| M9b a promoted `SAY` never releases its register | 9 red | 9 golden streams |
| M10 `Const` builds the value with `text` instead of `literal` | **0 red** | -- |
| M14 the region does not spend the first-instruction permission | 1 red | case files (`loop-header-boundaries`' own row) |
| M15 the region is not granted the first-instruction permission | 1 red | case files (**this task's** row, alone) |

**M10's green is legitimate rather than a gap.** `Interp::literal`'s own doc comment says a `SmallInt`
already reaches every consumer in the crate and that a literal's tagged form is which of two existing
representations it starts in. So the wrong call there costs an allocation per pass and not an answer,
and `Op::Const`'s doc now says exactly that rather than leaving a reader to assume a test guards it.

**M14 and M15 falsified a comment that was in the tree before this task, and its premise fails
twice.** `run_clause_region` said the permission take and the grant in front of it were both
unobservable, on the premise that an op which grants follows every region and grants again before any
`PROCEDURE` is reached. The take has been load-bearing since the loop header was flattened: a
construct whose body clauses are stepped by a nested, **non-granting** driver entry has no later grant
at all, so without it a `PROCEDURE` as a loop body's first instruction is permitted. The grant became
load-bearing with **this** promotion: a promoted clause that is itself the activation's first
instruction has to consume the pending flag, or the next clause's grant consumes it and a `PROCEDURE`
behind an assignment is permitted. Each half now names the row that catches it.

## The `varlookup` discharge

### The prediction, recorded before the release binary was built

Written at `3bcbcc53`:

> **The residual does NOT close.** The hypothesis is that the driver's cost is paid per body-range
> *entry* and so dilutes as a range holds more ops. Promoting `Assignment` does not increase what a
> range entry buys: `varlookup`'s `DO` body is entered once per pass either way, holds the same two
> clauses either way, and the per-entry cost is untouched. What changes is what each clause costs, and
> that trades one dispatch for two. **|delta| under 1% on the ratio**, sign not predicted. Neither of
> `varlookup`'s body clauses holds a literal (`x = x + 1` is an operator, `y = x` a variable read), so
> **neither gets an `Op::Const` at all** -- the part of this task with a plausible per-clause saving
> does not run on this axis even once inside the loop. Control: `emptyloop`'s body is a single `nop`
> and should not move at all.

### The verdict

**REFUTED. The residual does not close, and it widened.**

Criterion 4's estimator, both arms of one binary through `REXX_ENGINE`, interleaved inside each round
with the within-round order alternating, rounds outermost, 9 pairs, `ulimit -v 8388608`, one fresh
empty directory, at `f55ea409`:

| axis | tree-walker median | IR median | IR/TW | IR faster |
|---|---:|---:|---:|---:|
| `varlookup` (19e6) | 5.2210 s | 5.4941 s | **1.0523** | 0 of 9 |
| `emptyloop` (25e6) | 2.9629 s | 2.9294 s | 0.9887 | 9 of 9 |

`instructions:u`, base against head interleaved in one sitting:

| axis | arm | base | head | delta |
|---|---|---:|---:|---:|
| `varlookup` | tree-walker | 71.6307e9 | 72.5807e9 | **+1.326%** |
| `varlookup` | IR | 73.9297e9 | 77.8437e9 | **+5.294%** |
| `emptyloop` | tree-walker | 37.8507e9 | 37.8507e9 | **+0.000%** |
| `emptyloop` | IR | 40.3257e9 | 40.3257e9 | **+0.000%** |

`varlookup`'s IR/TW on instructions is **1.0321 at base and 1.0725 at head**. The prediction was right
in sign and wrong in size: |delta| is 4 percentage points on the ratio, not under 1.

**The control worked and it is what makes the rest readable.** `emptyloop` is byte-identical to base
on both arms to nine significant figures, so nothing in this task touches a body whose clauses it does
not promote. Its **wall clock**, meanwhile, read IR/TW 1.0526 in one sitting of this session and 0.9887
in another, on binaries with identical instruction counts -- the anchor's ~8% environment spread, in
this session, on this axis. No wall-clock reading of `emptyloop` here means anything, in either
direction, and its 0.9887 above is recorded for completeness and claimed for nothing.

### The headline op pair has no performance evidence, and Task 7-M must not start by profiling it

Measured after the fact, by counting `Op::Const` executions on every registered axis at `n` and at
`2n`: the figure is independent of `n` on all seven, so **no `Op::Const` executes inside any measured
loop.** It runs **once** on `emptyloop` (the closing `say 'done'`) and **once** on `strings` (the
subject string assigned before the loop), and **zero** times on `varlookup`, `arith`, `compound`,
`alloc4c` and `startup`. `Op::TraceLiteral` follows it one for one, and does less, because its work is
gated off on an untraced run.

**So the task's own headline pair -- the native constant load and the literal's value line -- has no
measurement behind it in either direction**, and none of the numbers in this report is evidence about
them. Stated at both ops rather than only here.

**Two consequences, and the second is what 7-M needs.** The +78 instructions per body clause come from
the clause machinery -- `Op::Clause`, `Op::TraceClause`, `Op::EvalExpr`, `Op::Store`,
`run_clause_region`, `run_region_ops` -- and **not** from `Const` or from `Store`-with-a-literal, since
`varlookup`'s body has no literal in an assignment's value position at all: `x = x + 1` is an operator
and `y = x` a variable read, both reaching `Op::EvalExpr`. A task that begins by profiling the constant
path will be profiling ops that never run on the axis it is trying to fix.

### Which remedy follows

**Reconsidering the two-level shape**, and the number says why it is that one rather than either of the
other two.

The gap is **per promoted clause**, not per body-range entry:

| | IR minus TW, per pass | per body clause |
|---|---:|---:|
| `varlookup` at base | 121 | 60.5 |
| `varlookup` at head | 277 | **138.5** |

This task adds **78 instructions per body clause** on `varlookup` and zero per body entry, which is
what refutes amortisation directly: the quantity that grew is the one paid per clause. So

* **hoisting the op range so it travels with `BodyEngine`** removes `run_bounded_from_chunk`'s
  `op_at(start)` and `run_ops`' `op_at(end)`, both per body-range *entry*. It cannot touch the 78.
* **a single-op fast path** addresses a range holding one op. `varlookup`'s promoted regions hold three
  and four ops, so it does not apply either.
* **the two-level shape** is where the 78 sits. A promoted clause pays `run_clause_region` (nine
  arguments) plus `run_region_ops` (seven) plus a second op-dispatch loop over a nineteen-arm match,
  plus a redundant `instructions.get` bounds check per op that names an instruction, plus the
  `RegionEnd`/`ClauseRegion` wrapping -- where the tree-walker pays one `step` dispatch. Every future
  promotion multiplies it, because it is per clause.

## The broader cost, which is not part of the discharge and should not be lost in it

The regression is on every registered axis whose loop body holds an assignment or a `SAY`, and it is on
**both** arms:

| axis | tree-walker | IR |
|---|---:|---:|
| `emptyloop` | +0.000% | +0.000% |
| `arith` | +0.206% | +0.843% |
| `compound` | +0.325% | +1.327% |
| `strings` | +0.442% | +1.810% |
| `alloc4c` | +0.471% | +1.925% |
| `varlookup` | +1.326% | +5.294% |

**The tree-walker column is the extraction and nothing else, established by probe rather than
argued.** A build with `step`'s two arms written out inline, `assign_evaluated`/`say_evaluated` left in
place for the ops, reads `varlookup` 71.6307e9 and `emptyloop` 37.8507e9 on the tree-walker arm --
base exactly, nine significant figures, both axes -- with the IR arm unmoved. So the whole
tree-walker cost is that `step` now reaches its `Assignment` and `Say` bodies through a shared
function.

**That probe is not the remedy.** Writing the arms out while the shared function exists for the ops is
two implementations of one instruction, which is the phase's own rule and was a Task 4b' review
finding. What was tried instead, and measured:

* `#[inline]` on both: recovers 722,000,000 on `varlookup`'s IR arm (19 per body clause) and reads
  exactly zero on every tree-walker cell and on both `emptyloop` cells. **Kept** (`f55ea409`).
* `#[inline(always)]` on both: recovers a further 76,000,000 on `varlookup`'s IR arm and costs
  `emptyloop` 550,000,000 on **both** arms -- a program whose loop body enters neither function -- so
  it perturbs `step`'s codegen rather than removing a call. Rejected.
* `#[inline(always)]` on `assign_expr_target` as well: `varlookup` tree-walker 72.7517e9, worse than
  either. Rejected.
* reverting the `eval.rs` change to `echo_literal`: `varlookup` tree-walker unmoved, IR arm 76,000,000
  better. Not the tree-walker cause.

So the residual tree-walker cost after `f55ea409` is +0.2% to +1.3% depending on the axis, attributed
to the sharing, with **no mechanism named below "LLVM's inlining decisions in `step` changed"**. Per
this phase's own rule that is recorded and not claimed, and it is the concern this report hands back.

## What I could not verify

* **Why the extraction costs `step` anything at all.** With `#[inline]` the emitted code should be what
  it was; it is not, reproducibly, to nine significant figures. Three inline permutations were measured
  and none reads base on the tree-walker arm. Naming the mechanism needs a disassembly comparison that
  this task did not do.
* **`arith`'s instruction count is not deterministic** to better than about 0.08%: its base binary read
  30.4006e9 twice and 30.4236e9 once in the same session. Its `+0.206%` row is therefore `+0.13%` to
  `+0.21%` depending on which base sample, and it is the only axis where that was seen.
* **`StackSpan` diverges between the engines for a literal-only program, and nothing tests it.**
  Now measured rather than reasoned: `say 'x'` reports `max_depth` **1** on the tree-walker and **0** on
  the compiled stream, while `say 1 + 2 + 3` reports **3** with identical `bytes` on both. `Op::Const`
  does not enter `eval`, and the field counts `eval` recursion, so 0 is the honest answer -- but it is
  an engine-visible difference in a public field and `ir_dual.rs` compares only stdout, stderr and exit
  status. **Recorded on `StackSpan` itself in the fix round**, since a gitignored report is not where
  the next reader meets it. Anything sizing a stack from it measures an operator chain, which recurses
  identically on both engines.
* **The oracle was not run against the IR arm's own output**, only against the tree-walker's; the case
  file's expected bytes are the oracle's and the two engines are asserted equal, which composes to the
  same claim but is not the same measurement.

## Fix round 1

Verdicts addressed: I2, I3, M4, M5, M6, M7, plus the out-of-scope record. No measurement changed, so
the benchmark suite was not re-run.

**I2.** `ir_dual.rs`'s module doc said the sweep "is not evidence about expression evaluation", that
"every promotion so far shares its semantics", and that a re-implementing promotion is what it exists
to catch "when it arrives". All three are now false, and the last two were the shape this tree has
rotted on before -- a claim about where the promoted/unpromoted boundary sits. Rewritten to state the
property instead: what it cannot see is *work the two engines share*, and what it does see includes an
op whose emission is not delegated. The `>L>` measurement is written in, with the oracle differential's
blindness to the same line stated beside it so the pair's division of labour is on the page. A bullet
was added to the "what it does prove" list.

**I3.** Both ops now carry the measurement, and the report has it under its own heading above. The
count came from a throwaway `Op::Const` execution counter run over every axis at `n` and `2n`,
reverted afterwards with the source restored to a matching sha256.

**M4.** On `StackSpan` itself, and on its `max_depth` field, with the probe's numbers. Not a
`KNOWN_DIVERGENCES` row: that table's rows are stdout, and its test asserts the two arms *differ*,
which does not fit a field no comparison reads.

**M5.** `compile::assert_literal_echoes_follow_their_load`, an unconditional `assert!` for the reason
its two siblings are, plus three tests: an echo in front of its load, an echo reading a register the op
in front of it did not write, and the adjacent success. **The second is the one a position-only check
would miss** -- the line lands in the right place with the wrong value in it -- and it is why the
assertion checks `dst == src` rather than just the variant.

**M6.** Corrected in place, with the correction visible rather than silent.

**M7.** The plan's Task 7 text now says `Store` *reaches* `assign_expr_target` through
`assign_evaluated`, and says why splitting at `assign_expr_target` instead would have left the `>>>`
line and the value render written twice -- which is the defect that rule exists to prevent, so the
indirection is the rule being followed rather than an exception to it.

**The out-of-scope finding, reproduced before recording it.** A `DO` block that is a branch body, whose
last body clause queues a requeueing handler, reports the second delivery's `SIGL` one clause early on
**both** engines. My own capture: `if 1 = 1 then do` / `zq = raiser()` / `end` gives oracle `G ran 5`
against `G ran 4` on both arms, and an unpromoted `CALL raiser` in the same slot gives `G ran 6` against
`G ran 5`, also on both arms. The absolute line numbers differ from the review's summary because the
programs differ by a line; the offsets and the finding are the same, and the unpromoted-`CALL` spelling
is the control that says it is pre-existing. Recorded beside `KNOWN_DIVERGENCES`, marked as **not** a
row, because that table holds programs where the two arms disagree and here they agree with each other
and differ from the oracle. It is the same elided-instruction family: the oracle ends that branch at the
block's real `END`, which carries a boundary of its own.
