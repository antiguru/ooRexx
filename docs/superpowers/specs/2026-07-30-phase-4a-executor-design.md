# Phase 4a executor design

**Goal:** run a classic Rexx program that has no procedures, no `PARSE` and no builtin calls, byte-for-byte as the oracle runs it, and fix the execution model while doing it.

**Status:** revision 5, 2026-07-30.
Revision 5 fixed five more Criticals, of which the sharpest is that revision 4's *new* comparison rule was wrong: the oracle strips leading blanks and tabs and decides a leftover tail against a space, and none of revision 4's eight measured cells could tell that from the padding rule it stated. The correct algorithm was already ported in this workspace, so the spec had been directing an implementer to hand-write a divergent second copy. Also fixed: the assertion table's CONCATENATION rows would have passed while testing nothing, D16's growth invariant is false in 4b, the raiser families were a closed list missing major 33 and 98.913, and D19 claimed to record three numbers while recording none.
Revision 4 came from a fresh reader with no prior context, asked only whether the document stands on its own. It found two things three correctness passes had not: the ordinary comparison algorithm (`=`, `>`, `<`) was **absent entirely**, while its strict sibling was spelled out in three examples, and `Settings` was owned by `Interp` in one section and by each activation in another. Both are now measured and stated, along with the logical-value rule, multi-level tail keys, when a plan is built, and where the message catalogue comes from.
Revision 1 was reviewed by two independent agents and did not survive: 16 Critical, 16 Important and 7 Minor findings, plus two wrong claims.
Revision 2 fixed all of them and introduced six new Critical findings of its own, which is this project's measured base rate for fix rounds: `NUMERIC FORM` is captured at creation exactly as `DIGITS` is and revision 2 fixed only `DIGITS`; a hash-mapped stem cannot reproduce `DO OVER`'s traversal order; the coverage criterion added to close a blindness finding demanded a witness for a loop form that needs Phase 5; D19's interpreter thread could not be handed an `!Send` program; its stack-sizing rule pointed at the side that *creates* a divergence; and its two variable-pool bullets, written for two different prior findings, could not both hold.
Every measured transcript below was re-run for this revision.
The findings that changed the design are named at the point they changed it, because a spec that hides its own corrections invites the same mistake twice.

This document is the spec for Phase 4a alone.
Phases 4b and 4c get their own spec and plan, and this document defines their boundaries so that nothing falls between them.

## Scope of this document

Phase 4 in the parent plan (`docs/superpowers/plans/2026-07-27-rust-rewrite.md`, section 2) is one row: "Non-OO Rexx runs: assignment, `DO` (all variants), `IF`, `SELECT`, `CALL`, `PARSE`, `SAY`, `SIGNAL`, conditions, and all 81 builtin functions."
That is larger than Phases 0 to 3 combined, so it is split into three sub-phases.
Each produces software that runs, and each closes against its own named corpus.

## What phases 1 to 3 hand over

Verified against the tree at commit `9f68662a`.
Three of these are not settled interfaces but things 4a must change, and they are marked.

From `rexx-core` (628 lines of `src`):

* `ObjRef`, a tagged 64-bit handle: `Decoded::Heap { slot: u32, generation: u32 }`, `Decoded::SmallInt(i64)`, `Decoded::Nil`.
  The inline integer range is asymmetric: `SMALL_INT_MIN` is `-(2^61)` and `SMALL_INT_MAX` is `2^61 - 1`.
* `Heap` with `alloc(Body) -> ObjRef`, `alloc_with(BehaviourId, Body)`, `get`, `get_mut`, `collect(&RootSet) -> CollectStats`, `live_count()`.
* `Body`, currently `String(String) | Array(Vec<ObjRef>) | Instance(Vec<(String, ObjRef)>) | WeakRef(ObjRef)`, with an exhaustive `trace` and no wildcard arm.
  **4a changes this enum**, which means 4a edits `rexx-core`; see D15.
* `RootSet` holds exactly `globals: Vec<(String, ObjRef)>` and `temps: Vec<ObjRef>`, and `iter()` yields those two.
  **There is no place in it for an activation's variables**; see D16.
* `BehaviourTable` with `define`, `set_superclass`, `lookup`, and `BehaviourId::{STRING, ARRAY, OBJECT}`.

From `rexx-num`:

* `Number::parse(&str) -> Option<Number>`, `format(u64) -> String`, **`format_form(digits, Form) -> String`**, `format_with`, `trunc`, `round_to`, `whole_value(usize) -> Option<i64>`, `is_zero`, `zero`, `one`.
  `format_form` is the one D15 needs; listing only `format(u64)` in revision 2 is what steered its rendering rule into capturing half the state.
* `add`, `sub`, `mul`, `div(.., DivOp)`, `pow`, each `(&self, &Number, digits: u64) -> Result<Number, ArithError>`.
* `compare` is a **free function** over string operands taking both `digits` and `fuzz`, not a method on `Number`.
* `Settings` with `digits()`, `fuzz()`, `form()`, `set_digits_str`, `set_fuzz_str`, `set_form_str`.
* `ArithError` and `FormatError`, each with `code()`, `additional()`, `message()`.

From `rexx-parse`:

* `parse_program(Vec<u8>) -> Result<Program, ParseError>` and `parse_interpret(Vec<u8>) -> Result<Fragment, ParseError>`.
* `Program { source, instructions, directives, labels, symbols }` and `Fragment { source, instructions, symbols }`.
* `CodeBody { instructions, labels }` exists **only per directive body**.
  The main program body is not a `CodeBody`, and `Fragment` has no label table at all.
  **4a changes this**; see "The borrow shape".
* Instruction lists are flat and source-ordered, and every jump target is a `usize` index into the same body's `instructions`.
* Every node carries a byte range into the retained source.
  Phase 3's gate records that the containment property is vacuous by construction, because `Expr::new` widens a node's span over its children, so a span can be too wide and no parser test can tell.
* `compound_parts(&str) -> (&str, Vec<Tail>)`, with `Tail::Constant` standing for itself and `Tail::Variable` supplying its value.

Phase 3's four handovers bind this phase: `resolveCalls` (`LanguageParser.cpp:1690`) is not ported, `TRACE`'s value lines are Phase 4's, `Message` names want an intern table, and nothing in Phase 3 observes AST shape.

## The split

| | Deliverable | Closes against |
|---|---|---|
| 4a | Value model, variables, expression evaluation, control flow. `Assignment`, `Label` (a traced no-op), `Say`, `Nop`, `Do`/`Loop` in every variant **except `WITH`**, `If`/`Then`/`Else`, `Select` including `SELECT CASE`, `When`, `WhenCase`, `Otherwise`, `Leave`, `Iterate`, `End`, `Drop`, `Numeric`, `Trace`, `Exit`. | A named L0 subset plus a table-driven `base/expressions` harness |
| 4b | `Call`, `Return`, `Procedure`, `Use`, `Signal` in all three forms, condition traps, `Raise`, `Interpret`, `Push`, `Queue`, and the in-process queue. Routine resolution, which is handover 1. | The `base/keyword` L1 groups |
| 4c | `Parse`, `Arg`, `Pull`, the `Address` instruction's environment-name tracking, and the 66 in-scope builtins, ticked off one at a time. | The `base/bif` L1 groups, 66 rows |

The parent plan's Phase 4 row closes when 4c closes.

