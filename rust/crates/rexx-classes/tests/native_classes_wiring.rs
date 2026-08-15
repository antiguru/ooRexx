//! Task 3's own probes: the metaclass graph (D44, R6) and the native class
//! set [`native_classes`] builds. Every oracle-derived fact here was
//! recorded before this file was written, from probes run as
//! `( ulimit -v 1048576; LD_LIBRARY_PATH=.../ooRexx/build/lib .../ooRexx/build/bin/rexx FILE )`
//! against `/home/moritz/dev/repos/ooRexx/build/bin/rexx`, from a fresh
//! scratch directory, never the scratchpad root. The task report carries
//! every probe program in full; this file states only the recorded answer
//! each assertion reproduces.

use rexx_classes::{ClassKind, ClassRegistry, deferred_classes, native_classes, setup_class_names};
use rexx_core::ObjRef;

/// Bootstrap just `.Object`/`.Class`, the way [`native_classes`] does,
/// without the other twenty-five checklist classes -- what the metaclass
/// tests below build on, isolated from the derived-table machinery.
fn bootstrap_object_and_class() -> (ClassRegistry, ObjRef, ObjRef) {
    let mut r = ClassRegistry::new();
    let class_id = r.reserve_id();
    let object_id = r.define_class("Object", None, ClassKind::Regular, class_id);
    r.define_reserved(
        class_id,
        "Class",
        Some(object_id),
        ClassKind::Regular,
        class_id,
    );
    r.refresh_class_behaviour(class_id);
    r.bootstrap_root_class_behaviour(object_id, class_id);
    (r, object_id, class_id)
}

// ---------------------------------------------------------------------
// D44 self-reference: `.class` is an instance of itself.
// ---------------------------------------------------------------------

/// `.class~metaclass~id` recorded `"Class"` -- `.Class` names itself as its
/// own metaclass. A class-graph assertion this shallow is exactly the
/// second version of the wiring-blind-spot trap the brief names: it passes
/// even if the self-merge that makes `.class~hasmethod('SUBCLASS')` true
/// (next test) never ran, since `metaclass()` just returns the stored
/// field regardless of what the cascade actually built.
#[test]
fn class_is_an_instance_of_itself_by_identity() {
    let (r, _object, class) = bootstrap_object_and_class();
    assert_eq!(r.metaclass(class), class);
    assert_eq!(r.class_of(class), class);
}

// The load-bearing half of the same edge -- "does the self-merge actually
// carry SUBCLASS etc. onto .Class's own class-behaviour, and does .Object's
// dedicated bootstrap path do the same" -- is checked below, against the
// real `native_classes()` bootstrap rather than this file's minimal
// two-class helper (which adds no instance methods at all, so it cannot
// witness a self-merge that has nothing to merge):
// `every_native_class_answers_class_instance_methods_on_its_class_side`
// covers both `.Class` and `.Object`, among the other eleven.

// ---------------------------------------------------------------------
// R6: the `mixinclass class` row, and the negative control that shows why
// it is "reachable only through METACLASS".
// ---------------------------------------------------------------------

/// `mixinclass_class1.rex`: `Mixin mixinclass class`; `MySingleton subclass
/// object inherit Mixin`. Recorded: `Error 98.943: Class "The MySingleton
/// class" is not a subclass of "The Mixin class" base class "The Class
/// class"` -- an ordinary Object-hierarchy class can never `~inherit` a
/// `mixinclass class` mixin, because its own *instance*-behaviour never has
/// `.Class` in scope (only the class-behaviour does, via the metaclass
/// merge every class gets).
#[test]
#[should_panic(expected = "not a subclass of")]
fn an_ordinary_object_hierarchy_class_cannot_inherit_a_mixinclass_class_mixin() {
    let (mut r, object, class) = bootstrap_object_and_class();
    let mixin = r.define_class("Mixin", Some(class), ClassKind::Mixin, class);
    r.add_class_method(mixin, "GREET");
    let ordinary = r.define_class("MySingleton", Some(object), ClassKind::Regular, class);
    r.inherit(ordinary, mixin);
}

/// `q3.rex`: `Mixin mixinclass class` with a class method `GREET`;
/// `MyMeta subclass Class` (no explicit `inherit` yet); then
/// `.MyMeta~inherit(.Mixin)`. Recorded: the `~inherit` call raises no
/// error, `MyMeta~hasmethod('GREET')= 1`, `MyMeta~greet= "hi from mixin"`,
/// and `MyMeta~superclasses` is `Class Mixin` (`.Class` first, from
/// `subclass`, then `.Mixin` appended by `inherit`). This is the mechanism
/// D44 and R6 both cite: `MyMeta subclass Class` gives `MyMeta`'s own
/// *instance*-behaviour `.Class` in scope through the ordinary ancestor
/// cascade (not the metaclass merge, which is class-behaviour-only), which
/// is the one way `inherit`'s instance-side base-class check can ever pass
/// for a `mixinclass class` mixin.
#[test]
fn a_class_subclassing_class_itself_can_inherit_a_mixinclass_class_mixin() {
    let (mut r, _object, class) = bootstrap_object_and_class();
    let mixin = r.define_class("Mixin", Some(class), ClassKind::Mixin, class);
    r.add_class_method(mixin, "GREET");
    let mymeta = r.define_class("MyMeta", Some(class), ClassKind::Regular, class);

    r.inherit(mymeta, mixin);

    assert!(
        r.class_has_method(mymeta, "GREET"),
        "q3.rex recorded 'mymeta_hasmethod_greet= 1'"
    );
    assert_eq!(
        r.superclasses(mymeta),
        [class, mixin],
        "q3.rex recorded 'mymeta_superclasses= Class Mixin'"
    );
}

