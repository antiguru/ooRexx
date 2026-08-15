//! Every test here carries an oracle answer recorded *before* this crate's
//! code was written, from a probe run as
//! `( ulimit -v 1048576; LD_LIBRARY_PATH=.../ooRexx/build/lib .../ooRexx/build/bin/rexx FILE )`
//! against `/home/moritz/dev/repos/ooRexx/build/bin/rexx`. Each test names
//! its probe file (kept in the task report, not the tree) and states the
//! exact recorded stdout it reproduces.
//!
//! One mechanism, `inheritInstanceMethods`, has no live probe: it is
//! deleted from every saved image by `removeSetupMethods` before the
//! oracle ships (D39), so no running script can ever send it -- measured,
//! `Error 97.1: Object "The Target class" does not understand message
//! "INHERITINSTANCEMETHODS"`. Its tests below are built from
//! `RexxClass::inheritInstanceMethods` (`ClassClass.cpp:558-586`) read
//! directly, cross-checked against its *effect* on the shipped `.Supplier`,
//! `.Set` and `.Bag` classes, which a running script can and does measure.

use rexx_classes::{ClassGraph, ClassKind, MethodId};
use rexx_core::ObjRef;

/// Every test builds its own tiny graph, so a plain counter naming test
/// bodies' local classes is enough; no two tests share a `ClassGraph`.
fn id(n: u32) -> ObjRef {
    ObjRef::heap(n, 0)
}

// ---------------------------------------------------------------------
// Probe: mixin_merge.rex / mixin_merge2.rex -- two mixins define one name.
// ---------------------------------------------------------------------

/// `mixin_merge.rex`: `Mixin1` and `Mixin2` (`mixinclass object`) both
/// define `METHODA`; `Combo subclass object inherit Mixin1 Mixin2`.
/// Recorded: `methoda= M1` -- the *first-listed* `INHERIT` mixin wins.
#[test]
fn first_listed_inherit_mixin_wins_a_name_conflict() {
    let object = id(1);
    let mixin1 = id(2);
    let mixin2 = id(3);
    let combo = id(4);

    let mut g = ClassGraph::new();
    g.define_class(object, None, ClassKind::Regular);
    g.define_class(mixin1, Some(object), ClassKind::Mixin);
    g.define(mixin1, "METHODA", MethodId(1));
    g.define_class(mixin2, Some(object), ClassKind::Mixin);
    g.define(mixin2, "METHODA", MethodId(2));

    g.define_class(combo, Some(object), ClassKind::Regular);
    g.inherit(combo, mixin1);
    g.inherit(combo, mixin2);

    assert_eq!(
        g.lookup_at(g.instance_behaviour_handle(combo), "METHODA"),
        Some((mixin1, MethodId(1))),
        "mixin_merge.rex recorded 'methoda= M1'"
    );
}

/// `mixin_merge2.rex`: the same two mixins, `INHERIT`ed in the opposite
/// order. Recorded: `methoda= M2` -- confirms the rule is "first-listed
/// wins", not "the mixin named `Mixin1` always wins".
#[test]
fn reversing_the_inherit_list_reverses_the_winner() {
    let object = id(1);
    let mixin1 = id(2);
    let mixin2 = id(3);
    let combo = id(4);

    let mut g = ClassGraph::new();
    g.define_class(object, None, ClassKind::Regular);
    g.define_class(mixin1, Some(object), ClassKind::Mixin);
    g.define(mixin1, "METHODA", MethodId(1));
    g.define_class(mixin2, Some(object), ClassKind::Mixin);
    g.define(mixin2, "METHODA", MethodId(2));

    g.define_class(combo, Some(object), ClassKind::Regular);
    g.inherit(combo, mixin2);
    g.inherit(combo, mixin1);

    assert_eq!(
        g.lookup_at(g.instance_behaviour_handle(combo), "METHODA"),
        Some((mixin2, MethodId(2))),
        "mixin_merge2.rex recorded 'methoda= M2'"
    );
}

