## Task 3.7c: Block structure and the control stack

**Added after Task 3.6, because 3.7b was quietly absorbing a second task.** 3.7b is
139 lines of composition; `translateBlock` (`LanguageParser.cpp:1176`) is **509
lines raising 13 distinct errors**, measured. Folding one into the other would have
made a small, independently reviewable task into the largest in the phase and hidden
the block work inside a step called "implement the composition".

**A brief that names a file must also name what that file calls out to.** Task 3.7
missed error 99.925 at four call sites because its `syntaxError` lives in
`LanguageParser.cpp` while the task was scoped to `DirectiveParser.cpp`'s 2,867
lines. Scoping a task to a file scopes its blind spot to the same boundary. **This
task inherits exactly that exposure**: `translateBlock` is in `LanguageParser.cpp`,
but the errors it raises and the state it reads reach into `InstructionParser.cpp`
and the instruction classes. Enumerate the call-outs before implementing, and treat
any `syntaxError` reachable from `translateBlock` as in scope wherever it is
written.

**Files:**
- Create: `rust/crates/rexx-parse/src/block.rs`
- Modify: `rust/crates/rexx-parse/src/ast.rs`, `src/lib.rs`

**Interfaces:**
- Consumes: the `Vec<Instruction>` and `Vec<Directive>` that Task 3.7b assembles,
  plus `ParseCtx` for `SourceKind` and spans.
- Produces: the same instructions with their chain and jump indices filled in, and
  the block-structure errors. `Instruction` gains its `next` and jump-target
  fields **here**, because this is the first task that can populate them — Task 3.6
  deliberately omitted them rather than ship fields nothing sets.

**Ordering, and one gate consequence.** 3.7b accepts every valid program without
this task, because a flat `Vec` in index order is already the chain. What it cannot
do is *reject* an invalid block structure, so **gate criterion 4 cannot be fully met
until this task lands**. Criterion 2's `samples/` round-trip only needs valid files
and is unaffected.

- [ ] **Step 1: Port the control stack**

`pushDo`/`popDo`/`topDo`/`topDoType`/`topBlockInstruction`
(`LanguageParser.hpp:306-312`). A stack of instruction indices, not of nodes, since
the instructions live in a `Vec`.

- [ ] **Step 2: Record the oracle's answer for all 13 errors before implementing**

The thirteen, from `translateBlock` itself: `Error_Incomplete_do_else`,
`Error_Incomplete_do_then`, `Error_Then_expected_if`, `Error_Then_expected_when`,
`Error_Unexpected_end_else`, `Error_Unexpected_end_nodo`,
`Error_Unexpected_end_then`, `Error_Unexpected_label_do`,
`Error_Unexpected_label_if`, `Error_Unexpected_label_select`,
`Error_Unexpected_then_else`, `Error_Unexpected_when_otherwise`,
`Error_When_expected_whenotherwise`. **10.7** belongs on this list too, being the
mismatch number when an `END` fails to close a `SELECT` rather than a `DO`.

Capture number and sub-number for each from `build/bin/rexxc` with a minimal
program, and put the raw output in the report. Do not infer a sub-number from the
symbolic name.

- [ ] **Step 3: `END` matching**

`END` takes an optional block name, and **the name may be any symbol**:
`isSymbol()` is class-agnostic, so a number is legal. Measured: `end 1`,
`end loop`, `end a.` and `end a.1` all **parse**, while `end a b` is 21.909 and is
raised in Task 3.6 already.

**The mismatch number depends on what the `END` failed to close**, so do not assume
one: `do` / `nop` / `end 1` is **10.3**, and `select` / `when 1=1 then nop` /
`end 1` is **10.7**. Both measured. An earlier draft of this step said 10.3 flatly,
and a first test in Task 3.6 asserted 20.909 for `end 1` and was wrong, so capture
each from the oracle rather than deriving it from the shape of the rule.

- [ ] **Step 4: The must-be-first checks, and the per-body exposed-variable table**

`EXPOSE` and `USE LOCAL` must be the first instruction (99.907/99.910), read from
`lastInstruction`. Those two are `translateBlock`'s.

**99.913 is not, and an earlier draft of this step said otherwise.** It is raised in
`guardNew`, a `nextInstruction` constructor, so it is not a block error and not
method-specific. Measured, all three:

| program | result |
|---|---|
| `guard on when 1` in the **main program**, no method anywhere | **99.913** |
| `::method m` / `expose a` / `guard on when b` | **99.913** |
| `::method m` / `expose a` / `guard on when a` | **rc 0** |

So the rule is that a `GUARD` expression must reference **at least one variable that
is exposed at that point**, and what this step actually owes is the **per-body
exposed-variable table** that the check consults. Task 3.6 deferred it for exactly
that reason: it has no such table.

That also means the check logically belongs at instruction-construction time rather
than at block-assembly time. Either revisit the `Guard` instructions once the table
exists, or state plainly in the report why doing it at assembly time is equivalent.
Do not describe it as a block-structure error, which is how this went wrong the
first time.

- [ ] **Step 5: Finish `SELECT CASE`'s `WHEN` — two changes, not one**

Task 3.6 threaded the sub-number, so **35.934** is one argument at one call site.
The second change is the node: `parseCaseWhenList` builds a **list of case values**
where `parseLogical` builds an **AND**, so `when a, b then` inside `select case`
currently has the wrong shape. Single-expression `WHEN`s are identical either way,
which is why this is easy to miss and why it needs its own test.

- [ ] **Step 6: Commit**

---

