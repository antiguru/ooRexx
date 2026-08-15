# Review P-D: every verification, every "Done when", every instrument

**Subject:** `docs/superpowers/plans/2026-08-15-phase-5a-native-layer.md`
**Lens:** the checks. Not whether the plan builds the right thing -- whether the things it proposes
to check with could tell.
**Method:** every oracle run wrapped exactly as the ground rules require, three descriptors read
separately, from a fresh empty directory with absolute paths. Rust runs use
`rust/target/release/rexx-run` as built. Every count below comes from `/bin/grep -a`.
**Repo state read:** `e72a2427f`.

---

## Summary

17 findings: 2 CRITICAL, 7 MAJOR, 5 MEDIUM, 3 MINOR.

The single worst one is F1: **the five-answer wiring assertion the plan added specifically to stop a
mixin-blind build is itself blind to the prologue's mixin block.** `inheritInstanceMethods` does not
appear in `~superClasses`, and it is what five of the prologue's ~40 executable clauses do. That
assertion is also the entirety of Task 9's substantive check and one of Task 10's subset items, so
one blind spot propagates through three tasks.

**What reproduced correctly.** Before the findings, the plan's own measured figures. I re-ran every
number it quotes and every one came out as stated: Task 5's `::CONSTANT` pair (oracle rc 159 / empty
stdout and this crate rc 0 / `prolog`; and rc 157 with 99.906 for the no-`::CLASS` form), Task 6's
eager-bind shape (rc 166, empty stdout, `Error 90.998`), Task 7's three `hasMethod` answers (0, 0, 1)
and the `~defineClassMethod` refusal (97.1, rc 159), and Task 4's `.LOCAL` figures (oracle prints
`The Local Directory` on both routes at rc 0; this crate exits 120 with
`rexx-exec: an environment symbol is not implemented`). Its two claims about the tree also hold:
`coverage.rs:151`'s `assert_program_has_only_routine_directives` rejects any non-`::ROUTINE`
directive, and `trace_oracle.rs:661` really does carry a hardcoded
`["phase-4a.txt", "phase-4b.txt", "phase-4c.txt"]` with no directory-read assertion, unlike
`corpus.rs`, `coverage.rs`, `ir_dual.rs` and `collect_stress.rs`, which each pin `SUBSET_FILES`
against a read of `rust/corpus/`. The quoted `rexx-arms` invocation's flags all exist
(`--build`, `--axis`, `--rounds`, `--task`, `--commit`, `--baseline` are all parsed at
`crates/rexx-bench/src/bin/rexx-arms.rs:101-120`). This plan's factual half is in good shape. Its
instruments are not.

---

## Part 1: the three questions, per task

### Task 1 -- the behaviour table

**1. A defect it trips on.** `inherit` implemented as a copy rather than in place: row 4's probe
(`~inherit` with an instance already created) then reports the old instance not seeing the method,
against the oracle's `1`. I measured the oracle's row-3 and row-4 answers and the plan has them
right -- with a pre-existing instance, `~define` leaves it at `hasMethod` 0 and a later `~new` at 1;
`~inherit` puts both at 1.

**2. A defect it does not trip on.** The monotonic version. Task 1 requires "a per-behaviour
monotonic version, bumped by every cascade", and **not one row of the table, and nothing in
Verification or Done when, observes it.** A version field initialised to 0 and never incremented
passes every row. The plan's own justification for adding it now ("adding it afterwards means
finding every site that mutates a dictionary") is exactly the reason a dead one is expensive: the
sites will have been written against a field nothing checks.

**3. Reportable done without the work?** Yes -- see F3. Under the representation the tree actually
has, "copies the behaviour first, then replaces the entry" and "mutates in place" are the same
observable, so an implementer who does the cheap thing passes rows 3 and 4 and the negative control
alike.

### Task 2 -- class objects, registry, metaclass graph

**1. Trips on.** A subclass edge wired to the wrong parent: `.Array~superClass~id` would stop being
`Object`, and `~superClasses` would stop matching.

**2. Does not trip on.** F1 and F6, below. Also the merge *order* under multiple inheritance:
measured, `.D~superClasses` for `::CLASS D INHERIT A B` renders
`The Object class,The A class,The B class` while the winning method is `A`'s. Reverse the merge so
`B` wins and all five answers are byte-identical.

**3. Reportable done without the work?** Yes. `~class` and `~metaClass` answer `Class` and
`~superClass` answers `Object` for every one of `.Bag .Set .Relation .Array .String .Directory
.Supplier .Message` (measured). Only `~superClasses` carries information for most of the set, and
the deferral-table clause is satisfied by deferring everything the grep names.

