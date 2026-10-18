-- A Stem subclass whose INIT does not forward to its superclass.
signal on syntax name trapped
o = .zz~new
say 'items' o~items
o['I'] = 'v'
say 'put' o~items o~hasIndex('I')
exit 0
trapped: say 'syntax' condition('O')~code
exit 0
::class zz subclass Stem
::method init
  nop