Explicitly assigned elsewhere, so that no instruction is merely absent: **`LoopKind::With` is Phase 5's**, because `DO WITH ... OVER` sends `SUPPLIER` to its target and nothing in 4a answers a message — measured, `do with index i item v over 'abc'` is `Error 97.1`, "Object \"abc\" does not understand message \"SUPPLIER\"", rc 159; `Expose` and `Options` are Phase 5's, since neither is reachable outside a method or a package context; `Message`, `Guard`, `Reply`, `Forward`, every directive, and environment symbols beyond `.nil`, `.true` and `.false` are Phase 5's; `Command` dispatch and the rest of `Address` are Phase 7's under D18.
`Label` is 4a's rather than 4b's even though nothing jumps to one in 4a, because a label is a traced no-op that any program may contain, and making it fail loudly would silently exclude every labelled program from the gate's subset.

## Decisions

New decision blocks for section 1 of the parent plan, numbered after D14.

### D15 value representation

**Decided: mirror the oracle's representation, including where a rendering comes from.**

Rexx makes the difference between a value that came from text and a value that came from arithmetic observable:

```
x = '007'   ;  say x  ->  007        ;  say x + 0  ->  7
```

Revision 1 got the second half of this wrong in a way that would have produced wrong output everywhere.
**A number's rendering is fixed when the number is created, under the `NUMERIC DIGITS` in force at that moment, and never afterwards.**
Measured:

```
numeric digits 9 ; y = 1 / 3
numeric digits 3 ; say y            ->  0.333333333      (not 0.333)
                   z = 1 / 3 ; say z ->  0.333

numeric digits 9 ; x = 1e10 + 0 ; say x  ->  1E+10
numeric digits 20; say x                 ->  1E+10        (unchanged, forever)
                   y2 = 1e10 + 0 ; say y2 ->  10000000000
```

`x` and `y2` are the same numeric value and render differently, permanently.
The oracle stores this on the value as `NumberStringBase::createdDigits`.
So the representation is:

* `Body::Text { bytes: Vec<u8>, num: Option<Result<Box<Number>, NotNumeric>> }` for a value whose identity is its bytes.
  The cache is tri-state on purpose: `None` is "not yet asked", and the two `Result` arms distinguish "is this number" from "is not a number", so a non-numeric string does not re-run `from_utf8` and `Number::parse` on every comparison.
  `NotNumeric` is a `rexx-core` type, since `Body::Text` is, and it is the same amendment task.
  **The cache holds the exact parse and is never rounded at fill time.**
  Rounding belongs to the operation, which is why the cache is safe across a settings change: measured, `x = '1.234567890123456789'` gives `1.2346` under `DIGITS 5` and `1.234567890123456789` under `DIGITS 20`, from the same stored parse.
* `Body::Num { value: Number, created_digits: u32, created_form: Form, text: Option<Vec<u8>> }` for a value whose identity is its number.
  `text` is formatted with `format_form(created_digits, created_form)`, so it caches a pure function of the object and **is never invalidated**.
  Revision 1's rule, "invalidated on any settings change", produces exactly the divergence it claimed to prevent.
  `NUMERIC FORM` is captured at creation exactly as `DIGITS` is, which revision 2 missed while fixing `DIGITS`: the same bug, one field over. Measured:

  ```
  numeric form engineering ; x = 1e10 + 0 ; say x  ->  10E+9
  numeric form scientific  ;               say x  ->  10E+9      (unchanged)
                             y = 1e10 + 0 ; say y  ->  1E+10
  ```

  An implementation formatting with `settings.form()` prints `1E+10` for `x`.
* `ObjRef::SmallInt`, admissible only when the exact result is whole, inside the tag's range, **and** its decimal digit count is at most the `DIGITS` in force at that operation. It renders as plain decimal, which under that condition is correct by construction.
  It is form-independent: under the digit-count condition the rendering carries no exponent, so `engineering` and `scientific` agree and no `created_form` is needed on the tag.
  **The admissibility check happens once, when the value is created, and is never re-derived from the current settings** — the same "fixed at creation" rule as `Body::Num`, stated rather than left to be inferred by analogy, because the tag visibly has no room to store what it was checked against.
  The narrow condition is not fussiness. Measured under `numeric digits 1`:

  ```
  x = 15 + 0 ;  say x   ->  2E+1
  say x + 6             ->  3E+1      /* x is 20, so 26 -> 3E+1 */
  say 15 + 6            ->  2E+1      /* 21 -> 2E+1 */
  ```

  An implementation that keeps `SmallInt(15)` because 15 "fits the tag" answers `2E+1` for `x + 6`.
  And two `SmallInt`-shaped results can differ observably: `a = 20 + 0` at `DIGITS 9` renders `20`, `b = 15 + 0` at `DIGITS 1` renders `2E+1`, `a = b` is 1 and `a == b` is 0.
* `Decoded::Nil` is a value with no bytes. Measured, `say .nil` prints `The NIL object`, so string conversion special-cases it and every accessor branches on `decode()` before assuming a heap object. `.true` and `.false` need no representation at all: they are the one-byte strings `1` and `0`, and `.true == 1` is 1.

`Body::String(String)` is deleted: it holds a UTF-8 Rust `String`, and D14 closed on byte strings, so it cannot hold `reverse('ää')`.

**Where these variants live, and what that costs.**
Revision 1 claimed the variants were "private to `rexx-exec`'s value module" and the cost of being wrong "contained".
Both were false: `Body` is `pub enum Body` in `rexx-core`, and its `trace` is the exhaustive match Phase 1's whole GC-safety argument rests on.
So 4a amends `rexx-core`:

* Three variants added, `Body::trace` extended for `Stem`'s reachable values, `Body::String` removed along with the `heap.rs` tests that construct it.
* **`rexx-core` gains a dependency on `rexx-num`**, because `Body::Text` holds a `Number`. That edge did not exist and points the object model at the arithmetic core.
* Together with D16's root-set change, this is **one amendment task against `rexx-core`**, not two discovered ones.

**Cost of being wrong,** stated on the right axis: the constructors and accessors in `rexx-exec::value` are the only callers, so a third representation is a change to that module plus one arm of `Body::trace`. It is not free, and the instruction loop is not what it would disturb.

### D15a stems

Stems are part of the value model and got four things wrong in revision 1, so they are stated separately.
All four are measured:

```
u. = 'd' ; u.1 = 'one' ; drop u.1
say u.1               ->  U.1     /* uninitialised, NOT the default */
say u.2               ->  d       /* the default still applies      */

a. = 1    ; b. = a. ; a.1 = 2       ; say b.1  ->  2      /* one shared object */
r. = 'rd' ; u  = r. ; drop r.       ; say u    ->  rd     /* old object intact */
s. = 'def'; t  = s. ; s. = 'other'  ; say t    ->  def    /* old object intact */

say q.                ->  Q.      /* no default: own name, with the period */
t2 = q. ; say t2      ->  Q.      /* still, once aliased into a variable   */
w. = 'wd' ; say w.    ->  wd

i = 'abc' ; v.i = 'val' ; say v.i v.ABC    ->  val V.ABC
```

Therefore `Body::Stem { name: Box<[u8]>, default: Option<ObjRef>, tails: HashMap<Vec<u8>, Option<ObjRef>> }`:

* The **tombstone** is the `Option` inside the map. A dropped tail is present-and-`None`, which does not take the default; an absent key does.
* The **name** is on the object, because a Stem aliased into a simple variable still renders `Q.` and the reference site supplies no name there.
* `stem. = expr` and `drop stem.` **replace** the Stem object and rebind the variable; they do not mutate in place. A tail assignment does mutate. Both are observable through the aliasing above, so an implementer who deep-copies `b. = a.` answers 1 where the oracle answers 2.
* Tail keys are the tail variable's **value verbatim, case-sensitively**. Revision 1's `b = 2, c = 1` example cannot discriminate this; `i = 'abc'` can.
  Note the deliberate asymmetry with D16, which keys variable *names* by their upcased spelling: a name is upcased, a tail value is not, and the two rules sit in different decision blocks precisely because they are easy to swap.
