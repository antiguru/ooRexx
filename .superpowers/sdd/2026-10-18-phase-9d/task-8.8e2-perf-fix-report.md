# Task 8.8e2 performance repair round 1

The repair replaces the existing weak executable registry with `MethodRows<ObjRef>` at source `1233b81da730142057fcf810e324d10d0e55c109`. The controller's completed original e2 comparison reports 12 programs over budget, with sendloop at +1.01245264%, dispatch at +0.67687242%, and dispatchclass at +0.73155584%, without spread or output errors. The [diagnosis](task-8.8e2-perf-diagnosis.md) attributes the ordinary-send increment to approximately 28 instructions per executable-index miss. This candidate removes hashing from an out-of-prefix miss when the existing minted map is empty, while preserving the exact entry-time activation root. Matched profiling and scoped review remain controller-owned acceptance steps.

## Representation and ownership

The authoritative registry now reuses the same storage helper as native entries and access scopes. Its vector seals after library bootstrap and at the direct object-model construction boundary, keeping later high MethodIds in the removable map while allowing older identities to fill vector holes. First-wins registration uses a noncollecting get/insert pair, and explicit replacement still overwrites the existing row. Keyed iteration preserves the minimum-identity fallback, while keyed retention removes dead-program identities and value retention removes swept objects. Neither half becomes a GC root, and the activation, class, and object ownership paths remain unchanged.

The new contract tests cover duplicate insertion on both sides of the seal, keyed pruning of equal values, holes, replacement, and independent tables with overlapping ids. A 9,990-identity insert/remove loop leaves the vector length and map row count unchanged. The interpreter test runs four successive copy-producing programs under each construction path, clears EXIT_VALUE_ROOT with a harmless return, and verifies that actual collection sweeps each copy and its registry entry. MethodId allocation remains local to each interpreter's ClassRegistry, and the test creates fresh independent interpreters for the two construction paths. Existing exact replacement/removal/alias tests and the valid enhanced-metaclass nil-scope ownership proof remain intact.

## Storage cost and retained limits

The actual library bootstrap creates a dense prefix of 1,032 slots with capacity 1,734, allocating 27,744 bytes for its `Option<ObjRef>` elements. The direct construction path keeps an empty prefix with zero capacity. Both capacities stay fixed through the repeated-program test, and the runtime map returns to its class-owned baseline after copies are collected. This is a measured prefix allocation and bounded-churn check, not a whole-interpreter memory or startup-time comparison against the prior HashMap. Collection and keyed fallback now scan that fixed prefix, so the new footprint and scan cost remain visible tradeoffs.

Registered-method calls still allocate and root through existing activation cold storage. When a program leaves runtime registry entries live, an out-of-prefix miss can still use the minted-map lookup. This slice does not claim those costs disappeared, and it adds no MethodId side map, dependency, unsafe code, refusal, or semantic departure. It changes no scheduler, native-worker, command, or stdin implementation and preserves f's semantic rulings. The controller must assess actual matched medians before accepting the performance repair.

## Verification

All final checks below ran against committed source `1233b81da730142057fcf810e324d10d0e55c109` with source frozen. The bounded driver records exact commands, source revisions, environment, elapsed time, separate output channels, and exit status in the [evidence directory](../../../docs/superpowers/records/2026-10-18-phase-9d/t8.8/runtime/e2/perf-fix-r1/). The first green run exposed an incorrect test expectation that all runtime entries should disappear, because the reflected `Object~STRING` Method is correctly retained by `class_owns`. The corrected test keeps that independently observed class-owned baseline while still requiring every unowned copy and its identity entry to disappear. An initial formatting edit command used the repository root instead of `rust/` and returned status 1 for missing Cargo.toml, then formatting from the correct directory and the committed formatting check passed.

| Record | Status | Result |
| --- | ---: | --- |
| method-rows-red | 101 | Expected compile red for missing keyed APIs and the old HashMap representation. |
| method-rows-green | 101 | Five tests passed; the class-owned baseline expectation failed. |
| identity-storage-final | 0 | 12 tests passed, including 201 identity and 90 enhanced-metaclass actual collections. |
| cleanup-worker-controls | 0 | 14 tests passed, including isolated native/command/stdin spawn failures, recovery and unused-worker release. |
| semantic-policy-controls | 0 | 33 tests passed, including normal/every-allocation semantic boundaries and all five selected policy integrations. |
| fmt | 0 | Workspace formatting check passed. |
| clippy | 0 | Affected interpreter crate, all targets, warnings denied. |
| release-build | 0 | Capped release build passed in 91.63 seconds. |

The refusal-site source check passed without any table change, so no metadata refresh was needed. The root-owned whole-suite, corpus, workspace, and all-program profile gates were not duplicated. No filesystem permission failure or approval wait occurred during this repair, and the user's archive and controller-owned performance records remain unstaged. The original e2 failed measurements and all prior pins are preserved. This handoff leaves no running command and makes no Phase 9 or Phase 10 closure claim.

## Commits and release pin

* `ce8f1e59b`: Pure move of the existing MethodRows tests into the adjacent test file.
* `8953b1165`: Storage and weak-registry contract regressions, committed before red.
* `92336e8dd`: Registry representation and compatibility adaptation, committed before green.
* `1233b81da`: Correct the test's class-owned Method expectation before final verification.

The new pin is `/tmp/codex-p9d-t88-pins/8.8e2-perf-r1-rexx`, with SHA-256 `470dcb41db61ab7a4d64f3276ce28e539d5686d130be245a4ec7b30e6d6725ac`. Its source revision is `1233b81da730142057fcf810e324d10d0e55c109`, with Rust subtree `8b6fc7672eaf5bcea5a38072043be49fc6cdd0eb`. It was built by rustc 1.99.0, commit `b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, and [release-pin.json](../../../docs/superpowers/records/2026-10-18-phase-9d/t8.8/runtime/e2/perf-fix-r1/release-pin.json) records compiler, Cargo, lockfile and build-record provenance. The pre-repair pin `/tmp/codex-p9d-t88-pins/8.8f-r1-rexx` was rehashed after the build and remains `b695b78fe24903b4be5a2874c8cd128193bbe165f92b50a4d6a8a9956d03d3be`. The controller receives source/Git ownership after the evidence commit and owns scoped review plus measured acceptance.
