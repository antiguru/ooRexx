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

//! `ClassGraph`: the oracle's `RexxClass` cascade over `MethodDict`.
//!
//! A class is identified by an [`ObjRef`] (D29) and carries two behaviours
//! (D44): an instance-side one, bestowed on whatever `~new` produces, and a
//! class-side one, which the class *object itself* answers to. Three
//! operations rebuild them, each reproducing a distinct oracle mechanism:
//!
//! * [`ClassGraph::define`] -- `~define` / `DEFINE`. Copies the instance
//!   behaviour to a fresh [`BehaviourHandle`] before cascading, so an
//!   object created before the call keeps answering the old method set
//!   (D43).
//! * [`ClassGraph::inherit`] -- `~inherit`. Appends a mixin to the
//!   ancestor list and rebuilds both behaviours **in place**, cascading to
//!   every subclass -- no handle is ever replaced, so already-created
//!   instances see the change immediately.
//! * [`ClassGraph::inherit_instance_methods`] -- `inheritInstanceMethods`.
//!   Donates a class's own methods into another's, rewritten to the
//!   recipient's own scope, **without adding a superclass edge** and
//!   **without cascading to the recipient's own subclasses**. This is the
//!   one mechanism a class-graph assertion (`ancestors`/`~superClasses`)
//!   cannot see at all: `tests/behaviour_wiring.rs` is built around that
//!   fact.
//!
//! What this crate does not model: message dispatch, method bodies, and the
//! `SYNTAX` conditions themselves -- [`ClassGraph::inherit`] answers an
//! [`InheritRefusal`] naming the condition the oracle reports, and the caller
//! raises it, because the oracle's messages substitute a class's
//! `~defaultName` and carry a traceback frame that belongs to whoever sent
//! the message.
//! A metaclass's own instance dictionary merging into a class's class-side
//! behaviour (`RexxClass::createClassBehaviour`'s `metaClass->mergeInstanceBehaviour`
//! branch, D44) **is** modelled, in [`ClassGraph::cascade_build`]'s
//! `Side::Class` arm -- added by Task 3 ("class objects, the registry, and
//! the metaclass graph") together with the `metaclass` field
//! [`ClassGraph::define_class`] now takes, and exercised by
//! `tests/native_classes_wiring.rs`'s `mixinclass class` and `.class`-is-an-
//! instance-of-itself probes.

use crate::method_dict::{MethodDict, MethodId, MethodSlot};
use rexx_core::ObjRef;
use std::collections::{BTreeSet, HashMap};

/// Whether a class was declared with `SUBCLASS`/plain inheritance or with
/// `MIXINCLASS`. The only difference this makes is `base_class`: a
/// `Mixin`'s is inherited from the class named after `MIXINCLASS`
/// (`RexxClass::mixinClass`, `ClassClass.cpp:1514-1529`), a `Regular`
/// class's is itself.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ClassKind {
    Regular,
    Mixin,
}

/// Why [`ClassGraph::inherit`] refused to build the edge, each variant named
/// for the `SYNTAX` condition `RexxClass::inherit` reports in its place
/// (`ClassClass.cpp:1298`-`1332`).
///
/// The caller raises: this crate models no condition machinery, and the
/// oracle's messages substitute a class's `~defaultName`, which is
/// [`crate::ClassRegistry`]'s to render and not this graph's.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum InheritRefusal {
    /// `Error_Execution_mixinclass` (`:1299`), 98.942: the target is not a
    /// `MIXINCLASS`, so nothing may inherit it.
    NotAMixin,
    /// `Error_Execution_recursive_inherit` (`:1311`, `:1317`), 98.944: one of
    /// the two classes is already an ancestor of the other, in whichever
    /// direction.
    Recursive,
    /// `Error_Execution_baseclass` (`:1323`, `:1329`), 98.943: the inheriting
    /// class does not already have the mixin's base class in scope. The base
    /// class travels with the refusal because the oracle's message names it.
    BaseClass(ObjRef),
    /// `Error_Execution_uninherit` (`:1350`, `:1407`), 98.945: a class named
    /// as a position or as an `~uninherit` target is not one this class
    /// inherits. The class travels with the refusal because the oracle's
    /// message names it, and it is the *named* class rather than the mixin:
    /// measured, `.K~inherit(.M1, .M2)` reports `has not inherited class
    /// "The M2 class"`.
    NotInherited(ObjRef),
}

/// Which of a class's two behaviours (D44) an internal cascade operation
/// targets.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Side {
    Instance,
    Class,
}

/// One flattened behaviour, plus the monotonic version D29 asks for: "one
/// field, bumped by every cascade". Nothing in this crate reads it for any
/// purpose beyond that assertion -- it exists so a future per-call-site
/// cache (D28's revisit condition) has something cheap to check rather
/// than needing to be invented later against every mutating site.
struct Behaviour {
    dict: MethodDict,
    version: u64,
}

/// An index into [`ClassGraph`]'s behaviour storage. Two classes can never
/// share one: [`ClassGraph::define`] always allocates a fresh handle for
/// the class it is called on, and every other class keeps the handle it
/// was given at [`ClassGraph::define_class`] time for as long as it exists.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct BehaviourHandle(usize);

