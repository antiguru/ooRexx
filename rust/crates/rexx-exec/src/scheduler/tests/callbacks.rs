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

//! API callbacks from native calls on the interpreter's pool threads and
//! from threads running none (spec 2026-09-29 2.4, section 4), over routines
//! this test and `rexx_api::load` define.

use std::sync::Mutex;
use std::thread::ThreadId;
use std::time::Duration;

use rexx_api::layout::{RexxCallContext_, ValueDescriptor};
use rexx_api::values::{ARGUMENT_TERMINATOR, code};

use super::pool::{Ran, SHAPE, Shape, run_shaped};

/// A routine that answers nothing.
static SIGNATURE: [u16; 2] = [code::REXX_OBJECT_PTR, ARGUMENT_TERMINATOR];

/// The threads [`here`] ran on, each under the test's number.
static THREADS: Mutex<Vec<(usize, ThreadId)>> = Mutex::new(Vec::new());

extern "C-unwind" fn here<const TEST: usize>(
    _context: *mut RexxCallContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return SIGNATURE.as_ptr().cast_mut();
    }
    THREADS
        .lock()
        .expect("unpoisoned")
        .push((TEST, std::thread::current().id()));
    std::ptr::null_mut()
}

/// Sleeps `MILLIS` milliseconds.
extern "C-unwind" fn nap<const MILLIS: u64>(
    _context: *mut RexxCallContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return SIGNATURE.as_ptr().cast_mut();
    }
    std::thread::sleep(Duration::from_millis(MILLIS));
    std::ptr::null_mut()
}

fn library() -> rexx_api::load::Library {
    rexx_api::load::routines_only(&[
        ("SENDTWICE", rexx_api::load::send_twice),
        ("SENDAWAITSEND", rexx_api::load::send_await_send),
        ("SENDKEEPING", rexx_api::load::send_keeping),
        ("SENDTHROUGH", rexx_api::load::send_through),
        (
            "SENDFROMANOTHERTHREAD",
            rexx_api::load::send_from_another_thread,
        ),
        ("HERE1", here::<1>),
        ("HERE2", here::<2>),
        ("HERE3", here::<3>),
        ("NAPLONG", nap::<1500>),
        ("NAPLONGER", nap::<3000>),
    ])
}

/// Runs `source` with this module's routines as the library `callbacktest`.
fn run(source: &str) -> Ran {
    run_shaped(
        source,
        Shape {
            library: (b"callbacktest", library),
            ..SHAPE
        },
    )
    .expect("the run did not panic")
}

/// The threads `HERE` ran on for the test numbered `test`.
fn threads_of(test: usize) -> Vec<ThreadId> {
    THREADS
        .lock()
        .expect("unpoisoned")
        .iter()
        .filter(|(ran_for, _)| *ran_for == test)
        .map(|(_, thread)| *thread)
        .collect()
}

/// A native call on a pool thread calls back from its own thread: each
/// callback runs, on that thread, and the call answers what the last one
/// answered.
#[test]
fn a_callback_from_the_calls_own_thread_runs() {
    let ran = run(
        "m = .t~new~start('idle')\nsay SENDTWICE(.r~new, 'A', 'B')\nsay m~result\n\
         ::requires 'callbacktest' LIBRARY\n::class t\n::method idle\n  call SysSleep 0.2\n  \
         return 'idle'\n::class r\n::method a\n  call HERE1\n  return 'a'\n\
         ::method b\n  return 'b'\n",
    );
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "b\nidle\n");
    assert_eq!(ran.exits, 1);
    let threads = threads_of(1);
    assert_eq!(threads.len(), 1);
    assert_ne!(
        threads[0], ran.thread,
        "the callback ran on the interpreter's thread"
    );
}

