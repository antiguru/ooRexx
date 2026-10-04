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

//! Native calls and commands on the interpreter's pool threads (spec
//! 2026-09-29 2.2, 2.6, 2.7), over routines this test defines.

use std::sync::{Condvar, Mutex};
use std::thread::ThreadId;
use std::time::{Duration, Instant};

use rexx_api::layout::{RexxCallContext_, ValueDescriptor};
use rexx_api::values::{ARGUMENT_TERMINATOR, code};

use crate::{Invocation, Outcome};

/// A routine that answers nothing.
static SIGNATURE: [u16; 2] = [code::REXX_OBJECT_PTR, ARGUMENT_TERMINATOR];

/// Two calls meeting: how many are here, met and gave up.
struct Meeting {
    counts: Mutex<(usize, usize, usize)>,
    arrived: Condvar,
}

impl Meeting {
    const fn new() -> Meeting {
        Meeting {
            counts: Mutex::new((0, 0, 0)),
            arrived: Condvar::new(),
        }
    }

    fn reset(&self) {
        *self.counts.lock().expect("unpoisoned") = (0, 0, 0);
    }

    /// Arrives, and waits up to `patience` for a second call to be here.
    fn arrive(&self, patience: Duration) {
        let mut counts = self.counts.lock().expect("unpoisoned");
        counts.0 += 1;
        self.arrived.notify_all();
        let end = Instant::now() + patience;
        while counts.0 < 2 {
            let now = Instant::now();
            if now >= end {
                counts.0 -= 1;
                counts.2 += 1;
                return;
            }
            counts = self
                .arrived
                .wait_timeout(counts, end - now)
                .expect("unpoisoned")
                .0;
        }
        counts.1 += 1;
    }

    /// How many met and how many gave up.
    fn outcome(&self) -> (usize, usize) {
        let counts = self.counts.lock().expect("unpoisoned");
        (counts.1, counts.2)
    }
}

static MEETINGS: [Meeting; 2] = [Meeting::new(), Meeting::new()];

/// How long the meeting in each slot waits for its second call.
const PATIENCE: [Duration; 2] = [Duration::from_secs(10), Duration::from_millis(300)];

extern "C-unwind" fn meet<const SLOT: usize>(
    _context: *mut RexxCallContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return SIGNATURE.as_ptr().cast_mut();
    }
    MEETINGS[SLOT].arrive(PATIENCE[SLOT]);
    std::ptr::null_mut()
}

/// The threads [`nap`] and [`here`] ran on.
static THREADS: Mutex<Vec<(&str, ThreadId)>> = Mutex::new(Vec::new());

/// Sleeps a fifth of a second.
extern "C-unwind" fn nap(
    _context: *mut RexxCallContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return SIGNATURE.as_ptr().cast_mut();
    }
    THREADS
        .lock()
        .expect("unpoisoned")
        .push(("nap", std::thread::current().id()));
    std::thread::sleep(Duration::from_millis(200));
    std::ptr::null_mut()
}

extern "C-unwind" fn here(
    _context: *mut RexxCallContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return SIGNATURE.as_ptr().cast_mut();
    }
    THREADS
        .lock()
        .expect("unpoisoned")
        .push(("here", std::thread::current().id()));
    std::ptr::null_mut()
}

fn library() -> rexx_api::load::Library {
    rexx_api::load::routines_only(&[
        ("MEET", meet::<0>),
        ("MEETBRIEFLY", meet::<1>),
        ("NAP", nap),
        ("HERE", here),
        ("SENDTHENAWAIT", rexx_api::load::send_then_await),
    ])
}

/// A run: its outcome, its driver exits, and the interpreter's thread.
struct Ran {
    outcome: Outcome,
    exits: u64,
    thread: ThreadId,
}

impl Ran {
    fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.outcome.stdout).into_owned()
    }

    fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.outcome.stderr).into_owned()
    }
}

/// Runs `source` on an interpreter thread of its own, with this module's
/// routines as the library `pooltest`, the test libraries on its library
/// path, and its pool threads given `pool_stack` bytes where that is set.
fn run(source: &str, pool_stack: Option<usize>) -> Ran {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../build/lib")
        .canonicalize()
        .expect("the worktree's build/lib is three directories above this crate");
    let invocation = Invocation::none()
        .with_deadline(Duration::from_secs(60))
        .with_environment(vec![
            (
                b"LD_LIBRARY_PATH".to_vec(),
                directory.into_os_string().into_encoded_bytes(),
            ),
            (b"PATH".to_vec(), b"/usr/bin:/bin".to_vec()),
        ]);
    let text = source.as_bytes().to_vec();
    std::thread::Builder::new()
        .stack_size(crate::INTERPRETER_STACK_BYTES)
        .spawn(move || {
            crate::install::offer_library(b"pooltest", library);
            if let Some(bytes) = pool_stack {
                crate::set_pool_stack(bytes);
            }
            let outcome = crate::execute_on(
                "/tmp/pool.rex",
                text,
                false,
                invocation,
                Some(crate::INTERPRETER_STACK_BYTES),
            );
            Ran {
                outcome,
                exits: crate::scheduler::native_exits(),
                thread: std::thread::current().id(),
            }
        })
        .expect("the interpreter thread")
        .join()
        .expect("the run did not panic")
}

