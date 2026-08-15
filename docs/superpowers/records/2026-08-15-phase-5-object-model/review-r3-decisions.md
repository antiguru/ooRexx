# Review R3 -- the decisions, D25 to D36

Reviewer lens: the decision blocks themselves. Which decision will have to be reversed during Phase 5,
and which decision the spec failed to make will block a task.

Everything labelled CONFIRMED rests on a command run in this session, named at the finding.
Everything labelled PLAUSIBLE is reasoning from documents I read, and says so.

## Method, and what I could not reach

**What I ran.** The oracle under the standard wrapper from `.../scratchpad/r3probe`, a directory I
created empty for this review, with absolute paths and stdout/stderr/status read as three descriptors.
`rust/target/release/rexx-run` for the Rust side. `/bin/grep -a` for every count and every exhaustive
search. Nothing from `corpus/oracle-crashes.txt` was run.

**What I searched for, so the gaps are visible.** In `CoreClasses.orx`, `StreamClasses.orx` and both
`PlatformObjects.orx`: `^\s*::[a-zA-Z]+` for the directive population; `EXTERNAL` on directive lines;
the words `self`, `super`, `expose`, `forward`, `raise`, `signal`, `interpret`, `guard`, `reply`,
`.methods`, `.context`, `~new`. In the Rust tree: `CallSite`, `BodyKey`, `BehaviourTable`,
`BehaviourId`, `MethodId`, `directive_gap`, `instruction_owner`, `expr_owner`, `SPLIT_TABLE_PHASES`,
`AXES`, `PREFIX_OFFSET`, `hyperfine`, `bench-suite`. In the C++: `createInstance()` in `Setup.cpp`,
`BASEIMAGELOAD`, `sendMessage(GlobalNames::INIT`, the three `ClassClass.cpp` citations.

**What those terms cannot reach.** A method-body construct that `CoreClasses.orx` reaches only through
an expression I did not name -- I read the prolog (lines 39-125) in full but sampled the 4,000 lines of
method bodies by keyword only. A native entry point required by the *bootstrap* that is neither a
directive nor a keyword I grepped. And the Windows and BSD legs: I read the Windows
`PlatformObjects.orx` but ran nothing on any platform but this one.

## Findings by severity

| | count |
|---|---|
| Critical | 5 (C1-C5) |
| Important | 10 (I1-I10) |
| Minor | 7 (M1-M7) |

Four of these are addressed to the human partner as challenges to settled calls and are marked
**[CHALLENGES A SETTLED CALL]**: C1 and I9 against D25's mitigation, C5 against D27's boundary, I1
against D28's silence about D24, and C2/C3 against D26's three-file scope. None of them asks for the
call to be reversed; each says the *reasoning* or the *mitigation* recorded beside it does not hold.

---

# Part 1 -- the two headline answers

## The decision that will be reversed: D26's three-file scope, and it fails late

**CONFIRMED.** D26 says all three `.orx` files are executed in this phase, and exit criterion 1 makes
that the gate. Three measurements, each run this session:

1. **`::METHOD ... EXTERNAL 'LIBRARY REXX <name>'` binds eagerly, at directive-install time, before the
   main body runs.** Program: `say 'MAIN RAN'` / `::class foo` / `::method bar external 'LIBRARY REXX
   nosuchentrypoint'`. Oracle: **rc 166, stdout empty**, stderr `Error 90.998: Unable to find external
   method "nosuchentrypoint".` Stdout being empty is the load-bearing half -- presence is use.

2. **`StreamClasses.orx` is mostly external-method declarations.** `/bin/grep -a -n -i -E
   '^[[:space:]]*::(method|routine|attribute).*external'` names, among others, `stream_init`,
   `stream_charin`, `stream_lineout`, `stream_position`, `qualify`, `query_exists`, `file_exists`,
   `file_qualify`, `file_list`, `file_make_dir`, `rexx_create_queue`, `rexx_push_queue`. These are the
   stream model, the file system and the external data queue.

3. **The roadmap's Phase 7 row is, verbatim, "`StreamClasses.orx` runs"** (`2026-07-27-rust-rewrite.md:445`),
   and D11 assigns `.File` to Phase 7 by name.

So Phase 5's criterion 1 claims a deliverable the roadmap assigns to Phase 7, and under the oracle's own
eager-binding rule it cannot be met by deferring the bodies: the *declarations* must resolve.

**Why the failure is late rather than early.** D25's method is to grow the native set "one refusal at a
time" by running `CoreClasses.orx`. `call 'StreamClasses.orx' rexxPackage` is at `CoreClasses.orx:122`,
the second-to-last executable clause of its prolog, after `.environment` is populated, after the sixteen
`.String~defineClassMethod` calls, and after all eleven `~inherit` / `~inheritInstanceMethods` calls.
A discovery-driven phase reaches it only once the entire native layer is standing. `rexx-lib` will by
then have embedded all three files, sha256'd all three, and been written to run all three.

**Cost of reversing.** `rexx-lib`'s embed set and its bootstrap entry point; the sha256 manifest; exit
criterion 1; exit criterion 6, because a cold-start figure that omits `StreamClasses.orx` is not the
figure D2 asks about (D2 sizes the target as "5,203 lines of `CoreClasses.orx` + `StreamClasses.orx`",
`2026-07-27-rust-rewrite.md:157`); and the roadmap's Phase 7 row, which would then be partly Phase 5's.

**Contradicts:** the roadmap's Phase 7 row; D11 (`.File` is Phase 7's); `directive_gap`'s own
`::METHOD EXTERNAL` arm, which today carries the owner string **"Phase 7"**
(`rexx-exec/src/lib.rs:1020`).

## The decision not made that will block a task: object variables, `EXPOSE`, `self`, `super` and `FORWARD`

**CONFIRMED.** The spec decides `::CLASS`, `::METHOD`, `::ATTRIBUTE`, `::OPTIONS`, `::REQUIRES`,
`REPLY` and `GUARD`. It never mentions the object variable pool, `EXPOSE`, `self`, `super`, `FORWARD`,
or the `INIT`/`UNINIT` protocol -- and every one of them is Phase 5's in the tree's own tables and is
used by the file the phase must run.

