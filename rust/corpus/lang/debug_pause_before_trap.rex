/* The debug pause after a clause comes before a CALL ON handler the clause
   queued runs. */
call on user u name h
trace ?a
x = f()
say 'after' x
exit
f:
  raise user u return 7
h:
  say 'handler'
  return
