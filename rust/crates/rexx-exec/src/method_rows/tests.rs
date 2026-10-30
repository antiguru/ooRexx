use super::MethodRows;
use rexx_classes::MethodId;

/// After the seal, an id the vector covers still lands in the vector,
/// and one past it in the map.
#[test]
fn a_sealed_table_files_by_the_vectors_reach() {
    let mut rows = MethodRows::new();
    rows.insert(MethodId(3), 'a');
    rows.seal();
    rows.insert(MethodId(1), 'b');
    rows.insert(MethodId(9), 'c');
    assert_eq!(rows.extent(), (4, 1));
    assert_eq!(rows.get(MethodId(1)), Some(&'b'));
    rows.remove(MethodId(9));
    assert_eq!(rows.extent(), (4, 0));
}

/// A map row below the vector's end is refused in a debug build.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "sits below the vector's end")]
fn a_minted_row_below_the_vector_is_refused() {
    let mut rows = MethodRows::new();
    rows.insert(MethodId(3), 'a');
    rows.seal();
    rows.minted.insert(MethodId(1), 'b');
    rows.get_mut(MethodId(1));
}

/// Registering another wrapper must not replace an identity already exposed.
#[test]
fn first_registration_wins_on_both_sides_of_the_seal() {
    let mut rows = MethodRows::new();
    rows.insert_if_absent(MethodId(3), 'a');
    rows.insert_if_absent(MethodId(3), 'b');
    rows.seal();
    rows.insert_if_absent(MethodId(1), 'c');
    rows.insert_if_absent(MethodId(1), 'd');
    rows.insert_if_absent(MethodId(9), 'e');
    rows.insert_if_absent(MethodId(9), 'f');
    assert_eq!(rows.get(MethodId(3)), Some(&'a'));
    assert_eq!(rows.get(MethodId(1)), Some(&'c'));
    assert_eq!(rows.get(MethodId(9)), Some(&'e'));
    rows.insert(MethodId(3), 'g');
    rows.insert(MethodId(9), 'h');
    assert_eq!(rows.get(MethodId(3)), Some(&'g'));
    assert_eq!(rows.get(MethodId(9)), Some(&'h'));
    assert_eq!(rows.extent(), (4, 1));
}

/// Keyed pruning must distinguish identities with the same object value.
#[test]
fn entries_and_keyed_retention_preserve_identity_and_holes() {
    let mut rows = MethodRows::new();
    rows.insert(MethodId(3), 'a');
    rows.seal();
    rows.insert(MethodId(1), 'a');
    rows.insert(MethodId(9), 'a');
    rows.insert(MethodId(11), 'b');
    rows.retain_entries(|method, _| method != MethodId(1) && method != MethodId(9));
    let mut entries: Vec<_> = rows
        .entries()
        .map(|(method, row)| (method.0, *row))
        .collect();
    entries.sort_unstable();
    assert_eq!(entries, [(3, 'a'), (11, 'b')]);
    assert_eq!(rows.extent(), (4, 1));
    rows.retain(|row| *row != 'a');
    assert_eq!(
        rows.entries()
            .map(|(method, row)| (method.0, *row))
            .collect::<Vec<_>>(),
        [(11, 'b')]
    );
    rows.remove(MethodId(11));
    assert!(rows.is_empty());
    assert_eq!(rows.extent(), (4, 0));
}

/// Runtime identities must leave no growing dense prefix or live map rows.
#[test]
fn sealed_churn_and_independent_tables_do_not_accumulate_identities() {
    let mut rows = MethodRows::new();
    rows.insert(MethodId(3), 'a');
    rows.seal();
    let mut other = MethodRows::new();
    other.seal();
    other.insert_if_absent(MethodId(3), 'b');
    for id in 10..10_000 {
        rows.insert_if_absent(MethodId(id), 'c');
        rows.remove(MethodId(id));
        assert_eq!(rows.extent(), (4, 0));
    }
    rows.retain_entries(|_, _| false);
    assert!(rows.is_empty());
    assert_eq!(other.get(MethodId(3)), Some(&'b'));
    assert_eq!(other.extent(), (0, 1));
}

/// Both object-model entry paths seal the weak registry before user churn.
#[test]
fn executable_registry_stays_bounded_and_weak_across_programs() {
    fn run(interp: &mut crate::Interp, source: &[u8]) -> rexx_core::ObjRef {
        interp
            .run(rexx_parse::parse_program(source.to_vec()).expect("program parses"))
            .expect("program runs")
            .expect("program returns")
    }
    for library in [false, true] {
        let mut interp = crate::Interp::new();
        if library {
            interp.bootstrap_library().expect("library bootstrap");
        }
        run(&mut interp, b"return 0\n");
        let prefix = interp.executable_objects.extent().0;
        if library {
            assert!(
                prefix > 0,
                "bootstrap registrations did not enter the prefix"
            );
        }
        assert_eq!(interp.executable_objects.extent().1, 0);
        let capacity = interp.executable_objects.dense.capacity();
        eprintln!(
            "library={library} prefix={prefix} capacity={capacity} bytes={}",
            capacity * std::mem::size_of::<Option<rexx_core::ObjRef>>()
        );
        // Reflection retains the original Method through its class. Only
        // the copies below should disappear when their exit root is cleared.
        let original = run(&mut interp, b"return .object~method('STRING')\n");
        run(&mut interp, b"return 0\n");
        interp.collect_now();
        assert!(interp.heap.get(original).is_some());
        let retained = interp.executable_objects.extent().1;
        let mut issued = Vec::new();
        for _ in 0..4 {
            let object = run(&mut interp, b"return .object~method('STRING')~copy\n");
            let method = interp.executable_sources[&object]
                .installed
                .expect("copy identity");
            assert!(
                !issued.contains(&method),
                "later program reused an issued identity"
            );
            issued.push(method);
            assert_eq!(interp.executable_objects.get(method), Some(&object));
            run(&mut interp, b"return 0\n");
            interp.collect_now();
            assert!(
                interp.heap.get(object).is_none(),
                "weak index retained the copy"
            );
            assert!(interp.executable_objects.get(method).is_none());
            assert_eq!(interp.executable_objects.extent().0, prefix);
            assert_eq!(interp.executable_objects.dense.capacity(), capacity);
        }
        assert_eq!(
            interp.executable_objects.extent().1,
            retained,
            "runtime registrations accumulated beyond the class-owned Method"
        );
    }
}
