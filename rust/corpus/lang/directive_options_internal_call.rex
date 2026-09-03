/* An internal CALL is still inside the activation that started from the
   package, so a label's own clauses escalate exactly as the main body's do
   -- where a ::ROUTINE and a ::METHOD start from the package afresh, a label
   inherits. */
signal on syntax
call lab
say 'unreached'
exit

lab:
say zzzunset
return

syntax:
say 'trapped' rc

::options novalue syntax
