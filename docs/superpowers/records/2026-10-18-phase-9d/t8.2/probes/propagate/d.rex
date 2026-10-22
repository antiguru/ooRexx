call outer
exit
outer:
  signal on syntax
  call inner
  return
inner:
  say 1/0
  return
syntax:
  call s
  return
s:
  raise propagate
