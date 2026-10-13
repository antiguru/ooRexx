# Phase 9b Task 4 re-review, fix round 1 (`1cbce6cde..ff222442b`)

Verdict: **APPROVED.** No open findings. Read-only; the tree was not edited.

## Ruled items

- **Qualifier ratification and witness.** `rexxutil_qualified_names.rex` is in `phase-9.txt`. Oracle and crate match on stdout, stderr and rc (158) under unset TZ, Europe/Dublin, Pacific/Auckland and America/New_York. Run from a fresh `mktemp -d` under `memcap 4G`; the oracle ran under `ulimit -v 1048576` and `timeout`. Resolved.
- **Date-time zone and gap rule.** `rexxutil_file_date_time.rex` matches the oracle on all three descriptors (rc 216) under the same four TZ settings. `datetime_zone` passes 22/22, including Zurich and New_York. The witness sets TZ itself with `VALUE(..., 'ENVIRONMENT')`: Dublin 2020, 2021 and 2045 gaps, the edges, the repeated hour, UTC, Kolkata, a POSIX rule, `:Europe/Zurich`, empty and an unknown name. `make_time` implements glibc's rule: a candidate that reads back wins, the earlier of two; for a skipped reading it takes the `is_dst` candidate, else the offset before the skip. Resolved.
- **`push_temp` dropped.** Gone at `files.rs:663`. Resolved.
- **Entry-39 test guard.** `Scratch` is unique per run and removed in `Drop`. Run: 1 passed, nothing left in `target/tmp` or `/tmp`. Cleanup on a failed assertion runs by unwinding, and the workspace sets no `panic = "abort"`. Resolved.
- **`datetime_zone` byte comparison.** It compares `Vec<u8>` for stdout and stderr, and renders lossily only in the failure message. Resolved.
- **`.File absolutePath` on an `ff` name.** Witnessed in the qualified-names file and matches the oracle. Resolved.

## Dependency

- `Cargo.lock` gains only `tz-rs 0.7.3` and the edge to it from `rexx-exec`. It has no dependencies of its own. `grep unsafe` over its `src/` finds nothing (`#![forbid(unsafe_code)]`), and our diff adds no `unsafe`.
- `tz::` is used only in `files.rs`. `program_zone` is called only in `get_file_date_time` and `set_file_date_time`, after the `stat` succeeds, so no other path pays for it.
- `TimeZone::local()` reads `/etc/localtime`, not the process `TZ`, which matches glibc for an unset program `TZ`. The run with TZ unset on the oracle and the crate agrees.

## New defects in the fix diff

None found. The removed "only external crate" sentence in `Cargo.toml` was already stale (rustix and libc are there).

## Not checked

- The records re-run at `ab305b53e` (suite, corpus, collect_stress). I ran none of them.
- Zones with a 30-minute daylight saving (Lord_Howe). The report says the whole witness was compared under it by hand; I did not repeat that.
