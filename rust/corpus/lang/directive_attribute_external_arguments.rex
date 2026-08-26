/* A bound setter runs the entry point rather than storing the value, so the
   assignment hands one argument to a procedure that declares none, and the
   report carries the Compiled method line that entry point's own activation
   contributes. */
say 'main ran'
.k~at = 5

::class k

::attribute at set class external 'LIBRARY REXX file_separator'