### Task 3 -- the native message send and the security seam

**1. Trips on.** A send that resolves the wrong method for a primitive receiver -- the
oracle-differential half is real and would catch it.

**2. Does not trip on.** A second dispatch path. See F4: counting call sites of the chokepoint
cannot see a path that does not call it, which is the only kind of second path that matters.

**3. Reportable done without the work?** The differential half, no. The security half, yes
entirely -- the plan states the assertion's existence as the deliverable and never states its
mechanism.

### Task 4 -- `.environment`, `.local`, `.context`, `.methods`

**1. Trips on.** `.LOCAL` resolving to the string `.LOCAL` instead of the Local Directory. Measured:
oracle `The Local Directory` on both `say .LOCAL` and `say value('.LOCAL')`, rc 0; this crate exits
120 on the first.

**2. Does not trip on.** The unresolved-`.NAME` fallback, which is what 8 of the 9 data rows in
`corpus/bif-exempt.txt` carrying `UNATTRIBUTED:an environment symbol` actually exercise. Measured:
`say .funnyconst` prints `.FUNNYCONST` and `say value('.a')` prints `.A`, both rc 0. Neither shape is
in Task 4's verification list. So the Verification is blind to the defect that keeps the Done when's
own rows failing.

**3. Reportable done without the work?** Half of the Done when is already true at HEAD -- F7.

### Task 5 -- the directives that install

**1. Trips on.** Lazy or absent evaluation of `::CONSTANT (expr)`. Verified on both sides; see F10's
first paragraph.

**2. Does not trip on.** Evaluation *order*. F10.

**3. Reportable done without the work?** For `::METHOD` and `::ATTRIBUTE`, partly: "install" with
bodies never entered is observable only through `~hasMethod`, and the plan does not say what the
per-directive differential program is.

### Task 6 -- the native entry-point registry

**1. Trips on.** The eager-bind failure shape -- reproduced exactly (rc 166, empty stdout,
`Error 90.998: Unable to find external method "no_such_entry_point"`; this crate exits 120 with
`::METHOD EXTERNAL is not implemented (Phase 7)`).

**2. Does not trip on.** Nothing, because the check cannot be written -- F5.

**3. Reportable done without the work?** The registry could contain every name and resolve every one
to the same stub, and the eager-bind differential would still pass.

### Task 7 -- the Package object and the setup methods

**1. Trips on.** Leaving `DEFINECLASSMETHOD` on `.String` after the bootstrap.

**2. Does not trip on.** Never providing it. F12.

**3. Reportable done without the work?** Yes for the removal half, and the Done when's first clause
("the prologue reaches its `~inherit` block") names no instrument at all.

### Task 8 -- the collection sends

**1. Trips on.** `publicClasses[name]` returning the wrong element, or `~put` not storing.

**2. Does not trip on.** That `DO OVER` an object *dispatches*. F13.

**3. Reportable done without the work?** The owners.rs clause, yes -- it has no subject.

### Task 9 -- `rexx-lib` and the bootstrap

**1. Trips on.** A bootstrap that raises: rc would not be 0 and stderr would not be empty.

**2. Does not trip on.** Almost everything else the prologue does. F2.

**3. Reportable done without the work?** Yes, and cleanly. F2.

### Task 10 -- the 5a gate

**1. Trips on.** A subset file added to `rust/corpus/` and not wired into `corpus.rs`,
`coverage.rs`, `ir_dual.rs` or `collect_stress.rs` -- each of those four asserts `SUBSET_FILES`
against a directory read, so they redden. The plan's claim here is correct.

**2. Does not trip on.** `trace_oracle.rs`, which the plan itself names -- correctly. Plus the
new-crate lint gap (F16).

**3. Reportable done without the work?** The subset list is the risk; see F14 and Part 2.

---

## Part 2: the eight instruments named in the brief

### F1 -- Task 2's five-answer wiring assertion is still blind, and blind to the prologue's own mixin block. CRITICAL, CONFIRMED

The plan's reasoning for adding `~superClasses` is correct as far as it goes: `.DateTime~superClass~id`
and `.Array~superClass~id` are both `Object`, so the other four are blind to mixin edges. But
`~superClasses` reports only what `~inherit` did. The prologue does its mixin work two different
ways, and only one of them shows up.

Measured, one oracle run:

