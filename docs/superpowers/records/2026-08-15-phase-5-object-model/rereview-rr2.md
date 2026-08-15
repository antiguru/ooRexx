# RR2 -- re-review of the Phase 5 spec, architecture lens

**Subject:** `docs/superpowers/specs/2026-08-15-phase-5-object-model.md`, read at HEAD `bd892ce77`.
**Lens:** is this the object model the interpreter should have. Not citations, not decomposability.
**Anchoring:** the five earlier review reports were not read.

Every oracle run below used the ground rules' wrapper verbatim, from
`…/scratchpad/rr2`, a directory I created empty, with absolute paths and stdout, stderr and
status read as three descriptors.

Severity: **High** = the design has to change before a plan is written against it. **Medium** = the
plan can proceed but will discover this and pay for it. **Low** = worth a sentence in the spec.

---

## High

### H1. `Body::Instance(Vec<(String, ObjRef)>)` cannot represent an ooRexx object's variables -- CONFIRMED

Instance variables in ooRexx are **scoped by defining class**, not by name. Two classes in one
hierarchy each get their own `vv` on the same object.

Probe `scope.rex`, oracle, rc 0, empty stderr:

```
b = .Sub~new ; b~setboth ; say "A sees:" b~geta ; say "B sees:" b~getb
::class Base
::method setbase / expose vv / vv = "base value"
::method geta    / expose vv / return vv
::class Sub subclass Base
::method setboth / expose vv / vv = "sub value" / self~setbase
::method getb    / expose vv / return vv
```

```
A sees: base value
B sees: sub value
```

Had the finding been wrong -- had the pool been one flat namespace -- both lines would read
`base value`, since `setbase` runs last.

The oracle's representation is `RexxObject::objectVariables`, a **chain of `VariableDictionary`,
one per scope**: `ObjectClass.cpp:2488` (`getObjectVariables(RexxClass *scope)`) walks the chain
for a dictionary whose `isScope(scope)` matches and creates one if absent;
`ObjectClass.cpp:2372` (`getObjectVariable(name)`, no scope) walks the whole chain and returns the
first match. Each dictionary is a hash table. `Vec<(String, ObjRef)>` is a single flat association
list with no scope key and a linear probe.

The spec's "Value representation" section names `Body`'s 80-byte bound and `ExprKind::List`, and
mentions `Body::Instance` **once**, as the thing a new kind is boxed behind. It never examines it.

Two things follow that the plan needs told:

* The 80-byte assertion is not the guard here and never will be. `Vec<(String, ObjRef)>` and a
  scope-keyed shape (`Vec<(ObjRef, HashMap<..>)>`, or a `Box` to one) are both 24 bytes, so the
  correct representation trips nothing and the wrong one trips nothing either. The spec's only
  value-representation guard is blind to its most consequential value-representation question.
* `String` keys are also out of step with the rest of the crate, which keys byte-wise
  (`Body::Stem`'s `tails: HashMap<Vec<u8>, Option<ObjRef>>`, `Interp::routines`'
  `HashMap<Box<[u8]>, _>`).

### H2. `~define` does not reach existing instances; `~inherit` does. D29 has no way to express that -- CONFIRMED

Probe `defcopy.rex`, oracle, rc 0, empty stderr:

```
c = .Object~subclass("C") ; old = c~new ; c~define("ZZ", "return 42") ; new = c~new
d = .Object~subclass("D") ; oldd = d~new ; d~inherit(.Mix)
```

```
old responds: 0
new responds: 1
old-after-inherit responds: 1
```

`RexxClass::defineMethod` (`ClassClass.cpp:819`) says it in a comment and does it in code: *"make a
copy of the instance behaviour so any previous objects aren't enhanced"*,
`setField(instanceBehaviour, instanceBehaviour->copy())`, and only then `updateInstanceSubClasses()`.
`RexxClass::updateSubClasses` (`:1036`) and `updateInstanceSubClasses` (`:1071`) instead
`clearMethodDictionary()` and rebuild **in place**, so an object already pointing at that behaviour
sees the change.

