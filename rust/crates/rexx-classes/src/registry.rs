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

//! `ClassRegistry`: class objects, and the name-to-class registry, over
//! [`ClassGraph`](crate::ClassGraph).
//!
//! A class object is an [`ObjRef`] identity plus its id string; everything
//! else the brief lists (two behaviour ids, the superclass list, the
//! subclass list) is `ClassGraph`'s own `ClassDef`, already carrying them --
//! this module owns identity allocation and the id string, `ClassGraph`
//! owns the cascade. The registry is the name half: a class is found by the
//! string a `::CLASS` directive or a primitive bootstrap gave it, matching
//! oracle's `TheEnvironment->put(classObj, className)`
//! (`Setup.cpp:203`) -- one uppercased name resolves to one class, exactly
//! that direction, nothing about `.environment`/`.local`/`.NAME` resolution
//! order itself (D33's, not this module's).
//!
//! What this module does not do: allocate an `ObjRef` for anything other
//! than a class (no ordinary object arena), or resolve a name through any
//! path other than this flat table (no `.environment`/`.local` chain, no
//! `.context`/`.rexxinfo`-style dynamic instance).

use crate::class_graph::{ClassGraph, ClassKind, InheritRefusal};
use crate::method_dict::{MethodId, MethodSlot};
use rexx_core::ObjRef;
use std::collections::HashMap;

pub struct ClassRegistry {
    graph: ClassGraph,
    next_id: u32,
    next_method: u32,
    /// Id string as declared (`~id`), keyed by identity.
    names: HashMap<ObjRef, String>,
    /// `~defaultName`, keyed by identity -- see [`ClassRegistry::default_name`].
    default_names: HashMap<ObjRef, String>,
    /// `~objectName=`'s store for a class object, keyed by identity, holding
    /// only the classes something has renamed.
    ///
    /// Separate from `default_names` because `~defaultName` keeps answering
    /// the declared form after a rename, and because a class identity is the
    /// one handle the arena does not hold, so there is no object to put the
    /// name in -- every other renameable object carries its own.
    object_names: HashMap<ObjRef, String>,
    /// Uppercased name -> identity, the registry's own lookup direction --
    /// oracle's `TheEnvironment->put(classObj, getUpperGlobalName(name))`.
    by_name: HashMap<String, ObjRef>,
}

impl Default for ClassRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ClassRegistry {
    pub fn new() -> Self {
        Self {
            graph: ClassGraph::new(),
            next_id: 0,
            next_method: 0,
            names: HashMap::new(),
            default_names: HashMap::new(),
            object_names: HashMap::new(),
            by_name: HashMap::new(),
        }
    }

    /// Allocate a fresh identity without registering it as a class yet --
    /// what bootstrapping `.Object` and `.Class` needs, since each names the
    /// other's id before either exists (see
    /// [`ClassGraph::define_class`]'s own doc comment for why a forward
    /// reference like this is safe).
    pub fn reserve_id(&mut self) -> ObjRef {
        // **A class identity comes out of `rexx-core`'s reserved range, not
        // out of the arena's.** Both counters used to start at zero, so the
        // first class and the first heap slot were the same sixty-four bits;
        // `ObjRef::class` is what keeps them disjoint, and `ObjRef::class_id`
        // is what a consumer asks before treating a handle as a value. See
        // `rexx_core::CLASS_SLOT_BASE`.
        let id = ObjRef::class(self.next_id).expect("a program declares fewer than 2^31 classes");
        self.next_id += 1;
        id
    }

    fn next_method_id(&mut self) -> MethodId {
        let id = MethodId(self.next_method);
        self.next_method += 1;
        id
    }

    /// Register `id` (already allocated by [`Self::reserve_id`]) as a class
    /// named `name`, and build its graph entry -- oracle's
    /// `RexxClass::subclass`/bootstrap construction plus
    /// `completeSystemClass`'s environment registration, fused into one
    /// call since this crate models no separate install step.
    pub fn define_reserved(
        &mut self,
        id: ObjRef,
        name: &str,
        superclass: Option<ObjRef>,
        kind: ClassKind,
        metaclass: ObjRef,
    ) {
        self.graph.define_class(id, superclass, kind, metaclass);
        self.names.insert(id, name.to_string());
        self.default_names.insert(id, format!("The {name} class"));
        self.by_name.insert(name.to_ascii_uppercase(), id);
    }

