call outer
exit
outer:
  signal on syntax
  say 1/0
  return
syntax:
  interpret 'raise propagate'
