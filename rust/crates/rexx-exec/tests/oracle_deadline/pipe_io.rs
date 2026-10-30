use super::*;
use std::process::{Command, Stdio};

const INPUT_LEN: usize = 2 * 1024 * 1024;
const OUTPUT_LEN: usize = 256 * 1024;

fn program(dir: &std::path::Path, source: &[u8]) -> PathBuf {
    let path = dir.join("pipe.rex");
    fs::write(&path, source).expect("write pipe witness");
    path.canonicalize().expect("absolute witness")
}

fn output_before_input(dir: &std::path::Path) -> PathBuf {
    program(dir, b"call charout , copies('O', 262144)\n.error~charout(copies('E', 262144))\nx = charin(, , 2097152)\nif length(x) \\= 2097152 then exit 99\nexit 23\n")
}

fn assert_channels(outcome: &support::oracle::CppOutcome) {
    assert_eq!(outcome.termination, Termination::Exited(23));
    assert_eq!(outcome.stdout, vec![b'O'; OUTPUT_LEN]);
    assert_eq!(outcome.stderr, vec![b'E'; OUTPUT_LEN]);
}

#[test]
fn oversized_stdin_to_a_never_reader_obeys_run_within_env_deadline() {
    if !gate_mode() { return; }
    let dir = fresh_run_dir();
    let file = program(&dir, b"do forever\nend\n");
    let input = vec![b'I'; INPUT_LEN];
    let start = Instant::now();
    let outcome = support::oracle::locate().run_within_env(&file, &[], Some(&input), Duration::from_millis(500), &[]);
    assert_eq!(outcome.termination, Termination::TimedOut);
    assert!(did_not_finish(&outcome));
    assert!(start.elapsed() < Duration::from_secs(8));
    fs::remove_dir_all(dir).expect("remove witness");
}

#[test]
fn oversized_stdin_to_a_never_reader_obeys_run_in_with_deadline() {
    if !gate_mode() { return; }
    let dir = fresh_run_dir();
    let file = program(&dir, b"do forever\nend\n");
    let input = vec![b'I'; INPUT_LEN];
    let start = Instant::now();
    let outcome = support::oracle::locate().run_in_with(&file, &dir, &[], Some(&input));
    assert_eq!(outcome.termination, Termination::TimedOut);
    assert!(did_not_finish(&outcome));
    assert!(start.elapsed() < ORACLE_DEADLINE * 3);
    fs::remove_dir_all(dir).expect("remove witness");
}

#[test]
fn output_before_input_completes_through_run_within_env_and_wrapper() {
    if !gate_mode() { return; }
    let dir = fresh_run_dir();
    let file = output_before_input(&dir);
    let input = vec![b'I'; INPUT_LEN];
    let oracle = support::oracle::locate();
    assert_channels(&oracle.run_within_env(&file, &[], Some(&input), Duration::from_secs(10), &[]));
    assert_channels(&oracle.run_with(&file, &[], Some(&input)));
    fs::remove_dir_all(dir).expect("remove witness");
}

#[test]
fn output_before_input_completes_through_run_in_with() {
    if !gate_mode() { return; }
    let dir = fresh_run_dir();
    let file = output_before_input(&dir);
    let input = vec![b'I'; INPUT_LEN];
    assert_channels(&support::oracle::locate().run_in_with(&file, &dir, &[], Some(&input)));
    fs::remove_dir_all(dir).expect("remove witness");
}

#[test]
fn output_before_input_completes_through_write_and_wait() {
    let dir = fresh_run_dir();
    let file = dir.join("pipe.py");
    fs::write(&file, b"import os, sys\nos.write(1, b'O' * 262144)\nos.write(2, b'E' * 262144)\ndata = sys.stdin.buffer.read()\nsys.exit(23 if data == b'I' * 2097152 else 99)\n").expect("write finite child");
    let output = support::oracle::write_and_wait(Command::new("python3").arg(&file).stdin(Stdio::piped()), &vec![b'I'; INPUT_LEN], &file);
    assert_eq!(output.status.code(), Some(23));
    assert_eq!(output.stdout, vec![b'O'; OUTPUT_LEN]);
    assert_eq!(output.stderr, vec![b'E'; OUTPUT_LEN]);
    fs::remove_dir_all(dir).expect("remove witness");
}

#[test]
fn early_exit_with_oversized_stdin_preserves_exit_status() {
    if !gate_mode() { return; }
    let dir = fresh_run_dir();
    let file = program(&dir, b"exit 17\n");
    let input = vec![b'I'; INPUT_LEN];
    let oracle = support::oracle::locate();
    for outcome in [oracle.run_with(&file, &[], Some(&input)), oracle.run_in_with(&file, &dir, &[], Some(&input))] {
        assert_eq!(outcome.termination, Termination::Exited(17));
        assert!(outcome.stdout.is_empty());
        assert!(outcome.stderr.is_empty());
    }
    let output = support::oracle::write_and_wait(Command::new("python3").args(["-c", "raise SystemExit(17)"]).stdin(Stdio::piped()), &input, &file);
    assert_eq!(output.status.code(), Some(17));
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    fs::remove_dir_all(dir).expect("remove witness");
}
