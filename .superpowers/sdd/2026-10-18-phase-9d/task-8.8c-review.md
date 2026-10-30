# Task 8.8c review

## Verdicts

Spec compliance: **Needs changes** for the redirected-target pair and the owned-child positive control. The seven selected closures, two explicitly open diagnoses and inventory changes otherwise follow the assigned scope and controller rulings. Timeout reporting preserves the hard wall deadline, identity classification rejects unsupported markers, and overlap checks preserve exact ordinary/gap observations plus TZ presence and value. Quality: **Needs changes**, with two high-confidence scheduling races below and no other correctness finding in the reviewed range.

## Findings

### P2: The sleep observation still races two timers

At `rust/corpus/lang/command_redirected_target_waits_elsewhere.rex:36`, the callback starts an asynchronous 0.5-second alarm, sleeps for one second, then immediately reads the alarm's flag at line 38. The native Alarm implementation in `/home/moritz/dev/repos/ooRexx/interpreter/RexxClasses/CoreClasses.orx:1555` replies before starting its timer at line 1557 and invokes the notification only after the asynchronous wait at line 1565. A valid schedule can delay either timer startup or notification until after the callback resumes from a correct SysSleep, producing `O:a slept 0` and failing the corpus, stress and focused checks. The retained positive samples and deterministic omission controls do not establish an ordering edge, so both redirected-target closures still lack their required scheduling-independent witness.

Remove the competing-timer assumption from the corpus observation. Observe explicit completion after SysSleep returns, and retain actual sleep/wait discrimination in deterministic timer or scheduler coverage where the wait state is observable. Update the sleep omission control and generated sourceline fixture to match that division of proof. A startup acknowledgement alone is insufficient because notification execution can still lag behind the sleeping callback's continuation.

### P2: Python can reap the owned child before exec

At `rust/crates/rexx-exec/tests/rexxutil_process.rs:143`, the Python command discards the Popen object immediately before replacing Python with the interpreter. The installed `/usr/lib/python3.14/subprocess.py:1132` destructor calls `_internal_poll` at line 1142, and its POSIX implementation calls `waitpid(pid, WNOHANG)` at line 2021. If the exit-5 child finishes before that destructor poll, Python consumes its status and the correctly implemented SysWait later returns -1 instead of the required 1280. Retain the Popen object in a local variable through `os.execv`, so successful exec transfers the still-owned child without a destructor poll.

## Evidence and limits

The review read the scoped `b300b9c25..03471cc90` diff, the brief, dispatch, implementation report, DST diagnosis and retained timing records. `command-status.tsv`, the timeout association assertion, final timezone output, native/process results and retained GroupRun contexts support the reported focused successes. The TZ regression records demonstrate absent becoming empty before the repair and absent, empty and named-zone restoration after it, while the unchanged environment reader copies the saved bytes before mutation. The production native-watch arm retains its original channel/thread behavior, and the supplied outcome exists only under cfg(test).

SysFileTree and the stderr fixture correctly remain open after bounded non-reproduction with richer retained diagnostics. Their passing records do not establish a cause, and neither receives a licence or skip from this slice. Full corpus, suite/workspace, policy and all-program own-BASE performance checks remain controller-owned batch gates, so this review makes no batch-completion claim. No build, test, source, index or HEAD change was made during review, and the two findings follow from source-level permitted schedules rather than a new failing run.
