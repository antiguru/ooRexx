# Phase 5: the inherited surface, and the decisions still open

Input to a Phase 5 spec, not the spec. Part 1 assembles every place the tree already names Phase 5 as
the owner of something. Part 2 lists the design decisions writing that spec will have to make, and
**deliberately does not answer any of them** -- they are Moritz's calls, and an answered question here
would read as settled.

**Committed 2026-08-15, verified at `11638b91e`, and not re-verified since.** Twenty-four commits
landed between that revision and the one committing this file, closing the rest of
`2026-08-15-task-6-review-and-run-split.md`. One factual consequence was corrected on the way in:
`ir/mod.rs` and `builtin/mod.rs` were renamed to `ir.rs` and `builtin.rs` at `334ca279e`, and the
`CallSite` entry below now names `ir.rs`. `tests/support/mod.rs` keeps its name deliberately and
the two references to it are correct. Nothing else in this document cites a file and line, so
nothing else could go stale silently -- the anchors are function and type names, which is why they
were chosen. Read the rest as a reading of `11638b91e`, and check anything load-bearing against the
tree before building on it.

## How this was verified

Every entry was opened at the tree rather than copied from a gate document or a report. Where a gate
document's claim has moved since it was written, this document says so and cites what the tree reads
today.

**HEAD moved while this was written.** The reading started at `ac09e3abe` with another agent's
uncommitted edits to `rust/crates/rexx-exec/src/{clause,lib,run}.rs` in the tree; those landed as
`11638b91e` ("Give an INTERPRET fragment the condition queue the oracle gives it") before this was
finished, so what is cited below is HEAD. That diff was inspected either way: its hunks sit in
`PendingTrap`, `Interp::fragment_depth` and the `INTERPRET` clause-boundary path, and none of them falls
inside `instruction_owner`, `expr_owner`, `directive_gap`, `Loud`, or `exec_use`. The transcripts below
are unaffected by that work.

`rust/target/release/rexx-run` was built by another agent and carries those edits. It was used, not
rebuilt.

Oracle runs used the standard wrapper -- `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib
.../build/bin/rexx FILE )` -- with stdout, stderr and status read as three descriptors, from a fresh
empty directory created for this session. Counts stated below were taken with `/bin/grep -a` or
`wc -l` in this session and each says how.

---

# Part 1: the surface Phase 5 inherits

## 1. The roadmap's own Phase 5 row

`docs/superpowers/plans/2026-07-27-rust-rewrite.md`, the phase table:

> | 5 | Object model | 4 | **`CoreClasses.orx` parses and executes**; 32 classes exist and respond;
> `::class`/`::method`/`::routine`/`::requires` work; security manager interception points in place
> (D12); cold start measured and recorded against C++ (D2) | L2 |

**Marked correction, 2026-08-15, after the Phase 5 design spec's review panel.** "The three assets that
gate names" is wrong and the paragraph below inherits the error. The **Phase 5** gate row quoted just above
names `CoreClasses.orx` and nothing else; `StreamClasses.orx` is named by the roadmap's **Phase 7** row
("`StreamClasses.orx` runs"), and the crate-layout line at `:518` says `rexx-lib` *loads* the two files, not
that Phase 5 runs both. The measurements in the paragraph are sound; the ownership claim in its first six
words is not. See `2026-08-15-phase-5-object-model.md`, "What the bootstrap actually reaches", which found
that `StreamClasses.orx` binds Phase 7's natives eagerly at directive-install time and cannot run here.

The three assets, measured at the oracle checkout this session:
`interpreter/RexxClasses/CoreClasses.orx` is 4,193 lines by `wc -l` and holds 32 lines matching
`^::class`/`^::CLASS` by `/bin/grep -c`; `interpreter/RexxClasses/StreamClasses.orx` is 1,010 lines.
The roadmap's line 82 makes the consequence explicit: "a Rust core that can execute `CoreClasses.orx`
inherits 32 classes for free."

Other Phase 5 sentences in the same file, verified at their lines:

* `:450` -- "Phases 6, 7, and 8 are independent of each other and may run in parallel once Phase 5
  closes." Phase 5 is the only serialising point left in the plan.
* `:482` -- 4e "founds OO dispatch for Phase 5 by making a call site a patchable slot". The slot
  exists; see section 8.
* `:2447`-`:2451` -- **performance work is suspended for the duration of Phase 5**, under three
  conditions: pin the standing as a recorded interleaved baseline first, run the axes during Phase 5
  as a regression guard rather than an optimisation target, and measure `apply_binary` *after* Phase 5
  because "Rexx objects can override the operator methods, so Phase 5 must add an object check to that
  path".
* `:2475` -- the risk row: "`CoreClasses.orx` needs semantics not yet built | Phase 5 stalls with a
  long tail of gaps | Expected, not a surprise. Triage per gap; do not start Phases 6-8 until Phase 5
  closes".
* `:518` -- the planned file structure names two crates that **do not exist in the tree**: `rexx-lib/`
  ("Phase 5: loads CoreClasses.orx / StreamClasses.orx") and `rexx-classes/` ("Phase 4-5").
  `ls rust/crates/` this session gives `rexx-bench`, `rexx-core`, `rexx-exec`, `rexx-extract`,
  `rexx-inventory`, `rexx-num`, `rexx-oracle`, `rexx-parse` and nothing else.

### D2 -- the saved image, blocked on Phase 5

`:136`-`:151`. The decision is already made and is **(a), no image; "Ship (a) either way."** What Phase
5 owes is the measurement: "Phase 5 exit measures cold start for (a) with hyperfine against
`build/bin/rexx`", against an **absolute** threshold of about 50 ms over the C++ startup, not a ratio.
The document records the C++ side as median 5.1 ms for a one-line `say` (50 runs after 10 warmups, not
re-measured here), making the target "parse and execute 5,203 lines ... in under ~55 ms".

### D12 -- the security manager, split across Phases 5 and 7

`:332`-`:335`. **Phase 5** builds the security manager object, its installation path, and the hooks in
dispatch and name resolution -- `.local`/`.environment` lookup and external function resolution.
Phase 7 adds the command-issuance and stream hooks. The stated reason for fixing the *interception
design* in Phase 5 is that "Phase 7 adds call sites to an existing mechanism rather than inventing a
second one". ooTest covers it directly at `base/security.manager/SecurityManager.testGroup`.

