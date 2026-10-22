call sub 1
r = .routine~new('FOO', 'return "added"')
.context~package~addRoutine('FOO', r)
call sub 2
exit
sub: procedure
  use arg i
  signal on syntax name s
  say 'pass' i foo()
  return
s: say 'pass' i 'raised' condition('O')~code
  return
::routine foo
  return 1/0
