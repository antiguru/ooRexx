# Task 8.8e2: Executable identity and pool failures

## Scope

This slice follows the reviewed e1 changes. Read `task-8.8e-brief.md`, `task-8.8-dispatch.md`, `task-8.8-flags-investigation.md` and `task-8.8-p6-investigation.md`. Its assigned work is the active-redefinition executable-flags item, the failed pool-thread spawn item, and gate 481's non-class-scope invariant. Keep the other semantic residuals assigned to 8.8f.

## Executable identity

The original flags probe still fails when FOO is redefined or removed before the first `.context~executable` query. Lazy lookup substitutes BODY for the executing FOO copy. The investigation identifies the exact registered Method available at method entry and the existing `ActivationCold.executable` storage. Preserve that executing identity and its flags across replacement, removal, alias/copy cases and actual collection.

* Reuse existing object storage and GC roots. Add no numeric-id side map or representation refusal.
* Keep ordinary-method lookup and cold allocation costs visible. Record the all-program own-BASE performance obligation, and preserve a release pin after verification.
* Witness the original active-redefinition case, a first read before replacement, removal, alias/copy behavior and collection while the original method survives only through execution.
* Retain the existing licensed RESULT/collection-timing disposition separately.

## Failed pool reservation

Distinguish configured inline mode and test-only fixed saturation from an actual OS thread-spawn failure. Preserve the former policies and propagate the latter as the documented system-service failure, using the existing huge-stack failure seam where faithful. Native calls, commands and stdin are separate consumers and all require explicit proof. A missing worker must not make a blocking job run inline while holding the baton.

* Derive the error contract from the local reference and existing error catalogue before choosing the construction.
* Check ownership and cleanup when reservation fails. In particular, determine whether a command child already exists and ensure that returning the error leaves no owned running child or blocked pipe reader.
* Preserve pin/baton/caller state, input ownership and future progress after a trapped failure. Use bounded actual spawn failure rather than host resource exhaustion or a broad injection framework.
* Keep production-root and cancellation invariants intact, with no new unsafe site or dependency.

## Non-class method scope

Use the P6 producer/ownership analysis and a valid enhanced-metaclass probe. Establish the precise invariant for nil-scoped entries and their registered Method objects across collection. Preserve the current assertion unless actual supported construction reaches the fallback. A fabricated impossible internal Selection is not a language witness, and no reachable refusal becomes GUARD on agent judgement.

## Report and checks

Follow commit-before-verification, foreground bounds and memory caps. Run focused regression, scheduler/native/input/command and collection checks justified by the change, plus formatting and clippy. Keep full batch gates and aggregate performance with the controller, preserve the runtime release pin, and write `task-8.8e2-report.md` with exact evidence and remaining judgements. Close only these assigned items and retain all other open rows.
