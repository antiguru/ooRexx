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

//! What a corpus program runs with besides its own text: `<name>.d/`,
//! `<name>.env` and `<name>.stdin` beside it, laid into a run directory and an
//! invocation the same way for every harness that runs corpus programs.

#![allow(dead_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// The fixture directory, environment, working directory and standard input
/// one corpus program runs with.
#[derive(Clone, Default)]
pub struct Sidecar {
    /// Copied into the run directory before each side runs.
    pub fixtures: Option<PathBuf>,
    /// `NAME=VALUE` overrides, `{run}` in a value spelled as the run
    /// directory's absolute path.
    pub environment: Vec<(String, String)>,
    /// A subdirectory of the run directory to run from, from `CWD=`.
    pub cwd: Option<String>,
    /// `<name>.stdin`, handed to both sides as standard input.
    pub stdin: Option<Vec<u8>>,
}

/// One separately removable part of a [`Sidecar`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Half {
    Fixtures,
    /// One environment override, by name.
    Variable(String),
    Cwd,
    Stdin,
}

impl Sidecar {
    /// Whether anything beside the program asked for one.
    pub fn present(&self) -> bool {
        !self.halves().is_empty()
    }

    /// The parts this sidecar has.
    pub fn halves(&self) -> Vec<Half> {
        let mut found = Vec::new();
        if self.fixtures.is_some() {
            found.push(Half::Fixtures);
        }
        for (name, _) in &self.environment {
            found.push(Half::Variable(name.clone()));
        }
        if self.cwd.is_some() {
            found.push(Half::Cwd);
        }
        if self.stdin.is_some() {
            found.push(Half::Stdin);
        }
        found
    }

    /// This sidecar with `half` removed and everything else kept.
    pub fn without(&self, half: &Half) -> Sidecar {
        let mut rest = self.clone();
        match half {
            Half::Fixtures => rest.fixtures = None,
            Half::Variable(name) => rest.environment.retain(|(held, _)| held != name),
            Half::Cwd => rest.cwd = None,
            Half::Stdin => rest.stdin = None,
        }
        rest
    }
}

/// `<name>.d/`, `<name>.env` and `<name>.stdin` for one corpus entry, whose
/// path `rel_path` is relative to `corpus_dir`.
///
/// # Panics
/// If `rel_path` does not end in `.rex`, or the `.env` holds a line that is
/// not `NAME=VALUE`.
pub fn sidecar_for(corpus_dir: &Path, rel_path: &str) -> Sidecar {
    let stem = rel_path
        .strip_suffix(".rex")
        .unwrap_or_else(|| panic!("corpus entry {rel_path} does not end in .rex"));
    let fixtures = corpus_dir.join(format!("{stem}.d"));
    let mut sidecar = Sidecar {
        fixtures: fixtures.is_dir().then_some(fixtures),
        ..Sidecar::default()
    };
    sidecar.stdin = fs::read(corpus_dir.join(format!("{stem}.stdin"))).ok();
    let env_path = corpus_dir.join(format!("{stem}.env"));
    let Ok(text) = fs::read_to_string(&env_path) else {
        return sidecar;
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, value) = line
            .split_once('=')
            .unwrap_or_else(|| panic!("{}: {line:?} is not a `NAME=VALUE`", env_path.display()));
        if name == "CWD" {
            sidecar.cwd = Some(value.to_string());
        } else {
            sidecar
                .environment
                .push((name.to_string(), value.to_string()));
        }
    }
    sidecar
}

/// Empties `dir`, creating it when it is not there.
pub fn empty_run_directory(dir: &Path) {
    if dir.exists() {
        fs::remove_dir_all(dir).unwrap_or_else(|e| panic!("cannot empty {}: {e}", dir.display()));
    }
    fs::create_dir_all(dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
}

/// Copies `from`'s tree into `to`, which must exist.
fn copy_tree(from: &Path, to: &Path) {
    for entry in
        fs::read_dir(from).unwrap_or_else(|e| panic!("cannot read {}: {e}", from.display()))
    {
        let entry =
            entry.unwrap_or_else(|e| panic!("cannot read an entry of {}: {e}", from.display()));
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            fs::create_dir_all(&target)
                .unwrap_or_else(|e| panic!("cannot create {}: {e}", target.display()));
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap_or_else(|e| {
                panic!(
                    "cannot copy {} to {}: {e}",
                    entry.path().display(),
                    target.display()
                )
            });
        }
    }
}

/// Empties the run directory and lays the sidecar's fixtures back into it,
/// answering the directory the program is to run from.
pub fn prepare_run_directory(dir: &Path, sidecar: &Sidecar) -> PathBuf {
    empty_run_directory(dir);
    if let Some(fixtures) = &sidecar.fixtures {
        copy_tree(fixtures, dir);
    }
    match &sidecar.cwd {
        None => dir.to_path_buf(),
        Some(relative) => {
            let cwd = dir.join(relative);
            fs::create_dir_all(&cwd)
                .unwrap_or_else(|e| panic!("cannot create {}: {e}", cwd.display()));
            cwd
        }
    }
}

/// The sidecar's overrides with `{run}` resolved against the run directory and
/// `{oraclelib}` against the oracle's own library directory.
///
/// `{oraclelib}` is what a program loading one of the oracle's compiled
/// extensions needs: the process loader read `LD_LIBRARY_PATH` before this
/// harness existed, so the value has to reach the interpreter's own
/// environment instead, and it is the same directory `Oracle::wrapped`
/// already puts on the spawned side.
pub fn resolved_environment(sidecar: &Sidecar, dir: &Path) -> Vec<(String, String)> {
    let run = dir.to_string_lossy().into_owned();
    let oracle_lib = super::oracle::oracle_root().join("lib");
    let oracle_lib = oracle_lib.to_string_lossy().into_owned();
    sidecar
        .environment
        .iter()
        .map(|(name, value)| {
            (
                name.clone(),
                value
                    .replace("{run}", &run)
                    .replace("{oraclelib}", &oracle_lib),
            )
        })
        .collect()
}

/// The in-process interpreter's invocation for one run from `directory`:
/// `overrides` laid over the process's own environment, and `stdin` as its
/// standard input.
pub fn invocation(
    directory: &Path,
    overrides: &[(String, String)],
    stdin: Option<&[u8]>,
) -> rexx_exec::Invocation {
    let mut invocation = rexx_exec::Invocation::none().with_directory(directory.to_path_buf());
    if let Some(bytes) = stdin {
        invocation = invocation.with_input(rexx_exec::ProgramInput::Bytes(bytes.to_vec()));
    }
    if !overrides.is_empty() {
        // Laid over the inherited environment rather than replacing it,
        // because `Oracle::run_in` sets its overrides on a spawned process
        // that inherits everything else, and the two sides have to read the
        // same environment.
        let mut environment: Vec<(Vec<u8>, Vec<u8>)> = env::vars()
            .filter(|(name, _)| !overrides.iter().any(|(over, _)| over == name))
            .map(|(name, value)| (name.into_bytes(), value.into_bytes()))
            .collect();
        environment.extend(
            overrides
                .iter()
                .map(|(name, value)| (name.clone().into_bytes(), value.clone().into_bytes())),
        );
        invocation = invocation.with_environment(environment);
    }
    invocation
}