/// A callback from a thread running no native call takes the baton, does
/// nothing, and leaves the oracle's 98.983 on the call whose context it
/// used (`Activity::validateThread`, `concurrency/Activity.cpp:3620`). The
/// holder serves it while it runs another activity, which it switches out
/// for the call's own; the baton is held once, for the whole member.
#[test]
fn a_callback_from_another_thread_takes_the_baton_and_raises_98_983() {
    let ran = run("m = .t~new~start('spin')\nsignal on syntax\n\
         say SENDFROMANOTHERTHREAD(.loud~new, 'SPEAK')\nsay 'unreached'\n\
         syntax:\nsay 'trapped' condition('o')~code\nsay m~result\n\
         ::requires 'callbacktest' LIBRARY\n::class t\n::method spin\n  \
         do i = 1 to 2000000\n  end\n  return 'spun'\n\
         ::class loud\n::method speak\n  say 'spoke'\n  return 1\n");
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "trapped 98.983\nspun\n");
    assert_eq!(ran.exits, 1);
    assert_eq!(ran.takes, 1, "the other thread took the baton once");
}

/// A thread context kept from a call on one pool thread, used from another
/// activity's call on another pool thread while the first call runs outside
/// any callback, does nothing and gives the user's own call 98.983.
#[test]
fn a_kept_context_used_from_another_threads_call_raises_98_983_there() {
    let ran = run("k = .keeper~new\nu = .t~new~start('use', k)\n\
         call SENDKEEPING k, 'KEEP'\nsay u~result\n\
         ::requires 'callbacktest' LIBRARY\n\
         ::class keeper\n::method init\n  expose kept\n  kept = .nil\n\
         ::method keep unguarded\n  expose kept\n  use arg kept\n\
         ::method kept unguarded\n  expose kept\n  return kept\n\
         ::class t\n::method use\n  use arg k\n  do while k~kept == .nil\n    call SysSleep 0.01\n  end\n  \
         signal on syntax\n  call SENDTHROUGH k~kept, .loud~new, 'SPEAK'\n  return 'answered'\n\
         syntax:\n  return 'trapped' condition('o')~code\n\
         ::class loud\n::method speak\n  say 'spoke'\n  return 1\n");
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "trapped 98.983\n");
}

/// A callback whose call's activity a pinned waiter waits for is answered by
/// the waiter's nested loop: the waiter's thread lends the baton to the
/// call's thread, which runs the callback.
#[test]
fn a_callback_while_the_holder_is_pinned_is_answered() {
    let ran = run(
        "m = .t~new~start('work')\na = .array~of(2, 1)\na~sortWith(.c~new(m))\n\
         say 'sorted' a~toString('L', ',')\n\
         ::requires 'callbacktest' LIBRARY\n::class t\n::method work\n  \
         return SENDTWICE(.r~new, 'A', 'B')\n\
         ::class r\n::method a\n  call HERE2\n  return 'a'\n::method b\n  return 'b'\n\
         ::class c\n::method init\n  expose m\n  use arg m\n\
         ::method compare\n  expose m\n  use arg l, r\n  say m~result\n  return l - r\n",
    );
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "b\nsorted 1,2\n");
    assert_eq!(ran.exits, 1);
    let threads = threads_of(2);
    assert_eq!(threads.len(), 1);
    assert_ne!(
        threads[0], ran.thread,
        "the callback ran on the interpreter's thread"
    );
}

