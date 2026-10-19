/* A variable list's (name) selector in DROP, PROCEDURE EXPOSE and EXPOSE is
   read as an expression reads it: TRACE I shows its >V> line, after >C> for
   a compound, and a stem or compound selector spells the list from its
   value. PROCEDURE EXPOSE reads its selector in the procedure's own frame,
   after exposing it, so a tail variable not yet exposed is unset there. */
list = 'a b'
a = 1; b = 2
drop (list)
say symbol('a') symbol('b')
list = 'c'; c = 3
call pe
lst. = 'x y'; x = 1; y = 2
drop (lst.)
say symbol('x') symbol('y') symbol('lst.')
q.1 = 'z'; z = 5
drop (q.1)
say symbol('z') symbol('q.1')
i = 2; a.2 = 'p q'; p = 'cp'; a.I = 'r'; r = 'cr'
call pa
call pb
call pc
.k~new~m
exit

pa: procedure expose (a.i)
  say symbol('p') symbol('r') symbol('a.2') symbol('a.I')
  return

pb: procedure expose (a.i) i
  say symbol('p') symbol('r') symbol('a.2') i
  return

pc: procedure expose (a.2)
  say symbol('p') symbol('a.2')
  return

pe: procedure expose (list)
  say c
  return

::class k
::method m
  expose zz
  zz = 'w'
  self~m2
  say zz
::method m2
  expose zz (zz)
  w = 'object w'
::options trace i
