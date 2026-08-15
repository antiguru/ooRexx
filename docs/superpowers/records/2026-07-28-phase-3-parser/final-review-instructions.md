# Final whole-branch review, Phase 3: the 35 keyword instructions

Reviewer slice: `rust/crates/rexx-parse/src/instruction.rs`, `instruction/tests.rs`,
`tests/program.rs`. Read-only review; oracle probes via `build/bin/rexxc` under
`ulimit -v 1048576`.

Status: COMPLETE. 178 differential probes run (scratchpad/kwprobes), Rust
side via a scratchpad-only `pcheck` CLI over the crate's public
`parse_program`, C++ side via `build/bin/rexxc -s` under `ulimit -v 1048576`.
Result: 1 Important finding (F1), 1 documented deviation confirmed (F2),
2 Minor citation nits (F3). Error numbers bulk-verified against
`RexxErrorCodes.h`, both directions, clean.

## Findings so far

### F1 (Important): `DO WHILE` / `DO UNTIL` with an empty condition raise the wrong sub-number

`instruction.rs:1285` (`loop_conditional`):

```rust
let condition = self.logical(Terminators::COND, if until { 909 } else { 908 })?;
```

The `missing` argument of `logical()` is the sub-number raised when the
condition is EMPTY. But in the C++, `parseLogical` (LanguageParser.cpp:4287)
raises `Error_Invalid_expression_logical_list` (35.929) itself the moment any
sub-expression comes back null, so `requiredLogicalExpression`'s per-caller
error (35.908 WHILE / 35.909 UNTIL) is dead code -- unreachable for every
caller. Oracle-confirmed:

- `do while`  + `end`: rexxc says **Error 35.929**, Rust parser says 35.908.
- `do until`  + `end`: rexxc says **Error 35.929**, Rust parser says 35.909.

The module itself knows this pattern: the IF arm passes 929 with a comment
saying exactly why ("929 ... which parseLogical raises before
requiredLogicalExpression's own 35.902 or 35.903 can be reached",
instruction.rs:2584), and GUARD passes 929 too. `loop_conditional` is the one
logical() call site that missed the rule. No test pins either number, so the
suite is green over the divergence. Same code 35 both sides, acceptance
identical, so run-time effect is limited to the `condition('o')~code` a
SIGNAL ON SYNTAX handler sees -- but by the project's own rule a wrong number
is a defect. Fix is passing 929 at this site (both WHILE and UNTIL arms).

### F2 (Minor, deliberate and documented): label spelled THEN after IF

`if 1 = 1` + `then: nop` is E35.1 under the oracle, E18.1 (E18.2 for WHEN)
in Rust. Documented at instruction.rs:238-246 with the architectural reason
(Task 3.4 splits the label clause before the pending-THEN test runs) and
pinned by `tests::a_label_after_an_if_is_rejected_by_the_label_guard`. Both
sides reject the program; only the number differs. Confirmed by probe k7.
Surfaced for ratification because the project rule says a wrong number is a
defect, and this one is knowingly wrong rather than accidental.

### F3 (Minor): two definition-position citations point mid-body

- instruction.rs:1265 cites `parseLoopConditional`
  (`InstructionParser.cpp:4600`); the definition is at 4582, 4600 is inside
  the body.
- instruction.rs:1393 cites `parseRedirectOutputOptions`
  (`InstructionParser.cpp:812`); the definition is at 795, 812 is inside the
  body.

Neither is the expr.rs defect class (a line inside a DIFFERENT function);
both land inside the named function. All other 33 citations in the slice are
exact or deliberate statement citations.

## Checks performed

- Read all 2711 lines of `rust/crates/rexx-parse/src/instruction.rs`. Oracle
  tree is in-repo (`interpreter/`), codes header is
  `interpreter/messages/RexxErrorCodes.h` with entries of the form
  `Error_Name = 27902` (code*1000+sub), so bulk verification is mechanical.

### Error numbers, bulk (priority 3) -- CLEAN

Two mechanical passes over `instruction.rs`, `instruction/tests.rs`,
`tests/program.rs` (scripts in scratchpad, `check_errors.py`, `check_pairs.py`):