/// `mixin_vs_super.rex`: `Combo subclass Base inherit Mixin1`, where `Base`
/// (the explicit superclass) and `Mixin1` both define `METHODA`. Recorded:
/// `methoda= BASE` -- an explicit superclass always outranks an inherited
/// mixin, regardless of `INHERIT` order.
#[test]
fn an_explicit_superclass_outranks_every_inherited_mixin() {
    let object = id(1);
    let base = id(2);
    let mixin1 = id(3);
    let combo = id(4);

    let mut g = ClassGraph::new();
    g.define_class(object, None, ClassKind::Regular);
    g.define_class(base, Some(object), ClassKind::Regular);
    g.define(base, "METHODA", MethodId(1));
    g.define_class(mixin1, Some(object), ClassKind::Mixin);
    g.define(mixin1, "METHODA", MethodId(2));

    g.define_class(combo, Some(base), ClassKind::Regular);
    g.inherit(combo, mixin1);

    assert_eq!(
        g.lookup_at(g.instance_behaviour_handle(combo), "METHODA"),
        Some((base, MethodId(1))),
        "mixin_vs_super.rex recorded 'methoda= BASE'"
    );
}

// ---------------------------------------------------------------------
// Probe: diamond.rex -- a diamond, for which scope wins.
// ---------------------------------------------------------------------

/// `diamond.rex`: `MixinA` and `MixinB` both `inherit MixinBase` and both
/// override `METHODX`; `Combo subclass object inherit MixinA MixinB`.
/// Recorded: `methodx= A_X` (first-listed mixin wins, same rule as the
/// non-diamond case) and `methody= BASE_Y` (the common ancestor's own,
/// untouched method comes through cleanly -- the diamond does not
/// duplicate or lose it).
#[test]
fn a_diamond_still_prefers_the_first_listed_mixin_and_keeps_the_common_ancestors_own_method() {
    let object = id(1);
    let mixin_base = id(2);
    let mixin_a = id(3);
    let mixin_b = id(4);
    let combo = id(5);

    let mut g = ClassGraph::new();
    g.define_class(object, None, ClassKind::Regular);

    g.define_class(mixin_base, Some(object), ClassKind::Mixin);
    g.define(mixin_base, "METHODX", MethodId(1)); // BASE_X
    g.define(mixin_base, "METHODY", MethodId(2)); // BASE_Y

    g.define_class(mixin_a, Some(object), ClassKind::Mixin);
    g.inherit(mixin_a, mixin_base);
    g.define(mixin_a, "METHODX", MethodId(3)); // A_X

    g.define_class(mixin_b, Some(object), ClassKind::Mixin);
    g.inherit(mixin_b, mixin_base);
    g.define(mixin_b, "METHODX", MethodId(4)); // B_X

    g.define_class(combo, Some(object), ClassKind::Regular);
    g.inherit(combo, mixin_a);
    g.inherit(combo, mixin_b);

    let handle = g.instance_behaviour_handle(combo);
    assert_eq!(
        g.lookup_at(handle, "METHODX"),
        Some((mixin_a, MethodId(3))),
        "diamond.rex recorded 'methodx= A_X'"
    );
    assert_eq!(
        g.lookup_at(handle, "METHODY"),
        Some((mixin_base, MethodId(2))),
        "diamond.rex recorded 'methody= BASE_Y'"
    );
}

// ---------------------------------------------------------------------
// Probe: define_visibility.rex / define_subclass_visibility.rex --
// ~define with an instance already created.
// ---------------------------------------------------------------------

/// `define_visibility.rex`: create `old = .Widget~new`, then
/// `.Widget~define('FOO', ...)`, then create `new = .Widget~new`. Recorded:
/// `old hasmethod FOO= 0`, `new hasmethod FOO= 1` -- the pre-existing
/// instance must NOT see a `~define`.
#[test]
fn define_does_not_reach_an_instance_created_before_it() {
    let object = id(1);
    let widget = id(2);

    let mut g = ClassGraph::new();
    g.define_class(object, None, ClassKind::Regular);
    g.define_class(widget, Some(object), ClassKind::Regular);

    let old_handle = g.instance_behaviour_handle(widget); // "old = .Widget~new"
    g.define(widget, "FOO", MethodId(1));
    let new_handle = g.instance_behaviour_handle(widget); // "new = .Widget~new"

    assert!(
        !g.has_method_at(old_handle, "FOO"),
        "define_visibility.rex recorded 'old hasmethod FOO= 0'"
    );
    assert!(
        g.has_method_at(new_handle, "FOO"),
        "define_visibility.rex recorded 'new hasmethod FOO= 1'"
    );
}