* A **multi-level tail** resolves each piece and joins the results with a period, which is the one key the map holds. The discriminating evidence is not `a.i.j` equalling `a.1.2`, which a tuple key satisfies too: it is that with `p = '1.2'`, `f.p` and `f.1.2` are the **same variable**, so the key really is the joined string and not a sequence of pieces.

A hash map, where the oracle has a balanced BST whose `memcmp` is 21.6% of stem-heavy runtime and is called only from `CompoundVariableTable::findEntry` (545 lines). This is the one place the rewrite is expected to beat the oracle rather than match it.

**And the hash costs one observable behaviour, which is a deviation and not a detail.**
`DO i OVER stem.` walks the tails in the oracle's tree order. Measured, inserting `1, 2, 3, 10, ZZ, B`:

```
do i over a.   ->   1 B 3 2 ZZ 10
```

That is neither insertion order, nor ascending, nor descending, nor byte order — a `BTreeMap` gives `1 10 2 3 B ZZ` — it is `CompoundVariableTable`'s tree shape.
Reproducing it means reproducing the tree, which is the whole cost the hash map exists to avoid.
**Decision: stem iteration order is not reproduced, and is recorded as a deviation** beside the parse-error-text deviation. The justification is measured rather than an appeal to ANSI leaving the order implementation-defined, since this is the project's first trade of an oracle behaviour for speed. Counted across the tree, and the count itself is the part to distrust: four greps with four patterns produced 6, 5, 2 and, from a reviewer, 7. End-of-line stem targets and list forms like `do number over 1.2, -0.003` defeat different patterns differently, and a precise census needs an AST walk rather than a regex — something 4a itself will be able to do once it runs. What every count agrees on, and what the decision rests on: **zero sites in ooTest, zero in `CoreClasses.orx` and `StreamClasses.orx`**, and a handful in `samples/windows/` under `oodialog/` and `ole/`, against 492, 16 and 154 uses of `DO OVER` overall. Most assign or clear each tail and are order-independent; at least two print in iteration order (`registry.rex`'s `do i over keys.; say keys.i` and `getOleConstants.rex:86`), and both are Windows-only samples that no gate runs.
Two consequences that must be written down rather than discovered: no corpus program may contain `DO OVER` on a stem, even though the oracle's order is deterministic and the corpus rule is only determinism; and criterion 1's `LoopKind::Over` witness is therefore a non-stem target, which is legal — measured, a string and a number each iterate once yielding themselves.
This is a semantic deviation chosen for performance, so it is the kind of call to reverse deliberately rather than by drift: reversing it means porting the tree and giving up the one measured win over the oracle.

### D16 variables, slots, and where the collector finds them

**Decided: a per-body resolution plan keyed by upcased name, cached on `Interp`, with the activation's slot *frame* allowed to grow and the frames owned by an extended `RootSet`.**

Variable lookup is 8.1% of runtime on the realistic mixed benchmark and 32.2% on stem-heavy code (perf profile, 2026-07-25); the oracle's answer is integer slots assigned at parse time (`RexxLocalVariables`, 602 lines).
Phase 3's AST carries `SymbolId` and no slots, so assignment moves to first execution. Four corrections to revision 1:

* **Keyed by upcased name, not `SymbolId`.** One `HashMap<Box<[u8]>, usize>` per plan, through which both a `SymbolId` and a compound's tail pieces resolve. Tail pieces must land on the *same* slot as a same-named variable elsewhere in the body: measured, `b = 2; say a.b` gives `A.2`, and after `a.2 = 'hit'`, `say a.b` gives `hit`. An implementer who gives tail pieces their own slots gets `A.B`.
* **The slot frame grows, and the activation records the name.** `DROP (v)` is in 4a's list and names its target at run time: measured, `v = 'X'; x = 1; drop (v); say x` prints `X`. A name that resolves to no existing slot allocates one, so a frame starts at the plan's length rather than being exactly it, and the growth happens in the storage the next bullet places in `RootSet`.
  Revisions 1 through 6 all said the frame grows and none said **where the grown slot's name is recorded**, which is a hole Task 3's pre-flight found by running the case rather than reading the text. The plan cannot record it: it is an `Rc`, shared and immutable, and it was built by an upfront pass that never saw the name. So the **activation** carries `extra: HashMap<Box<[u8]>, usize>` beside its `Rc<Plan>`; resolution is `plan.slot_of(name).or_else(|| extra.get(name))`; allocation writes both `extra` and `grow_slots`.
  This is not a `DROP (v)` special case. Measured, an interpreted fragment introduces bindings the *enclosing body's own later clauses* can see:

  ```rexx
  interpret "newvar = 7"
  say newvar + 1            ->  8
  interpret "zork = 42" ; interpret "say zork"   ->  42
  ```

  So `extra` is where every run-time-introduced binding lives, whatever introduced it.
* **An interpreted fragment's plan is ephemeral and is not cached.** Fragment text can differ on every execution, so any per-parse key misses every lookup while retaining every entry, and `do 1000000; interpret s; end` would accumulate a million dead plans. Build it, use it, drop it with the fragment; the durable state is in `extra` on the activation, which is where it has to be anyway. Caching interpreted plans is 4b's call, keyed by text, and only if it can show a hit rate. Revision 6's `(enclosing body, fragment id)` key was sound and useless, which is a worse combination than wrong.
* **Built by one upfront pass, at first execution.** The pass walks the body's AST once and returns a finished table; it is not populated lazily one name at a time. Revision 3 supported both readings in different sentences, and they are different algorithms — a lazy design threads a "seen this name?" check through every site that touches a variable. Run-time growth (the `DROP (v)` case below) is the *exception* to a finished table, not the normal path.
* **Cached on `Interp`, not on the body.** `Rc<Program>` gives shared immutable access, so nothing can be written into a `CodeBody` reached through one; revision 1's two central decisions contradicted each other. The cache is `plans: HashMap<BodyKey, Rc<Plan>>` on `Interp`, keyed by a **program id that `Interp`'s loader assigns when it calls `parse_program`** — held in the map's value alongside the `Rc`, never a raw pointer that a dropped `Rc` could let be reused — plus a body index.
  `Interp` drives the lookup at activation entry and hands the resulting `Rc<Plan>` to the new frame, so `run.rs` never reaches into the cache and `plan.rs` exposes only "build a plan for this body".
  **An `INTERPRET` fragment is the exception and needs saying, because the wording above excludes it**: a fragment comes from `parse_interpret`, so no loader assigns it a program id, and it runs inside the activation that created it, so its assignments must land in the *enclosing* frame's slots. Its plan is built against the enclosing plan's name map, resolves through the activation's `extra` map for anything the plan does not hold, and is **not** entered into the cache at all, for the reasons in the bullet above. Task 3's spike runs a fragment *and* the variable pool, which is how this was found.
* **`RootSet` owns the slot storage**, and this is where two of revision 2's bullets contradicted each other: a growable slot vector living in the activation cannot also be a borrowed slice handed to `RootSet` at frame entry, because the slice would neither see later growth nor survive the `&mut self` calls evaluation makes.
  So the storage moves: `RootSet` gains slot frames through `push_slots`/`pop_slots`, an activation holds a frame handle rather than its own `Vec`, and `collect`'s signature is unchanged, so `iter()` still yields everything.
  The invariant that makes a growable frame safe is that **only the top frame ever grows**. That holds **for 4a**, which has one frame, and for `INTERPRET`, which runs inside the activation that created it. It is **false in 4b** and the design must not be built as though it were general: measured, `sub: procedure expose zzz` with `zzz = 5` in the callee makes the caller print 9, so a callee writes into a caller's pool while the callee's frame is on top, and if the caller's body never names `ZZZ` its plan has no slot to write into. 4b either grows a non-top frame or resolves an exposed name to a slot in the caller's frame at call time; deciding that is 4b's, and it is recorded here because the storage design is 4a's and a wrong general claim would be built on.
  Storing slots in `temps` instead does not work: `push_temp` interleaves during evaluation, and `Option<ObjRef>` has no encoding in a `Vec<ObjRef>` because `ObjRef::NIL` cannot mean "unassigned" when `x = .nil` is legal 4a and `.nil` is a value.
  This is part of D15's single `rexx-core` amendment task.

