parse arg mode
signal on halt
signal on syntax name raised
address system 'echo a; sleep 1.13; echo b' with output using (.Bad~new)
raised:
signal off syntax
say 'trapped' condition('c') condition('o')~code
address system 'echo x; sleep 2.13; echo y' with output stem s.
say 'after cmd rc' rc
exit
halt:
  say 'HALT rc' rc 'stem' s.0 s.1 'line' sigl
  exit 0
::class Bad subclass OutputStream
::method lineout
  return 1 / 0