```
Bag      | class: Class | super: Object | meta: Class | supers: The Object class,The MapCollection class,The SetCollection class
Set      | class: Class | super: Object | meta: Class | supers: The Object class,The MapCollection class,The SetCollection class
Supplier | class: Class | super: Object | meta: Class | supers: The Object class
```

`.Set` and `.Bag` are **byte-identical on all five answers**. Yet `CoreClasses.orx:80-87` gives them
different method dictionaries: `.set~inheritInstanceMethods(.SetMixin)` versus
`.bag~inheritInstanceMethods(.ManyItemMixin)` followed by `.bag~inheritInstanceMethods(.BagMixin)`.
And `.Supplier`, whose five answers say it inherits nothing but `Object`, has
`.supplier~inheritInstanceMethods(.SupplierMixin)` applied to it.

That those calls did something is measurable, just not by the five:

```
supplier instance hasMethod ALLITEMS: 1
bag hasMethod UNION: 1
bag class superClasses: The Object class,The MapCollection class,The SetCollection class
```

`ALLITEMS` comes from `::class "SupplierMixin"` at `CoreClasses.orx:172`; `UNION` from
`::class "ManyItemMixin"` at `:218`. Neither mixin appears in `~superClasses`, because
`inheritInstanceMethods` is a "phony inherit" -- the file's own comment at `:76-78` says so.

**So a build that implements `inheritInstanceMethods` as a no-op passes Task 2 for every class,
passes Task 9's post-bootstrap check, and passes Task 10's "five wiring answers per class in the
native set".** That is five of the prologue's executable clauses unwitnessed by the check written to
witness the prologue.

**What would have come out differently.** If the five were sufficient, `.Set` and `.Bag` would
differ somewhere in that table, or `.Supplier` would name `SupplierMixin`. Neither does. The
discriminating observable exists and is cheap -- `~hasMethod` on an *instance* for a name each mixin
uniquely defines -- and is not in the plan.

A second, independent wiring error all five miss: merge order. Measured,
`::CLASS D INHERIT A B` where `A` and `B` are mixins of a common `Base` and all three define `m` --
`.D~superClasses` is `The Object class,The A class,The B class` and `.D~new~m` is `A`. Swap the merge
so `B` wins and the five-tuple does not move. This is Task 1's rows 1 and 2, and it is exactly the
thing the plan says the phase's per-class check covers.

**Fix, addressed to the plan's author.** The per-class assertion needs a sixth answer that reads the
merged dictionary, not the edge list. `~hasMethod` on a constructed instance for a name owned by
each mixin is the cheapest one that discriminates, and it is the only one that sees
`inheritInstanceMethods` at all.

### F2 -- Task 9's four checks are satisfied by a bootstrap that did nothing. CRITICAL, CONFIRMED-structural

Task 9 offers exit status, empty stdout, empty stderr, and "the post-bootstrap state answers Task 2's
five wiring questions **for every class the prologue installs**".

The first three are the answer to "the prologue ran and installed everything" and also the answer to
"the prologue's loops iterated zero times". `do name over publicClasses` over an empty directory
prints nothing and exits 0. The fourth is quantified over a set the implementation under test
produces: if nothing is installed, the set is empty and the assertion is vacuous. It is the
empty-set failure shape this project has already recorded, arriving through the quantifier rather
than through a no-op harness.

Concretely, here is post-bootstrap state that all four checks are blind to, every line measured on
the oracle in one run:

```
env objectname: The Environment Directory     <- CoreClasses.orx:55, .environment~objectname=
String NL: 0A                                 <- the :70-74 defineClassMethod loop, 16 names
String TAB: 09
env has ARRAY: 1                              <- the :63-67 .environment~put loop
env has LOCALSERVER: 0                        <- addClass vs addPublicClass, the whole point of :47-52
```

`LOCALSERVER` being **absent** is the load-bearing one: it is the difference between `addClass` and
`addPublicClass`, and Task 4's own prose leans on it ("A non-public class is not in `.environment`").
Nothing in Task 9 looks.

**Fix.** Task 9's check should be quantified over a *committed* list of names, not over what the
bootstrap installed, and should include at least `.environment~objectname`, one `.String` class
constant, one public class present in `.environment`, and one non-public class absent from it.

### F3 -- Task 1's negative control cannot go red under the representation the tree has. MAJOR, CONFIRMED

The control is "make `define` mutate in place instead of copying, and confirm the third row's test
goes red". Two problems, and the second is fatal.

