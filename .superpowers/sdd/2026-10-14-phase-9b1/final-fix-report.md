# Phase 9b1 final review: fix report

Base `24696e667`. Commits:

| commit | finding |
|---|---|
| `244b38561` | I1: `collect_stress` test and corpus program for the merged public class |
| `8fa4a5694` | M1, M2: freed-program and missing-caller guards refuse; refusal gate phrase |
| `1d733bb0c` | M3: comment prose |

All commands ran from `rust/` with `CARGO_INCREMENTAL=0`. Scratch: `final-fix/` in the session
scratchpad.

## I1

- Oracle, s18 as the review wrote it (3000-pass filler, fresh dir `final-fix/s18`): rc 0,
  `a The KX class` / `b 1 0` / `c kxv xx`, stderr empty. s18 is not in `oracle-crashes.txt`.
- Added `an_external_routines_merged_class_survives` to `collect_stress.rs`: the s18 shape with
  `ext18.rex` beside it through `assert_stress_prints_beside`, with `EXTERNAL_FILLER`.
- The sibling (`package_external_merge_held.rex`, Task 5 fix round 2) is also pinned in the corpus,
  so this one is too: `corpus/lang/package_external_merge_class_held.rex` (writes `extclass.rex` with
  `lineout`, 1000-pass filler), listed in `corpus/phase-9.txt`, with its
  `rexx-parse/tests/sourceline_oracle/` file. That file came from the module-doc driver run by the
  oracle on a scratch copy (`final-fix/srcg`). Its prolog ran there and wrote `extclass.rex`, not in
  the repository. The lines are `cmp`-identical to the program.
- The corpus program on its own: oracle rc 0, crate (fresh `target/release/rexx-run`) rc 0, stdout
  and stderr `cmp`-identical.

Mutant: `package_state.rs` copied to `final-fix/package_state.rs.saved`, then
`for table in [classes, public_classes, merged_public_classes]` became
`let _ = merged_public_classes;` + `for table in [classes, public_classes]`.

- `memcap 8G cargo test --release -p rexx-exec --test collect_stress --no-fail-fast`: 71 passed,
  1 failed, and that failure is `an_external_routines_merged_class_survives`, panicking at
  `dispatch.rs:1741:9` ("a live value").
- The `rexx-run` binary built by that run (14:55:56) gave rc 101 with the same panic on the corpus
  program in a plain run. I first read that as a head failure. A rebuild after the restore (15:01:14)
  gives rc 0, so the plain corpus program is also red under the mutant.
- Restored by copying back the saved file. `git diff` afterwards showed only the M3 line.

## M1

- `PHRASES` gains `"whose program was freed"`, and so does the module-doc grep. Plain `"was freed"`
  would also match the `debug_assert!` messages (`identities.rs:180` "...after its program was
  freed", and the new ones), which the scanner reads as literals too.
- `ALLOWED` gains `identities.rs:a package whose program was freed` as an internal guard. The reason
  is beside the rows: a program is freed only at the collection after its last package object goes,
  and every package object and executable holds its program, so only a holder the collector was not
  shown reaches a freed one.
- Controls, each run on a copy (`final-fix/rr.saved`) and restored with `cmp`:
  - with the identities row removed, `every_representation_refusal_is_on_the_allow_list` FAILED,
    listing `rexx-exec/src/environment/identities.rs: "a package whose program was freed"`;
  - with the phrase removed, the same test FAILED, listing the four `whose program was freed` rows as
    stale. So without the phrase, the gate cannot see these literals.

## M2

Each site follows `package_object`'s shape: a `debug_assert!`, then
`Loud::unknown_receiver(...)` in release.

- `dispatch/executable.rs` `block_source_lines` returns `Result`: "an executable whose program was
  freed".
- `dispatch/package.rs` `source_lines` returns `Result`: "a package whose program was freed". The
  REXX package (`program` `None`) still answers no lines.
- `environment.rs` `build_package_string_table` returns `Result<Option<_>>`: "a package whose program
  was freed". Its other `None` exits (not a package, no entries without `force`, the table's
  allocation or put failing) are unchanged as `Ok(None)`. The `Result` reaches `package_string_table`,
  `program_string_table`, `package_string_table_for_write`, `rexx_variable` (split, with the
  activation names moved to `activation_variable`), `routine_object`, `merged_routine_object`,
  `find_written_routine`, `package_find_routine` and `rooted_routine_objects`, and their callers.