So a behaviour is a value with **identity and lifetime**: an object is bound to the behaviour that
was current when it was created, and `define` rebinds the class while `inherit` mutates what both
are bound to.

D29 says the dictionary is "flattened at class-definition time, retaining scope ordering, and
rebuilt through a cascade to subclasses", and that "`BehaviourId` survives as an index for the
primitive fast path". An `Object` carrying a `BehaviourId` into a `Vec<BehaviourEntry>` that
`define` rebuilds at the same index answers `1` on line 1 above where the oracle answers `0`. The
spec has no sentence about this and nothing in criterion 3's five wiring messages can see it --
`~class`, `~superClass`, `~superClasses`, `~isA` and `~metaClass` are all class-side and all
unchanged by it.

**This also strengthens the spec's own conclusion while retiring its argument.** The spec justifies
flattening with multiple inheritance: a `superclass`-pointer chain walk resolves a diamond
differently. True, and insufficient -- a walk over the *merge-ordered scope list*, which the spec
separately requires the design to retain for `FORWARD CLASS(SUPER)`, resolves diamonds identically
to a flat merge and would need no flat map at all. What actually forces a materialised,
identity-bearing behaviour per class-version is H2, because a lookup-time walk from the class would
show `ZZ` to `old`. The spec argues the right conclusion from the weaker of the two available
reasons, and the stronger one is the one the implementer needs in front of them.

Third fact the one D29 sentence flattens: the `::CLASS` directive install path is a **different
C++ function with different effects**. `ClassDirective::install` (`ClassDirective.cpp:237`) calls
`RexxClass::defineMethods` (plural, `ClassClass.cpp:496`), which does **not** copy the behaviour and
does **not** cascade; `~defineMethods` from Rexx (`defineMethodsRexx`, `:518`) and `~define`
(`:819`) both do. One sentence, three behaviours.

### H3. The send path the spec transcribes is not `methodLookup`, and the `resolve` signature it makes the amendment bar cannot express the rest -- CONFIRMED

The spec: *"The ordinary send is `behaviour->methodLookup(msgname)` --
`interpreter/classes/ObjectClass.cpp:866`, one hash lookup"*, and it makes
`resolve(receiver_behaviour, name, start_scope, per_object) -> MethodId` the interface whose
violation is *"a spec amendment, not a refactor"*.

`RexxObject::messageSend` at that exact line does, in order:

1. `behaviour->methodLookup(msgname)`;
2. if the method `isSpecial()`: `checkPrivate` / `checkPackage`, which inspect the **calling**
   activation, not the receiver;
3. if `isProtected()`: `processProtectedMethod`, which is where the security manager sits -- the
   thing criterion 5 requires this phase to build hooks for;
4. if lookup found nothing: `processUnknown` (`:1002`), which re-dispatches `UNKNOWN` with the
   message name and an array of the original arguments.

Step 2 is measured, not inferred. `sm.rex`, oracle, **rc 159**, stdout empty, stderr
`Error 97.2:  Object "an Object" cannot accept private message "SETMETHOD" from this context.` The
identical call moved inside a method of the receiver (`sm2.rex`) is **rc 0**. Resolution therefore
depends on state that lives on `rexx-exec`'s activation stack, which the spec's table assigns to
`rexx-exec` and the signature does not accept.

Step 4 is not hypothetical for this phase: `CoreClasses.orx:1443` is `::METHOD unknown unguarded`,
inside `Monitor`, one of the classes criterion 1 must install and criterion 2 must instantiate.

`-> MethodId` also has no arm for "resolved to `.nil`", which is how both `~define(name)` with one
argument and `~setMethod(name)` with one argument **hide** an inherited method
(`ClassClass.cpp:842`, `ObjectClass.cpp:1842`).

The spec replaced a prose table with signatures precisely so the boundary could be enforced. The
signature it chose is under-specified against the C++ function it was read from, so the amendment
it forbids is due on day one.

### H4. Two equal short strings are one `ObjRef` in this crate and two objects on the oracle; `.IdentityTable` is in the native set -- CONFIRMED

