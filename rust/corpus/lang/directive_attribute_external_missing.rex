/* GET is prepended to the procedure rather than appended to the method name,
   and the getter is resolved before the setter, so the report names
   GETzzz_no_entry. The bind is eager, so the prologue does not print. */
say 'prolog ran'

::class k

::attribute at external 'LIBRARY REXX zzz_no_entry'
