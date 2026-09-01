/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

use crate::ObjRef;
use crate::bytes::Bytes;
use rexx_num::{Form, Number};
use std::collections::HashMap;

/// Identifies the behaviour (class + method dictionary) an object responds to.
///
/// Behaviours themselves live in a side table, not in the heap, because they
/// are created during bootstrap and never collected.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct BehaviourId(pub u16);

impl BehaviourId {
    pub const STRING: BehaviourId = BehaviourId(0);
    pub const ARRAY: BehaviourId = BehaviourId(1);
    pub const OBJECT: BehaviourId = BehaviourId(2);
    pub const STEM: BehaviourId = BehaviourId(3);
}

/// Names one flattened method dictionary in the class graph `rexx-classes`
/// builds -- the oracle's `RexxBehaviour` object, which an object points at.
///
/// Distinct from [`BehaviourId`] above, which indexes [`crate::BehaviourTable`]
/// and is not what a [`Body::Instance`] dispatches against.
///
/// **Declared in this crate so that [`Body::Instance`] can store one**, which
/// is what makes an instance dispatch against the behaviour its class held at
/// construction: `~define` repoints the class at a fresh handle and leaves
/// existing holders on the old one, while `~inherit` rebuilds the dictionary
/// the old handle already names. `rexx-classes` depends on this crate, so the
/// type cannot live there and be named here.
///
/// Only `rexx-classes` mints one; every other holder copies a handle it was
/// given.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct BehaviourHandle(usize);

