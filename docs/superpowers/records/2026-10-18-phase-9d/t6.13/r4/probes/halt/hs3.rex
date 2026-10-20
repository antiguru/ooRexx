call on halt
call time 'r'
address system 'echo a; sleep 1.5; echo b' with output using (.Sl~new)
say 'after' rc 'fast' (time('e') < 1.2)
exit
halt: say 'HALT handler rc' rc 'line' sigl; return
::class Sl subclass OutputStream
::method lineout
  use arg l
  if l == 'a' then address system 'sleep 2; echo n' with output stem q.
  say 'O:'l 'inner rc' rc
  return 0
