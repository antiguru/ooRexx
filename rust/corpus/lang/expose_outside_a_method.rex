/* EXPOSE outside a method invocation is 98.992, in both of the places a
   program can put one that the parser does not reject first: a top-level
   clause is the second half, and a ::ROUTINE's own first instruction is this
   one. Reached through a CALL, so the report carries both clauses. */
say 'before'
call helper
say 'unreached'

::routine helper
expose v
say 'unreached too'