An uninitialised read yields the derived name, and it does so **distinguishably**: the read returns the value together with an "was unset" signal, because `SIGNAL ON NOVALUE` in 4b changes what an uninitialised read does and the read path is 4a's. Retrofitting a raise into the hottest path later is the thing this sentence exists to prevent, and the Phase 4 gate program uses `signal on novalue`. `DROP` restores the unset state.

### D17 trace granularity

**Decided: the dispatch loop emits a trace event per evaluation step from the start, and 4a formats the value lines.**

`RexxActivation.hpp:90`-`110` enumerates 19 prefixes, `TRACE_PREFIX_CLAUSE` at `:92` through `TRACE_PREFIX_INVOCATION_EXIT` at `:110`.
Everything except the clause prefix carries an evaluated value; Phase 3 ships the clause prefix and nothing else.
Emitting an event per evaluation step forbids constant folding and expression fusion, which is accepted: the profile puts dispatch at 38.9% and allocation at 26% of realistic runtime, so folding was never where the time is, and the alternative is designing the dispatch loop twice.

**Trace goes to stderr.** Measured, with `trace r` the `*-*` and `>>>` lines are on stderr while `SAY` is on stdout. Because they are separate descriptors their relative interleaving is not observable, so two independently buffered sinks are safe.

What the decision buys, counted rather than inherited: revision 1 repeated the parent plan's figure of 239 expected trace-output lines in `TRACE.testGroup`, which matches no counting method. Anchoring the resource-block scan to `^[[:space:]]*::resource` — an unanchored match trips on a comment at `:161` that mentions the directive and swallows 51 lines of method code — gives **34 resource blocks and 342 lines of expected trace output, of which 128 are `*-*` clause lines and 214 carry a value or marker prefix**. Those lines are collected by 4b and 4c, not 4a, because an ooTest group is not runnable at all as extracted (see "L1, and why it is table-driven"). The parent plan's `:2412` carries the same wrong figure and is corrected with it.

### D18 command dispatch is not Phase 4's

**Decided: command dispatch, `RC` setting, and the `ERROR` and `FAILURE` conditions land in Phase 7 with the platform layer.**
The `Address` instruction's environment-name tracking is **4c's**, alongside the `ADDRESS()` builtin that reports it, and both are needed by the Phase 4 gate program.
A command clause fails loudly until Phase 7.

`ADDRESS()` is a *partial* exclusion, not an in-scope builtin: measured, `say address()` with no `ADDRESS` instruction prints `sh`, a platform-supplied default from the layer this decision defers.

### D19 dispatch shape and recursion depth

**Decided: expression evaluation is natively recursive on a dedicated interpreter thread with an explicitly sized stack, guarded by a depth counter; the instruction loop is one Rust frame per Rexx activation.**

The oracle has measured capabilities here that a naive recursive evaluator cannot match: it evaluates `x = 1+1+…+1` with **100,000 terms** and prints `100000`, and `say ((((…1…))))` with **20,000 nested parentheses**, both exit 0.
A `fn eval` recursive over a left-deep `Binary` chain uses one Rust frame per term, and the failure mode is a native stack overflow, which aborts with no message and no exit code: precisely the outcome the failing-loudly rule most wants to exclude.

**There are two cliffs, not one, and they behave differently.** Revisions 3 to 7 wrote D19 against flat term chains only. Task 3's spike measured the parenthesis axis and it is an order of magnitude shallower, and — decisively — the oracle *reports* there rather than dying:

| `say ((((…'a'…))))` | oracle | ours, debug, sized thread |
|---|---|---|
| 38,000 parens | rc 0 | rc 0 |
| 40,000 parens | **rc 245, Error 11.1** | rc 0 |
| 85,000 parens | rc 245 | rc 0 |
| 90,000 parens | rc 245 | **rc 134, SIGABRT, no message** |

So on parens we diverge in **both** directions: we succeed from 40,000 to 85,000 where the oracle raises `Insufficient control stack space`, and we abort past 90,000 where the oracle still raises. A flat 100,000-term chain, by contrast, the oracle evaluates and only dies at 150,000, with no condition at all.

Three consequences. The parenthesis recursion is in **`rexx-parse`**, not in `eval`, so D19's counter cannot see it and `parse_program` aborts before the executor exists. Because the oracle answers deep parens with the very 11.1 this design already names, a counter in the parser's subexpression recursion is **parity rather than a deviation** — unlike the evaluation-depth limit, where the oracle crashes and 11.1 is a chosen answer. And the corpus rule, written against the 100,000-term cliff, does not cover a paren cliff at 39,000.

**The bound is two-sided, which revision 2 got wrong by treating 100,000 as a capability rather than as the largest depth anyone had tried.** Measured: 100,000 terms prints `100000` and exits 0; **200,000 terms exits 139**, a SIGSEGV. So a generously sized stack is a *divergence*: `rexx-run` succeeding at 200,000 where the oracle dies is an exit-code difference criterion 1 reports as a failure.

So, and each of these three numbers is chosen and recorded rather than left to be inferred:

* **The sized thread belongs to the public entry point, not to the binary.** `rexx-exec`'s own entry creates it, so `rexx-run`, the L0 harness and the assertion-table harness all get it. A `cargo test` thread has a default stack far smaller than the one a depth limit is calibrated against, and the failure mode there is exactly the silent native overflow D19 exists to exclude — so specifying the thread only for `rexx-run`, while the harnesses run in-process, would leave every in-process caller on the cliff.
* That thread runs the interpreter with an explicit stack size. **That thread owns everything from `parse_program` onward** — bytes in, an outcome out — because `Rc<Program>` is `!Send` and a program parsed on the main thread cannot be handed across, which would be a compile error on day one. The same applies to the capturable output and trace sinks.
* `eval` carries a depth counter whose limit is **bounded on both sides**: at least the oracle's largest passing depth, 100,000 terms, and below what the thread's stack can survive. An upper bound alone is satisfied by a limit of 20,000, which diverges on every program between there and 100,000. The stack size, the measured per-frame cost and the resulting limit are **three numbers recorded in the plan's task reports**, not adjectives here; revision 3 said they were "chosen and recorded" while recording none, which asserted a specification it did not contain.
* The oracle's cliff is **between 100,000 and 150,000 terms**, not at 200,000: measured, 150,000 also exits 139. A corpus rule phrased against 200,000 would admit a 150,000-term program that SIGSEGVs.
* The 11.1 raise is reachable only by a **unit test**, since no differential program can cross the limit without crossing the oracle's cliff too. Without that test the depth path is untested by construction, which is what happened when the corpus was the only plan for it.
* At the limit 4a raises `11.1`, "Insufficient control stack space", which is a **chosen deviation and stated as one**: the oracle raises nothing here, it crashes, so no number is the reproducing answer and 11.1 is the closest condition the language has.
* The corpus carries an expression deep enough to exercise the path and well below both cliffs; a program near the oracle's 200,000-term cliff is excluded, because what it measures is a C++ stack size and not a language rule.

Activation depth is decided here and paid in 4b: measured, unbounded `CALL` recursion gives `Error 11.1`, "Insufficient control stack space", at rc 245 — a reportable condition, not a crash. One Rust frame per activation with an explicit counter produces it; a flat loop over the activation stack would too, and the choice is stated now because D17's own argument is that the dispatch loop should not be designed twice.

