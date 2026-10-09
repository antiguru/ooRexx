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

//! Native calls that leave their driver (spec 2026-09-29 P6-3, 2.6), over
//! the test libraries in the worktree's `build/lib`.

use crate::{Interp, Invocation, Outcome, SwitchMode};

const RUN_DEADLINE: std::time::Duration = std::time::Duration::from_secs(60);

/// A run of a program, with the driver exits it made and the times a
/// callback took the baton.
struct Counted {
    outcome: Outcome,
    exits: u64,
    takes: u64,
    /// The pool threads the run spawned.
    spawned: u64,
}

impl Counted {
    fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.outcome.stdout).into_owned()
    }

    fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.outcome.stderr).into_owned()
    }
}

/// Runs `source` on an interpreter thread of its own, collecting at every
/// allocation where `stress`, with the test libraries on its library path.
fn counted(source: &str, stress: bool, invocation: Invocation) -> Counted {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../build/lib")
        .canonicalize()
        .expect("the worktree's build/lib is three directories above this crate");
    let invocation = invocation.with_environment(vec![(
        b"LD_LIBRARY_PATH".to_vec(),
        directory.into_os_string().into_encoded_bytes(),
    )]);
    let text = source.as_bytes().to_vec();
    std::thread::Builder::new()
        .stack_size(crate::INTERPRETER_STACK_BYTES)
        .spawn(move || {
            let outcome = crate::execute_on(
                "/tmp/native.rex",
                text,
                stress,
                invocation,
                Some(crate::INTERPRETER_STACK_BYTES),
            );
            Counted {
                outcome,
                exits: crate::scheduler::native_exits(),
                takes: crate::dispatch::library::callback_takes(),
                spawned: crate::scheduler::threads_spawned(),
            }
        })
        .expect("the interpreter thread")
        .join()
        .expect("the run did not panic")
}

fn bounded() -> Invocation {
    Invocation::none().with_deadline(RUN_DEADLINE)
}

fn switched() -> Invocation {
    bounded().with_switch_mode(SwitchMode::EveryOpportunity)
}

/// A started activity and main each call a routine while the other is
/// alive: each call leaves its driver and its activity finishes it.
#[test]
fn a_native_call_where_another_activity_lives_leaves_its_driver() {
    let run = counted(
        "m = .t~new~start('work')\nsay TestIntArg(5)\nsay m~result\n\
         ::requires 'orxfunction' LIBRARY\n::class t\n::method work\n  return TestIntArg(7)\n",
        false,
        bounded(),
    );
    assert_eq!(run.outcome.exit_code, 0, "{}", run.stderr());
    assert_eq!(run.stdout(), "5\n7\n");
    assert_eq!(run.exits, 2);
}

/// A lone activity's call runs on the baton, with no driver exit
/// (ruling P43).
#[test]
fn a_lone_activitys_native_call_keeps_its_driver() {
    let run = counted(
        "say TestIntArg(5)\n::requires 'orxfunction' LIBRARY\n",
        false,
        bounded(),
    );
    assert_eq!(run.outcome.exit_code, 0, "{}", run.stderr());
    assert_eq!(run.stdout(), "5\n");
    assert_eq!(run.exits, 0);
}

/// Collecting at every allocation across driver exits whose calls build
/// their answers during the call (`RxCalcSqrt` calls back
/// `DoubleToObjectWithPrecision`): each answer is reached through the call's
/// frame after the collections at the exit and before `finish`.
#[test]
fn collect_stress_across_a_native_calls_driver_exit() {
    let source = "m = .t~new~start('work')\nsay RxCalcSqrt(16)\nsay m~result\n\
                  ::requires 'rxmath' LIBRARY\n::class t\n::method work\n  \
                  return RxCalcSqrt(2, 5)\n";
    let plain = counted(source, false, bounded());
    let stressed = counted(source, true, bounded());
    for run in [&plain, &stressed] {
        assert_eq!(run.outcome.exit_code, 0, "{}", run.stderr());
        assert_eq!(run.stdout(), "4\n1.4142\n");
        assert_eq!(run.exits, 2);
        assert!(run.takes >= 2, "{} callback takes", run.takes);
    }
    assert!(
        stressed.outcome.collections > plain.outcome.collections,
        "{} collections stressed, {} not",
        stressed.outcome.collections,
        plain.outcome.collections
    );
}

