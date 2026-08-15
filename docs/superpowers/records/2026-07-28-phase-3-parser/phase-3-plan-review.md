# Phase 3 parser plan — review

Target: `docs/superpowers/plans/2026-07-28-phase-3-parser.md`
Reviewed: 2026-07-28. Oracle: `build/bin/rexx` in this worktree.

Every finding below is tagged **VERIFIED** (I ran it or read the file) or
**PLAUSIBLE**. Probe files were written under `/tmp`.

---

## Verdict

**Not fit to execute as written.** The plan is a large improvement on a
listing-derived plan — its keyword extraction is exact, its
keywords-are-not-reserved analysis is right and important, and its
differential-set count is more accurate than `phase-2-gate.md` — but four
Critical defects would each cost real rework, and one of them is a repeat of
the exact Phase 2 gate failure the plan claims to have fixed.

Fix C1–C4 and I1–I3 before execution. The rest can be folded in as edits.

---

## 1. Critical

### C1 — Task 3.8 Step 1: the ground-truth recipe does not work

**VERIFIED.** The plan's recipe is:

```rexx
signal on syntax name oops
/* the malformed construct */
exit 0
oops: say "rc=" rc; say condition('o')~message
```

ooRexx parses the whole file before executing anything, so a syntax error in
the file aborts before `signal on syntax` is ever installed. Running
`/tmp/trap.rex` (a bad expression inside a subroutine, trap on line 1):

```
     5 *-* say 1 +
Error 35 running /tmp/trap.rex line 5:  Invalid expression.
Error 35.1:  Incorrect expression detected at "+".
rc=221
```

The trap did not fire. Nothing was captured through `condition('o')`.

Two recipes that do work, and the plan must pick one and state its limits:

- **(a) Run the bad file, capture stderr.** Gives number, sub-number, line and
  the two message texts, in the shape above. This is the only route that gives
  the *file's* line number.
- **(b) `INTERPRET` a fragment inside an installed trap.** Verified working:

  ```rexx
  call tryit
  exit 0
  tryit:
    signal on syntax name oops
    interpret "say 1 +"
    return
  oops: say condition('o')~code; return     /* prints 35.1 */
  ```

  Limitation the plan must record: `condition('o')~position` is the line of
  the `INTERPRET` instruction, not a position inside the fragment.

The team lead's briefing note ("a label is not allowed inside a DO block, so
put `signal on syntax` traps in a subroutine") is a separate, valid point about
label placement; it does not rescue this recipe.

### C2 — Exit gate criterion 3 and Task 3.8 Step 4: the oracle emits no column

**VERIFIED.** Full enumeration of the condition object at a syntax error
(`/tmp/cond.rex`, iterating `c~allIndexes~sort`):

```
ADDITIONAL = +          CODE = 35.1        CONDITION = SYNTAX
DESCRIPTION =           ERRORTEXT = Invalid expression.
INSTRUCTION = SIGNAL    MESSAGE = Incorrect expression detected at "+".
PACKAGE = a Package     POSITION = 5       PROGRAM = /tmp/cond.rex
PROPAGATED = 0          RC = 35            STACKFRAMES = a List
TRACEBACK = a List
```

`POSITION` is the **line**. There is no column field, and none appears in the
stderr format either. ooRexx locates an error by quoting the offending token
(`ADDITIONAL = +`), not by offset. `interpreter/messages/errnums.xml`
substitutions are line numbers and symbol names, never columns.

So:

- Exit gate: *"For every syntax error the parser raises: number, sub-number,
  line **and column** match `build/bin/rexx`, verified by provoking each one."*
- Task 3.8 Step 4: *"Verify position, not just number. Column numbers are the
  easiest thing to get subtly wrong…"*

Neither can be assessed. There is nothing on the oracle side to compare a
column against. This is structurally identical to the Phase 2 gate defect —
a criterion written against an interface that does not exist — and it is the
one thing the plan's Notes section most loudly promises it has avoided.

**Should say:** match **number, sub-number, line, and the substitution values**
(`ADDITIONAL`, and the rendered `MESSAGE`), all four observable. Keep an
internal column for the Rust diagnostic if wanted, but do not gate on it and
do not claim it is verified against the interpreter.

### C3 — Task 3.6 / 3.7 interfaces cannot produce what the tasks must produce

Both signatures take a single clause:

- Task 3.6: `parse_instruction(&Clause) -> Result<Instruction, ParseError>`
- Task 3.7: `parse_directive(&Clause) -> Result<Directive, ParseError>`

Eight of the eleven control-flow keywords the plan itself lists — `DO`/`END`,
`IF`/`THEN`/`ELSE`, `SELECT`/`WHEN`/`OTHERWISE` — span many clauses, and a
`::method` or `::routine` body is an arbitrary run of clauses following the
directive clause. Neither can come out of a one-clause function. Both need a
clause cursor/stream.

Underneath that sits an unanswered design question the plan never poses:
**is the AST a tree, or a flat instruction chain with block links?** The C++
builds a chain (`RexxInstruction` with a `nextInstruction` link, DO/END matched
through a parse-time control stack and patched at `END`), which is why
`ThenInstruction`, `ElseInstruction`, `EndInstruction` and
`OtherwiseInstruction` are all instruction classes in their own right — 52 of
them (**VERIFIED**, `ls interpreter/instructions/*.hpp | wc -l` = 52). The
parent plan's D13 note says "plain owned Rust data **inside one arena object
per code body**" (line 2368); the Phase 3 plan drops the arena half (see M7).
`Program { instructions: Vec<Instruction> }` reads either way, and Phase 4's
dispatch loop shape follows directly from the answer. Decide it in the plan.

### C4 — Three instruction kinds are assigned to no task

**VERIFIED.** `interpreter/instructions/` contains, beyond the 35 keyword
instructions: `AssignmentInstruction`, `CommandInstruction`,
`MessageInstruction`, `LabelInstruction`, plus `UseLocalInstruction`,
`WhenCaseInstruction` and `AddressWithInstruction`. `Token.hpp:207–213` has
`KEYWORD_ASSIGNMENT`, `KEYWORD_COMMAND`, `KEYWORD_MESSAGE`, `KEYWORD_LABEL`.

