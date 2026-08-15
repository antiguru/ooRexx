# Phase 5 -- the object model

**Status:** design, revised once after a five-reviewer adversarial panel. Not planned.
**Entry:** met. Phase 4f closed; `perf-baseline.md`'s "The pre-Phase-5 baseline" pins the standing at `b029abe77`.
**Blocks:** Phases 6, 7 and 8, which are independent of each other and may run in parallel once this closes.
**Decided:** 2026-08-15. Revised 2026-08-15 after review.

The evidence half of this spec is `2026-08-15-phase-5-inherited-surface.md`: every place the tree already
names Phase 5 as owner, read at the tree, plus the fourteen questions this document answers. Read it for
what is inherited; read this for what is decided. Where the two disagree, the survey is a dated reading of
`11638b91e` and this is the decision. **One of the survey's own claims is withdrawn here** -- see
[the bootstrap's reach](#what-the-bootstrap-actually-reaches).

**What the review changed, and it was the shape of the phase rather than its details.** The first draft's
headline criterion -- the three `.orx` files run to completion -- is not reachable by a phase scoped to the
object model, because two of the three bind natives that belong to Phases 6 and 7 and they bind them
*eagerly, at directive-install time*. The first draft also proposed a wiring check that could not fail, and
claimed a byte-for-byte corpus comparison that the corpus harness does not perform. All three are fixed
below and each is called out where it sits.

## The sentence the roadmap gets wrong, and what it changes

`2026-07-27-rust-rewrite.md:82` reads: *"a Rust core that can execute `CoreClasses.orx` inherits 32 classes
for free. Getting `CoreClasses.orx` to run is therefore the single highest-leverage milestone in the plan
(Phase 5), and everything before it is scaffolding for that moment."*

**The leverage claim survives. The mechanism does not.** `CoreClasses.orx` does not bootstrap an object
model; it runs on top of one that already exists. `interpreter/memory/Setup.cpp`'s `createImage()` builds
the primitive classes in C++ first, and *only then*, at `Setup.cpp:1785`, resolves `BASEIMAGELOAD`
(`"CoreClasses.orx"`, `platform/unix/PlatformDefinitions.h:99`) and runs it as a program with
`TheRexxPackage` as its single argument.

`createImage()` is the **image-build** path, run by `rexximage` at C++ build time, not the startup path.
That does not weaken the point: it is where the C++ decides what is native and what is sourced, which is
the question this phase has to answer, and the ordering it fixes is the ordering the restored image
reproduces.

The classes it builds, **in the order it builds them**, which is `createInstance()`'s call order in that
file and not an alphabetisation:

```
RexxClass RexxInteger RexxString RexxObject PointerClass BufferClass ArrayClass TableClass
IdentityTable RelationClass StringTable DirectoryClass SetClass BagClass ListClass QueueClass
NumberString MethodClass RoutineClass PackageClass RexxContext StemClass SupplierClass
MessageClass MutableBuffer WeakReference StackFrameClass RexxInfo VariableReference
EventSemaphoreClass MutexSemaphoreClass
```

`RexxClass` and `RexxInteger` come first because, as the file says, they "ha[ve] some special stuff";
`RexxString` and `RexxObject` next because they are "fairly critical". The rest of the order has no comment
on it and should not be assumed arbitrary.

What is in `CoreClasses.orx`, read at it: its directives are `::METHOD`, `::CLASS` and `::ATTRIBUTE`, and
nothing else. Its `::CLASS` directives define classes that are **not** primitives -- some declared
`MIXINCLASS` (`Collection`, `OrderedCollection`, `MapCollection`, `SetCollection`, `Comparable`,
`Comparator` and its seven subclasses, `Orderable`, `MessageNotification`, `AlarmNotification`,
`Singleton`), some plain classes that exist only to donate methods (`SupplierMixin`, `ManyItemMixin`,
`SetMixin`, `BagMixin` -- named "mixin" by the file but **not** declared `MIXINCLASS`, which matters
because the prologue installs them with `~inheritInstanceMethods` rather than `~inherit`), and some
ordinary Rexx-level classes (`Alarm`, `Ticker`, `Monitor`, `CircularQueue`, `Properties`, `DateTime`,
`TimeSpan`, `ArgUtil`, `Validate`, `LocalServer`, `TraceObject`). Its file-scope `::METHOD` block --
`string_cls_nl`, `string_cls_alnum` and the rest -- exists to be attached to primitive classes that the
native layer has already created; the file's own comment calls them *"unattached METHOD definitions for
the various enhanced objects created above"*.

So **`CoreClasses.orx` is a target, not a bootstrap.** It is the acceptance test for the native layer, not
the source of it. A native port of the file, Q1's option (b), would have been a category error.

## What the bootstrap actually reaches

**The first draft treated `CoreClasses.orx` as opaque and proposed discovering its requirements by running
it. Most of them are readable today, and this section reads them.** That does not overturn the decision to
grow the native set by running the file; it bounds it, which is what makes it a task rather than a loop.

### The prologue is the object-model workout

`CoreClasses.orx`'s executable top level runs from `use arg rexxPackage` to `exit`, ahead of the first
directive. Every clause in it is object-model machinery, and this is the single densest statement of what
the phase must build:

| the prologue does | what that requires |
|---|---|
| `use arg rexxPackage` | a live **Package object** passed as the program's argument |
| `rexxPackage~addClass`, `~addPublicClass`, `~objectname=` | three Package methods, one of them an attribute assignment |
| `.environment~objectname = "The Environment Directory"` | `.environment` as a real object accepting a message |
| `publicClasses = .context~package~publicClasses` | **`.context`**, the per-activation `RexxContext` reflection object, live during the bootstrap |
| `do name over publicClasses` | `DO OVER` a **collection object**, not an array |
| `publicClasses[name]` | the `[]` index message |
| `.environment~put(class, name)` | Directory `~put` |
| `.methods[("string_cls_" \|\| name)~upper]` | **`.methods`**, the package's unattached-method directory |
| `.String~defineClassMethod(name~upper, ...)` | a class-side method that mutates behaviour after definition |
| `.supplier~inheritInstanceMethods(.SupplierMixin)` and four more | method donation without a superclass edge |
| `.string~inherit(.Comparable)` and fifteen more | real mixin edges, and the `updateSubClasses` cascade |
| `call 'StreamClasses.orx' rexxPackage` and the same for `PlatformObjects.orx` | a `CALL` on a **program name** |

Two of those are not classes at all and are named nowhere in `Setup.cpp`'s `createInstance()` list:
`.context` and `.methods`. Both exit 120 today with `an environment symbol is not implemented`. The
prologue is therefore not satisfied by the class registry alone, and D33's directory work is a hard
prerequisite for it rather than a parallel track.

### The `EXTERNAL` wall, and why criterion 1 had to change

**`::METHOD ... EXTERNAL 'LIBRARY REXX name'` binds eagerly, when the directive is installed, and the
program's prologue never runs if a bind fails.** Measured this session against the oracle: a file whose
prologue is `say "prolog ran"` and which carries one `::METHOD zz EXTERNAL 'LIBRARY REXX nosuchentry'`
exits **166** with **empty stdout** and `Error 90.998: Unable to find external method "nosuchentry"`. The
same file naming a real entry point (`alarm_startTimer`) exits 0 and prints.

`::CONSTANT` with a parenthesised expression evaluates at install time too. Measured: a file whose
prologue is `say "prolog ran"` and which carries `::CONSTANT sep (.NoSuchClass~getThing)` exits **159**
with **empty stdout**.

Against that, what the three files carry:

* **`CoreClasses.orx`** has `::METHOD ... EXTERNAL` at `:1590`, `:1618`, `:1690`, `:1691`, `:1692` --
  `alarm_startTimer`, `alarm_stopTimer`, `ticker_createTimer`, `ticker_waitTimer`, `ticker_stopTimer`.
  Timers. Phase 6's subject. They are **bound** during bootstrap and **not called** by it.
* **`StreamClasses.orx`** carries a large family of `EXTERNAL` declarations naming stream and file
  natives, plus `::CONSTANT separator (.File~getSeparator)` and `::CONSTANT pathSeparator
  (.File~getPathSeparator)` at `:548`-`:549`, whose targets are declared at `:546`-`:547` as
  `external "LIBRARY REXX file_separator"` and `"LIBRARY REXX file_path_separator"`. Those two are
  **called** at install time, not merely bound.
* **`PlatformObjects.orx`** on unix is one line: `-- Nothing to do currently`.

`::METHOD EXTERNAL` is already a declared **Phase 7** refusal in this crate
(`rexx-exec/src/lib.rs`, `directive_gap`'s `::METHOD EXTERNAL` arm). And the roadmap's **Phase 7** row, not
Phase 5's, is the one that says *"`StreamClasses.orx` runs"*; the Phase 5 row names `CoreClasses.orx` and
nothing else. The crate-layout line at `:518` says `rexx-lib/ # Phase 5: loads CoreClasses.orx /
StreamClasses.orx` -- **loads**, which the first draft converted to *runs to completion*. The survey made
the same slip, quoting the one-asset gate row and then writing "the three assets that gate names"; that
sentence of the survey is withdrawn.

**The resolution, and it is [D37](#decisions-recorded-here).** Phase 5 builds a **native entry-point
registry**: every `LIBRARY REXX` name the three files declare is *registered*, so eager binding resolves
exactly as it does on the oracle. A registered entry Phase 5 does not implement raises this crate's
not-implemented condition **when invoked**, naming its owning phase. That confines the divergence to
invocation, which is where the later phase lands, and keeps install-time behaviour identical.

Two entry points are pulled into Phase 5 as an explicit, stated scope addition, because `CoreClasses.orx`
cannot reach its own `exit` without them: `file_separator` and `file_path_separator`. They return platform
constants and nothing about them is a stream. Everything else in `StreamClasses.orx` stays Phase 7's.

## The three layers

```mermaid
flowchart TD
    A["native layer, Rust: rexx-classes<br/>primitive classes, behaviours,<br/>.environment / .local / .context / .methods,<br/>dispatch, the native entry-point registry"] --> B["CoreClasses.orx<br/>prologue wires the primitives;<br/>directives add mixins and utility classes"]
    B -->|"CALL on a program name"| C["StreamClasses.orx: installs<br/>PlatformObjects.orx: one comment line"]
    A -.->|"acceptance test"| B
    C -.->|"bodies run"| D["Phase 7"]
```

**The line between the native layer and the sourced layer is discovered by running `CoreClasses.orx`**
(Q1, decided 2026-08-15 by the human partner), rather than transcribed from `Setup.cpp`.

**The discovery is bounded by the prologue table above, and that is what makes it a task list rather than
an open loop.** The first draft said "grown one refusal at a time" and left the growth unbounded. It is
worse than unbounded as stated: the refusal instrument changes character partway through. `directive_gap`
refuses at install and enumerates *forms*, so it is a finite, readable list; once `::CLASS`, `::METHOD` and
`::ATTRIBUTE` land, the file installs silently and every later failure arrives one clause at a time from
the prologue. So:

* the **install-time** obligations are enumerable today and are enumerated above;
* the **prologue** obligations are enumerable today and are enumerated above;
* what genuinely remains discoverable-only is the behaviour of the method bodies, and those are not needed
  for criterion 1 -- they are needed for criterion 2, which is a corpus subset the plan writes.

**The hazard the discovery choice carries, stated rather than argued away.** `createInstance()` does more
than make a class exist: it fixes metaclass links, superclass edges and the order the behaviours are built
in, and a wrong link there does not fail where it is made. Two mitigations, and the first draft's version
of the first one **did not work**:

* **The wiring assertion must include `~superClasses`.** The first draft proposed `~class`, `~superClass`,
  `~isA` and `~metaClass`. Measured on the oracle: `.DateTime~superClass~id` is `Object` and
  `.Array~superClass~id` is `Object`, so **every mixin edge is invisible to that set** -- including the
  `inherit Comparable Orderable` this document cites two sections above, and the `.array~inherit(
  .OrderedCollection)` the prologue installs. `.DateTime~superClasses~makestring("LINE", ",")` answers
  `The Object class,The Comparable class,The Orderable class` and is the answer that sees them. A build
  that wired no mixin at all passed the first draft's check. This is the project's own recurring failure
  mode -- a check that runs, exits 0, and cannot see its subject -- shipped inside the mitigation for it.
* **The `Setup.cpp` list is a checklist, not a plan.** Every class that file creates and this phase does
  not is recorded in a table with a reason, and `rexx-classes` carries that table as a test over its own
  registry. A reason is a sentence naming what would have to exist; "not needed yet" is not one.

### The native layer -- `rexx-classes`

New crate (Q3, decided 2026-08-15 by the human partner), as `2026-07-27-rust-rewrite.md:517` plans -- where
it is marked "Phase 4-5", and nothing in Phase 4 created it, so all of it is this phase's. It owns the
class objects for the primitive kinds and the registry that finds them; the behaviour tables and the method
dictionaries they flatten; the metaclass graph, including `.class` being an instance of itself;
`.environment`, `.local`, `.context` and `.methods`; and the native entry-point registry.

**The one-way dependency the first draft asserted does not hold, and pretending otherwise would put the
boundary in the wrong place.** `.SomeClass~new` runs that class's `init`, which is a Rexx method body, which
runs on the executor. So class-side behaviour reaches execution. The boundary that *does* hold is
**dependency inversion at a trait**, not a one-way crate edge:

| `rexx-classes` owns | `rexx-exec` owns |
|---|---|
| the class graph, behaviours, flattened dictionaries, the registry | method **bodies**, and running them |
| `resolve(receiver_behaviour, name, start_scope, per_object) -> MethodId` | the `MethodId -> body` table and the invoke path |
| `define(class, name, MethodId)` and the `updateSubClasses` cascade | argument evaluation, the activation stack, conditions |
| the entry-point registry | the trait implementation `rexx-classes` calls to run a body |

`rexx-classes` depends on `rexx-core` and declares a trait for "run this method id against this receiver".
`rexx-exec` implements it. Neither names the other's concrete types. **Reaching past this is a spec
amendment, not a refactor** -- and the amendment bar is why the first draft's three-row prose table is
replaced by signatures above.

**`resolve` takes a start scope and a per-object table, and the first draft's version could not express
`FORWARD CLASS(SUPER)`.** The ordinary send is `behaviour->methodLookup(msgname)` --
`interpreter/classes/ObjectClass.cpp:866`, one hash lookup, as the first draft said. The scope-override
send is `superMethod(msgname, startscope)` at `:919`, reaching `RexxBehaviour::superMethod`
(`interpreter/behaviour/RexxBehaviour.cpp:584`) and `MethodDictionary::findSuperMethod`; `MethodDictionary`
carries a `scopeList` and `scopeOrders` alongside the name map, plus an `instanceMethods` table for
per-object methods. `CoreClasses.orx` uses `forward class(super)` throughout. So the flattened dictionary
must **retain scope ordering**, not merely merge names.

### The sourced layer -- `rexx-lib`

New crate, `2026-07-27-rust-rewrite.md:518`. It owns the three `.orx` files and the bootstrap that runs
them. The files are **tracked in this repository** -- they sit under `interpreter/RexxClasses/` and
`interpreter/platform/unix/`, which this worktree contains -- so a build script reaches them by a path
relative to the workspace, the way `crates/rexx-inventory/build.rs` already reaches the C++ tree. Nothing
depends on an absolute path or on a second checkout, and CI's five platforms are unaffected. **They are
never edited.** A gap in our interpreter is fixed in our interpreter.

`rexx-lib` also owns the **program-name resolution for the two `CALL`s in the prologue**. `call
'StreamClasses.orx'` would otherwise go to the external file search, which `Loud::unresolved_call`'s own
doc assigns to Phase 7. `rexx-lib` intercepts the three bootstrap names ahead of that search and never
touches the search itself; anything else stays Phase 7's.

## Dispatch

**D24 settled the shape, and D28 below amends half of it.** D24's words are: *"Dispatch is implemented once
as `resolve` and `invoke`, not one fused `send`. **The IR's `Send` op caches the resolution** and calls the
invocation; the tree-walker and `eval.rs` call the same pair **uncached**."* The first draft quoted that
sentence with the caching clauses removed and then said the phase does not revisit D24, which hid an
amendment behind a misquote. The `resolve`/`invoke` split stands unchanged. The caching clause does not.

**Resolution is dynamic, with no per-call-site cache** (Q2, decided 2026-08-15 by the human partner).
`ir.rs`'s `CallSite` slot stays a classic-call cache and no send goes through it. Phase 4e's own later
amendment (`2026-08-09-phase-4e-ir.md:1082`) is the reason and it is worth repeating because the opposite
reading is the natural one: the classic call-site cache is correct **because it needs no guard**, which is
precisely the property a send cache lacks. The classic case validates the seam and not the caching
discipline. And the measurement that would justify a send cache does not exist -- `dispatch`, `alloc` and
`heapshape` have no Rust number at all.

D24's forward constraints for Phase 5 (`2026-08-09-phase-4e-ir.md:1141`) are five: selectors interned at
compile time, a `SmallInt` behaviour arm, a receiver in the calling convention, a wider patch entry, and
invalidation on behaviour mutation. D28 disposes of the last two by declining the cache. **The first three
are this phase's and the plan owes a task for each**; this spec does not decide their shape.

### The behaviour table is flattened, and that is semantics rather than caching

**This is the part most likely to be got wrong, because the existing type invites the wrong shape.**
`rexx-core`'s `BehaviourTable::lookup` walks a superclass chain at lookup time, following
`BehaviourEntry::superclass`. ooRexx does not resolve that way. `RexxClass::createInstanceBehaviour`
(`ClassClass.cpp:1148`) builds a **flat** method dictionary into the behaviour when the class is defined,
and `updateSubClasses` / `updateInstanceSubClasses` (`ClassClass.cpp:1036`, `:1071`) clear it, rebuild it,
and cascade the rebuild to every subclass, whenever anything upstream changes.

Two things follow, and neither is negotiable:

* **A chain walk gives different answers.** ooRexx has multiple inheritance through `MIXINCLASS` and
  `INHERIT`; `CoreClasses.orx` uses it directly (`::CLASS 'DateTime' public inherit Comparable Orderable`)
  and its prologue installs a further set of `~inherit` edges onto the primitives -- `.string` onto
  `Comparable`, the ordered collections onto `OrderedCollection`, eight map collections onto
  `MapCollection`, `.set` and `.bag` onto `SetCollection`, and `.message` onto both notification mixins.
  The method a diamond resolves to is whatever `createInstanceBehaviour`'s merge order produces.
  `BehaviourTable` as it stands cannot express a mixin at all -- `superclass` is a single
  `Option<BehaviourId>`.
* **"Always dynamic" and "flattened" are not in tension.** The decision not to cache is about the *call
  site*. The flattened dictionary is not a cache of resolution; it *is* resolution, and its invalidation is
  not an optimisation problem but the `updateSubClasses` cascade, which the oracle already specifies.

`BehaviourId` survives as an index into that table and is **not** the class's identity (Q5). A class is an
object -- `.class` is an instance of itself, confirmed on the oracle: `.class~class == .class` answers `1`
-- so its identity is an `ObjRef`. `BehaviourId` stays a `u16` index for the primitive fast path, derived
from the class object and never the other way round. The collector's root set stays enumerable because
class objects live in the arena like everything else, which is D1's own criterion.

## `.environment`, `.local`, `.context`, `.methods`

All four become real objects in `rexx-classes` from the first task, because the bootstrap prologue needs
all four before any method body runs. Every consumer resolves through them: `.NAME` environment-symbol
lookup, the class registry, and `VALUE`.

**The `VALUE` gap is the two-argument form, and the first draft's decision named a different one.**
`phase-4-exclusions.txt` keeps two separate KNOWN GAP rows and they are not interchangeable. Measured this
session, both interpreters:

```
say value('.LOCAL')       oracle rc 0 "The Local Directory"    this crate rc 0 ".LOCAL"
say value('.ARRAY')       oracle rc 0 "The Array class"        this crate rc 0 ".ARRAY"
say value('.LOCAL',,'')   oracle rc 0 "..LOCAL"                this crate rc 120, external-selector refusal
say .LOCAL                                                     this crate rc 120
```

The **two-argument** form is the silently-wrong one and is what this phase closes: no selector, a
leading-dot name, correct fallback to the upcased spelling when undefined, but no package-environment
lookup or reflection-name table in front of it. The **empty-selector** three-argument form is a loud
refusal whose oracle answer is `..LOCAL`, and closing it does not touch the silent path at all;
`phase-4-exclusions.txt`'s external-selector row records that only one of its three branches is Phase 5's.
Both the silent `value` route and the loud `say .LOCAL` route close here, together, or the asymmetry
survives.

## Directives

`::CLASS`, `::METHOD` and `::ATTRIBUTE` are this phase's.

**`::CONSTANT` is this phase's too, and it carries a divergence nothing had recorded.** The directive
parses today and a simple form runs; the **parenthesised expression form is accepted and never
evaluated**. Measured this session on a file whose prologue is `say "prolog"`:

```
::CONSTANT c (.NoSuchClass~m)     oracle rc 159, stdout empty     this crate rc 0, stdout "prolog"
```

The oracle evaluates the expression when the directive installs and the program never starts; this crate
parses it, stores nothing, and runs the prologue. `StreamClasses.orx:548` is exactly this form, calling a
native, so criterion 1 depends on closing it. **New KNOWN GAP** -- the plan's first task records it in
`phase-4-exclusions.txt` before fixing it.

**`::OPTIONS` and the `OPTIONS` instruction stay in this phase**, sequenced independently of dispatch
([D31](#decisions-recorded-here)). The first draft moved them out on the ground that package settings are
not the object model. That is true and it is not sufficient: a phase is a unit of work, not a taxonomy, and
moving them out created an owner nobody could spell while contradicting criterion 2, which asks for a
witness per `directive_gap` Phase 5 arm. They are cheap, they delete two over-refusals, and they gate
nothing -- so they land whenever the plan finds room.

**One supporting sentence from the first draft is withdrawn as false**: `::OPTIONS`'s effect is not
`digits()`, `form()` and `fuzz()`. `LanguageParser::optionsDirective`
(`interpreter/parser/DirectiveParser.cpp:948` onward) accepts `DIGITS`, `FORM`, `FUZZ`, `TRACE`, `NOVALUE`,
`ERROR`, `FAILURE`, `LOSTDIGITS`, `NOSTRING`, `NOTREADY`, `ALL`, `NOPROLOG`, `PROLOG` and `NUMERIC`, so it
reaches the condition regime and program prolog handling; `phase-4-exclusions.txt` describes it as changing
package settings unconditionally with `digits` as its worked example, and the first draft turned that one
example into an enumeration of the whole.

`::REQUIRES` stays in Phase 5 and lands **last**, after the bootstrap passes without it.
**The first draft's reason was half wrong.** `ir.rs`'s `CallSite` argument does rest on `Interp::routines`
being append-only and `::REQUIRES` does break that; the plan cache's `BodyKey` pairing does not depend on
it in the way the first draft claimed, and that half of the sentence is withdrawn. The remaining reason
stands, and `CoreClasses.orx` does not use `::REQUIRES`, so nothing is bought by landing it early.

`::ANNOTATE naming a target` is the fourth `directive_gap` Phase 5 arm and **nothing in this project has
measured what it does**. The plan's first task measures it; until then it is not in any criterion.

**`REPLY` and `GUARD` are checked at RUN time, not translation time, and the first draft got this wrong
from its own probe.** The error text says `Translation error` and the first draft read that as the
mechanism. Measured this session on the oracle:

```
if 1=0 then reply          rc 0, prints "reached"
if 1=0 then guard on       rc 0, prints "reached"
say "before" ; reply       rc 157, stdout "before", then Error 99.919
```

`reportException` sits inside `RexxInstructionReply::execute`. So a translation-time refusal would
**over-refuse programs the oracle runs**, and D32 is restated: Phase 5 implements the run-time legality
check, reached only when the instruction executes, exiting 157 with the oracle's bytes. The concurrency
behaviour of a `REPLY` that *is* inside a method is Phase 6's, and the plan must say what a Phase 5 method
containing a `REPLY` does -- this spec does not, and that is an open question below rather than a silent
gap.

## Trace: `>M>` and `>N>`

The 4c gate hands this phase a hole. **The first draft proposed two instruments for it and one of them
cannot see it.**

`tests/support`'s `descriptor_diffs` runs both sides through `normalize_stderr`, which collapses the
space run carrying nesting indent: an off-by-two indent on a `>M>` or `>N>` line normalises away, while a
content change on the same line does not. And `ir_dual` diffs **our two engines against each other**, both
of which format trace through one shared `trace.rs` -- so a wrong indent is wrong identically on both arms
and `ir_dual` stays green. `ir_dual` is a real instrument for a great many things and is not one for this.

So the instrument is **in-crate exact-stderr assertions**, in the shape the existing indent tests in
`rexx-exec/src/run/tests.rs` use, and they are the only thing pinning this. The plan owes: which nesting
depths, and where each expected byte string came from -- an expected string typed by the implementer
rather than captured from the oracle pins the implementation to itself.

`trace_oracle.rs`'s `PREFIX_COVERAGE` declares `>M>` and `>N>` as `Coverage::Owned("Phase 5")`. Satisfying
this section means changing those rows, and the plan owes that edit.

**A separate stderr divergence, also Phase 5's, that the first draft missed entirely.** An error traceback
through a method frame prints a scope line the oracle produces and this crate does not. Measured, `say b. + 1`
with `b.` untouched, both sides rc 215; the oracle's stderr opens with
`*-* Compiled method "+" with scope "String".` and this crate's does not. That is not a trace prefix, so
nothing in this section covers it, and any `phase-5.txt` program that touches it fails criterion 2 on
stderr. The plan owes a task.

## Value representation

`Body` is asserted at **at most** 80 bytes and its own comment says **"Phase 5 adds variants and is
expected to trip this"**. It trips deliberately or not at all: a new kind arrives boxed behind
`Body::Instance` unless a measurement is recorded that says widening is worth it for that kind (Q4). The
measurement is a two-build `rexx-arms` sitting against the pinned baseline, and "worth it" means the
movement exceeds the floor named in the bar below.

The assertion is an upper bound, so it catches a *widening past 80* and not a widening within the
headroom of the current widest variant. That is the guard this phase has; it is not a guard against every
growth, and the plan should not treat it as one.

`ExprKind::List` becomes a real Array in this phase, from the first commit that makes `~` work (Q10). The
ast doc's own discriminator is why: `(1,)~size` is 2 and `(1,,)~size` is 3, which no string approximation
reproduces, and which is unobservable until dispatch lands and observable immediately after.

## The bar

**Performance work is suspended for this phase**, per the roadmap's 2026-08-14 decision, conditional on
two things. The first is done: the standing is pinned at `b029abe77` in `perf-baseline.md`, with a staged
binary and `bench-baselines/pre-phase-5-arms.tsv`. The second is this phase's obligation.

**The guard is the six benchmark axes `rexx-arms` measures**: `alloc4c`, `arith`, `compound`, `emptyloop`,
`strings`, `varlookup`. The first draft omitted `emptyloop` and listed `rexxcps`; `rexxcps` is not an axis
in `bench-programs/` and it self-calibrates its own workload from wall clock, so it cannot serve as a
regression guard at all. It stays in `perf-baseline.md` as a standing figure and is not part of this
criterion.

**The floor.** `bench-baselines/README.md` states that interleaving removes the machine's drift from an
`across_builds` row and does not remove the code placement's, bounding the latter at ±0.74% on one axis
and recording 7.8% between two builds of the same source differing by one comment. **Those are two
different numbers and neither is "the floor" as a rule a task author can apply.** This phase's rule, stated
here so two people cannot answer differently: **a two-build `across_builds` movement under 1% on an axis
is not a finding; at or above 1% the task records it and says whether it is the change or the layout, and
an `arm_ratio` row, which carries no placement difference, is the tie-breaker where one is expressible.**

**What a guard run is.** `--build pinned=bench-baselines/pinned/rexx-run-pre-phase-5 --build
head=target/release/rexx-run`, interleaved, at each task that lands code in `rexx-exec`, `rexx-core` or
`rexx-classes` -- that is the decision procedure, not "touches an execution path".

`dispatch`, `alloc` and `heapshape` become runnable during this phase and their first readings are **new
measurements, not regressions**. Re-role them in `rexx-bench-suite`'s `AXES` table in the same commit that
removes the refusal, so the suite's blocked-axis assertion stays true rather than red.

**`apply_binary` is not optimised in this phase and not measured before it.** The 13.5%-of-allocations
figure comes from the roadmap's Phase 4 text and predates the pinned baseline; it is quoted as the reason
the roadmap gives, not as a current measurement. Rexx objects can override the operator methods, so this
phase must add an object check to that path and its shape is about to change.

## The gate

Exit criteria. Each one names the instrument, and where the instrument does not exist today the criterion
says who builds it.

1. **`CoreClasses.orx` translates, installs, and its prologue runs to `exit`, on both engines**, byte-identical
   to the oracle on all three descriptors. This is the phase's headline and it is now reachable: it needs
   the native entry-point registry (D37), the two `file_separator` natives, `StreamClasses.orx` and
   `PlatformObjects.orx` to *install*, and everything in the prologue table. **Executing
   `StreamClasses.orx`'s method bodies is Phase 7's**, per the roadmap's Phase 7 row.
2. **The `phase-5.txt` corpus subset passes byte for byte against the oracle, on both engines.** Two pieces
   of harness work are prerequisites and the plan owes a task for each:
   * `corpus.rs`'s comparison runs stderr through `normalize_stderr`. Criterion 2 says *byte for byte*, so
     the subset needs an unnormalised comparison mode, or the criterion is weaker than it reads. **Build
     the mode.**
   * `coverage.rs`'s `assert_program_has_only_routine_directives`, applied to the union of `SUBSET_FILES`,
     panics on any program carrying `::CLASS` or `::METHOD` -- which is every program this criterion needs.
     **Widen the walker.**

   The subset contains at minimum: an instance of each class the native layer creates; the wiring assertion
   of criterion 3 for each; one program per `MIXINCLASS` in `CoreClasses.orx` and one diamond that
   discriminates merge order from a chain walk; the method-frame traceback divergence; the two-argument
   `VALUE` route and the `say .LOCAL` route; the `::CONSTANT` expression form; and each of `directive_gap`'s
   four Phase 5 arms -- `::REQUIRES`, `::OPTIONS`, `::CLASS naming another class`, and `::ANNOTATE naming a
   target` if the plan's first task finds it in scope.
3. **Wiring is asserted against the oracle for every class in the native set**: `~class`, `~superClass`,
   `~superClasses`, `~isA` and `~metaClass`. `~superClasses` is not optional -- without it the assertion
   cannot see a missing mixin edge, which is the hazard it exists for.
4. **`>M>` and `>N>` are pinned by in-crate exact-stderr assertions**, expected bytes captured from the
   oracle rather than typed, and `trace_oracle.rs`'s `PREFIX_COVERAGE` rows updated. `ir_dual` is not
   evidence here.
5. **The security manager's interception points are in place (D12).** The roadmap's Phase 5 exit row
   requires this and the first draft dropped it to an open question without saying it was removing an
   inherited criterion. D12 assigns this phase the manager object, its installation path, and the hooks in
   dispatch and in `.local`/`.environment` lookup -- both of which are surfaces this phase builds, so
   omitting the hooks is exactly the retrofit D12 says costs touching every path twice.
6. **No guard axis moved beyond the floor**, under the rule stated in the bar, with a two-build sitting per
   task that lands code in `rexx-exec`, `rexx-core` or `rexx-classes`.
7. **Every class `Setup.cpp` creates is either in the native layer or in the deferral table with a reason**,
   where a reason names what would have to exist. Enforced as a test in `rexx-classes` over its own
   registry, so the table cannot drift from the code.
8. **The unsafe-block count and the list of crate roots carrying `deny` rather than `forbid` are reported**,
   per the roadmap's Global Constraints. Either growing without a Section 1 decision block fails the gate.
   This is inherited by every phase exit and the first draft omitted it.
9. **Cold start measured and recorded against the C++ build (D2).** This is a measurement, not a pass/fail:
   D2's rule is to build the image cache only if bootstrapping from source costs more than ~50 ms over the
   C++ startup, and the point of the criterion is that the number exists and D2 closes. **`hyperfine` is
   not installed on this machine**, so the instrument is `rexx-bench-suite`'s offset line, which measures
   the same thing through the same wrapper on both sides and is already the committed method; the plan
   records the substitution.

**The L2 rung is reported, not gated.** The roadmap's phase table names it, and the roadmap also records at
`2026-07-27-rust-rewrite.md:304` that the ooTest framework cannot start without `SysFileExists` and `.File`,
which are Phase 7's. A gate criterion that cannot be reached is worse than no criterion.

**Criterion 2 is what makes criterion 1 mean something.** A bootstrap that completes proves the file did
not raise; it does not prove a class responds. A registry of empty class objects satisfies criterion 1 and
fails criterion 2 on its first `~`.

## Risks

| risk | consequence | response |
|---|---|---|
| `CoreClasses.orx`'s method bodies open semantic gaps in bulk | criterion 2 stalls on a long tail | expected, and the roadmap says so. Triage per gap; do not start Phases 6-8 until this closes |
| the discovered native set is wired wrong | `~class` answers wrong, much later | criterion 3, **including `~superClasses`**, taken per class as each class lands, not at the end |
| a registered-but-unimplemented native is invoked and answers something | a silent wrong answer instead of a refusal | every unimplemented entry raises naming its owning phase; a corpus program per registered family calls one and pins the refusal |
| `Body` widens quietly within the current headroom | every program pays for a cold kind | the `size_of` assertion catches a widening past 80 and nothing below it; a new variant needs a recorded measurement regardless |
| the flattened dictionary is built as a chain walk | multiple inheritance resolves differently from the oracle | the discriminating diamond in `phase-5.txt`, from the first commit that defines a class |
| the crate boundary is in the wrong place | `rexx-exec` and `rexx-classes` fuse | the trait, and the signatures in the interface table; reaching past it is a spec amendment |
| a Phase 5 refactor costs a classic axis | nobody attributes it | the per-task two-build sitting under the stated floor rule |

## Decisions recorded here

* **D25.** The native/sourced line is **discovered by running `CoreClasses.orx`**, bounded by the
  install-time and prologue obligations enumerated above rather than left open. `Setup.cpp`'s list is a
  checklist, and the wiring of every class this phase builds is asserted against the oracle through
  `~class`, `~superClass`, **`~superClasses`**, `~isA` and `~metaClass`.
* **D26.** The three `.orx` files are **executed**, read from this repository's own tracked copy by a
  workspace-relative path, and never edited. No image is built; D2's threshold is measured at exit.
* **D27.** The object model lives in **new crates**, `rexx-classes` and `rexx-lib`. The boundary is a
  **trait**, not a one-way crate edge, because `~new` runs a Rexx `init` body. `resolve` takes a start
  scope and a per-object method table.
* **D28.** Message resolution is **dynamic, with no per-call-site cache**. **This amends D24**, whose text
  says the `Send` op caches the resolution. `ir.rs`'s `CallSite` is not extended to sends. Revisit only
  against a `dispatch` axis number, which does not exist yet.
* **D29.** The method dictionary is **flattened at class-definition time**, **retaining scope ordering**,
  and rebuilt through a cascade to subclasses. `BehaviourTable`'s chain walk is replaced, not extended. A
  class's identity is an `ObjRef`; `BehaviourId` survives as an index for the primitive fast path.
* **D30.** `::REQUIRES` lands **last in this phase**. Its reason is the `CallSite` table's append-only
  argument; the plan-cache half of the first draft's reason is withdrawn.
* **D31.** `::OPTIONS` and the `OPTIONS` instruction **stay in this phase**, sequenced independently of
  dispatch. The first draft moved them out and created an owner nobody could name; a phase is a unit of
  work, not a taxonomy.
* **D32.** `REPLY` and `GUARD` get their **run-time** legality check, not a translation-time one, exiting
  157 with the oracle's bytes. What a `REPLY` inside a Phase 5 method does is an open question below.
* **D33.** `.environment`, `.local`, `.context` and `.methods` are **real objects** from the first task,
  and the class registry, environment-symbol lookup and `VALUE`'s **two-argument** form all resolve
  through them. The three-argument empty-selector form is not this.
* **D34.** `ExprKind::List` is a **real Array** from the first commit that makes `~` work.
* **D35.** The performance guard is `alloc4c`, `arith`, `compound`, `emptyloop`, `strings` and `varlookup`,
  under the stated 1% floor rule, run per task that lands code in `rexx-exec`, `rexx-core` or
  `rexx-classes`. `dispatch`, `alloc` and `heapshape` are new measurements.
* **D36.** The gate is the nine criteria above. The L2 rung is reported, not gated.
* **D37.** Phase 5 builds a **native entry-point registry**. Every `LIBRARY REXX` name the three `.orx`
  files declare resolves at directive-install time, matching the oracle's eager bind; an entry this phase
  does not implement raises when **invoked**, naming its owning phase. `file_separator` and
  `file_path_separator` are implemented here, because `::CONSTANT` calls them at install time and
  `CoreClasses.orx` cannot reach `exit` without them. **This is a stated scope addition from Phase 7.**
* **D38.** `::CONSTANT` is **Phase 5's**, specifically its parenthesised expression form, which this crate
  accepts and never evaluates where the oracle evaluates it at install time. Measured; recorded as a KNOWN
  GAP before it is fixed. Criterion 1 needs it, because `StreamClasses.orx:548` is one.

## Open questions for the plan

* **What a `REPLY` or `GUARD` inside a Phase 5 method does.** D32 settles the outside-a-method case
  completely. Inside a method is reachable the moment `::METHOD` works, and this phase creates methods.
  Phase 6 owns the semantics; something has to happen in the meantime and this spec does not say what.
* **D24's three surviving forward constraints** -- selectors interned at compile time, a `SmallInt`
  behaviour arm, a receiver in the calling convention. Each is this phase's and none is designed here.
* **The security manager's interception shape (D12, Q14).** Criterion 5 requires the hooks. Their design is
  still not fixed, and it must be before the first dispatch call site is written, or Phase 7 invents a
  second mechanism.
* **`::ANNOTATE`.** Unmeasured. The plan's first task measures it and decides whether it is in criterion 2.
* **Whether the `createInstance()` order is load-bearing.** The order is quoted above; only four of its
  positions carry a stated reason.
* **What plays the oracle for a native method.** A collection primitive implemented in Rust has one, through
  a Rexx program. A method `CoreClasses.orx` defines has one by construction. The plan should say there is
  no third case, rather than leave it implied.
* **Where the `phase-5.txt` subset's programs come from.** Criterion 2 says "at minimum"; the rest is the
  plan's choice, and a subset chosen by the same person who wrote the implementation is a weak instrument.