impl BehaviourHandle {
    pub const fn new(index: usize) -> BehaviourHandle {
        BehaviourHandle(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

/// A byte string that failed `Number::parse_bytes` (D15). One marker and no
/// cause, which is not a simplification paid for later: nothing observable
/// distinguishes one cause from another. A Rexx program that uses a
/// non-numeric value in arithmetic gets error 41.1, "Nonnumeric value
/// ("val") used in arithmetic operation", which substitutes the *value* and
/// never says why it failed to parse. There is no separate `from_utf8` step
/// able to fail on its own either -- a parse over bytes either yields a
/// number or does not.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct NotNumeric;

/// The payload of a heap object.
///
/// Every variant that can reach another object must be handled in
/// `Body::trace`. Adding a variant without extending `trace` is the one way
/// to reintroduce the C++ implementation's defect class, so `trace` matches
/// exhaustively and must never gain a `_ =>` arm.
#[derive(Clone, Debug)]
pub enum Body {
    /// A value whose identity is its bytes (D15). `num` is a tri-state cache
    /// of the one parse those bytes ever get: `None` is "not yet asked", and
    /// must keep meaning exactly that -- treating it as "definitely not a
    /// number" after some other value has been filled in answers wrongly for
    /// a value nobody has asked about yet. The two `Result` arms tell "is a
    /// number" from "is not" apart so a non-numeric string is not re-parsed
    /// on every comparison.
    ///
    /// The cache holds the exact parse and is never rounded to fit a later
    /// `DIGITS`. Rounding belongs to the operation reading it, which is what
    /// keeps the cache safe across a `NUMERIC` change. Measured: `x =
    /// '1.234567890123456789'` gives `1.2346` under `DIGITS 5` and the full
    /// nineteen-digit value under `DIGITS 20`, both read from the one stored
    /// parse. An implementation that "helpfully" rounds at fill time creates
    /// exactly the staleness this tri-state exists to avoid.
    ///
    /// `bytes` is a [`Bytes`] and not a `Vec<u8>`: a short string's bytes live
    /// in the slot itself, which is what `RexxString`'s trailing
    /// `char stringData[4]` buys the interpreter. See that type for where its
    /// capacity comes from.
    Text {
        bytes: Bytes,
        num: Option<Result<Box<Number>, NotNumeric>>,
    },
    /// A value whose identity is its number (D15). `created_digits` and
    /// `created_form` are the `NUMERIC DIGITS`/`NUMERIC FORM` in force when
    /// this value was produced, and `text` -- formatted under exactly those,
    /// never the settings in force when it is later read -- is fixed for the
    /// object's whole lifetime once filled in.
    Num {
        value: Number,
        created_digits: u32,
        created_form: Form,
        text: Option<Vec<u8>>,
    },
    /// A stem: its own name, an optional default, and its tails keyed by the
    /// tail's value verbatim (D15a). A tail entry present and mapped to
    /// `None` is a tombstone -- an explicitly dropped tail, which must not
    /// fall back to the default -- and is distinct from the key being
    /// absent, which does.
    ///
    /// `name` has to live on the object rather than being derived from
    /// wherever it is read, and the case that forces this is a stem with no
    /// default at all: `q.1 = 'x'` never bare-assigns `q.` itself, so the
    /// object it auto-vivifies has `default: None`. Aliasing it into a
    /// plain variable and reading that alias back still answers `Q.` --
    /// `q.1 = 'x' ; y = q. ; say y` prints `Q.`, not the tail value and not
    /// anything derivable from `y`'s own identity, because by the time `y`
    /// is read the reference site (`q.`) is gone from context and only the
    /// object's own `name` field still says "Q.". A stem that *does* have a
    /// default (`w. = 'wd'`) does not need this to make the same point as
    /// forcefully, since one could still argue the rendered text travelled
    /// with the value; the tails-only case is the one with nothing else to
    /// carry it.
    Stem {
        name: Box<[u8]>,
        default: Option<ObjRef>,
        tails: crate::NameMap<Vec<u8>, Option<ObjRef>>,
    },
    /// An array's slots, in index order, `None` for a slot that holds no
    /// object at all.
    ///
    /// **An empty slot and a slot holding `.nil` are different values**, and
    /// every method that walks the slots has to tell them apart: measured,
    /// `(1,,3)~items` is `2` while `(1,.nil,3)~items` is `3`, and both have
    /// `~size` `3`. `ArrayClass::getRexx` answers `.nil` for an empty slot,
    /// so the two are indistinguishable through `~at` alone, which is why
    /// the distinction has to live in the body rather than be reconstructed
    /// from what a read answers.
    Array(Vec<Option<ObjRef>>),
    /// A user-defined object: the class it belongs to, the behaviour it
    /// dispatches against, the name something gave it, and its instance
    /// variables, one pool per scope (D40).
    ///
    /// **`behaviour` is fixed at construction and every message resolves
    /// against it** (D58), which is where the oracle keeps it: its object
    /// points at a `RexxBehaviour` and reaches its class only through that
    /// behaviour's `owningClass`. So `~define`, `~defineMethods` and
    /// `~delete`, which replace the class's behaviour object with a copy
    /// (`ClassClass.cpp:860`-`:862`, `:531`-`:533`, `:962`-`:964`, each with
    /// the C++'s own comment saying so), leave this object on the old one,
    /// while `~inherit` and `~uninherit`, which rebuild the existing object
    /// in place (`:1361`, `:1413`, both into `updateSubClasses` at `:1036`),
    /// are seen here at once.
    ///
    /// `class` is what `~class` answers and what the default rendering
    /// names; the copy carries the owning class across, so it does not move
    /// when the behaviour does.
    ///
    /// `name` is `None` until `~objectName=` sets one, and that is not the
    /// same as holding the default rendering: `RexxObject::objectName`
    /// (`classes/ObjectClass.cpp:1695`) reads a set name first and otherwise
    /// **sends** `DEFAULTNAME`, so a class overriding that method decides what
    /// an unnamed instance answers. Measured, oracle rc 0: with
    /// `::METHOD defaultName` returning a counter, two renderings print two
    /// different values.
    ///
    /// `own` is `SETMETHOD`'s dictionary, searched ahead of `behaviour` and
    /// `None` for an object nothing has attached a method to. Boxed so that
    /// an object that never takes one costs a pointer -- see this module's
    /// own width assertion for what a wider `Body` costs.
    Instance {
        class: ObjRef,
        behaviour: BehaviourHandle,
        name: Option<Box<[u8]>>,
        pools: ScopePools,
        own: Option<Box<ObjectMethods>>,
    },
    /// An object the interpreter builds for itself rather than one a program
    /// constructs: `.environment`, `.local`, a package's `.methods` table and
    /// an activation's `.context`.
    ///
    /// **Boxed, and that is the decision the assertion below asks for.** The
    /// payload is a class handle, a rendered name and a map; inline it is
    /// wider than [`Body::Stem`] and would widen every `Slot` in the arena for
    /// a value kind a program allocates at most a handful of. One pointer
    /// costs the indirection only where one of these objects is actually read.
    Native(Box<NativeObject>),
    /// A reference that does not keep its target alive. Traces to nothing --
    /// that is the whole point -- and the collector rewrites the target to
    /// `ObjRef::NIL` once it dies.
    WeakRef(ObjRef),
}

/// Every variable pool one object holds: one per scope that has bound a name
/// in it, found by walking the list (`ObjectClass.cpp:2489`, whose
/// `objectVariables` is a linked list of `VariableDictionary`s chained by
/// `nextDictionary` and searched the same way).
///
/// **A pool is a full variable pool, not a table of scalars**, and that is
/// measured rather than assumed: with `s.` exposed in a class method,
/// `s.1`/`s.2`/`s.beta` assigned in one send and `do i over s.` run in
/// another, the oracle prints ` 1=one BETA=three 2=two` at rc 0. So a value
/// here is whatever a local variable can hold, a stem object with its own
/// tails included, and nothing about tails belongs to this type -- the stem
/// object carries them, exactly as it does in a slot. What holds one entry to
/// being the stem *object* rather than a copy of it is
/// `corpus/lang/expose_stem.rex`, where a second variable takes the exposed
/// stem and sees a later write through it.
///
/// **Addressed by scope and name at every access rather than by an index
/// taken once.** An index would be one number smaller per binding and would
/// silently address another object's pool if the two ever came apart; the
/// name is what the language actually keys on, and a pool holds few enough
/// names that walking them is what the C++ does too.
#[derive(Clone, Debug, Default)]
pub struct ScopePools {
    /// The scope's class identity, and the names it has bound. A name with no
    /// value is **absent** rather than present-and-empty: `DROP` removes the
    /// entry, and a reader cannot tell the two apart -- measured, a class
    /// method exposing `v`, assigning it, and dropping it leaves a later
    /// `expose v` reading the derived name `V`, which is what a never-bound
    /// name reads as.
    pools: Vec<(ObjRef, Vec<(Box<[u8]>, ObjRef)>)>,
}

impl ScopePools {
    pub fn new() -> ScopePools {
        ScopePools { pools: Vec::new() }
    }

    /// The value `name` holds in `scope`'s pool, or `None` when the pool does
    /// not hold it -- which is the uninitialised state and not an error.
    pub fn get(&self, scope: ObjRef, name: &[u8]) -> Option<ObjRef> {
        let pool = self.pool(scope)?;
        pool.iter()
            .find(|(bound, _)| **bound == *name)
            .map(|(_, value)| *value)
    }

    /// Binds `name` in `scope`'s pool, creating the pool and the entry if
    /// this is the first write to either.
    pub fn set(&mut self, scope: ObjRef, name: &[u8], value: ObjRef) {
        let pool = match self.pools.iter().position(|(s, _)| *s == scope) {
            Some(index) => &mut self.pools[index].1,
            None => {
                self.pools.push((scope, Vec::new()));
                &mut self.pools.last_mut().expect("the pool just pushed").1
            }
        };
        match pool.iter_mut().find(|(bound, _)| **bound == *name) {
            Some(entry) => entry.1 = value,
            None => pool.push((name.into(), value)),
        }
    }

    /// Returns `name` to the uninitialised state in `scope`'s pool, which is
    /// what `DROP` on an exposed variable does. A name the pool never held is
    /// a no-op, exactly as `DROP` on a never-assigned local is.
    pub fn clear(&mut self, scope: ObjRef, name: &[u8]) {
        let Some(index) = self.pools.iter().position(|(s, _)| *s == scope) else {
            return;
        };
        self.pools[index].1.retain(|(bound, _)| **bound != *name);
    }

    fn pool(&self, scope: ObjRef) -> Option<&Vec<(Box<[u8]>, ObjRef)>> {
        self.pools
            .iter()
            .find(|(s, _)| *s == scope)
            .map(|(_, pool)| pool)
    }

    /// Appends every object these pools reach: each scope's own identity and
    /// every value bound in it.
    ///
    /// **Called from [`Body::trace`]'s `Instance` arm and nowhere else**, so
    /// that the collector's reachability for an object's variables is stated
    /// beside the storage it walks rather than a match arm away from it. A
    /// scope identity is a class handle and names no arena slot today
    /// ([`crate::CLASS_SLOT_BASE`]), which is the same position
    /// [`Body::Native`]'s class handle is in and is traced for the same
    /// reason: the arm stays correct on the day a class object is allocated
    /// like anything else.
    fn trace(&self, out: &mut Vec<ObjRef>) {
        for (scope, pool) in &self.pools {
            out.push(*scope);
            out.extend(pool.iter().map(|(_, value)| *value));
        }
    }
}

/// Names one method body in the dictionaries `rexx-classes` builds.
///
/// **Declared in this crate for [`BehaviourHandle`]'s reason**: an
/// [`ObjectMethods`] holds one per name and lives inside [`Body::Instance`],
/// and `rexx-classes` depends on this crate. It re-exports this name, so
/// every other holder still spells it `rexx_classes::MethodId`.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct MethodId(pub u32);

/// One method attached to a single object: the body it runs and the scope
/// its `EXPOSE` binds in.
///
/// `scope` is `RexxObject::setMethod`'s `targetScope`
/// (`classes/ObjectClass.cpp:1834`, `:1863`): [`ObjRef::NIL`] for the
/// default `FLOAT` option, which is the oracle's own `TheNilObject`, and the
/// object's class for `OBJECT`. Since a pool is found by scope *within one
/// object*, `NIL` gives every `FLOAT` method on an object one shared pool,
/// separate from the class's.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ObjectMethod {
    pub method: MethodId,
    pub scope: ObjRef,
}

/// The methods `SETMETHOD` has attached to one object, searched ahead of the
/// class behaviour (`MethodDictionary::addInstanceMethod`'s `addFront`,
/// `behaviour/MethodDictionary.cpp:399`).
///
/// A name mapped to `None` is `setMethod`'s no-method form, which stores the
/// oracle's `.nil` and hides whatever the class answers -- measured, oracle
/// rc 159: after `self~setMethod('MM')`, `hasMethod('MM')` is `0` and the
/// send is 97.1 even though the class defines `MM`.
///
/// Keys are upper case, and a lookup upcases too, which is what
/// `MethodDictionary` does on both sides.
#[derive(Clone, Debug, Default)]
pub struct ObjectMethods {
    entries: Vec<(Box<[u8]>, Option<ObjectMethod>)>,
}

impl ObjectMethods {
    pub fn new() -> ObjectMethods {
        ObjectMethods {
            entries: Vec::new(),
        }
    }

    /// What this object answers for `name`: `None` when it has no entry of
    /// its own, `Some(None)` for a hidden name, `Some(Some(method))` for one
    /// it defines.
    pub fn get(&self, name: &[u8]) -> Option<Option<ObjectMethod>> {
        self.entries
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, entry)| *entry)
    }