1. Every `Error_X` name mentioned in a comment vs the numbers stated or raised
   within a window around it, against `RexxErrorCodes.h`. Five flags, all
   benign on manual inspection: window-size artifacts (99.913 raised 14 lines
   below its naming comment; 35.915 as the `expr()` missing-sub one line down;
   35.929 as `logical()` sub), `Error_None` cited descriptively for a C++
   unreachable arm, and `Error_Interpretation_switch` = 49.2 matching the
   `UNREACHABLE_SWITCH` constant. No real mismatch.
2. Every `(code, sub)` pair the module can raise -- `self.error`,
   `ParseError::new`, `expr`/`logical` missing-subs (implicit 35),
   `required_end`, `forward_option`, `variable_list`, `block_name` -- exists in
   the header as some `Error_` constant. Zero missing.

This closes the check the prior run was starting. Existence and name-adjacency
verified; context-correctness of unnamed sites still rests on the "Measured"
claims, a sample of which is probed below.

### PARSE (priority 1) -- line-by-line vs C++ `parseNew` -- CLEAN so far

Compared `parse_instruction_body`/`parse_template`/`trigger_position` against
`LanguageParser::parseNew` (InstructionParser.cpp:3102-3440) clause by clause:

- Option loop: UPPER/LOWER mutually exclusive, CASELESS independent, each
  once; a duplicate falls through to the source switch and dies as 25.12
  (`Error_Invalid_subkeyword_parse`). Rust loop reproduces the fallthrough
  exactly (duplicate breaks with the option, which is not a source).
- Sources: ARG/LINEIN/PULL/SOURCE/VERSION bare; VAR wants a symbol (20.904,
  `Error_Symbol_expected_var`, header-confirmed) then `addVariable` =
  `need_variable`; VALUE takes an optional expr (default nullstring) then
  requires WITH (38.3). All match, including bare `parse` = 20.903.
- Template loop: EOC/comma emit End trigger only when targets pending; the
  five operator triggers `+ - = < >` map to Plus/Minus/Absolute/MinusLength/
  PlusLength; unknown operator 38.1; `(expr)` string trigger with
  caseless->Mixed; literal string trigger; numeric symbol = absolute
  (C++ `isNumericSymbol()` is exactly `subclass == SYMBOL_CONSTANT`, which is
  Rust `SymbolClass::Constant` -- Token.hpp:582 checked, so `1.2.3`-shaped
  symbols take the same route in both); lone `.` placeholder; anything else
  through `parseVariableOrMessageTerm` else 89.2; final else 38.1.
- Trigger position: `(expr)` (35.931 on empty), symbol rejected iff
  `isVariable()` = Variable|Stem|Compound (38.2, Token.hpp:572 matches Rust
  exactly, so `.foo` and `.` after `+` are ACCEPTED by both), EOC 38.901,
  other 38.2. All match.

### Differential harness

Built a scratchpad-only CLI (`scratchpad/pcheck`) that calls the crate's public
`parse_program` and prints `ok` or `E<code>.<sub>`, plus `diffrun.sh` which
runs each probe under both `build/bin/rexxc -s` (ulimit-wrapped) and the Rust
parser and flags divergences. Nothing in the repo touched.

PARSE probes, all AGREE (cpp = rs): `parse arg <3 a` ok, `parse arg >(x) a`
ok, `parse upper lower arg a` E25.12, `parse arg +. a` ok, `parse arg 1.2.3 a`
ok, `parse arg =x a` E38.2, `parse lower caseless value "A" with a` ok,
`parse caseless upper arg a` ok, `parse arg a . b , c d` ok,
`parse arg +.foo a` ok.

### DO/LOOP (priority 2) -- one finding (F1), otherwise CLEAN