**It has no instance to observe.** Task 1's own scope is "Data structure and its tests only; nothing
in `rexx-exec` calls it yet"; the object with an `ObjRef` identity arrives in Task 2. Row 3 is
phrased in terms of "the old instance", which does not exist in this task. The test must simulate one
by holding a behaviour handle, and the plan does not say it must capture that handle *before*
`define`. Written the obvious way -- ask the class for its behaviour, then `define`, then ask again --
the test passes under both behaviours, which is the failure the plan's own last sentence warns about
without stating the property that prevents it.

**The mutation is a no-op under the current representation.** `rexx-core::body.rs:22` is
`pub struct BehaviourId(pub u16)`, `behaviour.rs:31` is `entries: Vec<BehaviourEntry>` indexed by
it, and `body.rs:166` is `pub behaviour: BehaviourId` on the object. An id names a *slot*. "Copies
the behaviour first ... then replaces the entry" replaces the contents of the same slot, and every
holder of that `BehaviourId` sees the new contents -- identically to mutating in place. The copy is
observable only if `define` allocates a **new** `BehaviourId` and rebinds the class to it, which is
what the C++ does (`defineMethod`'s copy is installed into the class while existing objects keep
their old behaviour pointer) and which the plan never says.

So the implementer can write `define` as "clone, insert, store back at the same id", satisfy the
plan's wording, and the negative control's mutation changes nothing observable. The control passes
by not being a mutation.

**What would have come out differently.** If `BehaviourId` were a value handle rather than a slot
index, an object holding one would be immune to a slot rewrite and the control would bite. It is a
slot index; I read the three declarations.

**Fix.** State the property, not the ritual: `define` must leave the pre-existing behaviour handle
valid and unchanged, i.e. it allocates a new id. Then the negative control has something to remove.

### F4 -- Task 3's one-chokepoint assertion has no mechanism, no precedent, and contradicts the same task's "both engines". MAJOR, CONFIRMED

The plan says "Dispatch passes through exactly one chokepoint, and a test asserts there is exactly
one ... a count of call sites that reach method invocation, asserted at one".

**What would it count?** There is no instrument of this shape anywhere in the tree. I searched
`crates/rexx-exec/tests/*.rs` for `call site`, `call_sites` and `exactly one`; every hit is prose in
a doc comment about something else. So the mechanism is novel and unstated.

**The number cannot be one.** The Global Constraints require every construct to land in the
tree-walker and as a compiled op in the same task. `Engine` at `invocation.rs:136` has `TreeWalker`
and `Ir`, with `Engine::DEFAULT = Engine::Ir` (`:160`). Task 3 lands both `ExprKind::Message` and
`InstructionKind::Message`. That is four syntactic evaluation sites before anything internal.
Task 5's `::CONSTANT (expr)` evaluates a message at install time, Task 6 invokes an external entry,
and Task 8's `DO OVER` an object sends `MAKEARRAY` from inside the loop -- I measured that one: a
user class with `::METHOD makearray` prints `makearray called` under `do i over o`. Each of those is
another caller. The count is one only if the thing counted is the definition of `invoke`, which is a
tautology.

**The defect it cannot see.** A second dispatch path that does not call the chokepoint at all -- an
IR op that resolves and jumps straight to a native function pointer, a primitive fast path for
`~class`, a directive installer that calls a native method body directly. A count of call sites *of
the chokepoint* is structurally incapable of seeing a path that avoids it. That is the entire class
of thing Phase 7 will need the seam for.

**Fix.** Invert it. Make the property "nothing outside the dispatch module can name a native method
body" -- module privacy, checked by the compiler -- rather than "the one function has one caller".
The compiler can see a second path; a count cannot.

### F5 -- Task 6's check cannot be written in this phase, and not for the reason the brief guessed. MAJOR, CONFIRMED

Task 6 asks for "a corpus program per registered family that invokes one unimplemented entry and
pins the refusal".

**It is not blocked by method bodies in general.** Measured: a user program can declare
`::METHOD m CLASS EXTERNAL 'LIBRARY REXX file_case_sensitive'` on a `::CLASS K PUBLIC` and call
`.K~m` with no `~new` anywhere -- oracle rc 0, prints `prolog ran` then `1`. So a class-method
external is invocable within 5a's scope.

**It is blocked twice anyway.**

