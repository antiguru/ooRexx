# Task 8.8 final test maintenance

Both stale assertions now pass at source commit `794dbc6297a3a4471cc2a50e56c33ac65c2a514a`. The dispatch-seam inventory adds exactly the three proven moved consumer files and rejects four synthetic unlisted sibling paths through the same predicate used for actual source sites. The TraceObject check requires the ten named passing rows and exactly ten total rows, retaining outcome classification, masks, mode selection and refusal behavior. This is test-only gate maintenance, with runtime correctness, broader validation and performance acceptance remaining under root's ownership.

## Source and scope

The starting revision was `47c5a5ea4fbc9518b41ed640d25c3baa86212269`, with clean tracked state and only the existing user ZIP untracked. Pure-move commit `5f281e8266cc5613382be9f54cee27e92682aa26` extracts the complete `group_runs` and nested `whole_groups` responsibilities into ordinary module files, retaining their existing module paths and comments. Exact reconstruction of the original module body is recorded in `pure-move-source.json`, and the committed move compiled with all 63 historical test names unchanged before assertion edits began. The following assertion commit also applies rustfmt to the extracted modules, preserving their semantics apart from the two authorized test repairs.

The only changed Rust files relative to BASE are the four below. Existing calibration code remains byte-identical, and no production `src/`, Cargo, corpus, feature or dependency inputs changed. The TraceObject fixture remains byte-identical to its SVN pristine copy, SHA256 `98e4999d07fca459f1d2a13793a5e5d15b5b2dba82460c615e410d0fdcd0d9b3`. The measured normal pin `/tmp/codex-p9d-t88-pins/8.8-final-fix-r2-rexx` remains SHA256 `d9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145`.

* `rust/crates/rexx-exec/tests/concurrency_tests.rs`: replace extracted module body with its plain module declaration.
* `rust/crates/rexx-exec/tests/group_runs/mod.rs`: extracted responsibility and exact ten-row TraceObject expectation.
* `rust/crates/rexx-exec/tests/group_runs/whole_groups.rs`: extracted nested responsibility, with existing paths and behavior preserved.
* `rust/crates/rexx-exec/tests/dispatch_seam.rs`: three explicit consumer paths and four unlisted-path negative assertions.

## Retained failure and covering checks

The existing resumed default-run failures supplied the red evidence without rerunning either stale assertion. Their exact paths, sizes and SHA256 hashes appear in `final-proof.json`, and the two provenance diagnoses are retained beside that proof. The first extraction script encountered an existing `group_runs` directory and stopped at its directory-absence assertion before any write, after which the guard was narrowed to the two new files. This was an infrastructure-only guard failure, and the existing calibration descendant was neither replaced nor edited.

Every covering command used the committed source, except the explicitly labeled formatting edit before the assertion commit. The driver applies `CARGO_INCREMENTAL=0` and `CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target`, enforces the listed deadline and captures separate stdout, stderr and status JSON. All compilations used Cargo jobs four, and all commands ran in the foreground under the 8 GiB memory cap. The full raw outputs and exact command arrays are under `docs/superpowers/records/2026-10-18-phase-9d/t8.8/final-test-repair/checks/`.

| Record | Result | Elapsed seconds | Bound seconds |
| --- | --- | ---: | ---: |
| `pure-move-list` | Exit 0, 63 unchanged test names | 10.587 | 520 |
| `format-edit` | Exit 0, source formatting before commit | 2.570 | 120 |
| `dispatch-seam` | Exit 0, all six tests passed | 0.816 | 520 |
| `traceobject-and-negative` | Exit 0, both selected tests passed | 20.407 | 520 |
| `fmt` | Exit 0 | 2.519 | 120 |
| `clippy` | Exit 0, warnings denied | 1.067 | 520 |
| `final-list` | Exit 0, same 63 test names | 0.114 | 120 |

```text
memcap 8G python3 /tmp/t88-final-test-repair-check.py pure-move-list 520 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust cargo test -p rexx-exec -j4 --test concurrency_tests -- --list
memcap 8G python3 /tmp/t88-final-test-repair-check.py format-edit 120 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust cargo fmt --all
memcap 8G python3 /tmp/t88-final-test-repair-check.py dispatch-seam 520 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust cargo test -p rexx-exec -j4 --test dispatch_seam
memcap 8G python3 /tmp/t88-final-test-repair-check.py traceobject-and-negative 520 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust --env=REXX_CORPUS_GATE=1 cargo test -p rexx-exec -j4 --test concurrency_tests -- group_runs::the_outcome_table_of_the_trace_object_group group_runs::the_runner_passes_a_passing_group_and_reports_an_altered_test_as_differing --exact --nocapture --test-threads=1
memcap 8G python3 /tmp/t88-final-test-repair-check.py fmt 120 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust cargo fmt --all --check
memcap 8G python3 /tmp/t88-final-test-repair-check.py clippy 520 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust cargo clippy -p rexx-exec -j4 --test dispatch_seam --test concurrency_tests -- -D warnings
memcap 8G python3 /tmp/t88-final-test-repair-check.py final-list 120 /home/moritz/dev/repos/ooRexx-rust-rewrite/rust cargo test -p rexx-exec -j4 --test concurrency_tests -- --list
python3 /tmp/t88-final-test-repair-proof.py
```

The enabled TraceObject run printed ten `pass` rows, including all seven names absent from the stale three-pass expectation. The existing altered-test control still classified its deliberate change as differing, with oracle assertion count nine and crate assertion count five. Both selected tests passed, and the source list proves no Rust test identity was added, removed or renamed. No broad suite, corpus, workspace or performance run occurred in this slice, and no live process remains from these checks.

The optional `git diff --cached --check` returned status 2 solely for raw output whitespace. Its diagnostics identify trailing table-column tabs in `traceobject-and-negative.stderr` and final blank lines in two stdout records. These bytes are retained exactly as emitted rather than normalized to satisfy a source-format check. The separate cargo formatting check passed, and the raw whitespace diagnostics do not change any command verdict.

## Evidence handoff

The retained evidence directory is `docs/superpowers/records/2026-10-18-phase-9d/t8.8/final-test-repair/`. Its source reconstruction, population comparison, source hashes, immutable pin hash, raw check outputs and prior-failure hashes support the scoped review. `proof.py` records the exact source and unchanged-input assertions, while `check.py` preserves the bounded command driver used for this slice. Source and index ownership return to root after the evidence commit and final clean-status check, with the untouched ZIP remaining untracked.
