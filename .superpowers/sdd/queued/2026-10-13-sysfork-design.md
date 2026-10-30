**CLOSED 2026-10-30 by Phase 9d Task 8.8e1 (probe agrees).**

# SysFork: fork with helper threads

Moritz 2026-10-13: deferred (9b1 or later). Earlier grant: one unsafe fork() site in
builtin/rexxutil/process.rs, refusing when threaded. Measured (9b Task 7 fix round): rexx-run always
runs three threads, the main thread joining the interpreter, rexx-interp, and rexx-timer
(timer::State::start_timer, never exits). The oracle has one. A child that keeps running the Rexx
program needs:
(a) the timer's State mutex held across fork (atfork discipline), timer_running reset and a fresh
    timer started in the child;
(b) a child end path: rexx-run's main thread, which hands over buffered output and exits with rc,
    does not exist in the child;
(c) "threaded" redefined: no activity besides the caller, no rexx-pool thread, no other interpreter
    in the process (the corpus harness runs many).
Oracle behaviour: a plain fork works (3/3); with a second activity the child hangs (5/5,
oracle-crashes entry 46). No ooTest file mentions SysFork. Records:
.superpowers/sdd/2026-10-13-phase-9b/task-7-report.md.


## Task 8.8e1 evidence

SysFork refuses at status 120 with Deviation 42. No oracle fork or implementation is claimed. See `.superpowers/sdd/2026-10-18-phase-9d/task-8.8e1-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/records/e1/`.
