# Phase 5h — the mapped collections

Spec: `docs/superpowers/specs/2026-09-06-collections.md`. Runs after Phase 5g
(`2026-09-06-phase-5g-ordered-collections.md`), which owns the instrument, the scopes table, the
contents protocol, `Supplier`, and the store-plus-variable-pool mechanism this plan depends on.

Classes: `IdentityTable`, `Table`, `Set`, `Bag`, `Relation`, `Directory`, `StringTable`,
`Properties`, `Stem`.

**This plan's task boundaries are firm and its row counts are not.** The boundaries come from the
join of `Setup.cpp` and `CoreClasses.orx` against the documented method set, which is structure and
does not move. What moves is which rows are already right: 5g's Task 0(b) instrument re-probes every
row in scope with a real argument list, and until it has run, the count of rows each task below owns
is the `loud` count plus an unknown share of the `answers` ones. **Re-slice from that table before
starting**, and if a task's real size has changed by enough to matter, say so and split it — do not
carry a stale size through the phase.

**Every global constraint from 5g's plan applies here unchanged.** They are not restated; read that
file's *Global constraints* section as part of this one, including the two marked NEW.

BASE for Task 1 is the commit this plan lands in, or 5g's close, whichever is later.

---

## The structure, which is why the tasks are what they are

`Setup.cpp` builds all seven hash classes from one native behaviour by `InheritInstanceMethods`, and
`classes/support/HashContents.hpp` supplies four key-semantics variants underneath it. Every task
below is one of those variants plus the classes that use it:

* identity comparison, hashed by `getHashValue()` — `IdentityTable`;
* `EqualityHashContents`, comparing by `equalValue` and hashed by `hash()` — `Table`, `Set`;
* `MultiValueContents`, the equality one with `put` remapped to `addFront` — `Relation`, `Bag`;
* `StringHashContents`, comparing by `memCompare` and hashed by the string hash — `Directory`,
  `StringTable`, and `Properties` through `Directory`.

`Stem` is in none of them. It is a `MapCollection` by inheritance whose store is the language's
tails, it shares no entry point with the others, and `Body::Stem` already exists — so it is last and
separate (spec §7).

**`Relation` and `Bag` have identical native entry-point sets**, `Bag` adding only its two
`BagClass` overrides on top of `Relation`'s eight. They are one task.

**And four of these classes do not use `Collection`'s set operations at all.** The prolog at
`CoreClasses.orx:80-87` runs `.set~inheritInstanceMethods(.SetMixin)`,
`.bag~inheritInstanceMethods(.BagMixin)` and `.relation~/.bag~inheritInstanceMethods(.ManyItemMixin)`,
which puts `union xor intersection difference subSet putAll` on `Set`, `Bag` and `Relation` at the
**class's own scope**, on a mixin body — `SetMixin~union` copies the receiver and adds items where
`Collection~union` is single-valued. Those rows read `answers rc 163` today and are unmeasurable
until `put` works, so **the first thing Tasks 2 and 3 do after their store lands is run them**. A
task that finds them answering `Collection`'s semantics has found a real defect, not a passing row.

**`Properties` has no native entry point of its own**: every one of its rows resolves to
`Directory`'s, and its Rexx-level methods (`getProperty`, `setProperty`, `load`, `save`, the logical
and whole variants) are installed and already answering. What blocks it is 5g Task 6's subject, and
**it is a design branch to change knowingly rather than a gap to fill**: `native_directory_new`
(`dispatch.rs:7839`) gives a `Directory` subclass a plain instance *on purpose* so `expose` works,
and `a_directory_subclass_keeps_the_instance` (`dispatch.rs:11786`) pins the pair. `Properties` needs
an instance that is both at once. Run it before writing any code for it -- a `StringTable` subclass
already takes the other branch and round-trips at rc 0, so the mechanism exists and the question is
which convention `Directory` should be on.

---

## Task 1 — the object-keyed store, `IdentityTable` and `Table`

The phase's only new data structure, and the two classes that are the store with nothing on top.

The store holds `ObjRef` in **both** positions. **It is walked by the collector in the commit that
introduces it** (spec D101), with `collect_stress` as the witness — a key the collector cannot see is a
use-after-free that no small test produces, and a unit test that passes is not evidence here.

Then `HashCollection`'s shared surface over the 5g contents protocol:
`allIndexes allItems supplier index empty isEmpty items hasIndex hasItem remove
removeItem`, plus `at`/`[]`/`put`/`[]=`, whose `answers` verdicts are vacuous — `t['k'] = 'v'`
refuses at rc 120 today on both engines, measured 2026-09-06 at `3c6e60f16`.

**`makeArray` is written here as `allIndexes`, not as items** (spec D98). It is a virtual upstream,
and the first version of the 5g plan told Task 1 to write one shared body — which would answer
items. Measured on the oracle, `.Directory` with `k1`/`k2` gives `makeArray: k1 k2` against
`allItems: v1 v2`, where `Array` gives the opposite. Every class in this phase answers indexes
except `Stem`, which answers its tail array.

**`supplier` is 5g Task 1's** (spec D99): `.Supplier` itself is unimplemented, so no `supplier` row
in this phase can be witnessed until that task lands. Check it has before writing one.