*First, for the `CoreClasses.orx` family specifically.* Its five entries -- `alarm_startTimer`,
`alarm_stopTimer`, `ticker_createTimer`, `ticker_waitTimer`, `ticker_stopTimer` at `:1590`, `:1618`,
`:1690-1692` -- are every one of them `PRIVATE` **instance** methods on `.Alarm` and `.Ticker`.
Reaching one in its declared shape needs an instance, and measured, `.Alarm~new` runs
`Method INIT with scope "Alarm"` and fails with 93.901 for want of two arguments. `~new` and `init`
are 5b by the plan's own split. A corpus program can only reach those entries by *re-declaring* them
on its own class, which tests our registry's name table and not the `.orx` file's declaration.

*Second, and this one kills the vehicle outright.* `corpus.rs`'s module doc: every program named in
a subset file is "run under both interpreters, compared byte for byte on stdout and exit code, and on
stderr up to DEVIATION 0's own narrow indent normalisation". I searched that file for
`EXPECTED`/`EXEMPT`/`expected_diverg`/`allow`: the only normalisation is DEVIATION 0's leading-
indentation rule. **There is no expected-divergence mechanism.** A program that pins our
"unimplemented, Phase 6" refusal diverges from the oracle by construction -- measured, the oracle
runs `alarm_startTimer` and answers `Error 88.901`, exit 168, not our refusal. Such a program cannot
be a corpus program; it reddens the gate.

**Fix.** The refusal belongs in a `rexx-exec` unit test or in `loud.rs`'s witness tables, which
already exist for exactly this ("a construct that is loud and owned by a later phase"), not in a
differential corpus subset.

### F6 -- Task 2's derived checklist derives the wrong names and omits the ones the prologue needs. MAJOR, CONFIRMED

I ran the plan's own quoted command:

```
/bin/grep -acE "createInstance\(\)" interpreter/memory/Setup.cpp   ->  31
```

The 31 lines are C++ **type** names, not Rexx class names: `RexxClass`, `RexxInteger`, `RexxString`,
`RexxObject`, `PointerClass`, `BufferClass`, `ArrayClass`, `TableClass`, `IdentityTable`,
`RelationClass`, `StringTable`, `DirectoryClass`, `SetClass`, `BagClass`, `ListClass`, `QueueClass`,
`NumberString`, `MethodClass`, `RoutineClass`, `PackageClass`, `RexxContext`, `StemClass`,
`SupplierClass`, `MessageClass`, `MutableBuffer`, `WeakReference`, `StackFrameClass`, `RexxInfo`,
`VariableReference`, `EventSemaphoreClass`, `MutexSemaphoreClass`.

Three problems, each measured.

**Unsound.** Two of them are not Rexx classes at all. Measured on the oracle,
`.environment~hasIndex` answers `NO` for `NUMBERSTRING` and `INTEGER`, and `yes` for `RexxContext`,
`RexxInfo`, `Buffer`, `Pointer`, `StackFrame`, `EventSemaphore`, `VariableReference`. So the derived
list contains entries that can never be "in the native layer" and must sit in a deferral table
forever with a reason that is not a deferral.

**Incomplete for this phase.** Searched `Setup.cpp` for `Comparable`, `OrderedCollection`,
`MapCollection`, `SetCollection`, `MessageNotification`, `AlarmNotification` and `LocalServer`: the
only hit in the whole file is a comment at `:797`. **Every class named in the prologue's `~inherit`
block at `CoreClasses.orx:93-119`, and every class named in its `addClass` block at `:47-52`, is
absent from the derived list.** They are `::CLASS ... MIXINCLASS` directives in the `.orx` file, so
the checklist that is supposed to bound the native set says nothing about the classes 5a's exit
criterion depends on.

**The mapping is the hand-written part.** `RexxString`->`String`, `ArrayClass`->`Array`,
`RexxContext`->`RexxContext` (prefix kept), `IdentityTable`->`IdentityTable` (nothing stripped),
`EventSemaphoreClass`->`EventSemaphore`, `RexxInteger`->nothing. There is no rule; there is a table.
Criterion 7 asks for a derivation and this yields a derivation plus a hand-maintained table, with the
drift living in the table.

### F7 -- Task 4's Done when is already true at HEAD. MAJOR, CONFIRMED

"`expr_owner` gives `ExprKind::DotVariable` an owner, both in this task's commit."

At HEAD, `crates/rexx-exec/src/lib.rs:1226` lists `ExprKind::DotVariable(_)` in the arm that returns
`None`, and `crates/rexx-exec/tests/owners.rs:220` is
`ExprKind::DotVariable(_) => ("DotVariable", Owner::InScope)`. Under the reading "owned, i.e. in
scope", the clause is satisfied before the task starts. Under the reading "carries a phase string",
satisfying it would be a regression -- and `bif-exempt.txt:40-44` explains why the rows are labelled
`UNATTRIBUTED` in the first place: "`ExprKind::DotVariable` is loud and `expr_owner` gives it no
phase, so the harness has nothing to derive one from". Once the construct works, `None` is the
correct and unchanged answer. Either way the clause is not a check on the task's work.

