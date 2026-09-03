/* CALL ON leaves NOVALUE, LOSTDIGITS and NOSTRING escalating: only the three
   conditions a CALL trap can resume from are turned off by it, so the read
   below is still 98.986 where the same clause under SIGNAL ON ANY answers
   the variable's own name. */
call on any name h
signal on syntax
say zzzunset
say 'unreached'
exit

h: return

syntax:
say 'trapped' rc

::options novalue syntax