/// Two activities in native calls on pool threads call back in turn: each
/// call's first callback waits in a `GUARD ON WHEN` until both calls have
/// called back, a pinned wait on the callback's thread, and the second call,
/// entered last, calls back again once the first has returned, so the
/// calls end in the order they were entered. Each callback reaches its own
/// call.
#[test]
fn two_activities_in_native_calls_with_interleaved_callbacks() {
    let path = std::env::temp_dir().join(format!("rexx-interleaved-{}", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let ran = run(&format!(
        "m = .meet~new\na = .t~new~start('first', .side~new(m, 'A'))\n\
         b = .t~new~start('second', .side~new(m, 'B'))\nsay a~result b~result\n\
         ::requires 'callbacktest' LIBRARY\n\
         ::class t\n::method first\n  use arg side\n  r = SENDTWICE(side, 'FIRST', 'SECOND')\n  \
         call lineout '{path}', r\n  call lineout '{path}'\n  return r\n\
         ::method second\n  use arg side\n  return SENDAWAITSEND(side, 'FIRST', '{path}', 'SECOND')\n\
         ::class side\n::method init\n  expose m who\n  use arg m, who\n\
         ::method first\n  expose m who\n  call HERE3\n  m~arrive\n  return who'1'\n\
         ::method second\n  expose who\n  call HERE3\n  return who'2'\n\
         ::class meet\n::method init\n  expose n\n  n = 0\n\
         ::method arrive\n  expose n\n  n += 1\n  guard on when n >= 2\n",
        path = path.display()
    ));
    let written = std::fs::read_to_string(&path).unwrap_or_default();
    let _ = std::fs::remove_file(&path);
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "A2 B2\n");
    assert_eq!(written, "A2\n");
    let threads = threads_of(3);
    assert_eq!(threads.len(), 4);
    assert!(
        !threads.contains(&ran.thread),
        "a callback ran on the interpreter's thread"
    );
}

/// A callback that sends to a guarded method another activity holds waits,
/// pinned, on the callback's thread, whose nested loop runs the holder until
/// it releases the guard.
#[test]
fn a_callback_waits_for_a_contended_guard_on_its_own_thread() {
    let ran = run("g = .g~new\nh = .t~new~start('hold', g)\n\
         do while g~holding == 0\n  call SysSleep 0.01\nend\n\
         say SENDTWICE(g, 'TAKE', 'TAKE')\nsay h~result\n\
         ::requires 'callbacktest' LIBRARY\n\
         ::class t\n::method hold\n  use arg g\n  return g~hold\n\
         ::class g\n::method init\n  expose holding\n  holding = 0\n\
         ::method holding unguarded\n  expose holding\n  return holding\n\
         ::method hold\n  expose holding\n  holding = 1\n  call SysSleep 0.3\n  say 'releases'\n  \
         return 'held'\n::method take\n  say 'taken'\n  return 'took'\n");
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "releases\ntaken\ntaken\ntook\nheld\n");
    assert_eq!(ran.exits, 1);
}

/// A callback is served at the holder's next cold visit even while the
/// holder is pinned and nothing else is ready, so no slice ends its run:
/// a pinned busy-wait sees what the callback did.
#[test]
fn a_callback_is_served_at_a_pinned_holders_next_cold_visit() {
    let ran = run(
        "f = .flag~new\nm = .t~new~start('work', f)\na = .array~of(2, 1)\n\
         a~sortWith(.c~new(f))\nsay 'sorted' a~toString('L', ',') m~result\n\
         ::requires 'callbacktest' LIBRARY\n\
         ::class t\n::method work\n  use arg f\n  return SENDTWICE(f, 'MARK', 'MARK')\n\
         ::class flag\n::method init\n  expose set\n  set = 0\n\
         ::method set unguarded\n  expose set\n  return set\n\
         ::method mark unguarded\n  expose set\n  set = 1\n  return 'marked'\n\
         ::class c\n::method init\n  expose f\n  use arg f\n\
         ::method compare\n  expose f\n  use arg l, r\n  do while f~set == 0\n  end\n  \
         return l - r\n",
    );
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "sorted 1,2 marked\n");
}

/// A call abandoned while it runs on a pool thread (its park ended by an
/// Error 11 that nested notifier failures raise) completes no later call of
/// its activity: the second call returns only once its own native has.
#[test]
fn an_abandoned_calls_completion_does_not_complete_the_next_call() {
    let ran = run_shaped(
        "signal on syntax name abandoned\nb = .bad~new\ndo i = 1 to 3000\n  \
         m = .message~new('abc', 'length')\n  m~notify(b)\n  m~start\nend\n\
         call NAPLONG\nsay 'unreached'\n\
         abandoned:\nsay 'abandoned' condition('o')~code\n\
         signal on syntax name drained\n\
         drained:\ncall SysSleep 0.3\n\
         call time 'R'\ncall NAPLONGER\nsay 'waited' (time('E') >= 2.5)\n\
         ::requires 'callbacktest' LIBRARY\n\
         ::class bad inherit MessageNotification\n::method messageComplete\n  return 1/0\n",
        Shape {
            library: (b"callbacktest", library),
            interpreter_stack: 33 * 1024 * 1024,
            ..SHAPE
        },
    )
    .expect("the run did not panic");
    let stdout = ran.stdout();
    assert!(stdout.starts_with("abandoned 11.1\n"), "{stdout}");
    assert!(stdout.ends_with("waited 1\n"), "{stdout}");
}