/// A callback during a call that runs with the baton released takes the
/// baton, here to run a Rexx method, under collect stress; the same call
/// made under `INTERPRET`, which pins the activity, keeps the baton and
/// takes nothing.
#[test]
fn a_callback_during_an_off_baton_call_takes_the_baton() {
    const CLASSES: &str = "::requires 'orxfunction' LIBRARY\n::class t\n::method work\n  \
                           return TestIntArg(7)\n::class base\n::method who\n  return 'base'\n\
                           ::class sub subclass base\n::method who\n  return 'sub'\n\
                           ::class k\n\
                           ::method scoped external \"LIBRARY orxmethod TestSendMessageScoped\"\n";
    let off = counted(
        &format!(
            "m = .t~new~start('work')\n\
             say .k~new~scoped(.sub~new, 'who', .base, .array~new)\nsay m~result\n{CLASSES}"
        ),
        true,
        bounded(),
    );
    assert_eq!(off.outcome.exit_code, 0, "{}", off.stderr());
    assert_eq!(off.stdout(), "base\n7\n");
    assert_eq!(off.exits, 2);
    assert!(off.takes >= 1, "{} callback takes", off.takes);
    let pinned = counted(
        &format!("interpret \"say .k~new~scoped(.sub~new, 'who', .base, .array~new)\"\n{CLASSES}"),
        true,
        switched(),
    );
    assert_eq!(pinned.outcome.exit_code, 0, "{}", pinned.stderr());
    assert_eq!(pinned.stdout(), "base\n");
    assert_eq!((pinned.exits, pinned.takes), (0, 0));
}

/// A pinned waiter whose wait an activity returning from a native call
/// satisfies: the nested loop drains the call's completion and runs the
/// activity's `finish`, rather than refusing the wait.
#[test]
fn a_pinned_wait_is_satisfied_by_an_activity_returning_from_a_native_call() {
    let source = "m = .t~new~start('work')\na = .array~of(2, 1)\na~sortWith(.c~new(m))\n\
                  say 'sorted' a~toString('L', ',')\n\
                  ::requires 'orxfunction' LIBRARY\n::class t\n::method work\n  \
                  return TestIntArg(7)\n\
                  ::class c\n::method init\n  expose m\n  use arg m\n\
                  ::method compare\n  expose m\n  use arg l, r\n  say m~result\n  return l - r\n";
    for invocation in [bounded(), switched()] {
        let run = counted(source, false, invocation);
        assert_eq!(run.outcome.exit_code, 0, "{}", run.stderr());
        assert_eq!(run.stdout(), "7\nsorted 1,2\n");
        assert_eq!(run.exits, 1);
    }
}

/// A wait nothing can end, made by Rexx code a callback runs during an
/// off-baton call, is refused as it is where the call keeps the baton: the
/// call under the callback cannot complete while the loop above it waits.
#[test]
fn a_callbacks_wait_on_a_call_below_it_is_refused() {
    let source = "m = .message~new('abc', 'length')\n\
                  say .k~new~send0(.w~new(m), 'wait')\nsay 'after'\n\
                  ::class w\n::method init\n  expose m\n  use arg m\n\
                  ::method wait\n  expose m\n  return m~result\n\
                  ::class k\n::method send0 external \"LIBRARY orxmethod TestSendMessage0\"\n";
    let deadline = std::time::Duration::from_secs(5);
    let kept = counted(source, false, Invocation::none().with_deadline(deadline));
    let left = counted(
        source,
        false,
        Invocation::none()
            .with_deadline(deadline)
            .with_switch_mode(SwitchMode::EveryOpportunity),
    );
    assert_eq!((kept.exits, left.exits), (0, 1));
    for run in [&kept, &left] {
        assert_eq!(run.outcome.exit_code, 120);
        assert_eq!(run.stdout(), "");
        assert_eq!(
            run.stderr(),
            "rexx-exec: a wait that nothing left to run can end is not implemented\n"
        );
    }
}

