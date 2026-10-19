**OPEN: Deviation candidates for Moritz, found by the Phase 9d Task 6.9 review.** Both predate Task 6.9.

# Conditions: two oracle behaviours to rule on

Found 2026-10-19 by the Task 6.9 review (`task-6.9-review.md`, Minors 4 and 5).
* **A clause runs after a failed trap handler call.** When a trap handler call fails and
  `SIGNAL ON SYNTAX` takes the error, the oracle runs the next clause before the SIGNAL, and SIGL
  names that clause. With `signal on syntax name s; call on user u name queued; call rz`, where
  `rz` raises USER, then `say 'line4'` and `say 'line5'`, the oracle prints `line4` and
  `s 40.4 3 4`. The crate prints `s 40.4 3 3`. A missing handler label (`call on user u name say`,
  `call on notready name gone_call`) behaves the same. Untrapped, both sides report at line 2 and
  run nothing further. The crate does not copy this.
* **A non-SYNTAX `RAISE ... RETURN|EXIT expr` has no `RESULT` entry.** With
  `call on user u name mh; x = r()`, where `r` does
  `raise user u description 'd' return 'first'`, both sides print `mh hasResult 0 The NIL object`.
  rexxref (`condtra.xml`, RESULT) says the entry holds "Any result specified on the RETURN or EXIT
  options of a RAISE instruction". The oracle's `RexxActivation::raise`
  (`RexxActivation.cpp:1827`) passes the member `result`, not `resultObj`, to
  `createConditionObject`. The crate currently reproduces this. Following the documentation would
  add the entry and diverge from the oracle.