Counts from `/bin/grep -a -i -c` over `interpreter/RexxClasses/CoreClasses.orx`, this session:
`self` 214, `expose` 112, `~new` 110, `forward` 35, `super` 19, `.methods` 19, `.context` 5.

`rexx-exec/src/lib.rs:1205-1210` gives `InstructionKind::Expose`, `Options`, `Message`, `Guard`,
`Reply` and `Forward` the owner string `"Phase 5"`. `expr_owner` (`:1219-1250`) gives
`ExprKind::QualifiedCall`, `ClassResolver`, `Message` and `List` the same. The spec addresses `Message`,
`ClassResolver` and `List` (dispatch, D34) and `Reply`/`Guard` (D32). `Expose` and `Forward` are
addressed nowhere.

This is not a long-tail item. Exit criterion 2 requires "one program per mixin in `CoreClasses.orx`".
`::CLASS 'Collection' MIXINCLASS Object Public` opens a run of methods whose bodies begin `expose
items` (`:176`), `expose indexes` (`:185`), `expose indexes items` (`:194`). A corpus program that
instantiates a mixin's user and calls a method reaches an object variable pool on its first clause.
`ArrayClass`' `sort` is `forward message('STABLESORT')` (`:1226`). `Monitor`'s `unknown` method is the
`forward`-to-destination idiom (`:1443-1453`).

A task author writing task 3 or 4 of this phase has to invent: where an object's variable pool lives,
how a method activation's scope selects it, what `self` and `super` resolve to, and what `FORWARD`'s
seven option words do. None of that is in the spec, and none of it is a detail -- `EXPOSE` is a
variable-pool design decision of the same weight as D16.

---

# Part 2 -- decision by decision

For each: what would have to be discovered for it to be wrong; whether that discovery is likely and
early or late; what reversing costs; and whether it contradicts D1-D24 or a Global Constraint.

## D25 -- the native/sourced line is discovered by running `CoreClasses.orx`

**[CHALLENGES A SETTLED CALL -- the mitigation, not the call.]**

**C1 (Critical, CONFIRMED). The four-answer wiring assertion is structurally incapable of seeing the
failure it exists to detect.**

D25 says: "the wiring of every class this phase does build is asserted against the oracle through
`~class`, `~superClass`, `~isA` and `~metaClass`", and the body text calls those four "the observable
shadow of the wiring". The hazard it is offered against is a wrong metaclass link, superclass edge or
behaviour build order.

Negative control, run this session under the oracle wrapper. Two classes differing only by an
`INHERIT`:

```rexx
say 'A:' .A~class .A~superClass .A~isA(.class) .A~metaClass
say 'B:' .B~class .B~superClass .B~isA(.class) .B~metaClass
say 'A supers:' .A~superClasses~makestring('LINE',',')
say 'B supers:' .B~superClasses~makestring('LINE',',')
say 'A resp GREET:' .A~new~hasMethod('GREET') ' B resp GREET:' .B~new~hasMethod('GREET')
::class M mixinclass Object
::method greet
  return 'hi'
::class A
::class B inherit M
```

Output, rc 0:

```
A: The Class class The Object class 1 The Class class
B: The Class class The Object class 1 The Class class
A supers: The Object class
B supers: The Object class,The M class
A resp GREET: 0  B resp GREET: 1
```

**The four answers are byte-identical whether or not the mixin edge exists.** Only `~superClasses` --
which the spec's set omits -- and actually sending the method separate them. The same holds on a real
primitive: `.array~superClass` is `The Object class` and `.array~superClasses` is `The Object class,The
OrderedCollection class`.

This is the failure mode the spec's own risk table calls out ("the flattened dictionary is built as a
chain walk / multiple inheritance resolves differently from the oracle") and that `CoreClasses.orx`
creates eleven of in its prolog (`.string~inherit(.Comparable)`, `.array~inherit(.OrderedCollection)`,
`.list~`, `.queue~`, `.identityTable~`, `.table~`, `.stringTable~`, `.directory~`, `.relation~`,
`.set~`, `.bag~`, `.stem~` on `MapCollection`, plus `.set~`/`.bag~` on `SetCollection` and `.message~`
on both notification mixins -- read at `CoreClasses.orx:88-119`).

A third, unrelated observation from the same run: `~isA` applied to a *class object* answers about the
metaclass chain, not the instance chain. `.array~isA(.OrderedCollection)` is **0** even though every
Array responds to `OrderedCollection`'s methods. So one of the four answers does not mean what a reader
of D25 would take it to mean.

**Fix that would hold:** add `~superClasses` and a per-class responds-to probe (`~hasMethod` over the
merged name set, or `~methods`) to the assertion. Both are cheap and both go red on the control above.

**Discovery: late.** A wrong inherit edge passes the stated assertion and surfaces when a corpus
program sends a mixin method -- which criterion 2 only reaches at gate time.

**Cost of reversal:** none structural; this is a check to strengthen, not a decision to undo. But the
cost of *not* strengthening it is that D25's stated mitigation for its own named hazard is decoration,
which is the defect class the ground rules put first.

**Contradicts:** nothing in D1-D24 directly. It is a check that would have done the same thing had the
claim been false.

**I9 (Important, PLAUSIBLE, precedent CONFIRMED). Criterion 5's `Setup.cpp` checklist should be
derived at build time, not transcribed.**

Criterion 5: "Every class `Setup.cpp` creates is either in the native layer or in the deferral table
with a reason", carried by `rexx-classes` "as a test over its own registry". A hand-written list of
class names inside the Rust tree is a mutable in-repo aggregate: if `Setup.cpp` gains a class, the test
stays green and the criterion is satisfied by a table that omits it.

The precedent is in the tree and is exactly this shape. `rexx-inventory/build.rs` derives both the
message table and the builtin-name table from the C++ at build time, with a structural guard that
panics when the table format changes, and its module doc says "The C++ tree is the source of truth.
Nothing here is hand-maintained". The enumeration is a one-line grep:
`/bin/grep -a -n -E "createInstance\(\)" interpreter/memory/Setup.cpp` answers 31 lines this session,
and the spec's quoted list matches them in order exactly (I checked all 31 names and their order --
`RexxClass` at `:235` through `MutexSemaphoreClass` at `:323`). Deriving it costs a build script arm;
transcribing it costs the criterion's meaning.

