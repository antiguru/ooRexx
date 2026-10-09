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

use crate::{Interp, Invocation, Outcome, SwitchMode, parse_program, run_program};

/// A bound on every run here: a program end waits for an activity nothing
/// can wake.
const RUN_DEADLINE: std::time::Duration = std::time::Duration::from_secs(60);

fn run(source: &str) -> Outcome {
    run_program(
        "/tmp/scheduler.rex",
        source.as_bytes().to_vec(),
        Invocation::none().with_deadline(RUN_DEADLINE),
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

/// The waits of the inversion witnesses: main parks on `m0` at `$MAIN`; `s1`
/// waits pinned on `m1` and starts `s3`, which sends `m0` and starts `s2`;
/// `s2` waits pinned on `m2`, which main sends once it runs. The oracle
/// completes.
const HIDDEN_INVERSION: &str = "c = .w~new\n\
    m0 = .message~new(c, 'val', 'I', 0)\nm1 = .message~new(c, 'val', 'I', 1)\n\
    m2 = .message~new(c, 'val', 'I', 2)\nc~start('s1', m0, m1, m2)\n\
    $MAIN\nm2~send\nsay 'main sent m2'\nsay 'done'\n\
    ::class w\n::method val unguarded; use arg x; return x*10\n\
    ::method s1 unguarded\n  use arg m0, m1, m2\n  .w~new~start('s3', m0, m1, m2)\n\
    \x20 interpret 'say \"S1 got\" m1~result'\n\
    ::method s3 unguarded\n  use arg m0, m1, m2\n  m0~send\n  say 's3 sent m0'\n\
    \x20 .w~new~start('s2', m1, m2)\n\
    ::method s2 unguarded\n  use arg m1, m2\n  interpret 'say \"S2 got\" m2~result'\n  m1~send\n";

/// A pinned wait whose loop finds nothing to run while an activity is ready
/// below it on the stack is refused as inverted, loudly and with no owner:
/// main waits pinned inside `INTERPRET`, so `s1`'s loop sets it aside. The
/// oracle completes this program.
#[test]
fn an_inverted_pinned_wait_is_refused() {
    let outcome = run(&HIDDEN_INVERSION.replace("$MAIN", "interpret \"say 'main got' m0~result\""));
    assert_eq!(outcome.exit_code, 120);
    assert_eq!(stdout(&outcome), "s3 sent m0\n");
    assert_eq!(stderr(&outcome), INVERTED);
}

/// The same waits with main parked at its root driver, unpinned: its state
/// is all in its record, so `s2`'s loop runs it and every wait ends, as on
/// the oracle (ruling P30). The oracle's order of the last lines varies
/// between runs, so the lines are compared as a set.
#[test]
fn a_wait_main_can_end_from_its_root_is_satisfied() {
    let outcome = run(&HIDDEN_INVERSION.replace("$MAIN", "say 'main got' m0~result"));
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    let mut lines: Vec<String> = stdout(&outcome).lines().map(str::to_string).collect();
    lines.sort();
    assert_eq!(
        lines,
        [
            "S1 got 10",
            "S2 got 20",
            "done",
            "main got 0",
            "main sent m2",
            "s3 sent m0"
        ]
    );
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

/// A started activity whose wait nothing can end keeps the program's end
/// waiting, as the oracle's does, until the run's deadline; main's output
/// stands.
#[test]
fn a_started_wait_nothing_can_end_keeps_the_programs_end_waiting() {
    let began = std::time::Instant::now();
    let outcome = run_with(
        ".w~new~start('WAITON', .message~new('abc', 'length'))\nsay 'main done'\n\
         ::class w\n::method waiton\n  use arg m\n  say 'waits'\n  return m~result\n",
        Invocation::none().with_deadline(std::time::Duration::from_millis(300)),
    );
    assert!(began.elapsed() >= std::time::Duration::from_millis(300));
    assert_eq!(outcome.exit_code, crate::DEADLINE_EXIT);
    assert_eq!(stdout(&outcome), "main done\nwaits\n");
    assert_eq!(outcome.stderr, crate::DEADLINE_REPORT);
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

/// An activity that made no native call and ran no package hook made no
/// thread context, so its end retains nothing: a started send and a `REPLY`'s
/// continuation, both ended before termination.
#[test]
fn an_activity_without_a_native_call_retains_no_thread_context() {
    let source = b"m = .k~new~start('v')\nsay m~result\nsay .k~new~r\n\
                   ::class k\n::method v\n  return 'started'\n\
                   ::method r\n  reply 'replied'\n  say 'rest'\n"
        .to_vec();
    let kept = std::thread::Builder::new()
        .stack_size(crate::INTERPRETER_STACK_BYTES)
        .spawn(move || {
            let mut interp = Interp::new();
            let program = parse_program(source).expect("the program parses");
            let ran = interp
                .bootstrap_library()
                .and_then(|()| interp.run(program))
                .and_then(|_| interp.run_started_activities());
            assert!(ran.is_ok(), "{}", String::from_utf8_lossy(&interp.trace));
            assert_eq!(
                String::from_utf8_lossy(&interp.out),
                "started\nreplied\nrest\n"
            );
            assert!(interp.activities.idle.iter().all(Option::is_none));
            let kept = interp.activities.retired().len();
            assert!(interp.terminate().is_empty());
            kept
        })
        .expect("the interpreter thread")
        .join()
        .expect("the run did not panic");
    assert_eq!(kept, 0, "an ended activity's unmade context was kept");
}

fn run_with(source: &str, invocation: Invocation) -> Outcome {
    run_program("/tmp/scheduler.rex", source.as_bytes().to_vec(), invocation)
}

const INTERLEAVED: &str = "a = .t~new~start('run', 'a')\nb = .t~new~start('run', 'b')\n\
                           a~wait\nb~wait\nsay 'done'\n\
                           ::class t\n::method run\n  use arg tag\n  do i = 1 to 3\n    \
                           say tag i\n  end\n";

/// Under `EveryOpportunity` every clause boundary ends the slice, so two
/// started activities' lines alternate, the same on every run.
#[test]
fn started_activities_interleave_under_every_opportunity() {
    for _ in 0..3 {
        let outcome = run_with(
            INTERLEAVED,
            Invocation::none()
                .with_deadline(RUN_DEADLINE)
                .with_switch_mode(SwitchMode::EveryOpportunity),
        );
        assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
        assert_eq!(stdout(&outcome), "a 1\nb 1\na 2\nb 2\na 3\nb 3\ndone\n");
    }
    let outcome = run(INTERLEAVED);
    assert_eq!(stdout(&outcome), "a 1\na 2\na 3\nb 1\nb 2\nb 3\ndone\n");
}

/// A busy loop with no park point in it ends only because the timer ends its
/// slice and the activity that sets the flag runs; the loop's body is empty,
/// so the slice ends at the loop's own step.
#[test]
fn a_busy_loop_yields_to_the_activity_that_ends_it() {
    let outcome = run_with(
        "f = .flag~new\na = f~start('waitForFlag')\nf~start('setFlag')\nsay a~result\n\
         ::class flag\n::attribute done unguarded\n::method init\n  expose done\n  done = 0\n\
         ::method waitForFlag unguarded\n  do while \\self~done\n  end\n  return 'A ended'\n\
         ::method setFlag unguarded\n  self~done = 1\n",
        Invocation::none().with_deadline(std::time::Duration::from_secs(60)),
    );
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    assert_eq!(stdout(&outcome), "A ended\n");
}

const SORTED: &str = "b = .t~new~start('other')\narr = .array~of(3, 1, 2)\n\
                      arr~sortWith(.cmp~new)\nsay 'sorted' arr~makeString('L', ' ')\n\
                      ::class t\n::method other\n  say 'other ran'\n\
                      ::class cmp\n::method compare\n  use arg l, r\n  say 'cmp' l r\n  \
                      return l - r\n";

/// A slice that ends inside the comparator, where the sort pins the
/// activity, is deferred to the first clause after the sort: the other
/// activity's line follows the sort's, whichever comparator clause it fell
/// on. Clauses 4 to 12 are the comparator's.
#[test]
fn a_slice_inside_a_sort_comparator_waits_for_the_sort() {
    for clause in 4..=12 {
        let outcome = run_with(
            SORTED,
            Invocation::none()
                .with_deadline(RUN_DEADLINE)
                .with_switch_mode(SwitchMode::AtClause(clause)),
        );
        assert_eq!(
            stdout(&outcome),
            "cmp 1 3\ncmp 2 3\ncmp 2 1\nother ran\nsorted 1 2 3\n",
            "a slice at clause {clause}"
        );
    }
    let outcome = run_with(
        SORTED,
        Invocation::none()
            .with_deadline(RUN_DEADLINE)
            .with_switch_mode(SwitchMode::AtClause(3)),
    );
    assert_eq!(
        stdout(&outcome),
        "other ran\ncmp 1 3\ncmp 2 3\ncmp 2 1\nsorted 1 2 3\n"
    );
}

const HALT_START: &str = "m = .message~new(.t~new, 'delayValueReturn')\nsay m~halt\nm~start\nsay m~halt\n\
     m~wait\nsay m~hasResult m~hasError m~completed\nc = m~errorCondition\n\
     say c~code '['c~description']'\n\
     m = .message~new(.t~new, 'delayValueReturn')\nm~start\nsay m~halt('HALT Test')\n\
     m~wait\nsay m~hasResult m~hasError m~completed\nc = m~errorCondition\n\
     say c~code '['c~description']'\n\
     m = .message~new(.t~new, 'delayHaltReturn')\nm~start\nsay m~halt('HALT Test')\n\
     m~wait\nsay m~hasResult m~hasError m~completed\nc = m~result\n\
     say c~condition '['c~description']'\n\
     ::class t\n::method delayValueReturn\n  do i = 1 to 50\n  end\n  return arg(1)\n\
     ::method delayHaltReturn\n  signal on halt\n  do i = 1 to 50\n  end\n  x = 123\n\
     \x20 return .nil\n  halt:\n  return condition('o')\n";

/// Message.testGroup's `test_halt_start` (`:643`), its sleeps replaced by
/// loops so that the switch mode, not time, decides that the started
/// activity is running when the halt is asked.
#[test]
fn message_halt_in_the_shape_of_test_halt_start() {
    let outcome = run_with(
        HALT_START,
        Invocation::none()
            .with_deadline(RUN_DEADLINE)
            .with_switch_mode(SwitchMode::EveryOpportunity),
    );
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    assert_eq!(
        stdout(&outcome),
        "0\n1\n0 1 1\n4.1 []\n1\n0 1 1\n4.1 []\n1\n1 0 1\nHALT [HALT Test]\n"
    );
    assert_eq!(
        stderr(&outcome)
            .matches("Error 4.1:  Program interrupted with HALT condition.")
            .count(),
        2,
        "{}",
        stderr(&outcome)
    );
}

/// A second `Message~halt` while the started activity's `CALL ON HALT`
/// handler runs is dropped, as the oracle's delayed trap drops it (10 of 10
/// runs).
#[test]
fn a_message_halt_inside_its_handler_is_dropped() {
    let outcome = run_with(
        "d = .directory~new\nd~phase = 0\no = .w~new\nm = o~start('w', d)\n\
         do while d~phase == 0; end\nsay m~halt\ndo while d~phase == 1; end\nsay m~halt\n\
         d~phase = 3\nm~result\nsay 'main end'\nexit\n\
         ::class w\n::method w\n  use arg d\n  call on halt name h\n  d~phase = 1\n\
         \x20 do while d~phase < 4; end\n  say 'w end'\n  return\n\
         h:\n  say 'h in'\n  d~phase = 2\n  do while d~phase == 2; end\n  do i = 1 to 3; end\n\
         \x20 say 'h out'\n  d~phase = 4\n  return\n",
        Invocation::none().with_deadline(RUN_DEADLINE),
    );
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    assert_eq!(stdout(&outcome), "1\nh in\n1\nh out\nw end\nmain end\n");
}

/// A `Message~halt` of an activity buried below another's pinned region is
/// taken only once that region ends: `a` spins inside `INTERPRET`, `b` runs
/// in its pinned yields and spins there until main has halted `a`. The
/// oracle takes the halt at `a`'s next clause and prints `A halted` before
/// `B done` (30 of 30 runs).
#[test]
fn a_halt_of_an_activity_below_a_pinned_region_waits_for_the_region() {
    let outcome = run_with(
        "g = .gate~new\na = .t~new~start('a')\nb = .t~new~start('b', g)\n\
         do while \\g~started; end\na~halt\ng~sent = 1\nsay 'halt sent'\n\
         ::class gate\n::attribute started unguarded\n::attribute sent unguarded\n\
         ::method init\n  expose started sent\n  started = 0\n  sent = 0\n\
         ::class t\n::method a unguarded\n  signal on halt name h\n  \
         interpret 'do forever; end'\nh:\n  say 'A halted'\n\
         ::method b unguarded\n  use arg g\n  \
         interpret 'g~started = 1; do until g~sent; end; do i = 1 to 1000000; end'\n  \
         say 'B done'\n",
        Invocation::none()
            .with_deadline(RUN_DEADLINE)
            .with_switch_mode(SwitchMode::EveryOpportunity),
    );
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    assert_eq!(stdout(&outcome), "halt sent\nB done\nA halted\n");
}

/// The stress mode collects at every countdown visit and every switch as
/// well as at every allocation. Under either switch mode every clause visits
/// the countdown, so `AtClause` past the program's end differs from
/// `EveryOpportunity` by the switches alone; one activity never switches, so
/// there the switch mode adds the visits alone.
#[test]
fn the_stress_mode_collects_at_every_visit_and_switch() {
    let stressed = |source: &str, invocation: Invocation| {
        let outcome = crate::run_program_collect_every_alloc(
            "/tmp/scheduler.rex",
            source.as_bytes().to_vec(),
            invocation,
        );
        assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
        outcome
    };
    let switching = stressed(
        INTERLEAVED,
        Invocation::none()
            .with_deadline(RUN_DEADLINE)
            .with_switch_mode(SwitchMode::EveryOpportunity),
    );
    assert_eq!(stdout(&switching), "a 1\nb 1\na 2\nb 2\na 3\nb 3\ndone\n");
    let unswitched = stressed(
        INTERLEAVED,
        Invocation::none()
            .with_deadline(RUN_DEADLINE)
            .with_switch_mode(SwitchMode::AtClause(u64::MAX)),
    );
    assert!(
        switching.collections > unswitched.collections,
        "{} collections switching, {} visiting alone",
        switching.collections,
        unswitched.collections
    );
    let lone = "do i = 1 to 20\n  nop\nend\n";
    let visiting = stressed(
        lone,
        Invocation::none()
            .with_deadline(RUN_DEADLINE)
            .with_switch_mode(SwitchMode::EveryOpportunity),
    );
    let unvisited = stressed(lone, Invocation::none().with_deadline(RUN_DEADLINE));
    assert!(
        visiting.collections > unvisited.collections,
        "{} collections visiting every clause, {} not",
        visiting.collections,
        unvisited.collections
    );
}

/// A busy-wait inside `INTERPRET` and one inside a `CALL ON` handler, each
/// pinned, end once the activity that sets the flag runs: the second slice
/// that finds the waiter pinned runs the others on its stack.
#[test]
fn a_pinned_busy_wait_yields_to_the_activity_that_ends_it() {
    let waits = [
        (
            "interpret \"do while \\self~done; end\"\n  say 'interpreted ended'\n",
            "set\ninterpreted ended\nmain ended\n",
        ),
        (
            "call on error name h\n  'exit 1'\n  return\nh:\n  do while \\self~done\n  end\n  \
             say 'handler ended'\n  return\n",
            "set\nhandler ended\nmain ended\n",
        ),
    ];
    for (wait, expected) in waits {
        let source = format!(
            "f = .flag~new\na = f~start('waitForFlag')\nf~start('setFlag')\na~wait\n\
             say 'main ended'\n::class flag\n::attribute done unguarded\n::method init\n  \
             expose done\n  done = 0\n::method waitForFlag unguarded\n  {wait}\
             ::method setFlag unguarded\n  say 'set'\n  self~done = 1\n"
        );
        for invocation in [
            Invocation::none()
                .with_switch_mode(SwitchMode::EveryOpportunity)
                .with_deadline(std::time::Duration::from_secs(60)),
            Invocation::none().with_deadline(std::time::Duration::from_secs(60)),
        ] {
            let outcome = run_with(&source, invocation);
            assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
            assert_eq!(stdout(&outcome), expected);
        }
    }
}

const TWO_PINNED_WAITERS: &str = "g = .flag~new\na = g~start('wait', 'A')\nb = g~start('wait', 'B')\n\
    g~start('setFlag')\na~wait\nb~wait\nsay 'main ended'\n\
    ::class flag\n::attribute done unguarded\n::method init\n  expose done\n  \
    done = 0\n::method wait unguarded\n  use arg tag\n  \
    interpret \"do while \\self~done; end\"\n  say tag 'ended'\n\
    ::method setFlag unguarded\n  say 'set'\n  self~done = 1\n";

/// Two pinned busy-waiters on a flag a third activity sets: the second runs
/// on the first's stack and, pinned in turn, runs the setter on its own; both
/// end, the inner first, with no livelock between their yields.
#[test]
fn two_pinned_busy_waiters_both_end() {
    let source = TWO_PINNED_WAITERS;
    for invocation in [
        Invocation::none()
            .with_switch_mode(SwitchMode::EveryOpportunity)
            .with_deadline(std::time::Duration::from_secs(60)),
        Invocation::none().with_deadline(std::time::Duration::from_secs(60)),
    ] {
        let outcome = run_with(source, invocation);
        assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
        assert_eq!(stdout(&outcome), "set\nB ended\nA ended\nmain ended\n");
    }
}

/// The timer is disarmed at the switch that leaves no other activity ready,
/// not at the next countdown visit, which a run may never make.
#[test]
fn the_timer_disarms_when_no_other_activity_is_ready() {
    let source = b"m = .t~new~start('quick')\ndo i = 1 to 3000\nend\nsay m~result\n\
                   ::class t\n::method quick\n  return 'q'\n"
        .to_vec();
    let armed = std::thread::Builder::new()
        .stack_size(crate::INTERPRETER_STACK_BYTES)
        .spawn(move || {
            let mut interp = Interp::new();
            let program = parse_program(source).expect("the program parses");
            let ran = interp
                .bootstrap_library()
                .and_then(|()| interp.run(program))
                .and_then(|_| interp.run_started_activities());
            assert!(ran.is_ok(), "{}", String::from_utf8_lossy(&interp.trace));
            assert_eq!(String::from_utf8_lossy(&interp.out), "q\n");
            interp.timer.armed_in_registry()
        })
        .expect("the interpreter thread")
        .join()
        .expect("the run did not panic");
    assert!(!armed, "the timer is still armed with one activity left");
}

const MAIN_ENDS_PINNED: &str = "g = .flag~new\ng~start('spin')\ndo while g~started == 0\nend\n\
    say 'main sets'\ng~done = 1\nsay 'main ends'\nexit 3\n\
    ::class flag\n::attribute started unguarded\n::attribute done unguarded\n\
    ::method init\n  expose started done\n  started = 0\n  done = 0\n\
    ::method spin unguarded\n  self~started = 1\n  \
    interpret \"do while \\self~done; end\"\n  say 'spin ended'\n";

/// Main is not buried by its own root loop: a started activity's pinned
/// yields run it, and where its body ends there, the program ends with its
/// status once the pinned activity has (ruling P30).
#[test]
fn main_ends_inside_a_pinned_yield() {
    let source = MAIN_ENDS_PINNED;
    for invocation in [
        Invocation::none()
            .with_switch_mode(SwitchMode::EveryOpportunity)
            .with_deadline(std::time::Duration::from_secs(60)),
        Invocation::none().with_deadline(std::time::Duration::from_secs(60)),
    ] {
        let outcome = run_with(source, invocation);
        assert_eq!(outcome.exit_code, 3, "{}", stderr(&outcome));
        assert_eq!(stdout(&outcome), "main sets\nmain ends\nspin ended\n");
    }
}

/// A refusal in a started activity after main ended inside that activity's
/// pinned yield is reported at the program's end, as it is where main ends
/// at its root, under the timer and under `EveryOpportunity`. Where main
/// waits for the activity instead, the refusal answers main's wait.
#[test]
fn a_refusal_after_main_ended_in_a_nested_round_is_reported() {
    let source = |end: &str| {
        format!(
            "g = .flag~new\na = g~start('spin')\ndo while g~started == 0\nend\ng~done = 1\n{end}\n\
             ::class flag\n::attribute started unguarded\n::attribute done unguarded\n\
             ::method init\n  expose started done\n  started = 0\n  done = 0\n\
             ::method spin unguarded\n  self~started = 1\n  \
             interpret \"do while \\self~done; end\"\n  say 'spin ended'\n  \
             x = .RexxInfo~executable\n  say 'spin after' x\n"
        )
    };
    for (end, expected) in [
        ("say 'main ends'; exit 3", &["main ends", "spin ended"][..]),
        ("a~wait; say 'main ends'; exit 3", &["spin ended"][..]),
    ] {
        for invocation in [
            Invocation::none()
                .with_switch_mode(SwitchMode::EveryOpportunity)
                .with_deadline(std::time::Duration::from_secs(60)),
            Invocation::none().with_deadline(std::time::Duration::from_secs(60)),
        ] {
            let outcome = run_with(&source(end), invocation);
            assert_eq!(outcome.exit_code, 120, "{end}: {}", stderr(&outcome));
            assert_eq!(
                stderr(&outcome),
                "rexx-exec: method \"EXECUTABLE\" of class \"RexxInfo\" is not implemented \
                 (Phase 9)\n"
            );
            let printed = stdout(&outcome);
            let mut lines: Vec<&str> = printed.lines().collect();
            lines.sort_unstable();
            assert_eq!(lines, expected, "{end}");
        }
    }
}

const SLEEPERS: &str = "a = .t~new~start('nap', 'A', 2)\nb = .t~new~start('nap', 'B', 1)\n\
                        a~wait\nb~wait\nsay 'done'\n\
                        ::class t\n::method nap\n  use arg name, secs\n  call SysSleep secs\n  \
                        say name 'woke'\n";

/// Two started activities sleep at once: they wake in deadline order, the
/// longer sleeper started first, and the run takes about the longer sleep,
/// well below the sum.
#[test]
fn sleepers_wake_in_deadline_order_and_overlap() {
    for invocation in [
        Invocation::none().with_deadline(RUN_DEADLINE),
        Invocation::none()
            .with_deadline(RUN_DEADLINE)
            .with_switch_mode(SwitchMode::EveryOpportunity),
    ] {
        let began = std::time::Instant::now();
        let outcome = run_with(SLEEPERS, invocation);
        let took = began.elapsed();
        assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
        assert_eq!(stdout(&outcome), "B woke\nA woke\ndone\n");
        assert!(
            took >= std::time::Duration::from_secs(2)
                && took < std::time::Duration::from_millis(2900),
            "took {took:?}"
        );
    }
}

/// A sleep inside a sort comparator is a pinned wait: the activity that
/// wakes during it runs on the comparator's stack, so its line comes before
/// the sort ends.
#[test]
fn a_sleep_in_a_sort_comparator_runs_the_others_meanwhile() {
    let source = "b = .t~new~start('other')\narr = .array~of(3, 1, 2)\n\
                  arr~sortWith(.cmp~new)\nsay 'sorted' arr~makeString('L', ' ')\n\
                  ::class t\n::method other\n  call SysSleep 0.2\n  say 'other ran'\n\
                  ::class cmp\n::method compare\n  use arg l, r\n  call SysSleep 0.3\n  \
                  return l - r\n";
    for invocation in [
        Invocation::none().with_deadline(RUN_DEADLINE),
        Invocation::none()
            .with_deadline(RUN_DEADLINE)
            .with_switch_mode(SwitchMode::EveryOpportunity),
    ] {
        let outcome = run_with(source, invocation);
        assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
        assert_eq!(stdout(&outcome), "other ran\nsorted 1 2 3\n");
    }
}

/// A busy main sees the flag a sleeper sets: the countdown's visits wake the
/// sleeper when it is due, and the timer then ends main's slice.
#[test]
fn a_sleeper_wakes_while_main_is_busy() {
    let outcome = run_with(
        "f = .flag~new\nf~start('setLater')\ndo while \\f~done\nend\nsay 'saw flag'\n\
         ::class flag\n::attribute done unguarded\n::method init\n  expose done\n  done = 0\n\
         ::method setLater unguarded\n  call SysSleep 0.2\n  say 'set'\n  self~done = 1\n",
        Invocation::none().with_deadline(std::time::Duration::from_secs(60)),
    );
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    assert_eq!(stdout(&outcome), "set\nsaw flag\n");
}

/// A run's deadline ends a wait on a sleeper due after it.
#[test]
fn a_deadline_ends_a_wait_on_a_later_sleeper() {
    let began = std::time::Instant::now();
    let outcome = run_with(
        "call SysSleep 30\nsay 'woke'\n",
        Invocation::none().with_deadline(std::time::Duration::from_millis(300)),
    );
    assert!(began.elapsed() < std::time::Duration::from_secs(10));
    assert_eq!(outcome.exit_code, crate::DEADLINE_EXIT);
    assert_eq!(stdout(&outcome), "");
}

/// A run's deadline also ends the program's end waiting on a started
/// activity's sleep.
#[test]
fn a_deadline_ends_the_programs_wait_on_a_started_sleeper() {
    let began = std::time::Instant::now();
    let outcome = run_with(
        "t = .t~new~start('nap')\nsay 'main'\n::class t\n::method nap\n  call SysSleep 30\n",
        Invocation::none().with_deadline(std::time::Duration::from_millis(300)),
    );
    assert!(began.elapsed() < std::time::Duration::from_secs(10));
    assert_eq!(outcome.exit_code, crate::DEADLINE_EXIT);
    assert_eq!(stdout(&outcome), "main\n");
}

/// An inverted pinned wait is refused naming the kind of its wait under the
/// switch mode too, whose pinned yields run the same activities.
#[test]
fn an_inverted_pinned_wait_is_refused_under_every_opportunity() {
    let outcome = run_with(
        &HIDDEN_INVERSION.replace("$MAIN", "interpret \"say 'main got' m0~result\""),
        Invocation::none()
            .with_deadline(RUN_DEADLINE)
            .with_switch_mode(SwitchMode::EveryOpportunity),
    );
    assert_eq!(outcome.exit_code, 120);
    assert_eq!(stdout(&outcome), "s3 sent m0\n");
    assert_eq!(stderr(&outcome), INVERTED);
}

/// Nested pinned waits share one Rust stack, so the stack remaining bounds
/// their depth with 11.1: each started method waits pinned on the next, on
/// an interpreter thread small enough for the bound to fire.
#[test]
fn nested_pinned_waits_are_bounded_by_the_stack_remaining() {
    let outcome = crate::run_program_on_stack(
        "/tmp/scheduler.rex",
        b"say .w~new~chain(10500)\n::class w\n::method chain unguarded\n  use arg n\n  \
          if n = 0 then return 0\n  m = .w~new~start('chain', n - 1)\n  \
          interpret 'r = m~result'\n  return r + 1\n"
            .to_vec(),
        Invocation::none().with_deadline(RUN_DEADLINE),
        64 * 1024 * 1024,
    );
    assert_eq!(outcome.exit_code, 245, "{}", stderr(&outcome));
    assert_eq!(stdout(&outcome), "");
    assert!(
        stderr(&outcome)
            .contains("Error 11.1:  Insufficient control stack space; cannot continue execution.")
    );
}

/// One activity's recursion through `INTERPRET` stops at the stack remaining
/// before its activation count reaches the limit.
#[test]
fn recursion_is_bounded_by_the_stack_remaining() {
    let outcome = crate::run_program_on_stack(
        "/tmp/scheduler.rex",
        b"say f(1)\nexit\nf: procedure\n  use arg n\n  if n >= 9000 then return n\n  \
          interpret 'r = f(n + 1)'\n  return r\n"
            .to_vec(),
        Invocation::none().with_deadline(RUN_DEADLINE),
        64 * 1024 * 1024,
    );
    assert_eq!(outcome.exit_code, 245, "{}", stderr(&outcome));
    assert!(stderr(&outcome).contains("Error 11.1:"));
}

const REPLIED: &str = "say .c~new~m\nsay 'caller 1'\nsay 'caller 2'\n\
                       ::class c\n::method m\n  reply 'answered'\n  say 'rest 1'\n  say 'rest 2'\n";

/// The lines of `out` that start with `prefix`, in order.
fn lines_of<'a>(out: &'a str, prefix: &str) -> Vec<&'a str> {
    out.lines()
        .filter(|line| line.starts_with(prefix))
        .collect()
}

/// A `REPLY` asks for no switch: the sender runs on until an ordinary one,
/// here its end, and the rest of the method runs after it (ruling P34).
#[test]
fn a_reply_leaves_its_sender_running_until_an_ordinary_switch() {
    let outcome = run(REPLIED);
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    assert_eq!(
        stdout(&outcome),
        "answered\ncaller 1\ncaller 2\nrest 1\nrest 2\n"
    );
}

/// With another activity ready, a `REPLY`'s sender yields at its next clause
/// boundary, so the waiting continuation and the new one run before the
/// sender's next line; the first `REPLY`, with none ready, does not yield
/// (ruling P36). `AtClause` far past the program keeps the timer out.
#[test]
fn a_reply_yields_at_the_next_boundary_only_where_another_activity_is_ready() {
    let program = "o = .k~new\nsay o~a\nsay o~b\nsay 'main 1'\nsay 'main 2'\n\
                   ::class k\n::method a unguarded\n  reply 'ra'\n  say 'a rest'\n\
                   ::method b unguarded\n  reply 'rb'\n  say 'b rest'\n";
    let outcome = run_with(
        program,
        Invocation::none()
            .with_deadline(RUN_DEADLINE)
            .with_switch_mode(SwitchMode::AtClause(1_000_000)),
    );
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    assert_eq!(stdout(&outcome), "ra\nrb\na rest\nb rest\nmain 1\nmain 2\n");
    let outcome = run_with(
        REPLIED,
        Invocation::none()
            .with_deadline(RUN_DEADLINE)
            .with_switch_mode(SwitchMode::AtClause(1_000_000)),
    );
    assert_eq!(
        stdout(&outcome),
        "answered\ncaller 1\ncaller 2\nrest 1\nrest 2\n"
    );
}

/// The sender's and the continuation's lines interleave as the switch mode
/// schedules them, each side's own lines in order, and more than one
/// interleaving occurs across the modes.
#[test]
fn a_reply_continuation_interleaves_with_its_sender_under_the_switch_mode() {
    let modes = std::iter::once(SwitchMode::EveryOpportunity)
        .chain((1..=12).map(SwitchMode::AtClause))
        .collect::<Vec<_>>();
    let mut seen = std::collections::BTreeSet::new();
    for mode in modes {
        let outcome = run_with(
            REPLIED,
            Invocation::none()
                .with_deadline(RUN_DEADLINE)
                .with_switch_mode(mode.clone()),
        );
        let out = stdout(&outcome);
        assert_eq!(outcome.exit_code, 0, "{mode:?}: {}", stderr(&outcome));
        assert_eq!(out.lines().next(), Some("answered"), "{mode:?}: {out}");
        assert_eq!(
            lines_of(&out, "caller"),
            ["caller 1", "caller 2"],
            "{mode:?}"
        );
        assert_eq!(lines_of(&out, "rest"), ["rest 1", "rest 2"], "{mode:?}");
        assert_eq!(out.lines().count(), 5, "{mode:?}: {out}");
        seen.insert(out);
    }
    assert!(seen.len() > 1, "one interleaving only: {seen:?}");
}

/// A stream builtin that builds its `Stream` runs that send's body on a
/// nested driver under a pinned frame, so a switch-mode slice inside it is
/// deferred: on a `REPLY` continuation and on a started activity, each with
/// another activity ready.
#[test]
fn a_stream_a_builtin_builds_defers_the_slice_under_the_switch_mode() {
    for (program, expected) in [
        (
            "say .k~new~m\ndo i = 1 to 20; nop; end\nsay 'main end'\n::class k\n::method m\n  \
             reply 'r'\n  say stream('/nonexistent/dir/x', 'S')\n",
            ["UNKNOWN", "main end", "r"].as_slice(),
        ),
        (
            "m = .k~new~start('m')\ndo i = 1 to 20; nop; end\nsay m~result\n::class k\n\
             ::method m\n  return stream('/nonexistent/dir/x', 'S')\n",
            ["UNKNOWN"].as_slice(),
        ),
    ] {
        let outcome = run_with(
            program,
            Invocation::none()
                .with_deadline(RUN_DEADLINE)
                .with_switch_mode(SwitchMode::EveryOpportunity),
        );
        assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
        let mut lines = stdout(&outcome)
            .lines()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        lines.sort();
        assert_eq!(lines, expected, "{program}");
    }
}

/// An interpreter thread no larger than the stack margin is refused.
#[test]
#[should_panic(expected = "leaves nothing beyond")]
fn a_stack_within_the_margin_is_refused() {
    Interp::new().measure_stack(0, super::STACK_MARGIN);
}

/// Runs `source` unswitched, collecting at every allocation and, where
/// `switched`, under `EveryOpportunity`, each answering `expected` at exit 0.
fn in_modes(source: &str, switched: bool, expected: &str) {
    let mut outcomes = vec![
        ("unswitched", run(source)),
        (
            "collect",
            crate::run_program_collect_every_alloc(
                "/tmp/scheduler.rex",
                source.as_bytes().to_vec(),
                Invocation::none().with_deadline(RUN_DEADLINE),
            ),
        ),
    ];
    if switched {
        let invocation = Invocation::none()
            .with_deadline(RUN_DEADLINE)
            .with_switch_mode(SwitchMode::EveryOpportunity);
        outcomes.push(("every", run_with(source, invocation)));
    }
    for (mode, outcome) in outcomes {
        assert_eq!(outcome.exit_code, 0, "{mode}: {}", stderr(&outcome));
        assert_eq!(stdout(&outcome), expected, "{mode}");
    }
}

/// A `RexxContext` of an activation on another live activity reads that
/// activation, its activity's number and its stack; once the activation
/// has ended it is 98.981.
#[test]
fn a_context_reads_an_activation_of_another_live_activity() {
    in_modes(
        "o = .k~new\ngate = .message~new(o, 'open')\nm = o~start('hold', gate)\n\
         do while o~ctx == .nil\n  call SysSleep 0.01\nend\nc = o~ctx\n\
         say c~line c~name c~variables~x c~digits c~thread .context~thread\n\
         say c~stackFrames~items c~stackFrames[1]~name c~args[1]~messageName\n\
         gate~send\nm~wait\nsignal on syntax\nsay c~line\nexit\n\
         syntax: say condition('o')~code\n\
         ::class k\n::attribute ctx get unguarded\n\
         ::method init\n  expose ctx\n  ctx = .nil\n\
         ::method open unguarded\n\
         ::method hold unguarded\n  expose ctx\n  use arg gate\n  x = 'held'\n\
         \x20 numeric digits 12\n  ctx = .context\n  gate~wait\n",
        true,
        "28 HOLD held 12 2 1\n1 HOLD OPEN\n98.981\n",
    );
}

/// The replier `r` of [`a_context_follows_its_activation_to_a_reply_continuation`],
/// whose continuation waits on `gate` after setting `waiting`.
const REPLIER: &str = "::class k\n::attribute waiting get unguarded\n\
                       ::attribute inv get unguarded\n\
                       ::method init\n  expose waiting\n  waiting = 0\n\
                       ::method open unguarded\n\
                       ::method r unguarded\n  expose waiting inv\n  use arg gate\n  \
                       y = 'before'\n  inv = .context~invocation\n  reply .context\n  \
                       y = 'after'\n  waiting = 1\n  gate~wait\n";

/// A `RexxContext` of an activation a `REPLY` moved reads it on the new
/// activity, keeping its `~invocation`: before the continuation has run,
/// which only the sender-first schedule fixes, and while it waits.
#[test]
fn a_context_follows_its_activation_to_a_reply_continuation() {
    let pending = format!(
        "o = .k~new\ngate = .message~new(o, 'open')\nc = o~r(gate)\n\
         say c~line c~variables~y c~thread .context~thread (c~invocation == o~inv)\n\
         gate~send\n{REPLIER}"
    );
    in_modes(&pending, false, "18 before 2 1 1\n");
    let waiting = format!(
        "o = .k~new\ngate = .message~new(o, 'open')\nc = o~r(gate)\n\
         do while o~waiting == 0\n  call SysSleep 0.01\nend\n\
         say c~line c~variables~y c~thread (c~invocation == o~inv)\n\
         gate~send\n{REPLIER}"
    );
    in_modes(&waiting, true, "24 after 2 1\n");
}

/// A `REPLY` continuation keeps its method's own `RANDOM` seed, and main's
/// stream is main's own: oracle 5 of 5 runs `main 214`, `m 457`, `got 1`,
/// `cont 100`, `main 807`.
#[test]
fn a_reply_continuation_keeps_its_methods_seed_and_main_keeps_its_own() {
    in_modes(
        "say 'main' random(1,1000,3)\no = .t~new\nsay 'got' o~m\ncall SysSleep 0.2\n\
         say 'main' random(1,1000)\n\
         ::class t\n::method m\n  say 'm' random(1,1000,11)\n  reply 1\n  \
         say 'cont' random(1,1000)\n",
        true,
        "main 214\nm 457\ngot 1\ncont 100\nmain 807\n",
    );
}

/// A `REPLY` continuation keeps the elapsed clock its method reset before
/// the `REPLY`, and main's clock is main's own: oracle 5 of 5 runs
/// `got 1`, `cont 1 1`, `main 1`.
#[test]
fn a_reply_continuation_keeps_its_methods_elapsed_clock() {
    in_modes(
        "o = .t~new\nx = time('e')\ncall SysSleep 0.1\nsay 'got' o~m\ncall SysSleep 0.3\n\
         say 'main' (time('e') >= 0.4)\n\
         ::class t\n::method m\n  call SysSleep 0.1\n  call time 'r'\n  reply 1\n  \
         call SysSleep 0.02\n  e = time('e')\n  say 'cont' (e > 0) (e < 0.1)\n",
        true,
        "got 1\ncont 1 1\nmain 1\n",
    );
}

/// A method starts with an elapsed clock of its own, which its `REPLY`
/// continuation keeps: oracle 5 of 5 runs `m before zero 1`, `main got 7`,
/// `main elapsed 1`, `m after 1 1`, `m after below main 1`.
#[test]
fn a_method_starts_its_own_elapsed_clock_across_a_reply() {
    in_modes(
        "call time 'r'\ncall SysSleep 0.05\no = .t~new\nsay 'main got' o~m\n\
         e = time('e')\nsay 'main elapsed' (e >= 0.05)\ncall SysSleep 0.3\n\
         say 'm after below main' (o~after < time('e'))\n\
         ::class t\n::attribute after get unguarded\n::method m\n  expose after\n  \
         after = 99\n  say 'm before zero' (time('e') = 0)\n  reply 7\n  \
         call SysSleep 0.01\n  after = time('e')\n  say 'm after' (after > 0) (after < 0.2)\n",
        true,
        "m before zero 1\nmain got 7\nmain elapsed 1\nm after 1 1\nm after below main 1\n",
    );
}

/// A method's `SETLOCAL` list moves with its `REPLY` continuation, and main's
/// `ENDLOCAL` finds main's own list empty: oracle 5 of 5 runs `got 1`,
/// `cont endlocal 1`, `main endlocal 0`. One restore in the process
/// (`oracle-crashes.txt` entry 10b).
#[test]
fn a_reply_continuation_keeps_its_methods_setlocal_list() {
    in_modes(
        "o = .t~new\nsay 'got' o~m\ncall SysSleep 0.2\nsay 'main endlocal' endlocal()\n\
         ::class t\n::method m\n  call setlocal\n  call value 'P6X', 'yes', 'ENVIRONMENT'\n  \
         reply 1\n  say 'cont endlocal' endlocal()\n",
        true,
        "got 1\ncont endlocal 1\nmain endlocal 0\n",
    );
}

/// The condition a method's `SIGNAL ON` handler took moves with the `REPLY`
/// the handler makes, so `RAISE PROPAGATE` in the continuation re-raises it:
/// oracle 5 of 5 runs report 42.3 at rc 0 after `got 1`, `cont propagates`,
/// `main done`.
#[test]
fn a_reply_continuation_propagates_its_methods_condition() {
    let outcome = run(
        "o = .t~new\nsay 'got' o~m\ncall SysSleep 0.2\nsay 'main done'\n\
         ::class t\n::method m\n  signal on syntax\n  x = 1/0\n  return 0\n\
         syntax:\n  reply 1\n  say 'cont propagates'\n  raise propagate\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    assert_eq!(stdout(&outcome), "got 1\ncont propagates\nmain done\n");
    let reports = stderr(&outcome);
    assert!(reports.contains("Error 42.3:"), "{reports}");
    assert!(!reports.contains("98.918"), "{reports}");
}

/// A `REPLY` inside a `CALL ON` handler is the handler's, not the method's:
/// oracle 5 of 5 runs 91.999 at rc 165 with nothing on stdout.
#[test]
fn a_reply_in_a_call_on_handler_does_not_reply_for_the_method() {
    let outcome = run(
        "o = .t~new\nsay 'got' o~m\ncall SysSleep 0.2\nsay 'main done'\n\
         ::class t\n::method m\n  call on user foo name h\n  raise user foo\n  \
         say 'cont after handler'\n  signal on any name trapped\n  raise propagate\n  \
         say 'cont no propagate'\n  return\n\
         h:\n  reply 1\n  say 'handler continues'\n  return\n\
         trapped:\n  say 'cont trapped' condition('C')\n  return\n",
    );
    assert_eq!(outcome.exit_code, 165, "{}", stderr(&outcome));
    assert_eq!(stdout(&outcome), "");
    assert!(
        stderr(&outcome).contains("Error 91.999:"),
        "{}",
        stderr(&outcome)
    );
}

/// A `SELECT CASE` value travels with the activation a `REPLY` moves, and
/// an internal call's own `SELECT CASE` in the continuation does not reach
/// it: oracle 5 of 5 runs `got 1`, `cont other`, `main done`.
#[test]
fn a_reply_continuation_keeps_its_select_case_value() {
    in_modes(
        "o = .t~new\nsay 'got' o~m\ncall SysSleep 0.2\nsay 'main done'\n\
         ::class t\n::method m\n  select case 'a'\n    when 'a' then do\n      reply 1\n      \
         select case 'b'\n        when g() then when 'zz' then say 'cont absorbed zz'\n        \
         otherwise say 'cont other'\n      end\n    end\n    otherwise nop\n  end\n  return\n\
         g:\n  select case 'zz'\n    when 'zz' then return 'b'\n    otherwise return 'y'\n  end\n",
        true,
        "got 1\ncont other\nmain done\n",
    );
}

/// One store wakes every activity parked in a `GUARD WHEN` on the variable.
#[test]
fn a_store_wakes_every_guard_when_waiter() {
    in_modes(
        "o = .k~new\na = o~start('w', 'a')\nb = o~start('w', 'b')\ncall SysSleep 0.2\n\
         o~set\nsay a~result b~result\n\
         ::class k\n::method init\n  expose v\n  v = 0\n\
         ::method set unguarded\n  expose v\n  v = 1\n\
         ::method w unguarded\n  expose v\n  use arg n\n  guard off when v = 1\n  return n\n",
        true,
        "a b\n",
    );
}

/// A started send's failure whose notifier fails is replaced by the
/// notifier's failure: the message is told of that one, both runs of the
/// notifier fail and both are reported, the send's own failure never; the
/// waiter the completion woke runs once the first notifier failure has
/// unwound. Unswitched, the order the oracle shows.
#[test]
fn a_notifier_failing_in_a_started_activity_replaces_its_failure() {
    let outcome = run(
        "m = .message~new(.t~new, 'boom')\nm~notify(.bad~new)\nm~start\nm~wait\n\
                       say 'main' m~hasError m~completed m~errorCondition~code\n\
                       ::class t\n::method boom\n  return 1/0\n\
                       ::class bad inherit MessageNotification\n::method messageComplete\n  \
                       use arg msg\n  say 'notifier sees' msg~errorCondition~code\n  \
                       return .nil~foo\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    assert_eq!(
        stdout(&outcome),
        "notifier sees 42.3\nmain 1 1 42.3\nnotifier sees 97.1\n"
    );
    let reports = stderr(&outcome);
    assert_eq!(reports.matches("Error 97.1:").count(), 2, "{reports}");
    assert!(!reports.contains("42.3"), "{reports}");
}

/// A started send's success notification that fails is the send's failure:
/// the notifier runs again for it and once more for its own failure, the
/// last two are reported, and the waiter reads the result between.
#[test]
fn a_notifier_failing_on_a_started_success_is_the_sends_failure() {
    let outcome = run(
        "m = .message~new('abc', 'length')\nm~notify(.bad~new)\nm~start\nm~wait\n\
                       say 'main' m~result\n\
                       ::class bad inherit MessageNotification\n::method messageComplete\n  \
                       say 'in notifier'\n  return 1/0\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    assert_eq!(
        stdout(&outcome),
        "in notifier\nmain 3\nin notifier\nin notifier\n"
    );
    assert_eq!(stderr(&outcome).matches("Error 42.3:").count(), 2);
}

/// Started sends whose notifiers all fail yield in turn, each one's end
/// running on below the next one's round, which sets it aside: every
/// notifier runs, twice, and the program ends as the oracle's does.
#[test]
fn notifier_failures_in_several_started_activities_each_end_their_own() {
    let outcome = run(
        "b = .bad~new\ndo i = 1 to 3\n  m = .message~new('abc', 'length')\n  m~notify(b)\n  \
         m~start\nend\ncall SysSleep 0.2\nsay 'main end'\n\
         ::class bad inherit MessageNotification\n::method messageComplete\n  say 'notified'\n  \
         return 1/0\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    assert_eq!(
        stdout(&outcome),
        format!("{}main end\n", "notified\n".repeat(9))
    );
    assert_eq!(stderr(&outcome).matches("Error 42.3:").count(), 6);
}

/// A notifier that fails in a started activity run inside main's pinned
/// notifier wait replaces the message's outcome before main reads it
/// (ruling P38): main is buried below that round. The oracle mostly lets main
/// read first: `outer waited 1 42.3`, and `outer waited 3` at rc 0.
#[test]
fn a_notifier_failing_inside_a_pinned_notifier_wait_replaces_the_outcome() {
    let outer = |body: &str, bad: &str| {
        run(&format!(
            "m0 = .message~new('a', 'length')\nm0~notify(.outer~new)\nm0~send\n\
             say 'main end'\n::class outer inherit MessageNotification\n\
             ::method messageComplete\n{body}\
             ::class bad inherit MessageNotification\n::method messageComplete\n{bad}"
        ))
    };
    let replaced = outer(
        "  m = .message~new(.t~new, 'boom')\n  m~notify(.bad~new)\n  m~start\n  m~wait\n  \
         say 'outer waited' m~hasError m~errorCondition~code\n\
         ::class t\n::method boom\n  return 1/0\n",
        "  use arg msg\n  say 'notifier sees' msg~errorCondition~code\n  return .nil~foo\n",
    );
    assert_eq!(replaced.exit_code, 0, "{}", stderr(&replaced));
    assert_eq!(
        stdout(&replaced),
        "notifier sees 42.3\nnotifier sees 97.1\nouter waited 1 97.1\nmain end\n"
    );
    let raised = outer(
        "  m = .message~new('abc', 'length')\n  m~notify(.bad~new)\n  m~start\n  m~wait\n  \
         say 'outer waited' m~result\n",
        "  say 'in notifier'\n  return 1/0\n",
    );
    assert_eq!(raised.exit_code, 214, "{}", stderr(&raised));
    assert_eq!(stdout(&raised), "in notifier\nin notifier\nin notifier\n");
}

/// Notifier failures in more started activities than an interpreter stack
/// of 33 MiB has room to nest the yields of: the yields stop nesting, so no
/// Error 11 reaches main or another activity, and every notifier failure is
/// reported, as the oracle reports each.
#[test]
fn notifier_failures_too_many_to_nest_are_each_reported() {
    for (starts, waits) in [
        (
            "do i = 1 to 3000\n  m = .message~new('abc', 'length')\n  m~notify(.bad~new)\n  \
             m~start\nend\n",
            "do j = 1 to 200000\nend\nsay 'done'\n",
        ),
        (
            "s = .starter~new~start('go')\n",
            "call SysSleep 1\nsay 'done'\n",
        ),
    ] {
        let ran = pool::run_shaped(
            &format!(
                "signal on syntax\n{starts}{waits}exit\nsyntax:\nsay 'trapped' condition('o')~code\n\
                 ::class starter\n::method go\n  do i = 1 to 3000\n    \
                 m = .message~new('abc', 'length')\n    m~notify(.bad~new)\n    m~start\n  end\n\
                 ::class bad inherit MessageNotification\n::method messageComplete\n  return 1/0\n"
            ),
            pool::Shape {
                interpreter_stack: 33 * 1024 * 1024,
                ..pool::SHAPE
            },
        )
        .expect("the run did not panic");
        let stderr = ran.stderr();
        assert_eq!(ran.outcome.exit_code, 0, "{starts}");
        assert_eq!(ran.stdout(), "done\n", "{starts}");
        assert!(!stderr.contains("Error 11"), "{starts}");
        assert_eq!(stderr.matches("Error 42.3:").count(), 6000, "{starts}");
    }
}

/// The program does not wait for an activity a termination `UNINIT` starts:
/// it ends at once with the `UNINIT`'s own lines, as the oracle's does.
#[test]
fn an_activity_a_termination_uninit_starts_is_not_waited_for() {
    let began = std::time::Instant::now();
    let outcome = run(
        "o = .k~new\nsay 'main end'\nexit 7\n::class k\n::method uninit\n  \
                       say 'uninit starts'\n  .z~new~start('go')\n  say 'uninit after start'\n\
                       ::class z\n::method go\n  do forever\n    call SysSleep 1\n  end\n",
    );
    assert!(began.elapsed() < std::time::Duration::from_secs(10));
    assert_eq!(outcome.exit_code, 7, "{}", stderr(&outcome));
    assert_eq!(
        stdout(&outcome),
        "main end\nuninit starts\nuninit after start\n"
    );
    assert_eq!(stderr(&outcome), "");
}

/// The continuation a termination `UNINIT`'s `REPLY` starts is not waited
/// for (ruling P39), so its line never shows; the oracle prints it in most
/// runs, racing its own process exit.
#[test]
fn a_termination_uninits_reply_continuation_is_not_waited_for() {
    let outcome = run("o = .k~new\nsay 'main end'\n::class k\n::method uninit\n  \
                       say 'uninit before reply'\n  reply\n  say 'uninit after reply'\n");
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    assert_eq!(stdout(&outcome), "main end\nuninit before reply\n");
}

/// An object a collection readied while a started activity ran is
/// finalized when that activity ends, before main goes on.
#[test]
fn an_uninit_a_collection_readied_runs_when_its_activity_ends() {
    let outcome = crate::run_program_collect_every_alloc(
        "/tmp/scheduler.rex",
        b"m = .w~new~start('run')\nm~wait\nsay 'main after'\n\
          ::class k\n::method uninit\n  say 'uninit'\n\
          ::class w\n::method run\n  d = .k~new\n  drop d\n  s = 'a' || random()\n  \
          say 'activity end'\n"
            .to_vec(),
        Invocation::none().with_deadline(RUN_DEADLINE),
    );
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    assert_eq!(stdout(&outcome), "activity end\nuninit\nmain after\n");
}

/// Every refusal the `UNINIT`s at an activity's end meet is reported, in
/// order, not only the first.
#[test]
fn every_refusal_of_the_uninits_at_an_activitys_end_is_reported() {
    let outcome = crate::run_program_collect_every_alloc(
        "/tmp/scheduler.rex",
        b"m = .w~new~start('run')\nm~wait\nsay 'main end'\n\
          ::class k\n::method uninit\n  say 'uninit'\n  do label b\n    reply\n  end\n\
          ::class w\n::method run\n  a = .k~new\n  b = .k~new\n  drop a b\n  \
          s = 'a' || random()\n  say 'activity end'\n"
            .to_vec(),
        Invocation::none().with_deadline(RUN_DEADLINE),
    );
    let refusal =
        "rexx-exec: a REPLY its method body runs on a nested Rust frame is not implemented\n";
    assert_eq!(
        stderr(&outcome).matches(refusal).count(),
        2,
        "{}",
        stderr(&outcome)
    );
    assert_eq!(stdout(&outcome), "activity end\nuninit\nuninit\n");
}

/// A refusal in main ends the run at once, without waiting for an activity
/// that is still sleeping.
#[test]
fn a_refusal_in_main_does_not_wait_for_the_other_activities() {
    let began = std::time::Instant::now();
    let outcome = run(".w~new~start('nap')\nsay 'main'\nsay .k~new~m\n\
                       ::class w\n::method nap\n  call SysSleep 99999\n\
                       ::class k\n::method m\n  do label b\n    reply 1\n  end\n");
    assert!(began.elapsed() < std::time::Duration::from_secs(10));
    assert_eq!(outcome.exit_code, 120);
    assert_eq!(stdout(&outcome), "main\n");
    assert_eq!(
        stderr(&outcome),
        "rexx-exec: a REPLY its method body runs on a nested Rust frame is not implemented\n"
    );
}

/// A started activity blocked in a `GUARD WHEN` nothing can make true keeps
/// the program's end waiting, as the oracle's does, until the run's deadline;
/// main alone blocked there is the wait nothing left can end (ruling P27).
#[test]
fn a_blocked_guard_when_keeps_the_programs_end_waiting() {
    let began = std::time::Instant::now();
    let outcome = run_with(
        "m = .k~new~start('m')\nsay 'main done'\n\
         ::class k\n::method m\n  expose v\n  v = 0\n  say 'waits'\n  guard on when v = 1\n",
        Invocation::none().with_deadline(std::time::Duration::from_millis(300)),
    );
    assert!(began.elapsed() >= std::time::Duration::from_millis(300));
    assert_eq!(outcome.exit_code, crate::DEADLINE_EXIT);
    assert_eq!(stdout(&outcome), "main done\nwaits\n");
    assert_eq!(outcome.stderr, crate::DEADLINE_REPORT);
    let outcome =
        run("say .k~new~m\n::class k\n::method m\n  expose v\n  v = 0\n  guard on when v = 1\n");
    assert_eq!(
        stderr(&outcome),
        "rexx-exec: a wait that nothing left to run can end is not implemented\n"
    );
}

/// An uncancelled `.Ticker` keeps the program's end waiting, as the oracle's
/// does, until the run's deadline, and goes on triggering its target.
#[test]
fn an_uncancelled_ticker_keeps_the_programs_end_waiting() {
    let began = std::time::Instant::now();
    let outcome = run_with(
        "t = .t~new\nk = .ticker~new(0.05, t)\nsay 'main done'\n\
         ::class t inherit AlarmNotification\n::method init\n  expose n\n  n = 0\n\
         ::method triggered\n  expose n\n  n = n + 1\n  if n = 3 then say 'ticked'\n",
        Invocation::none().with_deadline(std::time::Duration::from_millis(2000)),
    );
    assert!(began.elapsed() >= std::time::Duration::from_millis(2000));
    assert_eq!(outcome.exit_code, crate::DEADLINE_EXIT);
    assert_eq!(stdout(&outcome), "main done\nticked\n");
    assert_eq!(outcome.stderr, crate::DEADLINE_REPORT);
}

/// A store that wakes an activity parked in a `GUARD WHEN` makes it ready
/// without a switch, so the storing activity runs on to its next wait or
/// end before the waiter runs, as the oracle's notifier does in a run of
/// clauses with no I/O (`TEST_BASE_ALARM`'s `triggered`); a store or `DROP`
/// that wakes nobody does not switch either, whether the variable was
/// watched once (`v`) or never (`w`): the oracle prints `a` first in 30 of 30
/// runs of each. `AtClause` far past the program keeps the timer out.
#[test]
fn a_store_wakes_a_parked_watcher_without_a_switch() {
    const CLASS: &str = "::class k\n::method arm\n  expose v\n  v = 0\n  guard on when v = 0\n\
                         ::method store unguarded\n  expose v\n  v = 1\n\
                         ::method dropit\n  expose v\n  drop v\n\
                         ::method plain\n  expose w\n  w = 1\n\
                         ::method waiter unguarded\n  expose v\n  guard off when v = 1\n  \
                         say 'woke'\n\
                         ::method b\n  say 'b'\n";
    let run_switched = |main: String| {
        run_with(
            &format!("{main}{CLASS}"),
            Invocation::none()
                .with_deadline(RUN_DEADLINE)
                .with_switch_mode(SwitchMode::AtClause(1_000_000)),
        )
    };
    for step in ["store", "dropit", "plain"] {
        let outcome = run_switched(format!(
            "o = .k~new\no~arm\nm = .k~new~start('b')\no~{step}\nsay 'a'\nm~wait\n"
        ));
        assert_eq!(outcome.exit_code, 0, "{step}: {}", stderr(&outcome));
        assert_eq!(stdout(&outcome), "a\nb\n", "{step}");
    }
    let outcome = run_switched(
        "o = .k~new\nm = o~start('waiter')\ncall SysSleep 0.05\no~store\nsay 'a'\nm~wait\n"
            .to_string(),
    );
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    assert_eq!(stdout(&outcome), "a\nwoke\n");
}

/// `collect` stops the world, which asserts that the caller holds the baton.
#[test]
#[should_panic(expected = "a collection on a thread not holding the interpreter's baton")]
fn a_collection_without_the_baton_fails_its_assertion() {
    let mut interp = Interp::new();
    interp.baton.release();
    interp.collect_now();
}

/// The thread that creates an interpreter holds its baton, so it collects.
#[test]
fn the_creating_thread_holds_the_baton_and_collects() {
    let mut interp = Interp::new();
    assert!(interp.baton.held_here());
    interp.collect_now();
}

/// Posts reach the holder in order, and the holder's `drain` answers nothing
/// once `INBOX` is clear.
#[test]
fn the_inbox_answers_posts_in_order_then_nothing() {
    let inbox = crate::timer::Inbox::<u32>::new();
    assert!(inbox.drain().is_empty());
    inbox.post(1);
    inbox.post(2);
    assert!(inbox.requests().pending(crate::timer::INBOX));
    assert_eq!(Vec::from(inbox.drain()), [1, 2]);
    assert!(!inbox.requests().pending(crate::timer::INBOX));
    assert!(inbox.drain().is_empty());
}

/// A post from another thread ends the holder's idle.
#[test]
fn a_post_from_another_thread_ends_an_idle() {
    let inbox = std::sync::Arc::new(crate::timer::Inbox::<u32>::new());
    let poster = {
        let inbox = std::sync::Arc::clone(&inbox);
        std::thread::spawn(move || inbox.post(7))
    };
    let mut taken = Vec::from(inbox.idle());
    taken.extend(inbox.drain());
    poster.join().expect("the poster");
    assert_eq!(taken, [7]);
}

/// `uniform:1` in FIFO order preempts at every contended step, which is what
/// `EveryOpportunity` does: on the switch-mode programs here whose output
/// reads no clock, both modes answer alike under any seed.
#[test]
fn uniform_1_in_fifo_order_runs_as_every_opportunity() {
    let inverted = HIDDEN_INVERSION.replace("$MAIN", "interpret \"say 'main got' m0~result\"");
    let rooted = HIDDEN_INVERSION.replace("$MAIN", "say 'main got' m0~result");
    let programs = [
        INTERLEAVED,
        SORTED,
        REPLIED,
        SLEEPERS,
        HALT_START,
        TWO_PINNED_WAITERS,
        MAIN_ENDS_PINNED,
        &inverted,
        &rooted,
    ];
    let mode = |mode: SwitchMode| {
        Invocation::none()
            .with_deadline(RUN_DEADLINE)
            .with_switch_mode(mode)
    };
    for program in programs {
        let every = run_with(program, mode(SwitchMode::EveryOpportunity));
        for seed in [1, 2] {
            let spec = format!("sim:{seed},uniform:1,order=fifo");
            let config = crate::SimConfig::parse(&spec).expect("a sim spec");
            let simulated = run_with(program, mode(SwitchMode::Sim(config)));
            assert_eq!(
                (simulated.exit_code, stdout(&simulated), stderr(&simulated)),
                (every.exit_code, stdout(&every), stderr(&every)),
                "{spec}: {program}"
            );
        }
    }
}

mod callbacks;
mod lent;
mod native;
mod pool;