/// `define_subclass_visibility.rex`: `Sub subclass Base`; create one
/// instance of each, then `.Base~define('QUUX', ...)`. Recorded:
/// `oldBase hasmethod QUUX= 0`, `oldSub hasmethod QUUX= 1` -- the
/// asymmetry is isolated to the *direct* recipient. A subclass's
/// already-created instance DOES see a superclass's `~define`, because
/// only `Base`'s handle is replaced; `Sub`'s is rebuilt in place.
#[test]
fn define_on_a_superclass_still_reaches_an_existing_subclass_instance() {
    let object = id(1);
    let base = id(2);
    let sub = id(3);

    let mut g = ClassGraph::new();
    g.define_class(object, None, ClassKind::Regular);
    g.define_class(base, Some(object), ClassKind::Regular);
    g.define_class(sub, Some(base), ClassKind::Regular);

    let old_base = g.instance_behaviour_handle(base); // "oldBase = .Base~new"
    let old_sub = g.instance_behaviour_handle(sub); // "oldSub = .Sub~new"

    g.define(base, "QUUX", MethodId(1));

    assert!(
        !g.has_method_at(old_base, "QUUX"),
        "define_subclass_visibility.rex recorded 'oldBase hasmethod QUUX= 0'"
    );
    assert!(
        g.has_method_at(old_sub, "QUUX"),
        "define_subclass_visibility.rex recorded 'oldSub hasmethod QUUX= 1'"
    );
}

// ---------------------------------------------------------------------
// Probe: inherit_visibility.rex / inherit_subclass_visibility.rex --
// ~inherit with an instance already created.
// ---------------------------------------------------------------------

/// `inherit_visibility.rex`: create `old = .Widget~new`, then
/// `.Widget~inherit(.Mixin1)` where `Mixin1` defines `BAR`. Recorded:
/// `old hasmethod BAR= 1` -- unlike `~define`, the pre-existing instance
/// MUST see an `~inherit`.
#[test]
fn inherit_reaches_an_instance_created_before_it() {
    let object = id(1);
    let widget = id(2);
    let mixin1 = id(3);

    let mut g = ClassGraph::new();
    g.define_class(object, None, ClassKind::Regular);
    g.define_class(widget, Some(object), ClassKind::Regular);
    g.define_class(mixin1, Some(object), ClassKind::Mixin);
    g.define(mixin1, "BAR", MethodId(1));

    let old_handle = g.instance_behaviour_handle(widget); // "old = .Widget~new"
    g.inherit(widget, mixin1);

    assert!(
        g.has_method_at(old_handle, "BAR"),
        "inherit_visibility.rex recorded 'old hasmethod BAR= 1'"
    );
    // The handle itself never changed -- `~inherit` never copies.
    assert_eq!(old_handle, g.instance_behaviour_handle(widget));
}

/// `inherit_subclass_visibility.rex`: `Sub subclass Base`; create one
/// instance of each, then `.Base~inherit(.Mixin1)` where `Mixin1` defines
/// `BAZ`. Recorded: `oldBase hasmethod BAZ= 1`, `oldSub hasmethod BAZ= 1`
/// -- `~inherit`'s cascade reaches every already-created instance, at
/// every depth, because no copy happens anywhere in it.
#[test]
fn inherit_on_a_superclass_reaches_an_existing_subclass_instance_too() {
    let object = id(1);
    let base = id(2);
    let sub = id(3);
    let mixin1 = id(4);

    let mut g = ClassGraph::new();
    g.define_class(object, None, ClassKind::Regular);
    g.define_class(base, Some(object), ClassKind::Regular);
    g.define_class(sub, Some(base), ClassKind::Regular);
    g.define_class(mixin1, Some(object), ClassKind::Mixin);
    g.define(mixin1, "BAZ", MethodId(1));

    let old_base = g.instance_behaviour_handle(base);
    let old_sub = g.instance_behaviour_handle(sub);

    g.inherit(base, mixin1);

    assert!(
        g.has_method_at(old_base, "BAZ"),
        "inherit_subclass_visibility.rex recorded 'oldBase hasmethod BAZ= 1'"
    );
    assert!(
        g.has_method_at(old_sub, "BAZ"),
        "inherit_subclass_visibility.rex recorded 'oldSub hasmethod BAZ= 1'"
    );
}

// ---------------------------------------------------------------------
// Probe: mixinclass_object_class_method.rex -- the class-behaviour side
// (D44).
// ---------------------------------------------------------------------

