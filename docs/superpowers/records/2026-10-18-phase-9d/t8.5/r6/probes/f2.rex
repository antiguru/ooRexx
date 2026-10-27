/* a cycle created by an UNKNOWN method while a stem forwards to it: s. = >s. inside */
signal on syntax
s. = .o~new
say s.~foo
say 'end'
exit
syntax: say 'SYNTAX' condition('O')~code; exit
::class o
::method unknown
  x. = >x.
  return x.~bar
