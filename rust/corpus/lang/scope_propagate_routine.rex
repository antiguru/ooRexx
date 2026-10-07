/* A routine called from a handler starts with no condition to propagate. */
signal on syntax
say 1/0
exit
syntax:
  call r
  say 'back'
  exit
::routine r
  signal on syntax name inner
  raise propagate
  say 'no raise'
  return
inner:
  say 'routine trapped' condition('O')~code
  return