/// A function call whose routine left its driver and answered nothing is
/// 44.1 under the call's spelling, as where the call kept the baton.
#[test]
fn a_function_whose_call_left_its_driver_and_answered_nothing_is_44_1() {
    let source = "say TestAddCommandEnvironment('T17', 'DIRECT')\n\
                  ::requires 'orxfunction' LIBRARY\n";
    let kept = counted(source, false, bounded());
    let left = counted(source, false, switched());
    assert_eq!((kept.exits, left.exits), (0, 1));
    for run in [&kept, &left] {
        assert_eq!(run.outcome.exit_code, 212);
        assert!(
            run.stderr().ends_with(
                "Error 44.1:  No data returned from function \"TESTADDCOMMANDENVIRONMENT\".\n"
            ),
            "{}",
            run.stderr()
        );
    }
    assert_eq!(kept.outcome.stderr, left.outcome.stderr);
}

/// The timer is armed while a native call is in flight, with no other
/// activity ready (spec 2026-09-29 section 4).
#[test]
fn the_timer_is_armed_for_a_call_in_flight() {
    let mut interp = Interp::new();
    interp.serve_requests(false).expect("no request fails");
    assert!(!interp.timer.armed_in_registry());
    interp.activities.in_flight = 1;
    interp.serve_requests(false).expect("no request fails");
    assert!(interp.timer.armed_in_registry());
    interp.activities.in_flight = 0;
    interp.serve_requests(false).expect("no request fails");
    assert!(!interp.timer.armed_in_registry());
}

/// The timer arms no slice in the simulation mode, even with a call in
/// flight.
#[test]
fn the_timer_arms_no_slice_in_the_simulation_mode() {
    let mut interp = Interp::new();
    let config = crate::SimConfig::parse("sim:1").expect("a sim spec");
    interp.set_switch_mode(SwitchMode::Sim(config));
    interp.activities.in_flight = 1;
    interp.serve_requests(false).expect("no request fails");
    assert!(!interp.timer.armed_in_registry());
    interp.activities.in_flight = 0;
}

/// In the simulation mode a native call that leaves its driver runs on the
/// baton: the pool spawns no thread, where it spawns one under the switch
/// mode, and the program prints the same.
#[test]
fn a_native_call_in_the_simulation_mode_spawns_no_pool_thread() {
    let source = "m = .t~new~start('work')\nsay TestIntArg(5)\nsay m~result\n\
                  ::requires 'orxfunction' LIBRARY\n::class t\n::method work\n  return TestIntArg(7)\n";
    let config = crate::SimConfig::parse("sim:1").expect("a sim spec");
    let switched_run = counted(source, false, switched());
    let sim_run = counted(
        source,
        false,
        bounded().with_switch_mode(SwitchMode::Sim(config)),
    );
    for run in [&switched_run, &sim_run] {
        assert_eq!(run.outcome.exit_code, 0, "{}", run.stderr());
        assert_eq!(run.stdout(), "5\n7\n");
        assert_eq!(run.exits, 2);
    }
    assert!(switched_run.spawned > 0);
    assert_eq!(sim_run.spawned, 0);
}

