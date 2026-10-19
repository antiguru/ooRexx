list = 'a b'
a = 1; b = 2
trace i
drop (list)
trace off
list = 'c'
c = 3
call pe
.k~new~m
lst. = 'x y'
x = 1
trace i
drop (lst.)
trace off
say symbol('x') symbol('lst.')
q.1 = 'y'
y = 5
trace i
drop (q.1)
trace off
say symbol('y') symbol('q.1')
return
pe: procedure expose (list)
trace i
nop
trace off
call pe2
return
pe2: trace i
call pe3
return
pe3: procedure expose (list)
say c
return
::class k
::method m
expose zz
zz = 'w'
self~setMethod("m2", .Method~new("", "expose zz (zz) v; ::options trace i"), "OBJECT")
self~m2
say zz w