The **command** clause is the *default* clause type: any clause that is not a
keyword instruction, an assignment, a label or a standalone message send is an
expression that is evaluated and issued to the current `ADDRESS`. Task 3.6 is
scoped as "the 35 keyword instructions" and its Step 4 asserts only that the 35
are reachable, so the single most common clause form in shell-driving Rexx —
and the one that catches every parser fall-through — has no task, no test and
no AST node named anywhere in the plan.

The plan does note in passing that "a clause whose second token is `=` is an
assignment", but only as an argument about keyword recognition; no task
produces an `Assignment` node.

---

## 2. Important

### I0 — The corpus program the plan tells you to add already exists

**VERIFIED.** The plan states, twice:

- Task 3.6 Step 2: *"the corpus will not catch the mistake —
  `rust/corpus/lang/` contains no program that uses a keyword as a variable.
  Add one."*
- Notes carried in: *"The L0 corpus contains no program that exercises it,
  which is exactly why it would go unnoticed."*

`rust/corpus/lang/keyword_as_variable.rex` exists, is 63 lines, and covers the
case more thoroughly than the plan's own examples: 15 keywords as variables,
`if if = 2 then …`, `DO` looping while `do` holds a value, `SELECT`/`WHEN`/
`OTHERWISE` while all three are variables, a stem named `end.`, a keyword as a
compound tail (`stem.if`), `PARSE` while `parse` is a variable, and a label
spelling a keyword. It runs clean under `build/bin/rexx`.

The reasoning around this claim is right; only the claim is wrong. It should
say the corpus already pins this and cite the file as the acceptance test.

### I1 — The LOC table total is wrong

**VERIFIED.** The five row values are each exact. Their sum is **14,638**:

```
4650 InstructionParser.cpp   4398 LanguageParser.cpp   2867 DirectiveParser.cpp
1955 Scanner.cpp              768 ProgramSource.cpp
```