Also: nothing in `Setup.cpp` creates a class outside `createInstance()`. `/bin/grep -a -n -E
"classNew|new RexxClass|createClass|makeClass" interpreter/memory/Setup.cpp` answers nothing, so
"every class `Setup.cpp` creates" is a well-defined set and the grep is its definition.

**Verified correct in D25:** the quoted `createInstance()` order, the `BASEIMAGELOAD` chain
(`platform/unix/PlatformDefinitions.h:99` -> `Setup.cpp:1785`), and `TheRexxPackage` as the single
argument (`Setup.cpp:1795-1798`, `runProgram(..., (RexxObject **)&args, 1, result)`).

## D26 -- the three `.orx` files are executed, embedded at build time, sha256 recorded

**[CHALLENGES A SETTLED CALL -- the scope, not the embedding.]**

**C2 (Critical, CONFIRMED). Criterion 1's `StreamClasses.orx` clause requires Phase 7's natives.**
See Part 1. The embedding mechanism is fine; the *three-file scope* is what has to go.

**C3 (Critical, CONFIRMED). Criterion 1's `CoreClasses.orx` clause requires Phase 6's natives.**
`/bin/grep -a -n -i -E '^[[:space:]]*::(method|routine|attribute).*external'
interpreter/RexxClasses/CoreClasses.orx` answers five lines, all timer entry points:

```
1590:::METHOD !startTimer PRIVATE unguarded EXTERNAL 'LIBRARY REXX alarm_startTimer'
1618:::METHOD !stopTimer PRIVATE unguarded EXTERNAL 'LIBRARY REXX alarm_stopTimer'
1690:::METHOD !createTimer PRIVATE unguarded EXTERNAL 'LIBRARY REXX ticker_createTimer'
1691:::METHOD !waitTimer PRIVATE unguarded EXTERNAL 'LIBRARY REXX ticker_waitTimer'
1692:::METHOD !stopTimer PRIVATE unguarded EXTERNAL 'LIBRARY REXX ticker_stopTimer'
```

Under the eager-binding measurement above, `CoreClasses.orx` does not translate unless those five names
bind. They are timers on `Alarm` and `Ticker`, which are concurrency, which is Phase 6's -- and today
`directive_gap` calls `::METHOD EXTERNAL` Phase 7's. Either way, not Phase 5's.

The spec's own three-consequence paragraph asserts the opposite in spirit: it reads the file's
directives as `::METHOD`, `::CLASS` and `::ATTRIBUTE` "and nothing else", which is true of the keyword
and false of what installing them requires.

**I7 (Important, CONFIRMED). Criterion 1's `PlatformObjects.orx` clause is vacuous here and
unsatisfiable on Windows.**

`interpreter/platform/unix/PlatformObjects.orx` is 27 bytes, one line: `-- Nothing to do currently`.
Run this session: oracle rc 0 stdout empty; `rust/target/release/rexx-run` **rc 0, stdout empty, stderr
empty**. So one third of criterion 1 already passes at HEAD, before Phase 5 begins, and cannot fail.

`interpreter/platform/windows/PlatformObjects.orx` is two lines and its second is `call
'orexxole.cls'`. That file exists at `extensions/platform/windows/ole/orexxole.cls` -- the OLE support
the roadmap places in Phase 10. Global Constraint `2026-07-27-rust-rewrite.md:35` requires every phase
gate to run on all five platforms, Windows/MSVC among them. So criterion 1 is met for free on Linux and
cannot be met on Windows.

**This answers the spec's own open question** ("Whether `PlatformObjects.orx` is in scope ... nothing
has read it yet") with a measurement rather than leaving it open.

**M5 (Minor, CONFIRMED -- and it clears D26 of the defect I was sent to look for).** I was asked
whether a build script reading `/home/moritz/dev/repos/ooRexx/interpreter/RexxClasses/` works anywhere
but this machine. **It does not need to.** The `.orx` files are tracked in *this* repository:
`git ls-files interpreter/RexxClasses/` names `CoreClasses.orx` and `StreamClasses.orx`, and
`interpreter/platform/{unix,windows}/PlatformObjects.orx` are present too. `rexx-inventory/build.rs`
already reads the C++ tree by a *relative* path (`../../../interpreter/messages/rexxmsg.xml`), which
from `rust/crates/rexx-inventory/` resolves inside this repo. sha256 of all three files here is
identical to the same paths under `/home/moritz/dev/repos/ooRexx` (checked this session), so the
"upstream sha256" D26 wants is a within-repo constant, not a cross-tree one. **The defect is only the
wording:** the ground rules and the spec both say "the read-only oracle tree", which in the ground
rules means `/home/moritz/dev/repos/ooRexx`. Name the in-repo path and the `rexx-inventory` precedent
in D26, or a task author will write an absolute path and break four platforms.

**I8 (Important, CONFIRMED). D26 does not say how the embedded files are reached, and the prolog needs
three things D26 never names.** Read at `CoreClasses.orx:39-125`:

* `use arg rexxPackage`, then `rexxPackage~addClass('LOCALSERVER', .LocalServer)`,
  `rexxPackage~addPublicClass(name, class)`, `rexxPackage~objectname = "The REXX Package"`. So a
  **Package object with those three methods must exist natively before the file runs.** The spec says
  `PackageClass` is on the `Setup.cpp` checklist; it never says these three methods are bootstrap
  prerequisites.
* `publicClasses = .context~package~publicClasses` -- `.context` (a `RexxContext`), `~package`,
  `~publicClasses`, and then `do name over publicClasses` and `publicClasses[name]`. **`.context` is
  never mentioned in the spec.**
* `.String~defineClassMethod(name~upper, .methods[("string_cls_" || name)~upper])` -- **`.methods`, the
  package's unattached-method directory, is never mentioned in the spec**, and it is the mechanism by
  which the file-scope `::METHOD` block the spec *does* discuss gets attached.
* `call 'StreamClasses.orx' rexxPackage` at `:122` is a `CALL` on a **file name**. D26 says the files
  are embedded "rather than reading them from a search path". Nothing says how the executor's
  external-program call path resolves a file name to an embedded blob. A task author hits this in week
  one and has to invent it.

**Discovery for D26 overall: late** (see Part 1). **Cost of reversal:** `rexx-lib`'s embed set,
manifest, bootstrap entry, criteria 1 and 6, and the roadmap's Phase 7 row.

