//! Task 3's own probes: the metaclass graph (D44, R6) and the native class
//! set [`native_classes`] builds.
//!
//! Every oracle-derived fact here is reproduced from
//! `oracle-probes/task3_fixround1.rex` (fix round 1) or from the probes
//! named in `task-3-report.md` (the original submission), each run as
//! `( ulimit -v 1048576; LD_LIBRARY_PATH=.../ooRexx/build/lib .../ooRexx/build/bin/rexx FILE )`
//! against `/home/moritz/dev/repos/ooRexx/build/bin/rexx`, from a fresh
//! scratch directory, never the scratchpad root -- `task3_fixround1.rex` is
//! committed precisely so this claim is checkable rather than merely
//! asserted.

use rexx_classes::{ClassKind, ClassRegistry, deferred_classes, native_classes, setup_class_names};
use rexx_core::ObjRef;
use std::collections::BTreeSet;

/// Bootstrap just `.Object`/`.Class`, the way [`native_classes`] does,
/// without the other checklist classes -- what the metaclass tests below
/// build on, isolated from the derived-table machinery.
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

fn set(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|s| s.to_string()).collect()
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
// `every_native_class_answers_metaclass_class_and_isa_object` covers
// `.Class` and `.Object` alongside every other native class.

/// `MethodDict::merge`'s distinguishing half against `merge_methods` alone
/// is that it carries the metaclass's *scope* into the class-behaviour, not
/// only its method entries.
///
/// This deliberately does not read `class_behaviour_has_scope(class, class)`:
/// `cascade_build` calls `target.add_scope(class)` *unconditionally* right
/// after the merge branch (`class_graph.rs`, the `own`/`target.add_scope`
/// lines following the merge), so a class's own scope lands in its own
/// class-behaviour regardless of whether the merge ran at all -- that
/// self-referential read cannot distinguish `merge` from `merge_methods`.
/// The one scope no other path can ever add is the metaclass's, read from a
/// class that is neither the metaclass itself nor descended from it:
/// `Widget` here is an ordinary `subclass Object`, so `.Class` is not among
/// its ancestors and `cascade_build`'s ordinary ancestor walk never visits
/// it -- the *only* way `.Class`'s scope can reach `Widget`'s
/// class-behaviour is the metaclass-merge branch, and only the
/// scope-copying half of `merge` (not `merge_methods`) marks it present.
/// Measured: substituting `merge_methods` for `merge` at the
/// `cascade_build` call site makes this test fail while every other test in
/// the crate stays green (`task-3-report.md` carries the transcript).
#[test]
fn the_metaclass_merge_carries_the_metaclasss_scope_not_just_its_methods() {
    let (mut r, object, class) = bootstrap_object_and_class();
    let widget = r.define_class("Widget", Some(object), ClassKind::Regular, class);
    assert!(
        r.class_behaviour_has_scope(widget, class),
        "a merge_methods-only implementation would still copy SUBCLASS etc. \
         onto Widget's class-behaviour (the metaclass merge's methods still \
         land) without ever marking .Class's own scope as present there, \
         since .Class is not one of Widget's ancestors and no other path \
         ever adds it"
    );
}

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
// Which checklist entries are native -- derived, not hand-maintained
// (Minor #10): `setup_class_names()` minus `deferred_classes()`, mapped to
// block names via `id_string`'s own inverse (every registered name is a
// block name), so lifting a deferral cannot leave this list stale.
// ---------------------------------------------------------------------

/// Checklist token -> block name, for the entries `native_classes.rs`'s
/// own doc comment names as irregular (`RexxClass`/`RexxInteger`/
/// `RexxString`/`RexxObject`) -- every other checklist token is its own
/// block name with a trailing `Class` stripped, or already identical.
fn block_name_for(token: &str) -> &str {
    match token {
        "RexxClass" => "Class",
        "RexxInteger" => "Integer",
        "RexxString" => "String",
        "RexxObject" => "Object",
        other => other.strip_suffix("Class").unwrap_or(other),
    }
}

/// Every checklist entry not in the deferral table, resolved to its
/// registered `ClassRegistry` identity. Panics (failing the test that calls
/// it) if a name claims to be native but is not actually registered, or
/// vice versa -- which is also exactly what
/// `native_classes_registers_exactly_the_undeferred_checklist_entries`
/// (`native_classes.rs`'s own internal test) checks from the other side.
fn native_class_ids(r: &ClassRegistry) -> Vec<(&'static str, ObjRef)> {
    let deferred = deferred_classes();
    setup_class_names()
        .iter()
        .filter(|token| !deferred.iter().any(|d| d.setup_class == **token))
        .map(|&token| {
            let block_name = block_name_for(token);
            let id = r
                .lookup(&block_name.to_ascii_uppercase())
                .unwrap_or_else(|| panic!("{token} (block {block_name:?}) should be native"));
            // `block_name` borrows from `token`, which borrows from the
            // `'static` `SETUP_CLASSES`/`CHECKLIST_TO_DEFINITION` tables, so
            // this is safe to widen back to `'static`.
            (block_name, id)
        })
        .collect()
}

// ---------------------------------------------------------------------
// Structural facts true for every native class regardless of whether
// `CoreClasses.orx` later touches it: metaclass, `~class`, and `~isA(.object)`
// are never affected by an `~inherit` call (it only appends ancestors, never
// changes `metaClass`, and `.Object` remains an ancestor once true). This is
// the R8 boundary made concrete: those hold for every native class;
// `~superClass`/`~superClasses` do not, for the R8 classes, and are
// asserted separately, against `Setup.cpp`'s own pre-prologue state rather
// than the live (post-prologue) oracle.
// ---------------------------------------------------------------------

#[test]
fn every_native_class_answers_metaclass_class_and_isa_object() {
    let r = native_classes();
    let object = r.lookup("OBJECT").expect(".Object is native");
    let class = r.lookup("CLASS").expect(".Class is native");

    for (name, id) in native_class_ids(&r) {
        assert_eq!(r.metaclass(id), class, "{name}~metaClass recorded 'Class'");
        assert_eq!(r.class_of(id), class, "{name}~class recorded 'Class'");
        assert!(r.is_a(id, object), "{name}~isA(.object) recorded 1");
    }
}

