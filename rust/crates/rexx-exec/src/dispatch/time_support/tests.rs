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

use std::time::{Duration, Instant};

use crate::{Invocation, Outcome, SwitchMode, run_program, run_program_collect_every_alloc};

/// A class whose methods are the timer natives, unguarded as `CoreClasses.orx`
/// binds them, with readers of the variables they set in its scope.
const NATIVES: &str = "\n::class t\n\
    ::method alarmStart unguarded external 'LIBRARY REXX alarm_startTimer'\n\
    ::method alarmStop unguarded external 'LIBRARY REXX alarm_stopTimer'\n\
    ::method tickerCreate unguarded external 'LIBRARY REXX ticker_createTimer'\n\
    ::method tickerWait unguarded external 'LIBRARY REXX ticker_waitTimer'\n\
    ::method tickerStop unguarded external 'LIBRARY REXX ticker_stopTimer'\n\
    ::method handle unguarded\n  expose eventSemHandle\n  return eventSemHandle\n\
    ::method started unguarded\n  expose timerStarted\n  return timerStarted\n\
    ::method cancelNow unguarded\n  expose canceled\n  canceled = .true\n";

/// Waits until `o`'s started `alarmStart` has set `TIMERSTARTED`.
const WAIT_STARTED: &str = "\nexit\nwaitStarted: procedure\n  use arg o\n  \
    do while o~started \\== 1\n    call SysSleep 0.01\n  end\n  return\n";

/// The outcome of `main` with `classes` after it in each mode, with how long
/// each run took. A deadline ends a wait nothing cancels.
fn in_modes(main: &str, classes: &str) -> Vec<(&'static str, Outcome, Duration)> {
    let text = format!("{main}{WAIT_STARTED}{classes}{NATIVES}").into_bytes();
    let path = "/tmp/timers.rex";
    let invocation = || Invocation::none().with_deadline(Duration::from_secs(30));
    ["unswitched", "every", "collect"]
        .into_iter()
        .map(|mode| {
            let began = Instant::now();
            let outcome = match mode {
                "every" => run_program(
                    path,
                    text.clone(),
                    invocation().with_switch_mode(SwitchMode::EveryOpportunity),
                ),
                "collect" => run_program_collect_every_alloc(path, text.clone(), invocation()),
                _ => run_program(path, text.clone(), invocation()),
            };
            (mode, outcome, began.elapsed())
        })
        .collect()
}

/// Runs `main` and `classes` in each mode: exit 0, `expected` on stdout, and a
/// run time from `at_least` up to well below any wait it cancels.
fn check(main: &str, classes: &str, expected: &str, at_least: Duration) {
    for (mode, outcome, took) in in_modes(main, classes) {
        let stderr = String::from_utf8_lossy(&outcome.stderr);
        assert_eq!(outcome.exit_code, 0, "{mode}: {stderr}");
        assert_eq!(String::from_utf8_lossy(&outcome.stdout), expected, "{mode}");
        assert!(
            took >= at_least && took < Duration::from_secs(10),
            "{mode}: took {took:?}"
        );
    }
}

/// `alarm_startTimer` sets its variables, waits its time and answers 0.
#[test]
fn an_alarm_timer_waits_its_time() {
    check(
        "o = .t~new\nsay o~alarmStart(0, 300) o~started o~handle~class~id\n",
        "",
        "0 1 Pointer\n",
        Duration::from_millis(300),
    );
}

/// A cancel before the timer is due ends its wait at once, in the remainder
/// and in the whole days alike.
#[test]
fn a_cancel_before_due_ends_an_alarm_wait() {
    for (days, millis) in [(0, 20_000), (2, 0)] {
        check(
            &format!(
                "o = .t~new\nm = o~start('alarmStart', {days}, {millis})\ncall waitStarted o\n\
                 o~cancelNow\nsay o~alarmStop(o~handle)\nsay m~result\n"
            ),
            "",
            "0\n0\n",
            Duration::ZERO,
        );
    }
}

/// A post that is not a cancel ends a wait in its remainder, and in its
/// whole days ends only the current day: the remainder still runs.
#[test]
fn a_post_ends_the_remainder_or_the_current_day() {
    check(
        "o = .t~new\nm = o~start('alarmStart', 0, 20000)\ncall waitStarted o\n\
         say o~alarmStop(o~handle)\nsay m~result\n",
        "",
        "0\n0\n",
        Duration::ZERO,
    );
    check(
        "o = .t~new\nm = o~start('alarmStart', 1, 400)\ncall waitStarted o\n\
         r = o~alarmStop(o~handle)\ncall SysSleep 0.1\nsay m~completed\nsay m~result\n",
        "",
        "0\n0\n",
        Duration::from_millis(400),
    );
}