    /// Adds or replaces the entry for `name`, which `addInstanceMethod`
    /// does in one step: it removes any method it added before under that
    /// name and puts the new one at the front.
    pub fn set(&mut self, name: &[u8], entry: Option<ObjectMethod>) {
        match self
            .entries
            .iter_mut()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
        {
            Some(slot) => slot.1 = entry,
            None => self.entries.push((name.to_ascii_uppercase().into(), entry)),
        }
    }

    /// Removes the entry for `name`.
    ///
    /// **A name this object never set is untouched**, which is what keeps
    /// `unsetMethod` off the class's dictionary:
    /// `MethodDictionary::removeInstanceMethod` removes from the main
    /// dictionary only when the instance dictionary held the name
    /// (`behaviour/MethodDictionary.cpp:365`-`:371`). Measured, oracle rc 0:
    /// `self~unsetMethod('MM')` for a class-defined `MM` leaves `o~mm`
    /// answering the class's.
    pub fn remove(&mut self, name: &[u8]) {
        self.entries
            .retain(|(key, _)| !key.eq_ignore_ascii_case(name));
    }

    /// Appends every object these entries reach, which is each defined
    /// method's scope -- [`ScopePools::trace`]'s position and its reason.
    fn trace(&self, out: &mut Vec<ObjRef>) {
        out.extend(
            self.entries
                .iter()
                .filter_map(|(_, entry)| *entry)
                .map(|ObjectMethod { scope, .. }| scope),
        );
    }
}

