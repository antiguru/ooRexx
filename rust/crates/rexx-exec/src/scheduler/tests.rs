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

use crate::{Interp, Invocation, Outcome, parse_program, run_program};

fn run(source: &str) -> Outcome {
    run_program(
        "/tmp/scheduler.rex",
        source.as_bytes().to_vec(),
        Invocation::none(),
    )
}

fn stdout(outcome: &Outcome) -> String {
    String::from_utf8_lossy(&outcome.stdout).into_owned()
}

fn stderr(outcome: &Outcome) -> String {
    String::from_utf8_lossy(&outcome.stderr).into_owned()
}

/// A started method nothing waits on runs once main has ended, so its line
/// comes after main's last one.
#[test]
fn a_started_method_nothing_waits_on_runs_after_mains_last_line() {
    let outcome = run(".k~new~start('HELLO')\nsay 'main done'\n\
                       ::class k\n::method hello\n  say 'started'\n");
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    assert_eq!(stdout(&outcome), "main done\nstarted\n");
}

/// The activities a message's send wakes run in the order they parked,
/// after the activity that completed it parks or ends.
#[test]
fn a_completed_message_wakes_its_waiters_in_the_order_they_parked() {
    let outcome = run("m = .message~new('abc', 'length')\n\
         a = .w~new~start('WAITON', m, 'a')\n\
         b = .w~new~start('WAITON', m, 'b')\n\
         c = .w~new~start('WAITON', m, 'c')\n\
         d = .w~new~start('SEND', m)\n\
         say 'main waits'\n\
         d~wait\n\
         say a~result b~result c~result\n\
         ::class w\n::method waiton\n  use arg m, name\n  say name 'waits'\n\
         \x20 r = m~result\n  say name 'woke'\n  return r\n\
         ::method send\n  use arg m\n  m~send\n  say 'sent'\n");
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    assert_eq!(
        stdout(&outcome),
        "main waits\na waits\nb waits\nc waits\nsent\na woke\nb woke\nc woke\n3 3 3\n"
    );
}

/// A wait under a Rust frame that pins the activity is refused, loudly and
/// with no owner, and nothing after it runs.
#[test]
fn a_wait_under_a_pinned_frame_is_refused() {
    let outcome = run("m = .message~new('abc', 'length')\n\
         t = .k~new~start('SEND', m)\n\
         a = .array~of(2, 1)\na~sortWith(.c~new(m))\nsay 'sorted'\n\
         ::class k\n::method send\n  use arg m\n  m~send\n\
         ::class c\n::method init\n  expose m\n  use arg m\n\
         ::method compare\n  expose m\n  use arg l, r\n  say m~result\n  return l - r\n");
    assert_eq!(outcome.exit_code, 120);
    assert_eq!(stdout(&outcome), "");
    assert_eq!(
        stderr(&outcome),
        "rexx-exec: a wait inside a frame that pins its activity is not implemented\n"
    );
}

/// A wait under each kind of pinning frame a program can open is refused as
/// pinned, through the pin depth every build counts; a started activity sends
/// the message, so the wait itself could end.
#[test]
fn a_wait_under_each_pinning_frame_is_refused_as_pinned() {
    const SETUP: &str = "m = .message~new('abc', 'length')\n.local~m = m\n\
                         t = .s~new~start('SEND', m)\n";
    const CLASSES: &str = "::class s\n::method send\n  use arg m\n  m~send\n\
                           ::class c\n::method unknown\n  return .m~result\n\
                           ::method '+'\n  return .m~result\n";
    let mut wrong = Vec::new();
    for (kind, program) in [
        ("Interpret", "interpret 'say .m~result'\n"),
        ("NestedLoop", "do label l\n  say .m~result\nend\n"),
        (
            "LoopHeader",
            "do i = 1 to 2 while f()\nend\nexit\nf: return .m~result\n",
        ),
        (
            "TrapHandler",
            "call on error name h\naddress system 'false'\nexit\nh:\n  say .m~result\n  return\n",
        ),
        ("Unknown", "say .c~new~foo\n"),
        ("Operator", "say .c~new + 1\n"),
        (
            "Program",
            "say f()\nexit\n::routine f\n  interpret 'return .m~result'\n",
        ),
    ] {
        let outcome = run(&format!("{SETUP}{program}{CLASSES}"));
        if outcome.exit_code != 120
            || stderr(&outcome)
                != "rexx-exec: a wait inside a frame that pins its activity is not implemented\n"
        {
            wrong.push(format!(
                "{kind}: rc {} {:?} {:?}",
                outcome.exit_code,
                stdout(&outcome),
                stderr(&outcome)
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// A started activity whose wait nothing can end is refused at the program's
/// end, where the oracle blocks for ever; main's output stands.
#[test]
fn a_started_wait_nothing_can_end_is_refused_at_the_programs_end() {
    let outcome = run(
        ".w~new~start('WAITON', .message~new('abc', 'length'))\nsay 'main done'\n\
         ::class w\n::method waiton\n  use arg m\n  say 'waits'\n  return m~result\n",
    );
    assert_eq!(outcome.exit_code, 120);
    assert_eq!(stdout(&outcome), "main done\nwaits\n");
    assert_eq!(
        stderr(&outcome),
        "rexx-exec: a wait that nothing left to run can end is not implemented\n"
    );
}

/// An ended activity's thread context stays with the interpreter, so an
/// address an extension took from it during a native call stays valid; a
/// native call on a started activity reads the instance through it.
#[test]
fn an_ended_activitys_thread_context_is_kept() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../build/lib")
        .canonicalize()
        .expect("the worktree's build/lib is three directories above this crate");
    let source = b"m = .k~new~start('V')\nsay m~result\n::class k\n\
                   ::method v external \"LIBRARY orxmethod TestInterpreterVersion\"\n"
        .to_vec();
    let kept = std::thread::Builder::new()
        .stack_size(crate::INTERPRETER_STACK_BYTES)
        .spawn(move || {
            let mut interp = Interp::new();
            interp.adopt_environment(vec![(
                b"LD_LIBRARY_PATH".to_vec(),
                directory.into_os_string().into_encoded_bytes(),
            )]);
            let program = parse_program(source).expect("the program parses");
            let ran = interp
                .bootstrap_library()
                .and_then(|()| interp.run(program))
                .and_then(|_| interp.run_started_activities());
            assert!(ran.is_ok(), "{}", String::from_utf8_lossy(&interp.trace));
            assert_eq!(String::from_utf8_lossy(&interp.out), "328448\n");
            assert_eq!(interp.activity.pin_depth, 0);
            let kept = interp.activities.retired().len();
            assert!(interp.terminate().is_empty());
            kept
        })
        .expect("the interpreter thread")
        .join()
        .expect("the run did not panic");
    assert_eq!(kept, 1, "the started activity's context was not kept");
}
