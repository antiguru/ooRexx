# Task 8.8f performance repair round 1

## Scope and source

This repair implements the three candidates authorized by `task-8.8f-perf-fix-brief.md` from verified dispatch HEAD `b7a82272f3e111d131982854fdc2cb1ecb86acb5` on `plan/rust-rewrite`. The measured original f gaps remain parse +0.22055592%, dirread +0.36465390%, and pingmsg +0.07291944%, as documented in the diagnosis and controller-owned matched profiles. The supplied pre-repair pin is `/tmp/codex-p9d-t88-pins/8.8a-perf-r1-rexx`, source `f56dfd25e4394ca32c7cfb18733d640a4f584ed9`, SHA256 `ecb06e984ed736233ce0e2ef779b74941488ae64fdec10333dfb2215c309198a`. This round neither rebuilds that pin nor claims a measured performance recovery.

Commit `64a2e9a10` first extracts USE binding into `run/use_args.rs` and hash probing into `dispatch/hash/probe.rs`. `pure-move-proof.json` verifies body and comment identity against the dispatch HEAD after normalizing only the visibility required by child modules, and the committed extraction passes `cargo check`. Fixture commits `61ef99d9a`, `977a3482e`, and `14c486880` establish and correct the new preservation controls before implementation. Runtime commit `4c11ee4449a1fe7b537c2a216ee7c10b1a5b12ab` contains the three repairs and formatting, with source frozen throughout subsequent checks and release construction.

## Preserved contracts

PARSE String and Mixed triggers now consume decoded inline text directly and retain the existing local `Rendered` scratch representation for SmallInt. Those immediate paths omit the identity required-string call and duplicate temporary root, while heap and other values retain fallible `required_string_value`, the converted-result root, and rendering. The original evaluated-operand root and TRACE operation remain before conversion, and source bytes are borrowed only after conversion finishes. Positional triggers and required-string raising behavior are unchanged, with their existing controls included in verification.

Hash probing selects a const-specialized shared traversal by its existing key protocol. Only StringValue, whose hash and index comparison do not send, omits generation snapshots and callback restart state. The callback-capable specialization still caches one hash, reloads and validates the store after sends, and preserves the existing bucket, candidate, and predecessor results. `baseline-agreement.json` verifies that the F1/F2 `removal.rs` and `relation.rs` files remain byte-identical to the dispatch HEAD, including their conservative roots and actual-current-slot semantics.

USE binding retains a supplied value temporarily before assignment only when it is a heap reference. Immediate values cannot be collected, while default-expression roots and all heap-value retention remain in their prior locations. Result and assignment TRACE lines still render separately and propagate their own failures at the existing before-write and after-write points. The new raising witness proves both boundaries without introducing a new oracle ruling or changing Deviations 44 and 72/77/78/79/80/81/82/83.

## Boundary evidence

Three new fixtures cover inline text, integer, and heap PARSE patterns in ordinary and caseless searches, positional movement, supplied and default USE values, and two raising USE trace positions. The final baseline run compares the supplied immutable pre-repair executable with the oracle and matches stdout, stderr, and status separately for every fixture. The first draft exposed a missing parenthesized USE default and a trace assertion that itself sent another naming message, and the next draft exposed a trailing-comma line continuation. Those fixture mistakes were corrected and committed before runtime implementation, with the initial outputs retained under `baseline/` and `baseline-corrected/` rather than treated as runtime failures or red regressions.

Final fixture source is under `perf-fix-r1/probes/`, with the identical source and expected stdout in `tests/semantic_boundaries/`. `parse-immediate-boundaries` prints six `[left][right]` lines and `[ab][cdef]`, while `use-value-boundaries` prints `12 xy 32 supplied` and `34 ab 28 default`. `trace-use-raise` records the first naming failure with `assigned 0`, then the second naming failure with `assigned 1`, both carrying code 98.900. The existing semantic harness also executes every previous conversion, raised callback, collision, F1/F2 reentry, and TRACE fixture normally and with collection on every allocation.

## Commands and results

All bounded checks use `memcap 8G python3 /tmp/t88f-perf-r1-check.py NAME BOUND CWD COMMAND...`, retained as `perf-fix-r1/check.py`. The driver explicitly writes to this round's `checks/` directory, sets `CARGO_INCREMENTAL=0` and `CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target`, and records the command, status, elapsed time, bound, stdout, and stderr independently. Cargo build/test/lint commands use `-j4`, source remains frozen while they run, and every outer bound is at most 520 seconds. Probe subprocesses use a fresh working directory each, `memcap 2G` for this interpreter, and a 1 GiB address-space cap plus `timeout -k 5 20` and the documented library path for the oracle.

| Record | Command after wrapper | Result |
| --- | --- | --- |
| `pure-move-check` | `cargo check -p rexx-exec -j4` | Status 0, 5.77 s. |
| `baseline-final-probes` | `python3 /tmp/t88f-perf-r1-probes.py baseline-final /tmp/codex-p9d-t88-pins/8.8a-perf-r1-rexx` | Status 0, 0.31 s; all three channel triples agree. |
| `boundaries-policy` | `cargo test -j4 -p rexx-exec --test semantic_boundaries --test coverage --test refusal_sites --test owners --test representation_refusals` | Status 0, 44.64 s; 41 tests, including every fixture normally and under allocation stress. |
| `focused-lib` | `cargo test -j4 -p rexx-exec --lib -- parse_template use_ hash directory trace` | Status 0, 52.76 s; 129 tests. |
| `trace-gc` | `cargo test -j4 -p rexx-exec --test trace_oracle --test collect_policy` | Status 0, 1.72 s; 48 tests. |
| `format` | `cargo fmt --all --check` | Status 0, 2.52 s. |
| `lint` | `cargo clippy -j4 -p rexx-exec --all-targets -- -D warnings` | Status 0, 9.73 s. |
| `release` | `cargo build -j4 --release -p rexx-exec --bin rexx` | Status 0, 89.02 s. |
| `final-probes` | `python3 /tmp/t88f-perf-r1-probes.py final /tmp/codex-p9d-t88-pins/8.8f-perf-r1-rexx` | Status 0, 0.36 s; all three channel triples agree with oracle and baseline. |

## Release and handoff

The final pin is `/tmp/codex-p9d-t88-pins/8.8f-perf-r1-rexx`, SHA256 `6ed1c08eb03d54c48d1fe3947ce376f09502aaeba34ac221c334b01b1572b86e`, built from `4c11ee4449a1fe7b537c2a216ee7c10b1a5b12ab`. `release-pin.json` records `rustc 1.99.0 (b940084d7 2026-09-28)`, LLVM 23.1.1, the complete compiler descriptor, source and build hashes, and equality between the copied pin and build executable. The exclusive-create pin script rehashes the pre-repair, original f, and corrected f-r1 pins and confirms all three remain unchanged. Final probes compare each stdout, stderr, and status against both the oracle and the supplied baseline, with all nine comparisons retained in `final-agreement.json`.

No unsafe code, dependency, global root API, identity side map, scheduler behavior, or descriptor interface is changed. No Deviation 84 or new semantic departure is introduced, and the existing count-defect decision remains pending with the controller. Root owns the scoped source review, matched all-program performance measurement, broad final gates, and subsequent M6 work. The functional checks and pin support that handoff but do not establish that the three measured budget gaps are closed.