### The value-layer rule handed forward

`docs/superpowers/plans/2026-08-11-value-representation-design.md:347`-`:366`, section "What the object
model needs from this". The rule, in the document's own words: hot variants (`Text`, `Num`) stay inline
and narrow; **cold variants are boxed, and a new class's instance state arrives boxed by default**.
The enforcement is already in the tree at `rexx-core/src/body.rs`:

```rust
const _: () = assert!(size_of::<Body>() <= 80);
```

whose doc says "**Phase 5 adds variants and is expected to trip this**, which is the point -- widening
the arena for a cold value kind should be a deliberate act with a boxed alternative weighed".

## 2. The runtime refusals, by construct

The mechanism is one function pair in `rust/crates/rexx-exec/src/lib.rs`: `instruction_owner` and
`expr_owner`, both exhaustive with no `_` arm, feeding `owned_message`, which renders
`"{name} is not implemented ({owner})"`. The exit code is `NOT_IMPLEMENTED_EXIT`, `pub const … i32 =
120`.

`tests/owners.rs`'s `EXPECTED_OUT_OF_SCOPE` is the committed set the tables are allowed to produce.
Read at HEAD, its Phase 5 rows are:

| category | tag | witness in `tests/loud.rs` |
|---|---|---|
| `InstructionKind` | `Call::Qualified` | `call ns:sub` |
| `InstructionKind` | `Expose` | `expose x` |
| `InstructionKind` | `Options` | `options 'x'` |
| `InstructionKind` | `Message` | `a~b` |
| `InstructionKind` | `Guard` | `guard on` |
| `InstructionKind` | `Reply` | `reply 5` |
| `InstructionKind` | `Forward` | `forward` |
| `ExprKind` | `QualifiedCall` | `say ns:foo(1)` |
| `ExprKind` | `ClassResolver` | `say ns:Bar` |
| `ExprKind` | `List` | `say (1, 2)` |
| `ExprKind` | `Message` | `say a~b` |
| `LoopKind` | `With` | (no witness row; see below) |

The other two rows in that constant are Phase 7's (`Command`, `Address::Command`).

**`LoopKind::With` is a Phase 5 row without a `loud.rs` witness**, and the reason is structural: a
`DO WITH` reaches `Loud::instruction` with `instruction.kind` still `InstructionKind::Do`, which this
crate implements, so `owned_message` deliberately emits no owner suffix. `run.rs`'s
`do_with_takes_the_loud_path` pins the exact unsuffixed string `"DO is not implemented"`.

### Measured, this session, both sides

Each program was run under the oracle wrapper from a fresh directory and then under
`REXX_ENGINE=ir rexx-run` (the message-send and list cases were also run under `REXX_ENGINE=tree-walker`,
byte-identical).

| program | oracle | this crate |
|---|---|---|
| `say 'abc'~length` | rc 0, stdout `3` | rc 120, `rexx-exec: a message send is not implemented (Phase 5)` |
| `say (1, 2)` | rc 0, stdout `1` then `2` | rc 120, `rexx-exec: a parenthesised list is not implemented (Phase 5)` |
| `expose x` | rc 158, Error 98.992 "The EXPOSE instruction may only be used from method invocations." | rc 120, `EXPOSE is not implemented (Phase 5)` |
| `forward` | rc 158, Error 98.947 "FORWARD can only be issued in an object method invocation." | rc 120, `FORWARD is not implemented (Phase 5)` |
| `reply 5` | rc 157, Error 99.919, a **translation** error | rc 120, `REPLY is not implemented (Phase 5)` |
| `guard on` | rc 157, Error 99.911, a **translation** error | rc 120, `GUARD is not implemented (Phase 5)` |
| `options 'x'` | **rc 0, no output at all** | rc 120, `OPTIONS is not implemented (Phase 5)` |
| `call ns:sub` | rc 158, Error 98.987 "Namespace \"NS\" not found in package ..." | rc 120, `CALL is not implemented (Phase 5)` |
| `do with index i over 'x'; say i; end` | rc 159, Error 97.1 "Object \"x\" does not understand message \"SUPPLIER\"." | rc 120, `DO is not implemented` |

Two things in that table matter to a spec and are easy to miss.

* **`REPLY` and `GUARD` are translation errors (99.x, rc 157) and `EXPOSE`/`FORWARD` are execution
  errors (98.x, rc 158).** So two of the four are refused before anything runs and two are refused
  when reached. Whichever way Phase 5 implements the legality check, that split is observable.
* **The bare `OPTIONS` *instruction* runs silently at rc 0 on the oracle.** This crate over-refuses it.
  That is separate from the `::OPTIONS` *directive*, which does have an effect (section 3).

### Sub-case refusals inside implemented constructs

These carry no owner suffix and so appear in no owner table, but their subject is Phase 5's:

