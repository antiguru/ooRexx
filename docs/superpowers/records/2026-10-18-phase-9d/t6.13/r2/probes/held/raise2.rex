m = .T~new~start("tick")
call time 'r'
b = .Bad~new
signal on syntax name raised
address system 'echo a; sleep 0.6; echo b; exit 0' with output using (b)
say 'not here'
raised:
say 'trapped' condition('o')~code 'after child' (time('e') > 0.5) 'calls' b~calls 'rc' rc
call SysSleep 1
say 'later calls' b~calls
signal off syntax
address system 'echo c; echo d' with output stem o.
say 'next' rc o.0 o.1 o.2
exit
::class Bad subclass OutputStream
::attribute calls
::method init
  self~calls = 0
::method lineout
  use arg l
  self~calls += 1
  if self~calls = 1 then return 1 / 0
  say 'lineout' l
  return 0
::class T
::method tick
  do 4; call SysSleep 0.3; end
