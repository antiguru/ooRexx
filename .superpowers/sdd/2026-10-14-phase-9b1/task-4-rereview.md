# Phase 9b1 Task 4 fix round 1 re-review (03e4cf1c7..bafd6fd43)

Verdict: all findings resolved, no open findings.

Method: `git archive bafd6fd43` copy with its own `CARGO_TARGET_DIR` under `/tmp` (deleted
afterwards), release build of `rexx-run`, runs under `memcap 4G timeout -k 5 20`, oracle runs per
global-constraints, each from a fresh empty directory (`ls -A` empty afterwards).

- I1 resolved. Writes to `PackageData::routines`: `install.rs:358` (`Installed` entries only, no
  object) and `dispatch/package.rs:920` (`install_routine`). The second is followed by the
  `.ROUTINES` table put, which a REXX copy now gets (`environment.rs:1134`). `~routines` answers a
  copy of the table on both sides, so a program cannot remove or replace a table entry behind the
  map (probe, identical output). Residual only if `directory_put` fails after the map insert, an
  allocation failure path, not reachable by a program. Mutant: `_ => return None` at
  `environment.rs:1141` turns `a_rexx_package_copys_added_routine_survives` red.
- I2 resolved. Order is own public, then registry, then merged. `lang/package_rexx_copy_lookups.rex`
  is identical to the oracle in stdout, stderr and status (rc 0).
- I3 resolved. One table with own entries replacing registry ones. Same program identical.
- Extra probes of mine, identical to the oracle (rc 0, stdout and stderr): a REXX copy that merges a
  package declaring `::class array public` and then `addPublicClass('ARRAY', .list)`, `addClass`
  of a registry name, importer and child-with-parent views; and routines added, replaced, public,
  copied and called after a 3000-pass filler.
- I4 resolved. Mutants, each in the archive copy, restored by `cmp`: dropping the `imports` trace
  turns `a_package_copy_held_as_an_import_survives` red (45 passed, 1 failed); dropping the `parent`
  trace turns `a_package_copy_held_as_a_parent_survives` red. Each ran the whole `collect_stress`.
- Minor resolved. `is_rexx_package` returns a loud refusal for a non-package; the callers
  (`package_find_public_class`) were updated and compile. The `options_armed` latch note was not
  asked to change. The report's concern 4 is now measured.
- Ruling-(a) witness `a_dropped_package_copy_is_collected` is present and passes; I did not mutate
  it (the global root it excludes no longer exists).
- No new defects found in the fix diff.
