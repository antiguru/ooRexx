i = 2; a.2 = 'p q'; p = 'cp'; q = 'cq'; a.I = 'r'; r = 'cr'
call pr1
call pr2
call pr3
call pr4
call pr5
k = 'I'
call pr6
exit
pr1: procedure expose (a.i)
  say 'pr1' symbol('p') symbol('r') symbol('a.2') symbol('a.I')
  return
pr2: procedure expose (a.i) i
  say 'pr2' symbol('p') symbol('r') symbol('a.2') i
  return
pr3: procedure expose (a.2)
  say 'pr3' symbol('p') symbol('a.2')
  return
pr4: procedure expose a. (a.i)
  say 'pr4' symbol('p') symbol('r')
  return
pr5: procedure expose (a.i.i)
  say 'pr5' symbol('a.2.2') symbol('a.I.I')
  return
pr6: procedure expose (a.k)
  say 'pr6' symbol('p') symbol('r') symbol('k')
  return
