/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:         */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

//! Bootstraps the native class set `interpreter/memory/Setup.cpp` builds in
//! C++, before `CoreClasses.orx` ever runs -- D25's checklist, per-class
//! against the oracle rather than transcribed (see this task's brief and
//! ruling).
//!
//! `build.rs` derives two tables from `Setup.cpp` (`include!`d below):
//! `SETUP_CLASSES`, the checklist (`X::createInstance();` calls, C++ type
//! names), and `CLASS_DEFINITIONS`, every `StartClassDefinition(Name)`
//! block's own ordered `AddMethod`/`AddClassMethod`/
//! `InheritInstanceMethods`/`RemoveMethod`/`HideMethod` sequence, keyed by
//! `Name` -- the macro-block name, which is the class's real `~id` string
//! (`RexxClass::RexxClass`'s `id = new_string(className)`, and
//! `CLASS_CREATE(name)`'s `#name` stringification pass the identical token).
//!
//! [`CHECKLIST_TO_DEFINITION`] is this file's own hand-carried
//! correspondence between the two (the irregular entries are `RexxClass`
//! -> `Class`, `RexxInteger` -> `Integer`, `RexxString` -> `String`,
//! `RexxObject` -> `Object`; every other checklist token is its own block
//! name with a trailing `Class` stripped, or already identical). Not itself
//! mechanically derived -- the stripping rule has exceptions a regex would
//! silently mis-handle -- but asserted exhaustive and accurate against both
//! derived lists by `tests/native_classes_wiring.rs`'s deferral-table test,
//! so a `Setup.cpp` edit that adds, renames or removes a class fails loudly
//! here rather than silently drifting.
//!
//! [`DEFERRALS`] is the deferral table: every checklist entry this module
//! does not build natively, and why. None for "not needed yet" -- each
//! names a concrete missing mechanism:
//!
//! * `RexxInteger`, `NumberString` -- each **masquerades as another class
//!   for `~class`**, measured: `CLASS_CREATE_SPECIAL(Integer, "String",
//!   RexxIntegerClass)` and the same for `NumberString`
//!   (`IntegerClass.cpp:2066`, `NumberStringClass.cpp:74`, whose own comment
//!   is "the number string class lies about its identity"), so
//!   `12345~class~id` and `(1.5)~class~id` both answer `"String"`. That
//!   needs a per-value class-identity override this registry does not build.
//!   Neither is `.NAME`-reachable either: measured, `value('.INTEGER')` and
//!   `value('.NUMBERSTRING')` both return the literal string
//!   `".INTEGER"`/`".NUMBERSTRING"`, an unresolved environment symbol.
//!
//! **A class `.NAME` cannot reach is still built.** `EndSpecialClassDefinition`
//! registers with `addToSystem` rather than `completeSystemClass`, which is
//! the kernel directory instead of `.environment`, and
//! [`ClassRegistry::define_system_class`] is that directory here: such a
//! class answers [`ClassRegistry::system_lookup`] and is absent from
//! [`ClassRegistry::registered`], so `.environment` never gains its name.
//! `RexxInfo` is the case with an instance in a program's hands -- measured,
//! `.RexxInfo` renders as `a RexxInfo` and `.RexxInfo~class~id` is
//! `RexxInfo`, while `::CLASS K SUBCLASS RexxInfo` is `99.949 "REXXINFO" is
//! not a valid class` -- and which blocks take this route is read off
//! `Setup.cpp`'s own closing macro (`ClassDefinition::system_only`), never
//! listed here.
//!
//! **`Queue`, `Stem` and `VariableReference` are the classes that need
//! [`Op::RemoveInstanceMethod`] and [`Op::HideInstanceMethod`]**, the two
//! operations [`replay`] carries beyond adding and donating. `Queue`
//! donates `Array`'s instance methods and then takes several of them back
//! off (`Setup.cpp:792`-`:804`); `VariableReference` (`:1307`-`:1312`) and
//! `Stem` (`:1399`-`:1404`) each hide `=`, `==`, `\=`, `\==`, `<>` and
//! `><`, which routes those names to `UNKNOWN`. Removal and hiding are
//! separate [`crate::MethodDict`] operations because the oracle answers
//! them apart: measured at rc 0, `.Queue~method("SORT")` raises 97.1 where
//! `.Stem~method("==")` prints `The NIL object`.
//!
//! **Ruling R8 (task review, 2026-08-16): the classes `CoreClasses.orx`
//! later mutates are built here, not deferred.** `RexxString`/`ArrayClass`/
//! `TableClass`/`IdentityTable`/`RelationClass`/`StringTable`/`DirectoryClass`/
//! `SetClass`/`BagClass`/`ListClass`/`MessageClass`/`SupplierClass` are
//! built, not on the grounds that `CoreClasses.orx` leaves what the *live,
//! fully-booted* oracle answers for them unchanged from `Setup.cpp` alone --
//! it does not -- but because deferring them on that basis is circular:
//! `CoreClasses.orx` **mutates** these classes, which means they must
//! already exist as native class objects before the prologue can touch
//! them. What is unbuildable today is their *post-prologue* state, not the
//! class itself, and deferring the class would leave whichever task runs
//! the prologue (Task 13) also having to create them first -- work outside
//! that task's scope. So every one of them is built here, from `Setup.cpp`
//! alone, exactly like every other checklist entry not in [`DEFERRALS`].
//!
//! **The verification bar for the classes this ruling names is deliberately
//! not the same as for the ones `CoreClasses.orx` never touches.** Byte-identity
//! against the live oracle is impossible
//! pre-prologue: the only oracle this crate can run is fully booted, past
//! the point these classes are still in the state this module builds.
//! `tests/native_classes_wiring.rs` instead: (a) asserts each one's derived
//! `own_instance_method_names`/`own_class_method_names` exactly, against a
//! set read from `Setup.cpp` -- checkable today, and the state each class
//! must be in when the prologue starts; (b) *measures* (not assumes) that
//! this crate's flattened set is a subset of the live oracle's for a
//! representative sample, and records the measured difference against the
//! `CoreClasses.orx` line that donates it: `~inherit`s of `OrderedCollection`,
//! `MapCollection`, `SetCollection`, `Comparable`, `MessageNotification` and
//! `AlarmNotification` (all `::CLASS MIXINCLASS` definitions **inside
//! `CoreClasses.orx` itself**, confirmed by `grep`), plus
//! the `~inheritInstanceMethods` donations, none of which add a superclass
//! edge: `.SupplierMixin` to `Supplier` (`:80`), `.ManyItemMixin` to
//! `Relation` (`:82`) and to `Bag` (`:83`), `.SetMixin` to `Set` (`:85`),
//! and `.BagMixin` to `Bag` again (`:87`) -- `Bag` receives two donors.
//! Full post-prologue correctness for the classes R8 names -- their
//! `~superClasses`, their complete flattened method set -- is explicitly
//! **Task 13's**, not asserted here.
//!
//! Every checklist entry not deferred is built: superclass `.Object` (root
//! excepted), metaclass `.Class` (`.Class` itself excepted, self-
//! referentially) -- `buildFinalClassBehaviour`'s own unconditional
//! `metaClass = TheClassClass` and `superClasses->addLast(TheObjectClass)`
//! (`ClassClass.cpp:713`, `:725`), the single bootstrap path every native
//! class goes through. `InheritInstanceMethods(source)` operations replay
//! through [`crate::ClassRegistry::donate_instance_methods`], which is
//! `RexxBehaviour::inheritInstanceMethods` (`RexxBehaviour.cpp:350`) --
//! what the macro actually calls, and **not**
//! `RexxClass::inheritInstanceMethods`, the Rexx-callable method of the same
//! name. The two differ in whether the donor's own dictionary is rewritten,
//! and [`crate::MethodDict::replace_methods_from`] carries what conflating
//! them costs. The `InheritInstanceMethods` users are the blocks
//! `CLASS_DEFINITIONS` carries an `Op::InheritInstanceMethods` for.