    /// Allocate a fresh identity and register it in one step -- the
    /// ordinary case, every class except the `.Object`/`.Class` bootstrap
    /// pair (which must reserve both ids before either is defined).
    pub fn define_class(
        &mut self,
        name: &str,
        superclass: Option<ObjRef>,
        kind: ClassKind,
        metaclass: ObjRef,
    ) -> ObjRef {
        let id = self.reserve_id();
        self.define_reserved(id, name, superclass, kind, metaclass);
        id
    }

    /// Allocate a fresh identity and give it an id string, **without**
    /// registering the name -- what a `::CLASS` directive creates.
    ///
    /// The oracle files an installed class against the package
    /// (`PackageClass::addInstalledClass`) and never into the environment;
    /// only `completeSystemClass`, an image-build path, does the latter. So a
    /// `::CLASS` named `Array` must not displace the environment's own
    /// `Array`, which registering it here would do -- [`Self::registered`] is
    /// what `.environment` is populated from.
    pub fn define_unregistered_class(
        &mut self,
        name: &str,
        superclass: Option<ObjRef>,
        kind: ClassKind,
        metaclass: ObjRef,
    ) -> ObjRef {
        let id = self.reserve_id();
        self.graph.define_class(id, superclass, kind, metaclass);
        self.names.insert(id, name.to_string());
        self.default_names.insert(id, format!("The {name} class"));
        id
    }

    /// `.NAME` resolution's terminal step: an uppercased lookup against this
    /// flat table. `None` for anything not in this registry -- deferred
    /// classes included, since deferring one is exactly declining to add it
    /// here.
    pub fn lookup(&self, name: &str) -> Option<ObjRef> {
        self.by_name.get(&name.to_ascii_uppercase()).copied()
    }

    /// `~id` -- the string a class was declared with, unmodified case.
    pub fn id_string(&self, class: ObjRef) -> &str {
        &self.names[&class]
    }

    /// `~defaultName` -- `RexxClass::defaultName` (`ClassClass.cpp:614`),
    /// which is the id with `The ` in front and ` class` behind it, and is
    /// what `SAY` prints for a class object.
    ///
    /// Stored rather than assembled on demand because the one caller that
    /// needs it renders a value through a shared borrow and has nowhere to put
    /// a freshly built string.
    pub fn default_name(&self, class: ObjRef) -> &str {
        &self.default_names[&class]
    }

    /// `~objectName` for a class object: what `~objectName=` last stored, or
    /// [`Self::default_name`] for a class nothing has renamed.
    ///
    /// This is `RexxObject::stringValue`'s answer and so the bytes `SAY`
    /// prints, which is why the rename is visible through every rendering of
    /// the class object and not only through the `~objectName` message.
    pub fn object_name(&self, class: ObjRef) -> &str {
        match self.object_names.get(&class) {
            Some(name) => name,
            None => self.default_name(class),
        }
    }

    /// `~objectName=` for a class object.
    pub fn set_object_name(&mut self, class: ObjRef, name: &str) {
        self.object_names.insert(class, name.to_string());
    }

    /// Every class this registry answers [`Self::lookup`] for, as
    /// (uppercased name, identity).
    ///
    /// The oracle's `completeSystemClass` puts exactly this pair into
    /// `.environment` (`Setup.cpp:203`), which is the one consumer.
    pub fn registered(&self) -> impl Iterator<Item = (&str, ObjRef)> {
        self.by_name.iter().map(|(name, id)| (name.as_str(), *id))
    }

    /// `~class`, for a class object: the class its own behaviour belongs to
    /// (`ClassClass.cpp:1615`, `:736`/`:803` for the primitive bootstrap
    /// path) -- [`ClassGraph::owning_class`], **not**
    /// [`ClassGraph::metaclass`].
    ///
    /// **They part iff the superclass is a metaclass and is not the
    /// named-or-inherited metaclass** -- measured, and stated as an `iff`
    /// because deriving from a metaclass is necessary and not sufficient.
    /// `::CLASS T SUBCLASS MC METACLASS M1` parts, answering `~class` `M1`
    /// and `~metaClass` `MC`, and so does `::CLASS T2 SUBCLASS MC` with no
    /// `METACLASS` named, answering `Class` and `MC`. But
    /// `::CLASS M3 SUBCLASS MC METACLASS MC` answers `MC` to both, and so do
    /// `::class MC MIXINCLASS Class` and `::class Z SUBCLASS Class` -- every
    /// one of those derives from a metaclass, and coincides because the
    /// superclass is the metaclass in play.
    ///
    /// Anywhere the superclass is not a metaclass, nothing overrides and the
    /// two agree: measured, `.string~class~id` and `.string~metaclass~id` are
    /// both `"Class"`, and so are `.object`'s and `.class`'s own. Every class
    /// `native_classes` builds names `.Object` as its superclass, or is
    /// `.Object` itself and names none, so none of them derives from a
    /// metaclass at all.
    ///
    /// An ordinary (non-class) object's `~class` is unrelated to any
    /// metaclass concept -- out of scope here, since this registry models
    /// only class objects.
    pub fn class_of(&self, class: ObjRef) -> ObjRef {
        self.graph.owning_class(class)
    }

