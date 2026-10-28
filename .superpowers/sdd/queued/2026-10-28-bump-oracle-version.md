**OPEN, queued 2026-10-28 by the controller at Moritz's request ("Queue a TODO to bump the oracle version. Several of the bugs have been fixed."); proposed: after Phase 9d closes, before Phase 10's gates.**

# Bump the oracle to current upstream trunk

The oracle binary (/home/moritz/dev/repos/ooRexx/build/bin/rexx, "Open Object Rexx Version 5.3.0 r0, Build
date: Jul 30 2026"; build/bin and build/lib from Jul 30 to Aug 5) predates upstream fixes Josep reported on
2026-10-22 against trunk r13285: #2082 r13250, #2083 r13268, #2085 r13264, #2093 r13251, #2068
r13284/r13285, #2098 r13260, #2087 r13262, #2117 r13272/r13273, SF #2069-#2078 r13179-r13195, the
cross-thread yield r13188 (defects.json entries marked FIXED upstream). The checkout's master is at
2e9a8470c (2026-10-21) but was not rebuilt, and has an uncommitted change in
interpreter/classes/NumberStringClass.cpp: ask Moritz before touching that tree; build the new oracle in a
separate worktree/build directory rather than over the current one.

Steps:
1. Build the current upstream trunk in its own directory; record its revision (`rexx -v`).
2. Re-run every defects.json entry marked FIXED upstream and every oracle-bug Deviation's probe on it;
   record which answers changed. A Deviation whose oracle bug is fixed upstream keeps its rule; its text
   names the fixed revision.
3. Re-derive the oracle-derived records: rust/corpus/errors/parse-errors.tsv (derive.sh), the suite tables'
   oracle-differences, corpus/sim-oracle/ and sim-gate.tsv outcomes, rust/corpus/oracle-crashes.txt (drop
   entries the new build no longer crashes on, after one capped run each).
4. Switch the standard oracle wrapper path in the dispatch rules and global-constraints.md, and the memory
   notes that name it.
