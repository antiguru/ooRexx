**RULED: 9d, ruled by Moritz 2026-10-19 ("proposal ok").** flaky test.

**OPEN: found by Phase 9d Task 6.2, not yet ruled.** `base/rexxutil/SysFileTree TEST_TREE_DIRECTORY` passes and fails between runs, on either side.

# `SysFileTree TEST_TREE_DIRECTORY` is intermittent on both sides

Found 2026-10-18 by Phase 9d Task 6.2 (`docs/superpowers/records/2026-10-18-phase-9d/t6.2/suite-sysfiletree-{1,2,3}/`).
In three reruns of the group, the test passed twice and then failed on the crate. Earlier records
show it failing on the oracle too. It has no row in `rust/corpus/suite-licences.txt` and is not in
`.github/known-test-failures/common.txt`. Without a row, it keeps showing up in every task's
whole-suite changed rows, and the Phase 9 close needs the suite to exit 0. Find the cause first:
timing, the directory layout the test builds, or harness state. Then either fix the harness or
take a `Flaky` row to Moritz.

## Task 8.8c bounded diagnosis

The original group and a private instrumented copy both passed every started test on the oracle and the crate, including TEST_TREE_DIRECTORY. Records under `docs/superpowers/records/2026-10-18-phase-9d/t8.8/timing/sysfiletree/` and `sysfiletree-trace/` preserve independent stdout/stderr, complete GroupRun state and run paths before cleanup. The private copy traces setup metadata, each assertion's option/path, expected file metadata, actual line, cwd, TZ and teardown, without modifying ootest. This remains open because no failed field or cause was reproduced, and no Flaky licence was added.

The bounded group's per-test results match the preceding `t8.7c/fix-r4/suite/tests.tsv` for its selected rows. The next owner is the controller running the pending full batch suite with REXX_SUITE_DIAGNOSTICS set, so the next failing assertion is retained instead of reduced to a verdict table. If it fails there, use the explicit private-copy diagnostic to compare the failing field and setup metadata before choosing an isolation or runtime fix. Passing reruns alone remain insufficient to close this item.
