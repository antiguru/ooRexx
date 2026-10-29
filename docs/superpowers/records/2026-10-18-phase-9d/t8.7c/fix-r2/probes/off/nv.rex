/* A NOVALUE raised by a line typed at a pause is not trapped. */
signal on novalue name h
trace ?a
x = 1
say 'after' x
exit
h: say 'handler' condition('C') condition('D') sigl; exit
