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

//! Runs every program in a corpus under two interpreters and reports the
//! first divergence in each.

use rexx_oracle::{Interpreter, diff, normalize};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let (mut cpp, mut rs, mut corpus) = (None, None, None);
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--cpp" => cpp = args.next().map(PathBuf::from),
            "--rs" => rs = args.next().map(PathBuf::from),
            "--corpus" => corpus = args.next().map(PathBuf::from),
            other => {
                eprintln!("unknown flag: {other}");
                return ExitCode::from(2);
            }
        }
    }
    let (Some(cpp), Some(rs), Some(corpus)) = (cpp, rs, corpus) else {
        eprintln!("usage: rexx-diff --cpp <bin> --rs <bin> --corpus <dir>");
        return ExitCode::from(2);
    };

    // Canonicalise before use. Each program runs with its own directory as
    // the child's cwd, and a relative binary path would then be resolved
    // against *that* directory rather than ours -- so `--cpp ../build/bin/rexx`
    // fails with a bare NotFound once the child cwd moves.
    let absolute = |p: PathBuf, what: &str| match std::fs::canonicalize(&p) {
        Ok(abs) => abs,
        Err(e) => {
            eprintln!("cannot resolve {what} {}: {e}", p.display());
            std::process::exit(2);
        }
    };
    let cpp = absolute(cpp, "--cpp");
    let rs = absolute(rs, "--rs");
    let corpus = absolute(corpus, "--corpus");

    let lib = |bin: &PathBuf| {
        bin.parent()
            .map(|d| vec![d.to_path_buf(), d.join("../lib")])
            .unwrap_or_default()
    };
    let reference = Interpreter {
        library_paths: lib(&cpp),
        binary: cpp,
    };
    let candidate = Interpreter {
        library_paths: lib(&rs),
        binary: rs,
    };

    let mut programs: Vec<PathBuf> = walk(&corpus);
    programs.sort();
    // A mistyped or empty corpus directory would otherwise report
    // "0 programs, 0 divergences" and exit 0 -- the phase's central
    // self-test passing by finding nothing.
    if programs.is_empty() {
        eprintln!("no .rex programs under {}", corpus.display());
        return ExitCode::from(2);
    }
    let mut divergences = 0usize;
    for program in &programs {
        let cwd = program.parent().expect("corpus entries have a parent");
        let a = reference
            .run(program, &[], cwd)
            .expect("reference interpreter runs");
        let b = candidate
            .run(program, &[], cwd)
            .expect("candidate interpreter runs");
        if let Some(d) = diff(&normalize(&a, cwd), &normalize(&b, cwd)) {
            divergences += 1;
            println!("DIVERGENCE {}\n{d:#?}\n", program.display());
        }
    }
    println!("{} programs, {divergences} divergences", programs.len());
    if divergences == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn walk(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        // Do not swallow this: an unreadable directory read as "empty" is how
        // a self-test reports success for work it never did.
        Err(e) => panic!("cannot read {}: {e}", dir.display()),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else if path.extension().is_some_and(|e| e == "rex") {
            out.push(path);
        }
    }
    out
}