`handle.rs` gives `TAG_TEXT` to byte strings of `INLINE_TEXT` = 7 bytes or fewer, carried in the
handle with *"no heap object behind them"*. Two such values with the same bytes are bit-identical
`ObjRef`s.

Probe `ident.rex`, oracle, rc 0:

```
s1 = "abc" ; s2 = "ab" || "c"
t = .IdentityTable~new ; t[s1] = "first" ; t[s2] = "second"
u = .Table~new         ; u[s1] = "first" ; u[s2] = "second"
i1 = 5 ; i2 = 2 + 3
ti = .IdentityTable~new ; ti[i1] = "a" ; ti[i2] = "b"
```

```
IdentityTable items: 2      t[s1]: first    t[s2]: second
Table items: 1
IdentityTable int items: 1
```

and `ident2.rex` under `numeric digits 20` (needed -- at the default `DIGITS 9` the oracle's
15-digit addresses round together and the comparison lies):

```
3-byte  same idhash: 0
12-byte same idhash: 0
int     same idhash: 1
```

So: two `"abc"`s are **distinct** identities on the oracle at any length, while two `5`s are the
**same** identity. `SmallInt` matches the oracle for free; `TAG_TEXT` does not, and the divergence
switches on at 8 bytes, which reads as arbitrary to anyone who hits it.

`IdentityTable` is in the `createInstance()` list the spec quotes, so criterion 7 puts it in the
native set and criterion 2 asks for *"an instance of each class the native layer creates"*. The
natural such program is the one above. There is no divergence decision recorded anywhere in the
spec, and the alternatives are not cheap: give every short string a heap object (undoing the
encoding), or add a per-value identity that `Copy` inline text has nowhere to store, or accept and
record the divergence. The spec should pick one; today it has not noticed the question.

### H5. The class registry is a new GC root, and the spec's reason that it is not is a non-sequitur -- CONFIRMED

The spec: *"The collector's root set stays enumerable because class objects live in the arena like
everything else, which is D1's own criterion."*

Living in the arena is what makes a class object **collectable**. It is not what makes it
**rooted**. `RootSet::iter` (`rexx-core/src/roots.rs:445`) yields exactly `globals`, `temps` and
assigned `slots`, and nothing else; `add_global` has one production caller in the whole workspace
(`rexx-exec/src/lib.rs:2664`, `EXIT_VALUE_ROOT`) -- every other hit of `/bin/grep -rn add_global`
over `crates/` is a test or a bench. A class object reachable only from a `rexx-classes` registry is
swept.

Two structures the cascade needs and the spec never names:

* **A per-class subclass list.** `updateSubClasses` and `updateInstanceSubClasses` both iterate
  `getSubClasses()`. The oracle holds it **weakly**: `RexxClass::addSubClass` (`ClassClass.cpp:482`)
  wraps each subclass in a `WeakReference`, and `getSubClasses` (`:470`) returns
  `subClasses->weakReferenceArray()`, pruning dead entries on read. Strong links would make every
  class ever created immortal, since the parent is immortal. `Body::WeakRef(ObjRef)` holds exactly
  one reference and there is no list-of-weak-references variant. `/bin/grep -ci weak` on the spec
  answers 4; all four hits are `weaken`, `weaker`, `weak instrument` and the `WeakReference` class
  name inside the quoted `createInstance()` list. Nothing about subclass links.
* **A root for `.environment`, `.local`, `.methods`, `.context` and the registry.** D33 makes all
  four real objects from the first task and says nothing about who roots them.

This is the finding most likely to appear as an intermittent, since a missed root is invisible until
a collection lands. See M6 for the harness that would witness it and which the spec does not know it
is enabling.

### H6. `EXPOSE` has no design, and the existing variable machinery cannot serve it -- CONFIRMED

`instruction_owner` (`rexx-exec/src/lib.rs:1205`) puts `InstructionKind::Expose` in
`Some("Phase 5")`. `/bin/grep -ci expose` on the spec answers **0**. The word does not appear.

