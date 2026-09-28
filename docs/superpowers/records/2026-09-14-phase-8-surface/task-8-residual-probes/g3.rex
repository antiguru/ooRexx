call r
say 'no'
::routine r
  signal on syntax name h
  interpret 'x = 1 + "a"'
h: interpret 'raise propagate'
