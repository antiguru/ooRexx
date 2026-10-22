call outer
say "back" result
exit
outer:
  signal on syntax
  say 1/0
  return
syntax:
  nop
  raise propagate return 5
