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

use crate::method_dict::{MethodDict, MethodSlot};
use rexx_core::MethodId;
use rexx_core::NameMap;
use rexx_core::{BehaviourHandle, ObjRef};
use std::collections::BTreeSet;

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
    /// The class this class object's own behaviour belongs to -- oracle's
    /// `behaviour->owningClass`, set from the metaclass `subclass` was given
    /// or defaulted to (`ClassClass.cpp:1615`), and what `~class` answers for
    /// a class object.
    /// ```text
    /// ::CLASS T  SUBCLASS MC METACLASS M1   ~metaClass MC     ~class M1      part
    /// ::CLASS T2 SUBCLASS MC                ~metaClass MC     ~class Class   part
    /// ::CLASS M3 SUBCLASS MC METACLASS MC   ~metaClass MC     ~class MC      same
    /// ::class MC MIXINCLASS Class           ~metaClass Class  ~class Class   same
    /// ::class Z  SUBCLASS Class             ~metaClass Class  ~class Class   same
    /// ::CLASS K  METACLASS M1               ~metaClass M1     ~class M1      same
    /// ```
    owning_class: ObjRef,
    /// This class may be named as another class's metaclass -- oracle's
    /// `META_CLASS` class flag, which `RexxClass::subclass` tests before it
    /// will build anything from the metaclass it was handed (`:1572`).
    is_metaclass: bool,
    /// This class's instances need `UNINIT` run when they are collected --
    /// oracle's `HAS_UNINIT` class flag, which is what
    /// `completeNewObject` reads to register each new instance
    /// (`ClassClass.cpp:1892`).
    has_uninit: bool,
    /// Some class this one inherits from carries `UNINIT` -- oracle's
    /// `PARENT_HAS_UNINIT`, set by each constructor that builds an
    /// inheritance edge: `subclass` (`:1634`), `mixinClass` (`:1525`) and
    /// `inherit` (`:1364`).
    parent_has_uninit: bool,
    /// `~new` on this class refuses -- oracle's `ABSTRACT` class flag, set by
    /// `RexxClass::makeAbstract` (`ClassClass.cpp:1754`) and read by
    /// `checkAbstract` (`:1741`) as the first step of `completeNewObject`.
    is_abstract: bool,
    /// This class may not be altered from Rexx -- oracle's `REXX_DEFINED`
    /// class flag. `RexxClass::liveGeneral` sets it on every class in the
    /// image under `PREPARINGIMAGE` (`ClassClass.cpp:136`-`:142`), which is
    /// every class the image holds: the ones `Setup.cpp` builds and the ones
    /// the interpreter's own Rexx-written library declares alike. A `::CLASS`
    /// a program declares does not carry it.
    rexx_defined: bool,
}

/// The class graph and its behaviour storage.
#[derive(Default)]
pub struct ClassGraph {
    /// [`NameMap`] for `MethodDict`'s reason: the cascade reads this map
    /// once per superclass per class while the library is being built.
    classes: NameMap<ObjRef, ClassDef>,
    behaviours: Vec<Behaviour>,
    /// Oracle's uninit table restricted to class objects -- the entries
    /// `RexxClass::checkUninit`'s `requiresUninit()` (`ClassClass.cpp:1224`)
    /// makes, in the order it makes them.
    uninit_classes: Vec<ObjRef>,
}

impl ClassGraph {
    pub fn new() -> Self {
        Self::default()
    }

