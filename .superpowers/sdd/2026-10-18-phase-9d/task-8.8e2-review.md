# Task 8.8e2 review

## Spec compliance

**Spec compliant for the assigned e2 slice.** Reviewed BASE `a7db381c765fafd81fe775245902d6476aefe072` through HEAD `61b661504` using the task-reviewer template and supplied source review package. The implementation covers registered executing Method identity, actual pool creation failure for all three consumers, and the supported nil-scope construction. No high-confidence correctness or invariant finding remains in this review.

* `rust/crates/rexx-exec/src/dispatch/method_entry.rs:124` captures the exact resolved registered executable in the existing activation field. `dispatch/method_entry/tests.rs:32` exercises replacement, removal, hiding, prefetch timing, distinct copies, and actual collection through the two committed corpus witnesses. The retained original after-only, prefetch-installed, and read-before differentials agree independently on stdout, stderr, and status.
* `rust/crates/rexx-exec/src/scheduler/pool.rs:142` distinguishes configured inline policy from actual creation errors, with cold 48.1 construction at `:286`. `scheduler.rs:702` reserves before leaving the native frame or incrementing in-flight state, and `:2157` delivers failure through the parked continuation. `dispatch/library.rs:523` uses immediate frame cleanup only for unstarted calls.
* `rust/crates/rexx-exec/src/command.rs:832` reserves before child creation. The guard at `:729` returns unused workers through the existing no-op mechanism, including exec failure and unwinding. `command/tests.rs:128` verifies worker return, while `scheduler/tests/spawn_failure.rs:59` verifies repeated trapped failures, absence of command execution, and survival of an unrelated child.
* `rust/crates/rexx-exec/src/input.rs:644` requeues unrelated posts before returning failure, and `:661` resets only the unstarted read marker. `input/tests.rs:456`, `:494`, and `:529` cover partial bytes and state preservation, all four LINEIN/CHARIN surfaces, repeated failure, and recovery using pooled and configured-inline reads.
* `rust/crates/rexx-exec/src/dispatch/introspection/tests.rs:4` obtains its class-side selection from supported Rexx construction, clears the previous exit-value root, and runs actual collection with the class rooted. It checks nil scope, the live mapping, and introspection identity without a reflected Method or active invocation supplying an extra root. The production assertion and nil-scope lookup remain unchanged.
* The source diff adds no unsafe site, dependency, identity side map, or representation refusal. Existing RESULT/collection-timing disposition remains separate in `task-8.8e2-report.md:9`. The approved f handoff changes are documentation of the controller's ruling, not extra e2 runtime implementation.

## Inspected invariants

* **Executable lifetime risk:** checked unchanged `rust/crates/rexx-exec/src/activation.rs:1027` and `:1059`, which include the existing cold executable in activation roots. Checked the supported nil-scope producer at `dispatch/class_protocol.rs:1330`, ownership at `environment/identities.rs:859`, package retention at `:554`, and class-owned tracing at `rust/crates/rexx-core/src/body.rs:1532`. These support the tested producer/lifetime invariant without asserting that arbitrary internal registry misuse is valid.
* **Native cancellation risk:** checked unchanged `rust/crates/rexx-exec/src/dispatch.rs:3215`, which cancels the call before applying the original continuation's remaining work on failure. Checked `dispatch/library.rs:861` and `:900`, which release kept values and lent buffers, and `scheduler.rs:678` and `:1502`, which wake the original activity and exit only prepared calls. The new stage preserves the existing abandoned-frame path for calls that actually left.
* **Unused reservation risk:** checked unchanged `rust/crates/rexx-exec/src/scheduler/pool.rs:210` and `:246`. Sending the no-op returns a reserved worker to the ordinary idle/retirement path, including pool closure, without relying on dropping a bare Worker to stop its waiting thread.
* **Cutoff handling:** the diff supplied complete new method-entry and test modules. Its native continuation, cleanup, introspection lookup, and worker-loop context ended before the unchanged callees needed for the named risks, so only those focused ranges and the unchanged ownership/root producers were inspected. Initial tool-output truncation was recovered in bounded chunks, with no new Git diff or broad source traversal.

## Verification evidence

* Read retained red identity and spawn outputs: both exited 101 and exposed the claimed substitution/inline execution failures. Read final focused output and metadata: eight spawn-failure tests passed at `ae6a6bd04`. The isolated helper at `rust/crates/rexx-exec/src/test_process.rs:14` requires an actual reserve error under its address-space limit before exercising each failure case.
* Read `docs/superpowers/records/2026-10-18-phase-9d/t8.8/runtime/e2/affected-units-final.stdout`: 1234 library tests passed at `f38a037b6`. Later focused records cover the added nil-scope proof and final assertion adjustment. `nil-invariant-final.stderr:4` records 201 identity collections and 90 enhanced-method collections, with both tests passing.
* Read successful metadata-refresh, stdin-metadata, sourceline, final clippy, complete formatting, and release-build records. Final clippy output contains no warning. No duplicate successful test or build was run for this review.
* Compared stored final crate/oracle stdout, stderr, and status separately for both new witnesses and the three original identity probes: all agree. Read and hashed both retained binaries against `release-pin.json:1`: e2 `00e29fde4456ee4845a76941263f495b827c083af35363cb54eb3ac8c6b7c733` and e1 `434d6567458248fdb804eebe9edb715a54ddb9ded650fcca994eed6c4b691e06` match.

## Limits

**Controller verification remains open.** All-program own-BASE performance, full corpus, whole suite with deltas, and workspace checks are explicitly outside this slice gate under `task-8.8e2-dispatch.md:35` and the review dispatch. Method-entry lookup/cold allocation and native-stage costs remain unmeasured, as recorded in `task-8.8e2-report.md:44` and `release-pin.json:17`. This approval does not certify those later gates or independent command collector-thread creation failures outside `Pool::reserve`.

## Issues

No Critical, Important, or Minor correctness/invariant findings.

## Assessment

**Task quality: Approved.** The changes reuse existing activation roots, continuation delivery, native-frame cleanup, and worker return behavior. The retained tests distinguish real OS spawn failure from deliberate inline policy and verify recovery as well as error delivery. The nil-scope proof uses supported construction and actual collection while leaving the production invariant intact.
