# Phase 9b1 final-fix re-review

Range `24696e667..d58e12600`, detached worktree at `d58e12600`, own target dir, `-j 2`, every run under memcap.

**Verdict: the fixes hold. No Critical, no Important.** Two Minor.

## Checked

- I1. Head: `collect_stress` `an_external_routines_merged_class_survives` passes (7 passed under filter).
  Mutant: `package_state.rs:125` becomes `let _ = merged_public_classes; for table in [classes, public_classes]`.
  `cargo test --release -p rexx-exec --test collect_stress --no-fail-fast`: 70 passed, 2 failed, the new test and
  `the_l0_subset_passes_again_under_collect_on_every_allocation`, both panicking at `dispatch.rs:1741:9` "a live value".
  The corpus program run plain under the mutant's `rexx-run`: same panic. Restored with `git checkout --`; `git status` empty.
  The main worktree was not touched.
- Head gates: `REXX_CORPUS_GATE=1 ... --test corpus` 29 passed, `1062 of 1062 matching`; `representation_refusals` 3,
  `refusal_sites` 5, `refusal_dispositions` 3 passed. The new corpus program prints `a The KX class` / `b 1 0` / `c kxv xx`, rc 0.
- M1/M2. All five sites (`executable.rs:431` `block_source_lines`, `dispatch/package.rs:485` `source_lines`,
  `environment.rs:1139` `build_package_string_table`, `context.rs` `build_internal_frame_at` and `build_native_frame_at` via
  `caller_depth`) now `debug_assert!` and then return `Loud::unknown_receiver`; no default or `unwrap_or(0)` remains.
  The remaining `programs.get(` sites (`call.rs:1052`, `lib.rs:2577`, `dispatch.rs:3216,3263`) already refuse with `missing_body`.
  Evidence: `grep -rn 'programs\.get(' crates/rexx-exec/src`.
  The `Result` change: grep of every call of the nine functions plus `rexx_variable` for `.ok()`, `unwrap_or`, `let _`, `.map(`
  finds only `merged_routine_object(merged)?.unwrap_or(ObjRef::NIL)` (`identities.rs:989,996`), where `?` is applied first and
  the `Option` default is the pre-existing `.nil` answer. Every other call uses `?` or returns the `Result` (`environment.rs:1050,1087,1097,1120`,
  `package.rs:656`). `rexx_variable` is called once (`environment.rs:560`) with `?`; the dot-variable order and `None`-on-no-program are unchanged.
  The refusal gate and `ALLOWED` rows pass (above).
- M3. `rexx-bench/src/lib.rs:76-83` and `package_state.rs:38` carry no task history or set size. No em-dashes added
  (`git diff ... | grep '^+' | grep -c '—'` is 0). New comments (`caller_depth`, the three "Refuses a ... that was freed" docs, the `ALLOWED` rationale)
  describe current behaviour.

## Minor

1. `environment.rs:1163,1166,1233` and `identities.rs:1076`: `build_package_string_table` now returns `Result`, but its allocation failure
   (`new_string_table`), missing `package_data_mut`, a failed `directory_put` and `find_written_routine`'s absent table still answer
   `Ok(None)` silently. They are pre-existing and outside the four named sites, but a failed table build is still skipped without a word
   (`~addRoutine`'s `.ROUTINES` write at `dispatch/package.rs:938`). Candidates to propagate the error now that the type allows it.
2. `rexx-bench/src/lib.rs:67-73`: three neighbouring entries (Phase 6.1, Phase 9a Task 8, 9a final review) still begin "Added in ...".
   They are outside the 9b1 range and outside the M3 finding; recorded only because the same rule applies.

Worktree and target dir removed.