/// `mixinclass_object_class_method.rex`: `GreeterMixin` (`mixinclass
/// object`) defines a CLASS method `GREET`; `MyClass subclass object
/// inherit GreeterMixin`. Recorded: `MyClass~hasmethod(GREET)= 1`,
/// `MyClass~greet= CLASS-SIDE GREET`, `inst~hasmethod(GREET)= 0` -- a plain
/// `INHERIT` cascades a mixin's own class-side methods into the
/// *class object's own* behaviour (D44's "a class carries two
/// behaviours"), not into its instances' behaviour.
#[test]
fn inherit_donates_a_mixins_class_side_methods_to_the_class_object_not_its_instances() {
    let object = id(1);
    let greeter_mixin = id(2);
    let my_class = id(3);

    let mut g = ClassGraph::new();
    g.define_class(object, None, ClassKind::Regular);
    g.define_class(greeter_mixin, Some(object), ClassKind::Mixin);

    // `::method greet class` -- a CLASS method, installed via
    // `class_define` (this crate's own setup convenience for the class
    // side, see its doc comment) before `my_class` inherits the mixin.
    g.class_define(greeter_mixin, "GREET", MethodId(1));

    g.define_class(my_class, Some(object), ClassKind::Regular);
    g.inherit(my_class, greeter_mixin);

    assert!(
        g.class_has_method(my_class, "GREET"),
        "mixinclass_object_class_method.rex recorded 'MyClass~hasmethod(GREET)= 1'"
    );
    assert_eq!(
        g.lookup_at(g.class_behaviour_handle(my_class), "GREET"),
        Some((greeter_mixin, MethodId(1))),
        "mixinclass_object_class_method.rex recorded 'MyClass~greet= CLASS-SIDE GREET'"
    );
    assert!(
        !g.has_method(my_class, "GREET"),
        "mixinclass_object_class_method.rex recorded 'inst~hasmethod(GREET)= 0'"
    );
}

// ---------------------------------------------------------------------
// Probe: inherit_instance_methods.rex, and the brief's own measured facts
// re-verified -- "a class-graph assertion cannot see
// inheritInstanceMethods".
// ---------------------------------------------------------------------

/// `inherit_instance_methods.rex`: `Target~inheritInstanceMethods(.Donor)`,
/// where `Donor` defines `DONATED`. Recorded: `Target~superClasses`
/// unchanged (`The Object class`, before and after) while an instance
/// created after the call answers `DONATED` and one created before it does
/// not -- the donation adds no superclass edge, and rebuilds only the
/// instance behaviour of the class it is sent to (no cascade).
#[test]
fn inherit_instance_methods_donates_without_a_superclass_edge() {
    let object = id(1);
    let target = id(2);
    let donor = id(3);

    let mut g = ClassGraph::new();
    g.define_class(object, None, ClassKind::Regular);
    g.define_class(target, Some(object), ClassKind::Regular);
    g.define_class(donor, Some(object), ClassKind::Regular);
    g.define(donor, "DONATED", MethodId(1));

    let ancestors_before: Vec<ObjRef> = g.ancestors(target).to_vec();
    let before_handle = g.instance_behaviour_handle(target); // "before = .Target~new"
    assert!(
        !g.has_method_at(before_handle, "DONATED"),
        "inherit_instance_methods.rex recorded 'before hasmethod DONATED= 0'"
    );

    g.inherit_instance_methods(target, donor);

    let after_handle = g.instance_behaviour_handle(target); // "after = .Target~new"

    assert_eq!(
        g.ancestors(target),
        ancestors_before,
        "inherit_instance_methods.rex recorded 'Target~superClasses' unchanged"
    );
    // No copy anywhere in inheritInstanceMethods (unlike `~define`): the
    // handle is rebuilt in place, so `before_handle` and `after_handle`
    // are the same entry, and it now has the donated method too.
    assert_eq!(before_handle, after_handle);
    assert!(
        g.has_method_at(after_handle, "DONATED"),
        "inherit_instance_methods.rex recorded 'after hasmethod DONATED= 1'"
    );
    // The donated method now answers to `target`'s own scope, not
    // `donor`'s -- `setMethodScope`, what makes `newScope` on a donated
    // method report the recipient rather than the class that wrote it.
    assert_eq!(
        g.lookup_at(after_handle, "DONATED"),
        Some((target, MethodId(1)))
    );
}