/// The payload of a [`Body::Native`]: what class the object answers to, the
/// text it renders as, and the name-to-value table it holds.
///
/// `rendered` is the answer to `~objectName`, stored rather than derived,
/// because the two objects this crate builds that carry one were given it by
/// a Rexx assignment (`CoreClasses.orx:55` and `:990`) and the default a class
/// id would produce is a different string.
///
/// `entries` is empty for an object that holds no table -- an activation's
/// context object is one -- rather than optional, because a reader asking for
/// a name it does not hold gets the same `None` either way.
///
/// `annotations` is the `StringTable` `~annotations` answers, which is
/// `Option` rather than empty because absent and empty are different answers:
/// the oracle's `getAnnotations()` creates the table on first ask and keeps
/// it, so a program that adds to one sees the addition through `~annotation`
/// afterwards, and a caller that has nowhere to keep the table must not hand
/// out one that silently forgets. The kinds of object that carry one are the
/// ones `memory/Setup.cpp` gives an `Annotations` method to.
///
/// `scope` is `MethodClass::scope`, the class a method object has been
/// installed on, and only a `Method` object ever carries one. It is
/// observable: `MethodClass::newScope` (`classes/MethodClass.cpp:183`) hands
/// back the same object when the scope is still unset and a copy when it is
/// not, so `~define`'s answer to `~method` afterwards is the very object it
/// was given or a different one depending on this field. Measured on the
/// oracle at rc 0 with `::method z` above `::class K`:
/// `m = .methods~z; .K~define("ZORK", m); say (m == .K~method("ZORK"))`
/// prints `1`, and the same pair with `.K2~method("M")` in place of
/// `.methods~z` prints `0`.
#[derive(Clone, Debug)]
pub struct NativeObject {
    class: ObjRef,
    rendered: Box<[u8]>,
    entries: HashMap<Box<[u8]>, ObjRef>,
    annotations: Option<ObjRef>,
    scope: Option<ObjRef>,
}