struct ClassDef {
    /// `Regular` or `Mixin` -- oracle's `isMixinClass`. `inherit`'s
    /// `Error_Execution_mixinclass` check reads this: only a `Mixin` may
    /// be inherited.
    kind: ClassKind,
    /// Cascade order: the explicit superclass first (if any), then every
    /// mixin `inherit` has folded in, in the order it was called --
    /// oracle's `superClasses`.
    superclasses: Vec<ObjRef>,
    /// Every class that must be rebuilt when this one's own dictionaries
    /// change: populated by both [`ClassGraph::define_class`] (the
    /// explicit superclass) and [`ClassGraph::inherit`] (a mixin), and
    /// walked identically either way -- oracle's `subClasses`, populated
    /// uniformly by `addSubClass` from both `subclass()` and `inherit()`.
    subclasses: Vec<ObjRef>,
    /// This class's own instance methods, always at its own scope --
    /// oracle's `instanceMethodDictionary`.
    own_instance_methods: MethodDict,
    /// This class's own class (static) methods -- oracle's
    /// `classMethodDictionary`.
    own_class_methods: MethodDict,
    instance_behaviour: BehaviourHandle,
    class_behaviour: BehaviourHandle,
    /// What `inherit`'s base-class compatibility check reads -- oracle's
    /// `baseClass` field.
    base_class: ObjRef,
    /// The class whose *instance* behaviour gets merged into this class's
    /// own class-behaviour by [`ClassGraph::cascade_build`] (D44,
    /// `RexxClass::createClassBehaviour`'s `metaClass->mergeInstanceBehaviour`
    /// branch, `ClassClass.cpp:1119-1127`) -- oracle's `metaClass` field.
    /// Every class built by `interpreter/memory/Setup.cpp` has `.Class` here,
    /// including `.Class` itself, self-referentially (measured:
    /// `.class~metaclass~id` is `"Class"`).
    ///
    /// **This is not what `~class` answers**, and [`ClassDef::owning_class`]
    /// is. `RexxClass::subclass` writes this field to the superclass at
    /// `:1590` and *then*, at `:1615`, hands `setOwningClass` the local
    /// `meta_class` it was given -- a different location, never reassigned by
    /// the write at `:1590`. See [`ClassDef::owning_class`] for exactly when
    /// the two end up holding different classes.
    metaclass: ObjRef,
    /// The class this class object's own behaviour belongs to -- oracle's
    /// `behaviour->owningClass`, set from the metaclass `subclass` was given
    /// or defaulted to (`ClassClass.cpp:1615`), and what `~class` answers for
    /// a class object.
    ///
    /// **Separate from [`ClassDef::metaclass`] because the oracle writes them
    /// from different values.** The exact condition, measured: **the two part
    /// iff the superclass is a metaclass and is not the named-or-inherited
    /// metaclass.** Where they part, this field holds the named-or-inherited
    /// metaclass and `metaclass` holds the superclass.
    ///
    /// ```text
    /// ::CLASS T  SUBCLASS MC METACLASS M1   ~metaClass MC     ~class M1      part
    /// ::CLASS T2 SUBCLASS MC                ~metaClass MC     ~class Class   part
    /// ::CLASS M3 SUBCLASS MC METACLASS MC   ~metaClass MC     ~class MC      same
    /// ::class MC MIXINCLASS Class           ~metaClass Class  ~class Class   same
    /// ::class Z  SUBCLASS Class             ~metaClass Class  ~class Class   same
    /// ::CLASS K  METACLASS M1               ~metaClass M1     ~class M1      same
    /// ```
    ///
    /// **Deriving from a metaclass is necessary and not sufficient**, which
    /// the `M3`, `MC` and `Z` rows are there to show: each derives from a
    /// metaclass and each coincides, because the superclass *is* the
    /// metaclass in play.
    ///
    /// `createClassBehaviour`'s merge reads [`ClassDef::metaclass`] and not
    /// this (`:1123`-`:1127`), so the merge follows the override.
    owning_class: ObjRef,
    /// This class may be named as another class's metaclass -- oracle's
    /// `META_CLASS` class flag, which `RexxClass::subclass` tests before it
    /// will build anything from the metaclass it was handed (`:1572`).
    ///
    /// **Deriving from a metaclass is what sets it** (`:1586`-`:1589`), so it
    /// travels down from `.Class` through `SUBCLASS` and `MIXINCLASS` alike,
    /// and [`ClassGraph::bootstrap_metaclass`] is what starts the chain.
    /// `::CLASS ... METACLASS` does *not* set it on the class being declared:
    /// measured, `.K~isMetaClass` is `0` for `::CLASS K METACLASS S` while
    /// `.S~isMetaClass` is `1` for `::CLASS S MIXINCLASS Class`.
    is_metaclass: bool,
    /// This class's instances need `UNINIT` run when they are collected --
    /// oracle's `HAS_UNINIT` class flag, which is what
    /// `completeNewObject` reads to register each new instance
    /// (`ClassClass.cpp:1892`).
    ///
    /// **What sets it, on either side, is an instance method named `UNINIT`
    /// becoming reachable from the class** -- by being added to it, which is
    /// `defineMethod` (`:854`), or by being inherited, which `checkUninit`
    /// (`:1217`) picks up from the **flattened** instance behaviour. So a
    /// class that inherits `UNINIT` and defines none has the flag. Measured
    /// on the oracle: `::CLASS P` with an instance `::METHOD uninit`,
    /// `::CLASS K SUBCLASS P`, and `.K~new` runs P's `uninit` for that
    /// instance. [`ClassGraph::define`] and [`ClassGraph::check_uninit`] are
    /// where this crate decides it.
    ///
    /// **A class-side `::METHOD uninit CLASS` does not set this**, measured:
    /// two `.K~new` instances and a class-side `uninit` print `uninit on K`
    /// once, not three times. That spelling reaches `hasUninitMethod` and
    /// registers the class **object** itself (`:1222`), which is object
    /// bookkeeping this crate does not model at all.
    has_uninit: bool,
    /// Some class this one inherits from carries `UNINIT` -- oracle's
    /// `PARENT_HAS_UNINIT`, set by each constructor that builds an
    /// inheritance edge: `subclass` (`:1634`), `mixinClass` (`:1525`) and
    /// `inherit` (`:1364`).
    parent_has_uninit: bool,
    /// `~new` on this class refuses -- oracle's `ABSTRACT` class flag, set by
    /// `RexxClass::makeAbstract` (`ClassClass.cpp:1754`) and read by
    /// `checkAbstract` (`:1741`) as the first step of `completeNewObject`.
    ///
    /// **Not inherited**, measured: with `::CLASS AB ABSTRACT`, both
    /// `::CLASS SUB SUBCLASS AB` and `.AB~subclass('KID')` construct at rc 0
    /// where `.AB~new` is 98.989.
    is_abstract: bool,
    /// This class may not be altered from Rexx -- oracle's `REXX_DEFINED`
    /// class flag. `RexxClass::liveGeneral` sets it on every class in the
    /// image under `PREPARINGIMAGE` (`ClassClass.cpp:136`-`:142`), which is
    /// every class the image holds: the ones `Setup.cpp` builds and the ones
    /// the interpreter's own Rexx-written library declares alike. A `::CLASS`
    /// a program declares does not carry it.
    ///
    /// **The five methods that read it are the five that mutate a class**:
    /// `defineMethod` (`:823`), `defineMethodsRexx` (`:522`), `deleteMethod`
    /// (`:955`), `inherit` (`:1290`) and `uninherit` (`:1382`), each raising
    /// 98.985 before it validates anything else. The check lives where the
    /// oracle puts it, in the method a program sends -- `rexx-exec`'s
    /// `dispatch::rexx_defined_lock`, which is open while the library
    /// bootstrap is running so that the library's own prologue can mutate the
    /// classes it has itself flagged.
    rexx_defined: bool,
}

/// The class graph and its behaviour storage.
///
/// Behaviour handles are never freed: an orphaned one (left behind by
/// [`ClassGraph::define`]) stays valid and frozen forever, which is
/// exactly the property D43's asymmetry depends on -- an object holding
/// that handle keeps answering to it undisturbed.
#[derive(Default)]
pub struct ClassGraph {
    classes: HashMap<ObjRef, ClassDef>,
    behaviours: Vec<Behaviour>,
    /// Oracle's uninit table restricted to class objects -- the entries
    /// `RexxClass::checkUninit`'s `requiresUninit()` (`ClassClass.cpp:1224`)
    /// makes, in the order it makes them.
    ///
    /// Insertion order, because a repeated `put` on an identity table replaces
    /// the value and leaves the entry where it is, so a class re-checked after
    /// a mutator keeps the position it first got.
    uninit_classes: Vec<ObjRef>,
}

impl ClassGraph {
    pub fn new() -> Self {
        Self::default()
    }

    fn alloc_behaviour(&mut self) -> BehaviourHandle {
        let handle = BehaviourHandle(self.behaviours.len());
        self.behaviours.push(Behaviour {
            dict: MethodDict::new(),
            version: 0,
        });
        handle
    }