* **`Loud::expression` reached from an assignment or `PARSE` target.** `run.rs`'s target arm doc records
  the measurement: `parse value 'a b' with q~x r` parses, and "the oracle answers `Error 97.1` (`Object
  "Q" does not understand message "X="`) where this crate reports a `Phase 5` gap".
* **`Loud::value_selector`** (`lib.rs`). `VALUE`'s three-argument form is refused wholesale. Its own doc
  and `phase-4-exclusions.txt`'s KNOWN GAP row split the oracle's dispatch three ways: an *empty*
  selector reads/writes `.environment` and is **Phase 5's**; `'ENVIRONMENT'` is the OS environment and
  is neither phase's; anything else is Phase 7's.
* **`ExprKind::DotVariable` beyond `.NIL`/`.TRUE`/`.FALSE`.** `eval.rs`'s arm answers those three and
  falls through to `Loud::expression(&expr.kind)` for every other name; `expr_owner` gives the variant
  `None`, so the message is the **unsuffixed** `"an environment symbol is not implemented"`.

  Measured this session, the pair that shows the gap's two faces:

  ```
  say value('.LOCAL')    oracle rc 0 "The Local Directory"   this crate rc 0 ".LOCAL"
  say .LOCAL             oracle rc 0 "The Local Directory"   this crate rc 120, unsuffixed message
  ```

  `phase-4-exclusions.txt`'s KNOWN GAP row ("VALUE ON A DEFINED ENVIRONMENT NAME IS SILENTLY WRONG AT
  RC 0") rules on this: declare the gap, do not build the subsystem to close it, because "the affected
  set is `.local`, `.environment`, every class name and every stream alias -- the whole Phase 5
  environment/`.local`-directory subsystem". **No owner is assigned to closing it**, on the ground that
  closing it *is* building the subsystem.

* **The stem-arithmetic stderr line.** `phase-4-exclusions.txt`'s KNOWN GAP row: for
  `b. = never touched ; say b. + 1` the oracle prints `*-* Compiled method "+" with scope "String".`
  ahead of the error lines and this crate omits it. The row attributes it: "method-dispatch
  bookkeeping ... which needs the object model 4a does not have, so it is Phase 5's."

## 3. The directives

`directive_gap` in `lib.rs` decides, per directive, whether installing it is a gap. Its predicate has
three clauses -- installing it runs code, changes a package setting a Phase 4 construct can read, or
resolves a name against a table this crate does not have. Its Phase 5 arms:

| arm | message |
|---|---|
| `DirectiveKind::Requires(_)` | `::REQUIRES is not implemented (Phase 5)` |
| `DirectiveKind::Options(_)` | `::OPTIONS is not implemented (Phase 5)` |
| `DirectiveKind::Class(..)` with `subclass`/`metaclass`/non-empty `inherit` | `::CLASS naming another class is not implemented (Phase 5)` |
| `DirectiveKind::Annotate(..)` with a non-`Package` target | `::ANNOTATE naming a target is not implemented (Phase 5)` |

`run.rs`'s directive tests assert these strings verbatim. `::ROUTINE EXTERNAL`, `::METHOD EXTERNAL` and
`::ATTRIBUTE EXTERNAL` are Phase 7's in the same function. Everything else installs and is **ignored**,
which the function's doc distinguishes from implemented: "4c has no object model, so a `::CLASS` that
names nothing and a `::METHOD` with a body are unreachable from any construct this phase runs."

### The two declared over-refusals, re-measured this session

`phase-4-exclusions.txt`'s section "EXCLUSIONS -- every directive except `::ROUTINE`, Phase 5" states
both. Both still hold:

* `say 'main ran'` with `::class foo subclass object` -- **oracle rc 0, stdout `main ran`**; this crate
  rc 120. The alternative is running `::class foo subclass zzznotaclass` at rc 0 where the oracle gives
  98.909 at rc 158.
* `say 'main ran'` with `::requires 'quiet.rex'`, the helper carrying only `::routine qfn public` --
  **oracle rc 0, stdout `main ran`**; this crate rc 120. With a helper whose first clause is
  `say 'PROLOG RAN'` and a `say hfn()` in the main body, the oracle prints `PROLOG RAN`, `main ran`,
  `HELPED` at rc 0. So presence is use: `::REQUIRES` loads and runs the required file's prolog, which
  is why ignoring it is not available.
* `::OPTIONS` has no over-refusal and the exclusions file says why: "its whole effect is to change
  package settings, unconditionally". Re-measured: `say digits() form() fuzz()` under
  `::options digits 12` prints `12 SCIENTIFIC 0` at rc 0 on the oracle, rc 120 here.

`::OPTIONS TRACE LABELS` is named a second time from inside the trace code: `run.rs`'s
`trace_invocation_entry` doc records that a routine in a file carrying `::options trace labels`
announces its `>I>`/`<I<` pair with identical bytes and no `trace` instruction of its own, and that this
crate refuses such a program rather than running it without the lines.

## 4. The `ExprKind` variants

`docs/superpowers/plans/phase-4-exclusions.txt`, section "EXPRKIND OWNERSHIP -- the expression forms
4a/4b do not evaluate", is the ruling of record. Read at HEAD it carries six rows, one of them
(`Call`) closed and kept for its history, one (`VariableReference`) 4b's, and four Phase 5's. The
section's own header says changing a row is a plan amendment.

What each parses to (`rexx-parse/src/ast.rs`, `enum ExprKind`) and what happens when one is evaluated
(`rexx-exec/src/eval.rs`, `expr_owner` in `lib.rs`):

* **`Message { target, name, super_class, args, cascade }`** -- `target~name`, `target~~name` and
  `target[...]`, all one variant. The ast doc gives the measurement for the collapse: `"abc"[2]` and
  `"abc"~"[]"(2)` both give `b`, because `parseCollectionMessage` builds a `RexxExpressionMessage`
  named `[]` (`LanguageParser.cpp:3317`). `name` is upcased for every spelling and is `Box<[u8]>`
  rather than a `SymbolId`, because a method name resolves against a behaviour keyed by string and
  because a name from a literal was never seen by the scanner. `cascade` is `~~`, which discards the
  result and yields the target. Evaluation: loud, `"a message send is not implemented (Phase 5)"`.
  `InstructionKind::Message` travels with it -- its `term` is always an `ExprKind::Message`.
* **`List(Vec<Option<Expr>>)`** -- a comma-separated parenthesised list, which builds an array. Unlike
  a call's argument list it **keeps trailing omitted elements**: the ast doc's measurement is
  `(1,)~size` is 2 and `(1,,)~size` is 3, against `f(1,)` passing one argument
  (`parseFullSubExpression` returns `total` where `parseArgList` returns `realcount`,
  `LanguageParser.cpp:3145`). Evaluation: loud, `"a parenthesised list is not implemented (Phase 5)"`.
  `RAISE ... ADDITIONAL (a, b)` is the one `RAISE` shape that still fails loudly, and it fails through
  this variant rather than through `RAISE`; `RAISE ... ARRAY (a, b)` reaches the identical oracle bytes.
* **`QualifiedCall { namespace, name, args }`** -- `ns:name(...)`. The exclusions row's reason: it
  needs a resolved package/namespace object, namespaces come from `::REQUIRES`, and `::REQUIRES` is on
  the Phase 5 side of the `::ROUTINE` carve-out. Evaluation: loud, `"a namespace-qualified call is not
  implemented (Phase 5)"`.
* **`ClassResolver { namespace, name }`** -- `ns:name` with no argument list. Evaluation: loud, `"a
  namespace-qualified class lookup is not implemented (Phase 5)"`.