**Contradicts:** the roadmap Phase 7 row; D11; Global Constraint `:35`; `directive_gap`'s `Phase 7`
owner strings.

## D27 -- new crates `rexx-classes` and `rexx-lib`, one-way dependency, three-operation interface

**[CHALLENGES A SETTLED CALL -- the boundary claim, not the crates.]**

**C5 (Critical, CONFIRMED). The stated one-way dependency is refuted by `~new`.**

D27: "`rexx-classes` depends on `rexx-core` ... and on nothing in `rexx-exec`. ... A method *body* is
`rexx-exec`'s ... `rexx-classes` holds only its id. That is what keeps the dependency one-way."

The counterexample is the most common operation in the language. `ClassClass.cpp:1899`:

```cpp
obj->sendMessage(GlobalNames::INIT, initArgs, argCount, result);
```

with the same pattern at `:1846` (subclass creation) and `:1631`. `~new` is a class method -- it lives
on the class object, which is `rexx-classes`' -- and it **invokes a Rexx-level `INIT` body**, which
runs on the executor. That is `rexx-classes` calling `rexx-exec`, and the interface table has no such
row: `resolve`, `define` and `the registry` all point the other way.

`~new` is not exotic here. `/bin/grep -a -i -c '~new' interpreter/RexxClasses/CoreClasses.orx` answers
110 this session.

The same shape recurs wherever a native method drives Rexx code: the sort comparator path
(`ArrayClass.cpp:2903` reports against `GlobalNames::COMPARE`), and `UNKNOWN` dispatch, which
`CoreClasses.orx:1443` relies on for `Monitor`.

**What actually keeps the boundary one-way** is a third crate or an inverted-callback trait: either
`rexx-classes` takes an `&mut dyn Invoker` supplied by `rexx-exec`, or the native methods live in
`rexx-exec` and `rexx-classes` holds only the tables. The spec says "reaching past it is a spec
amendment, not a refactor", which is a good rule and means this has to be settled *before* the crate
exists rather than at the first `~new`.

**Discovery: early**, and that is the good news. The first task that makes `.Array~new` work hits it.
**Cost of reversal at that point:** the crate split and the interface table -- a spec amendment as
D27's own rule requires, and small if taken in week one, large if taken after `rexx-classes` has a
registry, a behaviour table and a metaclass graph in it.

The `::ATTRIBUTE` and native-method halves of the question I was sent with are **fine**: `::ATTRIBUTE`
generates a getter/setter pair, but `rexx-exec` can generate them and call `define`, and a native
method is an id in the same table, which is genuinely how `CPPCode::resolveExportedMethod` works. It is
`~new` -> `INIT` that breaks it.

**M1 (Minor, CONFIRMED).** The body text says "The interface is two operations and one table"; D27 says
"the three-operation interface stated above". Same table, two counts.

**M6 (Minor, PLAUSIBLE).** `rexx-lib` "owns ... the bootstrap that runs them". Running a program needs
`rexx-exec`. `rexx-lib`'s dependency direction is never stated, and it is the one crate that must
depend on both.

**Contradicts:** `2026-07-27-rust-rewrite.md:529` -- "if a later phase wants to reach past [a crate
boundary], that is a signal the boundary is wrong". That line is on D27's side, not against it.

## D28 -- message resolution is dynamic, with no per-call-site cache

**[CHALLENGES A SETTLED CALL -- the silence about D24, not the call.]**

**I1 (Important, CONFIRMED). D28 reverses half of D24 and the spec says it does not revisit D24.**

The spec: "**D24 already settled the shape and this phase does not revisit it**: dispatch is `resolve`
and `invoke`, two operations, not one fused `send`."

D24, verbatim (`2026-08-08-phase-4e-ir-design.md:495`): "Dispatch is implemented once as **`resolve`
and `invoke`**, not one fused `send`. **The IR's `Send` op caches the resolution** and calls the
invocation; the tree-walker and `eval.rs` call the same pair uncached."

D28 keeps D24's first sentence and reverses its second, while asserting nothing is being revisited.
It also negates the first of the four architectural reasons Phase 4e was built for
(`2026-08-08-phase-4e-ir-design.md:13`): "**It founds OO dispatch.** A call site in an instruction
stream is a stable, patchable slot, which is what makes per-call-site inline caches natural ... This is
Phase 5's dependency, not 4f's." The roadmap repeats it at `:482`: 4e "founds OO dispatch for Phase 5
by making a call site a patchable slot".

**This is a recording defect, not necessarily a wrong decision.** D22's own amendment already says the
patch schema "does not generalise to sends" and names what a real send cache needs (behaviour identity
plus invalidation on behaviour mutation), so D28 is consistent with D22. The fix is to record D28 as an
**amendment to D24**, with D24's second sentence struck, and to correct the roadmap's `:482` claim
about what 4e bought -- otherwise a Phase 6 or 4f reader meets D24 first and reads it as current.

**What would have to be discovered for D28 to be wrong.** A `dispatch` axis reading showing resolution
dominating. The spec is right that no such number exists (`perf-baseline.md`'s pre-Phase-5 section:
"`dispatch`, `alloc` and `heapshape` have no Rust number at all"), and right that a cache built now is
built against a guess.

**Discovery: late but harmless.** It shows up as a `dispatch` number at the end of this phase, which is
exactly when the spec says to revisit.

**I10 (Important, PLAUSIBLE). D29 does not leave D28 cheap to reverse unless a version stamp is added
now.** Under D29 the dictionary is *rebuilt in place* by `updateSubClasses`. A future send cache
guarded on behaviour identity would therefore not notice a redefinition at all -- the `BehaviourId` is
unchanged and the contents are not. The guard has to be a per-behaviour monotonic version stamp, bumped
in the cascade. Adding it now is one field and one increment inside the cascade that D29 builds anyway;
adding it later means finding every site that mutates a dictionary. This is the one item that makes
D28's "revisit later" genuinely cheap, and the spec does not mention it.

**Contradicts:** D24 as written (recorded above); nothing in the Global Constraints.

## D29 -- the method dictionary is flattened at definition time, cascaded to subclasses

**The strongest decision in the spec, and the tension it is accused of is not real.**