    /// Register a new class and build its two initial behaviours by
    /// cascading from `superclass` -- oracle's `RexxClass::subclass`
    /// (`ClassClass.cpp:1562`). `superclass` is `None` only for a root
    /// class with no ancestor (a test's own stand-in for `.Object`).
    ///
    /// `metaclass` is oracle's `metaClass` field (D44): every class built by
    /// `Setup.cpp` passes `.Class`'s own id here, `.Class` included,
    /// self-referentially -- see [`ClassGraph::cascade_build`]'s metaclass
    /// merge branch, which is what makes that self-reference observable at
    /// all (`.class~hasmethod('SUBCLASS')` is `1`). A metaclass value that
    /// does not yet have its own entry in this graph is accepted: nothing
    /// dereferences it until a *later* class's class-behaviour cascade reads
    /// it, and a root class (`superclass: None`) never reads its own
    /// metaclass at all (the merge branch is skipped for it, matching
    /// oracle's `TheObjectClass != this` guard) -- which is exactly what
    /// bootstrapping `.Object` and `.Class`'s mutual reference requires: each
    /// names the other before both exist.
    ///
    /// **`UNINIT` propagates here for both kinds**, one arm each, because
    /// the oracle writes them as separate tests on separate functions:
    /// `subclass`'s (`ClassClass.cpp:1634`) reads the parent the new class is
    /// being derived from, and `mixinClass`'s (`:1525`) reads the class the
    /// `MIXINCLASS` target names. They ask the same question of the same
    /// object, and both are kept so that a build losing either one is a build
    /// something can catch.
    ///
    /// **Deriving from a metaclass overrides `metaclass`** (`:1586`-`:1591`):
    /// the new class becomes a metaclass itself and takes `superclass` as its
    /// own metaclass, whatever the caller passed. Measured: `::CLASS S
    /// MIXINCLASS Class METACLASS M1`, with `M1` a metaclass carrying an
    /// instance `who`, answers `.S~who` with 97.1 -- `S`'s class side is
    /// built from `.Class` and `M1` reaches it nowhere -- while the same
    /// `M1` under `::CLASS S SUBCLASS Object METACLASS M1` answers `from M1`.
    ///
    /// **The override moves one field and not the other.** `:1590` writes the
    /// `metaClass` field to the superclass; `:1615`, later, hands
    /// `setOwningClass` the local `meta_class`, which that write never
    /// touched. So `metaclass` here follows the override and
    /// [`ClassDef::owning_class`] keeps what the caller passed. See that
    /// field for exactly when the two end up different.
    pub fn define_class(
        &mut self,
        id: ObjRef,
        superclass: Option<ObjRef>,
        kind: ClassKind,
        metaclass: ObjRef,
    ) {
        let derived_from_metaclass = superclass.filter(|sup| self.classes[sup].is_metaclass);
        let owning_class = metaclass;
        let metaclass = derived_from_metaclass.unwrap_or(metaclass);
        let (base_class, parent_has_uninit) = match kind {
            ClassKind::Regular => (id, superclass.is_some_and(|sup| self.uninit_reaches(sup))),
            ClassKind::Mixin => {
                let target = superclass.expect("a mixin class names its MIXINCLASS target");
                (
                    self.classes[&target].base_class,
                    self.uninit_reaches(target),
                )
            }
        };
        let instance_behaviour = self.alloc_behaviour();
        let class_behaviour = self.alloc_behaviour();
        self.classes.insert(
            id,
            ClassDef {
                kind,
                superclasses: superclass.into_iter().collect(),
                subclasses: Vec::new(),
                own_instance_methods: MethodDict::new(),
                own_class_methods: MethodDict::new(),
                instance_behaviour,
                class_behaviour,
                base_class,
                metaclass,
                owning_class,
                is_metaclass: derived_from_metaclass.is_some(),
                has_uninit: false,
                parent_has_uninit,
                is_abstract: false,
                rexx_defined: false,
            },
        );
        if let Some(sup) = superclass {
            self.classes.get_mut(&sup).unwrap().subclasses.push(id);
        }
        self.rebuild_behaviour(id, Side::Instance);
        self.rebuild_behaviour(id, Side::Class);
    }

    /// Oracle's `hasUninitDefined() || parentHasUninitDefined()`, the exact
    /// pair every propagation site asks about the class it is deriving from
    /// (`ClassClass.cpp:1364`, `:1525`, `:1634`).
    fn uninit_reaches(&self, class: ObjRef) -> bool {
        let def = &self.classes[&class];
        def.has_uninit || def.parent_has_uninit
    }

    /// Whether `class`'s instances need `UNINIT` -- oracle's
    /// `hasUninitDefined`. See the field for what sets it.
    pub fn has_uninit(&self, class: ObjRef) -> bool {
        self.classes[&class].has_uninit
    }

    /// Oracle's `RexxClass::checkUninit` (`ClassClass.cpp:1210`-`:1226`), both
    /// halves of it: set [`Self::has_uninit`] when the class's **flattened**
    /// instance behaviour answers `UNINIT`, whether the class defines it or
    /// inherits it, and enter the class object itself in
    /// [`Self::uninit_classes`] when its **class** behaviour answers `UNINIT`
    /// -- `if (hasUninitMethod()) requiresUninit();` (`:1224`), which is where
    /// a `::METHOD uninit CLASS` lands.
    ///
    /// **Idempotent and one-way**, like the oracle's: it never clears the
    /// flag and never removes a table entry. `RexxObject::checkUninit`
    /// (`ObjectClass.cpp:2604`) is a different function that does clear, and
    /// it is about an object's own methods rather than a class's.
    pub fn check_uninit(&mut self, class: ObjRef) {
        let instance = self.behaviour_handle(class, Side::Instance);
        if self.behaviours[instance.0].dict.has_method("UNINIT") {
            self.classes.get_mut(&class).unwrap().has_uninit = true;
        }
        let class_side = self.behaviour_handle(class, Side::Class);
        if self.behaviours[class_side.0].dict.has_method("UNINIT")
            && !self.uninit_classes.contains(&class)
        {
            self.uninit_classes.push(class);
        }
    }

    /// Every class object whose own behaviour answers `UNINIT`, in the order
    /// [`Self::check_uninit`] entered them, taken out of the graph.
    pub fn take_uninit_classes(&mut self) -> Vec<ObjRef> {
        std::mem::take(&mut self.uninit_classes)
    }

    /// Recompute [`Self::parent_has_uninit`] from the class's current
    /// superclass list.
    ///
    /// **The oracle has no such function, and this one recomputes an answer
    /// its callers already have.** `RexxClass::subclass` propagates the
    /// flag inline (`ClassClass.cpp:1634`-`:1637`), reading the parent it is
    /// deriving from; [`ClassGraph::define_class`] asks `uninit_reaches` of
    /// that same parent, and `rexx-exec` calls this straight afterwards from
    /// both routes into that function -- a `::CLASS` directive's install and
    /// a `~subclass` or `~mixinClass` send -- with the superclass list still
    /// holding the one entry `define_class` read. The `INHERIT` sends that
    /// would push more run later, and [`ClassGraph::inherit`] propagates the
    /// flag itself at its own tail. So the two agree under every input, not
    /// only under the ones a corpus builds.
    ///
    /// **Nothing witnesses the call**: deleting it leaves the whole gated
    /// suite green, measured. It is kept because it stands where
    /// `ClassClass.cpp:1634` stands and because deleting it leaves this
    /// function and [`crate::ClassRegistry::refresh_parent_has_uninit`] with
    /// no caller at all.
    pub fn refresh_parent_has_uninit(&mut self, class: ObjRef) {
        let reached = self.classes[&class]
            .superclasses
            .clone()
            .into_iter()
            .any(|sup| self.uninit_reaches(sup));
        if reached {
            self.classes.get_mut(&class).unwrap().parent_has_uninit = true;
        }
    }

    /// Whether a class `class` inherits from defines `UNINIT` -- oracle's
    /// `parentHasUninitDefined`.
    pub fn parent_has_uninit(&self, class: ObjRef) -> bool {
        self.classes[&class].parent_has_uninit
    }

