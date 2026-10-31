# Scoped SysFileTree isolation review

The reviewed range is `4714f6521f54e583e86c579bd803a80067a6939d..ac446500e03c18b5143f2967ca30015e6d64b95f`, against the task-local isolation plan. The change addresses interference between independently enumerated snapshots of the shared temporary directory during the original SysFileTree test. This review inspected the complete five-file Rust/test diff, the pure-move and regression commits, the retained raw evidence, and preservation hashes. It performed no builds, test reruns, source changes, index changes, or subprocess experiments.

## Source and causal boundary

`rust/crates/rexx-exec/tests/suite/launch/mod.rs:17` selects only `base/rexxutil/SysFileTree`, and line 22 seeds its owned `run/sysfiletree-tmp` with the ordinary seven-byte file `stable-file.txt` and an empty `stable-directory`. The oracle prepares the directory after copying the group at line 56 and overrides only its child command environment at line 76. The crate prepares it inside `crate_run_once` at line 174 and applies the override after `set_environment` at line 214, so the environment clear cannot erase it. Other logical groups return `None` and retain their existing launch environment.

The existing retry loop at `launch/mod.rs:129` enters `crate_run_once` for each attempt, and `suite.rs:416` calls `fresh_copy_from`, whose `support/group_runner.rs:294` implementation removes the old run before copying. Consequently each attempt receives fresh seeds without sharing either interpreter's directory. Both launch routes capture diagnostics before `remove_work`, and `suite/diagnostics.rs:24` records the selected path and seed names in the external diagnostic bundle. The parent process environment is never mutated, and the synthetic fixture's observation file is outside both enumerated child directories and their removed work trees.

## Evidence checked

* The regression at `suite/fixtures/tests.rs:216` invokes both actual launch routes before checking results. At regression commit `0333f95b1`, both retained raw outputs fail `TEST_TEMPORARY_DIRECTORY` at fixture line 38 because the expected private path instead resolves to `/tmp`, with zero framework errors. Oracle stdout SHA256 is `e909eba18087444120be57b42712156e2f15853acb7148028388848edfa734d2`; crate stdout is `21b86f16124c29dc677708611b0051c21ff38a664dfe45cca101541c5ea1fc27`.
* At implementation commit `6f2c1346e`, both fixture runs pass 13 assertions. The fixture checks exact names, full returned paths, file/directory types, file bytes and size, and an empty child directory. The Rust regression checks unchanged parent TMPDIR, distinct child paths, successful cleanup, and preservation of its external observation; the second test at `suite/fixtures/tests.rs:276` checks exact selection and three negative cases.
* The unchanged original group passes 26/26 tests and 2014 assertions on each interpreter, once per side, with no failures or blocked tests. Its raw outputs are under `original-group/diagnostics`; oracle stdout SHA256 is `f5cb3ff66a7a83dab6a58e025e0e4c9491bc964078ed982f105d813eeeeaa160`, and crate stdout is `2802881211dfbfd9f981628eab1ab9f253c3c2be51db2d29f0ea5909e0f702ec`.
* The complete extracted launch block, including comments, is byte-identical between BASE and pure-move commit `0d8d1b29c`. Public functions remain re-exported from `suite.rs`, and the retained test listings remain 68 across that move, then add exactly the two named tests to reach 70, with no removals. Retained fixture output shows 21 passed and one pre-existing ignored diagnostic test, including existing retry fixtures; fmt and suite-target clippy statuses are zero.
* Independently recomputed hashes match all 70 manifest entries, all five current source hashes, and all 553 protected ooTest file hashes. The changed Rust paths are exactly the five planned test/harness files, leaving runtime, Cargo, corpus, and licence inputs unchanged. The immutable runtime pin still hashes to `d9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145`.

The evidence directory is `docs/superpowers/records/2026-10-18-phase-9d/t8.8/sysfiletree-isolation`. Its `final-proof.json` SHA256 is `30d445ddc218a6d5376c9645e600da6e857eec9fe245a936f87bfa4ecafc7f13`. The initial green command's argument-placement error remains recorded as status 1 and is superseded by the correctly filtered successful command, rather than counted as a test pass. Git HEAD and the three source commits match the requested sequence, and tracked status was clean during review.

## Declined to judge

* The precise historical creator of the additional `/tmp` entry remains unproven because the failing run did not retain its first snapshot's full expected names; this repair prevents the demonstrated shared-directory mechanism without claiming historical attribution.
* Atomic snapshot behavior under mutation inside the group's own private directory is outside this harness repair; neither interpreter acquires that new semantic contract.
* Broad frozen gates, performance acceptance, and overall Task 8.8 readiness remain root-owned pending obligations; focused retained evidence does not settle them.

## Verdict

**ADDRESSED.** Exact-group child isolation removes the demonstrated cross-group mutation mechanism while retaining nonempty file/directory coverage and the protected assertions. No new high-confidence correctness or invariant finding was identified in this scoped diff. No further fix is requested by this review, and this is not a Task 8.8 completion or final readiness claim.