The factual claims check out. `rexx-core/src/behaviour.rs`, read this session: `BehaviourEntry` holds
`superclass: Option<BehaviourId>` -- a single edge, so it cannot express a mixin -- and `lookup` walks
the chain at lookup time with a visited set. `ClassClass.cpp`'s `updateSubClasses`,
`updateInstanceSubClasses` and `createInstanceBehaviour` are where the spec says, and
`createInstanceBehaviour`'s own comment confirms the merge discipline: "we process the superclasses in
reverse order, starting with Object, and overlay the information from each class on top of the
previous."

**On the D28-versus-D29 tension I was asked to attack: the spec's distinction is clean.** A call-site
cache and a flattened dictionary are guarded by different things. The dictionary's invalidation is a
*push* -- the cascade runs at the mutation site and every dependent is reachable from the class object's
subclass list, which D29 places on the class object. A call-site cache's invalidation is a *pull* and
needs the version stamp of I10. They do not conflict, and the spec is right that flattening is
semantics.

**"Does anything break when a class is reopened mid-run?" -- yes, and it is required, not exotic.**
`CoreClasses.orx:88-119` reopens eleven already-built primitive classes with `~inherit` and four more
with `~inheritInstanceMethods`, *after* the native layer has finished building them and after
`.environment` is populated. So D29's phrase "built at definition time" understates the requirement:
the cascade must be driven by a **run-time message send** to an existing class with existing
subclasses, not only by a `::CLASS` directive install. The Rust cost model is a rebuild of the
flattened dictionary for the target and every transitive subclass, on each of those fifteen clauses,
during every cold start -- which is a cost that lands directly on criterion 6's D2 measurement and that
nothing in the spec has budgeted.

**M7 (Minor, CONFIRMED).** "`BehaviourTable`'s chain walk is replaced, not extended" is nearly free.
`/bin/grep -rn -a "BehaviourTable"` over `rust/crates` finds it only in its own file and in
`rexx-core/src/lib.rs:21`'s re-export -- **no consumer anywhere**. `BehaviourId` and `MethodId` are
used, but `BehaviourTable` itself is dead. So D29's reversal cost is essentially zero, and the spec's
framing ("the part most likely to be got wrong, because the existing type invites the wrong shape") is
right about the hazard and overstates the sunk cost.

**Cost of being wrong:** if flattening turns out too slow at bootstrap, the fallback is a linearised
walk, which D29 correctly says is a *different algorithm with different answers*. So it is not a
performance knob and cannot be reversed for performance. That is a property worth stating in D29.

**Discovery: early.** The first mixin diamond in `phase-5.txt`, which the risk table already asks for
"from the first commit that defines a class". Good.

**Contradicts:** nothing. D1's enumerable-root-set criterion is respected by keeping classes in the
arena, which the spec states.

## D30 -- `::REQUIRES` lands last in this phase

**The stated reason survives on one half and not on the other.**

**CONFIRMED, in D30's favour on `CallSite`.** `ir.rs`'s `CallSite` doc says, verbatim: "**The property
the table needs is that one, and not 'the map is written once'**, which is a stronger thing that
happens to be true today and would stop being the reason **if a `::REQUIRES` or an external-file call
ever installed a routine mid-run**." So `::REQUIRES` really is the named breaker of the `CallSite`
argument, and the reviewer's hypothesis -- that D28's "sends do not use `CallSite`" makes the reason
stale -- is **wrong**. `CallSite` is the *classic-call* cache; `::REQUIRES` installs *routines*. The two
meet, and D28 is irrelevant to it. The spec's sentence "it is the one thing that breaks the two
append-only arguments the `CallSite` table and the plan cache are built on" is accurate about
`CallSite`.

**I2b (Important, CONFIRMED). The plan-cache half of the reason is not supported.** `BodyKey` is
`{ program: ProgramId, directive: Option<usize> }` and `ProgramId(usize)` is an index into
`Interp::programs: Vec<Rc<Program>>`, handed out as `ProgramId(self.programs.len())` at
`lib.rs:2389-2390`. Loading a second program appends a new id and therefore a *fresh* key space; no
existing plan is invalidated and no key is reused. The plan cache's doc says its concern is that a key
"cannot be reused by a different program", which appending satisfies. **The plan cache is not broken by
`::REQUIRES`** on the evidence I can find. The survey's Q8 makes the same pairing, so the error is
inherited rather than introduced -- but D30 states it as settled fact.

**What D30 misses, and it is sharper than what it says.** `Interp::routines` is a **flat global
`HashMap<Box<[u8]>, InstalledRoutine>`** (`lib.rs:1647`), and `install_directives` raises the oracle's
99.903 on *any* duplicate insert (`:2489-2497`). Two independently-authored packages each exporting a
routine named, say, `MAIN` is legal on the oracle and would give a spurious translation error here. So
`::REQUIRES` breaks not "append-only" but **"one global routine namespace"**, and the fix is Q8's
option (a) -- a per-package resolution namespace. That is a bigger change than D30's wording implies,
and it is worth naming so the last task of the phase is not sized as a cache-invalidation job.

**Discovery: late by design**, which is D30's point and is right. **Cost of reversal:** if `::REQUIRES`
slips out of the phase, criterion 2 loses one of its retired over-refusals (see I3) and the roadmap's
Phase 5 row -- "`::class`/`::method`/`::routine`/`::requires` work" -- goes unmet.

**Contradicts:** nothing. D16 is untouched.

## D31 -- `::OPTIONS` and the `OPTIONS` instruction leave this phase

**I2 (Important, CONFIRMED). D31 creates an orphan the tree's own tests will reject.**

Nothing in the roadmap's phase table is a package-settings phase. Phase 6 is concurrency, 7 is streams
and platform, 8 is native API, 9 is core conformance, 10 is RXAPI and extensions. D31 says the unit
"can land before, during or after this phase" and names no owner.

That is not merely untidy -- it is **checked**. `rexx-exec/tests/owners.rs:393`:

```rust
pub(crate) const SPLIT_TABLE_PHASES: &[&str] = &["4b", "4c", "Phase 5", "Phase 7"];
```

