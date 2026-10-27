/* a cycle created by an UNKNOWN method while a stem forwards to it: Stem subclass, own default */
signal on syntax
s. = .o~new
say s.~foo
say 'end'
exit
syntax: say 'SYNTAX' condition('O')~code; exit
::class o
::method unknown
  t = .m~new
  t[] = t
  return t~bar
::class m subclass stem
