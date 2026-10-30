use super::pool::{SHAPE, Shape, run_shaped};
use crate::test_process::isolated;
use std::sync::atomic::{AtomicUsize, Ordering};

static BODIES: AtomicUsize = AtomicUsize::new(0);
extern "C-unwind" fn body_counter(
    _context: *mut rexx_api::layout::RexxCallContext_,
    arguments: *mut rexx_api::layout::ValueDescriptor,
) -> *mut u16 {
    static SIGNATURE: [u16; 2] = [
        rexx_api::values::code::REXX_OBJECT_PTR,
        rexx_api::values::ARGUMENT_TERMINATOR,
    ];
    if arguments.is_null() {
        return SIGNATURE.as_ptr().cast_mut();
    }
    BODIES.fetch_add(1, Ordering::SeqCst);
    std::ptr::null_mut()
}

fn counted_library() -> rexx_api::load::Library {
    rexx_api::load::routines_only(&[("HERE", body_counter)])
}

#[test]
fn native_spawn_failure_reaches_main_and_started_handlers() {
    if isolated(
        "scheduler::tests::spawn_failure::native_spawn_failure_reaches_main_and_started_handlers",
    ) {
        return;
    }
    let source = "m = .t~new~start('other')\ncall probe\ncall probe\nsay m~result\nexit\n\
        probe: procedure\nsignal on syntax name caught\ncall HERE\nsay 'BODY RAN'\nreturn\n\
        caught: say condition('o')~code condition('o')~message\nreturn\n\
        ::requires 'pooltest' LIBRARY\n::class t\n::method other\n\
        signal on syntax name caught\ncall HERE\nreturn 'BODY RAN'\n\
        caught: return condition('o')~code condition('o')~message\n";
    let ran = run_shaped(
        source,
        Shape {
            stack: Some(1 << 46),
            library: (b"pooltest", counted_library),
            ..SHAPE
        },
    )
    .expect("run");
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(
        ran.stdout(),
        "48.1 Failure in system service: ERROR CREATING THREAD.\n48.1 Failure in system service: ERROR CREATING THREAD.\n48.1 Failure in system service: ERROR CREATING THREAD.\n"
    );
    assert_eq!(BODIES.load(Ordering::SeqCst), 0);
    assert_eq!(ran.exits, 0);
    assert_eq!(ran.takes, 0);
    assert_eq!(ran.abandoned, 0);
}

#[test]
fn command_spawn_failure_creates_no_child_and_repeats() {
    if isolated(
        "scheduler::tests::spawn_failure::command_spawn_failure_creates_no_child_and_repeats",
    ) {
        return;
    }
    let dir = std::env::temp_dir().join(format!("rexx-spawn-failure-{}", std::process::id()));
    std::fs::create_dir(&dir).expect("fresh marker directory");
    let marker = dir.join("child");
    let source = format!(
        "call probe\ncall probe\nsay 'progress'\nexit\nprobe: procedure\nsignal on syntax name caught\naddress system 'echo child > {}'\nsay 'CHILD RAN'\nreturn\ncaught: say condition('o')~code\nreturn\n",
        marker.display()
    );
    let mut unrelated = std::process::Command::new("python3")
        .args([
            "-c",
            "import sys; print('ready',flush=True); sys.stdin.buffer.read(1)",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("unrelated child");
    let mut ready = String::new();
    std::io::BufRead::read_line(
        &mut std::io::BufReader::new(unrelated.stdout.take().expect("stdout")),
        &mut ready,
    )
    .expect("ready line");
    assert_eq!(ready, "ready\n");
    let ran = run_shaped(
        &source,
        Shape {
            stack: Some(1 << 46),
            ..SHAPE
        },
    )
    .expect("run");
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    let unrelated_alive = unrelated.try_wait().expect("unrelated status").is_none();
    std::io::Write::write_all(unrelated.stdin.as_mut().expect("stdin"), b"x")
        .expect("release unrelated child");
    assert!(unrelated.wait().expect("reap unrelated child").success());
    assert!(
        unrelated_alive,
        "reservation failure touched an unrelated child"
    );
    assert_eq!(ran.stdout(), "48.1\n48.1\nprogress\n");
    assert!(
        !marker.exists(),
        "no child may start before reservation succeeds"
    );
    std::fs::remove_dir(dir).expect("remove empty marker directory");
}

#[test]
fn guarded_native_spawn_failure_releases_receiver_for_progress() {
    if isolated(
        "scheduler::tests::spawn_failure::guarded_native_spawn_failure_releases_receiver_for_progress",
    ) {
        return;
    }
    let source = "o=.k~new\nm=o~start('other')\ncall probe o\nsay m~result\nexit\nprobe: procedure\nuse arg o\nsignal on syntax name caught\nx=o~native(5)\nsay 'BODY RAN'\nreturn\ncaught: say condition('o')~code o~plain\nreturn\n::class k\n::method native external 'LIBRARY orxmethod TestIntArg'\n::method plain\nreturn 'progress'\n::method other\nsignal on syntax name caught\nx=self~native(5)\nreturn 'BODY RAN'\ncaught: return condition('o')~code self~plain\n";
    let ran = run_shaped(
        source,
        Shape {
            stack: Some(1 << 46),
            ..SHAPE
        },
    )
    .expect("run");
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "48.1 progress\n48.1 progress\n");
    assert_eq!(ran.exits, 0);
    assert_eq!(ran.abandoned, 0);
}

#[test]
fn configured_inline_and_fixed_saturation_are_not_spawn_errors() {
    let disabled = crate::scheduler::Pool::new(1 << 46, 0);
    assert!(matches!(disabled.reserve(), Ok(None)));
    let pool = crate::scheduler::Pool::new(1 << 20, 1);
    pool.fix_at_bound();
    let worker = pool.reserve().expect("spawn").expect("worker");
    assert!(matches!(pool.reserve(), Ok(None)));
    let (sent, received) = std::sync::mpsc::channel();
    worker.run(Box::new(move || sent.send(()).expect("receiver")));
    received
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("body ran");
}
