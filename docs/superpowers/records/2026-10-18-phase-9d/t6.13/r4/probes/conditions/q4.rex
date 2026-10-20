signal on halt
o = .H~new
address system 'echo a; sleep 0.5; echo b; exit 0' with output using (o)
say 'after' rc
exit
halt: say 'HALT handler rc' rc 'line' sigl 'calls' o~calls; exit 0
::class H subclass OutputStream
::attribute calls
::method init
  self~calls = 0
::method lineout
  use arg l
  self~calls += 1
  say 'O:'l
  if l == 'a' then raise halt
  return 0