/// A server activity blocks in `SockAccept`, which main's later connect
/// ends. In the simulation mode the accept runs on the baton, main never
/// connects, and the call is interrupted and refused after `block=`.
#[test]
fn a_native_call_only_another_activity_can_end_is_refused_after_its_bound() {
    let port = 40_000 + std::process::id() % 10_000;
    let source = format!(
        "s = SockSocket('AF_INET', 'SOCK_STREAM', 0)\n\
         addr.family = 'AF_INET'; addr.port = {port}; addr.addr = '127.0.0.1'\n\
         call SockSetSockOpt s, 'SOL_SOCKET', 'SO_REUSEADDR', 1\n\
         say 'bind' SockBind(s, 'addr.') 'listen' SockListen(s, 1)\n\
         t = .srv~new~start('ACCEPT', s)\ncall SysSleep 0.1\n\
         c = SockSocket('AF_INET', 'SOCK_STREAM', 0)\nsay 'connect' SockConnect(c, 'addr.')\n\
         say 'got' t~result\ncall SockClose c; call SockClose s\n\
         ::routine SockSocket public external \"LIBRARY rxsock SockSocket\"\n\
         ::routine SockSetSockOpt public external \"LIBRARY rxsock SockSetSockOpt\"\n\
         ::routine SockBind public external \"LIBRARY rxsock SockBind\"\n\
         ::routine SockListen public external \"LIBRARY rxsock SockListen\"\n\
         ::routine SockAccept public external \"LIBRARY rxsock SockAccept\"\n\
         ::routine SockConnect public external \"LIBRARY rxsock SockConnect\"\n\
         ::routine SockClose public external \"LIBRARY rxsock SockClose\"\n\
         ::class srv\n::method accept\n  use arg s\n  a = SockAccept(s)\n  return a > 0\n"
    );
    let real = counted(&source, false, bounded());
    assert_eq!(real.outcome.exit_code, 0, "{}", real.stderr());
    assert_eq!(real.stdout(), "bind 0 listen 0\nconnect 0\ngot 1\n");
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let config = crate::SimConfig::parse("sim:1,block=0.5").expect("a sim spec");
        let began = std::time::Instant::now();
        let run = counted(
            &source,
            false,
            bounded().with_switch_mode(SwitchMode::Sim(config)),
        );
        let _ = sender.send((run, began.elapsed()));
    });
    let (run, took) = receiver
        .recv_timeout(std::time::Duration::from_secs(20))
        .expect("the run ended");
    assert_eq!(run.outcome.exit_code, 120);
    assert_eq!(run.stdout(), "bind 0 listen 0\n");
    assert_eq!(
        run.stderr(),
        "rexx-exec: a native call in the simulation mode that runs longer than its bound is \
         not implemented\n"
    );
    assert!(
        took >= std::time::Duration::from_millis(500),
        "took {took:?}"
    );
}

/// A guarded library method whose send waited for its guard runs once the
/// guard is granted, from the wait's continuation, which is outside every
/// driver: the call keeps the baton there.
#[test]
fn a_guarded_library_method_runs_after_its_guard_wait() {
    let source = "o = .k~new\nm = o~start('hold')\ncall SysSleep 0.05\nsay o~lib(5)\n\
                  say m~result\n::class k\n::method hold\n  call SysSleep 0.2\n  return 'held'\n\
                  ::method lib external \"LIBRARY orxmethod TestIntArg\"\n";
    for invocation in [bounded(), switched()] {
        let run = counted(source, false, invocation);
        assert_eq!(run.outcome.exit_code, 0, "{}", run.stderr());
        assert_eq!(run.stdout(), "5\nheld\n");
        assert_eq!(run.exits, 0);
    }
}

/// A library routine called again, by another spelling and by `CALL`,
/// reaches the row its name finds; the cached row is checked against the
/// name at every call in a debug build.
#[test]
fn a_library_routine_called_again_reaches_the_row_its_name_finds() {
    let run = counted(
        "say TestIntArg(1)\nsay testintarg(2)\ncall TestIntArg 3\nsay result\n\
         say TestNameArg()\nsay TestIntArg(4)\n::requires 'orxfunction' LIBRARY\n",
        false,
        bounded(),
    );
    assert_eq!(run.outcome.exit_code, 0, "{}", run.stderr());
    assert_eq!(run.stdout(), "1\n2\n3\nTESTNAMEARG\n4\n");
}

/// A started activity's loud refusal, met while main's call has left its
/// driver and waits to finish, ends main's park in place of the completion:
/// the call is abandoned, its frame popped, and the run ends with the
/// refusal. The empty stdout is that refusal's (P40), not a schedule the
/// oracle takes: measured over 5 runs, the oracle answers the entry and
/// prints `5` and `after` in every one.
#[test]
fn a_failure_ending_a_native_park_abandons_the_call() {
    let run = counted(
        "o = .t~new\nm = o~start('boom')\nsay TestIntArg(5)\nsay 'after'\n\
         ::requires 'orxfunction' LIBRARY\n::class t\n::method boom\n  \
         x = .local['STDQUE']\n",
        false,
        bounded(),
    );
    assert_eq!(run.outcome.exit_code, 120);
    assert_eq!(run.stdout(), "");
    assert_eq!(
        run.stderr(),
        "rexx-exec: directory entry \"STDQUE\" is not implemented (Phase 10)\n"
    );
    assert_eq!(run.exits, 1);
}