    /// `~metaClass`.
    pub fn metaclass(&self, class: ObjRef) -> ObjRef {
        self.graph.metaclass(class)
    }

    /// `~isMetaClass` -- see [`ClassGraph::is_metaclass`].
    pub fn is_metaclass(&self, class: ObjRef) -> bool {
        self.graph.is_metaclass(class)
    }

    /// See [`ClassGraph::bootstrap_metaclass`].
    pub fn bootstrap_metaclass(&mut self, class: ObjRef) {
        self.graph.bootstrap_metaclass(class);
    }

    /// `~baseClass` -- see [`ClassGraph::base_class`].
    pub fn base_class(&self, class: ObjRef) -> ObjRef {
        self.graph.base_class(class)
    }

    /// Whether `class`'s instances need `UNINIT` -- see
    /// [`ClassGraph::has_uninit`].
    pub fn has_uninit(&self, class: ObjRef) -> bool {
        self.graph.has_uninit(class)
    }

    /// See [`ClassGraph::check_uninit`].
    pub fn check_uninit(&mut self, class: ObjRef) {
        self.graph.check_uninit(class);
    }

    /// See [`ClassGraph::refresh_parent_has_uninit`].
    pub fn refresh_parent_has_uninit(&mut self, class: ObjRef) {
        self.graph.refresh_parent_has_uninit(class);
    }

    /// Whether a class `class` inherits from defines `UNINIT` -- see
    /// [`ClassGraph::parent_has_uninit`].
    pub fn parent_has_uninit(&self, class: ObjRef) -> bool {
        self.graph.parent_has_uninit(class)
    }

    /// `~superClass` -- the first entry of `~superClasses`, oracle's
    /// `superClass` field (singular), or `None` for `.Object`.
    pub fn superclass(&self, class: ObjRef) -> Option<ObjRef> {
        self.graph.ancestors(class).first().copied()
    }

    /// `~superClasses`.
    pub fn superclasses(&self, class: ObjRef) -> &[ObjRef] {
        self.graph.ancestors(class)
    }

    /// `~subClasses` -- oracle's `subClasses`, the reverse edge every
    /// `define_class`/`inherit` call maintains alongside `superclasses`.
    pub fn subclasses(&self, class: ObjRef) -> &[ObjRef] {
        self.graph.subclasses(class)
    }

    /// `~isA`/`~isSubclassOf` -- oracle's `RexxClass::isCompatibleWith`
    /// (`ClassClass.cpp:1660-1681`): `class == other`, or any of `class`'s
    /// ancestors is compatible with `other`, recursively over the same
    /// `superClasses` list `~superClasses` reads (not the flattened
    /// behaviour -- a mixin donated only through
    /// `inheritInstanceMethods` does **not** make `~isA` true, matching the
    /// oracle exactly).
    pub fn is_a(&self, class: ObjRef, other: ObjRef) -> bool {
        class == other
            || self
                .graph
                .ancestors(class)
                .iter()
                .any(|&sup| self.is_a(sup, other))
    }

    /// The class's own instance method set -- the sixth probe, not
    /// optional. Delegates to [`ClassGraph::method_names_at`] against the
    /// class's *current* instance behaviour.
    pub fn instance_method_names(&self, class: ObjRef) -> std::collections::BTreeSet<String> {
        self.graph
            .method_names_at(self.graph.instance_behaviour_handle(class))
    }

    /// `class`'s own, unflattened instance-method names (no ancestor
    /// donations) -- see [`ClassGraph::own_instance_method_names`].
    pub fn own_instance_method_names(&self, class: ObjRef) -> std::collections::BTreeSet<String> {
        self.graph.own_instance_method_names(class)
    }

