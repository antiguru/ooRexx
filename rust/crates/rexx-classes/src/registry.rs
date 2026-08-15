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

use crate::class_graph::{ClassGraph, ClassKind};
use crate::method_dict::MethodId;
use rexx_core::ObjRef;
use std::collections::HashMap;

pub struct ClassRegistry {
    graph: ClassGraph,
    next_id: u32,
    next_method: u32,
    /// Id string as declared (`~id`), keyed by identity.
    names: HashMap<ObjRef, String>,
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
            by_name: HashMap::new(),
        }
    }

    /// Allocate a fresh identity without registering it as a class yet --
    /// what bootstrapping `.Object` and `.Class` needs, since each names the
    /// other's id before either exists (see
    /// [`ClassGraph::define_class`]'s own doc comment for why a forward
    /// reference like this is safe).
    pub fn reserve_id(&mut self) -> ObjRef {
        let id = ObjRef::heap(self.next_id, 0);
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

    /// `~class`. For a class object specifically, oracle's `~class` and
    /// `~metaClass` coincide: a class's own `behaviour->setOwningClass`
    /// argument is always its metaclass (`ClassClass.cpp:1615`,
    /// `:736`/`:803` for the primitive bootstrap path), so the class an
    /// object answers to via `~class` and the class it names as its
    /// `~metaClass` are the same value, measured (`.string~class~id` and
    /// `.string~metaclass~id` both `"Class"`). This does not hold for an
    /// ordinary (non-class) object, whose `~class` is unrelated to any
    /// metaclass concept -- out of scope here, since this registry models
    /// only class objects.
    pub fn class_of(&self, class: ObjRef) -> ObjRef {
        self.graph.metaclass(class)
    }

    /// `~metaClass`.
    pub fn metaclass(&self, class: ObjRef) -> ObjRef {
        self.graph.metaclass(class)
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
    pub fn add_instance_method(&mut self, class: ObjRef, name: &str) {
        let method = self.next_method_id();
        self.graph.define(class, name, method);
    }

    /// Install a directly-added class (static) method -- oracle's
    /// `AddClassMethod`, matching [`ClassGraph::class_define`]'s own
    /// bootstrap-only semantics (no cascade, no handle reallocation).
    pub fn add_class_method(&mut self, class: ObjRef, name: &str) {
        let method = self.next_method_id();
        self.graph.class_define(class, name, method);
    }

    /// `~inherit` -- see [`ClassGraph::inherit`]. Exposed here because R6's
    /// `mixinclass class` probe needs it: `inheritInstanceMethods` (below)
    /// never adds a superclass edge, so it cannot reach the metaclass side
    /// the way `~inherit` does (D44).
    pub fn inherit(&mut self, class: ObjRef, mixin: ObjRef) {
        self.graph.inherit(class, mixin);
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
    /// uses `InheritInstanceMethods` on (`Array`, `IdentityTable`,
    /// `StringTable`, `Relation`) is itself a **direct** subclass of
    /// `.Object`, so its own flattened instance set is exactly "Object's
    /// methods plus its own" -- and the recipient already gets Object's
    /// methods through its *own* ordinary ancestor cascade regardless of the
    /// donation. The two mechanisms therefore agree on the resulting **name
    /// set** (what the six probes check) even though they disagree on which
    /// class a donated name's scope is attributed to (invisible to every one
    /// of the six probes, and not queryable through this crate's own public
    /// API either). Recorded here rather than silently: this is a
    /// substitution, not an oversight -- though as it happens, every
    /// `Setup.cpp` class that uses `InheritInstanceMethods` is itself in
    /// `native_classes.rs`'s deferral table for an unrelated reason
    /// (`CoreClasses.orx` changes its `~superClasses`/method set further),
    /// so this substitution is not currently exercised by the native class
    /// set; it is recorded for whichever later task lifts one of those
    /// deferrals.
    pub fn inherit_instance_methods(&mut self, class: ObjRef, source: ObjRef) {
        self.graph.inherit_instance_methods(class, source);
    }

    /// See [`ClassGraph::refresh_class_behaviour`] -- bootstrapping `.Class`
    /// needs this once, nothing else does.
    pub fn refresh_class_behaviour(&mut self, class: ObjRef) {
        self.graph.refresh_class_behaviour(class);
    }

    /// See [`ClassGraph::bootstrap_root_class_behaviour`] -- bootstrapping
    /// `.Object` needs this once, nothing else does.
    pub fn bootstrap_root_class_behaviour(&mut self, root: ObjRef, metaclass: ObjRef) {
        self.graph.bootstrap_root_class_behaviour(root, metaclass);
    }
}
