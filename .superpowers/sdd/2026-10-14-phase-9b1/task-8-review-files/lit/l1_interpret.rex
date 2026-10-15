say f()
signal on syntax
interpret 'raise syntax 40.1 additional("abcdefghijklmnopqrstuvwxyz0123456789") description "the long description literal here"'
say 'not reached'
syntax:
c = condition('O')
call filler
say c~code c~description c~additional[1]
say g()
o = .k~new
say o~m
say .k~new~cv
exit
f:
  interpret 'return "a returned literal that is long enough"'
g:
  interpret "call q 'an argument literal that is long enough', 'second long literal argument'"
  return result
q:
  call filler
  return arg(1) '|' arg(2)
filler: procedure
  do i = 1 to 300; z = copies('x', 100000); end
  return
::class k
::constant cv "a constant literal value long enough"
::method m
  reply 'a replied literal value long enough'
  call filler
  interpret 'self~setx("a set literal long enough")'
::method setx
  expose x
  use arg x
