**OPEN: ruled 9d, cluster native-stack-frames (Phase 9d inventory).** names StackFrame TESTNATIVECODEFRAME, that cluster's test.

# Native method frames missing from a live .context~stackFrames

Source: Phase 9b1 Task 3 report (.superpowers/sdd/2026-10-14-phase-9b1/task-3-report.md), concern 1.
L11 (native routine frames) is fixed; native METHOD frames (e.g. SORTWITH, Routine~call's CALL) are
still absent from a live stackFrames, so ooTest base/.../StackFrame TESTNATIVECODEFRAME stays red. The
send path is hot: any fix measures sendloop, dispatch, dispatchclass, pingmsg against the pin.
