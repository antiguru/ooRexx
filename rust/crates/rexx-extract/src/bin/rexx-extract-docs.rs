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

//! Derives the Phase 5 gate tables' row sets from `oodocs/` and from
//! `interpreter/parser/DirectiveParser.cpp`, and writes them to
//! `rust/corpus/docs/`.
//!
//! ```text
//! cargo run -p rexx-extract --bin rexx-extract-docs -- \
//!     --oodocs ../oodocs --interpreter ../interpreter --out corpus/docs
//! ```
//!
//! `--check` derives and compares without writing, which is what
//! `tests/extract_docs.rs` does in process; the flag is here so the same
//! comparison can be run by hand.

use rexx_extract::docs;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let (mut oodocs, mut interpreter, mut out) = (None, None, None);
    let mut check = false;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--oodocs" => oodocs = args.next().map(PathBuf::from),
            "--interpreter" => interpreter = args.next().map(PathBuf::from),
            "--out" => out = args.next().map(PathBuf::from),
            "--check" => check = true,
            other => {
                eprintln!("unknown flag: {other}");
                return ExitCode::from(2);
            }
        }
    }
    let (Some(oodocs), Some(interpreter), Some(out)) = (oodocs, interpreter, out) else {
        eprintln!(
            "usage: rexx-extract-docs --oodocs <dir> --interpreter <dir> --out <dir> [--check]"
        );
        return ExitCode::from(2);
    };

    let revision = match docs::svn_revision(&oodocs.join("rexxref")) {
        Ok(revision) => revision,
        Err(e) => {
            eprintln!(
                "cannot read the revision of {}: {e}",
                oodocs.join("rexxref").display()
            );
            return ExitCode::from(2);
        }
    };
    let files = match docs::derive(&oodocs, &interpreter, &revision) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };

    if !check && let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("cannot create {}: {e}", out.display());
        return ExitCode::from(2);
    }

    let mut differences = 0usize;
    for (name, set) in &files {
        let path = out.join(name);
        let rendered = set.render();
        if check {
            match std::fs::read_to_string(&path) {
                Ok(committed) if committed == rendered => {
                    println!("{name}: {} rows, unchanged", set.rows.len());
                }
                Ok(_) => {
                    differences += 1;
                    println!("{name}: DIFFERS from the committed file");
                }
                Err(e) => {
                    differences += 1;
                    println!("{name}: cannot read {}: {e}", path.display());
                }
            }
            continue;
        }
        if let Err(e) = std::fs::write(&path, &rendered) {
            eprintln!("cannot write {}: {e}", path.display());
            return ExitCode::from(2);
        }
        println!("{name}: {} rows", set.rows.len());
    }
    if differences > 0 {
        eprintln!("{differences} row sets differ from their committed copies");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}
