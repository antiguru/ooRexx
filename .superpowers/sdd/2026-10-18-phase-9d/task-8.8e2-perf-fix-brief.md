# Task 8.8e2: Performance fix round 1

## Confirmed gap

The first complete dispatch comparisons exceed the own-delta budget without a spread or output difference. Dispatch increases by 0.67687242% and class dispatch by 0.73155584% against e1. [The diagnosis](task-8.8e2-perf-diagnosis.md) attributes about 28 extra instructions per repeated send to the new executable-index miss, with cold allocation bypassed. Preserve the original e2 correctness contract while removing unnecessary ordinary-method lookup cost.

## Narrow repair

Evaluate replacing the existing weak `executable_objects` index representation with the existing `MethodRows<ObjRef>` container. This replaces the authoritative registry rather than adding a parallel numeric-id map. Follow the diagnosis's compatibility checklist, including first-wins insertion, keyed iteration and retain, both bootstrap sealing boundaries, late insertion below the sealed prefix, runtime insertion/removal above it, repeated programs and independent interpreters. Keep the index weak and preserve exact entry-time executable retention in the existing activation root.

Add only the meaningful storage-contract regressions required by the adaptation. Preserve replacement, removal, aliases, enhanced nil-scope ownership, active collection and all original worker-failure contracts. Capture the committed pre-repair source and release pin before editing, and run targeted red controls before implementation where the new container API exposes a missing contract. Do not weaken identity or GC assertions to meet an instruction budget.

## Checks and handoff

Use the common commit-before-verification, memory-cap and bounded foreground workflow. Run the relevant MethodRows/storage and e2 identity/GC controls, policy/source checks justified by the adaptation, formatting and affected all-target clippy. Preserve a release pin with its source/build hash for matched profiling, retaining the original failed measurements rather than replacing them. Root owns final all-program comparisons, whole-batch gates and source integration, and this repair must receive a scoped review before acceptance.

Append the fix evidence and exact commands/statuses to `task-8.8e2-report.md`, and write the compact repair result in `task-8.8e2-perf-fix-report.md`. Keep durable outputs under `docs/superpowers/records/2026-10-18-phase-9d/t8.8/runtime/e2/perf-fix-r1/`. Do not widen into numeric-power optimization, M6 tests, further semantic departures or phase closure, and do not spawn agents. The controller grants source/Git ownership only after f's reviewed handoff.
