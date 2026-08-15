# Task 3.7c report: block structure and the control stack

**Status: complete, and revised once after review.** Six commits: `1e51ada7`,
`c043400a`, `f1d30e2d` and `964aefd5` for the task, then `5cb6cca7` and
`42d1389a` for the review round.
317 crate tests, 501 workspace, all green. `cargo clippy --offline --all-targets
-- -D warnings` clean and `cargo fmt --check` clean, each exit status checked on
its own line. Zero `allow(dead_code)`. 90 mutations, every one applied and every
one caught.

Section 10 records what the review changed.

Everything below was measured against `build/bin/rexxc` or `build/bin/rexx`
before any code was written, each case in a fresh `mktemp -d`, with the whole
message printed rather than grepped. 121 probes for the task and 40 more for the
review round, all raw output in the appendices.

---

## 1. The call-out enumeration

The task warned that scoping a brief to a file scopes its blind spot to the same
boundary, and that `translateBlock`'s reach into `InstructionParser.cpp` and the
instruction classes was the exposure. It is, and it accounts for seven of the
twenty errors.

Everything `translateBlock` (`LanguageParser.cpp:1176`) calls, and whether it
can raise:

| call-out | where | raises |
|---|---|---|
| `blockSyntaxError` -> `blockError` | `LanguageParser.cpp:4180` | **14.1, 14.2, 14.3, 14.4, 14.5, 14.901** |
| `RexxInstructionSelect::matchEnd` | `SelectInstruction.cpp:181` | **7.1, 10.4, 10.7** |
| `RexxBaseBlockInstruction::matchEnd` -> `matchLabel` | `BaseDoInstruction.cpp:139`, `:172` | **10.2, 10.3** |
| `nextInstruction` -> `whenNew` | `InstructionParser.cpp:2708` | **9.1**, and **35.934** through `parseCaseWhenList` |
| `nextInstruction` -> `guardNew` | `InstructionParser.cpp:2646` | **99.913** |
| `nextInstruction` -> `exposeNew` | `InstructionParser.cpp:2315` | **99.907** |
| `nextInstruction` -> `useLocalNew` | `InstructionParser.cpp:2349` | **99.910** |
| `initializeForTranslation` | `LanguageParser.cpp:1150` | nothing |
| `nextClause`, `noClauseAvailable`, `resetPosition`, `nextReal`, `previousToken` | `LanguageParser.cpp` | nothing |
| `trimClause` -> `RexxClause::trim` | `Clause.cpp:138` | nothing |
| `pushDo`, `popDo`, `topDo`, `topDoType`, `topDoIsType` | `LanguageParser.hpp:306`-`312` | nothing |
| `topBlockInstruction` | `LanguageParser.cpp:1772` | nothing |
| `flushControl` | `LanguageParser.cpp:1919` | nothing directly, calls `endIfNew` and `addClause` |
| `addClause` | `LanguageParser.cpp:2544` | nothing |
| `addLabel` | `LanguageParser.cpp:2559` | nothing |
| `thenNew` | `InstructionParser.cpp:4110` | nothing |
| `endIfNew` | `InstructionParser.cpp:2285` | nothing |
| `RexxInstructionSelect::addWhen` | `SelectInstruction.cpp:265` | nothing |
| `RexxInstructionSelect::setOtherwise` | `SelectInstruction.cpp:277` | nothing |
| `RexxInstructionEnd::setStyle` | `EndInstruction.hpp:62` | nothing |
| `resolveCalls` -> `resolve` | `LanguageParser.cpp:1690`, `ExpressionFunction.cpp:154` | nothing |
| `RexxCode` constructor, `source->getLine` | | nothing |

The rest of `nextInstruction` is Task 3.6's and is not re-listed. The four in
this table that reach it are here because each reads block state that only this
task can supply.

### Twenty errors in scope, thirteen named

The task named thirteen. The list missed seven:

* **14.1, 14.2, 14.5, 14.901** -- the four remaining `blockError` arms. The list
  had `Error_Incomplete_do_then` and `Error_Incomplete_do_else`, which are two of
  the six that function raises. This is the predicted blind spot exactly:
  `blockError` is in `LanguageParser.cpp`, but it is a *different function*, and
  its `switch` is where the block kind picks the number.
* **10.2 and 10.4** -- the label-*mismatch* variants. The list had 10.3 and 10.7,
  which fire when the block has no name at all. Whether the block has a label is
  a second axis the symbolic names do not hint at.
* **9.1** -- `Error_Unexpected_when_when`, raised in `whenNew`, outside
  `translateBlock`.
* **7.1** -- `Error_When_expected_when`, raised in `matchEnd`, outside
  `translateBlock`. The task's own two calibration examples named it as the
  `select` / `end` answer without naming where it lives.

And **7.1's** reported position is unlike every neighbour's, which is worth
stating separately: `matchEnd` passes `getLocation()`, the SELECT's own location,
where 10.2/10.3/10.4/10.7 all pass `endLocation`. Measured, with the two four
lines apart, and confirmed twice.

### Two of the thirteen are unreachable

`Error_Unexpected_end_then` (**10.5**) and `Error_Unexpected_end_else`
(**10.6**) cannot be reached from source. They are the specific arms of the END
switch for a `type` of `KEYWORD_IFTHEN`/`KEYWORD_WHENTHEN` and `KEYWORD_ELSE`,
and by the time that switch runs `flushControl` has already rewritten a THEN
frame into a branch-end frame and popped an ELSE without pushing its own marker.

**The argument is structural, and that is what makes the omission safe rather
than merely unfalsified.** An `END` has `isControl() == false` and its type is not
`KEYWORD_ELSE`, so `translateBlock` always calls `flushControl` before reaching
the `END` arm of the switch. `flushControl` cannot return with `ELSE`, `IFTHEN` or
`WHENTHEN` on top: it pops an `ELSE` outright, and it rewrites a THEN into an
`ENDTHEN`/`ENDWHEN` marker. The type those two arms test for therefore cannot be
present when they are reached. The reviewer supplied this reasoning and it is
better evidence than any number of probes.

24 probes agree. Six are mine, covering both shapes on one line and on separate
lines and one nested inside a `DO`; the reviewer added eighteen more, covering
nested IF/ELSE, a named `end a`, `if 1 = 1 then; end`, a WHEN-as-THEN followed by
an `END`, an `OTHERWISE` holding a dangling THEN, and each shape inside a
`::METHOD`. Every one answers **10.1**. Mine:

| program | oracle |
|---|---|
| `if 1 = 1 then` / `end` | 10.1 |
| `if 1 = 1 then end` | 10.1 |
| `select` / `when 1 = 1 then` / `end` | 10.1 |
| `select` / `when 1 = 1 then end` | 10.1 |
| `if 1 = 1 then nop` / `else` / `end` | 10.1 |
| `if 1 = 1 then nop` / `else end` | 10.1 |

Raw output for all six is in batches A and B of the appendix. The two arms are
therefore **omitted** rather than written and left untestable, and the test
`an_end_closing_a_then_or_an_else_is_also_10_1` carries all six cases plus the
nested variant, along with a note saying what would overturn the conclusion:
any source where `rexxc` prints 10.5 or 10.6, in which case the assumption that
broke is that `flushControl` always runs first.

This is deliberately grounded in the probes rather than in the code reading that
produced the claim, because Task 3.6 kept an arm "for fidelity" expecting it dead
and `do i over x for 1 to 2` turned out to be the oracle's own answer, 49.2.
Reading the C++ said these two are dead; the six measurements are what makes it
evidence.

---

## 2. All twenty errors, as captured

Each row's raw `rexxc` output is in the appendix batch named. Sources use blank
lines wherever a reported line is asserted, so the reported line and the
substituted line can be told apart.

| error | symbolic name | minimal program | reported line | batch |
|---|---|---|---|---|
| **7.1** | `Error_When_expected_when` | `nop` / `select` / `end` | the SELECT's | A |
| **7.2** | `Error_When_expected_whenotherwise` | `nop` / `select` / `nop` / `end` | the offender's | A |
| **8.2** | `Error_Unexpected_then_else` | `nop` / `else nop` | the ELSE's | A |
| **9.1** | `Error_Unexpected_when_when` | `nop` / `when 1 = 1 then nop` | the WHEN's | A |
| **9.2** | `Error_Unexpected_when_otherwise` | `nop` / `otherwise nop` | the OTHERWISE's | A |
| **10.1** | `Error_Unexpected_end_nodo` | `nop` / `end` | the END's | A |
| **10.2** | `Error_Unexpected_end_control` | `do label a` / `nop` / `end b` | the END's | A |
| **10.3** | `Error_Unexpected_end_nocontrol` | `do` / `nop` / `end 1` | the END's | A |
| **10.4** | `Error_Unexpected_end_select` | `select label a` / `when 1 = 1 then nop` / `end b` | the END's | A |
| **10.5** | `Error_Unexpected_end_then` | **unreachable, six probes** | -- | A, B |
| **10.6** | `Error_Unexpected_end_else` | **unreachable, six probes** | -- | A, B |
| **10.7** | `Error_Unexpected_end_select_nolabel` | `select` / `when 1 = 1 then nop` / `end 1` | the END's | A |
| **14.1** | `Error_Incomplete_do_do` | `nop` / `do` / `nop` | the last instruction's | B |
| **14.2** | `Error_Incomplete_do_select` | `nop` / `select` / `when 1 = 1 then nop` | the last instruction's | B |
| **14.3** | `Error_Incomplete_do_then` | `nop` / `if 1 = 1 then` | the THEN's | B |
| **14.4** | `Error_Incomplete_do_else` | `nop` / `if 1 = 1 then nop` / `else` | the ELSE's | B |
| **14.5** | `Error_Incomplete_do_loop` | `nop` / `loop forever` / `nop` | the last instruction's | B |
| **14.901** | `Error_Incomplete_do_otherwise` | `select` / `when 1 = 1 then nop` / `otherwise nop` | the last instruction's | B |
| **18.1** | `Error_Then_expected_if` | `nop` / `if 1 = 1` / `nop` | the offender's | C |
| **18.2** | `Error_Then_expected_when` | `select` / `when 1 = 1` / `nop` / `end` | the offender's | C |
| **47.2** | `Error_Unexpected_label_do` | `do` / `lab:` / `nop` / `end` | the label's | C |
| **47.3** | `Error_Unexpected_label_if` | `if 1 = 1 then` / `lab:` / `nop` | the label's | C |
| **47.4** | `Error_Unexpected_label_select` | `select` / `lab:` / `when 1 = 1 then nop` / `end` | the label's | C |
| **99.907** | `Error_Translation_expose` | `nop` / `expose a` | the EXPOSE's | D |
| **99.910** | `Error_Translation_use_local` | `nop` / `use local a` | the USE LOCAL's | D |
| **99.913** | `Error_Translation_guard_expose` | `guard on when 1` | the GUARD's | D |
| **35.929** | `Error_Invalid_expression_logical_list` | `select` / `when 1 = 1, then nop` / `end` | the WHEN's | E |
| **35.934** | `Error_Invalid_expression_case_when_list` | `select case 1` / `when 1, then nop` / `end` | the WHEN's | E |

`14.901` is worth calling out on its own: a **three-digit** sub-number under
code 14, where its five neighbours are 1 through 5. Deriving it from its position
in the table would have given 6.

### The sub-number depends on two axes, not one

Nothing in `Error_Unexpected_end_control` versus `Error_Unexpected_end_nocontrol`
says which is the mismatch and which the "no name to match" case, and the same is
true of the SELECT pair. Both axes measured, all four combinations (batches A and
G):

| block | END | oracle |
|---|---|---|
| `do label a` | `end b` | 10.2 |
| `do i = 1 to 3` | `end j` | 10.2 |
| `do` | `end 1` | 10.3 |
| `loop forever` | `end 1` | 10.3 |
| `do 3` | `end 1` | 10.3 |
| `select label a` | `end b` | 10.4 |
| `select` | `end 1` | 10.7 |
| `select case 1` | `end 1` | 10.7 |

And a plain `DO` versus every other DO/LOOP form for 14.1 versus 14.5, where a
`LABEL` does **not** move the number even though `getEndStyle` does distinguish
one (batch G):

| header | oracle |
|---|---|
| `do` | 14.1 |
| `do label a` | **14.1** |
| `do while 1` | 14.5 |
| `do 3` | 14.5 |
| `loop` | 14.5 |
| `loop forever` | 14.5 |
| `do i over x` | 14.5 |

---

## 3. The four brief defects, and how each was settled

### 3.1 `block.rs` cannot be a post-pass over a finished `Vec<Instruction>`

The Interfaces section specified "the same instructions with their chain and jump
indices filled in", which is a post-pass. That cannot work. Three
`nextInstruction` constructors read block state *while they parse*:

* `whenNew` (`InstructionParser.cpp:2708`) calls `topBlockInstruction()` and
  branches on the answer three ways: no block or a non-SELECT block is error 9.1,
  a `KEYWORD_SELECT` takes `requiredLogicalExpression`, a `KEYWORD_SELECT_CASE`
  takes `parseCaseWhenList`. A WHEN cannot be parsed at all without the stack.
* `guardNew` (`:2646`) counts the exposed variables its expression captured and
  raises 99.913 on zero.
* `exposeNew` (`:2315`) and `useLocalNew` (`:2349`) test
  `lastInstruction->isType(KEYWORD_FIRST)`.

So `translate_block` owns the clause loop and `parse_instruction` takes a
`&mut Block`. `parse_instructions` is gone. This is also simply what the C++ does:
`translateBlock` *is* the loop, and the version with a separate pass never
existed.