That is not a citation gap, it is a structural one. `RootSet::alias_slot` binds a slot to another
**slot position** (`SlotRef`, an index into `RootSet::slots`). An object variable lives inside a
heap object's `Body::Instance`. No `SlotRef` can name it. So the crate's entire plan-resolved slot
fast path -- the one `roots.rs:74` documents as serving a load *"variable lookup is 8.1%/32.2% of
runtime"* -- does not reach an exposed variable, and every read and write inside a method body goes
through a by-name lookup in a heap object instead.

The axis this lands on is named in the spec's own bar. `bench-programs/dispatch.rex` is:

```
c = .counter~new
do i = 1 to n            /* n = 5000000 */
    c~bump
end
::method bump / expose total / total = total + 1
```

Every iteration is one send plus two exposed-variable accesses. The spec makes `dispatch` a "new
measurement, not a regression" and declines a send cache for want of a `dispatch` number -- while the
thing that number will mostly measure is a variable path the spec never designed. A plan that reads
the first `dispatch` figure as evidence about dispatch will be reading evidence about `EXPOSE`.

---

## Medium

### M1. The trait is not on the send path, and the amendment bar it carries is unenforceable -- PLAUSIBLE

Trace an ordinary `x~foo(1)` through the spec's two-crate split:

1. `rexx-exec`, holding `&mut Interp`, evaluates the receiver and the arguments;
2. it calls `rexx-classes`' `resolve(...)`, an immutable read, getting a `MethodId` (`Copy`);
3. it looks the `MethodId` up in its own body table and invokes.

Nothing crosses the trait. The trait -- *"run this method id against this receiver"* -- is reached
only when `rexx-classes` itself has to run a body, which is `~new` calling `init` and the class-side
natives.

Now look at what `~new` has to do. `Interp` (`lib.rs:1515`) is a **private** struct owning `heap`,
`roots`, `activations`, `programs`, `plans`, `out` and the trace sink. `~new` must allocate into
that heap, root the result across the `init` call, and be able to raise. If `rexx-classes` reaches
all of that through the trait, the trait is a facade over `Interp`'s whole surface under a different
name. "Neither names the other's concrete types" then stays literally true and stops meaning
anything: **adding a method to a trait never names a concrete type**, so the spec's stated bar --
*"Reaching past this is a spec amendment, not a refactor"* -- has nothing to catch. The bar is on
naming, and the risk is coupling.

The interface table also does not carry the work the phase actually has. Its only `rexx-classes`
mutator is `define(class, name, MethodId)`. The prologue table three sections earlier requires
`~subclass`, `~mixinClass`, `~inherit`, `~defineClassMethod`, `~inheritInstanceMethods`, `~new` and
`~addClass`/`~addPublicClass` on Package. Under the table's own split ("`rexx-exec` owns method
bodies, and running them") each of those is a body in `rexx-exec` that mutates the graph in
`rexx-classes`, through entry points the table does not list. Two tables in one document, one
enumerating the requirement and one enumerating the interface, and they do not meet.

One more, from the C++: `ClassDirective::install` (`ClassDirective.cpp:230`) implements
`::CLASS ... INHERIT x` as `classObject->sendMessage(GlobalNames::INHERIT, mixin, result)` -- a real
message send. There is no "install without dispatch" ordering available; directive install re-enters
the send path.

### M2. `StreamClasses.orx` has its own prologue, and it is not in the spec's prologue table -- CONFIRMED

`call 'StreamClasses.orx' rexxPackage` runs the file **as a program**, so its own executable top
level runs too. Read at `interpreter/RexxClasses/StreamClasses.orx:39-49`, ahead of its first
directive at `:54`:

```
use arg rexxPackage
publicClasses = .context~package~publicClasses
do name over publicClasses
   class = publicClasses[name]
   .environment~put(class, name)
   rexxPackage~addPublicClass(name, class)
end
```

Criterion 1 says these two files must *"install"*, and the spec calls its `CoreClasses.orx` prologue
table *"the single densest statement of what the phase must build"*. There is a second prologue and
it is not in the table. Its contents happen to be a subset of the first's, which is luck rather than
design, and the table is what a task author will work from.

