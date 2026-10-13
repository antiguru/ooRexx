# Phase 9b final fix report

Base `43a39ab01`. Commits:

- `3da644f47`: I1 and M1, with oracle-crashes entry 47.
- `7d4743be9`: M3 and M4 (entries 37, 45 and 48), **and M2**. The M2 commit's `git add` named the
  gitignored queued file without `-f`. The `&&` chain stopped before its commit, and the next
  commit took the M2 files. The message names M3/M4 only. The changes are right, the label is not.
  I did not amend.
- `1590a1cea`: M6.

## I1: name-resolved stem rooted

- **The fix.** `stems.rs` gets `push_temp(stem)` after each `context_stem_named` resolution:
  - `stem_argument` (copy, delete, insert)
  - `sort_target`'s whole-name branch
  - `sort_target`'s `s.x` prefix branch, a third site the review did not name
- **The witness.** `builtin::rexxutil::stems::tests::a_stem_named_by_its_variable_outlives_the_variable`
  reads `crates/rexx-exec/tests/stem_dropped_by_name_cases/crate`, which has five cases: copy,
  delete, insert, sort, and sort by prefix.
  - Each case runs normally and under `run_program_collect_every_alloc`. The test asserts that the
    two outcomes are equal and that collections > 0. The expected block is the crate's own
    answer, which makes it a crate-only pin.
  - Values are 18 to 24 bytes, so they are heap strings.
  - The `STRING` method is `expose s.; drop s.`, then `copies('filler ', 50)`.
- **Red before the fix.** At HEAD the copy case gives rc 216 and
  `40.922 Stem argument does not have a valid count for element 0` under stress. The normal run
  gives rc 0, `0 2`.
- **Mutation matrix.** Each push was deleted on its own, and the case file was cut to one case per
  run:

  | push deleted | cases red | cases green |
  |---|---|---|
  | `stem_argument` | copy, delete, insert | both sorts |
  | `sort_target` whole name | sort | the rest |
  | `sort_target` prefix | sort by prefix | the rest |
  | none | none | all five |

  The script is `scratchpad/i1/mut.py`. Its other rows come from a wrong header split in the
  script and are not cases.
- **Oracle, entry 47.** 3 runs of 3 per variant, under the standard wrapper, `memcap 4G`,
  `timeout -k 5 60`, from a fresh empty dir:
  - **One small allocation in `STRING`:** every program matches the crate's pinned answer.
  - **200000 allocations:** copy, delete and insert end in SIGSEGV, rc 139. Both sorts answer rc 0
    as normal.
  - **200000 allocations and no `drop`:** rc 0 with the variable intact.
  - **This differs from the review.** The review measured 40.922 for its own program, which is
    not recorded. My program crashes instead. The entry records both.
- No corpus row.

## M1: two roots

The two tests are
`stems::tests::a_dump_keeps_the_values_of_variables_a_string_method_drops` and
`messages::tests::a_fresh_make_array_answer_outlives_its_values_string_methods`. Both run under
collect-every-alloc.

- **The dump test.** A method exposes five heap-string variables plus `z = self`, and its `STRING`
  drops all five.
  - Deleting the `variables` push loop (`stems.rs:730-732`) turns it red: panic at
    `dispatch.rs:1710`.
- **The `SysFormatMessage` test.** `MAKEARRAY` answers a fresh `.array~of` of objects, and each
  object's `STRING` allocates.
  - Deleting `messages.rs:250` turns it red: `firstfirstfirstfirst /  / ` against the full line.
- **Green at the fix.**
- **Oracle.** 3 runs of 3, rc 0. The dump's sorted lines match, including `SELF` and `SUPER`. The
  format line matches.

## M2: SysCls

- **The fix.** `process.rs:65` now passes `OsStr::from_bytes(interp.cwd_bytes())`.
- **The test.** `process::tests::sys_cls_runs_in_a_directory_whose_name_is_not_utf8` runs `cd` into
  a directory whose name is `odd` followed by the byte `ff`.
  - Before the fix it answered -1 there, printing `odd 0`.
  - After the fix: `odd 1`.
  - The adjacent success is a plain directory.
- **The queued file.** `.superpowers/sdd/queued/2026-10-13-lossy-path-sites.md` gains the review's
  measurements for two sites:
  - `DIRECTORY()`, `platform.rs:181`
  - the ADDRESS command working directory, `command.rs:621`

  It also lists `install.rs:1905`, which the `cwd_text()` grep finds and the review did not probe.

## M3: `~` with HOME unset

- **Entry 48.** From source, not run on the oracle.
  - The review cited `SysFileSystem.cpp:486`, the line in this checkout's `interpreter/`. At the
    oracle's `cb9563364` the line is `:569`. The entry gives both.
  - `FileNameBuffer.hpp:122` is the same line in both.
- **The pin.** `builtin::platform::tests::a_tilde_with_home_unset_qualifies_without_a_home`:
  - with `HOME=/h`: `/h /h/x`
  - after `'unset HOME'`: `[] /x`

## M4

- **Entry 37** now names `an_untrapped_error_inside_the_sort_is_reported` and its case file.
- **Entry 45** now names `tests/rexxutil_process.rs`'s
  `a_command_whose_child_was_reaped_elsewhere_fails_with_127`.
  - The program: the command touches a marker, then runs `sleep 0.6; exit 5`. The main activity
    polls for the marker up to 400 times at 0.05 s, then calls `SysWait`.
  - It asserts the sorted lines `cmd rc 127`, `failure 127`, `wait 1280`.
  - `timeout -k 5 60` wraps `rexx-run`. Clippy forbids `Instant::now` and `thread::sleep` in tests.
  - It passed 3 runs of 3 in its final form.

## M6

- **Semicolons** replaced with full stops:
  - `environment.rs:620`
  - `install.rs:633`
  - `stream_handle.rs`, three doc comments
- **`Cargo.toml:67`:** ` -- ` became `:`.
- **`lib.rs` `set_cwd`:** the false sentence about its callers is deleted. The doc is now
  "Moves it."

## Checks

All runs are at the committed tree, `1590a1cea`.

- **`cargo fmt --all -- --check`:** exit 0. The first check exited 0 only through a pipe. It was
  re-run after `cargo fmt --all`.
- **`cargo clippy --workspace --all-targets -- -D warnings`:** exit 0. Its first run failed on the
  disallowed clock methods, and that was fixed.
- **`cargo test -p rexx-exec --lib`:** 1121 passed, 0 failed.
- **`REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus --test collect_stress --test refusal_dispositions --test refusal_sites --test owners --no-fail-fast`:**
  exit 0.
  - corpus: 29 passed, 1 ignored
  - collect_stress: 37 passed
  - owners: 6 passed
  - refusal_dispositions: 3 passed
  - refusal_sites: 5 passed
- **Perf: not run.** The changes are the stem name resolution, `SysCls`, tests and prose, and none
  of them is on a bench path.

## Concerns

- **`7d4743be9`'s message omits M2**, which that commit contains.
- **Entry 47's oracle answer differs from the review's 40.922.** That is consistent with "depends on
  collection". The review's program is not recorded, so its 40.922 stays the review's
  measurement.