* **`DotVariable(SymbolId)`** -- `.name`. `ExpressionDotVariable` and `SpecialDotVariable` are
  collapsed into one variant, and the ast doc explains why that is not two syntaxes: the C++ preloads
  `.nil`/`.true`/`.false` into `dotVariables` as `SpecialDotVariable` (`LanguageParser.cpp:782`-`784`)
  purely as a retrieval optimisation. **Partially in scope**, as section 2 records, and `expr_owner`
  answers `None` for it, so its loud path names no phase.

The IR side draws the identical line: `ir/compile.rs`'s `Call` arm comments that "`CALL ns:name` is
Phase 5's loud gap", and everything it does not special-case falls to `Op::Generic`, which re-enters
the tree-walking `step`.

## 5. `>M>` and `>N>`

`rexx-exec/tests/trace_oracle.rs`'s `PREFIX_COVERAGE` is the ownership record and is asserted rather
than described. Read at HEAD:

```rust
(">M>", Coverage::Owned("Phase 5")),
(">N>", Coverage::Owned("Phase 5")),
```

with `WITNESSED_PREFIX_COUNT = 16`, `OUT_OF_SCOPE_PREFIX_COUNT = 3` (the third is `+++`, Phase 7's),
`OWNER_PHASES = &["Phase 5", "Phase 7"]`, and a test asserting the table's prefix set equals
`support::TRACE_PREFIXES`, itself read from the oracle's own `trace_prefix_table`
(`RexxActivation.cpp:3567`-`3587`, read this session).

### The producers, read at the oracle

* `>M>` has **two**: `MessageInstruction.cpp:209` and `ExpressionMessage.cpp:219`, both calling
  `RexxActivation::traceMessage`. So the instruction and the expression forms each emit it.
* `>N>` has **one**: `ExpressionClassResolver.cpp:135`, calling `traceClassResolution`.

### Two measured transcripts, taken this session

```
trace i
zq = 'abc'~length
```

oracle rc 0, stderr:

```
     2 *-* zq = 'abc'~length
       >L>   "abc"
       >M>   "LENGTH" => "3"
       >>>   "3"
       >=>   ZQ <= "3"
```

This crate emits the `*-*` line and then the loud message, at rc 120.

```
trace i
zq = ns:hfn()
zr = ns:hcls
::requires 'helper.rex' namespace ns
```

with `helper.rex` holding `::routine hfn public` and `::class hcls public`: oracle rc 0, stderr:

```
     2 *-* zq = ns:hfn()
       >F>   HFN => "HELPED"
       >>>   "HELPED"
       >=>   ZQ <= "HELPED"
     3 *-* zr = ns:hcls
       >N>   NS:HCLS => "The HCLS class"
       >>>   "The HCLS class"
       >=>   ZR <= "The HCLS class"
```

**This corrects a doc comment in the tree.** `trace_oracle.rs`'s own prose for `>N>` reads
"`traceClassResolution`, a namespace-qualified name, which needs `::REQUIRES`; `ExprKind::QualifiedCall`
is Phase 5's there." Measured, a `QualifiedCall` traces `>F>` with the bare name, not `>N>`; `>N>` comes
from the `ClassResolver` form and carries the **qualified** name `NS:HCLS`. Both variants are Phase 5's,
so the ownership is right; the construct named beside the prefix is not.

### What Phase 5 does *not* automatically get

`phase-4c-gate.md`'s criterion 3 is explicit: "**THE CORPUS CANNOT PIN A TRACE INDENT, FOR ANY PREFIX,
AND THIS CRITERION DOES NOT CLAIM OTHERWISE.**" DEVIATION 0 normalises the run of spaces after the
marker on both sides in `tests/support/mod.rs`, shared by `corpus.rs` and `trace_oracle.rs`, and the
committed `.expected` files are normalised at comparison time. The gate's inheritance line says closing
it "needs either an unnormalised comparison mode or in-crate exact-stderr assertions". So a `>M>` line
landing at the wrong indent is invisible to every corpus-based instrument here.

## 6. The corpus and L1 tables

### `rust/corpus/bif-exempt.txt`

Counted this session with `/bin/grep -av '^#' … | cut -f2 | sort | uniq -c`: **81 rows**, of which

* **22 read `Phase 5`** -- `D2C::test10#1` and every `XRANGE::test_xrange_hex_hex`/`test_xrange_three`
  row. The file's header explains the category as "The row's own operands send a message ... Needs
  dispatch."
* **9 read `UNATTRIBUTED:an environment symbol`** -- `LENGTH::test025`, three `REVERSE` rows, four
  `STREAM` rows and `VALUE::test019`. The header: "`ExprKind::DotVariable` is loud and `expr_owner`
  gives it no phase, so the harness has nothing to derive one from."
* 47 read `4c` and 3 read `ANOMALY`.

`phase-4c-gate.md:411` reads "47 blocked on builtins ... 22 on message sends in their own operands
(Phase 5), 9 on an environment symbol, and 3 on `BEEP`". **That still holds exactly at HEAD.**

**One inaccuracy in that file's header, found by opening the source it describes.** The header
illustrates the Phase 5 category with "`XRANGE`'s hex-pair bodies write `'41'x~c2d`". `/bin/grep -a`
over `ootest/ooRexx/base/bif/XRANGE.testGroup` (checkout at r13178, confirmed with `svn info`) finds no
`'41'x` anywhere. The message sends those bodies actually contain are `self~sequence("space")` /
`self~sequence("alnum")` and `xrange()~copies(2)`; `start~c2d` and `n~d2c` live one level down, inside
the `::method range` helper that `sequence` calls. `D2C::test10`'s first row is
`self~assertSame('7F'x||'FF'x~copies(249), d2c(vlong))`, so the header's second example is right.

**`BEEP` is not Phase 5's**, despite being in the same file. Its three rows carry `ANOMALY`, the
harness's word for "exited non-zero and named no construct". `phase-4-exclusions.txt`'s KNOWN GAP row
rules on it directly: "**NO OWNER.** This is not a builtin gap, so Phase 7's file-and-stream rows do
not cover it; it is not dispatch, so Phase 5's do not either." The enumeration is external and exact --
`interpreter/runtime/NativeFunctions.h`, read this session, is three `INTERNAL_ROUTINE` lines:
`Directory`, `Filespec`, `Beep`, gathered into `rexx_routines[]` in `InternalPackage.cpp` -- and this
platform's `SysNativeFunctions.h` adds none. The row's measurements: `retc = beep(262, 1)` is oracle
rc 0 answering the null string against rc 213 Error 43.1 here; `filespec` and `directory` diverge
identically and reach no instrument, because `corpus/builtin-status.txt`'s rows *are*
`rexx_inventory::builtins::NAMES` and these three are not in it.

### `rust/corpus/keyword-exempt.txt`

Read in full this session: eight rows -- three `Phase 7`, three `RAISED`, two `4c`. The header states
flatly: "**No body here is blocked by Phase 5**, so 4b owes this table nothing further." The two `4c`
rows are `CALL::test_on_name` and `CALL::test_9`, which the header attributes to `CHARIN` and `LINEIN`,
whole exclusions and so Phase 7's.

**`phase-4c-gate.md`'s inheritance line has drifted in spelling here.** It reads "The eight remaining
`keyword-exempt.txt` rows are unowned. Three cannot be made to pass at all ... the other five are Phase
7's." At HEAD only three rows *spell* `Phase 7`; the other two spell `4c` and are Phase 7's only by the
header's own prose. The substance is unchanged and nothing in it is Phase 5's.

### `rexx-exec/tests/assertions.rs` -- the `base/expressions` L1 table

`/bin/grep -c 'unblocked_by: "Phase 5"'` gives **35** this session, and `/bin/grep -a` over the
`method:` fields gives `test_binary` 18, `test_hexadecimal` 15, `test_string_range` 2 -- all from
`Literals.testGroup`. So **every row in `EXEMPT` is Phase 5's**, and the module doc says why the
attribution is not "whichever construct it hits first": `test_string_range`'s prelude hits `xrange()`
first, but its very next prelude line is `all~changeStr(.String~cr, "")`, so implementing 4b's `Call`
would move the blocker one line and not make the row pass. `the_exempt_set_matches_the_current_blocked_rows`
asserts the set in both directions -- a row on the list that *starts* passing is as red as one off it
that fails.

### Corpus programs parked for Phase 5

`corpus/num/digits_rounding.rex`, `corpus/num/exponential.rex` and `corpus/num/operators.rex` exist on
disk and, checked this session with `/bin/grep -c` against each of `phase-4a.txt`, `phase-4b.txt` and
`phase-4c.txt`, appear in **none** of them. `phase-4a.txt`'s header and `corpus/README.md` both give the
reason: all three build an `ExprKind::List` from a comma in a `SAY`, so "they stay in `corpus/num/` for
Phase 5, once `List` exists". Until then they are compared against the oracle by nothing.

`phase-4c.txt`'s header records the directive half: "`::ROUTINE`, and calls into one (4c Task 13). No
other directive: the other eight are Phase 5's."

### The `base/keyword` L1 drop table

`docs/superpowers/plans/l1-coverage.md:560`-`:580` and `:624`-`:670`. Its committed breakdown gives
`body uses a message send` at 96 methods / 491 calls and calls that "the Phase 5 share". Its section
"The list 4c and Phase 5 inherit" opens "**Every phase gap here is 4c's. Not one body is blocked by
Phase 5**", which is about the *blocked bodies*, not the dropped ones. Of the 145 assertion calls in
the 13 groups that yield no extractable body at all, the table attributes 119 to `body uses a message
send (Phase 5)`. **These figures are quoted from that document and were not re-measured here** -- doing
so needs the extractor run, which needs a build.

### The `base/bif` drop table

`phase-4c-gate.md:451`: "**The `base/bif` drop table's largest categories are Phase 5's**: 418 calls in
bodies that send a message and 467 in bodies whose statements this extractor cannot carry."
`rexx-extract/src/bif.rs` still carries both reasons at HEAD -- `DropReason::MessageSend` ("body uses a
message send", doc: "Needs dispatch, which is Phase 5's") and `DropReason::UnsupportedStatement` ("The
body reaches a statement that is neither an assignment nor an ooTest assertion: a loop, an `IF`, a
`CALL`, a `PARSE`") -- and `DropReason::ALL` makes the harness print a per-reason breakdown. **The two
figures were not re-measured in this session**, for the same reason: it needs a `cargo test` run and
another agent is building in that target directory. A Phase 5 spec can recover them with one run.

## 7. The benchmark axes

`rexx-bench/src/bin/rexx-bench-suite.rs`'s `AXES`, read at HEAD, declares exactly three
`Role::Blocked`: **`alloc`, `dispatch`, `heapshape`**. Run this session against
`target/release/rexx-run` from a fresh directory, all three exit **120** with
`rexx-exec: a message send is not implemented (Phase 5)`.

The suite polices the role rather than trusting it. `every_blocked_axis_still_fails_on_this_crate`'s doc:
"When Phase 5 lands message sends these three exit 0, and without this they would go on being printed
as unrunnable with status 0 and an empty message while nothing timed them." So **Phase 5 landing message
sends turns the benchmark suite red by design**, and the repair is a deliberate re-roling.

`rexx-bench/src/lib.rs`'s `the_exemptions_are_true_of_the_programs_they_name` names the same moment:
a check keyed on "fails on this crate" "would have forced the wrong decision at Phase 5, when
`heapshape` starts running and its real reason for exemption still holds".

`phase-4d-gate.md` scopes three D9 dimensions to Phase 5 in its handover section (`:646`-`:648`):
`dispatch` ("a D9 dimension nothing in 4d measures"), `alloc.rex` (with `alloc4c.rex` covering the
allocation dimension on the 4c surface only), and `startup` ("against D2's absolute target of about 55
ms for 5,203 lines"). The same document's `:60` is where the roadmap's withdrawn debt route survives:
"The debt mechanism below survives only for `dispatch`, `alloc.rex` and `startup`, where it scopes work
to Phase 5 rather than conceding a bar."

The `startup` axis is reported by the harness as a fixed per-process offset with a standing caveat, the
same sentence in `rexx-bench-suite.rs`, `phase-4e-anchor.md:170` and `perf-baseline.md`: "**Not
comparable, and not a pass.** This crate has no `CoreClasses.orx` bootstrap yet (Phase 5), so it starts
fast by not doing the work the oracle does at startup."

## 8. Machinery already in the tree, waiting

* **`rexx-core::behaviour::BehaviourTable`** -- `define`, `set_superclass`, and a `lookup` that walks
  the superclass chain with a visited set, because "the bootstrap object graph is genuinely cyclic --
  `.class` is an instance of itself". `MethodId(pub u32)`, methods keyed by uppercased name. Grepped
  this session: it is re-exported from `rexx-core/src/lib.rs` and used by **nothing outside
  `rexx-core/tests/behaviour.rs`**. `rexx-exec` uses only the `BehaviourId` constants.
* **`BehaviourId(pub u16)`** with exactly four constants: `STRING = 0`, `ARRAY = 1`, `OBJECT = 2`,
  `STEM = 3`. Every allocation in `rexx-exec` picks one of `STRING`, `STEM` or (via `Heap::alloc`)
  `OBJECT`.
* **`Body::Array(Vec<ObjRef>)` and `Body::Instance(Vec<(String, ObjRef)>)`** exist and are traced by the
  collector's one exhaustive `match`. Grepped this session: both are constructed only in `rexx-core`'s
  own tests. `Body::Instance` is the shape the value-representation rule points at for a new class's
  instance state.
* **`Object::has_uninit` and `CollectStats::pending_uninit`** exist and work (`rexx-core/tests/uninit.rs`
  covers resurrection through a `WeakRef`). Nothing in `rexx-exec` ever sets the flag; `lib.rs`'s
  allocation path carries a `debug_assert!` that the list is empty and a comment naming itself as the
  site that owes delivery: "`UNINIT` needs a class to define it and message sends are Phase 5".
* **The patchable call site.** `ir.rs`'s `CallSite(Cell<Option<Resolved>>)` and `Calls`, one slot per
  `Op::Call`/`Op::CallExpr`. **Its correctness argument names `::REQUIRES` as the thing that would break
  it**: "`Interp::routines` never rebinds a name it has already bound ... The property the table needs is
  that one, and not 'the map is written once', which is a stronger thing that happens to be true today
  and would stop being the reason if a `::REQUIRES` or an external-file call ever installed a routine
  mid-run."
* **The plan cache's single-program assumption.** `phase-4f-record.md:3330` records the `BodyKey`
  tripwire and the same caveat: "`::REQUIRES` is a Phase 5 gap and an external routine file a Phase 7
  one, and **both are routes by which a second program could be loaded**. The tripwire is what will
  still be standing when either lands." It also records that only 2 of 381 programs and 7 of 1487 tests
  reach the plan-cache hit path at all, so the corpus is a weak witness there.
* **`Entry`, in `rexx-exec/src/activation.rs`.** Three arms -- `TopLevel`, `InternalCall`, `Routine` --
  and its doc carries an oracle table with a `::METHOD` row already measured: `PROCEDURE` first gives
  17.1 at rc 239, `USE LOCAL` first **runs at rc 0**. `exec_use`'s `Use::Local` arm reads
  `let method_invocation = match self.activation().entry { Entry::TopLevel | Entry::InternalCall |
  Entry::Routine => false, };` -- every arm false, no wildcard. The enum's doc states the guarantee:
  "Every reader matches on this exhaustively and without a wildcard, so an entry kind added here cannot
  be left unanswered at any of them." So adding `Entry::Method` is a compile error at every reader, and
  `exec_use`'s 98.993-versus-99.910 choice is one of them.
* **The parser is already there.** `rexx-parse/benches/parse.rs` includes `CoreClasses.orx` by path and
  asserts 41 top-level instructions, 347 directives and 2,390 nested instructions (`phase-3-gate.md`
  criterion 4, "MET", records the run and 2.64 ms for the parse). Those assertions run under
  `cargo bench -- --test`; they were **not** re-run here.

## 9. Where the gate documents have moved

* `phase-4c-gate.md`'s inheritance line "the other five are Phase 7's" no longer matches the file's
  spelling; see section 6.
* `trace_oracle.rs`'s `>N>` prose names `ExprKind::QualifiedCall`; the measured producer is the
  `ClassResolver` form. See section 5.
* `bif-exempt.txt`'s header illustrates the Phase 5 category with a string that is not in the source
  it describes. See section 6.
* `2026-08-01-phase-4bc-scoping.md:570` states the post-4b/4c residue as "Phase 5's six
  `InstructionKind` variants ..., Phase 5's four `ExprKind` variants ..., and Phase 7's `Command`".
  At HEAD `owners.rs`'s `EXPECTED_OUT_OF_SCOPE` additionally carries `InstructionKind::Call::Qualified`
  (Phase 5), `InstructionKind::Address::Command` (Phase 7) and `LoopKind::With` (Phase 5), the first
  two arriving with the arm-grained split.
* `phase-4c-gate.md`'s "`ExprKind::DotVariable` is loud and carries no owner" still holds, but the
  wording invites a wrong reading: `owners.rs` records the variant as `Owner::InScope`, because three
  of its names evaluate. It is *partly* in scope and *wholly* unattributed.

---

# Part 2: the open design decisions

Each is a question with its alternatives and what each would cost. **None is answered here.**

### Q1. Is the object model bootstrapped by running `CoreClasses.orx`, or built natively in Rust?

* **(a) Run the Rexx source**, as the oracle does, and inherit 32 classes from a file this project does
  not have to write or keep in sync. Costs: every semantic gap in the file becomes a Phase 5 blocker in
  bulk (the roadmap's own risk row), and startup pays parse-plus-execute on every run against D2's
  absolute ~55 ms budget.
* **(b) Build the core classes in Rust** and treat `CoreClasses.orx` as a specification to be read
  rather than executed. Costs: 4,193 lines of behaviour hand-ported, with no differential instrument
  that can tell a faithful port from a plausible one; the file changes upstream and the port silently
  stops matching.
* **(c) A split** -- native for the classes the bootstrap itself needs, sourced for the rest.
* *Evidence each would need:* for (a), a parse-and-execute timing against the 5.1 ms C++ baseline, and
  a triage pass over `CoreClasses.orx`'s construct inventory saying which gaps it opens; for (b), a
  statement of what plays the oracle's role when the Rust class is the only implementation; for (c),
  the boundary drawn in terms of the bootstrap's own dependency order rather than convenience.

### Q2. Is method lookup resolved at compile time in the IR, or always dynamic?

The IR already has the slot: `CallSite(Cell<Option<Resolved>>)`, one per call op, populated on first
resolution. Its whole correctness argument is that `Interp::routines` is append-only -- and it names
`::REQUIRES` as the thing that breaks that.

* **(a) Extend the existing per-site cache to message sends**, keyed by receiver behaviour with a guard.
* **(b) Always dynamic**, a behaviour-table walk per send, and no cache at all until it is measured to
  be needed.
* **(c) Cache with explicit invalidation** on any event that can redefine a method.
* *What the two-engine split forces:* the tree-walking `step` and the compiled `run_ops` must agree byte
  for byte, so whatever caching exists on one side must not be observable on the other -- and `ir_dual`
  is nearly the only thing pinning tree-walker behaviour at all. `Op::Generic` re-enters `step`, so a
  send left uncompiled is not a correctness problem but is a second dispatch path.
* *Evidence each would need:* a statement of what invalidates a cached lookup in this language
  (`::METHOD` at load, `~setMethod`, `~define`, a class reopened by a later `::REQUIRES`), and a
  measurement on the `dispatch` axis -- which is precisely the axis nothing currently measures.

### Q3. Where does the object model live -- new crates, or inside `rexx-exec`?

The roadmap's file structure names `rexx-classes/` and `rexx-lib/`; neither exists. `rexx-exec/src/run.rs`
is already large enough that its split is an open plan item of its own
(`docs/superpowers/plans/2026-08-15-task-6-review-and-run-split.md`).

* **(a) New crates as planned.** Costs: the boundary has to be drawn before the code exists, and the
  roadmap's own rule is that a later phase reaching past a crate boundary means the boundary is wrong.
* **(b) Inside `rexx-exec`**, deferring the split. Costs: the run.rs split gets harder, not easier.
* *Evidence:* what the interface between "the interpreter" and "the class library" actually is --
  whether a class can be defined without the executor, and whether the executor can dispatch without
  the class library.

### Q4. Does `Body` widen, or does every new object kind arrive boxed?

`const _: () = assert!(size_of::<Body>() <= 80);` will trip. The value-representation design hands
Phase 5 a rule (hot inline, cold boxed) but the rule is advice until this phase adopts or declines it.

* **(a) Adopt the rule**: `Body::Instance` and a boxed payload for anything new. Costs: a pointer chase
  on every instance-variable access, and a decision per class kind.
* **(b) Widen deliberately** for specific kinds, raising the bound with a recorded measurement each time.
* **(c) Restructure**, e.g. boxing `Stem` so `Num` sets the width -- explicitly *not* recommended as a
  standalone optimisation by the design doc, but possibly right as part of adopting the rule.
* *Evidence:* the measurement the doc says nobody has taken -- decomposing `rexxcps`' allocator family
  into `Text`, `Number` and tail keys -- plus, for any widening, the before/after on the interleaved
  benchmark baseline the roadmap requires be pinned before Phase 5 changes anything.

### Q5. What is the class registry keyed by, and does `BehaviourId` survive?

`BehaviourId` is a `u16` with four constants and `BehaviourTable` has never been instantiated by the
interpreter.

* **(a) Keep `BehaviourId` and grow the table**, one id per class including user classes.
* **(b) Replace it** with an `ObjRef` to a real class object, since in Rexx a class *is* an object and
  `.class` is an instance of itself.
* **(c) Both** -- an `ObjRef` identity with a `BehaviourId` fast path for the primitive kinds.
* *Evidence:* whether anything observable distinguishes them; how `~class`, `~isA` and metaclasses
  behave under each; and whether the collector's root set stays enumerable (D1's own criterion) if class
  objects live in the arena.

### Q6. What does Phase 5 do about `.environment` and `.local`, and does it close the `VALUE` gap?

The KNOWN GAP row rules "declare the gap, do not build the subsystem to close it" and assigns no owner,
on the ground that the owner *is* Phase 5. So the question is what closing it means.

* **(a) Build the directory objects first** and let `.NAME` lookup, `VALUE`'s empty-selector form and
  the class registry all resolve through one table. Costs: the whole subsystem lands as one unit and
  has no partial state.
* **(b) A name table without directory objects** -- enough to make `.LOCAL`, `.ARRAY` and class names
  resolve, with `~` on the result still loud. Costs: a second, temporary mechanism to delete later.
* *Evidence:* whether any construct in `CoreClasses.orx`'s own bootstrap needs a directory *object*
  before the class registry exists -- which is a read of the file, not an argument.

### Q7. In what order are the four `directive_gap` over-refusals retired, and which are actually Phase 5's?

`::CLASS naming another class` and `::ANNOTATE naming a target` are object-model refusals.
`::REQUIRES` is a loader. `::OPTIONS` is a **package settings** mechanism whose measured effect
(`digits() form() fuzz()`) has nothing to do with the object model, and the bare `OPTIONS` *instruction*
runs silently at rc 0 on the oracle today while this crate refuses it.

* **(a) Keep them together** as one Phase 5 unit, as the ownership tables have it.
* **(b) Split `::OPTIONS` and the `OPTIONS` instruction out** as package/settings work that could land
  ahead of dispatch -- which would also delete two over-refusals early.
* *Evidence:* what `::OPTIONS ALL` and `::OPTIONS TRACE LABELS` need that is not already in the tree
  (the `trace_invocation_entry` route is measured and known); and whether the `OPTIONS` instruction has
  any observable effect at all, since the oracle answers rc 0 with no output.

### Q8. What does `::REQUIRES` do to the caches?

Landing it makes a second program loadable mid-run, which is exactly the condition two caches say they
depend on not happening: the `CallSite` table's append-only argument and the plan cache's `BodyKey`
pairing.

* **(a) Preserve append-only** by making a required package's routines a separate resolution namespace
  rather than entries in `Interp::routines`.
* **(b) Allow rebinding and invalidate** both caches on load.
* **(c) Resolve eagerly at load** so nothing is ever re-resolved after execution starts.
* *Evidence:* what the oracle does when two packages export the same public routine name, and when a
  `::REQUIRES` appears in a file that is itself required; plus the plan-cache tripwire re-run with a
  program that loads two programs, which is the case it was built for and has never seen.

### Q9. Where is the Phase 5 / Phase 6 line for `REPLY` and `GUARD`?

Both are on Phase 5's owner list. Both are concurrency instructions, and Phase 6 owns concurrency,
activities and guard locks. Measured, both are refused by the oracle *at translation time* (99.919,
99.911) outside a method.

* **(a) Phase 5 implements the legality check only**, so a `REPLY` inside a method translates and then
  fails at run time in whatever way Phase 6 later replaces.
* **(b) Phase 5 implements them fully** with a degenerate single-activity semantics.
* **(c) Move both rows to Phase 6** and accept two Phase 5 owner rows changing phase, which is a plan
  amendment.
* *Evidence:* whether `CoreClasses.orx` itself uses either -- a grep of the file, not an argument -- and
  what the oracle does for `REPLY` in a method under no concurrency at all.

### Q10. Does `ExprKind::List` become a real Array, and when?

`2026-08-01-phase-4bc-scoping.md`'s D7 already framed this and left three options on the table
(A: stays Phase 5's; B: 4c ships a newline-joined-string approximation as a recorded deviation;
C: 4c ships nothing). 4c shipped nothing, so the live question is narrower: whether Phase 5's `List`
must be a real Array from the first commit, because three corpus programs and the whole `SAY`-with-comma
surface come back with it.

* *Evidence:* the ast doc's own measurement is the discriminator -- `(1,)~size` is 2 and `(1,,)~size`
  is 3, which no string approximation reproduces, but which is only observable once `~` works. So the
  approximation is undetectable until dispatch lands and detectable immediately after.

### Q11. What instrument proves "32 classes exist and respond"?

The Phase 5 gate's wording is a claim about behaviour, and this project's standing rule is that a
criterion needs an instrument that can fail.

* **(a) A `phase-5.txt` corpus subset** compared byte for byte against the oracle, extending the
  existing `corpus.rs` mechanism.
* **(b) The L2 rung** -- ooTest groups, which is what the phase table's Rung column says.
* **(c) An in-crate enumeration** asserting each class's presence and method set.
* *Evidence:* what a "responds" check looks like that a class registry containing 32 empty entries would
  fail; and whether the L2 harness can start at all -- `docs/superpowers/plans/2026-07-27-rust-rewrite.md:304`
  records that the ooTest framework "cannot start" without `SysFileExists` and `.File`, which are
  Phase 7's, so the rung named in the gate may not be reachable at the gate.

### Q12. How does Phase 5 pin the indent of `>M>` and `>N>`?

Criterion 3's normalisation means an off-by-two indent on a new trace line is invisible to every corpus
instrument here, and the 4c gate hands that forward as the first item Phase 5 inherits.

* **(a) An unnormalised comparison mode** in `tests/support/mod.rs`, applied to selected witnesses.
* **(b) In-crate exact-stderr assertions** for the message-send and class-resolution lines, the way the
  three `run.rs` indent tests work today.
* **(c) Accept the hole** and say so in the gate, as 4c did.
* *Evidence:* whether the oracle's indent for a `>M>` inside a nested expression is derivable from the
  rules already implemented, or is a new rule -- one transcript per nesting depth settles it.

### Q13. What is the performance regression guard, and who runs it?

The roadmap suspends optimisation for Phase 5 and makes that conditional on a pinned interleaved
baseline and on running the axes as a guard. But Phase 5 landing message sends turns three axes from
`Role::Blocked` to running, which turns the suite red by design.

* **(a) Re-role the three axes at the moment the refusal disappears**, and take a first real number for
  `dispatch`, `alloc` and `heapshape`.
* **(b) Keep the guard to the classic axes only** (`arith`, `compound`, `strings`, `varlookup`,
  `alloc4c`, `rexxcps`) and treat the three as new measurements rather than regressions.
* *Evidence:* the pinned baseline itself, which the roadmap says "cannot be reconstructed once the tree
  moves"; and a decision on `startup`, which stops being "not comparable" the moment a bootstrap exists
  and becomes D2's actual gate number.

### Q14. What shape does the security manager's interception design take (D12)?

D12 is settled as a split, not as a design. Phase 5 owes the object, its installation path, and the
hooks in dispatch and in `.local`/`.environment` lookup, in a form Phase 7 can add command and stream
call sites to.

* **(a) A trait object consulted at each interception point.**
* **(b) A per-activation optional handle**, mirroring the C++'s own placement.
* *Evidence:* what `base/security.manager/SecurityManager.testGroup` actually exercises -- it is in the
  checked-out suite and can be read now; and whether an interception point that is a no-op when no
  manager is installed costs anything on the paths the benchmark axes measure.

---

## Appendix: things adjacent to Phase 5 that are not Phase 5's

Recorded so a spec does not absorb them by proximity.

* **`BEEP`, `FILESPEC`, `DIRECTORY`** -- the interpreter's internal-package routines. Explicitly no
  owner; `phase-4-exclusions.txt` says Phase 5's rows do not cover them because it is not dispatch.
* **The eight `keyword-exempt.txt` rows** -- three unfixable by construction, five Phase 7's.
* **`Loud::compound_expose`** (`PROCEDURE EXPOSE a.1`) -- a disclosed gap inside an implemented
  instruction, with no owner and nothing scheduled.
* **`Loud::value_selector`'s (b) and (c) branches** -- the OS environment and the platform selector.
  Only the empty-selector branch is Phase 5's.
* **The `POS` window-overrun deviation** and the `changestr` crash behind it -- a permanent chosen
  divergence.
* **The 512 MiB interpreter-stack reservation** and the allocation-discipline cause behind the
  large-string thresholds -- `phase-4-exclusions.txt:1766` says outright: "This cause is UNASSIGNED. It
  is not a Phase 5 or Phase 7 feature."
* **The time-zone limb of `DATE`/`TIME`** -- "a time-zone database is Phase 5-**or-later**
  infrastructure", which is not an assignment.
* **The pre-Phase-5 defect plan** (`docs/superpowers/plans/2026-08-14-pre-phase-5-defects.md`) is a
  prerequisite, not inherited surface: its tasks are 4-era defects to be closed before Phase 5 starts.
  It was in flux while this document was written and is not enumerated here.
