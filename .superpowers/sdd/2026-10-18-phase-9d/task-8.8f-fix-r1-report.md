# Task 8.8f fix round 1: Reentrant collection consistency

## Scope and commits

This round addresses Important findings F1 and F2 from `task-8.8f-review.md`, using reviewed `d84ac3917` as FIX_BASE and verified dispatch HEAD `6849449c70726dc5e7077021c99971a75e2882a1`. Source changes are limited to `dispatch/hash/removal.rs` and `dispatch/hash/relation.rs`. The final source and fixture revision is `3e1aa9c879a90214bb5c680357975f58a0081abc`, with no new dependencies, unsafe code, descriptor interfaces, or scheduler changes. The original runtime pin remains unchanged, and no Deviation 84 or other new oracle departure is added.

Regression commits `ebcf44225` and `2beaa43cc` precede the red pin probes. Implementation commits `d7a279989` and `75db0f64a` repair F2 and F1 respectively, before their focused verification. Commits `8a488e529` and `e8ff00375` add the explicit send-count and unindexed Relation controls, pin the measured oracle counts, and apply formatting. Commit `3e1aa9c87` additionally roots saved backing-array identities across callbacks before final stress verification. All subsequent checks and release construction use frozen runtime source, with durable records under `docs/superpowers/records/2026-10-18-phase-9d/t8.8/semantics/fix-r1/`.

## F1: Return the value actually removed

A successful item comparison selects a live slot, but its callback can change the free list, prepend a collision, or overwrite the value in that slot. `take_at` now verifies that the backing arrays still belong to the receiver and re-finds the slot and predecessor in its current bucket chain. Removal uses that current free-list metadata and returns the value actually taken, which callers now propagate instead of returning the captured pre-callback item. A missing current location or replaced backing arrays makes the caller restart its search, while indexed Relation removal retains its cached hash.

The controller refined the boundary after paired controls showed that the oracle uses the current selected slot even when a callback changes its index. A Table overwrite returns `replacement`, and a Relation prepend or same-bucket key1-to-key2 reuse returns `extra`, because those are the values actually removed. An original-index identity restriction would instead invent a different result, so the implementation checks location provenance and current chain membership without imposing that restriction. This refinement follows the measured oracle and does not introduce a new departure or extend the pending count ruling.

| Probe | Original f | Oracle and corrected release |
| --- | --- | --- |
| `review-removeitem` | `removed victim items 2 victim-still-present 1` | `removed victim items 1 victim-still-present 0` |
| `removeitem-overwrite` | `removed victim items 0 victim-still-present 0` | `removed replacement items 0 victim-still-present 0` |
| `removeitem-reused` | `removed victim items 0 victim-still-present 0` | `removed extra items 0 victim-still-present 0` |
| `relation-removeitem` | `removed victim items 2 victim-still-present 1` | `removed extra items 1 victim-still-present 1` |
| `relation-removeitem-unindexed` | Added covering control | `removed extra items 1 victim-still-present 1` |

The e2 control reproduces the correct original Table insertion result, but its overwrite still returns the stale captured victim. On indexed Relation insertion, e2 returns NIL with both entries retained, which also differs from the oracle. These exact baseline results are retained in `e2/`, alongside separate stdout, stderr, and status descriptors. The corrected fixtures exercise both indexed and unindexed Relation removal, including current-chain changes that previously caused a skipped removal to be reported as successful.

## F2: Keep slots with their validated store

`slots_for` now returns the validated Store together with its selected slots, so ALLAT and indexed SUPPLIER never combine slot numbers with a pre-HASHCODE store. It computes HASHCODE once, snapshots and roots the bucket's index references, and validates the saved chain after observable equality mutates store state. Replacement arrays, changed chain links, or changed index occupants restart the search using that same hash. Saved backing arrays and captured indexes are temporary roots across callbacks, while items and supplier indexes are read after the search completes without another user callback.

Both ALLAT and indexed SUPPLIER have separate HASHCODE-replacement and equality-replacement fixtures. Each original f probe prints `answer old current new items 1`, while the oracle and corrected release print `answer new current new items 1`. The explicit equality-reentry counter control records `calls 1 2` on both sides, proving one HASHCODE and the two equality sends needed across replacement. The test's initial provisional one-equality expectation was corrected to the freshly measured paired result before its automated verification.

## Verification

All ten added fixtures run through the existing semantic-boundary harness both normally and with collection on every allocation. Previous conversion, raised-send, trace, hash collision, and reentry fixtures remain in that same run. The first covering run passes all 1,235 library tests and all three semantic tests, and the final run adds the relevant coverage, ownership, refusal-site, and representation checks. Complete successful corpus/suite/workspace runs are not repeated in this slice, because the controller owns those final gates and all-program own-BASE performance.