### 3.2 No `next` field, despite the brief naming one

`ast.rs:470` already argued that index order is the chain and a `next` field
would restate it. The brief said to add one anyway. Every path an instruction can
take into the chain was traced, and all four append:

* `flushControl`'s ELSE arm: `addClause(instruction)` then `addClause(second)`.
* `flushControl`'s IFTHEN/WHENTHEN arm: same order.
* `flushControl`'s fall-through arm: `addClause(instruction)`.
* the control/non-control split in `translateBlock` itself: `addClause` in one
  branch, `flushControl` in the other.

`addClause` is `lastInstruction->setNext(instruction); lastInstruction =
instruction;` and nothing else. So index order is the chain exactly, and only
jump targets were added. This is recorded on `CodeBody::instructions`.

### 3.3 The synthetic `EndIf` markers are not reproduced

I leaned towards materialising them, for fidelity: `RexxInstructionEndIf` is a
real executable node with its own jump target (`EndIf.cpp:132`) and three types
(ENDTHEN, ENDWHEN, ENDELSE). The coordinator overruled it with a `trace`
measurement I had not taken: the oracle emits **no** `*-*` line for one, so a
materialised marker would be an `Instruction` with no source clause, and two
gate criteria are stated over `clause_span` -- source-ordered and
non-overlapping, and one `*-*` line reconstructed per clause. Two exceptions is
a worse trade than one field.

So the targets live on the instructions that jump, and nothing is lost, because
`else_end` is exactly an index. What each field means, and where the C++ puts it:

| field | C++ | value |
|---|---|---|
| `If::false_target` | `else_location->nextInstruction` | the ELSE if there is one, else the instruction after the THEN branch |
| `When::false_target` | same | the next WHEN, the OTHERWISE, or the SELECT's END |
| `When::exit` | the ENDWHEN's `else_end->nextInstruction`, set by `fixWhen` | the instruction after the SELECT's END |
| `Else::then_exit` | the ENDTHEN's `else_end->nextInstruction` | the instruction after the ELSE branch |
| `Loop::end`, `Select::end` | `RexxBlockInstruction::end` | the matching END |
| `End::closes` | `RexxInstructionEnd::style` plus the block | what the END closed, and how |

`Else::then_exit` is on the ELSE and not on the THEN because that is where the
C++ writes it: `RexxInstructionElse::setEndInstruction`
(`ElseInstruction.cpp:141`) forwards to its parent, which is the THEN's marker,
and its own comment says "we poke the THEN so it knows where control goes after
its instruction completes". Executing an ELSE does nothing but trace (`:113`).

A `None` target means "control falls out of this body". A target is recorded from
`next_index()` while assembly is still running, so a branch that turns out to be
last records an index that never gets an instruction; `resolve_targets` clears
those in one pass at the end.

### 3.4 99.907 is not method-specific

Measured in the main program with no `::METHOD` anywhere:

```
nop
<blank>
expose a
  -> Error 99.907, line 3
```

The check is `lastInstruction->isType(KEYWORD_FIRST)` and nothing about it is
method-shaped. Same for 99.910. Third such correction in this phase.

---

## 4. What each measured behaviour turned into

### The exposed-variable table, and where 99.913 belongs

Done at instruction-construction time, in the port of `guardNew`, which is where
the C++ does it. The brief allowed arguing that assembly time is equivalent; it
is not needed, because the driver design of 3.1 makes `block` available to
`parse_instruction` and the faithful placement costs nothing.

The rule the measurements establish, which `isExposed`
(`LanguageParser.cpp:1991`) confirms, is a three-case ladder and the *order*
matters:

| state | exposed means |
|---|---|
| an `EXPOSE` was seen | the name is in that list |
| else a `USE LOCAL` was seen | the name is **not** in that list |
| neither | nothing is exposed |

All three measured, plus four consequences that no reading of the rule alone
would give (batch D):

| program | oracle | why |
|---|---|---|
| `expose a` / `guard on when a` | rc 0 | listed |
| `expose a` / `guard on when b` | 99.913 | not listed |
| `use local b` / `guard on when a` | rc 0 | inverted |
| `use local a` / `guard on when a` | 99.913 | inverted |
| `use local a` / `guard on when self` | **99.913** | `autoExpose` seeds SUPER, SELF, RC, RESULT, SIGL as local |
| `expose a.` / `guard on when a.1` | rc 0 | a compound contributes its STEM, `A.` |
| `expose a` / `guard on when a.1` | **99.913** | the stem of `A.1` is `A.`, not `A` |
| `expose (a)` / `guard on when a` | **99.913** | the indirect form never calls `expose` |
| `guard off when a` | 99.913 | GUARD OFF is checked too |
| `guard on` | rc 0 | no expression to check |

The C++ collects the names through `captureGuardVariable` on the
`addSimpleVariable` and `addStem` paths (`:2071`, `:2108`) as it evaluates the
expression. Walking the finished tree instead is equivalent, because capture is
unconditional over the whole expression and those two paths are reached for
exactly the variable references in it. A constant, a `.name` environment symbol
and a literal reach neither, so none of the three counts -- and `addCompound`'s
own comment says compound variables *do* get added, via the stem and each
non-constant tail piece.

### `END` matching

`END` takes an optional name and the name may be any symbol, class-agnostic.
Measured rc 0 for `end 1`, `end loop`, `end a.` and `end a.1`, each paired with a
`DO LABEL` of the same spelling (batch J). An omitted name always matches, even
when the block has a label. A literal is rejected at parse time with 20.909 and
trailing data with 21.909, both before any matching happens, which is why those
two are Task 3.6's and stay there.

### `SELECT CASE`'s WHEN: two changes, and the second needed a run-time probe

The sub-number is one argument at one call site, now that the WHEN knows its
enclosing block: 35.934 in a `SELECT CASE`, 35.929 in a plain `SELECT`. Six cases
measured (batches E and I), three of each.

The node is the half a sub-number cannot show, and a parse-only probe would have
shown both spellings accepting and told me nothing. Proven at run time with
`build/bin/rexx` instead (batch F):

```
select case 2                      select
  when 1, 2 then say "hit"           when 1, 2 then say "and hit"
  otherwise say "miss"               otherwise say "and miss"
end                                end

-> hit                             -> Error 34.6: Value of logical list
                                      expression element must be exactly
                                      "0" or "1"; found "2".
```

A list of case values against an AND of conditions. Also measured: `select case 3`
with the same WHEN takes the OTHERWISE, so it really is an OR-of-equals and not a
range; and `select` / `when 1, 1` runs its body, so the plain form really is an
AND. `WhenCase { values: Vec<Expr> }` is its own variant, mirroring
`RexxInstructionCaseWhen`, and `parse_case_when_list` never collapses a
one-element list where `parse_logical` does.

### One quirk reproduced rather than tidied away

`select` / `when 1 = 1 then` / `when 2 = 2 then nop` / `end` is **rc 0**,
measured. It is accepted because the second WHEN is the first one's THEN
instruction. `addWhen` runs only when the SELECT is the immediate top of the
control stack, and by the second WHEN a branch-end frame is on top, so the SELECT
collects one WHEN and the second is never linked to it or given an exit.
`whenNew` still accepts it, because `topBlockInstruction` drills past the
branch-end frame and finds the SELECT.

Transliterating rather than rationalising is what makes this come out right for
free, and there is a test for it. A tidied version that added every WHEN between
a SELECT and its END would give a different tree for a program the oracle
accepts.

### Per-body state

Each `translateBlock` call gets its own everything, and three consequences are
measured:

* `do` / `nop` / `::routine r` / `end` is **14.1**: a block cannot be closed
  across a directive.
* `expose a` / `::method m` / `expose b` is **rc 0**: `lastInstruction` is
  per-body, so a method body may start with an EXPOSE even though the main
  program already had one.
* a label of the same spelling in two bodies lands at index 0 of each.

`CodeBody { instructions, labels }` holds a body, and `body: bool` on the three
directive kinds that can carry one became `body: Option<CodeBody>`, per the
coordinator's answer. The label table moved from a pass over the finished list in
`lib.rs` into `Block::add_clause`, which is where `addLabel` sits in the C++, for
the same reason: only the assembler knows where one body ends.

---

## 5. Two divergences from the C++'s internal structure

Both are structural rather than behavioural, and both were checked against the
oracle on the shapes where they could show.

**The THEN arrives as its own clause.** `translateBlock` consumes the THEN token
itself, inside the IF arm, and never dispatches it. Task 3.6 already made a real
`THEN` clause produce an `InstructionKind::Then`, so here the pairing is carried
across one iteration in `Block::pending_then`. Nothing can come between the two,
because a clause after an `IF` that is not its THEN is 18.1 or 18.2. The
placement matters and is tested: a THEN that reached the SELECT-membership check
would be rejected as a non-WHEN inside a SELECT, and `select` / `when 1 = 1` /
`then nop` / `end` is measured rc 0.

**14.3 and 14.4 arrive through `blockError` rather than through the C++'s own
direct check.** The C++ raises them from the failed `nextClause()` right after a
THEN or an ELSE (`:1370`, `:1445`); here the THEN or ELSE is added, and the
end-of-body cleanup finds its frame still open and routes to `blockError`, whose
switch maps IFTHEN/WHENTHEN to 14.3 and ELSE to 14.4. The reported line comes out
the same either way, because the C++ check reports against the clause holding the
THEN while this reports against the THEN instruction, whose span is that keyword.
Discriminated with a blank line between the IF and its THEN:

```
nop
<blank>
if 1 = 1
<blank>
then
  -> Error 14 ... line 5:  Incomplete DO/LOOP/SELECT/IF.
     Error 14.3:  THEN on line 3 must be followed by an instruction.
```

Reported line 5, the THEN's; substituted line 3, the IF's. Both spellings tested,
and also the case where a directive rather than end of file terminates the body,
where the same three numbers come out (batch H).

---

## 6. Tests

315 crate tests, up from 276. `block/tests.rs` is new and holds the measured
table; `instruction/tests.rs` and `directive/tests.rs` moved to
`translate_block`, which means Task 3.6's and 3.7's suites now see block
validation too.

Three of 3.6's tests needed changing, and each was wrong in a way only the block
stack could reveal:

* `end_takes_an_optional_name_and_nothing_else` asserted `ok("select\nend")`,
  which is 7.1 in the oracle. It now pairs each `END` name with a `DO LABEL` of
  the same spelling, all measured rc 0.
* `guard_takes_on_or_off_and_an_optional_when` asserted `ok("guard on when 1")`,
  which is 99.913. It was already documented as a known deviation and is now the
  real number.
* `a_when_inside_select_case_still_gets_the_interim_35_929` was the pinned
  interim state and became
  `a_when_picks_its_grammar_from_the_enclosing_select` plus
  `a_when_inside_select_case_builds_a_value_list_not_an_and`.

### Both directions of every gate

Every rejection has an acceptance beside it: 10.2/10.3/10.4/10.7 against five
matching or omitted `END` names; 47.2/47.3/47.4 against a label after a finished
IF branch, which is allowed; 7.1 against a SELECT with a WHEN; 99.907/99.910
against an EXPOSE that is first; each 99.913 case against its inverse; and
`the_shapes_the_oracle_accepts_are_accepted` carries eight legal-but-unusual
programs whose structure a wrong stack would reject.

### Mutation testing: 78 applied, 78 caught

Script in section 8. It exits non-zero on a survivor **and** on a mutation whose
pattern matched anything but exactly once, because a never-applied mutation reads
exactly like a caught one.

The first run left three survivors, and two were real holes rather than
equivalent mutants:

* **`EndTarget { block: 0 }`** survived, because every case asserted put the
  block at index 0. Fixed by moving the `DO` behind a `NOP` and adding a nested
  pair, so the inner END is matched against the inner block.
* **a WHEN counted as a control instruction** survived, because that only skips
  `flushControl` when a branch is already pending as the WHEN arrives. The
  one-WHEN-is-another's-THEN shape has that property, so that test now asserts
  the first WHEN's `false_target` too, which the mutation moves from 4 to 6.
* the third was a pattern `cargo fmt` had rewrapped, so it was never applied.
  Fixed in the script.

Second run: 78 applied, 78 caught, none of them by a compile error. The first
run's report also mislabelled most catches "DID NOT COMPILE", because a failing
`cargo test` prints an `error:` line of its own; the detection now consults the
test-result lines first.

A clean pass is not evidence of coverage, so the two holes above are recorded as
what the run actually bought.

---

## 7. Concerns

**`Option` carries two different meanings across the jump-target fields.** On
`If::false_target`, `When::exit` and `Else::then_exit`, `None` means "control
falls out of this body" and is reachable. On `Loop::end`, `Select::end` and
`End::closes`, `None` means "not matched yet" and cannot survive into a returned
body, because an unmatched block is 14.x and an unmatched END is 10.1. Each
field's doc comment says which it is, and `Block::finish` carries a
`debug_assert!` over the second group, but the type does not distinguish them.
Removing the ambiguity would need a separate builder representation, which
seemed a poor trade for the one invariant it buys.

**`resolveCalls` is deliberately not ported.** `translateBlock`'s tail resolves
deferred `CALL`/`SIGNAL`/function targets against the label table
(`LanguageParser.cpp:1690`). It raises nothing, and its whole effect is
observable only at run time; `CodeBody::labels` now gives Phase 4 the same
information per body. Worth a line in whichever task owns call resolution, so it
is not assumed done.

