**CLOSED 2026-10-22 by Phase 9d Task 8.2 (9f8be0ec2).**
**RULED: 9d, ruled by Moritz 2026-10-19 ("proposal ok").** derive the semantics; follow the model; Deviation where the oracle departs.

**RULED by Moritz 2026-10-21, all three:**
* A failed trap handler call taken by SIGNAL ON SYNTAX signals at once; no clause runs after it and
  SIGL names the failing clause; Deviation ("Signal at once, Deviation").
* A non-SYNTAX `RAISE ... RETURN|EXIT expr` sets the condition object's RESULT entry, as rexxref
  says; Deviation recorded as an oracle bug (`RexxActivation::raise` passes `result`, not
  `resultObj`, RexxActivation.cpp:1827); revert if ooTest breaks ("Add RESULT, oracle bug"). A crate
  change.
* A 16.1 from a SIGNAL trap inside INTERPRET keeps the PROGRAM frame's file name; Deviation
  ("Keep file name, Deviation").

**Found by the Phase 9d Task 6.9 review.** Both predate Task 6.9.

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

The Task 6.9 re-review (`task-6.9-rereview.md`, Minor 1) adds a third. A 16.1 raised from a
SIGNAL trap inside INTERPRET reports one stack frame, labelled PROGRAM, with an empty name on the
oracle: `signal on syntax; interpret "signal on novalue name gone_x; y = qq"` gives
`sf PROGRAM 2 []`, where the crate gives `sf PROGRAM 2 [i2.rex]`. A NOTREADY from `linein` in the
fragment, and `signal on notready name gone_x` set before the INTERPRET, behave the same. For an
ordinary failure, `interpret "x = 1/0"`, both sides give `sf INTERPRET 2 []` and
`sf PROGRAM 2 [i1.rex]`. The oracle's frame takes the INTERPRET activation's name under the
PROGRAM label.