/// The brief's central hazard, independently re-verified against the
/// oracle (`wiring_blind_spot.rex`): `.Set~superClasses` and
/// `.Bag~superClasses` are byte-identical -- three entries, "The Object
/// class", "The MapCollection class", "The SetCollection class", in that
/// order, on both -- despite `.set~inheritInstanceMethods(.SetMixin)` and
/// `.bag~inheritInstanceMethods(.BagMixin)` (`CoreClasses.orx:85,87`)
/// donating different content.
///
/// `~superClasses` (`RexxClass::getSuperClasses`, `ClassClass.cpp:458-461`)
/// answers a plain copy of the class's own `superClasses` field, not a
/// flattened ancestor closure -- and reading `CoreClasses.orx:103-115`
/// shows exactly how `.Set` ends up with three entries there: it is built
/// in C++ with `superClasses = [Object]` alone, then
/// `.set~inherit(.MapCollection)` (`:108`) and `.set~inherit(.SetCollection)`
/// (`:114`) each append one more mixin, in that order, after the fact.
/// This test reproduces that exact shape rather than a shallower one: two
/// classes built the same way, ending up with the identical three-entry
/// chain the oracle measures, each then receiving a **different**
/// donation. A class-graph assertion (`ancestors`) cannot tell them apart;
/// a method-SET assertion (`method_names_at`) can and must.
#[test]
fn two_classes_with_identical_ancestors_can_still_answer_to_different_method_sets() {
    let object = id(1);
    let map_collection = id(2);
    let set_collection = id(3);
    let set_mixin = id(4);
    let bag_mixin = id(5);
    let set_like = id(6);
    let bag_like = id(7);

    let mut g = ClassGraph::new();
    g.define_class(object, None, ClassKind::Regular);
    g.define_class(map_collection, Some(object), ClassKind::Mixin);
    g.define_class(set_collection, Some(object), ClassKind::Mixin);

    g.define_class(set_mixin, Some(object), ClassKind::Regular);
    g.define(set_mixin, "SETONLY", MethodId(1));
    g.define_class(bag_mixin, Some(object), ClassKind::Regular);
    g.define(bag_mixin, "BAGONLY", MethodId(2));

    // Both start as a bootstrap SUBCLASS(Object), matching how the
    // primitive .Set/.Bag are built in Setup.cpp, then each receives the
    // same two ~inherit calls in the same order CoreClasses.orx does.
    g.define_class(set_like, Some(object), ClassKind::Regular);
    g.define_class(bag_like, Some(object), ClassKind::Regular);
    g.inherit(set_like, map_collection);
    g.inherit(set_like, set_collection);
    g.inherit(bag_like, map_collection);
    g.inherit(bag_like, set_collection);

    g.inherit_instance_methods(set_like, set_mixin);
    g.inherit_instance_methods(bag_like, bag_mixin);

    assert_eq!(
        g.ancestors(set_like),
        [object, map_collection, set_collection],
        "matches the measured .Set~superClasses shape: three entries, Object first"
    );
    assert_eq!(
        g.ancestors(set_like),
        g.ancestors(bag_like),
        "the class graph is byte-identical, exactly like .Set~superClasses and .Bag~superClasses"
    );

    let set_methods = g.method_names_at(g.instance_behaviour_handle(set_like));
    let bag_methods = g.method_names_at(g.instance_behaviour_handle(bag_like));
    assert!(set_methods.contains("SETONLY"));
    assert!(!set_methods.contains("BAGONLY"));
    assert!(bag_methods.contains("BAGONLY"));
    assert!(!bag_methods.contains("SETONLY"));
    assert_ne!(
        set_methods, bag_methods,
        "identical ancestors, disjoint method sets -- the exact blind spot \
         a class-graph-only assertion cannot see"
    );
}

// ---------------------------------------------------------------------
// `inherit`'s validation guards -- not oracle probes (these panic; no
// probe can run an invalid `inherit` against the built oracle without
// first tripping the identical `SYNTAX` condition and exiting), but each
// is a distinct check `RexxClass::inherit` (`ClassClass.cpp:1298-1332`)
// makes that this crate's own code must refuse identically. One test per
// guard, each stating what letting it through would have done.
// ---------------------------------------------------------------------