    /// `~method`'s own lookup: whether `name` is in `class`'s own,
    /// unflattened instance-method dictionary -- oracle's
    /// `RexxClass::method`, which retrieves from `instanceMethodDictionary`
    /// directly (`ClassClass.cpp:984`) and so answers nothing for an
    /// inherited, donated or class-side name. See
    /// [`ClassGraph::has_own_instance_method`].
    pub fn has_own_instance_method(&self, class: ObjRef, name: &str) -> bool {
        self.graph.has_own_instance_method(class, name)
    }

    /// What `~method` actually reads -- see [`ClassGraph::own_instance_slot`].
    pub fn own_instance_slot(&self, class: ObjRef, name: &str) -> Option<MethodSlot> {
        self.graph.own_instance_slot(class, name)
    }

    /// `class`'s own, unflattened class-method names -- see
    /// [`ClassGraph::own_class_method_names`].
    pub fn own_class_method_names(&self, class: ObjRef) -> std::collections::BTreeSet<String> {
        self.graph.own_class_method_names(class)
    }

    /// The class object's own (class-side) method set -- what `.class`'s
    /// class-behaviour merge (D44) donates on top of each class's own
    /// `::method ... class` definitions.
    pub fn class_method_names(&self, class: ObjRef) -> std::collections::BTreeSet<String> {
        self.graph
            .method_names_at(self.graph.class_behaviour_handle(class))
    }

    /// True once `scope` is folded into `class`'s current class-behaviour --
    /// see [`ClassGraph::has_scope_at`] for why this, not a method-presence
    /// check, is what witnesses [`MethodDict::merge`]'s scope-copying half
    /// of the metaclass merge (D44).
    pub fn class_behaviour_has_scope(&self, class: ObjRef, scope: ObjRef) -> bool {
        self.graph
            .has_scope_at(self.graph.class_behaviour_handle(class), scope)
    }

    /// True once `scope` is folded into `class`'s current instance
    /// behaviour -- the instance-side twin of
    /// [`Self::class_behaviour_has_scope`], and what
    /// `RexxObject::validateScopeOverride` (`classes/ObjectClass.cpp:1950`)
    /// asks of an ordinary receiver before a `target~name:scope` send may
    /// start its lookup at `scope`.
    pub fn instance_behaviour_has_scope(&self, class: ObjRef, scope: ObjRef) -> bool {
        self.graph
            .has_scope_at(self.graph.instance_behaviour_handle(class), scope)
    }

    /// An ordinary (unscoped) message resolution against `class`'s current
    /// instance behaviour: the scope the winning entry came from, and the
    /// method it names -- oracle's `RexxBehaviour::methodLookup`, which
    /// `RexxObject::messageSend` (`ObjectClass.cpp:866`) calls.
    ///
    /// The scope comes back because a caller needs it for more than the
    /// lookup: it is the class whose `~id` the oracle names in a native
    /// method's own traceback line, and it is the starting point a further
    /// scope-override send would take.
    pub fn lookup_instance_method(&self, class: ObjRef, name: &str) -> Option<(ObjRef, MethodId)> {
        self.graph
            .lookup_at(self.graph.instance_behaviour_handle(class), name)
    }

    /// An ordinary message resolution against the class object itself --
    /// `class`'s current class behaviour, which is what a send to the class
    /// rather than to one of its instances answers from.
    pub fn lookup_class_method(&self, class: ObjRef, name: &str) -> Option<(ObjRef, MethodId)> {
        self.graph
            .lookup_at(self.graph.class_behaviour_handle(class), name)
    }

    /// A scope-override message resolution against `class`'s current
    /// instance behaviour -- oracle's `RexxObject::superMethod`, which the
    /// scope-override `messageSend` (`ObjectClass.cpp:919`) calls. See
    /// [`MethodDict::lookup_from_scope`](crate::MethodDict::lookup_from_scope)
    /// for what `start_scope` selects.
    pub fn lookup_instance_method_from_scope(
        &self,
        class: ObjRef,
        name: &str,
        start_scope: ObjRef,
    ) -> Option<(ObjRef, MethodId)> {
        self.graph.lookup_from_scope_at(
            self.graph.instance_behaviour_handle(class),
            name,
            start_scope,
        )
    }

