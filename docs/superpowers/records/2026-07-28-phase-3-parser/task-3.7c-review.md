# Task 3.7c review: block structure and the control stack

Reviewed at `964aefd5` against the brief, the five plan rulings, the report and the
four-commit diff.
Everything below that says "measured" was measured in this review, in a fresh
`mktemp -d` per case with the whole `rexxc` message printed, not taken from the
report.

## Verdicts

1. **Spec compliance: compliant.** All five rulings are implemented as ruled, all
   six brief steps are done, and the twenty errors are in the code with the
   numbers, sub-numbers and reported lines the oracle gives. I re-measured 26 of
   the 20-error set plus 75 further shapes against `build/bin/rexxc` and the
   parser agrees with every one.
2. **Code quality: good, with one correctness defect.** The port is faithful
   where I could check it against the C++, the doc comments state contracts and
   put reasoning at the decision point, and the tests are honest in both
   directions. One divergence is real and is described below as Important 1: the
   99.913 guard check accepts eight measured shapes the oracle rejects, and the
   report's stated reason why walking the finished tree "is equivalent" is false.
3. **Gate criterion 4: not met.** Not because the block work is missing -- that
   part landed -- but because the criterion's wording states a one-directional
   property over a set nobody has enumerated, and read literally it is falsified
   by a deviation Task 3.6 already accepted. The audit section gives the
   replacement wording.

Counts: **0 Critical, 3 Important, 5 Minor.**

---

## What I re-measured

| batch | shapes | what |
|---|---|---|
| A | 27 | every one of the twenty errors that is reachable, plus 35.929/35.934, each with blank lines so the reported line is separable from the substituted one |
| B | 18 | fresh attempts at 10.5 and 10.6 that the report's six probes do not cover |
| C | 6 | the `then:` label shape and the directive-terminated 18.x shapes |
| D | 10 | end-of-file versus directive termination for 14.x and 18.x |
| E, F | 22 | every 99.913 variant, including the compound-variable cases |
| G | 18 | error precedence with two errors available, and twelve shapes that must be accepted |
| — | 385 | every program the crate's own tests assert an error on, run through `rexxc` and diffed against the parser |
| — | 192 | every program the crate's own tests assert *parses*, run through `rexxc` |

The 385 and 192 sets were collected by instrumenting `fn ok`/`fn err*` in
`instruction/tests.rs`, `directive/tests.rs` and `block/tests.rs` to print their
argument, running the suite, then restoring the three files from a copy. The tree
was verified clean afterwards (`git status --porcelain` empty) and the full suite
re-run: 315 tests, 0 failures.

**Errors: 101 oracle probes, 94 of them also run through the parser. Every one
agrees on number and sub-number. One agrees on number but not on the reported
line (Minor 3).** Of the 385 test-asserted error programs, 384 agree with the
oracle on number and sub-number and 383 also agree on the reported line.

