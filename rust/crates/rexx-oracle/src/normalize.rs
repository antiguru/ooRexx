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

use crate::Outcome;
use std::path::Path;

/// Removes the parts of an interpreter's output that legitimately differ
/// between two runs of the *same* interpreter: absolute paths and line
/// endings.
///
/// Anything this function strips is invisible to the differ, so strip as
/// little as possible. Every addition here is a class of divergence the
/// project can no longer detect.
pub fn normalize(raw: &Outcome, cwd: &Path) -> Outcome {
    Outcome {
        stdout: normalize_stream(&raw.stdout, cwd),
        stderr: normalize_stream(&raw.stderr, cwd),
        exit_code: raw.exit_code,
    }
}

fn normalize_stream(bytes: &[u8], cwd: &Path) -> Vec<u8> {
    let text = String::from_utf8_lossy(bytes);
    let folded = text.replace("\r\n", "\n");
    let cwd = cwd.to_string_lossy();
    folded.replace(cwd.as_ref(), "<CWD>").into_bytes()
}

/// The first place two outcomes disagree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Divergence {
    ExitCode { cpp: i32, rust: i32 },
    Stdout { cpp: String, rust: String },
    Stderr { cpp: String, rust: String },
}

pub fn diff(cpp: &Outcome, rust: &Outcome) -> Option<Divergence> {
    if cpp.exit_code != rust.exit_code {
        return Some(Divergence::ExitCode {
            cpp: cpp.exit_code,
            rust: rust.exit_code,
        });
    }
    if cpp.stdout != rust.stdout {
        return Some(Divergence::Stdout {
            cpp: String::from_utf8_lossy(&cpp.stdout).into_owned(),
            rust: String::from_utf8_lossy(&rust.stdout).into_owned(),
        });
    }
    if cpp.stderr != rust.stderr {
        return Some(Divergence::Stderr {
            cpp: String::from_utf8_lossy(&cpp.stderr).into_owned(),
            rust: String::from_utf8_lossy(&rust.stderr).into_owned(),
        });
    }
    None
}
