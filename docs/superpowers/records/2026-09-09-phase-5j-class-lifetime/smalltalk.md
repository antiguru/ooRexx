# Class lifetime in Smalltalk systems

Research report for the ooRexx-in-Rust collector design (Phase 5j).
Written 2026-09-09. No code in the ooRexx tree was changed.

## Sources actually read

Everything below that is attributed to a source was read from these artifacts,
not from memory. Line numbers are given where they help; they are line numbers
in the files as checked out at the revisions named here.

| Tag | What | Revision / provenance |
|---|---|---|
| **[Pharo]** | `pharo-project/pharo`, branch `Pharo12`, sparse checkout of `src/Kernel`, `src/Kernel-CodeModel`, `src/Collections-Weak`, `src/System-Support`, `src/Shift-ClassBuilder`, `src/System-Finalization` | commit `4689a46372929dd18b31925390202a40c8a3b7f4`, 2026-04-16 |
| **[PharoVM]** | `pharo-project/pharo-vm`, branch `pharo-12`, `smalltalksrc/VMMaker/SpurMemoryManager.class.st` | commit `8a9b398202957ff7c5c0f805ad72caf62605da6c`, 2026-09-04 |
| **[Squeak]** | Squeak trunk `Kernel-ct.1692.mcz` from `http://source.squeak.org/trunk/`, `snapshot/source.st`, CR→LF normalised (line numbers are therefore an artifact of my normalisation; class>>selector names are not) | package version `Kernel-ct.1692`, dated 31 August 2026 in its own `version` file |
| **[GST]** | `gnu-smalltalk/smalltalk`, `kernel/` | commit `768d3ef44c2bd485ea1e595edf00824d2f6a5384`, 2026-01-21 |
| **[Strongtalk]** | `talksmall/Strongtalk`, `vm/` | commit `39b336f8399230502535e7ac12c9c1814552e6da`, 2010-12-15 |
| **[Hayes97]** | Barry Hayes, *Ephemerons: A New Finalization Mechanism*, OOPSLA '97, pp. 176–183, ACM. Read as text extracted from the PDF at `https://static.aminer.org/pdf/PDF/000/522/273/ephemerons_a_new_finalization_mechanism.pdf` (8 pages) | — |
| **[Self91]** | Chambers, Ungar, Lee, *An Efficient Implementation of SELF, a Dynamically-Typed Object-Oriented Language Based on Prototypes*, Lisp and Symbolic Computation 4(3), 1991. `https://bibliography.selflanguage.org/_static/implementation.pdf` | — |
| **[SqWiki2176]** | Squeak wiki, "Cleaning up junk", `https://wiki.squeak.org/squeak/2176` | fetched 2026-09-09 |
| **[SBE]** | *Squeak by Example* §5.7 "Shared Variables", Black/Ducasse/Nierstrasz/Pollet, `https://eng.libretexts.org/Bookshelves/Computer_Science/Programming_Languages/Book:_Squeak_by_Example_(Black_Ducasse_Nierstrasz_and_Pollet)/05:_The_Smalltalk_Object_Model/5.07:_Shared_Variables` | fetched 2026-09-09 |

I could **not** obtain VisualWorks or VisualSmalltalk source. Every VisualWorks
claim below is sourced to [Hayes97] or is marked `UNSOURCED:`.

---

## 1. The direct answer

**In Pharo, Squeak and GNU Smalltalk, class collection is always explicit
unregistration first, ordinary GC second. The collector is never asked to
discover that a named class became unreachable, because a named class is
reachable by construction: the global namespace holds it strongly.**

Pharo says this in its own words. The class comment of `Class` [Pharo,
`src/Kernel-CodeModel/Class.class.st` lines 11–14]:

> When a class is removed from the system, we keep the instance in case it is
> still referenced. In that case, we declare the class as Obsolete.
> In order to know if a class is obsolete or not, we save the instances in a
> weak identity set called ObsoleteClasses. When the class is not referenced
> anymore, the GC will collect the class and remove it totally from the system.

So it is a *two-stage* answer, and the distinction matters for your design:

* **Stage 1, explicit.** `Class>>removeFromSystem:` severs, by hand, every
  bookkeeping edge the system itself owns. This is not a hint to the collector;
  it is a teardown.
* **Stage 2, GC.** After stage 1 the only remaining references are *stale user
  references* — an existing instance, a variable in a workspace, a literal in a
  method that was not recompiled. Whenever the last of those goes, the ordinary
  mark-sweep reclaims the class object like anything else. The class object is
  an ordinary heap object throughout; nothing about it is special to the
  collector except its class-table entry (§6).

There is no Smalltalk in the set I read where "the GC noticed nobody can name
this class any more" is the trigger. The trigger is always a programmer or a
tool calling a removal method.

