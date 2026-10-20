signal on halt
address system 'echo a; sleep 1.5; echo LATE; exit 3' with output stem s.
say 'not here'
exit
halt:
  say 'HALT rc' rc 'stem' s.0 s.1
  address system 'sleep 2; echo x; exit 5' with output using (.Lg~new)
  say 'second rc' rc
  exit 0
::class Lg subclass OutputStream
::method lineout
  use arg l
  say 'L:'l
  return 0
