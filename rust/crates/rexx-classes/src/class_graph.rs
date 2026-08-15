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
//! What this crate does not model: message dispatch, method bodies, error
//! raising for an invalid `inherit` (the two `assert!`s below are this
//! crate's own sanity checks, not the oracle's `SYNTAX` conditions -- that
//! belongs to whichever later task wires `~inherit` to a raise), and a
//! metaclass's own instance dictionary merging into a class's class-side
//! behaviour (`RexxClass::createClassBehaviour`'s `metaClass->mergeInstanceBehaviour`
//! branch) -- the class-behaviour cascade this crate builds is the
//! `INHERIT`-list half of D44, which is what the brief's probe exercises.

use crate::method_dict::{MethodDict, MethodId};
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
    pub fn define_class(&mut self, id: ObjRef, superclass: Option<ObjRef>, kind: ClassKind) {
        let base_class = match kind {
            ClassKind::Regular => id,
            ClassKind::Mixin => {
                let target = superclass.expect("a mixin class names its MIXINCLASS target");
                self.classes[&target].base_class
            }
        };
        let instance_behaviour = self.alloc_behaviour();
        let class_behaviour = self.alloc_behaviour();
        self.classes.insert(
            id,
            ClassDef {
                superclasses: superclass.into_iter().collect(),
                subclasses: Vec::new(),
                own_instance_methods: MethodDict::new(),
                own_class_methods: MethodDict::new(),
                instance_behaviour,
                class_behaviour,
                base_class,
            },
        );
        if let Some(sup) = superclass {
            self.classes.get_mut(&sup).unwrap().subclasses.push(id);
        }
        self.rebuild_behaviour(id, Side::Instance);
        self.rebuild_behaviour(id, Side::Class);
    }

    fn own_dict(&self, class: ObjRef, side: Side) -> &MethodDict {
        let def = &self.classes[&class];
        match side {
            Side::Instance => &def.own_instance_methods,
            Side::Class => &def.own_class_methods,
        }
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
    fn cascade_build(&self, class: ObjRef, target: &mut MethodDict, side: Side) {
        let superclasses = self.classes[&class].superclasses.clone();
        for sup in superclasses.into_iter().rev() {
            if !target.has_scope(sup) {
                self.cascade_build(sup, target, side);
            }
        }
        if !target.has_scope(class) {
            target.merge_methods(self.own_dict(class, side));
            target.add_scope(class);
        }
    }

    fn rebuild_behaviour(&mut self, class: ObjRef, side: Side) {
        let mut fresh = MethodDict::new();
        self.cascade_build(class, &mut fresh, side);
        let handle = self.behaviour_handle(class, side);
        let behaviour = &mut self.behaviours[handle.0];
        behaviour.dict = fresh;
        behaviour.version += 1;
    }

    /// Oracle's `updateSubClasses` (`:1036`): rebuild both behaviours in
    /// place, instance side first ("the added methods may have an impact
    /// on metaclasses"), then cascade to every subclass. This is what
    /// [`inherit`](Self::inherit) calls, and it never allocates a new
    /// handle -- every class it touches, direct or cascaded, keeps the id
    /// its instances were created with.
    fn update_sub_classes(&mut self, class: ObjRef) {
        self.rebuild_behaviour(class, Side::Instance);
        self.rebuild_behaviour(class, Side::Class);
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

    /// The class-side counterpart of [`update_instance_sub_classes`],
    /// used by [`class_define`](Self::class_define) alone: rebuild only
    /// the class behaviour, in place, then cascade.
    fn update_class_sub_classes(&mut self, class: ObjRef) {
        self.rebuild_behaviour(class, Side::Class);
        let subclasses = self.classes[&class].subclasses.clone();
        for sub in subclasses {
            self.update_class_sub_classes(sub);
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
    pub fn define(&mut self, class: ObjRef, name: &str, method: MethodId) {
        self.classes
            .get_mut(&class)
            .expect("define: unknown class")
            .own_instance_methods
            .add_method(name, class, method);
        let fresh = self.alloc_behaviour();
        self.classes.get_mut(&class).unwrap().instance_behaviour = fresh;
        self.update_instance_sub_classes(class);
    }

    /// Install a class (static) method directly, no oracle-named
    /// equivalent. There is no per-object "instance already created"
    /// concern on the class side: exactly one object is ever an instance
    /// of `class`'s own class-side behaviour, the class object itself, so
    /// unlike [`define`](Self::define) this never allocates a fresh
    /// handle -- it rebuilds `class`'s class behaviour in place and
    /// cascades, the class-side half of `updateSubClasses`. The oracle
    /// installs a `::METHOD ... CLASS` the same way, at class-definition
    /// time, before anything could have observed the class object's old
    /// state; this crate's own tests use it to seed a mixin's class-side
    /// method before any class `inherit`s it.
    pub fn class_define(&mut self, class: ObjRef, name: &str, method: MethodId) {
        self.classes
            .get_mut(&class)
            .expect("class_define: unknown class")
            .own_class_methods
            .add_method(name, class, method);
        self.update_class_sub_classes(class);
    }

    /// `~inherit`. Oracle's `RexxClass::inherit` (`:1287`) does **no**
    /// copy: it appends to `superClasses` and calls `updateSubClasses`,
    /// which rebuilds the existing behaviour objects in place. An
    /// instance created before this call sees the donated methods
    /// immediately, unlike [`define`](Self::define).
    ///
    /// The two `assert!`s are this crate's own sanity checks standing in
    /// for the oracle's `Error_Execution_baseclass` (`:1323`, `:1329`):
    /// `mixin` can only be inherited by a class whose own two behaviours
    /// already have `mixin`'s `base_class` in scope. Raising the oracle's
    /// actual syntax condition on violation is later work; this crate
    /// only refuses to build a graph the oracle itself would refuse.
    pub fn inherit(&mut self, class: ObjRef, mixin: ObjRef) {
        let mixin_base = self.classes[&mixin].base_class;
        assert!(
            self.behaviour_has_scope(class, Side::Class, mixin_base),
            "inherit: {class:?} itself is not a subclass of {mixin:?}'s MIXINCLASS base {mixin_base:?}"
        );
        assert!(
            self.behaviour_has_scope(class, Side::Instance, mixin_base),
            "inherit: {class:?}'s instances are not a subclass of {mixin:?}'s MIXINCLASS base {mixin_base:?}"
        );
        self.classes
            .get_mut(&class)
            .unwrap()
            .superclasses
            .push(mixin);
        self.classes.get_mut(&mixin).unwrap().subclasses.push(class);
        self.update_sub_classes(class);
    }

    /// `inheritInstanceMethods`. Oracle's `RexxClass::inheritInstanceMethods`
    /// (`:558`) rewrites `donor`'s own instance methods to `class`'s own
    /// scope, folds them into `class`'s own instance dictionary, then
    /// rebuilds `class`'s instance behaviour in place -- **no superclass
    /// edge** (`class`'s `superclasses` is untouched) and **no cascade**
    /// to `class`'s own subclasses (unlike [`Self::define`] and [`Self::inherit`], the
    /// oracle function's body has no call to either `updateSubClasses` or
    /// `updateInstanceSubClasses`).
    ///
    /// This exact method cannot be oracle-probed by running a script:
    /// `removeSetupMethods` (`:923`) deletes `~inheritInstanceMethods` from
    /// every saved image before it ships (D39), so no program that loads
    /// the built oracle can ever send it -- measured, `Error 97.1: Object
    /// "The Target class" does not understand message
    /// "INHERITINSTANCEMETHODS"`. This method's shape is read from the
    /// C++ source alone; its *effect* is what `.Supplier`, `.Set` and
    /// `.Bag` let a running script measure, and `tests/behaviour_wiring.rs`
    /// is built around exactly that distinction.
    pub fn inherit_instance_methods(&mut self, class: ObjRef, donor: ObjRef) {
        let mut donated = self.classes[&donor].own_instance_methods.clone();
        donated.set_method_scope(class);
        self.classes
            .get_mut(&class)
            .expect("inherit_instance_methods: unknown recipient class")
            .own_instance_methods
            .merge_methods(&donated);
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