**`Program` and `CodeBody` are asymmetric.** The main body's `instructions` and
`labels` are spelled out on `Program` while a directive's are a `CodeBody`. This
was the coordinator's call and is honest, but a reader may reasonably expect
`Program` to hold a `CodeBody` too.

**Gate criterion 4 should now be re-checked rather than assumed satisfied.** The
parser rejects invalid block structure, which is what the criterion was waiting
on, but "every error number the oracle raises" is a larger set than the twenty in
scope here, and I have not audited the criterion's own wording against what
landed.

---

## 8. Appendix: every probe

121 cases across ten batches, 14/15/15/20/15/6/14/12/4/6, each run in a fresh
`mktemp -d`, banner lines
stripped and everything else verbatim. `cat -A` output shows the program, so a
blank line is visible as such. The per-run temporary path is replaced by
`TMP/t.rex`, which is the only edit.

### Batch A: errors 7.1, 7.2, 8.2, 9.1, 9.2 and the 10.x family

```
=====  7.1 select with no WHEN
--- program
nop

select

end
--- output (rexxc)
     3 *-* select
Error 7 running TMP/t.rex line 3:  WHEN or OTHERWISE expected.
Error 7.1:  SELECT on line 3 requires WHEN.
--- rc=249

=====  7.2 non-WHEN inside select
--- program
nop

select

nop

end
--- output (rexxc)
     5 *-* nop
Error 7 running TMP/t.rex line 5:  WHEN or OTHERWISE expected.
Error 7.2:  SELECT on line 3 requires WHEN, OTHERWISE, or END.
--- rc=249

=====  8.2 else with no then above
--- program
nop

else nop
--- output (rexxc)
     3 *-* else nop
Error 8 running TMP/t.rex line 3:  Unexpected THEN or ELSE.
Error 8.2:  ELSE has no corresponding THEN clause.
--- rc=248

=====  9.1 when with no block at all
--- program
nop

when 1 = 1 then nop
--- output (rexxc)
     3 *-* when 1 = 1 then nop
Error 9 running TMP/t.rex line 3:  Unexpected WHEN or OTHERWISE.
Error 9.1:  WHEN has no corresponding SELECT.
--- rc=247

=====  9.1b when inside a DO
--- program
nop

do

when 1 = 1 then nop

end
--- output (rexxc)
     5 *-* when 1 = 1 then nop
Error 9 running TMP/t.rex line 5:  Unexpected WHEN or OTHERWISE.
Error 9.1:  WHEN has no corresponding SELECT.
--- rc=247

=====  9.2 otherwise with no select
--- program
nop

otherwise nop
--- output (rexxc)
     3 *-* otherwise nop
Error 9 running TMP/t.rex line 3:  Unexpected WHEN or OTHERWISE.
Error 9.2:  OTHERWISE has no corresponding SELECT.
--- rc=247

=====  9.2b otherwise inside a DO
--- program
nop

do

otherwise nop

end
--- output (rexxc)
     5 *-* otherwise nop
Error 9 running TMP/t.rex line 5:  Unexpected WHEN or OTHERWISE.
Error 9.2:  OTHERWISE has no corresponding SELECT.
--- rc=247

=====  10.1 bare end
--- program
nop

end
--- output (rexxc)
     3 *-* end
Error 10 running TMP/t.rex line 3:  Unexpected or unmatched END.
Error 10.1:  END has no corresponding DO, LOOP, or SELECT.
--- rc=246

=====  10.2 do label a / end b
--- program
nop

do label a

nop

end b
--- output (rexxc)
     7 *-* end b
Error 10 running TMP/t.rex line 7:  Unexpected or unmatched END.
Error 10.2:  Symbol following END ("B") must match block specification name ("A") on line 3 or be omitted.
--- rc=246

=====  10.3 do / end 1
--- program
nop

do

nop

end 1
--- output (rexxc)
     7 *-* end 1
Error 10 running TMP/t.rex line 7:  Unexpected or unmatched END.
Error 10.3:  END corresponding to block on line 3 must not have a symbol following it because there is no LABEL or control variable; found "1".
--- rc=246

=====  10.4 select label a / end b
--- program
nop

select label a

when 1 = 1 then nop

end b
--- output (rexxc)
     7 *-* end b
Error 10 running TMP/t.rex line 7:  Unexpected or unmatched END.
Error 10.4:  Symbol following END ("B") must match LABEL of SELECT specification ("A") on line 3 or be omitted.
--- rc=246

=====  10.5 end closing a THEN
--- program
nop

if 1 = 1 then

end
--- output (rexxc)
     5 *-* end
Error 10 running TMP/t.rex line 5:  Unexpected or unmatched END.
Error 10.1:  END has no corresponding DO, LOOP, or SELECT.
--- rc=246

=====  10.6 end closing an ELSE
--- program
nop

if 1 = 1 then nop

else

end
--- output (rexxc)
     7 *-* end
Error 10 running TMP/t.rex line 7:  Unexpected or unmatched END.
Error 10.1:  END has no corresponding DO, LOOP, or SELECT.
--- rc=246

=====  10.7 select / end 1
--- program
nop

select

when 1 = 1 then nop

end 1
--- output (rexxc)
     7 *-* end 1
Error 10 running TMP/t.rex line 7:  Unexpected or unmatched END.
Error 10.7:  END corresponding to SELECT on line 3 must not have a symbol following it because there is no LABEL; found "1".
--- rc=246
```

### Batch B: the six attempts at 10.5 and 10.6, and the 14.x family

```
=====  10.5 try: when-then then end
--- program
nop

select

when 1 = 1 then

end
--- output (rexxc)
     7 *-* end
Error 10 running TMP/t.rex line 7:  Unexpected or unmatched END.
Error 10.1:  END has no corresponding DO, LOOP, or SELECT.
--- rc=246

=====  10.5 try: if then on same line as end
--- program
nop

if 1 = 1 then end
--- output (rexxc)
     3 *-* if 1 = 1 then end
Error 10 running TMP/t.rex line 3:  Unexpected or unmatched END.
Error 10.1:  END has no corresponding DO, LOOP, or SELECT.
--- rc=246

=====  10.5 try: when then on same line as end
--- program
nop

select

when 1 = 1 then end
--- output (rexxc)
     5 *-* when 1 = 1 then end
Error 10 running TMP/t.rex line 5:  Unexpected or unmatched END.
Error 10.1:  END has no corresponding DO, LOOP, or SELECT.
--- rc=246

=====  10.6 try: else end on one line
--- program
nop

if 1 = 1 then nop

else end
--- output (rexxc)
     5 *-* else end
Error 10 running TMP/t.rex line 5:  Unexpected or unmatched END.
Error 10.1:  END has no corresponding DO, LOOP, or SELECT.
--- rc=246

=====  10.6 try: nested else then end
--- program
nop

do

if 1 = 1 then nop

else

end

end
--- output (rexxc)
    11 *-* end
Error 10 running TMP/t.rex line 11:  Unexpected or unmatched END.
Error 10.1:  END has no corresponding DO, LOOP, or SELECT.
--- rc=246

=====  14.1 unclosed do
--- program
nop

do

nop
--- output (rexxc)
     5 *-* nop
Error 14 running TMP/t.rex line 5:  Incomplete DO/LOOP/SELECT/IF.
Error 14.1:  DO instruction on line 3 requires matching END.
--- rc=242

=====  14.1b unclosed do at very end
--- program
do
--- output (rexxc)
     1 *-* do
Error 14 running TMP/t.rex line 1:  Incomplete DO/LOOP/SELECT/IF.
Error 14.1:  DO instruction on line 1 requires matching END.
--- rc=242

=====  14.2 unclosed select
--- program
nop

select

when 1 = 1 then nop
--- output (rexxc)
     5 *-* nop
Error 14 running TMP/t.rex line 5:  Incomplete DO/LOOP/SELECT/IF.
Error 14.2:  SELECT instruction on line 3 requires matching END.
--- rc=242

=====  14.3 if then at eof
--- program
nop

if 1 = 1 then
--- output (rexxc)
     3 *-* if 1 = 1 then
Error 14 running TMP/t.rex line 3:  Incomplete DO/LOOP/SELECT/IF.
Error 14.3:  THEN on line 3 must be followed by an instruction.
--- rc=242

=====  14.3b when then at eof
--- program
nop

select

when 1 = 1 then
--- output (rexxc)
     5 *-* when 1 = 1 then
Error 14 running TMP/t.rex line 5:  Incomplete DO/LOOP/SELECT/IF.
Error 14.3:  THEN on line 5 must be followed by an instruction.
--- rc=242

=====  14.4 else at eof
--- program
nop

if 1 = 1 then nop

else
--- output (rexxc)
     5 *-* else
Error 14 running TMP/t.rex line 5:  Incomplete DO/LOOP/SELECT/IF.
Error 14.4:  ELSE on line 5 must be followed by an instruction.
--- rc=242

=====  14.5 unclosed loop forever
--- program
nop

loop forever

nop
--- output (rexxc)
     5 *-* nop
Error 14 running TMP/t.rex line 5:  Incomplete DO/LOOP/SELECT/IF.
Error 14.5:  DO or LOOP instruction on line 3 requires matching END.
--- rc=242

=====  14.5b unclosed controlled do
--- program
nop

do i = 1 to 3

nop
--- output (rexxc)
     5 *-* nop
Error 14 running TMP/t.rex line 5:  Incomplete DO/LOOP/SELECT/IF.
Error 14.5:  DO or LOOP instruction on line 3 requires matching END.
--- rc=242

=====  14.901 unclosed otherwise
--- program
nop

select

when 1 = 1 then nop

otherwise nop
--- output (rexxc)
     7 *-* nop
Error 14 running TMP/t.rex line 7:  Incomplete DO/LOOP/SELECT/IF.
Error 14.901:  OTHERWISE on line 7 requires matching END.
--- rc=242

=====  14.901b otherwise with no instruction after
--- program
nop

select

when 1 = 1 then nop

otherwise
--- output (rexxc)
     7 *-* otherwise
Error 14 running TMP/t.rex line 7:  Incomplete DO/LOOP/SELECT/IF.
Error 14.901:  OTHERWISE on line 7 requires matching END.
--- rc=242
```

### Batch C: the 47.x family, 18.1/18.2, and the reported-line discriminators

```
=====  47.2 label inside do
--- program
nop

do

lab:

nop

end
--- output (rexxc)
     5 *-* lab:
Error 47 running TMP/t.rex line 5:  Unexpected label.
Error 47.2:  Labels are not allowed within a DO/LOOP block; found "LAB".
--- rc=209

=====  47.3 label inside if-then
--- program
nop

if 1 = 1 then

lab:

nop
--- output (rexxc)
     5 *-* lab:
Error 47 running TMP/t.rex line 5:  Unexpected label.
Error 47.3:  Labels are not allowed within an IF block; found "LAB".
--- rc=209

=====  47.3b label before else
--- program
nop

if 1 = 1 then nop

lab:

else nop
--- output (rexxc)
     7 *-* else nop
Error 47 running TMP/t.rex line 7:  Unexpected label.
Error 47.3:  Labels are not allowed within an IF block; found "ELSE".
--- rc=209

=====  47.4 label inside select
--- program
nop

select

lab:

when 1 = 1 then nop

end
--- output (rexxc)
     5 *-* lab:
Error 47 running TMP/t.rex line 5:  Unexpected label.
Error 47.4:  Labels are not allowed within a SELECT block; found "LAB".
--- rc=209

=====  47.4b label inside otherwise
--- program
nop

select

when 1 = 1 then nop

otherwise

lab:

nop

end
--- output (rexxc)
     9 *-* lab:
Error 47 running TMP/t.rex line 9:  Unexpected label.
Error 47.4:  Labels are not allowed within a SELECT block; found "LAB".
--- rc=209

=====  47.2b label inside loop
--- program
nop

loop forever

lab:

nop

end
--- output (rexxc)
     5 *-* lab:
Error 47 running TMP/t.rex line 5:  Unexpected label.
Error 47.2:  Labels are not allowed within a DO/LOOP block; found "LAB".
--- rc=209

=====  18.1 if with no then
--- program
nop

if 1 = 1

nop
--- output (rexxc)
     5 *-* nop
Error 18 running TMP/t.rex line 5:  THEN expected.
Error 18.1:  IF instruction on line 3 requires matching THEN clause.
--- rc=238

=====  18.2 when with no then
--- program
nop

select

when 1 = 1

nop

end
--- output (rexxc)
     7 *-* nop
Error 18 running TMP/t.rex line 7:  THEN expected.
Error 18.2:  WHEN instruction on line 5 requires matching THEN clause.
--- rc=238

=====  disc 14.3 then on its own line
--- program
nop

if 1 = 1

then
--- output (rexxc)
     5 *-* then
Error 14 running TMP/t.rex line 5:  Incomplete DO/LOOP/SELECT/IF.
Error 14.3:  THEN on line 3 must be followed by an instruction.
--- rc=242

=====  disc 14.4 else line vs if line
--- program
nop

if 1 = 1 then nop

nop

else
--- output (rexxc)
     7 *-* else
Error 8 running TMP/t.rex line 7:  Unexpected THEN or ELSE.
Error 8.2:  ELSE has no corresponding THEN clause.
--- rc=248

=====  disc 14.1 reported line is last instruction
--- program
do

nop

nop

nop
--- output (rexxc)
     7 *-* nop
Error 14 running TMP/t.rex line 7:  Incomplete DO/LOOP/SELECT/IF.
Error 14.1:  DO instruction on line 1 requires matching END.
--- rc=242

=====  ok labelled do end matches
--- program
nop

do label a

nop

end a
--- output (rexxc)
--- rc=0

=====  ok control variable is the label
--- program
nop

do i = 1 to 3

nop

end i
--- output (rexxc)
--- rc=0

=====  ok end with a period name
--- program
nop

do label a.

nop

end a.
--- output (rexxc)
--- rc=0

=====  ok end 1 with label 1
--- program
nop

do label 1

nop

end 1
--- output (rexxc)
--- rc=0
```

