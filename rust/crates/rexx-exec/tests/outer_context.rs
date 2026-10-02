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
    build_library(&probes().join("outer9b.cpp"), &dir.join("libouter9b.so"));
}

/// Builds the extension `source` as `library` and checks it NEEDs no
/// interpreter.
fn build_library(source: &Path, library: &Path) {
    let api = worktree().join("api");
    let built = Command::new("g++")
        .args(["-shared", "-fPIC", "-O1", "-static-libstdc++"])
        .arg(format!("-I{}", api.display()))
        .arg(format!("-I{}", api.join("platform/unix").display()))
        .arg(source)
        .arg("-o")
        .arg(library)
        .status()
        .expect("g++ runs");
    assert!(built.success(), "the forge did not build");
    let dynamic = Command::new("readelf")
        .arg("-d")
        .arg(library)
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

/// The program that calls the `outer7.cpp` member `member` from a started
/// activity, through the call context main's `Outer` kept.
fn elsewhere_program(member: char) -> String {
    format!(
        "x = 'main'\nsay Outer(.c~new)\nsay 'end' x\n::requires 'outer7' LIBRARY\n\
         ::class c\n::method run\n  x = 'run'\n  m = self~start('other')\n  \
         say 'started' m~result\n  return 'ret'\n\
         ::method other unguarded\n  x = 'other'\n  return OM('{member}')\n"
    )
}

/// Each member of a call context main's `Outer` kept (`outer_context/outer7.cpp`'s
/// `OM`), called from a started activity. The oracle's non-blocking members
/// read the top frame of the context's activity, here main's `RUN`, and
/// those answer the same here, except `GetContextFuzz`, which is 256 on the
/// oracle because it reads that Rexx activation as a native one. Every other
/// member raises the oracle's 98.983 in the started activity's call, where
/// the oracle raises it against main's activity and then ends in SIGSEGV or
/// hangs, so this is not a differential.
#[test]
fn a_kept_call_context_used_by_another_activity_answers_or_raises() {
    if !gate_mode() {
        eprintln!("outer_context: skipped without {GATE_ENV}");
        return;
    }
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("outer-context-elsewhere-{}", std::process::id()));
    let (forge, run) = (base.join("forge"), base.join("run"));
    fs::create_dir_all(&forge).expect("the forge directory");
    fs::create_dir_all(&run).expect("the run directory");
    let own = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/outer_context");
    build_library(&own.join("outer7.cpp"), &forge.join("libouter7.so"));
    let library_path = forge.display().to_string();
    let answered = [('d', "9"), ('f', "0"), ('o', "0"), ('r', "a Method")];
    for member in "abnrdfockvsitSg".chars() {
        let program = run.join(format!("k_{member}.rex"));
        fs::write(&program, elsewhere_program(member)).expect("the program");
        for stress in [false, true] {
            let ours = run_ours(&program, &run, &library_path, stress);
            let (stdout, stderr) = (
                String::from_utf8_lossy(&ours.stdout),
                String::from_utf8_lossy(&ours.stderr),
            );
            let context = format!("{member}, collecting at every allocation: {stress}");
            match answered.iter().find(|(answering, _)| *answering == member) {
                Some((_, value)) => assert_eq!(
                    (stdout.as_ref(), stderr.as_ref(), ours.exit_code),
                    (format!("started {value}\nret\nend main\n").as_str(), "", 0),
                    "{context}"
                ),
                None => {
                    assert_eq!((stdout.as_ref(), ours.exit_code), ("", 158), "{context}");
                    assert!(
                        stderr.contains(
                            "Error 98.983:  Execution thread does not match API thread context."
                        ),
                        "{context}: {stderr}"
                    );
                }
            }
        }
    }
    fs::remove_dir_all(&base).expect("cannot remove the run directory");
}
