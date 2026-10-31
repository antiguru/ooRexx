# SysFileTree harness isolation

The unchanged SysFileTree group now passes 26 of 26 tests on both interpreters with separate populated child temporary directories. Implementation commit `6f2c1346eb1855c4d84301dab7fed3a4c844f0bf` selects only `base/rexxutil/SysFileTree`, seeds one regular file and one empty directory, and sets TMPDIR only on that child's command. Existing fixture checks pass, including refusal recovery, early termination, markers, child ownership and fallback limits. This harness-only repair does not change runtime semantics, protected tests, suite licences or performance acceptance.

## Commit sequence and contracts

BASE was `4714f6521f54e583e86c579bd803a80067a6939d`, with clean tracked state and only the untouched user ZIP untracked. Pure-move commit `0d8d1b29cd33846a4d93e658afaca2d25397e745` extracts the eight launch/lifetime functions into `suite/launch/mod.rs`, preserving their complete bodies, comments and public parent re-exports. Its existing fixture run passed 19 tests with one existing ignored diagnostic, and its complete target listing retained all 68 names. Regression commit `0333f95b1ca96e576d1216986ead3803e6f06c2a` preceded the red run, and implementation commit `6f2c1346e` preceded every green check.

The real-launch regression executes both oracle and crate routes before checking either result. Before its Rexx assertions, the synthetic fixture writes actual TMPDIR and observed names to a quoted external observation path outside the run copies and enumerated directories. The first red showed both interpreters ran one test normally with zero syntax errors, but failed the private-path assertion because TMPDIR was `/tmp`. Green verifies distinct owned paths, exact seed names, file/directory kinds, seven exact file bytes, matching File and SysFileTree scans, unchanged parent TMPDIR, cleanup of child work and survival of the external observation until fixture cleanup.

The implementation adds only the planned two helpers and reuses the existing owned run lifetime. `prepare_temporary_directory` immediately follows `prepare` on each route, and crate TMPDIR assignment follows `set_environment`, which clears the child environment. Every crate retry calls `crate_run_once`, whose existing `prepare` calls `fresh_copy_from` to remove and recreate the run, so each attempt reseeds from a clean directory. Diagnostics capture the selected TMPDIR and seed names before the existing cleanup, while the selection test excludes root-level SysFileTree, Windows SysFileTree and Stream without changing their environments.

## Scope and identity

Exactly five Rust test/fixture files changed relative to BASE, listed in `final-proof.json`. All 68 existing Rust test identities remain, and exactly `fixtures::tests::sysfiletree_launches_use_distinct_populated_child_temporary_directories` and `fixtures::tests::sysfiletree_temporary_directory_selection_is_exact` were added. Production Rust, Cargo inputs, corpus rows and licences remain unchanged, and no normal runtime rebuild or profile run occurred. All 553 protected ooTest files in the frozen static-input manifest retain their recorded SHA256 hashes, including SysFileTree `0e3e287344577115c7c1ba9a270bda0316d2cf72b7fd9c164e95dc295ddece22`.

The immutable normal pin `/tmp/codex-p9d-t88-pins/8.8-corpus-registration-rexx` retains SHA256 `d9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145`. The original-group table records oracle and crate expected/started/passed counts of 26, one attempt per side, empty failed/blocked sets and equal denominator. Separate raw stdout, stderr and context bundles retain each child's selected private path and seed names outside the enumerated directories. The original-group verdict is green, and the historical actor behind the shared-directory race remains unproven as the retained diagnosis states.

## Checks and retained infrastructure error

Every runtime check used committed source, the existing target directory, Cargo jobs four and CARGO_INCREMENTAL=0. All commands ran in the foreground under memcap 8G with bounds of 120 or 520 seconds, while the existing child routes retained their oracle address-space and crate memory limits. The first green invocation placed two test filters before Cargo's `--`, so Cargo rejected it with status 1 before running tests; the corrected command and that original error are both retained. No failed output was overwritten or normalized, and no optional broad suite, workspace or profiling check followed acceptance.