Compared `create_loop` and its five helpers against C++ `createLoop`
(InstructionParser.cpp:1994-2196), `newControlledLoop` (1265),
`newDoOverLoop` (1432), `newDoWithLoop` (1582), `parseForeverLoop` (1860),
`parseCountLoop` (1916), `parseLoopConditional` (4600). Routing is
identical, including: LABEL/COUNTER prefix loop with repeat-breaks; `=` after
LABEL/COUNTER making a controlled loop over that name; `==` as 35.1
(`Error_Invalid_expression_general` = 35001, header-confirmed); OVER checked
before WITH; bare-DO-with-COUNTER as 27.905
(`Error_Invalid_do_simple_do_counter`, header-confirmed); WHILE/UNTIL
entering the conditional directly; count-loop fallback. Terminator sets
CONTROL/COND/OVER/IF/PARSE_WITH in expr.rs match Token.hpp:521-538 defines
bit-for-bit in membership.

25-case differential probe batch (d1-d25 in scratchpad/kwprobes): 23 agree,
including `do label l = 1 to 2` = E35.1 both, `do with over x` ok both,
`do i.1 = 1 to 2` ok both (compound control variable legal), evaluation-order
`do i = 1 for 2 by 3 to 4 while ...` ok both, `do with index i` E27.904 both,
duplicate INDEX E27.902 both, `do i over x by 1` ok both (BY absorbed into the
OVER expression by both, same terminator sets). The 2 disagreements are F1.

### CALL/SIGNAL/SELECT (priority 5) -- CLEAN

22 differential probes all agree, including: `call on any` and `signal on any`
ok both; `call on lostdigits` E25.1 vs `signal on lostdigits` ok; `call off
user hup` ok; `call off propagate` E25.2; `signal value` E35.915; `signal
1+1` E21.905; dynamic `call ("FOO") 1,2` ok; `select label 1` ok both (any
symbol class is a legal label name -- do not add a class check); `select
case` E35.933; `select label s extra` E25.923.

### ADDRESS/NUMERIC/TRACE/OPTIONS/INTERPRET/RAISE -- CLEAN

28 probes all agree, including `address system with input normal input
normal` E25.930, `with error using x` E35.1 (constant-expression form),
`with input stem a` E20.932 (non-stem), `numeric form value` E35.917,
`trace 99999999999` E24.1 (overflowing the 9-digit whole-number conversion
falls through to the setting check in both), `trace -a` E26.7, bare
`options`/`interpret` E35.913/E35.912, `raise syntax` E35.1, `raise any`
E25.906, ADDITIONAL+ARRAY exclusion E25.909.

### USE/GUARD/FORWARD/DROP/EXPOSE/PROCEDURE/misc -- CLEAN

