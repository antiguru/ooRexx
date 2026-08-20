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

//! This crate does not inherit `[workspace.lints]`, and this is what keeps
//! that from costing anything.
//!
//! Cargo refuses `lints.workspace = true` beside any local override, so a
//! crate that needs one line of its own has to spell the whole table out.
//! `rexx-core` needs `unsafe_code = "deny"` where the workspace says `forbid`
//! -- `bytes.rs` carries the one approved `unsafe` in this tree -- so its
//! `[lints.clippy]` is a copy of the workspace's.
//!
//! **A copy that nothing checks is a copy that drifts**, and every other
//! defence in this repository against that shape is an assertion rather than a
//! comment. This is that assertion: the two tables are compared line for line,
//! so adding a lint to the workspace and not here fails the suite instead of
//! quietly leaving this crate unlinted.

use std::path::Path;

/// The `[lints.clippy]` / `[workspace.lints.clippy]` body of a manifest, as
/// the `name = "level"` pairs it sets, in file order.
///
/// Parsed by hand rather than with a TOML crate: this file is the only reason
/// such a dependency would exist, the shape it reads is two flat tables of
/// string values, and a dependency added for one test is a dependency every
/// build of this crate then carries.
fn clippy_levels(manifest: &str, header: &str) -> Vec<(String, String)> {
    let mut inside = false;
    let mut levels = Vec::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            inside = line == header;
            continue;
        }
        if !inside || line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((name, level)) = line.split_once('=') else {
            continue;
        };
        levels.push((
            name.trim().to_string(),
            level.trim().trim_matches('"').to_string(),
        ));
    }
    levels
}

#[test]
fn the_lint_tables_agree() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace = std::fs::read_to_string(crate_dir.join("../../Cargo.toml"))
        .expect("the workspace manifest is two directories up from this crate");
    let own = std::fs::read_to_string(crate_dir.join("Cargo.toml")).expect("this crate's manifest");

    let theirs = clippy_levels(&workspace, "[workspace.lints.clippy]");
    let ours = clippy_levels(&own, "[lints.clippy]");

    assert!(
        !theirs.is_empty(),
        "no lints were read out of the workspace manifest, so this test is \
         comparing nothing -- the header it looks for has moved"
    );
    assert_eq!(
        ours, theirs,
        "rexx-core's [lints.clippy] has drifted from [workspace.lints.clippy]. \
         It is a copy on purpose (see this crate's Cargo.toml); bring it back \
         into step rather than deleting this test."
    );

    // The one line that is deliberately different, asserted in both directions
    // so that neither side can quietly become the other.
    let their_rust = clippy_levels(&workspace, "[workspace.lints.rust]");
    let our_rust = clippy_levels(&own, "[lints.rust]");
    assert_eq!(
        their_rust,
        vec![("unsafe_code".to_string(), "forbid".to_string())],
        "the workspace's unsafe_code line is what every other crate inherits"
    );
    assert_eq!(
        our_rust,
        vec![("unsafe_code".to_string(), "deny".to_string())],
        "this crate's exception is `deny`, which is how a granted exception is \
         written down (2026-07-27-rust-rewrite.md:23)"
    );
}
