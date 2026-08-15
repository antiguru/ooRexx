# Task 3.6 review — the 35 keyword instructions

## Verdicts

**Spec compliance: PASS.**
The brief's five steps are all done, and all three pre-flight rulings are honoured as written.
`ClauseCursor` lives in `clause.rs` beside `Clause`, `Instruction { kind, clause_span }` in `ast.rs` carries no `next` and no jump-target field, `split_before` takes two byte positions, and the `SELECT CASE` sub-number is threaded through `parse_logical`'s `missing` parameter rather than hard-coded.
Keyword recognition is positional and the assignment test precedes it.
The three exceptions to "the span includes its terminator" are implemented generally, not special-cased, and every span I measured against `trace r` matches the oracle byte for byte.

**Code quality: PASS with findings.**
No Critical.
Three Important, all of them handoff or test-coverage defects rather than wrong behaviour: the one documented deviation is produced by an untested guard whose removal silently accepts invalid input, the deferred-error list omits error 7.1, and the 99.913 note that 3.7b will act on is factually wrong about both the trigger and the mechanism.
Eight Minor.
Zero `unsafe`, no `chumsky`, no em-dashes in comments, no dead payload variants: every one of the 18 payload types and every enum variant I checked is constructed.

Independent verification I ran: 168 oracle probes across five fresh `mktemp -d` directories (never the scratchpad), 6 of the report's 16 mutations re-applied, 7 new mutations of my own, and ~120 parser probes through a temporary in-crate module that is now removed.
`git status` is clean and the 164 crate tests pass at `1d06d7c3`.

---

## The four adjudications

### 1. `SELECT CASE`'s `WHEN`: the interim state is safe, and 3.7b must change two things

Safe.
I measured both halves.
On the error path the interim number is wrong but the verdict is not: `select case 1` / `when then nop` is **35.934** in the oracle and **35.929** here, while plain `select` / `when then nop` is 35.929 in both, and `if , 1 then nop` is 35.929 in both.
Both reject, on already-invalid input, differing only in the sub-number — the same class as the accepted eager-scan deviation.
On the accept path `select case 1` / `when 1, 2 then nop` parses here into an AND of two conditions where the oracle builds a list of two case values.
That is not observable at parse time at all: both are `Ok`, the clause spans are identical (`"when 1, 2   "` in both, measured), and the difference first shows when Phase 4 evaluates the node.

What 3.7b must change, precisely:

* `instruction.rs:2538`, `self.logical(Terminators::IF, 929)` — 929 becomes a parameter so a `WHEN` whose enclosing block is a `SELECT CASE` gets 934.
* the node: `parseCaseWhenList` builds a case-value list where `parse_logical` builds an AND, so `InstructionKind::When { condition: Expr }` needs a second shape or a flag.

Both are reachable from one function, `if_instruction`, which is the "one call site" the ruling asked for.
The call site carries a comment naming 35.934 and the control stack, so a 3.7b implementer reading the code will find it.
One gap: **no test asserts the interim 35.929 for a `SELECT CASE` `WHEN`**, so nothing fails when 3.7b changes the number, and nothing flags it if 3.7b forgets.
Adding `assert_eq!(err("select case 1\nwhen then nop\nend"), (35, 929))` with a comment naming 35.934 as the target would make the handoff self-enforcing.

### 2. The `then: nop` divergence: your ruling is right, but the guard producing it is unprotected

I agree with the ruling.
Undoing 3.4's label split to recover one sub-number on input both implementations reject would trade a property verified across a million clauses for a cosmetic match, and hard-coding 35.1 without the path that produces it is worse than a named deviation.
I confirmed the oracle's own mechanism in `LanguageParser.cpp:1352`: `translateBlock` does `nextClause()` then `nextReal()` and tests `token->keyword() != KEYWORD_THEN` — the clause it gets still holds `then` and `:` together, so `thenNew` takes the `THEN` and the orphan `:` fails as 35.1.
Task 3.4 has already removed the colon, so there is nothing left to fail.
Both spellings behave the same way: `select` / `when 1 = 1` / `then: nop` is 35.1 in the oracle and 18.2 here.

