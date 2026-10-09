/* Under TRACE ?C only a traced command pauses, and a TRACE instruction met
   in debug is ignored. */
trace ?c
say 'c1'
x = 1
say 'c2'
'true'
say 'c3'
l:
say 'c4'
trace ?l
say 'c5'
m:
say 'c6'