/// A stop once an alarm's wait is over does nothing: its timer has ended.
#[test]
fn a_cancel_after_due_does_nothing() {
    check(
        "o = .t~new\nsay o~alarmStart(0, 10)\nh = o~handle\no~cancelNow\n\
         say o~alarmStop(h)\nsay o~alarmStart(0, 10)\n",
        "",
        "0\n0\n0\n",
        Duration::from_millis(20),
    );
}

/// A ticker's wait waits its interval; a post stays until reset, so every
/// later wait ends at once; a cancel ends the timer.
#[test]
fn a_ticker_post_stays_until_a_cancel() {
    check(
        "o = .t~new\nsay o~tickerCreate\nh = o~handle\nsay o~tickerWait(h, 0, 200)\n\
         say o~tickerStop(h)\nsay o~tickerWait(h, 0, 20000) o~tickerWait(h, 0, 20000)\n\
         o~cancelNow\nsay o~tickerStop(h)\nsay o~tickerWait(h, 0, 20000)\n",
        "",
        "0\n0\n0\n0 0\n0\n0\n",
        Duration::from_millis(200),
    );
}

/// A post before a wait with whole days is reset and ends the first day, so
/// the remainder is all that is waited.
#[test]
fn a_post_before_a_wait_of_days_ends_its_first_day() {
    check(
        "o = .t~new\nr = o~tickerCreate\nh = o~handle\nr = o~tickerStop(h)\n\
         say o~tickerWait(h, 1, 300)\n",
        "",
        "0\n",
        Duration::from_millis(300),
    );
}

/// A cancel ends a ticker's wait at once.
#[test]
fn a_cancel_ends_a_ticker_wait() {
    check(
        "o = .t~new\nr = o~tickerCreate\nh = o~handle\n\
         m = o~start('tickerWait', h, 0, 20000)\ncall SysSleep 0.2\no~cancelNow\n\
         say o~tickerStop(h)\nsay m~result\n",
        "",
        "0\n0\n",
        Duration::from_millis(200),
    );
}

/// A timer's wait under a pinned frame runs the other activities meanwhile.
#[test]
fn a_timer_wait_in_a_sort_comparator_runs_the_others_meanwhile() {
    check(
        "b = .other~new~start('run')\narr = .array~of(2, 1)\narr~sortWith(.cmp~new)\n\
         say 'sorted' arr~makeString('L', ' ')\n",
        "::class other\n::method run\n  call SysSleep 0.1\n  say 'other ran'\n\
         ::class cmp\n::method compare\n  use arg l, r\n  r2 = .t~new~alarmStart(0, 300)\n  \
         return l - r\n",
        "other ran\nsorted 1 2\n",
        Duration::from_millis(300),
    );
}

/// `.Alarm` fires on the activity its `REPLY` made, after main has ended,
/// and the program waits for it.
#[test]
fn an_alarm_fires_on_its_replied_activity() {
    check(
        "t = .target~new\na = .alarm~new(0.2, t)\nsay 'main' a~triggered\n",
        "::class target inherit AlarmNotification\n::method triggered\n  use arg alarm\n  \
         say 'triggered' alarm~triggered (.context~thread \\= 1)\n",
        "main 0\ntriggered 1 1\n",
        Duration::from_millis(200),
    );
}

/// `.Alarm~cancel` once the timer has started ends it before it fires.
#[test]
fn an_alarm_cancel_ends_its_wait() {
    check(
        "t = .target~new\na = .alarm~new(2 * 86400, t)\ncall SysSleep 0.2\na~cancel\n\
         say 'b' a~triggered a~canceled\n",
        "::class target inherit AlarmNotification\n::method triggered\n  say 'triggered'\n\
         ::method cancel\n  say 'cancel notified'\n",
        "cancel notified\nb 0 1\n",
        Duration::from_millis(200),
    );
}

/// A `.Ticker` triggers each interval until its target cancels it.
#[test]
fn a_ticker_triggers_until_cancelled() {
    check(
        "t = .counter~new\nk = .ticker~new(0.05, t)\nsay 'main'\n",
        "::class counter inherit AlarmNotification\n::method init\n  expose count\n  \
         count = 0\n::method triggered\n  expose count\n  use arg ticker\n  \
         count = count + 1\n  say 'tick' count\n  if count = 3 then ticker~cancel\n\
         ::method cancel\n  say 'cancel notified'\n",
        "main\ntick 1\ntick 2\ntick 3\ncancel notified\n",
        Duration::from_millis(150),
    );
}
