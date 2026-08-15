# The three design questions, closed by reading the C++ and probing the oracle

Written 2026-08-15, after Moritz chose "close them before planning". To be folded into
`docs/superpowers/specs/2026-08-15-phase-5-object-model.md` as decisions once the identity
confirmation agent has finished reading that file.

Every C++ line cited was opened. Every oracle transcript was taken under the standard wrapper from a
fresh empty directory with three descriptors read separately.

## 1. What replaces `Body::Instance`

**An object's instance variables are a per-scope variable POOL, not a name-to-value map**, and the
pool is the same kind of thing an activation already has.

`RexxObject` carries `VariableDictionary *objectVariables` (`classes/ObjectClass.hpp:600`).
`RexxObject::getObjectVariables(RexxClass *scope)` (`classes/ObjectClass.cpp:2489`) walks a **linked
list** of dictionaries comparing `dictionary->isScope(scope)`, and on a miss creates a new dictionary
and pushes it at the head. So the chain is one dictionary per scope that has actually been touched,
created lazily, found by linear walk.

**A `VariableDictionary` is a full variable pool, and an instance variable can be a stem.** Measured:

```
::CLASS K
::METHOD fill        expose s. ; s.1 = "one" ; s.2 = "two"
::METHOD get         expose s. ; use arg i ; return s.i
::METHOD tailcount   expose s. ; n = 0 ; do t over s. ; n = n + 1 ; end ; return n

o = .K~new ; o~fill ; say o~get(2) ; say o~tailcount
oracle rc 0, stdout "two" then "2"
```

So an instance variable holds stems with tails, iterable by `DO OVER`. `VariableDictionary`'s own
iterator carries `currentStem` and `returnStemValue` for exactly that.

**Consequence for the plan.** `Body::Instance(Vec<(String, ObjRef)>)` is wrong twice over: it cannot
hold two same-named variables at different scopes, and it cannot hold a stem. The replacement is a
small association from scope to the variable-pool structure this interpreter already has -- an
activation's storage is `rexx_core::SlotFrame` plus the stem machinery in `stem.rs`. A `Vec` with a
linear walk matches the C++'s own linked list and is right for the one or two scopes a real object
has.

**`EXPOSE` is then not new machinery.** It binds names in the running method to the pool for that
method's scope, which is structurally the operation `PROCEDURE EXPOSE` already performs against a
caller's pool. That is the reason `EXPOSE` and the instance-variable representation are one task and
not two.

## 2. The `~define` / `~inherit` asymmetry, mechanism and all

Measured first, then read. One class, two mutations, opposite visibility for an instance that
already exists:

| mutation on `K` | instance created before it | instance created after |
|---|---|---|
| `~define("LATER", m)` | rc 159, 97.1 "does not understand" | works |
| `~inherit(.M)` | **works** | works |

**The mechanism is one line and it is commented.** `RexxClass::defineMethod`
(`classes/ClassClass.cpp:819`) does, before touching the dictionary:

```c
// make a copy of the instance behaviour so any previous objects
// aren't enhanced
setField(instanceBehaviour, (RexxBehaviour *)instanceBehaviour->copy());
```

then `instanceMethodDictionary->replaceMethod(...)` and `updateInstanceSubClasses()`.

`RexxClass::inherit` (`:1287`) does **no copy**: it validates, appends the mixin to `superClasses`,
calls `mixin_class->addSubClass(this)`, then `updateSubClasses()` -- and `updateSubClasses`
(`:1036`) clears and rebuilds the **existing** behaviour object in place.

**So an object holds a pointer to a behaviour object, and that is the whole rule.** `~define`
repoints the class at a fresh copy, leaving existing instances on the old one. `~inherit` mutates
the object they already point at.

**It maps onto the field we already have.** `rexx_core::Object` carries `behaviour: BehaviourId`.
`define` allocates a new table entry and repoints the class; `inherit` mutates entry N in place and
cascades. Instances keep whatever id they were created with. Nothing about the field changes; what
changes is `BehaviourTable`'s lookup, which chain-walks today and must be a flat dictionary with
scope ordering.

## 3. The class-behaviour side

A class object carries **two** behaviours: its own (`behaviour`, inherited from `RexxObject` -- the
metaclass side, what messages the class itself answers) and `instanceBehaviour` (what its instances
get).

The two update paths differ exactly along that line, which is visible in what each mutation calls:

* `updateInstanceSubClasses` (`:1071`) rebuilds **only** `instanceBehaviour`, and is what
  `defineMethod` calls.
* `updateSubClasses` (`:1036`) rebuilds `instanceBehaviour` **and** `behaviour` -- via
  `createInstanceBehaviour` then `createClassBehaviour` (`:1148` and its neighbour), in that order,
  with the comment that instance methods "may have an impact on metaclasses". It is what `inherit`
  calls.

**That is why `::class "Singleton" mixinclass class` (`CoreClasses.orx:3974`) reaches the metaclass
side at all**: it arrives through `~inherit`, which rebuilds both. A design that models only
`createInstanceBehaviour`, as the spec's flattening section did, cannot express it.

**Consequence for the plan.** A class in `rexx-classes` holds two behaviour ids, not one, and the
cascade has two shapes. `phase-5.txt` needs a witness for the class-behaviour side that a
diamond over instance methods does not provide.