| Record | Status | Elapsed seconds | Result |
| --- | ---: | ---: | --- |
| `pure-move-fixtures` | 0 | 16.197 | 19 passed, one existing ignored |
| `pure-move-list` | 0 | 0.064 | 68 names unchanged |
| `format-regression` | 0 | 2.469 | Formatting edit before regression commit |
| `red-child-tmpdir` | 101 | 6.329 | Expected private-path failure on both child routes |
| `format-implementation` | 0 | 2.519 | Formatting edit before implementation commit |
| `green-child-tmpdir` | 1 | 0.032 | Cargo filter placement error, no tests ran |
| `green-child-tmpdir-v2` | 0 | 6.227 | Both new controls passed |
| `fixtures` | 0 | 10.185 | 21 passed, one existing ignored |
| `original-group` | 0 | 0.315 | Oracle 26/26 and crate 26/26, one attempt each |
| `fmt` | 0 | 2.519 | Formatting check passed |
| `clippy` | 0 | 0.716 | Affected suite target passed with warnings denied |
| `final-list` | 0 | 0.114 | 70 names, exactly two additions |

The raw check files are in `checks/`, with command arguments, selected environment, deadline, elapsed time and exit status in each JSON descriptor. `final-proof.json` maps these records to source commits, proves the exact population delta and records unchanged input and pin hashes. `manifest.json` records the byte counts and SHA256 hashes of the retained outputs, including raw whitespace. The following commands reproduce the launched checks, while broad validation and acceptance remain root-owned.

```text
memcap 8G python3 /tmp/t88-sysfiletree-isolation-check.py pure-move-fixtures 520 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust --env=REXX_SUITE_GATE=0 cargo test -j4 -p rexx-exec --test suite fixtures::tests::
memcap 8G python3 /tmp/t88-sysfiletree-isolation-check.py pure-move-list 120 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust cargo test -j4 -p rexx-exec --test suite -- --list
memcap 8G python3 /tmp/t88-sysfiletree-isolation-check.py format-regression 120 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust cargo fmt --all
memcap 8G python3 /tmp/t88-sysfiletree-isolation-check.py red-child-tmpdir 520 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust --env=REXX_SUITE_GATE=0 --env=REXX_SUITE_DIAGNOSTICS=/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-18-phase-9d/t8.8/sysfiletree-isolation/red-diagnostics cargo test -j4 -p rexx-exec --test suite fixtures::tests::sysfiletree_launches_use_distinct_populated_child_temporary_directories -- --exact --nocapture
memcap 8G python3 /tmp/t88-sysfiletree-isolation-check.py format-implementation 120 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust cargo fmt --all
memcap 8G python3 /tmp/t88-sysfiletree-isolation-check.py green-child-tmpdir 520 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust --env=REXX_SUITE_GATE=0 --env=REXX_SUITE_DIAGNOSTICS=/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-18-phase-9d/t8.8/sysfiletree-isolation/green-diagnostics cargo test -j4 -p rexx-exec --test suite fixtures::tests::sysfiletree_launches_use_distinct_populated_child_temporary_directories fixtures::tests::sysfiletree_temporary_directory_selection_is_exact -- --exact --nocapture
memcap 8G python3 /tmp/t88-sysfiletree-isolation-check.py green-child-tmpdir-v2 520 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust --env=REXX_SUITE_GATE=0 --env=REXX_SUITE_DIAGNOSTICS=/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-18-phase-9d/t8.8/sysfiletree-isolation/green-diagnostics cargo test -j4 -p rexx-exec --test suite -- fixtures::tests::sysfiletree_launches_use_distinct_populated_child_temporary_directories fixtures::tests::sysfiletree_temporary_directory_selection_is_exact --exact --nocapture
memcap 8G python3 /tmp/t88-sysfiletree-isolation-check.py fixtures 520 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust --env=REXX_SUITE_GATE=0 cargo test -j4 -p rexx-exec --test suite fixtures::tests::
memcap 8G python3 /tmp/t88-sysfiletree-isolation-check.py original-group 520 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust --env=REXX_SUITE_GATE=1 --env=REXX_SUITE_GROUPS=base/rexxutil/SysFileTree --env=REXX_SUITE_TABLE=/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-18-phase-9d/t8.8/sysfiletree-isolation/original-group/table.tsv --env=REXX_SUITE_DIAGNOSTICS=/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-18-phase-9d/t8.8/sysfiletree-isolation/original-group/diagnostics cargo test -j4 -p rexx-exec --test suite l3_core_suite -- --exact --nocapture
memcap 8G python3 /tmp/t88-sysfiletree-isolation-check.py fmt 120 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust cargo fmt --all --check
memcap 8G python3 /tmp/t88-sysfiletree-isolation-check.py clippy 520 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust cargo clippy -j4 -p rexx-exec --test suite -- -D warnings
memcap 8G python3 /tmp/t88-sysfiletree-isolation-check.py final-list 120 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust cargo test -j4 -p rexx-exec --test suite -- --list
python3 /tmp/t88-sysfiletree-isolation-proof.py
```
