# Task 8.8f performance repair round 2

## Scope and implementation

This round implements the narrow PARSE candidate authorized by `task-8.8f-perf-fix-r2-brief.md` from verified dispatch HEAD `f825415471a5f98635c09abda823bc8ab5199112`. The preserved P baseline is `/tmp/codex-p9d-t88-pins/8.8f-perf-r1-rexx`, SHA256 `6ed1c08eb03d54c48d1fe3947ce376f09502aaeba34ac221c334b01b1572b86e`, whose hash was checked before work. Runtime commit `cd418bd304e4bd1fedd339cb090ae861b1a460d8` restores separate String and Mixed arms using `required_string_value` followed by the prior packed `Rendered` path. The only retention change guards the converted-result temporary push with `needle != value`, and the initial operand root remains before TRACE and conversion.

`source-scope.json` verifies that `trigger.rs` is the only changed runtime file relative to the dispatch HEAD. USE, StringValue probing, and the F1/F2 removal and Relation implementations are byte-identical to that HEAD, including their existing roots and callback behavior. This round adds no tests that mirror implementation and instead reuses the established immediate, heap, raising, caseless, positional, TRACE, and allocation-stress controls. No global root API, unsafe code, dependency, side map, scheduler behavior, or semantic ruling is changed.

## Root lifetime argument

The original value is pushed immediately after `eval_in_parse`, before either TRACE naming or required-string conversion can allocate or send. `ObjRef` derives equality over its generational handle, so `needle != value` performs neither a language equality send nor allocation. `ActivityRoots::push_frame` records the current temporary length, and `pop_frame` truncates only the temporaries added after that mark (`rexx-core/src/roots.rs:455-465`). A matching answer therefore remains protected by the earlier operand root, while a distinct converted answer retains its own push before rendering.

A nested Rexx send runs its entered activation through `complete_send` (`dispatch.rs:3135-3147`), whose clauses open their own temporary watermark after the caller's existing roots (`run.rs:2578-2584`). Clause exit asserts that the nested work did not pop below that watermark before truncating its own temporaries (`run.rs:2610-2618`), and native pinned waits likewise bracket their roots with a later frame (`dispatch.rs:3161-3173`). The caller's PARSE clause remains active while conversion and rendering finish, so skipping its duplicate reference does not shorten the original value's lifetime. A conversion failure still propagates through `?` before the guard, and source bytes remain borrowed only after conversion and rooting have completed.

## Verification

Every build or test ran after the runtime commit with source frozen, under `memcap 8G python3 /tmp/t88f-perf-r2-check.py NAME BOUND CWD COMMAND...`. The retained `perf-fix-r2/check.py` explicitly writes this round's output directory, sets `CARGO_INCREMENTAL=0` and `CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target`, and records independent stdout, stderr, status, elapsed time, and the exact command. Cargo uses `-j4`, and the longest outer deadline is 520 seconds. The existing semantic harness exercises every fixture normally and with collection on every allocation, while focused library checks include late PARSE arguments and template-walk collection between targets.

| Record | Command after wrapper | Result |
| --- | --- | --- |
| `semantic-boundaries` | `cargo test -j4 -p rexx-exec --test semantic_boundaries` | Status 0, 37.08 s; 3 tests, including all normal/stress fixtures. |
| `parse-trace-lib` | `cargo test -j4 -p rexx-exec --lib -- parse_template trace` | Status 0, 50.96 s; 69 tests. |
| `parse-trace-oracle` | `cargo test -j4 -p rexx-exec --test trace_oracle parse_placeholder` | Status 0, 1.27 s; recorded PARSE trace witness. |
| `format` | `cargo fmt --all --check` | Status 0, 2.47 s. |
| `lint` | `cargo clippy -j4 -p rexx-exec --all-targets -- -D warnings` | Status 0, 8.68 s. |
| `release` | `cargo build -j4 --release -p rexx-exec --bin rexx` | Status 0, 91.34 s. |
| `paired-probes` | `python3 /tmp/t88f-perf-r2-probes.py final /tmp/codex-p9d-t88-pins/8.8f-perf-r2-rexx` | Status 0, 0.92 s; all 15 stdout/stderr/status comparisons agree. |

Five existing probes compare the oracle, preserved P executable, and new Q executable, with independent stdout, stderr, and status records under `final/`. They cover immediate text/integer and heap patterns, ordinary and caseless search, positional movement, makeString conversion, a raised DEFAULTNAME through a reference, trapped NOSTRING, and the existing long required-string boundary corpus witness. Every interpreter run exits 0, every stderr is empty, and `paired-agreement.json` records each channel's identical hash across all three sides. Oracle runs use fresh working directories, a 1 GiB address-space cap, the documented library path and `timeout -k 5 20`, while each Rust interpreter probe runs under `memcap 2G` with the same timeout.

## Emitted code and handoff

The ordinary inline pattern path now uses the prior packed representation, confirmed by disassembly of the immutable Q pin. At `0x3111eb` through `0x31120b` and `0x311447` through `0x311467`, the two arms extract the length, mask the handle bytes, combine them with OR, and perform the packed eight-byte store. The TAG_TEXT entries in the two conversion jump tables lead through `0x310c5d` and `0x310ce9` to those rendering blocks, bypassing both the duplicate-root push and its identity comparison. The comparison and conditional push remain after the rare conversion call at `0x310ea8`/`0x310eab` and `0x310efc`/`0x310eff`, preserving distinct-answer retention.

| Executable | `exec_parse` start | Symbol bytes | Stack reservation |
| --- | --- | ---: | ---: |
| A, before first f performance repair | `0x30f420` | 16,479 | `0x248` (584 bytes) |
| P, first f performance repair | `0x30f8c0` | 16,486 | `0x298` (664 bytes) |
| Q, this repair | `0x30f930` | 16,838 | `0x268` (616 bytes) |

P's identified scalar byte extraction and local copy are absent from Q's ordinary pattern path, and Q reserves 48 fewer stack bytes than P. The complete function nevertheless grows by 352 bytes and still reserves 32 more bytes than A, so neither total code size nor frame size establishes a performance result. `disassembly/` retains complete A/P/Q function listings, exact `nm` and `objdump` commands, hashes, and the decoded TAG_TEXT jump-table entries. This is static evidence for the targeted mechanism only, with no new profile run and no claim that matched own-instruction budgets pass.

The new pin is `/tmp/codex-p9d-t88-pins/8.8f-perf-r2-rexx`, SHA256 `22c317baa3ed111935a6577ac33097e428a53b64eb64b4697af2b85664e3f8eb`, built from `cd418bd304e4bd1fedd339cb090ae861b1a460d8`. `release-pin.json` records `rustc 1.99.0 (b940084d7 2026-09-28)`, LLVM 23.1.1, full compiler details, source hashes, the build descriptor hash, and equality of build and pin executable hashes. The exclusive-create pin script also rehashes P, A, original f, and corrected f-r1 and verifies that all four retain their previous hashes. `/tmp` had about 1.7 GiB free during testing and release construction, and this round encountered no space failure or cache/profile deletion.

The source and functional evidence are ready for controller-owned scoped review and full matched measurements. The original failed performance records and existing F1/F2 fixes remain intact, and no Deviation 84 or other semantic departure is written. Only the requested round's evidence and reports accompany the committed runtime change, leaving parent-owned performance records and the user ZIP unstaged. Final broad gates and performance acceptance remain controller work after this handoff.