**Phase 3's parser has the same exposure, and it is now measured: three recursions, three cliffs, on a default 2 MiB stack.**

| recursion | cliff | kind of fix |
|---|---|---|
| `block.rs::visit_expr`, a hand-written walk run per clause from `add_clause` | **2,450 terms** | iterative, Task 3b |
| the compiler's drop glue for a `Box<Expr>` chain | ~10,000-20,000 | iterative, Task 3b |
| `parse_subterm`'s parenthesis descent | ~85,000 debug on the sized thread | a counter raising 11.1, Task 3c |

Only the third is what this section originally anticipated. The first runs *during* parsing, so `parse_program` aborts before a `Program` exists for anyone to drop, and it is the shallowest by a wide margin — a 3,000-term expression, which the oracle evaluates without difficulty, is already past it.

The diagnosis matters as much as the numbers. An earlier draft asserted the drop glue was the cause, on the strength of a stack overflow plus the fact that a 512 MiB thread parsed the same file fine. That evidence fits every deep-recursion hypothesis and separates none of them. What separated them: `mem::forget` on the parsed result left the cliff unmoved, ruling out `Drop`; building the same tree without the parser showed `Drop` surviving six to eight times deeper; and a backtrace named the frame. Prefer an experiment that can distinguish the candidates over one that merely agrees with the favourite.

## Architecture

### The borrow shape

`Interp` owns `Heap`, `RootSet`, the activation stack, the plan cache, the trace sink and the output sink, and **does not own the AST**.

**`Settings` is per activation, not one field on `Interp`.** Revision 3 said both, in two sections that never met. Measured: with `numeric digits 7`, an internal `call sub` sees 7, sets its own to 3, and after `return` the caller still reports 7. So a frame carries its own `Settings`, inherited from its caller at call time, and a `NUMERIC` instruction mutates only the current frame's. The `TRACE` setting behaves the same way and is restored across a call: measured, a callee's `trace off` does not survive its `return`. 4a has one frame, which is exactly why this must be written down now rather than discovered by 4b.
Programs are held as `Rc<Program>`.

The discipline is one sentence, and it is the whole answer to the parent plan's named soft spot: **the instruction loop clones the `Rc` into a local on entry, and every `&CodeBody` and `&Expr` derives from that local.**
An activation's own `Rc` is a liveness anchor and is never borrowed through — borrowing `&self.activations.last().unwrap().program.…` and then calling `self.eval(…)` is an `E0502`, and that is the version revision 1's adjacent sentence pointed at.

Under that discipline the shape holds for an `INTERPRET` fragment created mid-instruction, for a `DO` control expression re-evaluated per iteration, and for 4b's body-calls-body case, where each Rust frame clones its own `Rc` into its own local.

**One `rexx-parse` change is required and is a task, not an assumption.**
`fn eval(&mut self, body: &CodeBody, expr: &Expr)` cannot be called for the body 4a actually runs: `Program` holds `instructions` and `labels` as sibling fields, `CodeBody` exists only per directive body, and `Fragment` has no labels at all — while the Task 1 spike runs a `Fragment`. `Program` gains `pub main: CodeBody` and `Fragment` gains `pub body: CodeBody`, preserving the existing derives. Whether an `INTERPRET` fragment may contain a label decides whether that label table is always empty, and is measured in the task rather than assumed.

**Task 1 is a spike that proves the shape end to end** — including a fragment whose `Rc` outlives the instruction that made it, and the variable pool, since a spike that avoids the pool proves less than it claims.
To be explicit, because the split table assigns `Interpret` to 4b and this reads like a contradiction otherwise: **4a builds the fragment-execution machinery and 4b builds the `INTERPRET` instruction on top of it.** The spike runs a fragment because that is the case that stresses the lifetime, not because 4a implements the keyword — an `INTERPRET` clause in a 4a program still fails loudly. It is kept, with the failing version in a comment, because the next phase to touch this will want to know which version does not compile.

`Rc` is not `Send`, which bites **twice and at different times**. Immediately, it constrains D19: the interpreter thread must own the parse, because nothing can hand it an `Rc<Program>`. Later, in Phase 6, it means either converting every `Rc<Program>` to `Arc` (mechanical but pervasive) or giving each thread its own package arena. Revision 2 recorded only the Phase 6 half, which is why the paragraph that should have caught D19's compile error pointed at the wrong phase.

### Crate layout

One new crate, `rexx-exec`, depending on `rexx-core`, `rexx-num`, `rexx-parse` and **`rexx-inventory`**.

That fourth dependency is the one revision 3 missed while giving `error.rs` a message catalogue to build. Phase 0 already generates `errors.rs` from `rexxmsg.xml`, **704 messages**, and it is the source for 7.3, the 34.x family and the `DO` control numbers; arithmetic's text comes from `rexx-num`'s `ArithError::message()` and needs nothing. Hand-transcribing text the tree already generates would be writing what exists.

```
rexx-exec/
  src/value.rs        the value model, conversions, string and number identity
  src/stem.rs         stems and compound tail resolution
  src/plan.rs         the per-body resolution pass (D16)
  src/activation.rs   one frame: a RootSet slot-frame handle over Option<ObjRef>
                      entries, block stack, pc, its own Settings, its Rc<Plan>
  src/eval.rs         expression evaluation and the operators
  src/run.rs          the instruction loop, control flow, DO block state
  src/trace.rs        trace events and prefix formatting (D17)
  src/error.rs        Raised, the condition payload, and the message catalogue
  src/lib.rs          Interp, and the public entry point
  src/bin/rexx-run.rs the runner the differential tests drive
```

One file per concept, as elsewhere in the workspace, each readable in one sitting; `run.rs` is the one at risk, and the split when it comes is the loop from the per-instruction handlers.

## Expression evaluation

Recursive over `Expr`, subject to D19. The tree already carries precedence and associativity, so evaluation never reconsiders either.

The operators are enumerated by name, because revision 1's two-family description lost eight of them and an implementer had nowhere to put three more:

* **Arithmetic**: `+ - * / % // **`, through `rexx-num` under the current settings.
* **Numeric-or-string comparison**: `= \= <> >< > < >= <= \> \<`, where `<>` and `><` are `\=` aliases.
  **Call `rexx-num`'s comparison. It is the whole algorithm, string fallback included, and it is already correct.**
  One amendment is needed first, and it belongs with the `rexx-parse` and `rexx-core` amendments rather than being discovered mid-task: `compare` today takes `&str` and re-parses both operands on every call. That cannot accept a byte string which is not valid UTF-8, which is D14's whole point, and it defeats D15's parse cache, whose stated purpose is that a non-numeric string does not re-run `from_utf8` and `Number::parse` on every comparison. So `rexx-num` gains a byte-slice entry point that accepts already-decoded operands, and `rexx-exec` calls that. Writing a second comparison in `rexx-exec` instead is what this whole paragraph exists to prevent.
  Revision 3 omitted this family entirely; revision 4 described it as "numeric if both are numeric, otherwise compare as strings with the shorter blank-padded on the right", which is wrong and would have had an implementer hand-write a second, divergent copy of code this workspace already contains.

  The real string rule is `RexxString::stringComp` (`StringClass.cpp:795`), ported at `rexx-num/src/compare.rs`'s `string_order`: strip **leading** blanks *and tabs* from both sides, compare the shared prefix byte for byte, and if one side is longer, it ties only when its leftover is all blank or tab, otherwise the first non-blank leftover byte decides against a literal space. Measured, and note that none of revision 4's eight cells could tell the two rules apart:

  ```
  ' a' = 'a'        -> 1        '09'x'a' = 'a'  -> 1        'a' = 'a'||'09'x -> 1
  'a'  = 'a '       -> 1        'a b' = 'a  b'  -> 0
  '01' = '1'        -> 1        ' 1 ' = 1       -> 1        'a' = 1          -> 0
  ```

  `'a' = 1` is 0 because one side is not numeric, so it falls to `string_order`; `' 1 ' = 1` is 1 because both are.
