/* ::OPTIONS NUMERIC INHERIT: a ::ROUTINE and a ::METHOD start from the
   settings in force where they were called, not from the package's own. */
numeric digits 20
numeric fuzz 7
numeric form engineering
call sub
o = .k~new
o~m

::options digits 12 numeric inherit

::routine sub
say 'routine' digits() fuzz() form()
return

::class k
::method m
say 'method' digits() fuzz() form()
