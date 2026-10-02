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

//! The context-variable members reached through a call context kept from an
//! enclosing native call, against the oracle. No extension the oracle builds
//! keeps a context, so the forged `outer9b.cpp` in the surface plan's Task 9
//! probes is compiled against this worktree's `api/` into the target
//! directory; it NEEDs nothing, which is asserted before it is loaded.
//! `o9b.rex` reads, sets and drops bound simple, stem and compound variables
//! through the kept context and must match on all three descriptors, run
//! plainly and with a collection at every allocation, which collects while
//! the outer call's caller is swapped in. `o9c.rex` and this test's own
//! `outer_context/o9d.rex` reach variables the outer activation never bound,
//! which grows a frame below the top one. Gate-only, Linux only.

#![cfg(target_os = "linux")]

mod support;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use rexx_exec::{Invocation, Outcome, run_program, run_program_collect_every_alloc};

const GATE_ENV: &str = "REXX_CORPUS_GATE";

fn gate_mode() -> bool {
    env::var(GATE_ENV).is_ok_and(|value| !value.is_empty() && value != "0")
}

fn worktree() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn probes() -> PathBuf {
    worktree().join("docs/superpowers/records/2026-09-14-phase-8-surface/task-9-probes")
}

/// Builds `libouter9b.so` into `dir` and checks it NEEDs no interpreter.
fn build_forge(dir: &Path) {
    let api = worktree().join("api");
    let built = Command::new("g++")
        .args(["-shared", "-fPIC", "-O1", "-static-libstdc++"])
        .arg(format!("-I{}", api.display()))
        .arg(format!("-I{}", api.join("platform/unix").display()))
        .arg(probes().join("outer9b.cpp"))
        .arg("-o")
        .arg(dir.join("libouter9b.so"))
        .status()
        .expect("g++ runs");
    assert!(built.success(), "the forge did not build");
    let dynamic = Command::new("readelf")
        .arg("-d")
        .arg(dir.join("libouter9b.so"))
        .output()
        .expect("readelf runs");
    let dynamic = String::from_utf8_lossy(&dynamic.stdout);
    assert!(
        !dynamic.contains("librexx"),
        "the forge NEEDs an interpreter library:\n{dynamic}"
    );
}

fn run_ours(program: &Path, run: &Path, library_path: &str, stress: bool) -> Outcome {
    let text = fs::read(program).expect("the probe");
    let mut environment: Vec<(Vec<u8>, Vec<u8>)> = env::vars()
        .filter(|(name, _)| name != "LD_LIBRARY_PATH")
        .map(|(name, value)| (name.into_bytes(), value.into_bytes()))
        .collect();
    environment.push((
        b"LD_LIBRARY_PATH".to_vec(),
        library_path.as_bytes().to_vec(),
    ));
    let invocation = Invocation::none()
        .with_directory(run.to_path_buf())
        .with_environment(environment);
    let path = program.to_string_lossy().into_owned();
    if stress {
        run_program_collect_every_alloc(&path, text, invocation)
    } else {
        run_program(&path, text, invocation)
    }
}

/// The forge built into a fresh directory named for `name`: the base to
/// remove, the run directory and the library path.
fn forged(name: &str) -> (PathBuf, PathBuf, String) {
    let base =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}-{}", std::process::id()));
    let (forge, run) = (base.join("forge"), base.join("run"));
    fs::create_dir_all(&forge).expect("the forge directory");
    fs::create_dir_all(&run).expect("the run directory");
    build_forge(&forge);
    let oracle = support::oracle::locate();
    let library_path = format!("{}:{}", oracle.lib_dir().display(), forge.display());
    (base, run, library_path)
}

#[test]
fn a_kept_outer_context_reaches_its_callers_variables() {
    if !gate_mode() {
        eprintln!("outer_context: skipped without {GATE_ENV}");
        return;
    }
    let (base, run, library_path) = forged("outer-context");
    let oracle = support::oracle::locate();

    let own = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/outer_context");
    for program in [
        probes().join("o9b.rex"),
        probes().join("o9c.rex"),
        own.join("o9d.rex"),
    ] {
        let theirs = oracle.run_in(&program, &run, &[("LD_LIBRARY_PATH", &library_path)]);
        let status = theirs.expect_exit_code();
        for stress in [false, true] {
            let ours = run_ours(&program, &run, &library_path, stress);
            assert_eq!(
                (
                    String::from_utf8_lossy(&ours.stdout),
                    String::from_utf8_lossy(&ours.stderr),
                    ours.exit_code
                ),
                (
                    String::from_utf8_lossy(&theirs.stdout),
                    String::from_utf8_lossy(&theirs.stderr),
                    status
                ),
                "{}, collecting at every allocation: {stress}",
                program.display()
            );
            assert!(
                !stress || ours.collections > 0,
                "the stress mode did not collect"
            );
        }
    }
    fs::remove_dir_all(&base).expect("cannot remove the run directory");
}

/// `outer_context/o9e.rex`: a started activity reads a variable through the
/// call context main's `Outer` kept, and is answered nothing. The oracle
/// raises 98.983 against main's activity, after which the started activity
/// ends in 44.1 and the program hangs (3 runs of 3), so this is not a
/// differential.
#[test]
fn a_kept_outer_context_used_by_another_activity_answers_nothing() {
    if !gate_mode() {
        eprintln!("outer_context: skipped without {GATE_ENV}");
        return;
    }
    let (base, run, library_path) = forged("outer-context-elsewhere");
    let program = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/outer_context/o9e.rex");
    for stress in [false, true] {
        let ours = run_ours(&program, &run, &library_path, stress);
        assert_eq!(
            (
                String::from_utf8_lossy(&ours.stdout),
                String::from_utf8_lossy(&ours.stderr),
                ours.exit_code
            ),
            ("started null\nret\nend main\n".into(), "".into(), 0),
            "collecting at every allocation: {stress}"
        );
    }
    fs::remove_dir_all(&base).expect("cannot remove the run directory");
}
