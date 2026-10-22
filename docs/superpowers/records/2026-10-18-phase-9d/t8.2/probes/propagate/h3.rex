call outer
exit
outer:
  signal on syntax
  say 1/0
  return
syntax:
  call s
  return 9
s:
  call t
  return
t:
  raise propagate return 5
