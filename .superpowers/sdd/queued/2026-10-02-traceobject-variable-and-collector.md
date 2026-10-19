**CLOSED 2026-10-19 by Phase 9d Task 6.10.** its VARIABLE entry, the TraceObject made for every line (collector, notify, option `P`) and the `T`, `S` and `F` formats on the default route are witnessed by `trace_object_probe_collect` and `trace_object_format_default_route`; `TEST_VARIABLE` passes, and `TEST_TRACEOBJECT_COLLECTOR` fails only on the package name, which is `dynamic-package-name`'s. The EXECUTABLE bullet stays under the class-methods.txt exclusion.

# TraceObject: the VARIABLE entry, and TraceObjects off the routed path

Found by Phase 6 S2-S5 Task 9, which gave routed TraceObjects their THREAD, INVOCATION, STACKFRAME,
CALLERSTACKFRAME, RECEIVER and (zero) guard entries.

- `VARIABLE` (`RexxActivation.cpp:5177-5184`): on `>V>` and `>=>` lines the oracle puts a
  `StringTable` of NAME, VALUE (the object) and ASSIGNMENT. This crate's trace emitters pass the
  value's text, not the object, so the entry is absent here.
- The oracle builds a TraceObject for every trace line (`processTraceInfo`), which is what feeds
  `.TraceObject~collector`, `~notify`, option `P` (no output) and the `T`/`S`/`F` formats on the
  default `.error` route. This crate builds one only when `.TRACEOUTPUT` is routed to something other
  than standard error, so those four do nothing on the default route.
- EXECUTABLE in a frame table is `.nil` for a native or `INTERPRET` level, where the oracle has an
  object (`StackFrame~EXECUTABLE` is not implemented: class-methods.txt exclusion).

`base/keyword/TRACE_TraceObject.testGroup` through the group runner
(`concurrency_tests.rs`, `the_outcome_table_of_the_trace_object_group`): `TEST_RECEIVER`,
`TEST_CALLER_STACK_FRAME` and `TEST_TRACEOBJECT_COLLECTOR_AND_NOTIFY_CLASS_ATTRIBUTES` differ on
the collector; `TEST_TRACEOBJECT_COLLECTOR` and `TEST_VARIABLE` are refused earlier at `DO COUNTER
... OVER` ("DO is not implemented").
