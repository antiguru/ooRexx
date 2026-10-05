# Task 20 re-review, fix round 1

Range `55dd10af8..a23624095` (HEAD `a23624095`). Reviewer: s4-t20-rr1. Scratch target dir deleted at the end.

## Verdict: APPROVED

0 Important, 0 Minor new findings. F1-F5: 5 resolved. Concerns: 1 not a flake in practice
(one residual, Note), 2 accepted as a Note, 3 pre-existing. 2 Notes in all.

## F1-F5

| Finding | Status | Evidence |
|---|---|---|
| F1 grown-buffer test raced its native | Resolved | `hold_buffer` writes `PATH.held` after both callbacks (`rexx-api/src/load.rs:919-923`); `grow` and `overlay` poll `PATH.held` first (`lent.rs`, `HELD`). `scheduler::tests::lent::` on default parallel threads: 60 of 60 green with four full-lib loops running alongside; `--test-threads=1`: 40 of 40 green. Oracle forge `grow.rex` 30 of 30 `abcd... / abcd 5026 / grown`, rc 0. |
| F2 false oracle claim | Resolved | Report's fix-round section says it was false and cites `NativeActivation.cpp:1304,1420`; forge `lent.cpp` carries HOLDBUFFER and FINISHEDINPLACE. I built it with `build.sh` and ran from fresh dirs: `grow` 30 of 30 and `growgc` 30 of 30 (`00000000 / abcd 5026 / grown`), `finishshort` 3 of 3 (`hello / helloxxxxx`), all rc 0, matching the report's table. |
| F3 row wording and placement | Resolved | The Phase 8 close row is deleted and DEVIATIONS entry 9 (`phase-4-exclusions.txt:1440`) is titled for the lifetime, says both agree until a collection, cites the forge and run counts. The oracle side of every sentence reproduced above (`growgc` 30 of 30). Spec 2.5's sentence is the review's replacement text, true as worded. |
| F4 FinishBufferString terminator | Resolved | `written()` returns every made byte plus the capped length; `finish_string` copies all made bytes and leaves the made-length NUL. Test now asserts `hello` and `helloxxxxx`. Mutant F4a (NUL written at the finished length after the copy), made by hand, whole `rexx-exec` lib `--no-fail-fast`: 994 passed, 1 failed, `finishing_a_buffer_string_keeps_its_kept_string_in_place` only. Reverted, `cmp` equal. |
| F5 abandon released what the native held | Resolved | `abandon_native_call` parks the frame in `Activities::abandoned` keyed by the frame token (`row << 32 | per-process-unique id`, so no cross-activity collision); frames are roots; `file_completions` ends it on an unmatched completion; an unmatched completion with no parked frame is a no-op. Mutant F5a (abandon ends the frame at once), by hand: 994 passed, 1 failed, `an_abandoned_calls_frame_holds_until_its_completion` ("the abandon freed the lent storage"). The scheduler test stays green under F5a, as the report said. Reverted, `cmp` equal. |

## Concerns

1. **F5 scheduler test's 0.3 s wait.** Not flaky in what I could produce.
   - 60 serial runs of the `lent::` filter with four full-lib loops alongside, 60 serial runs of the abandoned test alone, then 3 waves of 60 simultaneous processes of the abandoned test with 48 busy-loop burners on a 32-core machine: 0 failures in 300 runs.
   - It is still timing-shaped: a red there means the completion was not drained within the sleep plus whatever remains of the idle activity's 1 s, so it can only false-red, and only under a load far beyond the above. **Note N1**, not a finding. A deterministic fix, if wanted: loop in the Rexx program until a second marker the native writes after it returns.
2. **No test covers a native reading lent storage after an abandon.** Accepted, **Note N2**.
   - `fail_native_wait` fails the first wait before any callback is served, so no native can lend and read afterwards. The unit test observes the three things such a read would depend on directly (retired storage count, root, holder count), before and after `end_abandoned_call`, and F5a/F5c/F5d turn it red. The remaining gap is the read itself.
3. **F4 residual (later copy without an early `StringData` stops at the finished length).** Pre-existing, not introduced by Task 20.
   - At `2fa650384`, `finish_string` did `self.kept_strings.remove(&string)` and wrote the finished bytes only, so any later copy was made from the finished value. Task 20 (`6ceab29b0`) added the in-place update for an existing copy; `a48312f8e` extended it to the made bytes. The no-early-copy case is unchanged.

## Other verifications

- **F1's cause is as the fixer says.** The out-of-bounds read is in the test native: `hold_buffer` takes address and length in two callbacks, and a growth between them leaves old address, new length. The interpreter retired the old storage for its own length, and the oracle's two stubs have the same window. No interpreter bug sits behind it. With the rendezvous both callbacks precede any change.
- **Spec 2.5 sentence and DEVIATIONS 9:** true as worded; the oracle half reproduced by `growgc` (30 of 30 `00000000`). The Rust half (`61626364`) is asserted by the test suite's `a_buffer_grown_under_a_call_keeps_the_storage_the_call_holds`, green.
- **Mutants:** F4a and F5a re-run and real, as above. F5b-d and F4b not re-run.

## Check bar

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: no diagnostics, finished (its exit status was not captured because of a `| tail`; the output contained no warning or error line).
- `cargo test --workspace --release --no-run`: ok. `memcap 8G cargo test --workspace --release --no-fail-fast`: exit 0, 142 `test result` lines, 2987 passed, 0 failed.
- `git status` after the round: only the lead's `progress.md` modified.

## New findings

None.
