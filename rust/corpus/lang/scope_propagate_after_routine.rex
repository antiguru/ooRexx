/* A condition a routine handled is not main's to propagate. */
call r
signal on syntax
raise propagate
say 'main: no raise'
exit
syntax:
  say 'main trapped' condition('O')~code
  exit
::routine r
  signal on syntax name h
  say 1/0
h:
  say 'routine handled' condition('O')~code
  return