The table's total row says **17,483**. That figure is `wc -l
interpreter/parser/*.cpp interpreter/parser/*.hpp` — the whole directory,
including `Clause.cpp`, `Token.cpp`, `KeywordConstants.cpp` and the headers —
which is what the parent plan's line 55 counts. The Phase 3 table presents a
directory total as the sum of five files.

### I2 — "Do not sort the keyword table" is backwards

**VERIFIED.** Task 3.6 says: *"The table is not alphabetical and the C++
indexes it by position — a lesson already paid for in Phase 0 with the builtin
table. Do not sort it."* Repeated in Notes.

`keywordInstructions[]` (`KeywordConstants.cpp:66–103`) is strictly
alphabetical: ADDRESS, ARG, CALL, DO, DROP, ELSE, END, EXIT, EXPOSE, FORWARD,
GUARD, IF, INTERPRET, ITERATE, LEAVE, LOOP, NOP, NUMERIC, OPTIONS, OTHERWISE,
PARSE, PROCEDURE, PULL, PUSH, QUEUE, RAISE, REPLY, RETURN, SAY, SELECT, SIGNAL,
THEN, TRACE, USE, WHEN.

`RexxToken::resolveKeyword` (`KeywordConstants.cpp:417`) is a **binary search**
over the table — sortedness is a correctness requirement, not an accident —
and the code comes from `table[middle].keywordCode`, stored explicitly in each
`KeywordEntry`, never from the index. The Phase 0 builtin-table lesson does not
transfer to this table. (`builtinFunctions[]` is likewise sorted, with an
`#ifdef EBCDIC` variant reordering for EBCDIC collation.)

As written the plan asserts a false fact about the reference code and pins a
design instruction to it.

### I3 — Task 3.7 omits two directives, one of which needs a scanner mode

**VERIFIED.** `directives[]` (`KeywordConstants.cpp:52–63`) has **nine**
entries: ANNOTATE, ATTRIBUTE, CLASS, CONSTANT, METHOD, OPTIONS, REQUIRES,
RESOURCE, ROUTINE.

The plan lists seven: `::class`, `::method`, `::routine`, `::requires`,
`::attribute`, `::constant`, `::options`. Missing `::ANNOTATE` and
`::RESOURCE`. Both parse under the oracle today (probed `/tmp/res.rex`,
`/tmp/ann.rex`, both rc=0).

`::RESOURCE` matters disproportionately: its body is **raw text up to
`::END`**, not clauses. That is a scanner mode the plan has nowhere, and a
parser that tokenises a resource body as Rexx will fail on arbitrary content.

### I4 — The sub-keyword tables are invisible, so Task 3.6 is under-scoped

**VERIFIED** entry counts in `KeywordConstants.cpp`:

| table | entries |
|---|---|
| `directives` | 9 |
| `keywordInstructions` | 35 |
| `subKeywords` | 50 |
| `builtinFunctions` | 162 |
| `conditionKeywords` | 12 |
| `parseOptions` | 10 |
| `subDirectives` | 40 |

`parseOptions`: ARG CASELESS LINEIN LOWER PULL SOURCE UPPER VALUE VAR VERSION.
`subDirectives`: ABSTRACT ALL ATTRIBUTE CLASS CONDITION CONSTANT DELEGATE
DIGITS END ERROR EXTERNAL FAILURE FORM FUZZ GET GUARDED INHERIT LIBRARY
LOSTDIGITS METACLASS METHOD MIXINCLASS NAMESPACE NOPROLOG NOSTRING NOTREADY
NOVALUE NUMERIC PACKAGE PRIVATE PROLOG PROTECTED PUBLIC ROUTINE SET SUBCLASS
SYNTAX TRACE UNGUARDED UNPROTECTED.

The plan sizes Task 3.6 entirely by "35 keywords" and mentions none of the
other 112 option words. Those are what most of `InstructionParser.cpp`'s 4,650
lines is: `DO … TO … BY … FOR … WHILE … UNTIL … OVER`, `NUMERIC DIGITS/FUZZ/
FORM`, `SIGNAL ON … NAME`, `USE ARG/STRICT/NAMED`, `GUARD ON/OFF WHEN`,
`RAISE … DESCRIPTION/ADDITIONAL/ARRAY/RETURN/EXIT`. Task 3.6 as written reads
like five days of work and is closer to the whole phase.

### I5 — Task 3.7b Step 3 states a false behaviour as fact

**VERIFIED.** The plan: *"The first `::` directive ends the main instruction
stream. Confirm that against `build/bin/rexx` rather than assuming it — a
trailing instruction after a directive is a syntax error with a specific
number, and that number belongs in Task 3.8's table."*

```rexx
say 1
::routine r
  return 2
say 3
```

Runs, prints `1`, rc=0. There is no error: `say 3` becomes part of `r`'s body,
and `r` is never called. The first half of the claim is right; the "syntax
error with a specific number" is not, and Task 3.8 is instructed to record a
number that does not exist.

### I6 — Task 3.2's failing test contradicts its own Step 1

Step 1 says "Do not guess these". Step 2 then writes the guesses:

```rust
assert_eq!(src.line(3), None);
assert_eq!(src.line(0), None);
```

**VERIFIED** against the oracle — neither is "no line", both are errors:

| call | oracle |
|---|---|
| `sourceline(0)` | error **40.14** |
| `sourceline(99)` past end | error **40.34** |

`Option` remains a defensible Rust API — the BIF layer raises in Phase 4 — but
the plan should say that is the intent, and should record 40.14/40.34 now,
because Phase 3 is the only phase probing them and Phase 4 will need them.

**Clean, same probe:** `SOURCELINE(n)` strings carry no line terminator;
`sourceline()` on a 9-line file returns 9; a file with no trailing newline
counts its last line (`printf 'say sourceline()'` → 1).

### I7 — The spike corpus omits the case the parent plan calls the deciding hazard

The parent plan's D10 point 3 singles out `f(x)` versus `f (x)` as **"the
specific hazard of a combinator library"**. **VERIFIED** live, with `f = "VAR"`
and a `::routine f`:

```
say f(1)     ->  ROUTINE-CALLED-WITH-1
say f (1)    ->  VAR 1
```

Task 3.1's ten-line spike corpus contains no such pair. The spike therefore
measures `chumsky` against hand-written recursive descent on an axis where they
do not differ, and skips the one where the parent plan predicts they will.

Same omission for D10 point 4, the literal-suffix rule. **VERIFIED**:

```rexx
a="x"; b="y"
say a||b     ->  xy
say a""b     ->  x        /* ''b is an empty binary literal; b is never read */
```

Task 3.3 Step 1 does list `'…'x` / `'…'b` suffixes as a hard case, but no test
covers the silent-wrong-program variant, which is the one that bit this project
already.

### I8 — `a = b = c` cannot distinguish associativity at the obvious operands

**VERIFIED.** With `a=b=c=1`, both associativities give 1 — so the natural
probe proves nothing. The distinguishing case:

```rexx
a=2; b=2; c=1
say a = b = c        /* prints 1  => left-assoc (a=b)=c ; right-assoc gives 0 */
```

Comparison is left-associative. Task 3.1 lists the expression with no operands
attached, which is the plan's own "a probe drawn from one dimension cannot
reveal a second" note being violated in its first task.

### I9 — "CoreClasses.orx is almost entirely directives" is wrong

**VERIFIED.** `grep -c '^\s*::' interpreter/RexxClasses/CoreClasses.orx` = 347,
against 4,193 lines — **8.3%**. The file is overwhelmingly method *bodies*:
ordinary instructions and expressions.

Task 3.7 says "This task matters more than its size suggests: `CoreClasses.orx`
is almost entirely directives, so Task 3.10's throughput number depends on it."
The throughput number depends far more on Tasks 3.5 and 3.6. Task 3.7 is still
important — nothing parses without it — but the stated reason is false and
would misdirect optimisation effort.

### I10 — `INTERPRET` has no interface

The parent plan flags that "`INTERPRET` parses at runtime, so parser throughput
is on the execution path for some programs, not only at load". Phase 4 will
need to parse a fragment at runtime, against its own `ProgramSource`, with
INTERPRET's restrictions enforced (no labels, no directives, blocks complete
within the fragment).

The only public entry point in the plan is
`parse_program(text: String) -> Result<Program, ParseError>`, and everything
else is `pub(crate)`. Nothing states whether INTERPRET calls that, what
rejects a label inside it, or how the fragment's line numbers relate to the
enclosing program (see C1(b) — the oracle reports the INTERPRET instruction's
line). `rust/corpus/lang/interpret_dynamic.rex` exists and no task references
it.

---

## 3. Minor

### M1 — Task 3.4 cites the wrong file

**VERIFIED.** Step 1 says "Establish the rules from the C++:
`interpreter/parser/Clause.cpp`." That file is 211 lines and is the
`RexxClause` *container* — `newClause`, `newToken`, `trim`, `setStart`,
`setEnd`, `live`, `nextRealToken`. The clause-splitting loop is
`LanguageParser::nextClause()` at `interpreter/parser/LanguageParser.cpp:1009`.

The rules the step states are right, and both continuation forms check out:
`say 1,\n + 2` → 3 and `say 1 -\n + 2` → 3; `say "a" -\n "b"` → `a b`, so the
continuation contributes a blank.

### M2 — Comments separate tokens without producing a blank

**VERIFIED**, and not mentioned anywhere in Task 3.3:

```rexx
a = 5;  say a/*x*/a      ->  55      /* abuttal, no blank inserted */
ab = 9; say a/*x*/b      ->  AB      /* not 9: the comment splits the symbol */
```

A scanner that emits `Blank` for a comment prints `5 5`; one that deletes the
comment entirely prints `9`. Both are wrong, in opposite directions. Task 3.3's
Step 1 hard-case list should include this.

### M3 — `PARSE` templates and `ADDRESS … WITH` get no step

`PARSE` appears only as one of the eight "data" keywords. Its template grammar
is a sub-grammar in its own right: positional patterns (absolute `=n`, relative
`+n`/`-n`), literal patterns, variable patterns `(v)`, the placeholder `.`, and
the 10 `parseOptions` above. `rust/corpus/lang/parse_template.rex` exists and
no task references it.

`ADDRESS` appears only in the four-keyword "settings" family, but
`AddressWithInstruction.hpp` is a separate class — `ADDRESS … WITH INPUT /
OUTPUT / ERROR` redirection, with `CommandIOConfiguration.cpp` behind it.

### M4 — Task 3.9 needs nesting depth, not just spans

**VERIFIED** by running `rust/corpus/lang/trace_output.rex`:

```
     4 *-* if y > 5
     4 *-*   then
     4 *-*     say "big"