/// `q1.rex`/`q9.rex` recorded `.string~hasmethod('SUBCLASS')` and
/// `.object~hasmethod('SUBCLASS')` both `1` -- every native class's
/// class-behaviour carries `.Class`'s instance methods, not just `.Class`'s
/// and `.Object`'s own (special-cased) entries.
#[test]
fn every_native_class_answers_class_instance_methods_on_its_class_side() {
    let r = native_classes();
    for (name, id) in native_class_ids(&r) {
        assert!(
            r.class_has_method(id, "SUBCLASS"),
            "{name}~hasmethod('SUBCLASS') should be 1, matching every recorded probe"
        );
    }
}

/// `~isA` asserted both ways: `is_a(id, object)` true for everything above
/// is not enough by itself to distinguish a real ancestor walk from
/// `fn is_a(_, _) -> bool { true }` -- these are the pairs that must come
/// out false. `object_hasmethod_subclass=1`/measured superclass facts in
/// this session's probes already establish the hierarchy these read from;
/// no `.Pointer` is ever a `.Method`, nor is `.Object` ever one of its own
/// subclasses.
#[test]
fn is_a_is_false_for_unrelated_and_reversed_pairs() {
    let r = native_classes();
    let object = r.lookup("OBJECT").unwrap();
    let pointer = r.lookup("POINTER").unwrap();
    let method = r.lookup("METHOD").unwrap();
    let array = r.lookup("ARRAY").unwrap();

    assert!(!r.is_a(pointer, method), "Pointer is not a Method");
    assert!(
        !r.is_a(object, pointer),
        "Object is not one of its own subclasses"
    );
    assert!(!r.is_a(array, method), "Array is not a Method");
}

// ---------------------------------------------------------------------
// The classes `CoreClasses.orx` never touches (named in
// UNTOUCHED_BY_PROLOGUE below): full recorded structural facts,
// byte-identical against the live oracle.
// ---------------------------------------------------------------------