    /// Oracle's `RexxClass::makeAbstract` past its metaclass refusal, which
    /// the caller raises -- see [`ClassDef::is_abstract`].
    pub fn make_abstract(&mut self, class: ObjRef) {
        self.classes.get_mut(&class).unwrap().is_abstract = true;
    }

    /// Oracle's `RexxClass::isAbstract`, what `checkAbstract` reads.
    pub fn is_abstract(&self, class: ObjRef) -> bool {
        self.classes[&class].is_abstract
    }

    fn behaviour_handle(&self, class: ObjRef, side: Side) -> BehaviourHandle {
        let def = &self.classes[&class];
        match side {
            Side::Instance => def.instance_behaviour,
            Side::Class => def.class_behaviour,
        }
    }

    fn behaviour_has_scope(&self, class: ObjRef, side: Side, scope: ObjRef) -> bool {
        let handle = self.behaviour_handle(class, side);
        self.behaviours[handle.0].dict.has_scope(scope)
    }

    /// Oracle's `createInstanceBehaviour` / `createClassBehaviour`
    /// (`ClassClass.cpp:1098`, `:1148`), unified over which per-class
    /// dictionary side is folded in. Ancestors are processed
    /// furthest-inherited-first (`superclasses.iter().rev()`, matching the
    /// oracle's `for index = items() downto 1`), skipping a superclass
    /// already present -- the diamond guard -- then this class's own
    /// dictionary is folded in last. The most recently folded-in ancestor
    /// always wins a name conflict, which is the merge-order rule the
    /// mixin and diamond probes measure: the first-listed `INHERIT` mixin
    /// is folded in *last* among the mixins (closest to `class`'s own
    /// scope in cascade order), and an explicit `SUBCLASS` target is
    /// folded in last of all the ancestors, so it outranks every mixin.
    ///
    /// **`Side::Class` only** (D44, `createClassBehaviour:1116-1129`): before
    /// folding in this class's own class methods, and only once this class's
    /// scope is not yet present, its metaclass's *instance* behaviour is
    /// merged in wholesale (`MethodDict::merge`, not `merge_methods` --
    /// oracle's `mergeInstanceBehaviour` carries the metaclass's own scope
    /// history along, not just its methods) -- unless this class is the root
    /// (empty `superclasses`, standing in for oracle's `TheObjectClass ==
    /// this` guard: Object's class-behaviour never absorbs a metaclass).
    /// This is the mechanism that makes `.class~hasmethod('SUBCLASS')` true
    /// for every ordinary class (its metaclass is `.Class`, an instance
    /// method there) and, self-referentially, for `.Class` itself -- and the
    /// one `mixinclass class` (R6) depends on: it is the *only* path by
    /// which a class's class-behaviour ever gets `.Class` into its own scope
    /// list, which is what `inherit`'s base-class check for a `mixinclass
    /// class` mixin reads.
    fn cascade_build(
        classes: &HashMap<ObjRef, ClassDef>,
        behaviours: &[Behaviour],
        class: ObjRef,
        target: &mut MethodDict,
        side: Side,
    ) {
        let superclasses = classes[&class].superclasses.clone();
        for sup in superclasses.into_iter().rev() {
            if !target.has_scope(sup) {
                Self::cascade_build(classes, behaviours, sup, target, side);
            }
        }
        if !target.has_scope(class) {
            if side == Side::Class && !classes[&class].superclasses.is_empty() {
                let metaclass = classes[&class].metaclass;
                if !target.has_scope(metaclass) {
                    let meta_instance = classes[&metaclass].instance_behaviour;
                    target.merge(&behaviours[meta_instance.0].dict);
                }
            }
            let own = match side {
                Side::Instance => &classes[&class].own_instance_methods,
                Side::Class => &classes[&class].own_class_methods,
            };
            target.merge_methods(own);
            target.add_scope(class);
        }
    }

    /// Oracle's `instanceBehaviour->clearMethodDictionary()` followed by
    /// `createInstanceBehaviour(instanceBehaviour)` (or the class-side
    /// equivalent): clear the *existing* dictionary in place, then cascade
    /// into it, rather than building a separate dictionary and swapping it
    /// in. The handle's identity is unaffected either way; this is the
    /// literal oracle shape rather than an equivalent one.
    ///
    /// The dictionary being rebuilt is [`std::mem::take`]n out of
    /// `self.behaviours` before the cascade runs, rather than borrowed in
    /// place: `cascade_build`'s metaclass-merge branch needs read access to
    /// a *different* handle's dictionary (the metaclass's own instance
    /// behaviour), and that handle is always a distinct `Vec` slot from the
    /// one being rebuilt (a class's instance and class behaviours are always
    /// two separate `alloc_behaviour` calls, even when the metaclass being
    /// read is the class itself -- `.Class`'s own case), so this satisfies
    /// the borrow checker without changing what gets read or written.
    fn rebuild_behaviour(&mut self, class: ObjRef, side: Side) {
        let handle = self.behaviour_handle(class, side);
        let mut dict = std::mem::take(&mut self.behaviours[handle.0].dict);
        dict.clear();
        Self::cascade_build(&self.classes, &self.behaviours, class, &mut dict, side);
        let behaviour = &mut self.behaviours[handle.0];
        behaviour.dict = dict;
        behaviour.version += 1;
    }

    /// Oracle's `updateSubClasses` (`:1036`): rebuild both behaviours in
    /// place, instance side first ("the added methods may have an impact
    /// on metaclasses"), then [`check_uninit`](Self::check_uninit), then
    /// cascade to every subclass. This is what [`inherit`](Self::inherit)
    /// calls, and it never allocates a new handle -- every class it
    /// touches, direct or cascaded, keeps the id its instances were
    /// created with.
    ///
    /// **The `UNINIT` check is here rather than at the mutators** because
    /// the oracle's is (`:1052`, between `createClassBehaviour` and the
    /// subclass loop). An inherit can give a class a `UNINIT` it did not
    /// declare, and the cascade gives the same one to every subclass; both
    /// mutators that reach this function need that and neither says so
    /// itself.
    fn update_sub_classes(&mut self, class: ObjRef) {
        self.rebuild_behaviour(class, Side::Instance);
        self.rebuild_behaviour(class, Side::Class);
        self.check_uninit(class);
        let subclasses = self.classes[&class].subclasses.clone();
        for sub in subclasses {
            self.update_sub_classes(sub);
        }
    }

    /// Oracle's `updateInstanceSubClasses` (`:1071`): rebuild only the
    /// instance behaviour, in place, then cascade. This is what
    /// [`define`](Self::define) calls **after** repointing `class` itself
    /// at a fresh handle, so only the cascaded subclasses' behaviours are
    /// rebuilt in place -- the direct recipient's old handle is never
    /// touched again.
    fn update_instance_sub_classes(&mut self, class: ObjRef) {
        self.rebuild_behaviour(class, Side::Instance);
        let subclasses = self.classes[&class].subclasses.clone();
        for sub in subclasses {
            self.update_instance_sub_classes(sub);
        }
    }