// ---------------------------------------------------------------------
// The native class set: structural probes for all thirteen native classes,
// each recorded from `structural.rex` / `exact_methods.rex` against the
// live oracle. `~class` and `~metaClass` coincide for a class object
// (measured, e.g. `string_class= Class` and `string_metaclass= Class`); see
// `ClassRegistry::class_of`'s own doc comment for why.
// ---------------------------------------------------------------------

const NATIVE: &[&str] = &[
    "Class",
    "Object",
    "Pointer",
    "Buffer",
    "Method",
    "Routine",
    "Package",
    "RexxContext",
    "EventSemaphore",
    "MutexSemaphore",
    "MutableBuffer",
    "WeakReference",
    "StackFrame",
];

#[test]
fn every_native_class_matches_the_recorded_structural_facts() {
    let r = native_classes();
    let object = r.lookup("OBJECT").expect(".Object is native");
    let class = r.lookup("CLASS").expect(".Class is native");

    for &name in NATIVE {
        let id = r
            .lookup(&name.to_ascii_uppercase())
            .unwrap_or_else(|| panic!("{name} should be native"));
        assert_eq!(r.metaclass(id), class, "{name}~metaClass recorded 'Class'");
        assert_eq!(r.class_of(id), class, "{name}~class recorded 'Class'");
        assert!(r.is_a(id, object), "{name}~isA(.object) recorded 1");
        if name == "Object" {
            assert_eq!(
                r.superclass(id),
                None,
                "object_superclass recorded 'The NIL object'"
            );
            assert_eq!(r.superclasses(id), &[] as &[ObjRef]);
        } else {
            assert_eq!(
                r.superclass(id),
                Some(object),
                "{name}~superclass recorded 'Object'"
            );
            assert_eq!(
                r.superclasses(id),
                &[object],
                "{name}~superclasses recorded 'Object'"
            );
        }
    }
}

/// `q1.rex`/`q9.rex` recorded `.string~hasmethod('SUBCLASS')` and
/// `.object~hasmethod('SUBCLASS')` both `1` -- every native class's
/// class-behaviour carries `.Class`'s instance methods, not just `.Class`'s
/// and `.Object`'s own (special-cased) entries.
#[test]
fn every_native_class_answers_class_instance_methods_on_its_class_side() {
    let r = native_classes();
    for &name in NATIVE {
        let id = r.lookup(&name.to_ascii_uppercase()).unwrap();
        assert!(
            r.class_has_method(id, "SUBCLASS"),
            "{name}~hasmethod('SUBCLASS') should be 1, matching every recorded probe"
        );
    }
}

/// `exact_methods.rex` recorded `.class~instancemethods(.class)`'s full
/// scope-exact set with neither `DEFINECLASSMETHOD` nor
/// `INHERITINSTANCEMETHODS` present -- `removeSetupMethods()` deletes both
/// before the image ships (D39). Building them and never removing them (the
/// one path this crate's `MethodDict` supports) would be a corpus-visible
/// divergence.
#[test]
fn class_does_not_answer_the_two_setup_only_methods_remove_setup_methods_deletes() {
    let r = native_classes();
    let class = r.lookup("CLASS").unwrap();
    assert!(!r.has_method(class, "DEFINECLASSMETHOD"));
    assert!(!r.has_method(class, "INHERITINSTANCEMETHODS"));
}

/// `exact_methods.rex` recorded the scope-exact instance method sets below
/// for the four smallest native classes, verbatim against the live oracle
/// (`inst~instanceMethods(cls)`, sorted).
#[test]
fn small_native_classes_match_their_recorded_own_instance_method_sets() {
    let r = native_classes();
    let cases: &[(&str, &[&str])] = &[
        ("WeakReference", &["VALUE"]),
        ("Buffer", &[]),
        ("MutexSemaphore", &["ACQUIRE", "RELEASE", "UNINIT"]),
        (
            "EventSemaphore",
            &["ISPOSTED", "POST", "RESET", "UNINIT", "WAIT"],
        ),
    ];
    for &(name, expected) in cases {
        let id = r.lookup(&name.to_ascii_uppercase()).unwrap();
        let actual = r.own_instance_method_names(id);
        let expected_set: std::collections::BTreeSet<String> =
            expected.iter().map(|s| s.to_string()).collect();
        assert_eq!(actual, expected_set, "{name}'s own instance methods");
    }
}

// ---------------------------------------------------------------------
// The deferral table: a test over the derived list, not a fixed count.
// ---------------------------------------------------------------------

/// The brief's own done-when condition, restated as a property rather than
/// a count: every derived checklist entry is native or deferred, and every
/// deferral names a non-empty reason. `native_classes.rs`'s own internal
/// unit tests check the stronger, implementation-visible version of this
/// (the checklist-token-to-block-name mapping); this is the public-API
/// shape of the same guarantee.
#[test]
fn every_setup_class_is_native_or_deferred_with_a_reason() {
    let checklist = setup_class_names();
    let deferrals = deferred_classes();
    assert!(
        !checklist.is_empty(),
        "the derived checklist must not be empty"
    );
    for d in deferrals {
        assert!(
            checklist.contains(&d.setup_class),
            "{:?} is not in the derived checklist",
            d.setup_class
        );
        assert!(
            !d.reason.is_empty(),
            "{:?} has an empty deferral reason",
            d.setup_class
        );
        assert!(
            !d.reason.to_ascii_lowercase().contains("not needed yet"),
            "{:?}'s reason must name a mechanism, not \"not needed yet\"",
            d.setup_class
        );
    }
    // No duplicate deferrals.
    let mut seen = std::collections::HashSet::new();
    for d in deferrals {
        assert!(
            seen.insert(d.setup_class),
            "{:?} is deferred more than once",
            d.setup_class
        );
    }
}
