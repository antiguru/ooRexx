/* ::OPTIONS NOVALUE SYNTAX: an untrapped read of an unset variable is 98.986
   in the main body, in a ::ROUTINE and in a ::METHOD alike, and SIGNAL ON
   SYNTAX is what catches it. */
signal on syntax
say zzzunset0
say 'unreached'

zzzmain:
call sub
o = .k~new
o~m
say 'end'
exit

syntax:
say 'main' rc
signal zzzmain

::options novalue syntax

::routine sub
signal on syntax
say zzzunset1
return
syntax:
say 'routine' rc
return

::class k
::method m
signal on syntax
say zzzunset2
return
syntax:
say 'method' rc
return
