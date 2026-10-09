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

//! A wait that fails trappably and a send whose failure is settled after its
//! last holder dropped it, run in the simulation mode: each leaves nothing
//! behind that a later wait, a halt or a collection can reach.

use crate::scheduler::CancelledWaits;
use crate::{Invocation, Outcome, SimConfig, SwitchMode};

/// A run with this thread's test counters read at its end.
struct Counted {
    outcome: Outcome,
    cancelled: CancelledWaits,
    /// Every `set_native_entry` write, then those to a dead handle.
    writes: (u64, u64),
}

impl Counted {
    fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.outcome.stdout).into_owned()
    }

    fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.outcome.stderr).into_owned()
    }
}

/// Runs `source` under the simulation mode `spec` on an interpreter thread
/// of its own.
fn counted(source: &str, spec: &str) -> Counted {
    let config = SimConfig::parse(spec).expect("a sim spec");
    let invocation = Invocation::none()
        .with_deadline(super::RUN_DEADLINE)
        .with_switch_mode(SwitchMode::Sim(config));
    let text = source.as_bytes().to_vec();
    std::thread::Builder::new()
        .stack_size(crate::INTERPRETER_STACK_BYTES)
        .spawn(move || {
            let outcome = crate::execute_on(
                "/tmp/mutants.rex",
                text,
                false,
                invocation,
                Some(crate::INTERPRETER_STACK_BYTES),
            );
            Counted {
                outcome,
                cancelled: crate::scheduler::cancelled_waits(),
                writes: crate::environment::native_entry_writes(),
            }
        })
        .expect("the interpreter thread")
        .join()
        .expect("the run did not panic")
}

/// Asserts `run` ended at rc 0 with `stdout` and nothing on stderr.
fn assert_answers(run: &Counted, spec: &str, stdout: &str) {
    assert_eq!(
        (
            run.outcome.exit_code,
            run.stdout().as_str(),
            run.stderr().as_str()
        ),
        (0, stdout, ""),
        "{spec}"
    );
}

/// The seeded gate's program for a pinned sleep failing under `fail=wait:1`.
const STALE_SLEEPER: &str = include_str!("../../../tests/sim_gate/m11_stale_sleeper.rex");

/// A pinned `SysSleep` failed by `fail=wait:1` is trapped, and main then
/// parks on a started method's result: the failed sleep's deadline does not
/// end that park, so the result is the method's, after its 3 s.
#[test]
fn a_failed_pinned_sleep_leaves_no_deadline_to_end_a_later_wait() {
    for spec in [
        "sim:1,order=fifo,fail=wait:1",
        "sim:1,uniform:1,fail=wait:1",
        "sim:2,uniform:1,fail=wait:1",
    ] {
        let run = counted(STALE_SLEEPER, spec);
        assert_answers(&run, spec, "caught 11.1\nresult slow done after 1\n");
        assert_eq!(
            run.cancelled.sleeps, 1,
            "{spec}: the failed wait was not a sleep"
        );
    }
}

/// A `GUARD WHEN` reached through `INTERPRET` is a pinned wait.
const STALE_WHEN: &str = "call waiter .g~new\nm = .w~new~start('slow')\n\
                          signal on halt name mh\nsay 'result' m~result\nexit\n\
                          mh:\nsay 'main halted'\nexit\n\
                          waiter: procedure\n  use arg o\n  signal on syntax name caught\n  \
                          interpret 'o~waitfor'\n  say 'woke'\n  return\n\
                          caught:\n  say 'caught' condition('O')~code\n  return\n\
                          ::class g\n::method waitfor\n  expose flag\n  flag = 0\n  \
                          guard on when flag\n\
                          ::class w\n::method slow\n  signal on halt name sh\n  \
                          do i = 1 to 400\n    nop\n  end\n  return 'slow done'\n\
                          sh:\n  return 'slow halted'\n";

