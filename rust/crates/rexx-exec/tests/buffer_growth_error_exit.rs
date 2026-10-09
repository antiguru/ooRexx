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

//! A `MutableBuffer` grown by a mutator that then fails is still counted as
//! holding what it grew to. The failure needs a reservation that succeeds for
//! the buffer and fails for the mutator's result, so the run is capped by
//! `ulimit -v`, which an in-process run cannot be.

#![expect(clippy::disallowed_methods, reason = "this harness bounds a real run")]

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

/// The address-space cap, in KiB: the buffer's 2 GB reservation fits under
/// it and the result's second 2 GB does not. Measured on a debug build, the
/// second fails alone at every cap from 3 000 000 to 4 500 000, both fail at
/// 2 000 000, and both succeed at 5 000 000.
const CAP_KIB: u32 = 3_500_000;

/// **`INSERT` raises 5.0 after the buffer grew, and the collections that
/// follow, with the buffer live and then dropped, find the bytes it holds
/// counted.** In a debug build a collection that disagrees panics; at the
/// commit before the growth was charged where it happens, this run panicked
/// with the survivors holding 2000010805 body bytes against a running figure
/// of 11060.
#[test]
fn a_buffer_grown_before_a_failed_insert_is_counted() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("buffer-growth-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("a probe directory of this run's own");
    fs::write(
        dir.join("main.rex"),
        "b = .mutableBuffer~new\n\
         signal on syntax\n\
         b~insert('z', 2000000000)\n\
         say 'inserted'\n\
         exit\n\
         syntax:\n\
         say condition('o')~code b~getBufferSize\n\
         do i = 1 to 200000; x = .object~new; end\n\
         drop b\n\
         do i = 1 to 200000; x = .object~new; end\n\
         say 'done'\n",
    )
    .expect("the program is writable");
    let output = Command::new("bash")
        .arg("-c")
        .arg(format!("ulimit -v {CAP_KIB} && exec \"$0\" main.rex"))
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
        (Some(0), "5.0 2000000001\ndone\n".to_owned(), String::new())
    );
}
