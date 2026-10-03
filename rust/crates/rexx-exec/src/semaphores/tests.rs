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

use crate::{Invocation, Outcome, SwitchMode, run_program, run_program_collect_every_alloc};

/// A bound on every run here: a program end waits for an activity nothing
/// can wake.
const RUN_DEADLINE: std::time::Duration = std::time::Duration::from_secs(60);

const PATH: &str = "/tmp/semaphores.rex";

/// `source` unswitched, collecting at every allocation, and under
/// `EveryOpportunity`.
fn in_modes(source: &str) -> Vec<(&'static str, Outcome)> {
    let plain = || Invocation::none().with_deadline(RUN_DEADLINE);
    let bytes = || source.as_bytes().to_vec();
    vec![
        ("unswitched", run_program(PATH, bytes(), plain())),
        (
            "collect",
            run_program_collect_every_alloc(PATH, bytes(), plain()),
        ),
        (
            "every",
            run_program(
                PATH,
                bytes(),
                plain().with_switch_mode(SwitchMode::EveryOpportunity),
            ),
        ),
    ]
}

fn answers(source: &str, expected: &str) {
    for (mode, outcome) in in_modes(source) {
        assert_eq!(
            outcome.exit_code,
            0,
            "{mode}: {}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&outcome.stdout), expected, "{mode}");
    }
}

/// Each release readies the next activity waiting for the lock, in the order
/// they parked.
#[test]
fn a_release_readies_the_mutex_waiters_in_park_order() {
    answers(
        "m = .MutexSemaphore~new\nsay m~acquire\n\
         a = .w~new~start('take', m, 'a')\nb = .w~new~start('take', m, 'b')\n\
         c = .w~new~start('take', m, 'c')\ncall SysSleep 0.1\nsay m~release\n\
         a~wait\nb~wait\nc~wait\n\
         ::class w\n::method take\n  use arg m, n\n  m~acquire\n  say n\n  m~release\n",
        "1\n1\na\nb\nc\n",
    );
}

/// A post after a timed `Sys*Sem` wait's last poll does not end it: the
/// wait answers 121 and the unit stays for the next wait.
#[test]
fn a_post_after_the_last_poll_is_left_for_the_next_wait() {
    answers(
        "h = SysCreateEventSem()\nm = .w~new~start('wait', h)\n\
         call SysSleep 0.15\nsay SysPostEventSem(h)\nsay m~result\n\
         say SysWaitEventSem(h, 0)\n\
         ::class w\n::method wait\n  use arg h\n  return SysWaitEventSem(h, 150)\n",
        "0\n121\n0\n",
    );
}

/// A semaphore wait ends the chain the deadlock check follows, as the
/// oracle's `checkDeadLock` reads only messages and guard locks: the cycle
/// is not 98.905 but a wait nothing left can end.
#[test]
fn a_semaphore_wait_is_outside_the_deadlock_check() {
    for (mode, outcome) in in_modes(
        "m = .MutexSemaphore~new\nm~acquire\nmsg = .w~new~start('take', m)\n\
         say msg~result\n::class w\n::method take\n  use arg m\n  return m~acquire\n",
    ) {
        let stderr = String::from_utf8_lossy(&outcome.stderr);
        assert!(
            stderr.contains("a wait that nothing left to run can end")
                && !stderr.contains("98.905"),
            "{mode}: {stderr}"
        );
    }
}

/// A mutex an activity holds stays alive though nothing else reaches it:
/// its `UNINIT` runs at termination, after the holder has ended and released
/// it, and not at the holder's end.
#[test]
fn a_held_mutex_is_not_collected() {
    answers(
        "e = .EventSemaphore~new\nheld = .EventSemaphore~new\nmsg = .w~new~start('hold', e, held)\n\
         held~wait\ndo i = 1 to 50\n  x = .array~new(i)\nend\ne~post\nmsg~wait\nsay 'main done'\n\
         ::class w\n::method hold\n  use arg e, held\n  m = .mm~new\n  m~acquire\n  drop m\n  \
         held~post\n  e~wait\n\
         ::class mm subclass MutexSemaphore\n::method uninit\n  say 'uninit'\n  \
         forward class (super)\n",
        "main done\nuninit\n",
    );
}

/// A release readies the waiter without giving it the lock, so the
/// releaser's acquire in the same clause takes it again: the oracle's answer
/// in 18 of 30 runs of this program and 28 of 30 of the two-clause form
/// (ruling P47).
#[test]
fn a_releaser_takes_the_mutex_again_before_its_waiter() {
    answers(
        "m = .MutexSemaphore~new\nsay m~acquire\nmsg = .t~new~start('w', m)\n\
         call SysSleep 0.1\nsay 'main' m~release m~acquire(0)\ncall SysSleep 0.05\n\
         say 'main rel2' m~release\nsay 'w' msg~result\n\
         ::class t\n::method w\n  use arg m\n  r = m~acquire\n  return r m~release\n",
        "1\nmain 1 1\nmain rel2 1\nw 1 1\n",
    );
}

/// A name `sem_open` rejects without creating anything answers the empty
/// string: glibc's `EINVAL` for nothing left once the leading slashes go, or
/// a slash after them.
#[test]
fn a_name_sem_open_rejects_creates_nothing() {
    answers(
        "say '<'SysCreateEventSem('')'>'\nsay '<'SysCreateEventSem('///')'>'\n\
         say '<'SysCreateEventSem('a/b', 1)'>'\nsay '<'SysCreateMutexSem('/')'>'\n\
         say '<'SysCreateMutexSem('/a/b')'>'\n",
        "<>\n<>\n<>\n<>\n<>\n",
    );
}

/// A timed wait a post woke tests the post again, so after a post and a reset
/// in one clause it waits out its timeout, where the untimed waits answer 1.
/// The oracle's timed waiter races the reset for the semaphore's mutex: this
/// answer in 21 of 23 runs of this program, and in 6 of 10 with the post and
/// reset as two clauses (ruling P47).
#[test]
fn a_timed_wait_tests_the_post_again_after_a_pulse() {
    answers(
        "e = .EventSemaphore~new\nt = .t~new\nm1 = t~start('w', e, '')\n\
         m2 = t~start('w', e, 1)\nm3 = t~start('w', e, '')\ncall SysSleep 0.1\n\
         e~~post~reset\nsay 'pulse' m1~result m2~result m3~result e~isPosted\n\
         ::class t\n::method w unguarded\n  use arg e, t\n  if t == '' then return e~wait\n  \
         return e~wait(t)\n",
        "pulse 1 0 1 0\n",
    );
}