asserted at `:409`, and the identical closed set appears in `bif_assertions.rs:823` and
`keyword_assertions.rs:830`. `instruction_owner` gives `InstructionKind::Options` the string
`"Phase 5"` (`lib.rs:1206-1210`), `directive_gap` gives `DirectiveKind::Options(_)` the same
(`lib.rs:1038`), and `phase-4-exclusions.txt` records both rows against Phase 5 in two separate
sections (`:453` for the directive, `:2087-2103` for the instruction).

So executing D31 means either changing four in-tree sites to a phase name that does not exist in
`SPLIT_TABLE_PHASES` -- turning three assertions red -- or leaving them saying "Phase 5" while Phase 5
does not own them, which is a false statement in tables whose only job is to be true. `4a`'s exit
criterion 5 makes the owner set a **plan amendment**, not a task edit
(`2026-07-30-phase-4a-executor-design.md:453`).

**Fix:** either give the package-settings unit a roadmap row and add its name to `SPLIT_TABLE_PHASES`
as a plan amendment, or keep the two rows in Phase 5 and land them first. D31's stated benefit --
"deletes two over-refusals early" -- is available under the second option too.

**M3 (Minor, CONFIRMED). D31's supporting measurement is misstated.** The spec: "the bare `OPTIONS`
instruction runs silently at rc 0 on the oracle while this crate refuses it at 120". Measured this
session, a genuinely bare `options` (no expression):

```
oracle   rc 221, stderr:  1 *-* options / Error 35 ... / Error 35.913: Missing expression following OPTIONS keyword.
this crate rc 120, stderr: rexx-exec: 35.913: Missing expression following OPTIONS keyword.
```

The over-refusal is on `OPTIONS <expression>`: `options 'anything at all'` gives oracle rc 0, stdout
`ok`; this crate rc 120 `OPTIONS is not implemented (Phase 5)`. The term "bare" comes from
`phase-4-exclusions.txt:2087`, which measured `options 'nothing'` and called it bare, so the spec
inherited it -- but a task author who writes the corpus program the spec describes will get 35.913 from
both sides and conclude there is no gap.

**Discovery: immediate** -- the first task that tries to move the rows finds no owner.
**Cost of reversal:** two owner strings, one exclusions section, and the plan amendment.

## D32 -- `REPLY` and `GUARD` get their translation-time legality check only

**C3b (Critical, CONFIRMED). `CoreClasses.orx` uses both, and the spec answered Q9 without the evidence
Q9 named.**

The survey's Q9 evidence bullet: "whether `CoreClasses.orx` itself uses either -- **a grep of the file,
not an argument** -- and what the oracle does for `REPLY` in a method under no concurrency at all."

The grep, run this session over `interpreter/RexxClasses/CoreClasses.orx`:

```
1554: guard off        1555: reply        1560: guard on
1606: guard on  when timerStarted
1662: guard off        1663: reply        1670: guard on      1676: guard off
```

So `REPLY` appears inside `Alarm`'s and `Ticker`'s methods, and `GUARD` appears in `ON`, `OFF` and
`ON WHEN <expr>` forms. The spec's D32 rationale is: "the whole observable behaviour of both
instructions *outside a method* is a translation-time refusal that is already specified and already
testable, and it is this phase's. The run-time half is concurrency, which is Phase 6's".

**Three consequences the spec does not carry.**

1. The legality check must **accept** `REPLY` and `GUARD` inside a method, or `CoreClasses.orx` does not
   translate and criterion 1 fails at the directive-install step. D32 specifies only the refusal.
2. `GUARD ON WHEN <expr>` has a parse and a scope requirement (the expression reads object variables)
   that exists at translation time whether or not the run-time half lands.
3. What happens when a Phase 5 corpus program does `.Alarm~new(...)` and reaches `reply` at run time is
   **specified nowhere**. The survey's option (a) said "translates and then fails at run time in
   whatever way Phase 6 later replaces"; the spec chose (a) and dropped that clause. A loud refusal at
   run time is a defensible answer and is the project's own convention -- but it has to be written
   down, because the alternative is a `reply` that silently does nothing, which is the exact shape the
   failing-loudly rule exists to prevent.

**Verified correct in D32:** the transcripts. Run this session, `reply` alone in a file: oracle rc
**157**, `Error 99.919: REPLY can only be issued in an object method invocation.`; `guard on` alone:
rc **157**, `Error 99.911: GUARD can only be issued in an object method invocation.` This crate gives
rc 120 with the loud message for both.

**Discovery: early** -- the first attempt to translate `CoreClasses.orx` past `::CLASS` reaches
`Alarm`'s directives. **Cost of reversal:** small, if D32 gains the "inside a method" half now.

**Contradicts:** nothing in D1-D24. D3 places concurrency in Phase 6 and D32 respects that.

## D33 -- `.environment` and `.local` are real directory objects from the first task

**No finding. The strongest-supported decision in the spec.**

The measured transcripts reproduce exactly. Run this session, `say value('.LOCAL')` then
`say value('.ARRAY')`:

```
oracle    rc 0, stdout: The Local Directory / The Array class
this crate rc 0, stdout: .LOCAL / .ARRAY
```

The asymmetry the spec relies on -- silent on the `VALUE` path, loud on the expression path -- is real,
and `phase-4-exclusions.txt` does record it with no owner. The "one table, not two" argument is
supported by `Setup.cpp:1780-1783`, which installs the `LOCAL` method on `TheEnvironment` immediately
before resolving `BASEIMAGELOAD`, so a directory object that answers `~` is needed at bootstrap.

**What would have to be discovered for it to be wrong:** that a directory object is too slow to stand
up before the first task can do anything else. Nothing suggests that. **Discovery: early. Reversal
cost: low** -- the alternative it rejects (a name table now) is strictly less code, so backing into it
is always available.

**Contradicts:** nothing. D12's Phase 5 half explicitly names `.local`/`.environment` lookup as an
interception point, which D33 is a prerequisite for -- see the open-question note below.

## D34 -- `ExprKind::List` is a real Array from the first commit that makes `~` work

**No finding.** The discriminating transcript reproduces: `say (1,)~size` prints **2** and
`say (1,,)~size` prints **3**, oracle rc 0, run this session. `expr_owner` gives `ExprKind::List` the
owner `"Phase 5"` today, so no earlier phase shipped an approximation and there is nothing to delete.
**Discovery: early. Reversal cost: nil** -- there is no competing implementation.

