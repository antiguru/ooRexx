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

//! Runs a Rexx program under an interpreter and captures its observable output.

mod normalize;
pub use normalize::{Divergence, diff, normalize};

use std::path::{Path, PathBuf};
use std::process::Command;

/// Everything a Rexx program can be observed to produce.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: i32,
}

/// An interpreter under test, plus the loader paths it needs.
#[derive(Debug, Clone)]
pub struct Interpreter {
    pub binary: PathBuf,
    pub library_paths: Vec<PathBuf>,
}

impl Interpreter {
    pub fn run(&self, program: &Path, args: &[String], cwd: &Path) -> std::io::Result<Outcome> {
        let mut cmd = Command::new(&self.binary);
        cmd.arg(program).args(args).current_dir(cwd);
        let joined = std::env::join_paths(&self.library_paths)
            .expect("library paths must not contain the path separator");
        for var in ["LD_LIBRARY_PATH", "DYLD_LIBRARY_PATH"] {
            cmd.env(var, &joined);
        }
        let out = cmd.output()?;
        Ok(Outcome {
            stdout: out.stdout,
            stderr: out.stderr,
            exit_code: out.status.code().unwrap_or(-1),
        })
    }
}
