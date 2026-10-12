# Phase 9a Task 5 re-review (d2f89f559..cd171c821)

Read-only; no suite rerun. Witness run once on the oracle from /tmp/claude-1000/p9/rr5-8Xxr (rc 0, stderr empty).

* I1 addressed. supplier.rs now calls `array_argument_object` on the original value (object_protocol.rs:1680ff), which converts, roots and answers the array; the `a KN` name is reported (oracle stdout shows `object "a KN"` on both Supplier lines). `request_array` has no remaining caller and is deleted (array.rs).
* I2 addressed. lang/array_argument_callers.rex is in phase-9.txt:14. Oracle stdout read: per value seven lines, in caller order sendWith 88.913 "message arguments", startWith 98.913, run 88.913 "argument array" (reached through the receiver's own method), Message~new 88.913 "message arguments", callWith 98.913, Supplier~new x2 98.913; the 98.913 text names `an Object`, `a KN`, `an Array` (2-D) as it should; the MAKEARRAY-array and string values give 0/1/7 and ab results. Every claimed caller's conversion is reached. unknown_argument_list.rex header narrowed. Crate-side equality is the implementer's claim, not rerun.
* I3 addressed. refusal-dispositions.tsv:66 and refusal-sites.tsv unknown_receiver no longer say a Method copy refuses at COPY.
* Minor REXX_WHOLE_GROUPS_ONLY: addressed (concurrency_tests.rs asserts every named group is in the derived list).
* Minor interpreter_bin copy fallback: addressed (remove error other than NotFound panics; asserts absent before fs::copy).
* Minor check_path (dev,ino): addressed (inode equal to rexx-run skips the read; only other files are byte-compared; ours is read per non-matching ELF file, which is rare).
* Minor frame_executable doc: addressed (context.rs names native level, failure-site frame, failed executable_at).
* Six unblocked groups: addressed. Report's table is consistent with the baseline claims (RexxInfo and SourceFile green, four red with no new failing test); not rerun by me. VariableReference~copy still refuses (VarRef TEST_COPY), outside Step 1 scope.
* Deferred (Method~copy side tables, startWith error header): not judged.

New defects: none Critical or Important. Minor: the 2-D array case has no success counterpart on `run`/callWith lines beyond the shared failure text; acceptable.
