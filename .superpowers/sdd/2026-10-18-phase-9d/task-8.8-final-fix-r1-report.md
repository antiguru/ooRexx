# Task 8.8 scoped R1 follow-up

## Scope and derived behavior

R1 corrects the closed-alias EXISTS regression identified by the retained scoped review, starting from `101182a1fe9a4dd5eeabd63958a8b94150d147bc`. The original F1/F2/F3 fixes remain intact, including OpenFile retention across CLOSE and the hash contents lifetime correction. Only `dispatch/stream/query.rs::query_exists` changes at runtime, as recorded by `final-review/fix-r1/source-proof.json` and `runtime.patch`. Regression commit `85406a754` precedes red execution, and implementation commit `9d919b7c3832c55edc3a0a07974309f092102201` precedes every green check.

The local rexxref QUERY EXISTS description at `oodocs/rexxref/en-US/streamclasses.xml:925` specifies the filename or the null string when no file exists. `StreamInfo::streamExists` at `interpreter/streamLibrary/StreamNative.cpp:3400` gates its descriptor shortcut on `isopen` and otherwise checks the named file, while `resolveStreamName` preserves a previously populated qualification. An owned stdout alias therefore must not infer existence from its retained OpenFile while status is UNKNOWN. The fix adds that status guard and preserves the cached qualification only after named-file metadata confirms existence.

The two retained probes distinguish missing files from actual `./STDOUT` and `./STDOUT:` files containing `abc`, with stdout redirected to an unrelated ordinary file. Before opening, an existing named file answers its absolute qualified path, whereas after opening and closing the alias it answers the cached bare name. Both oracle and the old pin confirm that naming distinction, and the old pin fails only the closed missing-file answers by returning bare names. The fixtures classify and assert empty, bare and fully qualified answers separately, preserving the distinction without embedding a temporary directory name.

Each fixture also checks READY versus UNKNOWN and SIZE, POSITION, HANDLE and STREAMTYPE at new, open, closed and reopened states. Closed missing files yield empty SIZE/POSITION/HANDLE and UNKNOWN type, while closed present files yield SIZE 3 with the same empty position/handle and UNKNOWN type. Opened aliases yield SIZE 0, POSITION 1, HANDLE 1 and PERSISTENT type, independently of the ordinary named file. The existing descriptor harness additionally requires empty stdout, zero physical length and final descriptor offset zero for these probes.

## Commands and retained checks

All records reside under `docs/superpowers/records/2026-10-18-phase-9d/t8.8/final-review/fix-r1/`, preserving the earlier wave's evidence. Commands use `memcap 8G python3 /tmp/t88-final-fix-r1-check.py NAME BOUND CWD COMMAND...`, whose exact command, environment, stdout, stderr, status and elapsed time are retained in `checks/`. The driver sets `CARGO_INCREMENTAL=0` and `CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target`, with Cargo `-j4` and foreground bounds no greater than 520 seconds. Runtime source remained committed and frozen throughout verification.

| Record | Command after wrapper | Result |
| --- | --- | --- |
| `red-probes` | `python3 /tmp/t88-final-fix-r1-probes.py red /tmp/codex-p9d-t88-pins/8.8-final-fix-rexx` | Status 0, 0.46 s. All six interpreter runs exit 0. Old/current pin disagrees with oracle only on closed missing-alias answers. |
| `red-descriptor` | `cargo test -p rexx-exec -j4 --test semantic_boundaries owned_stdout_aliases_keep_their_buffered_positions` | Status 101, 0.47 s. Committed missing-file regression fails on both closed aliases. |
| `green-stream` | Focused Cargo invocation with nonexistent target `stream_reopen` | Status 101, 0.06 s before compilation. Command-only error retained, corrected below without source changes. |
| `green-stream-corrected` | `cargo test -p rexx-exec -j4 --test semantic_boundaries --test standard_stream_reopen --test stream_handle --test stream_model` | Status 0, 43.01 s. 46 tests pass, including all 41 normal/actual-allocation-stress semantic fixtures and nine alias fixtures. |
| `policies` | `cargo test -p rexx-exec -j4 --test collect_policy --test coverage --test owners --test refusal_sites --test representation_refusals` | Status 0, 3.92 s. 40 tests pass without regeneration or allow-list changes. |
| `format` | `cargo fmt --all --check` | Status 0, 2.67 s. |
| `lint` | `cargo clippy -p rexx-exec -j4 --all-targets -- -D warnings` | Status 0, 9.28 s. |
| `release` | `cargo build -p rexx-exec -j4 --release --bin rexx` | Status 0, 92.67 s. |
| `final-probes` | `python3 /tmp/t88-final-fix-r1-probes.py final /tmp/codex-p9d-t88-pins/8.8-final-fix-r2-rexx` | Status 0, 0.47 s. Both probes exactly agree with the oracle on stdout, stderr and status. |

The semantic runner retains all earlier hash, conversion, raising and actual-GC controls, and the descriptor group retains the seven earlier buffered-alias controls beside the two R1 cases. Existing ordinary stream-model and HANDLE/reopen targets passed with no assertion changes. There are no added or renamed Rust test functions or targets, and no runtime changes outside the one query. No new semantic departure, dependency, unsafe site, state field or map is introduced.

## Release pin and handoff

The exclusive new pin is `/tmp/codex-p9d-t88-pins/8.8-final-fix-r2-rexx`, SHA256 `d9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145`, built from `9d919b7c3832c55edc3a0a07974309f092102201`. `python3 /tmp/t88-final-fix-r1-pin.py` completed with status 0, and its retained copy `pin.py` records the exclusive-create operation and preserved-pin checks. `release-pin.json` contains the complete source SHA, compiler `rustc 1.99.0 (b940084d7 2026-09-28)` with LLVM 23.1.1, source/build hashes and matching build/pin executable hashes. The original R pin remains `1e528101880f326e816e85715fb8c5401e45c62382a7b172dd113c050206ff6d`, and Q plus the four older listed pins were independently rehashed without mutation.

Final probes run the oracle in fresh directories with its documented library path, a 1 GiB address-space limit and `timeout -k 5 20`, while each Rust interpreter uses `memcap 2G` and the same timeout. Every process exits 0, all stdout channels are empty, and complete query reports remain independently recorded on stderr. `python3 docs/superpowers/records/2026-10-18-phase-9d/t8.8/final-review/fix-r1/verify-probes.py` exits 0 and reports six exact oracle/crate channel agreements, also comparing each oracle channel with its committed fixture expectation. `paired-agreement.json` records all channel hashes and preserves the old R pin's differing missing-file stderr.

This follow-up resolves the demonstrated R1 behavior without reopening the original fixes or adding a semantic licence. Its only retained check failure after implementation is the misspelled test-target launch, which executed no tests and was corrected before the successful focused run. Raw Cargo output, including trailing blank lines, remains unmodified as evidence, while source/report whitespace checks pass. R1-only review, new all-program performance measurement and broad final gates remain controller-owned, and no budget acceptance or Task 8.8 completion is claimed here.
