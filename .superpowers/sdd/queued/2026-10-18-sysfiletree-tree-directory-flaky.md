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