const UNTOUCHED_BY_PROLOGUE: &[&str] = &[
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

/// The classes R8 un-deferred (named below): `CoreClasses.orx` mutates each
/// of these (an `~inherit` or `~inheritInstanceMethods` call this crate
/// does not, and cannot, replay), so their *live* `~superClasses` is not
/// what this module builds. Their pre-prologue `~superClasses` -- `.Object`
/// alone -- is still asserted below, explicitly as the pre-prologue state
/// `Setup.cpp` itself builds, not a live-oracle match.
const MUTATED_BY_PROLOGUE: &[&str] = &[
    "String",
    "Array",
    "IdentityTable",
    "Table",
    "StringTable",
    "Set",
    "Directory",
    "Relation",
    "Bag",
    "List",
    "Message",
    "Supplier",
];

#[test]
fn the_thirteen_classes_untouched_by_the_prologue_match_the_live_oracle_exactly() {
    let r = native_classes();
    let object = r.lookup("OBJECT").unwrap();

    for &name in UNTOUCHED_BY_PROLOGUE {
        let id = r.lookup(&name.to_ascii_uppercase()).unwrap();
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

/// `MUTATED_BY_PROLOGUE`'s pre-prologue `~superClasses` -- `.Object` alone --
/// asserted as exactly that: the state `Setup.cpp` builds before
/// `CoreClasses.orx` runs, not a claim about what the live, fully-booted
/// oracle answers (measured different: e.g. `.Array~superClasses` there is
/// `Object OrderedCollection`). Full post-prologue verification for these
/// classes is Task 13's.
#[test]
fn the_twelve_prologue_mutated_classes_have_the_pre_prologue_superclasses_setup_cpp_builds() {
    let r = native_classes();
    let object = r.lookup("OBJECT").unwrap();
    for &name in MUTATED_BY_PROLOGUE {
        let id = r.lookup(&name.to_ascii_uppercase()).unwrap();
        assert_eq!(
            r.superclass(id),
            Some(object),
            "{name}'s pre-prologue superclass is .Object, per Setup.cpp alone"
        );
        assert_eq!(r.superclasses(id), &[object]);
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

// ---------------------------------------------------------------------
// `~id` and `~subClasses`: named parts of the class object (the brief).
// ---------------------------------------------------------------------

/// `~id` for a sample spanning both groups above, recorded against the live
/// oracle (`.rexxcontext~id`, `.array~id`, etc. in this session's earlier
/// probes; `id_string` is exactly the block name every class is registered
/// under, so this also doubles as a registration-name sanity check).
#[test]
fn id_string_matches_the_recorded_id_for_a_representative_sample() {
    let r = native_classes();
    for &(name, expected) in &[
        ("Class", "Class"),
        ("Object", "Object"),
        ("RexxContext", "RexxContext"),
        ("Array", "Array"),
        ("Supplier", "Supplier"),
        ("StackFrame", "StackFrame"),
    ] {
        let id = r.lookup(&name.to_ascii_uppercase()).unwrap();
        assert_eq!(r.id_string(id), expected);
    }
}

/// `~subClasses`: `.Object`'s own subclass list must contain every other
/// native class, since every one of them names `.Object` as its direct
/// superclass -- the cascade's own bookkeeping (`ClassGraph::subclasses`,
/// populated by `define_class`) must record that edge in both directions,
/// not only the forward `superclass` one `superclasses()` already covers.
/// Built since Task 3's first submission; this is its first assertion.
#[test]
fn objects_subclass_list_contains_every_other_native_class() {
    let r = native_classes();
    let object = r.lookup("OBJECT").unwrap();
    let object_subclasses: std::collections::HashSet<ObjRef> =
        r.subclasses(object).iter().copied().collect();

    for (name, id) in native_class_ids(&r) {
        if name == "Object" {
            continue; // .Object is not its own subclass
        }
        assert!(
            object_subclasses.contains(&id),
            "{name} names .Object as its superclass, so .Object's own \
             subclasses list must contain {name} back"
        );
    }
}

// ---------------------------------------------------------------------
// R8: the prologue-mutated classes' own (derived, pre-prologue) method
// sets, exactly -- from `task3_fixround1.rex`'s Part A (`cls~methods(cls)`,
// scope-exact, needs no instantiation). `Set`, `Bag`, `Relation` and
// `Supplier` are asserted separately, below (`set_bag_relation_and_supplier_match_their_setup_cpp_derived_own_sets`):
// `~inheritInstanceMethods` rescopes their donated content onto the
// recipient itself, so `cls~methods(cls)` answers their post-donation set
// on the live oracle, not `Setup.cpp`'s own -- the same reason this
// technique cannot verify them the way it does every class in this table.
// ---------------------------------------------------------------------

#[test]
fn every_prologue_mutated_class_matches_its_recorded_own_instance_method_set() {
    let r = native_classes();
    let cases: &[(&str, &[&str])] = &[
        (
            "String",
            &[
                "",
                " ",
                "%",
                "&",
                "&&",
                "*",
                "**",
                "+",
                "-",
                "/",
                "//",
                "<",
                "<<",
                "<<=",
                "<=",
                "<>",
                "=",
                "==",
                ">",
                "><",
                ">=",
                ">>",
                ">>=",
                "?",
                "ABBREV",
                "ABS",
                "APPEND",
                "B2X",
                "BITAND",
                "BITOR",
                "BITXOR",
                "C2D",
                "C2X",
                "CASELESSABBREV",
                "CASELESSCHANGESTR",
                "CASELESSCOMPARE",
                "CASELESSCOMPARETO",
                "CASELESSCONTAINS",
                "CASELESSCONTAINSWORD",
                "CASELESSCOUNTSTR",
                "CASELESSENDSWITH",
                "CASELESSEQUALS",
                "CASELESSLASTPOS",
                "CASELESSMATCH",
                "CASELESSMATCHCHAR",
                "CASELESSPOS",
                "CASELESSSTARTSWITH",
                "CASELESSWORDPOS",
                "CEILING",
                "CENTER",
                "CENTRE",
                "CHANGESTR",
                "COMPARE",
                "COMPARETO",
                "CONTAINS",
                "CONTAINSWORD",
                "COPIES",
                "COUNTSTR",
                "D2C",
                "D2X",
                "DATATYPE",
                "DECODEBASE64",
                "DELSTR",
                "DELWORD",
                "ENCODEBASE64",
                "ENDSWITH",
                "EQUALS",
                "FLOOR",
                "FORMAT",
                "INSERT",
                "LASTPOS",
                "LEFT",
                "LENGTH",
                "LOWER",
                "MAKEARRAY",
                "MAKESTRING",
                "MATCH",
                "MATCHCHAR",
                "MAX",
                "MIN",
                "MODULO",
                "OVERLAY",
                "POS",
                "REPLACEAT",
                "REVERSE",
                "RIGHT",
                "ROUND",
                "SIGN",
                "SPACE",
                "STARTSWITH",
                "STRIP",
                "SUBCHAR",
                "SUBSTR",
                "SUBWORD",
                "SUBWORDS",
                "TRANSLATE",
                "TRUNC",
                "UPPER",
                "VERIFY",
                "WORD",
                "WORDINDEX",
                "WORDLENGTH",
                "WORDPOS",
                "WORDS",
                "X2B",
                "X2C",
                "X2D",
                "[]",
                "\\",
                "\\<",
                "\\<<",
                "\\=",
                "\\==",
                "\\>",
                "\\>>",
                "|",
                "||",
            ],
        ),
        (
            "Array",
            &[
                "ALLINDEXES",
                "ALLITEMS",
                "APPEND",
                "AT",
                "DELETE",
                "DIMENSION",
                "DIMENSIONS",
                "EMPTY",
                "FILL",
                "FIRST",
                "FIRSTITEM",
                "HASINDEX",
                "HASITEM",
                "INDEX",
                "INSERT",
                "ISEMPTY",
                "ITEMS",
                "LAST",
                "LASTITEM",
                "MAKEARRAY",
                "MAKESTRING",
                "NEXT",
                "PREVIOUS",
                "PUT",
                "REMOVE",
                "REMOVEITEM",
                "SECTION",
                "SIZE",
                "SORT",
                "SORTWITH",
                "STABLESORT",
                "STABLESORTWITH",
                "SUPPLIER",
                "TOSTRING",
                "[]",
                "[]=",
            ],
        ),
        (
            "IdentityTable",
            &[
                "ALLINDEXES",
                "ALLITEMS",
                "AT",
                "EMPTY",
                "HASINDEX",
                "HASITEM",
                "INDEX",
                "INIT",
                "ISEMPTY",
                "ITEMS",
                "MAKEARRAY",
                "PUT",
                "REMOVE",
                "REMOVEITEM",
                "SUPPLIER",
                "[]",
                "[]=",
            ],
        ),
        (
            "Table",
            &[
                "ALLINDEXES",
                "ALLITEMS",
                "AT",
                "EMPTY",
                "HASINDEX",
                "HASITEM",
                "INDEX",
                "INIT",
                "ISEMPTY",
                "ITEMS",
                "MAKEARRAY",
                "PUT",
                "REMOVE",
                "REMOVEITEM",
                "SUPPLIER",
                "[]",
                "[]=",
            ],
        ),
        (
            "StringTable",
            &[
                "ALLINDEXES",
                "ALLITEMS",
                "AT",
                "EMPTY",
                "ENTRY",
                "HASENTRY",
                "HASINDEX",
                "HASITEM",
                "INDEX",
                "INIT",
                "ISEMPTY",
                "ITEMS",
                "MAKEARRAY",
                "PUT",
                "REMOVE",
                "REMOVEENTRY",
                "REMOVEITEM",
                "SETENTRY",
                "SUPPLIER",
                "UNKNOWN",
                "[]",
                "[]=",
            ],
        ),
        (
            "Directory",
            &[
                "ALLINDEXES",
                "ALLITEMS",
                "AT",
                "EMPTY",
                "ENTRY",
                "HASENTRY",
                "HASINDEX",
                "HASITEM",
                "INDEX",
                "INIT",
                "ISEMPTY",
                "ITEMS",
                "MAKEARRAY",
                "PUT",
                "REMOVE",
                "REMOVEENTRY",
                "REMOVEITEM",
                "SETENTRY",
                "SETMETHOD",
                "SUPPLIER",
                "UNKNOWN",
                "UNSETMETHOD",
                "[]",
                "[]=",
            ],
        ),
        (
            "List",
            &[
                "ALLINDEXES",
                "ALLITEMS",
                "APPEND",
                "AT",
                "DELETE",
                "EMPTY",
                "FIRST",
                "FIRSTITEM",
                "HASINDEX",
                "HASITEM",
                "INDEX",
                "INIT",
                "INSERT",
                "ISEMPTY",
                "ITEMS",
                "LAST",
                "LASTITEM",
                "MAKEARRAY",
                "NEXT",
                "PREVIOUS",
                "PUT",
                "REMOVE",
                "REMOVEITEM",
                "SECTION",
                "SUPPLIER",
                "[]",
                "[]=",
            ],
        ),
        (
            "Message",
            &[
                "ARGUMENTS",
                "COMPLETED",
                "ERRORCONDITION",
                "HALT",
                "HASERROR",
                "HASRESULT",
                "MESSAGECOMPLETE",
                "MESSAGENAME",
                "NOTIFY",
                "REPLY",
                "REPLYWITH",
                "RESULT",
                "SEND",
                "SENDWITH",
                "START",
                "STARTWITH",
                "TARGET",
                "TRIGGERED",
                "WAIT",
            ],
        ),
        // `Supplier`, `Set`, `Bag` and `Relation` are deliberately not in
        // this table: `~inheritInstanceMethods` (unlike `~inherit`) rewrites
        // the donor's methods to the recipient's *own* scope
        // (`RexxClass::inheritInstanceMethods`, `ClassClass.cpp:558-586`,
        // `setMethodScope`, `:563`), so on the live oracle
        // `.Supplier~methods(.Supplier)` (and likewise `.Set`'s, `.Bag`'s,
        // `.Relation`'s) answers the post-donation set, not `Setup.cpp`'s
        // own -- the donor mixin's contribution is no longer
        // distinguishable by scope at all once `CoreClasses.orx` has run
        // it. This table's technique (`cls~methods(cls)`) therefore cannot
        // verify these classes' pre-prologue derivation against the
        // oracle the way it does for every other class here;
        // `own_instance_method_names` (trusted from `Setup.cpp` alone) and
        // the dedicated `*_flattened_set_*` tests below carry their actual
        // evidence instead.
    ];
    for &(name, expected) in cases {
        let id = r.lookup(&name.to_ascii_uppercase()).unwrap();
        assert_eq!(
            r.own_instance_method_names(id),
            set(expected),
            "{name}'s own (pre-prologue) instance methods, task3_fixround1.rex Part A"
        );
    }
}

/// `UNTOUCHED_BY_PROLOGUE`'s own instance method sets, exact --
/// `task3_fixround1.rex` Part A and this session's earlier probes agree.
#[test]
fn every_untouched_class_matches_its_recorded_own_instance_method_set() {
    let r = native_classes();
    let cases: &[(&str, &[&str])] = &[
        ("WeakReference", &["VALUE"]),
        ("Buffer", &[]),
        ("MutexSemaphore", &["ACQUIRE", "RELEASE", "UNINIT"]),
        (
            "EventSemaphore",
            &["ISPOSTED", "POST", "RESET", "UNINIT", "WAIT"],
        ),
        ("Pointer", &["=", "==", "ISNULL", "\\=", "\\=="]),
        (
            "Method",
            &[
                "ANNOTATION",
                "ANNOTATIONS",
                "ISABSTRACT",
                "ISATTRIBUTE",
                "ISCONSTANT",
                "ISGUARDED",
                "ISPACKAGE",
                "ISPRIVATE",
                "ISPROTECTED",
                "PACKAGE",
                "SCOPE",
                "SETGUARDED",
                "SETPRIVATE",
                "SETPROTECTED",
                "SETSECURITYMANAGER",
                "SETUNGUARDED",
                "SOURCE",
            ],
        ),
        (
            "Routine",
            &[
                "ANNOTATION",
                "ANNOTATIONS",
                "CALL",
                "CALLWITH",
                "PACKAGE",
                "SETSECURITYMANAGER",
                "SOURCE",
                "[]",
            ],
        ),
        (
            "Package",
            &[
                "ADDCLASS",
                "ADDPACKAGE",
                "ADDPUBLICCLASS",
                "ADDPUBLICROUTINE",
                "ADDROUTINE",
                "ANNOTATION",
                "ANNOTATIONS",
                "CLASSES",
                "DEFINEDMETHODS",
                "DIGITS",
                "FINDCLASS",
                "FINDNAMESPACE",
                "FINDPROGRAM",
                "FINDPUBLICCLASS",
                "FINDPUBLICROUTINE",
                "FINDROUTINE",
                "FORM",
                "FUZZ",
                "IMPORTEDCLASSES",
                "IMPORTEDPACKAGES",
                "IMPORTEDROUTINES",
                "LOADLIBRARY",
                "LOADPACKAGE",
                "LOCAL",
                "NAME",
                "NAMESPACES",
                "OPTIONS",
                "PROLOG",
                "PUBLICCLASSES",
                "PUBLICROUTINES",
                "RESOURCE",
                "RESOURCES",
                "ROUTINES",
                "SETSECURITYMANAGER",
                "SOURCE",
                "SOURCELINE",
                "SOURCESIZE",
                "TRACE",
            ],
        ),
        (
            "RexxContext",
            &[
                "ARGS",
                "CONDITION",
                "COPY",
                "DIGITS",
                "EXECUTABLE",
                "FORM",
                "FUZZ",
                "INTERPRETER",
                "INVOCATION",
                "LINE",
                "NAME",
                "PACKAGE",
                "RS",
                "STACKFRAMES",
                "THREAD",
                "VARIABLES",
            ],
        ),
        (
            "MutableBuffer",
            &[
                "APPEND",
                "CASELESSCHANGESTR",
                "CASELESSCONTAINS",
                "CASELESSCONTAINSWORD",
                "CASELESSCOUNTSTR",
                "CASELESSENDSWITH",
                "CASELESSLASTPOS",
                "CASELESSMATCH",
                "CASELESSMATCHCHAR",
                "CASELESSPOS",
                "CASELESSSTARTSWITH",
                "CASELESSWORDPOS",
                "CHANGESTR",
                "CONTAINS",
                "CONTAINSWORD",
                "COUNTSTR",
                "DELETE",
                "DELSTR",
                "DELWORD",
                "ENDSWITH",
                "GETBUFFERSIZE",
                "INSERT",
                "LASTPOS",
                "LENGTH",
                "LOWER",
                "MAKEARRAY",
                "MAKESTRING",
                "MATCH",
                "MATCHCHAR",
                "OVERLAY",
                "POS",
                "REPLACEAT",
                "SETBUFFERSIZE",
                "SETTEXT",
                "SPACE",
                "STARTSWITH",
                "STRING",
                "SUBCHAR",
                "SUBSTR",
                "SUBWORD",
                "SUBWORDS",
                "TRANSLATE",
                "UPPER",
                "VERIFY",
                "WORD",
                "WORDINDEX",
                "WORDLENGTH",
                "WORDPOS",
                "WORDS",
                "[]",
                "[]=",
            ],
        ),
        (
            "StackFrame",
            &[
                "ARGUMENTS",
                "CONTEXT",
                "EXECUTABLE",
                "INVOCATION",
                "LINE",
                "MAKESTRING",
                "NAME",
                "STRING",
                "TARGET",
                "TRACELINE",
                "TYPE",
            ],
        ),
    ];
    for &(name, expected) in cases {
        let id = r.lookup(&name.to_ascii_uppercase()).unwrap();
        assert_eq!(
            r.own_instance_method_names(id),
            set(expected),
            "{name}'s own instance methods, task3_fixround1.rex Part A"
        );
    }
}

/// Object and String's own set includes `""` and `" "` (the concatenation
/// operators), which cannot appear in a space-joined printout -- checked
/// separately via `hasmethod`, `task3_fixround1.rex` Part E.
#[test]
fn object_and_string_answer_the_concatenation_operator_names() {
    let r = native_classes();
    let object = r.lookup("OBJECT").unwrap();
    let string = r.lookup("STRING").unwrap();
    assert!(r.own_instance_method_names(object).contains(""));
    assert!(r.own_instance_method_names(object).contains(" "));
    assert!(r.own_instance_method_names(string).contains(""));
    assert!(r.own_instance_method_names(string).contains(" "));
}

/// `Class`'s and `Object`'s own instance method sets, exact -- asserted in
/// their own case here rather than in the table above because their
/// expected sets are long enough to want that, and because they are the
/// classes D39/D44's bootstrap-only mechanisms touch directly.
#[test]
fn class_and_object_match_their_recorded_own_instance_method_sets() {
    let r = native_classes();
    let class = r.lookup("CLASS").unwrap();
    let object = r.lookup("OBJECT").unwrap();
    assert_eq!(
        r.own_instance_method_names(class),
        set(&[
            "<>",
            "=",
            "==",
            "><",
            "ACTIVATE",
            "ANNOTATION",
            "ANNOTATIONS",
            "BASECLASS",
            "COPY",
            "DEFAULTNAME",
            "DEFINE",
            "DEFINEMETHODS",
            "DELETE",
            "ENHANCED",
            "HASHCODE",
            "ID",
            "INHERIT",
            "ISABSTRACT",
            "ISMETACLASS",
            "ISSUBCLASSOF",
            "METACLASS",
            "METHOD",
            "METHODS",
            "MIXINCLASS",
            "PACKAGE",
            "QUERYMIXINCLASS",
            "SUBCLASS",
            "SUBCLASSES",
            "SUPERCLASS",
            "SUPERCLASSES",
            "UNINHERIT",
            "\\=",
            "\\==",
        ]),
        "Class's own instance methods, task3_fixround1.rex Part A -- \
         DefineClassMethod/InheritInstanceMethods absent, per D39"
    );
    assert_eq!(
        r.own_instance_method_names(object),
        set(&[
            "",
            " ",
            "<>",
            "=",
            "==",
            "><",
            "CLASS",
            "COPY",
            "DEFAULTNAME",
            "HASHCODE",
            "HASMETHOD",
            "IDENTITYHASH",
            "INIT",
            "INSTANCEMETHOD",
            "INSTANCEMETHODS",
            "ISA",
            "ISINSTANCEOF",
            "ISNIL",
            "OBJECTNAME",
            "OBJECTNAME=",
            "REQUEST",
            "RUN",
            "SEND",
            "SENDWITH",
            "SETMETHOD",
            "START",
            "STARTWITH",
            "STRING",
            "UNSETMETHOD",
            "\\=",
            "\\==",
            "||",
        ]),
        "Object's own instance methods, task3_fixround1.rex Part A + Part E"
    );
}

/// Every class's own class-method set (usually just `New`, sometimes also
/// `Of`), by `hasmethod` spot check -- `task3_fixround1.rex` Part B.
#[test]
fn every_class_answers_its_recorded_own_class_methods() {
    let r = native_classes();
    for &(name, expected) in &[
        ("Class", &["NEW"] as &[&str]),
        ("Array", &["NEW", "OF"]),
        ("Set", &["NEW", "OF"]),
        ("Bag", &["NEW", "OF"]),
        ("List", &["NEW", "OF"]),
        ("Method", &["NEW", "NEWFILE", "LOADEXTERNALMETHOD"]),
        ("Routine", &["NEW", "NEWFILE", "LOADEXTERNALROUTINE"]),
        ("Package", &["NEW", "DEFAULTOPTIONS"]),
    ] {
        let id = r.lookup(&name.to_ascii_uppercase()).unwrap();
        for &m in expected {
            assert!(
                r.class_has_method(id, m),
                "{name}~hasmethod({m:?}) recorded 1"
            );
        }
    }
}

/// Every native class's own (unflattened) class-method set, exact, for
/// every checklist entry `native_classes()` builds. Absent a caller like
/// this one, a build that gave any native class a spurious or missing
/// class method would pass unnoticed. `Array`/`Set`/`Bag`/`List`'s `Of` and
/// `Method`/`Routine`/`Package`'s extra entries are corroborated against a
/// live oracle `hasmethod` check in the test above; every class not listed
/// in `new_only` below is `New` alone -- `Setup.cpp`'s own `AddClassMethod`
/// list for that block names nothing else, and `build.rs`'s derivation
/// (independently re-implemented against the oracle and shown to agree)
/// is what this assertion actually reads.
#[test]
fn every_native_class_matches_its_recorded_own_class_method_set() {
    let r = native_classes();
    let new_only = [
        "Object",
        "String",
        "IdentityTable",
        "Table",
        "StringTable",
        "Directory",
        "Relation",
        "Message",
        "RexxContext",
        "EventSemaphore",
        "MutexSemaphore",
        "MutableBuffer",
        "Supplier",
        "Pointer",
        "Buffer",
        "WeakReference",
        "StackFrame",
    ];
    for name in new_only {
        let id = r.lookup(&name.to_ascii_uppercase()).unwrap();
        assert_eq!(
            r.own_class_method_names(id),
            set(&["NEW"]),
            "{name}'s own class methods, from Setup.cpp's single AddClassMethod entry"
        );
    }
    let cases: &[(&str, &[&str])] = &[
        ("Class", &["NEW"]),
        ("Array", &["NEW", "OF"]),
        ("Set", &["NEW", "OF"]),
        ("Bag", &["NEW", "OF"]),
        ("List", &["NEW", "OF"]),
        ("Method", &["NEW", "NEWFILE", "LOADEXTERNALMETHOD"]),
        ("Routine", &["NEW", "NEWFILE", "LOADEXTERNALROUTINE"]),
        ("Package", &["NEW", "DEFAULTOPTIONS"]),
    ];
    for &(name, expected) in cases {
        let id = r.lookup(&name.to_ascii_uppercase()).unwrap();
        assert_eq!(
            r.own_class_method_names(id),
            set(expected),
            "{name}'s own class methods"
        );
    }
}

/// `class_method_names` (the *flattened* class-side set -- own class
/// methods plus everything the metaclass merge donates) needs a caller
/// that reads the flattened set directly, not only a presence check
/// through a different accessor. `.Array~class`'s flattened set must be a
/// superset of its own class methods (`New`, `Of`) and must also carry
/// `.Class`'s own instance methods (`SUBCLASS`), which is the metaclass
/// merge's contribution.
#[test]
fn class_method_names_reads_the_flattened_class_side_set() {
    let r = native_classes();
    let array = r.lookup("ARRAY").unwrap();
    let flattened = r.class_method_names(array);
    assert!(flattened.contains("NEW"));
    assert!(flattened.contains("OF"));
    assert!(flattened.contains("SUBCLASS"));
    assert!(
        flattened.len() > r.own_class_method_names(array).len(),
        "the flattened class-side set must be strictly larger than Array's \
         own class methods -- equal would mean the metaclass merge \
         contributed nothing"
    );
}

// ---------------------------------------------------------------------
// R8's measured subset: for a prologue-mutated class, this crate's
// flattened set is a subset of what the live oracle answers, and the
// measured difference is attributed to a specific `CoreClasses.orx` mixin.
// How that difference is measured depends on the donation. `Array` and `String`
// (which receive an ordinary `~inherit`, not `~inheritInstanceMethods`) are
// measured via `task3_fixround1.rex` Part C, a direct scope-exact query
// against the mixin class. `Set`/`Bag`/`Relation`/`Supplier` cannot be
// measured that way -- Part C's queries against `.SetMixin`/
// `.ManyItemMixin`/`.BagMixin`/`.SupplierMixin` themselves all come back
// empty, because `~inheritInstanceMethods` rescopes the donated methods to
// the recipient (`RexxClass::inheritInstanceMethods`'s `setMethodScope`,
// `ClassClass.cpp:563`) -- so their divergence is measured instead by
// diffing Part A's recorded `OWN_INSTANCE` line (which, for a recipient,
// already contains the rescoped donation) against `own_instance_method_names`'s
// `Setup.cpp`-derived value.
// ---------------------------------------------------------------------

/// A class's flattened instance method set -- own methods plus everything
/// inherited, which is what the cascade (`ClassGraph::cascade_build`) is
/// actually for. Also the witness that the `Side::Instance` cascade runs at
/// all: if it did nothing, `instance_method_names` would equal
/// `own_instance_method_names` exactly, and this test's own assertion that
/// Object's methods (`"COPY"`, `"HASHCODE"`) are present on classes that
/// never declare them directly would fail.
#[test]
fn a_native_classs_flattened_set_includes_its_ancestors_methods_the_cascade_witness() {
    let r = native_classes();
    let array = r.lookup("ARRAY").unwrap();
    let flattened = r.instance_method_names(array);
    // Array's own method -- present either way, not the interesting half.
    assert!(flattened.contains("SORT"));
    // Object's method, which Array never declares directly -- present only
    // if the ancestor cascade actually ran.
    assert!(
        flattened.contains("COPY"),
        "COPY is Object's own method; its presence on Array's flattened set \
         is what a no-op Side::Instance cascade would fail to produce"
    );
    assert!(flattened.contains("HASHCODE"));
    assert!(
        flattened.len() > r.own_instance_method_names(array).len(),
        "the flattened set must be strictly larger than Array's own -- \
         equal would mean nothing was inherited"
    );
}

/// `array_from_orderedcollection` (`task3_fixround1.rex` Part C): the names
/// recorded scoped to `.OrderedCollection` on a live `.Array` instance
/// include several Array already has under its own scope (`CoreClasses.orx:1011-1231`
/// redeclares them `ABSTRACT` before `Array`'s concrete versions were ever
/// donated -- irrelevant to the *name set* this task's probes check, only
/// to scope attribution, which no probe here reads) alongside genuinely new
/// ones. This is the measured difference R8 asks for: each name below is
/// attributable to `OrderedCollection`, confirmed present on a live
/// instance and absent from this crate's native (pre-prologue) set.
#[test]
fn arrays_flattened_set_is_a_measured_subset_of_the_live_oracles() {
    let r = native_classes();
    let array = r.lookup("ARRAY").unwrap();
    let native = r.instance_method_names(array);

    // Subset direction: every native name is one the recorded
    // `array_from_orderedcollection`/Array's own oracle answer also has --
    // trivially true for Array's own names (unaffected by the prologue) and
    // asserted directly for Object's, matching the cascade-witness test.
    for name in ["SORT", "COPY", "HASHCODE", "[]", "APPEND"] {
        assert!(native.contains(name));
    }

    // The measured difference: recorded live-oracle names beyond this
    // crate's native set, attributed to CoreClasses.orx:1011-1231's
    // OrderedCollection mixin.
    let extra_from_ordered_collection = [
        "APPENDALL",
        "DIFFERENCE",
        "INTERSECTION",
        "SUBSET",
        "UNION",
        "XOR",
    ];
    for name in extra_from_ordered_collection {
        assert!(
            !native.contains(name),
            "{name} recorded present on a live .Array instance via \
             OrderedCollection but should be absent from this crate's \
             pre-prologue native set -- if this now fails, native_classes.rs \
             has started building OrderedCollection's donation itself and \
             this test (and the R8 boundary) need revisiting"
        );
    }
}

// `string_from_comparable` (Part C): `.Comparable`'s method,
// `COMPARETO`, is scoped to `.Comparable` on a live `.String` instance --
// but `Setup.cpp` already gives `.String` its own `CompareTo`
// (`StringClass.cpp:688`, `AddMethod("CompareTo", ...)`), so this is a
// scope-attribution change, not a name-set one. Unlike
// `Array`/`Set`/`Bag`/`Relation`/`Supplier` (below), `String`'s
// `~inherit(.Comparable)` measurably donates no name this crate's native
// set lacks. `COMPARETO`'s presence is already asserted by the exact-set
// comparison in `every_prologue_mutated_class_matches_its_recorded_own_instance_method_set`'s
// `String` case above, so there is no separate test for it here -- a
// dedicated one would only restate that membership.

/// `Setup.cpp`'s own derivation for `Set`, `Bag`, `Relation` and `Supplier`
/// -- the classes `~inheritInstanceMethods` rescopes (excluded from the
/// exact-match table above for exactly that reason): each one's
/// *pre-prologue* own instance methods, asserted directly against this
/// crate's own bookkeeping rather than against a live oracle query that
/// cannot isolate them once the donation has run.
#[test]
fn set_bag_relation_and_supplier_match_their_setup_cpp_derived_own_sets() {
    let r = native_classes();
    let cases: &[(&str, &[&str])] = &[
        (
            "Set",
            &[
                "ALLINDEXES",
                "ALLITEMS",
                "AT",
                "EMPTY",
                "HASINDEX",
                "HASITEM",
                "INDEX",
                "INIT",
                "ISEMPTY",
                "ITEMS",
                "MAKEARRAY",
                "PUT",
                "REMOVE",
                "REMOVEITEM",
                "SUPPLIER",
                "[]",
                "[]=",
            ],
        ),
        (
            "Bag",
            &[
                "ALLAT",
                "ALLINDEX",
                "ALLINDEXES",
                "ALLITEMS",
                "AT",
                "EMPTY",
                "HASINDEX",
                "HASITEM",
                "INDEX",
                "INIT",
                "ISEMPTY",
                "ITEMS",
                "MAKEARRAY",
                "PUT",
                "REMOVE",
                "REMOVEALL",
                "REMOVEITEM",
                "SUPPLIER",
                "UNIQUEINDEXES",
                "[]",
                "[]=",
            ],
        ),
        (
            "Relation",
            &[
                "ALLAT",
                "ALLINDEX",
                "ALLINDEXES",
                "ALLITEMS",
                "AT",
                "EMPTY",
                "HASINDEX",
                "HASITEM",
                "INDEX",
                "INIT",
                "ISEMPTY",
                "ITEMS",
                "MAKEARRAY",
                "PUT",
                "REMOVE",
                "REMOVEALL",
                "REMOVEITEM",
                "SUPPLIER",
                "UNIQUEINDEXES",
                "[]",
                "[]=",
            ],
        ),
        ("Supplier", &["AVAILABLE", "INDEX", "INIT", "ITEM", "NEXT"]),
    ];
    for &(name, expected) in cases {
        let id = r.lookup(&name.to_ascii_uppercase()).unwrap();
        assert_eq!(
            r.own_instance_method_names(id),
            set(expected),
            "{name}'s own (pre-prologue) instance methods, from Setup.cpp directly"
        );
    }
}

/// `.Set`'s live instance, `task3_fixround1.rex` Part A's `Set|OWN_INSTANCE`
/// line, carries names beyond `own_instance_method_names(Set)` -- because
/// `~inheritInstanceMethods` rescopes `.SetMixin`'s donation
/// (`CoreClasses.orx:85`, `SetMixin` itself defined at `:411`) onto `Set`
/// itself (`RexxClass::inheritInstanceMethods`'s `setMethodScope`,
/// `ClassClass.cpp:563`), so a direct scope-exact query against
/// `.SetMixin` finds nothing there any more (confirmed empty by
/// `task3_fixround1.rex` Part C's `set_from_setmixin`) -- the names have to
/// be read from `Set`'s own line instead: `XOR`, `INTERSECTION`, `UNION`,
/// `SUBSET`, `PUTALL`. `.SetCollection` (`~inherit`ed separately, `:114`)
/// donates no additional name -- `grep` confirms its own definition
/// (`:1296`) declares no methods at all, so an empty result there is not
/// "abstract declarations `Set` already answers", it is that there is
/// nothing to declare. Paired with a native name's presence, since an
/// absence-only assertion cannot distinguish this crate's real flattened
/// set from a degenerate one that answers nothing at all.
#[test]
fn sets_flattened_set_lacks_setmixins_donated_methods() {
    let r = native_classes();
    let set_class = r.lookup("SET").unwrap();
    let native = r.instance_method_names(set_class);
    assert!(
        native.contains("ITEMS"),
        "Set's own method, unaffected by the donation"
    );
    for name in ["XOR", "INTERSECTION", "UNION", "SUBSET", "PUTALL"] {
        assert!(
            !native.contains(name),
            "{name} recorded present on a live .Set instance via SetMixin \
             but should be absent from this crate's pre-prologue native set"
        );
    }
}

/// `.Relation`'s live instance, `task3_fixround1.rex` Part A's
/// `Relation|OWN_INSTANCE` line, carries `.ManyItemMixin`'s donated methods
/// (`CoreClasses.orx:82`, `ManyItemMixin` itself defined at `:218`) rescoped
/// onto `Relation` for the identical reason `Set`'s case states in full:
/// `UNION`, `DIFFERENCE`, `XOR`, `INTERSECTION`, `SUBSET`.
#[test]
fn relations_flattened_set_lacks_manyitemmixins_donated_methods() {
    let r = native_classes();
    let relation = r.lookup("RELATION").unwrap();
    let native = r.instance_method_names(relation);
    assert!(
        native.contains("ITEMS"),
        "Relation's own method, unaffected by the donation"
    );
    for name in ["UNION", "DIFFERENCE", "XOR", "INTERSECTION", "SUBSET"] {
        assert!(
            !native.contains(name),
            "{name} recorded present on a live .Relation instance via \
             ManyItemMixin but should be absent from this crate's \
             pre-prologue native set"
        );
    }
}

/// `.Bag`'s live instance, `task3_fixround1.rex` Part A's `Bag|OWN_INSTANCE`
/// line, carries both `.ManyItemMixin`'s (`CoreClasses.orx:83`) and
/// `.BagMixin`'s (`:87`, `BagMixin` itself defined at `:557`) donated
/// methods, rescoped onto `Bag` the same way: `UNION`, `XOR`,
/// `INTERSECTION`, `DIFFERENCE`, `SUBSET`, `PUTALL` (`ManyItemMixin`'s and
/// `BagMixin`'s own donations overlap except for `BagMixin`'s own
/// `PUTALL`).
#[test]
fn bags_flattened_set_lacks_manyitemmixin_and_bagmixins_donated_methods() {
    let r = native_classes();
    let bag = r.lookup("BAG").unwrap();
    let native = r.instance_method_names(bag);
    assert!(
        native.contains("ITEMS"),
        "Bag's own method, unaffected by the donation"
    );
    for name in [
        "UNION",
        "XOR",
        "INTERSECTION",
        "DIFFERENCE",
        "SUBSET",
        "PUTALL",
    ] {
        assert!(
            !native.contains(name),
            "{name} recorded present on a live .Bag instance via \
             ManyItemMixin/BagMixin but should be absent from this crate's \
             pre-prologue native set"
        );
    }
}

/// `.Supplier`'s live instance, `task3_fixround1.rex` Part A's
/// `Supplier|OWN_INSTANCE` line, carries `.SupplierMixin`'s donated methods
/// (`CoreClasses.orx:80`) rescoped onto `Supplier`: `ALLINDEXES`,
/// `ALLITEMS`, `GETARRAYS`, `SUPPLIER`. A direct scope-exact query against
/// `.SupplierMixin` itself finds nothing there any more, confirmed empty by
/// `task3_fixround1.rex` Part C's `supplier_from_suppliermixin` -- the same
/// mechanism `Set`/`Bag`/`Relation` above have.
#[test]
fn suppliers_flattened_set_lacks_suppliermixins_donated_methods() {
    let r = native_classes();
    let supplier = r.lookup("SUPPLIER").unwrap();
    let native = r.instance_method_names(supplier);
    assert!(
        native.contains("AVAILABLE"),
        "Supplier's own method, unaffected by the donation"
    );
    for name in ["ALLITEMS", "ALLINDEXES", "GETARRAYS", "SUPPLIER"] {
        assert!(
            !native.contains(name),
            "{name} recorded present on a live .Supplier instance via \
             SupplierMixin (CoreClasses.orx:80) but should be absent from \
             this crate's pre-prologue native set"
        );
    }
}

// ---------------------------------------------------------------------
// The deferral table: a test over the derived list, not a fixed count.
// ---------------------------------------------------------------------

/// The brief's own done-when condition, restated as a property rather than
/// a count: every derived checklist entry is native or deferred, checked
/// both directions -- walking `checklist` to catch a name that is neither
/// native nor deferred, and separately walking `deferrals` to catch a
/// deferral naming a token the checklist does not have. What "every
/// deferral names a mechanism" actually reduces to in this test: the
/// reason is non-empty, does not contain the specific phrase "not needed",
/// and clears a length floor. That is a floor against an empty or
/// near-vacuous reason reappearing, not a check that the reason cites a
/// real artifact -- this test cannot verify that a citation is genuine, so
/// it does not claim to. `native_classes.rs`'s own internal unit tests
/// check the stronger, implementation-visible version of the
/// native-or-deferred property (the checklist-token-to-block-name mapping
/// and the registration itself); this is the public-API shape of the same
/// guarantee.
#[test]
fn every_setup_class_is_native_or_deferred_with_a_reason() {
    let checklist = setup_class_names();
    let deferrals = deferred_classes();
    assert!(
        !checklist.is_empty(),
        "the derived checklist must not be empty"
    );

    let r = native_classes();
    for &token in checklist {
        let deferred = deferrals.iter().find(|d| d.setup_class == token);
        match deferred {
            Some(d) => {
                assert!(
                    !d.reason.is_empty(),
                    "{token:?} has an empty deferral reason"
                );
                // What this actually checks: the reason avoids the specific
                // forbidden phrase and clears a length floor. Neither is a
                // real "names a mechanism" check -- a 41-character string
                // with no citation in it passes -- so this is a floor, not
                // a substitute for the human read the brief's ruling asks
                // for; it exists to catch an empty or near-empty reason
                // being silently reintroduced, nothing subtler.
                assert!(
                    !d.reason.to_ascii_lowercase().contains("not needed"),
                    "{token:?}'s reason must not decline on timing"
                );
                assert!(
                    d.reason.len() > 40,
                    "{token:?}'s reason ({:?}) is too short to be a real explanation",
                    d.reason
                );
            }
            None => {
                // Not deferred -- must actually be registered.
                let block_name = block_name_for(token);
                assert!(
                    r.lookup(&block_name.to_ascii_uppercase()).is_some(),
                    "{token:?} is neither deferred nor registered as {block_name:?}"
                );
            }
        }
    }
    // The reverse direction the loop above cannot see: every deferral's own
    // token must itself be a real checklist entry, or a stale deferral
    // (naming a class Setup.cpp no longer declares) would go unnoticed --
    // the loop above only ever looks *up* from the checklist into the
    // deferral table, never the other way.
    for d in deferrals {
        assert!(
            checklist.contains(&d.setup_class),
            "deferral {:?} names a token not in the derived checklist",
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

/// **Every identity this registry mints is a class identity**, disjoint from
/// anything `rexx_core::Heap` can allocate.
///
/// The registry's `ObjRef`s travel out of this crate -- a resolved send hands
/// one back as its scope, and a later phase makes a class object a value -- so
/// a handle that could equal an arena slot is one a consumer would resolve
/// against the arena. `rexx_core::CLASS_SLOT_BASE`'s own doc has the failure
/// mode; `rexx-core`'s `a_class_identity_is_never_an_arena_handle` is the
/// other half of the pair, over the handle type itself.
///
/// **Before `reserve_id` used `ObjRef::class`** it minted `ObjRef::heap(n, 0)`
/// from a counter starting at zero, so every assertion below answered `None`
/// and this test fails on the first class it looks at.
#[test]
fn every_registered_class_identity_is_outside_the_arenas_slot_range() {
    let registry = rexx_classes::native_classes();
    // Named rather than swept, so a lookup that started answering `None`
    // fails here instead of silently shrinking what this reads. The bootstrap
    // pair, a plain class and a donation recipient, which is every
    // construction path `native_classes` has.
    for name in ["Object", "Class", "String", "Array", "Bag"] {
        let class = registry
            .lookup(name)
            .unwrap_or_else(|| panic!("{name} is a native class"));
        assert!(
            class.class_id().is_some(),
            "the class registered for {name} has an identity the arena could also produce"
        );
    }
    // A class installed after the bootstrap takes its identity from the same
    // counter, so the property has to hold past the native set too.
    let mut registry = registry;
    let object = registry.lookup("Object").expect("Object is a native class");
    let metaclass = registry.lookup("Class").expect("Class is a native class");
    let user = registry.define_class("Widget", Some(object), ClassKind::Regular, metaclass);
    assert!(
        user.class_id().is_some(),
        "a class installed after the bootstrap has an arena-shaped identity"
    );
}
