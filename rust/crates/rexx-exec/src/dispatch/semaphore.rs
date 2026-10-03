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

//! `EventSemaphore`'s and `MutexSemaphore`'s methods
//! (`classes/EventSemaphore.cpp`, `classes/MutexSemaphore.cpp`), whose waits
//! park the activity.

use std::time::{Duration, Instant};

use rexx_core::{Decoded, ObjRef};
use rexx_num::Number;

use super::{Cleared, NativeStarted};
use crate::error::Raised;
use crate::scheduler::ParkReason;
use crate::semaphores::{Request, SemaphoreKey, SemaphoreWait, WaitKind};
use crate::{Failure, Interp};

/// `EventSemaphoreClass::post`.
pub(super) fn native_event_post(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    interp.post_event(receiver);
    Ok(None)
}

/// `EventSemaphoreClass::reset`.
pub(super) fn native_event_reset(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    interp.reset_event(receiver);
    Ok(None)
}

/// `EventSemaphoreClass::posted`.
pub(super) fn native_event_is_posted(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let posted = interp.event_posted(receiver);
    Ok(Some(interp.counted(usize::from(posted))))
}

/// `EventSemaphoreClass::wait(timeout)`: `1` at once where posted, the post
/// itself for a timeout of zero, else parked until a post or the timeout.
pub(super) fn native_event_wait(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let timeout = timeout_argument(interp, b"WAIT", args)?;
    let posted = interp.event_posted(receiver);
    if posted || timeout == Some(Duration::ZERO) {
        return Ok(NativeStarted::Ran(Some(
            interp.counted(usize::from(posted)),
        )));
    }
    park_point!(interp, crate::pinning::ParkKind::SemaphoreWait);
    let wait = semaphore_wait(receiver, WaitKind::Event, timeout);
    interp.park_native(ParkReason::Semaphore(wait), woken, receiver)
}

/// A woken wait's answer, which its re-test kept.
fn woken(interp: &mut Interp, _receiver: ObjRef) -> Result<Option<ObjRef>, Failure> {
    let answer = interp.take_semaphore_answer();
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `MutexSemaphoreClass::request(timeout)`, the method `ACQUIRE`: the lock
/// where it is free or the activity's own, `0` for a timeout of zero, else
/// parked until a release hands it over or the timeout.
pub(super) fn native_mutex_acquire(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let timeout = timeout_argument(interp, b"ACQUIRE", args)?;
    let acquired = match interp.request_mutex(receiver, timeout == Some(Duration::ZERO)) {
        Request::Acquired => true,
        Request::Refused => false,
        Request::Wait => {
            park_point!(interp, crate::pinning::ParkKind::SemaphoreWait);
            let wait = semaphore_wait(receiver, WaitKind::Mutex, timeout);
            return interp.park_native(ParkReason::Semaphore(wait), woken, receiver);
        }
    };
    Ok(NativeStarted::Ran(Some(
        interp.counted(usize::from(acquired)),
    )))
}

/// `MutexSemaphoreClass::release`.
pub(super) fn native_mutex_release(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let released = interp.release_mutex(receiver);
    Ok(Some(interp.counted(usize::from(released))))
}

/// `MutexSemaphoreClass::close`, the method `UNINIT`.
pub(super) fn native_mutex_uninit(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    interp.close_mutex(receiver);
    Ok(None)
}

/// A wait of `kind` on `object` for `timeout`, or for ever.
fn semaphore_wait(object: ObjRef, kind: WaitKind, timeout: Option<Duration>) -> SemaphoreWait {
    SemaphoreWait {
        key: SemaphoreKey::Object(object),
        kind,
        deadline: timeout.map(|timeout| Instant::now() + timeout),
    }
}

/// The largest timeout in seconds the oracle's `uint32_t` milliseconds hold;
/// beyond it, as below zero, the wait is for ever.
const LONGEST_SECONDS: f64 = 4_294_967.0;

/// A `WAIT` or `ACQUIRE` timeout, a `TimeSpan` read through its
/// `totalSeconds`, in whole milliseconds: `None` for ever.
fn timeout_argument(
    interp: &mut Interp,
    method: &[u8],
    args: &[Option<ObjRef>],
) -> Result<Option<Duration>, Failure> {
    let Some(Some(value)) = args.first().copied() else {
        return Ok(None);
    };
    let frame = interp.roots.activity_mut().push_frame();
    let seconds = timeout_seconds(interp, method, value);
    interp.roots.activity_mut().pop_frame(frame);
    let seconds = seconds?;
    Ok((0.0..=LONGEST_SECONDS)
        .contains(&seconds)
        .then(|| Duration::from_millis((seconds * 1000.0) as u64)))
}

/// `floatingPointArgument(seconds, "timeout")`.
fn timeout_seconds(
    interp: &mut Interp,
    #[cfg_attr(not(feature = "pinning"), expect(unused_variables))] method: &[u8],
    value: ObjRef,
) -> Result<f64, Failure> {
    let seconds = if interp.is_instance_of_rexx_class(value, b"TIMESPAN") {
        let caller = interp.caller();
        let sent = pinned!(
            interp,
            crate::pinning::PinKind::native(method),
            interp.send_message(value, b"TOTALSECONDS", None, &[], caller)
        )?;
        sent.unwrap_or(ObjRef::NIL)
    } else {
        value
    };
    if let Decoded::SmallInt(number) = seconds.decode() {
        #[expect(
            clippy::cast_precision_loss,
            reason = "`RexxInteger::doubleValue` is the same C conversion"
        )]
        return Ok(number as f64);
    }
    interp.roots.activity_mut().push_temp(seconds);
    let text = pinned!(
        interp,
        crate::pinning::PinKind::native(method),
        interp.required_string_value(seconds)
    )?;
    let bytes = interp.to_text(text).into_owned();
    if let Some(number) = Number::parse_bytes(&bytes) {
        return Ok(super::library::double_of(&number));
    }
    match &bytes[..] {
        b"nan" => Ok(f64::NAN),
        b"+infinity" => Ok(f64::INFINITY),
        b"-infinity" => Ok(f64::NEG_INFINITY),
        _ => Err(Raised::native_argument_not_a_number("timeout", &bytes).into()),
    }
}
