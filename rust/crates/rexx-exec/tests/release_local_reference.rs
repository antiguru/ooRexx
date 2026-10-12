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

//! `ReleaseLocalReference` through extensions in `release_local_reference/`,
//! each compiled with `c++` against this worktree's `api/` into a directory
//! of this run's own.

#![cfg(target_os = "linux")]
#![expect(
    clippy::disallowed_methods,
    reason = "this harness bounds real runs and reads their rusage"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use rexx_exec::{Invocation, Outcome, run_program, run_program_collect_every_alloc};

/// The `memcap` limit each run is under.
const MEMORY: &str = "2G";

/// The address-space limit in KiB where `memcap` is not on `PATH`.
const FALLBACK_LIMIT_KIB: u64 = 4_194_304;

fn worktree() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// Builds the extension `release_local_reference/<name>.cpp` into `dir` as
/// `lib<name>.so`.
fn build_library(dir: &Path, name: &str) {
    let api = worktree().join("api");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/release_local_reference/{name}.cpp"));
    let built = Command::new("c++")
        .args(["-shared", "-fPIC", "-O1"])
        .arg(format!("-I{}", api.display()))
        .arg(format!("-I{}", api.join("platform/unix").display()))
        .arg(&source)
        .arg("-o")
        .arg(dir.join(format!("lib{name}.so")))
        .status()
        .expect("`c++` runs, which this test needs to build its extension");
    assert!(built.success(), "lib{name}.so did not build");
}

/// A fresh directory of this run's own, named for `name`.
fn fresh_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("{name}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("a directory of this run's own");
    dir
}

/// Whether `name` is an executable file on this process's `PATH`.
fn on_path(name: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|path| std::env::split_paths(&path).any(|dir| dir.join(name).is_file()))
}

/// Runs `RelLoop(passes)` and answers the interpreter's peak resident set in
/// KiB, read from its own rusage by `/usr/bin/time`, the innermost wrapper.
fn peak_kib(dir: &Path, passes: u32) -> u64 {
    let program = dir.join(format!("loop{passes}.rex"));
    fs::write(
        &program,
        format!("say RelLoop({passes})\n::requires 'relloop' LIBRARY\n"),
    )
    .expect("the program is writable");
    let rss = dir.join(format!("rss{passes}"));
    let mut command = if on_path("memcap") {
        let mut command = Command::new("memcap");
        command.arg(MEMORY);
        command
    } else {
        let mut command = Command::new("bash");
        command.arg("-c").arg(format!(
            "ulimit -v {FALLBACK_LIMIT_KIB} && exec \"$0\" \"$@\""
        ));
        command
    };
    let output = command
        .args([
            "timeout",
            "-k",
            "5",
            "120",
            "/usr/bin/time",
            "-f",
            "%M",
            "-o",
        ])
        .arg(&rss)
        .arg(env!("CARGO_BIN_EXE_rexx-run"))
        .arg(&program)
        .current_dir(dir)
        .env("LD_LIBRARY_PATH", dir)
        .stdin(Stdio::null())
        .output()
        .expect("the capped run spawns");
    assert_eq!(
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned()
        ),
        (Some(0), "done\n".to_owned(), String::new()),
        "RelLoop({passes})"
    );
    let rss = fs::read_to_string(&rss).expect("/usr/bin/time wrote the rusage");
    rss.trim()
        .parse()
        .unwrap_or_else(|_| panic!("a maxrss in KiB, not {rss:?}"))
}

/// **A released local reference is reclaimable inside the call that made
/// it.** The oracle's release drops the object from the activation's save
/// list (`NativeActivation::removeLocalReference`,
/// `interpreter/execution/NativeActivation.cpp:1219`), so ten times the passes
/// cost it no more memory: measured, 19980 KB at 200,000 and 20864 KB at
/// 2,000,000.
#[test]
fn a_release_loop_runs_in_memory_that_does_not_grow_with_its_passes() {
    let dir = fresh_dir("release-loop");
    build_library(&dir, "relloop");
    let short = peak_kib(&dir, 200_000);
    let long = peak_kib(&dir, 2_000_000);
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
    assert!(
        long <= 2 * short,
        "peak RSS {long} KiB at 2,000,000 passes against {short} KiB at 200,000"
    );
}

fn run_in(dir: &Path, program: &str, stress: bool) -> Outcome {
    let invocation = Invocation::none()
        .with_directory(dir.to_path_buf())
        .with_environment(vec![(
            b"LD_LIBRARY_PATH".to_vec(),
            dir.as_os_str().as_encoded_bytes().to_vec(),
        )])
        .with_deadline(Duration::from_secs(60));
    let path = dir.join("relsem.rex").to_string_lossy().into_owned();
    let text = program.as_bytes().to_vec();
    match stress {
        true => run_program_collect_every_alloc(&path, text, invocation),
        false => run_program(&path, text, invocation),
    }
}

/// **A release drops that handle alone, and a second release, a release
/// through a context kept from an earlier call and a read after release
/// answer as the oracle's do.** `relsem.cpp`'s routines, measured on the
/// oracle with `RelOne(1000000)`: this stdout, empty stderr, rc 0.
#[test]
fn releases_answer_as_the_oracles_do() {
    let dir = fresh_dir("release-semantics");
    build_library(&dir, "relsem");
    let program = |passes: u32| {
        format!(
            "say RelTwice()\n\
             say RelStash()\n\
             say RelStale()\n\
             say RelOne({passes})\n\
             say RelLength()\n\
             say 'end'\n\
             ::requires 'relsem' LIBRARY\n"
        )
    };
    let ours = run_in(&dir, &program(1_000_000), false);
    assert_eq!(
        (
            String::from_utf8_lossy(&ours.stdout).into_owned(),
            String::from_utf8_lossy(&ours.stderr).into_owned(),
            ours.exit_code
        ),
        (
            "released twice\nkept\nstale released\nheld among many\n18\nend\n".to_owned(),
            String::new(),
            0
        )
    );
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
}

/// **Under a collection at every allocation, a released string goes and the
/// one held beside it stays.** The answers that read no released object
/// after an allocation are the oracle's; `RelTwice`'s released string is
/// collected before its answer is read and is refused loudly, a use after
/// release being outside the API contract
/// (`docs/superpowers/plans/phase-4-exclusions.txt`).
#[test]
fn under_a_collection_at_every_allocation_only_the_released_string_goes() {
    let dir = fresh_dir("release-stress");
    build_library(&dir, "relsem");
    let ours = run_in(
        &dir,
        "say RelStash()\n\
         say RelStale()\n\
         say RelOne(50)\n\
         say RelLength()\n\
         say RelTwice()\n\
         say 'end'\n\
         ::requires 'relsem' LIBRARY\n",
        true,
    );
    assert_eq!(
        (
            String::from_utf8_lossy(&ours.stdout).into_owned(),
            String::from_utf8_lossy(&ours.stderr).into_owned(),
            ours.exit_code
        ),
        (
            "kept\nstale released\nheld among many\n18\n".to_owned(),
            "rexx-exec: the handle is no longer held by this activation is not implemented\n"
                .to_owned(),
            120
        )
    );
    fs::remove_dir_all(&dir).expect("the run's directory is removable");
}