/// Guard: `mixin` must actually be declared `MIXINCLASS` -- oracle's
/// `Error_Execution_mixinclass` (`ClassClass.cpp:1299`). Absent this
/// guard, an ordinary (non-mixin) class would be silently accepted as an
/// `INHERIT` target: `combo.superclasses` would gain `plain` as an
/// ancestor the oracle would have refused outright, and every later
/// cascade would merge `plain`'s methods into `combo` as if the
/// relationship were legal.
#[test]
#[should_panic(expected = "is not a MIXINCLASS")]
fn inherit_refuses_a_non_mixin_class() {
    let object = id(1);
    let plain = id(2);
    let combo = id(3);

    let mut g = ClassGraph::new();
    g.define_class(object, None, ClassKind::Regular);
    g.define_class(plain, Some(object), ClassKind::Regular);
    g.define_class(combo, Some(object), ClassKind::Regular);

    g.inherit(combo, plain);
}

/// Guard: `inherit` refuses a `mixin` that is already an ancestor of
/// `class` -- oracle's `Error_Execution_recursive_inherit`
/// (`ClassClass.cpp:1311`). Absent this guard, re-inheriting the same
/// mixin would push a duplicate entry onto `class.superclasses` and a
/// duplicate cascade registration onto `mixin.subclasses`;
/// `cascade_build`'s own `has_scope` guard hides the method-merge
/// consequence, but the graph itself would be wrong, and a later rebuild
/// of `mixin` would redundantly rebuild `class` twice over.
#[test]
#[should_panic(expected = "re-inheriting it is a recursive inherit")]
fn inherit_refuses_re_inheriting_the_same_mixin() {
    let object = id(1);
    let mixin1 = id(2);
    let combo = id(3);

    let mut g = ClassGraph::new();
    g.define_class(object, None, ClassKind::Regular);
    g.define_class(mixin1, Some(object), ClassKind::Mixin);
    g.define_class(combo, Some(object), ClassKind::Regular);

    g.inherit(combo, mixin1);
    g.inherit(combo, mixin1); // already an ancestor
}

/// Guard: `inherit` refuses making `class` an ancestor of a `mixin` that
/// is itself already an ancestor of `class` -- oracle's
/// `Error_Execution_recursive_inherit`'s mirror check (`:1317`). Absent
/// this guard: `define_class(b, Some(a), Mixin)` (`B mixinclass A`) then
/// `inherit(a, b)` (`.A~inherit(.B)`) passes every other check (`B` is a
/// genuine `MIXINCLASS`, `A` is not already `B`'s ancestor, the
/// base-class checks hold trivially since `B`'s base is `A` itself),
/// leaves `a.superclasses = [.., b]` and `b.superclasses = [a]`, and
/// `update_sub_classes` (`class_graph.rs`) alternates `a -> b -> a`
/// forever -- a stack overflow where the oracle raises a clean `98.943`.
#[test]
#[should_panic(expected = "would make each an ancestor of the other")]
fn inherit_refuses_a_cycle_through_a_mixins_own_mixinclass_target() {
    let a = id(1);
    let b = id(2);

    let mut g = ClassGraph::new();
    g.define_class(a, None, ClassKind::Regular);
    g.define_class(b, Some(a), ClassKind::Mixin); // B mixinclass A

    g.inherit(a, b); // .A~inherit(.B) -- would cycle without the guard
}

// ---------------------------------------------------------------------
// Design-decision tests, not oracle probes: D29's scope-ordering data and
// monotonic version. These read `MethodDictionary::resolveSuperScope`
// (`MethodDictionary.cpp:569`) and `findSuperMethod` (`:434`) directly --
// Task 2 builds no dispatch, so there is no scope-override *send* to
// probe against the oracle; only the data these functions would read.
// ---------------------------------------------------------------------