    fn alloc_behaviour(&mut self) -> BehaviourHandle {
        let handle = BehaviourHandle::new(self.behaviours.len());
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
    /// the `uninit_classes` list when its **class** behaviour answers `UNINIT`
    /// -- `if (hasUninitMethod()) requiresUninit();` (`:1224`), which is where
    /// a `::METHOD uninit CLASS` lands.
    pub fn check_uninit(&mut self, class: ObjRef) {
        let instance = self.behaviour_handle(class, Side::Instance);
        if self.behaviours[instance.index()].dict.has_method("UNINIT") {
            self.classes.get_mut(&class).unwrap().has_uninit = true;
        }
        let class_side = self.behaviour_handle(class, Side::Class);
        if self.behaviours[class_side.index()]
            .dict
            .has_method("UNINIT")
            && !self.uninit_classes.contains(&class)
        {
            self.uninit_classes.push(class);
        }
    }

    /// Drops every trace of `dead` from the graph, and scrubs the subclass
    /// lists of the classes that survive.
    pub fn expunge(&mut self, dead: &[ObjRef]) {
        let ClassGraph {
            classes,
            behaviours,
            uninit_classes,
        } = self;
        for class in dead {
            // The dictionaries are the weight: a flattened behaviour holds
            // every method the class answers, inherited ones included.
            // Emptied rather than removed, because a behaviour is named by
            // its index and removing one would move every later index.
            if let Some(def) = classes.get(class) {
                for handle in [def.instance_behaviour, def.class_behaviour] {
                    behaviours[handle.index()].dict.clear();
                }
            }
            classes.remove(class);
            uninit_classes.retain(|held| held != class);
        }
        // A subclass list holds classes that may have died: the oracle's own
        // `subClasses` is a list of `WeakReference` and `getSubClasses`
        // prunes as it reads (`ClassClass.hpp:208`, `ClassClass.cpp:473`).
        for def in classes.values_mut() {
            def.subclasses.retain(|sub| !dead.contains(sub));
        }
    }

    /// Whether `class` is waiting to have its class-side `UNINIT` run.
    pub fn has_pending_class_uninit(&self, class: ObjRef) -> bool {
        self.uninit_classes.contains(&class)
    }

    /// Drops `class` from the pending list, for a finalizer a collection has
    /// already run.
    pub fn forget_uninit_class(&mut self, class: ObjRef) {
        self.uninit_classes.retain(|held| *held != class);
    }

    /// Every class object whose own behaviour answers `UNINIT`, in the order
    /// [`Self::check_uninit`] entered them, taken out of the graph.
    pub fn take_uninit_classes(&mut self) -> Vec<ObjRef> {
        std::mem::take(&mut self.uninit_classes)
    }

    /// Recompute [`Self::parent_has_uninit`] from the class's current
    /// superclass list.
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
        self.behaviours[handle.index()].dict.has_scope(scope)
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
    fn cascade_build(
        classes: &NameMap<ObjRef, ClassDef>,
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
                    target.merge(&behaviours[meta_instance.index()].dict);
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
    fn rebuild_behaviour(&mut self, class: ObjRef, side: Side) {
        let handle = self.behaviour_handle(class, side);
        let mut dict = std::mem::take(&mut self.behaviours[handle.index()].dict);
        dict.clear();
        Self::cascade_build(&self.classes, &self.behaviours, class, &mut dict, side);
        let behaviour = &mut self.behaviours[handle.index()];
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
        self.behaviours[handle.index()]
            .dict
            .add_method(name, class, method);
    }

    /// `~inherit`. Oracle's `RexxClass::inherit` (`:1287`) does **no**
    /// copy: it appends to `superClasses` and calls `updateSubClasses`,
    /// which rebuilds the existing behaviour objects in place. An
    /// instance created before this call sees the donated methods
    /// immediately, unlike [`define`](Self::define).
    pub fn inherit(&mut self, class: ObjRef, mixin: ObjRef) -> Result<(), InheritRefusal> {
        self.inherit_at(class, mixin, None)
    }

    /// `~inherit`'s own two-argument form: the same operation as
    /// [`Self::inherit`], with the optional `position` the `INHERIT` keyword
    /// of a `::CLASS` directive cannot supply.
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

    /// `~queryMixinClass` -- oracle's `isMixinClass`, the same
    /// [`ClassKind`] `inherit`'s `Error_Execution_mixinclass` check reads.
    pub fn is_mixin(&self, class: ObjRef) -> bool {
        matches!(self.classes[&class].kind, ClassKind::Mixin)
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
    pub fn refresh_class_behaviour(&mut self, class: ObjRef) {
        self.rebuild_behaviour(class, Side::Class);
    }

    /// Merge `metaclass`'s *current* instance behaviour into `root`'s
    /// class-behaviour, bypassing [`ClassGraph::cascade_build`]'s
    /// `Side::Class` "is this the root" guard entirely.
    pub fn bootstrap_root_class_behaviour(&mut self, root: ObjRef, metaclass: ObjRef) {
        let meta_instance = self.classes[&metaclass].instance_behaviour;
        let meta_dict = self.behaviours[meta_instance.index()].dict.clone();
        let handle = self.behaviour_handle(root, Side::Class);
        self.behaviours[handle.index()].dict.merge(&meta_dict);
        self.behaviours[handle.index()].version += 1;
    }

    /// Mark `class` a metaclass without deriving it from one -- oracle's
    /// `buildFinalClassBehaviour`'s `if (this == TheClassClass)
    /// setMetaClass();` (`ClassClass.cpp:744`-`:747`), which is where the
    /// chain [`ClassGraph::define_class`] propagates starts.
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
        self.behaviours[handle.index()].dict.has_method(name)
    }

    /// Every method name a specific handle answers to -- the method-SET
    /// assertion `ancestors` cannot substitute for.
    pub fn method_names_at(&self, handle: BehaviourHandle) -> BTreeSet<String> {
        self.behaviours[handle.index()].dict.method_names()
    }

    /// True once `scope` has been folded into a specific handle's dictionary
    /// -- what distinguishes [`MethodDict::merge`] (methods and scope
    /// history) from `merge_methods` alone (methods only): a build that
    /// replaced the former with the latter in `cascade_build`'s metaclass
    /// branch would still pass every `has_method_at`/`method_names_at`
    /// check (the donated *methods* still land) while this would go false
    /// for the metaclass's own scope specifically.
    pub fn has_scope_at(&self, handle: BehaviourHandle, scope: ObjRef) -> bool {
        self.behaviours[handle.index()].dict.has_scope(scope)
    }

    /// An ordinary (unscoped) lookup against a specific handle.
    pub fn lookup_at(&self, handle: BehaviourHandle, name: &str) -> Option<(ObjRef, MethodId)> {
        self.behaviours[handle.index()].dict.lookup(name)
    }

    /// A scope-override lookup against a specific handle -- see
    /// [`MethodDict::lookup_from_scope`].
    pub fn lookup_from_scope_at(
        &self,
        handle: BehaviourHandle,
        name: &str,
        start_scope: ObjRef,
    ) -> Option<(ObjRef, MethodId)> {
        self.behaviours[handle.index()]
            .dict
            .lookup_from_scope(name, start_scope)
    }

    /// The immediate superscope of `scope` within a specific handle's own
    /// ordering -- see [`MethodDict::resolve_super_scope`].
    pub fn resolve_super_scope_at(&self, handle: BehaviourHandle, scope: ObjRef) -> Option<ObjRef> {
        self.behaviours[handle.index()]
            .dict
            .resolve_super_scope(scope)
    }

    /// The monotonic version D29 asks for -- bumped once per
    /// `rebuild_behaviour` call against this
    /// handle.
    pub fn version_at(&self, handle: BehaviourHandle) -> u64 {
        self.behaviours[handle.index()].version
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
