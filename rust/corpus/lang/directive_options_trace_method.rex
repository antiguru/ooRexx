/* ::OPTIONS TRACE reaches a ::METHOD as well as a ::ROUTINE, and a routine
   that turns its own trace off announces its >I> and then owes no <I<. */
o = .k~new
o~m
call sub

::options trace a

::class k
::method m
y = 2
return 3

::routine sub
trace off
z = 4
return
