/* reply then forward on the new activity */
signal on syntax
s. = .o~new
say s.~foo
say 'end'
exit
syntax: say 'SYNTAX' condition('O')~code; exit
::class o
::method unknown
  reply 'replied'; x. = >x.; say 'after' x.~bar
::class m subclass stem
