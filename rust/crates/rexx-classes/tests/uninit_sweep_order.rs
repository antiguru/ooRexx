//! The order the termination sweep runs class objects in.
//!
//! Every expectation here is an oracle transcript recorded before this
//! crate's ordering was written, from probes run as
//! `( ulimit -v 1048576; LD_LIBRARY_PATH=.../ooRexx/build/lib .../ooRexx/build/bin/rexx FILE )`
//! against `/home/moritz/dev/repos/ooRexx/build/bin/rexx`, from a fresh empty
//! directory, three runs each, rc 0 with empty stderr throughout. Each test
//! states the program and the stdout it reproduces.

use rexx_classes::{ClassKind, ClassRegistry};

/// A registry holding `.Object` and `.Class`, and then one directive-style
/// class per name with a class-side `UNINIT`, declared in the order given.
fn declare(names: &[&str]) -> ClassRegistry {
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
    registry.refresh_class_behaviour(class_id);
    registry.bootstrap_root_class_behaviour(object_id, class_id);

    for name in names {
        let id =
            registry.define_unregistered_class(name, Some(object_id), ClassKind::Regular, class_id);
        registry.add_class_method(id, "UNINIT");
        registry.check_uninit(id);
    }
    registry
}

/// The sweep order as id strings.
fn sweep(registry: &ClassRegistry) -> Vec<String> {
    registry
        .uninit_classes_in_sweep_order()
        .into_iter()
        .map(|class| registry.id_string(class).to_string())
        .collect()
}

/// Four classes declared `C`, `B`, `A`, `D`, each with
/// `::METHOD uninit CLASS` saying its own id.
///
/// Recorded: `main` / `uninit on D` / `uninit on A` / `uninit on B` /
/// `uninit on C`. The sweep contradicts declaration order on every class,
/// which is what makes this the discriminating case: an implementation
/// walking the class registry in creation order answers the reverse.
#[test]
fn the_sweep_contradicts_declaration_order() {
    let registry = declare(&["C", "B", "A", "D"]);
    assert_eq!(sweep(&registry), ["D", "A", "B", "C"]);
}

/// Twenty classes declared `A` through `T`.
///
/// Recorded: `u D u E u F u G u H u I u J u K u L u M u N u O u P u Q u A
/// u R u B u S u C u T`. Twenty entries do not expand the oracle's table, so
/// this is the same bucket count as every other case here.
#[test]
fn twenty_classes_still_run_in_bucket_order() {
    let names: Vec<String> = ('A'..='T').map(|c| c.to_string()).collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let registry = declare(&refs);
    assert_eq!(
        sweep(&registry),
        [
            "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "A", "R", "B",
            "S", "C", "T"
        ]
    );
}

/// Three ids that share a bucket, declared in two orders.
///
/// Recorded: declared `AB`, `ZZ`, `K` the oracle answers `u AB u ZZ u K`,
/// and declared `K`, `ZZ`, `AB` it answers `u K u ZZ u AB`. So a bucket's
/// chain is entry order and reversing the declarations reverses the output --
/// the tie-break no other case here can see, since no other case collides.
#[test]
fn a_shared_bucket_keeps_entry_order() {
    let forward = declare(&["AB", "ZZ", "K"]);
    assert_eq!(sweep(&forward), ["AB", "ZZ", "K"]);
    let reverse = declare(&["K", "ZZ", "AB"]);
    assert_eq!(sweep(&reverse), ["K", "ZZ", "AB"]);
}

/// A class with no class-side `UNINIT` is not in the sweep at all, and an
/// instance-side one does not put it there.
///
/// `obdes.rex`'s neighbour, measured: `say 'main'` with `::class k` and
/// `::method uninit` (no `CLASS`) prints `main` and nothing else at rc 0,
/// where the same program with `::method uninit class` prints `uninit ran`
/// after it.
#[test]
fn an_instance_side_uninit_does_not_enter_the_sweep() {
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
    registry.refresh_class_behaviour(class_id);
    registry.bootstrap_root_class_behaviour(object_id, class_id);

    let id = registry.define_unregistered_class("K", Some(object_id), ClassKind::Regular, class_id);
    registry.add_instance_method(id, "UNINIT");
    registry.check_uninit(id);
    assert!(registry.has_uninit(id), "its instances need UNINIT");
    assert!(sweep(&registry).is_empty(), "the class object does not");
}