```

Three observations the Interfaces line ("the source-text slices `TRACE` needs
per clause") does not cover: the `IF` clause text ends with a **trailing
blank** and stops before `then`; `THEN` is a separate traced clause splitting
the line mid-way; and the indentation grows with block nesting depth, which is
structural information the AST must carry per clause. Spans alone will not
reconstruct this.

The `>L>` / `>V>` / `>O>` / `>>>` / `>=>` lines are execution trace and
correctly out of scope for Phase 3.

### M5 — Task 3.6 Step 4's completeness test cannot pass as written

```rust
for kw in ALL_KEYWORDS { assert!(parses_as_instruction(kw), "{kw} unhandled"); }
```

**VERIFIED** bare-keyword clauses under the oracle:

| clause | result |
|---|---|
| `nop` | ok |
| `then` | error 8 — Unexpected THEN or ELSE |
| `else` | error 8 |
| `when` | error 9 — Unexpected WHEN or OTHERWISE |
| `otherwise` | error 9 |
| `end` | error 10 — Unexpected or unmatched END |
| `parse` | error 20 — Symbol expected |

Only `NOP` parses standalone. The test needs a table of per-keyword minimal
*valid* snippets, not the bare word. (Those seven error numbers are useful
ground truth for Task 3.8 regardless.)

### M6 — `Program` has no label table

`Program { source, instructions, directives }`. `SIGNAL` resolves labels, and
the C++ builds a label directory at parse time (`LabelInstruction`). Nothing in
the plan says where labels land after Task 3.4 marks them on a `Clause`. Phase
4 cannot resolve `SIGNAL` without it.

### M7 — D13 is quoted with half of it missing

Global Constraints: *"The AST is plain owned Rust data (D13, closed). Not
garbage-collected, not reference-counted between nodes."* The parent plan
(line 2368) says "plain owned Rust data **inside one arena object per code
body**". The arena half is dropped, and it is the half that constrains node
representation (indices into a per-body arena rather than `Box`), which
interacts directly with C3.

---

## 4. Exit gate — criterion by criterion

The plan claims *"Every criterion above is reachable with a parser and the
oracle binary, and nothing else."* Assessed individually:

| # | criterion | satisfiable with parser + oracle? |
|---|---|---|
| 1 | 14 corpus programs parse; every construct appears in the AST | **Partly.** Parsing them is fine — all 14 run clean under the oracle today (verified). But *"every construct in them appears in the AST — not merely 'no error raised'"* has **no stated verification method**. See §5. |
| 2 | `CoreClasses.orx` + `StreamClasses.orx` parse end to end | **Yes.** Parser only. |
| 3 | number, sub-number, line **and column** match | **No.** The oracle emits no column — C2. Drop "column", add "substitution values". |
| 4 | `SOURCELINE(n)` matches for every line, incl. last line and no trailing newline | **Yes.** A `.rex` that prints every `sourceline(n)` gives the oracle side; verified reachable. |
| 5 | `TRACE` source reconstructible for `trace_output.rex` | **Yes**, given M4's nesting depth. Parser only. |
| 6 | throughput recorded against the ~55 ms budget, with a plain statement of fit | **Yes.** It asks only for a recorded number and an honest statement, which is the right shape for a phase that cannot measure the rest of cold start. |
| 7 | clippy clean, zero `unsafe` | **Yes.** |
| 8 | Phase 2's twelve sets still at 0 — 128,368 cases | **Yes.** |

**One criterion (3) smuggles in a dependency**, though not on execution as in
Phase 2 — on an interpreter interface that does not exist at all. One more (1)
is unassessable for a different reason: it has no method.

So the plan's claim is 6/8 true. That is a genuine improvement over Phase 2's
2/5, and the two failures are fixable by editing the criteria.

---

## 5. Placeholders and hand-waving

- **Exit gate 1, "every construct in them appears in the AST"** — no method,
  no threshold, no definition of "construct". Unfalsifiable as written. It
  needs something concrete: a node-kind histogram per program with a
  hand-written expected set, or an assertion that every `Instruction` and
  `Expr` variant is constructed at least once across the 14.
- **Task 3.3 Step 5** — *"Write a `.rex` driver that reads a program and
  reports what the interpreter makes of its structure … Where no such
  introspection exists, fall back to…"*. There is no such introspection: D13's
  own research (parent plan line 246) established that nothing exposes an
  object below `Method`/`Routine`/`Package`, and source comes back as text.
  The step should state the fallback as the method, not as a contingency.
- **Task 3.7 Steps 1–3** — "Enumerate the directives and their options from the
  C++", "Write a failing test per directive", "Implement, commit per
  directive". Three steps, no content, for the 2,867-line file. Compare Task
  3.6, which at least extracts its table with a concrete `grep`. Given I3 and
  I4 this is where the plan is thinnest relative to the work.
- **Task 3.5 Step 4** — "evaluate the parsed AST with a throwaway evaluator
  that uses `rexx-num` for arithmetic". The method is right and is the Phase 2
  method correctly generalised, but a throwaway evaluator over the full
  expression grammar (message sends, function calls, compound variables) is
  itself substantial, and the step gives it one sentence. Bound what the
  evaluator must cover — arithmetic and concatenation over literals and simple
  variables would be enough to catch precedence errors.
- **Task 3.1 Step 2** — "Timebox each" with no number given. The Notes lean
  hard on the timebox existing; it should state hours.
- **Task 3.9 Step 3** — "adjusting node spans if reconstruction is impossible"
  is the right instinct but has no acceptance condition.

No literal "TBD" appears anywhere in the plan.

---

## 6. Task decomposition

**Sound, with the exceptions already logged.** Dependency order is correct:
3.2 (source) → 3.3 (tokens) → 3.4 (clauses) → 3.5 (expressions) → 3.6/3.7
(instructions, directives) → 3.7b (composition) → 3.8 (errors) → 3.9 (trace) →
3.10 (bench). Nothing depends on something no earlier task produces.

Task 3.7b earns its place — the plan's stated reason ("a reviewer must be able
to reject the composition independently of the parts") is a real one and is the
kind of task Phase 2 lacked.

Interface consistency across tasks, checked name by name:

- `ProgramSource` (3.2) → consumed by 3.3 ✓; `position` used by 3.8 ✓.
- `Token`/`scan` (3.3) → `Vec<Token>` consumed by 3.4 ✓.
- `Clause` (3.4) → consumed by 3.5 ("clauses from Task 3.4"), 3.6, 3.7 ✓
  — but see C3, the consumption is single-clause where it must be a stream.
- `Expr`/`ast.rs` (3.5) → consumed by 3.6 ✓.
- **`ParseError` is used in Task 3.3's signature but not defined until Task
  3.8.** Minor ordering wrinkle: `scan(&ProgramSource) -> Result<Vec<Token>,
  ParseError>` at 3.3, `error.rs` created at 3.8. The type has to exist from
  3.3 onward; 3.8 should be described as completing it (message rendering,
  the number table) rather than creating it.
- `parse_expr(&[Token])` (3.5) versus `parse_instruction(&Clause)` (3.6) — one
  takes a token slice, the other a clause. 3.4's `Clause { tokens:
  Range<usize>, … }` holds a range into the token vector, so 3.6 must also
  carry the token vector to reach `parse_expr`. Not wrong, but the signatures
  as written don't compose without a context struct nobody names.

**Missing from the decomposition entirely** (each would need its own task or a
named step): the command/assignment/message instructions (C4), `PARSE`
templates (M3), the `::RESOURCE` raw-text mode (I3), the label table (M6), and
the INTERPRET entry point (I10).

---

## 7. What was checked and is clean

Stated explicitly rather than padded — these are the claims I tried to break
and could not.

**Counts, all exact:**

- 19 token classes — `Token.hpp:79–97`, `TOKEN_NULL` … `TOKEN_ASSIGNMENT`.
- 35 keyword→instruction mappings — `sed -n '/keywordInstructions\[\]/,/^};/p'
  … | grep -c 'KeywordEntry('` = 35.
- 36 keyword constants in `KeywordConstants.cpp` — 36 distinct `KEYWORD_`
  symbols, i.e. the 35 plus `KEYWORD_NONE`.
- 52 instruction classes — `ls interpreter/instructions/*.hpp | wc -l` = 52.
- 17 expression classes — `ls interpreter/expression/*.hpp | wc -l` = 17.
- 5,203 bootstrap lines — CoreClasses.orx 4,193 + StreamClasses.orx 1,010.
- 5.1 ms C++ cold start — `perf-baseline.md:101`, median 5.119 ms.
- The five individual LOC values (the total is wrong — I1).

**The keyword families:** all 35 names, and the 11/8/11/4/1 partition, are
exactly the extracted table with no invention and no omission. Checked as a set
against the `grep` output. The plan's own `grep -oE '"[A-Z]+", *KEYWORD_[A-Z_]+'`
in Task 3.6 Step 1 returns exactly the 35 and nothing else — the extraction
step works as written.

**Keywords are not reserved.** Every one of the plan's seven probe lines
behaves as claimed:

```
if = 2;  say if              ->  2
do = 3;  say do              ->  3
say = 4; say say             ->  4
end = 5; say end             ->  5
if if = 2 then say if        ->  2
do i = 1 to 2; say do; end   ->  3, 3
end. = 0; end.1 = 7          ->  7
```

The plan's inference from this — that a symbol is a keyword only by position,
so recognition belongs in clause dispatch and cannot live in the scanner — is
correct and is the plan's best single passage. It matches the C++ exactly:
`InstructionParser.cpp:179–196` checks first-token-is-symbol and
second-token-is-`OPERATOR_EQUAL` before any keyword lookup.

**Block comments nest.**

```
/* a /* b */ c */ say 3       ->  3         (whole comment consumed)
/* a /* b */ say 1            ->  Error 6.1: Unmatched comment delimiter
                                            ("/*") on line 1
