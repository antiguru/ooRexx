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
fn hold(baton: &Baton, inside: &AtomicUsize) {
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
        let baton = Arc::new(Baton::new());
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
        let baton = Arc::new(Baton::new());
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
