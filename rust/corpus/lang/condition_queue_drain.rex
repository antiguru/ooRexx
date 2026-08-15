/* What a clause boundary owes when more than one condition is waiting.

   A boundary takes everything the clause left pending, in the order it was
   queued -- not one and not the last. Every block below prints the handler
   names in order and each handler's SIGL, so the transcript reads back both
   the count and the order directly, on stdout, with no TRACE.

   The rule has a second half: what a handler queues WHILE IT RUNS is owed to
   the next boundary, never the one that delivered it. Blocks C and D are what
   separate the two halves, because an engine that simply drained until the
   queue emptied would run those handlers one clause too early.

   NOT HERE, and it is not an oversight: a clause that queues the SAME
   condition name twice. The oracle runs the first handler and then exits 139,
   so there is no behaviour to compare against and no correct answer for this
   file to assert. `corpus/oracle-crashes.txt` carries the program and the
   warning; a probe that widens any block below to a repeated name reintroduces
   it. */

call on user c1 name h1
call on user c2 name h2
call on user c3 name h3

/* Set before any block runs, and set explicitly in every one of them. An
   uninitialised REXX variable is its own name, and `zq > 0` on the string
   `ZQ` compares as text and is TRUE -- which makes every handler below
   requeue, and drives the oracle into the repeated-name shape that crashes it.
   Measured the hard way. */
zq = 0

/* A: one condition, the shape everything else is measured against. */
zt = 'A'
zr = ra()
say zt 'after' zr

/* B: two conditions queued by one clause, and then three. Both handlers are
   owed to that clause's own boundary and run in the order the conditions were
   raised, each reporting the raising clause's line rather than its own. */
zt = 'B'
zr = ra() + rb()
say zt 'after' zr
zt = 'C'
zr = ra() + rb() + rc()
say zt 'after' zr

/* D: a handler that requeues. The requeue is owed to the NEXT boundary, so
   `h2` runs at the `say`'s line and after nothing else -- an engine that kept
   draining would run it at the assignment's line instead. */
zt = 'D'
zq = 1
zr = ra()
say zt 'after' zr
zq = 0

/* E: both halves at once, over two passes of a loop. Pass one queues at the
   body clause and its handler requeues, so the requeue waits for the END; the
   END's handler requeues again, and pass two's body clause then owes TWO
   handlers -- the one left over and the one it queued itself -- both at its
   own line. That pairing is what an engine cannot produce by draining alone
   or by deferring alone. */
zt = 'E'
zq = 2
do zi = 1 to 2
  zr = ra()
end
say zt 'after' zr
zq = 0

exit 0

ra:
raise user c1 return 1
rb:
raise user c2 return 2
rc:
raise user c3 return 4

h1:
say zt 'h1' sigl
if zq > 0 then raise user c2 return 1
return

h2:
say zt 'h2' sigl
if zq > 1 then raise user c3 return 1
return

h3:
say zt 'h3' sigl
return
