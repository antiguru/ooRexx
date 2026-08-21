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
    metaclass: ObjRef,
    /// This class defines `UNINIT` itself, so its instances have to be
    /// registered for one when they are created -- oracle's `HAS_UNINIT`
    /// class flag, which `RexxClass::defineMethod` sets for a method of that
    /// name (`ClassClass.cpp:854`).
    has_uninit: bool,
    /// Some class this one inherits from carries `UNINIT` -- oracle's
    /// `PARENT_HAS_UNINIT`, set by each constructor that builds an
    /// inheritance edge: `subclass` (`:1634`), `mixinClass` (`:1525`) and
    /// `inherit` (`:1364`).
    parent_has_uninit: bool,
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
    pub fn define_class(
        &mut self,
        id: ObjRef,
        superclass: Option<ObjRef>,
        kind: ClassKind,
        metaclass: ObjRef,
    ) {
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
                has_uninit: false,
                parent_has_uninit,
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

    /// Whether `class` defines `UNINIT` itself -- oracle's
    /// `hasUninitDefined`.
    pub fn has_uninit(&self, class: ObjRef) -> bool {
        self.classes[&class].has_uninit
    }

    /// Whether a class `class` inherits from defines `UNINIT` -- oracle's
    /// `parentHasUninitDefined`.
    pub fn parent_has_uninit(&self, class: ObjRef) -> bool {
        self.classes[&class].parent_has_uninit
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
        def.own_instance_methods.add_method(name, class, method);
        if name.eq_ignore_ascii_case("UNINIT") {
            def.has_uninit = true;
        }
        let fresh = self.alloc_behaviour();
        self.classes.get_mut(&class).unwrap().instance_behaviour = fresh;
        self.update_instance_sub_classes(class);
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
        self.classes
            .get_mut(&class)
            .unwrap()
            .superclasses
            .push(mixin);
        self.classes.get_mut(&mixin).unwrap().subclasses.push(class);
        self.update_sub_classes(class);
        if self.uninit_reaches(mixin) {
            self.classes.get_mut(&class).unwrap().parent_has_uninit = true;
        }
        Ok(())
    }

    /// `inheritInstanceMethods`. Oracle's `RexxClass::inheritInstanceMethods`
    /// (`:558-586`) takes `donor`'s own instance-method dictionary **by
    /// pointer and rewrites it in place** (`MethodDictionary *sourceMethods
    /// = source->instanceMethodDictionary; sourceMethods->setMethodScope(this);`,
    /// `:560-562`), folds it into `class`'s own instance dictionary, then
    /// rebuilds `class`'s instance behaviour in place -- **no superclass
    /// edge** (`class`'s `superclasses` is untouched) and **no cascade**
    /// to `class`'s own subclasses (unlike [`Self::define`] and
    /// [`Self::inherit`], the oracle function's body has no call to either
    /// `updateSubClasses` or `updateInstanceSubClasses`).
    ///
    /// This crate does the identical in-place rewrite, not a clone: `donor`
    /// really is left with its own instance methods bearing `class`'s scope
    /// afterward, matching the oracle's aliasing exactly rather than a
    /// behaviourally-similar approximation of it. (There is no accessor in
    /// this crate's public API that can observe `donor`'s own dictionary
    /// directly after the call -- only the flattened behaviours are
    /// queryable -- so this fidelity is not independently exercised by a
    /// test; it is verified by matching the C++ line by line.)
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
    /// build, before any subclass exists. A `::CLASS` naming a `SUBCLASS`
    /// declared later in the same file is created before that superclass's
    /// class methods are added, so the subclass never sees them; `rexx-exec`
    /// rebuilds each class a program installs once that program's directives
    /// are all in.
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

    /// `class`'s own, unflattened instance-method names -- oracle's
    /// `instanceMethodDictionary`, what `instance~instanceMethods(class)`
    /// answers for a genuine instance (scope-filtered, so ancestor-donated
    /// names are excluded) -- the exact-match target for a per-class
    /// derivation, as opposed to [`Self::method_names_at`]'s flattened,
    /// whole-ancestry set.
    pub fn own_instance_method_names(&self, class: ObjRef) -> BTreeSet<String> {
        self.classes[&class].own_instance_methods.method_names()
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
