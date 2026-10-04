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

//! `loom` models of the baton, the inbox and the timer (spec 2026-09-29 R4),
//! compiled from the interpreter's own sources against `loom`'s primitives.
//! Empty unless built with `RUSTFLAGS="--cfg loom"`.

#![cfg(loom)]

#[path = "../src/baton.rs"]
mod baton;
#[path = "../src/sync.rs"]
mod sync;
#[path = "../src/timer.rs"]
mod timer;

use std::time::Duration;

use loom::sync::Arc;
use loom::sync::atomic::{AtomicUsize, Ordering};
use loom::thread;

use baton::Baton;
use timer::{INBOX, Inbox, Registration, SLICE};

/// Runs `body` as the baton's holder, failing if another thread is inside.
fn hold(baton: &Baton<u32>, inside: &AtomicUsize) {
    assert!(baton.held_here());
    assert_eq!(
        inside.fetch_add(1, Ordering::SeqCst),
        0,
        "two threads hold the baton"
    );
    inside.fetch_sub(1, Ordering::SeqCst);
}

/// The holder releases while a pool thread waits in `acquire`: the pool
/// thread takes the baton, never while the holder still has it.
#[test]
fn the_baton_passes_to_a_waiting_thread() {
    loom::model(|| {
        let baton = Arc::new(Baton::<u32>::new());
        let inside = Arc::new(AtomicUsize::new(0));
        baton.acquire();
        let pool = {
            let (baton, inside) = (Arc::clone(&baton), Arc::clone(&inside));
            thread::spawn(move || {
                baton.acquire();
                hold(&baton, &inside);
                baton.release();
            })
        };
        hold(&baton, &inside);
        baton.release();
        assert!(!baton.held_here());
        pool.join().expect("the pool thread");
    });
}

/// Two pool threads wait: each release passes the baton to one of them.
#[test]
fn the_baton_passes_to_one_waiting_thread_at_a_time() {
    loom::model(|| {
        let baton = Arc::new(Baton::<u32>::new());
        let inside = Arc::new(AtomicUsize::new(0));
        baton.acquire();
        let pool: Vec<_> = (0..2)
            .map(|_| {
                let (baton, inside) = (Arc::clone(&baton), Arc::clone(&inside));
                thread::spawn(move || {
                    baton.acquire();
                    hold(&baton, &inside);
                    baton.release();
                })
            })
            .collect();
        baton.release();
        for thread in pool {
            thread.join().expect("a pool thread");
        }
    });
}

/// Completions posted while the holder drains and idles reach it once
/// each, in order, and leave `INBOX` clear once all are taken.
#[test]
fn completions_posted_while_the_holder_drains_arrive_once_in_order() {
    loom::model(|| {
        let inbox = Arc::new(Inbox::<u32>::new());
        let poster = {
            let inbox = Arc::clone(&inbox);
            thread::spawn(move || {
                inbox.post(1);
                inbox.post(2);
            })
        };
        let mut taken = Vec::new();
        while taken.len() < 2 {
            taken.extend(inbox.drain());
            if taken.len() < 2 {
                taken.extend(inbox.idle());
            }
        }
        poster.join().expect("the poster");
        assert_eq!(taken, [1, 2]);
        assert!(!inbox.requests().pending(INBOX));
    });
}

/// The timer's `SLICE` for an armed interpreter whose holder idles on a
/// deadline is pending when the holder wakes.
#[test]
fn the_timer_sets_slice_while_the_holder_idles() {
    loom::model(|| {
        let mut registration = Registration::new();
        registration.arm();
        assert!(registration.armed_in_registry());
        sync::advance(Duration::from_millis(25));
        let posted = registration.idle_until(sync::now());
        assert!(posted.is_empty());
        assert!(registration.requests().pending(SLICE));
        registration.disarm();
        assert!(!registration.armed_in_registry());
        drop(registration);
        timer::join_timer();
    });
}

/// An idle whose deadline the timer finds due ends, whether the timer's
/// wake lands before or after the holder blocks; twice in a row, and with
/// the timer started anew after the registry emptied.
#[test]
fn an_idle_deadline_is_never_lost() {
    loom::model(|| {
        let registration = Registration::new();
        let posted = registration.idle_until(sync::now());
        assert!(posted.is_empty());
        let posted = registration.idle_until(sync::now());
        assert!(posted.is_empty());
        drop(registration);
        timer::join_timer();
        let registration = Registration::new();
        let posted = registration.idle_until(sync::now());
        assert!(posted.is_empty());
        drop(registration);
        timer::join_timer();
    });
}

