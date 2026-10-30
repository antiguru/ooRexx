# Task 8.8f: Performance repair round 1

## Measured gaps

[The diagnosis](task-8.8f-perf-diagnosis.md) localizes three confirmed regressions in the reviewed f source against e2. Parse increases by 0.22055592%, with 3.4 million extra self instructions in `exec_parse`. Directory reads increase by 0.36465390%, primarily in `probe_in` and its caller. Pingmsg increases by 0.07291944%, with almost its entire increase in `exec_use` on immediate arguments.

## Narrow repairs

* For PARSE String/Mixed triggers, use decoded inline text directly and the existing local integer rendering for SmallInt. Preserve the original operand evaluation/rooting and TRACE order, positional triggers, source-buffer borrow boundaries, and required-string conversion plus a rooted result for other values.
* For string-keyed probing, avoid callback generation snapshots and restart state only where the existing StringValue hash/equality protocol cannot send. Reuse shared traversal without copying a logic block. Preserve the callback-capable equality-key path, raised sends, predecessor contract, single HASHCODE and all F1/F2 reentry behavior.
* For USE arguments, omit temporary retention only for immediate values that cannot be collected. Keep heap, default and converted-value roots, distinct fallible rendering for each emitted trace line, and before/after-write gates. Do not change the global temporary-root API or assume every untraced argument is immediate.

These changes preserve semantics, including Deviations 44 and 72/77/78/79/80/81/82/83. The separate count defect remains pending user ruling 84, which this repair must not write. Keep comments and perform any necessary responsibility split as an early pure move with the method surface preserved. Do not optimize unrelated string copying, alter source-trace protocol, or remove conservative saved-store roots to address a profile that does not execute them.

## Evidence and handoff

The controller supplies the actual committed BASE and pre-repair pin after the numeric repair is reviewed. Existing semantic and callback regressions already establish the required behavior, while new boundary coverage should test immediate text/integer versus heap/default/raising paths where missing. Commit before verification and freeze source during each bounded command. Run relevant semantic allocation-stress, PARSE/USE/TRACE/hash controls, formatting and affected all-target clippy, preserving the original failed measurements.

Root owns matched all-program profiling and the final broad gates. Preserve a new release pin and source/build/compiler hashes, append this repair to the original f report, and write `task-8.8f-perf-fix-report.md`. Durable records belong under `docs/superpowers/records/2026-10-18-phase-9d/t8.8/semantics/perf-fix-r1/`. Return exact commands, statuses, commits and remaining limitations, without claiming the budget passed before measurement or spawning agents.
