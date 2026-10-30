# Task 8.8c re-review round 1

## Verdicts

Spec compliance: **Pass for the scoped fixes**. Quality: **Pass for the scoped fixes**, with both original findings addressed and no new Critical or Important issue in the fix diff. This review covers `03471cc90..420b46d6f`, final source `d7581cc4a`, and the implementation report reconciled at metadata commit `883937d25`. Full Task 8.8 acceptance and its controller-owned batch gates remain outside this verdict.

## Original findings

**ADDRESSED: The sleep observation still races two timers.** `rust/corpus/lang/command_redirected_target_waits_elsewhere.rex:36` now calls SysSleep before recording completion at line 37, with no competing alarm or notification flag. `rust/crates/rexx-exec/tests/timing_witnesses.rs:120` holds an unrelated notifier on a semaphore across the callback, and the retained red log shows the old callback printing slept 0 through that controlled ordering. The separate control at line 169 extracts the actual callback and distinguishes virtual-time advancement with SysSleep from its omission, while the full command transcript and semaphore/guard controls remain covered. The regenerated sourceline expectation and reconciled closure evidence describe this explicit division between callback completion and sleep-duration proof.

**ADDRESSED: Python can reap the owned child before exec.** `rust/crates/rexx-exec/tests/rexxutil_process/owned_child.py:11` observes exit status with waitid and WNOWAIT, then the retained branch at line 18 keeps Popen live through exec at line 24. The dropping branch at line 21 deliberately releases the already-waitable object, giving a deterministic destructor-poll counterexample without timing sleeps. `rust/crates/rexx-exec/tests/rexxutil_process.rs:157` requires retained status 1280, and the negative control at line 165 requires -1 after dropping. The retained red log reports the old discarded-object failure, while the final focused log confirms both corrected controls and the valid competing-waiter outcome.

## Evidence and limits

The retained `timing/fix-r1/command-status.tsv`, red logs and `final-focused.txt` substantiate the reported failures before correction and final successes of 29 timing tests and five process tests. The retained sourceline, formatting and all-target clippy logs each report exit 0, and the amended diff contains no production code change. No additional scoped evidence gap remains, and no observation on untouched code is raised by this re-review. SysFileTree and the stderr fixture remain unresolved outside this fix scope, with no build, test rerun, source, index or HEAD change performed during review.