```

**`--` is a line comment and `-` is not.** `say 1 -- comment` → 1;
`say 2 - 1` → 1.

**`-2 ** 2` is 4.** And it is genuinely precedence, not negative-literal
scanning — `x = 2; say -x ** 2` also gives **4**. `say -(2**2)` → -4,
`say (-2)**2` → 4. The plan's test comment "(C and Python give -4)" is right.

**`say a b + 1` → `1 3`** with `a=1; b=2`, consistent with the plan's asserted
shape `Concat(a, Add(b, 1))`. `say (a b) + 1` raises 41.1 (nonnumeric "1 2"),
confirming the grouping.

**Twelve differential sets, 128,368 cases** — correct, and *more* accurate than
`phase-2-gate.md`, which says eleven and 126,048. `gen-curated-sets.py` has 12
entries in `SETS`; regenerated and counted:

```
addsub 8712  addsub2 8112  muldiv 17424  md2 20184  pow 2112  cmp 32368
fmt 1800  fmt2 6720  fmt3 12136  fmtedge 640  fmtcarry 15840  signblank 2320
                                                            total = 128368
```

The gate document omits `signblank` (2,320). 126,048 + 2,320 = 128,368. The
plan is right; the gate doc is stale.

**Dependencies available offline:** `chumsky-0.13.0.crate` is in the cargo
registry cache (alongside 0.9.3). `criterion = "0.8.2"` matches
`rexx-num`, `rexx-core` and `rexx-bench` exactly, and the bench settings the
plan specifies — `sample_size(10)`, 500 ms warmup, 30 s measurement — are
verbatim what `rexx-num/benches/arith.rs:61–63` and
`rexx-bench/benches/interpreter.rs:36–38` use.

**All 14 `rust/corpus/lang/` programs run clean** under `build/bin/rexx` (rc=0
each), so exit gate criterion 1's oracle side is sound. `trace_output.rex` and
`keyword_as_variable.rex` both exist and are referenced.

**`rexx-inventory` has the message table** Task 3.8 says it consumes:
`pub mod errors` generated into `OUT_DIR` by a build script from the C++ tree.

**The Notes carried in** are accurate as history — the Phase 2 lessons they
cite (error boundaries, one-dimensional probes, the sorted-table cost, AST
must not discard source) match `phase-2-gate.md`. Only the keyword-table lesson
(I2) and the corpus claim (I0) are factually wrong.

---

## 8. Suggested order of fixes

1. C2 and C1 first — they change what Task 3.8 and the exit gate *are*.
2. C3, then C4 — they change the AST and therefore Tasks 3.5–3.7b.
3. I4 and I3 — they change Task 3.6/3.7 scope, which changes the phase estimate.
4. I0, I1, I2, I5, I9 — factual corrections, mechanical.
5. I6, I7, I8, and §5's placeholders — test and probe design.
6. The Minors as edits in place.

---

# Second pass — review of the fixes (b465ef4a, e019e228)

Same tagging: **VERIFIED** = ran it or read it. Focus is on whether the fixes
are right and what they introduced, not on re-deriving the originals.

## Verdict

**Closer, but not yet executable.** All four Criticals were addressed in
substance and the two most important ones (C1, C2) are now correct. But the
fix round introduced two Critical self-contradictions and seven Important
defects, and four findings from the first pass were not addressed at all.

The pattern the team lead named held: **the fixes edited the Notes section but
left the task body saying the opposite**, twice.

---

## New Critical

### R1 — Task 3.6 still carries the original "do not sort" text verbatim

**VERIFIED**, plan lines 458–461, unchanged from the first draft:

> `KeywordConstants.cpp` has 36 keyword constants and 35 keyword→instruction
> mappings; `interpreter/instructions/` has 52 classes. **The table is not
> alphabetical and the C++ indexes it by position** — a lesson already paid for
> in Phase 0 with the builtin table. Do not sort it.

The Notes section (lines 827–833) now says the correct opposite, at length and
accurately. So the document asserts both. The task body is what an implementer
reads while working; the Notes are read once. Delete lines 459–461's claim and
point at the Notes entry.

### R2 — `parse_directive` still takes a single clause

**VERIFIED**, plan line 582:

```
parse_directive(&Clause) -> Result<Directive, ParseError>
```

C3 was fixed for `parse_instruction` (now `&mut ClauseCursor`, correctly
justified) but not for `parse_directive`. A `::method` or `::routine` body is
an arbitrary run of clauses. The plan now *states* this problem in Task 3.6's
own new text — "`DO`/`END`, `IF`/`THEN`/`ELSE`, `SELECT`/`WHEN`/`OTHERWISE` and
every `::method` body span many clauses, so a function handed one clause cannot
parse any of them" — and then leaves Task 3.7 with exactly that signature.

---

## New Important

### R3 — the bare-keyword error numbers are now wrong

**VERIFIED.** Task 3.6 Step 4 (line 545): *"`then`, `else`, `when`,
`otherwise`, `end` and `procedure` alone are errors 8, 9, 10 and 20"*.

The edit swapped `parse` for `procedure` but kept `20`, which was `parse`'s
number:

| bare clause | error |
|---|---|
| `then`, `else` | 8 — Unexpected THEN or ELSE |
| `when`, `otherwise` | 9 — Unexpected WHEN or OTHERWISE |
| `end` | 10 — Unexpected or unmatched END |
| `procedure` | **17** — Unexpected PROCEDURE |
| `parse` | 20 — Symbol expected |

Should read "errors 8, 9, 10 and 17", or keep `parse` and restore 20.

### R4 — the `Message` clause row is half wrong

**VERIFIED.** The new five-clause-type table (line 449) gives Message as
*"starts with `~` or is a message send"*. A clause starting with `~` is not a
message instruction:

```
q = .queue~new
~append(1)      ->  Error 35 line 2: Invalid expression.
                    Error 35.1: Incorrect expression detected at "~".
