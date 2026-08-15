/* Where a plain DO offers a queued condition its boundary, on stdout.

   A CALL ON handler that ends in `raise ... return` leaves a second trapped
   condition pending, and the next clause boundary delivers it. That makes the
   printed order and the printed SIGL a direct reading of which clauses a
   construct ends and when -- with no TRACE anywhere, so nothing here depends
   on the trace stream or on DEVIATION 0's indent normalisation.

   The oracle's rule, which every block below reads back: what a handler
   queues WHILE IT RUNS is owed to the next clause boundary and not to the one
   that delivered it, and a plain DO is TWO clauses -- the header, which
   RexxInstructionSimpleDo::execute ends as soon as it has opened the block,
   and END, which is an instruction of its own. The rule's other half -- a
   boundary takes everything the clause left pending, not one -- is
   condition_queue_drain.rex's subject rather than this file's.

   Blocks A, C and K are the header: the condition pending when the DO runs is
   owed to the DO's own line, ahead of anything in the body. Blocks D and E are
   END: a condition requeued inside the block is owed to the END's line, ahead
   of whatever follows the block. B is the control that pins the pair apart --
   an empty body agrees under a single requeue whichever way the header clause
   is placed, and diverges only once a second requeue reaches for END.

   F, G, H, I and J are the adjacent successes. F is the one shape that runs
   the other way: a DO UNTIL whose condition raises has already had its END
   boundary served by the UNTIL test, so the requeued condition is owed to the
   clause AFTER the loop, and an engine that keeps a boundary of its own around
   the whole construct delivers it early instead. G and H are loops whose
   header is a clause the crate already ends before the body; I and J are the
   neighbouring constructs, which this file's own subject must not move. */

call on user c1 name h1
call on user c2 name h2
call on user c3 name h3

/* A: a plain DO with a body. The requeued handler is owed the DO's own line
   and runs ahead of the body, not after it. */
zq = 0
zt = 'A'
zv = 'unset'
zr = raiser()
do
  say zt 'body'
end
say zt 'after' zv zr

/* B: the same with an empty body -- the control. One requeue is delivered at
   the DO's line here whether the header clause ends before the body or spans
   it, because there is no body clause to reach first. */
zq = 0
zt = 'B'
zv = 'unset'
zr = raiser()
do
end
say zt 'after' zv zr

/* C: nested plain DOs, and nothing else. The delivery is owed to the OUTER
   DO's line; the inner DO's line is what an engine answers when the outer
   header clause is still open when the inner one begins. */
zq = 0
zt = 'C'
zv = 'unset'
zr = raiser()
do
  do
  end
end
say zt 'after' zv zr

/* D: two requeues over an empty block, which is what separates the header
   boundary from END's. The first is owed to the DO's line and the second to
   the END's, so both land before the SAY that follows the block. */
zq = 1
zt = 'D'
zv = 'unset'
zr = raiser()
do
end
say zt 'after' zv zr

/* E: the same pair, queued from inside the block instead. The body clause's
   own boundary takes the first, END's takes the second, and only the third is
   left for the clause after the block. */
zq = 1
zt = 'E'
zv = 'unset'
do
  zr = raiser()
end
say zt 'after' zv zr

/* F: a DO UNTIL whose condition raises. The UNTIL test is END's own clause and
   delivers the first; the requeue is then owed to the clause after the loop,
   so the SAY prints before the handler rather than after it. */
zq = 0
zt = 'F'
zv = 'unset'
do until raiser() > 0
  nop
end
say zt 'after' zv

/* G: a controlled loop, whose header this crate already ends before the body.
   The delivery is owed to the DO's line, exactly as in A. */
zq = 0
zt = 'G'
zv = 'unset'
zr = raiser()
do zi = 1 to 2
  say zt 'body' zi
end
say zt 'after' zv zr

/* H: DO WHILE, whose test is a clause at the DO's line on the first pass. */
zq = 0
zt = 'H'
zv = 'unset'
zn = 0
zr = raiser()
do while zn < 2
  zn = zn + 1
end
say zt 'after' zv zr

/* I: IF ... THEN with no block. The neighbouring construct, and one this
   file's subject must leave where it is. */
zq = 0
zt = 'I'
zv = 'unset'
if 1 = 1 then zr = raiser()
say zt 'after' zv zr

/* J: SELECT with a WHEN whose THEN raises. The other neighbour. */
zq = 0
zt = 'J'
zv = 'unset'
select
  when 1 = 1 then zr = raiser()
end
say zt 'after' zv zr

/* K: a labelled plain DO. The label makes the block leavable by name and
   changes nothing about where its two clauses end. */
zq = 0
zt = 'K'
zv = 'unset'
zr = raiser()
do label zl
  say zt 'body'
end
say zt 'after' zv zr

exit 0

raiser:
raise user c1 return 5

h1:
zv = 'set'
raise user c2 return 1

h2:
say zt 'trap2' sigl
if zq = 1 then raise user c3 return 1
return

h3:
say zt 'trap3' sigl
return
