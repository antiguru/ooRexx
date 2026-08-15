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
//! correspondence between the two (four of the thirty-one differ: `RexxClass`
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
//! names a concrete missing mechanism or a fact the oracle itself
//! contradicts a bare `Setup.cpp` reading:
//!
//! * `RexxInteger`, `NumberString`, `RexxInfo` -- **not reachable through
//!   this registry's own `.NAME` lookup at all**, measured directly:
//!   `value('.INTEGER')` and `value('.NUMBERSTRING')` both return the
//!   literal string `".INTEGER"`/`".NUMBERSTRING"` (an unresolved
//!   environment symbol), and `value('.REXXINFO')` returns a live
//!   `RexxInfo` *instance*, not a class. `RexxInteger`/`NumberString` are
//!   built via `CLASS_CREATE_SPECIAL` and registered with `addToSystem`
//!   (`IntegerClass.cpp:2066`, `NumberStringClass.cpp:74` -- the latter's
//!   own comment: "the number string class lies about its identity";
//!   `12345~class~id` and `(1.5)~class~id` both answer `"String"`).
//!   `RexxInfo` is also `addToSystem`-only (`EndSpecialClassDefinition`);
//!   only a pre-built *instance* is `addToEnvironment`'d
//!   (`Setup.cpp:1737`). This registry models environment-reachable class
//!   objects; none of these three is one.
//! * `QueueClass` -- Setup.cpp donates Array's instance methods
//!   (`InheritInstanceMethods(Array)`) then removes nine of them
//!   (`Dimension`, `Dimensions`, `Fill`, `sort`, `sortWith`, `stableSort`,
//!   `stableSortWith`, `makeString`, `toString`). `MethodDict` does not
//!   model method removal at all -- Task 2's own scope decision
//!   (`method_dict.rs`'s doc comment: "no probe this task specifies needs
//!   it"). Building `.Queue` here would leave those nine names present.
//! * `VariableReference`, `StemClass` -- Setup.cpp hides `=`, `==`, `\=`,
//!   `\==`, `<>`, `><` (`HideMethod`) so they redirect to `UNKNOWN`; the
//!   same not-modelled removal/tombstone mechanism as `Queue`'s.
//! * `RexxString`, `ArrayClass`, `TableClass`, `IdentityTable`,
//!   `RelationClass`, `StringTable`, `DirectoryClass`, `SetClass`,
//!   `BagClass`, `ListClass`, `MessageClass`, `SupplierClass` --
//!   **`CoreClasses.orx`'s own prologue changes what the live oracle
//!   answers for these, past what `Setup.cpp` alone builds**, measured
//!   directly against the running oracle (`~superClasses` and a
//!   scope-exact `~instanceMethods(class)` query), not inferred: `.String`
//!   gains a class-side extension (`ALNUM`/`ALPHA`/.../`XDIGIT`) with no
//!   `~inherit` at all; `.Array`/`.List`/`.Queue` each `~inherit`
//!   `.OrderedCollection`; `.IdentityTable`/`.Table`/`.StringTable`/
//!   `.Directory`/`.Relation`/`.Set`/`.Bag`/`.Stem` each `~inherit`
//!   `.MapCollection`; `.Set`/`.Bag` additionally `~inherit`
//!   `.SetCollection`; `.Message` `~inherit`s `.MessageNotification` and
//!   `.AlarmNotification`; `.Supplier`
//!   `~inheritInstanceMethods(.SupplierMixin)` (donating `allItems`,
//!   `allIndexes`, `getArrays`, `supplier` without a superclass edge, so
//!   `~superClasses` alone would miss it -- exactly the hazard this
//!   task's brief names). None of `OrderedCollection`, `MapCollection`,
//!   `SetCollection`, `Comparable`, `MessageNotification`,
//!   `AlarmNotification` or `SupplierMixin` is a `Setup.cpp`/C++ class --
//!   each is a `::CLASS ... MIXINCLASS` **defined inside
//!   `CoreClasses.orx` itself** (`interpreter/RexxClasses/CoreClasses.orx`,
//!   confirmed by `grep`), so building any of these twelve classes
//!   correctly needs the prologue this task cannot run (D25's own
//!   qualification, Task 13's). A `Setup.cpp`-only build would answer
//!   `~superClasses`/the method set wrong for every one of them --
//!   exactly "transcribing a plausible-looking class list ... and
//!   asserting it against itself," which the ruling forbids.
//!
//! Every other checklist entry is built: superclass `.Object` (root
//! excepted), metaclass `.Class` (`.Class` itself excepted, self-
//! referentially) -- `buildFinalClassBehaviour`'s own unconditional
//! `metaClass = TheClassClass` and `superClasses->addLast(TheObjectClass)`
//! (`ClassClass.cpp:713`, `:725`), the single bootstrap path every one of
//! these thirteen classes goes through, measured against the oracle to have
//! no further `CoreClasses.orx` extension. `InheritInstanceMethods(source)`
//! operations replay through [`crate::ClassRegistry::inherit_instance_methods`]
//! (`RexxClass::inheritInstanceMethods`'s donate-by-own-dictionary
//! semantics) rather than `RexxBehaviour::inheritInstanceMethods`'s
//! bootstrap-only donate-by-flattened-behaviour semantics
//! (`RexxBehaviour.cpp:350-361`, what `Setup.cpp`'s macro actually calls) --
//! see [`crate::ClassRegistry::inherit_instance_methods`]'s own doc comment
//! for why the two agree on every name this task's probes check. (None of
//! the thirteen native classes actually uses `InheritInstanceMethods` --
//! every donor `Setup.cpp` names one is itself deferred -- so this
//! substitution is recorded for completeness rather than currently
//! exercised.)