**But see Important I1.** The single disjunct that keeps this a rejection rather than a silent acceptance has no test at all.

### 3. 99.913 is not local, and it is not a `translateBlock` error — the note handed to 3.7b is wrong twice

Deferring is correct.
The report's justification is not, and a 3.7b implementer acting on it will build the wrong check.

Measured, all four with fresh files:

| probe | oracle |
|---|---|
| `::method m` / `expose a` / `guard on when b` | **99.913** |
| `guard on when 1` in the main program, no method at all | **99.913** |
| `::routine r` / `guard on when 1` | **99.913** |
| `::method m` / `expose a b` / `guard on when a & b` | rc 0 |

So it is not "in a method" (the main program raises it), and it does not "read the method's whole `EXPOSE` list" (an `EXPOSE` list containing `a` does not save `guard on when b`).
The actual mechanism, from `InstructionParser.cpp:2634`-`2647` and `LanguageParser.cpp:2008`-`2029`: `guardNew` calls `setGuard()` to turn on variable capture, the expression parser's variable resolution calls `captureGuardVariable`, which adds a variable **only if `isExposed(name)`**, and `isExposed` consults `exposedVariables` (built by `EXPOSE`) or the complement of `localVariables` (built by `USE LOCAL`).
Zero captured variables is 99.913.

Two corrections follow.
First, the report's C3 opens "Every item is a `translateBlock` error in the C++, not a `nextInstruction` one" — false for this row: it is raised inside `guardNew`, a `nextInstruction` constructor.
Second, where it belongs is **not** the control stack.
It needs (a) a per-code-body exposed-variable set accumulated by an earlier `EXPOSE` or `USE LOCAL`, and (b) a capture hook inside the expression parser's variable resolution.
3.6 as designed genuinely cannot hold it — `parse_instruction(&ParseCtx, &mut ClauseCursor)` has nowhere mutable to accumulate a set — so deferring is right, but the owner is whoever introduces the per-body variable table, which is the same task that owes the `SIGNAL` label table.
If that is not 3.7b, this row is on the wrong list.
`instruction.rs:1554`-`1559`'s doc comment needs the same correction: it says "It reads the `EXPOSE` list of the whole method", which will not reproduce either of the first three rows above.

### 4. `end 1` / `end loop` parse: confirmed, nothing here rejects them

Confirmed in both directions.
`end_name` accepts any `Tag::Symbol` and only then checks that nothing follows, so a number, a stem and a compound are all legal block names.
Measured, the oracle agrees and all four are matching errors, not parse errors: `do` / `end 1` is 10.3, `do` / `end loop` is 10.3, `do` / `end a.` is 10.3, `do` / `end a.1` is 10.3, and `do` / `end a b` is 21.909 which **is** raised here.
The parser produces `["DO", "END"]` for the first four.
`instruction.rs:206` tells 3.7b that 10.1, 10.2 and 10.3 are its own, and `tests.rs:404` pins that `select` / `end 1` parses.

One gap worth closing: neither the deferred list nor any test states that a **number or a stem is a legal block name**, only that `end 1` parses.
A 3.7b implementer writing the `END` matcher could reasonably add a parse-time symbol-class check and reintroduce a rejection.
One sentence on `end_name` fixes it.

---

## Findings

### Critical

None.

### Important

**I1. The one documented deviation is produced by an untested disjunct, and removing it accepts invalid input silently.**

`instruction.rs:277`:

```rust
if parser.clause.label.is_some() || parser.first_keyword() != Some(KW_THEN) {
    return Err(parser.error(18, missing_then_sub(which)));
}
```

I deleted `parser.clause.label.is_some() ||` and **all 164 tests still passed**.
No test anywhere mentions this input: `grep "then:"` in `tests.rs` finds only the bare-label case at line 308.

Failure scenario.
Input:

```rexx
if 1 = 1
then: nop
```

