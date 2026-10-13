# Phase 9b Task 9 re-review, fix round 1 (5dfb998ae..8726348ba)

Reviewer: t9-rereview. Every interpreter ran under `memcap 4G` from a fresh empty dir (oracle with `ulimit -v 1048576`). Worktree untouched; `target/release` rebuilt at HEAD (rexx-run). The I2 mutant ran in a git-archive copy with its own target dir, since deleted.

## Verdict: APPROVED. No open findings.

| finding | verdict | evidence |
|---|---|---|
| I1 | fixed | `probes/add1.rex` identical on both sides. `add2.rex` identical on every line except the known `copy` case (below). The corpus witness `rexxutil_library_identity.rex`, run on both sides, is byte-identical (a1, a2, REXX Filespec, a6 before/after reflection all `0 1`, `b.c 1`). `install_routine` now files `Installed` or `Internal(index)` with no directive proxy, and the `get_or_insert` is gone. |
| I1 consumers | no regression | Every reader of `routines` / `package_public_routines` (grep): `find_routine`, `find_written_routine` (identities.rs:937, 965), `routine_entries` (`~routines`, `~publicRoutines`), `merge_required` (install.rs:805), resolve (install.rs:902), the package-copy at identities.rs:1239-1246 (clones the map, type-agnostic), and the `install.rs:346-353` writers. All now carry `MergedRoutine`. User-defined path probe `u1.rex` (own `::routine` public and private, `.routines` identity, addRoutine/addPublicRoutine of a `.Routine~new`, items counts, `.Package~new` with `::requires library` plus a public routine, `addPackage` import, adding a library routine to a fresh package, sorted `allIndexes` of both tables, a non-Routine argument): oracle and crate identical, rc 0. |
| I2 | fixed | The first lines of the witness take `identityHash`, drop the object, allocate 50 strings and ask again. Mutant (delete the `add_global` in `internal_routine_object`) in an archive copy: `collect_stress the_l0_subset...` FAILED in 149.6 s, naming `lang/rexxutil_library_identity.rex`. At HEAD the report's run is green (37 passed). |
| M3 | fixed | `security_check` pushes the answered directory after `pop_frame`, with no allocation between `security_send` returning and the push. Callers (environment.rs:98, install.rs:746, dispatch.rs:2277, run/call.rs:557, command.rs:921, run.rs:4054) all pass `info` to `security_entry` with no intervening allocation, and no caller reaches `security_entry` without `security_check`. Temps join the caller's frame like the `index` temps the callers already push and never pop. `security_entry` no longer pushes. |
| M2 | fixed | The cell now reads "the standard input's answer: 1 0 in h3.rex, two lines left", which states what was measured. |

## Fix diff, new defects

None found. Only observation: `security_check`'s doc comment reflows awkwardly (a short line mid-paragraph); cosmetic.

## Out-of-scope observations

- The copy of a row's object (`u~copy`) is still refused with 88.914 where the oracle adds it. Pre-existing, documented in the exclusions row and queued to 9b1; confirmed in `add2.rex` (`copy 88.914` vs `copy added 1`).
- M1 from the first review (HANDLE:1/STDOUT swallowing write errors on a closed stdout) remains queued with the STDIN/standard-stream item.
