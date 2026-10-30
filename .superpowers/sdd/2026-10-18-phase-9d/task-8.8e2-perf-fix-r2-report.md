# Task 8.8e2 performance round 2: Rejected experiment

The controller rejected the retention-block reorder because the emitted code retained both targeted bookkeeping costs and lengthened the inspected ordinary path. Candidate commit `2cfd41033375edb95af4080710f46784e4947b02` moved only the unchanged retention block below package application, keeping it before flags, activation push, tracing, guard waits and user code. Restoration commit `c7ceb632c2c34e6a37e375182a239058587429dc` reverses that move by explicit edit. The full restored Rust subtree equals reviewed source `4c11ee4449a1fe7b537c2a216ee7c10b1a5b12ab`, with tree hash `9830e71cb6ef547b63e4e1b804bee8cf4598b020`. No round-2 runtime pin or whole-program profiles were created.

## Exact code-generation result

The comparison uses the actual supplied f performance pin and the candidate release build, rather than assuming H1's addresses still apply. The pre-repair pin is `/tmp/codex-p9d-t88-pins/8.8f-perf-r1-rexx`, SHA-256 `6ed1c08eb03d54c48d1fe3947ce376f09502aaeba34ac221c334b01b1572b86e`. The rejected candidate build hashes to `0a9651475867a24716c080301e3421e676c04c80503342d3d936ed978eadff7a`, and [candidate-build.json](../../../docs/superpowers/records/2026-10-18-phase-9d/t8.8/runtime/e2/perf-fix-r2/candidate-build.json) records its source, compiler, Cargo, lockfile and build-command provenance. Both binaries contain the package None tag and the 472-byte stack-to-stack callee copy before `push_activation`. The slightly smaller function size therefore does not demonstrate a cheaper ordinary send.

| Observation | Pre-repair | Rejected candidate |
| --- | --- | --- |
| `begin_invoke` start and size | `0x26c340`, 8,526 bytes | `0x26c340`, 8,513 bytes |
| Package None-tag initialization | `0x26d70b` | `0x26d700` |
| Package tag comparison/branch | `0x26dab6`, `0x26daba` | `0x26d991`, `0x26d995` |
| Hit-only `cold_mut` call | `0x26daa2` | `0x26db04` |
| Callee `memcpy`, size `0x1d8` | `0x26db54` | `0x26db57` |
| `push_activation` call | `0x26db60` | `0x26db63` |
| Inspected ordinary-path instructions | 89 | 92 |

The static path starts immediately after replacing the call context and ends at the `push_activation` call. It assumes no package options are armed, the resolved MethodId is above the dense prefix, and the minted map is empty, counting each called function as one call instruction without its body. The candidate adds a zeroing instruction, a stack load and an unconditional jump on its no-options branch, and the four registry miss compare/branch instructions remain. [ordinary-path-counts.json](../../../docs/superpowers/records/2026-10-18-phase-9d/t8.8/runtime/e2/perf-fix-r2/ordinary-path-counts.json) names the exact endpoints and taken branches, with the complete instruction lists and disassembly retained alongside it. This is a local static count, not a measured whole-program runtime delta, but it fails the stated code-generation objective and supplies no basis for accepting the reorder.

## Correctness controls

`PackageStart::apply` assigns only `settings`, `condition_syntax` and optional `trace_mode` into the local callee. It has no interpreter reference, send, yield or collector call, and the candidate still retained the same resolved executable before making the activation observable. Existing exact replacement/removal/alias and enhanced nil-scope tests supplied the identity coverage, while existing corpus programs supplied numeric inheritance and package trace-entry coverage. No additional witness was needed for this pure move, and no test assertion was weakened. The successful controls establish semantic preservation for the experiment, without establishing a performance benefit.

| Record | Exit | Result |
| --- | ---: | --- |
| identity-gc | 0 | Four exact identity/nil-scope controls passed, including 201 identity and 90 enhanced actual collections. |
| fmt | 0 | Formatting passed. |
| source-policy | 0 | Eight refusal-site and three representation-policy tests passed, with no table refresh. |
| release-build | 0 | Candidate release built in 120.36 seconds under the 8G cap. |
| options-differentials | 0 | Three existing package-options witnesses matched oracle stdout, stderr and status independently. |
| clippy | 0 | Affected interpreter crate, all targets, warnings denied. |

The option witnesses are `directive_options.rex`, `directive_options_numeric_inherit.rex`, and `directive_options_trace_method.rex`. Their output sizes are respectively 99/0, 49/0 and 0/541 bytes for stdout/stderr, with status 0 on both engines. Every probe ran from its own fresh directory under the 2G memory cap, and the oracle wrapper also imposed the established 1GiB address-space ceiling after checking the crash inventory. Exact commands, source hashes, output channels, statuses and bounded-driver programs are retained in the [evidence directory](../../../docs/superpowers/records/2026-10-18-phase-9d/t8.8/runtime/e2/perf-fix-r2/). No semantic ruling, public flag, registry representation, activation layout, f fast path or F1/F2 fix changed.

## Restoration and remaining gap

The [restoration record](../../../docs/superpowers/records/2026-10-18-phase-9d/t8.8/runtime/e2/perf-fix-r2/restoration.json) verifies both the full Rust subtree and the method-entry bytes against the reviewed pre-experiment source. `method_entry.rs` hashes to `d9b58102d88d57e4823669a5977df2cea3e9688fb9a68b25b834ed9bdd31049b` on both sides, and the restored Rust worktree is clean. The original f performance pin was rehashed after restoration and remains unchanged, while `/tmp/codex-p9d-t88-pins/8.8e2-perf-r2-rexx` does not exist. The build-cache path `/tmp/codex-p9d-t88-target/release/rexx` still holds the rejected candidate until a future build, so it is not the retained runtime artifact. Exact source restoration required no additional runtime checks, as directed by the controller.

This rejected experiment does not close the H1 residual gap. The completed H1-versus-original-e1 medians remain dispatch +0.12091429%, dispatchclass +0.13069467% and sendloop +0.18086132%, with the cross-source/intervening-f attribution limit from the [residual diagnosis](task-8.8e2-perf-r1-residual-diagnosis.md). Those measurements describe H1 and are not a new measurement of the restored f performance revision. The controller retains whole-program profiling, acceptance decisions and any further repair scope, with no broader redesign performed here. Source/Git ownership returns after the evidence commit, with no live command, new deviation or phase-closure claim.
