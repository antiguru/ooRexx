o = .Bad~new('O', 'a', 'user')
signal on any name h
address system 'echo a; sleep 0.3; echo b; exit 0' with output using (o)
say 'not here' rc
exit
h: say 'signalled' condition('c') 'calls' o~calls 'rc' rc
exit
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
    if self~kind = 'failure' then raise failure 8
  end
  return 0
