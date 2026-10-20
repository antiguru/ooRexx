o = .Sl~new
address system 'echo a; sleep 0.2; echo b; sleep 0.2; echo c' with output using (o)
say 'done' rc
exit
::class Sl subclass OutputStream
::method lineout
  use arg l
  call time 'r'
  if l == 'a' then call SysSleep 1
  say 'O:'l 'slept>0.9' (time('e') > 0.9)
  return 0
