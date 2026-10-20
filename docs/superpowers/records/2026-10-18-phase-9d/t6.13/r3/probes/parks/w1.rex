o = .Slow~new
address system 'echo a' with output using (o)
say 'done' rc 'calls' o~calls
exit
::class Slow subclass OutputStream
::attribute calls
::method init
  self~calls = 0
::method lineout
  use arg l
  self~calls += 1
  call SysSleep 0.1
  say 'O:'l
  return 0
