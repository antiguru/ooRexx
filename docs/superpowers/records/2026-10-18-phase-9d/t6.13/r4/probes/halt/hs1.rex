signal on halt
address system 'echo a; sleep 1.5; echo b' with output using (.Sl~new)
say 'after' rc
exit
halt: say 'HALT handler rc' rc 'line' sigl; exit 0
::class Sl subclass OutputStream
::method lineout
  use arg l
  if l == 'a' then call SysSleep 2
  say 'O:'l
  return 0
