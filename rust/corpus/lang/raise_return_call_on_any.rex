/* A CALL ON ANY trap does not take the conditions CALL ON cannot trap when a
 * RAISE ... RETURN offers them to the caller; USER and NOTREADY it takes, and
 * an untrapped NOMETHOD is 97.5. */
call on any name h
call r1
say 'novalue' result
call r2
say 'lostdigits' result
call r3
say 'nostring' result
call r4
say 'user' result
call r5
say 'notready' result
call r6
say 'not reached'
exit
r1: raise novalue return 1
r2: raise lostdigits return 2
r3: raise nostring return 3
r4: raise user u1 return 4
r5: raise notready return 5
r6: raise nomethod return 6
h: say 'h' condition('C'); return
