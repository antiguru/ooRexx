o = .Slow~new
address system 'echo a; sleep 0.2; echo b; sleep 1.5; echo c' with output using (o)
say 'done' rc 'calls' o~calls
exit
::class Slow subclass OutputStream
::attribute calls
::method init
  self~calls = 0
::method lineout
  use arg l
  self~calls += 1
  if l == 'a' then call SysSleep 0.6
  say 'O:'l
  return 0