## D35 -- the performance guard is the classic axes only, two-build sitting per task

**I5 (Important, CONFIRMED for the pointer, PLAUSIBLE for the consequence). Criterion 4's floor is not
where the spec says, and the guard's instrument is unnamed.**

The spec: "`bench-baselines/README.md` states the floor to read an `across_builds` movement against."
I read that file in full this session. **It contains no floor and no threshold.** It states what an
`across_builds` row is and quotes Task 8's `+/-0.74%` bound. The threshold sentence -- "A sub-1%
movement across builds is not a result" -- is in `perf-baseline.md:1032-1038`. A task author following
the spec's pointer arrives at a document with no number in it.

The consequence is worse than the sourcing. Criterion 4's guard is an `across_builds` comparison
(pinned binary against head binary), and Phase 4e's own gate text records that (a) both instruments are
required for any claim, (b) a cycle ratio "is comparable within a build and not across builds", with the
tree-walker's IPC ranging 3.8% on `varlookup` and 6.5% on `emptyloop` between builds, and (c)
`emptyloop` carries a between-build instruction-count sensitivity of 15 instructions per pass -- 11% of
the gap -- from code added to a function it executes. A 1% floor evaluated on cycles across two builds
is therefore not evaluable, and the spec does not say which instrument criterion 4 uses.

**Fix:** state the instrument (instructions, where the 1% floor was derived), and say what to do when
the two instruments disagree -- 4e's own rule is that a disagreement is "a tripwire that says look at
the denominator", not a veto.

**M4 (Minor, CONFIRMED). `emptyloop` is dropped from the guard with no reason given.** `AXES` in
`rexx-bench/src/bin/rexx-bench-suite.rs:145-186` gives `emptyloop` `Role::Loop`, the same role as the
five axes D35 guards. D35 names "`arith`, `compound`, `strings`, `varlookup`, `alloc4c` and `rexxcps`"
as "the classic axes", which reads as the complete set and is not. There may be a good reason --
`emptyloop` is the axis 4e found least trustworthy across builds -- but the spec should say so rather
than let the omission read as an oversight.

**Verified correct in D35:** `bench-baselines/pinned/rexx-run-pre-phase-5` and
`bench-baselines/pre-phase-5-arms.tsv` both exist, so the quoted `rexx-arms` invocation resolves.
`alloc`, `dispatch` and `heapshape` are `Role::Blocked` in `AXES`, so the re-roling instruction is
against a real table with a real assertion.

**Discovery: mid-phase**, at the first task that touches an execution path. **Reversal cost:** low; a
guard is a rule, not code.

**Contradicts:** nothing directly. Global Constraint `:39`'s parity gate is scoped to the roadmap's
Phase 4 row for the classic axes, and the roadmap's `:474` already reserves the debt route for
`dispatch`, `alloc.rex` and `startup` -- which is exactly what D35's "new measurements, not
regressions" says, so D35 is consistent with it.

## D36 -- the gate is a `phase-5.txt` corpus subset on both engines, plus in-crate `>M>`/`>N>` assertions

**I3 (Important, CONFIRMED). Criterion 2 contradicts D31.** Criterion 2 requires the subset to contain
"the four `directive_gap` over-refusals this phase retires". `directive_gap`'s Phase 5 arms are exactly
`::REQUIRES`, `::OPTIONS`, `::CLASS naming another class` and `::ANNOTATE naming a target`
(`lib.rs:1033`, `:1038`, `:1043-1048`, `:1053-1056`). **D31 removes `::OPTIONS` from the phase**, so
this phase retires three of them, not four. One of the two statements has to change.

**I6 (Important, CONFIRMED). `::CONSTANT` is unowned, and criterion 1 needs it.** The spec's directive
section names `::CLASS`, `::METHOD` and `::ATTRIBUTE` "in that order, because that is the order
`CoreClasses.orx` needs them". Directive census run this session:

```
CoreClasses.orx    ::method 303  ::class 32  ::attribute 12
StreamClasses.orx  ::method 139  ::class  7  ::attribute  5  ::constant 2
```

`StreamClasses.orx:548-549`:

```
::constant separator (.File~getSeparator)
::constant pathSeparator (.File~getPathSeparator)
```

Both evaluate a class method at install time, and both target methods are themselves `EXTERNAL`
(`:546-547`). `::CONSTANT` is silently accepted today -- `directive_gap` returns `None` for
`DirectiveKind::Constant(_)` (`lib.rs:1061`) -- so a program with one runs at rc 0 on both sides
(measured: `say 'main'` / `::class c` / `::constant k 5`, oracle rc 0 stdout `main`, this crate
identical). The moment `~` works, `c~k` must answer `5`, and nothing owns making it do so.

**M2 (Minor, CONFIRMED). Criterion 2's "one program per mixin" has no defined set, and the spec's own
enumeration mislabels part of it.** The spec's mixin list opens with `Collection`,
`OrderedCollection`, `MapCollection`, `SetCollection`, `Comparable`, "`Comparator` and its six
subclasses", `Orderable`, `SupplierMixin`, `ManyItemMixin`, `SetMixin`, `BagMixin`,
`MessageNotification`, `AlarmNotification`, `Singleton`. Read at the file
(`/bin/grep -a -n -i -E "^[[:space:]]*::class"`):

* `SupplierMixin` (`:172`), `ManyItemMixin` (`:218`), `SetMixin` (`:411`) and `BagMixin` (`:557`) carry
  **no `MIXINCLASS` keyword at all** -- they are plain `::class` directives, wired in by the prolog's
  `~inheritInstanceMethods` calls, which is a *different mechanism* from `~inherit` and which the spec
  never mentions.
* `Comparator`'s subclasses are `DescendingComparator`, `CaselessComparator`,
  `CaselessDescendingComparator`, `ColumnComparator`, `InvertingComparator`, `NumericComparator` and
  `CaselessColumnComparator` -- one more than "six".
* `Singleton` is `mixinclass class`, a mixin on the metaclass, not on `Object`.

So "one program per mixin" quantifies over a set that is either "carries `MIXINCLASS`" (which excludes
four names the spec calls mixins) or "the spec's prose list" (which is in-repo prose, the thing that
rots). Name it as a committed list the way `phase-4a.txt` is named, or derive it.

