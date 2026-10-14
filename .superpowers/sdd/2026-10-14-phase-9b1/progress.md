# Phase 9b1 ledger: uniform object model (pre-spec)

Source: .superpowers/sdd/queued/2026-10-13-phase-9b1-uniform-object-model.md (approach steps 1-4),
.superpowers/sdd/queued/2026-10-12-package-as-real-object.md, rust/CLAUDE.md "Specialization is internal".
Moritz 2026-10-14: "Push, continue" after 9b close (pushed ce3793971).

Step 1+2 (parallel scouts, read-only, opus):
- scout-leaks: inventory of representation leaks in the crate -> scout-leaks.md
- scout-deviations: re-judge Deviations against the principle + oracle identity semantics -> scout-deviations.md
Then: design spec (brainstorm with Moritz; open decision: string identity rule), plan, SDD.
scout-deviations done (scout-deviations.md): A = Dev 31 + Dev 4's "we merge" half; B (oracle bug, not copied) = 0,2,3,9,24-30,32,34,35; C = 1,6,7,10-23,33. New defect: WeakReference to an inline value (short string/SmallInt) cleared at every GC while held (heap.rs weak pass, resolve() None for inline) -> 9b1. Oracle identity: literals interned per compiled unit/INTERPRET; -10..100 process-wide cache; every other production fresh. Recommendation: license "separately produced equal values are one object?" as unspecified; require per-held-value consistency; own rule. Open for Moritz: under "identity is the value until materialized", OBJECTNAME= on 'abc' would name every unmaterialized 'abc' unless naming materializes the held value.
scout-leaks done (scout-leaks.md): L1 weakref to inline value -> .nil after GC (heap.rs:282); L2 arithmetic/prefix/DO-setup + on String subclass never sent (eval.rs:843/:646, loops.rs:1062); L3 class-side STRING not sent (reqstr.rs:198; queued item exists); L5 Package subclass ~copy rc 120; L8 ProgramIds never freed (Method~new 80k -> 1.66 GB vs 20 MB); L9 Message/Method/Routine side maps never pruned; L11 native routine frame missing from live stackFrames. Activation-scope state already closed by 6.1 (66b0f6854); queued clock/setlocal files likely stale.
Spec written, Moritz Lgtm; fable spec review (spec-review.md): not approved, H1 + 8 M + 11 L; all applied in spec revision (criterion 2 -> source grep with allow-list; program lifetime ruled: ids never reused, sweep frees unheld programs and prunes id-keyed rows; weakref-to-dropped-value licensed; section-1 witness criterion; perf headroom named; L12 Deviation approved with spec).