The other half of the Done when -- the `UNATTRIBUTED:an environment symbol` rows leaving
`corpus/bif-exempt.txt` -- names 9 data rows (`/bin/grep -ac` reports 10 including the comment at
`:40`): `LENGTH::test025#1`, `REVERSE::test017#1`, `REVERSE::test021#1`, `REVERSE::test026#1`,
`STREAM::test_relative_file_exists#1`, `STREAM::test_relative_file_exists2#1`,
`STREAM::test_relative_file_not_exists#1`, `STREAM::test_relative_file_not_exists2#1`,
`VALUE::test019#1`. The file's own header says what they are: "These rows read a `.NAME` the file
never sets -- REVERSE's `.funnyconst`, VALUE's `.a` -- which the oracle renders as the symbol's own
uppercased text." Measured: `say .funnyconst` -> `.FUNNYCONST`, rc 0. **That construct is in none of
Task 4's six verification programs.** The Verification and the Done when are about different
constructs.

### F8 -- the performance guard cannot attribute a regression in the code this phase writes. MAJOR, CONFIRMED

The guard's stated purpose is that "a regression is attributed rather than discovered later". Its six
axes are `alloc4c arith compound emptyloop strings varlookup`. I read all six programs in
`rust/bench-programs/`:

* `arith`, `compound`, `emptyloop`, `strings`, `varlookup` are arithmetic, stems, `nop`, string BIFs
  and simple variables. No message send, no class, no `.NAME`, no directive.
* `alloc4c`'s own header comment says it exists **because** the message-send version could not run:
  "restricted to the 4c surface (no message sends) ... `alloc.rex` allocates a fresh `.array` and
  `.string` every iteration via `.array~of`, `.string~new`, `~size` and `~length` -- all message
  sends, and this crate implements none yet".

So Tasks 1, 2, 6 and 9 land code the six axes never execute. For those tasks the guard can report
layout movement and nothing else, and the plan's tie-breaker language ("says whether it is the change
or the layout") presupposes the change is on the measured path.

Two omissions make this sharper. `rexx_bench::PROGRAMS` (`crates/rexx-bench/src/lib.rs:41-51`) also
contains **`dispatch`** -- whose source is a 5,000,000-iteration `c~bump` message-send loop, the exact
axis for the thing Task 3 builds -- and **`startup`**, the axis for the thing Task 9 could plausibly
move. Neither is in the guard's list, and the plan does not say to add `dispatch` when 5b makes it
runnable.

And the guard's own trigger is "any task that lands code in `rexx-exec`, `rexx-core` or
`rexx-classes`". **`rexx-lib` is not in that list.** Task 9, which embeds three `.orx` files and adds
a bootstrap path, is exempt from the guard by the guard's own wording.

**Answer to the brief's question:** no. A regression introduced by a new crate the axes never call is
invisible to that invocation except as layout noise, and the invocation would look exactly the same
had there been no regression.

### F9 -- Task 1's row 5 probe, as spelled, measures a refusal. MAJOR, CONFIRMED

The row is "`::class X mixinclass class` inherited into a class | the class-behaviour side, which an
instance-method diamond does not reach". I ran exactly that:

```
::CLASS X MIXINCLASS class
::METHOD cm
  return "from X"
::CLASS K INHERIT X
```

Oracle rc 158, stdout empty:
`Error 98.943: Class "The K class" is not a subclass of "The X class" base class "The Class class".`

So the probe as written pins an error, not a class-behaviour merge. The construct that does reach the
class-behaviour side is different, and I measured it working: `::CLASS M MIXINCLASS Object` carrying
`::METHOD cm CLASS`, then `.K~inherit(.M)` at run time, takes `.K~hasMethod("CM")` from `0` to `1`.
The row is achievable; its stated spelling is not it.

### F10 -- Task 5's `::CONSTANT` witness is real but blind to evaluation order. MEDIUM, CONFIRMED

The witness works. Measured, on `say "prolog"` plus `::CLASS K` plus `::CONSTANT c (.NoSuchClass~m)`:
oracle rc 159, stdout empty, stderr naming `97.1 Object ".NOSUCHCLASS" does not understand message
"M"`; this crate rc 0, stdout `prolog`. The no-`::CLASS` form is oracle rc 157, `99.906`. A
well-formed `(1+1)` is rc 0 / `prolog` on both. So the plan's claim that valid programs cannot see the
divergence is right, and the named witness does discriminate -- including against a lazy
implementation that evaluates at first reference, since the program never references the constant.