```

Only the second half is right — a standalone `q~append(1)` clause is the
message instruction (verified, runs). Drop "starts with `~` or".

The rest of the table checks out. `Command` as fallback is verified:
`address system` then `"echo hello-from-command"` prints the output and sets
`rc` to 0.

### R5 — Task 3.1 still gates the spike on column fidelity

**VERIFIED**, lines 149–150, unchanged: *"This is the axis most likely to
decide it, because Phase 3's gate is error messages with line **and column**."*

Column was removed from Task 3.8 and from the exit gate. Task 3.1 now measures
the two D10 candidates against a criterion the phase no longer has — and it is
the axis the step calls decisive. Should read "line and substitution values".

### R6 — `::RESOURCE`'s scanner mode is stated only after the scanner is built

The new text is right and well-placed as a discovery: *"Task 3.3's scanner must
be able to switch into a copy-until-delimiter mode and back. Discovering that
here rather than in Task 3.3 is the point of listing it."*

But Task 3.3 runs and commits four tasks earlier, and its Step 1 hard-case list
does not mention raw-text mode. The requirement needs to be in Task 3.3 Step 1,
with Task 3.7 citing it — otherwise the scanner is built, tested and committed
without it, which is the retrofit the plan elsewhere works hard to avoid.

### R7 — Task 3.6 Step 4's test depends on a decision Task 3.1 has not made

The replacement test asserts `parses_to_node_named(src, kw)` over a 35-entry
table, with entries like `("IF", "if 1 then nop")`. Whether `THEN`, `ELSE`,
`END`, `WHEN` and `OTHERWISE` are *nodes at all* depends on Step 3b's
tree-versus-chain choice: the C++ chain has `ThenInstruction`,
`ElseInstruction`, `EndInstruction` and `OtherwiseInstruction` as real classes,
but a tree folds all five into their parent, and `if 1 then nop` yields one
`If` node with no `Then` in sight.

Five of the 35 rows therefore cannot be written until 3.1 decides. Say so, or
make the expectation "the keyword is consumed by some node" rather than "a node
named for it exists".

The length assertion is a genuinely good addition and is independent of this.

### R8 — the round-trip gate method double-counts

Exit gate 1 now says: *"walking the AST in order and concatenating every node's
source span reproduces the original text with only comments and inter-token
blanks missing."*

Concrete, which is the improvement asked for. But spans nest — an
`Add(a, b)` node's span covers both operands' spans — so concatenating *every*
node's span reproduces the text several times over. It needs to say **leaf
nodes in source order**, or "each node's span contains its children's and the
top-level spans tile the source without gaps or overlap".

Second issue: "with only comments and inter-token blanks missing" gives away
the one thing that would catch a dropped `TOKEN_BLANK`, and blanks are
semantic in Rexx (abuttal). Excluding them from the round-trip means the check
cannot detect the abuttal bug it is best placed to catch.

### R9 — the spike corpus mislabels `say a""b`

Line 123: ``say a""b     /* null-string abuttal */``

It is **not** abuttal. `""b` scans as an *empty binary literal*, so the clause
concatenates `a` with `""` and never reads `b` at all — verified,
`a="x"; b="y"; say a""b` prints **x**, where `say a||b` prints `xy`. The whole
point of the case is that it looks like abuttal and is not. Labelled this way,
an implementer writes the expectation `xy` and the case tests nothing.

---

## First-pass findings not addressed

Answering the team lead's question directly — these are in the review file and
are not fixed in either commit.

- **I9 — "`CoreClasses.orx` is almost entirely directives"** survives verbatim
  at line 596. **Re-verified: 347 of 4,193 lines begin with `::` — 8.3%.** The
  file is mostly method bodies. Task 3.7 is still important; the stated reason
  is still false, and it points Task 3.10's optimisation effort at the wrong
  task.
- **I10 — `INTERPRET` has no interface.** Still nothing: `parse_program(text:
  String)` remains the sole entry point, with no statement of how a runtime
  fragment is parsed, what enforces INTERPRET's restrictions (no labels, no
  directives), or how fragment line numbers relate to the enclosing program.
  `rust/corpus/lang/interpret_dynamic.rex` is still referenced by no task.
- **I8 — `a = b = c` still has no operands** in the spike corpus. At
  `a=b=c=1` both associativities give 1, so the natural probe proves nothing.
  `a=2; b=2; c=1` → **1**, which proves left-associative `(a=b)=c` (right-assoc
  gives 0). One line in the corpus block fixes it.
- **M6 — partially fixed.** Task 3.6 Step 3 now requires a label table, which
  is the substantive half. But `Program { source, instructions, directives }`
  (line 619) still has no field to put it in, and that struct is the Phase 4
  interface.

Minor drift the edits left behind, all in the File structure block and Global
Constraints, all one-line fixes:

- line 64 — `instruction.rs # the 35 keyword instructions`; it now also owns the
  five non-keyword clause types.
