**CLOSED 2026-10-28 by Phase 9d Task 8.7b (probe agrees: docs/superpowers/records/2026-10-18-phase-9d/t8.7b/probes/frames/f1.rex, whose `send`, `sendwith`, `osend`, `call` and `callwith` rows agree at base; its `run`, `sort`, `eq`, `makearray` and `hash` rows are 2026-10-22-live-frames-omit-nested-primitives.md's, closed by 5053fbd9b).**
**OPEN: ruled 9d, cluster native-stack-frames (Phase 9d inventory).** names StackFrame TESTNATIVECODEFRAME, that cluster's test.

# Native method frames missing from a live .context~stackFrames

Source: Phase 9b1 Task 3 report (.superpowers/sdd/2026-10-14-phase-9b1/task-3-report.md), concern 1.
L11 (native routine frames) is fixed; native METHOD frames (e.g. SORTWITH, Routine~call's CALL) are
still absent from a live stackFrames, so ooTest base/.../StackFrame TESTNATIVECODEFRAME stays red. The
send path is hot: any fix measures sendloop, dispatch, dispatchclass, pingmsg against the pin.

Task 6.11 closed this for `SORTWITH` and `STABLESORTWITH`. Still open: the `Routine~call` CALL
frame (Task 6.11 report), and a `METHOD SENDWITH` frame the oracle shows for `sendWith` and the
crate does not (`task-6.11-review.md`, Minor 4).
The Task 6.11 round-2 re-review (`task-6.11-rereview-r2.md`, Minor 2) adds `Message~send`: the
oracle lists `METHOD SEND ... Compiled method "SEND" with scope "Message".` and the crate does not.