    /// `~define`. Oracle's `defineMethod` (`ClassClass.cpp:819`) copies
    /// `instanceBehaviour` *before* mutating it -- its own comment reads
    /// "make a copy of the instance behaviour so any previous objects
    /// aren't enhanced" -- then cascades only the instance side. The copy
    /// is why an instance created before this call keeps answering the
    /// old method set: `class` is repointed at a brand-new
    /// [`BehaviourHandle`] and the old one is never written to again,
    /// while every subclass keeps its own existing handle and is rebuilt
    /// in place (D43).
    ///
    /// **A method named `UNINIT` marks the class** (`:852`-`:857`), which is
    /// what makes the flag [`Self::has_uninit`] answers a fact about the
    /// class rather than something a caller has to set by hand. The name is
    /// matched case-insensitively because the oracle compares against the
    /// upcased lookup name `defineMethod` was handed, and
    /// [`MethodDict::add_method`] takes the key in either spelling.
    pub fn define(&mut self, class: ObjRef, name: &str, method: MethodId) {
        let def = self.classes.get_mut(&class).expect("define: unknown class");
        def.own_instance_methods.replace_method(name, class, method);
        if name.eq_ignore_ascii_case("UNINIT") {
            def.has_uninit = true;
        }
        self.copy_instance_behaviour(class);
        self.update_instance_sub_classes(class);
    }

    /// `~define` with the method argument omitted, and `Setup.cpp`'s
    /// `HideMethod`: a `.nil` tombstone under `name`, which the oracle's own
    /// comment calls "hiding this method definition"
    /// (`ClassClass.cpp:836`-`:843`).
    ///
    /// The name stops resolving on this class and on every subclass, and
    /// `~method` answers `The NIL object` for it rather than raising --
    /// [`MethodSlot::Hidden`] carries which reader sees which.
    pub fn hide(&mut self, class: ObjRef, name: &str) {
        self.classes
            .get_mut(&class)
            .expect("hide: unknown class")
            .own_instance_methods
            .hide_method(name);
        self.copy_instance_behaviour(class);
        self.update_instance_sub_classes(class);
    }

    /// `~delete`, and `Setup.cpp`'s `RemoveMethod`: take `name` out of
    /// `class`'s own instance dictionary -- `RexxClass::deleteMethod`
    /// (`ClassClass.cpp:952`). Answers whether there was anything to remove,
    /// which is what decides whether the subclasses are told: the oracle
    /// copies the behaviour unconditionally and calls
    /// `updateInstanceSubClasses` only inside the `if` (`:966`-`:971`).
    ///
    /// A name this class does not define is not an error -- measured on the
    /// oracle, `.K~delete("ZZZ")` for a `::class K` is rc 0 with nothing on
    /// either descriptor.
    pub fn delete(&mut self, class: ObjRef, name: &str) -> bool {
        let removed = self
            .classes
            .get_mut(&class)
            .expect("delete: unknown class")
            .own_instance_methods
            .remove_method(name);
        self.copy_instance_behaviour(class);
        if removed {
            self.update_instance_sub_classes(class);
        } else {
            // The oracle's copy carries the old contents across; this one is
            // built empty, so the class's own behaviour is rebuilt into it
            // either way and only the walk over the subclasses is conditional.
            self.rebuild_behaviour(class, Side::Instance);
        }
        removed
    }

    /// `~defineMethods`: a whole table of names in one mutation --
    /// `RexxClass::defineMethodsRexx` (`ClassClass.cpp:518`), whose
    /// `replaceMethods` is a `replaceMethod` per entry (`:536`,
    /// `MethodDictionary.cpp:221`-`:237`) and which copies the behaviour and
    /// cascades once for the whole table rather than once per name.
    ///
    /// `None` under a name is the oracle's `.nil` entry, which
    /// `createMethodDictionary` passes straight through -- its own comment
    /// reads "a method can be included in the table as the Nil object...this
    /// hides the method of that name and is allowed"
    /// (`ClassClass.cpp:1260`-`:1261`).
    pub fn define_methods(&mut self, class: ObjRef, methods: &[(String, Option<MethodId>)]) {
        let def = self
            .classes
            .get_mut(&class)
            .expect("define_methods: unknown class");
        for (name, method) in methods {
            match method {
                Some(id) => {
                    def.own_instance_methods.replace_method(name, class, *id);
                    // `checkUninit` (`ClassClass.cpp:543`) reads the flattened
                    // behaviour; this crate's `has_uninit` is the same fact
                    // asked of the name being installed, exactly as
                    // `define` above asks it.
                    if name.eq_ignore_ascii_case("UNINIT") {
                        def.has_uninit = true;
                    }
                }
                None => def.own_instance_methods.hide_method(name),
            }
        }
        self.copy_instance_behaviour(class);
        self.update_instance_sub_classes(class);
    }

    /// Point `class` at a fresh, empty [`BehaviourHandle`] -- the
    /// `setField(instanceBehaviour, instanceBehaviour->copy())` that
    /// `~define`, `~defineMethods` and `~delete` each take before they touch
    /// the dictionary, so that an object created before the call keeps
    /// answering the old method set (D43).
    ///
    /// The old handle is never written to again. The fresh one is empty on
    /// return and every caller rebuilds into it.
    fn copy_instance_behaviour(&mut self, class: ObjRef) {
        let fresh = self.alloc_behaviour();
        self.classes
            .get_mut(&class)
            .expect("copy_instance_behaviour: unknown class")
            .instance_behaviour = fresh;
    }

    /// Install a class (static) method directly -- oracle's
    /// `RexxClass::defineClassMethod` (`ClassClass.cpp:883-895`), itself a
    /// "special method to allow a class method to be added to a primitive
    /// class during image build" (its own doc comment), and one of the two
    /// methods `removeSetupMethods` strips before the oracle ships,
    /// alongside `~inheritInstanceMethods` (`:923-941`, `DEFINECLASSMETHOD`).
    /// It adds directly to `class`'s *current* class behaviour and to its
    /// own `classMethodDictionary` (`own_class_methods` here) -- **no
    /// clear, no rebuild-from-scratch, no cascade to subclasses**, unlike
    /// every other mutator in this file (`behaviour->defineMethod(name,
    /// addedMethod)` is a direct, unconditional `MethodDictionary::addMethod`
    /// on the already-built behaviour, not a `createClassBehaviour`
    /// rebuild). A class that already has subclasses when this is called
    /// would leave them unaware of the new method until their own class
    /// behaviour is next rebuilt for an unrelated reason -- the oracle has
    /// this exact gap too, consistent with the doc comment's restriction to
    /// image build, before any subclass yet exists. This crate's own tests
    /// use it to seed a mixin's class-side method before any class
    /// `inherit`s it, which is exactly that restriction in miniature.
    pub fn class_define(&mut self, class: ObjRef, name: &str, method: MethodId) {
        let handle = {
            let def = self
                .classes
                .get_mut(&class)
                .expect("class_define: unknown class");
            def.own_class_methods.add_method(name, class, method);
            def.class_behaviour
        };
        self.behaviours[handle.0]
            .dict
            .add_method(name, class, method);
    }