/// One interpreter's registration leaves, so the timer thread may decide to
/// end, while another registers and idles: the idle still ends.
#[test]
fn a_sleeper_registers_as_the_timer_exits() {
    loom::model(|| {
        let first = Registration::new();
        assert!(first.idle_until(sync::now()).is_empty());
        let other = thread::spawn(|| {
            let second = Registration::new();
            assert!(second.idle_until(sync::now()).is_empty());
        });
        drop(first);
        other.join().expect("the other interpreter");
        timer::join_timer();
    });
}

/// Touches interpreter state, failing if another thread is touching it.
fn touch(inside: &AtomicUsize) {
    assert_eq!(
        inside.fetch_add(1, Ordering::SeqCst),
        0,
        "two threads touch interpreter state"
    );
    inside.fetch_sub(1, Ordering::SeqCst);
}

/// A native call runs with the baton released while another thread takes
/// it; a callback the call makes takes the baton before touching
/// interpreter state and gives it back, and the call's thread posts its
/// completion and takes the baton back. No two threads touch that state at
/// once.
#[test]
fn a_callback_during_an_off_baton_call_takes_the_baton_first() {
    loom::model(|| {
        let baton = Arc::new(Baton::<u32>::new());
        let inside = Arc::new(AtomicUsize::new(0));
        let registration = Registration::new();
        let inbox = registration.inbox();
        baton.acquire();
        let other = {
            let (baton, inside) = (Arc::clone(&baton), Arc::clone(&inside));
            thread::spawn(move || {
                baton.acquire();
                touch(&inside);
                baton.release();
            })
        };
        baton.release();
        let took = baton.take_unless_held();
        touch(&inside);
        if took {
            baton.release();
        }
        inbox.post(1);
        baton.acquire();
        touch(&inside);
        baton.release();
        other.join().expect("the other thread");
        assert_eq!(Vec::from(registration.drain()), [1]);
    });
}

/// A loop with a native call in flight and nothing ready blocks on the
/// inbox with no deadline, and the completion the call's thread posts,
/// before or after the loop blocks, ends the wait.
#[test]
fn a_loop_with_a_call_in_flight_waits_for_its_completion() {
    loom::model(|| {
        let registration = Registration::new();
        let caller = {
            let inbox = registration.inbox();
            thread::spawn(move || inbox.post(7))
        };
        let in_flight = 1;
        let mut taken = Vec::from(registration.drain());
        while taken.len() < in_flight {
            taken.extend(registration.idle());
        }
        caller.join().expect("the call's thread");
        assert_eq!(taken, [7]);
        assert!(!registration.requests().pending(INBOX));
    });
}

/// The holder lends the baton to a pool thread and waits; the pool thread
/// touches interpreter state with what was lent and gives it back, and a
/// third thread taking the baton gets it only once the holder releases it.
/// No two threads touch that state at once.
#[test]
fn a_lent_baton_comes_back_before_its_lender_runs() {
    loom::model(|| {
        let baton = Arc::new(Baton::<u32>::new());
        let inside = Arc::new(AtomicUsize::new(0));
        baton.acquire();
        let pool = {
            let (baton, inside) = (Arc::clone(&baton), Arc::clone(&inside));
            thread::spawn(move || {
                assert_eq!(baton.await_lend(), 5);
                assert!(baton.held_here());
                assert_eq!(baton.lent(), Some(5));
                touch(&inside);
                baton.give_back();
                assert!(!baton.held_here());
            })
        };
        let other = {
            let (baton, inside) = (Arc::clone(&baton), Arc::clone(&inside));
            thread::spawn(move || {
                assert!(baton.take_unless_held());
                touch(&inside);
                baton.release();
            })
        };
        baton.lend(pool.thread().id(), 5);
        assert!(baton.held_here());
        assert_eq!(baton.lent(), None);
        touch(&inside);
        baton.release();
        pool.join().expect("the pool thread");
        other.join().expect("the other thread");
    });
}

/// A pool thread recalls the baton through the inbox while the holder idles
/// there; the holder lends it, and the pool thread's lend ends before the
/// holder touches interpreter state again.
#[test]
fn a_recall_posted_to_an_idle_holder_is_lent_the_baton() {
    loom::model(|| {
        let baton = Arc::new(Baton::<u32>::new());
        let inside = Arc::new(AtomicUsize::new(0));
        let registration = Registration::new();
        baton.acquire();
        let pool = {
            let (baton, inside, inbox) = (
                Arc::clone(&baton),
                Arc::clone(&inside),
                registration.inbox(),
            );
            thread::spawn(move || {
                inbox.post(1);
                baton.await_lend();
                touch(&inside);
                baton.give_back();
            })
        };
        let mut posted = Vec::from(registration.drain());
        while posted.is_empty() {
            posted.extend(registration.idle());
        }
        assert_eq!(posted, [1]);
        baton.lend(pool.thread().id(), 0);
        touch(&inside);
        baton.release();
        pool.join().expect("the pool thread");
    });
}
