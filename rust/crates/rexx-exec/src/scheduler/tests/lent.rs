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

//! Island memory lent to native calls on pool threads while the holder runs
//! other activities (spec 2026-09-29 2.5), over routines this test and
//! `rexx_api::load` define.

use rexx_api::layout::{RexxCallContext_, ValueDescriptor};
use rexx_api::values::{ARGUMENT_TERMINATOR, code};

use super::pool::{Ran, SHAPE, Shape, run_shaped};

/// A routine taking one `CSTRING` and answering nothing.
static TAKES_C_STRING: [u16; 3] = [code::REXX_OBJECT_PTR, code::CSTRING, ARGUMENT_TERMINATOR];

/// Asks for its argument as a `CSTRING`, which the interpreter keeps a copy
/// of, and does nothing with it.
extern "C-unwind" fn take_c_string(
    _context: *mut RexxCallContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return TAKES_C_STRING.as_ptr().cast_mut();
    }
    std::ptr::null_mut()
}

fn library() -> rexx_api::load::Library {
    rexx_api::load::routines_only(&[
        ("HOLDCSTRING", rexx_api::load::hold_c_string),
        ("HOLDBUFFER", rexx_api::load::hold_buffer),
        ("BUFFERHELD", rexx_api::load::buffer_held),
        ("BUFFERCHANGED", rexx_api::load::buffer_changed),
        ("FINISHEDINPLACE", rexx_api::load::finished_in_place),
        ("TAKECSTRING", take_c_string),
    ])
}

/// Runs `source` with this module's routines as the library `lenttest`,
/// after replacing `PATH` in it with a file of the test's own, which the
/// program writes to say its other activity is done, or the test's key for
/// `HOLDBUFFER`, with [`crate::set_fail_native_wait`] set where
/// `fail_native_wait`.
fn run_lent(test: &str, source: &str, fail_native_wait: bool) -> Ran {
    let path = std::env::temp_dir().join(format!("rexx-lent-{}-{test}", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let source = source.replace("PATH", &path.to_string_lossy());
    let ran = run_shaped(
        &source,
        Shape {
            library: (b"lenttest", library),
            fail_native_wait,
            ..SHAPE
        },
    )
    .expect("the run did not panic");
    let _ = std::fs::remove_file(&path);
    ran
}

fn run(test: &str, source: &str) -> Ran {
    run_lent(test, source, false)
}

const ROUTINES: &str = "::requires 'lenttest' LIBRARY\n::class t\n";

/// The method that says the other activity is done.
const DONE: &str = "  call lineout 'PATH', 'done'\n  call lineout 'PATH'\n";

/// The method that waits for `HOLDBUFFER` to hold its buffer's address.
const HELD: &str = "  do 500 until BUFFERHELD('PATH')\n    call SysSleep 0.01\n  end\n";

/// The method that says to `HOLDBUFFER` that its buffer has changed.
const CHANGED: &str = "  call BUFFERCHANGED 'PATH'\n";

/// **A handle-carried value's kept string outlives a prune made while its
/// call is in flight**: another activity asks for more kept strings than the
/// prune's bound while the call holds one.
#[test]
fn a_held_handle_carried_string_survives_another_activitys_prune() {
    let ran = run(
        "carried",
        &format!(
            "m = .t~new~start('churn')\nsay HOLDCSTRING('abc', 'PATH')\nsay m~result\n\
             {ROUTINES}::method churn\n  do i = 1 to 5000\n    call TAKECSTRING i\n  end\n\
             {DONE}  return 'churned'\n"
        ),
    );
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "1\nchurned\n");
}

/// **A heap string's kept string outlives a collection made while its call
/// is in flight**: the call roots the string.
#[test]
fn a_held_heap_string_survives_another_activitys_collection() {
    let ran = run(
        "heap",
        &format!(
            "s = copies('k', 40)\nm = .t~new~start('allocate')\n\
             say HOLDCSTRING(s, 'PATH')\nsay m~result\n\
             {ROUTINES}::method allocate\n  do i = 1 to 200000\n    a = .array~new(1)\n  end\n\
             {DONE}  return 'allocated'\n"
        ),
    );
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "1\nallocated\n");
    assert!(ran.outcome.collections > 0, "nothing collected");
}

/// **A buffer grown while a call holds its address keeps the old storage
/// until the call ends**: the call reads what it held, and its writes there
/// are not seen, as in the oracle, whose growth also moves the bytes.
#[test]
fn a_buffer_grown_under_a_call_keeps_the_storage_the_call_holds() {
    let ran = run(
        "grown",
        &format!(
            "b = .mutablebuffer~new('abcdefghijklmnopqrstuvwxyz')\nm = .t~new~start('grow', b)\n\
             say HOLDBUFFER(b, 'PATH', 'ZZ')\nsay b~substr(1, 4) b~length\nsay m~result\n\
             {ROUTINES}::method grow\n  use arg b\n{HELD}  b~append(copies('y', 5000))\n\
             {CHANGED}  return 'grown'\n"
        ),
    );
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(
        ran.stdout(),
        "abcdefghijklmnopqrstuvwxyz\nabcd 5026\ngrown\n"
    );
}

/// **A buffer changed in place while a call holds its address stays
/// shared**: the call reads the other activity's write, and Rexx reads the
/// call's, as both would through the oracle's one storage.
#[test]
fn a_buffer_changed_in_place_under_a_call_stays_shared() {
    let ran = run(
        "overlaid",
        &format!(
            "b = .mutablebuffer~new('abcdefghijklmnopqrstuvwxyz')\nm = .t~new~start('overlay', b)\n\
             say HOLDBUFFER(b, 'PATH', 'ZZ')\nsay b~substr(1, 4) b~length\nsay m~result\n\
             {ROUTINES}::method overlay\n  use arg b\n{HELD}  b~overlay('XY', 3)\n\
             {CHANGED}  return 'overlaid'\n"
        ),
    );
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(
        ran.stdout(),
        "abXYefghijklmnopqrstuvwxyz\nZZXY 26\noverlaid\n"
    );
}

/// **Finishing a buffer string keeps the address `StringData` answered for
/// it**, which then reads every byte written up to the length the string
/// was made at, as the oracle's one storage does (`RexxString::finish` sets
/// only the length): measured, oracle, `helloxxxxx` for the shorter finish.
#[test]
fn finishing_a_buffer_string_keeps_its_kept_string_in_place() {
    let ran = run(
        "finished",
        "say FINISHEDINPLACE('hello', 5)\nsay FINISHEDINPLACE('hello', 10)\n\
         ::requires 'lenttest' LIBRARY\n",
    );
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "hello\nhelloxxxxx\n");
}

/// **A call abandoned while it runs keeps its frame until its completion is
/// drained**, and no longer.
#[test]
fn an_abandoned_calls_frame_ends_with_its_completion() {
    let ran = run_lent(
        "abandoned",
        &format!(
            "s = .t~new~start('idle')\nsignal on syntax name abandoned\n\
             say HOLDCSTRING('abc', 'PATH')\nsay 'unreached'\n\
             abandoned:\nsay 'abandoned' condition('o')~code\n\
             call lineout 'PATH', 'done'\ncall lineout 'PATH'\ncall SysSleep 0.3\n\
             say s~result\n{ROUTINES}::method idle\n  call SysSleep 1\n  return 'idle'\n"
        ),
        true,
    );
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "abandoned 11.1\nidle\n");
    assert_eq!(ran.abandoned, 0, "the completion left the frame held");
}