**I4 (Important, CONFIRMED). Criterion 6 names a tool that is not installed and cannot be installed.**
`command -v hyperfine` answers nothing on this machine this session; `~/.cargo/bin` does not contain
it. `perf-baseline.md:97` and `:157` both say so explicitly -- "`hyperfine` is not installed in this
environment and cannot be installed (no network)" -- and `:159-163` recommends
`rexx-bench/src/bin/rexx-time.rs` as the permanent cold-start tool, on the ground that a five-platform
gate should not depend on a separately-installed binary. `perf-baseline.md:1064` nonetheless says
"Measure D2 with hyperfine against `build/bin/rexx`, as D2 says", and the spec's criterion 6 follows
that copy.

There is a second-order problem. D2's C++ figure of 5.1 ms is a *hyperfine* number
(`perf-baseline.md:1063` says so, distinguishing it from this suite's 5.823 ms). So measuring the Rust
side with `rexx-time` and comparing against 5.1 ms crosses instruments on a 50 ms threshold. **Both
sides must be re-measured on one instrument**, and criterion 6 should name `rexx-time`.

**Verified correct in D36:** the corpus mechanism is real and guarded --
`corpus.rs::the_differential_reads_every_phase_subset_file` asserts `SUBSET_FILES` against the files on
disk, and `corpus/` already holds `phase-4a.txt`, `phase-4b.txt`, `phase-4c.txt`, so `phase-5.txt`
slots in. The `>M>`/`>N>` premise is real: `tests/support/mod.rs:141-142` has `PREFIX_OFFSET = 7`,
`PREFIX_LENGTH = 3`, and a documented space-run collapse, so the differential genuinely cannot see
indent. Criterion 3's two-instrument answer is sound.

**Criterion 1's `PlatformObjects.orx` clause is already green at HEAD** -- see I7. That is a criterion
component that cannot fail.

**Discovery: at gate time**, which is the worst place. **Reversal cost:** criteria text only, but a
criterion discovered unmeetable at the gate is what stalls a phase.

---

# Part 3 -- decisions the spec should have made and did not

Each of these is a question a task author hits in week one with no answer in the document. In rough
order of how early it bites.

**1. Object variables, `EXPOSE`, `self`, `super`.** See Part 1. This is a D16-weight decision and the
spec has nothing.

**2. `FORWARD`.** Owned by `"Phase 5"` at `lib.rs:1210`, used 35 times in `CoreClasses.orx` and in
`StreamClasses.orx` (`forward message 'ARRAYIN'`, `forward class (super)`), and never mentioned.

**3. `::CONSTANT`.** See I6.

**4. `::METHOD EXTERNAL` / `::ROUTINE EXTERNAL` / `::ATTRIBUTE EXTERNAL`.** Currently "Phase 7". The
bootstrap needs them at Phase 5, eagerly. Either the phase boundary moves or the binding becomes lazy
-- and lazy binding is a *divergence*, because the oracle refuses at install time (rc 166, measured).
Somebody has to choose.

**5. How `call 'StreamClasses.orx' rexxPackage` resolves against an embedded blob.** See I8.

**6. `.methods` and `.context`.** Both are prerequisites of `CoreClasses.orx`'s prolog. Neither appears
in the spec. `.methods` is how the file-scope `::METHOD` block the spec *does* discuss gets attached.

**7. The Package object's bootstrap surface** -- `addClass`, `addPublicClass`, `objectname=`,
`publicClasses` -- which must exist natively before line 47 of the file runs.

**8. The `INIT` / `UNINIT` protocol.** `~new` sends `INIT` (`ClassClass.cpp:1899`). `UNINIT` is a
finalizer, which Phase 1 Task 1.6 built machinery for and which `StreamClasses.orx:180` declares as an
external method. Nothing in the spec says whether Phase 5 wires either.

**9. A per-behaviour version stamp on the flattened dictionary.** See I10. One field now, a sweep
later.

**10. `~superClasses` and a responds-to probe in the wiring assertion.** See C1.

**11. Which instrument criterion 4 is judged on, and what to do when the two disagree.** See I5.

**12. The security manager's interception shape (D12).** The spec lists this as an open question and
says "the plan must [fix it], before the first dispatch call site is written". That is correct, and I
flag only that D12 is a **decision block in the roadmap** with Phase 5 named as its owner
(`2026-07-27-rust-rewrite.md:330-335`), and the roadmap's Phase 5 exit row lists "security manager
interception points in place (D12)" as a gate item -- which **the spec's exit criteria do not
contain**. So D12 has silently dropped out of the gate as well as out of the decisions. That is a
CONFIRMED gap between the roadmap's Phase 5 row and this spec's criteria.

**13. Order dependence of the `createInstance()` sequence.** The spec's own open question. I note only
that it is answerable cheaply: copy the order, and the plan says whether it had to.

---

# Part 4 -- what I checked and found sound

Recorded so the report is not read as uniformly negative, and so a later reader does not re-run these.

* The `createInstance()` list and its order: 31 calls, matching the spec's quoted list name for name and
  position for position.
* `BASEIMAGELOAD` -> `Setup.cpp:1785` -> `runProgram` with `TheRexxPackage` as one argument.
* The three `ClassClass.cpp` citations behind D29.
* `BehaviourTable`'s single-edge superclass and lookup-time chain walk.
* `CallSite`'s doc naming `::REQUIRES` as its breaker (D30's `CallSite` half).
* The `>M>`/`>N>` normalisation premise (`PREFIX_OFFSET`).
* D33's `value('.LOCAL')` / `value('.ARRAY')` transcripts.
* D34's `(1,)~size` = 2 and `(1,,)~size` = 3.
* D32's 99.919 / 99.911 / rc 157 transcripts.
* The spec's "verified in this session" claim about `CoreClasses.orx`: reproduced exactly -- rc 120,
  stderr `rexx-exec: ::CLASS naming another class is not implemented (Phase 5)`.
* `bench-baselines/pinned/rexx-run-pre-phase-5` and `pre-phase-5-arms.tsv` exist; the `AXES` roles are
  as D35 describes.
* D26's embedding is buildable off this machine -- the `.orx` files are git-tracked in this repository
  and `rexx-inventory/build.rs` already reads the C++ tree relatively. The suspected CI-breaking defect
  is **not present**; only the wording invites it.
