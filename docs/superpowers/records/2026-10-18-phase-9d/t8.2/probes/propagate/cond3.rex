call outer
say 'main after outer'
exit
outer:
  signal on syntax
  say 1/0
  return
syntax:
  call s
  say 'no propagate from s'
  return
s:
  raise propagate
