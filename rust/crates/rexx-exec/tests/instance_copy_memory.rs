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

//! Copies of a large `String` subclass instance, made in a loop and dropped,
//! under the address-space cap the corpus gives the oracle. The corpus runs
//! this crate in-process and uncapped, so it cannot see a run whose bytes the
//! collector is never told about.

#![expect(clippy::disallowed_methods, reason = "this harness bounds a real run")]

mod support;

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use support::oracle::ORACLE_MEMORY_LIMIT_KIB;

/// **3000 copies of a 1,000,000-byte instance run to completion under the
/// cap.** Measured: oracle rc 0, `done 1000000`, maxrss 28 MB; the crate,
/// while a copy's text was charged nowhere, was OOM-killed under `memcap 2G`.
#[test]
fn copies_of_a_large_string_subclass_instance_run_under_the_oracles_memory_cap() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("instance-copies-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("a probe directory of this run's own");
    fs::write(
        dir.join("main.rex"),
        "big = .k~new(copies('x', 1000000))\n\
         do i = 1 to 3000; y = big~copy; end\n\
         say 'done' y~length y~class~id\n\
         ::class k subclass string\n",
    )
    .expect("the program is writable");
    let output = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "ulimit -v {ORACLE_MEMORY_LIMIT_KIB} && exec \"$0\" main.rex"
        ))
        .arg(env!("CARGO_BIN_EXE_rexx-run"))
        .current_dir(&dir)
        .stdin(Stdio::null())
        .output()
        .expect("the capped run spawns");
    assert_eq!(
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned()
        ),
        (Some(0), "done 1000000 K\n".to_owned(), String::new())
    );
}
