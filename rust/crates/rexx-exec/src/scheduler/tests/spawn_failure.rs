use super::pool::{SHAPE, Shape, run_shaped};
use crate::test_process::isolated;

#[test]
fn native_spawn_failure_reaches_main_and_started_handlers() {
    if isolated(
        "scheduler::tests::spawn_failure::native_spawn_failure_reaches_main_and_started_handlers",
    ) {
        return;
    }
    let source = "m = .t~new~start('other')\ncall probe\nsay m~result\nexit\n\
        probe: procedure\nsignal on syntax name caught\ncall HERE\nsay 'BODY RAN'\nreturn\n\
        caught: say condition('o')~code condition('o')~message\nreturn\n\
        ::requires 'pooltest' LIBRARY\n::class t\n::method other\n\
        signal on syntax name caught\ncall HERE\nreturn 'BODY RAN'\n\
        caught: return condition('o')~code condition('o')~message\n";
    let ran = run_shaped(
        source,
        Shape {
            stack: Some(1 << 46),
            ..SHAPE
        },
    )
    .expect("run");
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(
        ran.stdout(),
        "48.1 Failure in system service: ERROR CREATING THREAD.\n48.1 Failure in system service: ERROR CREATING THREAD.\n"
    );
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
    let ran = run_shaped(
        &source,
        Shape {
            stack: Some(1 << 46),
            ..SHAPE
        },
    )
    .expect("run");
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "48.1\n48.1\nprogress\n");
    assert!(
        !marker.exists(),
        "no child may start before reservation succeeds"
    );
    std::fs::remove_dir(dir).expect("remove empty marker directory");
}
