call outer
exit
::routine outer
  signal on syntax
  say 1/0
  return
syntax:
  call s
  return
s:
  raise propagate