- line 65 — `directive.rs # ::class, ::method, ::routine, ::requires,
  ::attribute`; Task 3.7 now correctly says nine.
- line 18 — Global Constraints still states D13 without the arena half that
  Step 3b restored.
- line 584–589 — "The other 36 `DIRECTIVE_*` constants are their option
  sub-keywords … — 40 of them per the `subDirectives` table" conflates two
  different counts in one sentence. Both numbers are right (**VERIFIED**: 45
  distinct `DIRECTIVE_*` constants, minus 9 top-level, = 36; `subDirectives[]`
  has 40 entries, because several names appear in both tables), but as written
  it reads as a contradiction.

---

## Fixes that are correct

Checked rather than assumed.

- **C1** — both recipes work. Route (a) captures the file's own line on stderr;
  route (b) fires the trap and yields `condition('o')~code`, with the POSITION
  caveat stated correctly. `rc 219` is consistent (256 − 37, the error for a
  stray `)`).
- **C2** — Task 3.8 Step 4's field list is exactly my dump of the condition
  object, and the reasoning ("ooRexx locates an error by quoting the offending
  token") is right. The gate now asks for substitution values. Retaining a
  column internally while forbidding it as a gate is the right call.
- **C3** (instruction half) — `ClauseCursor` with the justification stated. Step
  3b is a good addition and the arena half of D13 is restored.
- **C4** — the clause-type table is right apart from R4, and naming `Command`
  as the never-fail fallback is the important part.
- **I1** — 14,638 for the five files with the 17,483 directory total kept as a
  separate labelled row. Correct and clearer than either original.
- **I3** — nine directives, correctly named.
- **I4** — the sub-keyword tables now appear in "What is being replaced" *and*
  in Task 3.6 Step 3, with the right sizing advice.
- **I6** — the test now carries the 40.14/40.34 numbers as a comment and says
  why `None` is the right Rust shape. Better than either alternative.
- **I7** — `f(x)` / `f (x)` added with the parent plan's justification (only
  the `a""b` label is wrong, R9).
- **M1** — `Clause.cpp` correctly demoted to the data structure, splitting
  attributed to `LanguageParser.cpp` (`nextClause`).
- **M2** — the comment-produces-no-blank rule is in Task 3.3 Step 1 with the
  correct probe and both failure modes named.
- **M3** — PARSE templates, `ADDRESS … WITH` and SIGNAL/labels each get a
  commit in Task 3.6 Step 3.
- **M4** — TRACE nesting depth called out as insufficient-from-spans.
- **M5** — the keyword table with its length assertion (subject to R3 and R7).
- **I0** — both references now point at `keyword_as_variable.rex` as a
  must-pass, and the Notes entry records the sequence honestly.

---

## Exit gate — second assessment

| # | criterion | satisfiable with parser + oracle? |
|---|---|---|
| 1 | 14 programs parse **and** round-trip their spans | **Yes in principle** — the method is now stated, which was the gap. But it needs R8's tightening before it means anything: as worded it double-counts nested spans, and excluding blanks blinds it to the abuttal case. |
| 2 | `CoreClasses.orx` + `StreamClasses.orx` parse end to end | **Yes.** |
| 3 | number, sub-number, line, **substitution values**, by running a bad file and capturing stderr | **Yes — now fixed.** Every one of the four is observable. This was the criterion that failed the first pass. |
| 4 | `SOURCELINE(n)` for every line, incl. last and no trailing newline | **Yes.** |
| 5 | `TRACE` source reconstructible for `trace_output.rex` | **Yes**, and M4's depth requirement is now recorded so it will not be discovered late. |
| 6 | throughput recorded against the ~55 ms budget with a plain fit statement | **Yes.** |
| 7 | clippy clean, zero `unsafe` | **Yes.** |
| 8 | Phase 2's twelve sets at 0 — 128,368 cases | **Yes.** |

**8 of 8 are now satisfiable with a parser and the oracle binary**, against 6 of
8 in the first draft. The plan's claim in "Notes carried in" is true as of this
revision. Criterion 1 is the only one whose *wording* still needs work, and
that is a precision problem, not a reachability one.

## Suggested order for the second fix round

1. R1 and R2 — the two self-contradictions. Both are deletions or a signature
   change; neither needs new thinking.
2. R3, R4, R9, I9 — factual corrections, one line each.
3. R5, R6, R7 — cross-task consistency; R6 is the one with a real cost if
   missed, because it lands after the scanner ships.
