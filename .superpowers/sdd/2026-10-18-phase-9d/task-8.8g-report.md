# Task 8.8g: M6 request attribution

M6 is caught by the compiled seeded REPLY witness, using the source proof's preferred program and the existing simulation and pinning reports. Both named baselines pass, deleting only `switch_to`'s SLICE clear makes the REPLY test fail on one incoming sorting-key deferral, and the separate RETURN control passes under the same mutant. The inverse edit restores identical production bytes and both named tests pass again, with seven scheduler mutant tests passing after the final formatting correction. This closes the scheduler-mutant queue by an activity-attribution invariant, without asserting a language scheduling-cadence defect or timer-mode equivalence.

## Source and scope

Execution started at `a9bf23ed4e072fe3b02b39942206cd03051df9ad` in the existing linked `plan/rust-rewrite` worktree, with a clean Rust tree and an empty runtime diff against reviewed `4c11ee444`. The retained addition is confined to the existing `scheduler/tests/mutants.rs` module and gated by the `pinning` feature. A shared helper constructs the exact witness, varying only `reply 1` to `return 1`, and runs it with `Invocation::none()`, the existing 60-second deadline and `sim:1,fifo,order=fifo,floor=1000000`. No timer hook, fabricated activity state, production semantic change, unsafe code, dependency or new release path was added.

| Revision | Change |
| --- | --- |
| `7662a51a396dfcca329719a2dd4ab4b0b0c0016e` | Tests committed before baseline execution |
| `0ee2685ecdfd2aa3477b3c774c3429a4d5f8eda7` | Exact deletion of only `self.timer.requests().clear(SLICE);` in `switch_to` |
| `3c76dbeebc3fd830ae0465a44aa17744539eef01` | Exact inverse restoration before restored execution |
| `56f5985a357d2d754821cecf48c12f36cfd114a4` | Formatting-only wrapping of two test assertions |

The clear's surrounding comment and the `slice_deferred` reset remain unchanged. The restored scheduler SHA256 is `320baed0cc7f226ec7823caf529efdc1283affac8707355a4b9e46ada1da7747`, equal to its pre-mutation SHA256, and the baseline-versus-restored diff is empty. Only test-module lines move in the retained source diff, so no production refusal or source-table location changes and no table refresh is justified. The existing release cache was never executed or retained as a runtime pin, and immutable f-performance pin `6ed1c08e` remains authoritative.

## Measured witness

Every run checks exit code zero, exact stdout `1\n1 2\n7\n` and empty stderr before checking the attribution counters. Each simulation report records nine clause steps and six contended steps, proving actual contention below the explicit million-step fairness floor. Each run has zero pinned yields and zero unbalanced pin frames, while the REPLY run records five switches and the RETURN control records four. The mutant REPLY run reaches the sorting-key assertion with exactly one total deferral and fails there, leaving the successful language output unchanged.

| Action | Status | Selected tests | Sorting-key deferrals | Total deferrals | Pinned yields | Unbalanced |
| --- | --- | --- | --- | --- | --- | --- |
| `baseline-reply` | 0 | 1 | 0 | 0 | 0 | 0 |
| `baseline-return` | 0 | 1 | 0 | 0 | 0 | 0 |
| `mutant-reply` | 101, expected assertion failure | 1 | 1 | 1 | 0 | 0 |
| `mutant-return` | 0 | 1 | 0 | 0 | 0 | 0 |
| `restored-reply` | 0 | 1 | 0 | 0 | 0 | 0 |
| `restored-return` | 0 | 1 | 0 | 0 | 0 | 0 |

The sorting key is `[Native("SORTWITH"), SortComparator]`, with absent map entries treated as zero. The helper additionally requires zero total deferrals, so another pin key cannot conceal a stale request. All six named invocations selected one test, and the separate control ran despite the primary mutant failure. These measured results confirm the proof's predicted REPLY producer path and require no change to its zero-deferral baseline contract.

## Commands and checks

Durable evidence is under [t8.8/instruments/m6](../../../docs/superpowers/records/2026-10-18-phase-9d/t8.8/instruments/m6/). Each action has separate `.stdout`, `.stderr` and `.json` files containing the actual Cargo argv, source revision, scheduler hash, elapsed time, status, environment and 540-second bound. The retained `check.py` is the `/tmp/t88g-check.py` driver used by the following plain invocations, with `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target` and Cargo `-j4` set internally. All commands ran in the foreground, and no check exceeded its bound.

```text
memcap 8G python3 /tmp/t88g-check.py baseline-reply
memcap 8G python3 /tmp/t88g-check.py baseline-return
memcap 8G python3 /tmp/t88g-check.py mutant-reply
memcap 8G python3 /tmp/t88g-check.py mutant-return
memcap 8G python3 /tmp/t88g-check.py restored-reply
memcap 8G python3 /tmp/t88g-check.py restored-return
memcap 8G python3 /tmp/t88g-check.py focused
memcap 8G python3 /tmp/t88g-check.py fmt
memcap 8G python3 /tmp/t88g-check.py fmt-final
memcap 8G python3 /tmp/t88g-check.py focused-final
memcap 8G python3 /tmp/t88g-check.py clippy
```

| Check | Actual Cargo command | Status |
| --- | --- | --- |
| Both focused runs | `cargo test -j4 -p rexx-exec --lib --features pinning scheduler::tests::mutants:: -- --nocapture` | 0, 7 passed each |
| Initial formatting | `cargo fmt --all --check` | 1, two new assertions needed multiline wrapping |
| Final formatting | `cargo fmt --all --check` | 0 after committed formatting-only correction |
| Affected all-target clippy | `cargo clippy -j4 -p rexx-exec --all-targets --features pinning -- -D warnings` | 0 |

The initial formatting failure is retained rather than overwritten by the successful final action. An initial compound `git diff && git add` invocation reached a read-only `index.lock` error, and the subsequent plain approved `git add` succeeded before the mutant commit. Read-only discovery also returned exit 2 for nonexistent guessed paths and `memcap --help`, and a metadata patch rejected a mismatched anchor without applying changes. No baseline premise failed, no assertion was weakened, and no live command is concealed by these records.

The whole staged whitespace check returned 2 for blank trailing stdout lines and the retained rustfmt diff's literal trailing space. Those files preserve command output byte for byte. The authored-file whitespace check passes separately without altering that evidence. These output-format findings do not change the focused test, formatting or clippy results.

## Disposition and handoff

The queued scheduler record now corrects the historical universal seeded-unreachability claim because REPLY publishes a request outside the restricted `serve_requests` premise. The earlier Task 8.8d report remains intact as historical evidence, and its compiled M7, M9, M10, M11 and M12 catches and restoration revisions are linked separately from this M6 catch. The queue's first line closes at restoration `3c76dbeeb`, and only columns 4 and 6 of its inventory row change to `closed:3c76dbeeb`, preserving the row's category, probe field and reason. The progress ledger and controller handoff append this correction without rewriting historical measurements.

Root retains the broad final gates, whole-suite delta, final performance acceptance and whole-8.8 review. This test-only slice runs no full suite or new release performance path, writes no Deviation 84 entry and advances neither Tasks 9/10 nor phase closure. Parent performance outputs and the user's archive remain unstaged by this slice. Source and index ownership return after the evidence commit, with all launched commands completed.