**Why that is not a failure of nerve.** In Smalltalk the compiler binds a global
name to an `Association`/`LiteralVariable` object at *compile* time, and that
binding is stored as a literal in the compiled method
[Pharo, `CompiledMethod>>methodClass` line 506: `^self classBinding value`;
`CompiledMethod>>methodClass:` line 512 "set the class binding in the last
literal to aClass"]. So a class named anywhere in compiled code is strongly
reachable from that code, without the namespace's help. But `Smalltalk at:
#Foo` with a *computed* symbol is also legal, and there is no way for a
collector to see that coming. That is the reason a weak `SystemDictionary` is
not on the table anywhere: it would make dynamic name lookup unsound.
*(Inference — no source states this as the reason; but no system I read makes
the system dictionary weak, and all of them support runtime `at:` lookup.)*

**The one thing that translates directly** is §6: a *registry that does not
root*, swept of unmarked entries between marking and sweeping. Two production
mark-sweep collectors do exactly this, one of them for a class table.

---

## 2. What roots a class in practice

Taking the reader's list one item at a time. All "strong/weak" verdicts are
from source I read.

| Edge | Pharo 12 | Squeak trunk | Strong or weak |
|---|---|---|---|
| Global namespace name→class | `SystemDictionary`, `#superclass : 'IdentityDictionary'` [Pharo `src/System-Support/SystemDictionary.class.st` line ~28] | same class name | **strong** |
| Superclass → `subclasses` | `Class` instVar `subclasses`, an `Array` grown by `copyWith:` [Pharo `Class>>addSubclass:` line 161, `Class>>removeSubclass:` line 931] | identical shape [Squeak `Class>>addSubclass:`, `Class>>removeSubclass:`] | **strong** |
| Superclass → *obsolete* subclasses | `Behavior class` classVar `ObsoleteSubclasses`, a `WeakIdentityKeyDictionary` whose values are `WeakSet`s [Pharo `Behavior class>>initializeObsoleteSubclasses` line 46, `Behavior>>addObsoleteSubclass:` lines 59–68] | `WeakKeyToCollectionDictionary` whose values are `WeakArray`s [Squeak `Behavior class>>initializeObsoleteSubclasses`, `Behavior>>addObsoleteSubclass:`] | **weak, both sides** |
| Metaclass link (`Foo class`) | `Metaclass` instVar `thisClass`; `Class`'s class pointer is its metaclass | same | **strong, both directions** |
| Metaclass → its subclasses | `Metaclass>>addSubclass:` — `"Do nothing."` [Pharo line 47]; `Metaclass>>removeSubclass:` — `"Do nothing."` [line 317] | same, `"Do nothing."` [Squeak `Metaclass>>addSubclass:`] | **absent** — metaclasses keep no subclass list at all |
| Method dictionary → methods → class | `methodDict` instVar on `Behavior`; each `CompiledMethod`'s last literal is the class binding [Pharo `CompiledMethod>>methodClass:` line 512] | same | **strong, and a cycle** |
| Class variable pool | `Class` instVar `classPool` | same | **strong** |
| Shared pools | `Class` instVar `sharedPools` | same | **strong** |
| Package / category | `Class` instVar `packageTag`; `PackageTag` holds `classes` [Pharo `PackageTag>>removeClass:` line 175]; `PackageOrganizer>>removeClass:` line 266 | `category` instVar + `SystemOrganizer` | **strong, both directions** |
| Class comment source | `commentSourcePointer` — a SmallInteger index into `SourceFiles` [Pharo `Class>>comment` line 388: `SourceFiles sourceCodeAt: ptr`] | `sourcePointer` in the method trailer | **not a heap reference at all** |
| Per-class annotation/property table | `Behavior class` classVar `ClassProperties`, a `WeakIdentityKeyDictionary` [Pharo `Behavior class>>initializeClassProperties` line 40; `Behavior>>ensureProperties` line 657 returns `ClassProperties at: self ifAbsentPut: WeakKeyDictionary new`] | `Behavior` has no equivalent in the version I read | **weak keys** |
| Obsolete-class registry | `Class class` classVar `ObsoleteClasses := WeakIdentitySet new` [Pharo `Class class>>initialize` line 72; `Class>>isObsolete` line 705] | absent in Squeak | **weak** |

Two observations worth carrying into your design.

1. **Only the side tables were made weak, never the structural ones.** The
   weak collections are exactly: the obsolete-subclass memo, the class-property
   annotation table, and (Pharo) the obsolete-class set. The subclass list, the
   method dictionary, the class pool, the package membership and the namespace
   entry are all strong, in Pharo 12, Squeak trunk and GNU Smalltalk alike.
   *(Extent claim: I grepped for case-insensitive "weak" in Pharo's
   `Behavior.class.st`, `Class.class.st`, `ClassDescription.class.st`; in
   Squeak's `Kernel-ct.1692` source; and in GNU Smalltalk's `kernel/`. In GNU
   Smalltalk the only `weak` in `Behavior.st` is `allInstances` returning a
   `WeakArray`, unrelated.)*

2. **The metaclass keeps no subclass list.** `Metaclass>>addSubclass:` and
   `removeSubclass:` are both literally `"Do nothing."` in both Pharo and
   Squeak; `Metaclass>>obsoleteSubclasses` derives its answer from the instance
   side [Pharo line 278]. So the back-pointer problem exists on one side of the
   metaclass ladder only. If your metaclass link is symmetric, half your
   pinning is avoidable by simply not maintaining the metaclass-side list.

---

## 3. What class removal actually does

### Pharo

`Class>>removeFromSystem: logged` [Pharo `Class.class.st` lines 861–884], in
order:

1. `self release` — the hook whose comment in `Object>>release` [Pharo
   `src/Kernel/Object.class.st` line 1590] reads: *"Remove references to objects
   that may refer to the receiver. This message should be overridden by
   subclasses with any cycles, in which case the subclass should also include
   the expression super release."* That is a doctrinal statement: **breaking
   cycles is the object's job, by hand.**
2. `self unload`.
3. `self superclass addObsoleteSubclass: self` — the *weak* memo.
4. The global binding is re-classed in place:
   `Undeclared add: ((self environment associationAt: self name)
   primitiveChangeClassTo: UndeclaredVariable new)` [line 875]. Object identity
   of the binding is preserved so that literals in still-compiled methods keep
   working; only its class changes.
5. `self environment forgetClass: self` → `SystemDictionary>>forgetClass:`
   [`SystemDictionary.class.st` lines 259–265] which does
   `self organization removeClass: aClass` (package + extension back-pointers,
   via `PackageOrganizer>>removeClass:` line 266) and `self removeKey: aClass
   name ifAbsent: []`.
6. Deprecated aliases removed from the namespace.
7. `self obsolete`.

`Class>>obsolete` [lines 769–781]:

```smalltalk
self setName: 'AnObsolete' , self name.
Object class instSize + 1 to: self classSide instSize do:
    [:i | self instVarAt: i put: nil]. "Store nil over class instVars."
self classPool: nil.
self sharedPools: nil.
self hasClassSide ifTrue: [ self classSide obsolete].
ObsoleteClasses add: self.
super obsolete
```

and `super obsolete` reaches `ClassDescription>>obsolete` [line 766] which is
`self superclass removeSubclass: self.` then `super obsolete`, and
`Behavior>>obsolete` [line 1308] which is `"nothing to be done"`.

Note what this is: **the class-variable pool is nilled out, not made weak.**
`classPool: nil` is the direct answer to your "class → class-variable-pool →
class" cycle. Pharo does not reach for an ephemeron; it cuts the edge.

### Squeak

`Class>>removeFromSystem: logged` [Squeak `Class>>removeFromSystem:`]:
`self deactivate; unload.` then `superclass addObsoleteSubclass: self`, then
`self environment forgetClass: self logged: logged`, then `self obsolete`.

`Class>>obsolete` is character-for-character the same shape as Pharo's minus
the `ObsoleteClasses` line. `ClassDescription>>obsolete` additionally does
`self organization: nil` (drops the method-category organiser) and removes the
class from every trait it uses. `Behavior>>obsolete` differs from Pharo's — it
*does* act:

```smalltalk
obsolete
    "Invalidate and recycle local methods,
    e.g., zap the method dictionary if can be done safely."
    self canZapMethodDictionary
        ifTrue: [self methodDict: self emptyMethodDictionary].
```

with `Behavior>>canZapMethodDictionary ^true` and `Behavior class>>canZap...
^false` ("zapping the method dictionary of Behavior class or its subclasses
will cause the system to fail"). So Squeak also severs the
class↔method↔classBinding cycle explicitly, where it is safe to.

*(Inference: under a tracing collector this severing is not needed for
collectability — mark-sweep reclaims cycles. It is done to stop obsolete
instances running stale code, and to free the methods promptly. Do not read
"they cut the cycle" as evidence that a cycle is a problem for you.)*

### GNU Smalltalk

`Behavior>>addSubclass:` / `removeSubclass:` [GST `kernel/Behavior.st` lines
526–544] maintain `subClasses` as a strong `Array` with `copyWith:` /
`copyWithout:`, identical in spirit to the other two.

### Does the GC ever finish the job?

Yes, and there is a source that shows it being relied on.
`SystemNavigation>>obsoleteClasses` [Pharo `src/System-Support/
SystemNavigation.class.st` lines 352–366] runs `Smalltalk garbageCollect` and
*then* enumerates `Metaclass allInstances`, keeping those whose `soleInstance
isObsolete`. The GC pass is there precisely because obsolete classes that
became unreachable have gone by then; what is listed is the residue.

And the residue is a real, known, unsolved nuisance. The companion method
`SystemNavigation>>methodsReferencingObsoleteClasses` [line 332] exists to hunt
down who is still pointing at them, and its trailing comment suggests
`(Association allInstances select: [:a | (a value isKindOf: Class) and:
[a value isObsolete]])` — i.e. leftover namespace/pool associations are the
usual culprit. The Squeak wiki's advice for users who want the memory back is
the nuclear option [SqWiki2176]:

```smalltalk
Smalltalk obsoleteClassesDo: [ :c |
  c allInstancesDo: [ :i | i becomeForward: nil ].
  c becomeForward: nil ].
```

— *"Use the following at your own risk!"* That is: when explicit unregistration
is not enough, the fallback is not a smarter GC, it is `becomeForward:`, which
rewrites every pointer in the heap. That is worth knowing before you promise
GC-driven class collection to anybody.

There is one more artifact that should settle the question of whether these
systems trust their collector with registry liveness.
`SmalltalkImage>>cleanOutUndeclared` [Pharo `src/System-Support/
SmalltalkImage.class.st` lines 319–331] is a **hand-written mark-and-sweep over
one registry, in the image**: it collects all `Undeclared` keys, walks
`allBehaviorsDo:` → `methodsDo:` → `withAllNestedLiteralsDo:` to remove every
key that some method literal still references, and deletes the rest. It is
wired into `SmalltalkImage class>>cleanUp`. When Pharo needs to know whether a
registry entry is still wanted, it re-implements reachability by hand rather
than asking the collector.

---

## 4. The `subclasses` back-pointer

Your framing is right — a strong subclass list from a permanently-live root
class makes every subclass immortal. Here is what the systems actually did
about it.

**They did not make the list weak.** In Pharo 12, Squeak trunk and GNU
Smalltalk the `subclasses` slot is a plain strong `Array`. Instead:

1. **They made the list non-authoritative.** The `Class` class comment, present
   in both Pharo and Squeak in identical words:

   > The slot 'subclasses' is a redundant structure. It is never used during
   > execution, but is used by the development system to simplify or speed
   > certain operations.

   Nothing in the running semantics depends on it. Which means it is *safe* to
   drop entries from it — and that is what stage-1 removal does.

2. **Removal is what unlinks.** `ClassDescription>>obsolete` calls
   `self superclass removeSubclass: self`, so a removed class leaves its
   superclass's list at removal time, not at collection time.

3. **The only weak structure is the memo of the ones already removed.** A class
   that has been removed but may still have live instances is remembered in
   `Behavior class`'s `ObsoleteSubclasses`, a weak-keyed dictionary of weak sets
   [Pharo `Behavior>>addObsoleteSubclass:` lines 59–68: `obsoleteSubclasses :=
   self basicObsoleteSubclasses at: self ifAbsent: [ WeakSet new ].
   obsoleteSubclasses add: aClass`; Squeak's version builds a `WeakArray` by
   `copyWithout: nil` then `copyWith: aClass`]. Weak on both axes: the key
   (a superclass that itself became obsolete) and the element.

4. **There is periodic manual maintenance.** `Behavior class>>cleanUp` in Squeak
   is `self flushObsoleteSubclasses`, which is `ObsoleteSubclasses
   finalizeValues.` Pharo has `Behavior>>removeAllObsoleteSubclasses` [line
   1395] doing `self basicObsoleteSubclasses removeKey: self ifAbsent: []`.

**Documented history of changing this:** I found none. I did not find a commit,
issue, or comment in any of [Pharo], [Squeak], [GST] proposing or performing a
change of `subclasses` to a weak collection. The weak `ObsoleteSubclasses`
machinery carries a `'apb 7/12/2004'` author stamp in Squeak, so the
weak-obsolete-memo design is at least that old. `UNSOURCED:` I could not
determine whether a weak `subclasses` was ever debated on the Squeak or Pharo
mailing lists; I did not search list archives.

**Would a weak subclass list have helped?** No, and this is the load-bearing
point for your design. Making `subclasses` weak removes *one* of the strong
edges into a class. The namespace entry, the package's class list, and every
compiled method's class binding remain. Weakening one edge in a set of five
buys nothing. That is, I believe, why nobody did it.

---

## 5. Weak references, finalization, ephemerons

### The mechanism, from the source

[Hayes97] defines the problem exactly as you framed it. A property table with
weak keys and strong values fails when a value reaches its own key:

> The problem comes about when the "values" slot of some entry contains a
> direct or indirect reference to a key. An object might have itself as the
> property value... But when all other references to the object are deleted,
> the reference from the "value" slot lets the garbage collector find the key
> in phase one of the trace.

and making values weak fails too:

> If some object only exists as a value in a property table, the table must
> keep that entry, since the key that maps to that value may not have been
> collected, and so a query on that key is still possible.

The ephemeron:

> The first slot of an ephemeron is used to hold the key of a property, while
> the second slot holds the value, but the slots are neither "weak" nor
> "strong" in the previous sense.

Hayes' algorithm, in his words:

> An ephemeron-aware collector traces objects in three phases rather than two.
> The first phase traces up to ephemerons, but traces none of their fields. The
> second phase repeatedly traces ephemerons that can be classed as not
> maintaining unreachable properties. At the end of the second phase, the
> remaining ephemerons all represent unreachable properties. The third phase
> traces all remaining reachable objects (or tombstones the pointers to them).

Cost, from the paper:

> The portion of the algorithm presented that traces ephemerons adds running
> time O(nd) where n is the number of ephemerons and d is the length of the
> longest chain of ephemerons. In practice, d is small and the algorithm is
> linear in the number of ephemerons.

with a footnote giving a strictly-linear variant using one delay queue per
un-visited key.

Provenance, from the paper's own acknowledgements: *"Ephemerons were designed
by George Bosworth. They were first implemented for VisualSmalltalk by Roger
Thayer. Barry Hayes implemented them for VisualWorks."* And the legal notice:
*"Ephemerons have been a trade secret in some Digitalk and ParcPlace-Digitalk
products shipped more than one year ago."* So VisualWorks and VisualSmalltalk
shipped ephemerons before 1997. The multi-value extension (first field is the
key, all others are values) is noted as *"allowed in VisualWorks's and
VisualSmalltalk's implementation."*

### The Spur implementation, read line by line

This is the part worth translating, because Spur's collector is closer to yours
than Hayes' abstraction is. [PharoVM, `SpurMemoryManager.class.st`]

Object format 5 is the ephemeron; format 4 is the weak array
[`ephemeronFormat ^5` line 4979; `EphemeronLayout>>instanceSpecification ^5`
in Pharo; and the format table in `Behavior>>elementSize`'s comment: *"4 = weak
indexable objects with inst vars (WeakArray et al) / 5 = weak non-indexable
objects with inst vars (ephemerons) (Ephemeron)"*]. The key is slot 0
[`keyOfEphemeron:` line 8433: *"Answer the object the ephemeron guards. This is
its first element."*].

The whole full-GC mark phase, `markObjects:` [lines 9225–9248]:

```smalltalk
self initializeUnscannedEphemerons.
self initializeMarkStack.
self initializeWeaklingStack.
self initializeMournQueue.
marking := true.
self markAccessibleObjectsAndFireEphemerons.
self expungeDuplicateAndUnmarkedClasses: objectsShouldBeUnmarked...
self nilUnmarkedWeaklingSlots.
self freeUnscannedEphemerons.
...
marking := false
```

Inside the mark loop, `markAndShouldScan:` [~line 8915] decides whether to push
an object's fields:

```smalltalk
format = self weakArrayFormat ifTrue: "push weaklings on the weakling stack to scan later"
    [self push: objOop onObjStack: weaklingStack. ^false].
(format = self ephemeronFormat
 and: [self activeAndDeferredScan: objOop]) ifTrue: [^false].
^true
```

and `activeAndDeferredScan:` [line 1103] is *"Answer whether an ephemeron is
active (has an unmarked key) and was pushed on the unscanned ephemerons
stack."* — if the key is already marked or immediate, the ephemeron is scanned
as an ordinary object; otherwise it goes on a side stack and none of its fields
are traced.

The fixpoint is `markWeaklingsAndMarkAndFireEphemerons` [line 9274]:

```
repeat
  trace the strong (fixed) slots of every weakling discovered so far,
    repeating while new weaklings appear
  if the unscanned-ephemeron stack is empty -> done, return
  markInactiveEphemerons  "any whose key got marked since: scan-mark them, remove"
    if it found none, then every remaining ephemeron is genuinely active:
      fireAllUnscannedEphemerons   "queue each for #mourn"
      markAllUnscannedEphemerons   "then mark key and ephemeron, which may
                                    discover more ephemerons and weaklings"
```

The comment on `markInactiveEphemerons` states the invariant that forces the
loop:

> We cannot fire the ephemerons until all are found to be active since
> scan-marking an inactive ephemeron later in the set may render a
> previously-observed ephemeron (that remains in unscannedEphemerons) as
> inactive.

Overflow behaviour is graceful and worth copying:
`pushOnUnscannedEphemeronsStack:` [line 11398] grows by `realloc` and its
comment says *"Note that the ephemeron stack overflowing isn't a disaster; it
simply means treating the ephemeron as strong in this GC cycle."*

Weak-slot clearing is a separate, much cheaper pass after marking:
`nilUnmarkedWeaklingSlotsIn:` [line 9808] walks the indexed slots past
`numStrongSlotsOfWeakling:` and stores `nil` over any referent that is not
marked; if any were nilled the weakling is queued for finalization
[`nilUnmarkedWeaklingSlots` line 9786, `queueMourner:` line 11438].

### Ephemerons in the image

Pharo's `FinalizationRegistry` class comment [Pharo `src/System-Finalization/
FinalizationRegistry.class.st`] describes the intended use precisely:

> I am implemented internally as a collection of ephemerons, instances of
> `FinalizationRegistryEntry`, which are hidden to the user. Each
> `FinalizationRegistryEntry` will have the object of interest as key, the
> finalizer as value, and a back pointer to its container (myself). When the GC
> detects an object of interest is going to be collected, the
> `FinalizationRegistryEntry` is sent the message `#mourn`...

Note that this is a registry whose *entry* holds a back-pointer to the
registry, and whose value may reference the key. That is the shape ephemerons
exist for.

**But no Smalltalk I read uses an ephemeron to hold a class.** Pharo's
class-property table is a plain `WeakIdentityKeyDictionary`, not an ephemeron
collection [Pharo `Behavior class>>initializeClassProperties` line 40]. The
obsolete-subclass memo is a `WeakIdentityKeyDictionary` of `WeakSet`s. The
namespace is strong.

---

## 6. The simpler scheme, and two production precedents

You asked whether "clearing the registry entry when the class is unmarked,
before sweeping" gets most of the benefit. **Yes, and it is not a shortcut —
it is what two real mark-sweep collectors do, one of them for a class table.**

### Spur's class table

Spur gives every class a small integer index (its identity hash), stored in the
header of every instance. The table mapping index→class is `hiddenRootsObj`.
`markAndTraceHiddenRoots` [PharoVM lines 9007–9046] contains the exact decision
you are facing:

> If a class table page is weak we can mark and trace the hiddenRoots, which
> will not trace through class table pages because they are weak. But if class
> table pages are strong, **we must mark the pages and *not* trace them so that
> only classes reachable from the true roots will be marked, and unreachable
> classes will be left unmarked.**

and the code does the second:

```smalltalk
self setIsMarkedOf: hiddenRootsObj to: true.
self markAndTrace: classTableFirstPage.       "page 0: VM-known classes, strong"
1 to: numClassTablePages - 1 do:
    [:i| self setIsMarkedOf: (self fetchPointer: i ofObject: hiddenRootsObj) to: true]
```

The pages are kept alive; their *contents* are not traced. Classes get marked
only from real roots — including, crucially, from their own instances, via
`markAndTraceClassOf:` [line 8970]. That call sits on both branches of the main
scan loop `markLoopFrom:` [lines 9145–9225, calls at relative lines 33 and 56],
so every live non-immediate object marks its class.

Then, between marking and sweeping, `expungeDuplicateAndUnmarkedClasses:`
[lines 4995–5031] walks every table slot and does

```smalltalk
((expungeUnmarked and: [(self isMarkedOrPermanent: classOrNil) not])
   or: [(self rawHashBitsOf: classOrNil) ~= classIndex]) ifTrue:
    [self storePointerUnchecked: j ofObject: classTablePage withValue: nilObj. ...]
```

Note also page 0 is skipped — *"Avoid expunging the puns by not scanning the 0th
page"* — that is exactly your "built-in classes may keep a static lifetime".
Spur reserves the first class-table page for classes known to the VM and never
expunges it.

### Strongtalk's symbol table

[Strongtalk, `vm/memory/symbolTable.cpp` `symbolTable::follow_used_symbols`,
called from `MarkSweep::mark_sweep_phase1` at `vm/memory/markSweep.cpp` line
243, after all other roots have been followed]:

```cpp
if (e->is_symbol()) {
  if (e->get_symbol()->is_gc_marked())
    MarkSweep::follow_root((oop*) e);
  else
    e->clear(); // unreachable; clear entry
}
```

A name table that does not root its entries, cleared of unmarked entries during
the mark phase, in a mark-sweep collector. Strongtalk has weak arrays with a
two-phase register [`vm/oops/weakArrayKlass.hpp`: *"1. Transitively traverse all
object except the indexable part of weakArrays. Then a weakArray is encountered
it is registered. 2. Using the registered weakArrays continue the transitive
traverse."*] but **no ephemerons**: `grep -ril ephemeron strongtalk/vm` matches
zero files.

Strongtalk's *system dictionary* is a different story: `_systemDictionaryObj`
is enumerated by `Universe::roots_do` (I read the body: `vm/memory/universe.cpp`
lines 536–581, `f((oop*)&_systemDictionaryObj);` at line 569) and followed by
`MarkSweep::mark_sweep_phase1` via `Universe::oops_do`. So Strongtalk, like the
others, roots named globals strongly. It is the *symbol* table, not the
*global* table, that is weak.

### When the simple scheme is exactly as good as ephemerons, and when it is not

This is the analysis your reader needs, and it is mine, not a source's
(**inference**, though it follows directly from the algorithms quoted above):

* Your stated hazard — "class → class-variable-pool → class" — is a **cycle**,
  and a tracing mark-sweep collects unreachable cycles natively. A cycle only
  pins if something *outside* the cycle roots one member. So this is a *rooting*
  problem, not an ephemeron problem. Ephemerons would be the wrong tool.
* For a registry whose **value is the key** (name → class), "don't trace the
  registry; after marking, clear entries whose class is unmarked" produces
  **exactly** the outcome an ephemeron table would. Hayes' motivating
  degenerate case is *"An object might have itself as the property value"* — and
  in that case both schemes drop the entry iff the object is otherwise
  unreachable. Spur's class table is precisely this case.
* The simple scheme is **not** sufficient where the value is a *separate* object
  that must stay alive exactly as long as the key: a per-class annotation table,
  a class→metadata map. Not tracing the table would free live metadata; tracing
  it would pin dead classes. Tracing only the values of already-marked keys, and
  iterating because that may mark new keys, *is* the ephemeron algorithm — you
  cannot get it cheaper. So: if any of your class-keyed side tables holds
  something the class does not itself reference, that table (and only that
  table) needs ephemeron treatment.
* An intermediate answer that avoids ephemerons for such tables: make the side
  data a **field of the class object** rather than an entry in an external
  table. Then key-liveness and value-liveness coincide by construction and no
  weakness of any kind is needed. Pharo does this for the structural data
  (`classPool`, `sharedPools`, `packageTag` are all instance variables of
  `Class`) and uses external weak-keyed tables only for the non-structural
  extras (`ClassProperties`, `ObsoleteSubclasses`, `ObsoleteClasses`).
  *(Inference: I did not find a source stating that split as a rule; it is what
  the field lists and the class-variable declarations show.)*

---

## 7. `become:` and class mutation

When Pharo redefines a class it does **not** mutate the class object in place.
`ShiftClassInstaller>>migrateClassTo:` [Pharo `src/Shift-ClassBuilder/
ShiftClassInstaller.class.st` lines 218–256] builds a *new* class object, copies
class-side slots and class-variable values across, migrates the instances, and
then:

```smalltalk
{ self oldClass. builder oldMetaclass } , oldClassVariables asArray
    elementsForwardIdentityTo: { newClass. builder newMetaclass }, newClassVariables asArray.
```

— a one-way `become:` on the class, its metaclass, and *each class variable
binding*, all in one atomic `valueUninterruptably` block. Instance migration
[`migrateInstancesTo:` line 260] is the same move: `oldObjects
elementsForwardIdentityTo: newObjects copyHash: true`.

**What that implies about how many places hold a direct pointer to a class:
unknowably many, and the system does not try to enumerate them.** It delegates
the pointer rewrite to the VM. The only pointers it fixes up by name are the
method class-bindings [`updateBindings:of:` — `newClass methods do: [ :e | e
classBinding: aBinding ]`], because those are semantic, not just topological.

**Does this argue for or against ordinary object identity for classes?**
*Strongly for*, with one caveat. `become:` is only available for ordinary heap
objects; a class identity that lives in a reserved numeric handle range cannot
be `become:`d, so any scheme that needs wholesale pointer rewriting is closed to
you while classes are tagged handles. It also argues for it in the simpler
sense that the VM's `markAndTraceClassOf:` treats a class as an ordinary object
it can mark and push.

The caveat: **you may not need `become:` at all.** Smalltalk needs it because
redefining a class changes instance *shape*, so instances must be rebuilt and
the class object is rebuilt with them. If ooRexx's `::CLASS` / `define` /
`inherit` mutate the existing class object in place, you never rebuild, never
`become:`, and you avoid the one operation that is genuinely expensive on a
non-moving heap without forwarding pointers (Spur has lazy forwarders; without
them, `becomeForward:` means a full heap scan). **Confirm that before letting
the `become:` argument carry weight in your decision** — it is the strongest
Smalltalk argument for object identity, and it may simply not apply.

---

## 8. Self and Strongtalk

### Self: the case where it works, and why

Self has no classes; it has **maps**, an internal VM structure that plays the
class role. [Self91 §3]:

> We have invented maps as an implementation technique to efficiently represent
> members of a clone family. In our SELF object storage system, objects are
> represented by the values of their assignable slots, if any, and a pointer to
> the object's map; the map is shared by all members of the same clone family.
> ... From the implementation point of view, maps look much like classes, and
> achieve the same sorts of space savings for shared data. But maps are totally
> transparent at the SELF language level.

Maps are ordinary heap objects — they have their own map, the "map map", which
is its own map — and they are **collected**:

> All maps in new-space are linked together by their third words; after a
> scavenge the system traverses this list to finalize inaccessible maps.

and, from the discussion of compiled-code dependency lists:

> Links must be removed from their lists when a method is invalidated or a map
> is garbage-collected; lists are doubly-linked to speed these removals.

So Self *does* achieve GC-driven collection of class-like descriptors. The
reason it can is exactly the reason the others cannot: **a map has no name.**
Nothing looks a map up in a namespace; the only reference to a map is the
header word of its instances (plus the VM's own linked list, which is a
side-list traversed after marking, not a root). Self's descriptor lifetime
problem is your problem with the namespace deleted.

*(Inference: the transferable lesson is not "copy Self" — you have named
classes and cannot delete the namespace — but that the namespace is the whole
of the difficulty, and any part of your class metadata that is *not* named can
be collected on Self's terms, i.e. by making the instance→descriptor edge the
only strong one and sweeping the side list after marking.)*

### Strongtalk

Strongtalk does have classes, and treats them as ordinary `klassOop` heap
objects traversed by `MarkSweep`. Its globals live in `_systemDictionaryObj`, an
`objArrayOop` enumerated by `Universe::roots_do` and followed as a root
[`vm/memory/universe.cpp` `Universe::oops_do`, `vm/memory/universe.hpp` lines
75/119/158]. `Universe::add_global` / `remove_global_at` copy-grow and
copy-shrink that array, so global removal is again explicit. Strongtalk adds
nothing to the class-lifetime story beyond the weak symbol table of §6, which
is the useful part.

---

## 9. Translation to a precise, non-moving, non-generational, STW mark-sweep with no write barriers

Mechanism by mechanism. "Survives" means: implementable in your collector as
described, without adding a write barrier or a generation.

| Mechanism | Invariant that makes it sound | Cost | Survives translation? |
|---|---|---|---|
| **Strong namespace + explicit unregistration** (Pharo/Squeak/GST) | Removal is an operation, not an inference; the collector is never asked a question it cannot answer. | Zero GC cost. Every teardown step must be written and maintained by hand, and a missed step is a leak with no failing test. | **Yes, trivially.** This is your fallback and it is what every real system does. |
| **Non-rooting registry, swept of unmarked entries between mark and sweep** (Spur class table; Strongtalk symbol table) | Instances mark their own class, so any class with a live instance is marked before the sweep of the table. Entries are keyed by something the mark bit can be read from. | One extra linear pass over the registry per GC, between marking and sweeping. No extra marking work. No write barrier. | **Yes — this is the one to take.** It maps onto your design as: stop registering per-class variable pools as named globals; stop resolving class handles to `None`; make the class an arena object; keep the tagged handle as an *index*; do not trace the registry; clear unmarked entries in a pass after `mark` and before `sweep`. |
| **Reserved, never-swept region for built-ins** | Nothing user-visible can outlive the process anyway. | One branch in the sweep of the registry. | **Yes.** Spur does exactly this by skipping class-table page 0 (`"Avoid expunging the puns by not scanning the 0th page"`). Matches your "built-in classes may keep a static lifetime" decision directly. |
| **Weak arrays / weak-keyed dictionaries** | The weak slots hold nothing the program can observe becoming nil at a wrong time. | A weakling worklist accumulated during marking; one pass after marking to nil unmarked slots. No write barrier. | **Yes.** Spur's `nilUnmarkedWeaklingSlots` is ~20 lines and needs only a per-object "is weak" flag, a side stack, and a "strong prefix length" for the fixed slots. |
| **Ephemerons** | An ephemeron's value fields are traced iff its key is reachable otherwise. | See below. | **Yes, but you probably do not need them.** |
| **`become:` / `elementsForwardIdentityTo:`** | Identity is a heap address the VM may rewrite. | On a non-moving heap without forwarding pointers: a full heap scan per `become:`. Spur avoids this with lazy forwarders, which is a whole mechanism (`isForwarded:`, `followField:`, `fixFollowedField:...` appear throughout the mark code). | **Only at high cost.** Do not adopt it unless class redefinition genuinely requires rebuilding the class object. |
| **Hand-written reachability over a registry** (`cleanOutUndeclared`) | None; it is a heuristic sweep run at the user's request. | O(all methods × all literals) per invocation. | Works, but it is an admission of defeat. Listed because Pharo actually ships it. |

### Exactly what a mark-sweep must add to support ephemerons

From [PharoVM] and [Hayes97], the complete list for your collector:

1. **A per-object type discriminator for "ephemeron"**, and the convention that
   slot 0 is the key (Spur: `format = 5`, `keyOfEphemeron:` reads slot 0).
2. **A change in the mark loop's "should I scan this object's fields?"
   decision**: on encountering an unmarked ephemeron whose key is not yet
   marked, mark the ephemeron but push it on a side worklist and trace *none* of
   its fields (`markAndShouldScan:` / `activeAndDeferredScan:`). If the key is
   already marked, treat it as an ordinary object.
3. **A growable side worklist** allocated at GC start and freed at GC end
   (`initializeUnscannedEphemerons` / `freeUnscannedEphemerons`), with a
   documented graceful degradation on allocation failure: *treat the ephemeron
   as strong this cycle.*
4. **A fixpoint loop after the main mark completes**
   (`markWeaklingsAndMarkAndFireEphemerons`), which alternates:
   *(a)* rescan the worklist for entries whose key has since been marked, remove
   and fully trace those; *(b)* if none were found, every remaining entry is
   genuinely dead — fire them all, then trace them all (which may discover new
   ephemerons, so loop). The invariant forcing (a) before (b) is quoted in §5:
   scan-marking one entry can render another inactive, so you may not fire until
   a full pass finds no inactive entry.
5. **Interleaving with weak-slot processing**, if you have both: Spur's loop
   drains the weakling stack to a fixpoint *inside* the ephemeron loop, because
   ephemeron tracing discovers new weaklings and vice versa.
6. **A mourn/finalization queue** if firing is to be observable in the language.
   Your design may not need this at all.

None of that requires a write barrier, a generation, or object motion. What it
does require is that your mark loop is a *worklist* loop you can re-enter, which
it is. The cost is Hayes' O(n·d) with d the longest ephemeron chain, plus one
worklist allocation per GC.

**My recommendation:** implement the §6 non-rooting-registry-plus-sweep first.
It is a fraction of the code, it has two production precedents in mark-sweep
collectors, and for a `name → class` table it is *equivalent* to ephemerons, not
merely close. Reach for ephemerons only if you find a class-keyed side table
whose value is a distinct object that must live exactly as long as the class,
and only after checking whether that value can be made a field of the class
object instead.

---

## 10. The honest answer to the headline question

**For a language whose classes live in named namespaces, GC-driven class
collection is achievable for the *object*, but the *decision* is always
explicit.**

Precisely:

* No system I read lets the collector decide that a class in a live namespace
  has become garbage. Pharo, Squeak, GNU Smalltalk and Strongtalk all root the
  global name table strongly, and removal from it is a method call.
* All of them do let the collector reclaim the class object *after* removal —
  and Pharo's `Class` class comment says so as a design statement, not as an
  accident.
* The one system where a class-like descriptor is collected purely by
  reachability is Self, and it is precisely the system where the descriptor has
  no name.
* The one place the collector *does* decide, in a system with classes, is Spur's
  class table: an index→class map that deliberately does not trace, and is swept
  of unmarked entries between marking and sweeping. That is the mechanism worth
  copying, and it is the direct fix for your reason (1) — the reserved handle
  range that `resolve` returns `None` for.
* Your reason (2) — one named global root per class for its variable pool — has
  no counterpart in any Smalltalk. The class pool is a strong *field of the
  class* in Pharo, Squeak and GNU Smalltalk, reachable only through the class,
  and nilled at removal time. Making it a named global root is the whole of the
  immortality; it does not need weakening, it needs to stop being a root.

If you want the shortest possible statement for the decision record: *Smalltalk
does not GC-collect named classes; it explicitly unregisters them and lets GC
finish. The one part that generalises is the untraced-registry sweep, and your
class-variable-pool root has no precedent and should simply be removed.*

---

## 11. What I could not establish

* **VisualWorks class removal.** Not open source; I could not read
  `Class>>removeFromSystem` or the namespace implementation. Everything I say
  about VisualWorks comes from [Hayes97]'s acknowledgements and legal notice
  (that it implemented ephemerons, with a multi-value extension, before 1997).
  `UNSOURCED:` I have no evidence about whether VisualWorks namespaces are weak
  or how it handles obsolete classes.
* **VisualSmalltalk / Digitalk.** Same — only [Hayes97]'s attribution.
* **The Blue Book / Smalltalk-80 itself.** I did not read Goldberg & Robson. The
  claim that every class is named by a global variable in `Smalltalk`, an
  instance of `SystemDictionary`, is sourced to [SBE] §5.7 and to the Pharo and
  Squeak sources, not to the Blue Book. `UNSOURCED:` whether Smalltalk-80 v2
  shipped `Class>>removeFromSystem` at all.
* **Whether a weak `subclasses` collection was ever proposed.** I found no such
  change or discussion in the Pharo, Squeak or GNU Smalltalk sources. I did not
  search the squeak-dev or pharo-dev mailing-list archives, so absence here is
  weak evidence.
* **Whether Pharo's `Undeclared` entry retains the removed class.**
  `removeFromSystem:` does `primitiveChangeClassTo: UndeclaredVariable new` on
  the existing binding, and `UndeclaredVariable` inherits the `value` instance
  variable from `LiteralVariable`. I **infer** the class object is still in that
  `value` slot and therefore still reachable from `Undeclared` until
  `cleanOutUndeclared` runs — I did not find a `value: nil` and did not run an
  image to check. The trailing comment on
  `SystemNavigation>>methodsReferencingObsoleteClasses` suggesting
  `Association allInstances select: [:a | a value isObsolete]` is consistent
  with this but is not proof.
* **Any measured cost of ephemeron processing.** Neither [Hayes97] nor
  [PharoVM] gives a benchmark. The O(n·d) figure is Hayes' analysis, not a
  measurement, and I ran nothing.
* **Pharo/Squeak issue history on class-removal leaks.** A GitHub issue search
  on `pharo-project/pharo` for "obsolete class garbage" returned only
  performance issues (#6816 "Speedup-class-removal", #2369 "Huge performances
  regression on package removal"), which I did not open. I have no evidence
  about whether class-removal leaks are tracked as bugs.
