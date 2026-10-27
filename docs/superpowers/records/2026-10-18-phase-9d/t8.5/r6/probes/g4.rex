/* TRACE I inside a forward */
signal on syntax
s. = .o~new
say s.~foo
say 'end'
exit
syntax: say 'SYNTAX' condition('O')~code; exit
::class o
::method unknown
  trace i; x. = >x.; return x.~bar
::class m subclass stem