37 probes all agree, including: `use arg ..., a` E99.930, `use arg >a = 1`
E99.950, `use arg a b` E46.902, `use local a.b` E99.948, `use arg q~x` ok
(message-term target); GUARD family: `guard on when` E35.929 both (the
logical-list rule, correctly applied HERE unlike F1's site), `guard on when
1` in a method E99.913, `expose (a) b` + `guard on when a` E99.913 both
(indirect expose does not count for GUARD); `forward to 1 to 2` E25.917;
`drop .5` E31.2 vs `drop (.5)` E31.3 (class test vs spelling test, both
directions); `procedure foo` E25.17; `nop x` E21.901; `end "x"` E20.909,
`end a b` E21.909; `leave x y` E21.907; `iterate 1` ok both (any symbol is a
block name at parse time).

### C++ citations (priority 4) -- no wrong-function citations; two mid-body line numbers

Mechanically extracted all 35 `` `func` (`File.cpp:NNN`) `` citations in
`instruction.rs` + `instruction/tests.rs` and checked the named function
appears at the cited line. 30 exact. The 5 flagged all land INSIDE the named
function (checked by reading the C++), so the expr.rs defect class (right
name, line in a different function) is absent here:

- `parseLoopConditional` cited InstructionParser.cpp:4600; definition is 4582,
  4600 is its isSymbol dispatch. Definition-position citation pointing
  mid-body: Minor.
- `parseRedirectOutputOptions` cited :812; definition 795, 812 is its switch.
  Minor.
- `RexxToken::parseOption` cited KeywordConstants.cpp:551; definition 556, 551
  is its own doc comment. Fine.
- `addCompound` cited LanguageParser.cpp:2153: deliberate statement citation
  (the `addStem` lookup inside addCompound). Fine.
- `processVariableList` range :4487-:4496: the class-test statements, correct.
  Fine.

### Keywords-are-not-reserved (positional recognition) -- CLEAN, one documented deviation confirmed

16 probes: `if = 2; say if`, `do = 1`, `loop = 1`, `parse = 3`, `when = 1`,
`use = 5`, `signal += 1`, `end. = 1; say end.2`, `if.1 = 5`,
`if if = 2 then say if` all ok both sides. `select` + `when = 1 then nop` +
`end` is E7.2 both (the WHEN became an assignment, so the SELECT is empty).
`when: nop` inside select E47.4 both. `if 1 = 1` + `then = 7` E35.1 both
(THEN consumed as the pending THEN, `= 7` left as a command clause).

Confirmed deviation (documented at instruction.rs:238-246, pinned by
`tests::a_label_after_an_if_is_rejected_by_the_label_guard`): `if 1 = 1` +
`then: nop` is E35.1 under the oracle but E18.1 in Rust, because Task 3.4
splits the label clause before the pending-THEN check can see a THEN-spelled
label. Both reject; the number differs by design. Flagged as Minor for the
coordinator to ratify since the project rule is "a wrong number is a defect"
and this one is knowingly wrong.

### Interpret-mode guards -- CLEAN

C++ has exactly six parse-time interpret rejections (grep `isInterpret()` over
parser/*.cpp): label 47.1, expose 99.908, use local 99.915, forward 99.923,
guard 99.912, reply 99.924. Rust has the same six with the same numbers, all
header-confirmed. PROCEDURE is not parse-guarded in either.

### GUARD guard_exposes deep probes -- CLEAN

The compound-cache rule (an EARLIER reference to the same compound spelling
disqualifies a later `GUARD ON WHEN` on it) is real in the oracle and
reproduced: `expose a.` + `x = a.1` + `guard on when a.1` is E99.913 both
sides, while without the earlier reference it is ok both sides. Tail-variable
exposure (`expose i` + `guard on when a.i`) ok both sides. Indirect
`expose (a)` does not satisfy GUARD, E99.913 both sides.

### Family consistency (priority 6) -- CLEAN

Shared sub-syntax goes through shared routes: `variable_list` for
DROP/EXPOSE/PROCEDURE EXPOSE (per-instruction 20.sub as in C++
`processVariableList`); `forward_option` for FORWARD TO/CLASS/MESSAGE and
RAISE DESCRIPTION (constant-expression rule identical to C++, both probed);
ARRAY parsing duplicated in `forward()` and `raise()` but literally identical
(35.924 + paren arg list), matching the C++ which also duplicates it;
`USER name` composition duplicated in `raise()` and `condition_trap()`, both
composing `USER ` + name with 20.915, as the C++ does in two places;
implicit-VALUE pattern (token left in place) used the same way by ADDRESS,
TRACE, SIGNAL, NUMERIC FORM. `useNew`'s comma/default/alias handling read in
full (InstructionParser.cpp:4267-4460): trailing comma adds no slot, `,,`
adds one, alias accepts Variable|Stem only -- all as in Rust.

### Coverage

All 35 keyword instructions were read in the Rust source. Depth per
instruction: PARSE/ARG/PULL, DO/LOOP, CALL, SIGNAL, ADDRESS, NUMERIC, TRACE,
GUARD, FORWARD, RAISE, USE, EXPOSE, PROCEDURE, DROP, SELECT, IF/WHEN/THEN,
INTERPRET, OPTIONS, REPLY got line-by-line C++ comparison and/or multiple
differential probes. SAY/PUSH/QUEUE/RETURN/EXIT/NOP/LEAVE/ITERATE/END/ELSE
are trivial forms, each probed at least once (ok + error direction where one
exists). OTHERWISE was exercised only via the s4 probe (accepted in place);
its no-parse constructor matches `otherwiseNew` by inspection. Block ASSEMBLY
(END matching numbers 10.x, ELSE/OTHERWISE placement, 14.3) belongs to
`block::translate_block`, another reviewer's slice; probes m4/m5/k11/k14
touched it incidentally and agreed. `instruction/tests.rs` was scanned
mechanically for error-number claims (clean) but not re-derived test by test.