use crate::class_graph::ClassKind;
use crate::registry::ClassRegistry;
use rexx_core::ObjRef;

include!(concat!(env!("OUT_DIR"), "/setup_classes.rs"));

/// One checklist entry this module declines to build natively.
pub struct Deferral {
    /// The `SETUP_CLASSES` (checklist) token -- the C++ type name.
    pub setup_class: &'static str,
    /// What would have to exist for this task to build it instead.
    pub reason: &'static str,
}

/// Checklist token -> `CLASS_DEFINITIONS` block name, for every checklist
/// entry -- see the module doc comment for why this is hand-carried
/// rather than derived.
const CHECKLIST_TO_DEFINITION: &[(&str, &str)] = &[
    ("RexxClass", "Class"),
    ("RexxInteger", "Integer"),
    ("RexxString", "String"),
    ("RexxObject", "Object"),
    ("PointerClass", "Pointer"),
    ("BufferClass", "Buffer"),
    ("ArrayClass", "Array"),
    ("TableClass", "Table"),
    ("IdentityTable", "IdentityTable"),
    ("RelationClass", "Relation"),
    ("StringTable", "StringTable"),
    ("DirectoryClass", "Directory"),
    ("SetClass", "Set"),
    ("BagClass", "Bag"),
    ("ListClass", "List"),
    ("QueueClass", "Queue"),
    ("NumberString", "NumberString"),
    ("MethodClass", "Method"),
    ("RoutineClass", "Routine"),
    ("PackageClass", "Package"),
    ("RexxContext", "RexxContext"),
    ("StemClass", "Stem"),
    ("SupplierClass", "Supplier"),
    ("MessageClass", "Message"),
    ("MutableBuffer", "MutableBuffer"),
    ("WeakReference", "WeakReference"),
    ("StackFrameClass", "StackFrame"),
    ("RexxInfo", "RexxInfo"),
    ("VariableReference", "VariableReference"),
    ("EventSemaphoreClass", "EventSemaphore"),
    ("MutexSemaphoreClass", "MutexSemaphore"),
];