Oracle: rc 221, Error 35.1 — rejected.
With the disjunct present: 18.1 — rejected, the accepted deviation.
With the disjunct removed: **`Ok(["THEN", "NOP"])`** — accepted, and the label `then` is silently discarded, because `first_keyword()` reads the label clause's own `then` token and the THEN arm then throws the `Clause::label` away.
Right output is a rejection, either number.

Fix: pin both spellings, with the oracle's number in the comment so the test doubles as the deviation record.

```rust
// A named deviation: the oracle is 35.1 here, because its clause still holds
// `then` and `:` together and the orphan colon is what fails. Task 3.4 has
// already split the colon off, so the missing-THEN error fires instead. Both
// reject; only the number differs.
assert_eq!(err("if 1 = 1\nthen: nop"), (18, 1));
assert_eq!(err("select\nwhen 1 = 1\nthen: nop\nend"), (18, 2));
```

**I2. The deferred-error list omits error 7.1, a distinct `translateBlock` error that 3.6 accepts.**

`instruction.rs:203` lists "7.2, an instruction other than `WHEN`/`OTHERWISE`/`END` inside a `SELECT`" and nothing else in the 7 family.
Measured, 7.1 is a separate error with a separate trigger — a `SELECT` that never gets a `WHEN` at all:

| input | oracle |
|---|---|
| `select` / `end` | rc 249, **7.1** "SELECT on line 1 requires WHEN" |
| `select case 1` / `end` | rc 249, **7.1** |
| `select case 1` / `otherwise nop` / `end` | rc 249, **7.1** |
| `select` / `nop` / `end` | rc 249, 7.2 "requires WHEN, OTHERWISE, or END" |

Failure scenario.
Input `select` newline `end`.
Wrong output: 3.6 returns `Ok(["SELECT", "END"])` — correct for this task — and 3.7b, working from the list, implements only 7.2 and returns `Ok` for the whole program.
Right output: rc 249, 7.1.
Fix: add the row, worded as "7.1, a `SELECT` or `SELECT CASE` that closes with no `WHEN` in it, which `OTHERWISE` alone does not satisfy."

**I3. The 99.913 handoff note is wrong about both the trigger and the mechanism.**

Detailed under adjudication 3.
Failure scenario.
A 3.7b implementer follows `instruction.rs:1554`-`1559` ("It reads the `EXPOSE` list of the whole method") and implements "if a method has an `EXPOSE` and a `GUARD ON WHEN`, accept; otherwise 99.913".
Wrong output: `::method m` / `expose a` / `guard on when b` returns `Ok` where the oracle is 99.913, and `guard on when 1` in the main program returns `Ok` where the oracle is 99.913.
Right output: 99.913 for both.
The correct rule is "the `GUARD ON WHEN` expression must reference at least one variable that `isExposed` at that point", where `isExposed` is membership in the `EXPOSE` set or non-membership in the `USE LOCAL` set, and it applies in every code body, not only methods.

### Minor

**M1. `parse_instructions`' 18.x byte offset is right by coincidence in one path and its doc comment describes the wrong rule.**

Measured, the oracle reports 18.1 against the **offending clause's** line, not the `IF`'s: for `nop` / `nop` / `if 1 = 1` / `nop` / `nop` it prints "line 4" with the substitution naming line 3.
`parse_instruction`'s `parser.error(18, ...)` uses the current clause's byte, which is correct.
`parse_instructions:234` instead uses `out.last().clause_span.start`, documented as "against the IF's own location".
That is right whenever the `IF` is the last instruction, which is every end-of-body case, but wrong for one shape: `nop` / `if 1 = 1` / `::method m` reports line 3 in the oracle (the `::` clause) and would report line 2 here.
Within the stated "plausible line" tolerance, so behaviour is fine; the comment should say the oracle reports against the clause that failed to be a `THEN`, and that this path falls back to the `IF` because the `::` clause is never parsed.

**M2. The mutation script reproduced in the report does not reproduce mutation 12.**