    /// `~inherit`. Oracle's `RexxClass::inherit` (`:1287`) does **no**
    /// copy: it appends to `superClasses` and calls `updateSubClasses`,
    /// which rebuilds the existing behaviour objects in place. An
    /// instance created before this call sees the donated methods
    /// immediately, unlike [`define`](Self::define).
    ///
    /// The refusals are the oracle's own validation (`:1298`-`:1332`), each
    /// answered as the [`InheritRefusal`] named for the condition it reports:
    /// * `Error_Execution_mixinclass` (`:1299`) -- `mixin` must actually be
    ///   a `MIXINCLASS`.
    /// * `Error_Execution_recursive_inherit` (`:1311`, `mixin` already an
    ///   ancestor of `class`) and its mirror (`:1317`, `class` already an
    ///   ancestor of `mixin`). **The second of these is not decoration**:
    ///   without it, `define_class(b, Some(a), Mixin)` (`B mixinclass A`)
    ///   followed by `inherit(a, b)` (`.A~inherit(.B)`) passes every other
    ///   check, leaves `a.superclasses = [.., b]` and `b.superclasses =
    ///   [a]`, and `update_sub_classes` alternates `a -> b -> a` forever --
    ///   a stack overflow where the oracle raises a clean `98.944`.
    /// * `Error_Execution_baseclass` (`:1323`, `:1329`) -- `mixin` can only
    ///   be inherited by a class whose own two behaviours already have
    ///   `mixin`'s `base_class` in scope.
    ///
    /// **`UNINIT` propagates at the tail of this function** (`:1364`): a
    /// mixin that carries `UNINIT`, itself or through its own ancestors,
    /// gives the inheriting class `parent_has_uninit`. It runs after the
    /// cascade, as the oracle's does, and not at all on a refusal.
    pub fn inherit(&mut self, class: ObjRef, mixin: ObjRef) -> Result<(), InheritRefusal> {
        self.inherit_at(class, mixin, None)
    }

    /// `~inherit`'s own two-argument form: the same operation as
    /// [`Self::inherit`], with the optional `position` the `INHERIT` keyword
    /// of a `::CLASS` directive cannot supply.
    ///
    /// **`position` names where in the superclass list the mixin lands, and
    /// the entry goes *before* it.** The oracle spells this
    /// `superClasses->insertAfter(mixin_class, instanceIndex)`
    /// (`ClassClass.cpp:1353`), but `ArrayClass::insertAfter` is
    /// `insert(item, index)` (`ArrayClass.hpp:247`) and `insert` opens the
    /// gap *at* `index` (`ArrayClass.cpp:797`), so the item takes the
    /// position the named class held. Measured on the oracle, `::class K`
    /// with `.K~inherit(.M1)` then `.K~inherit(.M2, .Object)`:
    /// `~superClasses` reads `M2`, `Object`, `M1`, and the same pair with
    /// `.M1` as the position reads `Object`, `M2`, `M1`.
    ///
    /// A `position` this class does not already inherit is
    /// [`InheritRefusal::NotInherited`] naming it (`:1350`), checked after
    /// every validation the one-argument form does and before anything is
    /// changed.
    pub fn inherit_at(
        &mut self,
        class: ObjRef,
        mixin: ObjRef,
        position: Option<ObjRef>,
    ) -> Result<(), InheritRefusal> {
        if !matches!(self.classes[&mixin].kind, ClassKind::Mixin) {
            return Err(InheritRefusal::NotAMixin);
        }
        if self.behaviour_has_scope(class, Side::Class, mixin) {
            return Err(InheritRefusal::Recursive);
        }
        if self.behaviour_has_scope(mixin, Side::Class, class) {
            return Err(InheritRefusal::Recursive);
        }
        let mixin_base = self.classes[&mixin].base_class;
        if !self.behaviour_has_scope(class, Side::Class, mixin_base) {
            return Err(InheritRefusal::BaseClass(mixin_base));
        }
        if !self.behaviour_has_scope(class, Side::Instance, mixin_base) {
            return Err(InheritRefusal::BaseClass(mixin_base));
        }
        let at = match position {
            None => self.classes[&class].superclasses.len(),
            Some(position) => {
                match self.classes[&class]
                    .superclasses
                    .iter()
                    .position(|&sup| sup == position)
                {
                    Some(at) => at,
                    None => return Err(InheritRefusal::NotInherited(position)),
                }
            }
        };
        self.classes
            .get_mut(&class)
            .unwrap()
            .superclasses
            .insert(at, mixin);
        self.classes.get_mut(&mixin).unwrap().subclasses.push(class);
        self.update_sub_classes(class);
        if self.uninit_reaches(mixin) {
            self.classes.get_mut(&class).unwrap().parent_has_uninit = true;
        }
        Ok(())
    }

    /// `~uninherit`: take a mixin back out of the superclass list --
    /// `RexxClass::uninherit` (`ClassClass.cpp:1379`).
    ///
    /// The mixin must be a `MIXINCLASS` ([`InheritRefusal::NotAMixin`],
    /// `:1393`) and must be in the list at a position past the first
    /// ([`InheritRefusal::NotInherited`], `:1407`). **Past the first, not
    /// merely present**: the oracle's test is `instance_index > 1` over a
    /// 1-based list (`:1401`), so a class's own `SUBCLASS`/`MIXINCLASS`
    /// target is not removable this way. Measured, `::class M mixinclass
    /// Object` with `::class K subclass M` gives `.K~uninherit(.M)` the same
    /// 98.945 a mixin that was never inherited gets.
    ///
    /// Both behaviours are rebuilt in place and the change cascades
    /// (`updateSubClasses`, `:1413`), so this is `~inherit`'s exact reverse
    /// and not `~delete`'s copy-first shape.
    pub fn uninherit(&mut self, class: ObjRef, mixin: ObjRef) -> Result<(), InheritRefusal> {
        if !matches!(self.classes[&mixin].kind, ClassKind::Mixin) {
            return Err(InheritRefusal::NotAMixin);
        }
        let at = self.classes[&class]
            .superclasses
            .iter()
            .position(|&sup| sup == mixin)
            .filter(|at| *at > 0);
        let Some(at) = at else {
            return Err(InheritRefusal::NotInherited(mixin));
        };
        self.classes
            .get_mut(&class)
            .unwrap()
            .superclasses
            .remove(at);
        // `RexxClass::removeSubclass` (`:1424`) takes out the one entry it
        // finds, and a class reaches a mixin's subclass list once: a second
        // `~inherit` of the same mixin is the recursive refusal above.
        if let Some(sub) = self.classes[&mixin]
            .subclasses
            .iter()
            .position(|&sub| sub == class)
        {
            self.classes.get_mut(&mixin).unwrap().subclasses.remove(sub);
        }
        self.update_sub_classes(class);
        Ok(())
    }

    /// `Setup.cpp`'s `InheritInstanceMethods(source)` macro, which is
    /// `RexxBehaviour::inheritInstanceMethods` (`RexxBehaviour.cpp:350`):
    /// copy the donor's own methods into this class's dictionary under this
    /// class's scope, **leaving the donor alone**.
    ///
    /// [`MethodDict::replace_methods_from`] carries why this is a different
    /// operation from [`Self::inherit_instance_methods`] beside it, which is
    /// the Rexx-level method of the same name and does rewrite the donor.
    pub fn donate_instance_methods(&mut self, class: ObjRef, donor: ObjRef) {
        let donor_methods = self
            .classes
            .get(&donor)
            .expect("donate_instance_methods: unknown donor class")
            .own_instance_methods
            .clone();
        self.classes
            .get_mut(&class)
            .expect("donate_instance_methods: unknown recipient class")
            .own_instance_methods
            .replace_methods_from(&donor_methods, donor, class);
        self.rebuild_behaviour(class, Side::Instance);
    }

