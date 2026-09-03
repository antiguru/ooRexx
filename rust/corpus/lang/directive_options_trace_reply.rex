/* A method that REPLYs under a package trace setting announces its >I>/<I<
   pair twice: once around the clauses before the REPLY and once around the
   ones after it. */
o = .k~new
say o~m
say 'main done'

::options trace r

::class k
::method m
reply 7
z = 1
return