The same file also carries `::method charout abstract` (`:56`) and
`::CLASS 'OutputStream' public MIXINCLASS Object` (`:54`). `/bin/grep -ci abstract` on the spec
answers 0. The crate's parser already models it (`ast.rs:1474`, `:1512`, `:1542` carry
`abstract_`, and `directive.rs:588` sets it), so the parse exists and the install-time and
invoke-time semantics do not, and nothing in the spec assigns them.

### M3. The entry-point registry's own mitigation cannot live in `phase-5.txt` -- CONFIRMED by reading two of the spec's sentences together

Risks table: *"every unimplemented entry raises naming its owning phase; **a corpus program per
registered family calls one and pins the refusal**"*.

Criterion 2: *"The `phase-5.txt` corpus subset **passes byte for byte against the oracle**, on both
engines."*

A program that invokes `alarm_startTimer` gets a working timer from the oracle and this crate's
not-implemented condition. The two sentences cannot both be satisfied by the same program. The
refusal needs an in-crate assertion, or a derived table in the shape `corpus/keyword-exempt.txt`
already uses for loud messages -- not a corpus row whose pass condition is oracle agreement.

D37's *"confines the divergence to invocation"* is the right architectural call. Its stated witness
is not available in the place the spec puts it.

### M4. Adding `corpus/phase-5.txt` reddens four harness constants, not the two the spec names -- CONFIRMED

`SUBSET_FILES` is a separate `const` in `crates/rexx-exec/tests/corpus.rs:470`,
`coverage.rs:513`, `ir_dual.rs:1181` and `collect_stress.rs:137`, and each is asserted against a
fresh directory read (`phase_subset_files_on_disk`) by, respectively,
`the_differential_reads_every_phase_subset_file`, `the_union_reads_every_phase_subset_file`,
`the_dual_harness_reads_every_phase_subset_file` and
`the_stress_subset_reads_every_phase_subset_file`. Creating the file with none of them updated turns
all four red. The spec's criterion 2 owes a task for `corpus.rs` (unnormalised mode) and
`coverage.rs` (widen the walker) and does not mention the other two.

This is a *good* omission and should be written in rather than merely fixed. `collect_stress.rs`
running every Phase 5 program under forced collection is precisely the instrument that would witness
H5, and `ir_dual.rs` picking them up is what makes criteria 1 and 2's "on both engines" real work
rather than a phrase (see M8).

### M5. Declining the cache is settled; declining the *hook* for it is the expensive half -- PLAUSIBLE, addressed to the human partner as a challenge to the reasoning behind a settled call

D28 is not mine to overturn and I am not trying to. The reasoning is.

D28 disposes of two of D24's five forward constraints "by declining the cache", one of them
*invalidation on behaviour mutation*. That is not a cache feature. It is a version counter bumped
inside `updateSubClasses` and `updateInstanceSubClasses` -- one field, in two functions this phase
has to write regardless. Declining it now means that whoever adds a cache later must re-audit every
behaviour-mutating site instead of reading one number, which is the difference between a patch and a
rewrite. My brief asked what the phase would have to do to make adding a cache cheap later; this is
the whole answer, it costs a `u32`, and the spec declines it.

H2 is why identity alone will not substitute for the counter: `define` produces a **new** behaviour
(so a cache keyed on behaviour identity self-invalidates correctly) while `inherit` mutates one **in
place** (so the same cache is silently stale). Any future guard needs the version, not the pointer.

Second half. D28's revisit condition is *"Revisit only against a `dispatch` axis number, which does
not exist yet"*, and the bar says `dispatch` *"become[s] runnable during this phase"*. The phase
therefore creates its own revisit trigger, and nothing in the gate says what happens when it fires --
criterion 6 covers the six guard axes, and `dispatch` is explicitly not one of them. Either the
trigger should be dropped from D28's text or a criterion should read the number.

### M6. The spec's rejection of a chain walk does not rule out the shape that is observably identical and cheaper -- PLAUSIBLE

