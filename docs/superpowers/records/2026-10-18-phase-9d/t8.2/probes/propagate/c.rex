call outer
exit
outer:
  signal on syntax
  say 1/0
  return
syntax:
  call s
  return
s:
  call t
  return
t:
  raise propagate