What it does not see is *when* the expression is evaluated relative to the rest of the directive
block. Measured:

```
say "prolog"
::CLASS K
::CONSTANT c (.L~id)
::CLASS L
```

Oracle rc 0, stdout `prolog`. The forward reference resolves, so **every `::CLASS` object exists
before any `::CONSTANT` expression is evaluated.** An implementation that evaluates constants in
strict source order fails that program with rc 159 and passes the plan's stated witness unchanged.
Add it; it is four lines.

### F11 -- widening `coverage.rs` reopens the hole its guard was written to close. MEDIUM, CONFIRMED-structural

`coverage.rs:387-395`, on `each_instruction`: "Descending into a routine is what lets
`assert_program_has_only_routine_directives` admit one at all: the guard exists to stop a subset
program hiding constructs from criterion 1 inside a body nothing walks."

Task 5 widens the walker to admit `::CLASS` and `::METHOD`. Criterion 1 is parse-only -- it counts a
variant as covered when a subset program *constructs* it. But 5a runs no method body. So after the
widening, a variant that appears only inside a `::METHOD` body is credited as covered while nothing
in the phase ever executes it, and `corpus.rs` -- the half that "proves they execute correctly" --
cannot compensate, because it runs the program and the body is never entered. The guard's asymmetry
moves from "hidden and uncounted" to "counted and unexecuted", which is the worse of the two. The plan
should say that a variant whose only occurrence is inside a method body does not count for 5a.

### F12 -- Task 7 cannot tell "provided, used, removed" from "never provided". MEDIUM, CONFIRMED

Measured on the shipped oracle: `.String~hasMethod("DEFINECLASSMETHOD")` is `0`,
`.Supplier~hasMethod("INHERITINSTANCEMETHODS")` is `0`, `.Class~hasMethod("DEFINE")` is `1`, and
`.String~defineClassMethod(...)` raises `97.1` at rc 159. Every figure the plan quotes is right.

All three answers are also what you get from an implementation that never implemented either setup
method. The observable that discriminates is what `defineClassMethod` *did*: measured,
`c2x(.String~nl)` is `0A` and `c2x(.String~tab)` is `09`, from the `:70-74` loop. Task 7 does not
check it, and Task 9 does not either (F2). Its Done when's first clause -- "the prologue reaches its
`~inherit` block" -- names no instrument at all; it is self-report.

### F13 -- Task 8's ownership clause has no subject, and its verification cannot witness the dispatch. MEDIUM, CONFIRMED

"The ownership move edits all five `owners.rs` pinned items in this commit." There is no ownership
move: `owners.rs:247` is already `LoopKind::Over { .. } => ("Over", Owner::InScope)`, which the task's
own prose states. The clause is satisfied by editing nothing, and satisfied identically whether the
`DO OVER` work happened or not.

The substantive requirement is that `DO OVER` an object *sends* `makearray` or `supplier`. Measured,
the oracle does: `do i over o` on a class with `::METHOD makearray` prints `makearray called` first.
But the only receivers 5a can construct are native collections -- `.array~of`, `.directory~new` --
whose `makearray` an implementation can inline without any dispatch, and the stated verification uses
exactly those. A witness needs an instance of a user class, i.e. `~new`, i.e. 5b. (A
`::CONSTANT makearray (.array~of(10,20))` body works on the oracle -- I measured it, rc 0, prints
`10` and `20` -- but still needs `.Thing~new`.) So the check passes over a build with no dispatch in
the loop at all.

### F14 -- Task 10's subset list: one item is expressible only via a construct the plan never names. MEDIUM, CONFIRMED

Walking the list against 5a's scope:

* *a native message send to each primitive kind* -- expressible.
* *the five wiring answers per class in the native set* -- expressible, and blind (F1). Note the plan
  does not say how `~superClasses` is rendered byte for byte; `~makestring` is a sourced
  `OrderedCollection` method, so the rendering has to be `do c over ...~superClasses; say c~id; end`,
  which is Task 8's work.
