# Reference DESCRIPTION registration repair

## Scope and preserved coverage

This repair applies the authorized eleven-file test/input change from BASE `5091eba39531c661c407c0d6a985a99b60d281a1`. The complete original `reference_conversion_raises_in_clause.rex` remains executable-line identical, with only its stale comment corrected, and now resides in unfiled under existing approved Deviation 83. Its full source, stdout, empty stderr and status 0 are pinned in `semantic_boundaries`, whose source-equality assertion prevents the corpus program and ruled fixture from drifting. No runtime source, Cargo input, feature, dependency or numbered Deviation changed.

The replacement differential program `reference_conversion_without_description.rex` preserves the original NUMERIC/SIGNAL cases and independently exercises noisy/bad references through ERROR RC and USER ADDITIONAL before bad-reference SYNTAX RC. Both ERROR cases and both ADDITIONAL cases reach their following markers without sending DEFAULTNAME, while the final SYNTAX RC sends once and raises 98.900 in its clause. Oracle and measured normal pin agree byte-for-byte on all three channels, with empty stderr and status 0. The full ruled program still differs from the oracle exactly where Deviation 83 requires DESCRIPTION conversion at RAISE, and its complete pin channels match the retained strict-corpus failure.

```text
    (dn 1)
s1 98.900 boom.
    (dn 1)
s2 98.900 boom.
handler ERROR
after nz error
handler ERROR
after bd error
handler USER ZZ
after nz additional
handler USER ZZ
after bd additional
    (bad dn)
s3 98.900 boom.
```

The block above is the companion's exact shared stdout, retained independently for both interpreters. Its source and the complete ruled source were committed before oracle/pin measurements in `8b819a716`. Registration and oracle-generated SOURCELINE snapshots, respectively 47 and 51 lines, were committed in `f201e6f67` before green checks. A formatter-requested line wrap was then committed in `f65066b04fe1f2355ce64d8be581fa9e79641673`, with no semantic or test-name change.

## Files and populations

`implementation/source-proof.json` lists exactly eleven changed files beneath `rust/`, and `source-proof.py` asserts that allow-list against Git. Its production manifest records unchanged Git objects and current SHA256 hashes for all 300 production `src/`, Cargo, build-script and Cargo-configuration entries selected from the Rust tree. All other Rust paths remain unchanged, including suite inputs and existing test names, and the manifest/lockfile proof keeps dependency and feature declarations fixed. The measured populations are 1242 differential programs, four unfiled programs and 42 semantic fixtures, with the same five Rust test functions in the amended target.

* `rust/corpus/lang/reference_conversion_raises_in_clause.rex`
* `rust/corpus/lang/reference_conversion_without_description.rex`
* `rust/corpus/phase-9.txt`
* `rust/corpus/unfiled.txt`
* `rust/crates/rexx-exec/tests/semantic_boundaries.rs`
* `rust/crates/rexx-exec/tests/semantic_boundaries/reference-conversion-raises-in-clause.rex`
* `rust/crates/rexx-exec/tests/semantic_boundaries/reference-conversion-raises-in-clause.stdout`
* `rust/crates/rexx-exec/tests/semantic_boundaries/reference-conversion-raises-in-clause.stderr`
* `rust/crates/rexx-exec/tests/semantic_boundaries/reference-conversion-raises-in-clause.status`
* `rust/crates/rexx-parse/tests/sourceline_oracle/reference_conversion_raises_in_clause.txt`
* `rust/crates/rexx-parse/tests/sourceline_oracle/reference_conversion_without_description.txt`

## Commands and evidence

The retained red evidence is `t8.8/final-gates/attempt-1/release-corpus.{json,stdout,stderr}`, whose strict corpus run exited 101 after 147.33 seconds with 1241 of 1242 programs matching. That whole-corpus run was not repeated here. New records live under `docs/superpowers/records/2026-10-18-phase-9d/t8.8/corpus-registration-repair/implementation/`, with exact commands, stdout, stderr, statuses and elapsed times in `checks/`. Commands use `memcap 8G python3 /tmp/t88-corpus-registration-check.py NAME BOUND CWD COMMAND...`, `CARGO_INCREMENTAL=0`, target `/tmp/codex-p9d-t88-target`, Cargo `-j4` and foreground bounds no greater than 520 seconds.

| Record | Command after wrapper | Result |
| --- | --- | --- |
| `probes` | `python3 /tmp/t88-corpus-registration-probes.py /tmp/codex-p9d-t88-pins/8.8-final-fix-r2-rexx` | Status 0, 0.26 s. Four interpreter runs, all status 0. Companion has three exact channel matches; full ruled program has three exact fixture/pin matches. |
| `sourceline-generate` | `python3 /tmp/t88-corpus-registration-sourceline.py` | Status 0, 0.06 s. Established oracle workflow generated both snapshots. |
| `semantic-coverage` | `cargo test -p rexx-exec -j4 --test semantic_boundaries --test coverage` | Status 0, 3.57 s. Five semantic tests and 21 coverage tests pass, including all 42 fixtures normally and with actual collection at every allocation. |
| `corpus-registration` | `cargo test -p rexx-exec -j4 --test corpus -- every_lang_program_is_run_or_named_unfiled the_differential_reads_every_phase_subset_file every_sidecar_names_a_program_the_subset_runs` | Status 0, 3.42 s. Three registration tests pass; differential execution filtered out. |
| `format` | `cargo fmt --all --check` | Status 1, 2.77 s. One long assertion line required wrapping. |
| `sourceline-test` | `cargo test -p rexx-parse -j4 --test sourceline_oracle` | Status 0, 0.47 s. All corpus source lines match generated expectations. |
| `format-corrected` | `cargo fmt --all --check` | Status 0, 2.47 s after the formatting-only commit. |
| `lint` | `cargo clippy -p rexx-exec -j4 --test semantic_boundaries -- -D warnings` | Status 0, 0.16 s. |
| `release` | `cargo build -p rexx-exec -j4 --release --bin rexx --message-format=json-render-diagnostics` | Status 0, 0.11 s. Cargo reused the normal production artifact, reporting `fresh: true`, empty package features and `profile.test: false`. |

Oracle runs used fresh directories, the documented library path, a 1 GiB address-space cap and `timeout -k 5 20`; interpreter probes used `memcap 2G` and the same deadline. `channel-agreements.json` retains the separate channel hashes, and the source-line generator is copied from the established workflow with only its output root and two source names changed. The formatting-only follow-up was checked by fmt and clippy, without repeating already-passing runtime tests for a whitespace change. No whole corpus, whole suite, debug workspace, profile, service or mutation experiment ran during this repair.

## Binary identity and handoff

The new exclusive pin is `/tmp/codex-p9d-t88-pins/8.8-corpus-registration-rexx`, SHA256 `d9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145`. Its bytes match both the normal production build artifact and measured normal pin `8.8-final-fix-r2-rexx`, with no new profile needed to characterize this unchanged executable. `release-pin.json` records full source commit `f65066b04fe1f2355ce64d8be581fa9e79641673`, rustc 1.99.0 (b940084d7 2026-09-28), LLVM 23.1.1, the Cargo artifact, build/output hashes and production-manifest provenance. The exclusive-create driver rehashed all 18 older pin files before and after copying, and every hash remains unchanged.

The original successful suite and its complete delta remain untouched, and the controller owns review of their input/artifact identity before reuse. This report establishes the narrow registration repair and unchanged production executable, not a new full-corpus or workspace pass. The controller will resume those gates after scoped coverage review and a new freeze. No additional semantic ruling or runtime concern arose in the companion measurements.
