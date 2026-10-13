# Phase 9b Task 7, fix round 1: re-review

Diff `fe5191171..f55cf090d` (`fix1-fe5191171..f55cf090d.diff`). Read-only; probes ran from fresh empty
directories under `memcap 4G`, the crate with `rust/target/release/rexx-run` (no source file newer
than it), the oracle with `build/bin/rexx`.

## Verdict: approved, no Critical or Important findings.

## Findings under verification

- **load_requires follows the oracle: fixed.** `PackageClass.cpp:1319`/`:184` resolve from the original
  target; `InterpreterInstance::loadRequires` (`:1021-1045`) checks the instance cache under the
  original name, then under the resolved name (adding the original as an alias), and only then calls
  `PackageManager::loadRequires`, which sends the manager the short name and then the resolved one.
  The crate now does the same: `original` is bound first, the cache is consulted under it and under
  `found` (resolved from `original`) before the manager, the alias is inserted on a `found` hit, the
  manager's short-name answer is only looked up, and the search, every 43.901 and the cache entries use
  the original (also on the failure-removal path). Not-found names match `:1328`.
- **Cache before manager: fixed.** Witness section "a file loaded twice" prints `2 events` after each
  of three loads on both sides.
- **Callers** (`install.rs` `install_requires`, `load_package`, `load_package_global`): no regression.
  With no manager `check_requires_access` returns the name unchanged, so the flow is the old one with
  one resolution reused. Probes (identical output and rc on oracle and crate): `::requires` of a
  relative path, the same file by two spellings (`sub/b.cls`, `sub/b`, `./sub/../sub/b.cls`), a nested
  require from the required file's own directory, `Package~new` of a relative path twice (same
  identity), a missing file (43.901 naming the original, rc 213), and a failing `call` afterwards.
  Library `::requires` goes through `require_library`, which the diff does not touch (the crate's
  `::requires "rexxutil" library` answers 98.903 where the oracle loads it; that is the existing
  refusal documented at `install.rs:2124`, not this round).
- **Witness** `lang/requires_manager_rename.rex`: oracle and crate stdout and stderr byte-identical, rc 0
  both, and equal to the committed `fix-1b/requires-witness.oracle.out`. The crate leaves only the
  program in its directory.
- **SysFork:** no `unsafe` in the diff (`git diff ... | grep '^[+-].*unsafe'` prints only removed
  prose and the removed `unsafe_grant`; no reference to it remains). `rexx-run` prints the new message
  and rc 120. The reason is the one measured (`fix-1/threads.rex`: three threads) and matches the queued
  design note.
- **pty failed-read test:** real. `read_key` on a write-only pipe fails with EBADF, so it exercises the
  `None` arm; it asserts `None` and equal local modes, and `read_key` restores after any read, so a
  restore-only-on-`Ok(1)` mutant leaves ICANON/ECHO cleared and turns it red. It passes
  (`cargo test -p rexx-exec --lib rexxutil::process`, 6 passed). I did not re-run the mutation.
- **Doc minors** (wait doc, `cls` doc dash, `rexxutil.rs` module doc): done.

## Minor

1. `.superpowers/sdd/queued/2026-10-13-sysfork-design.md` is untracked (`git ls-files --error-unmatch`
   fails) but `lib.rs`, `process.rs` and `oracle-crashes.txt` cite it by path. Commit it with `add -f`.
2. The code comment at `install.rs` names `PackageClass.cpp:1319` for resolution; correct, but the
   `newRexx` path (`:184`) is not named. Cosmetic.

## Out-of-scope observation

The first cache lookup runs before the manager is asked, so a forbidding manager is not consulted for
an already loaded package. That is the oracle's order (`:1024`), and `TEST_REQUIRES_HANDLE_FORBIDDEN`
passes.
