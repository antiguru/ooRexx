//! Scheduling-independent observations for the redirected-target witness.

mod support;

use rexx_exec::{SimConfig, SwitchMode, run_program, run_program_collect_every_alloc};
use std::path::Path;

const EXPECTED: &[u8] = b"O:a slept 1\nO:b slept 0\nO:c slept 0\nsleep 0 3\nO:a waited 1\nO:b waited 0\nO:c waited 0\nsemaphore 0 3\nholding\nreleased\nO:a released 1\nO:b released 1\nO:c released 1\nguard 0 3 released 1\n";

#[test]
fn redirected_targets_preserve_wait_order_and_all_output() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus/lang/command_redirected_target_waits_elsewhere.rex")
        .canonicalize()
        .unwrap();
    let source = std::fs::read(&path).unwrap();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("redirected-timing");
    std::fs::create_dir_all(&dir).unwrap();
    let invocation = || {
        support::sidecar::invocation(&dir, &[], None)
            .with_deadline(std::time::Duration::from_secs(30))
    };
    for (mode, stress) in [
        (None, false),
        (None, true),
        (Some("sim:1"), false),
        (Some("sim:2,uniform:0.2"), false),
        (Some("sim:3,pct:3,k=40"), true),
    ] {
        let mut invoke = invocation();
        if let Some(mode) = mode {
            invoke = invoke.with_switch_mode(SwitchMode::Sim(SimConfig::parse(mode).unwrap()));
        }
        let run = if stress {
            run_program_collect_every_alloc(path.to_str().unwrap(), source.clone(), invoke)
        } else {
            run_program(path.to_str().unwrap(), source.clone(), invoke)
        };
        eprintln!(
            "mode={mode:?} stress={stress} status={} stdout={} stderr={:?}",
            run.exit_code,
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            run.exit_code,
            0,
            "{mode:?} stress={stress}: {:?}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            run.stdout,
            EXPECTED,
            "{mode:?} stress={stress}: {}",
            String::from_utf8_lossy(&run.stdout)
        );
        assert!(
            run.stderr.is_empty(),
            "{:?}",
            String::from_utf8_lossy(&run.stderr)
        );
        if stress {
            assert!(run.collections > 0);
        }
    }
    let oracle = support::oracle::locate();
    for index in 0..8 {
        let oracle_dir = dir.join(format!("oracle-{index}"));
        std::fs::create_dir(&oracle_dir).unwrap();
        let run = oracle.run_in(&path, &oracle_dir, &[]);
        eprintln!(
            "oracle status={:?} stdout={} stderr={:?}",
            run.termination,
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(run.termination, support::oracle::Termination::Exited(0));
        assert_eq!(
            run.stdout,
            EXPECTED,
            "{}",
            String::from_utf8_lossy(&run.stdout)
        );
        assert!(
            run.stderr.is_empty(),
            "{:?}",
            String::from_utf8_lossy(&run.stderr)
        );
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn removing_semaphore_wait_or_guard_ownership_breaks_the_observation() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus/lang/command_redirected_target_waits_elsewhere.rex");
    let source = std::fs::read_to_string(&path).unwrap();
    for (name, from, to) in [
        ("semaphore", "    .sem~wait", "    nop"),
        ("guard", "::method hold\n", "::method hold unguarded\n"),
    ] {
        assert!(source.contains(from), "{name} mutation anchor");
        let text = source.replacen(from, to, 1);
        let invocation = rexx_exec::Invocation::none()
            .with_deadline(std::time::Duration::from_secs(30))
            .with_switch_mode(SwitchMode::Sim(SimConfig::parse("sim:1").unwrap()));
        let run = run_program(path.to_str().unwrap(), text.into_bytes(), invocation);
        assert_eq!(
            run.exit_code,
            0,
            "{name}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_ne!(run.stdout, EXPECTED, "{name} omission escaped the witness");
        eprintln!("{name} omission: {}", String::from_utf8_lossy(&run.stdout));
    }
}


#[test]
fn a_late_unrelated_notification_cannot_change_callback_completion() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus/lang/command_redirected_target_waits_elsewhere.rex");
    let witness = std::fs::read_to_string(path).unwrap();
    let sleeper = witness.split_once("::class Sleeper").unwrap().1
        .split_once("::class Waiter").unwrap().0;
    // Release the old notification only after its value was read. This
    // represents delayed notification without relying on either timer's speed.
    let sleeper = sleeper.replace("    slept = .sleepMarked", "    slept = .sleepMarked\n    .notificationRelease~post");
    let source = format!(".local~notificationRelease = .EventSemaphore~new\n\
        .local~notificationEntered = .EventSemaphore~new\n\
        .local~sleepMarked = 0\n\
        late = .SleepMark~new~start('mark')\n\
        .notificationEntered~wait\n\
        o = .Sleeper~new\n\
        address system 'echo a' with output using (o)\n\
        .notificationRelease~post\nlate~wait\nexit\n\
        ::class Sleeper{sleeper}\n\
        ::class SleepMark\n::method mark\n\
        .notificationEntered~post\n.notificationRelease~wait\n.local~sleepMarked = 1\n");
    let run = run_program("/tmp/late-notification.rex", source.into_bytes(),
        rexx_exec::Invocation::none().with_deadline(std::time::Duration::from_secs(20))
        .with_switch_mode(SwitchMode::Sim(SimConfig::parse("sim:1").unwrap())));
    assert_eq!(run.exit_code, 0, "{}", String::from_utf8_lossy(&run.stderr));
    assert_eq!(run.stdout, b"O:a slept 1\n");
    assert!(run.stderr.is_empty(), "{}", String::from_utf8_lossy(&run.stderr));
}


/// Duration belongs to the virtual timer control, while the command witness
/// checks callback continuation and delivery across the unrelated wait.
#[test]
fn the_callback_sleep_advances_virtual_time_before_completion() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus/lang/command_redirected_target_waits_elsewhere.rex");
    let witness = std::fs::read_to_string(path).unwrap();
    let sleeper = witness.split_once("::class Sleeper").unwrap().1
        .split_once("::class Waiter").unwrap().0;
    assert!(sleeper.contains("    call SysSleep 1"));
    for (class, advanced) in [(sleeper.to_string(), 1), (sleeper.replace("    call SysSleep 1", "    nop"), 0)] {
        let source = format!("o = .Sleeper~new\ncall time 'R'\no~lineout('a')\nsay 'advanced' (time('E') >= 1)\nexit\n::class Sleeper{class}");
        let run = run_program("/tmp/callback-sleep.rex", source.into_bytes(),
            rexx_exec::Invocation::none().with_deadline(std::time::Duration::from_secs(20))
                .with_switch_mode(SwitchMode::Sim(SimConfig::parse("sim:1").unwrap())));
        assert_eq!(run.exit_code, 0, "{}", String::from_utf8_lossy(&run.stderr));
        assert_eq!(run.stdout, format!("O:a slept 1\nadvanced {advanced}\n").as_bytes());
        assert!(run.stderr.is_empty(), "{}", String::from_utf8_lossy(&run.stderr));
        eprintln!("virtual sleep control: {}", String::from_utf8_lossy(&run.stdout));
    }
}
