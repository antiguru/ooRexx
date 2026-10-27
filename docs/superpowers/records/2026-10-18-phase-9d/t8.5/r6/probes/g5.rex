/* a reference first */
signal on syntax
s. = .o~new
say s.~foo
say 'end'
exit
syntax: say 'SYNTAX' condition('O')~code; exit
::class o
::method unknown
  x. = >x.; q = >x.; return q~bar
::class m subclass stem