### Batch D: 99.907, 99.910 and every 99.913 variant

```
=====  99.907 expose not first
--- program
::method m

nop

expose a
--- output (rexxc)
     5 *-* expose a
Error 99 running TMP/t.rex line 5:  Translation error.
Error 99.907:  EXPOSE must be the first instruction executed after a method invocation.
--- rc=157

=====  99.907b expose after a label
--- program
::method m

lab:

expose a
--- output (rexxc)
     5 *-* expose a
Error 99 running TMP/t.rex line 5:  Translation error.
Error 99.907:  EXPOSE must be the first instruction executed after a method invocation.
--- rc=157

=====  99.907c expose first is ok
--- program
::method m

expose a

nop
--- output (rexxc)
--- rc=0

=====  99.910 use local not first
--- program
::method m

nop

use local a
--- output (rexxc)
     5 *-* use local a
Error 99 running TMP/t.rex line 5:  Translation error.
Error 99.910:  USE LOCAL must be the first instruction executed after a method invocation.
--- rc=157

=====  99.910b use local first is ok
--- program
::method m

use local a

nop
--- output (rexxc)
--- rc=0

=====  99.907d expose in main program first is ok
--- program
expose a

nop
--- output (rexxc)
--- rc=0

=====  99.907e expose in main program not first
--- program
nop

expose a
--- output (rexxc)
     3 *-* expose a
Error 99 running TMP/t.rex line 3:  Translation error.
Error 99.907:  EXPOSE must be the first instruction executed after a method invocation.
--- rc=157

=====  99.913 guard on when 1 in main program
--- program
guard on when 1
--- output (rexxc)
     1 *-* guard on when 1
Error 99 running TMP/t.rex line 1:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  99.913b method expose a guard on when b
--- program
::method m

expose a

guard on when b
--- output (rexxc)
     5 *-* guard on when b
Error 99 running TMP/t.rex line 5:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  99.913c method expose a guard on when a is ok
--- program
::method m

expose a

guard on when a
--- output (rexxc)
--- rc=0

=====  99.913d method use local b guard on when a
--- program
::method m

use local b

guard on when a
--- output (rexxc)
--- rc=0

=====  99.913e method use local a guard on when a
--- program
::method m

use local a

guard on when a
--- output (rexxc)
     5 *-* guard on when a
Error 99 running TMP/t.rex line 5:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  99.913f method no expose guard on when a
--- program
::method m

guard on when a
--- output (rexxc)
     3 *-* guard on when a
Error 99 running TMP/t.rex line 3:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  99.913g guard off when a with no expose
--- program
::method m

guard off when a
--- output (rexxc)
     3 *-* guard off when a
Error 99 running TMP/t.rex line 3:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  99.913h expose a. guard on when a.1
--- program
::method m

expose a.

guard on when a.1
--- output (rexxc)
--- rc=0

=====  99.913i expose a guard on when a + b
--- program
::method m

expose a

guard on when a & b
--- output (rexxc)
--- rc=0

=====  99.913j guard on with no when is ok
--- program
::method m

guard on
--- output (rexxc)
--- rc=0

=====  99.913k use local guard on when self
--- program
::method m

use local a

guard on when self
--- output (rexxc)
     5 *-* guard on when self
Error 99 running TMP/t.rex line 5:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  99.913l expose a guard on when a.1 stem not exposed
--- program
::method m

expose a

guard on when a.1
--- output (rexxc)
     5 *-* guard on when a.1
Error 99 running TMP/t.rex line 5:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  99.913m expose (a) indirect
--- program
::method m

expose (a)

guard on when a
--- output (rexxc)
     5 *-* guard on when a
Error 99 running TMP/t.rex line 5:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157
```

### Batch E: 35.929 vs 35.934, and the SELECT/END corners

```
=====  35.934 select case when with trailing comma
--- program
nop

select case 1

when 1, then nop

end
--- output (rexxc)
     5 *-* when 1, then nop
Error 35 running TMP/t.rex line 5:  Invalid expression.
Error 35.934:  Missing expression in WHEN case expression list.
--- rc=221

=====  35.929 plain select when with trailing comma
--- program
nop

select

when 1 = 1, then nop

end
--- output (rexxc)
     5 *-* when 1 = 1, then nop
Error 35 running TMP/t.rex line 5:  Invalid expression.
Error 35.929:  Missing expression in logical expression list.
--- rc=221

=====  35.934b select case when empty
--- program
nop

select case 1

when , 1 then nop

end
--- output (rexxc)
     5 *-* when , 1 then nop
Error 35 running TMP/t.rex line 5:  Invalid expression.
Error 35.934:  Missing expression in WHEN case expression list.
--- rc=221

=====  35.929b if with trailing comma
--- program
nop

if 1 = 1, then nop
--- output (rexxc)
     3 *-* if 1 = 1, then nop
Error 35 running TMP/t.rex line 3:  Invalid expression.
Error 35.929:  Missing expression in logical expression list.
--- rc=221

=====  9.1c when directly under otherwise
--- program
nop

select

when 1 = 1 then nop

otherwise nop

when 2 = 2 then nop

end
--- output (rexxc)
     9 *-* when 2 = 2 then nop
Error 9 running TMP/t.rex line 9:  Unexpected WHEN or OTHERWISE.
Error 9.1:  WHEN has no corresponding SELECT.
--- rc=247

=====  7.2b if inside select
--- program
nop

select

if 1 = 1 then nop

end
--- output (rexxc)
     5 *-* if 1 = 1 then nop
Error 7 running TMP/t.rex line 5:  WHEN or OTHERWISE expected.
Error 7.2:  SELECT on line 3 requires WHEN, OTHERWISE, or END.
--- rc=249

=====  7.2c do inside select
--- program
nop

select

do

end

end
--- output (rexxc)
     5 *-* do
Error 7 running TMP/t.rex line 5:  WHEN or OTHERWISE expected.
Error 7.2:  SELECT on line 3 requires WHEN, OTHERWISE, or END.
--- rc=249

=====  7.2d select case with otherwise only
--- program
nop

select case 1

otherwise nop

end
--- output (rexxc)
     3 *-* select case 1
Error 7 running TMP/t.rex line 3:  WHEN or OTHERWISE expected.
Error 7.1:  SELECT on line 3 requires WHEN.
--- rc=249

=====  ok label after endthen then plain instruction
--- program
nop

if 1 = 1 then nop

lab:

nop
--- output (rexxc)
--- rc=0

=====  47.4c label after a when
--- program
nop

select

when 1 = 1 then nop

lab:

when 2 = 2 then nop

end
--- output (rexxc)
     7 *-* lab:
Error 47 running TMP/t.rex line 7:  Unexpected label.
Error 47.4:  Labels are not allowed within a SELECT block; found "LAB".
--- rc=209

=====  ok select label a otherwise end a
--- program
nop

select label a

when 1 = 1 then nop

otherwise nop

end a
--- output (rexxc)
--- rc=0

=====  10.4b select label a otherwise end b
--- program
nop

select label a

when 1 = 1 then nop

otherwise nop

end b
--- output (rexxc)
     9 *-* end b
Error 10 running TMP/t.rex line 9:  Unexpected or unmatched END.
Error 10.4:  Symbol following END ("B") must match LABEL of SELECT specification ("A") on line 3 or be omitted.
--- rc=246

=====  ok end a. and end a.1 parse
--- program
nop

do label a.1

nop

end a.1
--- output (rexxc)
--- rc=0

=====  21.909 end a b
--- program
nop

do

nop

end a b
--- output (rexxc)
     7 *-* end a b
Error 21 running TMP/t.rex line 7:  Invalid data on end of clause.
Error 21.909:  Data must not follow the END name; found "B".
--- rc=235

=====  20.909 end with a literal
--- program
nop

do

nop

end "a"
--- output (rexxc)
     7 *-* end "a"
Error 20 running TMP/t.rex line 7:  Symbol expected.
Error 20.909:  Symbol expected after END keyword.
--- rc=236
```

### Batch F: the SELECT CASE value list, proven at run time with build/bin/rexx

```
=====  sem select case list matches second value
--- program
select case 2
  when 1, 2 then say "case-list hit"
  otherwise say "case-list miss"
end
--- output (rexx)
case-list hit
--- rc=0

=====  sem plain select same shape is an AND
--- program
select
  when 1, 2 then say "and hit"
  otherwise say "and miss"
end
--- output (rexx)
     2 *-*   when 1, 2 
Error 34 running TMP/t.rex line 2:  Logical value not 0 or 1.
Error 34.6:  Value of logical list expression element must be exactly "0" or "1"; found "2".
--- rc=222

=====  sem select case list matches first value
--- program
select case 1
  when 1, 2 then say "case-list hit first"
  otherwise say "miss"
end
--- output (rexx)
case-list hit first
--- rc=0

=====  sem select case list matches neither
--- program
select case 3
  when 1, 2 then say "hit"
  otherwise say "case-list miss"
end
--- output (rexx)
case-list miss
--- rc=0

=====  sem plain select and of two truths
--- program
select
  when 1, 1 then say "and of two truths hit"
  otherwise say "miss"
end
--- output (rexx)
and of two truths hit
--- rc=0

=====  sem select case single value
--- program
select case 2
  when 2 then say "single hit"
  otherwise say "miss"
end
--- output (rexx)
single hit
--- rc=0
```

### Batch G: which DO forms take 14.1 and which take 14.5, plus the 10.x pairings

```
=====  14.x do label a unclosed
--- program
nop

do label a

nop
--- output (rexxc)
     5 *-* nop
Error 14 running TMP/t.rex line 5:  Incomplete DO/LOOP/SELECT/IF.
Error 14.1:  DO instruction on line 3 requires matching END.
--- rc=242

=====  14.x do while unclosed
--- program
nop

do while 1

nop
--- output (rexxc)
     5 *-* nop
Error 14 running TMP/t.rex line 5:  Incomplete DO/LOOP/SELECT/IF.
Error 14.5:  DO or LOOP instruction on line 3 requires matching END.
--- rc=242

=====  14.x do 3 unclosed
--- program
nop

do 3

nop
--- output (rexxc)
     5 *-* nop
Error 14 running TMP/t.rex line 5:  Incomplete DO/LOOP/SELECT/IF.
Error 14.5:  DO or LOOP instruction on line 3 requires matching END.
--- rc=242

=====  14.x loop alone unclosed
--- program
nop

loop

nop
--- output (rexxc)
     5 *-* nop
Error 14 running TMP/t.rex line 5:  Incomplete DO/LOOP/SELECT/IF.
Error 14.5:  DO or LOOP instruction on line 3 requires matching END.
--- rc=242

=====  14.x do i over x unclosed
--- program
nop

do i over x

nop
--- output (rexxc)
     5 *-* nop
Error 14 running TMP/t.rex line 5:  Incomplete DO/LOOP/SELECT/IF.
Error 14.5:  DO or LOOP instruction on line 3 requires matching END.
--- rc=242

=====  14.x select case unclosed
--- program
nop

select case 1

when 1 then nop
--- output (rexxc)
     5 *-* nop
Error 14 running TMP/t.rex line 5:  Incomplete DO/LOOP/SELECT/IF.
Error 14.2:  SELECT instruction on line 3 requires matching END.
--- rc=242

=====  10.x do label a end 1
--- program
nop

do label a

nop

end 1
--- output (rexxc)
     7 *-* end 1
Error 10 running TMP/t.rex line 7:  Unexpected or unmatched END.
Error 10.2:  Symbol following END ("1") must match block specification name ("A") on line 3 or be omitted.
--- rc=246

=====  10.x loop forever end 1
--- program
nop

loop forever

nop

end 1
--- output (rexxc)
     7 *-* end 1
Error 10 running TMP/t.rex line 7:  Unexpected or unmatched END.
Error 10.3:  END corresponding to block on line 3 must not have a symbol following it because there is no LABEL or control variable; found "1".
--- rc=246

=====  10.x do i = 1 to 3 end j
--- program
nop

do i = 1 to 3

nop

end j
--- output (rexxc)
     7 *-* end j
Error 10 running TMP/t.rex line 7:  Unexpected or unmatched END.
Error 10.2:  Symbol following END ("J") must match block specification name ("I") on line 3 or be omitted.
--- rc=246

=====  10.x do 3 end 1
--- program
nop

do 3

nop

end 1
--- output (rexxc)
     7 *-* end 1
Error 10 running TMP/t.rex line 7:  Unexpected or unmatched END.
Error 10.3:  END corresponding to block on line 3 must not have a symbol following it because there is no LABEL or control variable; found "1".
--- rc=246

=====  10.x select case 1 end 1
--- program
nop

select case 1

when 1 then nop

end 1
--- output (rexxc)
     7 *-* end 1
Error 10 running TMP/t.rex line 7:  Unexpected or unmatched END.
Error 10.7:  END corresponding to SELECT on line 3 must not have a symbol following it because there is no LABEL; found "1".
--- rc=246

=====  ok do i over x end i
--- program
nop

x = .array~of(1)

do i over x

nop

end i
--- output (rexxc)
--- rc=0

=====  9.1 when inside select case under otherwise
--- program
nop

select case 1

when 1 then nop

otherwise nop

when 2 then nop

end
--- output (rexxc)
     9 *-* when 2 then nop
Error 9 running TMP/t.rex line 9:  Unexpected WHEN or OTHERWISE.
Error 9.1:  WHEN has no corresponding SELECT.
--- rc=247

=====  7.2 select case with a nop
--- program
nop

select case 1

nop

end
--- output (rexxc)
     5 *-* nop
Error 7 running TMP/t.rex line 5:  WHEN or OTHERWISE expected.
Error 7.2:  SELECT on line 3 requires WHEN, OTHERWISE, or END.
--- rc=249
```