**Two key protocols, one store, and the difference is not the obvious one.** `Table` compares by
`equalValue` and hashes by `hash()`, which is the three-way thing spec D96 sets out and not simply
`Object~hashCode`: a base-class object hashes internally, and only an object of a user class reaches
the `HASHCODE` body Phase 5f Task 4d landed. Getting that branch wrong in the cheap direction --
always sending `HASHCODE` -- lets a user's override change where a *string* key lands.
`IdentityTable` compares by reference identity **and still hashes with
`getHashValue()` rather than `identityHash`**; `HashContents.hpp`'s own comment says so and gives the
reason. An `IdentityTable` that hashes the handle is in deviation 4's neighbourhood and will
diverge on a key whose identity and value hashes differ. Write both, and make the witness a program
where the two classes must answer *differently* for the same insertions — two equal strings that are
not the same object.

If the 5g protocol does not carry this task's shared surface nearly for free, that is the protocol
being wrong, and 5g's Task 4 was supposed to have found it. Report which happened.

---

## Task 2 — `Set`

`EqualityHashContents` again, so no new store, plus the index-only rule and two overrides.

`IndexOnlyHashCollection::validateValueIndex` is the whole specification and it is already quoted in
the spec (D92): value required; index optional but, if given, must satisfy `isIndexEqual` against
the value or the send raises `Error_Incorrect_method_nomatch`; index then becomes the value.

**The `Bag`/`Set` `put` correction is not this task.** D92 puts it in the phase's first commit,
ahead of everything, because it is a live shipped divergence found by a survey rather than by a
gate — oracle rc 0 `1` against this crate's rc 168 `Error 88.901`, re-measured 2026-09-06 at
`3c6e60f16` on both engines. What this task owns is the rest of `Set`'s surface, on a store that by
then holds what `put` gave it.

`Set`'s two overrides route `hasItem` to the shared `hasIndex` body and `removeItem` to the shared
`remove` body, at arity 1 where the general classes take more. Read the arity from the scopes table,
not from the shared body's signature.

---

## Task 3 — `Relation` and `Bag`

`MultiValueContents`: the same equality store with `put` adding a second entry under an existing
index instead of replacing it. One task, because their entry-point sets are the same set.

`Relation`'s own surface — `allAt allIndex hasItem items removeAll removeItem supplier
uniqueIndexes` — is where the multi-value semantics become visible, and it is the half a
single-value store answers plausibly and wrongly. `items` takes an argument here where it takes none
elsewhere, `supplier` likewise, and `uniqueIndexes` exists only because indexes repeat.

**The witness is a relation with a repeated index**, and it must distinguish `items` from
`uniqueIndexes~items` and `allAt(i)` from `at(i)`. A witness built on one entry per index tests
`Table` with a different name on it.

`addFront` is in the name of the upstream method: the order entries come back in is defined for a
multi-value index, so the witness asserts it rather than sorting it away.

---

## Task 4 — the string-keyed classes, and `Properties`

`Directory`, `StringTable`, `Properties`.

**These three already have a store, and that is the risk rather than the relief.**
`NativeObject`'s `entries: HashMap<Box<[u8]>, ObjRef>` is what `.environment` and `.local` are built
on, its keys are held **already uppercased** by its callers for the reason its own doc gives, and
the `[] []= at put` rows for these classes read `answers` while `p['a'] = 'b'` on a `Properties`
refuses at rc 120. So the first question is not what to add but whether the existing store is the
one these classes should use, or whether it is the environment's and the collections get the
Task 1 store with a string key protocol on top. **Answer it in the report before writing the
surface** — a user `Directory` that silently uppercases its keys is a divergence that most probes
will not show.

Then `StringHashCollection`'s four — `entry hasEntry setEntry removeEntry` — which are the
string-key-only accessors, and `Directory`'s own two, `setMethod` and `unsetMethod`.

**`setMethod` is not a store operation.** It puts a *method* under a name so that looking the name
up runs it. That is a different mechanism from every other row in this phase and it is where a
directory stops being a map; if it does not fit the task, it is its own.

`Properties` should need no body at all (see above). Run it, then say so.

---

## Task 5 — `Stem`

Its store exists, its entry points are all its own, and its semantics are the language's: tails,
tombstones, the default value, and `unknown`.

`Body::Stem` already distinguishes a dropped tail from an absent one, which is the property the
collection surface must respect: `hasIndex` on a tombstoned tail, and what `allItems` does with a
stem that has a default, are the two places where a `MapCollection` reading of a stem and the
language's reading come apart. Measure both against the oracle before choosing.

`request` and `toDirectory` convert; `unknown` is the stem's message trap. None of these is a
collection primitive and none should be forced through the protocol — spec §7 says if `Stem` does
not fit, it gets its own task rather than a widened protocol, and this is that task.

---

## Task 6 — receiver overrides, the instrument, and close

Same shape as 5g's Task 7.

**(a) `RECEIVER_OVERRIDES`** for the classes in scope, now that they can hold something; refresh
`method-bodies.txt` and report the verdict movement that results.

**(b) Re-run 5g Task 0(b)'s instrument** and diff against its committed run. Every row this phase
claims must have moved *there*. A row that moved in `method-bodies.txt` and not in the instrument
has been given a signature, not a body.

**(c) The close report** enumerates what moved by diffing `method-bodies.txt` against BASE, not by
summing what the task reports claimed, and states what is left per class with an owner.

With this phase closed, `RexxQueue` is the only surveyed class still wholly loud, and it is Phase 7's
by D93.
