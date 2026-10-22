signal on syntax name top
call outer
exit
top:
  say "top" sigl condition("O")~position
  call t2
  exit
t2:
  raise propagate
outer:
  signal on syntax
  say 1/0
  return
syntax:
  call s
s:
  raise propagate
