call t 'drop'
call t 'pe'
return
t: procedure
  arg which
  signal on novalue name n1
  if which = 'DROP' then drop (nolist)
  else call pe
  say which': not raised'
  return
n1: say which':' condition('c') '['condition('d')']' sigl
  return
pe: procedure expose (nolist2)
return
