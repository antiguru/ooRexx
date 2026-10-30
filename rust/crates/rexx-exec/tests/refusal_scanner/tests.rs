use super::*;

#[test]
fn separate_test_modules_and_their_descendants_are_not_production() {
    let root = std::env::temp_dir().join(format!("refusal-modules-{}", std::process::id()));
    std::fs::create_dir_all(root.join("nested/tests")).unwrap();
    let fixtures = [
        ("lib.rs", "mod production;\nmod nested;\n#[cfg(test)]\nmod separate;\n"),
        ("production.rs", "fn real() -> Loud { Loud::real() }\n"),
        ("nested.rs", "#[cfg(test)]\nmod tests;\n"),
        ("nested/tests.rs", "mod child;\nfn hidden() -> Loud { Loud::hidden() }\n"),
        ("nested/tests/child.rs", "fn hidden_child() -> Loud { Loud::hidden_child() }\n"),
        ("separate.rs", "fn hidden_separate() -> Loud { Loud::hidden_separate() }\n"),
    ];
    for (path, text) in fixtures {
        std::fs::write(root.join(path), text).unwrap();
    }
    let files: Vec<_> = sources_in(&root).into_iter().map(|p| p.strip_prefix(&root).unwrap().to_path_buf()).collect();
    assert_eq!(files, ["lib.rs", "nested.rs", "production.rs"].map(PathBuf::from));
    let lines = code_lines(&root.join("production.rs"));
    assert!(lines.iter().any(|(_, code)| code.contains("Loud::real()")));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn constructor_ownership_is_independent_of_production_filename() {
    let mut results = Vec::new();
    for filename in ["lib.rs", "error.rs", "refusal.rs"] {
        let root = repo_root().join("crates/rexx-exec/src");
        let files = BTreeMap::from([
            (root.join(filename), vec![
                (1, "impl Loud {".into()),
                (2, "    fn planted() -> Loud {".into()),
                (3, "        Loud {}".into()),
                (4, "    }".into()),
                (5, "}".into()),
            ]),
            (root.join("dispatch.rs"), vec![(1, "Loud::planted();".into())]),
            (root.join("ir.rs"), vec![(1, "let planted = 1;".into())]),
        ]);
        let definitions = definitions(&files);
        assert_eq!(definitions["planted"].0, "Loud");
        assert!(bodies(&files, &definitions)["planted"].contains("Loud {}"));
        results.push(surface(&files, &definitions).0["planted"].clone());
    }
    assert_eq!(results, ["send", "send", "send"]);
}

#[test]
fn inline_test_refusals_are_excluded() {
    let root = std::env::temp_dir().join(format!("refusal-inline-{}.rs", std::process::id()));
    std::fs::write(&root, "fn real() -> Loud { Loud::real() }\n#[cfg(test)]\nmod tests {\n    fn hidden() -> Loud { Loud::hidden() }\n}\n").unwrap();
    let lines = code_lines(&root);
    assert!(lines.iter().any(|(_, code)| code.contains("Loud::real()")));
    assert!(!lines.iter().any(|(_, code)| code.contains("Loud::hidden()")));
    std::fs::remove_file(root).unwrap();
}