**The three line conventions are all reproduced.** Measured, with the SELECT on
line 3 and the END on line 5: 7.1 reports **line 3** (the SELECT's own), 7.2
reports **line 5** (the offending clause's), and 10.1/10.2/10.3/10.4/10.7 report
the END's. The parser gives 3, 5 and the END's respectively. 14.901's three-digit
sub-number is right, and 14.4 is reported against the ELSE's own line where 14.1,
14.2 and 14.5 are reported against the last instruction added -- all four match.

**10.5 and 10.6: I could not reach either, and the omission is stronger than
"six probes".** It is structural. In `translateBlock`, an `END` has
`isControl() == false` and `type != KEYWORD_ELSE`, so `flushControl(instruction)`
*always* runs before the `END` switch (`LanguageParser.cpp:1294`-`1301`), and
`flushControl` cannot return with `ELSE`, `IFTHEN` or `WHENTHEN` on top: the ELSE
arm pops and loops, the IFTHEN/WHENTHEN arm pops and pushes `ENDTHEN`/`ENDWHEN`,
and the third arm only runs when the top is none of the three. The switch
therefore never sees the three types its 10.5 and 10.6 arms test for. I also
tried 18 shapes the report's six do not cover -- nested IF/ELSE, `else end` on
one line, `if 1 = 1 then; end`, a WHEN acting as another WHEN's THEN followed by
`end`, an `otherwise` holding a dangling THEN, a named `end a` closing a labelled
DO whose body is a dangling THEN, and the same inside a `::method` body -- and
every one is **10.1** in the oracle and **10.1** in the parser. Recommend adding
the `flushControl` post-condition argument to the test's comment, because it is a
proof where the probes are only evidence.

**A third bad Task 3.6 assertion: none.** All 192 programs the tests assert parse
were run through `rexxc`. Two are rejected and neither is a block-structure
mistake: `::routine r external "LIBRARY x"` and
`::method 3 attribute external "LIBRARY x"` both get **98.903 "Unable to load
library"**, which is a load failure after translation, not a grammar rejection.
They belong to Task 3.7's directive work, the parser is right to accept them, and
they matter only because criterion 4's definition of "parse-time" mis-classifies
them -- see the audit.

**Mutations: I re-applied 7 of the 78 independently** (04, 20, 25, 32, 53, 66,
78) with my own script. All 7 were caught, each by a named assertion failing, and
the tests that caught them are the ones you would expect. **I confirmed the "not
by a compile error" claim on two of them** by running `cargo check --offline -p
rexx-parse --all-targets` under the mutation: exit 0 for mutation 04
(`Control::Otherwise => 6`) and exit 0 for mutation 32 (the largest replacement
in the set, an `InstructionKind::When` literal spliced into `instruction.rs`),
with `cargo test` then failing on
`block::tests::an_unclosed_block_picks_its_number_from_the_block_kind` and
`block::tests::a_when_outside_a_select_is_9_1`. Note that the report's own
"compiled" detection cannot be trusted at face value -- a failing `cargo test`
prints an `error:` line, which is the trap the report says it walked into on the
first run; the reason the claim holds is that all 78 lines in the log read
`test result: FAILED` rather than `DID NOT COMPILE`, plus these two checks.

---

## Important

### Important 1: the 99.913 guard check accepts eight measured shapes the oracle rejects, and the equivalence argument for it is false

`guard_exposes` (`instruction.rs`) walks the finished condition tree, and its doc
comment plus report section 4 justify that with: "capture is unconditional over
the whole expression and those two paths are reached for exactly the variable
references in it." That is true for `addSimpleVariable` and `addStem`. It is
**not** true for a compound variable. `addCompound`
(`LanguageParser.cpp:2124`-`2131`) returns early on a cache hit, **before** it
calls `addStem`, so a compound whose exact name was already added to this body's
`variables` table never reaches `captureGuardVariable` at all. The C++'s own
comment three lines below the early return says "NOTE: compound variables do get
added to the guard list", which is exactly what the port trusted.

Measured, all eight rejected by `rexxc` with **99.913** and all eight accepted
(rc 0) by the parser:

| program (inside `::method m`) | oracle | parser |
|---|---|---|
| `expose a.` / `x = a.1` / `guard on when a.1` | 99.913 | rc 0 |
| `expose a.` / `say a.1` / `guard on when a.1` | 99.913 | rc 0 |
| `expose a.` / `drop a.1` / `guard on when a.1` | 99.913 | rc 0 |
| `expose a. a.1` / `guard on when a.1` | 99.913 | rc 0 |
| `expose a.` / `guard on when a.1` / `guard on when a.1` | 99.913 | rc 0 |
| `expose a.` / `z = a.1 + 1` / `guard off when a.1` | 99.913 | rc 0 |
| `expose b` / `x = a.b` / `guard on when a.b` | 99.913 | rc 0 |
| main program: `use local b` / `x = a.b` / `guard on when a.b` | 99.913 | rc 0 |

The rule the measurements establish: a compound reference in a `GUARD ... WHEN`
expression contributes nothing if that **exact** compound name was already
referenced anywhere earlier in the same code body -- by an assignment, a `SAY`, a
`DROP`, an `EXPOSE` list, or a previous `GUARD`. Two controls confirm the cause
rather than a different rule: `expose a.` / `x = a.2` / `guard on when a.1` is
**rc 0** (different name, not cached), and the same shape with the earlier
reference moved into a *second* `::method` body is **rc 0** (the table is
per-body).

**Concrete failure scenario.** A method that reads a stem element and then waits
on it -- `expose queue.` / `say queue.1` / `guard on when queue.1` -- is rejected
by `rexxc` and accepted by this parser. Phase 4 would then execute a program the
oracle refuses to translate. Nothing in any gate catches it: all 11 real
`GUARD ... WHEN` uses under `samples/` and `interpreter/RexxClasses/` name simple
variables, so the 301-file round-trip and the two `.orx` files are unaffected,
and criterion 4 as worded does not look in this direction at all.

Two acceptable resolutions, and either is fine, but the false equivalence claim
must not survive:

* **Port it.** `Block` gains a per-body set of compound names already referenced;
  `guard_exposes` treats an `ExprKind::Compound` as contributing nothing when its
  name is already in the set; the set is filled from each instruction's
  expressions *and* from the variable lists of `EXPOSE`/`DROP`/`PROCEDURE`, after
  the current instruction's own guard check runs.
* **Record it as a deviation**, the way Task 3.6's `then:` case is recorded: a
  test pinning the divergence in both directions with the eight measured shapes,
  a comment on `guard_exposes` naming `addCompound`'s cache early-return as the
  cause, and a line in the plan. This is defensible given no corpus file
  exercises it, but it changes criterion 4's answer and must be listed there.

### Important 2: the `resolveCalls` deferral is recorded only in the report

The reasoning holds. I read `resolveCalls` (`LanguageParser.cpp:1688`),
`RexxInstructionCall::resolve` (`CallInstruction.cpp:136`) and the one
registration site (`addReference`, five call sites): it raises nothing, it only
fills `targetInstruction` from the label table, and the builtin-versus-internal
decision is made at construction from `name->builtin()`, not here. Nothing in
Phase 3 depends on it, and `CodeBody::labels` gives Phase 4 the same per-body
table.

What is missing is the record. The report says "worth a line in whichever task
owns call resolution, so it is not assumed done", and that line exists nowhere
else: `grep -rn "resolveCalls\|resolve_calls"` over the plan, the progress file
and `rust/` returns nothing. There is no Phase 4 plan file yet, so the note has
no natural home and will be lost. Put it on `CodeBody::labels`' doc comment,
which is where a Phase 4 implementer will actually be standing, and add a line to
the parent plan's Phase 4 notes.

### Important 3: criterion 4's wording, not its subject matter, is what blocks it

Detailed in the audit section below. Listed here because it is an action, not
just an observation: the criterion needs rewriting before anyone can tick it, and
the rewrite has to name a corpus.

---

## Minor

### Minor 1: `is_control` includes `Otherwise`, which the C++'s `isControl()` does not

`block.rs:697` claims to be `isControl()` and its comment says "True for every
block instruction and additionally for a bare `IF`". `RexxInstructionOtherwise`
(`OtherwiseInstruction.hpp:55`) overrides `isBlock()` to true and does **not**
override `isControl()`, so it inherits `false` from `RexxInstruction.hpp:83`. An
OTHERWISE therefore goes through `flushControl` in the C++ and straight to
`addClause` here.

Behaviourally I could not make it show. The divergence only bites when the top of
the stack is `IfThen`, `WhenThen` or `Else` as the OTHERWISE arrives, and every
such shape is 9.2 either way -- the C++ rewrites the frame first and then finds a
non-SELECT on top; the Rust finds the un-rewritten frame, which is also not a
SELECT. Measured both of the interesting ones: `select` / `when 1 = 1 then` /
`otherwise nop` / `end` is **9.2** in oracle and parser, and `select` /
`otherwise if 1 = 1 then` / `end` is **10.1** in both. When the OTHERWISE is
legal the top is a SELECT and `flushControl`'s third arm is a bare `addClause`,
so the chain is identical.

Fix the comment rather than the code: say that OTHERWISE is grouped with the
control instructions here because the only shapes where that differs are rejected
either way, and name the two measurements. As written the comment asserts a rule
about the C++ that the C++ does not follow, which is the kind of statement a
later reader will build on.

### Minor 2: `Block::finish`'s `debug_assert!` should be an `assert!`

The judgement asked for: the `debug_assert` is *sufficient as a check* but is not
what enforces the invariant. The invariant holds structurally -- `translate_block`
returns `Err` unless the top frame is `First` at end of body, every `Do`/`Loop`/
`Select` is pushed when added, and `match_end` either errors or writes `closes`
-- so the assertion is belt-and-braces, and a release consumer seeing `None` in
the second group would mean the driver had a bug rather than that the invariant
was unenforced.

That said, the objection in the brief is right and the fix is nearly free.
`resolve_targets` already walks every instruction immediately before it, so
promoting `debug_assert!` to `assert!` adds one linear pass with a cheap match
per body -- nothing against the 55 ms budget for 5,203 lines -- and buys the
release consumer the same guarantee the tests get. Given "prefer correctness over
performance", make it an `assert!`. The alternative the report considered and
rejected (a separate builder representation that makes the second group
non-optional in the returned type) is the properly typed answer and stays a
reasonable thing to decline; this is the cheap 90%.

### Minor 3: 18.1 and 18.2 report the IF's line where the oracle reports the directive's

The one line disagreement in the 385-program sweep. Measured:

```
nop
<blank>
if 1 = 1
<blank>
::routine r
  -> Error 18 ... line 5:  THEN expected.
     Error 18.1:  IF instruction on line 3 requires matching THEN clause.
```

The parser reports **line 3**. At end of file instead of a directive, the oracle
also reports the IF's line (`nop` / blank / `if 1 = 1` is line 3, matched), so
the convention differs only when a `::` clause is what ends the body: the C++
consumes that clause as the offending one, while `translate_block` breaks out of
the clause loop before `parse_instruction` sees it and falls into the
no-offending-clause branch, which uses the IF's byte. Same for 18.2. Within
criterion 4's "plausible line", so no fix is required, but the test that pins
these three shapes (`block/tests.rs:282`-`284`) uses `err` rather than `err_at`
and its comment says "the same numbers come out" -- add that the *line* does not,
and why, so the next reader does not tighten it to `err_at` and get a surprise.

### Minor 4: `directive/tests.rs` duplicates `lib.rs`'s body-slot dispatch

`parse_with_symbols` grew a `wants_body` match plus a `set_body` helper, 20 lines
that reimplement `lib.rs:277`'s `directive_body`. `directive_body` is a private
item of the crate root and is therefore visible to descendant modules as
`crate::directive_body`, so the test can use the same
`if let Some(slot) = directive_body(&mut directive.kind) { *slot = translate_block(..)? }`
shape the real composition uses and drop both helpers. That also removes the
`panic!("cannot hold a body")` arm, which is a second place a new
body-carrying directive kind would have to be remembered.

### Minor 5: the report's 14.3/14.4 routing note is right but understated

`block.rs:429`-`435` says the reported line "comes out the same either way"
because the C++ reports against the clause holding the THEN and this reports
against the THEN instruction. I confirmed both spellings and both terminators:
same-line THEN (line 3), own-line THEN (line 5), directive-terminated (line 5),
and 14.4 likewise. Worth noting in the comment that the equality depends on the
THEN's `clause_span` being exactly the keyword, which is Task 3.4's split, so a
change there would break this silently.

---

## Things I checked and found correct

* **The jump-target semantics against the C++.** `If::false_target` is
  `else_location->nextInstruction`, and `flush_control`'s IfThen arm computes it
  as `next_index()` *after* appending the branch's last instruction, which is
  exactly where the synthetic `ENDTHEN` marker's `nextInstruction` points.
  `Else::then_exit` is computed after appending too, matching the C++'s order in
  `flushControl`'s ELSE arm (`addClause(instruction)` then `addClause(marker)`),
  so it lands on the instruction after the ELSE branch and not on the marker.
  `When::exit = end + 1` matches `fixWhen`'s `else_end->nextInstruction`.
  `resolve_targets` clearing `>= len` is right because `end + 1 == len` is the
  only way to exceed it.
* **The must-be-first check order.** `exposeNew` and `useLocalNew`
  (`InstructionParser.cpp:2316`, `:2349`) check interpret, then placement, then
  the list, and `useLocalNew` calls `autoExpose()` before reading the list. The
  port reproduces all four orderings, including the `USE LOCAL` with no names
  still seeding the five specials.
* **The `EXPOSE (a)` asymmetry.** `processVariableList` calls `expose()` only on
  the direct symbol path, and the port matches. Measured 99.913.
* **Error precedence with two errors available.** `do` / `if 1 = 1` at EOF is
  18.1 not 14.1; `do` / `if 1 = 1 then` is 14.3; `do` / `lab:` is 47.2; a nested
  END-name mismatch inside an unclosed outer DO is 10.2 on the inner END's line.
  All four match the oracle, which means the 18-before-block_error ordering in
  the end-of-body branch is the right way round.
* **The WHEN-as-THEN quirk and eleven other accept-shapes**, including
  `select` / `when 1 = 1` / `then nop` / `end`, `select case 1` with a
  multi-element WHEN and an OTHERWISE, and `expose a` / `::method m` /
  `expose b`. All rc 0 in both.
* **No em-dashes and no structuring semicolons** in the new comments.
* Ruling 1 (`parse_instructions` gone, `parse_instruction` takes `&mut Block`),
  ruling 2 (no `next`), ruling 3 (no `EndIf` nodes; `instructions` stays one node
  per clause), ruling 4 (10.5/10.6 omitted with the probes recorded in the test
  itself), ruling 5 (`body: Option<CodeBody>`) are all implemented as ruled.

---

## Gate criterion 4 audit

The criterion reads:

> For every **parse-time** error the parser raises: the **number and sub-number**
> match the oracle, on a **plausible line**. [...] Message text and substitution
> values are deliberately NOT gated. [...] "Parse-time" is defined by `rexxc`,
> not by judgement: an input `rexxc` rejects is a parse error and belongs here;
> an input `rexxc` accepts is not.

Three separate problems, in descending order of importance.

**(a) It quantifies in one direction only, and the wrong one.** "Every error the
parser raises" is a *soundness* condition: it forbids raising a wrong number, and
says nothing about failing to raise a right one. A parser that accepted every
input would satisfy it vacuously, having raised no errors. That cannot be what
was meant, because the criterion's own note says the reason it could not be met
before this task is that "until 3.7c lands the parser accepts programs the oracle
rejects" -- a *completeness* statement. The wording and the stated rationale are
about different properties. This is precisely the Phase 2 failure mode the brief
warns against: a criterion written so that the capability it exists to gate is
not what it tests. And it is not hypothetical here -- Important 1's eight
compound-`GUARD` shapes are exactly errors the oracle raises and the parser does
not, and they sit entirely inside this blind spot.

**(b) The set is not enumerated, so as written the criterion is not checkable.**
"Every parse-time error the parser raises" ranges over all inputs. Nobody has
written down the comparison set. What actually exists, and what I measured, is
the crate's own error corpus: **385 distinct programs** the tests assert an error
on. Against `rexxc`: **384 agree on number and sub-number, 383 also agree on the
reported line.** The two exceptions are known:

* `if 1 = 1` / `then: nop` (and its `select` / `when 1 = 1` / `then: nop` / `end`
  spelling) is **18.1/18.2 here and 35.1 in the oracle**. Both reject; Task 3.6
  recorded this as an accepted deviation with a test whose whole purpose is to
  keep it visible (`a_label_after_an_if_is_rejected_by_the_label_guard`). Read
  literally, the criterion is **false** because of it.
* the directive-terminated 18.1 line, Minor 3, which "plausible line" permits.

**(c) The definition of "parse-time" mis-classifies non-translation
rejections.** "An input `rexxc` rejects is a parse error" makes
`::routine r external "LIBRARY x"` a parse error, because `rexxc` rejects it with
**98.903 "Unable to load library"**. That is a load failure after translation,
its outcome depends on what is installed on the machine, and the parser is right
to accept it. Two inputs in the crate's own accept-corpus have this shape. Under
today's one-directional wording they are harmless; under a fixed two-directional
wording they would be counted as misses, so the carve-out has to be written at
the same time.

### What it should say

Replace the criterion with two directions over a named corpus, and list the
exceptions by number and reason:

> **Soundness.** For every input in the parser's error corpus -- the programs the
> crate's own tests assert an error on, currently 385, extractable by
> instrumenting the test helpers -- the number and sub-number match
> `build/bin/rexxc` on a plausible line. Message text and substitution values are
> not gated. Recorded exceptions, each with a test that pins both directions:
> * `if 1 = 1` / `then: nop` and its `WHEN` spelling: 18.1/18.2 here, 35.1 in the
>   oracle. Both reject. Reason in Task 3.6's report.
>
> **Completeness.** For every input in that corpus, plus the 301 `samples/`
> files, `CoreClasses.orx` and `StreamClasses.orx`, the parser accepts exactly
> what `rexxc` accepts. Exception: a rejection that is not a translation error --
> `98.9xx` load failures such as `::ROUTINE ... EXTERNAL "LIBRARY x"`, currently
> two inputs -- is not a parse error and must be accepted.
>
> **"Parse-time"** means: `rexxc` rejects it *and* the failure is a translation
> error. Bare `procedure` (17.1), bare `leave` (28.1) and `x = 1/0` (42.3) still
> get rc 0 from `rexxc` and are still Phase 4's, and this parser must accept
> them.

Two things make that version checkable where the current one is not: the corpus
is named and reproducible, and the exceptions are enumerated rather than being
resolved by how loosely a reader reads "plausible" and "raises".

### The answer

**Not met** -- because (i) read literally it is falsified by the recorded
18.1/18.2-versus-35.1 deviation, and (ii) the property it is trying to state,
that the parser rejects what the oracle rejects, is not the property it states,
and that property is also currently false: the eight compound-`GUARD` shapes in
Important 1 are accepted here and rejected by the oracle.

**It would be met if reworded as above and Important 1 is settled** -- either by
porting `addCompound`'s cache behaviour, or by listing the compound-`GUARD` case
as a third recorded exception with a pinning test. The implementer was right to
decline to declare it met, and right about the reason as far as it goes; the
larger problem is the direction the sentence quantifies in, not only the size of
the set.