/// The deferral table -- see the module doc comment for each reason in full.
const DEFERRALS: &[Deferral] = &[
    Deferral {
        setup_class: "RexxInteger",
        reason: "CLASS_CREATE_SPECIAL(Integer, \"String\", RexxIntegerClass): an Integer \
                 value's ~class answers \"String\", not \"Integer\" (measured, 12345~class~id). \
                 Needs a per-value class-identity override this registry does not build.",
    },
    Deferral {
        setup_class: "NumberString",
        reason: "CLASS_CREATE_SPECIAL(NumberString, \"String\", RexxClass), same masquerade \
                 (measured, (1.5)~class~id answers \"String\"); own C++ comment: \"the number \
                 string class lies about its identity\". Same missing mechanism as RexxInteger.",
    },
];

/// The checklist, unfiltered -- `SETUP_CLASSES`, `build.rs`-derived.
pub fn setup_class_names() -> &'static [&'static str] {
    SETUP_CLASSES
}

/// The deferral table.
pub fn deferred_classes() -> &'static [Deferral] {
    DEFERRALS
}

fn definition_for(block_name: &str) -> &'static ClassDefinition {
    CLASS_DEFINITIONS
        .iter()
        .find(|d| d.name == block_name)
        .unwrap_or_else(|| panic!("no CLASS_DEFINITIONS block named {block_name:?}"))
}