    /// A scope-override message resolution against the class object itself --
    /// the class-behaviour twin of [`Self::lookup_instance_method_from_scope`].
    ///
    /// `RexxObject::superMethod` reads the *receiver's* behaviour, and for a
    /// send to a class object that behaviour is the class behaviour, so the
    /// override applies on this side as well as the instance side.
    pub fn lookup_class_method_from_scope(
        &self,
        class: ObjRef,
        name: &str,
        start_scope: ObjRef,
    ) -> Option<(ObjRef, MethodId)> {
        self.graph
            .lookup_from_scope_at(self.graph.class_behaviour_handle(class), name, start_scope)
    }

    /// The scope a `SUPER` reference resolves to for a method found at `scope`
    /// in `class`'s instance behaviour -- `RexxObject::superScope`.
    pub fn instance_super_scope(&self, class: ObjRef, scope: ObjRef) -> Option<ObjRef> {
        self.graph
            .resolve_super_scope_at(self.graph.instance_behaviour_handle(class), scope)
    }

    /// The same for a method found in `class`'s class behaviour, which is
    /// what a class method's own `SUPER` reads.
    pub fn class_super_scope(&self, class: ObjRef, scope: ObjRef) -> Option<ObjRef> {
        self.graph
            .resolve_super_scope_at(self.graph.class_behaviour_handle(class), scope)
    }

    /// `~hasMethod` against the class's current instance behaviour.
    pub fn has_method(&self, class: ObjRef, name: &str) -> bool {
        self.graph.has_method(class, name)
    }

    /// `~hasMethod` sent to the class object itself.
    pub fn class_has_method(&self, class: ObjRef, name: &str) -> bool {
        self.graph.class_has_method(class, name)
    }

    /// Install a directly-added instance method with a freshly minted
    /// [`MethodId`] -- a bootstrap convenience standing in for
    /// `Setup.cpp`'s `AddMethod`/`AddProtectedMethod`/`AddPrivateMethod`/
    /// `AddUnguardedMethod` macros, none of whose visibility distinctions
    /// this crate models (Task 2: no probe needs them). Unlike
    /// [`ClassGraph::define`] (`~define`, which allocates a fresh handle so
    /// a pre-existing instance is unaffected), this is bootstrap-time
    /// population before any instance can exist, so the handle churn
    /// `define` costs is spent for no reason here -- accepted rather than
    /// building a second populate-then-cascade-once primitive, since no
    /// axis this task is guarded against (D35) ever calls it.
    ///
    /// Returns the minted [`MethodId`] -- added for Phase 5a Task 4's own
    /// caller, which records a `(program, directive)` pair against it so a
    /// later dispatch can find the method's own body; nothing in this crate
    /// itself needs the value back.
    pub fn add_instance_method(&mut self, class: ObjRef, name: &str) -> MethodId {
        let method = self.next_method_id();
        self.graph.define(class, name, method);
        method
    }

    /// Install a directly-added class (static) method -- oracle's
    /// `AddClassMethod`, matching [`ClassGraph::class_define`]'s own
    /// bootstrap-only semantics (no cascade, no handle reallocation).
    ///
    /// Returns the minted [`MethodId`], for the same reason
    /// [`Self::add_instance_method`] does.
    pub fn add_class_method(&mut self, class: ObjRef, name: &str) -> MethodId {
        let method = self.next_method_id();
        self.graph.class_define(class, name, method);
        method
    }

    /// `~inherit` -- see [`ClassGraph::inherit`]. Exposed here because R6's
    /// `mixinclass class` probe needs it: `inheritInstanceMethods` (below)
    /// never adds a superclass edge, so it cannot reach the metaclass side
    /// the way `~inherit` does (D44).
    pub fn inherit(&mut self, class: ObjRef, mixin: ObjRef) -> Result<(), InheritRefusal> {
        self.graph.inherit(class, mixin)
    }

    /// `~inherit` with a position -- see [`ClassGraph::inherit_at`].
    pub fn inherit_at(
        &mut self,
        class: ObjRef,
        mixin: ObjRef,
        position: Option<ObjRef>,
    ) -> Result<(), InheritRefusal> {
        self.graph.inherit_at(class, mixin, position)
    }

    /// `~uninherit` -- see [`ClassGraph::uninherit`].
    pub fn uninherit(&mut self, class: ObjRef, mixin: ObjRef) -> Result<(), InheritRefusal> {
        self.graph.uninherit(class, mixin)
    }