* **Strict comparison**: `== \== >> << >>= <<= \>> \<<`. These are not merely "byte" comparisons: there is no padding and the shorter string is less. Measured, `'10' >> '9'` is 0 while `'10' > '9'` is 1; `'a' << 'a '` is 1 while `'a' = 'a '` is 1 and `'a' == 'a '` is 0.
* **Logical**: `& | &&`, each with its own 34.x logical-value check on both operands.
  A logical value is **exactly the one-character string `0` or `1`**, with no coercion whatever. Measured with `if x then`: `'1'` is accepted, while `' 1 '`, `'01'`, `'1.0'` and `''` are each error 34. The same rule governs `IF`, `WHEN`, `DO WHILE`/`UNTIL` and `ExprKind::Logical`.
* **Concatenation**: `Abuttal`, `Blank`, `||`, over bytes.

`Operator::Backslash` cannot appear in a `Binary` node and is correctly absent.

The rest of the tree: `Prefix` is `+ - \`; `Literal` and `Constant` are the bytes the parser decoded, so `say 1e5` prints `1E5`; `Variable`, `Stem` and `Compound` go through the plan's slots; `Logical` is the comma list in a condition and is an AND of its parts, each checked for 0 or 1; `DotVariable` is `.nil`, `.true` and `.false` only.
`Call`, `QualifiedCall`, `Message`, `ClassResolver`, `List` and `VariableReference` are not 4a's and fail loudly.

A `WhenCase` value is compared with the `SELECT`'s expression using `==`, not `=`. Measured: `select case '007'` with `when 7` does not match.

## Control flow

```rust
enum Flow { Next, Goto(usize), Exit(Option<ObjRef>) }
```

A program counter walks the body's `instructions`; each step answers `Flow` or raises.
Loop state is a per-activation `Vec<Block>` holding the control variable's slot, the `to`, `by` and `for` values, the iteration counter, the block's label and its `end` index.
`LEAVE` and `ITERATE` unwind that stack to the matching label and jump.
Evaluation order inside a controlled loop is `Controlled::order`, which Phase 3 recorded because an expression can have side effects.

## Errors, and the reporting subsystem

`Result<T, Raised>`, where `Raised` carries the condition name, the number and sub-number, and the substitution values.
4a's raisers, **measured rather than enumerated from memory, and treated as an open list the catalogue task closes**: arithmetic; a `SELECT` reaching its `END` with no `WHEN` taken (7.3); the logical-value checks, which are six sub-numbers and not one — 34.1 `IF`, 34.2 `WHEN`, 34.3 `WHILE`, 34.4 `UNTIL`, 34.6 the comma list, 34.901 for `&`/`|`/`\`; the `NUMERIC` instruction, which raises **26.5** for `DIGITS`, **26.6** for `FUZZ` and **33.1** when `DIGITS` does not exceed `FUZZ`, so major 33 is a family revision 4's closed list of four did not have; the `DO` control conversions, now measured rather than paraphrased: **41.1** for a non-numeric initial, `TO` or `BY` value, **26.3** for a non-whole or negative `FOR` count, and **26.2** for a non-whole or negative `DO` repetitor, while a non-whole *control* value raises nothing at all, since `do i = 1.5 to 3` is legal — an earlier draft attributed 26.2 and 26.3 to "non-whole control values", which points a probe at the one case that never raises; and `DO OVER` on a non-collection, `do i over .nil`, which is **98.913** at rc 158 from two constructs both in 4a's scope.
No trapping: `SIGNAL ON` is 4b's.

Criterion 1 compares stderr and the exit code byte for byte, so "terminates with the oracle's message" is a subsystem and is named as its own task. Measured:

```
     3 *-* end
