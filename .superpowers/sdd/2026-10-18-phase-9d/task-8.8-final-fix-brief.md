# Task 8.8 whole-review fix wave

## Scoped follow-up R1

The scoped review at `9d7da3281` marks F1/F2/F3 addressed and identifies one fix-induced EXISTS regression, retained at `t8.8/final-review/rereview.md`. The controller amended the task-local single-wave stop in the progress ledger to finish this narrow defect under the existing correctness authorization, without another broad review. An owned stdout alias retains OpenFile across CLOSE for position state, but QUERY EXISTS must use the unopened named-file fallback while its status is UNKNOWN. Commit regressions before red execution and the narrow guard correction before green verification, preserving the retained state and already-passing alias behaviors.

Cover missing and present named-file fallback for closed STDOUT and STDOUT: aliases in the existing owned-descriptor harness, with opened positive cases and related closed metadata/status controls. Use a literal path such as `./STDOUT` for the ordinary named-file fixture so reserved standard-stream names do not invalidate the observer. Reuse the established normal/stress and stream checks, and limit production edits to this status/fallback defect unless a directly related problem is demonstrated. Build a fresh immutable pin at `/tmp/codex-p9d-t88-pins/8.8-final-fix-r2-rexx`, preserving the previous pin, then append the R1 report and return a clean tracked handoff for scoped R1-only review.

The complete whole-source review found three Important issues, retained at `docs/superpowers/records/2026-10-18-phase-9d/t8.8/final-review/review.md`. Root verified their decision points against current source and local contracts before dispatch. The initial consolidated fix wave and its scoped review are complete, with the narrow R1 follow-up authorized above. The original f implementer owns all three corrections, with no other source or index owner active.

## Findings and scope

* F1: `dispatch/stream.rs:2341` bypasses unopened-standard handling in the owned-file stdout position shortcut. Preserve null POSITION for closed/unopened `.stdout`, `STDOUT` and `STDOUT:` receivers after validating options, with opened stdout as the positive control.
* F2: `dispatch/stream.rs:2351` omits the queried receiver's `StreamState.held` from logical WRITE CHAR position. Account for buffered aliases without double-counting interpreter output or changing SYS descriptor semantics, and retain before/after flush controls.
* F3: `dispatch/hash/relation.rs:250-254` reads a captured Store's item array after observable index equality can replace it. Validate current storage and relevant slot state after callbacks before another read, reuse existing proof/search helpers, and restart with the already computed hash when required. Cover continuation after item equality too, without changing the established index-before-item callback protocol.

Use the complete findings verbatim from the retained review. Derive expected semantics from local rexxref, compare permitted oracle programs in fresh directories with separate stdout/stderr/status, and stop for any new oracle departure rather than assuming Deviations 82/83/84 cover it. Existing hash-reentry and opened-stdout witnesses remain controls. Read `corpus/oracle-crashes.txt` before selecting any oracle probe.

## Implementation and proof

Commit regression tests before their red execution, then commit implementation before green verification. For the unwieldy stream module, preserve the user-required early pure-move step by extracting the query responsibility into an adjacent module with unchanged public entry points and all comments retained, before changing behavior. Keep that move independently inspectable and limit its scope to the responsibility needed for these fixes. No new unsafe sites, dependencies, numeric-id maps, broad framework, or unrelated optimization is authorized.

The regressions must execute actual host-owned redirected stdout and report query values without implicitly opening/flushing the stdout receiver. Cover unopened and closed state, alias forms, buffered held bytes, logical CHAR and descriptor SYS, invalid-option precedence, and opened positive cases using existing descriptor harnesses. HASITEM witnesses must exercise index equality replacing an emptied/repopulated Relation, continued searches after equality, one HASHCODE computation, and normal plus actual collection-on-allocation runs. Reuse established semantic-boundary and collision/Bag controls where applicable instead of building another harness.

Run focused tests covering the amended code, semantic-boundary normal/stress checks, formatting, affected-crate clippy and a release build. Preserve the first red/failed records, exact commands, exit statuses, selected test counts, source commits and binary hashes. Use `CARGO_INCREMENTAL=0`, Cargo `-j4`, `/tmp/codex-p9d-t88-target`, `memcap 8G` for builds/tests and bounded separate foreground driver actions at most 540 seconds. The saved-prefix invocation is plain `memcap 8G python3 DRIVER ACTION`, with environment, timeouts and descriptor capture inside the driver.

Build an immutable new runtime pin named `/tmp/codex-p9d-t88-pins/8.8-final-fix-rexx`, retaining its full source commit and SHA256. Preserve the Q pin, raw profiles and all historical evidence. The separate profiler will measure all 33 programs against Q after scoped approval, and root owns the subsequent broad final gates, whole-suite delta, performance disposition and original flakes. Do not rerun prior mutation experiments or the whole suite within this fix wave.

## Coordination and handoff

Verify the root-supplied BASE before any source or index write, and remain in the existing linked branch worktree. Write a fix report beside this brief with each finding's reproduction, derived rule, tests, commands, results, commits and any unresolved concern, and retain cited raw records under `t8.8/final-review/fix/`. Do not stage another owner's records, the user's archive or ignored Python bytecode. Do not spawn subagents, push, merge, remove caches or change unrelated worktrees.

Return short status, source/evidence commits, immutable pin/hash, a test-result line and actual concerns. Source/index ownership returns to root only after a clean tracked handoff, with any foreground command that cannot finish reported BLOCKED rather than abandoned. Root will verify status/log, covering test outputs and the exact net fix diff before scoped rereview. No final-gate freeze exists yet, and no original-budget acceptance or Task 8.8 completion may be claimed.
