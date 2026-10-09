/* CALL ON and CALL OFF have no debug pause. */
trace ?a
call on error
call off error
call on halt name h
signal off error
say 1
exit
h: return
