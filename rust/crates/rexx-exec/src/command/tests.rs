use super::*;

/// `scan_cmd`'s own split, against the C++ loop's answers.
#[test]
fn path_splits_on_blanks_and_keeps_quoted_runs_whole() {
    let split = |text: &str| scan_command(text.as_bytes()).expect("under the argument cap");
    assert_eq!(
        split("echo one two"),
        [b"echo".to_vec(), b"one".to_vec(), b"two".to_vec()]
    );
    assert_eq!(split("a\tb"), [b"a".to_vec(), b"b".to_vec()]);
    assert_eq!(split("  a   b  "), [b"a".to_vec(), b"b".to_vec()]);
    assert_eq!(split("\"a b\" c"), [b"a b".to_vec(), b"c".to_vec()]);
    assert_eq!(split("\"a b\""), [b"a b".to_vec()]);
    // A quote that closes nothing keeps every byte after it.
    assert_eq!(split("\"a b"), [b"a b".to_vec()]);
    assert_eq!(split(""), Vec::<Vec<u8>>::new());
}

/// More arguments than the cap is a failed split, not a truncated one.
#[test]
fn a_command_past_the_argument_cap_does_not_split() {
    let text = vec![b'a'; MAX_COMMAND_ARGS * 2]
        .iter()
        .map(|byte| (*byte as char).to_string())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(scan_command(text.as_bytes()).is_none());
}

/// The guard that decides whether a command can be handled without a
/// shell.
#[test]
fn a_metacharacter_outside_quotes_needs_a_shell() {
    assert!(!shell_metacharacter(b"cd sub"));
    assert!(shell_metacharacter(b"cd sub ; true"));
    assert!(shell_metacharacter(b"echo a | cat"));
    // Quoted and escaped ones do not count.
    assert!(!shell_metacharacter(b"echo \"a ; b\""));
    assert!(!shell_metacharacter(b"echo a \\; b"));
}

/// Every registered environment name, and the shell each resolves to.
#[test]
fn the_registered_environments_resolve_to_their_shells() {
    let shell = |name: &[u8]| match handler_for(name) {
        Some(Handler::Shell(shell)) => shell,
        _ => panic!("{} named no shell", String::from_utf8_lossy(name)),
    };
    for name in [b"".as_slice(), b"COMMAND", b"SYSTEM", b"sh", b"SH"] {
        assert_eq!(shell(name), "sh");
    }
    assert_eq!(shell(b"BASH"), "bash");
    assert_eq!(shell(b"tcsh"), "tcsh");
    assert!(matches!(handler_for(b"path"), Some(Handler::Path)));
    assert!(handler_for(b"nosuchhandler").is_none());
}

/// `corpus/oracle-crashes.txt` entry 44: the oracle dies on `cd ~` with
/// `HOME` unset. Here the shell answers it, as the C++ hands over a bare
/// `cd` or `cd ~/x` with `HOME` unset, and the directory stays. The rc is
/// the shell's own and is not asserted.
#[test]
fn cd_home_with_home_unset_goes_to_the_shell() {
    let source = b"here = directory()\n\
        'unset HOME'\n\
        'cd ~'\n\
        say 'stayed' (directory() == here)\n";
    let outcome = crate::run_program("/t.rex", source.to_vec(), crate::Invocation::none());
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(String::from_utf8_lossy(&outcome.stdout), "stayed 1\n");
}

/// `unquote`'s byte rules, the C++ loop's answers.
#[test]
fn unquote_drops_quotes_and_escapes_one_byte() {
    assert_eq!(unquote(b"\"a\"b"), b"ab");
    assert_eq!(unquote(b"a\\ b"), b"a b");
    assert_eq!(unquote(b"a\\\"b"), b"a\"b");
    assert_eq!(unquote(b"a\\\\b"), b"a\\b");
    assert_eq!(unquote(b"ab\\"), b"ab");
}

#[test]
fn an_externally_reaped_child_maps_to_127_and_failure() {
    use rustix::process::{Pid, WaitOptions, waitpid};
    let running = std::process::Command::new("python3")
        .args(["-c", "raise SystemExit(5)"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("child");
    let pid = Pid::from_raw(i32::try_from(running.id()).unwrap()).unwrap();
    let (_, status) = waitpid(Some(pid), WaitOptions::empty())
        .expect("external reap")
        .expect("status");
    assert_eq!(status.exit_status(), Some(5));
    let waited = collect(running, None, None, [true, true], None);
    assert_eq!(waited.rc, 127);
    assert_eq!(
        CommandOutcome::of(waited.rc).status.condition(),
        Some("FAILURE")
    );
    assert!(waited.out.is_empty() && waited.err.is_empty());
}

#[test]
fn an_owned_child_keeps_its_status_and_error_mapping() {
    let running = std::process::Command::new("python3")
        .args(["-c", "raise SystemExit(5)"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("child");
    let waited = collect(running, None, None, [true, true], None);
    assert_eq!(waited.rc, 5);
    assert_eq!(
        CommandOutcome::of(waited.rc).status.condition(),
        Some("ERROR")
    );
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "waits for real pool workers to return"
)]
fn unused_command_worker_returns_after_drop_unwind_and_exec_failure() {
    let mut interp = Interp::new();
    interp.pool = crate::scheduler::Pool::new(1 << 20, 1);
    let reserve = || ReservedWorker::new(interp.pool.reserve().expect("spawn").expect("worker"));
    drop(reserve());
    let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _worker = reserve();
        panic!("unwind before dispatch");
    }));
    assert!(unwound.is_err());
    for _ in 0..3 {
        assert!(matches!(
            start(
                &interp,
                &Handler::Path,
                b"/no-such-command-t88e2",
                None,
                None
            ),
            Ok(Child::Ran(Waited {
                rc: UNKNOWN_COMMAND,
                ..
            }))
        ));
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while interp.pool.threads() > 1 && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    interp.pool.fix_at_bound();
    let worker = loop {
        if let Some(worker) = interp.pool.reserve().expect("reserve") {
            break worker;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "reserved worker never returned"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    assert_eq!(interp.pool.threads(), 1);
    worker.run(Box::new(|| {}));
}
