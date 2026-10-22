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
  signal on syntax name s2
  interpret "raise propagate"
  return
s2:
  say "s2" condition("O")~code
  raise propagate
