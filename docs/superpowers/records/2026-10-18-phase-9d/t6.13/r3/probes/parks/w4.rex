o = .Slow~new
address system 'echo a; sleep 0.3; echo b' with output using (o)
say 'done' rc 'calls' o~calls
exit
::class Slow subclass OutputStream
::attribute calls
::method init
  self~calls = 0
::method lineout
  use arg l
  self~calls += 1
  address system 'true'
  say 'O:'l
  return 0
