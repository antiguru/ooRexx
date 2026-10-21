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

//! A library routine's imported `Routine` object, asked for in a loop, under
//! the address-space cap the corpus gives the oracle. The corpus runs this
//! crate in-process and uncapped, so it cannot see a run that exceeds it.
//!
//! It loads the oracle checkout's `librxmath.so`. On a machine without that
//! checkout it fails naming what is missing: it calls
//! [`support::oracle::locate`], as `licensed_divergences.rs` and
//! `datetime_zone.rs` do, and checks the library the same way.

#![expect(
    clippy::disallowed_methods,
    reason = "this harness times or bounds real runs"
)]

mod support;

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use support::oracle::{ORACLE_MEMORY_LIMIT_KIB, locate};

/// `main.rex` holding `source`, run under the oracle's address-space cap from a
/// directory of its own named after `tag`, with the oracle's libraries on the
/// load path: the exit status, stdout and stderr.
fn run_capped(tag: &str, library: &str, source: &str) -> (Option<i32>, String, String) {
    let oracle = locate();
    let library = oracle.lib_dir().join(library);
    assert!(
        library.is_file(),
        "the oracle's {} is missing, which this test loads; without it the run \
         would fail for that reason and not for the one this test is about",
        library.display()
    );
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("{tag}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("a probe directory of this run's own");
    fs::write(dir.join("main.rex"), source).expect("the program is writable");
    let output = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "ulimit -v {ORACLE_MEMORY_LIMIT_KIB} && exec \"$0\" main.rex"
        ))
        .arg(env!("CARGO_BIN_EXE_rexx"))
        .current_dir(&dir)
        .env("LD_LIBRARY_PATH", oracle.lib_dir())
        .stdin(Stdio::null())
        .output()
        .expect("the capped run spawns");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// **A loop of `importedRoutines` sends runs to completion under the cap.**
/// Measured, 400,000 sends under `::requires 'rxmath' LIBRARY` and `ulimit -v
/// 1048576`: oracle rc 0; the crate, while each send built a new `Routine`
/// for every library routine the table holds, aborted at rc 134 with its
/// output lost.
#[test]
fn imported_routines_sent_in_a_loop_run_under_the_oracles_memory_cap() {
    assert_eq!(
        run_capped(
            "imported-routines",
            "librxmath.so",
            "p = .context~package\n\
             do i = 1 to 400000\n\
             \x20 t = p~importedRoutines\n\
             \x20 if i // 100000 = 0 then say i\n\
             end\n\
             say 'done'\n\
             ::requires 'rxmath' LIBRARY\n",
        ),
        (
            Some(0),
            "100000\n200000\n300000\n400000\ndone\n".to_owned(),
            String::new()
        )
    );
}

/// **A fresh `Routine` copy called by name from each INTERPRET runs under the
/// cap.** Each fragment's call site pins its copy and the fragment's end
/// unpins it. Measured, peak RSS at 200,000 passes: oracle 20 MB, this crate
/// 46 MB; with the fragment-end unpin disabled, 591 MB at 200,000.
#[test]
fn interpreted_calls_to_fresh_routine_copies_run_under_the_oracles_memory_cap() {
    assert_eq!(
        run_capped(
            "interpreted-copies",
            "liborxfunction.so",
            "do i = 1 to 400000\n\
             \x20 .routines['Y'] = .routines~i~copy\n\
             \x20 interpret 'call y 1'\n\
             end\n\
             say 'done' result\n\
             ::routine i external 'LIBRARY orxfunction TestIntArg'\n",
        ),
        (Some(0), "done 1\n".to_owned(), String::new())
    );
}

/// **A fresh raising `Routine` copy called by name at one site runs under the
/// cap.** The raise makes the site forget the copy, which unpins it. Measured,
/// peak RSS at 200,000 passes: oracle 21 MB, this crate 39 MB; with the
/// unpin skipped, 744 MB.
#[test]
fn a_raising_routine_copy_rebound_at_one_site_runs_under_the_oracles_memory_cap() {
    assert_eq!(
        run_capped(
            "raising-copies",
            "liborxfunction.so",
            "do i = 1 to 300000\n\
             \x20 .routines['Y'] = .routines~r~copy\n\
             \x20 call cy\n\
             end\n\
             say 'done'\n\
             exit 0\n\
             cy:\n\
             \x20 signal on syntax name s\n\
             \x20 call y 93900\n\
             s: return\n\
             ::routine r external 'LIBRARY orxfunction TestRaiseException0'\n",
        ),
        (Some(0), "done\n".to_owned(), String::new())
    );
}
