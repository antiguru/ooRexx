signal on any name t
address system 'echo a; sleep 0.6; echo LATE' with output using (.Slow~new)
say 'no'
exit
t: say 'trapped' condition('c')
address system 'sleep 1.2; echo x' with output stem t.
say 'second rc' rc 't' t.0 t.1
exit
::class Slow subclass OutputStream
::method lineout
  use arg l
  call SysSleep 0.1
  say 'O:'l
  return 0