const MEETING: &str = "m = .t~new~start('other')\ncall MEET\nsay 'main'\nsay m~result\n\
                       ::requires 'pooltest' LIBRARY\n::class t\n::method other\n  \
                       call MEET\n  return 'other'\n";

/// Two activities each in a native call that waits for the other: the
/// calls run at once on pool threads, so they meet.
#[test]
fn two_native_calls_waiting_for_each_other_meet_on_pool_threads() {
    MEETINGS[0].reset();
    let ran = run(MEETING, None);
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "main\nother\n");
    assert_eq!(ran.exits, 2);
    assert_eq!(MEETINGS[0].outcome(), (2, 0));
}

/// Where no pool thread can be spawned, a call runs on the releasing thread
/// and the ready activity waits for the next thread that takes the baton:
/// the calls run one after the other, and each gives up.
#[test]
fn a_spawn_failure_leaves_ready_activities_to_the_next_baton_holder() {
    MEETINGS[1].reset();
    let source = MEETING.replace("MEET\n", "MEETBRIEFLY\n");
    let ran = run(&source, Some(1 << 46));
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "main\nother\n");
    assert_eq!(ran.exits, 2);
    assert_eq!(MEETINGS[1].outcome(), (0, 2));
}

/// Rexx code a callback runs on a pool thread, recursing without bound
/// through native calls that call back, raises 11.1 when that thread's stack
/// runs low, rather than overflowing it.
#[test]
fn deep_pinned_recursion_on_a_pool_thread_raises_11() {
    let ran = run(
        "m = .t~new~start('idle')\nsay .k~new~send0(.r~new, 'deep')\n\
         ::class t\n::method idle\n  call SysSleep 0.1\n\
         ::class r\n::method deep\n  return .k~new~send0(self, 'deep')\n\
         ::class k\n::method send0 external \"LIBRARY orxmethod TestSendMessage0\"\n",
        None,
    );
    assert_eq!(ran.exits, 1);
    assert!(
        ran.stderr().contains("Error 11.1:"),
        "{}: {}",
        ran.outcome.exit_code,
        ran.stderr()
    );
}

/// A pinned waiter whose wait an activity's native call satisfies: the call
/// runs on a pool thread, the nested loop blocks on the inbox until it
/// completes, and the pinned frames resume on the interpreter's thread.
#[test]
fn a_pinned_wait_blocks_on_the_inbox_while_a_pool_thread_runs_the_call() {
    let ran = run(
        "m = .t~new~start('work')\na = .array~of(2, 1)\na~sortWith(.c~new(m))\n\
         say 'sorted' a~toString('L', ',')\n\
         ::requires 'pooltest' LIBRARY\n::class t\n::method work\n  call NAP\n  return 7\n\
         ::class c\n::method init\n  expose m\n  use arg m\n\
         ::method compare\n  expose m\n  use arg l, r\n  say m~result\n  call HERE\n  \
         return l - r\n",
        None,
    );
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "7\nsorted 1,2\n");
    assert_eq!(ran.exits, 1);
    let threads = THREADS.lock().expect("unpoisoned");
    let napped = threads.iter().filter(|(name, _)| *name == "nap").count();
    assert!(napped >= 1);
    let mine = |name| {
        threads
            .iter()
            .filter(|(ran_as, thread)| *ran_as == name && *thread == ran.thread)
            .count()
    };
    assert_eq!(mine("nap"), 0, "the call ran on the interpreter's thread");
    assert!(mine("here") >= 1, "the pinned frames resumed elsewhere");
}

/// An activity in a long command does not stop another activity's output:
/// the command's child is waited for on a pool thread.
#[test]
fn a_long_command_does_not_stop_another_activitys_output() {
    let ran = run(
        "m = .t~new~start('ticks')\n'sleep 0.5'\nsay 'after' rc\nsay m~result\n\
         ::class t\n::method ticks\n  do i = 1 to 3\n    say 'tick' i\n    \
         call SysSleep 0.05\n  end\n  return 'done'\n",
        None,
    );
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "tick 1\ntick 2\ntick 3\nafter 0\ndone\n");
}

/// A lone activity's native call keeps the baton for its whole length
/// (ruling P43), so an activity its callback starts runs only once the call
/// has returned: the call waits for that activity's file in vain.
#[test]
fn a_lone_call_keeps_the_baton_after_its_callback_starts_an_activity() {
    let path = std::env::temp_dir().join(format!("rexx-pool-{}", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let path = path.to_string_lossy().into_owned();
    let ran = run(
        &format!(
            "say SENDTHENAWAIT(.k~new, 'KICK', '{path}')\n::requires 'pooltest' LIBRARY\n\
             ::class k\n::method kick\n  self~start('w')\n\
             ::method w\n  call lineout '{path}', 'w'\n  call lineout '{path}'\n  say 'w'\n"
        ),
        None,
    );
    let _ = std::fs::remove_file(&path);
    assert_eq!(ran.outcome.exit_code, 0, "{}", ran.stderr());
    assert_eq!(ran.stdout(), "0\nw\n");
    assert_eq!(ran.exits, 0);
}
