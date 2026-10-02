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

//! The timers `.Alarm` and `.Ticker` drive (`platform/unix/TimeSupport.cpp`).
//! Each waits on a timer of the scheduler's, whose number the
//! `EVENTSEMHANDLE` pointer holds, as a park that a post ends early.

use std::time::{Duration, Instant};

use rexx_api::values::MAX_WHOLENUMBER;
use rexx_core::{BehaviourId, Body, Decoded, NativeState, ObjRef};
use rexx_num::{DIGITS64, Number};

use super::{Cleared, NativeResume, NativeStarted};
use crate::error::Raised;
use crate::scheduler::{ParkReason, TIMER_DAY, TimerId};
use crate::{Failure, Interp};

/// `alarm_startTimer(numdays, alarmtime)`: sets `EVENTSEMHANDLE` and
/// `TIMERSTARTED`, then waits the days and milliseconds or until a cancel.
pub(super) fn alarm_start_timer(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    scope: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let days = whole_number_argument(interp, args, 0)?;
    let millis = whole_number_argument(interp, args, 1)?;
    let id = interp.create_timer();
    let handle = handle_object(interp, id);
    set_object_variable(interp, receiver, scope, b"EVENTSEMHANDLE", handle)?;
    let started = interp.counted(1);
    set_object_variable(interp, receiver, scope, b"TIMERSTARTED", started)?;
    timed_wait(interp, id, handle, days, millis, alarm_woken)
}

/// An alarm's wait is over, whichever way: its timer ends with the call.
fn alarm_woken(interp: &mut Interp, handle: ObjRef) -> Result<Option<ObjRef>, Failure> {
    if let Some(id) = timer_of(interp, handle) {
        interp.end_timer_wait(id);
        interp.remove_timer(id);
    }
    Ok(Some(interp.counted(0)))
}

/// `alarm_stopTimer(eventSemHandle)` and `ticker_stopTimer(eventSemHandle)`:
/// posts the timer.
pub(super) fn stop_timer(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    scope: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let id = pointer_argument(interp, args, 0)?;
    let cancelled = is_cancelled(interp, receiver, scope)?;
    if let Some(id) = id {
        interp.post_timer(id, cancelled);
    }
    Ok(NativeStarted::Ran(Some(interp.counted(0))))
}

/// `ticker_createTimer()`: a timer for the ticker's waits, set as
/// `EVENTSEMHANDLE`.
pub(super) fn ticker_create_timer(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    scope: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let id = interp.create_timer();
    let handle = handle_object(interp, id);
    set_object_variable(interp, receiver, scope, b"EVENTSEMHANDLE", handle)?;
    Ok(NativeStarted::Ran(Some(interp.counted(0))))
}

/// `ticker_waitTimer(eventSemHandle, numdays, alarmtime)`: one interval's
/// wait. A post that came before it ends it at once as a cancel, and
/// otherwise where its whole days are none; where they are some, the post is
/// reset and ends the first day.
pub(super) fn ticker_wait_timer(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    scope: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let id = pointer_argument(interp, args, 0)?;
    let mut days = whole_number_argument(interp, args, 1)?;
    let millis = whole_number_argument(interp, args, 2)?;
    let done = interp.counted(0);
    let Some(id) = id else {
        return Ok(NativeStarted::Ran(Some(done)));
    };
    let Some(handle) = args.first().copied().flatten() else {
        unreachable!("pointer_argument answered a timer")
    };
    match interp.timer_posted(id) {
        None => return Ok(NativeStarted::Ran(Some(done))),
        Some(false) => {}
        Some(true) => {
            if is_cancelled(interp, receiver, scope)? {
                interp.remove_timer(id);
                return Ok(NativeStarted::Ran(Some(done)));
            }
            if days <= 0 {
                return Ok(NativeStarted::Ran(Some(done)));
            }
            interp.reset_timer(id);
            days -= 1;
        }
    }
    timed_wait(interp, id, handle, days, millis, ticker_woken)
}

/// A ticker's wait is over: a cancel ends its timer.
fn ticker_woken(interp: &mut Interp, handle: ObjRef) -> Result<Option<ObjRef>, Failure> {
    if let Some(id) = timer_of(interp, handle) {
        let (posted, cancelled) = interp.end_timer_wait(id);
        if posted && cancelled {
            interp.remove_timer(id);
        }
    }
    Ok(Some(interp.counted(0)))
}

