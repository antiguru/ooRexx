call p1; say 'p1' symbol('A.') symbol('A.1')
call p2; say 'p2' symbol('B.')
.local~novalue = .nv~new
call p3; say 'p3' symbol('C.') symbol('C.1') symbol('VC.1')
call p4; say 'p4' symbol('D.') symbol('VD.1')
exit
p1: procedure expose a. (a.1)
  say 'in p1' symbol('A.') symbol('A.1')
  return
p2: procedure expose (b.1)
  say 'in p2' symbol('B.') symbol('B.1')
  return
p3: procedure expose c. (c.1)
  say 'in p3' symbol('C.') symbol('C.1')
  return
p4: procedure expose (d.1)
  say 'in p4' symbol('D.')
  return
::class nv
::method novalue; use arg n; say '  hook' n; return 'V'n
