/* conversion inside a forward */
signal on syntax
s. = .o~new
say s.~foo
say 'end'
exit
syntax: say 'SYNTAX' condition('O')~code; exit
::class o
::method unknown
  x. = >x.; return length(x.)
::class m subclass stem
