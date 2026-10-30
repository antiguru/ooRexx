# Task 8.8f: PARSE performance repair round 2

## Measured gap and evidence

The first f performance repair increases parse own instructions by 0.34815184% against its numeric-repair BASE, confirmed in three rounds. [The residual diagnosis](task-8.8f-perf-r1-residual-diagnosis.md) attributes essentially the whole increase to `exec_parse`, whose self cost grows by 5.4 million instructions. Disassembly shows scalarized InlineText extraction and a larger frame instead of the previous packed rendering. Keep the separately reviewed StringValue and USE improvements, whose code is outside this repair.

## Narrow candidate

Restore the prior String/Mixed required-string conversion and `Rendered` representation rather than directly unpacking inline text. Omit only the duplicate converted-result temporary root when its handle equals the already-rooted original operand. Preserve the original operand root before TRACE, required conversion and raised failures, the converted-result root when needed, positional triggers and source-buffer borrowing. Do not change the global root API, decode inline bytes manually, remove heap roots or broaden into parser semantics.

The equality check compares generational handles and does not itself send or allocate. Prove that the original temporary root remains active across conversion, making an equal converted handle already protected. Other results retain their independent root, including object, reference and stem-default conversions. If this candidate does not improve emitted ordinary-path code, report the bounded result before expanding the experiment.

## Checks and handoff

The controller supplies the actual committed BASE after M6 review, with the preserved first f-performance pin as the runtime baseline. Commit before verification, freeze source during checks and use the plain approved capped-driver pattern with an explicit new output directory. Run the relevant immediate/heap/raising PARSE and semantic allocation-stress controls, positional and TRACE behavior, formatting and affected all-target clippy. Existing meaningful fixtures can be reused without duplicating implementation-shaped tests.

Write `task-8.8f-perf-fix-r2-report.md`, append the result to the f report, and retain exact commands, statuses, source/build/compiler hashes and a distinct release pin under the corresponding `semantics/perf-fix-r2` records. Root owns scoped review, full matched all-program profiling and final broad gates. The original failed measurements, all prior pins, F1/F2 fixes and the pending 84 ruling remain intact. No new unsafe, dependency, side map, semantic departure or agent is authorized.
