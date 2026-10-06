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

//! The interpreter's state on another OS thread (decision D-U2, spec
//! 2026-09-29 2.5): a pool thread reaches it only through what the baton's
//! holder lends it, and only while it holds the baton.

#![allow(unsafe_code)]

use std::ptr::NonNull;

use crate::Interp;
use crate::dispatch::library::OffBaton;

/// The baton every interpreter's state is reached under.
pub(crate) type InterpBaton = crate::baton::Baton<Island>;

/// A value of the interpreter's that moves to another OS thread.
pub(crate) struct Islanded<T>(T);

/// What an [`Islanded`] may carry: each payload is listed here, and its
/// argument in the `Send` grant below.
pub(crate) trait IslandPayload {}

impl IslandPayload for NonNull<Interp> {}

impl IslandPayload for (Box<OffBaton>, rexx_api::ffi::ThreadContext) {}

// SAFETY: D-U2. The island is reachable only through one root pointer and the
// baton, and a thread does no refcount operation and no interior access on an
// island value (derives a borrow, clones or drops an `Rc`, reads or writes a
// `Cell`) off the baton. A value made here is made on the baton, moves without
// being touched, and is taken out only by `Islanded::take`, which requires the
// baton; so no two threads ever touch it at once, which is the property its
// `Rc` and `Cell` interior needs. A call run where it is made keeps a
// `ThreadContext` clone across the release, made before the release and
// dropped after the reacquire; a pool thread's clone moves here and is dropped
// under a lend. `Sync` is not granted. Per payload: the root pointer
// `NonNull<Interp>` is dereferenced only by `Lent::interp`, while its lender
// waits; a native call's `(Box<OffBaton>, ThreadContext)`, which holds
// `ObjRef`s and an `Rc`, is taken out by `PooledCall::run` under a lend and
// put back on the activity's record, and its context dropped, under the
// recall's lend.
unsafe impl<T: IslandPayload> Send for Islanded<T> {}

impl<T> Islanded<T> {
    /// `value`, made by the baton's holder.
    pub(crate) fn new(value: T, baton: &InterpBaton) -> Islanded<T> {
        assert!(baton.held_here(), "an island value moved off the baton");
        Islanded(value)
    }

    /// The value, for the baton's holder.
    pub(crate) fn take(self, baton: &InterpBaton) -> T {
        assert!(baton.held_here(), "an island value taken off the baton");
        self.0
    }
}

/// The interpreter's root pointer, which the baton's holder lends with it.
pub(crate) type Island = Islanded<NonNull<Interp>>;

impl Clone for Island {
    fn clone(&self) -> Island {
        *self
    }
}

impl Copy for Island {}

impl Island {
    /// The root pointer of `interp`, which its holder lends while it waits.
    pub(crate) fn of(interp: &mut Interp) -> Island {
        Islanded(NonNull::from(interp))
    }

    /// The interpreter as the API reaches it.
    pub(crate) fn host(self) -> NonNull<dyn rexx_api::values::Host> {
        self.0
    }
}

/// The baton lent to this thread, given back when this drops.
pub(crate) struct Lent<'b> {
    baton: &'b InterpBaton,
    island: Island,
}

impl<'b> Lent<'b> {
    /// Waits until the baton is lent to this thread.
    pub(crate) fn wait(baton: &'b InterpBaton) -> Lent<'b> {
        Lent {
            island: baton.await_lend(),
            baton,
        }
    }

    /// The interpreter, for as long as this lend.
    pub(crate) fn interp(&mut self) -> &mut Interp {
        // SAFETY: the lender derived the pointer from its own `&mut Interp`
        // and waits in `Baton::lend`, touching nothing, until this thread
        // gives the baton back, which only this value's drop does. The answer
        // borrows `self` uniquely, so it ends before that drop and no second
        // borrow is live beside it.
        unsafe { self.island.0.as_mut() }
    }
}

impl Drop for Lent<'_> {
    /// Gives the baton back, except as a panic unwinds: the pool thread's
    /// job posts the panic first, so the lender drains it before it runs a
    /// clause.
    fn drop(&mut self) {
        if !std::thread::panicking() {
            self.baton.give_back();
        }
    }
}

// The interpreter moves to another thread only as an `Islanded` payload, and a
// frame not at all (static_assertions' `assert_not_impl_any`).
trait AmbiguousIfSend<A> {
    fn some_item() {}
}
impl<T: ?Sized> AmbiguousIfSend<()> for T {}
impl<T: ?Sized + Send> AmbiguousIfSend<u8> for T {}
const _: fn() = || {
    let _ = <Interp as AmbiguousIfSend<_>>::some_item;
    let _ = <rexx_core::RegFrame<'static> as AmbiguousIfSend<_>>::some_item;
};

#[cfg(test)]
impl IslandPayload for i32 {}

#[cfg(test)]
mod tests {
    use super::{InterpBaton, Islanded};

    /// An island value is made only by the baton's holder, in every build.
    #[test]
    #[should_panic(expected = "an island value moved off the baton")]
    fn an_island_value_is_made_only_on_the_baton() {
        let baton = InterpBaton::new();
        let _ = Islanded::new(1, &baton);
    }

    /// An island value is taken out only by the baton's holder, in every
    /// build.
    #[test]
    #[should_panic(expected = "an island value taken off the baton")]
    fn an_island_value_is_taken_only_on_the_baton() {
        let baton = InterpBaton::new();
        baton.acquire();
        let value = Islanded::new(1, &baton);
        std::thread::scope(|scope| {
            scope
                .spawn(|| value.take(&baton))
                .join()
                .unwrap_or_else(|payload| std::panic::resume_unwind(payload))
        });
    }
}