* **a mixin diamond that discriminates merge order from a chain walk** -- *not* expressible the
  obvious way. Discriminating the winner means invoking it, and reflection does not substitute:
  measured, iterating `.D~instanceMethods` yields no entry for the inherited `M` at all. With
  `::METHOD` bodies this needs 5b. It **is** expressible with `::CONSTANT`: measured,
  `::CLASS A MIXINCLASS Base` / `::CONSTANT m "A"`, same for `B`, `::CLASS D INHERIT A B`, then
  `say .D~m` prints `A` at rc 0 with no Rexx body entered. The plan should say so; written the
  natural way this item is a task that cannot be done in the phase that owns it.
* *a class-behaviour-side witness* -- expressible, but not as Task 1 row 5 spells it (F9).
* *the four directives including the failing-expression `::CONSTANT`* -- expressible; note it is a
  nonzero-rc program, which `corpus.rs` compares on exit code and handles.
* *both `VALUE` routes and `say .LOCAL`* -- expressible; measured oracle answers above.
* *both `DO OVER` shapes* -- expressible, and blind to the dispatch (F13).
* *one unimplemented native entry point per registered family* -- **not expressible as a corpus
  program** (F5).

### F15 -- Task 2's `~isA` names no argument, and every argument in reach answers 1. MINOR, CONFIRMED

Measured: `.Supplier~isA(.Object)` is `1`, `.Bag~isA(.Class)` is `1`. With the argument unspecified,
the natural choices (`.Object`, `.Class`) are `1` for every class in the native set under any wiring
whatsoever. The plan must name the argument per class, and the informative choice is a mixin the
class is supposed to have -- which is also the only choice that would catch F1's merge/mixin errors.

### F16 -- Task 10's unsafe report is blind to a new crate that opts out. MINOR, CONFIRMED

The report item is "the unsafe-block count and the list of crate roots carrying `deny` rather than
`forbid`". `rust/Cargo.toml:10-22` sets `[workspace.lints.rust] unsafe_code = "forbid"`, and the
comment there is explicit that `deny` at a crate root *is* the record of a granted exception.

But `[workspace.lints]` applies to nothing without a per-crate opt-in. I checked all eight existing
crates: each carries `[lints]` / `workspace = true` explicitly. (My first grep for `lints.workspace`
found none of them and would have produced a false finding -- recorded here because the pattern, not
the tree, was wrong.) The two new crates this plan adds, `rexx-classes` and `rexx-lib`, must add
those two lines themselves. A crate that omits them carries **neither** `deny` nor `forbid`, so it
does not appear on a list of roots carrying `deny`, and the phase-exit report is silent about it. Add
"every crate root carries the workspace lints opt-in" to the reported set.

### F17 -- "the three `.orx` files" is four files. MINOR, PLAUSIBLE

`git ls-files` names `interpreter/RexxClasses/CoreClasses.orx`,
`interpreter/RexxClasses/StreamClasses.orx`, `interpreter/platform/unix/PlatformObjects.orx` and
`interpreter/platform/windows/PlatformObjects.orx`. Task 9 says three files, embeds each by a
workspace-relative path and records "each file's sha256 so a drift is a build failure". Which
`PlatformObjects.orx`, and whether the sha256 pin is per-platform, is unstated. This is a plan gap
rather than a blind check, hence PLAUSIBLE.

---

## What I searched for and could not reach

* **A second dispatch path in the tree today.** There is none to find: `/bin/grep -rn` for
  `ExprKind::Message|InstructionKind::Message` across `crates/` returns only parse, plan, owner-table
  and loud-witness sites. F4 is therefore a claim about a check the plan proposes, argued from the
  shape of the count, not from an existing second path.
* **An expected-divergence mechanism in the corpus harness.** Searched `corpus.rs` for `DEVIATION`,
  `EXEMPT`, `expected_diverg` and `allow`. Only DEVIATION 0's indentation normalisation exists. A
  mechanism spelled some third way would falsify F5's second half.
* **Whether the `dispatch` and `startup` axes were deliberately excluded from the guard.** I read
  `PROGRAMS` and `NOT_BENCHMARKED`'s doc but did not read every task brief in `bench-baselines/`; if
  an earlier phase recorded a reason, F8's second paragraph is weaker than stated.
* **`rexx-arms` actually running.** I verified its flags parse and its baseline artifacts exist
  (`bench-baselines/pinned/rexx-run-pre-phase-5`, `target/release/rexx-arms`) but did not run a
  five-round sitting; F8 is an argument about the axes' content, which I read in full, not about the
  tool.
* **The oracle's `.Bag`/`.Set` method dictionaries in full.** F1 rests on two names, `ALLITEMS` and
  `UNION`. I did not enumerate every method each mixin contributes; a reader wanting the strongest
  form of the finding should.
