# Reentrant Table removal count

An index equality method empties its Table while REMOVE is searching it. The oracle returns .nil and then ITEMS returns -1, with status 0 and empty stderr on both retained runs. The frozen e2 binary returns .nil and ITEMS 0, while the first single-probe f implementation incorrectly returns the detached old item and ITEMS 0. The local reference defines ITEMS as the number of items in the collection (rexxref/en-US/collclasses.xml:464-477), so an emptied Table has zero items.

The implementation correction preserves e2 by checking the receiver's store after observable equality and abandoning stale slot information. It retains one HASHCODE computation for the operation and restarts against the current bucket where necessary. Returning .nil agrees with the oracle, while the nonnegative count preserves the existing crate invariant. The oracle count defect is recorded separately from Deviations 82 and 83 as Deviation 84.

Moritz approved Deviation 84 on 2026-10-30 with the exact reply, "Yes, 0." The preceding question recommended keeping ITEMS at 0 and recording the oracle defect as Deviation 84. The numbered entry in `docs/superpowers/plans/phase-4-exclusions.txt` records that contract and this witness. This ruling authorizes the count difference in the retained reentrant removal case.
