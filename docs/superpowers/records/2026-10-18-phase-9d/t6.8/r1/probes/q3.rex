i = 2; a.2 = 'p q'; p = 'cp'
signal on syntax name e
call pr
say 'back'
exit
e: say 'E' condition('o')~code condition('o')~message
exit
pr: procedure expose (a.i)
  say 'pr' symbol('p')
  return
::options novalue syntax