    /// `~define` with a method -- see [`ClassGraph::define`]. The
    /// [`MethodId`] is the caller's, not minted here: `~define` is handed a
    /// `Method` object whose identity the caller already has to keep, unlike
    /// [`Self::add_instance_method`]'s bootstrap population.
    pub fn define_instance_method(&mut self, class: ObjRef, name: &str, method: MethodId) {
        self.graph.define(class, name, method);
    }

    /// `~define` with the method omitted, and `Setup.cpp`'s `HideMethod` --
    /// see [`ClassGraph::hide`].
    pub fn hide_instance_method(&mut self, class: ObjRef, name: &str) {
        self.graph.hide(class, name);
    }

    /// `~delete`, and `Setup.cpp`'s `RemoveMethod` -- see
    /// [`ClassGraph::delete`].
    pub fn delete_instance_method(&mut self, class: ObjRef, name: &str) -> bool {
        self.graph.delete(class, name)
    }

    /// `~defineMethods` -- see [`ClassGraph::define_methods`].
    pub fn define_instance_methods(
        &mut self,
        class: ObjRef,
        methods: &[(String, Option<MethodId>)],
    ) {
        self.graph.define_methods(class, methods);
    }

    /// A fresh [`MethodId`] that names no method yet -- what a caller
    /// installing a method object of its own needs, since the identity a
    /// dictionary entry carries is this crate's to allocate and the body
    /// behind it is not.
    pub fn mint_method_id(&mut self) -> MethodId {
        self.next_method_id()
    }

    /// Oracle's `isRexxDefined` -- see [`ClassGraph::is_rexx_defined`].
    pub fn is_rexx_defined(&self, class: ObjRef) -> bool {
        self.graph.is_rexx_defined(class)
    }

    /// Oracle's `setRexxDefined` -- see [`ClassGraph::set_rexx_defined`].
    pub fn set_rexx_defined(&mut self, class: ObjRef) {
        self.graph.set_rexx_defined(class);
    }

    /// `inheritInstanceMethods` by name -- oracle's `Setup.cpp` bootstrap
    /// macro `InheritInstanceMethods(source)`. The oracle macro donates from
    /// `source`'s already-*flattened* instance behaviour, re-scoped to the
    /// recipient (`RexxBehaviour::inheritInstanceMethods`,
    /// `RexxBehaviour.cpp:350-361`) -- a bootstrap-only mechanism distinct
    /// from `RexxClass::inheritInstanceMethods` (the one
    /// [`ClassGraph::inherit_instance_methods`] models, which donates from
    /// `source`'s own *unflattened* dictionary). This crate has no reason to
    /// build a second donation mechanism: every donor a `Setup.cpp` class
    /// uses `InheritInstanceMethods` on (`Array`, donating to `Queue` at
    /// `Setup.cpp:775`; `IdentityTable`, donating to `StringTable` at
    /// `:881` and to `Table`/`Set`/`Relation` at `:861,908,958`;
    /// `StringTable`, donating to `Directory` at `:933`;
    /// `Relation`, donating to `Bag` at `:988`) is itself a **direct**
    /// subclass of `.Object`, so its own flattened instance set is exactly
    /// "Object's methods plus its own" -- and the recipient already gets
    /// Object's methods through its *own* ordinary ancestor cascade
    /// regardless of the donation. The two mechanisms therefore agree on
    /// the resulting **name set** (what this task's probes check) even
    /// though they disagree on which class a donated name's scope is
    /// attributed to (invisible to those probes, and not queryable through
    /// this crate's own public API either). `native_classes.rs`'s replay
    /// loop calls this for `Table`, `StringTable`, `Set`, `Directory`,
    /// `Relation` and `Bag`.
    pub fn inherit_instance_methods(&mut self, class: ObjRef, source: ObjRef) {
        self.graph.inherit_instance_methods(class, source);
    }

    /// See [`ClassGraph::refresh_class_behaviour`] for what it is for.
    /// Bootstrapping `.Class` is the caller.
    pub fn refresh_class_behaviour(&mut self, class: ObjRef) {
        self.graph.refresh_class_behaviour(class);
    }

    /// See [`ClassGraph::bootstrap_root_class_behaviour`] -- bootstrapping
    /// `.Object` needs this once, nothing else does.
    pub fn bootstrap_root_class_behaviour(&mut self, root: ObjRef, metaclass: ObjRef) {
        self.graph.bootstrap_root_class_behaviour(root, metaclass);
    }
}