### Batch H: the shapes that must still be ACCEPTED, and the directive boundary

```
=====  then on its own line inside select
--- program
nop

select

when 1 = 1

then nop

end
--- output (rexxc)
--- rc=0

=====  else directly inside a select
--- program
nop

select

else nop

end
--- output (rexxc)
     5 *-* else nop
Error 7 running TMP/t.rex line 5:  WHEN or OTHERWISE expected.
Error 7.2:  SELECT on line 3 requires WHEN, OTHERWISE, or END.
--- rc=249

=====  else after a when
--- program
nop

select

when 1 = 1 then nop

else nop

end
--- output (rexxc)
     7 *-* else nop
Error 8 running TMP/t.rex line 7:  Unexpected THEN or ELSE.
Error 8.2:  ELSE has no corresponding THEN clause.
--- rc=248

=====  when whose then instruction is another when
--- program
nop

select

when 1 = 1 then

when 2 = 2 then nop

end
--- output (rexxc)
--- rc=0

=====  if then terminated by a directive
--- program
nop

if 1 = 1 then

::routine r
--- output (rexxc)
     3 *-* then
Error 14 running TMP/t.rex line 3:  Incomplete DO/LOOP/SELECT/IF.
Error 14.3:  THEN on line 3 must be followed by an instruction.
--- rc=242

=====  if with no then terminated by a directive
--- program
nop

if 1 = 1

::routine r
--- output (rexxc)
     5 *-* ::routine r
Error 18 running TMP/t.rex line 5:  THEN expected.
Error 18.1:  IF instruction on line 3 requires matching THEN clause.
--- rc=238

=====  do terminated by a directive
--- program
nop

do

::routine r
--- output (rexxc)
     3 *-* do
Error 14 running TMP/t.rex line 3:  Incomplete DO/LOOP/SELECT/IF.
Error 14.1:  DO instruction on line 3 requires matching END.
--- rc=242

=====  expose after a directive body starts
--- program
::routine r

nop

expose a
--- output (rexxc)
     5 *-* expose a
Error 99 running TMP/t.rex line 5:  Translation error.
Error 99.907:  EXPOSE must be the first instruction executed after a method invocation.
--- rc=157

=====  then inside a do
--- program
nop

do

if 1 = 1

then nop

end
--- output (rexxc)
--- rc=0

=====  otherwise inside select case then when
--- program
nop

select case 1

when 1 then nop

otherwise nop

end
--- output (rexxc)
--- rc=0

=====  nested select in a when
--- program
nop

select

when 1 = 1 then

select

when 2 = 2 then nop

end

end
--- output (rexxc)
--- rc=0

=====  when inside a nested do inside a select
--- program
nop

select

when 1 = 1 then

do

when 2 = 2 then nop

end

end
--- output (rexxc)
     9 *-* when 2 = 2 then nop
Error 9 running TMP/t.rex line 9:  Unexpected WHEN or OTHERWISE.
Error 9.1:  WHEN has no corresponding SELECT.
--- rc=247
```

### Batch I: the four SELECT CASE missing-element cases

```
=====  case when empty first element
--- program
select case 1
when , 1 = 1 then nop
end
--- output (rexxc)
     2 *-* when , 1 = 1 then nop
Error 35 running TMP/t.rex line 2:  Invalid expression.
Error 35.934:  Missing expression in WHEN case expression list.
--- rc=221

=====  case when nothing at all
--- program
select case 1
when then nop
end
--- output (rexxc)
     2 *-* when then nop
Error 35 running TMP/t.rex line 2:  Invalid expression.
Error 35.934:  Missing expression in WHEN case expression list.
--- rc=221

=====  plain when nothing at all
--- program
select
when then nop
end
--- output (rexxc)
     2 *-* when then nop
Error 35 running TMP/t.rex line 2:  Invalid expression.
Error 35.929:  Missing expression in logical expression list.
--- rc=221

=====  case when trailing comma
--- program
select case 1
when 1 = 1, then nop
end
--- output (rexxc)
     2 *-* when 1 = 1, then nop
Error 35 running TMP/t.rex line 2:  Invalid expression.
Error 35.934:  Missing expression in WHEN case expression list.
--- rc=221
```

### Batch J: END names that match, and the two parse-level END errors

```
=====  do label loop end loop
--- program
do label loop
nop
end loop
--- output (rexxc)
--- rc=0

=====  select label 1 when end 1
--- program
select label 1
when 1 = 1 then nop
end 1
--- output (rexxc)
--- rc=0

=====  do label a.1 end a.1
--- program
do label a.1
nop
end a.1
--- output (rexxc)
--- rc=0

=====  end literal inside a do
--- program
do
nop
end "x"
--- output (rexxc)
     3 *-* end "x"
Error 20 running TMP/t.rex line 3:  Symbol expected.
Error 20.909:  Symbol expected after END keyword.
--- rc=236

=====  end trailing data inside a do
--- program
do
nop
end a b
--- output (rexxc)
     3 *-* end a b
Error 21 running TMP/t.rex line 3:  Invalid data on end of clause.
Error 21.909:  Data must not follow the END name; found "B".
--- rc=235

=====  guard on when 1 inside select case body ok
--- program
select case 1
when 1, 2 then nop
end
--- output (rexxc)
--- rc=0
```

---

## 9. Appendix: the mutation script and its run

### The run

```
01 an unclosed DO gets the loop number                                   test result: FAILED. 231 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
02 an unclosed loop gets the block number                                test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
03 an unclosed SELECT gets the DO number                                 test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
04 an unclosed OTHERWISE gets the table position                         test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
05 an unclosed THEN and ELSE are swapped                                 test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
06 a block error is reported against the block, not the last instruction test result: FAILED. 233 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
07 a label in a DO gets the IF number                                    test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
08 a label in an IF gets the SELECT number                               test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
09 a label in a SELECT gets the DO number                                test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
10 a label after a finished WHEN branch is allowed                       test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
11 a label after a finished IF branch is rejected                        test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
12 a label at the top level is rejected                                  test result: FAILED. 219 passed; 16 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
13 no label placement is checked at all                                  test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
14 a label immediately before an ELSE is allowed                         test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
15 the two DO mismatch numbers are swapped                               test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
16 the two SELECT mismatch numbers are swapped                           test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
17 an unnamed DO gets the named number                                   test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
18 an unnamed SELECT gets the named number                               test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
19 a DO mismatch is reported with the SELECT numbers                     test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
20 a mismatching name is accepted                                        test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
21 an omitted END name is rejected                                       test result: FAILED. 201 passed; 34 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
22 an END with nothing open is accepted                                  test result: FAILED. 233 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
23 an END on an OTHERWISE closes the OTHERWISE                           test result: FAILED. 221 passed; 14 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
24 a SELECT with no WHEN is accepted                                     test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
25 a SELECT with no WHEN is reported against the END                     test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
26 a non-WHEN inside a SELECT is accepted                                test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
27 an END inside a SELECT trips the membership check                     test result: FAILED. 225 passed; 10 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
28 the membership check does not run for an ELSE                         test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
29 an ELSE may follow a finished WHEN branch                             test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
30 an ELSE with no THEN is accepted                                      test result: FAILED. 233 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
31 an OTHERWISE outside a SELECT is accepted                             test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
32 a WHEN outside a SELECT is accepted                                   test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
33 the enclosing-block search stops at the first frame                   test result: FAILED. 229 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
34 the enclosing-block search runs from the bottom                       test result: FAILED. 233 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
35 a SELECT CASE WHEN uses the logical grammar                           test result: FAILED. 233 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
36 the case list raises the logical number                               test result: FAILED. 233 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
37 EXPOSE may appear anywhere                                            test result: FAILED. 233 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
38 USE LOCAL gets the EXPOSE number                                      test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
39 USE LOCAL may appear anywhere                                         test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
40 a leading label does not break the must-be-first rule                 test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
41 a GUARD WHEN expression needs no exposed variable                     test result: FAILED. 230 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
42 an EXPOSE list exposes nothing                                        test result: FAILED. 231 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
43 the indirect EXPOSE form exposes its name too                         test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
44 USE LOCAL does not invert the rule                                    test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
45 an EXPOSE list is not consulted                                       test result: FAILED. 231 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s
46 USE LOCAL seeds no special names                                      test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
47 USE LOCAL does not record its own names                               test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
48 a guard expression is not walked into its subterms                    test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
49 a compound contributes only its stem                                  test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
50 a compound contributes its whole name rather than its stem            test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
51 an IF's false target is off by one                                    test result: FAILED. 227 passed; 8 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
52 an ELSE's exit is off by one                                          test result: FAILED. 232 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
53 a WHEN's exit is the END rather than past it                          test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
54 a target past the end is kept rather than cleared                     test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
55 no target is resolved at all                                          test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
56 a block does not learn its END                                        test result: FAILED. 210 passed; 25 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
57 a SELECT does not learn its END                                       test result: FAILED. 219 passed; 16 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
58 an END does not record which block it closed                          test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
59 the two DO styles are swapped                                         test result: FAILED. 233 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
60 a loop's style respects its label                                     test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
61 a SELECT with no OTHERWISE gets the OTHERWISE style                   test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
62 a labelled OTHERWISE is not distinguished                             test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
63 a WHEN joins its SELECT even when a branch is open                    test result: FAILED. 219 passed; 16 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
64 the OTHERWISE is not recorded on its SELECT                           test result: FAILED. 232 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
65 a THEN goes through the ordinary dispatch                             test result: FAILED. 205 passed; 30 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
66 a WHEN counts as a control instruction                                test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
67 an IF does not count as a control instruction                         test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
68 a plain DO opens a loop frame                                         test result: FAILED. 231 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
69 SELECT CASE opens a plain SELECT frame                                test result: FAILED. 233 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
70 the pending-branch loop runs for an ELSE too                          test result: FAILED. 221 passed; 14 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
71 the pending-branch loop runs once rather than to exhaustion           test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
72 flushControl's ELSE arm stops instead of going around                 test result: FAILED. 232 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
73 a closed block does not flush its pending branch                      test result: FAILED. 230 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
74 an unclosed block at the end of a body is accepted                    test result: FAILED. 231 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
75 the body-end branch cleanup does not run                              test result: FAILED. 224 passed; 11 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
76 a missing THEN at the end of a body is not reported                   test result: FAILED. 232 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
77 the last label of a name wins                                         test result: FAILED. 234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
78 a label index is off by one                                           test result: FAILED. 176 passed; 59 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s

BASELINE_after_revert                                                    ALL PASSED (7 suites)

78 mutations, 0 survived or were never applied
```

### The script

