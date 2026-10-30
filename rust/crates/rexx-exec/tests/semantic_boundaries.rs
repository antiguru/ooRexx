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
        if path.file_stem().unwrap() == "raise-untraced" {
            assert_eq!(
                source,
                std::fs::read("../../corpus/lang/raise_untraced_renders_nothing.rex").unwrap()
            );
        }
        let stressed = rexx_exec::run_program_collect_every_alloc(
            "probe.rex",
            source.clone(),
            Invocation::none().with_deadline(std::time::Duration::from_secs(10)),
        );
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
        assert_eq!(stressed.exit_code, result.exit_code, "{}", path.display());
        assert_eq!(stressed.stdout, result.stdout, "{}", path.display());
        assert_eq!(stressed.stderr, result.stderr, "{}", path.display());
        assert!(stressed.collections > 0, "{}", path.display());
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

#[test]
fn owned_stdout_queries_observe_the_redirected_file() {
    use std::io::{Read, Seek};
    use std::process::{Command, Stdio};
    let directory = std::env::temp_dir().join(format!("rexx-stdout-query-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let program = directory.join("probe.rex");
    std::fs::write(&program, b"call lineout 'STDERR', .stdout~query('size') .stdout~query('position') .stdout~query('position write char') .stdout~query('position sys')\ncall charout , 'abc'\ncall lineout 'STDERR', 'logical' .stdout~query('position write char')\ncall lineout 'STDERR', 'sys' .stdout~query('position sys') 'size' .stdout~query('size')\naddress system 'true'\ncall lineout 'STDERR', 'handover' .stdout~query('position write char')\n").unwrap();
    let mut output = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(directory.join("stdout"))
        .unwrap();
    let child = Command::new("memcap")
        .args(["2G", "timeout", "20"])
        .arg(env!("CARGO_BIN_EXE_rexx"))
        .arg(&program)
        .current_dir(&directory)
        .stdin(Stdio::null())
        .stdout(output.try_clone().unwrap())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert!(child.status.success(), "{:?}", child.stderr);
    assert_eq!(
        child.stderr,
        b"0 1 1 0\nlogical 4\nsys 3 size 3\nhandover 4\n"
    );
    assert_eq!(output.metadata().unwrap().len(), 3);
    assert_eq!(output.stream_position().unwrap(), 3);
    output.rewind().unwrap();
    let mut bytes = Vec::new();
    output.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"abc");
    std::fs::remove_dir_all(directory).unwrap();
}