/// A linear chain, no mixins: `resolve_super_scope` (oracle's
/// `resolveSuperScope`) must answer each class's true immediate ancestor,
/// and `None` (oracle's `.nil`) for the topmost scope.
#[test]
fn resolve_super_scope_answers_the_immediate_ancestor_in_a_linear_chain() {
    let object = id(1);
    let base = id(2);
    let sub = id(3);

    let mut g = ClassGraph::new();
    g.define_class(object, None, ClassKind::Regular);
    g.define_class(base, Some(object), ClassKind::Regular);
    g.define_class(sub, Some(base), ClassKind::Regular);

    let handle = g.instance_behaviour_handle(sub);
    assert_eq!(
        g.lookup_at(handle, "NONEXISTENT"),
        None,
        "sanity: nobody defines this method"
    );
    assert_eq!(g.resolve_super_scope_at(handle, sub), Some(base));
    assert_eq!(g.resolve_super_scope_at(handle, base), Some(object));
    assert_eq!(
        g.resolve_super_scope_at(handle, object),
        None,
        "the topmost scope has no superscope, matching the oracle's .nil"
    );
}

/// The diamond graph again: a scope-override lookup rooted at `MixinB`
/// finds `MixinB`'s own `METHODX`, not `MixinA`'s -- even though an
/// ordinary (unscoped) lookup on the same behaviour answers `MixinA`'s.
/// This is the exact case the brief cites for why scope ordering has to
/// be retained at all: "a scope-override send cannot be answered without
/// them".
#[test]
fn a_scope_rooted_lookup_can_disagree_with_the_ordinary_lookup() {
    let object = id(1);
    let mixin_base = id(2);
    let mixin_a = id(3);
    let mixin_b = id(4);
    let combo = id(5);

    let mut g = ClassGraph::new();
    g.define_class(object, None, ClassKind::Regular);
    g.define_class(mixin_base, Some(object), ClassKind::Mixin);
    g.define(mixin_base, "METHODX", MethodId(1));
    g.define_class(mixin_a, Some(object), ClassKind::Mixin);
    g.inherit(mixin_a, mixin_base);
    g.define(mixin_a, "METHODX", MethodId(2));
    g.define_class(mixin_b, Some(object), ClassKind::Mixin);
    g.inherit(mixin_b, mixin_base);
    g.define(mixin_b, "METHODX", MethodId(3));
    g.define_class(combo, Some(object), ClassKind::Regular);
    g.inherit(combo, mixin_a);
    g.inherit(combo, mixin_b);

    let handle = g.instance_behaviour_handle(combo);
    assert_eq!(
        g.lookup_at(handle, "METHODX"),
        Some((mixin_a, MethodId(2))),
        "ordinary lookup: first-listed mixin wins, as in diamond.rex"
    );
    assert_eq!(
        g.lookup_from_scope_at(handle, "METHODX", mixin_b),
        Some((mixin_b, MethodId(3))),
        "a lookup rooted at MixinB's own scope finds MixinB's own override, \
         not MixinA's -- unreachable from an ordinary lookup alone"
    );
}

/// D29: "every behaviour carries a monotonic version, bumped by the
/// cascade". Not an oracle fact (there is nothing in a running script
/// that reads it) -- a design decision the spec records, checked here as
/// a property of this crate's own bookkeeping: every cascade that touches
/// a handle bumps its version by exactly one, and a handle `define`
/// orphans stops advancing forever.
#[test]
fn every_cascade_against_a_handle_bumps_its_version_by_exactly_one() {
    let object = id(1);
    let base = id(2);
    let sub = id(3);
    let mixin1 = id(4);

    let mut g = ClassGraph::new();
    g.define_class(object, None, ClassKind::Regular);
    g.define_class(base, Some(object), ClassKind::Regular);
    g.define_class(sub, Some(base), ClassKind::Regular);
    g.define_class(mixin1, Some(object), ClassKind::Mixin);
    g.define(mixin1, "X", MethodId(1));

    let sub_handle = g.instance_behaviour_handle(sub);
    let v0 = g.version_at(sub_handle);

    g.inherit(base, mixin1); // cascades to sub in place
    assert_eq!(g.version_at(sub_handle), v0 + 1);

    let orphaned = g.instance_behaviour_handle(base);
    let v_base = g.version_at(orphaned);
    g.define(base, "Y", MethodId(2)); // orphans `orphaned`, allocates a new handle
    assert_eq!(
        g.version_at(orphaned),
        v_base,
        "an orphaned handle's version is frozen -- nothing writes to it again"
    );
    // The fresh handle `define` allocates starts its own count at zero and
    // is bumped once by its own rebuild -- it is a different entry, so its
    // version is not comparable to the orphaned handle's.
    let fresh = g.instance_behaviour_handle(base);
    assert_ne!(fresh, orphaned);
    assert_eq!(g.version_at(fresh), 1);
}