```python
#!/usr/bin/env python3
"""Apply one mutation to block.rs or instruction.rs, run the crate tests, revert.

Each mutation is a (name, file, old, new) tuple targeting one load-bearing
decision. A mutation that no test catches is a hole in the tests, and a mutation
whose pattern matches nothing was never applied at all -- which reads exactly
like a mutation that was caught. Both are failures, and the script exits
non-zero on either.
"""
import os
import subprocess
import sys

ROOT = "/home/moritz/dev/repos/ooRexx-rust-rewrite/rust"
BLK = os.path.join(ROOT, "crates/rexx-parse/src/block.rs")
INS = os.path.join(ROOT, "crates/rexx-parse/src/instruction.rs")

MUTATIONS = [
    # ---- blockError: the six 14.x numbers ----
    ("01 an unclosed DO gets the loop number", BLK,
     "            Control::Do => 1,", "            Control::Do => 5,"),
    ("02 an unclosed loop gets the block number", BLK,
     "            Control::Loop => 5,", "            Control::Loop => 1,"),
    ("03 an unclosed SELECT gets the DO number", BLK,
     "            Control::Select | Control::SelectCase => 2,",
     "            Control::Select | Control::SelectCase => 1,"),
    ("04 an unclosed OTHERWISE gets the table position", BLK,
     "            Control::Otherwise => 901,", "            Control::Otherwise => 6,"),
    ("05 an unclosed THEN and ELSE are swapped", BLK,
     "            Control::IfThen | Control::WhenThen => 3,\n"
     "            // `Error_Incomplete_do_else`.\n"
     "            Control::Else => 4,",
     "            Control::IfThen | Control::WhenThen => 4,\n"
     "            // `Error_Incomplete_do_else`.\n"
     "            Control::Else => 3,"),
    ("06 a block error is reported against the block, not the last instruction", BLK,
     "        ParseError::new(14, sub, self.last_byte())",
     "        ParseError::new(14, sub, self.top().index.map_or(0, |i| self.instructions[i].clause_span.start))"),

    # ---- the misplaced-label check ----
    ("07 a label in a DO gets the IF number", BLK,
     "            Control::Do | Control::Loop => 2,", "            Control::Do | Control::Loop => 3,"),
    ("08 a label in an IF gets the SELECT number", BLK,
     "            Control::IfThen | Control::Else => 3,",
     "            Control::IfThen | Control::Else => 4,"),
    ("09 a label in a SELECT gets the DO number", BLK,
     "            | Control::Otherwise => 4,", "            | Control::Otherwise => 2,"),
    ("10 a label after a finished WHEN branch is allowed", BLK,
     "            | Control::EndWhen\n            | Control::Otherwise => 4,",
     "            | Control::Otherwise => 4,\n            Control::EndWhen => return None,"),
    ("11 a label after a finished IF branch is rejected", BLK,
     "            Control::First | Control::EndThen => return None,",
     "            Control::EndThen => 3,\n            Control::First => return None,"),
    ("12 a label at the top level is rejected", BLK,
     "            Control::First | Control::EndThen => return None,",
     "            Control::First => 2,\n            Control::EndThen => return None,"),
    ("13 no label placement is checked at all", BLK,
     "            if let Some(error) = block.label_error(instruction.clause_span.start) {\n"
     "                return Err(error);\n"
     "            }\n", ""),
    ("14 a label immediately before an ELSE is allowed", BLK,
     "            if block.last_is_label() {\n                return Err(ParseError::new(47, 3, byte));\n            }\n", ""),

    # ---- matchLabel: the four 10.x mismatch numbers ----
    ("15 the two DO mismatch numbers are swapped", BLK,
     "            (Some(_), false) => 2,", "            (Some(_), false) => 3,"),
    ("16 the two SELECT mismatch numbers are swapped", BLK,
     "            (Some(_), true) => 4,", "            (Some(_), true) => 7,"),
    ("17 an unnamed DO gets the named number", BLK,
     "            (None, false) => 3,", "            (None, false) => 2,"),
    ("18 an unnamed SELECT gets the named number", BLK,
     "            (None, true) => 7,", "            (None, true) => 4,"),
    ("19 a DO mismatch is reported with the SELECT numbers", BLK,
     "        Self::match_label(label, self.end_name(end), false, end_byte)?;",
     "        Self::match_label(label, self.end_name(end), true, end_byte)?;"),
    ("20 a mismatching name is accepted", BLK,
     "            (Some(label), _) if label == end_name => return Ok(()),",
     "            (Some(_), _) => return Ok(()),"),
    ("21 an omitted END name is rejected", BLK,
     "        let Some(end_name) = end_name else {\n            return Ok(());\n        };",
     "        let end_name = match end_name {\n            Some(name) => name,\n"
     "            None => return Err(ParseError::new(10, 3, end_byte)),\n        };"),

    # ---- 10.1, the END with nothing to close ----
    ("22 an END with nothing open is accepted", BLK,
     "        if !frame.kind.is_block() {",
     "        if false && !frame.kind.is_block() {"),
    ("23 an END on an OTHERWISE closes the OTHERWISE", BLK,
     "            Control::Otherwise => self.pop(),", "            Control::Otherwise => frame,"),

    # ---- 7.1 and 7.2 ----
    ("24 a SELECT with no WHEN is accepted", BLK,
     "        if whens.is_empty() {", "        if false && whens.is_empty() {"),
    ("25 a SELECT with no WHEN is reported against the END", BLK,
     "            let byte = self.instructions[select].clause_span.start;\n"
     "            return Err(ParseError::new(7, 1, byte));",
     "            return Err(ParseError::new(7, 1, end_byte));"),
    ("26 a non-WHEN inside a SELECT is accepted", BLK,
     "        if block.top().kind.is_select() && !(is_when || is_otherwise || is_end) {",
     "        if false && block.top().kind.is_select() && !(is_when || is_otherwise || is_end) {"),
    ("27 an END inside a SELECT trips the membership check", BLK,
     "&& !(is_when || is_otherwise || is_end) {", "&& !(is_when || is_otherwise) {"),
    ("28 the membership check does not run for an ELSE", BLK,
     "        if block.top().kind.is_select() && !(is_when || is_otherwise || is_end) {\n"
     "            return Err(ParseError::new(7, 2, byte));\n        }\n",
     "        if block.top().kind.is_select() && !(is_when || is_otherwise || is_end || is_else) {\n"
     "            return Err(ParseError::new(7, 2, byte));\n        }\n"),

    # ---- 8.2 and 9.2 ----
    ("29 an ELSE may follow a finished WHEN branch", BLK,
     "            if block.top().kind != Control::EndThen {",
     "            if !matches!(block.top().kind, Control::EndThen | Control::EndWhen) {"),
    ("30 an ELSE with no THEN is accepted", BLK,
     "            if block.top().kind != Control::EndThen {\n"
     "                return Err(ParseError::new(8, 2, byte));\n            }\n", ""),
    ("31 an OTHERWISE outside a SELECT is accepted", BLK,
     "            if !frame.kind.is_select() {\n                return Err(ParseError::new(9, 2, byte));\n            }",
     "            if !frame.kind.is_select() {\n                continue;\n            }"),

    # ---- 9.1 and the enclosing-SELECT search ----
    ("32 a WHEN outside a SELECT is accepted", INS,
     "                EnclosingSelect::None => return Err(self.error(9, 1)),",
     "                EnclosingSelect::None => InstructionKind::When {\n"
     "                    condition: self.logical(Terminators::IF, 929)?,\n"
     "                    false_target: None,\n                    exit: None,\n                },"),
    ("33 the enclosing-block search stops at the first frame", BLK,
     "            .find(|frame| frame.kind.is_block())", "            .find(|_| true)"),
    ("34 the enclosing-block search runs from the bottom", BLK,
     "            .iter()\n            .rev()\n            .find(|frame| frame.kind.is_block())",
     "            .iter()\n            .find(|frame| frame.kind.is_block())"),

    # ---- the SELECT CASE grammar ----
    ("35 a SELECT CASE WHEN uses the logical grammar", INS,
     "                EnclosingSelect::Plain => InstructionKind::When {",
     "                EnclosingSelect::Plain | EnclosingSelect::Case => InstructionKind::When {"),
    ("36 the case list raises the logical number", INS,
     "                    values: parse_case_when_list(self.ctx, &mut self.cursor, Terminators::IF)?,",
     "                    values: vec![self.logical(Terminators::IF, 929)?],"),

    # ---- the must-be-first checks ----
    ("37 EXPOSE may appear anywhere", INS,
     "                if !block.at_body_start() {\n                    return Err(self.error(99, 907));\n                }\n", ""),
    ("38 USE LOCAL gets the EXPOSE number", INS,
     "            return Err(self.error(99, 910));", "            return Err(self.error(99, 907));"),
    ("39 USE LOCAL may appear anywhere", INS,
     "        if !block.at_body_start() {\n            return Err(self.error(99, 910));\n        }\n", ""),
    ("40 a leading label does not break the must-be-first rule", BLK,
     "    pub(crate) fn at_body_start(&self) -> bool {\n        self.instructions.is_empty()\n    }",
     "    pub(crate) fn at_body_start(&self) -> bool {\n        self.instructions\n"
     "            .iter()\n            .all(|i| matches!(i.kind, InstructionKind::Label { .. }))\n    }"),

    # ---- the exposed-variable table and 99.913 ----
    ("41 a GUARD WHEN expression needs no exposed variable", INS,
     "                if !self.guard_exposes(&condition, block) {\n"
     "                    return Err(self.error(99, 913));\n                }\n", ""),
    ("42 an EXPOSE list exposes nothing", INS,
     "                    if let VariableRef::Direct(id) = name {\n"
     "                        block.expose(Box::from(self.ctx.symbols.name(*id).as_bytes()));\n"
     "                    }",
     "                    let _ = name;"),
    ("43 the indirect EXPOSE form exposes its name too", INS,
     "                    if let VariableRef::Direct(id) = name {",
     "                    if let VariableRef::Direct(id) | VariableRef::Indirect(id) = name {"),
    ("44 USE LOCAL does not invert the rule", BLK,
     "            return !local.iter().any(|n| n.as_ref() == name);",
     "            return local.iter().any(|n| n.as_ref() == name);"),
    ("45 an EXPOSE list is not consulted", BLK,
     "        if let Some(exposed) = &self.exposed {\n"
     "            return exposed.iter().any(|n| n.as_ref() == name);\n        }\n", ""),
    ("46 USE LOCAL seeds no special names", BLK,
     "            [b\"SUPER\".as_slice(), b\"SELF\", b\"RC\", b\"RESULT\", b\"SIGL\"]",
     "            [b\"\".as_slice()]"),
    ("47 USE LOCAL does not record its own names", INS,
     "            block.local_variable(Box::from(self.ctx.symbols.name(id).as_bytes()));\n", ""),
    ("48 a guard expression is not walked into its subterms", INS,
     "        let mut found = false;\n"
     "        condition.kind.for_each_child(&mut |child| {\n"
     "            found = found || self.guard_exposes(child, block);\n        });\n        found",
     "        false"),
    ("49 a compound contributes only its stem", INS,
     "                block.is_exposed(stem.as_bytes())\n"
     "                    || tails.iter().any(|tail| match tail {\n"
     "                        Tail::Variable(piece) => block.is_exposed(piece.as_bytes()),\n"
     "                        Tail::Constant(_) => false,\n                    })",
     "                block.is_exposed(stem.as_bytes())"),
    ("50 a compound contributes its whole name rather than its stem", INS,
     "                let (stem, tails) = compound_parts(name);\n"
     "                block.is_exposed(stem.as_bytes())",
     "                let (_stem, tails) = compound_parts(name);\n"
     "                block.is_exposed(name.as_bytes())"),

    # ---- the jump targets ----
    ("51 an IF's false target is off by one", BLK,
     "                    let target = self.next_index();\n"
     "                    self.set_false_target(parent, target);",
     "                    let target = self.next_index() + 1;\n"
     "                    self.set_false_target(parent, target);"),
    ("52 an ELSE's exit is off by one", BLK,
     "                    let target = self.next_index();\n"
     "                    self.set_then_exit(frame.index(), target);",
     "                    let target = self.next_index() - 1;\n"
     "                    self.set_then_exit(frame.index(), target);"),
    ("53 a WHEN's exit is the END rather than past it", BLK,
     "                    *exit = Some(end + 1);", "                    *exit = Some(end);"),
    ("54 a target past the end is kept rather than cleared", BLK,
     "                if target.is_some_and(|t| t >= len) {", "                if target.is_some_and(|t| t > len) {"),
    ("55 no target is resolved at all", BLK,
     "        self.resolve_targets();\n", ""),
    ("56 a block does not learn its END", BLK,
     "            InstructionKind::Do(body) | InstructionKind::Loop(body) => body.end = Some(end),\n"
     "            other => panic!(\"match_do_end on {other:?}\"),",
     "            InstructionKind::Do(_) | InstructionKind::Loop(_) => {}\n"
     "            other => panic!(\"match_do_end on {other:?}\"),"),
    ("57 a SELECT does not learn its END", BLK,
     "                *slot = Some(end);", "                let _ = slot;"),
    ("58 an END does not record which block it closed", BLK,
     "            InstructionKind::End { closes, .. } => *closes = Some(EndTarget { block, style }),",
     "            InstructionKind::End { closes, .. } => *closes = Some(EndTarget { block: 0, style }),"),

    # ---- the END style ----
    ("59 the two DO styles are swapped", BLK,
     "            (true, None) => EndStyle::Do,\n            (true, Some(_)) => EndStyle::LabeledDo,",
     "            (true, None) => EndStyle::LabeledDo,\n            (true, Some(_)) => EndStyle::Do,"),
    ("60 a loop's style respects its label", BLK,
     "            (false, _) => EndStyle::Loop,", "            (false, _) => EndStyle::LabeledDo,"),
    ("61 a SELECT with no OTHERWISE gets the OTHERWISE style", BLK,
     "            (false, _) => EndStyle::Select,", "            (false, _) => EndStyle::Otherwise,"),
    ("62 a labelled OTHERWISE is not distinguished", BLK,
     "            (true, true) => EndStyle::LabeledOtherwise,", "            (true, true) => EndStyle::Otherwise,"),

    # ---- the SELECT's own bookkeeping ----
    ("63 a WHEN joins its SELECT even when a branch is open", BLK,
     "            let frame = block.top();\n            if frame.kind.is_select() {",
     "            let frame = block.top();\n            if !frame.kind.is_select() {"),
    ("64 the OTHERWISE is not recorded on its SELECT", BLK,
     "                    otherwise: slot, ..\n                } => *slot = Some(otherwise),",
     "                    otherwise: slot, ..\n                } => *slot = None,"),

    # ---- the loop shape and the add/flush split ----
    ("65 a THEN goes through the ordinary dispatch", BLK,
     "        if matches!(instruction.kind, InstructionKind::Then) {",
     "        if false && matches!(instruction.kind, InstructionKind::Then) {"),
    ("66 a WHEN counts as a control instruction", BLK,
     "            InstructionKind::If { .. } | InstructionKind::Otherwise",
     "            InstructionKind::If { .. } | InstructionKind::Otherwise | InstructionKind::When { .. }"),
    ("67 an IF does not count as a control instruction", BLK,
     "    opens(kind).is_some()\n        || matches!(\n"
     "            kind,\n            InstructionKind::If { .. } | InstructionKind::Otherwise\n        )",
     "    opens(kind).is_some() || matches!(kind, InstructionKind::Otherwise)"),
    ("68 a plain DO opens a loop frame", BLK,
     "            LoopKind::Simple => Control::Do,", "            LoopKind::Simple => Control::Loop,"),
    ("69 SELECT CASE opens a plain SELECT frame", BLK,
     "        InstructionKind::Select { case: Some(_), .. } => Some(Control::SelectCase),",
     "        InstructionKind::Select { case: Some(_), .. } => Some(Control::Select),"),
    ("70 the pending-branch loop runs for an ELSE too", BLK,
     "        if !is_else {\n            while matches!(block.top().kind, Control::EndThen | Control::EndWhen) {",
     "        if true {\n            while matches!(block.top().kind, Control::EndThen | Control::EndWhen) {"),
    ("71 the pending-branch loop runs once rather than to exhaustion", BLK,
     "        if !is_else {\n            while matches!(block.top().kind, Control::EndThen | Control::EndWhen) {\n"
     "                block.pop();\n                block.flush_control(None);\n            }\n        }",
     "        if !is_else && matches!(block.top().kind, Control::EndThen | Control::EndWhen) {\n"
     "            block.pop();\n            block.flush_control(None);\n        }"),
    ("72 flushControl's ELSE arm stops instead of going around", BLK,
     "                    self.set_then_exit(frame.index(), target);\n"
     "                    // The C++ goes around again rather than breaking, so a stack",
     "                    self.set_then_exit(frame.index(), target);\n                    break;\n"
     "                    // The C++ goes around again rather than breaking, so a stack"),
    ("73 a closed block does not flush its pending branch", BLK,
     "            block.match_end(end, byte)?;\n"
     "            // The block that just closed may itself have been an IF's or a\n"
     "            // WHEN's instruction, so any pending branch is flushed now.\n"
     "            block.flush_control(None);\n", "            block.match_end(end, byte)?;\n"),
    ("74 an unclosed block at the end of a body is accepted", BLK,
     "            if top != Control::First {\n                return Err(block.block_error(top));\n            }\n", ""),
    ("75 the body-end branch cleanup does not run", BLK,
     "            while matches!(block.top().kind, Control::EndThen | Control::EndWhen) {\n"
     "                block.pop();\n                block.flush_control(None);\n            }\n"
     "            let top = block.top().kind;", "            let top = block.top().kind;"),
    ("76 a missing THEN at the end of a body is not reported", BLK,
     "            if let Some((which, byte)) = cursor.take_expected_then() {\n"
     "                return Err(ParseError::new(18, missing_then_sub(which), byte));\n            }\n", ""),

    # ---- the label table ----
    ("77 the last label of a name wins", BLK,
     "            self.labels.entry(name.clone()).or_insert(index);",
     "            self.labels.insert(name.clone(), index);"),
    ("78 a label index is off by one", BLK,
     "        let index = self.instructions.len();\n"
     "        if let InstructionKind::Label { name } = &instruction.kind {",
     "        let index = self.instructions.len() + 1;\n"
     "        if let InstructionKind::Label { name } = &instruction.kind {"),
]


def run():
    r = subprocess.run(
        ["cargo", "test", "--offline", "-p", "rexx-parse"],
        cwd=ROOT, capture_output=True, text=True)
    out = r.stdout + r.stderr
    failed = [l for l in out.splitlines() if l.startswith("test result: FAILED")]
    if failed:
        return failed[0]
    results = [l for l in out.splitlines() if l.startswith("test result:")]
    if results:
        return "ALL PASSED (%d suites)" % len(results)
    # No test ran at all, so the mutation broke the build. Checked last, because
    # a failing test also makes cargo print an `error:` line of its own.
    if "could not compile" in out or "error[" in out:
        return "DID NOT COMPILE"
    return "NO RESULT"


failures = []
for name, path, old, new in MUTATIONS:
    original = open(path).read()
    count = original.count(old)
    if count != 1:
        print("%-72s NOT APPLIED: pattern found %d times" % (name, count))
        failures.append(name)
        continue
    open(path, "w").write(original.replace(old, new))
    try:
        result = run()
    finally:
        open(path, "w").write(original)
    caught = result == "DID NOT COMPILE" or result.startswith("test result: FAILED")
    print("%-72s %s" % (name, result))
    if not caught:
        failures.append(name)

print()
print("BASELINE_after_revert".ljust(72), run())
print()
print("%d mutations, %d survived or were never applied" % (len(MUTATIONS), len(failures)))
for name in failures:
    print("  SURVIVED/UNAPPLIED: %s" % name)
sys.exit(1 if failures else 0)
```