Granting H2 -- flattening is semantics -- "clear it, rebuild it, and cascade the rebuild to every
subclass" is still a transcription of an implementation, not a derivation from behaviour. A
dirty-flag variant is observably identical: `updateSubClasses` marks each subclass's behaviour
dirty and returns; a dirty behaviour rebuilds its flat map from the merge-ordered scope list before
its next lookup. Copy-on-define is untouched (the copy is what H2 needs, and it is independent of
when the map is filled), diamond merge order is untouched, and the cascade goes from
O(subclasses x total methods) to O(subclasses).

This matters in Rust more than in C++ for a reason the spec should state: a class is an `ObjRef`
into the arena, so a cascade is a graph walk through `heap.get()` with a `&mut` on the heap, over a
subclass list that must be pruned of dead weak entries as it goes (H5). The eager version does that
walk on **every** `~define`; the lazy version does it once per definition burst.

The spec's stated reason for rejecting alternatives is that `BehaviourTable`'s `superclass`-pointer
chain walk mis-resolves diamonds and *"cannot express a mixin at all"*. Both true. Neither touches
the lazy flattened variant, which is still flattened and still merge-ordered. The spec does not rule
this out; it does not consider it.

### M7. `~identityHash` is address-derived and varies per run -- CONFIRMED

Three consecutive oracle runs of the same program, first line each:

```
a -140371645989841
a -140040466371537
a -140560966386641
```

So no `phase-5.txt` program may print one, and any program that reaches one indirectly is
uncomparable. It is reached indirectly by the acceptance target itself:
`CoreClasses.orx:1305` and `:3848` are both
`return (self~identityHash - other~identityHash)~sign`, inside `compareTo`. Under the default
`NUMERIC DIGITS 9` that arithmetic rounds a 15-digit oracle address; this crate will produce values
at a completely different magnitude, where the same rounding lands differently. `/bin/grep -ci
identityhash` on the spec answers 0.

Whatever `~identityHash` answers is a decision this phase fixes forever -- it is the seed for
`IdentityTable`, `~hashCode` and every `compareTo` above. It should be in the spec.

### M8. "On both engines" makes the IR half load-bearing, and the spec declines to design it -- PLAUSIBLE

Criteria 1 and 2 both end *"on both engines"*. `ir/compile.rs` and `ir/drive.rs` must therefore gain
message sends, directive install, class definition and `FORWARD CLASS(SUPER)`, and
`ir_dual.rs` will pick up every `phase-5.txt` program automatically (M4).

D24's three surviving forward constraints -- selectors interned at compile time, a `SmallInt`
behaviour arm, a receiver in the calling convention -- **are** that work, and the spec files them
under "Open questions for the plan", with *"none is designed here"*. The headline criterion rests on
a body of work the spec explicitly declines to design, in the crate's second-largest module pair.

### M9. `~setMethod` / `~unsetMethod` on an individual object are in the signature and nowhere else -- CONFIRMED

`resolve`'s fourth parameter is `per_object`, so the spec knows they exist. `/bin/grep -ci setmethod`
on the spec answers 0, and nothing else in the document mentions per-object methods.

Measured, `sm2.rex`, oracle, rc 0:

```
o greet: hi
o class: The THING class
copy responds: 1
fresh responds: 0
o isA Thing: 1
```

So a per-object method survives `~copy`, and neither `~class` nor `~isA` moves. The oracle's
mechanism is `RexxObject::defineInstanceMethods` (`ObjectClass.cpp:2260`):
`setField(behaviour, behaviour->copy())` -- **one fresh behaviour per object that ever receives a
`~setMethod`**, created at run time, unbounded. That is directly against
`rexx-core/src/body.rs:19`'s own doc, *"Behaviours themselves live in a side table, not in the
heap, because they are created during bootstrap and never collected"*, and against `BehaviourId`
being a `u16`.

The spec's `per_object` table is the better answer, and it is only a parameter name. Where the table
lives is undesigned: `Object` (`body.rs:164`) has `behaviour`, `body` and `has_uninit` and no room,
and `Body::Instance` is the object's variables, not its methods. `~setMethod` is also privacy-gated
on the caller (H3), which is a third thing the signature does not carry.

---

