/* SIGNAL OFF turns the escalation off as well, with no SIGNAL ON before it:
   both reads answer their rendering where the same clauses without these two
   would be 98.986 and 98.973. */
signal off novalue
say 'novalue' zzzunset1
signal off nostring
say 'nostring' .array
say 'done'

::options all syntax