---

## 10. The review round

Review verdict: spec compliant, code quality good with one correctness defect,
and it agreed that gate criterion 4 is not met. 0 Critical, 3 Important, 5 Minor.
Two of the three Importants were the coordinator's (the criterion-4 rewrite and
finding a home for the `resolveCalls` note). One was mine, and it was a real
defect in behaviour rather than in presentation.

### I1: the compound cache, and a claim of mine that was false

I wrote that "walking the finished tree is equivalent" to the C++'s
`captureGuardVariable` collection. It is not, and the counter-example is
perverse:

```
::method m / expose a. /            guard on when a.1   ->  rc 0
::method m / expose a. / say a.1 /  guard on when a.1   ->  99.913
```

An **earlier** reference to the same compound makes a **later** guard illegal.
`addCompound` (`LanguageParser.cpp:2124`) returns the cached retriever on a hit,
**before** reaching the `addStem` and `addSimpleVariable` calls that do the
capturing, so the second reference records nothing and the guard sees an empty
list.

This is almost certainly an upstream defect rather than a design. Both sibling
functions capture unconditionally and say why in a comment that names this exact
case:

```
// we need to always perform the capturing test because we allow e. g.
// USE LOCAL; GUARD ON/OFF WHEN v > 0
captureGuardVariable(varname, retriever);
```

`addSimpleVariable` (`:2069`) and `addStem` (`:2106`) both carry it.
`addCompound`'s early return defeats exactly what it describes. The oracle
defines behaviour, so it is reproduced.

**It ports, and no deviation needs recording.** 37 further probes, and the port
agreed with the oracle on every one at the first run.

Seventeen shapes for the rule itself:

| program | oracle |
|---|---|
| `expose a.` / `guard on when a.1` | rc 0 |
| `expose a.` / `say a.1` / `guard on when a.1` | **99.913** |
| `expose a.` / `guard on when a.1` / `say a.1` | rc 0 |
| `expose a.` / `guard on when a.1` / `guard on when a.1` | **99.913** |
| `expose a.` / `guard on when a.1 & a.1` | rc 0 |
| `expose a` / `say a` / `guard on when a` | rc 0 |
| `expose a.` / `say a.` / `guard on when a.` | rc 0 |
| `expose a.` / `say a.` / `guard on when a.1` | rc 0 |
| `expose a.` / `say a.2` / `guard on when a.1` | rc 0 |
| `expose i` / `guard on when a.i` | rc 0 |
| `expose i` / `say a.i` / `guard on when a.i` | **99.913** |
| `expose a.` / `a.1 = 1` / `guard on when a.1` | **99.913** |
| `expose a.` / `if a.1 = 1 then nop` / `guard on when a.1` | **99.913** |
| `expose a.` / `say a.1` / `::method n` / `expose a.` / `guard on when a.1` | rc 0 |
| `expose a. b.` / `say a.1` / `guard on when a.1 & b.1` | rc 0 |
| `use local x` / `say a.1` / `guard on when a.1` | **99.913** |
| `use local x` / `guard on when a.1` | rc 0 |

The rule is exact: a compound reference feeds a `GUARD ... WHEN` only on the
**first** occurrence of that exact spelling in the body. A simple variable and a
stem are unaffected, because their own paths capture unconditionally. The cache
is per body, because `initializeForTranslation` (`:1145`) rebuilds `variables`.

**How it is implemented, and why not at the funnels.** The faithful placement
would be the two points that correspond to `addText`/`addVariable`, which would
mean threading the cache through `expr.rs`. It is not needed, because the
intra-clause ordering turns out not to matter: within one guard expression the
first occurrence captures and a later duplicate is a no-op on a set, which
`guard on when a.1 & a.1` being rc 0 confirms. So the requirement reduces to a
per-body **set** of names referenced by instructions **already in the chain**,
which `Block::add_clause` can fill after each instruction is parsed. That is why
the guard never sees its own instruction, and it is why no signature outside
`block.rs` changed.

**Which slots feed it is narrower than "every symbol", and that is the part a
token scan could not do.** `end a.1` and `drop a.1` spell the symbol
identically. Twenty more shapes, both directions:

| slot | program | oracle |
|---|---|---|
| block name | `do label a.1` / `nop` / `end a.1` | rc 0 |
| block name | `leave a.1` | rc 0 |
| block name | `iterate a.1` | rc 0 |
| `SELECT` label | `select label a.1` | rc 0 |
| label | `signal a.1` | rc 0 |
| routine name | `call a.1` | rc 0 |
| environment | `address a.1` | rc 0 |
| trap label | `signal on syntax name a.1` | rc 0 |
| variable list | `drop a.1` | **99.913** |
| variable list | `expose a.1` | **99.913** |
| variable list | `procedure expose a.1` | **99.913** |
| variable list, indirect | `drop (a.1)` | **99.913** |
| `PARSE VAR` source | `parse var a.1 x` | **99.913** |
| `PARSE` target | `parse value 1 with a.1` | **99.913** |
| `USE ARG` target | `use arg a.1` | **99.913** |
| control variable | `do a.1 = 1 to 2` | **99.913** |
| expression | `numeric digits a.1` | **99.913** |
| expression | `interpret a.1` | **99.913** |
| command | `a.1` | **99.913** |
| message target | `a.1~string` | **99.913** |

`forward message a.1` is 35.1 for an unrelated reason and is not informative.

`for_each_variable_name` in `block.rs` encodes the split, with an exhaustive
`match` on `InstructionKind` so that a new variant fails to compile rather than
silently contributing nothing.

### The five Minors, all taken

* **`is_control` included `Otherwise`.** `RexxInstructionOtherwise` derives from
  `RexxInstruction`, not `RexxBlockInstruction`, and overrides only `isBlock`
  (`OtherwiseInstruction.hpp:49`, `:55`), so `isControl()` is false and the C++
  routes an `OTHERWISE` through `flushControl`. Behaviourally invisible, because
  the branch-end frames are already popped and `flushControl` only appends -- but
  the comment claimed the C++'s shape and was wrong. Now spelled the C++'s way,
  with the invisibility stated. No mutation is added for it, because one would be
  an equivalent mutant by construction.
* **`debug_assert!` became `assert!`.** The reviewer's point lands: I flagged that
  `Option` carries two meanings and that only an assertion separates them, and
  then chose the assertion that does not hold where it matters. `resolve_targets`
  has just walked the same list, so the cost is not worth trading the guarantee.
* **18.1 and 18.2 reported the wrong line when a directive ends the body.**
  Measured: `nop` / blank / `if 1 = 1` / blank / `::routine r` reports line **5**,
  the directive's, and substitutes line 3. `nextClause()` succeeds on the `::`
  clause, so `clauseLocation` moves there. At plain end of file the two coincide,
  which is why the original tests passed. Both spellings now asserted, and both
  directions have a mutation.
* **`directive/tests.rs` duplicated `lib.rs`'s body dispatch.** It now calls
  `directive_body`, so a directive kind that gains a body cannot be handled one
  way in the test and another in the entry point.
* **The 14.3/14.4 line-equality comment rested silently on Task 3.4's THEN
  split.** Now said.

### Mutations after the round: 90, all applied, all caught

Twelve added for the new code: the cache consulted and not consulted, the
registry unpopulated, a name slot wrongly feeding it, a variable slot wrongly
not, the indirect spelling dropped, the expression walk not recursing, and both
directions of the 18.x line.

Five existing patterns needed re-anchoring because the round moved the code they
matched, which the script's "NOT APPLIED" check caught rather than reporting them
as passes. One added mutation, "an instruction sees its own references", turned
out to be an **equivalent mutant**: the guard check completes before `add_clause`
runs at all, so the registration's position relative to the `push` is
unobservable. It was replaced rather than excused, by the indirect-spelling one
above -- which found a real gap, since `drop (a.1)` had not been measured.

### Still open, and not mine

The reviewer upheld both remaining concerns. Gate criterion 4's wording needs a
rewrite: it defines a parse error as "an input `rexxc` rejects", and the reviewer
found two of Task 3.6's programs that `rexxc` rejects with **98.903 "Unable to
load library"**, a post-translation load failure rather than a grammar rejection.
The parser is right to accept them, and the criterion as written says otherwise.
The `resolveCalls` note still needs a home in whichever task owns call
resolution.

---

## 11. Appendix: the review round's probes

### Batch K: the compound cache, seventeen shapes