Error 7 running /abs/path/vB.rex line 3:  WHEN or OTHERWISE expected.
Error 7.3:  All WHEN expressions of SELECT are false; OTHERWISE expected.
rc=249
```

That is a clause echo **with trace off**, a major-number line carrying the absolute path and line number, a sub-number line with substitutions, two spaces after each colon, and `exit code = 256 - major`. A second measured case: 34.1 gives rc 222. The task delivers a message catalogue for 4a's four raiser families with major and sub-number text, the two-line format, the clause echo, and the exit-code rule, with oracle-captured expectations. Only arithmetic's text exists today.

The `DO` control family is the one nobody had measured. Confirmed here: `do i = 'x' to 3` and `do i = 1 to 'y'` both give **41.1**. Reported by review but not reproduced by me: **26.2** and **26.3** for non-whole control values, so the task enumerates the family against the oracle rather than trusting either account. And `do i = 1 by 0 to 3` **loops forever and raises nothing**, which is a behaviour to reproduce rather than an error to catalogue — my own first probe of it printed `Error 4.1`, which was `timeout` being converted to HALT and not the program doing anything.

### Failing loudly

Every feature 4a does not implement fails distinguishably: a dedicated process exit code and a message naming the construct and the sub-phase that owns it, never a plausible Rexx condition.
That code must sit **outside the band a Rexx error can produce**, which is `256 - major` for majors 3 to 99, so 157 to 253 — a not-implemented code of 245 would be indistinguishable from error 11. The chosen code is recorded in the plan and the harness treats it as a hard failure whatever the oracle did.
If an unimplemented builtin raised 43.1, a differential run would show what looks like a resolution bug, and a program expecting 43.1 would *pass*.
**An implementation gap must never be able to produce a passing test**, and criterion 5 below is what enforces it — in revision 1 this rule was prose that no criterion tested, while 4a's out-of-scope surface is larger than its in-scope surface.

## Output and trace sinks

`SAY` writes to a sink on `Interp`, defaulting to stdout. The trace sink defaults to **stderr**. Neither is the Phase 7 stream model: `.output` as an object, redirection and the stream classes are Phase 7's, and the sinks exist so a test can capture output without a subprocess.

## Testing

### L0 differential corpus

`rust/corpus/` programs run under `rexx-run` and under `build/bin/rexx`, compared through `rexx-oracle`'s `normalize` and `diff`, which compares exit code, then stdout, then stderr, byte for byte.
`normalize` masks exactly two things — CRLF folding and the cwd string — so criterion 1's zero is blind to line-ending differences and to a path that equals the cwd. Stated so the zero is read with its scope; the inherited rule that a self-test divergence means the corpus is at fault and never `normalize` still holds.

**4a must write most of its own corpus, and the inherited one is measured, not assumed.**
Of the 28 programs in `rust/corpus/`, 10 use only 4a features, and **none of the 10 contains a `LEAVE` or an `ITERATE`**; seven are numeric, so the set largely re-tests `rexx-num` through a new front end. The entire `DO`-variant coverage is in `do_variants.rex`, excluded by a single line, `do i over .array~of("x", "y")`. No single addition rescues this: the five programs one feature away each miss a *different* feature.
So 4a writes roughly 12 to 15 programs, listed in its plan, starting with a 4a-only cut of `do_variants.rex`, and covering the control-flow shapes in criterion 1, `LEAVE`/`ITERATE` by label, whole-stem versus tail `DROP` including the tombstone, `EXIT` with an expression, the created-digits and created-form transcripts from D15, the stem-aliasing transcripts from D15a, an expression at a depth the oracle handles, and one witness program per trace prefix.

The comparison runs as a `cargo test` with the oracle's expected output committed the way `tests/sourceline_oracle/` does it, so `cargo test` alone is the gate and a script regenerates the expectations. That applies to the trace expectations too.

### L1, and why it is table-driven

`rexx-extract` renders each test method as `::routine main public` plus a `::class shim public`, so **an extracted program's main body is empty and it executes nothing at all** — verified under the oracle itself: a file in exactly that shape with a `say` in the routine produces no output and exits 0. Even if the routine were driven, `self~assertSame(…)` has no `self` inside a routine. Nothing in the project rests on this: Phase 2's gate recorded its L1 criterion as CANNOT ASSESS rather than claiming a pass.

Revision 1's criterion 2 quantified over "extracted assertions that need only 4a features", which is therefore the **empty set**, satisfiable by declaring all of `base/expressions` blocked. It is replaced by the route Phase 2's gate already costed: extract the assertions as **data**, not programs.

`ootest/ooRexx/base/expressions/` holds 4,269 `assertSame` calls, of which 2,528 match a plain `<operand> <operator> <operand>` shape. 4a adds an extraction mode emitting one row per assertion: the expression text, the expected value, and the `NUMERIC DIGITS` in force. That setting changes throughout those files, from 1 to 100, so the extractor **scans sequentially and carries the setting**, rather than matching assertions in isolation — getting that wrong silently tests the wrong precision and still passes, which is the worst available outcome. The harness evaluates each row's expression through `rexx-exec` and compares to the expected value, needing no directives, no message sends and no builtins.

### What the tests cannot see

* **Intra-expression evaluation order** is invisible unless a side effect exposes it, and 4a has none inside an expression except trace output. Trace-output tests are the only observation of it, which is a second reason D17 lands here.
* **A too-wide `Expr` span** cannot be falsified by any Phase 3 test, because `Expr::new` widens by construction. Trace value lines print a subexpression's source text and are the first consumer that can falsify it; a mismatch there is a Phase 3 defect, not a formatting bug.
* **Two of Phase 3's three shape blind spots stay unobserved here.** A clause moved across a body boundary is unobservable in 4a by construction, since 4a runs one body and every directive is out of scope; it belongs to Phase 5. Argument attachment inside `Call`, `QualifiedCall`, `Message`, `List` and `VariableReference` is exercised by 4b and 4c, not 4a, which evaluates only `Logical` of the six. Revision 1 claimed handover 4 "lands here"; it lands here only for control-flow targets.
* **GC correctness under pressure** is invisible unless a collection happens at the right moment, which is what criterion 4 exists for.

Revision 1 also listed a stale value cache across a `NUMERIC` change as a blind spot. With created-digits on the value there is no such state, and without it the corpus catches it on the first `say` after a `NUMERIC` change — it was misclassified as invisible when it is loud.

## 4a exit gate

Each criterion names the set it quantifies over, each can fail, and no criterion's anti-vacuity requirement lives in prose beside it.

1. **The named L0 subset in `rust/corpus/phase-4a.txt` runs with zero divergences**, the harness reporting the program count rather than the criterion asserting it, and the subset satisfies a coverage property: every `InstructionKind`, `ExprKind`, `LoopKind`, `PrefixOp`, `EndStyle` and `Trace` variant in 4a's scope, and every `Operator` listed above, is constructed by at least one program in it, asserted by a macro-generated enumerating test with no wildcard arm, in the shape Phase 3's criterion 2 used so that a new variant is a compile error rather than a silent gap.
   The enumeration lists every variant, so a variant outside 4a carries the phase that owns it instead of a witness, and the test fails on a variant that carries neither. **The owner string must be one of the phases named in the split table or the "assigned elsewhere" paragraph, and the out-of-4a variant set is asserted the way the exclusions file is** — otherwise the owner arm is an unpoliced escape and any variant that turns out hard can be marked Phase 5's instead of getting a witness. The assignment is complete today: all 40 `InstructionKind` variants have an owner (20 in 4a, 9 in 4b, 4 in 4c, 6 in Phase 5, 1 in Phase 7) and all 15 `ExprKind` variants do (9 in scope, 6 failing loudly), so the escape is unexercised and the assertion costs nothing now and everything later. Without that arm the criterion demands a witness for `LoopKind::With`, which needs `SUPPLIER` and therefore Phase 5, and the criterion added to fix a blindness finding would itself have been unsatisfiable. `EndStyle` has never been gated by any phase.
   The subset must include: nested `DO` with `LEAVE` naming an outer label; `ITERATE` from inside a `SELECT` within a loop; `IF`/`ELSE` chains where the false target and the then-exit differ; a `SELECT` whose `WHEN` bodies are several instructions long with visible side effects, so that a wrong exit lands inside a later `WHEN`'s body; and `when 1 = 1 then` followed by `when 2 = 2 then nop`, where the second `WHEN` is the first's `THEN` instruction and is never collected into `whens`.
   Revision 1 asked instead for a `SELECT` whose `WHEN`s all share an exit, which is true by construction — `fixWhen` gives every `WHEN` of one `SELECT` the same exit — so no test could have failed it.
2. **The `base/expressions` assertion table passes**, every row evaluated, the count reported, and **each row compared byte for byte against the expected value, never numerically** — a numeric comparison would hide the entire created-digits and created-form story across thousands of rows, which is the defect class D15 exists to prevent.
   A row whose operands need 4b or 4c is listed with the sub-phase that unblocks it, and the extractor's sequential `NUMERIC DIGITS` tracking has its own test against a file that changes the setting mid-way.
   The table covers every assertion whose expression 4a can parse and evaluate, **including the PRECEDENCE (1,226) group**, which is self-contained literal arithmetic. Phase 2's costing excluded it and restricted itself to plain `<operand> <operator> <operand>` triples because Phase 2 had no parser; 4a has one, so that restriction is obsolete.
   **CONCATENATION (388) needs more than a triple and must not be added naively.** Every one of its assertions references variables `a` through `g` assigned at the top of the test method, which a row of (expression, expected, digits) cannot carry — and the failure is silent, not loud: with `a`..`g` unset, `(a==a) (b==a) … (g==a)` evaluates to `1 0 0 0 0 0 0`, which is exactly the expected value, because an uninitialised variable yields its own distinct name. So the group would report a pass while testing nothing, over the one group that carries NULs and blanks. A row therefore carries the method's **assignment prelude**, and any assertion whose prelude cannot be represented is listed as blocked rather than included.
   The harness also runs a **falsification check** over the whole table: perturbing an expected value must make that row fail. A table that cannot fail is the defect this criterion already had once.
3. **Trace output matches the oracle byte for byte on stderr**, over a committed table mapping **each of the 19 prefixes at `RexxActivation.hpp:90`-`110`** to either a witness program or the sub-phase that first emits it, with the harness failing when a 4a row has no witness. The prefix list is enumerated from the oracle's table, not from what the implementation turns out to emit, because "one program per prefix 4a can emit" is a set the implementation chooses and the failure it cannot see is 4a emitting nothing where the oracle emits something. Measured reachable from pure-4a code: `*-*`, `>>>`, `>=>`, `>L>`, `>V>`, `>O>`, `>K>`, `>C>` and `>P>`.
4. **The named L0 subset passes again under collect-on-every-allocation.**
5. **Every `InstructionKind` and `ExprKind` variant either executes or fails loudly**: an enumerating test with no wildcard arm asserts that each variant is in 4a's named set or produces the not-implemented exit code with a message naming the owning sub-phase. One criterion closes a surface larger than 4a's own and cannot rot.
6. **A mutation control**, replacing "substituting any other binary reports divergences on every program", which `/bin/true` satisfies and which demonstrates only that the harness notices *absent* output. A committed list of one-line mutations to `rexx-exec`, each of which the subset must catch, mapped onto the handed-over blind spots: off-by-one on `If::false_target`; off-by-one on `When::exit`; `Loop::end` off by one; `Controlled::order` evaluated in fixed To/By/For order; `Abuttal` treated as `Blank`; `=` treated as `==`; `LEAVE` unwinding one block too few; formatting with the current digits instead of the created digits; and formatting with the current form instead of the created form.
   The mechanism is the mutation script this project already uses, carrying its **exit-non-zero-on-an-unapplied-pattern guard** — that guard fired in four separate Phase 3 tasks, and without it a pattern that has gone stale reports coverage that does not exist. This is the one criterion a `cargo test` cannot be, since it edits the source it tests, so the gate records the script's output and the plan says so rather than leaving it to look like the others.
7. **Zero `unsafe`, `clippy -D warnings` clean, `cargo fmt` clean**, and the Task 1 spike committed with its findings written down.

## Phase 4 gate items decided now

### The exclusions file

`docs/superpowers/plans/phase-4-exclusions.txt`, one row per exclusion: the name, what is excluded, the phase that delivers it, and the failure it produces meanwhile.

The gate asserts the **set**, not a count: the excluded rows are exactly the fifteen names below plus the three partial rows, and adding a row is a plan amendment rather than a file edit — otherwise the file is an artifact of the phase being gated and any builtin that turns out hard can be excluded by editing it. The harness reports "66 in scope, three of them partial" as a derived number.

Fifteen whole exclusions from the 81 entries in `builtinTable[]` (`BuiltinFunctions.cpp:3042`):

* Phase 7, streams and platform: `CHARIN`, `CHAROUT`, `CHARS`, `LINEIN`, `LINEOUT`, `LINES`, `STREAM`, `QUALIFY`, `USERID`, `SETLOCAL`, `ENDLOCAL`.
* Phase 10, RXAPI: `RXQUEUE`, `RXFUNCADD`, `RXFUNCDROP`, `RXFUNCQUERY`.

Three partial rows, because a whole-builtin exclusion would overstate the gap:

* `VALUE`'s external-selector form, which reads a pool such as `ENVIRONMENT`: Phase 7. The variable-access form is 4c's.
* `ADDRESS()`'s platform default: Phase 7 supplies it, measured as `sh`. Reporting an environment set by an `ADDRESS` instruction is 4c's.
* `QUEUED` is in scope against 4b's in-process queue, and cross-process sharing with the oracle's rxapi-backed session queue will never match, so the row records that a differential run of `QUEUED` is single-program only.

Without this file, "all 81" is a criterion the phase ordering cannot satisfy, which is how Phase 2 came to fail three of five.

The file carries a **second section for semantic deviations**, which are not exclusions and must not be filed as ones. It holds two rows: `DO OVER` on a stem does not reproduce the oracle's traversal order (D15a), with the corpus consequence that no program may contain it; and 4a raises 11.1 at its evaluation-depth limit where the oracle crashes with no condition at all (D19). **The set assertion covers this section too**, so adding a deviation is a plan amendment rather than a file edit. A deviation is permanent and therefore strictly worse than an exclusion, which is assigned work: pinning the weaker thing and leaving the stronger one editable by the phase being gated would be backwards. A deviation is a permanent difference chosen on purpose; an exclusion is work assigned to a later phase. Filing one as the other is how a deviation stops being reviewed.

### The rexxcps gate

`samples/rexxcps.rex` is the end-of-4c gate. It reconstructs a clause mix from an analysis of 2.5 million lines of trace output and deliberately issues no commands, using an `RC=expression` and `PARSE` sequence instead.

Its dependencies, read from all 198 lines rather than sampled: `parse var`, `parse version`, `parse value`, `parse upper`, `parse source`, `trace value`, `trace off`, `signal on novalue`, one internal `call subroutine` and therefore a `Label`, the `call time 'R'` call-to-builtin form, **`address value` together with the `ADDRESS()` builtin** (line 143 is `trace value trace(); address value address()`), and eight builtins: `TIME`, `SUBSTR`, `FORMAT`, `WORD`, `TRACE`, `LENGTH`, `LEFT`, `ADDRESS`. Nothing from Phase 5 or later.
The `address value address()` line makes **D18's decision to keep tracking the environment name load-bearing for the gate program**, which is why that tracking is assigned to 4c above rather than left unowned.

Measured under the oracle on this machine on 2026-07-30: 16,608,454 clauses per second, 1.83 s wall, exit 0, reproduced within 1% on a second run. "Clauses" is the program's own nominal count rather than a measured tally, so the figure is meaningful only as a ratio between two interpreters running the identical program.

Four criteria:

R1. **Correctness before speed**, and not by string match. rexxcps's full stdout must equal the oracle's after masking the timing fields, which R3 needs anyway. "Prints no `Failed` line" is kept as a redundant check, not as the criterion: it is satisfied by printing nothing, so an executor that silently skipped clauses would pass it, exit 0, and report a *better* cps.
R2. **The cps ratio**, both interpreters, same machine, same session, with the formula written out because cps is higher-is-better while every other ratio in this project is a time: `ratio = oracle_cps / rust_cps`. Above 1.5 fails; between 1.0 and 1.5 is recorded as debt, the shape Phase 1 used, because the alternative is a sound design stalling a phase over 10%; at or below 1.0 passes.
R3. **An external cross-check.** rexxcps times itself with `TIME('R')`, our own builtin, so a defect there flatters the number and the benchmark cannot detect it. Wall-clock both runs externally and require the two ratios to agree within 10%.
R4. **The baseline is measured at gate time**, not reused: `perf-baseline.md` has no rexxcps row and its figures come from a different day.

rexxcps is not a corpus program — its output carries timings and the corpus rule is determinism — so it gets its own comparison rather than a loosened `normalize`. It adds to the seven `bench-programs/` dimensions (eight files, of which `heapshape.rex` is D1's GC harness) and replaces none.

### The two carried debts

Both re-measured at 4c, on equal footing, which neither earlier phase could do: D1's GC pause at 1.45x (26.5 ms against 18.2 ms), and Phase 2's arithmetic at 1.22x, itself a lower bound because it timed Rust arithmetic against a C++ number that included parse and dispatch.

## Risks

* **`rexx-core` is amended by a phase whose handover section calls it settled.** D15's variants, `Body::trace`, the `rexx-core -> rexx-num` edge and D16's slot frames are one task; if it grows past that, the phase has found a Phase 1 design problem rather than an integration detail, and that is a plan amendment.
* **The `Rc<Program>` shape is proven only for 4a's cases.** 4b's body-calls-body case is free under per-frame recursion; what is not free is the flat-loop variant, where the local must be re-derived at every frame transition. D19 chooses per-frame, so this risk is closed by a decision rather than deferred, and reopening D19 reopens it.
* **The error-reporting subsystem is the largest unbudgeted item.** Two catalogues, exact spacing, the clause echo and the exit-code rule, for four raiser families. If it does not fit, the honest move is to state that 4a's subset contains no raising program and move the raisers to 4b — not to leave "Conditions and errors in 4a" reading as delivered.
* **Trace output is the widest surface in 4a and the least documented.** 342 expected lines exist for the whole of `TRACE` and none of them is 4a-runnable, so the oracle is the only specification and every witness program is written against it.
* **`TRACE ?` requests interactive debug**, which pauses and reads stdin. 4a implements the instruction that can request it and the gate tests two non-interactive settings, so the plan measures what the oracle does with `?` and no tty, and 4a either reproduces that or fails loudly. Silently ignoring `?` is the exact shape the failing-loudly rule exists to prevent.
* **`corpus/README.md` hard-codes "24 programs"**, the count-rot this spec warns about, in the document it inherits its two corpus rules from. 4a fixes it to report rather than assert while it is adding programs.
