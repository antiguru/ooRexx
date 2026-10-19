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

//! Security-manager replacements the oracle answers and then crashes on, run
//! on this crate alone (`corpus/oracle-crashes.txt` entries 54 and 55). The
//! answers are rexxref's: the `STREAM` entry is the stream target, and the
//! `REQUIRES` entry `NAME` is the file loaded.

use rexx_exec::{Invocation, Outcome, run_program};

fn run(program: &str) -> Outcome {
    run_program(
        "security_replacements.rex",
        program.as_bytes().to_vec(),
        Invocation::none(),
    )
}

/// A `STREAM` replacement that is not a Stream, `.nil` included, is the
/// target of the builtin's message: 97.1, and the file is never written.
#[test]
fn a_stream_replacement_that_is_not_a_stream_is_the_target() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("security_replacements");
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    for (label, replacement) in [("nil", ".nil"), ("object", ".object~new")] {
        let file = dir.join(format!("{label}.txt"));
        let _ = std::fs::remove_file(&file);
        let program = format!(
            "mgr = .Mgr~new\n\
             r = .Routine~new('', \"signal on syntax; call lineout arg(1), 'written'; \
             return 'wrote'; syntax: return condition('o')~code condition('o')~message\")\n\
             r~setSecurityManager(mgr)\n\
             say r~call('{}')\n\
             ::class Mgr\n\
             ::method unknown\n  \
             use arg name, args\n  \
             if name \\= 'STREAM' then return 0\n  \
             args[1]~stream = {replacement}\n  \
             return 1\n",
            file.display()
        );
        let outcome = run(&program);
        assert_eq!(
            outcome.exit_code,
            0,
            "{label}: {}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        let target = if label == "nil" {
            "The NIL object"
        } else {
            "an Object"
        };
        assert_eq!(
            String::from_utf8_lossy(&outcome.stdout),
            format!("97.1 Object \"{target}\" does not understand message \"LINEOUT\".\n"),
            "{label}"
        );
        assert!(!file.exists(), "{label}: the replaced file was written");
    }
}

/// A `NAME` replacement that is not a string is read as its string value:
/// 3.1 naming `an Object`, after both `REQUIRES` events.
#[test]
fn a_requires_name_that_is_not_a_string_names_its_string_value() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("security_replacements");
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let file = dir.join("required.cls");
    std::fs::write(&file, "say 'loaded'\n").expect("the file to require");
    let program = format!(
        "mgr = .Mgr~new\n\
         r = .Routine~new('', \"signal on syntax; return .Package~new('', '')~loadPackage(\
         '{}'); syntax: return condition('o')~code condition('o')~message\")\n\
         r~setSecurityManager(mgr)\n\
         say r~call\n\
         say mgr~count\n\
         ::class Mgr\n\
         ::method init\n  expose count\n  count = 0\n\
         ::method count\n  expose count\n  return count\n\
         ::method unknown\n  \
         expose count\n  \
         use arg name, args\n  \
         if name \\= 'REQUIRES' then return 0\n  \
         count += 1\n  \
         args[1]['NAME'] = .object~new\n  \
         return 1\n",
        file.display()
    );
    let outcome = run(&program);
    assert_eq!(
        outcome.exit_code,
        0,
        "{}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "3.1 Failure during initialization: File \"an Object\" is unreadable.\n2\n"
    );
}