    /// `Class~inheritInstanceMethods`. Oracle's
    /// `RexxClass::inheritInstanceMethods` (`ClassClass.cpp:558`-`:586`)
    /// takes `donor`'s own instance-method dictionary **by pointer and
    /// rewrites it in place** (`MethodDictionary *sourceMethods =
    /// source->instanceMethodDictionary; sourceMethods->setMethodScope(this);`,
    /// `:560`-`:563`), folds it into `class`'s own instance dictionary, then
    /// rebuilds `class`'s instance behaviour in place -- **no superclass
    /// edge** (`class`'s `superclasses` is untouched) and **no cascade** to
    /// `class`'s own subclasses (unlike [`Self::define`] and
    /// [`Self::inherit`], the oracle function's body has no call to either
    /// `updateSubClasses` or `updateInstanceSubClasses`).
    ///
    /// This crate does the identical in-place rewrite, not a clone: `donor`
    /// really is left with its own instance methods bearing `class`'s scope
    /// afterward, matching the oracle's aliasing rather than a
    /// behaviourally-similar approximation of it.
    ///
    /// **`Setup.cpp`'s macro of the same name is a different C++ function**
    /// and reaches [`Self::donate_instance_methods`] above --
    /// [`MethodDict::replace_methods_from`] carries the distinction and what
    /// conflating the two costs.
    ///
    /// This exact method cannot be oracle-probed by running a script:
    /// `removeSetupMethods` (`:923`) deletes `~inheritInstanceMethods` from
    /// every saved image before it ships (D39), so no program that loads the
    /// built oracle can ever send it -- measured, `Error 97.1: Object "The
    /// Target class" does not understand message "INHERITINSTANCEMETHODS"`.
    /// This method's shape is read from the C++ source alone; its *effect* is
    /// what `.Supplier`, `.Set` and `.Bag` let a running script measure, and
    /// `tests/behaviour_wiring.rs` is built around exactly that distinction.
    pub fn inherit_instance_methods(&mut self, class: ObjRef, donor: ObjRef) {
        let mut donor_methods = std::mem::take(
            &mut self
                .classes
                .get_mut(&donor)
                .expect("inherit_instance_methods: unknown donor class")
                .own_instance_methods,
        );
        donor_methods.set_method_scope(class);
        self.classes
            .get_mut(&class)
            .expect("inherit_instance_methods: unknown recipient class")
            .own_instance_methods
            .merge_methods(&donor_methods);
        self.classes.get_mut(&donor).unwrap().own_instance_methods = donor_methods;
        self.rebuild_behaviour(class, Side::Instance);
    }

    /// The ancestor chain a class-graph assertion reads -- oracle's
    /// `~superClasses`. Two classes can answer identically here despite
    /// [`Self::inherit_instance_methods`] having donated different methods to
    /// each: that is the exact blind spot this crate's tests are built to
    /// demonstrate.
    pub fn ancestors(&self, class: ObjRef) -> &[ObjRef] {
        &self.classes[&class].superclasses
    }

    /// Every class registered as a direct subclass or `INHERIT` recipient of
    /// `class` -- oracle's `subClasses`, what a cascade (`update_sub_classes`
    /// / `update_instance_sub_classes`) walks. Exposed for assertions that
    /// need to see the cascade's own wiring, not just its effect.
    pub fn subclasses(&self, class: ObjRef) -> &[ObjRef] {
        &self.classes[&class].subclasses
    }

    /// `class`'s metaclass (D44) -- oracle's `~metaClass`. `.Class`'s own
    /// entry answers itself.
    pub fn metaclass(&self, class: ObjRef) -> ObjRef {
        self.classes[&class].metaclass
    }

    /// `class`'s own behaviour's owning class -- oracle's
    /// `behaviour->owningClass`, and what `~class` answers for a class
    /// object. See the field for why this is not [`ClassGraph::metaclass`].
    pub fn owning_class(&self, class: ObjRef) -> ObjRef {
        self.classes[&class].owning_class
    }

    /// Whether `class` may be named as another class's metaclass -- oracle's
    /// `isMetaClass`, and the flag `RexxClass::subclass` refuses on
    /// (`ClassClass.cpp:1572`, `Error_Translation_bad_metaclass`). See the
    /// field for what sets it.
    pub fn is_metaclass(&self, class: ObjRef) -> bool {
        self.classes[&class].is_metaclass
    }

    /// `~baseClass` -- oracle's `getBaseClass` (`Setup.cpp:455` binds it as a
    /// method of `.Class`). A `Regular` class answers itself; a `Mixin`
    /// answers whatever its `MIXINCLASS` target's own base class is, which is
    /// the value [`ClassGraph::inherit`]'s compatibility check reads.
    pub fn base_class(&self, class: ObjRef) -> ObjRef {
        self.classes[&class].base_class
    }

    /// Force a class-behaviour rebuild without adding a method -- an escape
    /// hatch [`ClassGraph::define`]/[`ClassGraph::class_define`] don't need
    /// and don't provide.
    ///
    /// **The bootstrap situation it was written for** is a class whose own
    /// metaclass is *itself* (`.Class`, D44's self-reference). Every other
    /// class's metaclass merge
    /// ([`ClassGraph::cascade_build`]'s `Side::Class` arm) reads an
    /// *already-fully-built* different class's instance behaviour, because
    /// bootstrap code builds a class's metaclass before the class itself.
    /// `.Class` cannot receive that treatment: its own initial
    /// `define_class` cascade merges its *own* (at that moment still empty)
    /// instance behaviour into its class-behaviour, and every later
    /// `define()` call that populates it rebuilds the instance side only
    /// (`update_instance_sub_classes`), never the class side. Bootstrap code
    /// must call this once after populating `.Class`'s own instance
    /// methods, or `.class~hasmethod('SUBCLASS')`-shaped facts come out
    /// false for `.Class` alone while every ordinary class gets them right.
    ///
    /// **The other situation is `class_define`'s own restriction being
    /// broken.** That function is `RexxClass::defineClassMethod`, which
    /// cascades to nothing because its own doc comment restricts it to image
    /// build, before any subclass exists. A caller that adds a class method
    /// to a class which already has subclasses owes them a rebuild; a caller
    /// that finishes each class before it builds the classes naming it owes
    /// nothing, because [`ClassGraph::define_class`]'s own cascade reads the
    /// finished dictionary.
    pub fn refresh_class_behaviour(&mut self, class: ObjRef) {
        self.rebuild_behaviour(class, Side::Class);
    }

    /// Merge `metaclass`'s *current* instance behaviour into `root`'s
    /// class-behaviour, bypassing [`ClassGraph::cascade_build`]'s
    /// `Side::Class` "is this the root" guard entirely.
    ///
    /// This is the one place that guard is wrong rather than merely
    /// inapplicable: oracle's root class (`.Object`) empirically *does*
    /// answer `~hasmethod('SUBCLASS')` (measured, `1`), because `.Object`
    /// and `.Class` bootstrap through `buildFinalClassBehaviour`
    /// (`ClassClass.cpp:654-748`), a dedicated path with its own
    /// unconditional `behaviour->merge(TheClassBehaviour)` (`:701`) --
    /// **not** through the general recursive `createClassBehaviour` cascade
    /// every other class goes through, whose `TheObjectClass != this` guard
    /// (this crate's `is_root` check) exists for a different reason: so
    /// that when `.Object` is visited *as another class's ancestor* during
    /// that class's own cascade, its own metaclass contribution is not
    /// folded in twice. This crate has one unified cascade rather than the
    /// oracle's two separate paths, so it cannot express "skip when
    /// visited as an ancestor, but not when built for itself" through the
    /// same guard `cascade_build` already uses -- bootstrap code calls this
    /// once, directly, instead.
    pub fn bootstrap_root_class_behaviour(&mut self, root: ObjRef, metaclass: ObjRef) {
        let meta_instance = self.classes[&metaclass].instance_behaviour;
        let meta_dict = self.behaviours[meta_instance.0].dict.clone();
        let handle = self.behaviour_handle(root, Side::Class);
        self.behaviours[handle.0].dict.merge(&meta_dict);
        self.behaviours[handle.0].version += 1;
    }

