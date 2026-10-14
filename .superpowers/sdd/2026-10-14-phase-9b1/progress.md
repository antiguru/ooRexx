# SDD ledger — plan: docs/superpowers/plans/2026-10-14-phase-9b1.md

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
Plan written: docs/superpowers/plans/2026-10-14-phase-9b1.md (3e73e84b7), 9 tasks; awaiting Moritz review.

Moritz 2026-10-14: plan "lgtm". SDD execution starts.

## Pre-flight scan

| pair / task | shared file or interface | finding |
|---|---|---|
| T1 / T2-T8 | representation_refusals.rs ALLOWED list | each task shrinks it; T1 tags owners. OK |
| T2 / T3 | none (heap.rs, eval.rs, loops.rs, reqstr.rs vs environment.rs, native.rs) | OK |
| T3 / T4 | environment.rs routine-object table (T3 copy field; T4 moves the table into Package body) | T3 must keep the copy's row as a field of the Routine object, not of the package table. OK as written |
| T4 / T5 | Package body (T4 produces, T5 traces/frees) | OK |
| T4 / T6 | program handle choice (T4 Step 1) consumed by T6 | T6 brief carries T4's recorded choice |
| T5 / T6 | sweep hook; package roots | T6 relies on T5's collected packages. Ordering ruled in plan |
| T6 / T7 | ProgramId-keyed maps (T6) vs ObjRef-keyed exec/message maps (T7) | disjoint sets; T7's trigger charge for programs needs T6's program table. OK by order |
| T7 / T8 | side maps | T8 enumerates what remains after T4-T7 |
| T1 self | Deviation numbers 36, 37 next free after 35 | verify at dispatch |
| T5 self | TESTRECLAIM01 criterion "within its own wait" vs harness timeout | consistent |
| T9 self | memory criteria from T5/T6/T7 | consistent |
No conflicts requiring a ruling.
Task 1: dispatched (implementer opus, BASE 3e73e84b7).
Task 1: implementer DONE_WITH_CONCERNS (5292aad56). objectName= message drops 'is not implemented' suffix (grouping in corpus.rs/gate_tables); 4th phrase 'is value-based' added.
Task 1 review: spec OK; quality I1 gate misses 4 differently worded refusals (context.rs:513, class_protocol.rs:995, identities.rs:1006, lib.rs:689); I2 no task removes Task-8-tagged rows, so Task 9's "only Dev 32 and 36" criterion is unmeetable. Q1: keep new objectName= wording, no gate loses coverage. Minors: witness temporaries' lifetimes; Dev 36 note L1/Task 2; Dev 4 provenance dropped.
Ruling: add phrases "this crate did not", "this crate does not", "this crate built"; owners: context.rs:513 Task 6, class_protocol.rs:995 Task 7, identities.rs:1006 Deviation 28, lib.rs:689 Phase 10 -- each site's own subject -- if wrong, retag.
Ruling: plan amended (controller edit, no agent live on the plan): Task 8 Step 4 removes or proves-unreachable each Task-8 row; Task 9 criterion = ALLOWED holds only permanent rows (Deviation-licensed, Phase 10, proven-unreachable internal guards) -- the spec criterion names Dev 32 and objectName= as the allow-list's core, and "the plan derives the rest" -- if wrong, Task 8 grows.
Task 1: fix round 1 dispatched (resume c1-impl).
Task 1: fix round 1 at e680ef13c; controller verified diff (phrases, owner tags, witness, Deviation notes) instead of a re-review -- mechanical list and doc edits, gate test self-checks every row -- if wrong, close review.
Task 1: complete (5292aad56, e680ef13c).
Task 2: dispatched (implementer opus, BASE 8db45ca6d).
Task 2: implementer DONE_WITH_CONCERNS (0a055271c, 2fbfe9cba, 7c756e734). arith +0.12% after receiver-mode operand_number in value.rs (first form +0.996%).
Ruling: oracle clearing a WeakReference to held .true/.Array/.environment/1 is an oracle bug, not copied (crate keeps them; Deviation 36 wording stands) -- if wrong, nothing to change in crate.
Ruling: class-side DEFAULTNAME not consulted when a class renders (pre-existing) -> queued.
Task 2 review: spec fails I1 (DO step + in controlled_step_wide converts first; oracle DoBlock.cpp:194 sends each step); M1 VarRef receiver arm unwitnessed; M2 full-set callgrind not recorded (reviewer: all under, hashnew +0.31%). Fix round 1 dispatched (resume c2-impl).
Task 2: fix round 1 at 1566bcbe2, 59c3ec337 (one-word loops.rs fix, witness lines, full-set callgrind exit 0, worst hashnew +0.31%). Controller read the diff instead of a re-review -- the fix is the reviewer's own verified one-word change -- if wrong, close review.
Task 2: complete (0a055271c, 2fbfe9cba, 7c756e734, 1566bcbe2, 59c3ec337).
Task 3: dispatched (implementer opus, BASE 485c13720).
Task 3: implementer DONE_WITH_CONCERNS (0bce0e10c, 25d47538e, 3536722be). Callgrind 11 programs under budget (hashnew +0.299%).
Ruling: native METHOD frames missing from live stackFrames (TESTNATIVECODEFRAME) -> queued (send-hot-path sibling of L11; needs its own perf budget work).
Ruling: filespec not sending STRING to an object arg (oracle 88.909) -> queued.
Task 3 review: spec PASS; I1 two of three new root tables (routines, package_public_routines) have no red test; I2 internal-routine calls (filespec loop) +4.51% whole-process vs Task 2 close, ~121 Ir/call in run_internal_as copying name+args, no bench program reaches it; q2 not covered by entry 37 (93.900 from element STRING inside sort, untrapped -> SIGSEGV). Shared scope slot sound. Minor body.rs:1088 doc.
Ruling: I2 is a regression on a common path even though no bench program covers it: remove the per-call copy, and add a bench program (intcall: an internal-routine call loop, e.g. filespec) to rust/bench-programs so the budget guards it; it must be within +0.5% of the pin -- the budget's intent is every hot path, the program set is its instrument -- if wrong, one extra bench program.
Ruling: I1: delete the root entries no test can witness and say what holds them, or witness them; q2 -> extend entry 37 if same defect else entry 49; fix the doc.
Task 3: fix round 1 dispatched (resume c3-impl).
Task 3: fix round 1 at 8f4ff5096, c0ffb9391, 386a4797f, a094cad6b, 1cd3928eb. intcall -10.73% vs pin (+2.78% vs Task 3 base, 67 Ir/call); extcall +0.23% vs Task 3 head. Package~routines now reads a written .ROUTINES table (oracle). Re-review dispatched (sonnet).
Task 3 re-review: PASS, no open findings (lent buffer swapped back on every exit; re-entry via next slot; intcall -10.73% vs pin reproduced).
Task 3: complete (0bce0e10c, 25d47538e, 3536722be, 8f4ff5096, c0ffb9391, 386a4797f, a094cad6b, 1cd3928eb).
Task 4: dispatched (implementer opus, BASE 097038252).
Task 4 question: package ~copy rooting. Ruling: (a) copies unrooted, collectable when unreachable -- oracle answer, no new root, (b)'s only test would pin a leak -- if wrong, Task 5 adds nothing. pkg_copy_weak witness flips to the oracle answer in Task 4.
Task 4: implementer DONE_WITH_CONCERNS (4a5b8b571, fc0266803, 7f26fbd71, b2ae5bd48, cd01d7fd6, 03e4cf1c7). (a) chosen (matches ruling). Body access via type_id downcast; startup +0.3219% (headroom 0.18%). REXX-package-copy class-merge shape unmeasured.
Task 4 review: spec fails (REXX-package copy not "like any other"); I1 REXX copy addRoutine of a row/library Routine collected (routines field untraced, REXX copy has no .ROUTINES table); I2 REXX copy findClass/findPublicClass skip addPackage-merged classes; I3 merge from REXX copy redefining ARRAY keeps .array (oracle .list); I4 imports and parent traces have no red test now that copies are collectable. Perf: Task 4's own share of startup +0.034%. Minors: options_armed process-wide; is_rexx_package treats non-package as REXX.
Ruling: "required packages" (spec bullet) read as the per-package imports field plus the global requires cache (Task 5 makes it weak) -- accepted.
Task 4: fix round 1 dispatched (resume c4-impl).
Task 4: copy-witness 2cf1141c2; fix round 1 at 5194b8428, bafd6fd43 (REXX copy gets .ROUTINES table on addRoutine; findClass order own-public, registry, merged; merge table single; imports/parent traces witnessed; is_rexx_package Result). startup +0.3229%. Re-review dispatched (sonnet).
Task 4 re-review: resolved, no open findings.
Task 4: complete (4a5b8b571, fc0266803, 7f26fbd71, b2ae5bd48, cd01d7fd6, 03e4cf1c7, 2cf1141c2, 5194b8428, bafd6fd43). startup +0.3229% vs pin (Task 4's own share +0.034%).