Its pattern `") => is_call,"` matches **zero** times against the committed text, because `cargo fmt` reflowed the arm to put `is_call` on its own line.
The report says in prose that this happened and that the mutation was re-run against the reflowed text, but ships the stale pattern, so anyone re-running the script gets 15 results and one `SKIPPED`.
I re-ran it with `"                is_call\n            }"` and got 1 test failed, matching the report's count.
Ship the pattern that ran.

**M3. Index-constant declaration order does not follow the tables it indexes.**

`KW_OTHERWISE: usize = 19` is declared before `KW_OPTIONS: usize = 18` (`instruction.rs:83`-`84`), and `SUB_FOREVER = 18` is declared after `SUB_FUZZ = 20` (`:148`-`151`).
Both tables are alphabetical, so a reader scanning for a gap cannot rely on the constant list being monotonic.
`keyword_indices_still_name_their_own_spellings` makes an error impossible, so this is readability only.

**M4. Four structuring semicolons in comments, against the project rule.**

`instruction.rs:35`, `:276`, `:383`, and `clause.rs:217`.
The last is copied verbatim from the brief, so only three are the implementer's.

**M5. Tests are in `src/instruction/tests.rs`, not the brief's `rust/crates/rexx-parse/tests/instruction.rs`.**

Justified in the module header — `ParseCtx`, `ClauseCursor` and `parse_instruction` are all `pub(crate)` and an integration test is a separate crate — and consistent with `expr/tests.rs` from 3.5.
Note only; the brief's file list should be corrected rather than the code.

**M6. `the_corpus_programs_parse`'s assertion is near-tautological and its comment overstates what it guards.**

`assert!(instructions.len() >= clauses.len())` with "A parse that stopped at the first clause would still be `Ok`".
`parse_instructions` is a `while let Some(clause) = cursor.peek()` loop whose only early exit is a `::` clause, so on a directive-free corpus it cannot stop early and still return `Ok`.
The assertion does catch a stray directive; the comment should say that.

**M7. `InstructionKind::Assignment { target: SymbolId }` drops the `SymbolClass` that `ExprKind` keeps in its variant.**

`a. = 1`, `a.b = 1` and `a = 1` produce indistinguishable payloads without re-deriving the class from the spelling.
Recoverable (`need_variable` has already narrowed it to Variable, Stem or Compound, and the three are distinguishable by the periods), and consistent with `ExprKind::Compound` deliberately not storing its tail decomposition.
A choice, not a defect — recorded because it is an asymmetry a Phase 4 reader will notice.

**M8. `required_end(&mut self, ...)` never mutates.** `&self` would do.

---

## Mutation testing

**6 of the report's 16 re-applied, all 6 reproducing the report's failure count exactly.**