    /// Mark `class` a metaclass without deriving it from one -- oracle's
    /// `buildFinalClassBehaviour`'s `if (this == TheClassClass)
    /// setMetaClass();` (`ClassClass.cpp:744`-`:747`), which is where the
    /// chain [`ClassGraph::define_class`] propagates starts.
    ///
    /// [`ClassGraph::define_class`] cannot produce this on its own: it reads
    /// the flag off the superclass, and `.Class`'s superclass is `.Object`,
    /// which is not a metaclass. Bootstrap code calls this once, for the same
    /// reason it calls [`ClassGraph::bootstrap_root_class_behaviour`] -- the
    /// oracle gives `.Object` and `.Class` a dedicated construction path and
    /// this crate has one unified one.
    pub fn bootstrap_metaclass(&mut self, class: ObjRef) {
        self.classes
            .get_mut(&class)
            .expect("bootstrap_metaclass: unknown class")
            .is_metaclass = true;
    }

    /// `class`'s own, unflattened instance-method names -- oracle's
    /// `instanceMethodDictionary`, what `instance~instanceMethods(class)`
    /// answers for a genuine instance (scope-filtered, so ancestor-donated
    /// names are excluded) -- the exact-match target for a per-class
    /// derivation, as opposed to [`Self::method_names_at`]'s flattened,
    /// whole-ancestry set.
    pub fn own_instance_method_names(&self, class: ObjRef) -> BTreeSet<String> {
        self.classes[&class].own_instance_methods.method_names()
    }

    /// Whether `name` is in `class`'s own, unflattened instance-method
    /// dictionary -- the same set [`Self::own_instance_method_names`]
    /// enumerates, asked about one name so a caller does not build the set to
    /// throw it away.
    pub fn has_own_instance_method(&self, class: ObjRef, name: &str) -> bool {
        self.classes[&class].own_instance_methods.has_method(name)
    }

    /// What `name` holds in `class`'s own, unflattened instance-method
    /// dictionary, tombstone included -- what `RexxClass::method` reads
    /// (`ClassClass.cpp:991`) and where it parts from
    /// [`Self::has_own_instance_method`] beside it: a hidden name is
    /// `Some(MethodSlot::Hidden)` here and `false` there, and the oracle
    /// answers the two the same way round.
    pub fn own_instance_slot(&self, class: ObjRef, name: &str) -> Option<MethodSlot> {
        self.classes[&class].own_instance_methods.slot(name)
    }

    /// Oracle's `isRexxDefined` -- see [`ClassDef::rexx_defined`].
    pub fn is_rexx_defined(&self, class: ObjRef) -> bool {
        self.classes[&class].rexx_defined
    }

    /// Oracle's `setRexxDefined` (`ClassClass.cpp:396`), which
    /// `liveGeneral` calls on every class it walks while the image is being
    /// prepared. There is no clearing counterpart, in this crate or in the
    /// oracle.
    pub fn set_rexx_defined(&mut self, class: ObjRef) {
        self.classes
            .get_mut(&class)
            .expect("set_rexx_defined: unknown class")
            .rexx_defined = true;
    }

    /// `class`'s own, unflattened class-method names -- oracle's
    /// `classMethodDictionary`, what `class~instanceMethods(class)` answers
    /// when sent to the class object itself (its own receiver behaviour is
    /// its class-behaviour, filtered to its own scope).
    pub fn own_class_method_names(&self, class: ObjRef) -> BTreeSet<String> {
        self.classes[&class].own_class_methods.method_names()
    }

    /// `class`'s current instance-behaviour handle -- what a freshly
    /// created instance would be given right now.
    pub fn instance_behaviour_handle(&self, class: ObjRef) -> BehaviourHandle {
        self.behaviour_handle(class, Side::Instance)
    }

    /// `class`'s current class-behaviour handle -- what the class object
    /// itself answers to right now.
    pub fn class_behaviour_handle(&self, class: ObjRef) -> BehaviourHandle {
        self.behaviour_handle(class, Side::Class)
    }

    /// `~hasMethod` against a specific, possibly stale, handle -- a
    /// simulated already-created instance's own frozen reference (D43).
    pub fn has_method_at(&self, handle: BehaviourHandle, name: &str) -> bool {
        self.behaviours[handle.0].dict.has_method(name)
    }

    /// Every method name a specific handle answers to -- the method-SET
    /// assertion `ancestors` cannot substitute for.
    pub fn method_names_at(&self, handle: BehaviourHandle) -> BTreeSet<String> {
        self.behaviours[handle.0].dict.method_names()
    }

    /// True once `scope` has been folded into a specific handle's dictionary
    /// -- what distinguishes [`MethodDict::merge`] (methods and scope
    /// history) from `merge_methods` alone (methods only): a build that
    /// replaced the former with the latter in `cascade_build`'s metaclass
    /// branch would still pass every `has_method_at`/`method_names_at`
    /// check (the donated *methods* still land) while this would go false
    /// for the metaclass's own scope specifically.
    pub fn has_scope_at(&self, handle: BehaviourHandle, scope: ObjRef) -> bool {
        self.behaviours[handle.0].dict.has_scope(scope)
    }

    /// An ordinary (unscoped) lookup against a specific handle.
    pub fn lookup_at(&self, handle: BehaviourHandle, name: &str) -> Option<(ObjRef, MethodId)> {
        self.behaviours[handle.0].dict.lookup(name)
    }

    /// A scope-override lookup against a specific handle -- see
    /// [`MethodDict::lookup_from_scope`].
    pub fn lookup_from_scope_at(
        &self,
        handle: BehaviourHandle,
        name: &str,
        start_scope: ObjRef,
    ) -> Option<(ObjRef, MethodId)> {
        self.behaviours[handle.0]
            .dict
            .lookup_from_scope(name, start_scope)
    }

    /// The immediate superscope of `scope` within a specific handle's own
    /// ordering -- see [`MethodDict::resolve_super_scope`].
    pub fn resolve_super_scope_at(&self, handle: BehaviourHandle, scope: ObjRef) -> Option<ObjRef> {
        self.behaviours[handle.0].dict.resolve_super_scope(scope)
    }

    /// The monotonic version D29 asks for -- bumped once per
    /// `rebuild_behaviour` call against this
    /// handle.
    pub fn version_at(&self, handle: BehaviourHandle) -> u64 {
        self.behaviours[handle.0].version
    }

    /// `~hasMethod` against `class`'s *current* instance behaviour --
    /// what `.class~new~hasMethod(name)` would answer right now.
    pub fn has_method(&self, class: ObjRef, name: &str) -> bool {
        self.has_method_at(self.instance_behaviour_handle(class), name)
    }

    /// `~hasMethod` sent to the class object itself, against `class`'s
    /// current class behaviour.
    pub fn class_has_method(&self, class: ObjRef, name: &str) -> bool {
        self.has_method_at(self.class_behaviour_handle(class), name)
    }
}