/// A `GUARD WHEN` failed by `fail=wait:1` is trapped, main then parks on a
/// started method's result, and `halt@K` halts both activities while the
/// method loops: the halt does not end main's park, so main takes its `HALT`
/// once the method has answered. Every spec runs before any is judged. A
/// stale park reason fails `pre:2,k=40` and `uniform:0.3` through the halt
/// alone (`result The NIL object`), and `uniform:1` without it.
#[test]
fn a_failed_guard_when_leaves_no_park_for_a_halt_to_end() {
    let mut failures = Vec::new();
    for spec in [
        "sim:2,pre:2,k=40,fail=wait:1,halt@20",
        "sim:3,uniform:0.3,fail=wait:1,halt@20",
        "sim:1,order=fifo,fail=wait:1,halt@20",
        "sim:1,uniform:1,fail=wait:1,halt@20",
        "sim:2,uniform:1,fail=wait:1,halt@60",
    ] {
        let run = counted(STALE_WHEN, spec);
        let answered = (
            run.outcome.exit_code,
            run.stdout(),
            run.stderr(),
            run.cancelled.when_parks,
        );
        let expected = (
            0,
            "caught 11.1\nresult slow halted\nmain halted\n".to_owned(),
            String::new(),
            1,
        );
        if answered != expected {
            failures.push(format!("{spec}: {answered:?}"));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

/// `hold` keeps `o`'s guard through a 2 s sleep while main's `touch`,
/// reached through `INTERPRET`, waits for the lock.
const STALE_GUARD_WAITER: &str = "o = .g~new\nh = o~start('hold')\ncall SysSleep 0.5\n\
                                  call toucher o\nh~wait\n\
                                  say 'later' o~start('touch')~result\nexit\n\
                                  toucher: procedure\n  use arg o\n  \
                                  signal on syntax name caught\n  interpret 'say o~touch'\n  \
                                  return\ncaught:\n  say 'caught' condition('O')~code\n  \
                                  return\n\
                                  ::class g\n::method hold\n  call SysSleep 2\n  say 'held'\n  \
                                  return\n::method touch\n  return 'touched'\n";

/// A guard-lock wait failed by `fail=wait:1` while another activity holds
/// the guard is trapped: the holder's release grants the lock to no one, so
/// a later send from a third activity gets it.
#[test]
fn a_failed_guard_lock_wait_leaves_no_place_in_the_queue() {
    for spec in [
        "sim:1,order=fifo,fail=wait:1",
        "sim:1,uniform:1,fail=wait:1",
        "sim:2,pre:2,k=40,fail=wait:1",
    ] {
        let run = counted(STALE_GUARD_WAITER, spec);
        assert_answers(&run, spec, "caught 11.1\nheld\nlater touched\n");
        assert_eq!(
            run.cancelled.guard_waits, 1,
            "{spec}: the failed wait was not a guard lock's"
        );
    }
}

/// A started message main drops at once, whose method `body` runs.
fn dropped_send(body: &str) -> String {
    format!(".t~new~start('boom')\nsay 'main'\n::class t\n::method boom\n  {body}\n")
}

/// A started send main dropped at once fails after main has ended; the
/// condition object is written to the message while the message is alive,
/// under collections at every allocation and at a tenth of them.
#[test]
fn a_dropped_failed_send_is_written_while_alive() {
    for seed in 1..=3 {
        for q in ["1", "0.1"] {
            let spec = format!("sim:{seed},gc={q}");
            let failed = counted(&dropped_send("x = 1 / 0"), &spec);
            let ended = counted(&dropped_send("x = 1"), &spec);
            assert_eq!(failed.stdout(), "main\n", "{spec}");
            assert!(
                failed.stderr().contains("Error 42.3:"),
                "{spec}: {}",
                failed.stderr()
            );
            assert_eq!(
                failed.writes.0,
                ended.writes.0 + 1,
                "{spec}: the failed send's condition was not written"
            );
            assert_eq!(failed.writes.1, 0, "{spec}: a write to a dead handle");
        }
    }
}
