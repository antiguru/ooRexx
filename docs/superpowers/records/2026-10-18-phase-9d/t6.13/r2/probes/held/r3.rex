call time 'r'
b = .Bad~new
e.0 = 9
signal on syntax name raised
address system 'echo a; echo e1 >&2; sleep 0.4; echo b; echo c; printf part' with output using (b) error stem e.
say 'not here'
raised:
say 'trapped' condition('o')~code 'calls' b~calls 'rc' rc 'err' e.0 e.1 '.rs' .rs
exit
::class Bad subclass OutputStream
::attribute calls
::method init
  self~calls = 0
::method lineout
  use arg l
  self~calls += 1
  if self~calls = 1 then return 1 / 0
  if l == 'c' then return 1 / 0
  say 'lineout' l
  return 0