/// Parks the running activity on `id` for `days` whole days and `millis`
/// milliseconds, with `woken` as the native's answer after.
fn timed_wait(
    interp: &mut Interp,
    id: TimerId,
    handle: ObjRef,
    days: i64,
    millis: i64,
    woken: NativeResume,
) -> Result<NativeStarted, Failure> {
    park_point!(interp, crate::pinning::ParkKind::Timer);
    let now = Instant::now();
    let whole_days = TIMER_DAY.saturating_mul(u32::try_from(days.max(0)).unwrap_or(u32::MAX));
    let remainder = Duration::from_millis(u64::try_from(millis).unwrap_or(0));
    let days_end = later(now, whole_days);
    let deadline = later(days_end, remainder);
    interp.begin_timer_wait(id, days_end);
    interp.park_native(
        ParkReason::Timer {
            deadline,
            cancel: id,
        },
        woken,
        handle,
    )
}

/// `from` plus `span`, or as far ahead as an `Instant` reaches.
fn later(from: Instant, span: Duration) -> Instant {
    from.checked_add(span)
        .unwrap_or_else(|| from + Duration::from_secs(u64::from(u32::MAX)))
}

/// A `.Pointer` holding `id`.
fn handle_object(interp: &mut Interp, id: TimerId) -> ObjRef {
    let class = interp
        .classes()
        .lookup("Pointer")
        .expect("Pointer is a native class");
    let behaviour = interp.classes().instance_behaviour_handle(class);
    let address = std::ptr::without_provenance_mut(id.address());
    interp.alloc_with(
        BehaviourId::OBJECT,
        Body::pointer(class, behaviour, address),
    )
}

/// The timer a handle object holds, or `None` for any other pointer.
fn timer_of(interp: &Interp, handle: ObjRef) -> Option<TimerId> {
    match &interp.heap.get(handle)?.body {
        Body::Instance {
            native: Some(state),
            ..
        } => match **state {
            NativeState::Pointer(address) => TimerId::from_address(address.addr()),
            _ => None,
        },
        _ => None,
    }
}

/// Stores `value` in the receiver's variable `name` of `scope`, as
/// `SetObjectVariable` does.
fn set_object_variable(
    interp: &mut Interp,
    receiver: ObjRef,
    scope: ObjRef,
    name: &[u8],
    value: ObjRef,
) -> Result<(), Failure> {
    let owner = interp.pool_owner(receiver)?;
    interp.set_pool_variable(owner, scope, name, value);
    Ok(())
}

/// Whether the receiver's `CANCELED` of `scope` is `.true`. The oracle tests
/// identity with `.true`; a string's identity here is its bytes (D15), so
/// any `1` passes.
fn is_cancelled(interp: &mut Interp, receiver: ObjRef, scope: ObjRef) -> Result<bool, Failure> {
    let owner = interp.pool_owner(receiver)?;
    let Some(cancelled) = interp
        .pools_of(owner)
        .and_then(|pools| pools.get(scope, b"CANCELED"))
    else {
        return Ok(false);
    };
    Ok(match cancelled.decode() {
        Decoded::SmallInt(number) => number == 1,
        _ => interp.to_text(cancelled).as_ref() == b"1",
    })
}

/// A `POINTER` argument: 88.901 where it is missing, 88.914 where it is not
/// a `.Pointer`, else the timer it holds, if any.
fn pointer_argument(
    interp: &Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<Option<TimerId>, Failure> {
    let Some(Some(value)) = args.get(index).copied() else {
        return Err(Raised::missing_native_argument(index + 1).into());
    };
    let is_pointer = matches!(
        interp.heap.get(value).map(|object| &object.body),
        Some(Body::Instance { native: Some(state), .. })
            if matches!(**state, NativeState::Pointer(_))
    );
    if !is_pointer {
        return Err(Raised::native_argument_not_an_instance(index + 1, "Pointer").into());
    }
    Ok(timer_of(interp, value))
}

/// A `wholenumber_t` argument: 88.901 where it is missing, 88.907 where it is
/// not a whole number within `Numerics::MAX_WHOLENUMBER`.
fn whole_number_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<i64, Failure> {
    let Some(Some(value)) = args.get(index).copied() else {
        return Err(Raised::missing_native_argument(index + 1).into());
    };
    let number = match value.decode() {
        Decoded::SmallInt(number) => Some(number),
        _ => {
            let text = interp.to_text(value);
            Number::parse_bytes(&text).and_then(|number| number.int64_value(DIGITS64))
        }
    };
    match number.filter(|number| (-MAX_WHOLENUMBER..=MAX_WHOLENUMBER).contains(number)) {
        Some(number) => Ok(number),
        None => {
            let found = interp.string_value_text(value);
            let (min, max) = (i128::from(-MAX_WHOLENUMBER), i128::from(MAX_WHOLENUMBER));
            Err(Raised::native_argument_outside_range(index + 1, min, max, &found).into())
        }
    }
}

#[cfg(test)]
mod tests;