Every command below runs through `memcap 8G python3 /tmp/t88f-r1-check.py NAME BOUND CWD COMMAND...`, whose retained copy is `fix-r1/check.py`. It sets `CARGO_INCREMENTAL=0` and `CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target`, and all Cargo commands use `-j4` where applicable. Each command's JSON descriptor, independent stdout/stderr, status, duration, and bound are retained under `fix-r1/checks/`. Individual interpreter probes use `memcap 2G` and a 20-second timeout, while oracle runs use fresh working directories, a 1 GiB address-space cap, and the recorded oracle library path.

| Record | Exact covering command after wrapper | Status and duration |
| --- | --- | --- |
| `red-probes` | `python3 /tmp/t88f-r1-probes.py red /tmp/codex-p9d-t88-pins/8.8f-rexx` | 0, 0.87 s; descriptor differences confirm both findings. |
| `e2-probes` | `python3 /tmp/t88f-r1-probes.py e2 /tmp/codex-p9d-t88-pins/8.8e2-rexx` | 0, 0.97 s; retained baseline comparison. |
| `focused` | `cargo test -j4 -p rexx-exec --lib --test semantic_boundaries` | 0, 80.85 s; 1,235 library and 3 semantic tests. |
| `boundaries-policy` | `cargo test -j4 -p rexx-exec --test semantic_boundaries --test coverage --test refusal_sites --test owners --test representation_refusals` | 0, 40.99 s; 3, 21, 8, 6, and 3 tests respectively. |
| `format` | `cargo fmt --all --check` | 0, 2.52 s. |
| `lint` | `cargo clippy -j4 -p rexx-exec --all-targets -- -D warnings` | 0, 9.03 s. |
| `rooted-boundaries` | `cargo test -j4 -p rexx-exec --test semantic_boundaries` | 0, 37.89 s; all fixtures normally and under collection on every allocation. |
| `rooted-format` | `cargo fmt --all --check` | 0, 2.52 s. |
| `rooted-lint` | `cargo clippy -j4 -p rexx-exec --all-targets -- -D warnings` | 0, 8.83 s. |
| `release-final` | `cargo build -j4 --release -p rexx-exec --bin rexx` | 0, 90.70 s. |
| `final-probes` | `python3 /tmp/t88f-r1-probes.py final /tmp/codex-p9d-t88-pins/8.8f-r1-rexx` | 0, 0.97 s; all ten stdout/stderr/status triples match the oracle exactly. |

## Infrastructure recovery and handoff

The first staging attempt used shell globs inside an unyielded tool batch and stalled before staging or launching its probes. The controller interrupted it after twelve minutes, and inspection confirmed unchanged HEAD and an empty index, with only fixture files created. Explicit directory/file staging calls then completed immediately, with commits and check launches separated for visibility. That attempt is an infrastructure interruption, not a red runtime or implementation failure.

The copied check driver initially retained the original evidence output path and temporarily overwrote three earlier `focused` records. After the live check completed, its new records were preserved under `fix-r1/checks/`, and only the accidentally overwritten files were restored from their exact committed bytes. `evidence-recovery.json` records their SHA256 values and verifies equality against `d84ac3917`; the driver now names the explicit `fix-r1/checks` path. No successful runtime check was repeated merely to relocate those records.

An initial release build passed in 93.06 seconds before a final source review added conservative roots for old array identities retained across callbacks. Commit `3e1aa9c87` adds those roots in the same two files, and its semantic stress, formatting, clippy, and release checks are recorded separately. Earlier successful library and policy checks remain valid evidence for their unchanged contracts. The final pin uses the rooted source revision.

The added saved-store roots conservatively retain backing arrays across callbacks. As the controller clarified after the commit, comparing generational ObjRef identities alone does not require keeping their objects live, and stale arrays are never dereferenced after a mismatch. Current-store reads are already protected by the rooted receiver, while captured index references are retained through the snapshot validation. The explicit saved-store retention therefore remains a conservative lifetime/performance choice for scoped re-review, rather than a claim that identity comparison itself needs liveness.

The original `/tmp/codex-p9d-t88-pins/8.8f-rexx` remains SHA256 `0c9fb8371ab3d6dcea780194e02c112ed7981447f7f396e606e5f418975583f0`. The corrected `/tmp/codex-p9d-t88-pins/8.8f-r1-rexx` has SHA256 `b695b78fe24903b4be5a2874c8cd128193bbe165f92b50a4d6a8a9956d03d3be`; `fix-r1/release-pin.json` records its source revision, compiler, build descriptor, and source-file hashes. The reentrant ITEMS=-1 oracle defect remains the separate pending user/controller decision documented in the original report, with no new numbered ruling here. Scoped re-review, final gates, and all-program profiling remain controller-owned handoff work.