/// `Setup.cpp:1809`'s `TheClassClass->removeSetupMethods()`: `TheClassClass`
/// specifically, applied at image-save time to delete exactly the names
/// below from `.Class`'s own instance methods (D39).
///
/// **Two registries are built from this list and each reaches the same final
/// state a different way.** [`native_classes`] never adds these names, which
/// is what every consumer but one wants: the shipped image does not have
/// them, and measured, the live oracle's `.class~instancemethods(.class)`
/// includes neither. [`native_classes_for_bootstrap`] adds them and
/// [`remove_setup_methods`] deletes them when the library's prologue is
/// done, which is the C++'s own three steps.
///
/// The deletion needs a walk as well as the delete, because a name taken out
/// of `.Class`'s dictionary is already in every class object's class
/// behaviour by then, through the metaclass merge
/// ([`crate::ClassGraph::bootstrap_root_class_behaviour`] is where
/// `.Object`'s picks `.Class`'s instance methods up here). That is why the
/// oracle walks `.Object`'s whole subclass tree deleting from each behaviour
/// as well (`ClassClass.cpp:923`-`:941`), and
/// [`ClassRegistry::delete_instance_method`] cascades over the *instance*
/// side alone, matching `RexxClass::deleteMethod`.
///
/// Scoped to the `"Class"` block alone in [`replay`], matching
/// `removeSetupMethods` being `TheClassClass`'s own method, not a blanket
/// rule -- harmless today either way, since no other block's `AddMethod`
/// list names either string, but scoping it is what keeps that true rather
/// than assuming it.
const REMOVED_BY_IMAGE_SAVE: &[&str] = &["DefineClassMethod", "InheritInstanceMethods"];

/// The names [`remove_setup_methods`] deletes, in the spelling a dictionary
/// is keyed by. `Setup.cpp` writes them mixed-case and every lookup upcases,
/// so a caller registering an implementation for one needs this spelling and
/// not [`REMOVED_BY_IMAGE_SAVE`]'s.
pub fn setup_method_names() -> Vec<String> {
    REMOVED_BY_IMAGE_SAVE
        .iter()
        .map(|name| name.to_ascii_uppercase())
        .collect()
}

fn replay(
    registry: &mut ClassRegistry,
    class: rexx_core::ObjRef,
    def: &ClassDefinition,
    keep_setup_methods: bool,
) {
    for op in def.ops {
        match op {
            Op::AddClassMethod(name) => {
                registry.add_class_method(class, name);
            }
            Op::AddInstanceMethod(name) => {
                if !keep_setup_methods
                    && def.name == "Class"
                    && REMOVED_BY_IMAGE_SAVE.contains(name)
                {
                    continue;
                }
                registry.add_instance_method(class, name);
            }
            Op::AddPrivateInstanceMethod(name) => {
                registry.add_private_instance_method(class, name);
            }
            Op::InheritInstanceMethods(source) => {
                let source_id = registry.lookup(source).unwrap_or_else(|| {
                    panic!("InheritInstanceMethods({source}) before {source} was built")
                });
                registry.donate_instance_methods(class, source_id);
            }
            Op::RemoveInstanceMethod(name) => {
                registry.delete_instance_method(class, name);
            }
            Op::HideInstanceMethod(name) => {
                registry.hide_instance_method(class, name);
            }
        }
    }
}

/// Build the native class registry: every checklist entry not in
/// [`deferred_classes`], wired as `Setup.cpp` wires it.
///
/// `.Object` and `.Class` bootstrap first and out of checklist order
/// (`RexxClass::createInstance()` precedes `RexxInteger::createInstance()`
/// in the checklist, but `.Class`'s own superclass is `.Object`, so
/// `.Object` must already be a graph entry before `.Class` can name it --
/// the real oracle resolves this the same circularity a different way, its
/// own dedicated two-class bootstrap constructor
/// (`ClassClass.cpp:654-748`/`:1854-1870`) rather than the ordinary
/// `subclass()` path every other primitive class goes through).
pub fn native_classes(mint: &mut dyn FnMut() -> ObjRef) -> ClassRegistry {
    build(false, mint)
}

