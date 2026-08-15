## Task 3.9: `TRACE` source lines (`*-*` only)

**Files:**
- Modify: `rust/crates/rexx-parse/src/source.rs`, `src/clause.rs`, `src/ast.rs`,
  `src/instruction.rs`
- Test: `rust/crates/rexx-parse/tests/sourceline.rs`

**Interfaces:**
- Consumes: `Instruction::clause_span`, introduced in Task 3.6 and set from the
  `Clause::span` that Task 3.4 produces, and the `ProgramSource` inside
  `Program`/`Fragment` from Task 3.7b. Do not go looking for `clause_span` in
  Task 3.4; what Task 3.4 produces is `Clause::span`.
- Produces: the source-text slice per clause that `TRACE`'s `*-*` line needs. Nothing else — no depth field, no value-trace hooks.

**The two accessors you need already exist, and a third is forbidden.**
`ProgramSource::span_bytes(Range<usize>) -> Option<&[u8]>` turns a span into bytes,
and `line_span(n) -> Option<Range<usize>>` gives a line's content range. Both landed
in Task 3.3. There is deliberately **no** whole-text accessor, because a second way
to reach the bytes invites re-deriving line boundaries that `ProgramSource` owns.

**`span_bytes` alone is not the answer, and this is the trap in this task.** A
continued clause's span **contains** the line terminator while `trace r` **drops** it
when joining the fragments. Measured: `say "x",` / newline / `    "y"` traces as
`say "x",    "y"` — the comma kept, the newline removed, and the continuation line's
four leading blanks kept, which is why the join is not a trim. An earlier draft of this
task displayed `say "x","y"` here, contradicting its own next clause; the measured
value is the one above. Separately, `say 1,` / `  + 2` has span `0..12`, terminator
included. So this task needs a **terminator-stripping join** over the span, not a raw
slice. It is neither a slice nor a simple trim: the bytes to drop are the terminators
*inside* the span, and nothing else.

`src/clause.rs`, `src/ast.rs` and `src/instruction.rs` are in the Files list
because Step 3 adjusts spans, and a span this task finds wrong is a span Task 3.4
or Task 3.6 produced. `src/source.rs` is listed only in case the join belongs
beside `span_bytes`; it needs no new accessor, and adding one is a finding to
report rather than a licence. `src/instruction.rs` specifically, because the end byte a
`THEN`/`ELSE`/`OTHERWISE` clause stops at is chosen by the instruction parser
when it calls `split_before`, so a wrong end byte is repaired there and nowhere
else.
The AST must retain whatever `TRACE` displays; discovering later that it does not
is a rework of every node type.

`TRACE` indents by nesting depth, but **do not store a depth on each node.**
Measured: indentation is static nesting within a code body plus one level per
call frame, so the static half is derivable from the AST's own structure and
the dynamic half is the executor's stack depth. Storing it would be an AST tax
that buys nothing and has to be kept correct forever.

Scope this task to `*-*` and stop. That marker is the *source* clause, so it
needs no evaluated value and no executor. It is **not** free, though: it needs
the clause spans that Task 3.4 produces and Task 3.6 splits, which are a
different thing from what `SOURCELINE` and error reporting need. `SOURCELINE`
slices whole lines out of the retained source and error reporting resolves a
single byte offset to a line, and neither can express `if y > 5 then say "big"`
as three separate texts. An earlier draft of this plan claimed the clause spans
came free with those two, and Task 3.4 consequently had no rule producing them.

**The value traces are deliberately out of scope for Phase 3, and for this
plan.** **Every marker except `*-*`** carries an evaluated value, so all of them
can only be produced by an executor. Do not enumerate them, and this is the third
attempt at that enumeration to be wrong. The obvious list `>L> >O> >V> >>> >=>`
is incomplete; so is any list built by matching `>X>`, because two of the
eighteen non-`*-*` prefixes are `<I<` and `+++`, and `>.>` defeats a
`[A-Za-z=]` class just as `>>>` does. The authority is the interpreter's own
table: **nineteen prefixes**, `*-*` plus eighteen
(`RexxActivation.hpp:92-110`), and `TRACE.testGroup` exercises all nineteen.
`>L>` leads at 58 occurrences and `>>>` follows at 49; `>K>` appears 33 times,
once in a two-line probe (`>K> "TO" => "2"` from `do i = 1 to 2`).
"Everything except `*-*`" cannot go stale; a list can, and did.

