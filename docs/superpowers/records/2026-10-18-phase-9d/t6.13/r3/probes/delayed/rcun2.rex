address system 'exit 7'
o = .Bad~new('O', 'a', 'error')
call on error name h
address system 'echo a; sleep 0.3; echo b; exit 5' with output using (o)
say 'after' rc .rs 'calls' o~calls
exit
h: say 'handler' condition('c') condition('d') 'rc' rc 'rs' .rs 'sigl' sigl
return
::class Bad subclass OutputStream
::attribute calls
::attribute tag
::attribute bad
::attribute kind
::method init
  use arg self~tag, self~bad, self~kind = 'zero'
  self~calls = 0
::method lineout
  use arg l
  self~calls += 1
  say self~tag':'l
  if wordpos(l, self~bad) > 0 then do
    if self~kind = 'zero' then return 1 / 0
    if self~kind = 'conv' then return 'x' + 1
    if self~kind = 'user' then raise user boom
    if self~kind = 'error' then raise error 9
    if self~kind = 'nr' then raise notready
  end
  return 0
