use rexx_oracle::{Outcome, normalize};
use std::path::Path;

#[test]
fn absolute_program_paths_are_replaced_by_a_placeholder() {
    let raw = Outcome {
        stdout: b"Error 43 running /home/someone/work/prog.rex line 7\n".to_vec(),
        stderr: Vec::new(),
        exit_code: 43,
    };
    let got = normalize(&raw, Path::new("/home/someone/work"));
    assert_eq!(
        String::from_utf8(got.stdout).unwrap(),
        "Error 43 running <CWD>/prog.rex line 7\n"
    );
}

#[test]
fn crlf_is_folded_so_windows_and_unix_compare_equal() {
    let raw = Outcome { stdout: b"a\r\nb\r\n".to_vec(), stderr: Vec::new(), exit_code: 0 };
    let got = normalize(&raw, Path::new("/tmp"));
    assert_eq!(String::from_utf8(got.stdout).unwrap(), "a\nb\n");
}