/// [`native_classes`] with the two methods `removeSetupMethods` deletes
/// still present -- the state the C++ image build runs `CoreClasses.orx` in,
/// before `Setup.cpp:1809` strips them.
///
/// **The only caller is the library bootstrap**, which needs
/// `.String~defineClassMethod` and `.supplier~inheritInstanceMethods` to
/// resolve while it runs and calls [`remove_setup_methods`] when it is done.
/// Every other consumer wants [`native_classes`], which is the shipped state.
pub fn native_classes_for_bootstrap(mint: &mut dyn FnMut() -> ObjRef) -> ClassRegistry {
    build(true, mint)
}

fn build(keep_setup_methods: bool, mint: &mut dyn FnMut() -> ObjRef) -> ClassRegistry {
    let mut registry = ClassRegistry::new();

    let class_id = mint();
    let object_id = registry.define_class(mint(), "Object", None, ClassKind::Regular, class_id);
    registry.define_reserved(
        class_id,
        "Class",
        Some(object_id),
        ClassKind::Regular,
        class_id,
    );
    // `.Class` is where metaclass-ness starts, and nothing derives it from a
    // metaclass -- its superclass is `.Object`. The oracle seeds it by hand
    // in the same dedicated bootstrap constructor (`ClassClass.cpp:744`-
    // `:747`); see `ClassGraph::bootstrap_metaclass`. Which of the classes
    // below inherits it is `ClassGraph::define_class`'s decision, read off
    // the superclass each is given here.
    registry.bootstrap_metaclass(class_id);
    replay(
        &mut registry,
        object_id,
        definition_for("Object"),
        keep_setup_methods,
    );
    replay(
        &mut registry,
        class_id,
        definition_for("Class"),
        keep_setup_methods,
    );
    // `.Class`'s own class-behaviour self-merge (D44's self-reference) ran
    // once already, inside `define_reserved`'s initial cascade, against its
    // *own* still-empty instance methods -- see
    // `ClassGraph::refresh_class_behaviour`'s doc comment for why this
    // second, explicit rebuild is the one bootstrap-only step every other
    // class does not need.
    registry.refresh_class_behaviour(class_id);
    // `.Object` bootstraps through a dedicated C++ path that unconditionally
    // merges `.Class`'s instance methods into `.Object`'s own class-behaviour
    // too (`.object~hasmethod('SUBCLASS')` measures `1`) -- see
    // `ClassGraph::bootstrap_root_class_behaviour`'s doc comment for why the
    // ordinary is-this-the-root cascade guard cannot produce that on its own.
    registry.bootstrap_root_class_behaviour(object_id, class_id);
    registry.set_rexx_defined(object_id);
    registry.set_rexx_defined(class_id);

    // `CLASS_DEFINITIONS` is in `Setup.cpp`'s own file order, which is a real
    // dependency order (a donor's `StartClassDefinition` block precedes
    // every recipient that `InheritInstanceMethods` from it) and *differs*
    // from the checklist's `createInstance()` order (`TableClass` precedes
    // `IdentityTable` there, but `IdentityTable`'s block precedes `Table`'s
    // -- `Table` donates from it). Replaying in checklist order instead
    // would call `donate_instance_methods` against a donor not yet built.
    for def in CLASS_DEFINITIONS {
        if def.name == "Object" || def.name == "Class" {
            continue; // already bootstrapped above
        }
        let setup_class = CHECKLIST_TO_DEFINITION
            .iter()
            .find(|(_, block_name)| *block_name == def.name)
            .unwrap_or_else(|| {
                panic!(
                    "CLASS_DEFINITIONS block {:?} has no checklist entry",
                    def.name
                )
            })
            .0;
        if DEFERRALS.iter().any(|d| d.setup_class == setup_class) {
            continue;
        }
        // `def.system_only` is which closing macro `Setup.cpp` used, and it
        // decides the directory alone: a system class is built, wired and
        // `REXX_DEFINED` exactly like every other entry here, and differs
        // only in answering `system_lookup` where the rest answer `lookup`.
        let id = if def.system_only {
            registry.define_system_class(
                mint(),
                def.name,
                Some(object_id),
                ClassKind::Regular,
                class_id,
            )
        } else {
            registry.define_class(
                mint(),
                def.name,
                Some(object_id),
                ClassKind::Regular,
                class_id,
            )
        };
        replay(&mut registry, id, def, keep_setup_methods);
        // `RexxClass::liveGeneral` sets `REXX_DEFINED` on every class it
        // reaches while the image is being prepared
        // (`ClassClass.cpp:136`-`:142`), which is every class this function
        // builds. Set here, and beside the two bootstrapped above, rather
        // than by a sweep over the finished registry: the sweep's only
        // source of classes is a `HashMap`, and a loop whose order is the
        // map's is a loop that could come to matter.
        registry.set_rexx_defined(id);
    }

    registry
}

