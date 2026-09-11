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
pub fn native_classes(mint: &mut dyn FnMut() -> ObjRef) -> ClassRegistry {
    build(false, mint)
}

/// [`native_classes`] with the two methods `removeSetupMethods` deletes
/// still present -- the state the C++ image build runs `CoreClasses.orx` in,
/// before `Setup.cpp:1809` strips them.
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
