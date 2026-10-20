signal on halt
call time 'r'
address system 'echo a; sleep 1.5; echo b' with output using (.Sl~new(1))
say 'after' rc
exit
halt: say 'HALT handler rc' rc 'line' sigl 'fast' (time('e') < 1.2); exit 0
::class Sl subclass OutputStream
::attribute d
::method init
  use arg self~d
::method lineout
  use arg l
  if self~d < 3 & l~left(1) == 'a' then address system 'echo a; sleep 2; echo n' with output using (.Sl~new(self~d + 1))
  say 'O'self~d':'l 'rc' rc
  return 0
