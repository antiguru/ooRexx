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
            Invocation::none().with_switch_mode(SwitchMode::EveryOpportunity),
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
            Invocation::none().with_switch_mode(SwitchMode::AtClause(clause)),
        );
        assert_eq!(
            stdout(&outcome),
            "cmp 1 3\ncmp 2 3\ncmp 2 1\nother ran\nsorted 1 2 3\n",
            "a slice at clause {clause}"
        );
    }
    let outcome = run_with(
        SORTED,
        Invocation::none().with_switch_mode(SwitchMode::AtClause(3)),
    );
    assert_eq!(
        stdout(&outcome),
        "other ran\ncmp 1 3\ncmp 2 3\ncmp 2 1\nsorted 1 2 3\n"
    );
}

/// Message.testGroup's `test_halt_start` (`:643`), its sleeps replaced by
/// loops so that the switch mode, not time, decides that the started
/// activity is running when the halt is asked.
#[test]
fn message_halt_in_the_shape_of_test_halt_start() {
    let outcome = run_with(
        "m = .message~new(.t~new, 'delayValueReturn')\nsay m~halt\nm~start\nsay m~halt\n\
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
         \x20 return .nil\n  halt:\n  return condition('o')\n",
        Invocation::none().with_switch_mode(SwitchMode::EveryOpportunity),
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
        Invocation::none().with_switch_mode(SwitchMode::EveryOpportunity),
    );
    assert_eq!(stdout(&switching), "a 1\nb 1\na 2\nb 2\na 3\nb 3\ndone\n");
    let unswitched = stressed(
        INTERLEAVED,
        Invocation::none().with_switch_mode(SwitchMode::AtClause(u64::MAX)),
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
        Invocation::none().with_switch_mode(SwitchMode::EveryOpportunity),
    );
    let unvisited = stressed(lone, Invocation::none());
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

/// Two pinned busy-waiters on a flag a third activity sets: the second runs
/// on the first's stack and, pinned in turn, runs the setter on its own; both
/// end, the inner first, with no livelock between their yields.
#[test]
fn two_pinned_busy_waiters_both_end() {
    let source = "g = .flag~new\na = g~start('wait', 'A')\nb = g~start('wait', 'B')\n\
                  g~start('setFlag')\na~wait\nb~wait\nsay 'main ended'\n\
                  ::class flag\n::attribute done unguarded\n::method init\n  expose done\n  \
                  done = 0\n::method wait unguarded\n  use arg tag\n  \
                  interpret \"do while \\self~done; end\"\n  say tag 'ended'\n\
                  ::method setFlag unguarded\n  say 'set'\n  self~done = 1\n";
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

/// Main is not buried by its own root loop: a started activity's pinned
/// yields run it, and where its body ends there, the program ends with its
/// status once the pinned activity has (ruling P30).
#[test]
fn main_ends_inside_a_pinned_yield() {
    let source = "g = .flag~new\ng~start('spin')\ndo while g~started == 0\nend\n\
                  say 'main sets'\ng~done = 1\nsay 'main ends'\nexit 3\n\
                  ::class flag\n::attribute started unguarded\n::attribute done unguarded\n\
                  ::method init\n  expose started done\n  started = 0\n  done = 0\n\
                  ::method spin unguarded\n  self~started = 1\n  \
                  interpret \"do while \\self~done; end\"\n  say 'spin ended'\n";
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
             x = .message~new(.nil, 'x')~replyWith(.array~new)\n  say 'spin after' x~items\n"
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
                "rexx-exec: method \"REPLYWITH\" of class \"Message\" is not implemented \
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
        Invocation::none(),
        Invocation::none().with_switch_mode(SwitchMode::EveryOpportunity),
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
        Invocation::none(),
        Invocation::none().with_switch_mode(SwitchMode::EveryOpportunity),
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

/// An inverted pinned wait is refused naming the kind of its wait under the
/// switch mode too, whose pinned yields run the same activities.
#[test]
fn an_inverted_pinned_wait_is_refused_under_every_opportunity() {
    let outcome = run_with(
        &HIDDEN_INVERSION.replace("$MAIN", "interpret \"say 'main got' m0~result\""),
        Invocation::none().with_switch_mode(SwitchMode::EveryOpportunity),
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
        Invocation::none(),
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
        Invocation::none(),
        64 * 1024 * 1024,
    );
    assert_eq!(outcome.exit_code, 245, "{}", stderr(&outcome));
    assert!(stderr(&outcome).contains("Error 11.1:"));
}
