/* A USER condition raised by a line typed at a pause is not trapped. */
call on user ping name h
trace ?a
x = 1
say 'after' x
exit
h: say 'handler' condition('C') condition('D') sigl; return
