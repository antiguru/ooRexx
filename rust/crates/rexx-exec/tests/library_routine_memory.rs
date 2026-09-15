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

mod support;

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use support::oracle::{ORACLE_MEMORY_LIMIT_KIB, oracle_root};

/// **A loop of `importedRoutines` sends runs to completion under the cap.**
/// Measured, 400,000 sends under `::requires 'rxmath' LIBRARY` and `ulimit -v
/// 1048576`: oracle rc 0; the crate, while each send built a new `Routine`
/// for every library routine the table holds, aborted at rc 134 with its
/// output lost.
#[test]
fn imported_routines_sent_in_a_loop_run_under_the_oracles_memory_cap() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("imported-routines-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("a probe directory of this run's own");
    fs::write(
        dir.join("main.rex"),
        "p = .context~package\n\
         do i = 1 to 400000\n\
         \x20 t = p~importedRoutines\n\
         \x20 if i // 100000 = 0 then say i\n\
         end\n\
         say 'done'\n\
         ::requires 'rxmath' LIBRARY\n",
    )
    .expect("the program is writable");
    let output = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "ulimit -v {ORACLE_MEMORY_LIMIT_KIB} && exec \"$0\" main.rex"
        ))
        .arg(env!("CARGO_BIN_EXE_rexx-run"))
        .current_dir(&dir)
        .env("LD_LIBRARY_PATH", oracle_root().join("lib"))
        .stdin(Stdio::null())
        .output()
        .expect("the capped run spawns");
    assert_eq!(
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned()
        ),
        (
            Some(0),
            "100000\n200000\n300000\n400000\ndone\n".to_owned(),
            String::new()
        )
    );
}