```
=====  R1 compound: guard alone
--- program
::method m
expose a.
guard on when a.1
--- output (rexxc)
--- rc=0

=====  R2 compound: an earlier reference kills the guard
--- program
::method m
expose a.
say a.1
guard on when a.1
--- output (rexxc)
     4 *-* guard on when a.1
Error 99 running TMP/t.rex line 4:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  R3 compound: guard FIRST, then the reference
--- program
::method m
expose a.
guard on when a.1
say a.1
--- output (rexxc)
--- rc=0

=====  R4 simple variable: an earlier reference is harmless
--- program
::method m
expose a
say a
guard on when a
--- output (rexxc)
--- rc=0

=====  R5 stem: an earlier reference is harmless
--- program
::method m
expose a.
say a.
guard on when a.
--- output (rexxc)
--- rc=0

=====  R6 stem reference then a compound guard
--- program
::method m
expose a.
say a.
guard on when a.1
--- output (rexxc)
--- rc=0

=====  R7 a different compound name is a different cache entry
--- program
::method m
expose a.
say a.2
guard on when a.1
--- output (rexxc)
--- rc=0

=====  R8 two guards on the same compound
--- program
::method m
expose a.
guard on when a.1
guard on when a.1
--- output (rexxc)
     4 *-* guard on when a.1
Error 99 running TMP/t.rex line 4:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  R9 the same compound twice inside one guard
--- program
::method m
expose a.
guard on when a.1 & a.1
--- output (rexxc)
--- rc=0

=====  R10 compound with a variable tail, earlier reference
--- program
::method m
expose i
say a.i
guard on when a.i
--- output (rexxc)
     4 *-* guard on when a.i
Error 99 running TMP/t.rex line 4:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  R11 compound with a variable tail, guard alone
--- program
::method m
expose i
guard on when a.i
--- output (rexxc)
--- rc=0

=====  R12 an earlier reference as an assignment target
--- program
::method m
expose a.
a.1 = 1
guard on when a.1
--- output (rexxc)
     4 *-* guard on when a.1
Error 99 running TMP/t.rex line 4:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  R13 an earlier reference inside an IF condition
--- program
::method m
expose a.
if a.1 = 1 then nop
guard on when a.1
--- output (rexxc)
     4 *-* guard on when a.1
Error 99 running TMP/t.rex line 4:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  R14 the cache is per body
--- program
::method m
expose a.
say a.1
::method n
expose a.
guard on when a.1
--- output (rexxc)
--- rc=0

=====  R15 one exposed compound killed, another still live
--- program
::method m
expose a. b.
say a.1
guard on when a.1 & b.1
--- output (rexxc)
--- rc=0

=====  R16 use local, compound, earlier reference
--- program
::method m
use local x
say a.1
guard on when a.1
--- output (rexxc)
     4 *-* guard on when a.1
Error 99 running TMP/t.rex line 4:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  R17 use local, compound, guard alone
--- program
::method m
use local x
guard on when a.1
--- output (rexxc)
--- rc=0
```

### Batch L: which symbol slots feed the cache, twenty shapes

```
=====  S1 end name
--- program
::method m
expose a.
do label a.1
nop
end a.1
guard on when a.1
--- output (rexxc)
--- rc=0

=====  S2 leave name
--- program
::method m
expose a.
do label a.1
leave a.1
end a.1
guard on when a.1
--- output (rexxc)
--- rc=0

=====  S3 iterate name
--- program
::method m
expose a.
do label a.1
iterate a.1
end a.1
guard on when a.1
--- output (rexxc)
--- rc=0

=====  S4 select label
--- program
::method m
expose a.
select label a.1
when 1 = 1 then nop
end a.1
guard on when a.1
--- output (rexxc)
--- rc=0

=====  S5 signal label
--- program
::method m
expose a.
signal a.1
guard on when a.1
--- output (rexxc)
--- rc=0

=====  S6 call routine name
--- program
::method m
expose a.
call a.1
guard on when a.1
--- output (rexxc)
--- rc=0

=====  S7 address environment name
--- program
::method m
expose a.
address a.1
guard on when a.1
--- output (rexxc)
--- rc=0

=====  S8 drop list
--- program
::method m
expose a.
drop a.1
guard on when a.1
--- output (rexxc)
     4 *-* guard on when a.1
Error 99 running TMP/t.rex line 4:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  S9 expose list holds the compound itself
--- program
::method m
expose a.1
guard on when a.1
--- output (rexxc)
     3 *-* guard on when a.1
Error 99 running TMP/t.rex line 3:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  S10 parse value target
--- program
::method m
expose a.
parse value 1 with a.1
guard on when a.1
--- output (rexxc)
     4 *-* guard on when a.1
Error 99 running TMP/t.rex line 4:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  S11 parse var source
--- program
::method m
expose a.
parse var a.1 x
guard on when a.1
--- output (rexxc)
     4 *-* guard on when a.1
Error 99 running TMP/t.rex line 4:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  S12 use arg target
--- program
::method m
expose a.
use arg a.1
guard on when a.1
--- output (rexxc)
     4 *-* guard on when a.1
Error 99 running TMP/t.rex line 4:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  S13 do control variable
--- program
::method m
expose a.
do a.1 = 1 to 2
nop
end
guard on when a.1
--- output (rexxc)
     6 *-* guard on when a.1
Error 99 running TMP/t.rex line 6:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  S14 numeric digits expression
--- program
::method m
expose a.
numeric digits a.1
guard on when a.1
--- output (rexxc)
     4 *-* guard on when a.1
Error 99 running TMP/t.rex line 4:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  S15 interpret expression
--- program
::method m
expose a.
interpret a.1
guard on when a.1
--- output (rexxc)
     4 *-* guard on when a.1
Error 99 running TMP/t.rex line 4:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  S16 signal on name
--- program
::method m
expose a.
signal on syntax name a.1
guard on when a.1
--- output (rexxc)
--- rc=0

=====  S17 procedure expose list
--- program
expose a.
procedure expose a.1
guard on when a.1
--- output (rexxc)
     3 *-* guard on when a.1
Error 99 running TMP/t.rex line 3:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  S18 forward message
--- program
::method m
expose a.
forward message a.1
guard on when a.1
--- output (rexxc)
     3 *-* forward message a.1
Error 35 running TMP/t.rex line 3:  Invalid expression.
Error 35.1:  Incorrect expression detected at "A.1".
--- rc=221

=====  S19 a bare command clause
--- program
::method m
expose a.
a.1
guard on when a.1
--- output (rexxc)
     4 *-* guard on when a.1
Error 99 running TMP/t.rex line 4:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  S20 a message target
--- program
::method m
expose a.
a.1~string
guard on when a.1
--- output (rexxc)
     4 *-* guard on when a.1
Error 99 running TMP/t.rex line 4:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157
```

### Batch M: the indirect variable spelling

```
=====  indirect drop feeds the cache?
--- program
::method m
expose a.
drop (a.1)
guard on when a.1
--- output (rexxc)
     4 *-* guard on when a.1
Error 99 running TMP/t.rex line 4:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157

=====  indirect expose feeds the cache?
--- program
::method m
expose a.
expose (a.1)
guard on when a.1
--- output (rexxc)
     3 *-* expose (a.1)
Error 99 running TMP/t.rex line 3:  Translation error.
Error 99.907:  EXPOSE must be the first instruction executed after a method invocation.
--- rc=157

=====  direct drop for comparison
--- program
::method m
expose a.
drop a.1
guard on when a.1
--- output (rexxc)
     4 *-* guard on when a.1
Error 99 running TMP/t.rex line 4:  Translation error.
Error 99.913:  GUARD instruction did not include references to exposed variables.
--- rc=157
```

### Batch N: 18.1 and 18.2 when a directive ends the body

```
=====  18.1 terminated by a directive
--- program
nop

if 1 = 1

::routine r
nop
--- output (rexxc)
     5 *-* ::routine r
Error 18 running TMP/t.rex line 5:  THEN expected.
Error 18.1:  IF instruction on line 3 requires matching THEN clause.
--- rc=238

=====  18.2 terminated by a directive
--- program
nop

select

when 1 = 1

::routine r
nop
--- output (rexxc)
     7 *-* ::routine r
Error 18 running TMP/t.rex line 7:  THEN expected.
Error 18.2:  WHEN instruction on line 5 requires matching THEN clause.
--- rc=238

=====  18.1 at plain end of file
--- program
nop

nop

if 1 = 1
--- output (rexxc)
     5 *-* if 1 = 1
Error 18 running TMP/t.rex line 5:  THEN expected.
Error 18.1:  IF instruction on line 5 requires matching THEN clause.
--- rc=238
```

---

## 12. Appendix: the final mutation run

```
01 an unclosed DO gets the loop number                                   test result: FAILED. 233 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
02 an unclosed loop gets the block number                                test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
03 an unclosed SELECT gets the DO number                                 test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
04 an unclosed OTHERWISE gets the table position                         test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
05 an unclosed THEN and ELSE are swapped                                 test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
06 a block error is reported against the block, not the last instruction test result: FAILED. 235 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
07 a label in a DO gets the IF number                                    test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
08 a label in an IF gets the SELECT number                               test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
09 a label in a SELECT gets the DO number                                test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
10 a label after a finished WHEN branch is allowed                       test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
11 a label after a finished IF branch is rejected                        test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
12 a label at the top level is rejected                                  test result: FAILED. 221 passed; 16 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
13 no label placement is checked at all                                  test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
14 a label immediately before an ELSE is allowed                         test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
15 the two DO mismatch numbers are swapped                               test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
16 the two SELECT mismatch numbers are swapped                           test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
17 an unnamed DO gets the named number                                   test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
18 an unnamed SELECT gets the named number                               test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
19 a DO mismatch is reported with the SELECT numbers                     test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
20 a mismatching name is accepted                                        test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
21 an omitted END name is rejected                                       test result: FAILED. 202 passed; 35 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
22 an END with nothing open is accepted                                  test result: FAILED. 235 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
23 an END on an OTHERWISE closes the OTHERWISE                           test result: FAILED. 223 passed; 14 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
24 a SELECT with no WHEN is accepted                                     test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
25 a SELECT with no WHEN is reported against the END                     test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
26 a non-WHEN inside a SELECT is accepted                                test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
27 an END inside a SELECT trips the membership check                     test result: FAILED. 226 passed; 11 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
28 the membership check does not run for an ELSE                         test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
29 an ELSE may follow a finished WHEN branch                             test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
30 an ELSE with no THEN is accepted                                      test result: FAILED. 235 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
31 an OTHERWISE outside a SELECT is accepted                             test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
32 a WHEN outside a SELECT is accepted                                   test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
33 the enclosing-block search stops at the first frame                   test result: FAILED. 231 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
34 the enclosing-block search runs from the bottom                       test result: FAILED. 235 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
35 a SELECT CASE WHEN uses the logical grammar                           test result: FAILED. 235 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
36 the case list raises the logical number                               test result: FAILED. 235 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
37 EXPOSE may appear anywhere                                            test result: FAILED. 235 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
38 USE LOCAL gets the EXPOSE number                                      test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
39 USE LOCAL may appear anywhere                                         test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
40 a leading label does not break the must-be-first rule                 test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
41 a GUARD WHEN expression needs no exposed variable                     test result: FAILED. 230 passed; 7 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
42 an EXPOSE list exposes nothing                                        test result: FAILED. 231 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
43 the indirect EXPOSE form exposes its name too                         test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
44 USE LOCAL does not invert the rule                                    test result: FAILED. 235 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
45 an EXPOSE list is not consulted                                       test result: FAILED. 231 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
46 USE LOCAL seeds no special names                                      test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
47 USE LOCAL does not record its own names                               test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
48 a guard expression is not walked into its subterms                    test result: FAILED. 235 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
49 a compound contributes only its stem                                  test result: FAILED. 235 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
50 a compound contributes its whole name rather than its stem            test result: FAILED. 234 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
51 an IF's false target is off by one                                    test result: FAILED. 229 passed; 8 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
52 an ELSE's exit is off by one                                          test result: FAILED. 234 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
53 a WHEN's exit is the END rather than past it                          test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
54 a target past the end is kept rather than cleared                     test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
55 no target is resolved at all                                          test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
56 a block does not learn its END                                        test result: FAILED. 211 passed; 26 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
57 a SELECT does not learn its END                                       test result: FAILED. 220 passed; 17 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
58 an END does not record which block it closed                          test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
59 the two DO styles are swapped                                         test result: FAILED. 235 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
60 a loop's style respects its label                                     test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
61 a SELECT with no OTHERWISE gets the OTHERWISE style                   test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
62 a labelled OTHERWISE is not distinguished                             test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
63 a WHEN joins its SELECT even when a branch is open                    test result: FAILED. 220 passed; 17 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
64 the OTHERWISE is not recorded on its SELECT                           test result: FAILED. 234 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
65 a THEN goes through the ordinary dispatch                             test result: FAILED. 206 passed; 31 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
66 a WHEN counts as a control instruction                                test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
67 an IF does not count as a control instruction                         test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
68 a plain DO opens a loop frame                                         test result: FAILED. 233 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
69 SELECT CASE opens a plain SELECT frame                                test result: FAILED. 235 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
70 the pending-branch loop runs for an ELSE too                          test result: FAILED. 223 passed; 14 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
71 the pending-branch loop runs once rather than to exhaustion           test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
72 flushControl's ELSE arm stops instead of going around                 test result: FAILED. 234 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
73 a closed block does not flush its pending branch                      test result: FAILED. 232 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
74 an unclosed block at the end of a body is accepted                    test result: FAILED. 233 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
75 the body-end branch cleanup does not run                              test result: FAILED. 226 passed; 11 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
76 a missing THEN at the end of a body is not reported                   test result: FAILED. 234 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
77 the last label of a name wins                                         test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
78 a label index is off by one                                           test result: FAILED. 177 passed; 60 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
79 the compound cache is never consulted                                 test result: FAILED. 235 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
80 every compound is treated as already cached                           test result: FAILED. 234 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
81 nothing is ever recorded as referenced                                test result: FAILED. 235 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
82 the indirect variable spelling does not feed the cache                test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
83 a block name on an END feeds the cache                                test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
84 a DO/LOOP label feeds the cache                                       test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
85 a DROP or EXPOSE list does not feed the cache                         test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
86 a controlled loop's control variable does not feed the cache          test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
87 an expression's variable leaves are not collected                     test result: FAILED. 235 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
88 the expression walk does not recurse                                  test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
89 a missing THEN is always reported against the IF                      test result: FAILED. 236 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
90 a missing THEN is always reported against the terminating clause      test result: FAILED. 235 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s

BASELINE_after_revert                                                    ALL PASSED (7 suites)

90 mutations, 0 survived or were never applied
```
