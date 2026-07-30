/* A nested DO where LEAVE names the outer loop's control variable, to catch
   a control-flow target wired to the wrong index. LEAVE/ITERATE name a loop
   by its control variable, not by a label -- a label immediately before a
   DO is rejected with error 47.2 ("Labels are not allowed within a DO/LOOP
   block"), and a label elsewhere does not satisfy error 28.3's "must match
   the label of a current loop" either. Only the control variable works.

   A LEAVE wired to only the inner loop would print "after inner 1", then
   continue into i=2 and i=3 of the outer loop; the correct wiring exits both
   loops on the first hit and prints nothing between "inner 1 1" and
   "after outer". */
do outer = 1 to 3
  do inner = 1 to 3
    if inner = 2 then leave outer
    say "inner" outer inner
  end
  say "after inner" outer
end
say "after outer"