impl NativeObject {
    pub fn new(class: ObjRef, rendered: &[u8]) -> NativeObject {
        NativeObject {
            class,
            rendered: rendered.into(),
            entries: HashMap::new(),
            annotations: None,
            scope: None,
        }
    }

    /// The class this object answers to.
    pub fn class(&self) -> ObjRef {
        self.class
    }

    /// The bytes `SAY` prints for this object.
    pub fn rendered(&self) -> &[u8] {
        &self.rendered
    }

    /// `~objectName=`: replaces the answer to `~objectName`, and with it
    /// every rendering of this object, since `RexxObject::stringValue` is a
    /// `~objectName` send.
    pub fn set_rendered(&mut self, rendered: &[u8]) {
        self.rendered = rendered.into();
    }

    /// The value stored under `key`, which callers hold already uppercased --
    /// the oracle stores every environment entry under
    /// `getUpperGlobalName(name)` and looks one up by `className->upper()`, so
    /// the case folding belongs to the caller that produced the key and not to
    /// each lookup.
    pub fn entry(&self, key: &[u8]) -> Option<ObjRef> {
        self.entries.get(key).copied()
    }

    pub fn set_entry(&mut self, key: &[u8], value: ObjRef) {
        self.entries.insert(key.into(), value);
    }

    /// Every key this object holds, as owned copies.
    ///
    /// Owned and not borrowed because the one caller reads the whole table
    /// and then allocates against `Interp`, which it cannot do while a
    /// borrow of the heap is live. The keys come back in no particular
    /// order -- the storage is a `HashMap` -- so a caller whose next step
    /// allocates sorts them first.
    pub fn keys(&self) -> Vec<Box<[u8]>> {
        self.entries.keys().cloned().collect()
    }

    /// The `StringTable` this object's `~annotations` answers, or `None` for
    /// an object whose builder gave it none.
    pub fn annotations(&self) -> Option<ObjRef> {
        self.annotations
    }

    pub fn set_annotations(&mut self, annotations: ObjRef) {
        self.annotations = Some(annotations);
    }

    /// `MethodClass::getScope`: the class this method object is installed on,
    /// or `None` for one nothing has installed yet.
    pub fn scope(&self) -> Option<ObjRef> {
        self.scope
    }

    /// `MethodClass::setScope` (`classes/MethodClass.cpp:219`).
    pub fn set_scope(&mut self, scope: ObjRef) {
        self.scope = Some(scope);
    }
}