- `dispatch/context.rs`: `build_internal_frame_at` (the named site) and `build_native_frame_at`
  (`:907`, which the review says it copies) share `caller_depth`, which refuses with "a stack frame
  whose calling activation is no longer live" where `.unwrap_or(0)` stood. `live_levels` lists these
  levels only beside the activation their caller names. The `:907` site was not on the brief's list.
- `ALLOWED` gains internal-guard rows for each new literal.
- `REXX_REFUSAL_SITES_REFRESH=1 cargo test --release -p rexx-exec --test refusal_sites`: 5 passed,
  `git diff --quiet corpus/refusal-sites.tsv` exit 0. The table is unchanged because no `Loud`
  constructor changed.

Reachability probes, run from fresh dirs in `final-fix/` against the oracle, release `rexx-run`, and
a debug `rexx-run` (linked 15:16:59, after the edits, so the `debug_assert!`s are live). Each has a
1000-pass `copies('x', 100000)` filler before it reads:

- `guards/g.rex`: `~source`, `~sourceSize`, `~sourceLine` of `Package~new`, `Routine~new` and
  `Method~new` packages and a routine from a dropped `Package~new`; `addRoutine` on a package copy
  with `findRoutine` and `~routines`; a written `.ROUTINES`, then `findRoutine`, `.METHODS` and
  `.RESOURCES`; `addPackage` of a dropped package with an added routine. Oracle rc 0, release rc 0,
  stdout and stderr identical; debug rc 0.
- `guards2/h.rex`: condition `stackFrames` from `filespec` and `SysStemSort` (internal levels) and
  `sortWith(.nil)` (native level), and `.context~stackFrames` in a `sortWith` comparator. Oracle and
  release rc 0, identical except `f4`: the crate has no `SORTWITH` frame under `COMPARE`, which is
  the queued `2026-10-14-native-method-frames` item (it names SORTWITH). Debug output is identical
  to release's.
- `package_external_merge_class_held.rex`: debug rc 0.

No probe reached a guard.

## M3

- `rexx-bench/src/lib.rs`: the three "Added in Phase 9b1 Task N's fix round:" prefixes are deleted,
  and what each line said about its program stays. The earlier Phase 6.1 and 9a lines were not in the
  finding and are untouched.
- `package_state.rs:38`: "which all three `setSecurityManager` setters write" became "which the
  `setSecurityManager` setters write".

## Checks

| command | result |
|---|---|
| `memcap 8G cargo test --release -p rexx-exec --test collect_stress external_routines` (before M2) | 7 passed |
| `memcap 8G cargo test --release -p rexx-exec --lib --test collect_stress --no-fail-fast` | exit 0; lib 1135 passed, `collect_stress` 72 passed |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus --no-fail-fast` | exit 0; 29 passed, 1 ignored; `1062 of 1062 matching` |
| `memcap 8G cargo test --release -p rexx-parse --test sourceline_oracle` | 1 passed |
| `memcap 8G cargo test --release -p rexx-exec --test representation_refusals --test refusal_sites --test refusal_dispositions` (after fmt) | 3, 5, 3 passed |
| `cargo fmt --all --check` | exit 0 (after `cargo fmt --all` rewrapped the new `debug_assert!`s and one `let ... else`) |
| `cargo clippy -p rexx-exec -p rexx-bench -p rexx-parse --all-targets -- -D warnings` | exit 0 |

The lib run had no `a_move_across_file_systems_copies_and_unlinks` failure this time. The review
saw that one as environmental. The full workspace gate, the debug gate and callgrind were not run,
per the brief.

## Concerns

- `rexx_variable` now returns `Result` on the dot-variable path, and `.METHODS`/`.ROUTINES`/
  `.RESOURCES` are matched before the activation names. Nothing was measured. The close's callgrind
  covers it.
- `build_native_frame_at` (`:907`) was changed along with the named `:955`.
- M4 and M5 were not in the brief and were not touched.
