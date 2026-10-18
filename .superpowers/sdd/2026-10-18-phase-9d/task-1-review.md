# Review: Phase 9d Task 1

Spec compliance: PASS. Task quality: PASS with minor findings.

## Checks run
* Re-derivation: the header's awk over the 9c close tests.tsv, diffed against the non-comment rows of `rust/corpus/suite-9d-set.txt`: identical (178 rows).
* Coverage: sorted `group test` of `clusters.tsv` against the set: no difference, no duplicates.
* Reproduced on both sides from fresh dirs (release build of the crate at HEAD): rexxinteger-min-representation (oracle returns an unrounded receiver, crate rounds under `numeric digits 1`), bag-hasitem-nil, stream-binary-records (oracle error 93 on a long record, crate accepts), lostdigits-not-raised, label-in-if-select-branch, identity-collection-equality, stream-open-write-only, datatype-whole-wide-numbers, collection-argument-error-numbers (93.923 vs 93.906), array-array-index, queue-removeitem (hole), stream-eof-state, variable-reference-list (drop of an unset list variable under NOVALUE SYNTAX), dynamic-package-name (`.context~name` CALL vs named).
* scclient/sfclient claim: probe `command-redirected` reproduces (crate: main resumes after the command; oracle: before it).
* requires-from-dynamic-code: the probe with pre-existing `y_st.cls` reproduces (oracle `static`, crate error 43.901).
* yaml note: TEST35 and TEST47 are blocked with Phase10(Extension) rows in tests.tsv.
* Constraints: largest record file is 25 KB, no prose states a set size without a command (the "178" appears nowhere in prose).

## Findings
* Minor: the yaml concern (two blocked tests excluded by the set filter) is only in the report, not in `clusters.md`, which Task 4 reads.
* Minor: call-on-trap-argument not independently reproduced (my probe did not reach the handler); relied on the recorded failure text.
* Minor: `stream-command-position` and `collection-set-operations` are admitted unprobed; the former may split.
* Minor: collection-set-operations: `Queue~difference` printed equal contents on both sides in my probe, and `equals` is not understood on either, so the cause rests on the test's assertEquals only.