/// **A `Body` is every object in the heap, and `Slot` is what the arena holds
/// one of per object**, so a variant that widens either widens the footprint of
/// programs that never construct it. Measured at 80 and 96 across a change that
/// took `Number` from 32 bytes to 40 -- `Body::Num`'s payload had headroom
/// against `Body::Stem`'s, which is what set the width then and sets it now.
///
/// **This bound is where [`Bytes`]'s inline capacity comes from.** That
/// capacity is the largest one this assertion still holds at, so raising it is
/// not a tuning knob: the next byte trips this line, which is the point.
///
/// Upper bounds rather than equalities: the claim is that nothing widened, and
/// shrinking needs no decision. **Phase 5 adds variants and is expected to trip
/// this**, which is the point -- widening the arena for a cold value kind should
/// be a deliberate act with a boxed alternative weighed, not a side effect.
const _: () = assert!(size_of::<Body>() <= 80);

impl Body {
    /// Appends every object this one can reach.
    ///
    /// This single exhaustive match replaces the 148 hand-written `live()`
    /// implementations in the C++ tree. It has no wildcard arm on purpose:
    /// adding a `Body` variant must be a compile error here, not a runtime
    /// use-after-free.
    pub fn trace(&self, out: &mut Vec<ObjRef>) {
        match self {
            // Neither reaches an `ObjRef`: a `Number` and a byte string are
            // both plain data, never a handle into the heap.
            Body::Text { .. } => {}
            Body::Num { .. } => {}
            Body::Stem { default, tails, .. } => {
                out.extend(default.iter().copied());
                // A tombstone (`None`) reaches nothing, same as a weak
                // reference clearing to `.nil` -- it is present but dead.
                out.extend(tails.values().filter_map(|t| *t));
            }
            // An empty slot reaches nothing, the same as a stem's tombstone
            // above.
            Body::Array(items) => out.extend(items.iter().filter_map(|item| *item)),
            // Every scope's pool, walked by the storage's own type -- see
            // [`ScopePools::trace`] for why the walk lives there.
            // The class handle is traced for the reason the `Native` arm
            // below traces its own: it names no arena slot today
            // ([`crate::CLASS_SLOT_BASE`]), and the arm stays correct on the
            // day a class object is allocated like anything else.
            Body::Instance {
                class, pools, own, ..
            } => {
                out.push(*class);
                pools.trace(out);
                // A one-off method's scope, in the position the class handle
                // above is in and traced for the same reason.
                if let Some(own) = own {
                    own.trace(out);
                }
            }
            Body::Native(native) => {
                // The class handle travels with the values. It names no arena
                // slot today ([`crate::CLASS_SLOT_BASE`]), so the collector
                // drops it on the floor; tracing it anyway is what keeps this
                // arm correct on the day a class object is allocated like
                // anything else.
                out.push(native.class);
                out.extend(native.entries.values().copied());
                // A method object's scope, in the position the class handle
                // above is in and traced for the same reason.
                out.extend(native.scope);
                // Reachable from the object alone once a program holds the
                // table and drops every other handle on it.
                out.extend(native.annotations);
            }
            // Deliberately reaches nothing: a weak reference must not keep
            // its target alive.
            Body::WeakRef(_) => {}
        }
    }
}

#[derive(Clone, Debug)]
pub struct Object {
    pub behaviour: BehaviourId,
    pub body: Body,
    /// Set when the object defines an `UNINIT` method. Such an object is
    /// resurrected by the collector and reported through
    /// `CollectStats::pending_uninit` rather than swept, and is cleared
    /// through [`crate::Heap::clear_uninit_all`] once the caller reports the
    /// finalizer has run.
    ///
    /// **Set through [`crate::Heap::set_uninit`] and nowhere else**, which is
    /// what the crate-private write permission enforces: the collector reaches
    /// these objects through a list that method appends to rather than by
    /// walking the arena, so a flag raised directly would name an object the
    /// sweeper never asks about.
    pub(crate) has_uninit: bool,
    /// Set once the collector has reported this object through
    /// `CollectStats::pending_uninit`, so that a later collection resurrects
    /// it again without reporting it twice -- oracle's `setReadyForUninit`
    /// (`classes/ObjectClass.hpp`), read by `MemoryObject::runUninits`.
    ///
    /// Cleared with [`Self::has_uninit`], through the same methods.
    pub(crate) ready_for_uninit: bool,
}

impl Object {
    /// Whether this object defines an `UNINIT` method. See the field.
    pub fn has_uninit(&self) -> bool {
        self.has_uninit
    }
}
