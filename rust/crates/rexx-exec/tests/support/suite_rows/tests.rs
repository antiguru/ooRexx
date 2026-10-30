use super::*;

fn fixture() -> (PathBuf, Context) {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("suite-row-classes-{}-{id}", std::process::id()));
    std::fs::create_dir_all(root.join("utilities")).unwrap();
    std::fs::write(root.join("utilities/rxapi.testGroup"), "::class helper\n::method TEST_HELPER\n::class actual subclass ooTestCase\n::method TEST_REAL\n::method ordinary\n::class inherited subclass actual\n::method TEST_INHERITED\n").unwrap();
    let context = Context {
        root: root.clone(),
        groups: BTreeSet::from(["utilities/rxapi".to_string()]),
        rxapi: BTreeSet::from([("utilities/rxapi".to_string(), "TEST_REAL".to_string())]),
        deviations: BTreeSet::new(),
        flaky: BTreeSet::new(),
        skipped_types: BTreeSet::new(),
        worktree: root.clone(),
    };
    (root, context)
}

#[test]
fn only_tests_of_test_classes_can_own_suite_rows() {
    let (root, context) = fixture();
    let mut row = Row {
        file: EXCLUSIONS.to_string(),
        line: 1,
        group: "utilities/rxapi".to_string(),
        test: "TEST_REAL".to_string(),
        owner: Owner::Phase10(Deferral::Rxapi),
        reason: "group".to_string(),
    };
    for name in ["TEST_REAL", "TEST_INHERITED"] {
        row.test = name.to_string();
        assert!(check_row(&row, &context).is_ok(), "{name}");
    }
    for name in ["TEST_HELPER", "ORDINARY", "TEST_MISSING"] {
        row.test = name.to_string();
        assert!(check_row(&row, &context).is_err(), "{name}");
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn owner_equality_covers_reaching_group_and_captured_refusal() {
    let (root, context) = fixture();
    let mut row = Row {
        file: EXCLUSIONS.to_string(),
        line: 1,
        group: "utilities/rxapi".to_string(),
        test: "TEST_REAL".to_string(),
        owner: Owner::Phase10(Deferral::Rxapi),
        reason: String::new(),
    };
    for reason in [
        "reaching_rxapi",
        "group",
        "refusal: rexx-exec: queue unavailable (Phase 10 Rxapi)",
    ] {
        row.reason = reason.to_string();
        row.owner = Owner::Phase10(Deferral::Rxapi);
        assert!(check_row(&row, &context).is_ok(), "{reason}");
        row.owner = Owner::Phase10(Deferral::Extension);
        assert!(check_row(&row, &context).is_err(), "{reason}");
    }
    std::fs::remove_dir_all(root).unwrap();
}
