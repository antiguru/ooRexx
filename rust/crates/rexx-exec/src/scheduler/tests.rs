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

/// A wait under a Rust frame that pins the activity runs the activity it
/// waits on from that frame, and goes on with its answer.
#[test]
fn a_pinned_wait_runs_the_activity_it_waits_on() {
    let outcome = run("m = .message~new('abc', 'length')\n\
         t = .k~new~start('SEND', m)\n\
         a = .array~of(2, 1)\na~sortWith(.c~new(m))\nsay 'sorted' a~toString('L', ',')\n\
         ::class k\n::method send\n  use arg m\n  m~send\n\
         ::class c\n::method init\n  expose m\n  use arg m\n\
         ::method compare\n  expose m\n  use arg l, r\n  say m~result\n  return l - r\n");
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    assert_eq!(stdout(&outcome), "3\nsorted 1,2\n");
}

/// A pinned wait under each kind of pinning frame a program can open runs
/// the started activity that ends it.
#[test]
fn a_pinned_wait_completes_under_each_pinning_frame() {
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
            "do i = 1 to 2 while f()\nend\nsay i\nexit\nf: return .m~result = 3\n",
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
        if outcome.exit_code != 0 || stdout(&outcome) != "3\n" {
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

/// The waits of the inversion witnesses: main parks unpinned on `m0`; `s1`
/// waits pinned on `m1` and starts `s3`, which sends `m0` (main is ready, set
/// aside by `s1`'s loop) and starts `s2`; `s2` waits pinned on `m2`, which
/// main sends once it runs. The oracle completes.
const HIDDEN_INVERSION: &str = "c = .w~new\n\
    m0 = .message~new(c, 'val', 'I', 0)\nm1 = .message~new(c, 'val', 'I', 1)\n\
    m2 = .message~new(c, 'val', 'I', 2)\nc~start('s1', m0, m1, m2)\n\
    say 'main got' m0~result\nm2~send\nsay 'main sent m2'\nsay 'done'\n\
    ::class w\n::method val unguarded; use arg x; return x*10\n\
    ::method s1 unguarded\n  use arg m0, m1, m2\n  .w~new~start('s3', m0, m1, m2)\n\
    \x20 interpret 'say \"S1 got\" m1~result'\n\
    ::method s3 unguarded\n  use arg m0, m1, m2\n  m0~send\n  say 's3 sent m0'\n\
    \x20 .w~new~start('s2', m1, m2)\n\
    ::method s2 unguarded\n  use arg m1, m2\n  interpret 'say \"S2 got\" m2~result'\n  m1~send\n";

/// A pinned wait whose loop finds nothing to run while an activity is ready
/// below it on the stack is refused as inverted, loudly and with no owner.
/// The oracle completes this program.
#[test]
fn an_inverted_pinned_wait_is_refused() {
    let outcome = run(HIDDEN_INVERSION);
    assert_eq!(outcome.exit_code, 120);
    assert_eq!(stdout(&outcome), "s3 sent m0\n");
    assert_eq!(stderr(&outcome), INVERTED);
}

const INVERTED: &str = "rexx-exec: a pinned wait for a message's completion that only an \
                        activity pinned below it can end is not implemented\n";

/// The inversion where the refusing loop itself set the ready activity
/// aside: main waits pinned in a comparator on `m1`; a started activity sends
/// `m1` and waits pinned on `m2`, which main sends once its comparator
/// returns. The oracle completes this program.
#[test]
fn an_inversion_the_refusing_loop_set_aside_is_refused_as_inverted() {
    let outcome = run(
        "m1 = .message~new('abc', 'length')\nm2 = .message~new('de', 'length')\n\
         t = .s~new~start('RUN', m1, m2)\n\
         a = .array~of(2, 1)\na~sortWith(.c~new(m1))\nsay 'main sorted'\nm2~send\nsay t~result\n\
         ::class s\n::method run\n  use arg m1, m2\n  m1~send\n\
         \x20 b = .array~of(2, 1)\n  b~sortWith(.c~new(m2))\n  return 'done'\n\
         ::class c\n::method init\n  expose m\n  use arg m\n\
         ::method compare\n  expose m\n  use arg l, r\n  say m~result\n  return l - r\n",
    );
    assert_eq!(outcome.exit_code, 120);
    assert_eq!(stdout(&outcome), "");
    assert_eq!(stderr(&outcome), INVERTED);
}

/// The same stack state where the oracle runs main on to a second wait that
/// nothing ends (two more lines, then a block for ever): `s2` wakes main and
/// nothing sends `s1`'s message. Main is ready below the refusing loop, so the
/// refusal is the inverted one.
#[test]
fn a_deadlock_with_a_buried_activity_ready_is_refused_as_inverted() {
    let outcome = run(BURIED_DEADLOCK);
    assert_eq!(outcome.exit_code, 120);
    assert_eq!(stdout(&outcome), "");
    assert_eq!(stderr(&outcome), INVERTED);
}

/// The deadlock witness: main waits pinned on `m1`; `s1` starts `s2` and
/// waits pinned on `m2`, which nothing sends; `s2` sends `m1`.
const BURIED_DEADLOCK: &str = "c = .w~new\n\
    m1 = .message~new(c, 'val', 'I', 1)\nm2 = .message~new(c, 'val', 'I', 2)\n\
    m3 = .message~new(c, 'val', 'I', 3)\nc~start('s1', m1, m2)\n\
    interpret 'say \"main got\" m1~result'\nsay 'main waits m3'\nsay m3~result\nsay 'done'\n\
    ::class w\n::method val unguarded; use arg x; return x*10\n\
    ::method s1 unguarded\n  use arg m1, m2\n  .w~new~start('s2', m1)\n\
    \x20 interpret 'say \"S1 got\" m2~result'\n\
    ::method s2 unguarded\n  use arg m1\n  m1~send\n";

/// A pinned wait nothing left to run can end is refused as such.
#[test]
fn a_pinned_wait_nothing_can_end_is_refused() {
    let outcome = run(
        "m = .message~new('abc', 'length')\na = .array~of(2, 1)\na~sortWith(.c~new(m))\n\
         ::class c\n::method init\n  expose m\n  use arg m\n\
         ::method compare\n  expose m\n  use arg l, r\n  say m~result\n  return l - r\n",
    );
    assert_eq!(outcome.exit_code, 120);
    assert_eq!(
        stderr(&outcome),
        "rexx-exec: a wait that nothing left to run can end is not implemented\n"
    );
}

/// `~result` of a message nothing ever sends is refused, owner none, where
/// the oracle blocks for ever (`oracle-crashes.txt`'s unsent `Message~result`
/// entry); the readers before it answer.
#[test]
fn the_result_of_a_message_never_sent_is_refused() {
    let outcome = run("m = .message~new('abc', 'length')
                       say m~hasResult m~completed m~target m~messageName
                       say m~result
say 'not reached'
");
    assert_eq!(outcome.exit_code, 120);
    assert_eq!(stdout(&outcome), "0 0 abc LENGTH\n");
    assert_eq!(
        stderr(&outcome),
        "rexx-exec: a wait that nothing left to run can end is not implemented\n"
    );
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
            // The call an extension that kept the started activity's instance
            // makes once that activity has ended.
            let version = interp
                .activities
                .retired()
                .first()
                .map(rexx_api::ffi::ThreadContext::interpreter_version_through_instance);
            assert_eq!(version, Some(328_448));
            assert!(interp.terminate().is_empty());
            kept
        })
        .expect("the interpreter thread")
        .join()
        .expect("the run did not panic");
    assert_eq!(kept, 1, "the started activity's context was not kept");
}
