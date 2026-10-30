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

//! Observable required-string and trace boundaries, including Deviations 44, 72, 77 and 83.

use rexx_exec::Invocation;
use std::path::Path;

mod watchdog;

#[test]
fn conversion_and_trace_boundaries() {
    let directory = Path::new("tests/semantic_boundaries");
    let mut cases = 0;
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "rex") {
            continue;
        }
        let source = std::fs::read(&path).unwrap();
        let result = watchdog::run_bounded("probe.rex", source, Invocation::none());
        let status = std::fs::read_to_string(path.with_extension("status"))
            .map(|status| status.trim().parse::<i32>().unwrap())
            .unwrap_or(0);
        assert_eq!(
            result.exit_code,
            status,
            "{}: {:?}",
            path.display(),
            result.stderr
        );
        assert_eq!(
            result.stdout,
            std::fs::read(path.with_extension("stdout")).unwrap(),
            "{}",
            path.display()
        );
        if let Ok(stderr) = std::fs::read(path.with_extension("stderr")) {
            assert_eq!(result.stderr, stderr, "{}", path.display());
        } else if !path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .starts_with("trace-")
        {
            assert!(
                result.stderr.is_empty(),
                "{}: {:?}",
                path.display(),
                result.stderr
            );
        }
        cases += 1;
    }
    assert!(cases > 0);
}

#[test]
fn buffered_invocations_do_not_query_the_hosts_stdout() {
    let result = watchdog::run_bounded(
        "probe.rex",
        b"say '['.stdout~query('size')']['.stdout~query('position')']['.stdout~query('position sys')']'\n".to_vec(),
        Invocation::none().with_standard_transient([true, false, true]),
    );
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout, b"[][][]\n");
    assert!(result.stderr.is_empty());
}