use crate::class_graph::ClassKind;
use crate::registry::ClassRegistry;

include!(concat!(env!("OUT_DIR"), "/setup_classes.rs"));

/// One checklist entry this module declines to build natively.
pub struct Deferral {
    /// The `SETUP_CLASSES` (checklist) token -- the C++ type name.
    pub setup_class: &'static str,
    /// What would have to exist for this task to build it instead.
    pub reason: &'static str,
}

/// Checklist token -> `CLASS_DEFINITIONS` block name, for all thirty-one
/// entries -- see the module doc comment for why this is hand-carried
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
        reason: "not .NAME-reachable: value('.INTEGER') returns the literal string \
                 \".INTEGER\" (unresolved environment symbol), measured. Also \
                 CLASS_CREATE_SPECIAL(Integer, \"String\", RexxIntegerClass): an Integer \
                 value's ~class answers \"String\", not \"Integer\" (measured, 12345~class~id). \
                 Needs a per-value class-identity override this registry does not build.",
    },
    Deferral {
        setup_class: "NumberString",
        reason: "not .NAME-reachable: value('.NUMBERSTRING') returns the literal string \
                 \".NUMBERSTRING\" (unresolved environment symbol), measured. Also \
                 CLASS_CREATE_SPECIAL(NumberString, \"String\", RexxClass), same masquerade \
                 (measured, (1.5)~class~id answers \"String\"); own C++ comment: \"the number \
                 string class lies about its identity\". Same missing mechanism as RexxInteger.",
    },
    Deferral {
        setup_class: "RexxInfo",
        reason: "not .NAME-reachable: value('.REXXINFO') returns a live RexxInfo INSTANCE, not \
                 the class (measured; .rexxinfo~id raises 97.1, \"a RexxInfo does not \
                 understand ID\"). Registered via addToSystem, not addToEnvironment; only a \
                 pre-built instance is addToEnvironment'd under REXXINFO (Setup.cpp:1737). This \
                 registry has no dot-variable path to the class object itself to model.",
    },
    Deferral {
        setup_class: "QueueClass",
        reason: "donates Array's instance methods then RemoveMethod's nine of them (Dimension, \
                 Dimensions, Fill, sort, sortWith, stableSort, stableSortWith, makeString, \
                 toString). MethodDict does not model method removal (Task 2's own scope \
                 decision, method_dict.rs: \"no probe this task specifies needs it\").",
    },
    Deferral {
        setup_class: "VariableReference",
        reason: "HideMethod's =, ==, \\=, \\==, <>, >< so they redirect to UNKNOWN -- the same \
                 not-modelled hideMethod/tombstone mechanism QueueClass needs.",
    },
    Deferral {
        setup_class: "StemClass",
        reason: "HideMethod's =, ==, \\=, \\==, <>, >< so they redirect to UNKNOWN -- the same \
                 not-modelled hideMethod/tombstone mechanism QueueClass needs. Also ~inherit's \
                 .MapCollection from CoreClasses.orx, measured (~superClasses gains it): a \
                 second, independent reason.",
    },
    Deferral {
        setup_class: "RexxString",
        reason: "CoreClasses.orx: .string~inherit(.Comparable) (measured, ~superClasses gains \
                 Comparable) and a class-side extension with no ~inherit at all (ALNUM, ALPHA, \
                 BLANK, CNTRL, CR, DIGIT, GRAPH, LOWER, NL, NULL, PRINT, PUNCT, SPACE, TAB, \
                 UPPER, XDIGIT -- measured via .string~instanceMethods(.string)). Comparable is \
                 a ::CLASS MIXINCLASS defined inside CoreClasses.orx itself, not a Setup.cpp/C++ \
                 class -- needs the prologue this task cannot run.",
    },
    Deferral {
        setup_class: "ArrayClass",
        reason: "CoreClasses.orx: .array~inherit(.OrderedCollection), measured (~superClasses \
                 gains OrderedCollection). OrderedCollection is a ::CLASS MIXINCLASS defined \
                 inside CoreClasses.orx itself -- needs the prologue this task cannot run.",
    },
    Deferral {
        setup_class: "ListClass",
        reason: "CoreClasses.orx: .list~inherit(.OrderedCollection), measured. Same missing \
                 mechanism as ArrayClass.",
    },
    Deferral {
        setup_class: "IdentityTable",
        reason: "CoreClasses.orx: .identityTable~inherit(.MapCollection), measured \
                 (~superClasses gains MapCollection). MapCollection is a ::CLASS MIXINCLASS \
                 defined inside CoreClasses.orx itself -- needs the prologue this task cannot \
                 run.",
    },
    Deferral {
        setup_class: "TableClass",
        reason: "CoreClasses.orx: .table~inherit(.MapCollection), measured. Same missing \
                 mechanism as IdentityTable.",
    },
    Deferral {
        setup_class: "StringTable",
        reason: "CoreClasses.orx: .stringTable~inherit(.MapCollection), measured. Same missing \
                 mechanism as IdentityTable.",
    },
    Deferral {
        setup_class: "DirectoryClass",
        reason: "CoreClasses.orx: .directory~inherit(.MapCollection), measured. Same missing \
                 mechanism as IdentityTable.",
    },
    Deferral {
        setup_class: "RelationClass",
        reason: "CoreClasses.orx: .relation~inherit(.MapCollection), measured, and its own \
                 instance-side content diverges too (INTERSECTION/UNION/XOR/SUBSET/DIFFERENCE \
                 present on a live instance via .relation~instanceMethods(.relation) that \
                 Setup.cpp alone never adds -- ManyItemMixin, donated by \
                 ~inheritInstanceMethods, another CoreClasses.orx-only mixin).",
    },
    Deferral {
        setup_class: "SetClass",
        reason: "CoreClasses.orx: .set~inherit(.MapCollection) and .set~inherit(.SetCollection), \
                 measured (~superClasses gains both), plus SetMixin's donated content \
                 (INTERSECTION/UNION/XOR/SUBSET measured present on a live instance) -- three \
                 independent CoreClasses.orx-only mixins.",
    },
    Deferral {
        setup_class: "BagClass",
        reason: "CoreClasses.orx: .bag~inherit(.MapCollection) and .bag~inherit(.SetCollection), \
                 measured, plus BagMixin's donated content (same shape as SetClass's).",
    },
    Deferral {
        setup_class: "MessageClass",
        reason: "CoreClasses.orx: .message~inherit(.MessageNotification) and \
                 .message~inherit(.AlarmNotification), measured (~superClasses gains both). \
                 Both are ::CLASS MIXINCLASS Object definitions inside CoreClasses.orx itself.",
    },
    Deferral {
        setup_class: "SupplierClass",
        reason: "CoreClasses.orx: .supplier~inheritInstanceMethods(.SupplierMixin), donating \
                 allItems/allIndexes/getArrays/supplier -- measured present on a live instance \
                 via .supplier~instanceMethods(.supplier), absent from Setup.cpp's own five \
                 (Available/Index/Next/Item/Init). No superclass edge (inheritInstanceMethods \
                 never adds one), so ~superClasses alone would miss this -- exactly the hazard \
                 this task's brief names; the method-set probe is what catches it.",
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

/// `Setup.cpp:1809`'s `TheClassClass->removeSetupMethods()`, applied at
/// image-save time to delete exactly these two names from `.Class`'s own
/// instance methods (D39). `MethodDict` has no removal primitive (Task 2's
/// own scope decision), so this bootstrap reproduces the *deleted* state by
/// never adding them in the first place, rather than adding then removing --
/// the same final answer `removeSetupMethods` leaves, reached without a
/// removal mechanism this crate does not otherwise need. Measured: the live
/// oracle's `.class~instancemethods(.class)` does not include either name;
/// keeping them would be the exact corpus-visible divergence D39 warns
/// about.
const REMOVED_BY_IMAGE_SAVE: &[&str] = &["DefineClassMethod", "InheritInstanceMethods"];

fn replay(registry: &mut ClassRegistry, class: rexx_core::ObjRef, def: &ClassDefinition) {
    for op in def.ops {
        match op {
            Op::AddClassMethod(name) => registry.add_class_method(class, name),
            Op::AddInstanceMethod(name) => {
                if REMOVED_BY_IMAGE_SAVE.contains(name) {
                    continue;
                }
                registry.add_instance_method(class, name)
            }
            Op::InheritInstanceMethods(source) => {
                let source_id = registry.lookup(source).unwrap_or_else(|| {
                    panic!("InheritInstanceMethods({source}) before {source} was built")
                });
                registry.inherit_instance_methods(class, source_id);
            }
            Op::RemoveInstanceMethod(name) => panic!(
                "{:?} removes {name:?}; a class with a RemoveInstanceMethod op must be in \
                 DEFERRALS, not replayed",
                def.name
            ),
            Op::HideInstanceMethod(name) => panic!(
                "{:?} hides {name:?}; a class with a HideInstanceMethod op must be in \
                 DEFERRALS, not replayed",
                def.name
            ),
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
pub fn native_classes() -> ClassRegistry {
    let mut registry = ClassRegistry::new();

    let class_id = registry.reserve_id();
    let object_id = registry.define_class("Object", None, ClassKind::Regular, class_id);
    registry.define_reserved(
        class_id,
        "Class",
        Some(object_id),
        ClassKind::Regular,
        class_id,
    );
    replay(&mut registry, object_id, definition_for("Object"));
    replay(&mut registry, class_id, definition_for("Class"));
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

    // `CLASS_DEFINITIONS` is in `Setup.cpp`'s own file order, which is a real
    // dependency order (a donor's `StartClassDefinition` block precedes
    // every recipient that `InheritInstanceMethods` from it) and *differs*
    // from the checklist's `createInstance()` order (`TableClass` precedes
    // `IdentityTable` there, but `IdentityTable`'s block precedes `Table`'s
    // -- `Table` donates from it). Replaying in checklist order instead
    // would call `inherit_instance_methods` against a donor not yet built.
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
        let id = registry.define_class(def.name, Some(object_id), ClassKind::Regular, class_id);
        replay(&mut registry, id, def);
    }

    registry
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
    /// module claims to build -- not the deferred ones, and nothing extra.
    #[test]
    fn native_classes_registers_exactly_the_undeferred_checklist_entries() {
        let registry = native_classes();
        for (token, block_name) in CHECKLIST_TO_DEFINITION {
            let deferred = DEFERRALS.iter().any(|d| d.setup_class == *token);
            let registered = registry.lookup(&block_name.to_ascii_uppercase()).is_some();
            assert_eq!(
                registered, !deferred,
                "{token:?} (block {block_name:?}): registered={registered}, deferred={deferred} -- \
                 must be exactly one of the two"
            );
        }
    }
}