## Low

### L1. `UNINIT` on a user class is a mechanism `rexx-core` already has and the spec never connects -- PLAUSIBLE

`RexxClass::defineMethod` sets `setHasUninitDefined()` when the name upcases to `UNINIT`
(`ClassClass.cpp:856`), and `checkUninit()` is called at the end of `defineMethods`,
`defineMethodsRexx`, `inheritInstanceMethods`, `updateSubClasses` and `updateInstanceSubClasses` --
i.e. the cascade D29 makes central re-derives it on every pass, and it must reach subclasses.

The crate already carries the receiving half: `Object::has_uninit`, and `Heap::collect`'s
resurrection into `CollectStats::pending_uninit`, with a comment citing the C++ ordering. Phase 5 is
the first phase that can reach it from Rexx source, since it is the first with `::METHOD`.
`/bin/grep -ci uninit` on the spec answers 0. This is not a missing feature so much as an existing
mechanism about to acquire its first real producer with nobody assigned to wire it.

### L2. Metaclasses are named and their construction path is not -- PLAUSIBLE

The spec mentions metaclasses in the hazard paragraph, asserts `.class~class == .class` (verified:
oracle answers `1`), and puts `~metaClass` in criterion 3. What it does not carry is that
`ClassDirective::install` (`ClassDirective.cpp:172-182`) resolves an explicit `METACLASS` name and
passes it into `subclass()`/`mixinClass()`, so `::CLASS x METACLASS y` is a live directive form with
an install-time failure mode (`Error_Execution_nometaclass`). Criterion 3 asserts metaclass links on
the native set; nothing covers a sourced class declaring one.

One related C++ detail worth a sentence, because it is a destructive side effect on a class the
caller did not name: `RexxClass::inheritInstanceMethods` (`ClassClass.cpp:558`), which the prologue
invokes as `.supplier~inheritInstanceMethods(.SupplierMixin)`, calls
`sourceMethods->setMethodScope(this)` -- it **rescopes the donor's own method dictionary**. It also
does not cascade to subclasses, unlike every other mutator in that file.

---

## What I searched for and did not find

Stated so a reader can judge the reach of the negative results.

* On the spec, case-insensitive `/bin/grep -c`: `EXPOSE`, `UNKNOWN`, `identityHash`, `setMethod`,
  `instance variable`, `private`, `protected`, `UNINIT`, `thread`, `abstract`, `hasMethod` all
  answer **0**; `weak` answers 4 and every hit is unrelated (H5); `copy` answers 1 and it is
  "this repository's own tracked copy". A site my terms cannot reach would be one that discusses
  these under a name I did not guess -- "object variable pool", "method visibility", "finaliser".
* I did **not** attack citation accuracy, gate vacuity, or plan decomposability. Where I re-ran a
  spec claim it was because the design rested on it: `~superClasses` sees prologue-installed mixin
  edges (oracle: `.String sc: The Object class,The Comparable class`;
  `.Array sc: The Object class,The OrderedCollection class`), and `.class~class == .class` answers
  `1`. Both hold.
* I did not run `CoreClasses.orx` end to end against either interpreter, and I did not build
  anything -- `rust/target/release/rexx-run` as it stands answers
  `rexx-exec: a message send is not implemented (Phase 5)` at rc 120 for `say "abc"~length`, which
  is the whole of this phase's starting position.
* Concurrency: I confirmed that object identity, `~identityHash` and per-object behaviours are
  fixed forever by this phase's choices, and that the spec addresses none of them. I did **not**
  investigate `~start`, `MessageClass` threading, or `GUARD`'s lock, all of which the spec correctly
  assigns to Phase 6.

## The one sentence for the human partner

Of the seventeen findings, five are the same defect wearing different clothes: **the spec designs
the class graph and does not design the object**. Instance variables (H1), object identity (H4),
per-object methods (M9), `EXPOSE` (H6) and behaviour lifetime (H2) are each load-bearing for
criteria 1 and 2, each already have a *wrong* representation committed in `rexx-core` today, and
none is examined by the value-representation section that exists for exactly this purpose.