| mutation | report | mine |
|---|---|---|
| 2 the variable list uses the spelling gate, not the class gate | 1 | **1** |
| 3 an `IF` clause ends at the END of its terminator | 4 | **4** |
| 4 a split that consumes the clause does not narrow its span | 2 | **2** |
| 5 an `IF` does not record that a `THEN` is expected | 13 | **13** |
| 7 keyword recognition runs before the assignment test | 3 | **3** |
| 12 `CALL ON` accepts every condition `SIGNAL ON` does | 1 | **1** (after fixing M2's pattern) |

Seven of my own, targeting the rulings the report's set did not:

| mutation | tests failed |
|---|---|
| N1 the pending-`THEN` flag is never cleared (`peek` for `take`) | 12 |
| N2 a label clause satisfies a pending `THEN` | **0 — see I1** |
| N3 `IF`/`WHEN` pass the `THEN` token's END, like `THEN`/`ELSE` do | 4 |
| N4 the message test runs before the assignment test | 0, and behaviourally equivalent |
| N5 `OTHERWISE` passes the clause end, not the keyword end | did not compile |
| N6 a misplaced `THEN` is accepted rather than 8.1 | 1 |
| N7 `END` accepts a non-symbol block name | 1 |

N4 is not a hole.
`message_term` returns `Some` only when a `~`, `~~` or `[` was applied, and `assignment` matches only when token 1 is `=`, `==` or an assignment operator, so whenever `assignment` would match, `message` has already returned `None` — the two orders are indistinguishable on every input.
The C++ order is preserved anyway, which is right.

---

## What I verified directly, and it all matched

**Positional keyword recognition.**
`keyword_as_variable.rex` parses with 19 assignments while `DO`, `SELECT`, `WHEN`, `OTHERWISE`, `IF`, `THEN`, `ELSE`, `PARSE` and `SAY` still work as keywords in the same file, and the committed test pins that count.
Beyond the corpus I probed 30 shapes the corpus does not cover, all rc 0 in the oracle and all correct here: a keyword as a stem (`end. = 0`, `end.1 = 7`, `if. = 1`, `if.1 = 2`, `say. = 1`), as a compound tail (`x.if = 1`), as a label (`if: nop`, `say: nop`, `"then": nop`), as the target of `DROP` (`drop if`, `drop do end select`), and as an operand of `EXPOSE`, `PARSE ARG`, `PARSE VAR`, `LEAVE`, `ITERATE`, `END`, `SIGNAL`, `CALL`, `USE ARG`, `NUMERIC DIGITS`, `TRACE VALUE`, `ADDRESS`, `SELECT CASE`, `DO LABEL` and a controlled `DO`.
`then = 1`, `else = 1`, `when = 1`, `otherwise = 1` and `nop = 1` are all assignments, so the assignment test genuinely precedes keyword lookup.
Leading indentation does not break it: `  a = 1` is an assignment and its span excludes the indent, matching `trace r`.

**The three span exceptions, measured against `trace r`.**
Every one identical to the oracle, byte for byte:

| input | oracle span | here |
|---|---|---|
| `if 1 = 1   then    say "a"` | `if 1 = 1   ` / `then` / `say "a"` | same |
| `if 1 = 1;` newline `then say "a"` | `if 1 = 1` / `then` / `say "a"` | same |
| `if 1 = 1   ;` newline `then nop` | `if 1 = 1   ` | same |
| `if 1 = 1   ` newline `then nop` | `if 1 = 1   ` | same |
| `if 1 = 1 -- c` newline `then nop` | `if 1 = 1 -- c` | same |
| `when 1 = 1   then    say "a"` | `when 1 = 1   ` / `then` / `say "a"` | same |
| `when 1 = 1;` newline `then nop` | `when 1 = 1` | same |
| `if 1 then   ` newline `nop` | `if 1 ` / `then` / `nop` | same |
| `otherwise    nop` | `otherwise` / `nop` | same |
| `if 0 then nop; else    nop` | `if 0 ` / `else` / `nop` | same |
| `if 1 then nop; else nop` | `if 1 ` / `then` / `nop;` | same |
| `nop;` | `nop;` | same |
| `here: ; nop` | `here:` / `nop` | same |
| `select case 1` / `when 1, 2   then nop` | `when 1, 2   ` | same |

So rule 2's ordinary case keeps its `;`, the label colon leaves the `;` in no clause, and `IF`/`WHEN` end at the start of their terminator in both spellings — including the cases the brief did not name, where the terminator is a line end after trailing blanks or after a comment.
`WHEN` matches `IF` everywhere.

**`split_before`'s two positions.**
For `if 1 = 1   then    say "a"` the condition is bytes 0..11 with all three blanks, `then` is 11..15 with none on either side despite four following, and `say "a"` starts at 19.
Bytes 15..19 belong to no clause.
Both `end_at` values in the brief are what the code passes, and mutation N3 (make `IF` pass the keyword's end instead of its start) fails 4 tests.

**The one-bit flag.**
It buys exactly 8.1, 18.1 and 18.2 and nothing else: `take_expected_then` has two readers, `parse_instruction`'s prologue and `parse_instructions`' epilogue, and both produce only error 18.
Every direction matches the oracle — `then` alone, `nop`/`then`, `if 1 then nop`/`then`, `if 1 then`/`then`, `if 1 then nop`/`else then` are all 8.1; `if 1 = 1`/`nop`, `if 1 = 1` alone, `if 1 = 1`/`"echo hi"`, `if 1 = 1`/`::method m` are all 18.1; the `WHEN` spellings are all 18.2.
It cannot get stuck: `take_expected_then` runs once per clause unconditionally, and mutation N1 (never clear it) fails 12 tests.
Nesting works: `if 1 then if 2 then nop` and `if 1 then`/`if 2 then`/`nop` both give five instructions with the right spans, and `if x then then nop` is 8.1 in both.

**The reachability table.**
35 rows, 35 distinct keywords, length asserted, and every source rc 0 under `rexxc`.
`procedure`, `leave` and `iterate` stand alone unwrapped, exactly as the brief requires, and all three produce their nodes here.

**Error numbers I re-derived rather than trusting.**
`CALL ON`/`OFF` and `SIGNAL ON`/`OFF` across all 11 conditions and both directions — 44 oracle runs.
The implementation's `rejected` table matches every cell: `PROPAGATE` rejected by both, `SYNTAX`/`NOVALUE`/`LOSTDIGITS`/`NOMETHOD`/`NOSTRING` rejected by `CALL` only, `ANY`/`ERROR`/`FAILURE`/`HALT`/`NOTREADY` accepted by both.
`TRACE`'s letter set is `ACEFILNOR` per the oracle's own message text, and `check_trace_setting` accepts exactly those nine.
`TRACE_DIGITS = 9` is right at the boundary: `trace 999999999` and `trace 1e8` are rc 0, `trace 1e9` and `trace 1234567890` are 24.1.
Twenty more numbers I checked and did not find in the report's table are all correct here: `address system with "x"` 20.933, `address system with input "f"` 25.933, `raise "syntax" 1` 20.914, `use` 25.905, `use strict` 25.929, `forward class 1 class 2` 25.921, `raise error 1 description "d" description "e"` 25.908, `nop 1` 21.901, `use arg 1+1` 31.2, `drop a~b` 20.901.
Nine comma-and-ordering shapes that could have hit the 49.2 fidelity arm by accident are rc 0 in both: `do i = 1, 2`, `do i over x, y`, `do while 1, 2`, `do with index i over x, y`, `do i = 1 to 2, 3`, `do i = 1 while 1 to 2`, `do i over x while 1 for 2`, `parse value 1, 2 with a`, `do i = 1 to 2 by 3 for 4 while 1`.
`then(1)` traces as `then` then `(1)` in the oracle and produces `["IF","THEN","<command>"]` with the same spans here.

---

## Not verifiable from the diff

* **Step 2's ordering** — whether one failing test per family was written before that family was implemented. Each commit contains its family's code and tests together, so the diff cannot show which came first.
* **The uncommitted breadth check** over `samples/` and `interpreter/RexxClasses/` (314 files, 29,492 of 33,840 clauses, 0 failures). The temporary `lib.rs` helper it needed was removed, so nothing in the tree reproduces it. The claim is plausible and consistent with what I did check, but it rests on the implementer's word.
* **Ten of the report's 16 mutations** — 1, 6, 8, 9, 10, 11, 13, 14, 15 and 16. I re-applied 2, 3, 4, 5, 7 and 12; all six matched. The pattern-drift problem in M2 means the reproduced script cannot be trusted to re-run the other ten unmodified either.
* **C4's probe-harness slip** — that the five `INTERPRET` run-time probes reused `p19-rt.rex`. Self-reported and not checkable after the fact; the five results are internally consistent and I confirmed all six `SourceKind::Interpret` rejections (47.1, 99.908, 99.915, 99.924, 99.912, 99.923) are produced by the committed code.
* **The 397 workspace tests, clippy, fmt and the dead-code gate grep** — verified by the coordinator, not re-run here. I did re-run the 164 crate tests before and after every mutation.