They are a real conformance item — that testGroup is 1,338 lines with **135**
lines carrying `*-*` and **243** carrying a value marker — but committing to them
shapes the executor, because emitting an event per evaluation step is what forbids
constant folding, operation fusion and skipping intermediate materialisation.

That tension belongs in a Phase 4 decision block, made deliberately, not
inherited from a Phase 3 plan that had no business deciding it. The one thing
Phase 3 owes Phase 4 is that the AST can *reconstruct* clause source; whether
the executor emits per-operation values is Phase 4's call, and the natural
answer is a separate traced path since `TRACE` is off by default.

- [ ] **Step 1: Capture `TRACE` output from the interpreter**

Run `rust/corpus/lang/trace_output.rex` under `build/bin/rexx` and record it
through `cat -A`, so trailing blanks are visible. That file sets `trace i`, not
`trace r`, so the real capture also carries value-marker lines and the program's own
output; the block below is the `*-*` lines only, which is all this task reconstructs.
The trailing blanks matter:

```
     2 *-* x = 1 + 1$
     3 *-* y = x * 3$
     4 *-* if y > 5 $        <- trailing blank; the clause STOPS before `then`
     4 *-*   then$           <- `then` is a clause of its own, mid-line
     4 *-*     say "big"$    <- third clause on the same source line
     5 *-* trace off$
```

Six `*-*` lines from a six-line file, three of them on source line 4. That is
Task 3.4's rule 4 and Task 3.6's `split_before` observed from the outside.

`trace_output.rex` contains **no** `;` and no label, so it does not exercise Task
3.4's rule 2 or rule 3. Record two scratch probes as well — put them in the
session scratchpad, do not edit the corpus, which Phase 4 also depends on:

```rexx
/* probe A: terminators are inside the clause span */
trace r
nop;
do i = 1 to 2; say i; end
trace off
```

```rexx
/* probe B: a label is its own clause, colon included */
trace r
here: nop; say "two"
trace off
```

Measured, probe A traces `nop;`, `do i = 1 to 2;`, `say i;` **with their
semicolons** (the loop body indented one level, which acceptance strips) **and
`end`**, which is a clause of its own and easy to leave off the list. Those four
clauses produce more than four `*-*` lines, per the per-line acceptance above.
Probe B traces `here:` / `nop;` / `say "two"` as three clauses.

- [ ] **Step 2: Write a failing test that reconstructs that text from the AST**
- [ ] **Step 3: Implement, adjusting *clause* spans if reconstruction is impossible**

Acceptance: for every `*-*` line the interpreter prints for `trace_output.rex`
and for both Step 1 probes, the text reconstructed from the clause that line came
from is **byte-identical** to it, after stripping the line number, the marker and
the leading indentation — and **nothing else**. In particular a terminating `;`
and a trailing blank before a `then` are part of the expected text, not
whitespace to be trimmed. Reconstruct from `Instruction::clause_span`, not from
the node's own extent; the two differ exactly where this task is hardest.

**This is a per-line comparison, not a comparison of two sequences**, and the
distinction is not pedantic. `trace r` re-traces a loop body once per iteration,
so the `*-*` lines outnumber the clauses. Measured: a `do i = 1 to 2` loop over
one `say` prints **seven** `*-*` lines for **three** clauses, because the header,
body and `end` repeat per iteration and the header prints once more for the
exit test. Asserting equal counts would fail on any program containing a loop.

`*-*` is the *source* marker and is the only one Phase 3 can produce. Under
`trace i` the interpreter also emits value markers, and every one of those carries
an evaluated **value**, which requires execution:

```
     2 *-* x = 1 + 1      <- source. This is Phase 3's business.
       >L>   "1"          <- value. Phase 4's.
       >O>   "+" => "2"   <- value.
       >>>   "2"          <- value.
       >=>   X <= "2"     <- value.
```

An earlier draft named `>>>`/`>V>` in this task's acceptance, which would have
made it and gate criterion 6 need an interpreter -- reintroducing the exact
Phase 2 failure into the one criterion that was already clean.

If a clause cannot be reconstructed, fix the span that produced it, in
`src/clause.rs` or wherever Task 3.6 set it, and record which construct needed it.
Do **not** widen the enclosing node's span to cover the shortfall: widening the
`If` node to cover `then say "big"` produces one span where the oracle prints
three lines, and a node whose span does not match its own source text is a defect
that surfaces again in error reporting.

- [ ] **Step 4: Commit**

---

