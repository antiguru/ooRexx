/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

//! Which code in this workspace is allowed to say `unsafe`, asserted.

use std::path::{Path, PathBuf};

/// The workspace root, which is two directories above this crate.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root is two directories above this crate")
}

/// Every `.rs` file under `crates/`, workspace-relative and sorted.
fn rust_files() -> Vec<PathBuf> {
    let root = workspace_root();
    let mut found = Vec::new();
    let mut stack = vec![root.join("crates")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("a readable directory under crates/") {
            let path = entry.expect("a readable directory entry").path();
            if path.is_dir() {
                // `target` directories hold generated sources that no policy
                // here is about, and a stray one would make this test depend
                // on whether somebody had built inside a crate.
                if path.file_name().is_some_and(|name| name == "target") {
                    continue;
                }
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let relative = path
                    .strip_prefix(&root)
                    .expect("every path came from under the root")
                    .to_owned();
                found.push(relative);
            }
        }
    }
    found.sort();
    found
}

/// `line` with any `//` comment cut off, so that prose about `unsafe` -- of
/// which `bytes.rs` has a good deal, including every `SAFETY:` note -- is not
/// read as code saying it.
fn code_of(line: &str) -> &str {
    match line.find("//") {
        Some(at) => &line[..at],
        None => line,
    }
}

fn files_where(predicate: impl Fn(&str) -> bool) -> Vec<String> {
    rust_files()
        .into_iter()
        .filter(|path| {
            let text = std::fs::read_to_string(workspace_root().join(path))
                .expect("a readable source file");
            text.lines().map(code_of).any(&predicate)
        })
        .map(|path| path.to_string_lossy().into_owned())
        .collect()
}

#[test]
fn only_the_granted_module_may_say_unsafe() {
    // **The needles are spelled with `concat!` so that this file is subject to
    // its own rule** rather than exempted from it. Written out, the source
    // would contain every string it searches for and would have to exclude
    // itself -- which is the shape `rust/CLAUDE.md` calls a claim falsifiable
    // by the act of committing it.
    let granted = files_where(|code| {
        code.contains(concat!("allow(", "unsafe_code)"))
            || code.contains(concat!("expect(", "unsafe_code)"))
    });
    assert_eq!(
        granted,
        vec!["crates/rexx-core/src/lib.rs".to_string()],
        "the set of `unsafe_code` opt-ins in this workspace has changed. Each \
         one is a decision of Moritz's, taken per site against the bar in \
         rust/CLAUDE.md; adding one here without that is what this test is for."
    );

    let uses = files_where(|code| {
        code.contains(concat!("unsafe", " {"))
            || code.contains(concat!("unsafe", " fn"))
            || code.contains(concat!("unsafe", " impl"))
            || code.contains(concat!("unsafe", " trait"))
    });
    assert_eq!(
        uses,
        vec!["crates/rexx-core/src/bytes.rs".to_string()],
        "an `unsafe` block appeared outside the one module granted permission \
         for it"
    );
}

/// The test above compares against a list, and a list is only evidence if the
/// scan behind it actually reads the tree.
#[test]
fn the_scan_reaches_the_whole_workspace() {
    let files = rust_files();
    assert!(
        files.len() > 50,
        "the scan found {} source files, which is too few to be the whole \
         workspace -- it is looking in the wrong place",
        files.len()
    );
    for expected in [
        "crates/rexx-core/src/bytes.rs",
        "crates/rexx-exec/src/run.rs",
        "crates/rexx-num/src/lib.rs",
        "crates/rexx-parse/src/lib.rs",
    ] {
        assert!(
            files.iter().any(|path| path.to_string_lossy() == expected),
            "the scan missed {expected}"
        );
    }
}