/// `RexxClass::removeSetupMethods` (`classes/ClassClass.cpp:923`-`:941`):
/// delete [`REMOVED_BY_IMAGE_SAVE`]'s names from `.Class`'s own instance
/// dictionary, and from every behaviour that merged them.
///
/// **Both halves, because deleting from `.Class` alone does not reach them.**
/// A name in `.Class`'s instance dictionary is already in every class
/// object's *class* behaviour by this point, through the metaclass merge, and
/// `ClassRegistry::delete_instance_method` cascades over the instance side
/// alone -- which is `RexxClass::deleteMethod` and is exactly why the oracle
/// walks `.Object`'s whole subclass tree as a second step. The walk here is
/// that tree, from `.Object` down, so it visits the classes a `::CLASS`
/// directive installed as well as the ones this file built.
pub fn remove_setup_methods(registry: &mut ClassRegistry) {
    let class = registry
        .lookup("Class")
        .expect("the registry always holds .Class");
    let object = registry
        .lookup("Object")
        .expect("the registry always holds .Object");
    for name in &setup_method_names() {
        registry.delete_instance_method(class, name);
    }
    // The same three steps [`build`] wires the two bootstrap classes with,
    // in the same order, because the walk in the middle passes through both
    // of them and the ordinary cascade is wrong for each: `.Class`'s class
    // behaviour is its own instance behaviour merged into itself, and
    // `.Object`'s needs the root merge the cascade's is-this-the-root guard
    // deliberately withholds. Measured, with the root merge left out:
    // `.Object~superClass` is 97.1 `does not understand message
    // "SUPERCLASS"` where the oracle answers `The NIL object`.
    registry.refresh_class_behaviour(class);
    let mut pending = vec![object];
    let mut seen = Vec::new();
    while let Some(next) = pending.pop() {
        if seen.contains(&next) {
            continue;
        }
        seen.push(next);
        pending.extend_from_slice(registry.subclasses(next));
        registry.refresh_class_behaviour(next);
    }
    registry.bootstrap_root_class_behaviour(object, class);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The deferral table as a test over the derived list, verbatim (the
    /// brief's own done-when condition): every `SETUP_CLASSES` checklist
    /// entry is in [`CHECKLIST_TO_DEFINITION`] exactly once, and every one
    /// of those is *either* built by [`native_classes`] *or* named in
    /// [`DEFERRALS`] -- never both, never neither. A `Setup.cpp` edit that
    /// adds, renames, or removes a `createInstance()` call fails this test
    /// rather than silently landing outside both sets.
    #[test]
    fn every_checklist_entry_is_mapped_and_either_built_or_deferred() {
        for &checklist_name in SETUP_CLASSES {
            let mapped: Vec<_> = CHECKLIST_TO_DEFINITION
                .iter()
                .filter(|(token, _)| *token == checklist_name)
                .collect();
            assert_eq!(
                mapped.len(),
                1,
                "{checklist_name:?} must appear in CHECKLIST_TO_DEFINITION exactly once, found {}",
                mapped.len()
            );
            let block_name = mapped[0].1;
            assert!(
                CLASS_DEFINITIONS.iter().any(|d| d.name == block_name),
                "{checklist_name:?} maps to block {block_name:?}, which CLASS_DEFINITIONS does not have"
            );

            let deferral_count = DEFERRALS
                .iter()
                .filter(|d| d.setup_class == checklist_name)
                .count();
            assert!(
                deferral_count <= 1,
                "{checklist_name:?} appears in DEFERRALS {deferral_count} times, want at most 1"
            );
        }
        // Every CHECKLIST_TO_DEFINITION entry corresponds to a real checklist
        // name -- catches a stale mapping entry after a Setup.cpp rename.
        for (token, _) in CHECKLIST_TO_DEFINITION {
            assert!(
                SETUP_CLASSES.contains(token),
                "{token:?} is in CHECKLIST_TO_DEFINITION but not in the derived checklist"
            );
        }
        // Every DEFERRALS entry names a real checklist token.
        for d in DEFERRALS {
            assert!(
                SETUP_CLASSES.contains(&d.setup_class),
                "deferral {:?} names a token the derived checklist does not have",
                d.setup_class
            );
            assert!(
                !d.reason.is_empty(),
                "{:?}'s deferral reason is empty",
                d.setup_class
            );
        }
        assert_eq!(
            CHECKLIST_TO_DEFINITION.len(),
            SETUP_CLASSES.len(),
            "CHECKLIST_TO_DEFINITION must cover the whole checklist, one entry each"
        );
    }

    /// `native_classes()` actually builds exactly the checklist entries this
    /// module claims to build, **in the directory `Setup.cpp`'s own closing
    /// macro puts each in** -- not the deferred ones, and nothing extra.
    ///
    /// Asserted as one of exactly three states per entry rather than as two
    /// booleans, because the failure this replaces is a class landing in
    /// *both* tables or in neither: a system class that also answered
    /// `lookup` would put its name into `.environment` and make `.RexxInfo`
    /// render as `The RexxInfo class`, which is measurably not what the
    /// oracle prints.
    #[test]
    fn every_checklist_entry_is_registered_in_the_directory_setup_cpp_names() {
        let mut next: u32 = 1;
        let registry = native_classes(&mut || {
            next += 1;
            ObjRef::heap(next, 0)
        });
        for (token, block_name) in CHECKLIST_TO_DEFINITION {
            let upper = block_name.to_ascii_uppercase();
            let environment = registry.lookup(&upper).is_some();
            let system = registry.system_lookup(&upper).is_some();
            let want = if DEFERRALS.iter().any(|d| d.setup_class == *token) {
                (false, false)
            } else if definition_for(block_name).system_only {
                (false, true)
            } else {
                (true, false)
            };
            assert_eq!(
                (environment, system),
                want,
                "{token:?} (block {block_name:?}) is in the wrong directory: \
                 (environment, system) reads {:?}",
                (environment, system)
            );
        }
    }

    /// The three-state test above is only as good as `system_only` telling
    /// blocks apart, so this is the discriminator's own witness: a class
    /// closed by `EndSpecialClassDefinition` and one closed by
    /// `EndClassDefinition`, read off the derived table.
    ///
    /// Without it, a `system_only` that answered `false` everywhere would
    /// leave every assertion above satisfied by the same arm it took before
    /// the split existed.
    #[test]
    fn system_only_separates_the_two_closing_macros() {
        assert!(
            definition_for("RexxInfo").system_only,
            "Setup.cpp:1285 closes the RexxInfo block with EndSpecialClassDefinition"
        );
        assert!(
            !definition_for("StackFrame").system_only,
            "Setup.cpp closes the StackFrame block with EndClassDefinition"
        );
    }
}
