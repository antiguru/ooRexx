/* After USE LOCAL a name first met at run time binds to the object too:
   through VALUE, INTERPRET, a stem's tail, and an internal call without
   PROCEDURE; SELF, RESULT and a PROCEDURE's own names stay local. */
o = .t~new
say o~m
say o~peek
::class t
::method m
use local a
call value 'DYN', 'd'
interpret 'dyn2 = "d2"'
s.1 = 'one'
a = 1
call sub
call isolated
result = 'r'
return a self~class~id result
sub: q = 'q'; return
isolated: procedure; z = 'z'; return
::method peek
expose dyn dyn2 a s. q z result
return dyn dyn2 a s.1 q z result