4. R8 — tighten the round-trip wording.
5. I10, I8, M6's struct field, and the File-structure drift.

---

# Third pass — review of a60d2529 / 696ecd25 / e18ea918

## Verdict

**Not executable yet — one Critical and five Importants.** All nine R findings
and all four named misses are genuinely addressed; R5 in particular is now
correct. But this round introduced defects again, and **the named failure mode
recurred**: Task 3.7 contradicts itself forty lines apart.

## New Critical

### T1 — Task 3.9's acceptance criterion names execution-trace lines

Line 863-865: *"the text reconstructed from the AST is **byte-identical** to
the corresponding `>>>`/`>V>` line the interpreter prints."*

**VERIFIED** from `trace_output.rex` output — those are the wrong markers:

```
     2 *-* x = 1 + 1        <- source clause. This is what Phase 3 reconstructs.
       >L>   "1"            <- literal VALUE, execution only
       >V>   X => "2"       <- variable VALUE, execution only
       >>>   "2"            <- result VALUE, execution only
```

`>V>` and `>>>` carry evaluated *values*, not source. Reconstructing them
requires running the program. As written, Task 3.9 and exit-gate criterion 6
demand an interpreter that does not exist until Phase 4 — the exact Phase 2
failure this plan is built to avoid, reintroduced into the one criterion that
was previously clean. It must say `*-*`.

## New Important

### T2 — R3's fix corrected one number and broke another

Line 604-605: *"`then` is error 8, `when` is 9, `otherwise` is 10, `procedure`
is 17 and `parse` is 20."*

**Re-measured, all seven:**

| bare clause | error |
|---|---|
| `then`, `else` | 8 |
| `when`, **`otherwise`** | **9** |
| `end` | **10** |
| `procedure` | 17 |
| `parse` | 20 |

`otherwise` is 9, not 10. 10 belongs to `end`, which this edit dropped from the
list. This number goes straight into a test.

### T3 — the named pattern recurred inside one task

Task 3.7 lines 666-670 now correctly say *"only 347 of `CoreClasses.orx`'s
4,193 lines start with `::`, 8.3%"*. Line 709-710, in Step 6 of the same task,
still says the file *"is almost entirely directives"*. Fixed in the prose,
left standing in the step.

### T4 — `ParseCtx` states a rule three signatures break

Task 3.3 line 299: *"Every `parse_*` in Tasks 3.5–3.7 takes `&ParseCtx` plus
its own cursor."* Only `parse_directive` does:

- line 426 — `parse_expr(&[Token]) -> Result<Expr, ParseError>`
- line 489 — `parse_instruction(&mut ClauseCursor) -> …`
- line 648 — `parse_directive(&ParseCtx, &mut ClauseCursor) -> …` ✓

`ParseCtx` is a good addition and the justification is right; the two older
signatures were not updated with it.

### T5 — `ParseError`'s shape disagrees between the tasks that define it

- Task 3.3 line 282: `{ code: u16, sub: u16, byte: usize, subs: Vec<String> }`
- Task 3.8 line 777: `{ code: u16, sub: u16, line: usize, column: usize, subs }`

`byte` vanishes, and `column` reappears despite the File structure block
(line 68) now listing `error.rs` as "number, sub-number, line, substitutions".
Step 4's "keep a column internally, never gate on it" is a defensible position,
but the struct should be stated once, in one place.

### T6 — the round-trip is still not achievable

The R8 fix is right about leaf nodes and right to keep blanks. But "reproduces
the original text with only comments removed" cannot hold: **leading
indentation, blank lines and the `,`/`-` continuation characters are inside no
node's span at all**, and the blank that marks abuttal is the *operator* — an
interior node — so leaf spans yield `ab` where the source has `a b`.

State it as the tiling property instead: every node's span contains its
children's, and the top-level spans cover the source in order without overlap
or gaps except for comments and inter-clause whitespace. That is checkable and
catches the dropped clause, which is the point.

## Minor

### T7 — Task 3.7 has two Step 5

Lines 702 and 712. The commit step should be Step 7.

## Fixes that are correct

R1 (with the Phase 0 builtin-table contrast, so the wrong lesson cannot be
re-imported), R2, R4, R5 (genuinely done — axis 2 now measures number,
sub-number, line and substitutions, and states there is no column), R6
(raw-text mode is in Task 3.3's list, ahead of the scanner being built), R7,
R9 (`""b` correctly called an empty binary literal), I8 (the associativity
arithmetic is right: `a=2 b=2 c=1` gives 1 left, 0 right), I9's prose half,
I10 (`parse_interpret` with all three restrictions), M6 (`labels:
BTreeMap<String, usize>`), the arena in Global Constraints, and the File
structure block.

Task 3.7 Step 1's extraction command was run: it returns **45**, as claimed.

The new gate criterion — every `Instruction` and `Expr` variant constructed at
least once across the 14 programs, asserted by enumeration — is a real
improvement and is satisfiable with a parser alone.

## Exit gate — third assessment

| # | criterion | satisfiable? |
|---|---|---|
| 1 | 14 programs parse **and** leaf-span round-trip | **Needs T6.** Reachable once restated as tiling. |
| 2 | every `Instruction`/`Expr` variant constructed | **Yes.** New, and good. |
| 3 | `CoreClasses.orx` + `StreamClasses.orx` parse | **Yes.** |
| 4 | number, sub-number, line, substitutions | **Yes.** |
| 5 | `SOURCELINE(n)` every line | **Yes.** |
| 6 | `TRACE` source reconstructible | **No as written — T1.** `>>>`/`>V>` need execution. With `*-*`, yes. |
| 7 | throughput against the budget | **Yes.** |
| 8 | clippy clean, zero `unsafe` | **Yes.** |
| 9 | twelve differential sets at 0 | **Yes.** |

**7 of 9 clean; 1 and 6 need T6 and T1.** Both are wording fixes, not design
changes.

## Is it executable?

**Not yet, but it is one short edit round away.** Fix T1 (one marker), T2 (one
number), T3 (one sentence), T4 and T5 (signature and struct consistency), and
T6 (restate the round-trip). None needs new investigation — every answer is
above. Nothing structural is wrong any more: the task order, the interfaces,
the sizing and the gate are all sound.
